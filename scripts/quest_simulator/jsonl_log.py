"""JSONL logging for quest simulation sessions.

Writes one JSON object per line to .logs/ for post-hoc analysis of
quest playthroughs.
"""

import json
import time
from pathlib import Path
from typing import Optional

from models import (
    AlgorithmParams,
    Card,
    DeckCard,
    Dreamcaller,
    DreamscapeNode,
    Dreamsign,
    Rarity,
    Resonance,
    ResonanceProfile,
    Site,
)

_PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
_LOG_DIR = _PROJECT_ROOT / ".logs"


def _resonance_name(r: Resonance) -> str:
    return r.value


def _resonances_list(rs: frozenset[Resonance]) -> list[str]:
    return sorted(_resonance_name(r) for r in rs)


def _card_dict(card: Card) -> dict[str, object]:
    return {
        "name": card.name,
        "card_number": card.card_number,
        "resonances": _resonances_list(card.resonances),
        "rarity": card.rarity.value,
        "energy_cost": card.energy_cost,
        "spark": card.spark,
    }


def _profile_dict(profile_snapshot: dict[Resonance, int]) -> dict[str, int]:
    return {
        _resonance_name(r): c
        for r, c in sorted(profile_snapshot.items(), key=lambda x: x[0].value)
    }


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
        params: AlgorithmParams,
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
                "algorithm_params": {
                    "exponent": params.exponent,
                    "floor_weight": params.floor_weight,
                    "neutral_base": params.neutral_base,
                    "staleness_factor": params.staleness_factor,
                },
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
    ) -> None:
        """Log a site visit with choices and state deltas."""
        self._write(
            {
                "event": "site_visit",
                "site_type": site_type,
                "choices": choices,
                "choice_made": choice_made,
                "state_changes": state_changes,
            }
        )

    def log_draft_pick(
        self,
        offered_cards: list[Card],
        weights: list[float],
        picked_card: Card,
        profile_snapshot: dict[Resonance, int],
    ) -> None:
        """Log a draft pick with offered cards, weights, and selection."""
        if len(offered_cards) != len(weights):
            raise ValueError(
                f"offered_cards length ({len(offered_cards)}) != "
                f"weights length ({len(weights)})"
            )
        offered = []
        for card, weight in zip(offered_cards, weights):
            offered.append({**_card_dict(card), "weight": round(weight, 4)})
        self._write(
            {
                "event": "draft_pick",
                "offered": offered,
                "picked": _card_dict(picked_card),
                "profile_after": _profile_dict(profile_snapshot),
            }
        )

    def log_shop_purchase(
        self,
        items_shown: list[Card],
        items_bought: list[Card],
        essence_spent: int,
    ) -> None:
        """Log a shop interaction with items shown, bought, and cost."""
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
        rare_pick: Optional[Card],
    ) -> None:
        """Log battle completion with reward details."""
        self._write(
            {
                "event": "battle_complete",
                "opponent_name": opponent_name,
                "essence_reward": essence_reward,
                "rare_pick": _card_dict(rare_pick) if rare_pick is not None else None,
            }
        )

    def log_session_end(
        self,
        deck: list[DeckCard],
        resonance_profile: ResonanceProfile,
        essence: int,
        completion_level: int,
        dreamsigns: list[Dreamsign],
        dreamcaller: Optional[Dreamcaller],
    ) -> None:
        """Log the end of a quest session with final state."""
        rarity_counts: dict[str, int] = {r.value: 0 for r in Rarity}
        for dc in deck:
            rarity_counts[dc.card.rarity.value] += 1

        self._write(
            {
                "event": "session_end",
                "total_cards": len(deck),
                "final_profile": _profile_dict(resonance_profile.snapshot()),
                "rarity_breakdown": rarity_counts,
                "essence": essence,
                "completion_level": completion_level,
                "dreamsigns": [
                    {"name": ds.name, "resonance": ds.resonance.value}
                    for ds in dreamsigns
                ],
                "dreamcaller": dreamcaller.name if dreamcaller is not None else None,
            }
        )

    def close(self) -> None:
        self._file.close()

    @property
    def path(self) -> Path:
        return self._path
