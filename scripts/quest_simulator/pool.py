"""Draft pool construction, refill, and staleness management."""

import random

from models import (
    Card,
    PoolEntry,
    PoolParams,
    Rarity,
    Resonance,
)


def generate_variance(
    rng: random.Random,
    params: PoolParams,
) -> dict[Resonance, float]:
    """Generate a per-resonance variance multiplier for pool construction."""
    return {r: rng.uniform(params.variance_min, params.variance_max) for r in Resonance}


def build_pool(
    cards: list[Card],
    params: PoolParams,
    variance: dict[Resonance, float],
) -> list[PoolEntry]:
    """Build a draft pool from card data with rarity-based copy counts.

    For each card, determines the base copy count by rarity and applies
    a per-resonance variance multiplier (average of the card's resonance
    variances). Neutral cards (no resonance) get unmodified base copies.
    """
    copies_map: dict[Rarity, int] = {
        Rarity.COMMON: params.copies_common,
        Rarity.UNCOMMON: params.copies_uncommon,
        Rarity.RARE: params.copies_rare,
        Rarity.LEGENDARY: params.copies_legendary,
    }

    pool: list[PoolEntry] = []
    for card in cards:
        base = copies_map[card.rarity]
        if card.resonances:
            multiplier = sum(variance[r] for r in card.resonances) / len(
                card.resonances
            )
            copies = max(1, round(base * multiplier))
        else:
            copies = base
        for _ in range(copies):
            pool.append(PoolEntry(card))
    return pool


def decay_staleness(pool: list[PoolEntry]) -> None:
    """Decrement staleness by 1 for all pool entries, minimum 0."""
    for entry in pool:
        entry.staleness = max(0, entry.staleness - 1)


def refill_pool(
    pool: list[PoolEntry],
    all_cards: list[Card],
    params: PoolParams,
) -> None:
    """Refill exhausted rarities in the pool at base copy counts.

    If any rarity has zero entries remaining in the pool, adds back
    entries for all cards of that rarity with staleness reset to 0.
    """
    copies_map: dict[Rarity, int] = {
        Rarity.COMMON: params.copies_common,
        Rarity.UNCOMMON: params.copies_uncommon,
        Rarity.RARE: params.copies_rare,
        Rarity.LEGENDARY: params.copies_legendary,
    }

    present_rarities: set[Rarity] = {e.card.rarity for e in pool}
    for rarity in Rarity:
        if rarity not in present_rarities:
            base = copies_map[rarity]
            for card in all_cards:
                if card.rarity == rarity:
                    for _ in range(base):
                        pool.append(PoolEntry(card, staleness=0))


def remove_entry(pool: list[PoolEntry], entry: PoolEntry) -> None:
    """Remove one matching entry from the pool by identity."""
    for i, e in enumerate(pool):
        if e is entry:
            pool.pop(i)
            return


def increment_staleness(entries: list[PoolEntry]) -> None:
    """Increment staleness by 1 for each entry in the list."""
    for entry in entries:
        entry.staleness += 1
