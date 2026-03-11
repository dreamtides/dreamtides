"""Draft site interaction for the quest simulator.

Implements the Draft site: 5 sequential picks using the round manager
to advance the 6-seat draft loop. Each pick filters the pack at seat 0
via show_n using the configured show_n count and strategy, presents the
cards to the player, and signals the round manager on completion.
"""

import random
from typing import Optional

import input_handler
import log_helpers
import render
import render_cards
import render_status
import resonance_filter
import round_manager
import show_n
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
    Each pick advances the round manager (AI seats pick, round
    starts if needed), filters the pack at seat 0 using the
    configured show_n count and strategy, and lets the player
    choose one.
    """
    show_n_count = state.draft_cfg.agents.show_n
    show_n_strategy = state.draft_cfg.agents.show_n_strategy

    for pick_index in range(PICKS_PER_DRAFT_SITE):
        pack = round_manager.advance_to_human_pick(state, logger=logger)

        human_res = resonance_filter.human_resonance_pair(state)
        eligible = resonance_filter.filter_off_resonance_duals(pack.cards, human_res)

        pick_rng = random.Random(state.rng.randint(0, 2**32))
        shown_cards = show_n.select_cards(
            eligible,
            show_n_count,
            show_n_strategy,
            pick_rng,
            human_w=state.human_agent.w,
            human_drafted=state.human_agent.drafted,
            scoring_cfg=state.draft_cfg.scoring,
        )

        # Log show-N filtering
        if logger is not None and shown_cards:
            scores = log_helpers.compute_show_n_scores(
                shown_cards, state.human_agent.w, show_n_strategy
            )
            shown_with_scores = []
            for card, score in zip(shown_cards, scores):
                entry = log_helpers.card_instance_dict(card)
                entry["score"] = score
                shown_with_scores.append(entry)

            filtered_out = [c for c in pack.cards if c not in shown_cards]
            filtered_scores = log_helpers.compute_show_n_scores(
                filtered_out, state.human_agent.w, show_n_strategy
            )
            filtered_top3 = []
            paired = list(zip(filtered_out, filtered_scores))
            paired.sort(key=lambda t: t[1], reverse=True)
            for card, score in paired[:3]:
                entry = log_helpers.card_instance_dict(card)
                entry["score"] = score
                filtered_top3.append(entry)

            logger.log_show_n_filter(
                strategy=show_n_strategy,
                pack_size=len(pack.cards),
                shown_count=len(shown_cards),
                shown_cards_with_scores=shown_with_scores,
                filtered_out_top3=filtered_top3,
                context="draft",
                global_pick_index=state.global_pick_index,
                round_index=state.round_index,
            )

        if not shown_cards:
            round_manager.advance_pick_no_card(state)
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
        render_cards.render_card_columns(shown_cards)
        print(render.draw_separator())

        option_labels = [render_cards.card_name(card) for card in shown_cards]
        if state.debug:
            option_labels.append("Debug")

        while True:
            chosen_index = input_handler.single_select(
                options=option_labels,
            )

            if state.debug and chosen_index == len(shown_cards):
                import debug_panel

                print(debug_panel.render_debug_panel(state))
                input_handler.wait_for_continue()
                # Re-display header and cards before re-prompting
                print(header)
                print()
                render_cards.render_card_columns(shown_cards)
                print(render.draw_separator())
                continue

            break

        chosen_card = shown_cards[chosen_index]

        # Signal the round manager
        round_manager.complete_human_pick(state, chosen_card, shown_cards)

        # Add to the player's deck
        state.add_card(chosen_card)

        # Log the pick
        if logger is not None:
            scores = log_helpers.compute_show_n_scores(
                shown_cards, state.human_agent.w, show_n_strategy
            )
            logger.log_draft_pick(
                offered_cards=shown_cards,
                weights=scores,
                picked_card=chosen_card,
                global_pick_index=state.global_pick_index,
                round_index=state.round_index,
                round_pick_count=state.round_pick_count,
                human_w_top3=log_helpers.top_n_w(state.human_agent.w),
                context="draft",
            )
            logger.log_preference_snapshot(
                global_pick_index=state.global_pick_index,
                preference_vector=state.human_agent.w,
                top_archetype_index=log_helpers.top_n_w(state.human_agent.w, 1)[0][0],
                concentration=log_helpers.w_concentration(state.human_agent.w),
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
        w=state.human_agent.w,
        deck_count=len(state.deck),
        essence=state.essence,
    )
    print(footer)
