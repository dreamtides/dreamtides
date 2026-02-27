"""Interactive mode for the draft simulation.

Steps through a quest pick-by-pick with colorful terminal output,
pausing after each pick for the user to press Enter.
"""

import os
import re
import sys

from models import Rarity, Resonance, ResonanceProfile, QuestResult, PickRecord, StrategyParams
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


def render_cards(pick: PickRecord, strat_params: StrategyParams) -> str:
    """Build side-by-side ASCII card art and arrow line."""
    cards = pick.offered
    weights = pick.weights
    picked_id = pick.picked.id
    max_w = max(weights) if weights else 1.0
    card_width = 19
    inner = card_width - 2  # 17
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
        cc = card_color(card.resonances)
        fit = resonance_fit(card.resonances)

        rar = rarity_label(card.rarity)

        # Line 1: resonance + rarity tag — fit into `inner` visible chars
        res_display = color_resonances(card.resonances)
        rar_display = f"[{rar}]"
        res_vis = visible_len(res_display)
        rar_vis = visible_len(rar_display)
        # " {res} {tag} " or " {res}{tag} " if tight
        avail = inner - 2 - rar_vis  # space for leading+trailing space and rarity
        if res_vis > avail:
            # Tight: no space between res and tag
            line1 = f" {truncate_visible(res_display, inner - 1 - rar_vis)}{rar_display}"
        else:
            space_between = inner - 1 - res_vis - rar_vis
            line1 = f" {res_display}{' ' * space_between}{rar_display}"

        # Line 2: power
        line2 = f"  pow: {card.power:2d}"

        # Line 3: weight bar (colored per resonance)
        bar = weight_bar(w, max_w)
        line3 = f" {cc}{bar}{RESET}"

        # Line 4: weight + fit
        line4 = f" wt:{w:5.1f} f:{fit:.0f}"

        # Build border characters
        if is_picked:
            bc = cc
            top = f"{bc}\u2554{'\u2550' * inner}\u2557{RESET}"
            bot = f"{bc}\u255a{'\u2550' * inner}\u255d{RESET}"
            lbr = f"{bc}\u2551{RESET}"
            rbr = f"{bc}\u2551{RESET}"
        else:
            top = f"\u250c{'\u2500' * inner}\u2510"
            bot = f"\u2514{'\u2500' * inner}\u2518"
            lbr = "\u2502"
            rbr = "\u2502"

        all_lines[0].append(top)
        all_lines[1].append(f"{lbr}{pad_right(line1, inner)}{rbr}")
        all_lines[2].append(f"{lbr}{pad_right(line2, inner)}{rbr}")
        all_lines[3].append(f"{lbr}{pad_right(line3, inner)}{rbr}")
        all_lines[4].append(f"{lbr}{pad_right(line4, inner)}{rbr}")
        all_lines[5].append(bot)

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


def build_pick_context() -> list[dict]:
    """Build a static context list mapping pick numbers to dreamscape/site/position.

    Quest structure:
    DS 0-1: 2 sites of 5 picks + 1 battle reward = 11 each (22 total)
    DS 2-3: 1 site of 5 picks + 1 battle reward = 6 each (12 total)
    DS 4-6: 1 battle reward each (3 total)
    Total: 37 picks
    """
    contexts = []
    pick = 0
    for ds in range(7):
        if ds <= 1:
            num_sites = 2
        elif ds <= 3:
            num_sites = 1
        else:
            num_sites = 0

        for site in range(num_sites):
            for pos in range(5):
                pick += 1
                contexts.append({
                    "pick": pick,
                    "dreamscape": ds,
                    "site": site,
                    "position": pos,
                    "is_battle_reward": False,
                })

        # Battle reward
        pick += 1
        contexts.append({
            "pick": pick,
            "dreamscape": ds,
            "site": None,
            "position": None,
            "is_battle_reward": True,
        })

    return contexts


def print_quest_banner(result: QuestResult, strategy_name: str, strat_params: StrategyParams):
    """Print the quest start banner."""
    dc = color_resonances(result.dreamcaller_resonances)
    # Compute starting bonus from first pick's profile minus picked card
    bonus_parts = []
    for r in sorted(result.dreamcaller_resonances, key=lambda r: r.value):
        if result.picks:
            first_profile = result.picks[0].profile_after
            bonus = first_profile.get(r, 0)
            if r in result.picks[0].picked.resonances:
                bonus -= 1
            bonus_parts.append(f"{color_resonance(r)}={bonus}")

    strat_detail = f"{strategy_name} (pow={strat_params.power_weight}, fit={strat_params.fit_weight})"
    bonus_str = ", ".join(bonus_parts) if bonus_parts else "none"

    w = 62
    print(f"\u2554{'\u2550' * w}\u2557")
    line1 = f"{BOLD}QUEST START{RESET}"
    print(f"\u2551  {pad_right(line1, w - 2 + len(line1) - visible_len(line1))}\u2551")
    dc_line = f"Dreamcaller: {dc}"
    print(f"\u2551  {pad_right(dc_line, w - 2 + len(dc_line) - visible_len(dc_line))}\u2551")
    print(f"\u2551  {pad_right(f'Strategy: {strat_detail}', w - 2)}\u2551")
    bonus_line = f"Starting bonus: {bonus_str}"
    print(f"\u2551  {pad_right(bonus_line, w - 2 + len(bonus_line) - visible_len(bonus_line))}\u2551")
    print(f"\u255a{'\u2550' * w}\u255d")
    print()


def print_pick_header(ctx: dict, total_picks: int):
    """Print header for a draft site pick."""
    ds = ctx["dreamscape"]
    site = ctx["site"]
    pos = ctx["position"]
    pick_num = ctx["pick"]
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


def print_battle_reward_header(ctx: dict, total_picks: int):
    """Print header for a battle reward pick."""
    ds = ctx["dreamscape"]
    pick_num = ctx["pick"]
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
    w = 62
    header = f"{BOLD}QUEST COMPLETE{RESET} \u2500\u2500 {total} cards drafted"
    print(f"\n\u2554{'\u2550' * w}\u2557")
    print(f"\u2551  {pad_right(header, w - 2 + len(header) - visible_len(header))}\u2551")
    print(f"\u255a{'\u2550' * w}\u255d")

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


def run_interactive(result: QuestResult, strategy_name: str, strat_params: StrategyParams):
    """Main orchestrator for interactive mode."""
    if not sys.stdout.isatty():
        print("Error: interactive mode requires a terminal", file=sys.stderr)
        sys.exit(1)

    contexts = build_pick_context()
    ctx_by_pick = {c["pick"]: c for c in contexts}
    total_picks = len(result.picks)

    print_quest_banner(result, strategy_name, strat_params)
    wait_for_input()

    for pick in result.picks:
        ctx = ctx_by_pick.get(pick.pick_number)
        if ctx is None:
            continue

        if ctx["is_battle_reward"]:
            print_battle_reward_header(ctx, total_picks)
        else:
            print_pick_header(ctx, total_picks)

        print()
        print(render_cards(pick, strat_params))
        print_profile(pick.profile_after)
        print_stats(pick.profile_after)
        wait_for_input()

    print_final_summary(result, strategy_name)
