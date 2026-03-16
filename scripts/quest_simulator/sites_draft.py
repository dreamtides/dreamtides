"""Draft site interaction for the quest simulator.

Implements the Draft site: 5 sequential picks using the draft strategy
to advance the 6-seat draft loop. Each pick generates a PickResult,
presents the shown cards to the player, and signals the strategy on
completion.
"""

import random
from typing import Optional

import input_handler
import log_helpers
import render
import render_cards
import render_status
from draft_strategy import ArchetypeDraftStrategy, RankDraftStrategy
from jsonl_log import SessionLogger
from quest_state import QuestState

PICKS_PER_DRAFT_SITE = 5


def run_draft(
    state: QuestState,
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
    is_enhanced: bool = False,
) -> None:
    """Run a Draft site interaction.

    Consumes 5 consecutive human-seat picks from the draft loop.
    Each pick uses the draft strategy to generate shown cards,
    lets the player choose one, and signals completion.
    """
    strategy = state.draft_strategy
    show_n_count = strategy.show_n_count

    for pick_index in range(PICKS_PER_DRAFT_SITE):
        result = strategy.generate_pick(n=show_n_count, logger=logger, context="draft")
        shown_cards = result.shown_cards
        eligible = result.all_eligible

        if not shown_cards:
            strategy.skip_pick()
            continue

        # Display header
        pick_label = f"Pick {pick_index + 1}/{PICKS_PER_DRAFT_SITE}"
        header = render.site_visit_header(
            dreamscape_name=dreamscape_name,
            site_type_label="Draft",
            pick_info=pick_label,
            dreamscape_number=dreamscape_number,
        )
        print(header)
        print()

        # Show card columns (images + text) above interactive selector
        render_cards.render_card_columns(shown_cards, debug=state.debug, show_archetype_icons=not state.simple_draft)
        print(render.draw_separator())

        option_labels = [render_cards.card_name(card) for card in shown_cards]
        is_simple_draft = isinstance(strategy, (ArchetypeDraftStrategy, RankDraftStrategy))
        if state.debug and not is_simple_draft:
            option_labels.append("Debug")

        # Build remaining cards data for web UI display
        remaining = [c for c in eligible if c not in shown_cards]
        remaining_data = None
        if remaining:
            remaining_data = {
                "remaining_cards": [
                    input_handler.make_card_option_data(
                        name=render_cards.card_name(c),
                        energy_cost=c.design.energy_cost,
                        card_type=c.design.card_type,
                        rules_text=c.design.rules_text,
                        spark=c.design.spark,
                        fitness=(
                            list(c.design.fitness)
                            if state.debug and hasattr(c.design, "fitness")
                            else None
                        ),
                    )
                    for c in remaining
                ]
            }

        while True:
            chosen_index = input_handler.single_select(
                options=option_labels,
                extra_data=remaining_data,
            )

            if state.debug and chosen_index == len(shown_cards):
                print(strategy.render_debug_panel())
                input_handler.wait_for_continue()
                # Re-display header and cards before re-prompting
                print(header)
                print()
                render_cards.render_card_columns(shown_cards, debug=state.debug, show_archetype_icons=not state.simple_draft)
                print(render.draw_separator())
                continue

            break

        chosen_card = shown_cards[chosen_index]

        # Signal the draft strategy
        strategy.complete_pick(chosen_card, shown_cards)

        # Add to the player's deck
        state.add_card(chosen_card)

        # Log the pick
        if logger is not None:
            show_n_strat = strategy.show_n_strategy
            scores = log_helpers.compute_show_n_scores(
                shown_cards, strategy.preference_vector, show_n_strat
            )
            logger.log_draft_pick(
                offered_cards=shown_cards,
                weights=scores,
                picked_card=chosen_card,
                global_pick_index=strategy.pick_index,
                round_index=strategy.round_index,
                round_pick_count=strategy.round_pick_count,
                human_w_top3=log_helpers.top_n_w(strategy.preference_vector),
                context="draft",
            )
            logger.log_preference_snapshot(
                global_pick_index=strategy.pick_index,
                preference_vector=strategy.preference_vector,
                top_archetype_index=log_helpers.top_n_w(strategy.preference_vector, 1)[
                    0
                ][0],
                concentration=log_helpers.w_concentration(strategy.preference_vector),
            )

    # Log the overall site visit
    if logger is not None:
        logger.log_site_visit(
            site_type="Draft",
            dreamscape=dreamscape_name,
            is_enhanced=is_enhanced,
            choices=[],
            choice_made=None,
            state_changes={
                "picks_completed": PICKS_PER_DRAFT_SITE,
                "deck_size_after": state.deck_count(),
            },
        )

    # Show the archetype preference footer
    footer = render_status.archetype_preference_footer(
        w=strategy.preference_vector,
        deck_count=len(state.deck),
        essence=state.essence,
        archetype_draft=state.simple_draft,
    )
    print(footer)
