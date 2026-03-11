"""Tests for the dreamsign site interactions."""

import random
from unittest.mock import patch

from models import Dreamsign
from quest_state import QuestState


class _MockStrategy:
    """Minimal draft strategy stand-in for tests that only need preference_vector."""

    def __init__(self) -> None:
        self.preference_vector: list[float] = [1.0] * 8


def _make_dreamsign(
    name: str = "Test Sign",
    effect_text: str = "Test effect",
    is_bane: bool = False,
) -> Dreamsign:
    return Dreamsign(
        name=name,
        effect_text=effect_text,
        is_bane=is_bane,
    )


def _make_all_dreamsigns() -> list[Dreamsign]:
    """Create a diverse set of dreamsigns for testing."""
    return [
        _make_dreamsign("Sign A"),
        _make_dreamsign("Sign B"),
        _make_dreamsign("Sign C"),
        _make_dreamsign("Sign D"),
        _make_dreamsign("Sign E"),
        _make_dreamsign("Sign F"),
        _make_dreamsign("Sign G"),
        _make_dreamsign("Bane Sign", is_bane=True),
    ]


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


class TestSelectOfferingDreamsigns:
    """Test that dreamsign selection for offerings filters out banes."""

    def test_selects_one_non_bane(self) -> None:
        from sites_dreamsign import select_offering_dreamsigns

        all_signs = _make_all_dreamsigns()
        rng = random.Random(42)
        result = select_offering_dreamsigns(all_signs, rng, count=1)
        assert len(result) == 1
        assert not result[0].is_bane

    def test_selects_three_for_enhanced(self) -> None:
        from sites_dreamsign import select_offering_dreamsigns

        all_signs = _make_all_dreamsigns()
        rng = random.Random(42)
        result = select_offering_dreamsigns(all_signs, rng, count=3)
        assert len(result) == 3
        assert all(not ds.is_bane for ds in result)

    def test_no_duplicates(self) -> None:
        from sites_dreamsign import select_offering_dreamsigns

        all_signs = _make_all_dreamsigns()
        rng = random.Random(42)
        result = select_offering_dreamsigns(all_signs, rng, count=3)
        names = [ds.name for ds in result]
        assert len(set(names)) == len(names)

    def test_deterministic_with_same_seed(self) -> None:
        from sites_dreamsign import select_offering_dreamsigns

        all_signs = _make_all_dreamsigns()
        rng1 = random.Random(99)
        rng2 = random.Random(99)
        r1 = select_offering_dreamsigns(all_signs, rng1, count=3)
        r2 = select_offering_dreamsigns(all_signs, rng2, count=3)
        assert [ds.name for ds in r1] == [ds.name for ds in r2]

    def test_handles_fewer_non_banes_than_requested(self) -> None:
        from sites_dreamsign import select_offering_dreamsigns

        signs = [
            _make_dreamsign("Only Good"),
            _make_dreamsign("Bane 1", is_bane=True),
            _make_dreamsign("Bane 2", is_bane=True),
        ]
        rng = random.Random(42)
        result = select_offering_dreamsigns(signs, rng, count=3)
        assert len(result) == 1

    def test_handles_empty_list(self) -> None:
        from sites_dreamsign import select_offering_dreamsigns

        rng = random.Random(42)
        result = select_offering_dreamsigns([], rng, count=1)
        assert len(result) == 0


class TestSelectDraftDreamsigns:
    """Test dreamsign selection for draft excludes held and banes."""

    def test_selects_three(self) -> None:
        from sites_dreamsign import select_draft_dreamsigns

        all_signs = _make_all_dreamsigns()
        rng = random.Random(42)
        result = select_draft_dreamsigns(all_signs, held=[], rng=rng)
        assert len(result) == 3
        assert all(not ds.is_bane for ds in result)

    def test_excludes_held_dreamsigns(self) -> None:
        from sites_dreamsign import select_draft_dreamsigns

        all_signs = _make_all_dreamsigns()
        held = [all_signs[0], all_signs[1]]  # Sign A, Sign B
        rng = random.Random(42)
        result = select_draft_dreamsigns(all_signs, held=held, rng=rng)
        held_names = {ds.name for ds in held}
        for ds in result:
            assert ds.name not in held_names

    def test_no_duplicates(self) -> None:
        from sites_dreamsign import select_draft_dreamsigns

        all_signs = _make_all_dreamsigns()
        rng = random.Random(42)
        result = select_draft_dreamsigns(all_signs, held=[], rng=rng)
        names = [ds.name for ds in result]
        assert len(set(names)) == len(names)

    def test_handles_fewer_available_than_three(self) -> None:
        from sites_dreamsign import select_draft_dreamsigns

        signs = [
            _make_dreamsign("Only One"),
            _make_dreamsign("Bane", is_bane=True),
        ]
        rng = random.Random(42)
        result = select_draft_dreamsigns(signs, held=[], rng=rng)
        assert len(result) == 1

    def test_selects_four_when_enhanced(self) -> None:
        from sites_dreamsign import DRAFT_ENHANCED_COUNT, select_draft_dreamsigns

        all_signs = _make_all_dreamsigns()
        rng = random.Random(42)
        result = select_draft_dreamsigns(
            all_signs,
            held=[],
            rng=rng,
            count=DRAFT_ENHANCED_COUNT,
        )
        assert len(result) == 4
        assert all(not ds.is_bane for ds in result)


class TestFormatDreamsignOption:
    """Test dreamsign display formatting."""

    def test_returns_list_of_strings(self) -> None:
        from sites_dreamsign import format_dreamsign_option

        ds = _make_dreamsign(
            "Sigil of Shifting Tides",
            effect_text="At the start of each battle, foresee 2.",
        )
        lines = format_dreamsign_option(ds, highlighted=False)
        assert isinstance(lines, list)
        assert all(isinstance(line, str) for line in lines)
        assert len(lines) >= 2

    def test_highlighted_marker(self) -> None:
        from sites_dreamsign import format_dreamsign_option

        ds = _make_dreamsign("Test Sign")
        lines_on = format_dreamsign_option(ds, highlighted=True)
        lines_off = format_dreamsign_option(ds, highlighted=False)
        assert ">" in lines_on[0]
        assert ">" not in lines_off[0]


class TestDreamsignPurge:
    """Test purge prompt when at dreamsign limit."""

    def test_purge_removes_selected_dreamsign(self) -> None:
        from sites_dreamsign import handle_dreamsign_purge

        state = _make_quest_state()
        # Fill dreamsigns to the limit
        for i in range(12):
            state.add_dreamsign(_make_dreamsign(f"Existing {i}"))
        assert state.is_over_dreamsign_limit()

        new_sign = _make_dreamsign("New Sign")
        # Mock single_select to remove index 0 (first dreamsign)
        with patch("sites_dreamsign.input_handler.single_select", return_value=0):
            handle_dreamsign_purge(state, new_sign)

        # Should have removed one and added the new one, net same count
        assert state.dreamsign_count() == 12
        assert any(ds.name == "New Sign" for ds in state.dreamsigns)
        assert state.dreamsigns[0].name != "Existing 0"

    def test_no_purge_when_under_limit(self) -> None:
        from sites_dreamsign import handle_dreamsign_purge

        state = _make_quest_state()
        state.add_dreamsign(_make_dreamsign("Existing"))
        new_sign = _make_dreamsign("New Sign")

        # Should add without prompting
        handle_dreamsign_purge(state, new_sign)

        assert state.dreamsign_count() == 2
        assert any(ds.name == "New Sign" for ds in state.dreamsigns)


class TestRunDreamsignOffering:
    """Integration tests for the Dreamsign Offering site."""

    def test_accept_adds_dreamsign(self) -> None:
        from sites_dreamsign import run_dreamsign_offering

        state = _make_quest_state()
        all_signs = _make_all_dreamsigns()
        assert state.dreamsign_count() == 0

        with patch("sites_dreamsign.input_handler.confirm_decline", return_value=True):
            run_dreamsign_offering(
                state=state,
                all_dreamsigns=all_signs,
                logger=None,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
            )

        assert state.dreamsign_count() == 1
        # The added dreamsign should not be a bane
        assert not state.dreamsigns[0].is_bane

    def test_decline_does_not_add(self) -> None:
        from sites_dreamsign import run_dreamsign_offering

        state = _make_quest_state()
        all_signs = _make_all_dreamsigns()

        with patch("sites_dreamsign.input_handler.confirm_decline", return_value=False):
            run_dreamsign_offering(
                state=state,
                all_dreamsigns=all_signs,
                logger=None,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
            )

        assert state.dreamsign_count() == 0

    def test_enhanced_uses_single_select(self) -> None:
        from sites_dreamsign import run_dreamsign_offering

        state = _make_quest_state()
        all_signs = _make_all_dreamsigns()

        # Pick the first option (not the skip option which is last)
        with patch("sites_dreamsign.input_handler.single_select", return_value=0):
            run_dreamsign_offering(
                state=state,
                all_dreamsigns=all_signs,
                logger=None,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                is_enhanced=True,
            )

        assert state.dreamsign_count() == 1

    def test_enhanced_skip_option(self) -> None:
        from sites_dreamsign import run_dreamsign_offering

        state = _make_quest_state()
        all_signs = _make_all_dreamsigns()

        # Pick the last option (skip) -- 3 dreamsigns + skip = index 3
        with patch("sites_dreamsign.input_handler.single_select", return_value=3):
            run_dreamsign_offering(
                state=state,
                all_dreamsigns=all_signs,
                logger=None,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                is_enhanced=True,
            )

        assert state.dreamsign_count() == 0

    def test_enhanced_skip_with_fewer_than_three_options(self) -> None:
        from sites_dreamsign import run_dreamsign_offering

        state = _make_quest_state()
        # Only one non-bane dreamsign available, so enhanced shows 1 + skip
        all_signs = [
            _make_dreamsign("Only One"),
            _make_dreamsign("Bane", is_bane=True),
        ]

        # Skip is the last option; with 1 dreamsign offered, skip is index 1
        with patch("sites_dreamsign.input_handler.single_select", return_value=1):
            run_dreamsign_offering(
                state=state,
                all_dreamsigns=all_signs,
                logger=None,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                is_enhanced=True,
            )

        assert state.dreamsign_count() == 0

    def test_logs_on_empty_offering(self) -> None:
        from sites_dreamsign import run_dreamsign_offering

        state = _make_quest_state()
        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(kwargs)

        # All banes, so no dreamsigns available
        all_signs = [
            _make_dreamsign("Bane 1", is_bane=True),
            _make_dreamsign("Bane 2", is_bane=True),
        ]

        run_dreamsign_offering(
            state=state,
            all_dreamsigns=all_signs,
            logger=FakeLogger(),  # type: ignore[arg-type]
            dreamscape_name="Test Dreamscape",
            dreamscape_number=1,
        )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "DreamsignOffering"
        assert log_calls[0]["choices"] == []
        assert log_calls[0]["choice_made"] is None

    def test_logs_when_logger_provided(self) -> None:
        from sites_dreamsign import run_dreamsign_offering

        state = _make_quest_state()
        all_signs = _make_all_dreamsigns()
        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(kwargs)

        with patch("sites_dreamsign.input_handler.confirm_decline", return_value=True):
            run_dreamsign_offering(
                state=state,
                all_dreamsigns=all_signs,
                logger=FakeLogger(),  # type: ignore[arg-type]
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
            )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "DreamsignOffering"


class TestRunDreamsignDraft:
    """Integration tests for the Dreamsign Draft site."""

    def test_pick_adds_dreamsign(self) -> None:
        from sites_dreamsign import run_dreamsign_draft

        state = _make_quest_state()
        all_signs = _make_all_dreamsigns()

        # Pick first option (not skip)
        with patch("sites_dreamsign.input_handler.single_select", return_value=0):
            run_dreamsign_draft(
                state=state,
                all_dreamsigns=all_signs,
                logger=None,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
            )

        assert state.dreamsign_count() == 1
        assert not state.dreamsigns[0].is_bane

    def test_skip_does_not_add(self) -> None:
        from sites_dreamsign import run_dreamsign_draft

        state = _make_quest_state()
        all_signs = _make_all_dreamsigns()

        # Pick the last option (skip) -- 3 dreamsigns + skip = index 3
        with patch("sites_dreamsign.input_handler.single_select", return_value=3):
            run_dreamsign_draft(
                state=state,
                all_dreamsigns=all_signs,
                logger=None,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
            )

        assert state.dreamsign_count() == 0

    def test_skip_with_fewer_than_three_options(self) -> None:
        from sites_dreamsign import run_dreamsign_draft

        state = _make_quest_state()
        # Only one non-bane dreamsign available
        all_signs = [
            _make_dreamsign("Only One"),
            _make_dreamsign("Bane", is_bane=True),
        ]

        # Skip is the last option; with 1 dreamsign offered, skip is index 1
        with patch("sites_dreamsign.input_handler.single_select", return_value=1):
            run_dreamsign_draft(
                state=state,
                all_dreamsigns=all_signs,
                logger=None,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
            )

        assert state.dreamsign_count() == 0

    def test_logs_on_empty_draft(self) -> None:
        from sites_dreamsign import run_dreamsign_draft

        state = _make_quest_state()
        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(kwargs)

        # All banes, so no dreamsigns available
        all_signs = [
            _make_dreamsign("Bane 1", is_bane=True),
            _make_dreamsign("Bane 2", is_bane=True),
        ]

        run_dreamsign_draft(
            state=state,
            all_dreamsigns=all_signs,
            logger=FakeLogger(),  # type: ignore[arg-type]
            dreamscape_name="Test Dreamscape",
            dreamscape_number=1,
        )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "DreamsignDraft"
        assert log_calls[0]["choices"] == []
        assert log_calls[0]["choice_made"] is None

    def test_logs_when_logger_provided(self) -> None:
        from sites_dreamsign import run_dreamsign_draft

        state = _make_quest_state()
        all_signs = _make_all_dreamsigns()
        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(kwargs)

        with patch("sites_dreamsign.input_handler.single_select", return_value=0):
            run_dreamsign_draft(
                state=state,
                all_dreamsigns=all_signs,
                logger=FakeLogger(),  # type: ignore[arg-type]
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
            )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "DreamsignDraft"

    def test_at_limit_triggers_purge(self) -> None:
        from sites_dreamsign import run_dreamsign_draft

        state = _make_quest_state()
        # Fill to the limit
        for i in range(12):
            state.add_dreamsign(_make_dreamsign(f"Existing {i}"))

        all_signs = _make_all_dreamsigns()

        # First call: single_select for draft pick (pick index 0)
        # Second call: single_select for purge (purge index 0)
        with patch(
            "sites_dreamsign.input_handler.single_select",
            side_effect=[0, 0],
        ):
            run_dreamsign_draft(
                state=state,
                all_dreamsigns=all_signs,
                logger=None,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
            )

        # Should still have 12 (removed one, added one)
        assert state.dreamsign_count() == 12

    def test_enhanced_offers_additional_option(self) -> None:
        from sites_dreamsign import run_dreamsign_draft

        state = _make_quest_state()
        all_signs = _make_all_dreamsigns()

        # Enhanced draft: 4 dreamsigns + skip = 5 options, skip is index 4
        with patch(
            "sites_dreamsign.input_handler.single_select", return_value=4
        ) as mock_select:
            run_dreamsign_draft(
                state=state,
                all_dreamsigns=all_signs,
                logger=None,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                is_enhanced=True,
            )

        # Verify that 5 options were presented (4 dreamsigns + Skip)
        call_args = mock_select.call_args
        options = (
            call_args[1]["options"] if "options" in call_args[1] else call_args[0][0]
        )
        assert len(options) == 5
        # Player skipped, so no dreamsign added
        assert state.dreamsign_count() == 0

    def test_enhanced_pick_adds_dreamsign(self) -> None:
        from sites_dreamsign import run_dreamsign_draft

        state = _make_quest_state()
        all_signs = _make_all_dreamsigns()

        # Pick first option in enhanced draft
        with patch("sites_dreamsign.input_handler.single_select", return_value=0):
            run_dreamsign_draft(
                state=state,
                all_dreamsigns=all_signs,
                logger=None,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                is_enhanced=True,
            )

        assert state.dreamsign_count() == 1
        assert not state.dreamsigns[0].is_bane

    def test_enhanced_logs_is_enhanced_true(self) -> None:
        from sites_dreamsign import run_dreamsign_draft

        state = _make_quest_state()
        all_signs = _make_all_dreamsigns()
        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(kwargs)

        with patch("sites_dreamsign.input_handler.single_select", return_value=0):
            run_dreamsign_draft(
                state=state,
                all_dreamsigns=all_signs,
                logger=FakeLogger(),  # type: ignore[arg-type]
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                is_enhanced=True,
            )

        assert len(log_calls) == 1
        assert log_calls[0]["is_enhanced"] is True
