"""Tests for sites_shop module."""

import random
from typing import Optional
from unittest.mock import patch

from models import (
    AlgorithmParams,
    Card,
    CardType,
    PoolEntry,
    PoolParams,
    Rarity,
    Resonance,
)
from quest_state import QuestState


def _make_card(
    name: str,
    card_number: int,
    rarity: Rarity = Rarity.COMMON,
    resonances: Optional[frozenset[Resonance]] = None,
    energy_cost: int = 2,
    spark: Optional[int] = 1,
) -> Card:
    return Card(
        name=name,
        card_number=card_number,
        energy_cost=energy_cost,
        card_type=CardType.CHARACTER,
        subtype=None,
        is_fast=False,
        spark=spark,
        rarity=rarity,
        rules_text=f"Rules for {name}.",
        resonances=resonances or frozenset(),
        tags=frozenset(),
    )


def _make_test_cards() -> list[Card]:
    """Create a set of test cards spanning rarities and resonances."""
    return [
        _make_card("Tide Card A", 1, Rarity.COMMON, frozenset({Resonance.TIDE})),
        _make_card("Tide Card B", 2, Rarity.COMMON, frozenset({Resonance.TIDE})),
        _make_card("Ember Card A", 3, Rarity.UNCOMMON, frozenset({Resonance.EMBER})),
        _make_card("Ember Card B", 4, Rarity.UNCOMMON, frozenset({Resonance.EMBER})),
        _make_card("Stone Card A", 5, Rarity.RARE, frozenset({Resonance.STONE})),
        _make_card("Zephyr Card A", 6, Rarity.COMMON, frozenset({Resonance.ZEPHYR})),
        _make_card("Ruin Card A", 7, Rarity.COMMON, frozenset({Resonance.RUIN})),
        _make_card("Neutral Card A", 8, Rarity.COMMON, frozenset()),
        _make_card(
            "Dual Card A",
            9,
            Rarity.LEGENDARY,
            frozenset({Resonance.TIDE, Resonance.RUIN}),
        ),
        _make_card("Stone Card B", 10, Rarity.UNCOMMON, frozenset({Resonance.STONE})),
    ]


def _make_pool(cards: list[Card]) -> list[PoolEntry]:
    """Create a simple pool with 1 entry per card."""
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


def _make_shop_config() -> dict[str, int]:
    return {
        "price_common": 50,
        "price_uncommon": 80,
        "price_rare": 120,
        "price_legendary": 200,
        "reroll_cost": 50,
        "discount_min": 30,
        "discount_max": 90,
        "items_count": 6,
    }


def _make_quest_state(
    cards: Optional[list[Card]] = None,
    pool: Optional[list[PoolEntry]] = None,
    seed: int = 42,
    essence: int = 500,
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


class TestGetPrice:
    """Tests for rarity-based pricing."""

    def test_common_price(self) -> None:
        from sites_shop import get_price

        config = _make_shop_config()
        assert get_price(Rarity.COMMON, config) == 50

    def test_uncommon_price(self) -> None:
        from sites_shop import get_price

        config = _make_shop_config()
        assert get_price(Rarity.UNCOMMON, config) == 80

    def test_rare_price(self) -> None:
        from sites_shop import get_price

        config = _make_shop_config()
        assert get_price(Rarity.RARE, config) == 120

    def test_legendary_price(self) -> None:
        from sites_shop import get_price

        config = _make_shop_config()
        assert get_price(Rarity.LEGENDARY, config) == 200


class TestApplyDiscount:
    """Tests for discount application."""

    def test_discount_rounds_to_nearest_ten(self) -> None:
        from sites_shop import apply_discount

        # 50% of 120 = 60, which is already a multiple of 10
        assert apply_discount(120, 50) == 60

    def test_discount_rounds_correctly(self) -> None:
        from sites_shop import apply_discount

        # 30% of 80 = 24 discount -> price 56, round to 60
        assert apply_discount(80, 30) == 60

    def test_discount_minimum_is_ten(self) -> None:
        from sites_shop import apply_discount

        # 90% of 50 = 45 discount -> price 5, round to 10
        result = apply_discount(50, 90)
        assert result >= 10

    def test_full_discount_clamps_to_ten(self) -> None:
        from sites_shop import apply_discount

        # Even 90% off shouldn't go below 10
        result = apply_discount(50, 90)
        assert result >= 10


class TestGenerateShopItems:
    """Tests for shop item generation."""

    def test_generates_correct_count(self) -> None:
        from sites_shop import generate_shop_items

        cards = _make_test_cards()
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool)
        config = _make_shop_config()
        params = _make_algorithm_params()

        items = generate_shop_items(state, params, config)
        assert len(items) == 6

    def test_exactly_one_discount(self) -> None:
        from sites_shop import generate_shop_items

        cards = _make_test_cards()
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool)
        config = _make_shop_config()
        params = _make_algorithm_params()

        items = generate_shop_items(state, params, config)
        discounted = [item for item in items if item.discounted_price is not None]
        assert len(discounted) == 1

    def test_discounted_price_is_less_than_original(self) -> None:
        from sites_shop import generate_shop_items

        cards = _make_test_cards()
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool)
        config = _make_shop_config()
        params = _make_algorithm_params()

        items = generate_shop_items(state, params, config)
        for item in items:
            if item.discounted_price is not None:
                assert item.discounted_price < item.base_price

    def test_small_pool_gives_fewer_items(self) -> None:
        from sites_shop import generate_shop_items

        cards = _make_test_cards()[:3]
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool)
        config = _make_shop_config()
        params = _make_algorithm_params()

        items = generate_shop_items(state, params, config)
        assert len(items) <= 3

    def test_items_have_pool_entries(self) -> None:
        from sites_shop import generate_shop_items

        cards = _make_test_cards()
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool)
        config = _make_shop_config()
        params = _make_algorithm_params()

        items = generate_shop_items(state, params, config)
        for item in items:
            assert item.pool_entry is not None
            assert item.pool_entry.card == item.card


class TestRunShop:
    """Tests for the main run_shop interaction."""

    def test_buying_cards_adds_to_deck(self) -> None:
        from sites_shop import run_shop

        cards = _make_test_cards()
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool)
        config = _make_shop_config()
        params = _make_algorithm_params()

        # Select index 0 and 1 (first two items)
        with patch(
            "sites_shop.input_handler.multi_select", return_value=[0, 1]
        ):
            run_shop(
                state=state,
                algorithm_params=params,
                pool_params=_make_pool_params(),
                shop_config=config,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == 2

    def test_buying_cards_deducts_essence(self) -> None:
        from sites_shop import run_shop

        cards = _make_test_cards()
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool, essence=500)
        config = _make_shop_config()
        params = _make_algorithm_params()
        initial_essence = state.essence

        # Buy one item
        with patch(
            "sites_shop.input_handler.multi_select", return_value=[0]
        ):
            run_shop(
                state=state,
                algorithm_params=params,
                pool_params=_make_pool_params(),
                shop_config=config,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.essence < initial_essence

    def test_buying_nothing_preserves_essence(self) -> None:
        from sites_shop import run_shop

        cards = _make_test_cards()
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool, essence=500)
        config = _make_shop_config()
        params = _make_algorithm_params()
        initial_essence = state.essence

        # Buy nothing
        with patch(
            "sites_shop.input_handler.multi_select", return_value=[]
        ):
            run_shop(
                state=state,
                algorithm_params=params,
                pool_params=_make_pool_params(),
                shop_config=config,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.essence == initial_essence
        assert state.deck_count() == 0

    def test_bought_cards_removed_from_pool(self) -> None:
        from sites_shop import run_shop

        cards = _make_test_cards()
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool)
        config = _make_shop_config()
        params = _make_algorithm_params()
        initial_pool_size = len(state.pool)

        with patch(
            "sites_shop.input_handler.multi_select", return_value=[0, 1]
        ):
            run_shop(
                state=state,
                algorithm_params=params,
                pool_params=_make_pool_params(),
                shop_config=config,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert len(state.pool) == initial_pool_size - 2

    def test_reroll_generates_new_items(self) -> None:
        """Selecting the reroll option should regenerate items."""
        from sites_shop import run_shop

        cards = _make_test_cards()
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool, essence=500)
        config = _make_shop_config()
        params = _make_algorithm_params()

        # First call: select reroll (index 6 = reroll option, the 7th item)
        # Second call: buy nothing (empty selection)
        call_count = [0]

        def mock_multi_select(
            options: list[str], **kwargs: object
        ) -> list[int]:
            call_count[0] += 1
            if call_count[0] == 1:
                # Select reroll option (last option)
                return [len(options) - 1]
            return []

        with patch(
            "sites_shop.input_handler.multi_select",
            side_effect=mock_multi_select,
        ):
            run_shop(
                state=state,
                algorithm_params=params,
                pool_params=_make_pool_params(),
                shop_config=config,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        # Reroll costs 50 essence
        assert state.essence == 500 - 50

    def test_reroll_free_when_enhanced(self) -> None:
        """First reroll is free when enhanced (Verdant biome)."""
        from sites_shop import run_shop

        cards = _make_test_cards()
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool, essence=500)
        config = _make_shop_config()
        params = _make_algorithm_params()

        call_count = [0]

        def mock_multi_select(
            options: list[str], **kwargs: object
        ) -> list[int]:
            call_count[0] += 1
            if call_count[0] == 1:
                return [len(options) - 1]  # reroll
            return []  # buy nothing

        with patch(
            "sites_shop.input_handler.multi_select",
            side_effect=mock_multi_select,
        ):
            run_shop(
                state=state,
                algorithm_params=params,
                pool_params=_make_pool_params(),
                shop_config=config,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
                is_enhanced=True,
            )

        # First reroll was free, so essence unchanged
        assert state.essence == 500

    def test_shop_with_logger(self) -> None:
        """Shop should call log_shop_purchase on the logger."""
        from sites_shop import run_shop

        cards = _make_test_cards()
        pool = _make_pool(cards)
        state = _make_quest_state(cards, pool)
        config = _make_shop_config()
        params = _make_algorithm_params()

        log_calls: list[dict[str, object]] = []

        class FakeLogger:
            def log_shop_purchase(
                self,
                items_shown: list[Card],
                items_bought: list[Card],
                essence_spent: int,
            ) -> None:
                log_calls.append({
                    "shown": list(items_shown),
                    "bought": list(items_bought),
                    "spent": essence_spent,
                })

        with patch(
            "sites_shop.input_handler.multi_select", return_value=[0]
        ):
            run_shop(
                state=state,
                algorithm_params=params,
                pool_params=_make_pool_params(),
                shop_config=config,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=FakeLogger(),  # type: ignore[arg-type]
            )

        assert len(log_calls) == 1
        assert len(log_calls[0]["shown"]) > 0  # type: ignore[arg-type]

    def test_shop_empty_pool(self) -> None:
        """Shop should handle an empty pool gracefully."""
        from sites_shop import run_shop

        cards = _make_test_cards()
        state = _make_quest_state(cards, pool=[])
        config = _make_shop_config()
        params = _make_algorithm_params()

        with patch(
            "sites_shop.input_handler.multi_select", return_value=[]
        ):
            run_shop(
                state=state,
                algorithm_params=params,
                pool_params=_make_pool_params(),
                shop_config=config,
                dreamscape_name="Test Dreamscape",
                dreamscape_number=1,
                logger=None,
            )

        assert state.deck_count() == 0
