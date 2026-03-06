"""Tests for the render_cards module."""

import os
import sys
import unittest

# Ensure NO_COLOR is set before importing render modules so ANSI codes
# are empty strings, making assertions on visible content straightforward.
os.environ["NO_COLOR"] = "1"

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "draft_simulator"))
sys.path.insert(0, os.path.join(os.path.dirname(__file__)))

from draft_models import CardDesign, CardInstance
from models import DeckCard, Dreamcaller, Dreamsign


def _make_design(
    name: str = "Whirlpool Seer",
    power: float = 0.72,
    commit: float = 0.60,
    flex: float = 0.30,
    fitness: list[float] | None = None,
) -> CardDesign:
    if fitness is None:
        fitness = [0.9, 0.1, 0.05, 0.0, 0.0, 0.0, 0.0, 0.05]
    return CardDesign(
        card_id="test_001",
        name=name,
        fitness=fitness,
        power=power,
        commit=commit,
        flex=flex,
    )


def _make_instance(
    name: str = "Whirlpool Seer",
    instance_id: int = 1001,
    **kwargs,
) -> CardInstance:
    return CardInstance(
        instance_id=instance_id, design=_make_design(name=name, **kwargs)
    )


def _make_deck_card(
    name: str = "Whirlpool Seer",
    is_bane: bool = False,
    is_transfigured: bool = False,
    transfig_note: str | None = None,
    **kwargs,
) -> DeckCard:
    return DeckCard(
        instance=_make_instance(name=name, **kwargs),
        is_bane=is_bane,
        is_transfigured=is_transfigured,
        transfig_note=transfig_note,
    )


class TestFormatCardDisplay(unittest.TestCase):
    """Tests for the format_card_display function."""

    def test_basic_two_line_format(self) -> None:
        from render_cards import format_card_display

        dc = _make_deck_card()
        lines = format_card_display(dc, highlighted=False)
        self.assertEqual(len(lines), 2)

    def test_highlighted_has_marker(self) -> None:
        from render_cards import format_card_display

        dc = _make_deck_card()
        lines = format_card_display(dc, highlighted=True)
        self.assertIn(">", lines[0])

    def test_not_highlighted_no_marker(self) -> None:
        from render_cards import format_card_display

        dc = _make_deck_card()
        lines = format_card_display(dc, highlighted=False)
        stripped = lines[0].lstrip()
        self.assertFalse(stripped.startswith(">"))

    def test_name_in_line1(self) -> None:
        from render_cards import format_card_display

        dc = _make_deck_card()
        lines = format_card_display(dc, highlighted=False)
        self.assertIn("Whirlpool Seer", lines[0])

    def test_power_in_line2(self) -> None:
        from render_cards import format_card_display

        dc = _make_deck_card()
        lines = format_card_display(dc, highlighted=False)
        self.assertIn("0.72", lines[1])

    def test_commit_in_line2(self) -> None:
        from render_cards import format_card_display

        dc = _make_deck_card()
        lines = format_card_display(dc, highlighted=False)
        self.assertIn("0.60", lines[1])

    def test_flex_in_line2(self) -> None:
        from render_cards import format_card_display

        dc = _make_deck_card()
        lines = format_card_display(dc, highlighted=False)
        self.assertIn("0.30", lines[1])

    def test_archetype_fitness_in_line2(self) -> None:
        from render_cards import format_card_display

        dc = _make_deck_card()
        lines = format_card_display(dc, highlighted=False)
        self.assertIn("A0=0.90", lines[1])

    def test_no_resonance_in_output(self) -> None:
        from render_cards import format_card_display

        dc = _make_deck_card()
        lines = format_card_display(dc, highlighted=False)
        combined = "\n".join(lines)
        self.assertNotIn("resonance", combined.lower())

    def test_no_rarity_in_output(self) -> None:
        from render_cards import format_card_display

        dc = _make_deck_card()
        lines = format_card_display(dc, highlighted=False)
        combined = "\n".join(lines)
        self.assertNotIn("rarity", combined.lower())

    def test_no_rules_text_in_output(self) -> None:
        from render_cards import format_card_display

        dc = _make_deck_card()
        lines = format_card_display(dc, highlighted=False)
        combined = "\n".join(lines)
        self.assertNotIn("rules_text", combined.lower())

    def test_bane_marker_shown(self) -> None:
        from render_cards import format_card_display

        dc = _make_deck_card(name="Cursed Shadow", is_bane=True)
        lines = format_card_display(dc, highlighted=False)
        combined = "\n".join(lines)
        self.assertIn("BANE", combined)

    def test_transfig_note_shown(self) -> None:
        from render_cards import format_card_display

        dc = _make_deck_card(
            name="Base Card",
            is_transfigured=True,
            transfig_note="Golden Base Card -- +1 effect",
        )
        lines = format_card_display(dc, highlighted=False)
        combined = "\n".join(lines)
        self.assertIn("Golden Base Card", combined)

    def test_accepts_card_instance(self) -> None:
        from render_cards import format_card_display

        instance = _make_instance()
        lines = format_card_display(instance, highlighted=False)
        self.assertEqual(len(lines), 2)
        self.assertIn("Whirlpool Seer", lines[0])

    def test_accepts_card_design(self) -> None:
        from render_cards import format_card_display

        design = _make_design()
        lines = format_card_display(design, highlighted=False)
        self.assertEqual(len(lines), 2)
        self.assertIn("Whirlpool Seer", lines[0])


class TestRenderFullDeckView(unittest.TestCase):
    """Tests for the render_full_deck_view function."""

    def _make_deck_cards(self) -> list[DeckCard]:
        return [
            _make_deck_card(name="Tide Runner", power=0.50, commit=0.30, flex=0.20),
            _make_deck_card(name="Ember Guard", power=0.60, commit=0.40, flex=0.10),
            _make_deck_card(name="Alpha Seer", power=0.80, commit=0.50, flex=0.30),
        ]

    def test_contains_deck_header(self) -> None:
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        output = render_full_deck_view(deck)
        self.assertIn("Deck: 3 cards", output)

    def test_contains_all_card_names(self) -> None:
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        output = render_full_deck_view(deck)
        self.assertIn("Tide Runner", output)
        self.assertIn("Ember Guard", output)
        self.assertIn("Alpha Seer", output)

    def test_sorted_by_name(self) -> None:
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        output = render_full_deck_view(deck)
        alpha_pos = output.index("Alpha Seer")
        ember_pos = output.index("Ember Guard")
        tide_pos = output.index("Tide Runner")
        self.assertLess(alpha_pos, ember_pos)
        self.assertLess(ember_pos, tide_pos)

    def test_bane_card_marked(self) -> None:
        from render_cards import render_full_deck_view

        deck = [_make_deck_card(name="Nightmare", is_bane=True)]
        output = render_full_deck_view(deck)
        self.assertIn("BANE", output)

    def test_transfigured_card_shows_note(self) -> None:
        from render_cards import render_full_deck_view

        deck = [
            _make_deck_card(
                name="Base Card",
                is_transfigured=True,
                transfig_note="Golden Base Card -- +1 effect",
            ),
        ]
        output = render_full_deck_view(deck)
        self.assertIn("Golden Base Card", output)

    def test_dreamsigns_displayed(self) -> None:
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        dreamsigns = [
            Dreamsign(
                name="Moon Sigil", effect_text="Draw 1 extra card.", is_bane=False
            ),
        ]
        output = render_full_deck_view(deck, dreamsigns=dreamsigns)
        self.assertIn("Moon Sigil", output)
        self.assertIn("Dreamsigns", output)

    def test_dreamcaller_displayed(self) -> None:
        from render_cards import render_full_deck_view

        deck = self._make_deck_cards()
        dreamcaller = Dreamcaller(
            name="Vesper, Twilight Arbiter",
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

    def test_import_core_functions(self) -> None:
        import render_cards

        self.assertTrue(hasattr(render_cards, "format_card_display"))
        self.assertTrue(hasattr(render_cards, "render_full_deck_view"))


if __name__ == "__main__":
    unittest.main()
