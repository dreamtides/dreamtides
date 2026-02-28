"""Tests for the render_status module."""

import os
import sys
import unittest
from typing import Optional


class TestResonanceProfileFooter(unittest.TestCase):
    def test_contains_all_resonances(self) -> None:
        from models import Resonance
        from render_status import resonance_profile_footer

        counts: dict[Resonance, int] = {
            Resonance.TIDE: 7,
            Resonance.EMBER: 0,
            Resonance.ZEPHYR: 0,
            Resonance.STONE: 1,
            Resonance.RUIN: 6,
        }
        result = resonance_profile_footer(counts=counts, deck_count=8, essence=300)
        self.assertIn("Tide 7", result)
        self.assertIn("Ember 0", result)
        self.assertIn("Zephyr 0", result)
        self.assertIn("Stone 1", result)
        self.assertIn("Ruin 6", result)

    def test_contains_deck_and_essence(self) -> None:
        from models import Resonance
        from render_status import resonance_profile_footer

        counts: dict[Resonance, int] = {r: 0 for r in Resonance}
        result = resonance_profile_footer(counts=counts, deck_count=8, essence=300)
        self.assertIn("8 cards", result)
        self.assertIn("Essence: 300", result)

    def test_has_single_separator_above_double_below(self) -> None:
        from models import Resonance
        from render_status import resonance_profile_footer

        counts: dict[Resonance, int] = {r: 0 for r in Resonance}
        result = resonance_profile_footer(counts=counts, deck_count=5, essence=100)
        lines = result.split("\n")
        # First line should be single-line separator (box-drawing horizontal)
        self.assertTrue(all(c == "\u2500" for c in lines[0]))
        # Last line should be double-line separator
        self.assertTrue(all(c == "\u2550" for c in lines[-1]))

    def test_pipe_separator_between_resonances(self) -> None:
        from models import Resonance
        from render_status import resonance_profile_footer

        counts: dict[Resonance, int] = {r: 0 for r in Resonance}
        result = resonance_profile_footer(counts=counts, deck_count=5, essence=100)
        self.assertIn("|", result)


class TestProfileBar(unittest.TestCase):
    def test_contains_all_resonances(self) -> None:
        from models import Resonance
        from render_status import profile_bar

        snapshot: dict[Resonance, int] = {
            Resonance.TIDE: 12,
            Resonance.EMBER: 2,
            Resonance.ZEPHYR: 2,
            Resonance.STONE: 3,
            Resonance.RUIN: 11,
        }
        result = profile_bar(snapshot)
        self.assertIn("Tide", result)
        self.assertIn("Ember", result)
        self.assertIn("Zephyr", result)
        self.assertIn("Stone", result)
        self.assertIn("Ruin", result)

    def test_contains_filled_blocks_and_percentages(self) -> None:
        from models import Resonance
        from render_status import profile_bar

        snapshot: dict[Resonance, int] = {
            Resonance.TIDE: 12,
            Resonance.EMBER: 2,
            Resonance.ZEPHYR: 2,
            Resonance.STONE: 3,
            Resonance.RUIN: 11,
        }
        result = profile_bar(snapshot)
        self.assertIn("\u2588", result)  # filled block
        self.assertIn("%", result)

    def test_sorted_by_count_descending(self) -> None:
        from models import Resonance
        from render_status import profile_bar

        snapshot: dict[Resonance, int] = {
            Resonance.TIDE: 12,
            Resonance.EMBER: 2,
            Resonance.ZEPHYR: 2,
            Resonance.STONE: 3,
            Resonance.RUIN: 11,
        }
        result = profile_bar(snapshot)
        lines = result.strip().split("\n")
        # Tide (12) should be first, Ruin (11) second
        self.assertIn("Tide", lines[0])
        self.assertIn("Ruin", lines[1])

    def test_zero_counts_use_dim_bars(self) -> None:
        from models import Resonance
        from render_status import profile_bar

        snapshot: dict[Resonance, int] = {r: 0 for r in Resonance}
        result = profile_bar(snapshot)
        self.assertIn("0.0%", result)
        self.assertIn("\u2591", result)  # light shade block for empty

    def test_max_count_gets_full_bar(self) -> None:
        from models import Resonance
        from render_status import profile_bar

        snapshot: dict[Resonance, int] = {
            Resonance.TIDE: 10,
            Resonance.EMBER: 0,
            Resonance.ZEPHYR: 0,
            Resonance.STONE: 0,
            Resonance.RUIN: 0,
        }
        result = profile_bar(snapshot, bar_width=20)
        # The max-count resonance should get a full 20-char bar
        lines = result.strip().split("\n")
        # First line (Tide with 10) should have 20 filled blocks
        self.assertIn("\u2588" * 20, lines[0])


class TestSiteHeader(unittest.TestCase):
    def test_contains_dreamscape_name_uppercased(self) -> None:
        from render_status import site_header

        result = site_header(
            dreamscape_name="Twilight Grove",
            site_type_label="Draft Site 1",
            dreamscape_number=3,
        )
        self.assertIn("TWILIGHT GROVE", result)

    def test_contains_site_type_label(self) -> None:
        from render_status import site_header

        result = site_header(
            dreamscape_name="Twilight Grove",
            site_type_label="Draft Site 1",
            dreamscape_number=3,
        )
        self.assertIn("Draft Site 1", result)

    def test_contains_dreamscape_number(self) -> None:
        from render_status import site_header

        result = site_header(
            dreamscape_name="Twilight Grove",
            site_type_label="Draft Site 1",
            dreamscape_number=3,
        )
        self.assertIn("[Dreamscape 3]", result)

    def test_optional_pick_counter(self) -> None:
        from render_status import site_header

        result = site_header(
            dreamscape_name="Twilight Grove",
            site_type_label="Draft Site 1",
            dreamscape_number=3,
            pick_info="Pick 1/5",
        )
        self.assertIn("Pick 1/5", result)

    def test_no_pick_counter(self) -> None:
        from render_status import site_header

        result = site_header(
            dreamscape_name="Crystal Spire",
            site_type_label="Shop",
            dreamscape_number=5,
        )
        self.assertNotIn("Pick", result)
        self.assertIn("CRYSTAL SPIRE", result)
        self.assertIn("Shop", result)

    def test_has_double_separators(self) -> None:
        from render_status import site_header

        result = site_header(
            dreamscape_name="Test",
            site_type_label="Battle",
            dreamscape_number=1,
        )
        lines = result.split("\n")
        self.assertTrue(all(c == "\u2550" for c in lines[0]))
        self.assertTrue(all(c == "\u2550" for c in lines[-1]))


class TestVictoryScreen(unittest.TestCase):
    def _build_victory(
        self, log_path: "Optional[str]" = ".logs/quest_test.jsonl"
    ) -> str:
        from models import Rarity, Resonance
        from render_status import victory_screen

        resonance_counts: dict[Resonance, int] = {
            Resonance.TIDE: 12,
            Resonance.EMBER: 2,
            Resonance.ZEPHYR: 2,
            Resonance.STONE: 3,
            Resonance.RUIN: 11,
        }
        rarity_counts: dict[Rarity, int] = {
            Rarity.COMMON: 12,
            Rarity.UNCOMMON: 13,
            Rarity.RARE: 8,
            Rarity.LEGENDARY: 1,
        }
        return victory_screen(
            battles_won=7,
            total_battles=7,
            dreamscapes_visited=7,
            dreamcaller_name="Vesper, Twilight Arbiter",
            dreamcaller_resonances=frozenset(
                {Resonance.TIDE, Resonance.RUIN}
            ),
            deck_size=34,
            rarity_counts=rarity_counts,
            resonance_counts=resonance_counts,
            neutral_count=5,
            dreamsign_count=4,
            essence=175,
            log_path=log_path,
        )

    def test_contains_victory_header(self) -> None:
        result = self._build_victory()
        self.assertIn("QUEST COMPLETE", result)
        self.assertIn("VICTORY", result)

    def test_contains_battle_stats(self) -> None:
        result = self._build_victory()
        self.assertIn("7/7", result)
        self.assertIn("Battles won", result)

    def test_contains_dreamscapes_visited(self) -> None:
        result = self._build_victory()
        self.assertIn("Dreamscapes visited: 7", result)

    def test_contains_dreamcaller_name_and_resonances(self) -> None:
        result = self._build_victory()
        self.assertIn("Vesper, Twilight Arbiter", result)
        self.assertIn("Tide", result)
        self.assertIn("Ruin", result)

    def test_contains_deck_size(self) -> None:
        result = self._build_victory()
        self.assertIn("Final Deck: 34 cards", result)

    def test_contains_rarity_breakdown(self) -> None:
        result = self._build_victory()
        self.assertIn("Common", result)
        self.assertIn("Uncommon", result)
        self.assertIn("Rare", result)
        self.assertIn("Legendary", result)
        self.assertIn("%", result)

    def test_contains_resonance_profile(self) -> None:
        result = self._build_victory()
        self.assertIn("Resonance Profile", result)
        self.assertIn("\u2588", result)  # bar chart blocks

    def test_contains_neutral_count(self) -> None:
        result = self._build_victory()
        self.assertIn("Neutral", result)

    def test_contains_dreamsigns_and_essence(self) -> None:
        result = self._build_victory()
        self.assertIn("Dreamsigns: 4", result)
        self.assertIn("Essence remaining: 175", result)

    def test_contains_log_path(self) -> None:
        result = self._build_victory()
        self.assertIn(".logs/quest_test.jsonl", result)

    def test_no_log_path(self) -> None:
        result = self._build_victory(log_path=None)
        self.assertNotIn("Log written", result)

    def test_has_double_separator_bookends(self) -> None:
        result = self._build_victory()
        lines = result.split("\n")
        self.assertTrue(all(c == "\u2550" for c in lines[0]))
        self.assertTrue(all(c == "\u2550" for c in lines[-1]))


class TestImportability(unittest.TestCase):
    def test_wildcard_import(self) -> None:
        """Verify the module can be imported with wildcard syntax."""
        import subprocess

        result = subprocess.run(
            [
                sys.executable,
                "-c",
                (
                    "import sys; sys.path.insert(0,'scripts/quest_simulator');"
                    "from render_status import *"
                ),
            ],
            capture_output=True,
            text=True,
            env={**os.environ, "NO_COLOR": "1"},
            cwd=os.path.join(os.path.dirname(__file__), "..", ".."),
        )
        self.assertEqual(result.returncode, 0, f"stderr: {result.stderr}")


if __name__ == "__main__":
    unittest.main()
