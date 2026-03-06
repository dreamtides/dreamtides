"""Card display formatting for the quest simulator.

Provides card display formatting for DeckCards and the full deck
viewer. Uses AYU Mirage palette colors from the draft simulator.
"""

import textwrap
from typing import Optional

import colors
from models import DeckCard, Dreamcaller, Dreamsign

import render

ARCHETYPE_NAMES = render.ARCHETYPE_NAMES


def _top_archetype(fitness: list[float]) -> str:
    """Return a label for the card's top archetype fitness value."""
    if not fitness:
        return ""
    best_idx = 0
    best_val = fitness[0]
    for i, v in enumerate(fitness):
        if v > best_val:
            best_val = v
            best_idx = i
    name = (
        ARCHETYPE_NAMES[best_idx] if best_idx < len(ARCHETYPE_NAMES) else f"A{best_idx}"
    )
    return f"{name}={best_val:.2f}"


def format_card_display(
    card_or_deck_card,
    highlighted: bool = False,
    max_width: int = render.CONTENT_WIDTH,
) -> list[str]:
    """Format a card as display lines.

    Accepts a DeckCard, CardInstance, or CardDesign. Shows the card
    name (colored), power, commit, flex, and top archetype fitness.

    For real cards (is_real=True), returns up to 4+ lines:
      Line 1: marker + colored card name (with transfig/bane markers)
      Line 2: energy cost + card type + subtype + rarity
      Line 3+: rules text (word-wrapped)
      Last:   power/commit/flex + top archetype
    For synthetic cards, returns 2 lines (name + stats).
    """
    marker = ">" if highlighted else " "

    # Unwrap to get the design and deck-card metadata
    is_bane = False
    transfig_note = None
    design = None

    if isinstance(card_or_deck_card, DeckCard):
        is_bane = card_or_deck_card.is_bane
        transfig_note = card_or_deck_card.transfig_note
        instance = card_or_deck_card.instance
        if hasattr(instance, "design"):
            design = instance.design
        else:
            design = instance
    elif hasattr(card_or_deck_card, "design"):
        # CardInstance
        design = card_or_deck_card.design
    else:
        # CardDesign directly
        design = card_or_deck_card

    # Extract name
    if design is not None and hasattr(design, "name"):
        card_name = design.name
    else:
        card_name = str(card_or_deck_card)

    # Build display name
    if transfig_note is not None:
        display_name = transfig_note
    else:
        display_name = card_name

    prefix = f"  {marker} "
    max_name_width = max_width - len(prefix) - 2
    if max_name_width < 1:
        max_name_width = 1
    if len(display_name) > max_name_width:
        display_name = display_name[: max_name_width - 1] + "\u2026"

    colored_name = colors.card(display_name)
    bane_marker = f"  {colors.c('[BANE]', 'error', bold=True)}" if is_bane else ""
    line1 = f"{prefix}{colored_name}{bane_marker}"

    lines = [line1]

    # For real cards, add type line and rules text
    is_real = getattr(design, "is_real", False) if design else False
    if is_real and design is not None:
        # Line 2: energy cost + card type + subtype + rarity
        type_parts: list[str] = []
        energy = getattr(design, "energy_cost", None)
        if energy is not None:
            type_parts.append(f"{energy}E")
        ct = getattr(design, "card_type", "")
        if ct:
            type_parts.append(ct)
        sub = getattr(design, "subtype", "")
        if sub:
            type_parts.append(f"- {sub}")
        rarity = getattr(design, "rarity", "")
        if rarity:
            type_parts.append(f" ({rarity.title()})")
        type_line = " ".join(type_parts)
        lines.append(f"    {colors.dim(type_line)}")

        # Line 3+: rules text
        rules = getattr(design, "rules_text", "")
        if rules:
            wrap_width = max_width - 4
            wrapped = textwrap.wrap(rules, width=wrap_width)
            for wline in wrapped:
                lines.append(f"    {wline}")

    # Stats line: power/commit/flex + top archetype
    details_parts: list[str] = []
    if design is not None:
        if hasattr(design, "power"):
            details_parts.append(f"Power: {colors.num(f'{design.power:.2f}')}")
        if hasattr(design, "commit"):
            details_parts.append(f"Commit: {colors.num(f'{design.commit:.2f}')}")
        if hasattr(design, "flex"):
            details_parts.append(f"Flex: {colors.num(f'{design.flex:.2f}')}")
        if hasattr(design, "fitness"):
            arch = _top_archetype(design.fitness)
            if arch:
                details_parts.append(colors.dim(arch))

    detail_str = "  ".join(details_parts) if details_parts else ""
    lines.append(f"    {detail_str}" if detail_str else "    ")

    return lines


def card_name(card) -> str:
    """Extract the display name from a CardInstance or CardDesign."""
    if hasattr(card, "design") and hasattr(card.design, "name"):
        return card.design.name
    if hasattr(card, "name"):
        return card.name
    return str(card)


def _deck_card_sort_key(dc: DeckCard) -> tuple[str, str]:
    """Sort key for deck cards: card name."""
    instance = dc.instance
    if hasattr(instance, "design") and hasattr(instance.design, "name"):
        return ("", instance.design.name)
    if hasattr(instance, "name"):
        return ("", instance.name)
    return ("\xff", "")


def render_full_deck_view(
    deck_cards: list[DeckCard],
    dreamsigns: Optional[list[Dreamsign]] = None,
    dreamcaller: Optional[Dreamcaller] = None,
    essence: Optional[int] = None,
) -> str:
    """Render a full deck viewer display.

    Shows all cards sorted by name, with dreamsigns, dreamcaller info,
    and essence.
    """
    sep = render.draw_double_separator()
    single_sep = render.draw_separator()
    total = len(deck_cards)
    lines: list[str] = [sep]

    # Header
    header_left = f"  {colors.header('DECK VIEWER')}"
    header_right = f"Deck: {total} cards"
    vis_left = render.visible_len(header_left)
    gap = max(2, render.CONTENT_WIDTH - vis_left - len(header_right))
    lines.append(f"{header_left}{' ' * gap}{header_right}")

    if essence is not None:
        lines.append(f"  Essence: {colors.num(essence)}")

    lines.append(sep)
    lines.append("")

    # Card list sorted by name
    sorted_deck = sorted(deck_cards, key=_deck_card_sort_key)

    for dc in sorted_deck:
        card_lines = format_card_display(dc, highlighted=False)
        lines.extend(card_lines)

    lines.append("")
    lines.append(single_sep)

    # Dreamsigns section
    if dreamsigns:
        lines.append("")
        lines.append(f"  {colors.section('Dreamsigns')} ({len(dreamsigns)})")
        for ds in dreamsigns:
            bane_label = (
                f" {colors.c('[BANE]', 'error', bold=True)}" if ds.is_bane else ""
            )
            lines.append(f"    {ds.name}{bane_label}")

    # Dreamcaller section
    if dreamcaller is not None:
        lines.append("")
        lines.append(single_sep)
        lines.append(f"  {colors.section('Dreamcaller')}")
        lines.append(f"    {dreamcaller.name}")
        lines.append(f'    "{dreamcaller.ability_text}"')

    lines.append("")
    lines.append(sep)

    return "\n".join(lines)
