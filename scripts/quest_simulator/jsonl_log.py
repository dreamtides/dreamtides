"""JSONL logging for quest simulation sessions.

Writes one JSON object per line to .logs/ for post-hoc analysis of
quest playthroughs.
"""

import json
import time
from pathlib import Path
from typing import Any, Optional

from models import (
    DeckCard,
    Dreamcaller,
    DreamscapeNode,
    Dreamsign,
    Site,
)

_PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
_LOG_DIR = _PROJECT_ROOT / ".logs"


def _deck_card_dict(dc: DeckCard) -> dict[str, object]:
    """Serialize a DeckCard to a dict."""
    instance = dc.instance
    result: dict[str, object] = {
        "is_bane": dc.is_bane,
        "is_transfigured": dc.is_transfigured,
    }
    if hasattr(instance, "design") and hasattr(instance.design, "name"):
        result["name"] = instance.design.name
    elif hasattr(instance, "name"):
        result["name"] = instance.name
    if hasattr(instance, "instance_id"):
        result["instance_id"] = instance.instance_id
    return result


class SessionLogger:
    """Writes JSONL events for a single quest session."""

    def __init__(self, seed: int) -> None:
        _LOG_DIR.mkdir(parents=True, exist_ok=True)
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        ns = time.time_ns()
        filename = f"quest_{timestamp}_{ns}_seed{seed}.jsonl"
        self._path = _LOG_DIR / filename
        self._file = open(self._path, "w")

    def _write(self, event: dict[str, object]) -> None:
        self._file.write(json.dumps(event, separators=(",", ":")) + "\n")
        self._file.flush()

    def log_session_start(
        self,
        seed: int,
        atlas_topology: list[DreamscapeNode],
    ) -> None:
        """Log the start of a quest session."""
        nodes = []
        for node in atlas_topology:
            nodes.append(
                {
                    "node_id": node.node_id,
                    "name": node.name,
                    "state": node.state.value,
                    "adjacent": node.adjacent,
                }
            )
        self._write(
            {
                "event": "session_start",
                "seed": seed,
                "atlas_nodes": nodes,
            }
        )

    def log_dreamscape_enter(
        self,
        dreamscape_name: str,
        completion_level: int,
        sites: list[Site],
    ) -> None:
        """Log entering a dreamscape."""
        self._write(
            {
                "event": "dreamscape_enter",
                "dreamscape_name": dreamscape_name,
                "completion_level": completion_level,
                "sites": [
                    {
                        "site_type": s.site_type.value,
                        "is_enhanced": s.is_enhanced,
                    }
                    for s in sites
                ],
            }
        )

    def log_site_visit(
        self,
        site_type: str,
        choices: list[str],
        choice_made: Optional[str],
        state_changes: dict[str, object],
        dreamscape: str = "",
        is_enhanced: bool = False,
    ) -> None:
        """Log a site visit with choices, state deltas, and context."""
        event: dict[str, object] = {
            "event": "site_visit",
            "site_type": site_type,
            "dreamscape": dreamscape,
            "is_enhanced": is_enhanced,
            "choices_offered": choices,
            "choice_made": choice_made,
            "state_changes": state_changes,
        }
        self._write(event)

    def log_draft_pick(
        self,
        offered_cards: list[Any],
        weights: list[float],
        picked_card: Any,
    ) -> None:
        """Log a draft pick with offered cards, weights, and selection."""
        if len(offered_cards) != len(weights):
            raise ValueError(
                f"offered_cards length ({len(offered_cards)}) != "
                f"weights length ({len(weights)})"
            )
        offered = []
        for card, weight in zip(offered_cards, weights):
            entry: dict[str, object] = {"weight": round(weight, 4)}
            if hasattr(card, "design") and hasattr(card.design, "name"):
                entry["name"] = card.design.name
            elif hasattr(card, "name"):
                entry["name"] = card.name
            offered.append(entry)

        picked: dict[str, object] = {}
        if hasattr(picked_card, "design") and hasattr(picked_card.design, "name"):
            picked["name"] = picked_card.design.name
        elif hasattr(picked_card, "name"):
            picked["name"] = picked_card.name

        self._write(
            {
                "event": "draft_pick",
                "offered": offered,
                "picked": picked,
            }
        )

    def log_shop_purchase(
        self,
        items_shown: list[Any],
        items_bought: list[Any],
        essence_spent: int,
    ) -> None:
        """Log a shop interaction with items shown, bought, and cost."""
        def _card_name(card: Any) -> str:
            if hasattr(card, "design") and hasattr(card.design, "name"):
                return card.design.name
            if hasattr(card, "name"):
                return card.name
            return str(card)

        self._write(
            {
                "event": "shop_purchase",
                "items_shown": [_card_name(c) for c in items_shown],
                "items_bought": [_card_name(c) for c in items_bought],
                "essence_spent": essence_spent,
            }
        )

    def log_battle_complete(
        self,
        opponent_name: str,
        essence_reward: int,
        rare_pick: Optional[Any],
    ) -> None:
        """Log battle completion with reward details."""
        pick_name: Optional[str] = None
        if rare_pick is not None:
            if hasattr(rare_pick, "design") and hasattr(rare_pick.design, "name"):
                pick_name = rare_pick.design.name
            elif hasattr(rare_pick, "name"):
                pick_name = rare_pick.name
        self._write(
            {
                "event": "battle_complete",
                "opponent_name": opponent_name,
                "essence_reward": essence_reward,
                "rare_pick": pick_name,
            }
        )

    def log_session_end(
        self,
        deck: list[DeckCard],
        essence: int,
        completion_level: int,
        dreamsigns: list[Dreamsign],
        dreamcaller: Optional[Dreamcaller],
    ) -> None:
        """Log the end of a quest session with final state."""
        self._write(
            {
                "event": "session_end",
                "total_cards": len(deck),
                "deck": [_deck_card_dict(dc) for dc in deck],
                "essence": essence,
                "completion_level": completion_level,
                "dreamsigns": [
                    {"name": ds.name}
                    for ds in dreamsigns
                ],
                "dreamcaller": dreamcaller.name if dreamcaller is not None else None,
            }
        )

    def log_error(
        self,
        site_type: str,
        error_message: str,
    ) -> None:
        """Log an error that occurred during a site visit."""
        self._write(
            {
                "event": "error",
                "site_type": site_type,
                "error_message": error_message,
            }
        )

    def close(self) -> None:
        self._file.close()

    @property
    def path(self) -> Path:
        return self._path
