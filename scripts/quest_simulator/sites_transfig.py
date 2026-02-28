"""Transfiguration site interaction for the quest simulator.

Implements transfiguration -- a card upgrade that adds a name prefix and
display note. There are 8 transfiguration types with eligibility rules.
Normal mode shows 3 random non-transfigured cards; enhanced (Prismatic
biome) mode shows the full deck for selection.
"""

import re
from enum import Enum
from typing import Optional

import input_handler
import render
from jsonl_log import SessionLogger
from models import Card, CardType, DeckCard
from quest_state import QuestState


class TransfigType(Enum):
    VIRIDIAN = "Viridian"
    GOLDEN = "Golden"
    SCARLET = "Scarlet"
    MAGENTA = "Magenta"
    AZURE = "Azure"
    BRONZE = "Bronze"
    ROSE = "Rose"
    PRISMATIC = "Prismatic"


_TRIGGER_KEYWORDS = ["judgment", "whenever", "at the start", "at the end", "when"]

_ENERGY_COST_PATTERN = re.compile(r"\d+\s*energy\s*:", re.IGNORECASE)

_TRANSFIG_NOTES: dict[TransfigType, str] = {
    TransfigType.VIRIDIAN: "halved cost",
    TransfigType.GOLDEN: "+1 to an effect",
    TransfigType.SCARLET: "doubled spark",
    TransfigType.MAGENTA: "additional trigger",
    TransfigType.AZURE: "also draws a card",
    TransfigType.BRONZE: "gains Reclaim",
    TransfigType.ROSE: "ability costs less",
    TransfigType.PRISMATIC: "all applicable upgrades",
}

_BASE_TYPES = [
    TransfigType.VIRIDIAN,
    TransfigType.GOLDEN,
    TransfigType.SCARLET,
    TransfigType.MAGENTA,
    TransfigType.AZURE,
    TransfigType.BRONZE,
    TransfigType.ROSE,
]


def is_eligible(card: Card, transfig_type: TransfigType) -> bool:
    """Check whether a card is eligible for a given transfiguration type."""
    if transfig_type == TransfigType.VIRIDIAN:
        return card.energy_cost is not None and card.energy_cost > 0
    elif transfig_type == TransfigType.GOLDEN:
        return bool(re.search(r"\d", card.rules_text))
    elif transfig_type == TransfigType.SCARLET:
        return card.card_type == CardType.CHARACTER
    elif transfig_type == TransfigType.MAGENTA:
        lower_text = card.rules_text.lower()
        return any(kw in lower_text for kw in _TRIGGER_KEYWORDS)
    elif transfig_type == TransfigType.AZURE:
        return card.card_type == CardType.EVENT
    elif transfig_type == TransfigType.BRONZE:
        return card.card_type == CardType.EVENT
    elif transfig_type == TransfigType.ROSE:
        return bool(_ENERGY_COST_PATTERN.search(card.rules_text))
    elif transfig_type == TransfigType.PRISMATIC:
        count = sum(1 for t in _BASE_TYPES if is_eligible(card, t))
        return count >= 2
    return False


def get_applicable_types(card: Card) -> list[TransfigType]:
    """Return all transfiguration types that a card is eligible for."""
    result: list[TransfigType] = []
    for t in _BASE_TYPES:
        if is_eligible(card, t):
            result.append(t)
    if len(result) >= 2:
        result.append(TransfigType.PRISMATIC)
    return result


def _build_transfig_note(
    transfig_type: TransfigType, card_name: str,
) -> str:
    """Build the display note for a transfigured card."""
    note = _TRANSFIG_NOTES[transfig_type]
    return f"{transfig_type.value} {card_name} -- {note}"


def _render_transfig_item(
    index: int,
    option: str,
    is_selected: bool,
    candidates: list[tuple[DeckCard, TransfigType]],
) -> str:
    """Render a single card for the transfiguration single-select menu."""
    if index >= len(candidates):
        # Skip option
        marker = ">" if is_selected else " "
        return f"  {marker} {option}"

    deck_card, transfig_type = candidates[index]
    card_lines = render.format_card(deck_card.card, highlighted=is_selected)
    note = _TRANSFIG_NOTES[transfig_type]
    type_label = f"{render.BOLD}{transfig_type.value}{render.RESET}"
    annotation = f"    {render.DIM}-> {type_label} {render.DIM}-- {note}{render.RESET}"
    return "\n".join([card_lines[0], card_lines[1], annotation])


def run_transfiguration(
    state: QuestState,
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
    is_enhanced: bool = False,
) -> None:
    """Run a Transfiguration site interaction.

    Normal mode selects 3 random non-transfigured cards from the deck,
    assigns each a random applicable transfiguration type, and lets the
    player pick one or skip.

    Enhanced mode (Prismatic biome) displays all non-transfigured cards
    and applies the best applicable type (Prismatic if 2+ types apply).
    """
    # Gather non-transfigured cards
    eligible_deck_cards = [dc for dc in state.deck if not dc.is_transfigured]

    if not eligible_deck_cards:
        print(
            f"  {render.DIM}No cards available for "
            f"transfiguration.{render.RESET}"
        )
        footer = render.resonance_profile_footer(
            counts=state.resonance_profile.snapshot(),
            deck_count=state.deck_count(),
            essence=state.essence,
        )
        print(footer)
        return

    enhanced_label = " (Enhanced)" if is_enhanced else ""
    header = render.site_visit_header(
        dreamscape_name=dreamscape_name,
        site_type_label=f"Transfiguration{enhanced_label}",
        pick_info="Choose a card to upgrade",
        dreamscape_number=dreamscape_number,
    )
    print(header)
    print()

    if is_enhanced:
        _run_enhanced(
            state, eligible_deck_cards, logger,
        )
    else:
        _run_normal(
            state, eligible_deck_cards, logger,
        )

    # Show resonance profile footer
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)


def _run_normal(
    state: QuestState,
    eligible_deck_cards: list[DeckCard],
    logger: Optional[SessionLogger],
) -> None:
    """Normal transfiguration: pick from 3 random cards."""
    # Select up to 3 random non-transfigured cards
    sample_size = min(3, len(eligible_deck_cards))
    sampled = state.rng.sample(eligible_deck_cards, sample_size)

    # For each sampled card, pick a random applicable transfiguration type
    candidates: list[tuple[DeckCard, TransfigType]] = []
    for dc in sampled:
        applicable = [t for t in _BASE_TYPES if is_eligible(dc.card, t)]
        if applicable:
            chosen_type = state.rng.choice(applicable)
            candidates.append((dc, chosen_type))

    if not candidates:
        print(
            f"  {render.DIM}No eligible cards found for "
            f"transfiguration.{render.RESET}"
        )
        if logger is not None:
            logger.log_site_visit(
                site_type="Transfiguration",
                choices=[],
                choice_made=None,
                state_changes={},
            )
        return

    print(
        f"  Choose a card to transfigure, or skip:"
    )
    print()

    # Build option labels
    option_labels = [
        f"{t.value} {dc.card.name}" for dc, t in candidates
    ]
    option_labels.append("Skip")

    def _render_fn(
        index: int,
        option: str,
        is_selected: bool,
        _candidates: list[tuple[DeckCard, TransfigType]] = candidates,
    ) -> str:
        return _render_transfig_item(
            index, option, is_selected, _candidates,
        )

    chosen_index = input_handler.single_select(
        options=option_labels,
        render_fn=_render_fn,
    )

    choice_name: Optional[str] = None
    if chosen_index < len(candidates):
        dc, transfig_type = candidates[chosen_index]
        note = _build_transfig_note(transfig_type, dc.card.name)
        dc.is_transfigured = True
        dc.transfig_note = note
        choice_name = note
        print()
        print(
            f"  {render.BOLD}{dc.card.name}{render.RESET} transfigured "
            f"to {render.BOLD}{transfig_type.value}{render.RESET}!"
        )
    else:
        print()
        print(f"  {render.DIM}Skipped transfiguration.{render.RESET}")

    if logger is not None:
        logger.log_site_visit(
            site_type="Transfiguration",
            choices=[f"{t.value} {dc.card.name}" for dc, t in candidates],
            choice_made=choice_name,
            state_changes={
                "transfigured_card": choice_name,
            },
        )


def _run_enhanced(
    state: QuestState,
    eligible_deck_cards: list[DeckCard],
    logger: Optional[SessionLogger],
) -> None:
    """Enhanced transfiguration: choose any card from the full deck."""
    print(
        f"  {render.BOLD}Prismatic Biome{render.RESET} -- "
        f"choose any card from your deck to transfigure."
    )
    print()

    # Build option labels for all non-transfigured cards
    option_labels = [dc.card.name for dc in eligible_deck_cards]
    option_labels.append("Skip")

    def _render_card_item(
        index: int,
        option: str,
        is_selected: bool,
        _deck: list[DeckCard] = eligible_deck_cards,
    ) -> str:
        if index >= len(_deck):
            marker = ">" if is_selected else " "
            return f"  {marker} {option}"
        card_lines = render.format_card(
            _deck[index].card, highlighted=is_selected,
        )
        return "\n".join(card_lines)

    chosen_index = input_handler.single_select(
        options=option_labels,
        render_fn=_render_card_item,
    )

    choice_name: Optional[str] = None
    if chosen_index < len(eligible_deck_cards):
        dc = eligible_deck_cards[chosen_index]
        applicable = get_applicable_types(dc.card)

        if TransfigType.PRISMATIC in applicable:
            best_type = TransfigType.PRISMATIC
        elif applicable:
            best_type = applicable[0]
        else:
            # Fallback: no applicable type, just skip
            print()
            print(
                f"  {render.DIM}No transfiguration types apply "
                f"to this card.{render.RESET}"
            )
            if logger is not None:
                logger.log_site_visit(
                    site_type="Transfiguration",
                    choices=[dc.card.name for dc in eligible_deck_cards],
                    choice_made=None,
                    state_changes={},
                )
            return

        note = _build_transfig_note(best_type, dc.card.name)
        dc.is_transfigured = True
        dc.transfig_note = note
        choice_name = note
        print()
        print(
            f"  {render.BOLD}{dc.card.name}{render.RESET} transfigured "
            f"to {render.BOLD}{best_type.value}{render.RESET}!"
        )
    else:
        print()
        print(f"  {render.DIM}Skipped transfiguration.{render.RESET}")

    if logger is not None:
        logger.log_site_visit(
            site_type="Transfiguration",
            choices=[dc.card.name for dc in eligible_deck_cards],
            choice_made=choice_name,
            state_changes={
                "transfigured_card": choice_name,
            },
        )
