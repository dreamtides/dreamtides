"""Purge site interaction for the quest simulator.

Implements voluntary purge (remove up to 3 or 6 cards from deck) and
forced deck-limit purge (remove cards until deck is at or below the
maximum size). Removed cards are permanently gone -- not returned to
the draft pool.
"""

from typing import Optional

import input_handler
import render
from jsonl_log import SessionLogger
from models import DeckCard
from quest_state import QuestState


def _render_purge_item(
    index: int,
    option: str,
    is_highlighted: bool,
    is_checked: bool,
    deck: list[DeckCard],
) -> str:
    """Render a single deck card for the purge multi-select menu."""
    if index >= len(deck):
        # "Done" / skip option
        marker = ">" if is_highlighted else " "
        check = "[x]" if is_checked else "[ ]"
        return f"  {marker} {check} {option}"

    card = deck[index].card
    card_lines = render.format_card(card, highlighted=is_highlighted)
    check = "[x]" if is_checked else "[ ]"

    line1 = card_lines[0]
    if is_highlighted:
        line1 = f"  > {check} " + line1.lstrip(" >")
    else:
        line1 = f"    {check} " + line1.lstrip(" ")

    return "\n".join([line1, card_lines[1]])


def run_purge(
    state: QuestState,
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
    is_enhanced: bool = False,
) -> None:
    """Run a voluntary Purge site interaction.

    Displays the full deck and allows the player to multi-select up to
    3 cards to permanently remove (6 if enhanced by the Ashen biome).
    Removed cards are not returned to the draft pool.
    """
    max_purge = 6 if is_enhanced else 3

    if not state.deck:
        print(f"  {render.DIM}Deck is empty -- nothing to purge.{render.RESET}")
        footer = render.resonance_profile_footer(
            counts=state.resonance_profile.snapshot(),
            deck_count=state.deck_count(),
            essence=state.essence,
        )
        print(footer)
        return

    # Display header
    enhanced_label = " (Enhanced)" if is_enhanced else ""
    header = render.site_visit_header(
        dreamscape_name=dreamscape_name,
        site_type_label=f"Purge{enhanced_label}",
        pick_info=f"Remove up to {max_purge} cards",
        dreamscape_number=dreamscape_number,
    )
    print(header)
    print()
    print(
        f"  Select up to {render.BOLD}{max_purge}{render.RESET} cards to "
        f"permanently remove from your deck ({state.deck_count()} cards)."
    )
    print()

    # Build option labels from the current deck
    deck_snapshot = list(state.deck)
    option_labels = [dc.card.name for dc in deck_snapshot]

    def _render_fn(
        index: int,
        option: str,
        is_highlighted: bool,
        is_checked: bool,
        _deck: list[DeckCard] = deck_snapshot,
    ) -> str:
        return _render_purge_item(index, option, is_highlighted, is_checked, _deck)

    selected_indices = input_handler.multi_select(
        options=option_labels,
        render_fn=_render_fn,
        max_selections=max_purge,
    )

    # Remove selected cards (iterate in reverse to preserve indices)
    removed_cards: list[DeckCard] = [deck_snapshot[i] for i in selected_indices]
    for dc in removed_cards:
        state.remove_card(dc)

    if removed_cards:
        print()
        print(
            f"  Purged {len(removed_cards)} card(s) from deck. "
            f"Deck now has {state.deck_count()} cards."
        )
    else:
        print()
        print(f"  {render.DIM}No cards purged.{render.RESET}")

    # Log the interaction
    if logger is not None:
        logger.log_site_visit(
            site_type="Purge",
            dreamscape=dreamscape_name,
            is_enhanced=is_enhanced,
            choices=[dc.card.name for dc in deck_snapshot],
            choice_made=(
                ", ".join(dc.card.name for dc in removed_cards)
                if removed_cards
                else None
            ),
            state_changes={
                "cards_removed": [dc.card.name for dc in removed_cards],
                "deck_size_after": state.deck_count(),
            },
            profile_snapshot=state.resonance_profile.snapshot(),
        )

    # Show resonance profile footer
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)


def forced_deck_limit_purge(
    state: QuestState,
    logger: Optional[SessionLogger],
    dreamscape_name: str = "",
) -> None:
    """Force the player to remove cards until deck is at or below max_deck.

    Called before battles when the deck exceeds the maximum size. The
    player must keep removing cards until the limit is satisfied. In
    non-interactive mode, excess cards are removed automatically.
    """
    if not state.is_over_deck_limit():
        return

    excess = state.deck_count() - state.max_deck

    print()
    print(
        f"  {render.BOLD}Deck Limit Exceeded!{render.RESET} "
        f"Your deck has {state.deck_count()} cards "
        f"(maximum {state.max_deck}). "
        f"You must remove at least "
        f"{excess} card(s)."
    )
    print()

    # In non-interactive mode, auto-remove random excess cards
    if not input_handler._is_interactive():
        to_remove = state.rng.sample(state.deck, excess)
        for dc in to_remove:
            state.remove_card(dc)
        print(
            f"  Auto-removed {len(to_remove)} card(s). "
            f"Deck: {state.deck_count()}/{state.max_deck}."
        )
    else:
        while state.is_over_deck_limit():
            current_excess = state.deck_count() - state.max_deck
            print(
                f"  {render.DIM}Must remove {current_excess} more card(s) "
                f"({state.deck_count()}/{state.max_deck}).{render.RESET}"
            )
            print()

            deck_snapshot = list(state.deck)
            option_labels = [dc.card.name for dc in deck_snapshot]

            def _render_fn(
                index: int,
                option: str,
                is_highlighted: bool,
                is_checked: bool,
                _deck: list[DeckCard] = deck_snapshot,
            ) -> str:
                return _render_purge_item(
                    index, option, is_highlighted, is_checked, _deck
                )

            selected_indices = input_handler.multi_select(
                options=option_labels,
                render_fn=_render_fn,
            )

            removed_cards: list[DeckCard] = [deck_snapshot[i] for i in selected_indices]
            for dc in removed_cards:
                state.remove_card(dc)

            if removed_cards:
                print(
                    f"  Removed {len(removed_cards)} card(s). "
                    f"Deck: {state.deck_count()}/{state.max_deck}."
                )
            else:
                print(
                    f"  {render.DIM}No cards selected. "
                    f"You must remove cards to continue.{render.RESET}"
                )

    print(
        f"  {render.BOLD}Deck is now within the limit "
        f"({state.deck_count()}/{state.max_deck}).{render.RESET}"
    )

    # Log the forced purge
    if logger is not None:
        logger.log_site_visit(
            site_type="ForcedPurge",
            dreamscape=dreamscape_name,
            choices=[],
            choice_made=None,
            state_changes={
                "deck_size_after": state.deck_count(),
            },
            profile_snapshot=state.resonance_profile.snapshot(),
        )

    # Show resonance profile footer
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)
