"""Draft strategy abstraction for the quest simulator.

Defines the DraftStrategy protocol for card selection and the
SixSeatDraftStrategy implementation that wraps the current round
manager, show_n, resonance filter, and agent logic.
"""

import random
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any, Callable, Optional

import agents
import colors
import commitment
import draft_runner
import log_helpers
import pack_generator
import render
import resonance_filter
import show_n
from config import AgentsConfig, ScoringConfig, SimulatorConfig
from draft_models import CardInstance, Pack
from jsonl_log import SessionLogger

PICKS_PER_ROUND = 10


@dataclass
class PickResult:
    """Result of generating a draft pick for the player."""

    shown_cards: list[CardInstance]
    all_eligible: list[CardInstance]


class DraftStrategy(ABC):
    """Abstract protocol for draft card selection."""

    @abstractmethod
    def generate_pick(
        self,
        n: int,
        logger: Optional[SessionLogger] = None,
        context: str = "draft",
    ) -> PickResult:
        """Generate a pick, returning shown cards and the full eligible set."""

    @abstractmethod
    def complete_pick(self, chosen: CardInstance, shown: list[CardInstance]) -> None:
        """Signal that the player chose a card from the shown set."""

    @abstractmethod
    def skip_pick(self) -> None:
        """Advance one pick step without taking a card."""

    @abstractmethod
    def update_after_external_pick(self, card: CardInstance) -> None:
        """Update the human agent after a pick outside the draft loop."""

    def render_debug_panel(self) -> str:
        """Return a debug panel string. Empty by default."""
        return ""

    @property
    @abstractmethod
    def preference_vector(self) -> list[float]:
        """The human agent's archetype preference vector."""

    @property
    @abstractmethod
    def drafted_cards(self) -> list[CardInstance]:
        """Cards the human agent has drafted so far."""

    @property
    @abstractmethod
    def pick_index(self) -> int:
        """Global pick index across all rounds."""

    @property
    @abstractmethod
    def round_index(self) -> int:
        """Current round index."""

    @property
    @abstractmethod
    def round_pick_count(self) -> int:
        """Number of picks completed in the current round."""

    @property
    @abstractmethod
    def show_n_count(self) -> int:
        """Default N for draft sites."""

    @property
    @abstractmethod
    def show_n_strategy(self) -> str:
        """Default show-N strategy name."""

    @property
    @abstractmethod
    def scoring_cfg(self) -> Any:
        """Scoring configuration for show-N."""

    @property
    @abstractmethod
    def cube(self) -> Any:
        """The CubeManager for drawing cards outside the draft loop."""

    @property
    @abstractmethod
    def draft_cfg(self) -> Any:
        """The full draft configuration."""


class SixSeatDraftStrategy(DraftStrategy):
    """Draft strategy wrapping the 6-seat draft loop.

    Owns all draft-engine state: agents, cube, packs, config, and
    pick counters. Delegates AI picks and pack rotation to the same
    sub-functions used by the canonical draft loop.
    """

    def __init__(
        self,
        rng: random.Random,
        human_agent: agents.AgentState,
        ai_agents: list[agents.AgentState],
        cube: Any,
        draft_cfg: SimulatorConfig,
        resonance_pair_fn: Callable[[], Optional[tuple[str, str]]],
    ) -> None:
        self._rng = rng
        self._human_agent = human_agent
        self._ai_agents = ai_agents
        self._cube = cube
        self._draft_cfg = draft_cfg
        self._resonance_pair_fn = resonance_pair_fn
        self._packs: Optional[list[Pack]] = None
        self._round_pick_count: int = 0
        self._round_index: int = 0
        self._global_pick_index: int = 0

    # -- DraftStrategy interface --

    def generate_pick(
        self,
        n: int,
        logger: Optional[SessionLogger] = None,
        context: str = "draft",
    ) -> PickResult:
        """Advance AI seats, filter pack at seat 0, and return shown cards."""
        pack = self._advance_to_human_pick(logger)

        human_res = self._resonance_pair_fn()
        eligible = resonance_filter.filter_off_resonance_duals(pack.cards, human_res)

        pick_rng = random.Random(self._rng.randint(0, 2**32))
        shown_cards = show_n.select_cards(
            eligible,
            n,
            self._draft_cfg.agents.show_n_strategy,
            pick_rng,
            human_w=self._human_agent.w,
            human_drafted=self._human_agent.drafted,
            scoring_cfg=self._draft_cfg.scoring,
        )

        # Log show-N filtering
        if logger is not None and shown_cards:
            strategy = self._draft_cfg.agents.show_n_strategy
            scores = log_helpers.compute_show_n_scores(
                shown_cards, self._human_agent.w, strategy
            )
            shown_with_scores = []
            for card, score in zip(shown_cards, scores):
                entry = log_helpers.card_instance_dict(card)
                entry["score"] = score
                shown_with_scores.append(entry)

            filtered_out = [c for c in pack.cards if c not in shown_cards]
            filtered_scores = log_helpers.compute_show_n_scores(
                filtered_out, self._human_agent.w, strategy
            )
            filtered_top3 = []
            paired = list(zip(filtered_out, filtered_scores))
            paired.sort(key=lambda t: t[1], reverse=True)
            for card, score in paired[:3]:
                entry = log_helpers.card_instance_dict(card)
                entry["score"] = score
                filtered_top3.append(entry)

            logger.log_show_n_filter(
                strategy=strategy,
                pack_size=len(pack.cards),
                shown_count=len(shown_cards),
                shown_cards_with_scores=shown_with_scores,
                filtered_out_top3=filtered_top3,
                context=context,
                global_pick_index=self._global_pick_index,
                round_index=self._round_index,
            )

        return PickResult(shown_cards=shown_cards, all_eligible=eligible)

    def complete_pick(self, chosen: CardInstance, shown: list[CardInstance]) -> None:
        """Complete a human pick: remove from pack, update agent, rotate."""
        assert self._packs is not None
        cfg = self._draft_cfg
        pack = self._packs[0]

        pack.cards.remove(chosen)

        visible_remaining = [c for c in shown if c is not chosen]

        agents.update_agent_after_pick(
            self._human_agent,
            chosen,
            visible_remaining,
            self._global_pick_index,
            self._round_index,
            pack.pack_id,
            cfg.agents.learning_rate,
            cfg.agents.openness_window,
        )

        self._rotate_and_advance()

    def skip_pick(self) -> None:
        """Advance one pick step without taking a card."""
        self._rotate_and_advance()

    def update_after_external_pick(self, card: CardInstance) -> None:
        """Update the human agent after a pick outside the draft loop."""
        cfg = self._draft_cfg
        agents.update_agent_after_pick(
            self._human_agent,
            card,
            [],
            self._global_pick_index,
            self._round_index,
            "discovery",
            cfg.agents.learning_rate,
            cfg.agents.openness_window,
        )

    def render_debug_panel(self) -> str:
        """Build a multi-line debug panel string."""
        lines: list[str] = []

        sep = render.draw_double_separator()
        lines.append(sep)
        lines.append(f"  {colors.header('DEBUG PANEL')}")
        lines.append(sep)

        # Draft status
        round_num = self._round_index + 1
        pick_in_round = self._round_pick_count
        global_pick = self._global_pick_index

        status = (
            f"  Round {colors.num(round_num)}  "
            f"Pick {colors.num(pick_in_round)}  "
            f"Global pick {colors.num(global_pick)}"
        )

        if self._packs is not None and len(self._packs) > 0:
            seat0_pack_size = len(self._packs[0].cards)
            status += f"  Pack[0]: {colors.num(seat0_pack_size)} cards"

        pool_size = self._cube.total_size
        status += f"  Pool: {colors.num(pool_size)}"

        lines.append(status)
        lines.append("")

        _BOT_COLORS = ["entity", "keyword", "constant", "tag", "string"]

        seat_order = [3, 4, 5, 0, 1, 2]
        seat_labels = {
            3: "3 upstream",
            4: "2 upstream",
            5: "LEFT (passes to you)",
            1: "RIGHT (receives from you)",
            2: "2 downstream",
        }

        for seat in seat_order:
            if seat == 0:
                lines.append(f"  {render.draw_separator()}")
                lines.append(f"  {colors.header('YOU (Seat 0)')}")
                lines.append(f"  {render.draw_separator()}")
                continue

            bot_idx = seat - 1
            color_role = _BOT_COLORS[bot_idx % len(_BOT_COLORS)]
            bot_name = colors.c(f"AI Agent {seat}", color_role, bold=True)
            position = seat_labels[seat]

            agent = self._ai_agents[bot_idx]
            w = agent.w

            total = sum(w)
            if total > 0:
                indexed = sorted(enumerate(w), key=lambda x: -x[1])
                top = indexed[:3]
                pref_parts = []
                for arch_idx, val in top:
                    pct = val / total * 100
                    if pct < 5:
                        continue
                    name = render.ARCHETYPE_NAMES[arch_idx]
                    pref_parts.append(f"{name} {pct:.0f}%")
                pref_str = ", ".join(pref_parts) if pref_parts else "uniform"
            else:
                pref_str = "uniform"

            conc = commitment.concentration(w)
            if conc >= 0.25:
                top_arch = max(range(len(w)), key=lambda i: w[i])
                arch_name = render.ARCHETYPE_NAMES[top_arch]
                commit_str = colors.c(f"Committed to {arch_name}", "accent", bold=True)
            else:
                commit_str = colors.dim("Exploring")

            if agent.committed_resonance is not None:
                p, s = agent.committed_resonance
                res_str = colors.c(f"{p}/{s}", "tag")
            else:
                res_str = colors.dim("undecided")

            drafted = len(agent.drafted)

            lines.append(f"  {bot_name}  <- {position}")
            lines.append(
                f"    {pref_str}  |  {commit_str}  |  "
                f"Res: {res_str}  |  {colors.num(drafted)} drafted"
            )

        lines.append("")
        lines.append(sep)
        return "\n".join(lines)

    # -- Read-only properties --

    @property
    def preference_vector(self) -> list[float]:
        return self._human_agent.w

    @property
    def drafted_cards(self) -> list[CardInstance]:
        return self._human_agent.drafted

    @property
    def pick_index(self) -> int:
        return self._global_pick_index

    @property
    def round_index(self) -> int:
        return self._round_index

    @property
    def round_pick_count(self) -> int:
        return self._round_pick_count

    @property
    def show_n_count(self) -> int:
        return self._draft_cfg.agents.show_n

    @property
    def show_n_strategy(self) -> str:
        return self._draft_cfg.agents.show_n_strategy

    @property
    def scoring_cfg(self) -> Any:
        return self._draft_cfg.scoring

    @property
    def cube(self) -> Any:
        return self._cube

    @property
    def draft_cfg(self) -> Any:
        return self._draft_cfg

    # -- Internal helpers --

    def _score_card_for_policy(
        self,
        card: CardInstance,
        ai_agent: agents.AgentState,
        policy: str,
        agents_cfg: AgentsConfig,
        scoring_cfg: ScoringConfig,
    ) -> float:
        """Score a card using the same formula as the AI policy."""
        if policy == "greedy":
            return agents.score_card_greedy(card, ai_agent, scoring_cfg)
        elif policy == "adaptive":
            return agents.score_card_adaptive(card, ai_agent, agents_cfg)
        elif policy == "signal_ignorant":
            return agents.score_card_signal_ignorant(card, ai_agent, agents_cfg)
        else:
            design = getattr(card, "design", card)
            return getattr(design, "power", 0.0)

    def _advance_to_human_pick(self, logger: Optional[SessionLogger] = None) -> Pack:
        """Generate packs if needed, run AI picks, return pack at seat 0."""
        cfg = self._draft_cfg

        if not self._packs:
            pack_rng = random.Random(self._rng.randint(0, 2**32))
            self._packs = [
                pack_generator.generate_pack(
                    cfg.pack_generation.strategy, self._cube, cfg, pack_rng
                )
                for _ in range(cfg.draft.seat_count)
            ]

            if logger is not None:
                logger.log_round_start(
                    round_index=self._round_index,
                    global_pick_index=self._global_pick_index,
                    pack_card_count=len(self._packs[0].cards),
                    seat_count=cfg.draft.seat_count,
                )

        assert self._packs is not None
        packs = self._packs
        pick_rng = random.Random(self._rng.randint(0, 2**32))

        # Run AI picks for seats 1-5
        for seat_idx in range(1, cfg.draft.seat_count):
            pack = packs[seat_idx]
            ai_agent = self._ai_agents[seat_idx - 1]
            candidates = list(pack.cards)

            if (
                ai_agent.committed_resonance is None
                and len(ai_agent.drafted) >= cfg.agents.ai_resonance_commit_pick
            ):
                top_arch = max(range(len(ai_agent.w)), key=lambda i: ai_agent.w[i])
                arch_name = render.ARCHETYPE_NAMES[top_arch]
                ai_agent.committed_resonance = render.ARCHETYPE_RESONANCE.get(arch_name)

            filtered = resonance_filter.filter_off_resonance_duals(
                candidates, ai_agent.committed_resonance
            )
            if filtered:
                candidates = filtered

            seat_rng = random.Random(pick_rng.randint(0, 2**32))
            chosen = agents.pick_card(
                candidates,
                ai_agent,
                cfg.agents.policy,
                cfg.agents,
                cfg.scoring,
                seat_rng,
                force_archetype=None,
            )

            if logger is not None and candidates:
                scores = [
                    (
                        c,
                        self._score_card_for_policy(
                            c,
                            ai_agent,
                            cfg.agents.policy,
                            cfg.agents,
                            cfg.scoring,
                        ),
                    )
                    for c in candidates
                ]
                scores.sort(key=lambda t: t[1], reverse=True)
                policy_optimal = scores[0][0]
                was_random = chosen is not policy_optimal
                chosen_score = next(s for c, s in scores if c is chosen)

                top_alts = []
                for alt_card, alt_score in scores[:3]:
                    if alt_card is not chosen:
                        entry = log_helpers.card_instance_dict(alt_card)
                        entry["score"] = round(alt_score, 4)
                        top_alts.append(entry)

                logger.log_ai_pick(
                    seat_index=seat_idx,
                    round_index=self._round_index,
                    global_pick_index=self._global_pick_index,
                    chosen=chosen,
                    chosen_score=chosen_score,
                    candidates_count=len(candidates),
                    top_alternatives=top_alts,
                    was_random=was_random,
                    agent_w_top3=log_helpers.top_n_w(ai_agent.w),
                )

            pack.cards.remove(chosen)

            visible_remaining = [c for c in candidates if c is not chosen]

            agents.update_agent_after_pick(
                ai_agent,
                chosen,
                visible_remaining,
                self._global_pick_index,
                self._round_index,
                pack.pack_id,
                cfg.agents.learning_rate,
                cfg.agents.openness_window,
            )

        return packs[0]

    def _rotate_and_advance(self) -> None:
        """Rotate packs and increment pick counters."""
        assert self._packs is not None
        self._packs = draft_runner._rotate_packs(self._packs, pass_left=True)

        self._round_pick_count += 1
        self._global_pick_index += 1

        if self._round_pick_count >= PICKS_PER_ROUND:
            self._round_pick_count = 0
            self._round_index += 1
            self._packs = None
