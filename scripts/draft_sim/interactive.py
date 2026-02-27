"""Interactive mode for the draft simulation.

Steps through a quest pick-by-pick with colorful terminal output,
pausing after each pick for the user to press Enter.
"""

import os
import re
import sys
from typing import Optional

from jsonl_log import SessionLogger
from models import (
    PickContext,
    PickRecord,
    QuestResult,
    Rarity,
    Resonance,
    ResonanceProfile,
    StrategyParams,
)
from output import classify_deck, has_splash, convergence_pick

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
RESET = "\033[0m"

# Disable colors when NO_COLOR is set or output is not a terminal
if os.environ.get("NO_COLOR") or not sys.stdout.isatty():
    RESONANCE_COLORS = {r: "" for r in Resonance}
    NEUTRAL_COLOR = ""
    BOLD = ""
    DIM = ""
    RESET = ""


def visible_len(s: str) -> int:
    """Length of string excluding ANSI escape sequences."""
    return len(re.sub(r"\033\[[0-9;]*m", "", s))


def pad_right(s: str, width: int) -> str:
    """Pad string to visible width, accounting for ANSI escape sequences."""
    return s + " " * max(0, width - visible_len(s))


def draw_box(lines: list[str], min_width: int = 62):
    """Draw a double-line box around lines of text.

    Each line can contain ANSI escape sequences. The box auto-sizes to
    fit the longest line (with a minimum of `min_width` visible columns
    including the border characters). Content is left-aligned with
    2-space indent inside the borders.
    """
    max_content = max((visible_len(line) for line in lines), default=0)
    inner = max(min_width - 2, max_content + 4)  # 2-space indent + 2-space padding
    print(f"\u2554{'\u2550' * inner}\u2557")
    for line in lines:
        print(f"\u2551  {pad_right(line, inner - 2)}\u2551")
    print(f"\u255a{'\u2550' * inner}\u255d")


def truncate_visible(s: str, max_width: int) -> str:
    """Truncate a string (possibly with ANSI codes) to max visible width."""
    vlen = visible_len(s)
    if vlen <= max_width:
        return s
    # Walk through chars, tracking visible length
    result = []
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
    # Append RESET to close any open sequences
    result.append(RESET)
    return "".join(result)


def color_resonance(res: Resonance) -> str:
    """Return colored resonance name."""
    return f"{RESONANCE_COLORS[res]}{res.value}{RESET}"


def color_resonances(resonances: frozenset) -> str:
    """Format a frozenset of resonances as colored joined string."""
    if not resonances:
        return f"{NEUTRAL_COLOR}Neutral{RESET}"
    return "+".join(
        color_resonance(r)
        for r in sorted(resonances, key=lambda r: r.value)
    )


def rarity_label(rarity: Rarity) -> str:
    """Single character rarity label, bold for R/L."""
    char = rarity.value[0]
    if rarity in (Rarity.RARE, Rarity.LEGENDARY):
        return f"{BOLD}{char}{RESET}"
    return char


def card_color(card_resonances: frozenset) -> str:
    """Primary resonance color for bar coloring (first alphabetically, gray for neutral)."""
    if not card_resonances:
        return NEUTRAL_COLOR
    primary = min(card_resonances, key=lambda r: r.value)
    return RESONANCE_COLORS[primary]


def weight_bar(weight: float, max_weight: float, width: int = 9) -> str:
    """Proportional bar of filled/empty blocks."""
    if max_weight <= 0:
        filled = 0
    else:
        filled = round(weight / max_weight * width)
    filled = max(0, min(width, filled))
    return "\u2588" * filled + "\u2591" * (width - filled)


def _build_card_lines(
    card,
    weight: float,
    max_weight: float,
    is_highlighted: bool,
    fit: float,
    card_width: int = 19,
) -> list[str]:
    """Build 6 lines for a single card display."""
    inner = card_width - 2
    cc = card_color(card.resonances)
    rar = rarity_label(card.rarity)

    # Line 1: resonance + rarity tag
    res_display = color_resonances(card.resonances)
    rar_display = f"[{rar}]"
    res_vis = visible_len(res_display)
    rar_vis = visible_len(rar_display)
    avail = inner - 2 - rar_vis
    if res_vis > avail:
        line1 = f" {truncate_visible(res_display, inner - 1 - rar_vis)}{rar_display}"
    else:
        space_between = inner - 1 - res_vis - rar_vis
        line1 = f" {res_display}{' ' * space_between}{rar_display}"

    # Line 2: power
    line2 = f"  pow: {card.power:2d}"

    # Line 3: weight bar
    bar = weight_bar(weight, max_weight)
    line3 = f" {cc}{bar}{RESET}"

    # Line 4: weight + fit
    line4 = f" wt:{weight:5.1f} f:{fit:.0f}"

    # Border colors
    sorted_res = sorted(card.resonances, key=lambda r: r.value) if card.resonances else []
    if len(sorted_res) >= 2:
        lc = RESONANCE_COLORS[sorted_res[0]]
        rc = RESONANCE_COLORS[sorted_res[1]]
    elif len(sorted_res) == 1:
        lc = rc = RESONANCE_COLORS[sorted_res[0]]
    else:
        lc = rc = NEUTRAL_COLOR

    half = inner // 2
    if is_highlighted:
        top = f"{lc}\u2554{'\u2550' * half}{RESET}{rc}{'\u2550' * (inner - half)}\u2557{RESET}"
        bot = f"{lc}\u255a{'\u2550' * half}{RESET}{rc}{'\u2550' * (inner - half)}\u255d{RESET}"
        lbr = f"{lc}\u2551{RESET}"
        rbr = f"{rc}\u2551{RESET}"
    else:
        top = f"{lc}\u250c{'\u2500' * half}{RESET}{rc}{'\u2500' * (inner - half)}\u2510{RESET}"
        bot = f"{lc}\u2514{'\u2500' * half}{RESET}{rc}{'\u2500' * (inner - half)}\u2518{RESET}"
        lbr = f"{lc}\u2502{RESET}"
        rbr = f"{rc}\u2502{RESET}"

    return [
        top,
        f"{lbr}{pad_right(line1, inner)}{rbr}",
        f"{lbr}{pad_right(line2, inner)}{rbr}",
        f"{lbr}{pad_right(line3, inner)}{rbr}",
        f"{lbr}{pad_right(line4, inner)}{rbr}",
        bot,
    ]


def render_cards(pick: PickRecord, strat_params: StrategyParams) -> str:
    """Build side-by-side ASCII card art and arrow line for a draft pick."""
    cards = pick.offered
    weights = pick.weights
    picked_id = pick.picked.id
    max_w = max(weights) if weights else 1.0
    card_width = 19
    gap = "  "

    # Compute fit for each card using pre-pick profile
    profile = ResonanceProfile()
    for r, c in pick.profile_after.items():
        profile.counts[r] = c
    for r in pick.picked.resonances:
        profile.counts[r] = max(0, profile.counts.get(r, 0) - 1)
    top2_res = {r for r, _ in profile.top_n(2)}

    def resonance_fit(card_res: frozenset) -> float:
        if not card_res:
            return 0.5
        matching = len(card_res & top2_res)
        return matching / len(card_res)

    # Build 6 lines per card
    all_lines: list[list[str]] = [[] for _ in range(6)]
    picked_idx = None

    for i, (card, w) in enumerate(zip(cards, weights)):
        is_picked = card.id == picked_id
        if is_picked:
            picked_idx = i
        fit = resonance_fit(card.resonances)
        lines = _build_card_lines(card, w, max_w, is_picked, fit, card_width)
        for row_idx, line in enumerate(lines):
            all_lines[row_idx].append(line)

    # Join lines horizontally
    output_lines = []
    for row in all_lines:
        output_lines.append(gap.join(row))

    # Arrow pointing to picked card
    if picked_idx is not None:
        card_center = picked_idx * (card_width + 2) + card_width // 2
        output_lines.append(" " * card_center + "\u25b2")
        output_lines.append(
            " " * card_center + f"\u2514\u2500\u2500 {pick.pick_reason}"
        )

    return "\n".join(output_lines)


def render_shop(pick: PickRecord, strat_params: StrategyParams) -> str:
    """Build 2-row x 3-column shop display with bought cards highlighted."""
    cards = pick.offered
    weights = pick.weights
    bought_ids = {c.id for c in (pick.bought or [])}
    max_w = max(weights) if weights else 1.0
    card_width = 19
    gap = "  "

    # Compute fit using pre-buy profile
    profile = ResonanceProfile()
    for r, c in pick.profile_after.items():
        profile.counts[r] = c
    # Subtract bought cards to get pre-buy profile
    for bought_card in (pick.bought or []):
        for r in bought_card.resonances:
            profile.counts[r] = max(0, profile.counts.get(r, 0) - 1)
    top2_res = {r for r, _ in profile.top_n(2)}

    def resonance_fit(card_res: frozenset) -> float:
        if not card_res:
            return 0.5
        return len(card_res & top2_res) / len(card_res)

    output_lines = []

    # Render in 2 rows of 3
    for row_start in range(0, len(cards), 3):
        row_cards = list(zip(
            cards[row_start:row_start + 3],
            weights[row_start:row_start + 3],
        ))

        all_lines: list[list[str]] = [[] for _ in range(6)]
        for card, w in row_cards:
            is_bought = card.id in bought_ids
            fit = resonance_fit(card.resonances)
            lines = _build_card_lines(card, w, max_w, is_bought, fit, card_width)
            for row_idx, line in enumerate(lines):
                all_lines[row_idx].append(line)

        for row in all_lines:
            output_lines.append(gap.join(row))

        # Buy/skip markers under each card
        markers = []
        for card, _ in row_cards:
            if card.id in bought_ids:
                label = f"  {BOLD}\u2713 BOUGHT{RESET}"
            else:
                label = f"  {DIM}  skipped{RESET}"
            markers.append(pad_right(label, card_width))
        output_lines.append(gap.join(markers))
        output_lines.append("")

    # Buy reasons summary
    if pick.buy_reasons:
        output_lines.append(f"  Bought {len(pick.bought or [])} of {len(cards)} cards:")
        for reason in (pick.buy_reasons or []):
            output_lines.append(f"    {reason}")

    return "\n".join(output_lines)


def profile_bar(profile_snapshot: dict, bar_width: int = 20) -> str:
    """Colored bar chart for all 5 resonances, sorted by count descending."""
    items = sorted(profile_snapshot.items(), key=lambda x: x[1], reverse=True)
    total = sum(c for _, c in items)
    max_count = max((c for _, c in items), default=1)
    lines = []
    for res, count in items:
        color = RESONANCE_COLORS.get(res, NEUTRAL_COLOR)
        if count == 0:
            bar = f"{DIM}{'\u2591' * bar_width}{RESET}"
            pct_str = f"{DIM}  0   (0.0%){RESET}"
        else:
            filled = round(count / max_count * bar_width) if max_count > 0 else 0
            filled = max(1, min(bar_width, filled))
            bar = f"{color}{'\u2588' * filled}{'\u2591' * (bar_width - filled)}{RESET}"
            pct = count / total * 100 if total > 0 else 0
            pct_str = f"{count:3d}  ({pct:4.1f}%)"
        name = f"{color}{res.value:8s}{RESET}"
        lines.append(f"    {name} {bar}  {pct_str}")
    return "\n".join(lines)


def stats_line(profile_snapshot: dict) -> str:
    """HHI / top-2 / eff.colors / class summary line."""
    profile = ResonanceProfile()
    for r, c in profile_snapshot.items():
        profile.counts[r] = c
    top2 = profile.top2_share()
    hhi = profile.hhi()
    eff = profile.effective_colors()
    cls = classify_deck(profile)
    return f"  Top-2: {top2:.1%}  |  HHI: {hhi:.3f}  |  Eff.Colors: {eff:.2f}  |  Class: {cls}"


def print_quest_banner(result: QuestResult, strategy_name: str, strat_params: StrategyParams):
    """Print the quest start banner."""
    dc = color_resonances(result.dreamcaller_resonances)
    # Compute starting bonus from first pick's profile minus picked card
    bonus_parts = []
    for r in sorted(result.dreamcaller_resonances, key=lambda r: r.value):
        if result.picks:
            first_profile = result.picks[0].profile_after
            bonus = first_profile.get(r, 0)
            if result.picks[0].context.is_shop:
                for card in (result.picks[0].bought or []):
                    if r in card.resonances:
                        bonus -= 1
            elif r in result.picks[0].picked.resonances:
                bonus -= 1
            bonus_parts.append(f"{color_resonance(r)}={bonus}")

    strat_detail = f"{strategy_name} (pow={strat_params.power_weight}, fit={strat_params.fit_weight})"
    bonus_str = ", ".join(bonus_parts) if bonus_parts else "none"

    # Format pool variance as percentage bias per resonance
    bias_parts = []
    for r in sorted(result.pool_variance.keys(), key=lambda r: r.value):
        mult = result.pool_variance[r]
        pct = (mult - 1.0) * 100
        sign = "+" if pct >= 0 else ""
        bias_parts.append(f"{color_resonance(r)} {sign}{pct:.0f}%")
    bias_str = ", ".join(bias_parts)

    banner_lines = [
        f"{BOLD}QUEST START{RESET}",
        f"Dreamcaller: {dc}",
        f"Strategy: {strat_detail}",
        f"Starting bonus: {bonus_str}",
        f"Pool bias: {bias_str}",
    ]
    if result.shop_count > 0:
        banner_lines.append(f"Shops: {result.shop_count} site(s) replaced drafts")
    draw_box(banner_lines)
    print()


def print_pick_header(ctx: PickContext, pick_num: int, total_picks: int):
    """Print header for a draft site pick."""
    ds = ctx.dreamscape
    site = ctx.site
    pos = ctx.position
    w = 70

    if pos == 0:
        print(f"\n{'\u2550' * w}")
    else:
        print(f"\n{'\u2500' * w}")

    site_label = f"Draft Site {site + 1}"
    header = f"  DREAMSCAPE {ds} \u2500\u2500 {site_label} \u2500\u2500 Pick {pos + 1}/5"
    pick_tag = f"[Pick {pick_num}/{total_picks}]"
    spacing = max(1, w - len(header) - len(pick_tag))
    print(f"{header}{' ' * spacing}{pick_tag}")

    if pos == 0:
        print(f"{'\u2550' * w}")
    else:
        print(f"{'\u2500' * w}")


def print_shop_header(ctx: PickContext, pick_num: int, total_picks: int):
    """Print header for a shop site."""
    ds = ctx.dreamscape
    site = ctx.site
    w = 70
    print(f"\n{'\u2550' * w}")
    header = f"  DREAMSCAPE {ds} \u2500\u2500 Shop (Site {site + 1})"
    pick_tag = f"[Pick {pick_num}/{total_picks}]"
    spacing = max(1, w - len(header) - len(pick_tag))
    print(f"{header}{' ' * spacing}{pick_tag}")
    print(f"{'\u2550' * w}")


def print_battle_reward_header(ctx: PickContext, pick_num: int, total_picks: int):
    """Print header for a battle reward pick."""
    ds = ctx.dreamscape
    w = 70
    print(f"\n{'\u2550' * w}")
    header = f"  DREAMSCAPE {ds} \u2500\u2500 Battle Reward (Rare+)"
    pick_tag = f"[Pick {pick_num}/{total_picks}]"
    spacing = max(1, w - len(header) - len(pick_tag))
    print(f"{header}{' ' * spacing}{pick_tag}")
    print(f"{'\u2550' * w}")


def print_profile(profile_snapshot: dict):
    """Print the resonance profile bar chart."""
    print(f"\n  Resonance Profile:")
    print(profile_bar(profile_snapshot))


def print_stats(profile_snapshot: dict):
    """Print the summary stats line."""
    print(stats_line(profile_snapshot))


def print_final_summary(result: QuestResult, strategy_name: str):
    """Print end-of-quest summary."""
    total = len(result.picks)
    print()
    draw_box([f"{BOLD}QUEST COMPLETE{RESET} \u2500\u2500 {total} cards drafted"])

    # Final resonance profile
    print(f"\n  Final Resonance Profile:")
    print(profile_bar(result.final_profile.snapshot()))
    print()

    # Deck statistics
    profile = result.final_profile
    t2 = profile.top2_share()
    hhi = profile.hhi()
    eff = profile.effective_colors()
    cls = classify_deck(profile)
    splash = has_splash(profile)
    print(f"  Deck Statistics:")
    print(f"    Total cards:    {len(result.deck)}")
    print(f"    Top-2 share:    {t2:.1%}")
    print(f"    HHI:            {hhi:.3f}")
    print(f"    Eff. Colors:    {eff:.2f}")
    print(f"    Classification: {cls}")
    print(f"    Splash:         {'yes' if splash else 'no'}")
    if result.shop_count > 0:
        print(f"    Shops visited:  {result.shop_count}")
    print()

    # Rarity breakdown
    rarity_counts: dict[Rarity, int] = {r: 0 for r in Rarity}
    for card in result.deck:
        rarity_counts[card.rarity] += 1
    deck_total = len(result.deck)
    print(f"  Rarity Breakdown:")
    for rarity in [Rarity.COMMON, Rarity.UNCOMMON, Rarity.RARE, Rarity.LEGENDARY]:
        c = rarity_counts[rarity]
        pct = c / deck_total * 100 if deck_total > 0 else 0
        print(f"    {rarity.value:10s}: {c:2d} ({pct:4.1f}%)")
    print()

    # Convergence
    conv = convergence_pick(result)
    print(f"  Convergence: Top-2 share exceeded 75% at pick {conv}")
    print()


def wait_for_input():
    """Prompt user to press Enter, handle Ctrl+C."""
    try:
        input(f"\n  {DIM}Press Enter to continue...{RESET}")
    except (KeyboardInterrupt, EOFError):
        print(f"\n\n  {DIM}Quest abandoned.{RESET}\n")
        sys.exit(0)


def run_interactive(
    result: QuestResult,
    strategy_name: str,
    strat_params: StrategyParams,
    logger: Optional[SessionLogger] = None,
):
    """Main orchestrator for interactive mode."""
    if not sys.stdout.isatty():
        print("Error: interactive mode requires a terminal", file=sys.stderr)
        sys.exit(1)

    total_picks = len(result.picks)

    print_quest_banner(result, strategy_name, strat_params)
    wait_for_input()

    for pick in result.picks:
        ctx = pick.context

        if ctx.is_battle_reward:
            print_battle_reward_header(ctx, pick.pick_number, total_picks)
        elif ctx.is_shop:
            print_shop_header(ctx, pick.pick_number, total_picks)
        else:
            print_pick_header(ctx, pick.pick_number, total_picks)

        print()
        if ctx.is_shop:
            print(render_shop(pick, strat_params))
        else:
            print(render_cards(pick, strat_params))
        print_profile(pick.profile_after)
        print_stats(pick.profile_after)

        if logger:
            logger.log_pick(pick, ctx)

        wait_for_input()

    if logger:
        logger.log_session_end(result)
        logger.close()
        print(f"  {DIM}Log written to {logger.path}{RESET}\n")

    print_final_summary(result, strategy_name)
