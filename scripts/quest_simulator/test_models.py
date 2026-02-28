"""Tests for quest simulator data models."""

import random
from types import MappingProxyType

from models import (
    BaneCard,
    CardType,
    Dreamcaller,
    EffectType,
    Journey,
    Resonance,
    ResonanceProfile,
    TagProfile,
    TemptingOffer,
)


class TestResonanceProfile:
    def test_initial_counts_are_zero(self) -> None:
        p = ResonanceProfile()
        for r in Resonance:
            assert p.counts[r] == 0
        assert p.total() == 0

    def test_add_increments(self) -> None:
        p = ResonanceProfile()
        p.add(Resonance.TIDE)
        assert p.counts[Resonance.TIDE] == 1
        p.add(Resonance.TIDE, 3)
        assert p.counts[Resonance.TIDE] == 4

    def test_remove_decrements_with_floor(self) -> None:
        p = ResonanceProfile()
        p.add(Resonance.EMBER, 5)
        p.remove(Resonance.EMBER, 3)
        assert p.counts[Resonance.EMBER] == 2
        p.remove(Resonance.EMBER, 10)
        assert p.counts[Resonance.EMBER] == 0

    def test_remove_never_goes_negative(self) -> None:
        p = ResonanceProfile()
        p.remove(Resonance.STONE, 5)
        assert p.counts[Resonance.STONE] == 0

    def test_total(self) -> None:
        p = ResonanceProfile()
        p.add(Resonance.TIDE, 3)
        p.add(Resonance.EMBER, 2)
        p.add(Resonance.RUIN, 1)
        assert p.total() == 6

    def test_top_n_returns_descending_order(self) -> None:
        p = ResonanceProfile()
        p.add(Resonance.TIDE, 10)
        p.add(Resonance.EMBER, 5)
        p.add(Resonance.ZEPHYR, 1)
        result = p.top_n(2)
        assert len(result) == 2
        assert result[0] == (Resonance.TIDE, 10)
        assert result[1] == (Resonance.EMBER, 5)

    def test_top_n_with_seeded_rng_is_deterministic(self) -> None:
        p = ResonanceProfile()
        p.add(Resonance.TIDE, 5)
        p.add(Resonance.EMBER, 5)
        p.add(Resonance.ZEPHYR, 5)
        rng = random.Random(42)
        result1 = p.top_n(2, rng=random.Random(42))
        result2 = p.top_n(2, rng=random.Random(42))
        assert result1 == result2

    def test_top_n_without_rng_works(self) -> None:
        p = ResonanceProfile()
        p.add(Resonance.TIDE, 10)
        result = p.top_n(1)
        assert result[0] == (Resonance.TIDE, 10)

    def test_snapshot_returns_copy(self) -> None:
        p = ResonanceProfile()
        p.add(Resonance.TIDE, 3)
        snap = p.snapshot()
        snap[Resonance.TIDE] = 999
        assert p.counts[Resonance.TIDE] == 3

    def test_copy_returns_independent_profile(self) -> None:
        p = ResonanceProfile()
        p.add(Resonance.TIDE, 5)
        p.add(Resonance.EMBER, 3)
        c = p.copy()
        c.add(Resonance.TIDE, 10)
        assert p.counts[Resonance.TIDE] == 5
        assert c.counts[Resonance.TIDE] == 15


class TestTagProfile:
    def test_initial_empty(self) -> None:
        t = TagProfile()
        assert t.counts == {}
        assert t.snapshot() == {}

    def test_add_creates_and_increments(self) -> None:
        t = TagProfile()
        t.add("tribal:warrior")
        assert t.counts["tribal:warrior"] == 1
        t.add("tribal:warrior", 2)
        assert t.counts["tribal:warrior"] == 3

    def test_remove_decrements_and_cleans_up(self) -> None:
        t = TagProfile()
        t.add("mechanic:draw", 3)
        t.remove("mechanic:draw", 1)
        assert t.counts["mechanic:draw"] == 2
        t.remove("mechanic:draw", 2)
        assert "mechanic:draw" not in t.counts

    def test_remove_missing_tag_is_safe(self) -> None:
        t = TagProfile()
        t.remove("nonexistent", 5)
        assert "nonexistent" not in t.counts

    def test_remove_clamps_to_zero(self) -> None:
        t = TagProfile()
        t.add("role:engine", 1)
        t.remove("role:engine", 100)
        assert "role:engine" not in t.counts

    def test_snapshot_returns_copy(self) -> None:
        t = TagProfile()
        t.add("tribal:mage", 2)
        snap = t.snapshot()
        snap["tribal:mage"] = 999
        assert t.counts["tribal:mage"] == 2

    def test_copy_returns_independent_profile(self) -> None:
        t = TagProfile()
        t.add("tribal:mage", 5)
        c = t.copy()
        c.add("tribal:mage", 10)
        assert t.counts["tribal:mage"] == 5
        assert c.counts["tribal:mage"] == 15


class TestEffectType:
    def test_all_journey_effect_types_exist(self) -> None:
        journey_types = [
            "add_cards", "add_essence", "remove_cards",
            "add_dreamsign", "gain_resonance",
        ]
        for et in journey_types:
            assert EffectType(et) is not None

    def test_all_offer_effect_types_exist(self) -> None:
        offer_types = [
            "add_cards", "add_essence", "add_dreamsign",
            "large_essence", "lose_essence", "add_bane_card",
            "add_bane_dreamsign", "remove_cards",
        ]
        for et in offer_types:
            assert EffectType(et) is not None


class TestFrozenImmutability:
    def test_dreamcaller_bonus_dicts_are_immutable(self) -> None:
        dc = Dreamcaller(
            name="Test",
            resonances=frozenset({Resonance.TIDE}),
            resonance_bonus=MappingProxyType({"Tide": 4}),
            tags=frozenset({"tribal:warrior"}),
            tag_bonus=MappingProxyType({"tribal:warrior": 2}),
            essence_bonus=50,
            ability_text="Test ability",
        )
        try:
            dc.resonance_bonus["new_key"] = 99  # type: ignore[index]
            assert False, "Should have raised TypeError"
        except TypeError:
            pass
        try:
            dc.tag_bonus["new_key"] = 99  # type: ignore[index]
            assert False, "Should have raised TypeError"
        except TypeError:
            pass

    def test_journey_uses_effect_type_enum(self) -> None:
        j = Journey(
            name="Test",
            description="Desc",
            effect_type=EffectType.ADD_CARDS,
            effect_value=1,
        )
        assert j.effect_type is EffectType.ADD_CARDS

    def test_bane_card_uses_card_type_enum(self) -> None:
        b = BaneCard(
            name="Test Bane",
            rules_text="Bad stuff",
            card_type=CardType.EVENT,
            energy_cost=99,
        )
        assert b.card_type is CardType.EVENT

    def test_tempting_offer_uses_effect_type_enum(self) -> None:
        t = TemptingOffer(
            reward_name="Reward",
            reward_description="Good",
            reward_effect_type=EffectType.ADD_CARDS,
            reward_value=2,
            cost_name="Cost",
            cost_description="Bad",
            cost_effect_type=EffectType.LOSE_ESSENCE,
            cost_value=100,
        )
        assert t.reward_effect_type is EffectType.ADD_CARDS
        assert t.cost_effect_type is EffectType.LOSE_ESSENCE
