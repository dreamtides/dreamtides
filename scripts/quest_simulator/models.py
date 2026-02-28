"""Data models for the quest simulator.

Defines all enums, frozen and mutable dataclasses, and profile classes
used throughout the quest simulator. This is the leaf dependency that
all other modules import from. Stdlib-only, no external dependencies.
"""

import random
from dataclasses import dataclass, field
from enum import Enum
from types import MappingProxyType
from typing import Optional


class Resonance(Enum):
    TIDE = "Tide"
    EMBER = "Ember"
    ZEPHYR = "Zephyr"
    STONE = "Stone"
    RUIN = "Ruin"


class Rarity(Enum):
    COMMON = "Common"
    UNCOMMON = "Uncommon"
    RARE = "Rare"
    LEGENDARY = "Legendary"


class CardType(Enum):
    CHARACTER = "Character"
    EVENT = "Event"


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
    GAIN_RESONANCE = "gain_resonance"
    LARGE_ESSENCE = "large_essence"
    LOSE_ESSENCE = "lose_essence"
    ADD_BANE_CARD = "add_bane_card"
    ADD_BANE_DREAMSIGN = "add_bane_dreamsign"


@dataclass(frozen=True)
class Card:
    name: str
    card_number: int
    energy_cost: Optional[int]
    card_type: CardType
    subtype: Optional[str]
    is_fast: bool
    spark: Optional[int]
    rarity: Rarity
    rules_text: str
    resonances: frozenset[Resonance]
    tags: frozenset[str]


@dataclass(frozen=True)
class AlgorithmParams:
    exponent: float
    floor_weight: float
    neutral_base: float
    staleness_factor: float


@dataclass(frozen=True)
class DraftParams:
    cards_per_pick: int
    picks_per_site: int


@dataclass(frozen=True)
class PoolParams:
    copies_common: int
    copies_uncommon: int
    copies_rare: int
    copies_legendary: int
    variance_min: float
    variance_max: float


@dataclass(frozen=True)
class Dreamcaller:
    name: str
    resonances: frozenset[Resonance]
    resonance_bonus: MappingProxyType[str, int]
    tags: frozenset[str]
    tag_bonus: MappingProxyType[str, int]
    essence_bonus: int
    ability_text: str


@dataclass(frozen=True)
class Dreamsign:
    name: str
    resonance: Resonance
    tags: frozenset[str]
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
    card_type: CardType
    energy_cost: int


@dataclass(frozen=True)
class Boss:
    name: str
    archetype: str
    ability_text: str
    deck_description: str
    is_final: bool
    resonances: frozenset[Resonance]


@dataclass
class DeckCard:
    card: Card
    is_transfigured: bool = False
    is_bane: bool = False
    transfig_note: Optional[str] = None


@dataclass
class PoolEntry:
    card: Card
    staleness: int = 0


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


class ResonanceProfile:
    """Tracks counts per resonance across the player's collection."""

    def __init__(self) -> None:
        self.counts: dict[Resonance, int] = {r: 0 for r in Resonance}

    def add(self, resonance: Resonance, amount: int = 1) -> None:
        self.counts[resonance] += amount

    def remove(self, resonance: Resonance, amount: int = 1) -> None:
        self.counts[resonance] = max(0, self.counts[resonance] - amount)

    def total(self) -> int:
        return sum(self.counts.values())

    def top_n(
        self, n: int, rng: Optional[random.Random] = None
    ) -> list[tuple[Resonance, int]]:
        """Return top-n resonances by count, descending. Ties broken randomly."""
        items = list(self.counts.items())
        (rng or random).shuffle(items)
        return sorted(items, key=lambda x: x[1], reverse=True)[:n]

    def snapshot(self) -> dict[Resonance, int]:
        return dict(self.counts)

    def copy(self) -> "ResonanceProfile":
        p = ResonanceProfile()
        p.counts = dict(self.counts)
        return p


class TagProfile:
    """Tracks counts per tag string across the player's collection."""

    def __init__(self) -> None:
        self.counts: dict[str, int] = {}

    def add(self, tag: str, amount: int = 1) -> None:
        self.counts[tag] = self.counts.get(tag, 0) + amount

    def remove(self, tag: str, amount: int = 1) -> None:
        current = self.counts.get(tag, 0)
        new_val = max(0, current - amount)
        if new_val == 0:
            self.counts.pop(tag, None)
        else:
            self.counts[tag] = new_val

    def snapshot(self) -> dict[str, int]:
        return dict(self.counts)

    def copy(self) -> "TagProfile":
        p = TagProfile()
        p.counts = dict(self.counts)
        return p
