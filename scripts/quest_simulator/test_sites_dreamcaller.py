"""Tests for the Dreamcaller Draft site interaction."""

import random
from types import MappingProxyType

from models import (
    Card,
    CardType,
    Dreamcaller,
    PoolEntry,
    Rarity,
    Resonance,
)


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


def _make_dreamcaller(
    name: str = "Test Caller",
    resonances: frozenset[Resonance] = frozenset({Resonance.TIDE}),
    resonance_bonus: dict[str, int] | None = None,
    tags: frozenset[str] = frozenset({"tribal:warrior"}),
    tag_bonus: dict[str, int] | None = None,
    essence_bonus: int = 50,
    ability_text: str = "Test ability",
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
        ability_text=ability_text,
    )


def _make_quest_state(
    essence: int = 250,
    seed: int = 42,
) -> "QuestState":
    from quest_state import QuestState

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
    )


class TestSelectDreamcallers:
    """Test that 3 dreamcallers are randomly selected from the full list."""

    def test_selects_three_dreamcallers(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        all_callers = [
            _make_dreamcaller(name=f"Caller {i}") for i in range(8)
        ]
        rng = random.Random(42)
        result = select_dreamcallers(all_callers, rng)
        assert len(result) == 3

    def test_all_selected_from_pool(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        all_callers = [
            _make_dreamcaller(name=f"Caller {i}") for i in range(8)
        ]
        rng = random.Random(42)
        result = select_dreamcallers(all_callers, rng)
        for dc in result:
            assert dc in all_callers

    def test_no_duplicates(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        all_callers = [
            _make_dreamcaller(name=f"Caller {i}") for i in range(8)
        ]
        rng = random.Random(42)
        result = select_dreamcallers(all_callers, rng)
        assert len(set(id(dc) for dc in result)) == 3

    def test_deterministic_with_same_seed(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        all_callers = [
            _make_dreamcaller(name=f"Caller {i}") for i in range(8)
        ]
        rng1 = random.Random(99)
        rng2 = random.Random(99)
        result1 = select_dreamcallers(all_callers, rng1)
        result2 = select_dreamcallers(all_callers, rng2)
        assert [dc.name for dc in result1] == [dc.name for dc in result2]

    def test_handles_exactly_three(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        all_callers = [
            _make_dreamcaller(name=f"Caller {i}") for i in range(3)
        ]
        rng = random.Random(42)
        result = select_dreamcallers(all_callers, rng)
        assert len(result) == 3


class TestFormatDreamcallerOption:
    """Test the display formatting of a dreamcaller option."""

    def test_returns_list_of_strings(self) -> None:
        from sites_dreamcaller import format_dreamcaller_option

        dc = _make_dreamcaller(
            name="Vesper, Twilight Arbiter",
            resonances=frozenset({Resonance.TIDE, Resonance.RUIN}),
            resonance_bonus={"Tide": 4, "Ruin": 4},
            tag_bonus={"mechanic:reclaim": 2, "mechanic:dissolve": 1},
            essence_bonus=50,
            ability_text="Whenever you dissolve an enemy character, draw a card.",
        )
        lines = format_dreamcaller_option(dc, highlighted=False)
        assert isinstance(lines, list)
        assert all(isinstance(line, str) for line in lines)
        assert len(lines) >= 3

    def test_highlighted_marker(self) -> None:
        from sites_dreamcaller import format_dreamcaller_option

        dc = _make_dreamcaller(name="Test Caller")
        lines_on = format_dreamcaller_option(dc, highlighted=True)
        lines_off = format_dreamcaller_option(dc, highlighted=False)
        # The highlighted version should have '>' marker, non-highlighted ' '
        assert any(">" in line for line in lines_on)
        # The non-highlighted version should NOT have '>' marker in the name line
        first_line_off = lines_off[0]
        # The name line for non-highlighted starts with spaces, not '>'
        assert first_line_off.lstrip().startswith(" ") or not first_line_off.lstrip().startswith(">")


class TestApplyDreamcaller:
    """Test that applying a dreamcaller updates quest state correctly."""

    def test_sets_dreamcaller_on_state(self) -> None:
        from sites_dreamcaller import apply_dreamcaller

        state = _make_quest_state()
        dc = _make_dreamcaller(name="Vesper")
        apply_dreamcaller(state, dc)
        assert state.dreamcaller is dc

    def test_applies_resonance_bonus(self) -> None:
        from sites_dreamcaller import apply_dreamcaller

        state = _make_quest_state()
        dc = _make_dreamcaller(
            resonance_bonus={"Tide": 4, "Ruin": 3},
        )
        apply_dreamcaller(state, dc)
        assert state.resonance_profile.counts[Resonance.TIDE] == 4
        assert state.resonance_profile.counts[Resonance.RUIN] == 3

    def test_applies_essence_bonus(self) -> None:
        from sites_dreamcaller import apply_dreamcaller

        state = _make_quest_state(essence=250)
        dc = _make_dreamcaller(essence_bonus=50)
        apply_dreamcaller(state, dc)
        assert state.essence == 300

    def test_applies_tag_bonus(self) -> None:
        from sites_dreamcaller import apply_dreamcaller

        state = _make_quest_state()
        dc = _make_dreamcaller(
            tags=frozenset({"mechanic:reclaim"}),
            tag_bonus={"mechanic:reclaim": 2},
        )
        apply_dreamcaller(state, dc)
        assert state.tag_profile.counts["mechanic:reclaim"] == 2
