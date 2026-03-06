"""Tests for the render module."""

import os
import sys
import unittest

# Ensure NO_COLOR is set before importing render modules so ANSI codes
# are empty strings, making assertions on visible content straightforward.
os.environ["NO_COLOR"] = "1"

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "draft_simulator"))
sys.path.insert(0, os.path.join(os.path.dirname(__file__)))


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

    def test_24bit_ansi_codes(self) -> None:
        from render import visible_len

        self.assertEqual(visible_len("\033[38;2;255;204;102mtext\033[0m"), 4)


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


class TestTruncateVisible(unittest.TestCase):
    def test_short_string_unchanged(self) -> None:
        from render import truncate_visible

        result = truncate_visible("hello", 10)
        self.assertEqual(result, "hello")

    def test_truncates_long_string(self) -> None:
        from render import truncate_visible, visible_len

        result = truncate_visible("hello world this is long", 10)
        self.assertLessEqual(visible_len(result), 10)


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


class TestHeaderTemplates(unittest.TestCase):
    def test_quest_start_banner(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(seed=42, starting_essence=250)
        self.assertIn("42", result)
        self.assertIn("250", result)
        self.assertIn("540", result)

    def test_quest_start_banner_custom_count(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(seed=42, starting_essence=250, card_count=100)
        self.assertIn("100", result)

    def test_quest_start_banner_no_pool_variance(self) -> None:
        from render import quest_start_banner

        result = quest_start_banner(seed=42, starting_essence=250)
        self.assertNotIn("variance", result.lower())
        self.assertNotIn("rarity", result.lower())
        self.assertNotIn("algorithm", result.lower())

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


class TestColorsIntegration(unittest.TestCase):
    def test_uses_colors_module(self) -> None:
        """Render module imports and uses the colors module."""
        import render

        self.assertTrue(hasattr(render, "colors"))

    def test_no_raw_ansi_constants(self) -> None:
        """BOLD/DIM/RESET/STRIKETHROUGH are not raw ANSI escape sequences."""
        import render

        # With NO_COLOR=1 set, the shims should be empty strings
        for name in ("BOLD", "DIM", "RESET", "STRIKETHROUGH"):
            val = getattr(render, name, "")
            self.assertEqual(val, "", f"{name} should be empty when NO_COLOR is set")


if __name__ == "__main__":
    unittest.main()
