"""Tests for quest simulator quest state module."""

import random
from collections import Counter
from types import MappingProxyType

from models import (
    Card,
    CardType,
    DeckCard,
    Dreamcaller,
    Dreamsign,
    PoolEntry,
    Rarity,
    Resonance,
    ResonanceProfile,
    TagProfile,
)
from quest_state import QuestState


def _make_card(
    name: str = "Test Card",
    card_number: int = 1,
    energy_cost: int = 3,
    resonances: frozenset[Resonance] = frozenset(),
    tags: frozenset[str] = frozenset(),
    rarity: Rarity = Rarity.COMMON,
    spark: int = 2,
    card_type: CardType = CardType.CHARACTER,
) -> Card:
    return Card(
        name=name,
        card_number=card_number,
        energy_cost=energy_cost,
        card_type=card_type,
        subtype=None,
        is_fast=False,
        spark=spark,
        rarity=rarity,
        rules_text="Test rules",
        resonances=resonances,
        tags=tags,
    )


def _make_dreamsign(
    name: str = "Test Sign",
    resonance: Resonance = Resonance.TIDE,
    tags: frozenset[str] = frozenset(),
) -> Dreamsign:
    return Dreamsign(
        name=name,
        resonance=resonance,
        tags=tags,
        effect_text="Test effect",
        is_bane=False,
    )


def _make_dreamcaller(
    name: str = "Test Caller",
    resonances: frozenset[Resonance] = frozenset({Resonance.TIDE}),
    resonance_bonus: dict[str, int] | None = None,
    tags: frozenset[str] = frozenset({"tribal:warrior"}),
    tag_bonus: dict[str, int] | None = None,
    essence_bonus: int = 50,
) -> Dreamcaller:
    if resonance_bonus is None:
        resonance_bonus = {"Tide": 4}
    if tag_bonus is None:
        tag_bonus = {"tribal:warrior": 2}
    return Dreamcaller(
        name=name,
        resonances=resonances,
        resonance_bonus=MappingProxyType(resonance_bonus),
        tags=tags,
        tag_bonus=MappingProxyType(tag_bonus),
        essence_bonus=essence_bonus,
        ability_text="Test ability",
    )


def _make_quest_state(
    essence: int = 250,
    seed: int = 42,
    max_deck: int = 50,
    min_deck: int = 25,
    max_dreamsigns: int = 12,
) -> QuestState:
    cards = [
        _make_card(name=f"Card{i}", card_number=i, rarity=Rarity.COMMON)
        for i in range(5)
    ]
    pool = [PoolEntry(c) for c in cards]
    rng = random.Random(seed)
    variance = {r: 1.0 for r in Resonance}
    return QuestState(
        essence=essence,
        pool=pool,
        rng=rng,
        all_cards=cards,
        pool_variance=variance,
        max_deck=max_deck,
        min_deck=min_deck,
        max_dreamsigns=max_dreamsigns,
    )


class TestQuestStateInit:
    def test_initial_fields(self) -> None:
        from quest_state import QuestState

        rng = random.Random(42)
        cards = [_make_card()]
        pool = [PoolEntry(cards[0])]
        variance = {r: 1.0 for r in Resonance}

        state = QuestState(
            essence=250,
            pool=pool,
            rng=rng,
            all_cards=cards,
            pool_variance=variance,
        )
        assert state.essence == 250
        assert state.deck == []
        assert state.dreamsigns == []
        assert state.dreamcaller is None
        assert state.completion_level == 0
        assert state.pool is pool
        assert state.rng is rng
        assert state.all_cards is cards
        assert state.pool_variance is variance
        assert state.max_deck == 50
        assert state.min_deck == 25
        assert state.max_dreamsigns == 12

    def test_custom_limits(self) -> None:
        state = _make_quest_state(max_deck=40, min_deck=20, max_dreamsigns=8)
        assert state.max_deck == 40
        assert state.min_deck == 20
        assert state.max_dreamsigns == 8

    def test_resonance_profile_starts_empty(self) -> None:
        state = _make_quest_state()
        for r in Resonance:
            assert state.resonance_profile.counts[r] == 0

    def test_tag_profile_starts_empty(self) -> None:
        state = _make_quest_state()
        assert state.tag_profile.counts == {}


class TestAddCard:
    def test_adds_deck_card(self) -> None:
        state = _make_quest_state()
        card = _make_card(resonances=frozenset({Resonance.TIDE}))
        state.add_card(card)
        assert len(state.deck) == 1
        assert state.deck[0].card is card
        assert state.deck[0].is_bane is False
        assert state.deck[0].is_transfigured is False

    def test_updates_resonance_profile(self) -> None:
        state = _make_quest_state()
        card = _make_card(resonances=frozenset({Resonance.TIDE, Resonance.RUIN}))
        state.add_card(card)
        assert state.resonance_profile.counts[Resonance.TIDE] == 1
        assert state.resonance_profile.counts[Resonance.RUIN] == 1
        assert state.resonance_profile.counts[Resonance.EMBER] == 0

    def test_updates_tag_profile(self) -> None:
        state = _make_quest_state()
        card = _make_card(tags=frozenset({"tribal:warrior", "role:finisher"}))
        state.add_card(card)
        assert state.tag_profile.counts["tribal:warrior"] == 1
        assert state.tag_profile.counts["role:finisher"] == 1

    def test_multiple_adds_accumulate(self) -> None:
        state = _make_quest_state()
        card1 = _make_card(
            name="A",
            card_number=1,
            resonances=frozenset({Resonance.TIDE}),
            tags=frozenset({"tribal:warrior"}),
        )
        card2 = _make_card(
            name="B",
            card_number=2,
            resonances=frozenset({Resonance.TIDE}),
            tags=frozenset({"tribal:warrior"}),
        )
        state.add_card(card1)
        state.add_card(card2)
        assert len(state.deck) == 2
        assert state.resonance_profile.counts[Resonance.TIDE] == 2
        assert state.tag_profile.counts["tribal:warrior"] == 2


class TestAddBaneCard:
    def test_adds_bane_deck_card(self) -> None:
        state = _make_quest_state()
        card = _make_card(name="Bane", resonances=frozenset(), tags=frozenset())
        state.add_bane_card(card)
        assert len(state.deck) == 1
        assert state.deck[0].is_bane is True
        assert state.deck[0].card is card

    def test_bane_card_does_not_update_profiles(self) -> None:
        state = _make_quest_state()
        card = _make_card(
            name="Bane",
            resonances=frozenset({Resonance.TIDE}),
            tags=frozenset({"tribal:warrior"}),
        )
        state.add_bane_card(card)
        # Bane cards have no resonance/tags to track per task spec
        assert state.resonance_profile.counts[Resonance.TIDE] == 0
        assert state.tag_profile.counts == {}


class TestRemoveCard:
    def test_removes_deck_card(self) -> None:
        state = _make_quest_state()
        card = _make_card(resonances=frozenset({Resonance.TIDE}))
        state.add_card(card)
        assert len(state.deck) == 1
        dc = state.deck[0]
        state.remove_card(dc)
        assert len(state.deck) == 0

    def test_updates_resonance_profile_on_remove(self) -> None:
        state = _make_quest_state()
        card = _make_card(resonances=frozenset({Resonance.TIDE, Resonance.RUIN}))
        state.add_card(card)
        dc = state.deck[0]
        state.remove_card(dc)
        assert state.resonance_profile.counts[Resonance.TIDE] == 0
        assert state.resonance_profile.counts[Resonance.RUIN] == 0

    def test_updates_tag_profile_on_remove(self) -> None:
        state = _make_quest_state()
        card = _make_card(tags=frozenset({"tribal:warrior"}))
        state.add_card(card)
        dc = state.deck[0]
        state.remove_card(dc)
        assert "tribal:warrior" not in state.tag_profile.counts


class TestProfileConsistency:
    def test_add_then_remove_leaves_profiles_unchanged(self) -> None:
        state = _make_quest_state()
        card = _make_card(
            resonances=frozenset({Resonance.TIDE, Resonance.EMBER}),
            tags=frozenset({"tribal:warrior", "mechanic:draw", "role:engine"}),
        )
        initial_resonance = state.resonance_profile.snapshot()
        initial_tags = state.tag_profile.snapshot()

        state.add_card(card)
        dc = state.deck[0]
        state.remove_card(dc)

        assert state.resonance_profile.snapshot() == initial_resonance
        assert state.tag_profile.snapshot() == initial_tags

    def test_add_multiple_remove_all_leaves_profiles_unchanged(self) -> None:
        state = _make_quest_state()
        cards = [
            _make_card(
                name=f"C{i}",
                card_number=i,
                resonances=frozenset({Resonance.TIDE}),
                tags=frozenset({"tribal:warrior"}),
            )
            for i in range(5)
        ]
        initial_resonance = state.resonance_profile.snapshot()
        initial_tags = state.tag_profile.snapshot()

        for c in cards:
            state.add_card(c)
        for dc in list(state.deck):
            state.remove_card(dc)

        assert state.resonance_profile.snapshot() == initial_resonance
        assert state.tag_profile.snapshot() == initial_tags


class TestSetDreamcaller:
    def test_sets_dreamcaller(self) -> None:
        state = _make_quest_state()
        dc = _make_dreamcaller()
        state.set_dreamcaller(dc)
        assert state.dreamcaller is dc

    def test_applies_resonance_bonus(self) -> None:
        state = _make_quest_state()
        dc = _make_dreamcaller(
            resonance_bonus={"Tide": 4, "Ruin": 3},
        )
        state.set_dreamcaller(dc)
        assert state.resonance_profile.counts[Resonance.TIDE] == 4
        assert state.resonance_profile.counts[Resonance.RUIN] == 3

    def test_applies_tag_bonus(self) -> None:
        state = _make_quest_state()
        dc = _make_dreamcaller(
            tags=frozenset({"mechanic:reclaim"}),
            tag_bonus={"mechanic:reclaim": 2},
        )
        state.set_dreamcaller(dc)
        assert state.tag_profile.counts["mechanic:reclaim"] == 2

    def test_adds_essence_bonus(self) -> None:
        state = _make_quest_state(essence=250)
        dc = _make_dreamcaller(essence_bonus=50)
        state.set_dreamcaller(dc)
        assert state.essence == 300

    def test_replacing_dreamcaller_removes_old_bonuses(self) -> None:
        state = _make_quest_state(essence=100)
        dc1 = _make_dreamcaller(
            name="First",
            resonance_bonus={"Tide": 4},
            tag_bonus={"tribal:warrior": 2},
            essence_bonus=50,
        )
        state.set_dreamcaller(dc1)
        assert state.resonance_profile.counts[Resonance.TIDE] == 4
        assert state.tag_profile.counts["tribal:warrior"] == 2
        assert state.essence == 150

        dc2 = _make_dreamcaller(
            name="Second",
            resonance_bonus={"Ruin": 3},
            tag_bonus={"mechanic:dissolve": 1},
            essence_bonus=30,
        )
        state.set_dreamcaller(dc2)
        assert state.dreamcaller is dc2
        assert state.resonance_profile.counts[Resonance.TIDE] == 0
        assert state.resonance_profile.counts[Resonance.RUIN] == 3
        assert "tribal:warrior" not in state.tag_profile.counts
        assert state.tag_profile.counts["mechanic:dissolve"] == 1
        assert state.essence == 130

    def test_setting_same_dreamcaller_twice_is_idempotent(self) -> None:
        state = _make_quest_state(essence=100)
        dc = _make_dreamcaller(
            resonance_bonus={"Tide": 4},
            tag_bonus={"tribal:warrior": 2},
            essence_bonus=50,
        )
        state.set_dreamcaller(dc)
        state.set_dreamcaller(dc)
        assert state.resonance_profile.counts[Resonance.TIDE] == 4
        assert state.tag_profile.counts["tribal:warrior"] == 2
        assert state.essence == 150


class TestAddDreamsign:
    def test_adds_to_list(self) -> None:
        state = _make_quest_state()
        ds = _make_dreamsign()
        state.add_dreamsign(ds)
        assert len(state.dreamsigns) == 1
        assert state.dreamsigns[0] is ds

    def test_updates_resonance_profile(self) -> None:
        state = _make_quest_state()
        ds = _make_dreamsign(resonance=Resonance.RUIN)
        state.add_dreamsign(ds)
        assert state.resonance_profile.counts[Resonance.RUIN] == 1

    def test_updates_tag_profile(self) -> None:
        state = _make_quest_state()
        ds = _make_dreamsign(tags=frozenset({"mechanic:dissolve"}))
        state.add_dreamsign(ds)
        assert state.tag_profile.counts["mechanic:dissolve"] == 1


class TestRemoveDreamsign:
    def test_removes_from_list(self) -> None:
        state = _make_quest_state()
        ds = _make_dreamsign()
        state.add_dreamsign(ds)
        state.remove_dreamsign(ds)
        assert len(state.dreamsigns) == 0

    def test_updates_resonance_profile(self) -> None:
        state = _make_quest_state()
        ds = _make_dreamsign(resonance=Resonance.RUIN)
        state.add_dreamsign(ds)
        state.remove_dreamsign(ds)
        assert state.resonance_profile.counts[Resonance.RUIN] == 0

    def test_updates_tag_profile(self) -> None:
        state = _make_quest_state()
        ds = _make_dreamsign(tags=frozenset({"mechanic:dissolve"}))
        state.add_dreamsign(ds)
        state.remove_dreamsign(ds)
        assert "mechanic:dissolve" not in state.tag_profile.counts


class TestEssence:
    def test_spend_essence(self) -> None:
        state = _make_quest_state(essence=250)
        state.spend_essence(100)
        assert state.essence == 150

    def test_spend_all_essence(self) -> None:
        state = _make_quest_state(essence=100)
        state.spend_essence(100)
        assert state.essence == 0

    def test_spend_essence_raises_value_error(self) -> None:
        state = _make_quest_state(essence=50)
        try:
            state.spend_essence(100)
            assert False, "Should have raised ValueError"
        except ValueError:
            pass

    def test_gain_essence(self) -> None:
        state = _make_quest_state(essence=100)
        state.gain_essence(50)
        assert state.essence == 150


class TestCompletionLevel:
    def test_increment_completion(self) -> None:
        state = _make_quest_state()
        assert state.completion_level == 0
        state.increment_completion()
        assert state.completion_level == 1
        state.increment_completion()
        assert state.completion_level == 2


class TestDeckLimits:
    def test_is_over_deck_limit(self) -> None:
        state = _make_quest_state(max_deck=3)
        assert not state.is_over_deck_limit()
        for i in range(4):
            state.add_card(_make_card(name=f"C{i}", card_number=i))
        assert state.is_over_deck_limit()

    def test_is_not_over_at_max(self) -> None:
        state = _make_quest_state(max_deck=3)
        for i in range(3):
            state.add_card(_make_card(name=f"C{i}", card_number=i))
        assert not state.is_over_deck_limit()

    def test_is_under_deck_limit(self) -> None:
        state = _make_quest_state(min_deck=3)
        assert state.is_under_deck_limit()
        state.add_card(_make_card(name="C1", card_number=1))
        assert state.is_under_deck_limit()
        state.add_card(_make_card(name="C2", card_number=2))
        assert state.is_under_deck_limit()
        state.add_card(_make_card(name="C3", card_number=3))
        assert not state.is_under_deck_limit()

    def test_is_over_dreamsign_limit(self) -> None:
        state = _make_quest_state(max_dreamsigns=2)
        assert not state.is_over_dreamsign_limit()
        state.add_dreamsign(_make_dreamsign(name="DS1"))
        assert not state.is_over_dreamsign_limit()
        state.add_dreamsign(_make_dreamsign(name="DS2"))
        assert state.is_over_dreamsign_limit()


class TestAutoFillDeck:
    def test_duplicates_whole_deck_to_exceed_min(self) -> None:
        """With 2 cards and min_deck=5, whole-deck duplication gives 6 (3 copies)."""
        state = _make_quest_state(min_deck=5)
        card1 = _make_card(name="A", card_number=1)
        card2 = _make_card(name="B", card_number=2)
        state.add_card(card1)
        state.add_card(card2)
        assert len(state.deck) == 2
        state.auto_fill_deck()
        # 2 * 3 = 6 > 5
        assert len(state.deck) == 6

    def test_nine_cards_become_twenty_seven(self) -> None:
        """Design doc example: 9 cards should become 27 (3 copies) with min_deck=25."""
        state = _make_quest_state(min_deck=25)
        for i in range(9):
            state.add_card(_make_card(name=f"Card{i}", card_number=i))
        state.auto_fill_deck()
        # 9 * 3 = 27 > 25
        assert len(state.deck) == 27

    def test_no_fill_when_already_exceeds_min(self) -> None:
        """When deck already exceeds min_deck, no duplication happens."""
        state = _make_quest_state(min_deck=2)
        state.add_card(_make_card(name="A", card_number=1))
        state.add_card(_make_card(name="B", card_number=2))
        state.add_card(_make_card(name="C", card_number=3))
        state.auto_fill_deck()
        assert len(state.deck) == 3

    def test_single_card_duplicated_whole_deck(self) -> None:
        """With 1 card and min_deck=4, whole-deck duplication gives copies of the one card."""
        state = _make_quest_state(min_deck=4)
        card = _make_card(name="Only Card", card_number=1)
        state.add_card(card)
        state.auto_fill_deck()
        # 1 card: 1, 2, 3, 4, 5 -> first exceeding 4 is 5
        assert len(state.deck) == 5
        for dc in state.deck:
            assert dc.card.name == "Only Card"

    def test_whole_deck_preserves_card_composition(self) -> None:
        """Each copy of the deck should contain all original cards in order."""
        state = _make_quest_state(min_deck=7)
        card_a = _make_card(name="A", card_number=1)
        card_b = _make_card(name="B", card_number=2)
        card_c = _make_card(name="C", card_number=3)
        state.add_card(card_a)
        state.add_card(card_b)
        state.add_card(card_c)
        state.auto_fill_deck()
        # 3 * 3 = 9 > 7
        assert len(state.deck) == 9
        names = [dc.card.name for dc in state.deck]
        assert names == ["A", "B", "C", "A", "B", "C", "A", "B", "C"]

    def test_auto_fill_updates_profiles(self) -> None:
        state = _make_quest_state(min_deck=3)
        card = _make_card(
            name="Res Card",
            card_number=1,
            resonances=frozenset({Resonance.TIDE}),
            tags=frozenset({"tribal:warrior"}),
        )
        state.add_card(card)
        assert state.resonance_profile.counts[Resonance.TIDE] == 1
        assert state.tag_profile.counts["tribal:warrior"] == 1
        state.auto_fill_deck()
        # 1 card, min_deck=3: 1, 2, 3, 4 -> first exceeding 3 is 4
        assert len(state.deck) == 4
        assert state.resonance_profile.counts[Resonance.TIDE] == 4
        assert state.tag_profile.counts["tribal:warrior"] == 4

    def test_empty_deck_no_fill(self) -> None:
        """An empty deck should not cause infinite loop."""
        state = _make_quest_state(min_deck=5)
        state.auto_fill_deck()
        assert len(state.deck) == 0


class TestDeckCount:
    def test_deck_count(self) -> None:
        state = _make_quest_state()
        assert state.deck_count() == 0
        state.add_card(_make_card())
        assert state.deck_count() == 1

    def test_dreamsign_count(self) -> None:
        state = _make_quest_state()
        assert state.dreamsign_count() == 0
        state.add_dreamsign(_make_dreamsign())
        assert state.dreamsign_count() == 1


class TestDeckByRarity:
    def test_deck_by_rarity(self) -> None:
        state = _make_quest_state()
        state.add_card(_make_card(name="C1", card_number=1, rarity=Rarity.COMMON))
        state.add_card(_make_card(name="C2", card_number=2, rarity=Rarity.COMMON))
        state.add_card(_make_card(name="U1", card_number=3, rarity=Rarity.UNCOMMON))
        state.add_card(_make_card(name="R1", card_number=4, rarity=Rarity.RARE))
        result = state.deck_by_rarity()
        assert result[Rarity.COMMON] == 2
        assert result[Rarity.UNCOMMON] == 1
        assert result[Rarity.RARE] == 1
        assert result[Rarity.LEGENDARY] == 0

    def test_empty_deck_all_zero(self) -> None:
        state = _make_quest_state()
        result = state.deck_by_rarity()
        for r in Rarity:
            assert result[r] == 0
