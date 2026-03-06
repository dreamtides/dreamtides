"""Tests for sites_transfig module."""

import random
import sys
from pathlib import Path
from typing import Optional
from unittest.mock import patch

# Ensure draft_simulator is importable
_DRAFT_SIM_DIR = str(Path(__file__).resolve().parent.parent / "draft_simulator")
if _DRAFT_SIM_DIR not in sys.path:
    sys.path.insert(0, _DRAFT_SIM_DIR)

from draft_models import CardDesign, CardInstance
from models import DeckCard
from quest_state import QuestState

_NEXT_INSTANCE_ID = 0


def _make_design(
    name: str = "Test Card",
    card_id: str = "test_001",
    power: float = 0.0,
    commit: float = 0.0,
    flex: float = 0.0,
    fitness: Optional[list[float]] = None,
) -> CardDesign:
    return CardDesign(
        card_id=card_id,
        name=name,
        fitness=fitness if fitness is not None else [],
        power=power,
        commit=commit,
        flex=flex,
    )


def _make_instance(design: CardDesign) -> CardInstance:
    global _NEXT_INSTANCE_ID
    _NEXT_INSTANCE_ID += 1
    return CardInstance(instance_id=_NEXT_INSTANCE_ID, design=design)


def _make_quest_state(
    seed: int = 42,
    essence: int = 250,
) -> QuestState:
    rng = random.Random(seed)

    import agents

    human_agent = agents.create_agent(archetype_count=8)
    ai_agents = [agents.create_agent(archetype_count=8) for _ in range(5)]

    import card_generator
    import cube_manager
    from config import SimulatorConfig
    from draft_models import CubeConsumptionMode

    cfg = SimulatorConfig()
    cfg.draft.seat_count = 6
    cfg.draft.pack_size = 20
    cfg.cards.archetype_count = 8
    cfg.cards.source = "synthetic"
    cfg.cube.distinct_cards = 10
    cfg.cube.copies_per_card = 1
    cfg.cube.consumption_mode = "with_replacement"
    cfg.refill.strategy = "no_refill"
    cfg.pack_generation.strategy = "seeded_themed"

    cards = card_generator.generate_cards(cfg, rng)
    cube = cube_manager.CubeManager(
        designs=cards,
        copies_per_card=1,
        consumption_mode=CubeConsumptionMode.WITH_REPLACEMENT,
    )

    return QuestState(
        essence=essence,
        rng=rng,
        human_agent=human_agent,
        ai_agents=ai_agents,
        cube=cube,
        draft_cfg=cfg,
    )


class TestTransfigurationEligibility:
    """Tests for transfiguration type eligibility checking."""

    def test_viridian_eligible_when_power_high(self) -> None:
        """Viridian requires power > 0.3."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(power=0.5)
        assert is_eligible(design, TransfigType.VIRIDIAN)

    def test_viridian_ineligible_when_power_low(self) -> None:
        """Viridian requires power > 0.3; low power should fail."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(power=0.1)
        assert not is_eligible(design, TransfigType.VIRIDIAN)

    def test_viridian_ineligible_when_power_zero(self) -> None:
        """Viridian requires power > 0.3; zero power should fail."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(power=0.0)
        assert not is_eligible(design, TransfigType.VIRIDIAN)

    def test_golden_eligible_when_flex_high(self) -> None:
        """Golden requires flex > 0.3."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(flex=0.5)
        assert is_eligible(design, TransfigType.GOLDEN)

    def test_golden_ineligible_when_flex_low(self) -> None:
        """Golden requires flex > 0.3; low flex should fail."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(flex=0.1)
        assert not is_eligible(design, TransfigType.GOLDEN)

    def test_scarlet_eligible_when_commit_high(self) -> None:
        """Scarlet requires commit > 0.5."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(commit=0.7)
        assert is_eligible(design, TransfigType.SCARLET)

    def test_scarlet_ineligible_when_commit_low(self) -> None:
        """Scarlet requires commit > 0.5; low commit should fail."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(commit=0.3)
        assert not is_eligible(design, TransfigType.SCARLET)

    def test_magenta_eligible_with_high_fitness(self) -> None:
        """Magenta requires top fitness > 0.7."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(fitness=[0.8, 0.2, 0.1])
        assert is_eligible(design, TransfigType.MAGENTA)

    def test_magenta_ineligible_with_low_fitness(self) -> None:
        """Magenta requires top fitness > 0.7; all low should fail."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(fitness=[0.5, 0.3, 0.2])
        assert not is_eligible(design, TransfigType.MAGENTA)

    def test_magenta_ineligible_with_empty_fitness(self) -> None:
        """Magenta requires fitness; empty should fail."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(fitness=[])
        assert not is_eligible(design, TransfigType.MAGENTA)

    def test_azure_eligible_when_power_very_high(self) -> None:
        """Azure requires power > 0.5."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(power=0.7)
        assert is_eligible(design, TransfigType.AZURE)

    def test_azure_ineligible_when_power_moderate(self) -> None:
        """Azure requires power > 0.5; moderate power should fail."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(power=0.4)
        assert not is_eligible(design, TransfigType.AZURE)

    def test_bronze_eligible_when_flex_very_high(self) -> None:
        """Bronze requires flex > 0.5."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(flex=0.7)
        assert is_eligible(design, TransfigType.BRONZE)

    def test_bronze_ineligible_when_flex_moderate(self) -> None:
        """Bronze requires flex > 0.5; moderate flex should fail."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(flex=0.4)
        assert not is_eligible(design, TransfigType.BRONZE)

    def test_rose_eligible_when_commit_and_flex_both_above_threshold(self) -> None:
        """Rose requires commit > 0.3 and flex > 0.3."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(commit=0.5, flex=0.5)
        assert is_eligible(design, TransfigType.ROSE)

    def test_rose_ineligible_when_only_commit_high(self) -> None:
        """Rose requires both commit > 0.3 and flex > 0.3."""
        from sites_transfig import is_eligible, TransfigType

        design = _make_design(commit=0.5, flex=0.1)
        assert not is_eligible(design, TransfigType.ROSE)

    def test_prismatic_eligible_for_multiple_types(self) -> None:
        """Prismatic requires eligibility for 2+ other types."""
        from sites_transfig import is_eligible, TransfigType

        # High power -> Viridian + Azure, high flex -> Golden + Bronze
        design = _make_design(power=0.8, flex=0.8)
        assert is_eligible(design, TransfigType.PRISMATIC)

    def test_prismatic_ineligible_for_single_type(self) -> None:
        """Prismatic requires 2+ other types; only 1 should fail."""
        from sites_transfig import is_eligible, TransfigType

        # Only power > 0.3 (Viridian), nothing else
        design = _make_design(power=0.4, commit=0.0, flex=0.0)
        assert not is_eligible(design, TransfigType.PRISMATIC)


class TestGetApplicableTypes:
    """Tests for finding all applicable transfiguration types."""

    def test_high_power_and_flex_card(self) -> None:
        """A card with high power and flex should match multiple types."""
        from sites_transfig import get_applicable_types, TransfigType

        design = _make_design(power=0.8, flex=0.8)
        types = get_applicable_types(design)
        assert TransfigType.VIRIDIAN in types
        assert TransfigType.GOLDEN in types
        assert TransfigType.AZURE in types
        assert TransfigType.BRONZE in types
        assert TransfigType.PRISMATIC in types

    def test_single_qualifying_type(self) -> None:
        """A card with only moderate power should match only Viridian."""
        from sites_transfig import get_applicable_types, TransfigType

        design = _make_design(power=0.4, commit=0.0, flex=0.0)
        types = get_applicable_types(design)
        assert TransfigType.VIRIDIAN in types
        assert len(types) == 1  # No Prismatic since < 2 types

    def test_no_applicable_types(self) -> None:
        """A card with all low stats should match nothing."""
        from sites_transfig import get_applicable_types

        design = _make_design(power=0.0, commit=0.0, flex=0.0, fitness=[])
        types = get_applicable_types(design)
        assert len(types) == 0


class TestNormalTransfiguration:
    """Tests for normal (non-enhanced) transfiguration flow."""

    def test_normal_selects_three_cards(self) -> None:
        """Normal mode should offer exactly 3 non-transfigured cards."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        for i in range(10):
            design = _make_design(
                name=f"Card {i}",
                card_id=f"card_{i}",
                power=0.5 + i * 0.01,
                flex=0.5,
            )
            state.add_card(_make_instance(design))

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
        for i in range(4):
            design = _make_design(
                name=f"Card {i}",
                card_id=f"card_{i}",
                power=0.5,
                flex=0.5,
            )
            state.add_card(_make_instance(design))
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
            design = _make_design(
                name=f"Card {i}",
                card_id=f"card_{i}",
                power=0.5,
                flex=0.5,
            )
            state.add_card(_make_instance(design))

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
            design = _make_design(
                name=f"Card {i}",
                card_id=f"card_{i}",
                power=0.5,
                flex=0.5,
            )
            state.add_card(_make_instance(design))

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
        """
        from sites_transfig import _run_normal, TransfigType

        state = _make_quest_state(seed=42)
        for i in range(10):
            design = _make_design(
                name=f"Card {i}",
                card_id=f"card_{i}",
                power=0.5 if i < 4 else 0.0,
                commit=0.0,
                flex=0.5 if i < 4 else 0.0,
            )
            state.add_card(_make_instance(design))

        eligible_deck_cards = [dc for dc in state.deck if not dc.is_transfigured]

        # Mock is_eligible so that only cards 0-3 are eligible for any type
        eligible_names = {f"Card {i}" for i in range(4)}
        original_is_eligible = __import__("sites_transfig").is_eligible

        def mock_is_eligible(design_obj, transfig_type: TransfigType) -> bool:
            name = getattr(design_obj, "name", "")
            if name not in eligible_names:
                return False
            return original_is_eligible(design_obj, transfig_type)

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
            design = _make_design(
                name=f"Card {i}",
                card_id=f"card_{i}",
                power=0.5 if i < 2 else 0.0,
                commit=0.0,
                flex=0.5 if i < 2 else 0.0,
            )
            state.add_card(_make_instance(design))

        eligible_deck_cards = [dc for dc in state.deck if not dc.is_transfigured]

        # Only 2 cards are eligible
        eligible_names = {"Card 0", "Card 1"}
        original_is_eligible = __import__("sites_transfig").is_eligible

        def mock_is_eligible(design_obj, transfig_type: TransfigType) -> bool:
            name = getattr(design_obj, "name", "")
            if name not in eligible_names:
                return False
            return original_is_eligible(design_obj, transfig_type)

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
            design = _make_design(
                name=f"Card {i}",
                card_id=f"card_{i}",
                power=0.5,
                flex=0.5,
            )
            state.add_card(_make_instance(design))
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
        # Card with high power + high flex -> Viridian + Golden + Azure + Bronze -> Prismatic
        design = _make_design(
            name="Multi Card",
            card_id="multi_001",
            power=0.8,
            flex=0.8,
        )
        state.add_card(_make_instance(design))

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
        # Only power > 0.3 (Viridian), nothing else
        design = _make_design(
            name="Simple Card",
            card_id="simple_001",
            power=0.4,
            commit=0.0,
            flex=0.0,
        )
        state.add_card(_make_instance(design))

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
        assert "Viridian" in dc.transfig_note
        assert "halved cost" in dc.transfig_note


class TestTransfigNote:
    """Tests for the transfiguration note format."""

    def test_note_format_includes_type_name_and_card_name(self) -> None:
        """Transfig note should be 'TypeName CardName -- note text'."""
        from sites_transfig import run_transfiguration

        state = _make_quest_state(seed=42)
        design = _make_design(
            name="Whirlpool Seer",
            card_id="whirlpool_001",
            power=0.5,
            flex=0.5,
        )
        state.add_card(_make_instance(design))

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

    def test_viridian_explains_power(self) -> None:
        """Viridian explanation should mention power value."""
        from sites_transfig import eligibility_explanation, TransfigType

        design = _make_design(power=0.5)
        explanation = eligibility_explanation(design, TransfigType.VIRIDIAN)
        assert "0.50" in explanation
        assert "power" in explanation.lower()

    def test_golden_explains_flex(self) -> None:
        """Golden explanation should mention flex value."""
        from sites_transfig import eligibility_explanation, TransfigType

        design = _make_design(flex=0.5)
        explanation = eligibility_explanation(design, TransfigType.GOLDEN)
        assert "0.50" in explanation
        assert "flex" in explanation.lower()

    def test_scarlet_explains_commit(self) -> None:
        """Scarlet explanation should mention commit value."""
        from sites_transfig import eligibility_explanation, TransfigType

        design = _make_design(commit=0.7)
        explanation = eligibility_explanation(design, TransfigType.SCARLET)
        assert "0.70" in explanation
        assert "commit" in explanation.lower()

    def test_magenta_explains_fitness(self) -> None:
        """Magenta explanation should mention top fitness value."""
        from sites_transfig import eligibility_explanation, TransfigType

        design = _make_design(fitness=[0.9, 0.2, 0.1])
        explanation = eligibility_explanation(design, TransfigType.MAGENTA)
        assert "0.90" in explanation
        assert "fitness" in explanation.lower()

    def test_azure_explains_power(self) -> None:
        """Azure explanation should mention power value."""
        from sites_transfig import eligibility_explanation, TransfigType

        design = _make_design(power=0.7)
        explanation = eligibility_explanation(design, TransfigType.AZURE)
        assert "0.70" in explanation
        assert "power" in explanation.lower()

    def test_bronze_explains_flex(self) -> None:
        """Bronze explanation should mention flex value."""
        from sites_transfig import eligibility_explanation, TransfigType

        design = _make_design(flex=0.7)
        explanation = eligibility_explanation(design, TransfigType.BRONZE)
        assert "0.70" in explanation
        assert "flex" in explanation.lower()

    def test_rose_explains_commit_and_flex(self) -> None:
        """Rose explanation should mention both commit and flex."""
        from sites_transfig import eligibility_explanation, TransfigType

        design = _make_design(commit=0.5, flex=0.5)
        explanation = eligibility_explanation(design, TransfigType.ROSE)
        assert "commit" in explanation.lower()
        assert "flex" in explanation.lower()

    def test_prismatic_lists_applicable_types(self) -> None:
        """Prismatic explanation should list all applicable sub-types."""
        from sites_transfig import eligibility_explanation, TransfigType

        design = _make_design(power=0.8, flex=0.8)
        explanation = eligibility_explanation(design, TransfigType.PRISMATIC)
        assert "Viridian" in explanation
        assert "Golden" in explanation
        assert "Azure" in explanation
        assert "Bronze" in explanation


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
        """Card preview should show 'TransfigType Name'."""
        from sites_transfig import _render_transfig_item, TransfigType

        design = _make_design(
            name="Whirlpool Seer",
            card_id="whirlpool_001",
            power=0.5,
            commit=0.3,
            flex=0.4,
        )
        dc = DeckCard(instance=_make_instance(design))
        candidates = [(dc, TransfigType.VIRIDIAN)]

        rendered = _render_transfig_item(0, "Test", True, candidates)
        assert "Viridian" in rendered
        assert "Whirlpool Seer" in rendered

    def test_preview_shows_eligibility_reason(self) -> None:
        """Card preview should include an eligibility explanation."""
        from sites_transfig import _render_transfig_item, TransfigType

        design = _make_design(
            name="Whirlpool Seer",
            card_id="whirlpool_001",
            power=0.5,
        )
        dc = DeckCard(instance=_make_instance(design))
        candidates = [(dc, TransfigType.VIRIDIAN)]

        rendered = _render_transfig_item(0, "Test", True, candidates)
        assert "Eligible" in rendered or "eligible" in rendered

    def test_skip_option_labeled(self) -> None:
        """Skip option should be clearly visible."""
        from sites_transfig import _render_transfig_item, TransfigType

        design = _make_design(name="Test", card_id="test_001", power=0.5)
        dc = DeckCard(instance=_make_instance(design))
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
            design = _make_design(
                name=f"Card {i}",
                card_id=f"card_{i}",
                power=0.5,
                flex=0.5,
            )
            state.add_card(_make_instance(design))

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
            design = _make_design(
                name=f"Card {i}",
                card_id=f"card_{i}",
                power=0.5,
                flex=0.5,
            )
            state.add_card(_make_instance(design))

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
