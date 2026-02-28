"""Dreamcaller Draft site interaction.

Presents 3 random dreamcallers for the player to choose from via
arrow-key single-select. The selected dreamcaller's resonance bonuses,
tag bonuses, and essence bonus are applied to quest state.
"""

import random
from typing import Optional

from input_handler import single_select
from jsonl_log import SessionLogger
from models import Dreamcaller, Resonance
from quest_state import QuestState
from render import (
    BOLD,
    RESET,
    color_resonance,
    color_resonances,
    draw_double_separator,
    resonance_profile_footer,
)

DREAMCALLER_COUNT = 3


def select_dreamcallers(
    all_dreamcallers: list[Dreamcaller],
    rng: random.Random,
) -> list[Dreamcaller]:
    """Select up to 3 random dreamcallers from the full list."""
    count = min(DREAMCALLER_COUNT, len(all_dreamcallers))
    return rng.sample(all_dreamcallers, count)


def format_dreamcaller_option(
    dc: Dreamcaller,
    highlighted: bool = False,
) -> list[str]:
    """Format a dreamcaller for display in the selection menu.

    Returns a list of display lines showing the dreamcaller's name,
    resonances, ability text, essence bonus, resonance bonuses, and
    tag bonuses.
    """
    marker = ">" if highlighted else " "
    res_str = color_resonances(dc.resonances)

    # Line 1: marker + name + resonance symbols
    line1 = f"  {marker} {BOLD}{dc.name}{RESET}  {res_str}"

    # Line 2: ability text
    line2 = f"      \"{dc.ability_text}\""

    # Line 3: essence bonus + resonance bonus values
    res_bonus_parts: list[str] = []
    for res_name, amount in sorted(dc.resonance_bonus.items()):
        try:
            res_enum = Resonance(res_name)
            res_bonus_parts.append(f"{color_resonance(res_enum)}+{amount}")
        except ValueError:
            res_bonus_parts.append(f"{res_name}+{amount}")
    res_bonus_str = ", ".join(res_bonus_parts) if res_bonus_parts else "none"
    line3 = f"      Essence: +{dc.essence_bonus}  |  Resonance: {res_bonus_str}"

    # Line 4: tag bonus values
    tag_parts: list[str] = []
    for tag, amount in sorted(dc.tag_bonus.items()):
        tag_parts.append(f"{tag}+{amount}")
    tag_str = ", ".join(tag_parts) if tag_parts else "none"
    line4 = f"      Tags: {tag_str}"

    return [line1, line2, line3, line4]


def apply_dreamcaller(state: QuestState, dc: Dreamcaller) -> None:
    """Apply a dreamcaller's bonuses to quest state."""
    state.set_dreamcaller(dc)


def run_dreamcaller_draft(
    state: QuestState,
    all_dreamcallers: list[Dreamcaller],
    logger: Optional[SessionLogger] = None,
    dreamscape_name: str = "",
    dreamscape_number: int = 1,
) -> None:
    """Run the Dreamcaller Draft site interaction.

    Selects 3 random dreamcallers, displays them for the player to
    choose via arrow-key navigation, applies bonuses, logs the
    selection, and shows the resonance profile footer.
    """
    choices = select_dreamcallers(all_dreamcallers, state.rng)

    # Display header
    sep = draw_double_separator()
    print(sep)
    left = f"  {dreamscape_name.upper()} -- Dreamcaller Draft"
    right = f"[Dreamscape {dreamscape_number}]"
    gap = max(2, 70 - len(left) - len(right))
    print(f"{left}{' ' * gap}{right}")
    print(sep)
    print()
    print(f"  {BOLD}Choose your Dreamcaller:{RESET}")
    print()

    # Build display lines for each dreamcaller
    option_labels = [dc.name for dc in choices]

    def render_fn(index: int, option: str, is_selected: bool) -> str:
        dc = choices[index]
        lines = format_dreamcaller_option(dc, highlighted=is_selected)
        return "\n".join(lines)

    selected_index = single_select(option_labels, render_fn=render_fn)
    selected = choices[selected_index]

    # Apply selection
    apply_dreamcaller(state, selected)

    # Display confirmation
    print()
    res_str = color_resonances(selected.resonances)
    print(f"  {BOLD}Selected:{RESET} {selected.name} ({res_str})")
    print(f"  Essence: +{selected.essence_bonus} -> {state.essence}")

    # Log the selection
    if logger is not None:
        logger.log_site_visit(
            site_type="DreamcallerDraft",
            choices=[dc.name for dc in choices],
            choice_made=selected.name,
            state_changes={
                "dreamcaller": selected.name,
                "essence_delta": selected.essence_bonus,
                "resonance_bonus": dict(selected.resonance_bonus),
                "tag_bonus": dict(selected.tag_bonus),
            },
        )

    # Show resonance profile footer
    print()
    footer = resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)
