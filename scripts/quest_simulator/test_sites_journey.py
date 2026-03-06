"""Tests for Dream Journey and Tempting Offer site interactions."""

import random
import sys
from pathlib import Path
from typing import Optional
from unittest.mock import patch

_DRAFT_SIM_DIR = str(Path(__file__).resolve().parent.parent / "draft_simulator")
if _DRAFT_SIM_DIR not in sys.path:
    sys.path.insert(0, _DRAFT_SIM_DIR)

import agents
import card_generator
import cube_manager
from config import SimulatorConfig
from draft_models import CardDesign, CardInstance, CubeConsumptionMode

from models import (
    BaneCard,
    DeckCard,
    Dreamsign,
    EffectType,
    Journey,
    TemptingOffer,
)
from quest_state import QuestState


def _build_cfg() -> SimulatorConfig:
    cfg = SimulatorConfig()
    cfg.draft.seat_count = 6
    cfg.draft.pack_size = 20
    cfg.draft.human_seats = 1
    cfg.draft.alternate_direction = False
    cfg.agents.show_n = 4
    cfg.agents.show_n_strategy = "sharpened_preference"
    cfg.agents.policy = "adaptive"
    cfg.agents.ai_optimality = 0.80
    cfg.agents.learning_rate = 3.0
    cfg.agents.openness_window = 3
    cfg.cards.archetype_count = 8
    cfg.cards.source = "synthetic"
    cfg.cube.distinct_cards = 540
    cfg.cube.copies_per_card = 1
    cfg.cube.consumption_mode = "with_replacement"
    cfg.refill.strategy = "no_refill"
    cfg.pack_generation.strategy = "seeded_themed"
    return cfg


def _make_quest_state(
    essence: int = 250,
    seed: int = 42,
) -> QuestState:
    rng = random.Random(seed)
    cfg = _build_cfg()
    cards = card_generator.generate_cards(cfg, rng)
    cube = cube_manager.CubeManager(cards, 1, CubeConsumptionMode.WITH_REPLACEMENT)
    return QuestState(
        essence=essence,
        rng=rng,
        human_agent=agents.create_agent(8),
        ai_agents=[agents.create_agent(8) for _ in range(5)],
        cube=cube,
        draft_cfg=cfg,
    )


def _make_card_instance(
    name: str = "Test Card",
    instance_id: int = 1,
) -> CardInstance:
    design = CardDesign(
        card_id=name,
        name=name,
        fitness=[0.5] * 8,
        power=0.5,
        commit=0.5,
        flex=0.5,
    )
    return CardInstance(instance_id=instance_id, design=design)


def _make_journeys() -> list[Journey]:
    return [
        Journey(
            name="Test Journey A",
            description="Adds cards",
            effect_type=EffectType.ADD_CARDS,
            effect_value=2,
        ),
        Journey(
            name="Test Journey B",
            description="Adds essence",
            effect_type=EffectType.ADD_ESSENCE,
            effect_value=100,
        ),
        Journey(
            name="Test Journey C",
            description="Removes cards",
            effect_type=EffectType.REMOVE_CARDS,
            effect_value=1,
        ),
        Journey(
            name="Test Journey D",
            description="Adds dreamsign",
            effect_type=EffectType.ADD_DREAMSIGN,
            effect_value=1,
        ),
    ]


def _make_dreamsigns() -> list[Dreamsign]:
    return [
        Dreamsign(
            name="Test Sign A",
            effect_text="Test effect A",
            is_bane=False,
        ),
        Dreamsign(
            name="Test Sign B",
            effect_text="Test effect B",
            is_bane=False,
        ),
        Dreamsign(
            name="Bane Sign",
            effect_text="Bane effect",
            is_bane=True,
        ),
    ]


def _make_bane_cards() -> list[BaneCard]:
    return [
        BaneCard(
            name="Creeping Dread",
            rules_text="Cannot be played.",
            card_type="Event",
            energy_cost=99,
        ),
        BaneCard(
            name="Nightmare Residue",
            rules_text="Lose 1 energy when drawn.",
            card_type="Event",
            energy_cost=8,
        ),
    ]


def _make_offers() -> list[TemptingOffer]:
    return [
        TemptingOffer(
            reward_name="Cards Reward",
            reward_description="Gain cards",
            reward_effect_type=EffectType.ADD_CARDS,
            reward_value=2,
            cost_name="Lose Essence",
            cost_description="Pay essence",
            cost_effect_type=EffectType.LOSE_ESSENCE,
            cost_value=50,
        ),
        TemptingOffer(
            reward_name="Essence Reward",
            reward_description="Gain essence",
            reward_effect_type=EffectType.ADD_ESSENCE,
            reward_value=150,
            cost_name="Bane Card",
            cost_description="Gain a bane",
            cost_effect_type=EffectType.ADD_BANE_CARD,
            cost_value=1,
        ),
        TemptingOffer(
            reward_name="Large Essence Reward",
            reward_description="Gain large essence",
            reward_effect_type=EffectType.LARGE_ESSENCE,
            reward_value=300,
            cost_name="Bane Dreamsign",
            cost_description="Gain a bane sign",
            cost_effect_type=EffectType.ADD_BANE_DREAMSIGN,
            cost_value=1,
        ),
        TemptingOffer(
            reward_name="Dreamsign Reward",
            reward_description="Gain a dreamsign",
            reward_effect_type=EffectType.ADD_DREAMSIGN,
            reward_value=1,
            cost_name="Remove Cards",
            cost_description="Lose cards",
            cost_effect_type=EffectType.REMOVE_CARDS,
            cost_value=2,
        ),
    ]


# ===========================================================================
# Dream Journey tests
# ===========================================================================


class TestSelectJourneys:
    """Test journey selection picks the correct number of options."""

    def test_selects_two_by_default(self) -> None:
        from sites_journey import select_journeys

        journeys = _make_journeys()
        rng = random.Random(42)
        result = select_journeys(journeys, rng, is_enhanced=False)
        assert len(result) == 2

    def test_selects_three_when_enhanced(self) -> None:
        from sites_journey import select_journeys

        journeys = _make_journeys()
        rng = random.Random(42)
        result = select_journeys(journeys, rng, is_enhanced=True)
        assert len(result) == 3

    def test_all_selected_from_pool(self) -> None:
        from sites_journey import select_journeys

        journeys = _make_journeys()
        rng = random.Random(42)
        result = select_journeys(journeys, rng, is_enhanced=False)
        for j in result:
            assert j in journeys

    def test_deterministic_with_same_seed(self) -> None:
        from sites_journey import select_journeys

        journeys = _make_journeys()
        rng1 = random.Random(99)
        rng2 = random.Random(99)
        r1 = select_journeys(journeys, rng1, is_enhanced=False)
        r2 = select_journeys(journeys, rng2, is_enhanced=False)
        assert [j.name for j in r1] == [j.name for j in r2]

    def test_handles_fewer_than_requested(self) -> None:
        from sites_journey import select_journeys

        journeys = _make_journeys()[:1]
        rng = random.Random(42)
        result = select_journeys(journeys, rng, is_enhanced=False)
        assert len(result) == 1


class TestApplyJourneyEffect:
    """Test that each journey effect type mutates quest state correctly."""

    def test_add_cards_effect(self) -> None:
        from sites_journey import apply_journey_effect

        state = _make_quest_state()
        journey = Journey(
            name="Card Journey",
            description="Add cards",
            effect_type=EffectType.ADD_CARDS,
            effect_value=2,
        )
        initial_deck = state.deck_count()

        apply_journey_effect(state, journey, _make_dreamsigns())

        assert state.deck_count() == initial_deck + 2

    def test_add_cards_returns_card_names(self) -> None:
        from sites_journey import apply_journey_effect

        state = _make_quest_state()
        journey = Journey(
            name="Card Journey",
            description="Add cards",
            effect_type=EffectType.ADD_CARDS,
            effect_value=1,
        )

        changes = apply_journey_effect(state, journey, _make_dreamsigns())

        assert "cards_added" in changes
        assert len(changes["cards_added"]) == 1
        assert isinstance(changes["cards_added"][0], str)

    def test_add_essence_effect(self) -> None:
        from sites_journey import apply_journey_effect

        state = _make_quest_state(essence=100)
        journey = Journey(
            name="Essence Journey",
            description="Add essence",
            effect_type=EffectType.ADD_ESSENCE,
            effect_value=75,
        )

        apply_journey_effect(state, journey, _make_dreamsigns())

        assert state.essence == 175

    def test_remove_cards_effect(self) -> None:
        from sites_journey import apply_journey_effect

        state = _make_quest_state()
        # Pre-populate deck
        for i in range(3):
            state.add_card(_make_card_instance(f"Card {i}", i))
        assert state.deck_count() == 3

        journey = Journey(
            name="Remove Journey",
            description="Remove cards",
            effect_type=EffectType.REMOVE_CARDS,
            effect_value=2,
        )

        apply_journey_effect(state, journey, _make_dreamsigns())

        assert state.deck_count() == 1

    def test_remove_cards_does_not_exceed_deck_size(self) -> None:
        from sites_journey import apply_journey_effect

        state = _make_quest_state()
        state.add_card(_make_card_instance("Single Card", 1))

        journey = Journey(
            name="Over-Remove",
            description="Remove 5",
            effect_type=EffectType.REMOVE_CARDS,
            effect_value=5,
        )

        apply_journey_effect(state, journey, _make_dreamsigns())

        assert state.deck_count() == 0

    def test_add_dreamsign_effect(self) -> None:
        from sites_journey import apply_journey_effect

        state = _make_quest_state()
        dreamsigns = _make_dreamsigns()
        journey = Journey(
            name="Dreamsign Journey",
            description="Add dreamsign",
            effect_type=EffectType.ADD_DREAMSIGN,
            effect_value=1,
        )

        apply_journey_effect(state, journey, dreamsigns)

        assert state.dreamsign_count() == 1
        assert not state.dreamsigns[0].is_bane

    def test_large_essence_effect(self) -> None:
        from sites_journey import apply_journey_effect

        state = _make_quest_state(essence=100)
        journey = Journey(
            name="Large Essence Journey",
            description="Gain large essence",
            effect_type=EffectType.LARGE_ESSENCE,
            effect_value=300,
        )

        changes = apply_journey_effect(state, journey, _make_dreamsigns())

        assert state.essence == 400
        assert changes["essence_delta"] == 300

    def test_gain_resonance_not_handled(self) -> None:
        """GAIN_RESONANCE no longer exists in EffectType."""
        assert not hasattr(EffectType, "GAIN_RESONANCE")


class TestRunDreamJourney:
    """Integration tests for run_dream_journey."""

    def test_selecting_a_journey_applies_effect(self) -> None:
        from sites_journey import run_dream_journey

        state = _make_quest_state(essence=100)
        journeys = [
            Journey("Essence A", "Add 75", EffectType.ADD_ESSENCE, 75),
            Journey("Essence B", "Add 100", EffectType.ADD_ESSENCE, 100),
            Journey("Essence C", "Add 50", EffectType.ADD_ESSENCE, 50),
        ]
        with patch("sites_journey.input_handler.single_select", return_value=0):
            run_dream_journey(
                state=state,
                all_journeys=journeys,
                all_dreamsigns=_make_dreamsigns(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.essence > 100

    def test_selecting_close_skips_effect(self) -> None:
        from sites_journey import run_dream_journey

        state = _make_quest_state(essence=100)
        journeys = _make_journeys()
        # 2 options + 1 close = 3 options, so index 2 is close
        with patch("sites_journey.input_handler.single_select", return_value=2):
            run_dream_journey(
                state=state,
                all_journeys=journeys,
                all_dreamsigns=_make_dreamsigns(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.essence == 100
        assert state.deck_count() == 0

    def test_enhanced_shows_three_options(self) -> None:
        from sites_journey import run_dream_journey

        state = _make_quest_state(essence=100)
        journeys = _make_journeys()
        # Enhanced: 3 options + 1 close = 4, so close is at index 3
        with patch(
            "sites_journey.input_handler.single_select", return_value=3
        ) as mock_select:
            run_dream_journey(
                state=state,
                all_journeys=journeys,
                all_dreamsigns=_make_dreamsigns(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                is_enhanced=True,
            )

        call_args = mock_select.call_args
        assert (
            len(call_args[1]["options"]) == 4
            if "options" in call_args[1]
            else len(call_args[0][0]) == 4
        )

    def test_logs_interaction(self) -> None:
        from sites_journey import run_dream_journey

        state = _make_quest_state(essence=100)
        journeys = [
            Journey("Essence A", "Add 75", EffectType.ADD_ESSENCE, 75),
            Journey("Essence B", "Add 100", EffectType.ADD_ESSENCE, 100),
        ]
        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(kwargs)

        with patch("sites_journey.input_handler.single_select", return_value=0):
            run_dream_journey(
                state=state,
                all_journeys=journeys,
                all_dreamsigns=_make_dreamsigns(),
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "DreamJourney"


# ===========================================================================
# Tempting Offer tests
# ===========================================================================


class TestSelectOffers:
    """Test offer selection picks the correct number of options."""

    def test_selects_two_by_default(self) -> None:
        from sites_journey import select_offers

        offers = _make_offers()
        rng = random.Random(42)
        result = select_offers(offers, rng, is_enhanced=False)
        assert len(result) == 2

    def test_selects_three_when_enhanced(self) -> None:
        from sites_journey import select_offers

        offers = _make_offers()
        rng = random.Random(42)
        result = select_offers(offers, rng, is_enhanced=True)
        assert len(result) == 3

    def test_all_selected_from_pool(self) -> None:
        from sites_journey import select_offers

        offers = _make_offers()
        rng = random.Random(42)
        result = select_offers(offers, rng, is_enhanced=False)
        for o in result:
            assert o in offers


class TestApplyRewardEffect:
    """Test that each reward effect type is applied correctly."""

    def test_add_cards_reward(self) -> None:
        from sites_journey import apply_reward_effect

        state = _make_quest_state()
        initial_deck = state.deck_count()

        apply_reward_effect(
            state,
            EffectType.ADD_CARDS,
            2,
            _make_dreamsigns(),
        )

        assert state.deck_count() == initial_deck + 2

    def test_add_essence_reward(self) -> None:
        from sites_journey import apply_reward_effect

        state = _make_quest_state(essence=100)

        apply_reward_effect(
            state,
            EffectType.ADD_ESSENCE,
            150,
            _make_dreamsigns(),
        )

        assert state.essence == 250

    def test_large_essence_reward(self) -> None:
        from sites_journey import apply_reward_effect

        state = _make_quest_state(essence=100)

        apply_reward_effect(
            state,
            EffectType.LARGE_ESSENCE,
            300,
            _make_dreamsigns(),
        )

        assert state.essence == 400

    def test_add_dreamsign_reward(self) -> None:
        from sites_journey import apply_reward_effect

        state = _make_quest_state()

        apply_reward_effect(
            state,
            EffectType.ADD_DREAMSIGN,
            1,
            _make_dreamsigns(),
        )

        assert state.dreamsign_count() == 1
        assert not state.dreamsigns[0].is_bane


class TestApplyCostEffect:
    """Test that each cost effect type is applied correctly."""

    def test_lose_essence_cost(self) -> None:
        from sites_journey import apply_cost_effect

        state = _make_quest_state(essence=200)

        apply_cost_effect(
            state,
            EffectType.LOSE_ESSENCE,
            50,
            _make_bane_cards(),
            _make_dreamsigns(),
        )

        assert state.essence == 150

    def test_lose_essence_floors_at_zero(self) -> None:
        from sites_journey import apply_cost_effect

        state = _make_quest_state(essence=30)

        apply_cost_effect(
            state,
            EffectType.LOSE_ESSENCE,
            50,
            _make_bane_cards(),
            _make_dreamsigns(),
        )

        assert state.essence == 0

    def test_add_bane_card_cost(self) -> None:
        from sites_journey import apply_cost_effect

        state = _make_quest_state()
        banes = _make_bane_cards()

        apply_cost_effect(
            state,
            EffectType.ADD_BANE_CARD,
            1,
            banes,
            _make_dreamsigns(),
        )

        assert state.deck_count() == 1
        assert state.deck[0].is_bane

    def test_bane_card_is_synthetic_card_design(self) -> None:
        """Bane cards should be constructed as CardDesign/CardInstance."""
        from sites_journey import apply_cost_effect

        state = _make_quest_state()
        banes = _make_bane_cards()

        apply_cost_effect(
            state,
            EffectType.ADD_BANE_CARD,
            1,
            banes,
            _make_dreamsigns(),
        )

        dc = state.deck[0]
        assert isinstance(dc, DeckCard)
        assert isinstance(dc.instance, CardInstance)
        assert isinstance(dc.instance.design, CardDesign)
        assert dc.instance.design.power == 0.0
        assert dc.instance.design.commit == 0.0
        assert dc.instance.design.flex == 0.0
        assert dc.instance.design.fitness == [0.0] * 8
        assert dc.instance.design.name in [b.name for b in banes]

    def test_bane_card_instance_id_avoids_collision(self) -> None:
        """Bane card instance_ids should use a large offset to avoid collision."""
        from sites_journey import BANE_INSTANCE_ID_OFFSET, apply_cost_effect

        state = _make_quest_state()
        banes = _make_bane_cards()

        apply_cost_effect(
            state,
            EffectType.ADD_BANE_CARD,
            1,
            banes,
            _make_dreamsigns(),
        )

        dc = state.deck[0]
        assert dc.instance.instance_id >= BANE_INSTANCE_ID_OFFSET

    def test_add_bane_dreamsign_cost(self) -> None:
        from sites_journey import apply_cost_effect

        state = _make_quest_state()
        dreamsigns = _make_dreamsigns()

        apply_cost_effect(
            state,
            EffectType.ADD_BANE_DREAMSIGN,
            1,
            _make_bane_cards(),
            dreamsigns,
        )

        assert state.dreamsign_count() == 1
        assert state.dreamsigns[0].is_bane

    def test_remove_cards_cost(self) -> None:
        from sites_journey import apply_cost_effect

        state = _make_quest_state()
        for i in range(4):
            state.add_card(_make_card_instance(f"Card {i}", i))
        assert state.deck_count() == 4

        apply_cost_effect(
            state,
            EffectType.REMOVE_CARDS,
            2,
            _make_bane_cards(),
            _make_dreamsigns(),
        )

        assert state.deck_count() == 2


class TestRunTemptingOffer:
    """Integration tests for run_tempting_offer."""

    def test_selecting_offer_applies_reward_then_cost(self) -> None:
        from sites_journey import run_tempting_offer

        state = _make_quest_state(essence=200)
        offers = [
            TemptingOffer(
                reward_name="Gain Essence",
                reward_description="Gain 150",
                reward_effect_type=EffectType.ADD_ESSENCE,
                reward_value=150,
                cost_name="Lose Essence",
                cost_description="Pay 50",
                cost_effect_type=EffectType.LOSE_ESSENCE,
                cost_value=50,
            ),
            TemptingOffer(
                reward_name="Gain Essence 2",
                reward_description="Gain 100",
                reward_effect_type=EffectType.ADD_ESSENCE,
                reward_value=100,
                cost_name="Lose Essence 2",
                cost_description="Pay 25",
                cost_effect_type=EffectType.LOSE_ESSENCE,
                cost_value=25,
            ),
        ]

        with patch("sites_journey.input_handler.single_select", return_value=0):
            run_tempting_offer(
                state=state,
                all_offers=offers,
                all_banes=_make_bane_cards(),
                all_dreamsigns=_make_dreamsigns(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.essence > 200

    def test_selecting_decline_all_skips(self) -> None:
        from sites_journey import run_tempting_offer

        state = _make_quest_state(essence=200)
        offers = _make_offers()

        with patch("sites_journey.input_handler.single_select", return_value=2):
            run_tempting_offer(
                state=state,
                all_offers=offers,
                all_banes=_make_bane_cards(),
                all_dreamsigns=_make_dreamsigns(),
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
            )

        assert state.essence == 200
        assert state.deck_count() == 0

    def test_enhanced_shows_three_options(self) -> None:
        from sites_journey import run_tempting_offer

        state = _make_quest_state(essence=200)
        offers = _make_offers()

        with patch(
            "sites_journey.input_handler.single_select", return_value=3
        ) as mock_select:
            run_tempting_offer(
                state=state,
                all_offers=offers,
                all_banes=_make_bane_cards(),
                all_dreamsigns=_make_dreamsigns(),
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
                is_enhanced=True,
            )

        call_args = mock_select.call_args
        assert (
            len(call_args[1]["options"]) == 4
            if "options" in call_args[1]
            else len(call_args[0][0]) == 4
        )

    def test_logs_interaction(self) -> None:
        from sites_journey import run_tempting_offer

        state = _make_quest_state(essence=200)
        offers = [
            TemptingOffer(
                reward_name="Gain",
                reward_description="Gain 100",
                reward_effect_type=EffectType.ADD_ESSENCE,
                reward_value=100,
                cost_name="Lose",
                cost_description="Lose 50",
                cost_effect_type=EffectType.LOSE_ESSENCE,
                cost_value=50,
            ),
            TemptingOffer(
                reward_name="Gain 2",
                reward_description="Gain 75",
                reward_effect_type=EffectType.ADD_ESSENCE,
                reward_value=75,
                cost_name="Lose 2",
                cost_description="Lose 25",
                cost_effect_type=EffectType.LOSE_ESSENCE,
                cost_value=25,
            ),
        ]
        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_site_visit(self, **kwargs: object) -> None:
                log_calls.append(kwargs)

        with patch("sites_journey.input_handler.single_select", return_value=0):
            run_tempting_offer(
                state=state,
                all_offers=offers,
                all_banes=_make_bane_cards(),
                all_dreamsigns=_make_dreamsigns(),
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "TemptingOffer"
