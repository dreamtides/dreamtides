"""Data models for the draft simulator.

Defines all enums, frozen dataclasses, and type aliases used throughout
the draft simulator. This is the leaf dependency that all other modules
import from. Stdlib-only, no external dependencies.
"""

from dataclasses import dataclass
from enum import Enum


class PackGenerationStrategy(Enum):
    UNIFORM = "Uniform"
    RARITY_WEIGHTED = "Rarity Weighted"
    SEEDED_THEMED = "Seeded Themed"


class RefillStrategy(Enum):
    NO_REFILL = "No Refill"
    UNIFORM_REFILL = "Uniform Refill"
    CONSTRAINED_REFILL = "Constrained Refill"


class ShowNStrategy(Enum):
    UNIFORM = "Uniform"
    POWER_BIASED = "Power Biased"
    CURATED = "Curated"
    SIGNAL_RICH = "Signal Rich"
    TOP_SCORED = "Top Scored"


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


class CardSource(Enum):
    SYNTHETIC = "Synthetic"
    FILE = "File"


@dataclass(frozen=True)
class CardDesign:
    """A card design template with archetype fitness vector and attributes."""

    card_id: str
    name: str
    fitness: list[float]
    power: float
    commit: float
    flex: float


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
