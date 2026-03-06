"""Status display functions for the quest simulator.

Provides site header, battle-specific display formatting, and
victory screen. Imports shared ANSI constants and box-drawing
utilities from render.py.
"""

from typing import Optional

from models import Boss
from render import (
    BOLD,
    CONTENT_WIDTH,
    DIM,
    RESET,
    draw_double_separator,
    draw_separator,
)


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
        lines: list[str] = [
            sep,
            title,
            sep,
            "",
            f"    {BOLD}>> {boss_info.name} <<{RESET}",
            f"    Archetype: {boss_info.archetype}",
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
    return "\n".join(
        [
            sep,
            f"  {BOLD}VICTORY!{RESET}",
            sep,
        ]
    )


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
    deck_size: int,
    dreamsign_count: int,
    essence: int,
    log_path: Optional[str] = None,
) -> str:
    """Build the victory screen text."""
    sep = draw_double_separator()

    lines: list[str] = [
        sep,
        f"  {BOLD}QUEST COMPLETE -- VICTORY!{RESET}",
        sep,
        "",
        f"  Battles won: {battles_won}/{total_battles}",
        f"  Dreamscapes visited: {dreamscapes_visited}",
        f"  Dreamcaller: {dreamcaller_name}",
        "",
        f"  Final Deck: {deck_size} cards",
        "",
        f"  Dreamsigns: {dreamsign_count}",
        f"  Essence remaining: {essence}",
    ]

    if log_path is not None:
        lines.append("")
        lines.append(f"  {DIM}Log written to {log_path}{RESET}")

    lines.append(sep)
    return "\n".join(lines)
