"""Status display functions for the quest simulator.

Provides the resonance profile footer, resonance bar chart, victory
screen, site header, and battle-specific display formatting. Imports
shared ANSI constants and box-drawing utilities from render.py.
"""

from typing import Optional

from models import Boss, Rarity, Resonance
from render import (
    BOLD,
    CONTENT_WIDTH,
    DIM,
    NEUTRAL_COLOR,
    RESET,
    RESONANCE_COLORS,
    color_resonance,
    color_resonances,
    draw_double_separator,
    draw_separator,
)


def resonance_profile_footer(
    counts: dict[Resonance, int],
    deck_count: int,
    essence: int,
) -> str:
    """Build the resonance profile footer shown after each site interaction."""
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


def site_header(
    dreamscape_name: str,
    site_type_label: str,
    dreamscape_number: int,
    pick_info: Optional[str] = None,
) -> str:
    """Build a site visit header with optional pick counter."""
    sep = draw_double_separator()
    left_parts = [f"  {dreamscape_name.upper()}", site_type_label]
    if pick_info is not None:
        left_parts.append(pick_info)
    left = " -- ".join(left_parts)
    right = f"[Dreamscape {dreamscape_number}]"
    gap = max(2, CONTENT_WIDTH - len(left) - len(right))
    return "\n".join([sep, f"{left}{' ' * gap}{right}", sep])


def battle_header(
    battle_number: int,
    total_battles: int,
    boss_info: Optional[Boss],
) -> str:
    """Build a dramatic battle header display.

    For boss encounters (miniboss or final boss), shows a full dramatic
    introduction with name, archetype, and ability text inside a
    double-line separator box. For Dream Guardian battles, shows a
    shorter display with just the name and level indicator.
    """
    sep = draw_double_separator()

    if boss_info is not None:
        if boss_info.is_final:
            encounter_label = "FINAL BOSS"
        else:
            encounter_label = "MINIBOSS ENCOUNTER"
        title = f"  {BOLD}BATTLE {battle_number} -- {encounter_label}{RESET}"
        res_str = color_resonances(boss_info.resonances)
        lines: list[str] = [
            sep,
            title,
            sep,
            "",
            f"    {BOLD}>> {boss_info.name} <<{RESET}",
            f"    Archetype: {boss_info.archetype}  ({res_str})",
            f'    "{boss_info.ability_text}"',
            "",
            sep,
        ]
    else:
        title = f"  {BOLD}BATTLE {battle_number}/{total_battles}{RESET}"
        lines = [
            sep,
            title,
            "",
            f"    Dream Guardian  (Level {battle_number})",
            sep,
        ]

    return "\n".join(lines)


def battle_victory_message() -> str:
    """Build a visually distinct victory message with box-drawing flourish."""
    sep = draw_double_separator()
    return "\n".join([
        sep,
        f"  {BOLD}VICTORY!{RESET}",
        sep,
    ])


def battle_reward_summary(
    essence_reward: int,
    rare_pick_count: int,
) -> str:
    """Build the reward summary display showing essence gained and rare pick framing."""
    lines: list[str] = [
        f"  Essence reward: {BOLD}+{essence_reward}{RESET}",
        "",
        f"  {BOLD}Battle Reward{RESET} -- Choose a rare card:",
    ]
    return "\n".join(lines)


def battle_completion_progress(
    new_completion: int,
    total_battles: int,
) -> str:
    """Build the completion progress display shown after battle."""
    return f"  {BOLD}Completion: {new_completion}/{total_battles}{RESET}"


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
    # Format: "    Common:     12 (35.3%)"
    # The label+colon is left-padded, count is right-aligned
    for rarity in [Rarity.COMMON, Rarity.UNCOMMON, Rarity.RARE, Rarity.LEGENDARY]:
        c = rarity_counts.get(rarity, 0)
        pct = c / deck_size * 100 if deck_size > 0 else 0
        label = f"{rarity.value}:"
        lines.append(f"    {label:12s} {c:2d} ({pct:4.1f}%)")

    lines.append("")
    lines.append("  Resonance Profile:")
    lines.append(profile_bar(resonance_counts, neutral_count=neutral_count))

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
