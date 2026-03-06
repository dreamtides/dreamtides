"""Card display formatting for the quest simulator.

Provides card display formatting for DeckCards and the full deck
viewer. Used by all site types that display cards.
"""

from typing import Optional

from models import DeckCard, Dreamcaller, Dreamsign

import render


def format_card_display(
    deck_card: DeckCard,
    highlighted: bool = False,
    max_width: int = render.CONTENT_WIDTH,
) -> list[str]:
    """Format a DeckCard as 2 display lines.

    Line 1: marker + card name
    Line 2: card details (if available from the instance)

    If the card is a DeckCard with a transfig_note, the note is shown
    after the name on line 1.
    """
    marker = ">" if highlighted else " "
    instance = deck_card.instance

    # Extract name from instance if available
    if hasattr(instance, "design") and hasattr(instance.design, "name"):
        card_name = instance.design.name
    elif hasattr(instance, "name"):
        card_name = instance.name
    else:
        card_name = str(instance) if instance is not None else "<unknown>"

    if deck_card.transfig_note is not None:
        display_name = deck_card.transfig_note
    else:
        display_name = card_name

    prefix = f"  {marker} "
    max_name_width = max_width - len(prefix) - 2
    if max_name_width < 1:
        max_name_width = 1
    if len(display_name) > max_name_width:
        display_name = display_name[: max_name_width - 1] + "\u2026"

    line1 = f"{prefix}{display_name}"

    # Line 2: card details
    details_parts: list[str] = []
    if hasattr(instance, "design"):
        design = instance.design
        if hasattr(design, "power"):
            details_parts.append(f"Power: {design.power}")
        if hasattr(design, "commit"):
            details_parts.append(f"Commit: {design.commit}")
    detail_str = "  ".join(details_parts) if details_parts else ""
    line2 = f"    {detail_str}" if detail_str else "    "

    return [line1, line2]


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
    header_left = f"  {render.BOLD}DECK VIEWER{render.RESET}"
    header_right = f"Deck: {total} cards"
    vis_left = render.visible_len(header_left)
    gap = max(2, render.CONTENT_WIDTH - vis_left - len(header_right))
    lines.append(f"{header_left}{' ' * gap}{header_right}")

    if essence is not None:
        lines.append(f"  Essence: {essence}")

    lines.append(sep)
    lines.append("")

    # Card list sorted by name
    sorted_deck = sorted(deck_cards, key=_deck_card_sort_key)

    for dc in sorted_deck:
        card_lines = format_card_display(dc, highlighted=False)
        if dc.is_bane:
            card_lines[0] = card_lines[0] + f"  {render.BOLD}[BANE]{render.RESET}"
        lines.extend(card_lines)

    lines.append("")
    lines.append(single_sep)

    # Dreamsigns section
    if dreamsigns:
        lines.append("")
        lines.append(f"  {render.BOLD}Dreamsigns{render.RESET} ({len(dreamsigns)})")
        for ds in dreamsigns:
            bane_label = f" {render.BOLD}[BANE]{render.RESET}" if ds.is_bane else ""
            lines.append(f"    {ds.name}{bane_label}")

    # Dreamcaller section
    if dreamcaller is not None:
        lines.append("")
        lines.append(single_sep)
        lines.append(f"  {render.BOLD}Dreamcaller{render.RESET}")
        lines.append(f"    {dreamcaller.name}")
        lines.append(f'    "{dreamcaller.ability_text}"')

    lines.append("")
    lines.append(sep)

    return "\n".join(lines)
