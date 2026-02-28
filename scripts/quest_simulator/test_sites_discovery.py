"""Tests for Discovery Draft and Specialty Shop site interactions."""

import random
from typing import Optional

from models import (
    AlgorithmParams,
    Card,
    CardType,
    DraftParams,
    PoolEntry,
    Rarity,
    Resonance,
    ResonanceProfile,
    TagProfile,
)
from quest_state import QuestState


def _make_card(
    name: str = "Test Card",
    card_number: int = 1,
    tags: frozenset[str] = frozenset(),
    rarity: Rarity = Rarity.COMMON,
    resonances: frozenset[Resonance] = frozenset(),
    energy_cost: Optional[int] = 3,
    spark: Optional[int] = 2,
    rules_text: str = "Test rules",
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
        rules_text=rules_text,
        resonances=resonances,
        tags=tags,
    )


def _make_pool_entries(
    tag: str,
    count: int,
    start_number: int = 1,
    rarity: Rarity = Rarity.COMMON,
    resonances: frozenset[Resonance] = frozenset(),
) -> list[PoolEntry]:
    """Create pool entries whose cards all have the given tag."""
    return [
        PoolEntry(
            _make_card(
                name=f"{tag} Card {i}",
                card_number=start_number + i,
                tags=frozenset({tag}),
                rarity=rarity,
                resonances=resonances,
            )
        )
        for i in range(count)
    ]


def _default_params() -> AlgorithmParams:
    return AlgorithmParams(
        exponent=1.4,
        floor_weight=0.5,
        neutral_base=3.0,
        staleness_factor=0.3,
    )


def _default_tag_config() -> dict[str, float]:
    return {
        "scale": 1.5,
        "min_theme_cards": 6,
        "relevance_boost": 2.0,
        "depth_factor": 0.1,
    }


def _default_shop_config() -> dict[str, int]:
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
    essence: int = 500,
    seed: int = 42,
    pool: Optional[list[PoolEntry]] = None,
    all_cards: Optional[list[Card]] = None,
) -> QuestState:
    rng = random.Random(seed)
    if pool is None:
        pool = []
    if all_cards is None:
        all_cards = [e.card for e in pool]
    variance = {r: 1.0 for r in Resonance}
    return QuestState(
        essence=essence,
        pool=pool,
        rng=rng,
        all_cards=all_cards,
        pool_variance=variance,
    )


class TestDiscoveryDraftThemeSelection:
    def test_selects_themed_cards_from_pool(self) -> None:
        """Discovery draft should filter pool by the selected theme tag."""
        from sites_discovery import _select_discovery_cards

        themed = _make_pool_entries("tribal:warrior", 10, start_number=1)
        other = _make_pool_entries("tribal:mage", 10, start_number=100)
        pool = themed + other

        profile = ResonanceProfile()
        tag_profile = TagProfile()
        tag_profile.add("tribal:warrior", 5)
        rng = random.Random(42)

        result = _select_discovery_cards(
            pool=pool,
            resonance_profile=profile,
            tag_profile=tag_profile,
            params=_default_params(),
            rng=rng,
            cards_per_pick=4,
            tag_config=_default_tag_config(),
        )

        assert result is not None
        selected_entries, tag = result
        assert len(selected_entries) <= 4
        assert tag is not None

    def test_falls_back_to_unthemed_when_no_eligible_tags(self) -> None:
        """When no tag has enough cards, falls back to unthemed selection."""
        from sites_discovery import _select_discovery_cards

        # Each tag has only 3 cards -- below min_theme_cards of 6
        pool = (
            _make_pool_entries("tag_a", 3, start_number=1)
            + _make_pool_entries("tag_b", 3, start_number=100)
        )
        profile = ResonanceProfile()
        tag_profile = TagProfile()
        rng = random.Random(42)

        result = _select_discovery_cards(
            pool=pool,
            resonance_profile=profile,
            tag_profile=tag_profile,
            params=_default_params(),
            rng=rng,
            cards_per_pick=4,
            tag_config=_default_tag_config(),
        )

        assert result is not None
        selected_entries, tag = result
        assert tag is None  # Unthemed fallback
        assert len(selected_entries) <= 4

    def test_returns_none_for_empty_pool(self) -> None:
        """Empty pool returns None."""
        from sites_discovery import _select_discovery_cards

        profile = ResonanceProfile()
        tag_profile = TagProfile()
        rng = random.Random(42)

        result = _select_discovery_cards(
            pool=[],
            resonance_profile=profile,
            tag_profile=tag_profile,
            params=_default_params(),
            rng=rng,
            cards_per_pick=4,
            tag_config=_default_tag_config(),
        )

        assert result is None


class TestDiscoveryDraftPicking:
    def test_normal_picks_one_card(self) -> None:
        """Normal (non-enhanced) discovery draft should pick 1 card."""
        from sites_discovery import _apply_discovery_pick

        cards = [
            _make_card(name=f"Card {i}", card_number=i, tags=frozenset({"tag_a"}))
            for i in range(4)
        ]
        entries = [PoolEntry(c) for c in cards]
        paired = [(e, 1.0) for e in entries]

        # Simulate picking index 1
        picked_indices = [1]
        result = _apply_discovery_pick(paired, picked_indices)
        picked, unpicked = result
        assert len(picked) == 1
        assert picked[0] is entries[1]
        assert len(unpicked) == 3

    def test_enhanced_picks_multiple_cards(self) -> None:
        """Enhanced discovery draft should allow picking multiple cards."""
        from sites_discovery import _apply_discovery_pick

        cards = [
            _make_card(name=f"Card {i}", card_number=i, tags=frozenset({"tag_a"}))
            for i in range(4)
        ]
        entries = [PoolEntry(c) for c in cards]
        paired = [(e, 1.0) for e in entries]

        picked_indices = [0, 2, 3]
        result = _apply_discovery_pick(paired, picked_indices)
        picked, unpicked = result
        assert len(picked) == 3
        assert len(unpicked) == 1


class TestShopPricing:
    def test_price_by_rarity(self) -> None:
        """Shop prices should match rarity-based pricing."""
        from sites_discovery import _compute_price

        config = _default_shop_config()
        assert _compute_price(Rarity.COMMON, config) == 50
        assert _compute_price(Rarity.UNCOMMON, config) == 80
        assert _compute_price(Rarity.RARE, config) == 120
        assert _compute_price(Rarity.LEGENDARY, config) == 200

    def test_discount_in_range(self) -> None:
        """Discount should be between 30% and 90%, rounded to nearest 10."""
        from sites_discovery import _apply_discount

        rng = random.Random(42)
        for _ in range(100):
            base_price = 100
            discounted = _apply_discount(base_price, rng, 30, 90)
            # Price should be between 10 and 70 (100 * 0.1 to 100 * 0.7)
            assert 10 <= discounted <= 70
            assert discounted % 10 == 0

    def test_discount_rounds_to_nearest_ten(self) -> None:
        """Discounted price should always be a multiple of 10."""
        from sites_discovery import _apply_discount

        for seed in range(50):
            rng = random.Random(seed)
            result = _apply_discount(120, rng, 30, 90)
            assert result % 10 == 0
            assert result > 0


class TestSpecialtyShopItems:
    def test_selects_themed_items(self) -> None:
        """Specialty shop should filter items by theme tag."""
        from sites_discovery import _select_specialty_items

        themed = _make_pool_entries("tribal:warrior", 10, start_number=1)
        other = _make_pool_entries("tribal:mage", 10, start_number=100)
        pool = themed + other

        profile = ResonanceProfile()
        tag_profile = TagProfile()
        tag_profile.add("tribal:warrior", 5)
        rng = random.Random(42)

        result = _select_specialty_items(
            pool=pool,
            resonance_profile=profile,
            tag_profile=tag_profile,
            params=_default_params(),
            rng=rng,
            items_count=6,
            tag_config=_default_tag_config(),
        )

        assert result is not None
        items, tag = result
        assert len(items) <= 6
        assert tag is not None

    def test_falls_back_when_no_eligible_tags(self) -> None:
        """When no tag qualifies, falls back to unthemed selection."""
        from sites_discovery import _select_specialty_items

        # Too few cards per tag
        pool = (
            _make_pool_entries("tag_a", 3, start_number=1)
            + _make_pool_entries("tag_b", 3, start_number=100)
        )
        profile = ResonanceProfile()
        tag_profile = TagProfile()
        rng = random.Random(42)

        result = _select_specialty_items(
            pool=pool,
            resonance_profile=profile,
            tag_profile=tag_profile,
            params=_default_params(),
            rng=rng,
            items_count=6,
            tag_config=_default_tag_config(),
        )

        assert result is not None
        items, tag = result
        assert tag is None
        assert len(items) <= 6

    def test_returns_none_for_empty_pool(self) -> None:
        """Empty pool returns None."""
        from sites_discovery import _select_specialty_items

        profile = ResonanceProfile()
        tag_profile = TagProfile()
        rng = random.Random(42)

        result = _select_specialty_items(
            pool=[],
            resonance_profile=profile,
            tag_profile=tag_profile,
            params=_default_params(),
            rng=rng,
            items_count=6,
            tag_config=_default_tag_config(),
        )

        assert result is None

    def test_fewer_items_when_not_enough_themed_cards(self) -> None:
        """If filtered pool has fewer cards than items_count, offer what's available."""
        from sites_discovery import _select_specialty_items

        # Only 4 cards with this tag -- items_count is 6
        pool = _make_pool_entries("tribal:warrior", 4, start_number=1)
        # Add extras so the tag is eligible (need at least 6 for min_theme_cards)
        pool += _make_pool_entries("tribal:warrior", 4, start_number=100)

        profile = ResonanceProfile()
        tag_profile = TagProfile()
        tag_profile.add("tribal:warrior", 5)
        rng = random.Random(42)

        result = _select_specialty_items(
            pool=pool,
            resonance_profile=profile,
            tag_profile=tag_profile,
            params=_default_params(),
            rng=rng,
            items_count=6,
            tag_config=_default_tag_config(),
        )

        assert result is not None
        items, tag = result
        assert tag == "tribal:warrior"
        assert len(items) <= 6


class TestRuntimeTagConfigKeys:
    """Verify functions work with config keys as produced by site_dispatch.

    The config.toml [tags] section uses 'scale' (not 'tag_scale'), and
    site_dispatch builds the tag_config dict directly from those keys.
    """

    def _runtime_tag_config(self) -> dict[str, float]:
        """Tag config keys as they appear in config.toml and site_dispatch."""
        return {
            "scale": 1.5,
            "min_theme_cards": 6,
            "relevance_boost": 2.0,
            "depth_factor": 0.1,
        }

    def test_select_discovery_cards_with_runtime_keys(self) -> None:
        """Discovery card selection should work with config.toml key names."""
        from sites_discovery import _select_discovery_cards

        pool = _make_pool_entries("tribal:warrior", 10, start_number=1)
        profile = ResonanceProfile()
        tag_profile = TagProfile()
        tag_profile.add("tribal:warrior", 5)
        rng = random.Random(42)

        result = _select_discovery_cards(
            pool=pool,
            resonance_profile=profile,
            tag_profile=tag_profile,
            params=_default_params(),
            rng=rng,
            cards_per_pick=4,
            tag_config=self._runtime_tag_config(),
        )

        assert result is not None

    def test_select_specialty_items_with_runtime_keys(self) -> None:
        """Specialty shop item selection should work with config.toml key names."""
        from sites_discovery import _select_specialty_items

        pool = _make_pool_entries("tribal:warrior", 10, start_number=1)
        profile = ResonanceProfile()
        tag_profile = TagProfile()
        tag_profile.add("tribal:warrior", 5)
        rng = random.Random(42)

        result = _select_specialty_items(
            pool=pool,
            resonance_profile=profile,
            tag_profile=tag_profile,
            params=_default_params(),
            rng=rng,
            items_count=6,
            tag_config=self._runtime_tag_config(),
        )

        assert result is not None


class TestSpecialtyShopPurchasing:
    def test_compute_shop_items_with_discount(self) -> None:
        """One item should have a discount applied."""
        from sites_discovery import ShopItem, _prepare_shop_items

        cards = [
            _make_card(
                name=f"Card {i}",
                card_number=i,
                rarity=Rarity.COMMON,
                tags=frozenset({"tag_a"}),
            )
            for i in range(6)
        ]
        entries = [PoolEntry(c) for c in cards]
        paired = [(e, 1.0) for e in entries]
        rng = random.Random(42)
        config = _default_shop_config()

        items = _prepare_shop_items(paired, rng, config)

        assert len(items) == 6
        discount_count = sum(
            1 for item in items if item.discounted_price is not None
        )
        assert discount_count == 1

        for item in items:
            assert isinstance(item, ShopItem)
            assert item.base_price > 0

    def test_total_cost_calculation(self) -> None:
        """Total cost should sum the effective prices of selected items."""
        from sites_discovery import ShopItem, _total_cost

        entry = PoolEntry(_make_card())
        items = [
            ShopItem(entry=entry, base_price=50, discounted_price=None),
            ShopItem(entry=entry, base_price=80, discounted_price=30),
            ShopItem(entry=entry, base_price=120, discounted_price=None),
        ]
        # Select items 0 and 1
        assert _total_cost(items, [0, 1]) == 80  # 50 + 30
        # Select all
        assert _total_cost(items, [0, 1, 2]) == 200  # 50 + 30 + 120
        # Select none
        assert _total_cost(items, []) == 0

    def test_effective_price_uses_discount_when_available(self) -> None:
        """Effective price should use discounted_price when set."""
        from sites_discovery import ShopItem, _effective_price

        entry = PoolEntry(_make_card())
        item_no_discount = ShopItem(entry=entry, base_price=100, discounted_price=None)
        item_discounted = ShopItem(entry=entry, base_price=100, discounted_price=30)

        assert _effective_price(item_no_discount) == 100
        assert _effective_price(item_discounted) == 30
