"""Terminal display formatting for the quest simulator.

Provides ANSI color constants, box-drawing utilities, card display
formatting, resonance formatting helpers, and header/footer/banner
templates. Follows the patterns from scripts/draft_sim/interactive.py.
"""

import os
import re
import sys
from typing import Optional

from models import Card, Rarity, Resonance

CONTENT_WIDTH = 70

# ANSI color codes per resonance
RESONANCE_COLORS: dict[Resonance, str] = {
    Resonance.TIDE: "\033[94m",
    Resonance.EMBER: "\033[91m",
    Resonance.ZEPHYR: "\033[92m",
    Resonance.STONE: "\033[93m",
    Resonance.RUIN: "\033[95m",
}
NEUTRAL_COLOR = "\033[90m"
BOLD = "\033[1m"
DIM = "\033[2m"
STRIKETHROUGH = "\033[9m"
RESET = "\033[0m"

# Disable colors when NO_COLOR is set or output is not a terminal
if os.environ.get("NO_COLOR") or not sys.stdout.isatty():
    RESONANCE_COLORS = {r: "" for r in Resonance}
    NEUTRAL_COLOR = ""
    BOLD = ""
    DIM = ""
    STRIKETHROUGH = ""
    RESET = ""


def visible_len(s: str) -> int:
    """Length of string excluding ANSI escape sequences."""
    return len(re.sub(r"\033\[[0-9;]*m", "", s))


def pad_right(s: str, width: int) -> str:
    """Pad string to visible width, accounting for ANSI escape sequences."""
    return s + " " * max(0, width - visible_len(s))


def truncate_visible(s: str, max_width: int) -> str:
    """Truncate a string (possibly with ANSI codes) to max visible width."""
    vlen = visible_len(s)
    if vlen <= max_width:
        return s
    result: list[str] = []
    vis = 0
    in_escape = False
    for ch in s:
        if in_escape:
            result.append(ch)
            if ch == "m":
                in_escape = False
            continue
        if ch == "\033":
            in_escape = True
            result.append(ch)
            continue
        if vis >= max_width:
            break
        result.append(ch)
        vis += 1
    result.append(RESET)
    return "".join(result)


def draw_box(lines: list[str], min_width: int = 70) -> None:
    """Draw a double-line box around lines of text.

    Each line can contain ANSI escape sequences. The box auto-sizes to
    fit the longest line (with a minimum of `min_width` visible columns
    including the border characters). Content is left-aligned with
    2-space indent inside the borders.
    """
    max_content = max((visible_len(line) for line in lines), default=0)
    inner = max(min_width - 2, max_content + 4)
    print(f"\u2554{'\u2550' * inner}\u2557")
    for line in lines:
        print(f"\u2551  {pad_right(line, inner - 2)}\u2551")
    print(f"\u255a{'\u2550' * inner}\u255d")


def draw_separator() -> str:
    """Full-width single-line separator."""
    return "\u2500" * CONTENT_WIDTH


def draw_double_separator() -> str:
    """Full-width double-line separator."""
    return "\u2550" * CONTENT_WIDTH


def color_resonance(res: Resonance) -> str:
    """Return colored resonance name."""
    return f"{RESONANCE_COLORS[res]}{res.value}{RESET}"


def color_resonances(resonances: frozenset[Resonance]) -> str:
    """Format a frozenset of resonances as colored joined string."""
    if not resonances:
        return f"{NEUTRAL_COLOR}Neutral{RESET}"
    return "+".join(
        color_resonance(r) for r in sorted(resonances, key=lambda r: r.value)
    )


def rarity_badge(rarity: Rarity) -> str:
    """Single-character rarity badge: [C], [U], [R], [L]."""
    char = rarity.value[0]
    if rarity in (Rarity.RARE, Rarity.LEGENDARY):
        return f"{BOLD}[{char}]{RESET}"
    return f"[{char}]"


def card_color(card_resonances: frozenset[Resonance]) -> str:
    """Primary resonance color (first alphabetically, gray for neutral)."""
    if not card_resonances:
        return NEUTRAL_COLOR
    primary = min(card_resonances, key=lambda r: r.value)
    return RESONANCE_COLORS[primary]


def format_card(card: Card, highlighted: bool = False) -> list[str]:
    """Format a card as 2 display lines.

    Line 1: marker + name (colored) + resonance + rarity badge + cost + spark
    Line 2: quoted rules text (truncated if not highlighted)
    """
    marker = ">" if highlighted else " "
    name_color = card_color(card.resonances)

    res_str = color_resonances(card.resonances)
    badge = rarity_badge(card.rarity)

    cost_str = (
        f"Cost: {card.energy_cost}" if card.energy_cost is not None else "Cost: -"
    )
    spark_str = f"Spark: {card.spark}" if card.spark is not None else ""

    # Build the right side: resonance badge cost spark
    right_parts = [res_str, badge, cost_str]
    if spark_str:
        right_parts.append(spark_str)
    right_side = "   ".join(right_parts)

    prefix = f"  {marker} "
    vis_right = visible_len(right_side)
    # Maximum visible width available for the name (2 gap minimum)
    max_name_width = CONTENT_WIDTH - len(prefix) - 2 - vis_right
    if max_name_width < 1:
        max_name_width = 1
    display_name = card.name
    if len(display_name) > max_name_width:
        display_name = display_name[: max_name_width - 1] + "\u2026"
    colored_name = f"{name_color}{display_name}{RESET}"

    line1 = f"{prefix}{colored_name}"
    vis_line1 = visible_len(line1)
    gap = max(2, CONTENT_WIDTH - vis_line1 - vis_right)
    line1 = f"{line1}{' ' * gap}{right_side}"

    # Line 2: quoted rules text
    quoted = f'"{card.rules_text}"'
    if highlighted:
        line2 = f"    {quoted}"
    else:
        max_text_width = CONTENT_WIDTH - 4  # 4 chars indent
        if len(quoted) > max_text_width:
            quoted = quoted[: max_text_width - 4] + '..."'
        line2 = f"    {quoted}"

    return [line1, line2]


def profile_bar(
    profile_snapshot: dict[Resonance, int],
    bar_width: int = 20,
    neutral_count: int = 0,
) -> str:
    """Colored bar chart for all 5 resonances, sorted by count descending.

    When neutral_count is provided, it is included in the percentage
    denominator so that resonance percentages and the neutral percentage
    together sum to 100%.
    """
    items = sorted(profile_snapshot.items(), key=lambda x: x[1], reverse=True)
    total = sum(c for _, c in items) + neutral_count
    max_count = max((c for _, c in items), default=1)
    lines: list[str] = []
    for res, count in items:
        color = RESONANCE_COLORS.get(res, NEUTRAL_COLOR)
        if count == 0:
            bar = " " * bar_width
            pct_str = f"{DIM}  0   (0.0%){RESET}"
        else:
            filled = round(count / max_count * bar_width) if max_count > 0 else 0
            filled = max(1, min(bar_width, filled))
            bar = f"{color}{'\u2588' * filled}{RESET}{' ' * (bar_width - filled)}"
            pct = count / total * 100 if total > 0 else 0
            pct_str = f"{count:3d}  ({pct:4.1f}%)"
        name = f"{color}{res.value:8s}{RESET}"
        lines.append(f"    {name} {bar}  {pct_str}")
    return "\n".join(lines)


def quest_start_banner(
    seed: int,
    starting_essence: int,
    pool_size: int,
    unique_cards: int = 0,
    pool_variance: Optional[dict[Resonance, float]] = None,
    rarity_entries: Optional[dict[Rarity, int]] = None,
    algorithm_params_str: Optional[str] = None,
) -> str:
    """Build the quest start banner text.

    When pool_variance and rarity_entries are provided, shows pool bias
    per resonance and composition breakdown by rarity. When
    algorithm_params_str is provided, shows the active algorithm
    parameters (for CLI override visibility).
    """
    from render_status import pool_bias_line, pool_composition_summary

    sep = draw_double_separator()
    lines: list[str] = [
        sep,
        f"  {BOLD}DREAMTIDES QUEST{RESET}{' ' * (CONTENT_WIDTH - 18 - len(f'Seed: {seed}') - 2)}Seed: {seed}",
        sep,
        "",
        f"  Starting essence: {starting_essence}",
    ]

    if pool_variance is not None and rarity_entries is not None:
        lines.append(
            pool_composition_summary(
                unique_cards=unique_cards,
                total_entries=pool_size,
                rarity_entries=rarity_entries,
            )
        )
        lines.append(pool_bias_line(pool_variance))
    elif unique_cards > 0:
        lines.append(f"  Draft pool: {unique_cards} cards ({pool_size} entries)")
    else:
        lines.append(f"  Draft pool: {pool_size} entries")

    if algorithm_params_str is not None:
        lines.append(algorithm_params_str)

    lines.append("")
    lines.append(f"  {DIM}Press Enter to begin...{RESET}")
    lines.append(sep)
    return "\n".join(lines)


def atlas_header(
    essence: int,
    completion: int,
    total_battles: int,
    deck_count: int,
    dreamsign_count: int,
) -> str:
    """Build the atlas header text."""
    sep = draw_double_separator()
    left1 = f"  {BOLD}DREAM ATLAS{RESET}"
    right1 = f"Essence: {essence}"
    vis_left1 = visible_len(left1)
    gap1 = max(2, CONTENT_WIDTH - vis_left1 - len(right1))
    line1 = f"{left1}{' ' * gap1}{right1}"

    left2 = f"  Completion: {completion}/{total_battles}"
    right2 = f"Deck: {deck_count} cards | Dreamsigns: {dreamsign_count}"
    gap2 = max(2, CONTENT_WIDTH - len(left2) - len(right2))
    line2 = f"{left2}{' ' * gap2}{right2}"

    return "\n".join([sep, line1, line2, sep])


def site_visit_header(
    dreamscape_name: str,
    site_type_label: str,
    pick_info: str,
    dreamscape_number: int,
) -> str:
    """Build a site visit header."""
    sep = draw_double_separator()
    left = f"  {dreamscape_name.upper()} -- {site_type_label} -- {pick_info}"
    right = f"[Dreamscape {dreamscape_number}]"
    gap = max(2, CONTENT_WIDTH - len(left) - len(right))
    return "\n".join([sep, f"{left}{' ' * gap}{right}", sep])


def resonance_profile_footer(
    counts: dict[Resonance, int],
    deck_count: int,
    essence: int,
) -> str:
    """Build the resonance profile footer text."""
    single_sep = draw_separator()
    double_sep = draw_double_separator()

    res_parts: list[str] = []
    for r in Resonance:
        c = counts.get(r, 0)
        colored = f"{color_resonance(r)} {c}"
        res_parts.append(colored)
    res_line = f"  Resonance: {' | '.join(res_parts)}"
    deck_line = f"  Deck: {deck_count} cards | Essence: {essence}"

    return "\n".join([single_sep, res_line, deck_line, double_sep])


def victory_screen(
    battles_won: int,
    total_battles: int,
    dreamscapes_visited: int,
    dreamcaller_name: str,
    dreamcaller_resonances: frozenset[Resonance],
    deck_size: int,
    rarity_counts: dict[Rarity, int],
    resonance_counts: dict[Resonance, int],
    neutral_count: int,
    dreamsign_count: int,
    essence: int,
    log_path: Optional[str] = None,
) -> str:
    """Build the victory screen text."""
    sep = draw_double_separator()
    dc_res = color_resonances(dreamcaller_resonances)

    lines: list[str] = [
        sep,
        f"  {BOLD}QUEST COMPLETE -- VICTORY!{RESET}",
        sep,
        "",
        f"  Battles won: {battles_won}/{total_battles}",
        f"  Dreamscapes visited: {dreamscapes_visited}",
        f"  Dreamcaller: {dreamcaller_name} ({dc_res})",
        "",
        f"  Final Deck: {deck_size} cards",
    ]

    # Rarity breakdown with colons and aligned columns
    for rarity in [Rarity.COMMON, Rarity.UNCOMMON, Rarity.RARE, Rarity.LEGENDARY]:
        c = rarity_counts.get(rarity, 0)
        pct = c / deck_size * 100 if deck_size > 0 else 0
        label = f"{rarity.value}:"
        lines.append(f"    {label:12s} {c:2d} ({pct:4.1f}%)")

    lines.append("")
    lines.append("  Resonance Profile:")
    lines.append(profile_bar(resonance_counts, neutral_count=neutral_count))

    # Neutral count line
    total_res = sum(resonance_counts.values()) + neutral_count
    neutral_pct = neutral_count / total_res * 100 if total_res > 0 else 0
    neutral_label = f"{NEUTRAL_COLOR}{'Neutral':8s}{RESET}"
    neutral_space = " " * 20
    lines.append(
        f"    {neutral_label} {neutral_space}  {neutral_count:3d}  ({neutral_pct:4.1f}%)"
    )

    lines.append("")
    lines.append(f"  Dreamsigns: {dreamsign_count}")
    lines.append(f"  Essence remaining: {essence}")

    if log_path is not None:
        lines.append("")
        lines.append(f"  {DIM}Log written to {log_path}{RESET}")

    lines.append(sep)
    return "\n".join(lines)
