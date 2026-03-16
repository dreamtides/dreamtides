"""Data models for the draft simulator v2.

Defines all enums, frozen dataclasses, and type aliases used throughout
the draft simulator. This is the leaf dependency that all other modules
import from. Stdlib-only, no external dependencies.

v2 changes: removed CardSource, RefillStrategy, power_biased ShowNStrategy.
Added rarity_value to CardDesign. Renamed power_biased -> rarity_biased.
"""

from dataclasses import dataclass
from enum import Enum


class PackGenerationStrategy(Enum):
    UNIFORM = "Uniform"
    RARITY_WEIGHTED = "Rarity Weighted"
    SEEDED_THEMED = "Seeded Themed"


class ShowNStrategy(Enum):
    UNIFORM = "Uniform"
    RARITY_BIASED = "Rarity Biased"
    CURATED = "Curated"
    SIGNAL_RICH = "Signal Rich"
    TOP_SCORED = "Top Scored"
    PLAN_PLUS_POWER = "Plan Plus Power"


class PickPolicy(Enum):
    GREEDY = "Greedy"
    ARCHETYPE_LOYAL = "Archetype Loyal"
    FORCE = "Force"
    ADAPTIVE = "Adaptive"
    SIGNAL_IGNORANT = "Signal Ignorant"


class CubeConsumptionMode(Enum):
    WITH_REPLACEMENT = "With Replacement"
    WITHOUT_REPLACEMENT = "Without Replacement"


class SeedingPolicy(Enum):
    SEQUENTIAL = "Sequential"
    HASHED = "Hashed"


RARITY_VALUES: dict[str, float] = {
    "common": 0.0,
    "uncommon": 0.33,
    "rare": 0.67,
}


@dataclass(frozen=True)
class CardDesign:
    """A card design template with archetype fitness vector and attributes."""

    card_id: str
    name: str
    fitness: list[float]
    rarity: str = ""
    rarity_value: float = 0.0
    rules_text: str = ""
    energy_cost: int | None = None
    card_type: str = ""
    subtype: str = ""
    spark: int | None = None
    is_fast: bool = False
    image_number: int | None = None
    resonance: tuple[str, ...] = ()
    original_rarity: str = ""
    w1_rank: int = 0


@dataclass(frozen=True)
class CardInstance:
    """One physical copy of a card in the cube, identified by instance_id."""

    instance_id: int
    design: CardDesign


@dataclass(frozen=True)
class Pack:
    """A pack of card instances with an archetype profile."""

    pack_id: str
    cards: list[CardInstance]
    archetype_profile: list[float]
