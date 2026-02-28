"""Transfiguration site interaction for the quest simulator.

Implements transfiguration -- a card upgrade that adds a name prefix and
display note. There are 8 transfiguration types with eligibility rules.
Normal mode shows 3 random non-transfigured cards; enhanced (Prismatic
biome) mode shows the full deck for selection. Each card preview shows
the proposed transformation, eligibility reason, and colored type name.
"""

import os
import re
import sys
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

# ANSI color codes for each transfiguration type
_TRANSFIG_COLORS: dict[TransfigType, str] = {
    TransfigType.VIRIDIAN: "\033[92m",
    TransfigType.GOLDEN: "\033[93m",
    TransfigType.SCARLET: "\033[91m",
    TransfigType.MAGENTA: "\033[95m",
    TransfigType.AZURE: "\033[94m",
    TransfigType.BRONZE: "\033[2;33m",
    TransfigType.ROSE: "\033[35m",
    TransfigType.PRISMATIC: "\033[1;97m",
}

if os.environ.get("NO_COLOR") or not sys.stdout.isatty():
    _TRANSFIG_COLORS = {t: "" for t in TransfigType}


def transfig_type_color(transfig_type: TransfigType) -> str:
    """Return the ANSI color code for a transfiguration type."""
    return _TRANSFIG_COLORS[transfig_type]


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


def eligibility_explanation(card: Card, transfig_type: TransfigType) -> str:
    """Return a human-readable reason why a card is eligible for a type."""
    if transfig_type == TransfigType.VIRIDIAN:
        return f"energy cost is {card.energy_cost} > 0"
    elif transfig_type == TransfigType.GOLDEN:
        return "rules text contains numbers"
    elif transfig_type == TransfigType.SCARLET:
        return "card is a Character"
    elif transfig_type == TransfigType.MAGENTA:
        lower_text = card.rules_text.lower()
        found = [kw for kw in _TRIGGER_KEYWORDS if kw in lower_text]
        if found:
            return f"has trigger keyword \"{found[0]}\""
        return "has trigger keywords"
    elif transfig_type == TransfigType.AZURE:
        return "card is an Event"
    elif transfig_type == TransfigType.BRONZE:
        return "card is an Event"
    elif transfig_type == TransfigType.ROSE:
        return "has activated ability with energy cost"
    elif transfig_type == TransfigType.PRISMATIC:
        applicable = [t for t in _BASE_TYPES if is_eligible(card, t)]
        names = ", ".join(t.value for t in applicable)
        return f"eligible for {names}"
    return ""


def _colored_type_label(transfig_type: TransfigType) -> str:
    """Return the transfiguration type name with its color applied."""
    color = _TRANSFIG_COLORS[transfig_type]
    return f"{color}{render.BOLD}{transfig_type.value}{render.RESET}"


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
    """Render a single card for the transfiguration single-select menu.

    Shows a 4-line preview:
    Line 1: card name with resonance color + stats
    Line 2: quoted rules text
    Line 3: Transfiguration type and effect note (colored)
    Line 4: Eligibility reason (dim)
    """
    if index >= len(candidates):
        # Skip option
        marker = ">" if is_selected else " "
        return f"  {marker} {render.DIM}Skip transfiguration{render.RESET}"

    deck_card, transfig_type = candidates[index]
    card = deck_card.card

    # Line 1: name -> Transfig Name + stats
    marker = ">" if is_selected else " "
    name_color = render.card_color(card.resonances)
    res_str = render.color_resonances(card.resonances)
    badge = render.rarity_badge(card.rarity)
    cost_str = (
        f"Cost: {card.energy_cost}"
        if card.energy_cost is not None
        else "Cost: -"
    )
    spark_str = f"Spark: {card.spark}" if card.spark is not None else ""

    right_parts = [res_str, badge, cost_str]
    if spark_str:
        right_parts.append(spark_str)
    right_side = "   ".join(right_parts)

    transformed_name = f"{transfig_type.value} {card.name}"
    prefix = f"  {marker} "
    vis_right = render.visible_len(right_side)
    max_name_width = render.CONTENT_WIDTH - len(prefix) - 2 - vis_right
    if max_name_width < 1:
        max_name_width = 1

    display_name = transformed_name
    if len(display_name) > max_name_width:
        display_name = display_name[: max_name_width - 1] + "\u2026"
    colored_name = f"{name_color}{display_name}{render.RESET}"

    line1 = f"{prefix}{colored_name}"
    vis_line1 = render.visible_len(line1)
    gap = max(2, render.CONTENT_WIDTH - vis_line1 - vis_right)
    line1 = f"{line1}{' ' * gap}{right_side}"

    # Line 2: quoted rules text
    quoted = f'"{card.rules_text}"'
    if is_selected:
        line2 = f"    {quoted}"
    else:
        max_text_width = render.CONTENT_WIDTH - 4
        if len(quoted) > max_text_width:
            quoted = quoted[: max_text_width - 4] + '..."'
        line2 = f"    {quoted}"

    # Line 3: transfiguration type and effect
    type_label = _colored_type_label(transfig_type)
    note = _TRANSFIG_NOTES[transfig_type]
    line3 = f"    Transfiguration: {type_label}{render.DIM} -- {note}{render.RESET}"

    # Line 4: eligibility reason
    reason = eligibility_explanation(card, transfig_type)
    line4 = f"    {render.DIM}(Eligible: {reason}){render.RESET}"

    return "\n".join([line1, line2, line3, line4])


def _render_enhanced_item(
    index: int,
    option: str,
    is_selected: bool,
    deck: list[DeckCard],
    assigned_types: list[TransfigType],
) -> str:
    """Render a single card for the enhanced transfiguration menu.

    Shows the card with the best applicable transfiguration type and all
    eligible types listed for Prismatic.
    """
    if index >= len(deck):
        marker = ">" if is_selected else " "
        return f"  {marker} {render.DIM}Skip transfiguration{render.RESET}"

    dc = deck[index]
    card = dc.card
    best_type = assigned_types[index]

    # Line 1: transformed name + stats
    marker = ">" if is_selected else " "
    name_color = render.card_color(card.resonances)
    res_str = render.color_resonances(card.resonances)
    badge = render.rarity_badge(card.rarity)
    cost_str = (
        f"Cost: {card.energy_cost}"
        if card.energy_cost is not None
        else "Cost: -"
    )
    spark_str = f"Spark: {card.spark}" if card.spark is not None else ""

    right_parts = [res_str, badge, cost_str]
    if spark_str:
        right_parts.append(spark_str)
    right_side = "   ".join(right_parts)

    transformed_name = f"{best_type.value} {card.name}"
    prefix = f"  {marker} "
    vis_right = render.visible_len(right_side)
    max_name_width = render.CONTENT_WIDTH - len(prefix) - 2 - vis_right
    if max_name_width < 1:
        max_name_width = 1

    display_name = transformed_name
    if len(display_name) > max_name_width:
        display_name = display_name[: max_name_width - 1] + "\u2026"
    colored_name = f"{name_color}{display_name}{render.RESET}"

    line1 = f"{prefix}{colored_name}"
    vis_line1 = render.visible_len(line1)
    gap = max(2, render.CONTENT_WIDTH - vis_line1 - vis_right)
    line1 = f"{line1}{' ' * gap}{right_side}"

    # Line 2: quoted rules text
    quoted = f'"{card.rules_text}"'
    if is_selected:
        line2 = f"    {quoted}"
    else:
        max_text_width = render.CONTENT_WIDTH - 4
        if len(quoted) > max_text_width:
            quoted = quoted[: max_text_width - 4] + '..."'
        line2 = f"    {quoted}"

    # Line 3: transfiguration type and effect
    type_label = _colored_type_label(best_type)
    note = _TRANSFIG_NOTES[best_type]
    line3 = f"    Transfiguration: {type_label}{render.DIM} -- {note}{render.RESET}"

    # Line 4: eligibility reason
    reason = eligibility_explanation(card, best_type)
    line4 = f"    {render.DIM}(Eligible: {reason}){render.RESET}"

    return "\n".join([line1, line2, line3, line4])


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
            dreamscape_name, is_enhanced,
        )
    else:
        _run_normal(
            state, eligible_deck_cards, logger,
            dreamscape_name, is_enhanced,
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
    dreamscape_name: str = "",
    is_enhanced: bool = False,
) -> None:
    """Normal transfiguration: pick from 3 random cards."""
    # Pre-filter to cards that have at least one applicable transfiguration
    # type, then sample from that filtered list to avoid showing fewer
    # than 3 candidates.
    transfig_eligible = [
        dc for dc in eligible_deck_cards
        if any(is_eligible(dc.card, t) for t in _BASE_TYPES)
    ]
    sample_size = min(3, len(transfig_eligible))
    sampled = state.rng.sample(transfig_eligible, sample_size)

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
                dreamscape=dreamscape_name,
                is_enhanced=is_enhanced,
                choices=[],
                choice_made=None,
                state_changes={},
                profile_snapshot=state.resonance_profile.snapshot(),
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
    option_labels.append("Skip transfiguration")

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
        type_label = _colored_type_label(transfig_type)
        print()
        print(
            f"  {render.BOLD}{dc.card.name}{render.RESET} transfigured "
            f"to {type_label}!"
        )
        print(
            f"  {render.DIM}Now: {transfig_type.value} {dc.card.name} "
            f"-- {_TRANSFIG_NOTES[transfig_type]}{render.RESET}"
        )
    else:
        print()
        print(f"  {render.DIM}Skipped transfiguration.{render.RESET}")

    if logger is not None:
        logger.log_site_visit(
            site_type="Transfiguration",
            dreamscape=dreamscape_name,
            is_enhanced=is_enhanced,
            choices=[f"{t.value} {dc.card.name}" for dc, t in candidates],
            choice_made=choice_name,
            state_changes={
                "transfigured_card": choice_name,
            },
            profile_snapshot=state.resonance_profile.snapshot(),
        )


def _run_enhanced(
    state: QuestState,
    eligible_deck_cards: list[DeckCard],
    logger: Optional[SessionLogger],
    dreamscape_name: str = "",
    is_enhanced: bool = True,
) -> None:
    """Enhanced transfiguration: choose any card from the full deck."""
    print(
        f"  {render.BOLD}Prismatic Biome{render.RESET} -- "
        f"choose any card from your deck to transfigure."
    )
    print()

    # Pre-compute best type for each card
    assigned_types: list[TransfigType] = []
    for dc in eligible_deck_cards:
        applicable = get_applicable_types(dc.card)
        if TransfigType.PRISMATIC in applicable:
            assigned_types.append(TransfigType.PRISMATIC)
        elif applicable:
            assigned_types.append(applicable[0])
        else:
            # Fallback to Viridian label (card will show "no types apply")
            assigned_types.append(TransfigType.VIRIDIAN)

    # Build option labels
    option_labels = [
        f"{assigned_types[i].value} {dc.card.name}"
        for i, dc in enumerate(eligible_deck_cards)
    ]
    option_labels.append("Skip transfiguration")

    def _render_card_item(
        index: int,
        option: str,
        is_selected: bool,
        _deck: list[DeckCard] = eligible_deck_cards,
        _types: list[TransfigType] = assigned_types,
    ) -> str:
        return _render_enhanced_item(
            index, option, is_selected, _deck, _types,
        )

    chosen_index = input_handler.single_select(
        options=option_labels,
        render_fn=_render_card_item,
    )

    choice_name: Optional[str] = None
    if chosen_index < len(eligible_deck_cards):
        dc = eligible_deck_cards[chosen_index]
        best_type = assigned_types[chosen_index]
        applicable = get_applicable_types(dc.card)

        if not applicable:
            print()
            print(
                f"  {render.DIM}No transfiguration types apply "
                f"to this card.{render.RESET}"
            )
            if logger is not None:
                logger.log_site_visit(
                    site_type="Transfiguration",
                    dreamscape=dreamscape_name,
                    is_enhanced=is_enhanced,
                    choices=[dc.card.name for dc in eligible_deck_cards],
                    choice_made=None,
                    state_changes={},
                    profile_snapshot=state.resonance_profile.snapshot(),
                )
            return

        note = _build_transfig_note(best_type, dc.card.name)
        dc.is_transfigured = True
        dc.transfig_note = note
        choice_name = note
        type_label = _colored_type_label(best_type)
        print()
        print(
            f"  {render.BOLD}{dc.card.name}{render.RESET} transfigured "
            f"to {type_label}!"
        )
        print(
            f"  {render.DIM}Now: {best_type.value} {dc.card.name} "
            f"-- {_TRANSFIG_NOTES[best_type]}{render.RESET}"
        )
    else:
        print()
        print(f"  {render.DIM}Skipped transfiguration.{render.RESET}")

    if logger is not None:
        logger.log_site_visit(
            site_type="Transfiguration",
            dreamscape=dreamscape_name,
            is_enhanced=is_enhanced,
            choices=[dc.card.name for dc in eligible_deck_cards],
            choice_made=choice_name,
            state_changes={
                "transfigured_card": choice_name,
            },
            profile_snapshot=state.resonance_profile.snapshot(),
        )
