"""Tests for quest_sim entry point.

Validates CLI argument parsing, initialization sequence, and banner
display without requiring interactive terminal input.
"""

import argparse
import os
import sys
import unittest
from typing import Any, Optional

os.environ["NO_COLOR"] = "1"

# Ensure the quest_simulator and draft_simulator directories are on sys.path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "draft_simulator_v2"))
sys.path.insert(0, os.path.dirname(__file__))


class TestBuildParser(unittest.TestCase):
    """Tests for the CLI argument parser construction."""

    def setUp(self) -> None:
        from quest_sim import build_parser

        self.parser = build_parser()

    def test_default_seed_is_none(self) -> None:
        args = self.parser.parse_args([])
        self.assertIsNone(args.seed)

    def test_seed_short_flag(self) -> None:
        args = self.parser.parse_args(["-s", "42"])
        self.assertEqual(args.seed, 42)

    def test_seed_long_flag(self) -> None:
        args = self.parser.parse_args(["--seed", "99"])
        self.assertEqual(args.seed, 99)

    def test_seed_only_flag(self) -> None:
        args = self.parser.parse_args(["--seed", "7"])
        self.assertEqual(args.seed, 7)


class TestQuestStartBannerWithCards(unittest.TestCase):
    """Tests for the updated quest_start_banner with card_count parameter."""

    def test_banner_with_default_card_count(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(
            seed=42,
            starting_essence=250,
        )
        self.assertIn("540", result)
        self.assertIn("42", result)
        self.assertIn("250", result)

    def test_banner_with_custom_card_count(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(seed=42, starting_essence=250, card_count=100)
        self.assertIn("100", result)

    def test_banner_contains_header(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(
            seed=1,
            starting_essence=300,
        )
        self.assertIn("DREAMTIDES QUEST", result)
        self.assertIn("Seed: 1", result)

    def test_banner_contains_press_enter(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(seed=1, starting_essence=250)
        self.assertIn("Press Enter to begin", result)

    def test_banner_no_pool_variance(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(seed=42, starting_essence=250)
        self.assertNotIn("variance", result.lower())
        self.assertNotIn("Pool bias", result)

    def test_banner_no_rarity(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(seed=42, starting_essence=250)
        self.assertNotIn("rarity", result.lower())
        self.assertNotIn("Common:", result)
        self.assertNotIn("Uncommon:", result)

    def test_banner_no_algorithm_params(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(seed=42, starting_essence=250)
        self.assertNotIn("Algorithm:", result)


class TestModuleImport(unittest.TestCase):
    """Tests that the module imports cleanly."""

    def test_import_main(self) -> None:
        from quest_sim import main

        self.assertTrue(callable(main))

    def test_import_build_parser(self) -> None:
        from quest_sim import build_parser

        self.assertTrue(callable(build_parser))


class TestExceptionHandling(unittest.TestCase):
    """Tests for top-level exception handling in quest_sim."""

    def test_keyboard_interrupt_restores_terminal(self) -> None:
        """KeyboardInterrupt should restore terminal and exit cleanly."""
        from unittest.mock import MagicMock, patch

        with patch("quest_sim.main", side_effect=KeyboardInterrupt):
            with patch(
                "quest_sim.input_handler.ensure_terminal_restored"
            ) as mock_restore:
                with self.assertRaises(SystemExit) as ctx:
                    import quest_sim

                    quest_sim._run_with_error_handling()
                mock_restore.assert_called_once()

    def test_generic_exception_restores_terminal(self) -> None:
        """Unhandled exceptions should restore terminal before re-raising."""
        from unittest.mock import MagicMock, patch

        with patch("quest_sim.main", side_effect=RuntimeError("boom")):
            with patch(
                "quest_sim.input_handler.ensure_terminal_restored"
            ) as mock_restore:
                with self.assertRaises(SystemExit):
                    import quest_sim

                    quest_sim._run_with_error_handling()
                mock_restore.assert_called()


if __name__ == "__main__":
    unittest.main()
