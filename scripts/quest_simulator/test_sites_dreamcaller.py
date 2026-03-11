"""Tests for the Dreamcaller Draft site interaction."""

import random
from unittest.mock import patch

from models import Dreamcaller
from quest_state import QuestState


class _MockStrategy:
    """Minimal draft strategy stand-in for tests that only need preference_vector."""

    def __init__(self) -> None:
        self.preference_vector: list[float] = [1.0] * 8


def _make_dreamcaller(
    name: str = "Test Caller",
    archetype: str = "Flash",
    essence_bonus: int = 50,
    ability_text: str = "Test ability",
) -> Dreamcaller:
    return Dreamcaller(
        name=name,
        archetype=archetype,
        essence_bonus=essence_bonus,
        ability_text=ability_text,
    )


def _make_quest_state(
    essence: int = 250,
    seed: int = 42,
) -> QuestState:
    rng = random.Random(seed)
    state = QuestState(
        essence=essence,
        rng=rng,
    )
    state.draft_strategy = _MockStrategy()  # type: ignore[assignment]
    return state


class TestSelectDreamcallers:
    """Test that 3 dreamcallers are randomly selected from the full list."""

    def test_selects_three_dreamcallers(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        all_callers = [_make_dreamcaller(name=f"Caller {i}") for i in range(8)]
        rng = random.Random(42)
        result = select_dreamcallers(all_callers, rng)
        assert len(result) == 3

    def test_all_selected_from_pool(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        all_callers = [_make_dreamcaller(name=f"Caller {i}") for i in range(8)]
        rng = random.Random(42)
        result = select_dreamcallers(all_callers, rng)
        for dc in result:
            assert dc in all_callers

    def test_no_duplicates(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        all_callers = [_make_dreamcaller(name=f"Caller {i}") for i in range(8)]
        rng = random.Random(42)
        result = select_dreamcallers(all_callers, rng)
        assert len(set(id(dc) for dc in result)) == 3

    def test_deterministic_with_same_seed(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        all_callers = [_make_dreamcaller(name=f"Caller {i}") for i in range(8)]
        rng1 = random.Random(99)
        rng2 = random.Random(99)
        result1 = select_dreamcallers(all_callers, rng1)
        result2 = select_dreamcallers(all_callers, rng2)
        assert [dc.name for dc in result1] == [dc.name for dc in result2]

    def test_handles_exactly_three(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        all_callers = [_make_dreamcaller(name=f"Caller {i}") for i in range(3)]
        rng = random.Random(42)
        result = select_dreamcallers(all_callers, rng)
        assert len(result) == 3

    def test_handles_fewer_than_three(self) -> None:
        from sites_dreamcaller import select_dreamcallers

        all_callers = [_make_dreamcaller(name=f"Caller {i}") for i in range(2)]
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
            name="Shatter Archetype Dreamcaller",
            essence_bonus=50,
            ability_text="Whenever you dissolve an enemy character, draw a card.",
        )
        lines = format_dreamcaller_option(dc, highlighted=True)
        assert isinstance(lines, list)
        assert all(isinstance(line, str) for line in lines)
        # Highlighted should have multiple detail lines
        assert len(lines) >= 2
        full_text = "\n".join(lines)
        # Must contain essence bonus
        assert "+50" in full_text or "Essence" in full_text
        # Must contain ability text
        assert "dissolve" in full_text

    def test_non_highlighted_shows_condensed_view(self) -> None:
        from sites_dreamcaller import format_dreamcaller_option

        dc = _make_dreamcaller(
            name="Shatter Archetype Dreamcaller",
            essence_bonus=50,
            ability_text="Whenever you dissolve an enemy character, draw a card.",
        )
        lines = format_dreamcaller_option(dc, highlighted=False)
        # Condensed should be fewer lines than highlighted
        highlighted_lines = format_dreamcaller_option(dc, highlighted=True)
        assert len(lines) < len(highlighted_lines)
        # Condensed should still show name
        full_text = "\n".join(lines)
        assert "Shatter" in full_text

    def test_highlighted_marker(self) -> None:
        from sites_dreamcaller import format_dreamcaller_option

        dc = _make_dreamcaller(name="Test Caller")
        lines_on = format_dreamcaller_option(dc, highlighted=True)
        lines_off = format_dreamcaller_option(dc, highlighted=False)
        # The highlighted version should have '>' prefix on the name line
        assert ">" in lines_on[0]
        # The non-highlighted version should not have '>' prefix
        assert ">" not in lines_off[0]

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


class TestFormatConfirmation:
    """Test the confirmation message after dreamcaller selection."""

    def test_shows_dreamcaller_name(self) -> None:
        from sites_dreamcaller import format_confirmation

        dc = _make_dreamcaller(name="Shatter Archetype Dreamcaller")
        result = format_confirmation(dc, essence_after=300)
        assert "Shatter" in result

    def test_shows_essence_with_total(self) -> None:
        from sites_dreamcaller import format_confirmation

        dc = _make_dreamcaller(essence_bonus=50)
        result = format_confirmation(dc, essence_after=300)
        assert "+50" in result
        assert "300" in result


class TestApplyDreamcaller:
    """Test that applying a dreamcaller updates quest state correctly."""

    def test_sets_dreamcaller_on_state(self) -> None:
        from sites_dreamcaller import apply_dreamcaller

        state = _make_quest_state()
        dc = _make_dreamcaller(name="Shatter Archetype Dreamcaller")
        apply_dreamcaller(state, dc)
        assert state.dreamcaller is dc

    def test_applies_essence_bonus(self) -> None:
        from sites_dreamcaller import apply_dreamcaller

        state = _make_quest_state(essence=250)
        dc = _make_dreamcaller(essence_bonus=50)
        apply_dreamcaller(state, dc)
        assert state.essence == 300


class TestRunDreamcallerDraft:
    """Integration test for the full draft flow with mocked interactive input."""

    def test_applies_selection_and_prints_output(self, capsys: object) -> None:
        from sites_dreamcaller import run_dreamcaller_draft

        state = _make_quest_state(essence=250)
        all_callers = [
            _make_dreamcaller(
                name=f"Caller {i}",
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

    def test_logs_selection_when_logger_provided(self) -> None:
        from sites_dreamcaller import run_dreamcaller_draft

        state = _make_quest_state(essence=250)
        all_callers = [_make_dreamcaller(name=f"Caller {i}") for i in range(5)]
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
