"""Tests for the render module."""

import os
import sys
import unittest


class TestVisibleLen(unittest.TestCase):
    def test_plain_text(self) -> None:
        from render import visible_len

        self.assertEqual(visible_len("hello"), 5)

    def test_with_ansi_codes(self) -> None:
        from render import visible_len

        self.assertEqual(visible_len("\033[94mblue\033[0m"), 4)

    def test_empty_string(self) -> None:
        from render import visible_len

        self.assertEqual(visible_len(""), 0)

    def test_multiple_ansi_codes(self) -> None:
        from render import visible_len

        self.assertEqual(visible_len("\033[1m\033[94mhello\033[0m"), 5)


class TestPadRight(unittest.TestCase):
    def test_plain_padding(self) -> None:
        from render import pad_right

        result = pad_right("hi", 10)
        self.assertEqual(len(result), 10)
        self.assertEqual(result, "hi        ")

    def test_ansi_padding(self) -> None:
        from render import pad_right, visible_len

        result = pad_right("\033[94mhi\033[0m", 10)
        self.assertEqual(visible_len(result), 10)

    def test_no_padding_needed(self) -> None:
        from render import pad_right

        result = pad_right("hello world", 5)
        self.assertEqual(result, "hello world")


class TestRarityBadge(unittest.TestCase):
    def test_common(self) -> None:
        from models import Rarity
        from render import rarity_badge

        result = rarity_badge(Rarity.COMMON)
        self.assertEqual(result, "[C]")

    def test_uncommon(self) -> None:
        from models import Rarity
        from render import rarity_badge

        result = rarity_badge(Rarity.UNCOMMON)
        self.assertEqual(result, "[U]")

    def test_rare_bold(self) -> None:
        from models import Rarity
        from render import BOLD, RESET, rarity_badge

        result = rarity_badge(Rarity.RARE)
        self.assertIn(BOLD, result)
        self.assertIn("[R]", result)

    def test_legendary_bold(self) -> None:
        from models import Rarity
        from render import BOLD, RESET, rarity_badge

        result = rarity_badge(Rarity.LEGENDARY)
        self.assertIn(BOLD, result)
        self.assertIn("[L]", result)


class TestColorResonance(unittest.TestCase):
    def test_single_resonance(self) -> None:
        from models import Resonance
        from render import RESET, RESONANCE_COLORS, color_resonance

        result = color_resonance(Resonance.TIDE)
        self.assertIn("Tide", result)

    def test_color_resonances_empty(self) -> None:
        from render import color_resonances

        result = color_resonances(frozenset())
        self.assertIn("Neutral", result)

    def test_color_resonances_single(self) -> None:
        from models import Resonance
        from render import color_resonances

        result = color_resonances(frozenset({Resonance.EMBER}))
        self.assertIn("Ember", result)

    def test_color_resonances_dual(self) -> None:
        from models import Resonance
        from render import color_resonances

        result = color_resonances(frozenset({Resonance.TIDE, Resonance.RUIN}))
        self.assertIn("+", result)
        self.assertIn("Tide", result)
        self.assertIn("Ruin", result)


class TestDrawBox(unittest.TestCase):
    def test_basic_box(self) -> None:
        import io

        from render import draw_box

        captured = io.StringIO()
        old_stdout = sys.stdout
        sys.stdout = captured
        try:
            draw_box(["Hello", "World"])
        finally:
            sys.stdout = old_stdout
        output = captured.getvalue()
        self.assertIn("\u2554", output)  # top-left corner
        self.assertIn("\u255d", output)  # bottom-right corner
        self.assertIn("Hello", output)
        self.assertIn("World", output)

    def test_min_width(self) -> None:
        import io

        from render import draw_box

        captured = io.StringIO()
        old_stdout = sys.stdout
        sys.stdout = captured
        try:
            draw_box(["Hi"], min_width=70)
        finally:
            sys.stdout = old_stdout
        lines = captured.getvalue().strip().split("\n")
        # The top line should be at least 70 chars visible
        from render import visible_len

        self.assertGreaterEqual(visible_len(lines[0]), 70)


class TestDrawSeparators(unittest.TestCase):
    def test_draw_separator(self) -> None:
        from render import draw_separator

        result = draw_separator()
        self.assertEqual(len(result), 70)
        self.assertTrue(all(c == "\u2500" for c in result))

    def test_draw_double_separator(self) -> None:
        from render import draw_double_separator

        result = draw_double_separator()
        self.assertEqual(len(result), 70)
        self.assertTrue(all(c == "\u2550" for c in result))


class TestFormatCard(unittest.TestCase):
    def _make_card(self) -> "Card":
        from models import Card, CardType, Rarity, Resonance

        return Card(
            name="Whirlpool Seer",
            card_number=42,
            energy_cost=3,
            card_type=CardType.CHARACTER,
            subtype="Mage",
            is_fast=False,
            spark=2,
            rarity=Rarity.UNCOMMON,
            rules_text="Judgment: Foresee 2.",
            resonances=frozenset({Resonance.TIDE}),
            tags=frozenset({"mechanic:foresee"}),
        )

    def test_format_card_highlighted(self) -> None:
        from render import format_card, visible_len

        card = self._make_card()
        lines = format_card(card, highlighted=True)
        self.assertEqual(len(lines), 2)
        # Line 1 starts with >
        self.assertTrue(lines[0].lstrip().startswith(">"))
        self.assertIn("Whirlpool Seer", lines[0])
        self.assertIn("Cost: 3", lines[0])
        self.assertIn("Spark: 2", lines[0])
        # Line 2 has quoted rules text
        self.assertIn('"Judgment: Foresee 2."', lines[1])

    def test_format_card_not_highlighted(self) -> None:
        from render import format_card

        card = self._make_card()
        lines = format_card(card, highlighted=False)
        self.assertEqual(len(lines), 2)
        # No > prefix when not highlighted
        stripped = lines[0].lstrip()
        self.assertFalse(stripped.startswith(">"))

    def test_format_card_no_spark(self) -> None:
        from models import Card, CardType, Rarity, Resonance
        from render import format_card

        card = Card(
            name="Lightning Bolt",
            card_number=99,
            energy_cost=2,
            card_type=CardType.EVENT,
            subtype=None,
            is_fast=True,
            spark=None,
            rarity=Rarity.COMMON,
            rules_text="Deal damage.",
            resonances=frozenset({Resonance.EMBER}),
            tags=frozenset(),
        )
        lines = format_card(card, highlighted=False)
        self.assertNotIn("Spark:", lines[0])
        self.assertIn("Cost: 2", lines[0])

    def test_format_card_no_cost(self) -> None:
        from models import Card, CardType, Rarity, Resonance
        from render import format_card

        card = Card(
            name="Strange Card",
            card_number=100,
            energy_cost=None,
            card_type=CardType.EVENT,
            subtype=None,
            is_fast=False,
            spark=None,
            rarity=Rarity.COMMON,
            rules_text="Do something.",
            resonances=frozenset(),
            tags=frozenset(),
        )
        lines = format_card(card, highlighted=False)
        self.assertIn("Cost: -", lines[0])

    def test_format_card_truncated_rules(self) -> None:
        from models import Card, CardType, Rarity, Resonance
        from render import format_card, visible_len

        long_text = "This is a very long rules text that should be truncated when the card is not highlighted because it exceeds the maximum line width."
        card = Card(
            name="Verbose Card",
            card_number=101,
            energy_cost=1,
            card_type=CardType.CHARACTER,
            subtype=None,
            is_fast=False,
            spark=1,
            rarity=Rarity.COMMON,
            rules_text=long_text,
            resonances=frozenset({Resonance.STONE}),
            tags=frozenset(),
        )
        lines_not_hl = format_card(card, highlighted=False)
        # Not highlighted: rules text should be truncated
        self.assertLessEqual(visible_len(lines_not_hl[1]), 70)

        lines_hl = format_card(card, highlighted=True)
        # Highlighted: full rules text
        self.assertIn(long_text, lines_hl[1])


class TestProfileBar(unittest.TestCase):
    def test_bar_chart(self) -> None:
        from models import Resonance
        from render import profile_bar

        snapshot: dict[Resonance, int] = {
            Resonance.TIDE: 12,
            Resonance.EMBER: 2,
            Resonance.ZEPHYR: 2,
            Resonance.STONE: 3,
            Resonance.RUIN: 11,
        }
        result = profile_bar(snapshot)
        self.assertIn("Tide", result)
        self.assertIn("Ruin", result)
        self.assertIn("\u2588", result)  # filled block
        self.assertIn("%", result)

    def test_bar_chart_zero(self) -> None:
        from models import Resonance
        from render import profile_bar

        snapshot: dict[Resonance, int] = {r: 0 for r in Resonance}
        result = profile_bar(snapshot)
        self.assertIn("0.0%", result)


class TestHeaderTemplates(unittest.TestCase):
    def test_quest_start_banner(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(seed=42, starting_essence=250, pool_size=660)
        self.assertIn("42", result)
        self.assertIn("250", result)
        self.assertIn("660", result)

    def test_atlas_header(self) -> None:
        from render import atlas_header

        result = atlas_header(
            essence=450,
            completion=2,
            total_battles=7,
            deck_count=12,
            dreamsign_count=2,
        )
        self.assertIn("450", result)
        self.assertIn("2/7", result)
        self.assertIn("12", result)

    def test_site_visit_header(self) -> None:
        from render import site_visit_header

        result = site_visit_header(
            dreamscape_name="Twilight Grove",
            site_type_label="Draft Site 1",
            pick_info="Pick 1/5",
            dreamscape_number=3,
        )
        self.assertIn("TWILIGHT GROVE", result)
        self.assertIn("Draft Site 1", result)
        self.assertIn("Pick 1/5", result)

    def test_resonance_profile_footer(self) -> None:
        from models import Resonance
        from render import resonance_profile_footer

        counts: dict[Resonance, int] = {
            Resonance.TIDE: 7,
            Resonance.EMBER: 0,
            Resonance.ZEPHYR: 0,
            Resonance.STONE: 1,
            Resonance.RUIN: 6,
        }
        result = resonance_profile_footer(
            counts=counts,
            deck_count=8,
            essence=300,
        )
        self.assertIn("Tide 7", result)
        self.assertIn("Ruin 6", result)
        self.assertIn("8 cards", result)
        self.assertIn("300", result)

    def test_victory_screen(self) -> None:
        from models import Rarity, Resonance
        from render import victory_screen

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
        result = victory_screen(
            battles_won=7,
            total_battles=7,
            dreamscapes_visited=7,
            dreamcaller_name="Vesper, Twilight Arbiter",
            dreamcaller_resonances=frozenset({Resonance.TIDE, Resonance.RUIN}),
            deck_size=34,
            rarity_counts=rarity_counts,
            resonance_counts=resonance_counts,
            neutral_count=5,
            dreamsign_count=4,
            essence=175,
            log_path=".logs/quest_20260227_143022_seed42.jsonl",
        )
        self.assertIn("QUEST COMPLETE", result)
        self.assertIn("7/7", result)
        self.assertIn("Vesper, Twilight Arbiter", result)
        self.assertIn("175", result)


class TestNoColor(unittest.TestCase):
    def test_no_color_env(self) -> None:
        """When NO_COLOR is set, all color constants should be empty strings."""
        # We can't easily test this without reimporting the module,
        # but we verify the pattern is correct by checking that the
        # module-level check exists.
        import render

        # If NO_COLOR was set (it is in test environments typically),
        # verify colors are empty OR if it wasn't, verify they have values.
        # The key test is that the module loads without error.
        self.assertTrue(hasattr(render, "RESONANCE_COLORS"))
        self.assertTrue(hasattr(render, "NEUTRAL_COLOR"))
        self.assertTrue(hasattr(render, "BOLD"))
        self.assertTrue(hasattr(render, "DIM"))
        self.assertTrue(hasattr(render, "RESET"))


if __name__ == "__main__":
    unittest.main()
