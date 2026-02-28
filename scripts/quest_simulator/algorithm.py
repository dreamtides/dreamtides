"""Core weighting algorithm and card selection for the quest simulator."""

import random

from models import (
    AlgorithmParams,
    Card,
    PoolEntry,
    Rarity,
    Resonance,
    ResonanceProfile,
)


def compute_weight(
    card: Card,
    profile: ResonanceProfile,
    params: AlgorithmParams,
) -> float:
    """Compute resonance weight for a card given the player's profile."""
    if not card.resonances:
        return params.neutral_base

    affinity_sum = sum(
        profile.counts[r] ** params.exponent for r in card.resonances
    )
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
    eligible = pool if not rare_only else [
        e for e in pool if e.card.rarity in (Rarity.RARE, Rarity.LEGENDARY)
    ]

    if not eligible:
        return []

    # Compute weights
    weights: list[float] = []
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
        selected = _diversity_check(selected, eligible, weights, indices)

    # Rarity guarantee: at least 1 uncommon+ for n >= 4
    if len(selected) >= 4:
        selected = _rarity_guarantee(selected, eligible, weights, indices)

    return selected


def _diversity_check(
    selected: list[tuple[PoolEntry, float]],
    eligible: list[PoolEntry],
    weights: list[float],
    remaining_indices: list[int],
) -> list[tuple[PoolEntry, float]]:
    """If all selected cards share one resonance, swap lowest for a different one."""
    resonance_sets = [e.card.resonances for e, _ in selected]
    if not all(rs for rs in resonance_sets):
        return selected  # Has neutrals, diversity OK

    common: frozenset[Resonance] = resonance_sets[0]
    for rs in resonance_sets[1:]:
        common = common & rs
        if not common:
            return selected  # Not all sharing a resonance

    if not common:
        return selected

    shared_res = next(iter(common))
    # All share at least one resonance; find best replacement from different resonance
    best_replacement: tuple[PoolEntry, float] | None = None
    best_weight = -1.0
    for idx in remaining_indices:
        entry = eligible[idx]
        if shared_res not in entry.card.resonances:
            if weights[idx] > best_weight:
                best_weight = weights[idx]
                best_replacement = (entry, weights[idx])

    if not best_replacement:
        # Fallback: search the full eligible set (excluding already-selected
        # entries) for a replacement, as if refilling the pool with fresh copies.
        selected_entries = {id(e) for e, _ in selected}
        for idx in range(len(eligible)):
            if idx in remaining_indices:
                continue  # Already checked above
            entry = eligible[idx]
            if id(entry) in selected_entries:
                continue
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
) -> list[tuple[PoolEntry, float]]:
    """Ensure at least 1 uncommon+ card in the selection."""
    has_uncommon_plus = any(
        e.card.rarity in (Rarity.UNCOMMON, Rarity.RARE, Rarity.LEGENDARY)
        for e, _ in selected
    )
    if has_uncommon_plus:
        return selected

    best_replacement: tuple[PoolEntry, float] | None = None
    best_weight = -1.0
    for idx in remaining_indices:
        entry = eligible[idx]
        if entry.card.rarity in (Rarity.UNCOMMON, Rarity.RARE, Rarity.LEGENDARY):
            if weights[idx] > best_weight:
                best_weight = weights[idx]
                best_replacement = (entry, weights[idx])

    if not best_replacement:
        # Fallback: search the full eligible set (excluding already-selected
        # entries) for a replacement, as if refilling the pool with fresh copies.
        selected_entries = {id(e) for e, _ in selected}
        for idx in range(len(eligible)):
            if idx in remaining_indices:
                continue  # Already checked above
            entry = eligible[idx]
            if id(entry) in selected_entries:
                continue
            if entry.card.rarity in (Rarity.UNCOMMON, Rarity.RARE, Rarity.LEGENDARY):
                if weights[idx] > best_weight:
                    best_weight = weights[idx]
                    best_replacement = (entry, weights[idx])

    if best_replacement:
        min_idx = min(range(len(selected)), key=lambda i: selected[i][1])
        selected[min_idx] = best_replacement

    return selected
