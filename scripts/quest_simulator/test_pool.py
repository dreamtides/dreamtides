"""Tests for quest simulator pool module."""

import random

from models import (
    Card,
    CardType,
    PoolEntry,
    PoolParams,
    Rarity,
    Resonance,
)


def _make_pool_params(
    copies_common: int = 4,
    copies_uncommon: int = 3,
    copies_rare: int = 2,
    copies_legendary: int = 1,
    variance_min: float = 0.75,
    variance_max: float = 1.25,
) -> PoolParams:
    return PoolParams(
        copies_common=copies_common,
        copies_uncommon=copies_uncommon,
        copies_rare=copies_rare,
        copies_legendary=copies_legendary,
        variance_min=variance_min,
        variance_max=variance_max,
    )


def _make_card(
    name: str = "Test Card",
    card_number: int = 1,
    resonances: frozenset[Resonance] = frozenset(),
    rarity: Rarity = Rarity.COMMON,
) -> Card:
    return Card(
        name=name,
        card_number=card_number,
        energy_cost=3,
        card_type=CardType.CHARACTER,
        subtype=None,
        is_fast=False,
        spark=2,
        rarity=rarity,
        rules_text="Test rules",
        resonances=resonances,
        tags=frozenset(),
    )


class TestGenerateVariance:
    def test_returns_all_resonances(self) -> None:
        from pool import generate_variance

        rng = random.Random(42)
        params = _make_pool_params()
        result = generate_variance(rng, params)
        assert set(result.keys()) == set(Resonance)

    def test_values_within_bounds(self) -> None:
        from pool import generate_variance

        rng = random.Random(42)
        params = _make_pool_params(variance_min=0.75, variance_max=1.25)
        result = generate_variance(rng, params)
        for v in result.values():
            assert 0.75 <= v <= 1.25

    def test_deterministic_with_same_seed(self) -> None:
        from pool import generate_variance

        params = _make_pool_params()
        result1 = generate_variance(random.Random(42), params)
        result2 = generate_variance(random.Random(42), params)
        assert result1 == result2


class TestBuildPool:
    def test_neutral_card_gets_base_copies(self) -> None:
        from pool import build_pool

        card = _make_card(rarity=Rarity.COMMON, resonances=frozenset())
        params = _make_pool_params(copies_common=4)
        variance = {r: 1.0 for r in Resonance}
        result = build_pool([card], params, variance)
        assert len(result) == 4
        assert all(e.card == card for e in result)

    def test_uncommon_card_copy_count(self) -> None:
        from pool import build_pool

        card = _make_card(rarity=Rarity.UNCOMMON, resonances=frozenset())
        params = _make_pool_params(copies_uncommon=3)
        variance = {r: 1.0 for r in Resonance}
        result = build_pool([card], params, variance)
        assert len(result) == 3

    def test_rare_card_copy_count(self) -> None:
        from pool import build_pool

        card = _make_card(rarity=Rarity.RARE, resonances=frozenset())
        params = _make_pool_params(copies_rare=2)
        variance = {r: 1.0 for r in Resonance}
        result = build_pool([card], params, variance)
        assert len(result) == 2

    def test_legendary_card_copy_count(self) -> None:
        from pool import build_pool

        card = _make_card(rarity=Rarity.LEGENDARY, resonances=frozenset())
        params = _make_pool_params(copies_legendary=1)
        variance = {r: 1.0 for r in Resonance}
        result = build_pool([card], params, variance)
        assert len(result) == 1

    def test_variance_increases_copies(self) -> None:
        from pool import build_pool

        card = _make_card(
            rarity=Rarity.COMMON,
            resonances=frozenset({Resonance.TIDE}),
        )
        params = _make_pool_params(copies_common=4)
        variance = {r: 1.25 for r in Resonance}
        result = build_pool([card], params, variance)
        # 4 * 1.25 = 5.0 -> round to 5
        assert len(result) == 5

    def test_variance_decreases_copies(self) -> None:
        from pool import build_pool

        card = _make_card(
            rarity=Rarity.COMMON,
            resonances=frozenset({Resonance.TIDE}),
        )
        params = _make_pool_params(copies_common=4)
        variance = {r: 0.75 for r in Resonance}
        result = build_pool([card], params, variance)
        # 4 * 0.75 = 3.0 -> round to 3
        assert len(result) == 3

    def test_minimum_one_copy(self) -> None:
        from pool import build_pool

        card = _make_card(
            rarity=Rarity.LEGENDARY,
            resonances=frozenset({Resonance.TIDE}),
        )
        params = _make_pool_params(copies_legendary=1)
        variance = {r: 0.1 for r in Resonance}  # Very low variance
        result = build_pool([card], params, variance)
        # max(1, round(1 * 0.1)) = max(1, 0) = 1
        assert len(result) == 1

    def test_dual_resonance_averages_variance(self) -> None:
        from pool import build_pool

        card = _make_card(
            rarity=Rarity.COMMON,
            resonances=frozenset({Resonance.TIDE, Resonance.RUIN}),
        )
        params = _make_pool_params(copies_common=4)
        variance = {
            Resonance.TIDE: 1.0,
            Resonance.RUIN: 1.5,
            Resonance.EMBER: 1.0,
            Resonance.ZEPHYR: 1.0,
            Resonance.STONE: 1.0,
        }
        result = build_pool([card], params, variance)
        # average of 1.0 and 1.5 = 1.25, 4 * 1.25 = 5.0 -> 5
        assert len(result) == 5

    def test_all_entries_have_zero_staleness(self) -> None:
        from pool import build_pool

        card = _make_card(rarity=Rarity.COMMON)
        params = _make_pool_params(copies_common=4)
        variance = {r: 1.0 for r in Resonance}
        result = build_pool([card], params, variance)
        assert all(e.staleness == 0 for e in result)

    def test_pool_size_approximately_660_for_220_cards(self) -> None:
        from pool import build_pool

        # Create 220 cards with realistic rarity distribution:
        # ~110 Common, ~60 Uncommon, ~35 Rare, ~15 Legendary
        cards: list[Card] = []
        for i in range(110):
            cards.append(_make_card(name=f"C{i}", card_number=i, rarity=Rarity.COMMON))
        for i in range(60):
            cards.append(
                _make_card(name=f"U{i}", card_number=110 + i, rarity=Rarity.UNCOMMON)
            )
        for i in range(35):
            cards.append(
                _make_card(name=f"R{i}", card_number=170 + i, rarity=Rarity.RARE)
            )
        for i in range(15):
            cards.append(
                _make_card(name=f"L{i}", card_number=205 + i, rarity=Rarity.LEGENDARY)
            )
        params = _make_pool_params()
        # With all variance at 1.0, exact count:
        # 110*4 + 60*3 + 35*2 + 15*1 = 440 + 180 + 70 + 15 = 705
        # With real variance it varies, but approximately
        variance = {r: 1.0 for r in Resonance}
        result = build_pool(cards, params, variance)
        assert len(result) == 705  # Exact with 1.0 variance


class TestDecayStaleness:
    def test_decrements_staleness(self) -> None:
        from pool import decay_staleness

        card = _make_card()
        pool = [PoolEntry(card, staleness=3), PoolEntry(card, staleness=5)]
        decay_staleness(pool)
        assert pool[0].staleness == 2
        assert pool[1].staleness == 4

    def test_staleness_does_not_go_below_zero(self) -> None:
        from pool import decay_staleness

        card = _make_card()
        pool = [PoolEntry(card, staleness=0), PoolEntry(card, staleness=1)]
        decay_staleness(pool)
        assert pool[0].staleness == 0
        assert pool[1].staleness == 0

    def test_empty_pool_does_not_error(self) -> None:
        from pool import decay_staleness

        pool: list[PoolEntry] = []
        decay_staleness(pool)
        assert pool == []


class TestRefillPool:
    def test_no_refill_when_all_rarities_present(self) -> None:
        from pool import refill_pool

        cards = [
            _make_card(name="C1", card_number=1, rarity=Rarity.COMMON),
            _make_card(name="U1", card_number=2, rarity=Rarity.UNCOMMON),
            _make_card(name="R1", card_number=3, rarity=Rarity.RARE),
            _make_card(name="L1", card_number=4, rarity=Rarity.LEGENDARY),
        ]
        pool = [PoolEntry(c) for c in cards]
        params = _make_pool_params()
        initial_len = len(pool)
        refill_pool(pool, cards, params)
        assert len(pool) == initial_len

    def test_refill_exhausted_rarity(self) -> None:
        from pool import refill_pool

        common_card = _make_card(name="C1", card_number=1, rarity=Rarity.COMMON)
        uncommon_card = _make_card(name="U1", card_number=2, rarity=Rarity.UNCOMMON)
        all_cards = [common_card, uncommon_card]
        # Pool has only uncommon, common is exhausted
        pool = [PoolEntry(uncommon_card)]
        params = _make_pool_params(copies_common=4)
        refill_pool(pool, all_cards, params)
        # Should have added 4 common entries back
        assert len(pool) == 5  # 1 uncommon + 4 common
        common_entries = [e for e in pool if e.card.rarity == Rarity.COMMON]
        assert len(common_entries) == 4

    def test_refill_entries_have_zero_staleness(self) -> None:
        from pool import refill_pool

        common_card = _make_card(name="C1", card_number=1, rarity=Rarity.COMMON)
        uncommon_card = _make_card(name="U1", card_number=2, rarity=Rarity.UNCOMMON)
        pool = [PoolEntry(uncommon_card, staleness=5)]
        params = _make_pool_params(copies_common=4)
        refill_pool(pool, [common_card, uncommon_card], params)
        new_entries = [e for e in pool if e.card.rarity == Rarity.COMMON]
        assert all(e.staleness == 0 for e in new_entries)

    def test_refill_multiple_exhausted_rarities(self) -> None:
        from pool import refill_pool

        common_card = _make_card(name="C1", card_number=1, rarity=Rarity.COMMON)
        uncommon_card = _make_card(name="U1", card_number=2, rarity=Rarity.UNCOMMON)
        rare_card = _make_card(name="R1", card_number=3, rarity=Rarity.RARE)
        all_cards = [common_card, uncommon_card, rare_card]
        # Pool has only rare; common and uncommon are exhausted
        pool = [PoolEntry(rare_card)]
        params = _make_pool_params(copies_common=4, copies_uncommon=3)
        refill_pool(pool, all_cards, params)
        # Should add 4 common + 3 uncommon
        assert len(pool) == 8  # 1 rare + 4 common + 3 uncommon


class TestRemoveEntry:
    def test_removes_one_matching_entry(self) -> None:
        from pool import remove_entry

        card = _make_card()
        e1 = PoolEntry(card)
        e2 = PoolEntry(card)
        pool = [e1, e2]
        remove_entry(pool, e1)
        assert len(pool) == 1
        assert pool[0] is e2

    def test_remove_from_larger_pool(self) -> None:
        from pool import remove_entry

        card_a = _make_card(name="A", card_number=1)
        card_b = _make_card(name="B", card_number=2)
        e_a = PoolEntry(card_a)
        e_b1 = PoolEntry(card_b)
        e_b2 = PoolEntry(card_b)
        pool = [e_a, e_b1, e_b2]
        remove_entry(pool, e_b1)
        assert len(pool) == 2
        assert e_a in pool
        assert e_b2 in pool

    def test_remove_nonexistent_entry_no_error(self) -> None:
        from pool import remove_entry

        card = _make_card()
        e1 = PoolEntry(card)
        e_missing = PoolEntry(card, staleness=99)
        pool = [e1]
        # Removing a non-present entry should not crash
        remove_entry(pool, e_missing)
        assert len(pool) == 1


class TestIncrementStaleness:
    def test_increments_by_one(self) -> None:
        from pool import increment_staleness

        card = _make_card()
        entries = [PoolEntry(card, staleness=0), PoolEntry(card, staleness=3)]
        increment_staleness(entries)
        assert entries[0].staleness == 1
        assert entries[1].staleness == 4

    def test_empty_list_no_error(self) -> None:
        from pool import increment_staleness

        entries: list[PoolEntry] = []
        increment_staleness(entries)
        assert entries == []
