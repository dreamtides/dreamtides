"""Incremental round manager for the quest draft loop.

Advances the 6-seat draft loop one human pick at a time. Generates
packs at round boundaries, runs AI picks at seats 1-5, and rotates
packs after each completed pick step.

This module mirrors the round/pick/seat structure of
draft_runner.run_draft() (lines 112-203) but operates incrementally
rather than in batch. It delegates to the same sub-functions
(pack_generator.generate_pack, agents.pick_card,
agents.update_agent_after_pick, draft_runner._rotate_packs) to
ensure behavioral parity with the canonical draft loop.
"""

import random
from typing import Optional

import agents
import draft_runner
import log_helpers
import pack_generator
import render
import resonance_filter
from config import AgentsConfig, ScoringConfig
from draft_models import CardInstance, Pack
from jsonl_log import SessionLogger

PICKS_PER_ROUND = 10


def _score_card_for_policy(
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
        return getattr(design, "rarity_value", 0.0)


def advance_to_human_pick(state, logger: Optional[SessionLogger] = None):
    """Ensure packs exist and run AI picks, returning the pack at seat 0.

    If no round is active (state.packs is None or empty), generates
    fresh packs to start a new round. Then runs AI picks for seats 1-5
    on their current packs. Returns the pack at seat 0 so the calling
    site handler can filter and present cards to the player.
    """
    cfg = state.draft_cfg

    # Start a new round if needed
    if not state.packs:
        pack_rng = random.Random(state.rng.randint(0, 2**32))
        state.packs = [
            pack_generator.generate_pack(
                cfg.pack_generation.strategy, state.cube, cfg, pack_rng
            )
            for _ in range(cfg.draft.seat_count)
        ]

        if logger is not None:
            logger.log_round_start(
                round_index=state.round_index,
                global_pick_index=state.global_pick_index,
                pack_card_count=len(state.packs[0].cards),
                seat_count=cfg.draft.seat_count,
            )

    pick_rng = random.Random(state.rng.randint(0, 2**32))

    # Run AI picks for seats 1-5
    for seat_idx in range(1, cfg.draft.seat_count):
        pack = state.packs[seat_idx]
        ai_agent = state.ai_agents[seat_idx - 1]
        candidates = list(pack.cards)

        # Commit AI resonance pair once enough cards are drafted
        if (
            ai_agent.committed_resonance is None
            and len(ai_agent.drafted) >= cfg.agents.ai_resonance_commit_pick
        ):
            top_arch = max(range(len(ai_agent.w)), key=lambda i: ai_agent.w[i])
            arch_name = render.ARCHETYPE_NAMES[top_arch]
            ai_agent.committed_resonance = render.ARCHETYPE_RESONANCE.get(arch_name)

        # Filter out off-resonance dual cards
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

        # Log AI pick details
        if logger is not None and candidates:
            scores = [
                (
                    c,
                    _score_card_for_policy(
                        c, ai_agent, cfg.agents.policy, cfg.agents, cfg.scoring
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
                round_index=state.round_index,
                global_pick_index=state.global_pick_index,
                chosen=chosen,
                chosen_score=chosen_score,
                candidates_count=len(candidates),
                top_alternatives=top_alts,
                was_random=was_random,
                agent_w_top3=log_helpers.top_n_w(ai_agent.w),
                agent_w=ai_agent.w,
                committed_resonance=ai_agent.committed_resonance,
                drafted_count=len(ai_agent.drafted),
                concentration=log_helpers.w_concentration(ai_agent.w),
            )

        pack.cards.remove(chosen)

        visible_remaining = [c for c in candidates if c is not chosen]

        agents.update_agent_after_pick(
            ai_agent,
            chosen,
            visible_remaining,
            state.global_pick_index,
            state.round_index,
            pack.pack_id,
            cfg.agents.learning_rate,
            cfg.agents.openness_window,
        )

    return state.packs[0]


def complete_human_pick(state, chosen_card, shown_cards):
    """Complete a human pick step after the player has chosen a card.

    Removes the chosen card from the pack at seat 0, updates the
    human agent, rotates all packs, and increments pick counters.
    If the round boundary is reached, resets for a new round.
    """
    cfg = state.draft_cfg
    pack = state.packs[0]

    pack.cards.remove(chosen_card)

    visible_remaining = [c for c in shown_cards if c is not chosen_card]

    agents.update_agent_after_pick(
        state.human_agent,
        chosen_card,
        visible_remaining,
        state.global_pick_index,
        state.round_index,
        pack.pack_id,
        cfg.agents.learning_rate,
        cfg.agents.openness_window,
    )

    _rotate_and_advance(state)


def advance_pick_no_card(state):
    """Advance one pick step without taking a card.

    Used by shop reroll and 'buy nothing' cases. No card is removed
    from the pack, but packs still rotate and counters increment.
    The human agent is not updated since no card was selected.
    """
    _rotate_and_advance(state)


def _rotate_and_advance(state):
    """Rotate packs and increment pick counters.

    Uses draft_runner._rotate_packs() for rotation to stay in sync
    with the canonical draft loop. Always passes left since the quest
    uses alternate_direction=False.
    """
    state.packs = draft_runner._rotate_packs(state.packs, pass_left=True)

    state.round_pick_count += 1
    state.global_pick_index += 1

    if state.round_pick_count >= PICKS_PER_ROUND:
        state.round_pick_count = 0
        state.round_index += 1
        state.packs = None
