"""Tests for site_dispatch module."""

import random
from dataclasses import dataclass
from types import MappingProxyType
from typing import Any, Optional
from unittest.mock import patch

from models import (
    AlgorithmParams,
    BaneCard,
    Boss,
    Card,
    CardType,
    DraftParams,
    Dreamcaller,
    Dreamsign,
    EffectType,
    Journey,
    PoolEntry,
    PoolParams,
    Rarity,
    Resonance,
    Site,
    SiteType,
    TemptingOffer,
)
from quest_state import QuestState


def _make_card(
    name: str = "Test Card",
    card_number: int = 1,
    rarity: Rarity = Rarity.COMMON,
    resonances: Optional[frozenset[Resonance]] = None,
) -> Card:
    return Card(
        name=name,
        card_number=card_number,
        energy_cost=2,
        card_type=CardType.CHARACTER,
        subtype=None,
        is_fast=False,
        spark=1,
        rarity=rarity,
        rules_text=f"Rules for {name}.",
        resonances=resonances or frozenset(),
        tags=frozenset(),
    )


def _make_test_cards() -> list[Card]:
    return [
        _make_card("Card A", 1, Rarity.COMMON, frozenset({Resonance.TIDE})),
        _make_card("Card B", 2, Rarity.COMMON, frozenset({Resonance.TIDE})),
        _make_card("Card C", 3, Rarity.UNCOMMON, frozenset({Resonance.EMBER})),
        _make_card("Card D", 4, Rarity.UNCOMMON, frozenset({Resonance.EMBER})),
        _make_card("Card E", 5, Rarity.RARE, frozenset({Resonance.STONE})),
        _make_card("Card F", 6, Rarity.COMMON, frozenset({Resonance.ZEPHYR})),
        _make_card("Card G", 7, Rarity.COMMON, frozenset({Resonance.RUIN})),
        _make_card("Card H", 8, Rarity.COMMON, frozenset()),
        _make_card(
            "Card I", 9, Rarity.LEGENDARY, frozenset({Resonance.TIDE, Resonance.RUIN})
        ),
        _make_card("Card J", 10, Rarity.UNCOMMON, frozenset({Resonance.STONE})),
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


def _make_draft_params() -> DraftParams:
    return DraftParams(cards_per_pick=4, picks_per_site=5)


def _make_pool_params() -> PoolParams:
    return PoolParams(
        copies_common=4,
        copies_uncommon=3,
        copies_rare=2,
        copies_legendary=1,
        variance_min=0.75,
        variance_max=1.25,
    )


def _make_quest_state(seed: int = 42) -> QuestState:
    cards = _make_test_cards()
    pool = _make_pool(cards)
    rng = random.Random(seed)
    variance = {r: 1.0 for r in Resonance}
    return QuestState(
        essence=250,
        pool=pool,
        rng=rng,
        all_cards=cards,
        pool_variance=variance,
    )


def _make_config() -> dict[str, dict[str, Any]]:
    return {
        "quest": {
            "starting_essence": 250,
            "max_deck": 50,
            "min_deck": 25,
            "max_dreamsigns": 12,
            "total_battles": 7,
            "miniboss_battle": 4,
        },
        "shop": {
            "price_common": 50,
            "price_uncommon": 80,
            "price_rare": 120,
            "price_legendary": 200,
            "reroll_cost": 50,
            "discount_min": 30,
            "discount_max": 90,
            "items_count": 6,
        },
        "battle_rewards": {
            "base_essence": 100,
            "per_level": 25,
            "rare_pick_count": 3,
        },
        "essence_sites": {
            "amount_level_0": 200,
            "amount_level_2": 250,
            "amount_level_4": 300,
        },
        "tags": {
            "scale": 1.5,
            "min_theme_cards": 6,
            "relevance_boost": 2.0,
            "depth_factor": 0.1,
        },
    }


def _make_dreamcallers() -> list[Dreamcaller]:
    return [
        Dreamcaller(
            name="Test Caller",
            resonances=frozenset({Resonance.TIDE}),
            resonance_bonus=MappingProxyType({"Tide": 3}),
            tags=frozenset({"mechanic:draw"}),
            tag_bonus=MappingProxyType({"mechanic:draw": 1}),
            essence_bonus=50,
            ability_text="Test ability.",
        ),
    ]


def _make_dreamsigns() -> list[Dreamsign]:
    return [
        Dreamsign(
            name="Test Sign",
            resonance=Resonance.TIDE,
            tags=frozenset(),
            effect_text="Test effect.",
            is_bane=False,
        ),
    ]


def _make_journeys() -> list[Journey]:
    return [
        Journey(
            name="Test Journey",
            description="A test journey.",
            effect_type=EffectType.ADD_ESSENCE,
            effect_value=100,
        ),
    ]


def _make_offers() -> list[TemptingOffer]:
    return [
        TemptingOffer(
            reward_name="Test Reward",
            reward_description="A reward.",
            reward_effect_type=EffectType.ADD_ESSENCE,
            reward_value=50,
            cost_name="Test Cost",
            cost_description="A cost.",
            cost_effect_type=EffectType.LOSE_ESSENCE,
            cost_value=25,
        ),
    ]


def _make_banes() -> list[BaneCard]:
    return [
        BaneCard(
            name="Test Bane",
            rules_text="Bad things happen.",
            card_type=CardType.EVENT,
            energy_cost=0,
        ),
    ]


def _make_bosses() -> list[Boss]:
    return [
        Boss(
            name="Test Boss",
            archetype="Aggro",
            ability_text="Test boss ability.",
            deck_description="A test deck.",
            is_final=False,
            resonances=frozenset({Resonance.EMBER}),
        ),
    ]


class TestVisitSiteExists:
    """Test that visit_site function exists and is importable."""

    def test_import(self) -> None:
        from site_dispatch import visit_site

        assert callable(visit_site)

    def test_site_data_exists(self) -> None:
        from site_dispatch import SiteData

        assert SiteData is not None

    def test_visit_context_exists(self) -> None:
        from site_dispatch import VisitContext

        assert VisitContext is not None


class TestVisitSiteMarksVisited:
    """Test that visit_site marks the site as visited after dispatch."""

    def test_marks_site_visited_for_essence(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.ESSENCE, is_enhanced=False, is_visited=False)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test Scape", dreamscape_number=1)

        assert not site.is_visited

        with patch(
            "site_dispatch.sites_essence.run_essence",
        ):
            visit_site(site, state, data, None, ctx)

        assert site.is_visited

    def test_marks_site_visited_for_purge(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.PURGE, is_enhanced=False, is_visited=False)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test Scape", dreamscape_number=1)

        with patch("site_dispatch.sites_purge.run_purge"):
            visit_site(site, state, data, None, ctx)

        assert site.is_visited


class TestDispatchRouting:
    """Test that visit_site dispatches to the correct handler."""

    def test_dispatches_draft(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.DRAFT, is_enhanced=False)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_draft.run_draft") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_dreamcaller_draft(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.DREAMCALLER_DRAFT)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_dreamcaller.run_dreamcaller_draft") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_discovery_draft(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.DISCOVERY_DRAFT)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_discovery.run_discovery_draft") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_shop(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.SHOP)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_shop.run_shop") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_specialty_shop(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.SPECIALTY_SHOP)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_discovery.run_specialty_shop") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_dreamsign_offering(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.DREAMSIGN_OFFERING)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_dreamsign.run_dreamsign_offering") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_dreamsign_draft(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.DREAMSIGN_DRAFT)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_dreamsign.run_dreamsign_draft") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_dream_journey(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.DREAM_JOURNEY)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_journey.run_dream_journey") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_tempting_offer(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.TEMPTING_OFFER)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_journey.run_tempting_offer") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_purge(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.PURGE)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_purge.run_purge") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_essence(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.ESSENCE)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_essence.run_essence") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_transfiguration(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.TRANSFIGURATION)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_transfig.run_transfiguration") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_duplication(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.DUPLICATION)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_misc.run_duplication") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_reward_site(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.REWARD_SITE)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_misc.run_reward") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_cleanse(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.CLEANSE)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_misc.run_cleanse") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()

    def test_dispatches_battle(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.BATTLE)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_battle.run_battle") as mock:
            visit_site(site, state, data, None, ctx)

        mock.assert_called_once()


class TestEnhancedFlag:
    """Test that the enhanced flag is passed through to handlers."""

    def test_enhanced_flag_passed_to_draft(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.DRAFT, is_enhanced=True)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_draft.run_draft") as mock:
            visit_site(site, state, data, None, ctx)

        _, kwargs = mock.call_args
        assert kwargs.get("is_enhanced") is True

    def test_enhanced_flag_passed_to_essence(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.ESSENCE, is_enhanced=True)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_essence.run_essence") as mock:
            visit_site(site, state, data, None, ctx)

        _, kwargs = mock.call_args
        assert kwargs.get("is_enhanced") is True

    def test_not_enhanced_by_default(self) -> None:
        from site_dispatch import SiteData, VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.PURGE, is_enhanced=False)
        data = SiteData(
            dreamcallers=_make_dreamcallers(),
            dreamsigns=_make_dreamsigns(),
            journeys=_make_journeys(),
            offers=_make_offers(),
            banes=_make_banes(),
            bosses=_make_bosses(),
            algorithm_params=_make_algorithm_params(),
            draft_params=_make_draft_params(),
            pool_params=_make_pool_params(),
            config=_make_config(),
        )
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("site_dispatch.sites_purge.run_purge") as mock:
            visit_site(site, state, data, None, ctx)

        _, kwargs = mock.call_args
        assert kwargs.get("is_enhanced") is False
