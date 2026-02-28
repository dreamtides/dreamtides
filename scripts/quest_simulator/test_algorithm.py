"""Tests for quest simulator algorithm module."""

import random

from models import (
    AlgorithmParams,
    Card,
    CardType,
    PoolEntry,
    Rarity,
    Resonance,
    ResonanceProfile,
)


def _make_params(
    exponent: float = 1.4,
    floor_weight: float = 0.5,
    neutral_base: float = 3.0,
    staleness_factor: float = 0.3,
) -> AlgorithmParams:
    return AlgorithmParams(
        exponent=exponent,
        floor_weight=floor_weight,
        neutral_base=neutral_base,
        staleness_factor=staleness_factor,
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


class TestComputeWeight:
    def test_neutral_card_returns_neutral_base(self) -> None:
        from algorithm import compute_weight

        card = _make_card(resonances=frozenset())
        profile = ResonanceProfile()
        params = _make_params(neutral_base=3.0)
        assert compute_weight(card, profile, params) == 3.0

    def test_single_resonance_with_zero_profile(self) -> None:
        from algorithm import compute_weight

        card = _make_card(resonances=frozenset({Resonance.TIDE}))
        profile = ResonanceProfile()
        params = _make_params(floor_weight=0.5, exponent=1.4)
        # 0.5 + 0^1.4 = 0.5
        assert compute_weight(card, profile, params) == 0.5

    def test_single_resonance_with_profile_counts(self) -> None:
        from algorithm import compute_weight

        card = _make_card(resonances=frozenset({Resonance.TIDE}))
        profile = ResonanceProfile()
        profile.add(Resonance.TIDE, 5)
        params = _make_params(floor_weight=0.5, exponent=1.4)
        expected = 0.5 + 5**1.4
        result = compute_weight(card, profile, params)
        assert abs(result - expected) < 1e-9

    def test_dual_resonance_sums_affinities(self) -> None:
        from algorithm import compute_weight

        card = _make_card(resonances=frozenset({Resonance.TIDE, Resonance.RUIN}))
        profile = ResonanceProfile()
        profile.add(Resonance.TIDE, 3)
        profile.add(Resonance.RUIN, 2)
        params = _make_params(floor_weight=0.5, exponent=1.4)
        expected = 0.5 + 3**1.4 + 2**1.4
        result = compute_weight(card, profile, params)
        assert abs(result - expected) < 1e-9


class TestApplyStaleness:
    def test_zero_staleness_returns_original(self) -> None:
        from algorithm import apply_staleness

        assert apply_staleness(5.0, 0, 0.3) == 5.0

    def test_staleness_reduces_weight(self) -> None:
        from algorithm import apply_staleness

        result = apply_staleness(5.0, 2, 0.3)
        expected = 5.0 / (1.0 + 2 * 0.3)
        assert abs(result - expected) < 1e-9

    def test_high_staleness(self) -> None:
        from algorithm import apply_staleness

        result = apply_staleness(10.0, 10, 0.5)
        expected = 10.0 / (1.0 + 10 * 0.5)
        assert abs(result - expected) < 1e-9


class TestSelectCards:
    def test_select_from_empty_pool_returns_empty(self) -> None:
        from algorithm import select_cards

        profile = ResonanceProfile()
        params = _make_params()
        rng = random.Random(42)
        result = select_cards([], 4, profile, params, rng)
        assert result == []

    def test_select_returns_correct_count(self) -> None:
        from algorithm import select_cards

        cards = [
            _make_card(name=f"Card {i}", card_number=i, rarity=Rarity.UNCOMMON)
            for i in range(10)
        ]
        pool = [PoolEntry(c) for c in cards]
        profile = ResonanceProfile()
        params = _make_params()
        rng = random.Random(42)
        result = select_cards(pool, 4, profile, params, rng)
        assert len(result) == 4

    def test_select_returns_entries_and_weights(self) -> None:
        from algorithm import select_cards

        cards = [
            _make_card(name=f"Card {i}", card_number=i, rarity=Rarity.UNCOMMON)
            for i in range(5)
        ]
        pool = [PoolEntry(c) for c in cards]
        profile = ResonanceProfile()
        params = _make_params()
        rng = random.Random(42)
        result = select_cards(pool, 2, profile, params, rng)
        assert len(result) == 2
        for entry, weight in result:
            assert isinstance(entry, PoolEntry)
            assert isinstance(weight, float)
            assert weight > 0

    def test_rare_only_filters_pool(self) -> None:
        from algorithm import select_cards

        common_cards = [
            _make_card(name=f"Common {i}", card_number=i, rarity=Rarity.COMMON)
            for i in range(8)
        ]
        rare_cards = [
            _make_card(name=f"Rare {i}", card_number=100 + i, rarity=Rarity.RARE)
            for i in range(3)
        ]
        pool = [PoolEntry(c) for c in common_cards + rare_cards]
        profile = ResonanceProfile()
        params = _make_params()
        rng = random.Random(42)
        result = select_cards(pool, 2, profile, params, rng, rare_only=True)
        assert len(result) == 2
        for entry, _ in result:
            assert entry.card.rarity in (Rarity.RARE, Rarity.LEGENDARY)

    def test_staleness_affects_weights(self) -> None:
        from algorithm import select_cards

        fresh_card = _make_card(name="Fresh", card_number=1, rarity=Rarity.UNCOMMON)
        stale_card = _make_card(name="Stale", card_number=2, rarity=Rarity.UNCOMMON)
        fresh_entry = PoolEntry(fresh_card)
        stale_entry = PoolEntry(stale_card, staleness=10)
        pool = [fresh_entry, stale_entry]
        profile = ResonanceProfile()
        params = _make_params()
        rng = random.Random(42)
        result = select_cards(pool, 2, profile, params, rng)
        weights = {e.card.name: w for e, w in result}
        assert weights["Fresh"] > weights["Stale"]

    def test_minimum_weight_floor(self) -> None:
        from algorithm import select_cards

        card = _make_card(
            name="Zero Weight",
            card_number=1,
            resonances=frozenset({Resonance.TIDE}),
            rarity=Rarity.UNCOMMON,
        )
        entry = PoolEntry(card, staleness=1000)
        pool = [entry]
        profile = ResonanceProfile()
        params = _make_params()
        rng = random.Random(42)
        result = select_cards(pool, 1, profile, params, rng)
        assert len(result) == 1
        _, weight = result[0]
        assert weight >= 0.001

    def test_select_more_than_pool_size(self) -> None:
        from algorithm import select_cards

        cards = [
            _make_card(name=f"Card {i}", card_number=i, rarity=Rarity.UNCOMMON)
            for i in range(3)
        ]
        pool = [PoolEntry(c) for c in cards]
        profile = ResonanceProfile()
        params = _make_params()
        rng = random.Random(42)
        result = select_cards(pool, 10, profile, params, rng)
        assert len(result) == 3


class TestDiversityCheck:
    def test_all_same_resonance_gets_swapped(self) -> None:
        from algorithm import select_cards

        # Create 10 Tide cards (enough for pool) and 5 Ruin cards
        tide_cards = [
            _make_card(
                name=f"Tide {i}",
                card_number=i,
                resonances=frozenset({Resonance.TIDE}),
                rarity=Rarity.UNCOMMON,
            )
            for i in range(10)
        ]
        ruin_cards = [
            _make_card(
                name=f"Ruin {i}",
                card_number=100 + i,
                resonances=frozenset({Resonance.RUIN}),
                rarity=Rarity.UNCOMMON,
            )
            for i in range(5)
        ]
        pool = [PoolEntry(c) for c in tide_cards + ruin_cards]
        profile = ResonanceProfile()
        profile.add(Resonance.TIDE, 20)  # Heavy Tide bias
        params = _make_params()

        # Run many times -- diversity check should prevent all-Tide in n>=4
        all_same_count = 0
        for seed in range(50):
            rng = random.Random(seed)
            result = select_cards(pool, 4, profile, params, rng)
            resonance_sets = [e.card.resonances for e, _ in result]
            all_tide = all(Resonance.TIDE in rs for rs in resonance_sets)
            non_tide_exists = any(
                Resonance.TIDE not in rs for rs in resonance_sets
            )
            if all_tide and not non_tide_exists:
                all_same_count += 1
        # Most runs should have diversity
        assert all_same_count < 50

    def test_mixed_resonances_not_swapped(self) -> None:
        from algorithm import select_cards

        cards = [
            _make_card(
                name=f"Res {r.value}",
                card_number=i,
                resonances=frozenset({r}),
                rarity=Rarity.UNCOMMON,
            )
            for i, r in enumerate(Resonance)
        ]
        pool = [PoolEntry(c) for c in cards]
        profile = ResonanceProfile()
        for r in Resonance:
            profile.add(r, 5)
        params = _make_params()
        rng = random.Random(42)
        result = select_cards(pool, 4, profile, params, rng)
        assert len(result) == 4

    def test_small_selection_skips_diversity(self) -> None:
        """Selections with fewer than 4 cards skip the diversity check."""
        from algorithm import select_cards

        tide_cards = [
            _make_card(
                name=f"Tide {i}",
                card_number=i,
                resonances=frozenset({Resonance.TIDE}),
                rarity=Rarity.UNCOMMON,
            )
            for i in range(5)
        ]
        pool = [PoolEntry(c) for c in tide_cards]
        profile = ResonanceProfile()
        profile.add(Resonance.TIDE, 10)
        params = _make_params()
        rng = random.Random(42)
        result = select_cards(pool, 3, profile, params, rng)
        # All 3 can be Tide (no diversity check for n < 4)
        assert len(result) == 3


class TestRarityGuarantee:
    def test_all_common_gets_uncommon_swap(self) -> None:
        from algorithm import select_cards

        common_cards = [
            _make_card(
                name=f"Common {i}",
                card_number=i,
                resonances=frozenset({Resonance.TIDE}),
                rarity=Rarity.COMMON,
            )
            for i in range(10)
        ]
        uncommon_cards = [
            _make_card(
                name=f"Uncommon {i}",
                card_number=100 + i,
                resonances=frozenset({Resonance.RUIN}),
                rarity=Rarity.UNCOMMON,
            )
            for i in range(3)
        ]
        pool = [PoolEntry(c) for c in common_cards + uncommon_cards]
        profile = ResonanceProfile()
        profile.add(Resonance.TIDE, 20)  # Strong Tide bias to favor commons
        params = _make_params()

        # Over multiple seeds, rarity guarantee should ensure uncommon+ presence
        has_uncommon_count = 0
        for seed in range(50):
            rng = random.Random(seed)
            result = select_cards(pool, 4, profile, params, rng)
            has_uncommon = any(
                e.card.rarity in (Rarity.UNCOMMON, Rarity.RARE, Rarity.LEGENDARY)
                for e, _ in result
            )
            if has_uncommon:
                has_uncommon_count += 1
        # All runs should have at least 1 uncommon+
        assert has_uncommon_count == 50

    def test_already_has_uncommon_no_swap(self) -> None:
        from algorithm import select_cards

        mixed_cards = [
            _make_card(name="Common 1", card_number=1, rarity=Rarity.COMMON),
            _make_card(name="Common 2", card_number=2, rarity=Rarity.COMMON),
            _make_card(name="Uncommon", card_number=3, rarity=Rarity.UNCOMMON),
            _make_card(name="Common 3", card_number=4, rarity=Rarity.COMMON),
        ]
        pool = [PoolEntry(c) for c in mixed_cards]
        profile = ResonanceProfile()
        params = _make_params()
        rng = random.Random(42)
        result = select_cards(pool, 4, profile, params, rng)
        assert len(result) == 4

    def test_small_selection_skips_rarity_guarantee(self) -> None:
        """Selections with fewer than 4 cards skip rarity guarantee."""
        from algorithm import select_cards

        common_cards = [
            _make_card(name=f"Common {i}", card_number=i, rarity=Rarity.COMMON)
            for i in range(3)
        ]
        pool = [PoolEntry(c) for c in common_cards]
        profile = ResonanceProfile()
        params = _make_params()
        rng = random.Random(42)
        result = select_cards(pool, 3, profile, params, rng)
        # All 3 can be common (no rarity guarantee for n < 4)
        assert len(result) == 3
        assert all(e.card.rarity == Rarity.COMMON for e, _ in result)
