"""Card display formatting for the quest simulator.

Provides the standard 2-line card display format, card list rendering,
shop card display with pricing, and deck summary display. Used by all
site types that display cards (drafts, shops, rewards, purge,
duplication, transfiguration).
"""

from typing import Optional, Union

from models import Card, DeckCard, Rarity, Resonance

import render


def format_card_display(
    card_or_deck_card: Union[Card, DeckCard],
    highlighted: bool = False,
    max_width: int = render.CONTENT_WIDTH,
) -> list[str]:
    """Format a card (or DeckCard) as 2 display lines.

    Line 1: marker + name (colored by primary resonance) + resonance
            display + rarity badge + cost + spark
    Line 2: quoted rules text (full if highlighted, truncated otherwise)

    If the card is a DeckCard with a transfig_note, the note is shown
    after the name on line 1.
    """
    if isinstance(card_or_deck_card, DeckCard):
        card = card_or_deck_card.card
        transfig_note = card_or_deck_card.transfig_note
    else:
        card = card_or_deck_card
        transfig_note = None

    marker = ">" if highlighted else " "
    name_color = render.card_color(card.resonances)

    res_str = render.color_resonances(card.resonances)
    badge = render.rarity_badge(card.rarity)

    cost_str = (
        f"Cost: {card.energy_cost}"
        if card.energy_cost is not None
        else "Cost: -"
    )
    spark_str = f"Spark: {card.spark}" if card.spark is not None else ""

    # Build the right side: resonance badge cost spark
    right_parts = [res_str, badge, cost_str]
    if spark_str:
        right_parts.append(spark_str)
    right_side = "   ".join(right_parts)

    prefix = f"  {marker} "
    vis_right = render.visible_len(right_side)

    # Display name: use transfig_note if present, otherwise card name
    if transfig_note is not None:
        display_name = transfig_note
    else:
        display_name = card.name

    # Maximum visible width available for the name (2 gap minimum)
    max_name_width = max_width - len(prefix) - 2 - vis_right
    if max_name_width < 1:
        max_name_width = 1
    if len(display_name) > max_name_width:
        display_name = display_name[: max_name_width - 1] + "\u2026"
    colored_name = f"{name_color}{display_name}{render.RESET}"

    line1 = f"{prefix}{colored_name}"
    vis_line1 = render.visible_len(line1)
    gap = max(2, max_width - vis_line1 - vis_right)
    line1 = f"{line1}{' ' * gap}{right_side}"

    # Line 2: quoted rules text
    quoted = f'"{card.rules_text}"'
    if highlighted:
        line2 = f"    {quoted}"
    else:
        max_text_width = max_width - 4  # 4 chars indent
        if len(quoted) > max_text_width:
            quoted = quoted[: max_text_width - 4] + '..."'
        line2 = f"    {quoted}"

    return [line1, line2]


def render_card_list(
    cards: list[Card],
    selected_index: int,
    weights: Optional[list[float]] = None,
) -> str:
    """Render a list of cards, highlighting the selected card.

    Each card is rendered using format_card_display. If weights are
    provided, they are displayed alongside each card.

    Returns a single string with all cards separated by newlines.
    """
    if not cards:
        return ""

    result_lines: list[str] = []
    for i, card in enumerate(cards):
        is_highlighted = i == selected_index
        lines = format_card_display(card, highlighted=is_highlighted)
        if weights is not None and i < len(weights):
            weight_str = f"    {render.DIM}Weight: {weights[i]:.2f}{render.RESET}"
            lines.append(weight_str)
        result_lines.extend(lines)

    return "\n".join(result_lines)


def format_shop_card(
    card: Card,
    price: int,
    highlighted: bool = False,
    original_price: Optional[int] = None,
) -> list[str]:
    """Format a card for shop display with price information.

    Extends the standard 2-line card display with a third line showing
    the price. If original_price is provided (indicating a discount),
    the original price is shown struck through and the discounted price
    is highlighted.
    """
    lines = format_card_display(card, highlighted=highlighted)

    if original_price is not None and original_price != price:
        price_line = (
            f"    Price: "
            f"{render.DIM}{render.STRIKETHROUGH}{original_price}e"
            f"{render.RESET} "
            f"{render.BOLD}{price}e{render.RESET}"
        )
    else:
        price_line = f"    Price: {price}e"

    lines.append(price_line)
    return lines


def format_deck_summary(deck_cards: list[DeckCard]) -> str:
    """Produce a compact deck summary with card count and rarity breakdown.

    Returns a multi-line string showing total card count and a per-rarity
    breakdown with counts and percentages.
    """
    total = len(deck_cards)

    rarity_counts: dict[Rarity, int] = {r: 0 for r in Rarity}
    for dc in deck_cards:
        rarity_counts[dc.card.rarity] = rarity_counts.get(dc.card.rarity, 0) + 1

    lines: list[str] = [f"  Deck: {total} cards"]

    for rarity in [Rarity.COMMON, Rarity.UNCOMMON, Rarity.RARE, Rarity.LEGENDARY]:
        count = rarity_counts.get(rarity, 0)
        pct = count / total * 100 if total > 0 else 0
        lines.append(f"    {rarity.value:12s} {count:2d} ({pct:4.1f}%)")

    return "\n".join(lines)
