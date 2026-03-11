"""Debug panel for AI draft bot state during quest mode.

Renders a multi-line display showing draft status, seating layout,
and per-bot archetype preferences with commitment detection.
"""

from quest_state import QuestState


def render_debug_panel(state: QuestState) -> str:
    """Build a multi-line debug panel string."""
    return state.draft_strategy.render_debug_panel()
