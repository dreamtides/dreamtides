"""Tests for site_dispatch module."""

import random
from typing import Any, Optional
from unittest.mock import patch

from models import (
    BaneCard,
    Boss,
    Dreamcaller,
    Dreamsign,
    EffectType,
    Journey,
    Site,
    SiteType,
    TemptingOffer,
)
from quest_state import QuestState


def _make_quest_state(seed: int = 42) -> QuestState:
    rng = random.Random(seed)
    return QuestState(
        essence=250,
        rng=rng,
        human_agent=None,
        ai_agents=[],
        cube=None,
        draft_cfg=None,
        packs=[],
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
            "reroll_cost": 50,
            "items_count": 6,
            "discount_min": 30,
            "discount_max": 90,
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
    }


def _make_dreamcallers() -> list[Dreamcaller]:
    return [
        Dreamcaller(
            name="Test Caller",
            essence_bonus=50,
            ability_text="Test ability.",
        ),
    ]


def _make_dreamsigns() -> list[Dreamsign]:
    return [
        Dreamsign(
            name="Test Sign",
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
            card_type="Event",
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
        ),
    ]


def _make_site_data():
    from site_dispatch import SiteData

    return SiteData(
        dreamcallers=_make_dreamcallers(),
        dreamsigns=_make_dreamsigns(),
        journeys=_make_journeys(),
        offers=_make_offers(),
        banes=_make_banes(),
        bosses=_make_bosses(),
        config=_make_config(),
    )


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
        from site_dispatch import VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.ESSENCE, is_enhanced=False, is_visited=False)
        data = _make_site_data()
        ctx = VisitContext(dreamscape_name="Test Scape", dreamscape_number=1)

        assert not site.is_visited

        with patch("sites_essence.run_essence"):
            visit_site(site, state, data, None, ctx)

        assert site.is_visited

    def test_marks_site_visited_for_purge(self) -> None:
        from site_dispatch import VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.PURGE, is_enhanced=False, is_visited=False)
        data = _make_site_data()
        ctx = VisitContext(dreamscape_name="Test Scape", dreamscape_number=1)

        with patch("sites_purge.run_purge"):
            visit_site(site, state, data, None, ctx)

        assert site.is_visited


class TestDispatchRouting:
    """Test that visit_site dispatches to the correct handler."""

    def _dispatch(self, site_type: SiteType, mock_path: str) -> None:
        from site_dispatch import VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=site_type, is_enhanced=False)
        data = _make_site_data()
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch(mock_path) as mock:
            visit_site(site, state, data, None, ctx)
        mock.assert_called_once()

    def test_dispatches_draft(self) -> None:
        self._dispatch(SiteType.DRAFT, "sites_draft.run_draft")

    def test_dispatches_dreamcaller_draft(self) -> None:
        self._dispatch(
            SiteType.DREAMCALLER_DRAFT, "sites_dreamcaller.run_dreamcaller_draft"
        )

    def test_dispatches_discovery_draft(self) -> None:
        self._dispatch(SiteType.DISCOVERY_DRAFT, "sites_discovery.run_discovery_draft")

    def test_dispatches_shop(self) -> None:
        self._dispatch(SiteType.SHOP, "sites_shop.run_shop")

    def test_dispatches_specialty_shop(self) -> None:
        self._dispatch(SiteType.SPECIALTY_SHOP, "sites_discovery.run_specialty_shop")

    def test_dispatches_dreamsign_offering(self) -> None:
        self._dispatch(
            SiteType.DREAMSIGN_OFFERING, "sites_dreamsign.run_dreamsign_offering"
        )

    def test_dispatches_dreamsign_draft(self) -> None:
        self._dispatch(SiteType.DREAMSIGN_DRAFT, "sites_dreamsign.run_dreamsign_draft")

    def test_dispatches_dream_journey(self) -> None:
        self._dispatch(SiteType.DREAM_JOURNEY, "sites_journey.run_dream_journey")

    def test_dispatches_tempting_offer(self) -> None:
        self._dispatch(SiteType.TEMPTING_OFFER, "sites_journey.run_tempting_offer")

    def test_dispatches_purge(self) -> None:
        self._dispatch(SiteType.PURGE, "sites_purge.run_purge")

    def test_dispatches_essence(self) -> None:
        self._dispatch(SiteType.ESSENCE, "sites_essence.run_essence")

    def test_dispatches_transfiguration(self) -> None:
        self._dispatch(SiteType.TRANSFIGURATION, "sites_transfig.run_transfiguration")

    def test_dispatches_duplication(self) -> None:
        self._dispatch(SiteType.DUPLICATION, "sites_misc.run_duplication")

    def test_dispatches_reward_site(self) -> None:
        self._dispatch(SiteType.REWARD_SITE, "sites_misc.run_reward")

    def test_dispatches_cleanse(self) -> None:
        self._dispatch(SiteType.CLEANSE, "sites_misc.run_cleanse")

    def test_dispatches_battle(self) -> None:
        self._dispatch(SiteType.BATTLE, "sites_battle.run_battle")


class TestEnhancedFlag:
    """Test that the enhanced flag is passed through to handlers."""

    def test_enhanced_flag_passed_to_draft(self) -> None:
        from site_dispatch import VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.DRAFT, is_enhanced=True)
        data = _make_site_data()
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("sites_draft.run_draft") as mock:
            visit_site(site, state, data, None, ctx)

        _, kwargs = mock.call_args
        assert kwargs.get("is_enhanced") is True

    def test_enhanced_flag_passed_to_essence(self) -> None:
        from site_dispatch import VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.ESSENCE, is_enhanced=True)
        data = _make_site_data()
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("sites_essence.run_essence") as mock:
            visit_site(site, state, data, None, ctx)

        _, kwargs = mock.call_args
        assert kwargs.get("is_enhanced") is True

    def test_not_enhanced_by_default(self) -> None:
        from site_dispatch import VisitContext, visit_site

        state = _make_quest_state()
        site = Site(site_type=SiteType.PURGE, is_enhanced=False)
        data = _make_site_data()
        ctx = VisitContext(dreamscape_name="Test", dreamscape_number=1)

        with patch("sites_purge.run_purge") as mock:
            visit_site(site, state, data, None, ctx)

        _, kwargs = mock.call_args
        assert kwargs.get("is_enhanced") is False
