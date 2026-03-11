"""Tests for sites_purge module."""

import random
from unittest.mock import patch

from quest_state import QuestState


class _MockStrategy:
    """Minimal draft strategy stand-in for tests that only need preference_vector."""

    def __init__(self) -> None:
        self.preference_vector: list[float] = [1.0] * 8


def _make_quest_state(
    seed: int = 42,
    essence: int = 250,
    max_deck: int = 50,
) -> QuestState:
    rng = random.Random(seed)
    state = QuestState(
        essence=essence,
        rng=rng,
        max_deck=max_deck,
    )
    state.draft_strategy = _MockStrategy()  # type: ignore[assignment]
    return state


def _populate_deck(state: QuestState, count: int) -> None:
    """Add string card instances to the deck until it has `count` cards."""
    for i in range(count):
        state.add_card(f"card_instance_{i}")


class TestRunPurge:
    """Tests for voluntary purge site interaction."""

    def test_purge_removes_selected_cards(self) -> None:
        """Selected cards should be removed from the deck."""
        from sites_purge import run_purge

        state = _make_quest_state()
        _populate_deck(state, 10)
        initial_deck_size = state.deck_count()

        # Select first two cards for purging (indices 0 and 1)
        with patch("sites_purge.input_handler.multi_select", return_value=[0, 1]):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == initial_deck_size - 2

    def test_purge_default_max_is_three(self) -> None:
        """Without enhancement, max selections should be 3."""
        from sites_purge import run_purge

        state = _make_quest_state()
        _populate_deck(state, 10)
        initial_deck_size = state.deck_count()

        captured_kwargs: list[dict[str, object]] = []

        def mock_multi_select(options: list[str], **kwargs: object) -> list[int]:
            captured_kwargs.append(kwargs)
            return [0, 1, 2]

        with patch(
            "sites_purge.input_handler.multi_select",
            side_effect=mock_multi_select,
        ):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == initial_deck_size - 3
        # Verify max_selections was passed as 3
        assert captured_kwargs[0].get("max_selections") == 3

    def test_purge_enhanced_max_is_six(self) -> None:
        """When enhanced (Ashen biome), max selections should be 6."""
        from sites_purge import run_purge

        state = _make_quest_state()
        _populate_deck(state, 10)
        initial_deck_size = state.deck_count()

        captured_kwargs: list[dict[str, object]] = []

        def mock_multi_select(options: list[str], **kwargs: object) -> list[int]:
            captured_kwargs.append(kwargs)
            return [0, 1, 2, 3, 4, 5]

        with patch(
            "sites_purge.input_handler.multi_select",
            side_effect=mock_multi_select,
        ):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                is_enhanced=True,
            )

        assert state.deck_count() == initial_deck_size - 6
        assert captured_kwargs[0].get("max_selections") == 6

    def test_purge_skip_removes_nothing(self) -> None:
        """Selecting no cards should not change the deck."""
        from sites_purge import run_purge

        state = _make_quest_state()
        _populate_deck(state, 10)
        initial_deck_size = state.deck_count()

        # Select nothing (empty list)
        with patch("sites_purge.input_handler.multi_select", return_value=[]):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == initial_deck_size

    def test_purge_empty_deck(self) -> None:
        """Purge with an empty deck should handle gracefully."""
        from sites_purge import run_purge

        state = _make_quest_state()
        assert state.deck_count() == 0

        # Should not even call multi_select
        run_purge(
            state=state,
            dreamscape_name="Test Dreamscape",
            dreamscape_number=1,
            logger=None,
        )

        assert state.deck_count() == 0

    def test_purge_with_logger(self) -> None:
        """Purge should log the interaction via log_site_visit."""
        from sites_purge import run_purge

        state = _make_quest_state()
        _populate_deck(state, 5)

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(dict(kwargs))

        with patch("sites_purge.input_handler.multi_select", return_value=[0, 1]):
            run_purge(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "Purge"


class TestForcedDeckLimitPurge:
    """Tests for forced deck-limit purge before battles."""

    def test_forced_purge_not_triggered_under_limit(self) -> None:
        """When deck <= max_deck, forced purge should do nothing."""
        from sites_purge import forced_deck_limit_purge

        state = _make_quest_state(max_deck=50)
        _populate_deck(state, 50)
        assert state.deck_count() == 50

        # Should not call multi_select at all
        with patch(
            "sites_purge.input_handler.multi_select",
            return_value=[],
        ) as mock_ms:
            forced_deck_limit_purge(state=state, logger=None)

        mock_ms.assert_not_called()
        assert state.deck_count() == 50

    def test_forced_purge_removes_until_at_limit(self) -> None:
        """Forced purge should remove cards until deck <= max_deck."""
        from sites_purge import forced_deck_limit_purge

        state = _make_quest_state(max_deck=10)
        _populate_deck(state, 13)
        assert state.deck_count() == 13

        # Need to remove at least 3 cards (13 - 10 = 3)
        with patch(
            "sites_purge.input_handler._is_interactive",
            return_value=True,
        ), patch(
            "sites_purge.input_handler.multi_select",
            return_value=[0, 1, 2],
        ):
            forced_deck_limit_purge(state=state, logger=None)

        assert state.deck_count() <= 10

    def test_forced_purge_loops_until_satisfied(self) -> None:
        """If player doesn't remove enough, forced purge should loop."""
        from sites_purge import forced_deck_limit_purge

        state = _make_quest_state(max_deck=10)
        _populate_deck(state, 13)
        # Need to remove 3, but player only removes 1 each time

        call_count = [0]

        def mock_multi_select(options: list[str], **kwargs: object) -> list[int]:
            call_count[0] += 1
            return [0]  # Remove 1 card each time

        with patch(
            "sites_purge.input_handler._is_interactive",
            return_value=True,
        ), patch(
            "sites_purge.input_handler.multi_select",
            side_effect=mock_multi_select,
        ):
            forced_deck_limit_purge(state=state, logger=None)

        assert state.deck_count() <= 10
        assert call_count[0] == 3  # 3 iterations to remove 3 cards

    def test_forced_purge_auto_removes_non_interactive(self) -> None:
        """In non-interactive mode, forced purge auto-removes excess cards."""
        from sites_purge import forced_deck_limit_purge

        state = _make_quest_state(max_deck=10)
        _populate_deck(state, 13)
        assert state.deck_count() == 13

        with patch("sites_purge.input_handler._is_interactive", return_value=False):
            forced_deck_limit_purge(state=state, logger=None)

        assert state.deck_count() == 10

    def test_forced_purge_with_logger(self) -> None:
        """Forced purge should log the interaction."""
        from sites_purge import forced_deck_limit_purge

        state = _make_quest_state(max_deck=5)
        _populate_deck(state, 7)

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(dict(kwargs))

        with patch("sites_purge.input_handler._is_interactive", return_value=False):
            forced_deck_limit_purge(state=state, logger=FakeLogger())  # type: ignore[arg-type]

        assert state.deck_count() <= 5
        assert len(log_calls) == 1

    def test_forced_purge_log_includes_dreamscape(self) -> None:
        """Forced purge JSONL log should include the dreamscape name."""
        from sites_purge import forced_deck_limit_purge

        state = _make_quest_state(max_deck=5)
        _populate_deck(state, 7)

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(dict(kwargs))

        with patch("sites_purge.input_handler._is_interactive", return_value=False):
            forced_deck_limit_purge(
                state=state,
                logger=FakeLogger(),  # type: ignore[arg-type]
                dreamscape_name="Twilight Grove",
            )

        assert state.deck_count() <= 5
        assert len(log_calls) == 1
        assert log_calls[0]["dreamscape"] == "Twilight Grove"
