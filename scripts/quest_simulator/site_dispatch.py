"""Site dispatch for the quest simulator.

Routes each site visit to the appropriate handler function based on
the site's SiteType. Provides a unified entry point so that the
quest flow module only needs to call visit_site() for any site type.
"""

from dataclasses import dataclass
from typing import Any, Optional

import sites_battle
import sites_discovery
import sites_draft
import sites_dreamcaller
import sites_dreamsign
import sites_essence
import sites_journey
import sites_misc
import sites_purge
import sites_shop
import sites_transfig
from jsonl_log import SessionLogger
from models import (
    AlgorithmParams,
    BaneCard,
    Boss,
    DraftParams,
    Dreamcaller,
    Dreamsign,
    Journey,
    PoolParams,
    Site,
    SiteType,
    TemptingOffer,
)
from quest_state import QuestState


@dataclass(frozen=True)
class SiteData:
    """All loaded data needed by site handlers.

    Created once at startup and passed unchanged through the quest
    flow to visit_site.
    """

    dreamcallers: list[Dreamcaller]
    dreamsigns: list[Dreamsign]
    journeys: list[Journey]
    offers: list[TemptingOffer]
    banes: list[BaneCard]
    bosses: list[Boss]
    algorithm_params: AlgorithmParams
    draft_params: DraftParams
    pool_params: PoolParams
    config: dict[str, dict[str, Any]]


@dataclass(frozen=True)
class VisitContext:
    """Display context for the current dreamscape visit."""

    dreamscape_name: str
    dreamscape_number: int


def visit_site(
    site: Site,
    quest_state: QuestState,
    data: SiteData,
    logger: Optional[SessionLogger],
    context: VisitContext,
) -> None:
    """Dispatch a site visit to the appropriate handler.

    Routes based on site.site_type, passes site.is_enhanced to
    handlers that support it, and marks the site as visited after
    the handler completes.
    """
    site_type = site.site_type
    enhanced = site.is_enhanced
    name = context.dreamscape_name
    number = context.dreamscape_number

    if site_type == SiteType.DRAFT:
        sites_draft.run_draft(
            state=quest_state,
            algorithm_params=data.algorithm_params,
            draft_params=data.draft_params,
            pool_params=data.pool_params,
            dreamscape_name=name,
            dreamscape_number=number,
            logger=logger,
            is_enhanced=enhanced,
        )

    elif site_type == SiteType.DREAMCALLER_DRAFT:
        sites_dreamcaller.run_dreamcaller_draft(
            state=quest_state,
            all_dreamcallers=data.dreamcallers,
            logger=logger,
            dreamscape_name=name,
            dreamscape_number=number,
        )

    elif site_type == SiteType.DISCOVERY_DRAFT:
        tag_config: dict[str, float] = {
            k: float(v) for k, v in data.config.get("tags", {}).items()
        }
        sites_discovery.run_discovery_draft(
            state=quest_state,
            params=data.algorithm_params,
            logger=logger,
            dreamscape_name=name,
            dreamscape_number=number,
            is_enhanced=enhanced,
            cards_per_pick=data.draft_params.cards_per_pick,
            picks_per_site=data.draft_params.picks_per_site,
            tag_config=tag_config,
        )

    elif site_type == SiteType.SHOP:
        shop_config: dict[str, Any] = dict(data.config.get("shop", {}))
        sites_shop.run_shop(
            state=quest_state,
            algorithm_params=data.algorithm_params,
            pool_params=data.pool_params,
            shop_config=shop_config,
            dreamscape_name=name,
            dreamscape_number=number,
            logger=logger,
            is_enhanced=enhanced,
        )

    elif site_type == SiteType.SPECIALTY_SHOP:
        shop_config_ss: dict[str, int] = {
            k: int(v) for k, v in data.config.get("shop", {}).items()
        }
        tag_config_ss: dict[str, float] = {
            k: float(v) for k, v in data.config.get("tags", {}).items()
        }
        sites_discovery.run_specialty_shop(
            state=quest_state,
            params=data.algorithm_params,
            logger=logger,
            dreamscape_name=name,
            dreamscape_number=number,
            is_enhanced=enhanced,
            shop_config=shop_config_ss,
            tag_config=tag_config_ss,
        )

    elif site_type == SiteType.DREAMSIGN_OFFERING:
        sites_dreamsign.run_dreamsign_offering(
            state=quest_state,
            all_dreamsigns=data.dreamsigns,
            logger=logger,
            dreamscape_name=name,
            dreamscape_number=number,
            is_enhanced=enhanced,
        )

    elif site_type == SiteType.DREAMSIGN_DRAFT:
        sites_dreamsign.run_dreamsign_draft(
            state=quest_state,
            all_dreamsigns=data.dreamsigns,
            logger=logger,
            dreamscape_name=name,
            dreamscape_number=number,
            is_enhanced=enhanced,
        )

    elif site_type == SiteType.DREAM_JOURNEY:
        sites_journey.run_dream_journey(
            state=quest_state,
            all_journeys=data.journeys,
            all_dreamsigns=data.dreamsigns,
            algorithm_params=data.algorithm_params,
            pool_params=data.pool_params,
            dreamscape_name=name,
            dreamscape_number=number,
            logger=logger,
            is_enhanced=enhanced,
        )

    elif site_type == SiteType.TEMPTING_OFFER:
        sites_journey.run_tempting_offer(
            state=quest_state,
            all_offers=data.offers,
            all_banes=data.banes,
            all_dreamsigns=data.dreamsigns,
            algorithm_params=data.algorithm_params,
            pool_params=data.pool_params,
            dreamscape_name=name,
            dreamscape_number=number,
            logger=logger,
            is_enhanced=enhanced,
        )

    elif site_type == SiteType.PURGE:
        sites_purge.run_purge(
            state=quest_state,
            dreamscape_name=name,
            dreamscape_number=number,
            logger=logger,
            is_enhanced=enhanced,
        )

    elif site_type == SiteType.ESSENCE:
        essence_config: dict[str, int] = {
            k: int(v) for k, v in data.config.get("essence_sites", {}).items()
        }
        sites_essence.run_essence(
            state=quest_state,
            essence_config=essence_config,
            dreamscape_name=name,
            dreamscape_number=number,
            logger=logger,
            is_enhanced=enhanced,
        )

    elif site_type == SiteType.TRANSFIGURATION:
        sites_transfig.run_transfiguration(
            state=quest_state,
            dreamscape_name=name,
            dreamscape_number=number,
            logger=logger,
            is_enhanced=enhanced,
        )

    elif site_type == SiteType.DUPLICATION:
        sites_misc.run_duplication(
            state=quest_state,
            dreamscape_name=name,
            dreamscape_number=number,
            logger=logger,
            is_enhanced=enhanced,
        )

    elif site_type == SiteType.REWARD_SITE:
        sites_misc.run_reward(
            state=quest_state,
            dreamscape_name=name,
            dreamscape_number=number,
            logger=logger,
            all_dreamsigns=data.dreamsigns,
        )

    elif site_type == SiteType.CLEANSE:
        sites_misc.run_cleanse(
            state=quest_state,
            dreamscape_name=name,
            dreamscape_number=number,
            logger=logger,
        )

    elif site_type == SiteType.BATTLE:
        battle_config: dict[str, int] = {
            k: int(v)
            for k, v in data.config.get("battle_rewards", {}).items()
        }
        quest_config: dict[str, int] = {
            k: int(v)
            for k, v in data.config.get("quest", {}).items()
        }
        sites_battle.run_battle(
            state=quest_state,
            battle_config=battle_config,
            quest_config=quest_config,
            algorithm_params=data.algorithm_params,
            bosses=data.bosses,
            dreamscape_name=name,
            dreamscape_number=number,
            logger=logger,
        )

    site.is_visited = True
