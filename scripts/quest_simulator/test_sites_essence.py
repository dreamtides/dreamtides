"""Tests for sites_essence module."""

import random
from unittest.mock import patch

from quest_state import QuestState


class _MockAgent:
    """Minimal agent stand-in for tests that don't need full draft logic."""

    def __init__(self) -> None:
        self.w: list[float] = [0.1] * 8


def _make_quest_state(
    seed: int = 42,
    essence: int = 250,
    completion_level: int = 0,
) -> QuestState:
    rng = random.Random(seed)
    state = QuestState(
        essence=essence,
        rng=rng,
        human_agent=_MockAgent(),
        ai_agents=[],
        cube=None,
        draft_cfg=None,
        packs=[],
    )
    state.completion_level = completion_level
    return state


def _essence_config() -> dict[str, int]:
    return {
        "amount_level_0": 200,
        "amount_level_2": 250,
        "amount_level_4": 300,
    }


class TestEssenceAmount:
    """Tests for essence amount calculation based on completion level."""

    def test_level_0_gives_200(self) -> None:
        from sites_essence import compute_essence_amount

        config = _essence_config()
        assert compute_essence_amount(0, config) == 200

    def test_level_1_gives_200(self) -> None:
        from sites_essence import compute_essence_amount

        config = _essence_config()
        assert compute_essence_amount(1, config) == 200

    def test_level_2_gives_250(self) -> None:
        from sites_essence import compute_essence_amount

        config = _essence_config()
        assert compute_essence_amount(2, config) == 250

    def test_level_3_gives_250(self) -> None:
        from sites_essence import compute_essence_amount

        config = _essence_config()
        assert compute_essence_amount(3, config) == 250

    def test_level_4_gives_300(self) -> None:
        from sites_essence import compute_essence_amount

        config = _essence_config()
        assert compute_essence_amount(4, config) == 300

    def test_level_5_gives_300(self) -> None:
        from sites_essence import compute_essence_amount

        config = _essence_config()
        assert compute_essence_amount(5, config) == 300

    def test_level_6_gives_300(self) -> None:
        from sites_essence import compute_essence_amount

        config = _essence_config()
        assert compute_essence_amount(6, config) == 300


class TestEssenceEnhanced:
    """Tests for enhanced (Crystalline) doubling."""

    def test_enhanced_doubles_level_0(self) -> None:
        from sites_essence import compute_essence_amount

        config = _essence_config()
        amount = compute_essence_amount(0, config, enhanced=True)
        assert amount == 400

    def test_enhanced_doubles_level_2(self) -> None:
        from sites_essence import compute_essence_amount

        config = _essence_config()
        amount = compute_essence_amount(2, config, enhanced=True)
        assert amount == 500

    def test_enhanced_doubles_level_4(self) -> None:
        from sites_essence import compute_essence_amount

        config = _essence_config()
        amount = compute_essence_amount(4, config, enhanced=True)
        assert amount == 600

    def test_not_enhanced_does_not_double(self) -> None:
        from sites_essence import compute_essence_amount

        config = _essence_config()
        amount = compute_essence_amount(0, config, enhanced=False)
        assert amount == 200


class TestRunEssence:
    """Tests for run_essence site interaction."""

    def test_essence_gain_added_to_state(self) -> None:
        """Running the essence site should add the correct amount."""
        from sites_essence import run_essence

        state = _make_quest_state(essence=250, completion_level=0)
        config = _essence_config()

        with patch("sites_essence.input_handler.wait_for_continue"):
            run_essence(
                state=state,
                essence_config=config,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                is_enhanced=False,
            )

        assert state.essence == 450  # 250 + 200

    def test_essence_gain_enhanced(self) -> None:
        """Enhanced essence site should double the gain."""
        from sites_essence import run_essence

        state = _make_quest_state(essence=100, completion_level=2)
        config = _essence_config()

        with patch("sites_essence.input_handler.wait_for_continue"):
            run_essence(
                state=state,
                essence_config=config,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                is_enhanced=True,
            )

        assert state.essence == 600  # 100 + 500 (250 * 2)

    def test_essence_gain_at_high_level(self) -> None:
        """Level 4+ should give the highest amount."""
        from sites_essence import run_essence

        state = _make_quest_state(essence=0, completion_level=5)
        config = _essence_config()

        with patch("sites_essence.input_handler.wait_for_continue"):
            run_essence(
                state=state,
                essence_config=config,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                is_enhanced=False,
            )

        assert state.essence == 300

    def test_essence_logs_site_visit(self) -> None:
        """Running the essence site should log the visit when logger provided."""
        from sites_essence import run_essence

        state = _make_quest_state(essence=250, completion_level=1)
        config = _essence_config()
        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(dict(kwargs))

        with patch("sites_essence.input_handler.wait_for_continue"):
            run_essence(
                state=state,
                essence_config=config,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
                is_enhanced=False,
            )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "Essence"
        changes = log_calls[0]["state_changes"]
        assert changes["essence_gained"] == 200  # type: ignore[comparison-overlap]
        assert log_calls[0]["is_enhanced"] is False

    def test_essence_no_logger_does_not_crash(self) -> None:
        """Running with no logger should not raise."""
        from sites_essence import run_essence

        state = _make_quest_state(essence=250, completion_level=0)
        config = _essence_config()

        with patch("sites_essence.input_handler.wait_for_continue"):
            run_essence(
                state=state,
                essence_config=config,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                is_enhanced=False,
            )

        assert state.essence == 450
