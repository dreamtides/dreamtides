"""Tests for quest_sim entry point.

Validates CLI argument parsing, initialization sequence, and banner
display without requiring interactive terminal input.
"""

import argparse
import os
import sys
import unittest
from typing import Any, Optional

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


class TestQuestStartBannerPoolStats(unittest.TestCase):
    """Tests for pool statistics in the quest start banner."""

    def _make_banner(
        self,
        variance: Optional[dict[Any, float]] = None,
        rarity_entries: Optional[dict[Any, int]] = None,
        algorithm_params_str: Optional[str] = None,
    ) -> str:
        from models import Rarity, Resonance
        from render import quest_start_banner

        v: Optional[dict[Resonance, float]] = variance or {
            Resonance.TIDE: 1.12,
            Resonance.EMBER: 0.92,
            Resonance.ZEPHYR: 1.03,
            Resonance.STONE: 0.85,
            Resonance.RUIN: 1.18,
        }
        re: Optional[dict[Rarity, int]] = rarity_entries or {
            Rarity.COMMON: 308,
            Rarity.UNCOMMON: 261,
            Rarity.RARE: 78,
            Rarity.LEGENDARY: 13,
        }
        return quest_start_banner(
            seed=42,
            starting_essence=250,
            pool_size=660,
            unique_cards=220,
            pool_variance=v,
            rarity_entries=re,
            algorithm_params_str=algorithm_params_str,
        )

    def test_banner_contains_pool_bias(self) -> None:
        result = self._make_banner()
        self.assertIn("Pool bias:", result)

    def test_banner_contains_resonance_bias_values(self) -> None:
        result = self._make_banner()
        self.assertIn("+12%", result)
        self.assertIn("-8%", result)

    def test_banner_contains_composition_breakdown(self) -> None:
        result = self._make_banner()
        self.assertIn("Common:", result)
        self.assertIn("Uncommon:", result)

    def test_banner_contains_rarity_percentages(self) -> None:
        result = self._make_banner()
        self.assertIn("46.7%", result)

    def test_banner_shows_algorithm_params_when_provided(self) -> None:
        result = self._make_banner(
            algorithm_params_str="  Algorithm: exponent=2.0, floor=0.5, neutral=3.0, staleness=0.3"
        )
        self.assertIn("Algorithm:", result)
        self.assertIn("exponent=2.0", result)

    def test_banner_omits_algorithm_params_when_none(self) -> None:
        result = self._make_banner(algorithm_params_str=None)
        self.assertNotIn("Algorithm:", result)

    def test_banner_still_contains_header_and_footer(self) -> None:
        result = self._make_banner()
        self.assertIn("DREAMTIDES QUEST", result)
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
