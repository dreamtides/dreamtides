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
                "Test Card", 1,
                rules_text=f"Some text {keyword} something happens.",
            )
            assert is_eligible(card, TransfigType.MAGENTA), (
                f"Expected Magenta eligible for keyword '{keyword}'"
            )

    def test_magenta_ineligible_without_triggers(self) -> None:
        """Magenta requires trigger keywords; none should fail."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card("Test Card", 1, rules_text="Deal 3 damage.")
        assert not is_eligible(card, TransfigType.MAGENTA)

    def test_azure_eligible_for_event(self) -> None:
        """Azure requires Event card type."""
        from sites_transfig import is_eligible, TransfigType

        card = _make_card(
            "Test Card", 1, card_type=CardType.EVENT, spark=None,
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
            "Test Card", 1, card_type=CardType.EVENT, spark=None,
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
            "Test Card", 1,
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
            "Test Card", 1,
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
            "Test Card", 1,
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
            "Test Card", 1,
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
            "Test Card", 1,
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
            "Test Card", 1,
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
                f"Card {i}", i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)

        captured_options: list[list[str]] = []

        def mock_single_select(
            options: list[str], **kwargs: object,
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
                f"Card {i}", i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)
            if i < 3:
                state.deck[i].is_transfigured = True

        captured_options: list[list[str]] = []

        def mock_single_select(
            options: list[str], **kwargs: object,
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
                f"Card {i}", i,
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
                f"Card {i}", i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)

        captured_options: list[list[str]] = []

        def mock_single_select(
            options: list[str], **kwargs: object,
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


class TestEnhancedTransfiguration:
    """Tests for enhanced (Prismatic biome) transfiguration flow."""

    def test_enhanced_shows_full_deck(self) -> None:
        """Enhanced mode should show all non-transfigured cards."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        for i in range(8):
            card = _make_card(
                f"Card {i}", i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)
        # Mark 2 as transfigured
        state.deck[0].is_transfigured = True
        state.deck[1].is_transfigured = True

        captured_options: list[list[str]] = []

        def mock_single_select(
            options: list[str], **kwargs: object,
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
            "Multi Card", 1,
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
            "Simple Card", 1,
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
            "Whirlpool Seer", 1,
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


class TestTransfigurationLogging:
    """Tests for logging in transfiguration interactions."""

    def test_logs_site_visit(self) -> None:
        """Transfiguration should log the interaction."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        for i in range(5):
            card = _make_card(
                f"Card {i}", i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(
                self,
                site_type: str,
                choices: list[str],
                choice_made: object,
                state_changes: dict[str, object],
            ) -> None:
                log_calls.append({
                    "site_type": site_type,
                    "choices": list(choices),
                    "choice_made": choice_made,
                    "state_changes": dict(state_changes),
                })

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
                f"Card {i}", i,
                energy_cost=i + 1,
                rules_text=f"Deal {i + 1} damage.",
            )
            state.add_card(card)

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(
                self,
                site_type: str,
                choices: list[str],
                choice_made: object,
                state_changes: dict[str, object],
            ) -> None:
                log_calls.append({
                    "site_type": site_type,
                    "choices": list(choices),
                    "choice_made": choice_made,
                    "state_changes": dict(state_changes),
                })

        captured_options: list[list[str]] = []

        def mock_single_select(
            options: list[str], **kwargs: object,
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
