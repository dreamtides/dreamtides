"""Tests for the render_cards module."""

import os
import sys
import unittest

# Ensure NO_COLOR is set before importing render modules so ANSI codes
# are empty strings, making assertions on visible content straightforward.
os.environ["NO_COLOR"] = "1"

sys.path.insert(0, os.path.join(os.path.dirname(__file__)))

from models import Card, CardType, DeckCard, Rarity, Resonance


def _make_card(
    name: str = "Whirlpool Seer",
    energy_cost: int | None = 3,
    spark: int | None = 2,
    rarity: Rarity = Rarity.UNCOMMON,
    rules_text: str = "Judgment: Foresee 2.",
    resonances: frozenset[Resonance] | None = None,
) -> Card:
    if resonances is None:
        resonances = frozenset({Resonance.TIDE})
    return Card(
        name=name,
        card_number=42,
        energy_cost=energy_cost,
        card_type=CardType.CHARACTER,
        subtype="Mage",
        is_fast=False,
        spark=spark,
        rarity=rarity,
        rules_text=rules_text,
        resonances=resonances,
        tags=frozenset({"mechanic:foresee"}),
    )


class TestFormatCardDisplay(unittest.TestCase):
    """Tests for the format_card_display function."""

    def test_basic_two_line_format(self) -> None:
        from render_cards import format_card_display

        card = _make_card()
        lines = format_card_display(card, highlighted=False)
        self.assertEqual(len(lines), 2)

    def test_highlighted_has_marker(self) -> None:
        from render_cards import format_card_display

        card = _make_card()
        lines = format_card_display(card, highlighted=True)
        self.assertIn(">", lines[0])

    def test_not_highlighted_no_marker(self) -> None:
        from render_cards import format_card_display

        card = _make_card()
        lines = format_card_display(card, highlighted=False)
        stripped = lines[0].lstrip()
        self.assertFalse(stripped.startswith(">"))

    def test_name_in_line1(self) -> None:
        from render_cards import format_card_display

        card = _make_card()
        lines = format_card_display(card, highlighted=False)
        self.assertIn("Whirlpool Seer", lines[0])

    def test_cost_in_line1(self) -> None:
        from render_cards import format_card_display

        card = _make_card()
        lines = format_card_display(card, highlighted=False)
        self.assertIn("Cost: 3", lines[0])

    def test_spark_in_line1(self) -> None:
        from render_cards import format_card_display

        card = _make_card()
        lines = format_card_display(card, highlighted=False)
        self.assertIn("Spark: 2", lines[0])

    def test_rarity_badge_in_line1(self) -> None:
        from render_cards import format_card_display

        card = _make_card()
        lines = format_card_display(card, highlighted=False)
        self.assertIn("[U]", lines[0])

    def test_rules_text_quoted_in_line2(self) -> None:
        from render_cards import format_card_display

        card = _make_card()
        lines = format_card_display(card, highlighted=False)
        self.assertIn('"Judgment: Foresee 2."', lines[1])

    def test_no_spark_omitted(self) -> None:
        from render_cards import format_card_display

        card = _make_card(spark=None)
        lines = format_card_display(card, highlighted=False)
        self.assertNotIn("Spark:", lines[0])

    def test_no_cost_shows_dash(self) -> None:
        from render_cards import format_card_display

        card = _make_card(energy_cost=None)
        lines = format_card_display(card, highlighted=False)
        self.assertIn("Cost: -", lines[0])

    def test_highlighted_full_rules_text(self) -> None:
        from render_cards import format_card_display

        long_text = "A" * 200
        card = _make_card(rules_text=long_text)
        lines = format_card_display(card, highlighted=True)
        self.assertIn(long_text, lines[1])

    def test_not_highlighted_truncates_long_rules(self) -> None:
        from render import visible_len
        from render_cards import format_card_display

        long_text = "This is a very long rules text that should be truncated " * 3
        card = _make_card(rules_text=long_text)
        lines = format_card_display(card, highlighted=False)
        self.assertLessEqual(visible_len(lines[1]), 70)

    def test_accepts_deck_card(self) -> None:
        from render_cards import format_card_display

        card = _make_card()
        deck_card = DeckCard(card=card)
        lines = format_card_display(deck_card, highlighted=False)
        self.assertEqual(len(lines), 2)
        self.assertIn("Whirlpool Seer", lines[0])

    def test_transfigured_shows_note(self) -> None:
        from render_cards import format_card_display

        card = _make_card()
        deck_card = DeckCard(
            card=card,
            is_transfigured=True,
            transfig_note="Golden Whirlpool Seer -- +2 Spark",
        )
        lines = format_card_display(deck_card, highlighted=False)
        # The transfig note should appear somewhere in the output
        combined = "\n".join(lines)
        self.assertIn("Golden Whirlpool Seer", combined)

    def test_resonance_in_line1(self) -> None:
        from render_cards import format_card_display

        card = _make_card(resonances=frozenset({Resonance.TIDE}))
        lines = format_card_display(card, highlighted=False)
        self.assertIn("Tide", lines[0])

    def test_neutral_card_resonance(self) -> None:
        from render_cards import format_card_display

        card = _make_card(resonances=frozenset())
        lines = format_card_display(card, highlighted=False)
        self.assertIn("Neutral", lines[0])


class TestRenderCardList(unittest.TestCase):
    """Tests for the render_card_list function."""

    def test_renders_multiple_cards(self) -> None:
        from render_cards import render_card_list

        cards = [_make_card(name="Card A"), _make_card(name="Card B")]
        output = render_card_list(cards, selected_index=0)
        self.assertIn("Card A", output)
        self.assertIn("Card B", output)

    def test_selected_index_highlighted(self) -> None:
        from render_cards import render_card_list

        cards = [_make_card(name="Card A"), _make_card(name="Card B")]
        output = render_card_list(cards, selected_index=1)
        # Card B's line should contain the > marker
        output_lines = output.split("\n")
        card_b_line = [l for l in output_lines if "Card B" in l]
        self.assertTrue(len(card_b_line) > 0)
        self.assertIn(">", card_b_line[0])

    def test_no_selection(self) -> None:
        from render_cards import render_card_list

        cards = [_make_card(name="Card A")]
        output = render_card_list(cards, selected_index=-1)
        self.assertIn("Card A", output)

    def test_with_weights(self) -> None:
        from render_cards import render_card_list

        cards = [_make_card(name="Card A"), _make_card(name="Card B")]
        weights = [0.75, 0.25]
        output = render_card_list(cards, selected_index=0, weights=weights)
        self.assertIn("Card A", output)

    def test_empty_list(self) -> None:
        from render_cards import render_card_list

        output = render_card_list([], selected_index=-1)
        self.assertEqual(output, "")


class TestFormatShopCard(unittest.TestCase):
    """Tests for the format_shop_card function."""

    def test_includes_price(self) -> None:
        from render_cards import format_shop_card

        card = _make_card()
        lines = format_shop_card(card, price=80, highlighted=False)
        combined = "\n".join(lines)
        self.assertIn("80", combined)

    def test_discount_display(self) -> None:
        from render_cards import format_shop_card

        card = _make_card()
        lines = format_shop_card(
            card, price=80, highlighted=False,
            original_price=120,
        )
        combined = "\n".join(lines)
        self.assertIn("80", combined)
        self.assertIn("120", combined)

    def test_no_discount(self) -> None:
        from render_cards import format_shop_card

        card = _make_card()
        lines = format_shop_card(card, price=50, highlighted=False)
        # Should have at least 3 lines: card line1, card line2, price line
        self.assertGreaterEqual(len(lines), 3)

    def test_highlighted_shop_card(self) -> None:
        from render_cards import format_shop_card

        card = _make_card()
        lines = format_shop_card(card, price=80, highlighted=True)
        self.assertIn(">", lines[0])


class TestDeckSummary(unittest.TestCase):
    """Tests for the format_deck_summary function."""

    def test_basic_summary(self) -> None:
        from render_cards import format_deck_summary

        cards = [
            DeckCard(card=_make_card(rarity=Rarity.COMMON)),
            DeckCard(card=_make_card(rarity=Rarity.COMMON)),
            DeckCard(card=_make_card(rarity=Rarity.UNCOMMON)),
            DeckCard(card=_make_card(rarity=Rarity.RARE)),
        ]
        output = format_deck_summary(cards)
        self.assertIn("4", output)  # total card count
        self.assertIn("Common", output)
        self.assertIn("Rare", output)

    def test_empty_deck(self) -> None:
        from render_cards import format_deck_summary

        output = format_deck_summary([])
        self.assertIn("0", output)

    def test_rarity_counts(self) -> None:
        from render_cards import format_deck_summary

        cards = [
            DeckCard(card=_make_card(rarity=Rarity.COMMON)),
            DeckCard(card=_make_card(rarity=Rarity.COMMON)),
            DeckCard(card=_make_card(rarity=Rarity.LEGENDARY)),
        ]
        output = format_deck_summary(cards)
        self.assertIn("Legendary", output)
        self.assertIn("3", output)  # total


class TestImport(unittest.TestCase):
    """Test that the module can be imported cleanly."""

    def test_import_all(self) -> None:
        import render_cards

        self.assertTrue(hasattr(render_cards, "format_card_display"))
        self.assertTrue(hasattr(render_cards, "render_card_list"))
        self.assertTrue(hasattr(render_cards, "format_shop_card"))
        self.assertTrue(hasattr(render_cards, "format_deck_summary"))


if __name__ == "__main__":
    unittest.main()
