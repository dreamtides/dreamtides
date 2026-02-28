"""Tests for Dream Journey and Tempting Offer site interactions."""

import random
from typing import Optional
from unittest.mock import patch

from models import (
    AlgorithmParams,
    BaneCard,
    Card,
    CardType,
    DeckCard,
    Dreamsign,
    EffectType,
    Journey,
    PoolEntry,
    PoolParams,
    Rarity,
    Resonance,
    TemptingOffer,
)
from quest_state import QuestState


def _make_card(
    name: str = "Test Card",
    card_number: int = 1,
    energy_cost: int = 3,
    resonances: frozenset[Resonance] = frozenset(),
    tags: frozenset[str] = frozenset(),
    rarity: Rarity = Rarity.COMMON,
    spark: int = 2,
    card_type: CardType = CardType.CHARACTER,
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
        rules_text=f"Rules for {name}.",
        resonances=resonances,
        tags=tags,
    )


def _make_test_cards() -> list[Card]:
    """Create a set of test cards spanning rarities and resonances."""
    return [
        _make_card("Tide Card A", 1, rarity=Rarity.COMMON, resonances=frozenset({Resonance.TIDE})),
        _make_card("Tide Card B", 2, rarity=Rarity.COMMON, resonances=frozenset({Resonance.TIDE})),
        _make_card("Ember Card A", 3, rarity=Rarity.UNCOMMON, resonances=frozenset({Resonance.EMBER})),
        _make_card("Ember Card B", 4, rarity=Rarity.UNCOMMON, resonances=frozenset({Resonance.EMBER})),
        _make_card("Stone Card A", 5, rarity=Rarity.RARE, resonances=frozenset({Resonance.STONE})),
        _make_card("Zephyr Card A", 6, rarity=Rarity.COMMON, resonances=frozenset({Resonance.ZEPHYR})),
        _make_card("Ruin Card A", 7, rarity=Rarity.COMMON, resonances=frozenset({Resonance.RUIN})),
        _make_card("Neutral Card A", 8, rarity=Rarity.COMMON, resonances=frozenset()),
        _make_card("Dual Card A", 9, rarity=Rarity.LEGENDARY, resonances=frozenset({Resonance.TIDE, Resonance.RUIN})),
        _make_card("Stone Card B", 10, rarity=Rarity.UNCOMMON, resonances=frozenset({Resonance.STONE})),
    ]


def _make_pool(cards: list[Card]) -> list[PoolEntry]:
    return [PoolEntry(card) for card in cards]


def _make_algorithm_params() -> AlgorithmParams:
    return AlgorithmParams(
        exponent=1.4,
        floor_weight=0.5,
        neutral_base=3.0,
        staleness_factor=0.3,
    )


def _make_pool_params() -> PoolParams:
    return PoolParams(
        copies_common=4,
        copies_uncommon=3,
        copies_rare=2,
        copies_legendary=1,
        variance_min=0.75,
        variance_max=1.25,
    )


def _make_quest_state(
    cards: Optional[list[Card]] = None,
    pool: Optional[list[PoolEntry]] = None,
    essence: int = 250,
    seed: int = 42,
) -> QuestState:
    test_cards = cards or _make_test_cards()
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
        Journey(
            name="Test Journey E",
            description="Gains resonance",
            effect_type=EffectType.GAIN_RESONANCE,
            effect_value=2,
        ),
    ]


def _make_dreamsigns() -> list[Dreamsign]:
    return [
        Dreamsign(
            name="Test Sign A",
            resonance=Resonance.TIDE,
            tags=frozenset({"mechanic:foresee"}),
            effect_text="Test effect A",
            is_bane=False,
        ),
        Dreamsign(
            name="Test Sign B",
            resonance=Resonance.EMBER,
            tags=frozenset(),
            effect_text="Test effect B",
            is_bane=False,
        ),
        Dreamsign(
            name="Bane Sign",
            resonance=Resonance.RUIN,
            tags=frozenset(),
            effect_text="Bane effect",
            is_bane=True,
        ),
    ]


def _make_bane_cards() -> list[BaneCard]:
    return [
        BaneCard(
            name="Creeping Dread",
            rules_text="Cannot be played.",
            card_type=CardType.EVENT,
            energy_cost=99,
        ),
        BaneCard(
            name="Nightmare Residue",
            rules_text="Lose 1 energy when drawn.",
            card_type=CardType.EVENT,
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
        initial_pool = len(state.pool)

        apply_journey_effect(
            state, journey, _make_algorithm_params(), _make_pool_params(),
            _make_dreamsigns(),
        )

        assert state.deck_count() == initial_deck + 2
        assert len(state.pool) == initial_pool - 2

    def test_add_essence_effect(self) -> None:
        from sites_journey import apply_journey_effect

        state = _make_quest_state(essence=100)
        journey = Journey(
            name="Essence Journey",
            description="Add essence",
            effect_type=EffectType.ADD_ESSENCE,
            effect_value=75,
        )

        apply_journey_effect(
            state, journey, _make_algorithm_params(), _make_pool_params(),
            _make_dreamsigns(),
        )

        assert state.essence == 175

    def test_remove_cards_effect(self) -> None:
        from sites_journey import apply_journey_effect

        cards = _make_test_cards()
        state = _make_quest_state(cards=cards)
        # Pre-populate deck
        for c in cards[:3]:
            state.add_card(c)
        assert state.deck_count() == 3

        journey = Journey(
            name="Remove Journey",
            description="Remove cards",
            effect_type=EffectType.REMOVE_CARDS,
            effect_value=2,
        )

        apply_journey_effect(
            state, journey, _make_algorithm_params(), _make_pool_params(),
            _make_dreamsigns(),
        )

        assert state.deck_count() == 1

    def test_remove_cards_does_not_exceed_deck_size(self) -> None:
        from sites_journey import apply_journey_effect

        state = _make_quest_state()
        # Add only 1 card
        state.add_card(_make_test_cards()[0])

        journey = Journey(
            name="Over-Remove",
            description="Remove 5",
            effect_type=EffectType.REMOVE_CARDS,
            effect_value=5,
        )

        apply_journey_effect(
            state, journey, _make_algorithm_params(), _make_pool_params(),
            _make_dreamsigns(),
        )

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

        apply_journey_effect(
            state, journey, _make_algorithm_params(), _make_pool_params(),
            dreamsigns,
        )

        assert state.dreamsign_count() == 1
        # Should be a non-bane dreamsign
        assert not state.dreamsigns[0].is_bane

    def test_add_dreamsign_at_limit_triggers_purge(self) -> None:
        from sites_journey import apply_journey_effect

        state = _make_quest_state()
        dreamsigns = _make_dreamsigns()
        # Fill to limit
        for i in range(state.max_dreamsigns):
            state.add_dreamsign(Dreamsign(
                name=f"Sign {i}",
                resonance=Resonance.TIDE,
                tags=frozenset(),
                effect_text="",
                is_bane=False,
            ))

        journey = Journey(
            name="Excess Dreamsign",
            description="Add dreamsign",
            effect_type=EffectType.ADD_DREAMSIGN,
            effect_value=1,
        )

        # In non-interactive mode, auto-purge should handle the limit
        apply_journey_effect(
            state, journey, _make_algorithm_params(), _make_pool_params(),
            dreamsigns,
        )

        # Should still be at limit (added one, purged one)
        assert state.dreamsign_count() == state.max_dreamsigns
        # The new dreamsign should have been added (not silently dropped)
        non_filler = [ds for ds in state.dreamsigns if not ds.name.startswith("Sign ")]
        assert len(non_filler) == 1

    def test_gain_resonance_effect(self) -> None:
        from sites_journey import apply_journey_effect

        state = _make_quest_state()
        # Give some resonance profile
        state.resonance_profile.add(Resonance.TIDE, 5)
        state.resonance_profile.add(Resonance.EMBER, 2)

        journey = Journey(
            name="Resonance Journey",
            description="Gain resonance",
            effect_type=EffectType.GAIN_RESONANCE,
            effect_value=3,
        )

        apply_journey_effect(
            state, journey, _make_algorithm_params(), _make_pool_params(),
            _make_dreamsigns(),
        )

        # Top resonance was Tide (5), should now be 8
        assert state.resonance_profile.counts[Resonance.TIDE] == 8

    def test_gain_resonance_with_all_zero_profile(self) -> None:
        from sites_journey import apply_journey_effect

        state = _make_quest_state()
        # All counts are 0; gain_resonance should still add to one resonance
        journey = Journey(
            name="Zero Resonance",
            description="Gain resonance",
            effect_type=EffectType.GAIN_RESONANCE,
            effect_value=2,
        )

        apply_journey_effect(
            state, journey, _make_algorithm_params(), _make_pool_params(),
            _make_dreamsigns(),
        )

        # Should have added 2 to some resonance
        assert state.resonance_profile.total() == 2


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
        # Select index 0 (first journey) -- skip is the last option
        with patch("sites_journey.input_handler.single_select", return_value=0):
            run_dream_journey(
                state=state,
                all_journeys=journeys,
                all_dreamsigns=_make_dreamsigns(),
                algorithm_params=_make_algorithm_params(),
                pool_params=_make_pool_params(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # Essence should have changed (exact value depends on which 2 are selected)
        assert state.essence > 100

    def test_selecting_close_skips_effect(self) -> None:
        from sites_journey import run_dream_journey

        state = _make_quest_state(essence=100)
        journeys = _make_journeys()
        # Select the last option (Close/skip)
        # Normal journey: 2 options + 1 close = 3 options, so index 2 is close
        with patch("sites_journey.input_handler.single_select", return_value=2):
            run_dream_journey(
                state=state,
                all_journeys=journeys,
                all_dreamsigns=_make_dreamsigns(),
                algorithm_params=_make_algorithm_params(),
                pool_params=_make_pool_params(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # Nothing should have changed
        assert state.essence == 100
        assert state.deck_count() == 0

    def test_enhanced_shows_three_options(self) -> None:
        from sites_journey import run_dream_journey

        state = _make_quest_state(essence=100)
        journeys = _make_journeys()
        # Enhanced: 3 options + 1 close = 4, so close is at index 3
        with patch("sites_journey.input_handler.single_select", return_value=3) as mock_select:
            run_dream_journey(
                state=state,
                all_journeys=journeys,
                all_dreamsigns=_make_dreamsigns(),
                algorithm_params=_make_algorithm_params(),
                pool_params=_make_pool_params(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                is_enhanced=True,
            )

        # Verify that 4 options were presented (3 journeys + Close)
        call_args = mock_select.call_args
        assert len(call_args[1]["options"]) == 4 if "options" in call_args[1] else len(call_args[0][0]) == 4

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
                algorithm_params=_make_algorithm_params(),
                pool_params=_make_pool_params(),
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
            state, EffectType.ADD_CARDS, 2,
            _make_algorithm_params(), _make_pool_params(), _make_dreamsigns(),
        )

        assert state.deck_count() == initial_deck + 2

    def test_add_essence_reward(self) -> None:
        from sites_journey import apply_reward_effect

        state = _make_quest_state(essence=100)

        apply_reward_effect(
            state, EffectType.ADD_ESSENCE, 150,
            _make_algorithm_params(), _make_pool_params(), _make_dreamsigns(),
        )

        assert state.essence == 250

    def test_large_essence_reward(self) -> None:
        from sites_journey import apply_reward_effect

        state = _make_quest_state(essence=100)

        apply_reward_effect(
            state, EffectType.LARGE_ESSENCE, 300,
            _make_algorithm_params(), _make_pool_params(), _make_dreamsigns(),
        )

        assert state.essence == 400

    def test_add_dreamsign_reward(self) -> None:
        from sites_journey import apply_reward_effect

        state = _make_quest_state()

        apply_reward_effect(
            state, EffectType.ADD_DREAMSIGN, 1,
            _make_algorithm_params(), _make_pool_params(), _make_dreamsigns(),
        )

        assert state.dreamsign_count() == 1
        assert not state.dreamsigns[0].is_bane

    def test_add_dreamsign_reward_at_limit_triggers_purge(self) -> None:
        from sites_journey import apply_reward_effect

        state = _make_quest_state()
        dreamsigns = _make_dreamsigns()
        # Fill to limit
        for i in range(state.max_dreamsigns):
            state.add_dreamsign(Dreamsign(
                name=f"Sign {i}",
                resonance=Resonance.TIDE,
                tags=frozenset(),
                effect_text="",
                is_bane=False,
            ))

        apply_reward_effect(
            state, EffectType.ADD_DREAMSIGN, 1,
            _make_algorithm_params(), _make_pool_params(), dreamsigns,
        )

        # Should not exceed limit
        assert state.dreamsign_count() == state.max_dreamsigns
        # New dreamsign should have been added (not dropped)
        non_filler = [ds for ds in state.dreamsigns if not ds.name.startswith("Sign ")]
        assert len(non_filler) == 1


class TestApplyCostEffect:
    """Test that each cost effect type is applied correctly."""

    def test_lose_essence_cost(self) -> None:
        from sites_journey import apply_cost_effect

        state = _make_quest_state(essence=200)

        apply_cost_effect(
            state, EffectType.LOSE_ESSENCE, 50,
            _make_bane_cards(), _make_dreamsigns(),
        )

        assert state.essence == 150

    def test_lose_essence_floors_at_zero(self) -> None:
        from sites_journey import apply_cost_effect

        state = _make_quest_state(essence=30)

        apply_cost_effect(
            state, EffectType.LOSE_ESSENCE, 50,
            _make_bane_cards(), _make_dreamsigns(),
        )

        assert state.essence == 0

    def test_add_bane_card_cost(self) -> None:
        from sites_journey import apply_cost_effect

        state = _make_quest_state()
        banes = _make_bane_cards()

        apply_cost_effect(
            state, EffectType.ADD_BANE_CARD, 1,
            banes, _make_dreamsigns(),
        )

        assert state.deck_count() == 1
        assert state.deck[0].is_bane

    def test_add_bane_dreamsign_cost(self) -> None:
        from sites_journey import apply_cost_effect

        state = _make_quest_state()
        dreamsigns = _make_dreamsigns()

        apply_cost_effect(
            state, EffectType.ADD_BANE_DREAMSIGN, 1,
            _make_bane_cards(), dreamsigns,
        )

        assert state.dreamsign_count() == 1
        assert state.dreamsigns[0].is_bane

    def test_add_bane_dreamsign_at_limit_triggers_purge(self) -> None:
        from sites_journey import apply_cost_effect

        state = _make_quest_state()
        dreamsigns = _make_dreamsigns()
        # Fill to limit with non-bane dreamsigns
        for i in range(state.max_dreamsigns):
            state.add_dreamsign(Dreamsign(
                name=f"Sign {i}",
                resonance=Resonance.TIDE,
                tags=frozenset(),
                effect_text="",
                is_bane=False,
            ))

        apply_cost_effect(
            state, EffectType.ADD_BANE_DREAMSIGN, 1,
            _make_bane_cards(), dreamsigns,
        )

        # Should not exceed limit
        assert state.dreamsign_count() == state.max_dreamsigns
        # The bane dreamsign should have been added
        bane_signs = [ds for ds in state.dreamsigns if ds.is_bane]
        assert len(bane_signs) == 1

    def test_remove_cards_cost(self) -> None:
        from sites_journey import apply_cost_effect

        state = _make_quest_state()
        for c in _make_test_cards()[:4]:
            state.add_card(c)
        assert state.deck_count() == 4

        apply_cost_effect(
            state, EffectType.REMOVE_CARDS, 2,
            _make_bane_cards(), _make_dreamsigns(),
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

        # Select index 0 (first offer)
        with patch("sites_journey.input_handler.single_select", return_value=0):
            run_tempting_offer(
                state=state,
                all_offers=offers,
                all_banes=_make_bane_cards(),
                all_dreamsigns=_make_dreamsigns(),
                algorithm_params=_make_algorithm_params(),
                pool_params=_make_pool_params(),
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # Should be 200 + 150 - 50 = 300 or 200 + 100 - 25 = 275
        # depending on which 2 are selected by rng
        assert state.essence > 200

    def test_selecting_decline_all_skips(self) -> None:
        from sites_journey import run_tempting_offer

        state = _make_quest_state(essence=200)
        offers = _make_offers()

        # Decline All is the last option (index 2 for 2 offers)
        with patch("sites_journey.input_handler.single_select", return_value=2):
            run_tempting_offer(
                state=state,
                all_offers=offers,
                all_banes=_make_bane_cards(),
                all_dreamsigns=_make_dreamsigns(),
                algorithm_params=_make_algorithm_params(),
                pool_params=_make_pool_params(),
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

        # Enhanced: 3 offers + 1 decline = 4 options. Decline at index 3.
        with patch("sites_journey.input_handler.single_select", return_value=3) as mock_select:
            run_tempting_offer(
                state=state,
                all_offers=offers,
                all_banes=_make_bane_cards(),
                all_dreamsigns=_make_dreamsigns(),
                algorithm_params=_make_algorithm_params(),
                pool_params=_make_pool_params(),
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=None,
                is_enhanced=True,
            )

        call_args = mock_select.call_args
        assert len(call_args[1]["options"]) == 4 if "options" in call_args[1] else len(call_args[0][0]) == 4

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
                algorithm_params=_make_algorithm_params(),
                pool_params=_make_pool_params(),
                dreamscape_name="Test",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 1
        assert log_calls[0]["site_type"] == "TemptingOffer"
