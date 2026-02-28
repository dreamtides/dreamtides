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


class TestRenderShopGrid(unittest.TestCase):
    """Tests for the render_shop_grid function."""

    def _make_shop_items(
        self, count: int = 6, discount_index: int = 2
    ) -> list[tuple[Card, int, int | None]]:
        """Create shop items as (card, price, original_price_or_none) tuples."""
        cards_data = [
            ("Tide Runner", Rarity.COMMON, frozenset({Resonance.TIDE}), 50),
            ("Ember Guard", Rarity.UNCOMMON, frozenset({Resonance.EMBER}), 80),
            ("Stone Titan", Rarity.RARE, frozenset({Resonance.STONE}), 120),
            ("Zephyr Scout", Rarity.COMMON, frozenset({Resonance.ZEPHYR}), 50),
            ("Ruin Walker", Rarity.UNCOMMON, frozenset({Resonance.RUIN}), 80),
            ("Dual Blade", Rarity.LEGENDARY, frozenset({Resonance.TIDE, Resonance.EMBER}), 200),
        ]
        items: list[tuple[Card, int, int | None]] = []
        for i in range(min(count, len(cards_data))):
            name, rarity, resonances, price = cards_data[i]
            card = _make_card(name=name, rarity=rarity, resonances=resonances)
            if i == discount_index:
                items.append((card, 60, price))  # discounted from original
            else:
                items.append((card, price, None))
        return items

    def test_grid_has_two_rows(self) -> None:
        from render_cards import render_shop_grid

        items = self._make_shop_items()
        output = render_shop_grid(items)
        # With 6 items in 2 rows of 3, there should be content from all 6
        self.assertIn("Tide Runner", output)
        self.assertIn("Zephyr Scout", output)

    def test_all_items_present(self) -> None:
        from render_cards import render_shop_grid

        items = self._make_shop_items()
        output = render_shop_grid(items)
        for card, _price, _orig in items:
            self.assertIn(card.name, output)

    def test_prices_displayed(self) -> None:
        from render_cards import render_shop_grid

        items = self._make_shop_items()
        output = render_shop_grid(items)
        # Non-discounted common price
        self.assertIn("50e", output)

    def test_discount_shows_both_prices(self) -> None:
        from render_cards import render_shop_grid

        items = self._make_shop_items(discount_index=2)
        output = render_shop_grid(items)
        # Original price of the discounted item (120) and discounted (60)
        self.assertIn("120", output)
        self.assertIn("60e", output)

    def test_rarity_badges_present(self) -> None:
        from render_cards import render_shop_grid

        items = self._make_shop_items()
        output = render_shop_grid(items)
        self.assertIn("[C]", output)
        self.assertIn("[U]", output)
        self.assertIn("[R]", output)

    def test_fewer_than_six_items(self) -> None:
        from render_cards import render_shop_grid

        items = self._make_shop_items(count=3)
        output = render_shop_grid(items)
        self.assertIn("Tide Runner", output)
        self.assertIn("Stone Titan", output)

    def test_column_width_constraint(self) -> None:
        from render import visible_len
        from render_cards import render_shop_grid

        items = self._make_shop_items()
        output = render_shop_grid(items)
        for line in output.split("\n"):
            self.assertLessEqual(
                visible_len(line), 70,
                f"Line too wide: {repr(line)}"
            )

    def test_rules_text_present(self) -> None:
        from render_cards import render_shop_grid

        items = self._make_shop_items()
        output = render_shop_grid(items)
        # Rules text should appear (may be truncated)
        self.assertIn("Judgment", output)


class TestWeightBar(unittest.TestCase):
    """Tests for the weight_bar function."""

    def test_full_bar_at_max_weight(self) -> None:
        from render_cards import weight_bar

        bar = weight_bar(5.0, 5.0, width=10)
        # At max weight, bar should be all filled blocks
        self.assertEqual(bar, "\u2588" * 10)

    def test_empty_bar_at_zero_weight(self) -> None:
        from render_cards import weight_bar

        bar = weight_bar(0.0, 5.0, width=10)
        # At zero weight, bar should be all empty blocks
        self.assertEqual(bar, "\u2591" * 10)

    def test_half_bar(self) -> None:
        from render_cards import weight_bar

        bar = weight_bar(2.5, 5.0, width=10)
        self.assertEqual(len(bar), 10)
        filled = bar.count("\u2588")
        empty = bar.count("\u2591")
        self.assertEqual(filled, 5)
        self.assertEqual(empty, 5)

    def test_proportional_scaling(self) -> None:
        from render_cards import weight_bar

        bar_low = weight_bar(1.0, 10.0, width=10)
        bar_high = weight_bar(8.0, 10.0, width=10)
        filled_low = bar_low.count("\u2588")
        filled_high = bar_high.count("\u2588")
        self.assertGreater(filled_high, filled_low)

    def test_zero_max_weight(self) -> None:
        from render_cards import weight_bar

        bar = weight_bar(0.0, 0.0, width=10)
        self.assertEqual(bar, "\u2591" * 10)

    def test_default_width(self) -> None:
        from render_cards import weight_bar

        bar = weight_bar(5.0, 5.0)
        self.assertEqual(len(bar), 10)


class TestResonanceMatchIndicator(unittest.TestCase):
    """Tests for the resonance_match_indicator function."""

    def test_match_when_card_resonance_in_top(self) -> None:
        from render_cards import resonance_match_indicator

        top_resonances = frozenset({Resonance.TIDE, Resonance.RUIN})
        card_resonances = frozenset({Resonance.TIDE})
        result = resonance_match_indicator(card_resonances, top_resonances)
        self.assertIn("match", result.lower())
        self.assertNotIn("partial", result.lower())
        self.assertNotIn("off-color", result.lower())

    def test_partial_when_some_overlap(self) -> None:
        from render_cards import resonance_match_indicator

        top_resonances = frozenset({Resonance.TIDE, Resonance.RUIN})
        card_resonances = frozenset({Resonance.TIDE, Resonance.EMBER})
        result = resonance_match_indicator(card_resonances, top_resonances)
        self.assertIn("partial", result.lower())

    def test_off_color_when_no_overlap(self) -> None:
        from render_cards import resonance_match_indicator

        top_resonances = frozenset({Resonance.TIDE, Resonance.RUIN})
        card_resonances = frozenset({Resonance.EMBER})
        result = resonance_match_indicator(card_resonances, top_resonances)
        self.assertIn("off-color", result.lower())

    def test_neutral_card_is_off_color(self) -> None:
        from render_cards import resonance_match_indicator

        top_resonances = frozenset({Resonance.TIDE, Resonance.RUIN})
        card_resonances: frozenset[Resonance] = frozenset()
        result = resonance_match_indicator(card_resonances, top_resonances)
        self.assertIn("off-color", result.lower())

    def test_empty_top_resonances(self) -> None:
        from render_cards import resonance_match_indicator

        top_resonances: frozenset[Resonance] = frozenset()
        card_resonances = frozenset({Resonance.TIDE})
        result = resonance_match_indicator(card_resonances, top_resonances)
        # With no top resonances, everything is off-color
        self.assertIn("off-color", result.lower())


class TestFormatDraftCard(unittest.TestCase):
    """Tests for the format_draft_card function."""

    def test_includes_weight_bar(self) -> None:
        from render_cards import format_draft_card

        card = _make_card()
        lines = format_draft_card(
            card,
            weight=3.5,
            max_weight=5.0,
            highlighted=False,
            top_resonances=frozenset({Resonance.TIDE}),
        )
        combined = "\n".join(lines)
        # Should contain filled block characters
        self.assertIn("\u2588", combined)

    def test_includes_numeric_weight(self) -> None:
        from render_cards import format_draft_card

        card = _make_card()
        lines = format_draft_card(
            card,
            weight=4.2,
            max_weight=5.0,
            highlighted=False,
            top_resonances=frozenset({Resonance.TIDE}),
        )
        combined = "\n".join(lines)
        self.assertIn("wt:", combined)
        self.assertIn("4.2", combined)

    def test_includes_match_indicator(self) -> None:
        from render_cards import format_draft_card

        card = _make_card(resonances=frozenset({Resonance.TIDE}))
        lines = format_draft_card(
            card,
            weight=3.0,
            max_weight=5.0,
            highlighted=False,
            top_resonances=frozenset({Resonance.TIDE, Resonance.RUIN}),
        )
        combined = "\n".join(lines).lower()
        self.assertTrue(
            "match" in combined
            or "partial" in combined
            or "off-color" in combined,
        )

    def test_within_70_columns(self) -> None:
        from render import visible_len
        from render_cards import format_draft_card

        card = _make_card()
        lines = format_draft_card(
            card,
            weight=4.2,
            max_weight=5.0,
            highlighted=False,
            top_resonances=frozenset({Resonance.TIDE}),
        )
        for line in lines:
            self.assertLessEqual(
                visible_len(line), 70,
                f"Line exceeds 70 columns: {repr(line)}"
            )

    def test_highlighted_has_marker(self) -> None:
        from render_cards import format_draft_card

        card = _make_card()
        lines = format_draft_card(
            card,
            weight=3.0,
            max_weight=5.0,
            highlighted=True,
            top_resonances=frozenset({Resonance.TIDE}),
        )
        self.assertIn(">", lines[0])

    def test_has_more_lines_than_basic(self) -> None:
        from render_cards import format_card_display, format_draft_card

        card = _make_card()
        basic = format_card_display(card, highlighted=False)
        draft = format_draft_card(
            card,
            weight=3.0,
            max_weight=5.0,
            highlighted=False,
            top_resonances=frozenset({Resonance.TIDE}),
        )
        self.assertGreater(len(draft), len(basic))


class TestRenderDraftCardList(unittest.TestCase):
    """Tests for the render_draft_card_list function."""

    def test_renders_with_weight_bars(self) -> None:
        from render_cards import render_draft_card_list

        cards = [_make_card(name="Card A"), _make_card(name="Card B")]
        weights = [3.0, 5.0]
        top_res = frozenset({Resonance.TIDE})
        output = render_draft_card_list(
            cards, selected_index=0, weights=weights,
            top_resonances=top_res,
        )
        self.assertIn("Card A", output)
        self.assertIn("Card B", output)
        self.assertIn("\u2588", output)

    def test_weight_values_shown(self) -> None:
        from render_cards import render_draft_card_list

        cards = [_make_card(name="Card A")]
        weights = [4.2]
        top_res = frozenset({Resonance.TIDE})
        output = render_draft_card_list(
            cards, selected_index=0, weights=weights,
            top_resonances=top_res,
        )
        self.assertIn("wt:", output)

    def test_empty_list(self) -> None:
        from render_cards import render_draft_card_list

        output = render_draft_card_list(
            [], selected_index=-1, weights=[],
            top_resonances=frozenset(),
        )
        self.assertEqual(output, "")



class TestRenderFullDeckView(unittest.TestCase):
    """Tests for the render_full_deck_view function."""

    def _make_deck_cards(self) -> list[DeckCard]:
        """Create a small deck with mixed resonances and rarities."""
        return [
            DeckCard(card=_make_card(
                name="Tide Runner",
                rarity=Rarity.COMMON,
                resonances=frozenset({Resonance.TIDE}),
                energy_cost=2,
                spark=1,
            )),
            DeckCard(card=_make_card(
                name="Ember Guard",
                rarity=Rarity.UNCOMMON,
                resonances=frozenset({Resonance.EMBER}),
                energy_cost=3,
                spark=2,
            )),
            DeckCard(card=_make_card(
                name="Alpha Seer",
                rarity=Rarity.RARE,
                resonances=frozenset({Resonance.TIDE}),
                energy_cost=5,
                spark=3,
            )),
            DeckCard(card=_make_card(
                name="Neutral Wanderer",
                rarity=Rarity.COMMON,
                resonances=frozenset(),
                energy_cost=1,
                spark=0,
            )),
        ]

    def test_contains_deck_header(self) -> None:
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        output = render_full_deck_view(deck)
        self.assertIn("Deck: 4 cards", output)

    def test_contains_all_card_names(self) -> None:
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        output = render_full_deck_view(deck)
        self.assertIn("Tide Runner", output)
        self.assertIn("Ember Guard", output)
        self.assertIn("Alpha Seer", output)
        self.assertIn("Neutral Wanderer", output)

    def test_sorted_by_resonance_then_name(self) -> None:
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        output = render_full_deck_view(deck)
        # Ember comes before Tide alphabetically in resonance names,
        # and Neutral cards come last. Within Tide, Alpha Seer comes
        # before Tide Runner alphabetically.
        ember_pos = output.index("Ember Guard")
        alpha_pos = output.index("Alpha Seer")
        tide_pos = output.index("Tide Runner")
        neutral_pos = output.index("Neutral Wanderer")
        self.assertLess(ember_pos, alpha_pos)
        self.assertLess(alpha_pos, tide_pos)
        self.assertLess(tide_pos, neutral_pos)

    def test_contains_rarity_breakdown(self) -> None:
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        output = render_full_deck_view(deck)
        self.assertIn("Common", output)
        self.assertIn("Uncommon", output)
        self.assertIn("Rare", output)

    def test_contains_resonance_profile(self) -> None:
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        output = render_full_deck_view(deck)
        self.assertIn("Resonance Profile", output)

    def test_shows_rarity_badge(self) -> None:
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        output = render_full_deck_view(deck)
        self.assertIn("[C]", output)
        self.assertIn("[U]", output)
        self.assertIn("[R]", output)

    def test_shows_cost_and_spark(self) -> None:
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        output = render_full_deck_view(deck)
        self.assertIn("Cost:", output)
        self.assertIn("Spark:", output)

    def test_transfigured_card_shows_note(self) -> None:
        from render_cards import render_full_deck_view

        deck = [
            DeckCard(
                card=_make_card(name="Base Card", resonances=frozenset({Resonance.TIDE})),
                is_transfigured=True,
                transfig_note="Golden Base Card -- +1 effect",
            ),
        ]
        output = render_full_deck_view(deck)
        self.assertIn("Golden Base Card", output)

    def test_bane_card_marked(self) -> None:
        from render_cards import render_full_deck_view

        deck = [
            DeckCard(
                card=_make_card(name="Nightmare", resonances=frozenset()),
                is_bane=True,
            ),
        ]
        output = render_full_deck_view(deck)
        self.assertIn("BANE", output)

    def test_dreamsigns_displayed(self) -> None:
        from models import Dreamsign
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        dreamsigns = [
            Dreamsign(
                name="Moon Sigil",
                resonance=Resonance.TIDE,
                tags=frozenset(),
                effect_text="Draw 1 extra card.",
                is_bane=False,
            ),
        ]
        output = render_full_deck_view(deck, dreamsigns=dreamsigns)
        self.assertIn("Moon Sigil", output)
        self.assertIn("Dreamsigns", output)

    def test_dreamcaller_displayed(self) -> None:
        from models import Dreamcaller
        from render_cards import render_full_deck_view
        from types import MappingProxyType

        deck = self._make_deck_cards()
        dreamcaller = Dreamcaller(
            name="Vesper, Twilight Arbiter",
            resonances=frozenset({Resonance.TIDE, Resonance.RUIN}),
            resonance_bonus=MappingProxyType({"Tide": 4, "Ruin": 4}),
            tags=frozenset(),
            tag_bonus=MappingProxyType({}),
            essence_bonus=50,
            ability_text="Dissolve an enemy character to draw a card.",
        )
        output = render_full_deck_view(deck, dreamcaller=dreamcaller)
        self.assertIn("Vesper, Twilight Arbiter", output)
        self.assertIn("Dreamcaller", output)

    def test_essence_displayed(self) -> None:
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        output = render_full_deck_view(deck, essence=350)
        self.assertIn("350", output)

    def test_empty_deck(self) -> None:
        from render_cards import render_full_deck_view

        output = render_full_deck_view([])
        self.assertIn("Deck: 0 cards", output)


class TestImport(unittest.TestCase):
    """Test that the module can be imported cleanly."""

    def test_import_all(self) -> None:
        import render_cards

        self.assertTrue(hasattr(render_cards, "format_card_display"))
        self.assertTrue(hasattr(render_cards, "render_card_list"))
        self.assertTrue(hasattr(render_cards, "format_shop_card"))
        self.assertTrue(hasattr(render_cards, "format_deck_summary"))
        self.assertTrue(hasattr(render_cards, "render_shop_grid"))
        self.assertTrue(hasattr(render_cards, "weight_bar"))
        self.assertTrue(hasattr(render_cards, "resonance_match_indicator"))
        self.assertTrue(hasattr(render_cards, "format_draft_card"))
        self.assertTrue(hasattr(render_cards, "render_draft_card_list"))
        self.assertTrue(hasattr(render_cards, "render_full_deck_view"))


if __name__ == "__main__":
    unittest.main()
