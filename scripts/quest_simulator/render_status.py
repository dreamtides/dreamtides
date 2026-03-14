"""Status display functions for the quest simulator.

Provides site header, battle-specific display formatting, archetype
preference footer, and victory screen. Uses AYU Mirage palette colors
from the draft simulator.
"""

from typing import Optional

import colors
from models import Boss
from render import (
    ARCHETYPE_NAMES,
    CONTENT_WIDTH,
    draw_double_separator,
    draw_separator,
    visible_len,
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
        title = f"  {colors.header(f'BATTLE {battle_number} -- {encounter_label}')}"
        lines: list[str] = [
            sep,
            title,
            sep,
            "",
            f"    {colors.c(f'>> {boss_info.name} <<', 'accent', bold=True)}",
            f"    Archetype: {boss_info.archetype}",
            f'    "{boss_info.ability_text}"',
            "",
            sep,
        ]
    else:
        title = f"  {colors.header(f'BATTLE {battle_number}/{total_battles}')}"
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
            f"  {colors.header('VICTORY!')}",
            sep,
        ]
    )


def battle_reward_summary(
    essence_reward: int,
) -> str:
    """Build the reward summary display showing essence gained."""
    lines: list[str] = [
        f"  Essence reward: {colors.c(f'+{essence_reward}', 'accent', bold=True)}",
        "",
        f"  {colors.section('Battle Reward')} -- Choose a card:",
    ]
    return "\n".join(lines)


def battle_completion_progress(
    new_completion: int,
    total_battles: int,
) -> str:
    """Build the completion progress display shown after battle."""
    return f"  {colors.header(f'Completion: {new_completion}/{total_battles}')}"


def archetype_preference_footer(
    w: list[float],
    deck_count: int,
    essence: int,
    archetype_draft: bool = False,
) -> str:
    """Build the archetype preference footer.

    Shows the top 2-3 archetype preferences by weight, deck count,
    and essence. Called from site modules after each interaction.
    In archetype draft mode, only the deck/essence status line is shown.
    """
    sep = draw_separator()
    lines: list[str] = [sep]

    if not archetype_draft:
        # Find top 3 archetypes by weight
        indexed = sorted(enumerate(w), key=lambda x: x[1], reverse=True)
        top_n = min(3, len(indexed))
        top = indexed[:top_n]

        total = sum(w)

        pref_parts: list[str] = []
        for idx, weight in top:
            name = ARCHETYPE_NAMES[idx] if idx < len(ARCHETYPE_NAMES) else f"A{idx}"
            if total > 0:
                pct = weight / total * 100
                bar_len = int(pct / 5)
                bar = "\u2588" * bar_len
                pref_parts.append(
                    f"    {colors.label(name)}: {colors.num(f'{pct:.0f}%')} {colors.c(bar, 'accent')}"
                )
            else:
                pref_parts.append(f"    {colors.label(name)}: {colors.num('0%')}")

        lines.append(f"  {colors.section('Archetype Preferences')}")
        lines.extend(pref_parts)

    status = (
        f"  Deck: {colors.num(deck_count)} cards  |  " f"Essence: {colors.num(essence)}"
    )
    lines.append(status)
    lines.append(sep)
    return "\n".join(lines)


def victory_screen(
    battles_won: int,
    total_battles: int,
    dreamscapes_visited: int,
    dreamcaller_name: str,
    deck_size: int,
    dreamsign_count: int,
    essence: int,
    w: Optional[list[float]] = None,
    log_path: Optional[str] = None,
    archetype_draft: bool = False,
) -> str:
    """Build the victory screen text.

    Shows quest completion stats and archetype preference visualization.
    In archetype draft mode, the archetype preferences section is omitted.
    """
    sep = draw_double_separator()

    lines: list[str] = [
        sep,
        f"  {colors.header('QUEST COMPLETE -- VICTORY!')}",
        sep,
        "",
        f"  Battles won: {colors.num(f'{battles_won}/{total_battles}')}",
        f"  Dreamscapes visited: {colors.num(dreamscapes_visited)}",
        f"  Dreamcaller: {dreamcaller_name}",
        "",
        f"  Final Deck: {colors.num(deck_size)} cards",
        "",
        f"  Dreamsigns: {colors.num(dreamsign_count)}",
        f"  Essence remaining: {colors.num(essence)}",
    ]

    # Archetype preference visualization (skip in archetype draft mode)
    if w is not None and not archetype_draft:
        lines.append("")
        lines.append(f"  {colors.section('Archetype Preferences')}")
        total = sum(w)
        indexed = sorted(enumerate(w), key=lambda x: x[1], reverse=True)
        for idx, weight in indexed:
            if weight <= 0:
                continue
            name = ARCHETYPE_NAMES[idx] if idx < len(ARCHETYPE_NAMES) else f"A{idx}"
            if total > 0:
                pct = weight / total * 100
                bar_len = int(pct / 5)
                bar = "\u2588" * bar_len
                lines.append(
                    f"    {colors.label(name)}: {colors.num(f'{pct:.0f}%')} {colors.c(bar, 'accent')}"
                )

    if log_path is not None:
        lines.append("")
        lines.append(f"  {colors.dim(f'Log written to {log_path}')}")

    lines.append(sep)
    return "\n".join(lines)
