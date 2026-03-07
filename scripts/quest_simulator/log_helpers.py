"""Shared helpers for computing show-N scores and preference stats for logging.

Provides utility functions used by JSONL logging to serialize card data,
compute show-N strategy scores, and summarize preference vector state.
"""

from typing import Any


def compute_show_n_scores(
    shown_cards: list[Any],
    human_w: list[float] | None,
    strategy: str,
) -> list[float]:
    """Recompute the sharpened_preference scores for shown cards.

    Returns a score per card using the same formula as the show-N
    strategy (without the Gaussian noise term). For strategies that
    don't use preference-based scoring, returns uniform 1.0 scores.
    """
    if strategy == "sharpened_preference" and human_w:
        w_sharp = [v**4.0 for v in human_w]
        w_norm = _normalize(w_sharp)
        scores = []
        for card in shown_cards:
            design = getattr(card, "design", card)
            fitness = getattr(design, "fitness", [])
            pref = _dot(fitness, w_norm)
            power = getattr(design, "power", 0.0)
            scores.append(0.3 * power + 0.7 * pref)
        return [round(s, 4) for s in scores]

    if strategy == "top_scored" and human_w:
        w_norm = _normalize(human_w)
        scores = []
        for card in shown_cards:
            design = getattr(card, "design", card)
            fitness = getattr(design, "fitness", [])
            pref = _dot(fitness, w_norm)
            power = getattr(design, "power", 0.0)
            scores.append(0.3 * power + 0.7 * pref)
        return [round(s, 4) for s in scores]

    return [1.0] * len(shown_cards)


def top_n_w(w: list[float], n: int = 3) -> list[tuple[int, float]]:
    """Return the top N (index, value) pairs from the preference vector."""
    indexed = [(i, v) for i, v in enumerate(w)]
    indexed.sort(key=lambda t: t[1], reverse=True)
    return [(i, round(v, 4)) for i, v in indexed[:n]]


def w_concentration(w: list[float]) -> float:
    """Compute max(w)/sum(w), measuring archetype commitment.

    Returns 0.0 if the sum is zero.
    """
    total = sum(w)
    if total <= 0.0:
        return 0.0
    return round(max(w) / total, 4)


def card_instance_dict(card: Any) -> dict[str, object]:
    """Serialize a CardInstance for logging.

    Includes name, card_id, power, commit, flex, and top 3 fitness values.
    """
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
    if hasattr(design, "fitness"):
        fitness = design.fitness
        top = sorted(fitness, reverse=True)[:3] if fitness else []
        result["top_fitness"] = [round(v, 4) for v in top]
    return result


def _normalize(w: list[float]) -> list[float]:
    """Normalize a vector so its elements sum to 1.0."""
    total = sum(w)
    if total <= 0.0:
        return [1.0 / len(w)] * len(w) if w else []
    return [v / total for v in w]


def _dot(a: list[float], b: list[float]) -> float:
    """Compute the dot product of two vectors of equal length."""
    return sum(x * y for x, y in zip(a, b))
