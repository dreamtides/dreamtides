"""JSONL logging for interactive draft simulation sessions.

Writes one JSON object per line to .logs/ for post-hoc analysis of
interactive sessions.
"""

import json
import os
import time
from pathlib import Path

from models import PickRecord, QuestResult, Rarity, Resonance, StrategyParams

# Project root is 3 levels up from this file
_PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
_LOG_DIR = _PROJECT_ROOT / ".logs"


def _resonance_name(r: Resonance) -> str:
    return r.value


def _resonances_list(rs: frozenset) -> list[str]:
    return sorted(_resonance_name(r) for r in rs)


def _card_dict(card) -> dict:
    return {
        "id": card.id,
        "resonances": _resonances_list(card.resonances),
        "rarity": card.rarity.value,
        "power": card.power,
    }


def _profile_dict(profile_snapshot: dict[Resonance, int]) -> dict[str, int]:
    return {_resonance_name(r): c for r, c in sorted(
        profile_snapshot.items(), key=lambda x: x[0].value
    )}


class SessionLogger:
    """Writes JSONL events for a single interactive session."""

    def __init__(self, seed: int):
        _LOG_DIR.mkdir(parents=True, exist_ok=True)
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        filename = f"draft_{timestamp}_seed{seed}.jsonl"
        self._path = _LOG_DIR / filename
        self._file = open(self._path, "w")

    def _write(self, event: dict):
        self._file.write(json.dumps(event, separators=(",", ":")) + "\n")
        self._file.flush()

    def log_session_start(
        self,
        seed: int,
        result: QuestResult,
        strategy_name: str,
        strat_params: StrategyParams,
        algo_params,
    ):
        self._write({
            "event": "session_start",
            "seed": seed,
            "strategy": strategy_name,
            "strategy_params": {
                "power_weight": strat_params.power_weight,
                "fit_weight": strat_params.fit_weight,
            },
            "algorithm_params": {
                "exponent": algo_params.exponent,
                "floor_weight": algo_params.floor_weight,
                "neutral_base": algo_params.neutral_base,
                "staleness_factor": algo_params.staleness_factor,
            },
            "dreamcaller_resonances": _resonances_list(
                result.dreamcaller_resonances
            ),
            "pool_variance": {
                _resonance_name(r): round(m, 4)
                for r, m in sorted(
                    result.pool_variance.items(), key=lambda x: x[0].value
                )
            },
            "total_picks": len(result.picks),
        })

    def log_pick(self, pick: PickRecord, context: dict):
        offered = []
        for card, weight in zip(pick.offered, pick.weights):
            offered.append({**_card_dict(card), "weight": round(weight, 4)})

        self._write({
            "event": "pick",
            "pick_number": pick.pick_number,
            "dreamscape": context["dreamscape"],
            "site": context["site"],
            "is_battle_reward": context["is_battle_reward"],
            "offered": offered,
            "picked": _card_dict(pick.picked),
            "pick_reason": pick.pick_reason,
            "profile_after": _profile_dict(pick.profile_after),
        })

    def log_session_end(self, result: QuestResult):
        profile = result.final_profile
        rarity_counts = {r.value: 0 for r in Rarity}
        for card in result.deck:
            rarity_counts[card.rarity.value] += 1

        self._write({
            "event": "session_end",
            "total_cards": len(result.deck),
            "final_profile": _profile_dict(result.final_profile.snapshot()),
            "top2_share": round(profile.top2_share(), 4),
            "hhi": round(profile.hhi(), 4),
            "effective_colors": round(profile.effective_colors(), 4),
            "rarity_breakdown": rarity_counts,
        })

    def close(self):
        self._file.close()

    @property
    def path(self) -> Path:
        return self._path
