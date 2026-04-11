#!/usr/bin/env python3

"""Unit tests for the balance test CLI helpers."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
SCRIPTS_DIR = SCRIPT_DIR.parent.parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import balance_test


class BalanceTestCliTests(unittest.TestCase):
    """Tests for balance_test argument parsing and mode selection."""

    def test_parse_args_accepts_mode_and_no_build(self) -> None:
        args = balance_test.parse_args(["--no-build", "--mode", "coin"])

        self.assertTrue(args.no_build)
        self.assertEqual(args.mode, "coin")

    def test_selected_modes_returns_only_requested_mode(self) -> None:
        self.assertEqual(
            balance_test.selected_modes("bonus-energy", []), ["bonus-energy"]
        )


if __name__ == "__main__":
    unittest.main()
