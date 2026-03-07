"""Tests for quest simulator data models."""

from models import (
    BaneCard,
    DeckCard,
    Dreamcaller,
    EffectType,
    Journey,
    TemptingOffer,
)


class TestEffectType:
    def test_all_journey_effect_types_exist(self) -> None:
        journey_types = [
            "add_cards",
            "add_essence",
            "remove_cards",
            "add_dreamsign",
        ]
        for et in journey_types:
            assert EffectType(et) is not None

    def test_gain_resonance_removed(self) -> None:
        """GAIN_RESONANCE was removed from EffectType."""
        for et in EffectType:
            assert et.value != "gain_resonance"

    def test_all_offer_effect_types_exist(self) -> None:
        offer_types = [
            "add_cards",
            "add_essence",
            "add_dreamsign",
            "large_essence",
            "lose_essence",
            "add_bane_card",
            "add_bane_dreamsign",
            "remove_cards",
        ]
        for et in offer_types:
            assert EffectType(et) is not None


class TestDeckCard:
    def test_deck_card_holds_instance(self) -> None:
        dc = DeckCard(instance="placeholder")
        assert dc.instance == "placeholder"
        assert dc.is_transfigured is False
        assert dc.is_bane is False
        assert dc.transfig_note is None

    def test_deck_card_bane_flag(self) -> None:
        dc = DeckCard(instance="placeholder", is_bane=True)
        assert dc.is_bane is True


class TestDreamcaller:
    def test_dreamcaller_fields(self) -> None:
        dc = Dreamcaller(
            name="Test",
            archetype="Flash",
            essence_bonus=50,
            ability_text="Test ability",
        )
        assert dc.name == "Test"
        assert dc.archetype == "Flash"
        assert dc.essence_bonus == 50
        assert dc.ability_text == "Test ability"


class TestBaneCard:
    def test_bane_card_type_is_str(self) -> None:
        b = BaneCard(
            name="Test Bane",
            rules_text="Bad stuff",
            card_type="Event",
            energy_cost=99,
        )
        assert b.card_type == "Event"


class TestFrozenImmutability:
    def test_journey_uses_effect_type_enum(self) -> None:
        j = Journey(
            name="Test",
            description="Desc",
            effect_type=EffectType.ADD_CARDS,
            effect_value=1,
        )
        assert j.effect_type is EffectType.ADD_CARDS

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
