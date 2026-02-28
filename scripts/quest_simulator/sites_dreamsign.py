"""Dreamsign site interactions for the quest simulator.

Implements two dreamsign acquisition sites: Dreamsign Offering
(accept/decline 1, or pick-1-of-3 when enhanced) and Dreamsign Draft
(pick 1 of 3 or skip). Enforces the dreamsign limit of 12 with a
purge prompt when the player would exceed it.
"""

import random
from typing import Optional

import input_handler
import render
from jsonl_log import SessionLogger
from models import Dreamsign, Resonance
from quest_state import QuestState

DRAFT_COUNT = 3
SKIP_LABEL = "Skip"


def select_offering_dreamsigns(
    all_dreamsigns: list[Dreamsign],
    rng: random.Random,
    count: int = 1,
) -> list[Dreamsign]:
    """Select random non-bane dreamsigns for an offering.

    Returns up to `count` dreamsigns, excluding banes. If fewer
    non-bane dreamsigns are available than requested, returns all
    available.
    """
    non_bane = [ds for ds in all_dreamsigns if not ds.is_bane]
    n = min(count, len(non_bane))
    if n == 0:
        return []
    return rng.sample(non_bane, n)


def select_draft_dreamsigns(
    all_dreamsigns: list[Dreamsign],
    held: list[Dreamsign],
    rng: random.Random,
) -> list[Dreamsign]:
    """Select 3 random non-bane dreamsigns excluding already-held ones.

    Returns up to 3 dreamsigns that are neither banes nor already
    held by the player.
    """
    held_names = {ds.name for ds in held}
    available = [
        ds
        for ds in all_dreamsigns
        if not ds.is_bane and ds.name not in held_names
    ]
    n = min(DRAFT_COUNT, len(available))
    if n == 0:
        return []
    return rng.sample(available, n)


def format_dreamsign_option(
    ds: Dreamsign,
    highlighted: bool = False,
) -> list[str]:
    """Format a dreamsign for display in a selection menu.

    Returns a list of display lines: name with resonance, and
    effect text on the next line.
    """
    marker = ">" if highlighted else " "
    res_str = render.color_resonance(ds.resonance)
    name_color = render.RESONANCE_COLORS.get(ds.resonance, render.NEUTRAL_COLOR)
    line1 = f"  {marker} {name_color}{ds.name}{render.RESET}  {res_str}"
    line2 = f"      \"{ds.effect_text}\""
    return [line1, line2]


def handle_dreamsign_purge(
    state: QuestState,
    new_dreamsign: Dreamsign,
) -> None:
    """Add a dreamsign, prompting to purge one if at the limit.

    If the player is at or above the dreamsign limit (12), shows the
    existing dreamsigns in a single-select menu and removes the
    chosen one before adding the new one.
    """
    if state.is_over_dreamsign_limit():
        print()
        print(
            f"  {render.BOLD}Dreamsign limit reached ({state.max_dreamsigns})."
            f" Choose one to remove:{render.RESET}"
        )
        print()

        option_labels = [ds.name for ds in state.dreamsigns]

        def _render_purge_option(
            index: int,
            option: str,
            is_selected: bool,
        ) -> str:
            ds = state.dreamsigns[index]
            lines = format_dreamsign_option(ds, highlighted=is_selected)
            return "\n".join(lines)

        purge_index = input_handler.single_select(
            options=option_labels,
            render_fn=_render_purge_option,
        )
        purged = state.dreamsigns[purge_index]
        state.remove_dreamsign(purged)
        print(f"  Removed: {purged.name}")

    state.add_dreamsign(new_dreamsign)


def run_dreamsign_offering(
    state: QuestState,
    all_dreamsigns: list[Dreamsign],
    logger: Optional[SessionLogger],
    dreamscape_name: str = "",
    dreamscape_number: int = 1,
    is_enhanced: bool = False,
) -> None:
    """Run a Dreamsign Offering site interaction.

    Normal: show 1 dreamsign, accept or decline.
    Enhanced (Celestial biome): show 3 dreamsigns, pick 1 or skip.
    """
    count = DRAFT_COUNT if is_enhanced else 1
    offered = select_offering_dreamsigns(all_dreamsigns, state.rng, count=count)

    if not offered:
        print("  No dreamsigns available.")
        if logger is not None:
            logger.log_site_visit(
                site_type="DreamsignOffering",
                choices=[],
                choice_made=None,
                state_changes={
                    "dreamsign_added": None,
                    "dreamsign_count": state.dreamsign_count(),
                },
            )
        footer = render.resonance_profile_footer(
            counts=state.resonance_profile.snapshot(),
            deck_count=state.deck_count(),
            essence=state.essence,
        )
        print(footer)
        return

    # Display header
    label = "Dreamsign Offering (Enhanced)" if is_enhanced else "Dreamsign Offering"
    header = render.site_visit_header(
        dreamscape_name=dreamscape_name,
        site_type_label=label,
        pick_info="Choose a dreamsign" if is_enhanced else "Accept or decline",
        dreamscape_number=dreamscape_number,
    )
    print(header)
    print()

    chosen: Optional[Dreamsign] = None

    if is_enhanced:
        # Enhanced: single_select with skip option
        option_labels = [ds.name for ds in offered] + [SKIP_LABEL]

        def _render_enhanced_option(
            index: int,
            option: str,
            is_selected: bool,
            _offered: list[Dreamsign] = offered,
        ) -> str:
            if index < len(_offered):
                lines = format_dreamsign_option(
                    _offered[index], highlighted=is_selected
                )
                return "\n".join(lines)
            marker = ">" if is_selected else " "
            return f"  {marker} {render.DIM}{SKIP_LABEL}{render.RESET}"

        selected_index = input_handler.single_select(
            options=option_labels,
            render_fn=_render_enhanced_option,
        )
        if selected_index < len(offered):
            chosen = offered[selected_index]
    else:
        # Normal: display the single dreamsign and confirm/decline
        for ds in offered:
            lines = format_dreamsign_option(ds, highlighted=True)
            for line in lines:
                print(line)
        print()

        accepted = input_handler.confirm_decline()
        if accepted:
            chosen = offered[0]

    if chosen is not None:
        handle_dreamsign_purge(state, chosen)
        print()
        res_str = render.color_resonance(chosen.resonance)
        print(f"  {render.BOLD}Acquired:{render.RESET} {chosen.name} ({res_str})")
    else:
        print()
        print(f"  {render.DIM}Declined.{render.RESET}")

    # Log
    if logger is not None:
        logger.log_site_visit(
            site_type="DreamsignOffering",
            choices=[ds.name for ds in offered],
            choice_made=chosen.name if chosen is not None else None,
            state_changes={
                "dreamsign_added": chosen.name if chosen is not None else None,
                "dreamsign_count": state.dreamsign_count(),
            },
        )

    # Show resonance profile footer
    print()
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)


def run_dreamsign_draft(
    state: QuestState,
    all_dreamsigns: list[Dreamsign],
    logger: Optional[SessionLogger],
    dreamscape_name: str = "",
    dreamscape_number: int = 1,
) -> None:
    """Run a Dreamsign Draft site interaction.

    Shows 3 non-bane dreamsigns (excluding already-held), lets the
    player pick 1 or skip. If at dreamsign limit, triggers a purge
    prompt before adding.
    """
    offered = select_draft_dreamsigns(
        all_dreamsigns, held=state.dreamsigns, rng=state.rng
    )

    if not offered:
        print("  No dreamsigns available for drafting.")
        if logger is not None:
            logger.log_site_visit(
                site_type="DreamsignDraft",
                choices=[],
                choice_made=None,
                state_changes={
                    "dreamsign_added": None,
                    "dreamsign_count": state.dreamsign_count(),
                },
            )
        footer = render.resonance_profile_footer(
            counts=state.resonance_profile.snapshot(),
            deck_count=state.deck_count(),
            essence=state.essence,
        )
        print(footer)
        return

    # Display header
    header = render.site_visit_header(
        dreamscape_name=dreamscape_name,
        site_type_label="Dreamsign Draft",
        pick_info="Pick 1 or skip",
        dreamscape_number=dreamscape_number,
    )
    print(header)
    print()

    # Build options with skip
    option_labels = [ds.name for ds in offered] + [SKIP_LABEL]

    def _render_draft_option(
        index: int,
        option: str,
        is_selected: bool,
        _offered: list[Dreamsign] = offered,
    ) -> str:
        if index < len(_offered):
            lines = format_dreamsign_option(
                _offered[index], highlighted=is_selected
            )
            return "\n".join(lines)
        marker = ">" if is_selected else " "
        return f"  {marker} {render.DIM}{SKIP_LABEL}{render.RESET}"

    selected_index = input_handler.single_select(
        options=option_labels,
        render_fn=_render_draft_option,
    )

    chosen: Optional[Dreamsign] = None
    if selected_index < len(offered):
        chosen = offered[selected_index]
        handle_dreamsign_purge(state, chosen)
        print()
        res_str = render.color_resonance(chosen.resonance)
        print(f"  {render.BOLD}Acquired:{render.RESET} {chosen.name} ({res_str})")
    else:
        print()
        print(f"  {render.DIM}Skipped.{render.RESET}")

    # Log
    if logger is not None:
        logger.log_site_visit(
            site_type="DreamsignDraft",
            choices=[ds.name for ds in offered],
            choice_made=chosen.name if chosen is not None else None,
            state_changes={
                "dreamsign_added": chosen.name if chosen is not None else None,
                "dreamsign_count": state.dreamsign_count(),
            },
        )

    # Show resonance profile footer
    print()
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)
