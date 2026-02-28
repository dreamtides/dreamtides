"""Core weighting algorithm, pool generation, and card selection."""

import math
import random

from ds_models import (
    AlgorithmParams,
    PoolEntry,
    PoolParams,
    Rarity,
    Resonance,
    ResonanceProfile,
    SimCard,
)

RARITIES = [Rarity.COMMON, Rarity.UNCOMMON, Rarity.RARE, Rarity.LEGENDARY]
RARITY_WEIGHTS = [0.55, 0.25, 0.15, 0.05]


def generate_pool(
    params: PoolParams,
    rng: random.Random,
    num_unique: int = 360,
) -> tuple[list[PoolEntry], dict[Resonance, float]]:
    """Generate a pool of cards with rarity-based copy counts and resonance variance.

    Returns (pool, variance) where variance maps each resonance to its
    random starting multiplier.
    """
    resonances = list(Resonance)
    variance = {
        r: rng.uniform(params.variance_min, params.variance_max) for r in resonances
    }

    cards: list[SimCard] = []
    card_id = 0

    # Distribute cards: 70% single-res, 10% dual, 20% neutral
    n_single = int(num_unique * 0.70)
    n_dual = int(num_unique * 0.10)
    n_neutral = num_unique - n_single - n_dual

    # Single-resonance cards, evenly distributed
    per_res = n_single // len(resonances)
    remainder = n_single - per_res * len(resonances)
    for i, res in enumerate(resonances):
        count = per_res + (1 if i < remainder else 0)
        for _ in range(count):
            rarity = rng.choices(RARITIES, weights=RARITY_WEIGHTS, k=1)[0]
            power = _random_power(rarity, rng)
            cards.append(SimCard(card_id, frozenset([res]), rarity, power))
            card_id += 1

    # Dual-resonance cards
    pairs = [(a, b) for i, a in enumerate(resonances) for b in resonances[i + 1 :]]
    per_pair = n_dual // len(pairs)
    pair_remainder = n_dual - per_pair * len(pairs)
    for i, (a, b) in enumerate(pairs):
        count = per_pair + (1 if i < pair_remainder else 0)
        for _ in range(count):
            rarity = rng.choices(RARITIES, weights=RARITY_WEIGHTS, k=1)[0]
            power = _random_power(rarity, rng)
            cards.append(SimCard(card_id, frozenset([a, b]), rarity, power))
            card_id += 1

    # Neutral cards
    for _ in range(n_neutral):
        rarity = rng.choices(RARITIES, weights=RARITY_WEIGHTS, k=1)[0]
        power = _random_power(rarity, rng)
        cards.append(SimCard(card_id, frozenset(), rarity, power))
        card_id += 1

    # Build pool entries with copy counts scaled by resonance variance
    pool: list[PoolEntry] = []
    copies_by_rarity = {
        Rarity.COMMON: params.copies_common,
        Rarity.UNCOMMON: params.copies_uncommon,
        Rarity.RARE: params.copies_rare,
        Rarity.LEGENDARY: params.copies_legendary,
    }

    for card in cards:
        base_copies = copies_by_rarity[card.rarity]
        if card.resonances:
            multiplier = _resonance_multiplier(card.resonances, variance)
            copies = max(1, round(base_copies * multiplier))
        else:
            copies = base_copies
        for _ in range(copies):
            pool.append(PoolEntry(card))

    return pool, variance


def compute_weight(
    card: SimCard,
    profile: ResonanceProfile,
    params: AlgorithmParams,
) -> float:
    """Compute resonance weight for a card given the player's profile."""
    if not card.resonances:
        return params.neutral_base

    affinity_sum = sum(profile.counts[r] ** params.exponent for r in card.resonances)
    return params.floor_weight + affinity_sum


def apply_staleness(weight: float, staleness: int, factor: float) -> float:
    """Apply staleness penalty to a weight."""
    return weight / (1.0 + staleness * factor)


def select_cards(
    pool: list[PoolEntry],
    n: int,
    profile: ResonanceProfile,
    params: AlgorithmParams,
    rng: random.Random,
    rare_only: bool = False,
) -> list[tuple[PoolEntry, float]]:
    """Select n cards from pool via weighted sampling without replacement.

    Returns list of (entry, weight) tuples for the selected cards.
    """
    eligible = (
        pool
        if not rare_only
        else [e for e in pool if e.card.rarity in (Rarity.RARE, Rarity.LEGENDARY)]
    )

    if not eligible:
        return []

    # Compute weights
    weights = []
    for entry in eligible:
        w = compute_weight(entry.card, profile, params)
        w = apply_staleness(w, entry.staleness, params.staleness_factor)
        weights.append(max(w, 0.001))

    # Weighted sampling without replacement
    selected: list[tuple[PoolEntry, float]] = []
    indices = list(range(len(eligible)))
    remaining_weights = list(weights)

    for _ in range(min(n, len(eligible))):
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
        selected.append((eligible[chosen_idx], weights[chosen_idx]))
        indices.pop(chosen_pos)

    # Resonance diversity check: if all n cards share a single resonance and n >= 4
    if len(selected) >= 4:
        selected = _diversity_check(selected, eligible, weights, indices, pool)

    # Rarity guarantee: at least 1 uncommon+ for n >= 4
    if len(selected) >= 4:
        selected = _rarity_guarantee(selected, eligible, weights, indices, pool)

    return selected


def _random_power(rarity: Rarity, rng: random.Random) -> int:
    """Generate a random power level for a card."""
    base = rng.randint(1, 10)
    if rarity in (Rarity.RARE, Rarity.LEGENDARY):
        base = min(10, base + 2)
    return base


def _resonance_multiplier(
    resonances: frozenset, variance: dict[Resonance, float]
) -> float:
    """Average variance multiplier for a card's resonances."""
    return sum(variance[r] for r in resonances) / len(resonances)


def _diversity_check(
    selected: list[tuple[PoolEntry, float]],
    eligible: list[PoolEntry],
    weights: list[float],
    remaining_indices: list[int],
    pool: list[PoolEntry],
) -> list[tuple[PoolEntry, float]]:
    """If all selected cards share one resonance, swap lowest for a different one."""
    resonance_sets = [e.card.resonances for e, _ in selected]
    if not all(rs for rs in resonance_sets):
        return selected  # Has neutrals, diversity OK

    common = resonance_sets[0]
    for rs in resonance_sets[1:]:
        common = common & rs
        if not common:
            return selected  # Not all sharing a resonance

    if not common:
        return selected

    shared_res = next(iter(common))
    # All share at least one resonance; find best replacement from different resonance
    best_replacement = None
    best_weight = -1.0
    for idx in remaining_indices:
        entry = eligible[idx]
        if shared_res not in entry.card.resonances:
            if weights[idx] > best_weight:
                best_weight = weights[idx]
                best_replacement = (entry, weights[idx])

    if best_replacement:
        # Replace lowest-weight selected card
        min_idx = min(range(len(selected)), key=lambda i: selected[i][1])
        selected[min_idx] = best_replacement

    return selected


def _rarity_guarantee(
    selected: list[tuple[PoolEntry, float]],
    eligible: list[PoolEntry],
    weights: list[float],
    remaining_indices: list[int],
    pool: list[PoolEntry],
) -> list[tuple[PoolEntry, float]]:
    """Ensure at least 1 uncommon+ card in the selection."""
    has_uncommon_plus = any(
        e.card.rarity in (Rarity.UNCOMMON, Rarity.RARE, Rarity.LEGENDARY)
        for e, _ in selected
    )
    if has_uncommon_plus:
        return selected

    best_replacement = None
    best_weight = -1.0
    for idx in remaining_indices:
        entry = eligible[idx]
        if entry.card.rarity in (Rarity.UNCOMMON, Rarity.RARE, Rarity.LEGENDARY):
            if weights[idx] > best_weight:
                best_weight = weights[idx]
                best_replacement = (entry, weights[idx])

    if best_replacement:
        min_idx = min(range(len(selected)), key=lambda i: selected[i][1])
        selected[min_idx] = best_replacement

    return selected
