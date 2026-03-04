"""Draft runner: round orchestration and full draft execution.

Integrates card generation, cube management, pack generation, agent
policies, Show-N strategies, and refill strategies into a complete
multi-seat draft simulation. Produces a DraftResult containing per-seat
pools, preference vector histories, commitment data, and optional
per-pick trace records. Stdlib-only, no external dependencies.
"""

import random
import sys
from dataclasses import dataclass
from typing import Optional

import agents
import card_generator
import commitment
import config
import cube_manager
import deck_scorer
import pack_generator
import refill
import show_n
from draft_models import CardDesign, CardInstance, CubeConsumptionMode, Pack
from utils import argmax


@dataclass(frozen=True)
class PickTrace:
    """Record of a single pick for debugging and analysis."""

    round_index: int
    pick_index: int
    seat_index: int
    pack_id: str
    pack_card_ids: list[int]
    shown_card_ids: list[int]
    chosen_card_id: int
    agent_w_snapshot: list[float]
    card_score: float


@dataclass(frozen=True)
class SeatResult:
    """Final result data for one seat after the draft."""

    drafted: list[CardInstance]
    final_w: list[float]
    deck_value: float
    commitment_pick: Optional[int]
    committed_archetype: Optional[int]
    pick_history: list[tuple[int, int, str, str]]
    w_history: list[list[float]]


@dataclass(frozen=True)
class DraftResult:
    """Complete output of one draft run."""

    seat_results: list[SeatResult]
    traces: list[PickTrace]
    seed: int


def run_draft(
    cfg: config.SimulatorConfig,
    seed: int,
    trace_enabled: bool = False,
) -> DraftResult:
    """Execute a full multi-seat draft and return the result.

    Generates or loads the card pool, creates a cube, initializes agents,
    executes all rounds, computes per-seat deck_value and commitment
    detection, and returns a DraftResult.
    """
    rng = random.Random(seed)

    cards = card_generator.generate_cards(cfg, rng)

    consumption_mode = (
        CubeConsumptionMode.WITH_REPLACEMENT
        if cfg.cube.consumption_mode == "with_replacement"
        else CubeConsumptionMode.WITHOUT_REPLACEMENT
    )
    cube = cube_manager.CubeManager(
        designs=cards,
        copies_per_card=cfg.cube.copies_per_card,
        consumption_mode=consumption_mode,
    )
    cube_manager.validate_supply(cfg, cube.total_size)

    seat_count = cfg.draft.seat_count
    archetype_count = cfg.cards.archetype_count

    agent_states = [agents.create_agent(archetype_count) for _ in range(seat_count)]
    w_histories: list[list[list[float]]] = [[] for _ in range(seat_count)]
    all_traces: list[PickTrace] = []

    _validate_pick_constraints(cfg)

    global_pick_index = 0

    for round_idx in range(cfg.draft.round_count):
        pack_rng = random.Random(rng.randint(0, 2**32))
        packs = [
            pack_generator.generate_pack(
                cfg.pack_generation.strategy, cube, cfg, pack_rng
            )
            for _ in range(seat_count)
        ]

        round_env_profile = refill.compute_round_environment_profile(packs)

        picks_this_round = cfg.draft.picks_per_round[round_idx]
        pass_left = _pass_direction_is_left(round_idx, cfg.draft.alternate_direction)

        for pick_k in range(picks_this_round):
            pick_rng = random.Random(rng.randint(0, 2**32))

            for seat_idx in range(seat_count):
                pack = packs[seat_idx]
                agent = agent_states[seat_idx]
                is_human = seat_idx < cfg.draft.human_seats

                if is_human:
                    shown = show_n.select_cards(
                        pack.cards,
                        cfg.agents.show_n,
                        cfg.agents.show_n_strategy,
                        random.Random(pick_rng.randint(0, 2**32)),
                        human_w=agent.w,
                    )
                    candidates = shown
                else:
                    candidates = list(pack.cards)
                    shown = candidates

                seat_rng = random.Random(pick_rng.randint(0, 2**32))
                chosen = agents.pick_card(
                    candidates,
                    agent,
                    cfg.agents.policy,
                    cfg.agents,
                    cfg.scoring,
                    seat_rng,
                )

                if trace_enabled:
                    score = _compute_trace_score(chosen, agent, cfg.agents.policy, cfg)
                    trace = PickTrace(
                        round_index=round_idx,
                        pick_index=global_pick_index,
                        seat_index=seat_idx,
                        pack_id=pack.pack_id,
                        pack_card_ids=[c.instance_id for c in pack.cards],
                        shown_card_ids=[c.instance_id for c in shown],
                        chosen_card_id=chosen.instance_id,
                        agent_w_snapshot=list(agent.w),
                        card_score=score,
                    )
                    all_traces.append(trace)

                pack.cards.remove(chosen)

                agents.update_agent_after_pick(
                    agent,
                    chosen,
                    pack.cards,
                    global_pick_index,
                    round_idx,
                    pack.pack_id,
                    cfg.agents.learning_rate,
                    cfg.agents.openness_window,
                )

                w_histories[seat_idx].append(list(agent.w))

            packs = _rotate_packs(packs, pass_left)

            _apply_refill(packs, cube, cfg, round_env_profile, pick_rng)

            global_pick_index += 1

    seat_results = _build_seat_results(agent_states, w_histories, cfg)

    return DraftResult(
        seat_results=seat_results,
        traces=all_traces,
        seed=seed,
    )


def _validate_pick_constraints(cfg: config.SimulatorConfig) -> None:
    """Validate pick/pack size constraints for the draft configuration.

    In no-refill mode, rejects configurations where any round's pick count
    exceeds pack_size. In refill mode, warns if any round's pick count
    exceeds 2 * pack_size.
    """
    for r, picks in enumerate(cfg.draft.picks_per_round):
        if cfg.refill.strategy == "no_refill":
            if picks > cfg.draft.pack_size:
                raise ValueError(
                    f"Round {r}: picks_per_round={picks} exceeds "
                    f"pack_size={cfg.draft.pack_size} in no_refill mode"
                )
        else:
            if picks > 2 * cfg.draft.pack_size:
                print(
                    f"WARNING: Round {r}: picks_per_round={picks} exceeds "
                    f"2 * pack_size={2 * cfg.draft.pack_size} in refill mode",
                    file=sys.stderr,
                )


def _pass_direction_is_left(
    round_index: int,
    alternate_direction: bool,
) -> bool:
    """Determine whether packs pass left for this round.

    Left by default. When alternate_direction is true, odd-numbered
    rounds pass right instead.
    """
    if alternate_direction and round_index % 2 == 1:
        return False
    return True


def _rotate_packs(packs: list[Pack], pass_left: bool) -> list[Pack]:
    """Rotate packs one position in the specified direction.

    Left pass: seat s passes to seat (s + 1) % seat_count.
    Right pass: seat s passes to seat (s - 1) % seat_count.
    """
    n = len(packs)
    if n <= 1:
        return packs

    if pass_left:
        return [packs[-1]] + packs[:-1]
    else:
        return packs[1:] + [packs[0]]


def _apply_refill(
    packs: list[Pack],
    cube: cube_manager.CubeManager,
    cfg: config.SimulatorConfig,
    round_env_profile: list[float],
    rng: random.Random,
) -> None:
    """Apply the configured refill strategy to each pack."""
    if cfg.refill.strategy == "no_refill":
        return

    for pack in packs:
        refill_rng = random.Random(rng.randint(0, 2**32))
        if cfg.refill.strategy == "uniform_refill":
            card = refill.uniform_refill(cube, refill_rng)
        elif cfg.refill.strategy == "constrained_refill":
            if cfg.refill.fingerprint_source == "round_environment":
                signal = round_env_profile
            else:
                signal = pack.archetype_profile
            card = refill.constrained_refill(
                cube,
                signal,
                cfg.refill.fidelity,
                cfg.refill.commit_bias,
                refill_rng,
            )
        else:
            raise ValueError(f"Unknown refill strategy: {cfg.refill.strategy!r}")
        pack.cards.append(card)


def _compute_trace_score(
    card: CardInstance,
    agent: agents.AgentState,
    policy: str,
    cfg: config.SimulatorConfig,
) -> float:
    """Compute the policy-specific score for trace recording."""
    if policy == "greedy":
        return agents.score_card_greedy(card, agent, cfg.scoring)
    elif policy == "adaptive":
        return agents.score_card_adaptive(card, agent, cfg.agents)
    elif policy == "signal_ignorant":
        return agents.score_card_signal_ignorant(card, agent, cfg.agents)
    elif policy == "archetype_loyal":
        best = argmax(agent.w)
        return card.design.fitness[best]
    elif policy == "force":
        return card.design.fitness[0]
    return 0.0


def _build_seat_results(
    agent_states: list[agents.AgentState],
    w_histories: list[list[list[float]]],
    cfg: config.SimulatorConfig,
) -> list[SeatResult]:
    """Build SeatResult for each seat from agent state and w history."""
    results: list[SeatResult] = []

    for seat_idx, agent in enumerate(agent_states):
        dv = deck_scorer.deck_value(agent.drafted, agent.w, cfg.scoring)

        commit_result = commitment.detect_commitment(
            w_histories[seat_idx], cfg.commitment
        )

        results.append(
            SeatResult(
                drafted=list(agent.drafted),
                final_w=list(agent.w),
                deck_value=dv,
                commitment_pick=commit_result.commitment_pick,
                committed_archetype=commit_result.committed_archetype,
                pick_history=list(agent.pick_history),
                w_history=w_histories[seat_idx],
            )
        )

    return results
