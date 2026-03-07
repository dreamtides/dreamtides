"""Draft site interaction for the quest simulator.

Implements the Draft site: 5 sequential picks using the round manager
to advance the 6-seat draft loop. Each pick filters the pack at seat 0
via show_n using the configured show_n count and strategy, presents the
cards to the player, and signals the round manager on completion.
"""

import random
from typing import Optional

import input_handler
import render
import render_cards
import render_status
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
        pack = round_manager.advance_to_human_pick(state)

        pick_rng = random.Random(state.rng.randint(0, 2**32))
        shown_cards = show_n.select_cards(
            pack.cards,
            show_n_count,
            show_n_strategy,
            pick_rng,
            human_w=state.human_agent.w,
            human_drafted=state.human_agent.drafted,
            scoring_cfg=state.draft_cfg.scoring,
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

        # Show card images once (static, above the interactive selector)
        for card in shown_cards:
            img_lines = render_cards.format_card_display(
                card, highlighted=False, show_images=True
            )
            for line in img_lines:
                print(line)
        print()

        # Build display options
        option_labels = [render_cards.card_name(card) for card in shown_cards]

        def _render_card_option(
            index: int,
            option: str,
            is_selected: bool,
            _cards=shown_cards,
        ) -> str:
            card = _cards[index]
            lines = render_cards.format_card_display(
                card,
                highlighted=is_selected,
            )
            return "\n".join(lines)

        chosen_index = input_handler.single_select(
            options=option_labels,
            render_fn=_render_card_option,
        )

        chosen_card = shown_cards[chosen_index]

        # Signal the round manager
        round_manager.complete_human_pick(state, chosen_card, shown_cards)

        # Add to the player's deck
        state.add_card(chosen_card)

        # Log the pick
        if logger is not None:
            logger.log_draft_pick(
                offered_cards=shown_cards,
                weights=[1.0] * len(shown_cards),
                picked_card=chosen_card,
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
