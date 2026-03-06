"""Data models for the quest simulator.

Defines all enums, frozen and mutable dataclasses used throughout the
quest simulator. This is the leaf dependency that all other modules
import from. Stdlib-only, no external dependencies.
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Optional


class Biome(Enum):
    VERDANT = "Verdant"
    CELESTIAL = "Celestial"
    TWILIGHT = "Twilight"
    INFERNAL = "Infernal"
    ASHEN = "Ashen"
    CRYSTALLINE = "Crystalline"
    PRISMATIC = "Prismatic"
    MIRRORED = "Mirrored"
    ARCANE = "Arcane"


class NodeState(Enum):
    COMPLETED = "Completed"
    AVAILABLE = "Available"
    UNAVAILABLE = "Unavailable"


class SiteType(Enum):
    BATTLE = "Battle"
    DRAFT = "Draft"
    DREAMCALLER_DRAFT = "DreamcallerDraft"
    DISCOVERY_DRAFT = "DiscoveryDraft"
    SHOP = "Shop"
    SPECIALTY_SHOP = "SpecialtyShop"
    DREAMSIGN_OFFERING = "DreamsignOffering"
    DREAMSIGN_DRAFT = "DreamsignDraft"
    DREAM_JOURNEY = "DreamJourney"
    TEMPTING_OFFER = "TemptingOffer"
    PURGE = "Purge"
    ESSENCE = "Essence"
    TRANSFIGURATION = "Transfiguration"
    DUPLICATION = "Duplication"
    REWARD_SITE = "RewardSite"
    CLEANSE = "Cleanse"


class EffectType(Enum):
    ADD_CARDS = "add_cards"
    ADD_ESSENCE = "add_essence"
    REMOVE_CARDS = "remove_cards"
    ADD_DREAMSIGN = "add_dreamsign"
    LARGE_ESSENCE = "large_essence"
    LOSE_ESSENCE = "lose_essence"
    ADD_BANE_CARD = "add_bane_card"
    ADD_BANE_DREAMSIGN = "add_bane_dreamsign"


@dataclass(frozen=True)
class Dreamcaller:
    name: str
    essence_bonus: int
    ability_text: str


@dataclass(frozen=True)
class Dreamsign:
    name: str
    effect_text: str
    is_bane: bool


@dataclass(frozen=True)
class Journey:
    name: str
    description: str
    effect_type: EffectType
    effect_value: int


@dataclass(frozen=True)
class TemptingOffer:
    reward_name: str
    reward_description: str
    reward_effect_type: EffectType
    reward_value: int
    cost_name: str
    cost_description: str
    cost_effect_type: EffectType
    cost_value: int


@dataclass(frozen=True)
class BaneCard:
    name: str
    rules_text: str
    card_type: str
    energy_cost: int


@dataclass(frozen=True)
class Boss:
    name: str
    archetype: str
    ability_text: str
    deck_description: str
    is_final: bool


@dataclass
class DeckCard:
    """A card in the player's deck, wrapping a draft CardInstance."""

    instance: Any
    is_transfigured: bool = False
    is_bane: bool = False
    transfig_note: Optional[str] = None


@dataclass
class Site:
    site_type: SiteType
    is_enhanced: bool = False
    is_visited: bool = False


@dataclass
class DreamscapeNode:
    node_id: int
    name: str
    biome: Biome
    sites: list[Site]
    state: NodeState
    adjacent: list[int]
