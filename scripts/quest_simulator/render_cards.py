"""Card display formatting for the quest simulator.

Provides the standard 2-line card display format, card list rendering,
shop card display with pricing, deck summary display, and draft-specific
card display with weight bars and resonance match indicators. Used by
all site types that display cards (drafts, shops, rewards, purge,
duplication, transfiguration).
"""

from typing import Optional, Union

from models import Card, DeckCard, Rarity, Resonance

import render

MATCH_COLOR = "\033[92m"
PARTIAL_COLOR = "\033[93m"
OFF_COLOR_DIM = "\033[2m"

if render.RESET == "":
    MATCH_COLOR = ""
    PARTIAL_COLOR = ""
    OFF_COLOR_DIM = ""


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


def _format_grid_cell(
    card: Card,
    price: int,
    original_price: Optional[int],
    col_width: int,
) -> list[str]:
    """Format a single card for the shop grid.

    Returns a fixed number of lines (4) for vertical alignment:
    Line 1: name (colored by resonance, truncated)
    Line 2: resonance display + rarity badge
    Line 3: price (with discount if applicable)
    Line 4: truncated rules text
    """
    name_color = render.card_color(card.resonances)
    max_name = col_width
    name = card.name
    if len(name) > max_name:
        name = name[: max_name - 1] + "\u2026"
    colored_name = f"{name_color}{name}{render.RESET}"
    line1 = render.pad_right(colored_name, col_width)

    res_str = render.color_resonances(card.resonances)
    badge = render.rarity_badge(card.rarity)
    res_badge = f"{res_str} {badge}"
    line2 = render.pad_right(res_badge, col_width)

    if original_price is not None and original_price != price:
        price_str = (
            f"{render.DIM}{render.STRIKETHROUGH}{original_price}e"
            f"{render.RESET} "
            f"{render.BOLD}{price}e{render.RESET}"
        )
    else:
        price_str = f"{price}e"
    line3 = render.pad_right(price_str, col_width)

    # Truncated rules text
    rules = card.rules_text
    if len(rules) > col_width:
        rules = rules[: col_width - 3] + "..."
    line4 = render.pad_right(f"{render.DIM}{rules}{render.RESET}", col_width)

    return [line1, line2, line3, line4]


def render_shop_grid(
    items: list[tuple[Card, int, Optional[int]]],
    col_width: int = 21,
    gap: str = "  ",
) -> str:
    """Render shop items in a 2x3 grid layout.

    Each item is a tuple of (card, effective_price, original_price_or_None).
    Items are arranged in rows of 3. Each cell shows the card name (colored
    by resonance), rarity badge, resonance display, price (with discount
    highlighting), and truncated rules text.

    Returns a single string with newlines.
    """
    if not items:
        return ""

    lines_per_cell = 4
    output_lines: list[str] = []

    for row_start in range(0, len(items), 3):
        row_items = items[row_start : row_start + 3]
        cells: list[list[str]] = []
        for card, price, original_price in row_items:
            cells.append(_format_grid_cell(card, price, original_price, col_width))

        # Pad rows with fewer than 3 items
        while len(cells) < 3:
            cells.append([render.pad_right("", col_width)] * lines_per_cell)

        for line_idx in range(lines_per_cell):
            joined = gap.join(cells[col][line_idx] for col in range(len(cells)))
            output_lines.append(joined)

        # Blank line between rows
        if row_start + 3 < len(items):
            output_lines.append("")

    return "\n".join(output_lines)


def weight_bar(weight: float, max_weight: float, width: int = 10) -> str:
    """Proportional bar of filled/empty block characters."""
    if max_weight <= 0:
        filled = 0
    else:
        filled = round(weight / max_weight * width)
    filled = max(0, min(width, filled))
    return "\u2588" * filled + "\u2591" * (width - filled)


def resonance_match_indicator(
    card_resonances: frozenset[Resonance],
    top_resonances: frozenset[Resonance],
) -> str:
    """Return a colored match label based on resonance overlap.

    Returns "match" (green) if all card resonances are in the player's
    top resonances, "partial" (yellow) if some overlap, or "off-color"
    (dim) if no overlap or the card is neutral.
    """
    if not card_resonances or not top_resonances:
        return f"{OFF_COLOR_DIM}off-color{render.RESET}"

    overlap = card_resonances & top_resonances
    if overlap == card_resonances:
        return f"{MATCH_COLOR}match{render.RESET}"
    elif overlap:
        return f"{PARTIAL_COLOR}partial{render.RESET}"
    else:
        return f"{OFF_COLOR_DIM}off-color{render.RESET}"


def format_draft_card(
    card: Card,
    weight: float,
    max_weight: float,
    highlighted: bool,
    top_resonances: frozenset[Resonance],
    max_width: int = render.CONTENT_WIDTH,
) -> list[str]:
    """Format a card for draft display with weight bar and match indicator.

    Extends the standard 2-line card display with a third line showing
    a resonance-colored weight bar, numeric weight value, and match
    indicator.
    """
    lines = format_card_display(card, highlighted=highlighted, max_width=max_width)

    bar = weight_bar(weight, max_weight)
    bar_color = render.card_color(card.resonances)
    match_label = resonance_match_indicator(card.resonances, top_resonances)

    weight_line = f"    {bar_color}{bar}{render.RESET}  wt: {weight:.1f}  {match_label}"
    lines.append(weight_line)

    return lines


def render_draft_card_list(
    cards: list[Card],
    selected_index: int,
    weights: list[float],
    top_resonances: frozenset[Resonance],
) -> str:
    """Render a list of draft cards with weight bars and match indicators.

    Each card is rendered using format_draft_card. Returns a single
    string with all cards separated by newlines.
    """
    if not cards:
        return ""

    max_weight = max(weights) if weights else 0.0
    result_lines: list[str] = []
    for i, card in enumerate(cards):
        is_highlighted = i == selected_index
        w = weights[i] if i < len(weights) else 0.0
        card_lines = format_draft_card(
            card,
            weight=w,
            max_weight=max_weight,
            highlighted=is_highlighted,
            top_resonances=top_resonances,
        )
        result_lines.extend(card_lines)

    return "\n".join(result_lines)



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
