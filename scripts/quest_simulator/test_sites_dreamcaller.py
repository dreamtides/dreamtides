"""Tests for the Dreamcaller Draft site interaction."""

import random
from types import MappingProxyType
from unittest.mock import patch

from models import (
    Card,
    CardType,
    Dreamcaller,
    PoolEntry,
    Rarity,
    Resonance,
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

    def test_handles_fewer_than_three(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        all_callers = [
            _make_dreamcaller(name=f"Caller {i}") for i in range(2)
        ]
        rng = random.Random(42)
        result = select_dreamcallers(all_callers, rng)
        assert len(result) == 2

    def test_handles_single_dreamcaller(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        all_callers = [_make_dreamcaller(name="Solo Caller")]
        rng = random.Random(42)
        result = select_dreamcallers(all_callers, rng)
        assert len(result) == 1
        assert result[0].name == "Solo Caller"

    def test_handles_empty_list(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        rng = random.Random(42)
        result = select_dreamcallers([], rng)
        assert len(result) == 0


class TestFormatDreamcallerOption:
    """Test the display formatting of a dreamcaller option."""

    def test_highlighted_returns_full_details(self) -> None:
        from sites_dreamcaller import format_dreamcaller_option

        dc = _make_dreamcaller(
            name="Vesper, Twilight Arbiter",
            resonances=frozenset({Resonance.TIDE, Resonance.RUIN}),
            resonance_bonus={"Tide": 4, "Ruin": 4},
            tags=frozenset({"mechanic:reclaim", "mechanic:dissolve"}),
            tag_bonus={"mechanic:reclaim": 2, "mechanic:dissolve": 1},
            essence_bonus=50,
            ability_text="Whenever you dissolve an enemy character, draw a card.",
        )
        lines = format_dreamcaller_option(dc, highlighted=True)
        assert isinstance(lines, list)
        assert all(isinstance(line, str) for line in lines)
        # Highlighted should have multiple detail lines
        assert len(lines) >= 5
        full_text = "\n".join(lines)
        # Must contain resonance bonus info
        assert "+4" in full_text
        # Must contain tags
        assert "mechanic:reclaim" in full_text or "Tags:" in full_text
        # Must contain essence bonus
        assert "+50" in full_text or "Essence" in full_text
        # Must contain ability text
        assert "dissolve" in full_text

    def test_non_highlighted_shows_condensed_view(self) -> None:
        from sites_dreamcaller import format_dreamcaller_option

        dc = _make_dreamcaller(
            name="Vesper, Twilight Arbiter",
            resonances=frozenset({Resonance.TIDE, Resonance.RUIN}),
            resonance_bonus={"Tide": 4, "Ruin": 4},
            tags=frozenset({"mechanic:reclaim", "mechanic:dissolve"}),
            tag_bonus={"mechanic:reclaim": 2, "mechanic:dissolve": 1},
            essence_bonus=50,
            ability_text="Whenever you dissolve an enemy character, draw a card.",
        )
        lines = format_dreamcaller_option(dc, highlighted=False)
        # Condensed should be fewer lines than highlighted
        highlighted_lines = format_dreamcaller_option(dc, highlighted=True)
        assert len(lines) < len(highlighted_lines)
        # Condensed should still show name
        full_text = "\n".join(lines)
        assert "Vesper" in full_text

    def test_highlighted_marker(self) -> None:
        from sites_dreamcaller import format_dreamcaller_option

        dc = _make_dreamcaller(name="Test Caller")
        lines_on = format_dreamcaller_option(dc, highlighted=True)
        lines_off = format_dreamcaller_option(dc, highlighted=False)
        # The highlighted version should have '  > ' prefix on the name line
        assert lines_on[0].startswith("  > ")
        # The non-highlighted version should have '    ' prefix (space marker)
        assert lines_off[0].startswith("    ")
        assert not lines_off[0].startswith("  > ")

    def test_highlighted_shows_resonance_line(self) -> None:
        from sites_dreamcaller import format_dreamcaller_option

        dc = _make_dreamcaller(
            name="Test Caller",
            resonances=frozenset({Resonance.TIDE, Resonance.RUIN}),
        )
        lines = format_dreamcaller_option(dc, highlighted=True)
        full_text = "\n".join(lines)
        assert "Resonance" in full_text

    def test_highlighted_shows_resonance_bonus_line(self) -> None:
        from sites_dreamcaller import format_dreamcaller_option

        dc = _make_dreamcaller(
            name="Test Caller",
            resonance_bonus={"Tide": 4, "Ruin": 4},
        )
        lines = format_dreamcaller_option(dc, highlighted=True)
        full_text = "\n".join(lines)
        assert "Resonance Bonus" in full_text
        assert "+4" in full_text

    def test_highlighted_shows_essence_bonus(self) -> None:
        from sites_dreamcaller import format_dreamcaller_option

        dc = _make_dreamcaller(
            name="Test Caller",
            essence_bonus=75,
        )
        lines = format_dreamcaller_option(dc, highlighted=True)
        full_text = "\n".join(lines)
        assert "Essence Bonus" in full_text
        assert "+75" in full_text

    def test_highlighted_shows_ability_text_quoted(self) -> None:
        from sites_dreamcaller import format_dreamcaller_option

        dc = _make_dreamcaller(
            name="Test Caller",
            ability_text="Draw two cards when you play an event.",
        )
        lines = format_dreamcaller_option(dc, highlighted=True)
        full_text = "\n".join(lines)
        assert '"Draw two cards when you play an event."' in full_text

    def test_highlighted_shows_tags(self) -> None:
        from sites_dreamcaller import format_dreamcaller_option

        dc = _make_dreamcaller(
            name="Test Caller",
            tags=frozenset({"mechanic:reclaim", "mechanic:dissolve"}),
        )
        lines = format_dreamcaller_option(dc, highlighted=True)
        full_text = "\n".join(lines)
        assert "Tags:" in full_text
        assert "mechanic:reclaim" in full_text


class TestFormatConfirmation:
    """Test the confirmation message after dreamcaller selection."""

    def test_shows_dreamcaller_name(self) -> None:
        from sites_dreamcaller import format_confirmation

        dc = _make_dreamcaller(name="Vesper, Twilight Arbiter")
        result = format_confirmation(dc, essence_after=300)
        assert "Vesper" in result

    def test_shows_resonance_bonuses(self) -> None:
        from sites_dreamcaller import format_confirmation

        dc = _make_dreamcaller(
            resonance_bonus={"Tide": 4, "Ruin": 3},
        )
        result = format_confirmation(dc, essence_after=300)
        assert "+4" in result
        assert "+3" in result
        assert "Resonance" in result

    def test_shows_tag_bonuses(self) -> None:
        from sites_dreamcaller import format_confirmation

        dc = _make_dreamcaller(
            tag_bonus={"mechanic:reclaim": 2, "mechanic:dissolve": 1},
        )
        result = format_confirmation(dc, essence_after=300)
        assert "mechanic:reclaim" in result
        assert "Tags:" in result

    def test_shows_essence_with_total(self) -> None:
        from sites_dreamcaller import format_confirmation

        dc = _make_dreamcaller(essence_bonus=50)
        result = format_confirmation(dc, essence_after=300)
        assert "+50" in result
        assert "300" in result

    def test_omits_empty_tag_bonuses(self) -> None:
        from sites_dreamcaller import format_confirmation

        dc = _make_dreamcaller(tag_bonus={})
        result = format_confirmation(dc, essence_after=300)
        # Should not show an empty Tags line
        assert "Tags:" not in result


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


class TestRunDreamcallerDraft:
    """Integration test for the full draft flow with mocked interactive input."""

    def test_applies_selection_and_prints_output(self, capsys: object) -> None:
        from sites_dreamcaller import run_dreamcaller_draft

        state = _make_quest_state(essence=250)
        all_callers = [
            _make_dreamcaller(
                name=f"Caller {i}",
                resonance_bonus={"Tide": i + 1},
                tag_bonus={"tribal:warrior": i},
                essence_bonus=10 * (i + 1),
            )
            for i in range(5)
        ]
        # Mock single_select to return index 1 (second option)
        with patch("sites_dreamcaller.single_select", return_value=1):
            run_dreamcaller_draft(
                state,
                all_callers,
                logger=None,
                dreamscape_name="Twilight Reach",
                dreamscape_number=1,
            )
        # The selected dreamcaller should be applied to state
        assert state.dreamcaller is not None
        assert state.essence > 250
        # Check printed output contains header and confirmation
        captured = capsys.readouterr()  # type: ignore[union-attr]
        assert "Dreamcaller Draft" in captured.out
        assert "Selected:" in captured.out
        assert "Resonance" in captured.out or "Deck" in captured.out

    def test_confirmation_shows_applied_bonuses(self, capsys: object) -> None:
        from sites_dreamcaller import run_dreamcaller_draft

        vesper = _make_dreamcaller(
            name="Vesper, Twilight Arbiter",
            resonances=frozenset({Resonance.TIDE, Resonance.RUIN}),
            resonance_bonus={"Tide": 4, "Ruin": 4},
            tags=frozenset({"mechanic:reclaim", "mechanic:dissolve"}),
            tag_bonus={"mechanic:reclaim": 2, "mechanic:dissolve": 1},
            essence_bonus=50,
        )
        state = _make_quest_state(essence=250)
        # Only provide the one caller so selection is deterministic
        all_callers = [vesper]
        with patch("sites_dreamcaller.single_select", return_value=0):
            run_dreamcaller_draft(
                state,
                all_callers,
                logger=None,
                dreamscape_name="Test",
                dreamscape_number=1,
            )
        captured = capsys.readouterr()  # type: ignore[union-attr]
        # Confirmation should show the selected dreamcaller name
        assert "Vesper" in captured.out
        # Should show the resonance bonus amounts applied
        assert "Tide" in captured.out
        assert "Ruin" in captured.out
        # Should show essence bonus applied
        assert "+50" in captured.out or "300" in captured.out

    def test_logs_selection_when_logger_provided(self) -> None:
        from sites_dreamcaller import run_dreamcaller_draft

        state = _make_quest_state(essence=250)
        all_callers = [
            _make_dreamcaller(name=f"Caller {i}") for i in range(5)
        ]
        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(kwargs)

        with patch("sites_dreamcaller.single_select", return_value=0):
            run_dreamcaller_draft(
                state,
                all_callers,
                logger=FakeLogger(),  # type: ignore[arg-type]
                dreamscape_name="Test",
                dreamscape_number=1,
            )
        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "DreamcallerDraft"
        assert "choice_made" in log_calls[0]
