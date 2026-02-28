"""Essence site interaction for the quest simulator.

Grants a fixed amount of essence that scales with the player's completion
level. Enhanced (Crystalline biome) doubles the amount.
"""

from typing import Optional

import input_handler
import render
from jsonl_log import SessionLogger
from quest_state import QuestState


def compute_essence_amount(
    completion_level: int,
    essence_config: dict[str, int],
    enhanced: bool = False,
) -> int:
    """Determine the essence amount for a given completion level.

    Level 0-1 uses amount_level_0, level 2-3 uses amount_level_2,
    level 4+ uses amount_level_4. Enhanced doubles the result.
    """
    if completion_level >= 4:
        amount = essence_config["amount_level_4"]
    elif completion_level >= 2:
        amount = essence_config["amount_level_2"]
    else:
        amount = essence_config["amount_level_0"]

    if enhanced:
        amount *= 2

    return amount


def run_essence(
    state: QuestState,
    essence_config: dict[str, int],
    dreamscape_name: str,
    dreamscape_number: int,
    logger: Optional[SessionLogger],
    is_enhanced: bool = False,
) -> None:
    """Run an Essence site interaction.

    Computes the essence gain based on completion level and enhancement,
    displays the result, waits for the player to continue, adds essence
    to quest state, logs the interaction, and shows the resonance footer.
    """
    amount = compute_essence_amount(
        state.completion_level, essence_config, enhanced=is_enhanced
    )

    # Display header
    enhanced_label = "Essence (Enhanced)" if is_enhanced else "Essence"
    header = render.site_visit_header(
        dreamscape_name=dreamscape_name,
        site_type_label=enhanced_label,
        pick_info="Instant",
        dreamscape_number=dreamscape_number,
    )
    print(header)
    print()

    # Display the essence gain
    essence_color = render.BOLD
    print(f"  You gain {essence_color}{amount}{render.RESET} essence!")
    if is_enhanced:
        print(f"  {render.DIM}(Crystalline biome doubled the amount){render.RESET}")
    print()

    # Wait for player to continue
    input_handler.wait_for_continue()

    # Apply the essence gain
    state.gain_essence(amount)

    # Log the interaction
    if logger is not None:
        logger.log_site_visit(
            site_type="Essence",
            dreamscape=dreamscape_name,
            is_enhanced=is_enhanced,
            choices=[],
            choice_made=None,
            state_changes={
                "essence_gained": amount,
            },
            profile_snapshot=state.resonance_profile.snapshot(),
        )

    # Show resonance profile footer
    footer = render.resonance_profile_footer(
        counts=state.resonance_profile.snapshot(),
        deck_count=state.deck_count(),
        essence=state.essence,
    )
    print(footer)
