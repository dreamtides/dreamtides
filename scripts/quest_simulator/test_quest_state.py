"""Tests for quest simulator quest state module."""

import random

from models import (
    Dreamcaller,
    Dreamsign,
)
from quest_state import QuestState


def _make_dreamsign(
    name: str = "Test Sign",
) -> Dreamsign:
    return Dreamsign(
        name=name,
        effect_text="Test effect",
        is_bane=False,
    )


def _make_dreamcaller(
    name: str = "Test Caller",
    essence_bonus: int = 50,
) -> Dreamcaller:
    return Dreamcaller(
        name=name,
        archetype="Flash",
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
    rng = random.Random(seed)
    return QuestState(
        essence=essence,
        rng=rng,
        max_deck=max_deck,
        min_deck=min_deck,
        max_dreamsigns=max_dreamsigns,
    )


class TestQuestStateInit:
    def test_initial_fields(self) -> None:
        rng = random.Random(42)
        state = QuestState(
            essence=250,
            rng=rng,
        )
        assert state.essence == 250
        assert state.deck == []
        assert state.dreamsigns == []
        assert state.dreamcaller is None
        assert state.completion_level == 0
        assert state.rng is rng
        assert state.draft_strategy is None
        assert state.max_deck == 50
        assert state.min_deck == 25
        assert state.max_dreamsigns == 12

    def test_custom_limits(self) -> None:
        state = _make_quest_state(max_deck=40, min_deck=20, max_dreamsigns=8)
        assert state.max_deck == 40
        assert state.min_deck == 20
        assert state.max_dreamsigns == 8


class TestAddCard:
    def test_adds_deck_card(self) -> None:
        state = _make_quest_state()
        state.add_card("instance_placeholder")
        assert len(state.deck) == 1
        assert state.deck[0].instance == "instance_placeholder"
        assert state.deck[0].is_bane is False
        assert state.deck[0].is_transfigured is False

    def test_multiple_adds_accumulate(self) -> None:
        state = _make_quest_state()
        state.add_card("inst_a")
        state.add_card("inst_b")
        assert len(state.deck) == 2


class TestAddBaneCard:
    def test_adds_bane_deck_card(self) -> None:
        state = _make_quest_state()
        state.add_bane_card("bane_instance")
        assert len(state.deck) == 1
        assert state.deck[0].is_bane is True
        assert state.deck[0].instance == "bane_instance"


class TestRemoveCard:
    def test_removes_deck_card(self) -> None:
        state = _make_quest_state()
        state.add_card("inst")
        assert len(state.deck) == 1
        dc = state.deck[0]
        state.remove_card(dc)
        assert len(state.deck) == 0


class TestSetDreamcaller:
    def test_sets_dreamcaller(self) -> None:
        state = _make_quest_state()
        dc = _make_dreamcaller()
        state.set_dreamcaller(dc)
        assert state.dreamcaller is dc

    def test_adds_essence_bonus(self) -> None:
        state = _make_quest_state(essence=250)
        dc = _make_dreamcaller(essence_bonus=50)
        state.set_dreamcaller(dc)
        assert state.essence == 300

    def test_replacing_dreamcaller_removes_old_essence_bonus(self) -> None:
        state = _make_quest_state(essence=100)
        dc1 = _make_dreamcaller(name="First", essence_bonus=50)
        state.set_dreamcaller(dc1)
        assert state.essence == 150

        dc2 = _make_dreamcaller(name="Second", essence_bonus=30)
        state.set_dreamcaller(dc2)
        assert state.dreamcaller is dc2
        assert state.essence == 130

    def test_setting_same_dreamcaller_twice_is_idempotent(self) -> None:
        state = _make_quest_state(essence=100)
        dc = _make_dreamcaller(essence_bonus=50)
        state.set_dreamcaller(dc)
        state.set_dreamcaller(dc)
        assert state.essence == 150


class TestAddDreamsign:
    def test_adds_to_list(self) -> None:
        state = _make_quest_state()
        ds = _make_dreamsign()
        state.add_dreamsign(ds)
        assert len(state.dreamsigns) == 1
        assert state.dreamsigns[0] is ds


class TestRemoveDreamsign:
    def test_removes_from_list(self) -> None:
        state = _make_quest_state()
        ds = _make_dreamsign()
        state.add_dreamsign(ds)
        state.remove_dreamsign(ds)
        assert len(state.dreamsigns) == 0


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
            state.add_card(f"inst_{i}")
        assert state.is_over_deck_limit()

    def test_is_not_over_at_max(self) -> None:
        state = _make_quest_state(max_deck=3)
        for i in range(3):
            state.add_card(f"inst_{i}")
        assert not state.is_over_deck_limit()

    def test_is_under_deck_limit(self) -> None:
        state = _make_quest_state(min_deck=3)
        assert state.is_under_deck_limit()
        state.add_card("inst_1")
        assert state.is_under_deck_limit()
        state.add_card("inst_2")
        assert state.is_under_deck_limit()
        state.add_card("inst_3")
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
        state = _make_quest_state(min_deck=5)
        state.add_card("inst_a")
        state.add_card("inst_b")
        assert len(state.deck) == 2
        state.auto_fill_deck()
        assert len(state.deck) == 6

    def test_no_fill_when_already_exceeds_min(self) -> None:
        state = _make_quest_state(min_deck=2)
        state.add_card("inst_a")
        state.add_card("inst_b")
        state.add_card("inst_c")
        state.auto_fill_deck()
        assert len(state.deck) == 3

    def test_single_card_duplicated(self) -> None:
        state = _make_quest_state(min_deck=4)
        state.add_card("inst_only")
        state.auto_fill_deck()
        assert len(state.deck) == 5
        for dc in state.deck:
            assert dc.instance == "inst_only"

    def test_empty_deck_no_fill(self) -> None:
        state = _make_quest_state(min_deck=5)
        state.auto_fill_deck()
        assert len(state.deck) == 0


class TestDeckCount:
    def test_deck_count(self) -> None:
        state = _make_quest_state()
        assert state.deck_count() == 0
        state.add_card("inst")
        assert state.deck_count() == 1

    def test_dreamsign_count(self) -> None:
        state = _make_quest_state()
        assert state.dreamsign_count() == 0
        state.add_dreamsign(_make_dreamsign())
        assert state.dreamsign_count() == 1
