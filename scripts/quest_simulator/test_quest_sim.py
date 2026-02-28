"""Tests for quest_sim entry point.

Validates CLI argument parsing, initialization sequence, and banner
display without requiring interactive terminal input.
"""

import argparse
import os
import sys
import unittest

# Ensure the quest_simulator directory is on sys.path
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

    def test_exponent_flag(self) -> None:
        args = self.parser.parse_args(["--exponent", "2.0"])
        # pyre-fixme[6]: assertAlmostEqual does not accept float second arg
        self.assertAlmostEqual(args.exponent, 2.0)

    def test_floor_weight_flag(self) -> None:
        args = self.parser.parse_args(["--floor-weight", "0.3"])
        # pyre-fixme[6]: assertAlmostEqual does not accept float second arg
        self.assertAlmostEqual(args.floor_weight, 0.3)

    def test_neutral_base_flag(self) -> None:
        args = self.parser.parse_args(["--neutral-base", "5.0"])
        # pyre-fixme[6]: assertAlmostEqual does not accept float second arg
        self.assertAlmostEqual(args.neutral_base, 5.0)

    def test_staleness_factor_flag(self) -> None:
        args = self.parser.parse_args(["--staleness-factor", "0.1"])
        # pyre-fixme[6]: assertAlmostEqual does not accept float second arg
        self.assertAlmostEqual(args.staleness_factor, 0.1)

    def test_all_defaults_are_none(self) -> None:
        args = self.parser.parse_args([])
        self.assertIsNone(args.exponent)
        self.assertIsNone(args.floor_weight)
        self.assertIsNone(args.neutral_base)
        self.assertIsNone(args.staleness_factor)

    def test_all_flags_together(self) -> None:
        args = self.parser.parse_args([
            "--seed", "7",
            "--exponent", "1.8",
            "--floor-weight", "0.7",
            "--neutral-base", "4.0",
            "--staleness-factor", "0.5",
        ])
        self.assertEqual(args.seed, 7)
        # pyre-fixme[6]: assertAlmostEqual does not accept float second arg
        self.assertAlmostEqual(args.exponent, 1.8)
        # pyre-fixme[6]: assertAlmostEqual does not accept float second arg
        self.assertAlmostEqual(args.floor_weight, 0.7)
        # pyre-fixme[6]: assertAlmostEqual does not accept float second arg
        self.assertAlmostEqual(args.neutral_base, 4.0)
        # pyre-fixme[6]: assertAlmostEqual does not accept float second arg
        self.assertAlmostEqual(args.staleness_factor, 0.5)


class TestQuestStartBannerWithCards(unittest.TestCase):
    """Tests for the updated quest_start_banner with unique_cards parameter."""

    def test_banner_with_unique_cards(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(
            seed=42,
            starting_essence=250,
            pool_size=660,
            unique_cards=220,
        )
        self.assertIn("220 cards", result)
        self.assertIn("660 entries", result)
        self.assertIn("42", result)
        self.assertIn("250", result)

    def test_banner_without_unique_cards(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(seed=42, starting_essence=250, pool_size=660)
        self.assertIn("660 entries", result)
        self.assertNotIn("cards", result)

    def test_banner_contains_header(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(
            seed=1,
            starting_essence=300,
            pool_size=500,
            unique_cards=100,
        )
        self.assertIn("DREAMTIDES QUEST", result)
        self.assertIn("Seed: 1", result)

    def test_banner_contains_press_enter(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(seed=1, starting_essence=250, pool_size=500)
        self.assertIn("Press Enter to begin", result)


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
            with patch("quest_sim.input_handler.ensure_terminal_restored") as mock_restore:
                with self.assertRaises(SystemExit) as ctx:
                    import quest_sim
                    quest_sim._run_with_error_handling()
                mock_restore.assert_called_once()

    def test_generic_exception_restores_terminal(self) -> None:
        """Unhandled exceptions should restore terminal before re-raising."""
        from unittest.mock import MagicMock, patch

        with patch("quest_sim.main", side_effect=RuntimeError("boom")):
            with patch("quest_sim.input_handler.ensure_terminal_restored") as mock_restore:
                with self.assertRaises(SystemExit):
                    import quest_sim
                    quest_sim._run_with_error_handling()
                mock_restore.assert_called()


if __name__ == "__main__":
    unittest.main()
