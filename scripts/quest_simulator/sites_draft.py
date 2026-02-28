"""Draft site interaction for the quest simulator.

Implements the Draft site: 5 sequential picks of 4 cards each,
using resonance-weighted selection from the draft pool. Picked cards
are added to the deck and removed from the pool; unpicked cards
receive +1 staleness.
"""

from typing import Optional

import algorithm
import input_handler
import pool as pool_module
import render
from jsonl_log import SessionLogger
from models import (
    AlgorithmParams,
    Card,
    DraftParams,
    PoolParams,
)
from quest_state import QuestState


def run_draft(
    state: QuestState,
    algorithm_params: AlgorithmParams,
    draft_params: DraftParams,
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
) -> None:
    """Run a Draft site interaction.

    Performs picks_per_site sequential picks. Each pick selects
    cards_per_pick cards from the pool via resonance-weighted
    selection, presents them to the player, and processes the choice.
    """
    for pick_index in range(draft_params.picks_per_site):
        # If pool is empty, attempt a refill
        if not state.pool:
            pool_module.refill_pool(
                state.pool,
                state.all_cards,
                _pool_params_from_state(state),
            )

        # Select cards from pool
        selections = algorithm.select_cards(
            pool=state.pool,
            n=draft_params.cards_per_pick,
            profile=state.resonance_profile,
            params=algorithm_params,
            rng=state.rng,
        )

        if not selections:
            # No cards available even after refill; skip this pick
            continue

        offered_entries = [entry for entry, _ in selections]
        offered_weights = [weight for _, weight in selections]
        offered_cards = [entry.card for entry in offered_entries]

        # Display header
        pick_label = f"Pick {pick_index + 1}/{draft_params.picks_per_site}"
        header = render.site_visit_header(
            dreamscape_name=dreamscape_name,
            site_type_label="Draft",
            pick_info=pick_label,
            dreamscape_number=dreamscape_number,
        )
        print(header)
        print()

        # Build display options and render function for single_select
        option_labels = [card.name for card in offered_cards]

        def _render_card_option(
            index: int,
            option: str,
            is_selected: bool,
            _cards: list[Card] = offered_cards,
        ) -> str:
            card = _cards[index]
            lines = render.format_card(card, highlighted=is_selected)
            return "\n".join(lines)

        # Player selection via arrow-key single select
        chosen_index = input_handler.single_select(
            options=option_labels,
            render_fn=_render_card_option,
        )

        # Process the pick
        picked_entry = offered_entries[chosen_index]
        picked_card = picked_entry.card

        # Add picked card to deck
        state.add_card(picked_card)

        # Remove picked entry from pool
        pool_module.remove_entry(state.pool, picked_entry)

        # Increment staleness on unpicked entries
        unpicked = [e for i, e in enumerate(offered_entries) if i != chosen_index]
        pool_module.increment_staleness(unpicked)

        # Log the pick
        if logger is not None:
            logger.log_draft_pick(
                offered_cards=offered_cards,
                weights=offered_weights,
                picked_card=picked_card,
                profile_snapshot=state.resonance_profile.snapshot(),
            )

    # After all picks, show the resonance profile footer
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)


def _pool_params_from_state(state: QuestState) -> PoolParams:
    """Extract pool params from quest state for refill operations.

    Uses default copy counts; these could be made configurable via
    QuestState if needed.
    """
    return PoolParams(
        copies_common=4,
        copies_uncommon=3,
        copies_rare=2,
        copies_legendary=1,
        variance_min=0.75,
        variance_max=1.25,
    )
