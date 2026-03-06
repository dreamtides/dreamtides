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

import agents
import draft_runner
import pack_generator
from draft_models import CardInstance, Pack

PICKS_PER_ROUND = 10


def advance_to_human_pick(state):
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

    pick_rng = random.Random(state.rng.randint(0, 2**32))

    # Run AI picks for seats 1-5
    for seat_idx in range(1, cfg.draft.seat_count):
        pack = state.packs[seat_idx]
        ai_agent = state.ai_agents[seat_idx - 1]
        candidates = list(pack.cards)

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
