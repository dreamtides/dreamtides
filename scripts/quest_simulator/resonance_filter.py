"""Resonance-based card filtering for the quest simulator.

Restricts dual-resonance cards so only those matching the player's or
AI bot's resonance pair are offered. Single-resonance and neutral cards
are always eligible.
"""

from typing import Optional

import render
from draft_models import CardInstance
from quest_state import QuestState


def human_resonance_pair(state: QuestState) -> Optional[tuple[str, str]]:
    """Return the resonance pair for the human's dreamcaller, or None."""
    if state.dreamcaller is None:
        return None
    return render.ARCHETYPE_RESONANCE.get(state.dreamcaller.archetype)


def filter_off_resonance_duals(
    cards: list[CardInstance],
    resonance_pair: Optional[tuple[str, str]],
) -> list[CardInstance]:
    """Filter out dual-resonance cards that don't match the resonance pair.

    Cards with 0 or 1 resonance tags are always kept. Dual-resonance
    cards are kept only if their resonance set matches the given pair.
    If resonance_pair is None, all cards are returned.
    """
    if resonance_pair is None:
        return cards
    pair_set = set(resonance_pair)
    return [
        c
        for c in cards
        if len(c.design.resonance) < 2 or set(c.design.resonance) == pair_set
    ]
