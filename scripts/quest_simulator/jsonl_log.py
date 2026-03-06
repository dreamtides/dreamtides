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
    """Serialize a DeckCard to a dict with CardDesign/CardInstance fields."""
    instance = dc.instance
    result: dict[str, object] = {
        "is_bane": dc.is_bane,
        "is_transfigured": dc.is_transfigured,
    }
    if hasattr(instance, "instance_id"):
        result["instance_id"] = instance.instance_id
    design = getattr(instance, "design", instance)
    if hasattr(design, "name"):
        result["name"] = design.name
    if hasattr(design, "card_id"):
        result["card_id"] = design.card_id
    if hasattr(design, "power"):
        result["power"] = round(design.power, 4)
    if hasattr(design, "commit"):
        result["commit"] = round(design.commit, 4)
    if hasattr(design, "flex"):
        result["flex"] = round(design.flex, 4)
    if hasattr(design, "fitness"):
        fitness = design.fitness
        top = sorted(fitness, reverse=True)[:3] if fitness else []
        result["top_fitness"] = [round(v, 4) for v in top]
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
        draft_config: Optional[dict[str, object]] = None,
    ) -> None:
        """Log the start of a quest session with draft configuration."""
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
        event: dict[str, object] = {
            "event": "session_start",
            "seed": seed,
            "atlas_nodes": nodes,
        }
        if draft_config is not None:
            event["draft_config"] = draft_config
        self._write(event)

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
            design = getattr(card, "design", card)
            if hasattr(design, "name"):
                entry["name"] = design.name
            if hasattr(design, "card_id"):
                entry["card_id"] = design.card_id
            if hasattr(design, "power"):
                entry["power"] = round(design.power, 4)
            if hasattr(design, "commit"):
                entry["commit"] = round(design.commit, 4)
            if hasattr(design, "flex"):
                entry["flex"] = round(design.flex, 4)
            offered.append(entry)

        picked: dict[str, object] = {}
        picked_design = getattr(picked_card, "design", picked_card)
        if hasattr(picked_design, "name"):
            picked["name"] = picked_design.name
        if hasattr(picked_design, "card_id"):
            picked["card_id"] = picked_design.card_id
        if hasattr(picked_design, "power"):
            picked["power"] = round(picked_design.power, 4)
        if hasattr(picked_design, "commit"):
            picked["commit"] = round(picked_design.commit, 4)
        if hasattr(picked_design, "flex"):
            picked["flex"] = round(picked_design.flex, 4)

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
        def _card_dict(card: Any) -> dict[str, object]:
            design = getattr(card, "design", card)
            result: dict[str, object] = {}
            if hasattr(design, "name"):
                result["name"] = design.name
            if hasattr(design, "card_id"):
                result["card_id"] = design.card_id
            if hasattr(design, "power"):
                result["power"] = round(design.power, 4)
            if hasattr(design, "commit"):
                result["commit"] = round(design.commit, 4)
            if hasattr(design, "flex"):
                result["flex"] = round(design.flex, 4)
            return result

        self._write(
            {
                "event": "shop_purchase",
                "items_shown": [_card_dict(c) for c in items_shown],
                "items_bought": [_card_dict(c) for c in items_bought],
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
        preference_vector: Optional[list[float]] = None,
    ) -> None:
        """Log the end of a quest session with final state and preference vector."""
        event: dict[str, object] = {
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
        if preference_vector is not None:
            event["preference_vector"] = [round(v, 4) for v in preference_vector]
        self._write(event)

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
