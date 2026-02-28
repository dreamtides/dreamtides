"""Tests for sites_transfig module."""

import random
import re
from typing import Optional
from unittest.mock import patch

from models import (
    Card,
    CardType,
    DeckCard,
    PoolEntry,
    Rarity,
    Resonance,
)
from quest_state import QuestState


def _make_card(
    name: str,
    card_number: int,
    rarity: Rarity = Rarity.COMMON,
    resonances: Optional[frozenset[Resonance]] = None,
    energy_cost: Optional[int] = 2,
    spark: Optional[int] = 1,
    card_type: CardType = CardType.CHARACTER,
    rules_text: Optional[str] = None,
    tags: Optional[frozenset[str]] = None,
) -> Card:
    return Card(
        name=name,
        card_number=card_number,
        energy_cost=energy_cost,
        card_type=card_type,
        subtype=None,
        is_fast=False,
        spark=spark,
        rarity=rarity,
        rules_text=rules_text or f"Rules for {name}.",
        resonances=resonances or frozenset(),
        tags=tags or frozenset(),
    )


def _make_pool(cards: list[Card]) -> list[PoolEntry]:
    return [PoolEntry(card) for card in cards]


def _make_quest_state(
    cards: Optional[list[Card]] = None,
    pool: Optional[list[PoolEntry]] = None,
    seed: int = 42,
    essence: int = 250,
) -> QuestState:
    test_cards = cards or []
    test_pool = pool or _make_pool(test_cards)
    rng = random.Random(seed)
    variance = {r: 1.0 for r in Resonance}
    return QuestState(
        essence=essence,
        pool=test_pool,
        rng=rng,
        all_cards=test_cards,
        pool_variance=variance,
    )


class TestTransfigurationEligibility:
    """Tests for transfiguration type eligibility checking."""

    def test_viridian_eligible_when_cost_positive(self) -> None:
        """Viridian requires energy_cost > 0."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card("Test Card", 1, energy_cost=3)
        assert is_eligible(card, TransfigType.VIRIDIAN)

    def test_viridian_ineligible_when_cost_zero(self) -> None:
        """Viridian requires energy_cost > 0; cost 0 should fail."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card("Test Card", 1, energy_cost=0)
        assert not is_eligible(card, TransfigType.VIRIDIAN)

    def test_viridian_ineligible_when_cost_none(self) -> None:
        """Viridian requires energy_cost > 0; None cost should fail."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card("Test Card", 1, energy_cost=None)
        assert not is_eligible(card, TransfigType.VIRIDIAN)

    def test_golden_eligible_when_rules_has_digits(self) -> None:
        """Golden requires digits in rules_text."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card("Test Card", 1, rules_text="Deal 3 damage.")
        assert is_eligible(card, TransfigType.GOLDEN)

    def test_golden_ineligible_when_no_digits(self) -> None:
        """Golden requires digits in rules_text; no digits should fail."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card("Test Card", 1, rules_text="Draw a card.")
        assert not is_eligible(card, TransfigType.GOLDEN)

    def test_scarlet_eligible_for_character(self) -> None:
        """Scarlet requires Character card type."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card("Test Card", 1, card_type=CardType.CHARACTER)
        assert is_eligible(card, TransfigType.SCARLET)

    def test_scarlet_ineligible_for_event(self) -> None:
        """Scarlet requires Character; Event should fail."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card("Test Card", 1, card_type=CardType.EVENT, spark=None)
        assert not is_eligible(card, TransfigType.SCARLET)

    def test_magenta_eligible_with_trigger_keyword(self) -> None:
        """Magenta requires trigger keywords in rules_text."""
        from sites_transfig import is_eligible, TransfigType

        for keyword in ["judgment", "whenever", "at the start", "at the end", "when"]:
            card = _make_card(
                "Test Card",
                1,
                rules_text=f"Some text {keyword} something happens.",
            )
            assert is_eligible(
                card, TransfigType.MAGENTA
            ), f"Expected Magenta eligible for keyword '{keyword}'"

    def test_magenta_ineligible_without_triggers(self) -> None:
        """Magenta requires trigger keywords; none should fail."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card("Test Card", 1, rules_text="Deal 3 damage.")
        assert not is_eligible(card, TransfigType.MAGENTA)

    def test_azure_eligible_for_event(self) -> None:
        """Azure requires Event card type."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card(
            "Test Card",
            1,
            card_type=CardType.EVENT,
            spark=None,
        )
        assert is_eligible(card, TransfigType.AZURE)

    def test_azure_ineligible_for_character(self) -> None:
        """Azure requires Event; Character should fail."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card("Test Card", 1, card_type=CardType.CHARACTER)
        assert not is_eligible(card, TransfigType.AZURE)

    def test_bronze_eligible_for_event(self) -> None:
        """Bronze requires Event card type."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card(
            "Test Card",
            1,
            card_type=CardType.EVENT,
            spark=None,
        )
        assert is_eligible(card, TransfigType.BRONZE)

    def test_bronze_ineligible_for_character(self) -> None:
        """Bronze requires Event; Character should fail."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card("Test Card", 1, card_type=CardType.CHARACTER)
        assert not is_eligible(card, TransfigType.BRONZE)

    def test_rose_eligible_with_energy_cost_pattern(self) -> None:
        """Rose requires an energy cost pattern in rules_text."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card(
            "Test Card",
            1,
            rules_text="Pay 2 energy: Draw a card.",
        )
        assert is_eligible(card, TransfigType.ROSE)

    def test_rose_ineligible_without_cost_pattern(self) -> None:
        """Rose requires energy cost pattern; plain text should fail."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card("Test Card", 1, rules_text="Draw a card.")
        assert not is_eligible(card, TransfigType.ROSE)

    def test_prismatic_eligible_for_multiple_types(self) -> None:
        """Prismatic requires eligibility for 2+ other types."""
        from sites_transfig import is_eligible, TransfigType

        # Character with cost > 0 and digits in text -> Viridian + Golden + Scarlet
        card = _make_card(
            "Test Card",
            1,
            energy_cost=3,
            card_type=CardType.CHARACTER,
            rules_text="Deal 5 damage.",
        )
        assert is_eligible(card, TransfigType.PRISMATIC)

    def test_prismatic_ineligible_for_single_type(self) -> None:
        """Prismatic requires 2+ other types; only 1 should fail."""
        from sites_transfig import is_eligible, TransfigType

        # Event with no special text -> only Azure and Bronze
        # Actually that's 2, let me make one that only qualifies for 1
        # Character with cost 0, no digits, no trigger -> only Scarlet
        card = _make_card(
            "Test Card",
            1,
            energy_cost=0,
            card_type=CardType.CHARACTER,
            rules_text="Draw a card.",
        )
        assert not is_eligible(card, TransfigType.PRISMATIC)


class TestGetApplicableTypes:
    """Tests for finding all applicable transfiguration types."""

    def test_character_with_cost_and_digits(self) -> None:
        """A character with cost > 0 and digits should match multiple types."""
        from sites_transfig import get_applicable_types, TransfigType

        card = _make_card(
            "Test Card",
            1,
            energy_cost=3,
            card_type=CardType.CHARACTER,
            rules_text="Deal 5 damage.",
        )
        types = get_applicable_types(card)
        assert TransfigType.VIRIDIAN in types
        assert TransfigType.GOLDEN in types
        assert TransfigType.SCARLET in types
        # Should also have Prismatic since 3 types qualify
        assert TransfigType.PRISMATIC in types

    def test_event_only_types(self) -> None:
        """An event card should match Azure and Bronze."""
        from sites_transfig import get_applicable_types, TransfigType

        card = _make_card(
            "Test Card",
            1,
            card_type=CardType.EVENT,
            spark=None,
            rules_text="Draw a card.",
        )
        types = get_applicable_types(card)
        assert TransfigType.AZURE in types
        assert TransfigType.BRONZE in types
        # 2 types -> Prismatic
        assert TransfigType.PRISMATIC in types

    def test_no_applicable_types(self) -> None:
        """A card with cost=0, no digits, and Character type with no triggers."""
        from sites_transfig import get_applicable_types

        card = _make_card(
            "Test Card",
            1,
            energy_cost=0,
            card_type=CardType.CHARACTER,
            rules_text="Draw a card.",
        )
        types = get_applicable_types(card)
        # Only Scarlet (Character type), so no Prismatic
        assert len(types) == 1


class TestNormalTransfiguration:
    """Tests for normal (non-enhanced) transfiguration flow."""

    def test_normal_selects_three_cards(self) -> None:
        """Normal mode should offer exactly 3 non-transfigured cards."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        # Add 10 cards
        for i in range(10):
            card = _make_card(
                f"Card {i}",
                i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)

        captured_options: list[list[str]] = []

        def mock_single_select(
            options: list[str],
            **kwargs: object,
        ) -> int:
            captured_options.append(list(options))
            return len(options) - 1  # Select Skip

        with patch(
            "sites_transfig.input_handler.single_select",
            side_effect=mock_single_select,
        ):
            run_transfiguration(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # 3 cards + 1 Skip option
        assert len(captured_options) == 1
        assert len(captured_options[0]) == 4  # 3 cards + Skip

    def test_normal_skips_already_transfigured(self) -> None:
        """Normal mode should not offer already transfigured cards."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        # Add 4 cards, mark 3 as transfigured
        for i in range(4):
            card = _make_card(
                f"Card {i}",
                i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)
            if i < 3:
                state.deck[i].is_transfigured = True

        captured_options: list[list[str]] = []

        def mock_single_select(
            options: list[str],
            **kwargs: object,
        ) -> int:
            captured_options.append(list(options))
            return len(options) - 1  # Skip

        with patch(
            "sites_transfig.input_handler.single_select",
            side_effect=mock_single_select,
        ):
            run_transfiguration(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # Only 1 non-transfigured card + Skip
        assert len(captured_options) == 1
        assert len(captured_options[0]) == 2

    def test_normal_marks_card_transfigured(self) -> None:
        """Selecting a card should mark it as transfigured with a note."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        for i in range(5):
            card = _make_card(
                f"Card {i}",
                i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)

        # Select first offered card (index 0)
        with patch(
            "sites_transfig.input_handler.single_select",
            return_value=0,
        ):
            run_transfiguration(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # Exactly one card should be transfigured
        transfigured = [dc for dc in state.deck if dc.is_transfigured]
        assert len(transfigured) == 1
        assert transfigured[0].transfig_note is not None

    def test_normal_skip_does_not_transfigure(self) -> None:
        """Selecting Skip should not transfigure any card."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        for i in range(5):
            card = _make_card(
                f"Card {i}",
                i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)

        captured_options: list[list[str]] = []

        def mock_single_select(
            options: list[str],
            **kwargs: object,
        ) -> int:
            captured_options.append(list(options))
            return len(options) - 1  # Skip is the last option

        with patch(
            "sites_transfig.input_handler.single_select",
            side_effect=mock_single_select,
        ):
            run_transfiguration(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        transfigured = [dc for dc in state.deck if dc.is_transfigured]
        assert len(transfigured) == 0

    def test_normal_empty_deck(self) -> None:
        """Empty deck should handle gracefully with no crash."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)

        # Should not crash
        run_transfiguration(
            state=state,
            dreamscape_name="Test Dreamscape",
            dreamscape_number=1,
            logger=None,
        )
        assert state.deck_count() == 0

    def test_normal_never_drops_cards_after_sampling(self) -> None:
        """Normal mode should pre-filter eligible cards rather than
        sampling first and then silently dropping ineligible ones.

        When some cards have no applicable transfiguration types, those
        cards should be excluded before sampling, so the player always
        sees up to 3 eligible candidates.
        """
        from sites_transfig import _run_normal, _BASE_TYPES, TransfigType

        state = _make_quest_state(seed=42)
        # Add 10 cards
        for i in range(10):
            card = _make_card(
                f"Card {i}",
                i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)

        eligible_deck_cards = [dc for dc in state.deck if not dc.is_transfigured]

        # Mock is_eligible so that only cards 0-3 are eligible for any type,
        # and cards 4-9 are ineligible for all types. This simulates a
        # situation where some cards have no applicable transfiguration.
        eligible_names = {f"Card {i}" for i in range(4)}
        original_is_eligible = __import__("sites_transfig").is_eligible

        def mock_is_eligible(card: Card, transfig_type: TransfigType) -> bool:
            if card.name not in eligible_names:
                return False
            return original_is_eligible(card, transfig_type)

        captured_options: list[list[str]] = []

        def mock_single_select(
            options: list[str],
            **kwargs: object,
        ) -> int:
            captured_options.append(list(options))
            return len(options) - 1  # Skip

        with patch(
            "sites_transfig.is_eligible",
            side_effect=mock_is_eligible,
        ), patch(
            "sites_transfig.input_handler.single_select",
            side_effect=mock_single_select,
        ):
            _run_normal(state, eligible_deck_cards, None)

        # Should show 3 candidates + Skip = 4 options
        # The old code could show fewer because it sampled from all 10
        # and then dropped the 6 ineligible ones.
        assert len(captured_options) == 1
        assert len(captured_options[0]) == 4, (
            f"Expected 4 options (3 eligible cards + Skip), got "
            f"{len(captured_options[0])}: {captured_options[0]}"
        )

    def test_normal_fewer_than_three_eligible_shows_all(self) -> None:
        """When fewer than 3 cards are eligible, show all eligible cards."""
        from sites_transfig import _run_normal, TransfigType

        state = _make_quest_state(seed=42)
        for i in range(10):
            card = _make_card(
                f"Card {i}",
                i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)

        eligible_deck_cards = [dc for dc in state.deck if not dc.is_transfigured]

        # Only 2 cards are eligible
        eligible_names = {f"Card {0}", f"Card {1}"}
        original_is_eligible = __import__("sites_transfig").is_eligible

        def mock_is_eligible(card: Card, transfig_type: TransfigType) -> bool:
            if card.name not in eligible_names:
                return False
            return original_is_eligible(card, transfig_type)

        captured_options: list[list[str]] = []

        def mock_single_select(
            options: list[str],
            **kwargs: object,
        ) -> int:
            captured_options.append(list(options))
            return len(options) - 1  # Skip

        with patch(
            "sites_transfig.is_eligible",
            side_effect=mock_is_eligible,
        ), patch(
            "sites_transfig.input_handler.single_select",
            side_effect=mock_single_select,
        ):
            _run_normal(state, eligible_deck_cards, None)

        # 2 eligible cards + Skip = 3 options
        assert len(captured_options) == 1
        assert len(captured_options[0]) == 3


class TestEnhancedTransfiguration:
    """Tests for enhanced (Prismatic biome) transfiguration flow."""

    def test_enhanced_shows_full_deck(self) -> None:
        """Enhanced mode should show all non-transfigured cards."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        for i in range(8):
            card = _make_card(
                f"Card {i}",
                i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)
        # Mark 2 as transfigured
        state.deck[0].is_transfigured = True
        state.deck[1].is_transfigured = True

        captured_options: list[list[str]] = []

        def mock_single_select(
            options: list[str],
            **kwargs: object,
        ) -> int:
            captured_options.append(list(options))
            return 0  # Select first

        with patch(
            "sites_transfig.input_handler.single_select",
            side_effect=mock_single_select,
        ):
            run_transfiguration(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                is_enhanced=True,
            )

        # 6 non-transfigured cards + Skip
        assert len(captured_options) == 1
        assert len(captured_options[0]) == 7

    def test_enhanced_applies_best_type(self) -> None:
        """Enhanced mode should apply Prismatic if 2+ types apply."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        # Card that qualifies for Viridian + Golden + Scarlet -> Prismatic
        card = _make_card(
            "Multi Card",
            1,
            energy_cost=3,
            card_type=CardType.CHARACTER,
            rules_text="Deal 5 damage.",
        )
        state.add_card(card)

        with patch(
            "sites_transfig.input_handler.single_select",
            return_value=0,
        ):
            run_transfiguration(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                is_enhanced=True,
            )

        dc = state.deck[0]
        assert dc.is_transfigured
        assert dc.transfig_note is not None
        assert "Prismatic" in dc.transfig_note
        assert "all applicable upgrades" in dc.transfig_note

    def test_enhanced_applies_single_type_when_only_one(self) -> None:
        """Enhanced mode should apply the single type if only one qualifies."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        # Character with cost=0, no digits, no triggers -> only Scarlet
        card = _make_card(
            "Simple Card",
            1,
            energy_cost=0,
            card_type=CardType.CHARACTER,
            rules_text="Draw a card.",
        )
        state.add_card(card)

        with patch(
            "sites_transfig.input_handler.single_select",
            return_value=0,
        ):
            run_transfiguration(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                is_enhanced=True,
            )

        dc = state.deck[0]
        assert dc.is_transfigured
        assert dc.transfig_note is not None
        assert "Scarlet" in dc.transfig_note
        assert "doubled spark" in dc.transfig_note


class TestTransfigNote:
    """Tests for the transfiguration note format."""

    def test_note_format_includes_type_name_and_card_name(self) -> None:
        """Transfig note should be 'TypeName CardName -- note text'."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        card = _make_card(
            "Whirlpool Seer",
            1,
            energy_cost=3,
            card_type=CardType.CHARACTER,
            rules_text="Draw a card.",
        )
        state.add_card(card)

        with patch(
            "sites_transfig.input_handler.single_select",
            return_value=0,
        ):
            run_transfiguration(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                is_enhanced=True,
            )

        dc = state.deck[0]
        assert dc.is_transfigured
        assert dc.transfig_note is not None
        # Should contain the card name and have the format "Type Name -- note"
        assert "Whirlpool Seer" in dc.transfig_note
        assert " -- " in dc.transfig_note


class TestEligibilityExplanation:
    """Tests for eligibility explanation strings."""

    def test_viridian_explains_cost(self) -> None:
        """Viridian explanation should mention energy cost value."""
        from sites_transfig import eligibility_explanation, TransfigType

        card = _make_card("Test Card", 1, energy_cost=3)
        explanation = eligibility_explanation(card, TransfigType.VIRIDIAN)
        assert "3" in explanation
        assert "cost" in explanation.lower()

    def test_golden_explains_numbers_in_text(self) -> None:
        """Golden explanation should mention rules text numbers."""
        from sites_transfig import eligibility_explanation, TransfigType

        card = _make_card("Test Card", 1, rules_text="Deal 5 damage.")
        explanation = eligibility_explanation(card, TransfigType.GOLDEN)
        assert "number" in explanation.lower() or "digit" in explanation.lower()

    def test_scarlet_explains_character_type(self) -> None:
        """Scarlet explanation should mention character type."""
        from sites_transfig import eligibility_explanation, TransfigType

        card = _make_card("Test Card", 1, card_type=CardType.CHARACTER)
        explanation = eligibility_explanation(card, TransfigType.SCARLET)
        assert "character" in explanation.lower()

    def test_magenta_explains_trigger(self) -> None:
        """Magenta explanation should mention the trigger keyword found."""
        from sites_transfig import eligibility_explanation, TransfigType

        card = _make_card(
            "Test Card",
            1,
            rules_text="Judgment: deal damage.",
        )
        explanation = eligibility_explanation(card, TransfigType.MAGENTA)
        assert "trigger" in explanation.lower() or "judgment" in explanation.lower()

    def test_azure_explains_event_type(self) -> None:
        """Azure explanation should mention event type."""
        from sites_transfig import eligibility_explanation, TransfigType

        card = _make_card(
            "Test Card",
            1,
            card_type=CardType.EVENT,
            spark=None,
        )
        explanation = eligibility_explanation(card, TransfigType.AZURE)
        assert "event" in explanation.lower()

    def test_bronze_explains_event_type(self) -> None:
        """Bronze explanation should mention event type."""
        from sites_transfig import eligibility_explanation, TransfigType

        card = _make_card(
            "Test Card",
            1,
            card_type=CardType.EVENT,
            spark=None,
        )
        explanation = eligibility_explanation(card, TransfigType.BRONZE)
        assert "event" in explanation.lower()

    def test_rose_explains_energy_ability(self) -> None:
        """Rose explanation should mention activated ability."""
        from sites_transfig import eligibility_explanation, TransfigType

        card = _make_card(
            "Test Card",
            1,
            rules_text="Pay 2 energy: Draw a card.",
        )
        explanation = eligibility_explanation(card, TransfigType.ROSE)
        assert "activated" in explanation.lower() or "ability" in explanation.lower()

    def test_prismatic_lists_applicable_types(self) -> None:
        """Prismatic explanation should list all applicable sub-types."""
        from sites_transfig import eligibility_explanation, TransfigType

        card = _make_card(
            "Test Card",
            1,
            energy_cost=3,
            card_type=CardType.CHARACTER,
            rules_text="Deal 5 damage.",
        )
        explanation = eligibility_explanation(card, TransfigType.PRISMATIC)
        assert "Viridian" in explanation
        assert "Golden" in explanation
        assert "Scarlet" in explanation


class TestTransfigTypeColor:
    """Tests for transfiguration type color mapping."""

    def test_viridian_returns_green(self) -> None:
        """Viridian should have green color code."""
        from sites_transfig import transfig_type_color, TransfigType

        color = transfig_type_color(TransfigType.VIRIDIAN)
        # Green ANSI code contains 32 or 92
        assert "32" in color or "92" in color or color == ""

    def test_golden_returns_yellow(self) -> None:
        """Golden should have yellow color code."""
        from sites_transfig import transfig_type_color, TransfigType

        color = transfig_type_color(TransfigType.GOLDEN)
        assert "33" in color or "93" in color or color == ""

    def test_scarlet_returns_red(self) -> None:
        """Scarlet should have red color code."""
        from sites_transfig import transfig_type_color, TransfigType

        color = transfig_type_color(TransfigType.SCARLET)
        assert "31" in color or "91" in color or color == ""

    def test_all_types_have_colors(self) -> None:
        """Every TransfigType should have a color entry."""
        from sites_transfig import transfig_type_color, TransfigType

        for t in TransfigType:
            color = transfig_type_color(t)
            assert isinstance(color, str)


class TestRenderTransfigPreview:
    """Tests for the polished transfiguration card preview rendering."""

    def test_preview_shows_transformed_name(self) -> None:
        """Card preview should show 'Name -> TransfigType Name'."""
        from sites_transfig import _render_transfig_item, TransfigType

        card = _make_card(
            "Whirlpool Seer",
            1,
            energy_cost=3,
            spark=2,
            rarity=Rarity.UNCOMMON,
            resonances=frozenset({Resonance.TIDE}),
            rules_text="Judgment: Foresee 2.",
        )
        dc = DeckCard(card=card)
        candidates = [(dc, TransfigType.VIRIDIAN)]

        rendered = _render_transfig_item(0, "Test", True, candidates)
        # Should include the transformed name somewhere
        assert "Viridian" in rendered
        assert "Whirlpool Seer" in rendered

    def test_preview_shows_eligibility_reason(self) -> None:
        """Card preview should include an eligibility explanation."""
        from sites_transfig import _render_transfig_item, TransfigType

        card = _make_card(
            "Whirlpool Seer",
            1,
            energy_cost=3,
            spark=2,
            rules_text="Judgment: Foresee 2.",
        )
        dc = DeckCard(card=card)
        candidates = [(dc, TransfigType.VIRIDIAN)]

        rendered = _render_transfig_item(0, "Test", True, candidates)
        # Should include "Eligible" and cost info
        assert "Eligible" in rendered or "eligible" in rendered

    def test_skip_option_labeled(self) -> None:
        """Skip option should be clearly visible."""
        from sites_transfig import _render_transfig_item, TransfigType

        card = _make_card("Test", 1)
        dc = DeckCard(card=card)
        candidates = [(dc, TransfigType.VIRIDIAN)]

        rendered = _render_transfig_item(
            1,
            "Skip transfiguration",
            True,
            candidates,
        )
        assert "Skip" in rendered


class TestTransfigurationLogging:
    """Tests for logging in transfiguration interactions."""

    def test_logs_site_visit(self) -> None:
        """Transfiguration should log the interaction."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        for i in range(5):
            card = _make_card(
                f"Card {i}",
                i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(dict(kwargs))

        with patch(
            "sites_transfig.input_handler.single_select",
            return_value=0,
        ):
            run_transfiguration(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "Transfiguration"

    def test_logs_skip_as_none_choice(self) -> None:
        """When player skips, choice_made should be None."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        for i in range(5):
            card = _make_card(
                f"Card {i}",
                i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(dict(kwargs))

        captured_options: list[list[str]] = []

        def mock_single_select(
            options: list[str],
            **kwargs: object,
        ) -> int:
            captured_options.append(list(options))
            return len(options) - 1  # Skip

        with patch(
            "sites_transfig.input_handler.single_select",
            side_effect=mock_single_select,
        ):
            run_transfiguration(
                state=state,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 1
        assert log_calls[0]["choice_made"] is None
