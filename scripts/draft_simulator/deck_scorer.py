"""Deck value scoring for drafted card pools.

Evaluates how good a pool of drafted cards is by computing a weighted
combination of raw power, archetype coherence, and focus bonus. Each
component is normalized to [0, 1] and the final score is clamped to
[0, 1]. Stdlib-only, no external dependencies.
"""

from typing import Sequence, Union

from config import ScoringConfig
from draft_models import CardDesign, CardInstance
from utils import argmax


def deck_value(
    pool: Sequence[Union[CardDesign, CardInstance]],
    w: list[float],
    scoring: ScoringConfig,
) -> float:
    """Score a card pool as a float in [0, 1].

    Takes drafted cards (as CardDesign or CardInstance objects) and
    the agent's preference vector w. Returns a deterministic score
    for any pool size (including partial pools during draft).
    """
    if not pool or not w:
        return 0.0

    designs = _to_designs(pool)

    raw = raw_power(designs)
    coherence = archetype_coherence(designs, w, scoring.secondary_weight)
    focus = focus_bonus(designs, w, scoring.focus_threshold, scoring.focus_saturation)

    score = (
        scoring.weight_power * raw
        + scoring.weight_coherence * coherence
        + scoring.weight_focus * focus
    )
    return max(0.0, min(1.0, score))


def deck_value_breakdown(
    pool: Sequence[Union[CardDesign, CardInstance]],
    w: list[float],
    scoring: ScoringConfig,
) -> tuple[float, float, float, float]:
    """Return (raw_power, archetype_coherence, focus_bonus, final_score)."""
    if not pool or not w:
        return (0.0, 0.0, 0.0, 0.0)

    designs = _to_designs(pool)

    raw = raw_power(designs)
    coherence = archetype_coherence(designs, w, scoring.secondary_weight)
    focus = focus_bonus(designs, w, scoring.focus_threshold, scoring.focus_saturation)

    score = (
        scoring.weight_power * raw
        + scoring.weight_coherence * coherence
        + scoring.weight_focus * focus
    )
    clamped = max(0.0, min(1.0, score))
    return (raw, coherence, focus, clamped)


def raw_power(pool: list[CardDesign]) -> float:
    """Mean power across all cards in the pool."""
    if not pool:
        return 0.0
    return sum(c.power for c in pool) / len(pool)


def archetype_coherence(
    pool: list[CardDesign],
    w: list[float],
    secondary_weight: float,
) -> float:
    """Compute archetype coherence from primary and secondary archetypes.

    The effective archetype is argmax(w). Primary coherence is the mean
    fitness for that archetype across all cards. Secondary coherence
    uses the second-highest archetype in w, discounted by secondary_weight.
    Result is clamped to [0, 1].
    """
    if not pool or not w:
        return 0.0

    primary_arch = argmax(w)
    secondary_arch = _second_argmax(w)

    primary_coherence = sum(c.fitness[primary_arch] for c in pool) / len(pool)
    secondary_coherence = sum(c.fitness[secondary_arch] for c in pool) / len(pool)

    raw = primary_coherence + secondary_weight * secondary_coherence
    return max(0.0, min(1.0, raw))


def focus_bonus(
    pool: list[CardDesign],
    w: list[float],
    threshold: float,
    saturation: float,
) -> float:
    """Fraction of on-plan cards mapped through a diminishing-returns ramp.

    On-plan cards have fitness for the effective archetype >= threshold.
    The fraction is mapped through a linear ramp that saturates at
    the saturation level: at or above saturation the bonus is 1.0,
    below that it scales linearly from 0.0 to 1.0.
    """
    if not pool or not w:
        return 0.0

    primary_arch = argmax(w)
    on_plan_count = sum(1 for c in pool if c.fitness[primary_arch] >= threshold)
    fraction = on_plan_count / len(pool)

    if saturation <= 0.0:
        return 1.0
    return max(0.0, min(1.0, fraction / saturation))


def pool_from_instances(instances: list[CardInstance]) -> list[CardDesign]:
    """Extract CardDesign objects from a list of CardInstance objects."""
    return [inst.design for inst in instances]


def _to_designs(
    pool: Sequence[Union[CardDesign, CardInstance]],
) -> list[CardDesign]:
    """Normalize a mixed pool to a list of CardDesign objects."""
    result: list[CardDesign] = []
    for item in pool:
        if isinstance(item, CardInstance):
            result.append(item.design)
        else:
            result.append(item)
    return result


def _second_argmax(values: list[float]) -> int:
    """Return the index of the second-highest value."""
    if len(values) < 2:
        return 0

    first_index = argmax(values)
    second_index = 0 if first_index != 0 else 1
    second_value = values[second_index]

    for i in range(len(values)):
        if i != first_index and values[i] > second_value:
            second_value = values[i]
            second_index = i

    return second_index
