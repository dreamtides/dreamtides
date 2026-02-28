"""Dreamcaller Draft site interaction.

Presents 3 random dreamcallers for the player to choose from via
arrow-key single-select. The selected dreamcaller's resonance bonuses,
tag bonuses, and essence bonus are applied to quest state.

The highlighted option shows full details (resonance, resonance bonus,
tags, essence bonus, ability text) while non-highlighted options show
a condensed view (name + resonance only) to reduce visual clutter.
"""

import random
from typing import Optional

import render
import render_status
from input_handler import single_select
from jsonl_log import SessionLogger
from models import Dreamcaller, Resonance
from quest_state import QuestState

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

    When highlighted, returns a detailed multi-line display with
    resonance, resonance bonus, tags, essence bonus, and ability text.
    When not highlighted, returns a condensed single-line display with
    name and resonance only.
    """
    marker = ">" if highlighted else " "
    res_str = render.color_resonances(dc.resonances)

    if not highlighted:
        # Condensed view: name + resonance only
        return [f"  {marker} {dc.name}  ({res_str})"]

    # Full detail view for the highlighted option
    lines: list[str] = []

    # Line 1: marker + bold name
    lines.append(f"  {marker} {render.BOLD}{dc.name}{render.RESET}")

    # Line 2: Resonance symbols
    lines.append(f"      Resonance: {res_str}")

    # Line 3: Resonance bonus values
    res_bonus_parts: list[str] = []
    for res_name, amount in sorted(dc.resonance_bonus.items()):
        try:
            res_enum = Resonance(res_name)
            res_bonus_parts.append(f"{render.color_resonance(res_enum)} +{amount}")
        except ValueError:
            res_bonus_parts.append(f"{res_name} +{amount}")
    res_bonus_str = ", ".join(res_bonus_parts) if res_bonus_parts else "none"
    lines.append(f"      Resonance Bonus: {res_bonus_str}")

    # Line 4: Tags
    tag_parts = sorted(dc.tags)
    tag_str = ", ".join(tag_parts) if tag_parts else "none"
    lines.append(f"      Tags: {tag_str}")

    # Line 5: Essence bonus (bold)
    lines.append(f"      Essence Bonus: {render.BOLD}+{dc.essence_bonus}{render.RESET}")

    # Line 6: Quoted ability text (dimmed)
    lines.append(f'      {render.DIM}"{dc.ability_text}"{render.RESET}')

    return lines


def apply_dreamcaller(state: QuestState, dc: Dreamcaller) -> None:
    """Apply a dreamcaller's bonuses to quest state."""
    state.set_dreamcaller(dc)


def format_confirmation(
    dc: Dreamcaller,
    essence_after: int,
) -> str:
    """Build the confirmation message after dreamcaller selection.

    Shows the selected dreamcaller name, resonance, applied bonuses
    (resonance, tag, essence), and resulting essence total.
    """
    res_str = render.color_resonances(dc.resonances)
    lines: list[str] = [
        f"  {render.BOLD}Selected:{render.RESET} {dc.name} ({res_str})",
    ]

    # Resonance bonuses applied
    res_bonus_parts: list[str] = []
    for res_name, amount in sorted(dc.resonance_bonus.items()):
        try:
            res_enum = Resonance(res_name)
            res_bonus_parts.append(f"{render.color_resonance(res_enum)} +{amount}")
        except ValueError:
            res_bonus_parts.append(f"{res_name} +{amount}")
    if res_bonus_parts:
        lines.append(f"  Resonance: {', '.join(res_bonus_parts)}")

    # Tag bonuses applied
    tag_parts: list[str] = []
    for tag, amount in sorted(dc.tag_bonus.items()):
        tag_parts.append(f"{tag} +{amount}")
    if tag_parts:
        lines.append(f"  Tags: {', '.join(tag_parts)}")

    # Essence bonus
    lines.append(
        f"  Essence: {render.BOLD}+{dc.essence_bonus}{render.RESET}"
        f" -> {essence_after}"
    )

    return "\n".join(lines)


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
    print(
        render_status.site_header(
            dreamscape_name=dreamscape_name,
            site_type_label="Dreamcaller Draft",
            dreamscape_number=dreamscape_number,
        )
    )
    print()
    print(f"  {render.BOLD}Choose your Dreamcaller:{render.RESET}")
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

    # Display confirmation with applied bonuses
    print()
    print(format_confirmation(selected, state.essence))

    # Log the selection
    if logger is not None:
        logger.log_site_visit(
            site_type="DreamcallerDraft",
            dreamscape=dreamscape_name,
            is_enhanced=False,
            choices=[dc.name for dc in choices],
            choice_made=selected.name,
            state_changes={
                "dreamcaller": selected.name,
                "essence_delta": selected.essence_bonus,
                "resonance_bonus": dict(selected.resonance_bonus),
                "tag_bonus": dict(selected.tag_bonus),
            },
            profile_snapshot=state.resonance_profile.snapshot(),
        )

    # Show resonance profile footer
    print()
    footer = render_status.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)
