"""Show-N card selection strategies for the draft simulator.

Selects a subset of N cards from a pack to present to the human seat.
Eight strategies: Uniform, Power-biased, Curated, Signal-rich, Top-scored,
Sharpened-preference, Plan-plus-power, and Deck-value-greedy. All strategies
use explicit rng parameter and weighted sampling without replacement.
Stdlib-only, no external dependencies.
"""

import random

import deck_scorer
from config import ScoringConfig
from draft_models import CardInstance
from utils import argmax


def select_cards(
    pack_cards: list[CardInstance],
    n: int,
    strategy: str,
    rng: random.Random,
    human_w: list[float] | None = None,
    human_drafted: list[CardInstance] | None = None,
    scoring_cfg: ScoringConfig | None = None,
) -> list[CardInstance]:
    """Select N cards from the pack using the specified strategy.

    If the pack has fewer than N cards, all cards are returned.
    """
    if len(pack_cards) <= n:
        return list(pack_cards)

    if strategy == "uniform":
        return _select_uniform(pack_cards, n, rng)
    elif strategy == "power_biased":
        return _select_power_biased(pack_cards, n, rng)
    elif strategy == "curated":
        return _select_curated(pack_cards, n, rng, human_w)
    elif strategy == "signal_rich":
        return _select_signal_rich(pack_cards, n, rng)
    elif strategy == "top_scored":
        return _select_top_scored(pack_cards, n, rng, human_w)
    elif strategy == "sharpened_preference":
        return _select_sharpened_preference(pack_cards, n, rng, human_w)
    elif strategy == "plan_plus_power":
        return _select_plan_plus_power(pack_cards, n, rng, human_w)
    elif strategy == "deck_value_greedy":
        return _select_deck_value_greedy(
            pack_cards,
            n,
            rng,
            human_w,
            human_drafted,
            scoring_cfg,
        )
    else:
        raise ValueError(f"Unknown Show-N strategy: {strategy!r}")


def _select_uniform(
    cards: list[CardInstance],
    n: int,
    rng: random.Random,
) -> list[CardInstance]:
    """Select N cards uniformly at random."""
    return _weighted_sample(cards, [1.0] * len(cards), n, rng)


def _select_power_biased(
    cards: list[CardInstance],
    n: int,
    rng: random.Random,
) -> list[CardInstance]:
    """Select N cards with probability proportional to power."""
    weights = [max(c.design.power, 0.001) for c in cards]
    return _weighted_sample(cards, weights, n, rng)


def _select_curated(
    cards: list[CardInstance],
    n: int,
    rng: random.Random,
    human_w: list[float] | None,
) -> list[CardInstance]:
    """Select N cards with on-plan and off-plan guarantees.

    At least 1 on-plan card (fitness for best archetype >= 0.6) if
    available; at least 1 off-plan but strong card (fitness for best
    archetype < 0.3, power >= 0.5) if available. Remaining slots
    filled by power-weighted sampling. Falls back to power-biased if
    constraints cannot be met.
    """
    if human_w is None or not human_w:
        return _select_power_biased(cards, n, rng)

    best_arch = argmax(human_w)

    on_plan_candidates = [c for c in cards if c.design.fitness[best_arch] >= 0.6]
    off_plan_candidates = [
        c for c in cards if c.design.fitness[best_arch] < 0.3 and c.design.power >= 0.5
    ]

    selected: list[CardInstance] = []
    used_ids: set[int] = set()

    # Guarantee at least 1 on-plan card if available and slots remain
    if on_plan_candidates and len(selected) < n:
        pick = on_plan_candidates[rng.randrange(len(on_plan_candidates))]
        selected.append(pick)
        used_ids.add(pick.instance_id)

    # Guarantee at least 1 off-plan strong card if available and slots remain
    if len(selected) < n:
        off_plan_remaining = [
            c for c in off_plan_candidates if c.instance_id not in used_ids
        ]
        if off_plan_remaining:
            pick = off_plan_remaining[rng.randrange(len(off_plan_remaining))]
            selected.append(pick)
            used_ids.add(pick.instance_id)

    # Fill remaining slots by power-weighted sampling
    remaining_needed = n - len(selected)
    if remaining_needed > 0:
        remaining_cards = [c for c in cards if c.instance_id not in used_ids]
        if remaining_cards:
            weights = [max(c.design.power, 0.001) for c in remaining_cards]
            fill = _weighted_sample(remaining_cards, weights, remaining_needed, rng)
            selected.extend(fill)

    return selected


def _select_signal_rich(
    cards: list[CardInstance],
    n: int,
    rng: random.Random,
) -> list[CardInstance]:
    """Select N cards with probability proportional to commit * 2 + power."""
    weights = [max(c.design.commit * 2.0 + c.design.power, 0.001) for c in cards]
    return _weighted_sample(cards, weights, n, rng)


def _select_top_scored(
    cards: list[CardInstance],
    n: int,
    rng: random.Random,
    human_w: list[float] | None,
) -> list[CardInstance]:
    """Select top N cards by combined power and preference score.

    Score = 0.3 * power + 0.7 * dot(fitness, normalize(w)) + gauss(0, 0.05).
    Falls back to power-biased when w is None or empty.
    """
    if human_w is None or not human_w:
        return _select_power_biased(cards, n, rng)

    w_norm = _ts_normalize(human_w)
    scored: list[tuple[float, int, CardInstance]] = []
    for idx, card in enumerate(cards):
        pref = _ts_dot(card.design.fitness, w_norm)
        score = 0.3 * card.design.power + 0.7 * pref + rng.gauss(0.0, 0.05)
        scored.append((score, idx, card))

    scored.sort(key=lambda t: t[0], reverse=True)
    return [t[2] for t in scored[:n]]


def _select_sharpened_preference(
    cards: list[CardInstance],
    n: int,
    rng: random.Random,
    human_w: list[float] | None,
) -> list[CardInstance]:
    """Select top N cards by combined power and sharpened preference score.

    Sharpens the preference vector by raising each component to the 4th power
    before normalizing, which amplifies dominant archetype preferences.
    Score = 0.3 * power + 0.7 * dot(fitness, normalize(sharpen(w))) + gauss(0, 0.05).
    Falls back to power-biased when w is None or empty.
    """
    if human_w is None or not human_w:
        return _select_power_biased(cards, n, rng)

    w_sharp = [v**4.0 for v in human_w]
    w_norm = _ts_normalize(w_sharp)
    scored: list[tuple[float, int, CardInstance]] = []
    for idx, card in enumerate(cards):
        pref = _ts_dot(card.design.fitness, w_norm)
        score = 0.3 * card.design.power + 0.7 * pref + rng.gauss(0.0, 0.05)
        scored.append((score, idx, card))

    scored.sort(key=lambda t: t[0], reverse=True)
    return [t[2] for t in scored[:n]]


def _select_plan_plus_power(
    cards: list[CardInstance],
    n: int,
    rng: random.Random,
    human_w: list[float] | None,
) -> list[CardInstance]:
    """Reserve slots for on-plan cards, fill the rest by power.

    Reserves up to 3 slots for cards whose fitness for the best archetype
    meets the on-plan threshold (0.3). Remaining slots are filled by
    off-plan cards sorted by power descending. Falls back to power-biased
    when human_w is None/empty or when concentration (max(w)/sum(w)) is
    below 0.15.
    """
    on_plan_threshold = 0.3
    concentration_threshold = 0.15
    max_on_plan_slots = 3

    if human_w is None or not human_w:
        return _select_power_biased(cards, n, rng)

    w_sum = sum(human_w)
    if w_sum <= 0.0:
        return _select_power_biased(cards, n, rng)

    concentration = max(human_w) / w_sum
    if concentration < concentration_threshold:
        return _select_power_biased(cards, n, rng)

    best_arch = argmax(human_w)

    on_plan = [c for c in cards if c.design.fitness[best_arch] >= on_plan_threshold]
    off_plan = [c for c in cards if c.design.fitness[best_arch] < on_plan_threshold]

    # Sort on-plan by fitness desc, then power desc for tie-breaking
    on_plan.sort(
        key=lambda c: (c.design.fitness[best_arch], c.design.power), reverse=True
    )

    # Take up to max_on_plan_slots best on-plan cards
    selected = on_plan[: min(max_on_plan_slots, len(on_plan))]

    # Sort off-plan by power desc and fill remaining slots
    off_plan.sort(key=lambda c: c.design.power, reverse=True)
    remaining_needed = n - len(selected)
    selected += off_plan[:remaining_needed]

    # If we still need more (not enough off-plan), take more on-plan
    if len(selected) < n:
        remaining_on = on_plan[min(max_on_plan_slots, len(on_plan)) :]
        selected += remaining_on[: n - len(selected)]

    return selected


def _select_deck_value_greedy(
    cards: list[CardInstance],
    n: int,
    rng: random.Random,
    human_w: list[float] | None,
    human_drafted: list[CardInstance] | None,
    scoring_cfg: ScoringConfig | None,
) -> list[CardInstance]:
    """Select top N cards by marginal deck_value improvement.

    For each card in the pack, computes deck_value(human_drafted + [card])
    using the human's preference vector and scoring config. Returns the
    top N cards by that score. Falls back to power-biased when any
    required input is missing.
    """
    if human_w is None or not human_w or human_drafted is None or scoring_cfg is None:
        return _select_power_biased(cards, n, rng)

    scored: list[tuple[float, int, CardInstance]] = []
    for idx, card in enumerate(cards):
        trial_pool: list[CardInstance] = list(human_drafted) + [card]
        value = deck_scorer.deck_value(trial_pool, human_w, scoring_cfg)
        scored.append((value, idx, card))

    scored.sort(key=lambda t: t[0], reverse=True)
    return [t[2] for t in scored[:n]]


def _ts_normalize(w: list[float]) -> list[float]:
    """Normalize a vector so its elements sum to 1.0."""
    total = sum(w)
    if total <= 0.0:
        return [1.0 / len(w)] * len(w) if w else []
    return [v / total for v in w]


def _ts_dot(a: list[float], b: list[float]) -> float:
    """Compute the dot product of two vectors of equal length."""
    return sum(x * y for x, y in zip(a, b))


def _weighted_sample(
    items: list[CardInstance],
    weights: list[float],
    n: int,
    rng: random.Random,
) -> list[CardInstance]:
    """Weighted sampling without replacement.

    Follows the quest_simulator/algorithm.py pattern: maintain active
    indices, pick via cumulative weight selection, remove chosen index.
    """
    selected: list[CardInstance] = []
    indices = list(range(len(items)))
    remaining_weights = list(weights)

    for _ in range(min(n, len(items))):
        total = sum(remaining_weights[i] for i in indices)
        if total <= 0:
            break
        r = rng.uniform(0, total)
        cumulative = 0.0
        chosen_pos = 0
        for pos, idx in enumerate(indices):
            cumulative += remaining_weights[idx]
            if cumulative >= r:
                chosen_pos = pos
                break
        chosen_idx = indices[chosen_pos]
        selected.append(items[chosen_idx])
        indices.pop(chosen_pos)

    return selected
