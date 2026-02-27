"""Data models for the draft resonance simulation."""

from dataclasses import dataclass, field
from enum import Enum
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


class Strategy(Enum):
    SYNERGY = "synergy"
    POWER_CHASER = "power_chaser"
    RIGID = "rigid"


@dataclass(frozen=True)
class AlgorithmParams:
    exponent: float = 1.4
    floor_weight: float = 0.5
    neutral_base: float = 3.0
    staleness_factor: float = 0.3


@dataclass(frozen=True)
class PoolParams:
    copies_common: int = 4
    copies_uncommon: int = 3
    copies_rare: int = 2
    copies_legendary: int = 1
    variance_min: float = 0.75
    variance_max: float = 1.25


@dataclass(frozen=True)
class QuestParams:
    dreamcaller_bonus: int = 4
    mono_dreamcaller: bool = False


@dataclass(frozen=True)
class StrategyParams:
    strategy: Strategy = Strategy.SYNERGY
    power_weight: float = 1.0
    fit_weight: float = 1.5


@dataclass(frozen=True)
class SimCard:
    id: int
    resonances: frozenset  # frozenset[Resonance]
    rarity: Rarity
    power: int


@dataclass
class PoolEntry:
    card: SimCard
    staleness: int = 0


class ResonanceProfile:
    def __init__(self):
        self.counts: dict[Resonance, int] = {r: 0 for r in Resonance}

    def add(self, resonance: Resonance, amount: int = 1):
        self.counts[resonance] += amount

    def total(self) -> int:
        return sum(self.counts.values())

    def total_resonance_bearing(self) -> int:
        """Total count of resonance symbols from non-neutral cards."""
        return self.total()

    def top_n(self, n: int) -> list[tuple[Resonance, int]]:
        """Return top-n resonances by count, descending."""
        return sorted(self.counts.items(), key=lambda x: x[1], reverse=True)[:n]

    def top2_share(self) -> float:
        """Fraction of resonance symbols in the top 2 resonances."""
        total = self.total()
        if total == 0:
            return 0.0
        top2_total = sum(c for _, c in self.top_n(2))
        return top2_total / total

    def hhi(self) -> float:
        """Herfindahl-Hirschman Index. ~1.0=mono, ~0.5=dual, ~0.2=scattered."""
        total = self.total()
        if total == 0:
            return 0.0
        return sum((c / total) ** 2 for c in self.counts.values())

    def effective_colors(self) -> float:
        """Inverse HHI: effective number of colors."""
        h = self.hhi()
        return 1.0 / h if h > 0 else 0.0

    def snapshot(self) -> dict[Resonance, int]:
        return dict(self.counts)

    def copy(self) -> "ResonanceProfile":
        p = ResonanceProfile()
        p.counts = dict(self.counts)
        return p


@dataclass
class PickRecord:
    pick_number: int
    offered: list[SimCard]
    weights: list[float]
    picked: SimCard
    pick_reason: str
    profile_after: dict[Resonance, int]


@dataclass
class QuestResult:
    picks: list[PickRecord]
    final_profile: ResonanceProfile
    deck: list[SimCard]
    dreamcaller_resonances: frozenset  # frozenset[Resonance]
