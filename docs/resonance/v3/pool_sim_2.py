#!/usr/bin/env python3
"""
Rarity x Lane Locking Simulation

Investigates how rarity should interact with the Lane Locking draft algorithm.

Tests 5 rarity models:
  A: Flat (cosmetic rarity, uniform power)
  B: Standard TCG (180C/100U/60R/20L, power scales with rarity)
  C: Roguelike-skewed (120C/120U/80R/40L, power scales)
  D: Rarity-symbol correlation (rares have 2-3 symbols, commons have 1)
  E: Inverse correlation (rares have 1 focused symbol, commons have 2-3)

All models use Lane Locking at thresholds (3, 8), primary=2, secondary/tertiary=1.
"""

import random
import statistics
from collections import Counter
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ─── Constants ───────────────────────────────────────────────────────────────

NUM_CARDS = 360
NUM_GENERIC = 36
NUM_ARCHETYPE_CARDS = NUM_CARDS - NUM_GENERIC  # 324
NUM_ARCHETYPES = 8
CARDS_PER_ARCHETYPE = NUM_ARCHETYPE_CARDS // NUM_ARCHETYPES  # 40 (remainder distributed)
PACK_SIZE = 4
NUM_PICKS = 30
NUM_SIMULATIONS = 1000

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    ("Flash",        "Zephyr", "Ember"),
    ("Blink",        "Ember",  "Zephyr"),
    ("Storm",        "Ember",  "Stone"),
    ("Self-Discard", "Stone",  "Ember"),
    ("Self-Mill",    "Stone",  "Tide"),
    ("Sacrifice",    "Tide",   "Stone"),
    ("Warriors",     "Tide",   "Zephyr"),
    ("Ramp",         "Zephyr", "Tide"),
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]


# ─── Data Classes ────────────────────────────────────────────────────────────

class Rarity(Enum):
    COMMON = 0
    UNCOMMON = 1
    RARE = 2
    LEGENDARY = 3


RARITY_NAMES = {Rarity.COMMON: "C", Rarity.UNCOMMON: "U",
                Rarity.RARE: "R", Rarity.LEGENDARY: "L"}


@dataclass
class SimCard:
    id: int
    symbols: list
    archetype: Optional[str]
    archetype_fitness: dict
    rarity: Rarity
    power: float

    @property
    def primary_resonance(self) -> Optional[str]:
        return self.symbols[0] if self.symbols else None


# ─── Adjacency & Fitness ────────────────────────────────────────────────────

def get_archetype_index(name: str) -> int:
    for i, (n, _, _) in enumerate(ARCHETYPES):
        if n == name:
            return i
    raise ValueError(f"Unknown archetype: {name}")


def circle_distance(i: int, j: int) -> int:
    d = abs(i - j)
    return min(d, NUM_ARCHETYPES - d)


def are_adjacent(i: int, j: int) -> bool:
    return circle_distance(i, j) == 1


def compute_fitness(card_archetype: Optional[str]) -> dict:
    """Compute fitness tiers following the archetype circle model."""
    fitness = {}
    if card_archetype is None:
        for name, _, _ in ARCHETYPES:
            fitness[name] = "B"
        return fitness

    home_idx = get_archetype_index(card_archetype)
    home_primary = ARCHETYPES[home_idx][1]
    home_secondary = ARCHETYPES[home_idx][2]

    for i, (name, arch_pri, arch_sec) in enumerate(ARCHETYPES):
        if name == card_archetype:
            fitness[name] = "S"
        elif are_adjacent(home_idx, i) and home_primary in (arch_pri, arch_sec):
            fitness[name] = "A"
        elif home_secondary in (arch_pri, arch_sec) and name not in fitness:
            fitness[name] = "B"
        elif home_primary in (arch_pri, arch_sec):
            fitness[name] = "C"
        elif home_secondary in (arch_pri, arch_sec):
            fitness[name] = "B"
        else:
            fitness[name] = "F"

    return fitness


# ─── Power Scaling by Rarity ────────────────────────────────────────────────

def flat_power(_rarity: Rarity) -> float:
    """Model A: All cards have similar power regardless of rarity."""
    return random.uniform(4.5, 5.5)


def scaled_power(rarity: Rarity) -> float:
    """Models B, C: Power scales with rarity."""
    ranges = {
        Rarity.COMMON:    (2.0, 5.0),
        Rarity.UNCOMMON:  (4.0, 7.0),
        Rarity.RARE:      (6.0, 9.0),
        Rarity.LEGENDARY: (8.0, 10.0),
    }
    lo, hi = ranges[rarity]
    return random.uniform(lo, hi)


# ─── Symbol Generation ──────────────────────────────────────────────────────

def make_symbols(num_symbols: int, primary: str, secondary: str) -> list:
    """Generate a symbol list for a card in an archetype."""
    if num_symbols == 1:
        return [primary] if random.random() < 0.80 else [secondary]
    elif num_symbols == 2:
        r = random.random()
        if r < 0.55:
            return [primary, secondary]
        elif r < 0.80:
            return [primary, primary]
        elif r < 0.95:
            return [secondary, primary]
        else:
            return [secondary, secondary]
    else:  # 3
        r = random.random()
        if r < 0.45:
            return [primary, primary, secondary]
        elif r < 0.75:
            return [primary, secondary, primary]
        elif r < 0.90:
            return [primary, secondary, secondary]
        else:
            return [secondary, primary, secondary]


def symbol_count_for_card_default(card_index: int, n_cards: int) -> int:
    """Default distribution: 25% 1-sym, 55% 2-sym, 20% 3-sym."""
    n1 = round(n_cards * 0.25)
    n2 = round(n_cards * 0.55)
    if card_index < n1:
        return 1
    elif card_index < n1 + n2:
        return 2
    else:
        return 3


# ─── Card Pool Generators (one per rarity model) ────────────────────────────

def generate_pool_model_a() -> list:
    """
    Model A: Flat rarity. All cards ~equal power. Rarity is cosmetic only.
    Distribution: 180C/100U/60R/20L (standard ratios) but power is uniform.
    """
    cards = []
    card_id = 0
    rarity_pool = (
        [Rarity.COMMON] * 180 + [Rarity.UNCOMMON] * 100 +
        [Rarity.RARE] * 60 + [Rarity.LEGENDARY] * 20
    )
    random.shuffle(rarity_pool)
    rarity_idx = 0

    cards_per = distribute_cards_per_archetype()

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        n_cards = cards_per[arch_idx]
        for i in range(n_cards):
            num_sym = symbol_count_for_card_default(i, n_cards)
            symbols = make_symbols(num_sym, primary, secondary)
            rarity = rarity_pool[rarity_idx] if rarity_idx < len(rarity_pool) else Rarity.COMMON
            rarity_idx += 1
            cards.append(SimCard(
                id=card_id, symbols=symbols, archetype=arch_name,
                archetype_fitness=compute_fitness(arch_name),
                rarity=rarity, power=flat_power(rarity),
            ))
            card_id += 1

    for _ in range(NUM_GENERIC):
        rarity = rarity_pool[rarity_idx] if rarity_idx < len(rarity_pool) else Rarity.COMMON
        rarity_idx += 1
        cards.append(SimCard(
            id=card_id, symbols=[], archetype=None,
            archetype_fitness=compute_fitness(None),
            rarity=rarity, power=flat_power(rarity),
        ))
        card_id += 1

    random.shuffle(cards)
    return cards


def generate_pool_model_b() -> list:
    """
    Model B: Standard TCG. 180C/100U/60R/20L. Power scales with rarity.
    Symbol distribution: default 25/55/20, independent of rarity.
    """
    return _generate_pool_with_rarity_counts(
        {Rarity.COMMON: 180, Rarity.UNCOMMON: 100, Rarity.RARE: 60, Rarity.LEGENDARY: 20},
        power_fn=scaled_power, symbol_fn=symbol_count_for_card_default,
    )


def generate_pool_model_c() -> list:
    """
    Model C: Roguelike-skewed. 120C/120U/80R/40L. More rares & legendaries.
    Power scales with rarity.
    """
    return _generate_pool_with_rarity_counts(
        {Rarity.COMMON: 120, Rarity.UNCOMMON: 120, Rarity.RARE: 80, Rarity.LEGENDARY: 40},
        power_fn=scaled_power, symbol_fn=symbol_count_for_card_default,
    )


def generate_pool_model_d() -> list:
    """
    Model D: Rarity-symbol correlation. Higher rarity = more symbols.
    Commons: mostly 1 symbol, Uncommons: mostly 2, Rares: 2-3, Legendaries: 3.
    Power also scales with rarity.
    Distribution: 180C/100U/60R/20L (standard).
    """
    def symbol_fn_d(rarity: Rarity) -> int:
        if rarity == Rarity.COMMON:
            return random.choices([1, 2], weights=[0.75, 0.25])[0]
        elif rarity == Rarity.UNCOMMON:
            return random.choices([1, 2, 3], weights=[0.15, 0.70, 0.15])[0]
        elif rarity == Rarity.RARE:
            return random.choices([2, 3], weights=[0.50, 0.50])[0]
        else:  # Legendary
            return random.choices([2, 3], weights=[0.30, 0.70])[0]

    return _generate_pool_rarity_symbol_correlated(
        {Rarity.COMMON: 180, Rarity.UNCOMMON: 100, Rarity.RARE: 60, Rarity.LEGENDARY: 20},
        power_fn=scaled_power, symbol_count_fn=symbol_fn_d,
    )


def generate_pool_model_e() -> list:
    """
    Model E: Inverse correlation. Higher rarity = fewer, more focused symbols.
    Commons: 2-3 symbols, Uncommons: 1-2, Rares: 1, Legendaries: 1.
    Power scales with rarity.
    Distribution: 180C/100U/60R/20L (standard).
    """
    def symbol_fn_e(rarity: Rarity) -> int:
        if rarity == Rarity.COMMON:
            return random.choices([2, 3], weights=[0.65, 0.35])[0]
        elif rarity == Rarity.UNCOMMON:
            return random.choices([1, 2], weights=[0.50, 0.50])[0]
        elif rarity == Rarity.RARE:
            return random.choices([1, 2], weights=[0.80, 0.20])[0]
        else:  # Legendary
            return 1

    return _generate_pool_rarity_symbol_correlated(
        {Rarity.COMMON: 180, Rarity.UNCOMMON: 100, Rarity.RARE: 60, Rarity.LEGENDARY: 20},
        power_fn=scaled_power, symbol_count_fn=symbol_fn_e,
    )


# ─── Pool Generation Helpers ────────────────────────────────────────────────

def distribute_cards_per_archetype() -> list:
    """Split NUM_ARCHETYPE_CARDS across 8 archetypes."""
    cards_per = [NUM_ARCHETYPE_CARDS // NUM_ARCHETYPES] * NUM_ARCHETYPES
    leftover = NUM_ARCHETYPE_CARDS - sum(cards_per)
    for i in range(leftover):
        cards_per[i] += 1
    return cards_per


def _generate_pool_with_rarity_counts(
    rarity_counts: dict, power_fn, symbol_fn,
) -> list:
    """Generate pool where rarity is assigned from a fixed distribution,
    symbols use the default distribution (independent of rarity)."""
    cards = []
    card_id = 0

    # Build rarity assignment list
    rarity_list = []
    for r, count in rarity_counts.items():
        rarity_list.extend([r] * count)
    random.shuffle(rarity_list)
    rarity_idx = 0

    cards_per = distribute_cards_per_archetype()

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        n_cards = cards_per[arch_idx]
        for i in range(n_cards):
            num_sym = symbol_fn(i, n_cards)
            symbols = make_symbols(num_sym, primary, secondary)
            rarity = rarity_list[rarity_idx] if rarity_idx < len(rarity_list) else Rarity.COMMON
            rarity_idx += 1
            cards.append(SimCard(
                id=card_id, symbols=symbols, archetype=arch_name,
                archetype_fitness=compute_fitness(arch_name),
                rarity=rarity, power=power_fn(rarity),
            ))
            card_id += 1

    for _ in range(NUM_GENERIC):
        rarity = rarity_list[rarity_idx] if rarity_idx < len(rarity_list) else Rarity.COMMON
        rarity_idx += 1
        cards.append(SimCard(
            id=card_id, symbols=[], archetype=None,
            archetype_fitness=compute_fitness(None),
            rarity=rarity, power=power_fn(rarity),
        ))
        card_id += 1

    random.shuffle(cards)
    return cards


def _generate_pool_rarity_symbol_correlated(
    rarity_counts: dict, power_fn, symbol_count_fn,
) -> list:
    """Generate pool where symbol count depends on rarity."""
    cards = []
    card_id = 0

    # For each archetype, distribute the rarity counts proportionally
    cards_per = distribute_cards_per_archetype()
    total_archetype = sum(cards_per)

    # Build per-archetype rarity lists
    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        n_cards = cards_per[arch_idx]
        # Proportional rarity assignment for this archetype
        arch_rarities = []
        for r, total_count in rarity_counts.items():
            # Remove generic allocation proportionally
            archetype_share = total_count * n_cards / NUM_CARDS
            arch_rarities.extend([r] * round(archetype_share))
        # Pad or trim to exact count
        while len(arch_rarities) < n_cards:
            arch_rarities.append(Rarity.COMMON)
        arch_rarities = arch_rarities[:n_cards]
        random.shuffle(arch_rarities)

        for i in range(n_cards):
            rarity = arch_rarities[i]
            num_sym = symbol_count_fn(rarity)
            symbols = make_symbols(num_sym, primary, secondary)
            cards.append(SimCard(
                id=card_id, symbols=symbols, archetype=arch_name,
                archetype_fitness=compute_fitness(arch_name),
                rarity=rarity, power=power_fn(rarity),
            ))
            card_id += 1

    # Generic cards: distribute rarities proportionally
    generic_rarities = []
    for r, total_count in rarity_counts.items():
        generic_share = total_count * NUM_GENERIC / NUM_CARDS
        generic_rarities.extend([r] * round(generic_share))
    while len(generic_rarities) < NUM_GENERIC:
        generic_rarities.append(Rarity.COMMON)
    generic_rarities = generic_rarities[:NUM_GENERIC]
    random.shuffle(generic_rarities)

    for i in range(NUM_GENERIC):
        rarity = generic_rarities[i]
        cards.append(SimCard(
            id=card_id, symbols=[], archetype=None,
            archetype_fitness=compute_fitness(None),
            rarity=rarity, power=power_fn(rarity),
        ))
        card_id += 1

    random.shuffle(cards)
    return cards


# ─── Lane Locking Algorithm ─────────────────────────────────────────────────

@dataclass
class LaneLockState:
    symbol_counts: dict = field(default_factory=lambda: {r: 0 for r in RESONANCES})
    locked_slots: list = field(default_factory=list)
    thresholds_triggered: dict = field(default_factory=lambda: {r: set() for r in RESONANCES})

    def add_card(self, card: SimCard, threshold_low: int = 3, threshold_high: int = 8):
        new_locks = []
        if not card.symbols:
            return new_locks
        for i, sym in enumerate(card.symbols):
            weight = 2 if i == 0 else 1
            self.symbol_counts[sym] += weight

        for res in RESONANCES:
            count = self.symbol_counts[res]
            if len(self.locked_slots) >= PACK_SIZE:
                break
            if count >= threshold_low and "low" not in self.thresholds_triggered[res]:
                self.thresholds_triggered[res].add("low")
                self.locked_slots.append(res)
                new_locks.append((res, "low"))
                if len(self.locked_slots) >= PACK_SIZE:
                    break
            if count >= threshold_high and "high" not in self.thresholds_triggered[res]:
                self.thresholds_triggered[res].add("high")
                self.locked_slots.append(res)
                new_locks.append((res, "high"))
                if len(self.locked_slots) >= PACK_SIZE:
                    break
        return new_locks


def build_pack(pool: list, state: LaneLockState) -> list:
    """Build a 4-card pack using Lane Locking. No rarity manipulation."""
    pack = []
    used_ids = set()

    for res in state.locked_slots[:PACK_SIZE]:
        matching = [c for c in pool if c.primary_resonance == res and c.id not in used_ids]
        if matching:
            chosen = random.choice(matching)
            pack.append(chosen)
            used_ids.add(chosen.id)
        else:
            available = [c for c in pool if c.id not in used_ids]
            if available:
                chosen = random.choice(available)
                pack.append(chosen)
                used_ids.add(chosen.id)

    num_open = PACK_SIZE - len(pack)
    if num_open > 0:
        available = [c for c in pool if c.id not in used_ids]
        if len(available) >= num_open:
            chosen = random.sample(available, num_open)
            pack.extend(chosen)
        else:
            pack.extend(available)

    return pack


# ─── Player Strategy ────────────────────────────────────────────────────────

TIER_VALUES = {"S": 5, "A": 4, "B": 3, "C": 1, "F": 0}


def evaluate_archetype_strength(drafted: list) -> dict:
    scores = {name: 0 for name in ARCHETYPE_NAMES}
    for card in drafted:
        for arch, tier in card.archetype_fitness.items():
            scores[arch] += TIER_VALUES[tier]
    return scores


def card_fits_archetype(card: SimCard, arch_name: str) -> bool:
    return card.archetype_fitness.get(arch_name, "F") in ("S", "A")


def card_is_off_archetype(card: SimCard, arch_name: str) -> bool:
    return card.archetype_fitness.get(arch_name, "F") in ("C", "F")


def pick_archetype_committed(pack: list, drafted: list,
                             committed_arch: Optional[str]) -> tuple:
    """Pick best card for committed archetype. Commits at pick 5."""
    if committed_arch is None and len(drafted) >= 5:
        scores = evaluate_archetype_strength(drafted)
        committed_arch = max(scores, key=scores.get)

    if committed_arch is None:
        if drafted:
            scores = evaluate_archetype_strength(drafted)
            best_arch = max(scores, key=scores.get)
        else:
            best_arch = random.choice(ARCHETYPE_NAMES)
        best_card = max(pack, key=lambda c: (
            TIER_VALUES.get(c.archetype_fitness.get(best_arch, "F"), 0), c.power))
    else:
        best_card = max(pack, key=lambda c: (
            TIER_VALUES.get(c.archetype_fitness.get(committed_arch, "F"), 0), c.power))

    return best_card, committed_arch


def pick_power_chaser(pack: list, drafted: list) -> SimCard:
    """Always picks highest raw power."""
    return max(pack, key=lambda c: c.power)


# ─── Evaluation Helpers ─────────────────────────────────────────────────────

def get_unique_archetypes_with_sa(pack: list) -> set:
    archetypes_present = set()
    for card in pack:
        for arch_name in ARCHETYPE_NAMES:
            if card.archetype_fitness.get(arch_name, "F") in ("S", "A"):
                archetypes_present.add(arch_name)
    return archetypes_present


# ─── Simulation Core ────────────────────────────────────────────────────────

@dataclass
class DraftMetrics:
    early_unique_archetypes_per_pack: list = field(default_factory=list)
    early_archetype_fit_per_pack: list = field(default_factory=list)
    late_archetype_fit_per_pack: list = field(default_factory=list)
    late_off_archetype_per_pack: list = field(default_factory=list)
    convergence_pick: Optional[int] = None
    deck_concentration: float = 0.0
    drafted_card_ids: list = field(default_factory=list)
    # Rarity-specific metrics
    total_power: float = 0.0
    rarity_counts_drafted: dict = field(default_factory=lambda: {r: 0 for r in Rarity})
    rare_in_locked_slot: int = 0
    rare_in_open_slot: int = 0
    # Draft tension: how often did player face a high-power off-archetype vs low-power on-archetype choice?
    tension_moments: int = 0
    total_packs_after_commit: int = 0


def run_single_draft(pool: list, strategy: str = "archetype_committed") -> DraftMetrics:
    state = LaneLockState()
    drafted = []
    metrics = DraftMetrics()
    committed_arch = None
    convergence_found = False

    for pick_num in range(NUM_PICKS):
        pack = build_pack(pool, state)

        if drafted:
            scores = evaluate_archetype_strength(drafted)
            current_best_arch = max(scores, key=scores.get)
        else:
            current_best_arch = None

        num_locked = min(len(state.locked_slots), PACK_SIZE)

        # Early metrics
        if pick_num < 5:
            unique_archs = get_unique_archetypes_with_sa(pack)
            metrics.early_unique_archetypes_per_pack.append(len(unique_archs))
            if current_best_arch:
                fit_count = sum(1 for c in pack
                                if card_fits_archetype(c, current_best_arch))
                metrics.early_archetype_fit_per_pack.append(fit_count)

        # Late metrics
        if pick_num >= 5 and current_best_arch:
            fit_count = sum(1 for c in pack
                            if card_fits_archetype(c, current_best_arch))
            metrics.late_archetype_fit_per_pack.append(fit_count)
            off_count = sum(1 for c in pack
                            if card_is_off_archetype(c, current_best_arch))
            metrics.late_off_archetype_per_pack.append(off_count)
            if not convergence_found and fit_count >= 2:
                metrics.convergence_pick = pick_num + 1
                convergence_found = True

        # Rarity tracking per slot
        for slot_idx, c in enumerate(pack):
            is_locked_slot = slot_idx < num_locked
            if c.rarity in (Rarity.RARE, Rarity.LEGENDARY):
                if is_locked_slot:
                    metrics.rare_in_locked_slot += 1
                else:
                    metrics.rare_in_open_slot += 1

        # Draft tension detection (after commitment)
        if current_best_arch and pick_num >= 5:
            metrics.total_packs_after_commit += 1
            on_arch = [c for c in pack if card_fits_archetype(c, current_best_arch)]
            off_arch = [c for c in pack if not card_fits_archetype(c, current_best_arch)]
            if on_arch and off_arch:
                best_on_power = max(c.power for c in on_arch)
                best_off_power = max(c.power for c in off_arch)
                # Tension: off-archetype card is 2+ power higher than best on-archetype
                if best_off_power - best_on_power >= 2.0:
                    metrics.tension_moments += 1

        # Pick
        if strategy == "archetype_committed":
            chosen, committed_arch = pick_archetype_committed(
                pack, drafted, committed_arch)
        elif strategy == "power_chaser":
            chosen = pick_power_chaser(pack, drafted)
        else:
            raise ValueError(f"Unknown strategy: {strategy}")

        state.add_card(chosen)
        drafted.append(chosen)
        metrics.drafted_card_ids.append(chosen.id)
        metrics.total_power += chosen.power
        metrics.rarity_counts_drafted[chosen.rarity] += 1

    # Deck concentration
    if drafted:
        scores = evaluate_archetype_strength(drafted)
        best_arch = max(scores, key=scores.get)
        sa_count = sum(1 for c in drafted if card_fits_archetype(c, best_arch))
        metrics.deck_concentration = sa_count / len(drafted)

    return metrics


def run_model_simulation(pool: list, n: int = NUM_SIMULATIONS) -> dict:
    """Run simulations for both strategies."""
    results = {}
    for strategy in ["archetype_committed", "power_chaser"]:
        all_metrics = []
        for _ in range(n):
            m = run_single_draft(pool, strategy)
            all_metrics.append(m)
        results[strategy] = all_metrics
    return results


def compute_aggregate(results: dict) -> dict:
    agg = {}
    for strategy, all_metrics in results.items():
        s = {}

        all_early_archs = [v for m in all_metrics
                           for v in m.early_unique_archetypes_per_pack]
        s["early_unique_archs"] = (statistics.mean(all_early_archs)
                                   if all_early_archs else 0)

        all_early_fit = [v for m in all_metrics
                         for v in m.early_archetype_fit_per_pack]
        s["early_arch_fit"] = (statistics.mean(all_early_fit)
                               if all_early_fit else 0)

        all_late_fit = [v for m in all_metrics
                        for v in m.late_archetype_fit_per_pack]
        s["late_arch_fit"] = (statistics.mean(all_late_fit)
                              if all_late_fit else 0)

        all_late_off = [v for m in all_metrics
                        for v in m.late_off_archetype_per_pack]
        s["late_off_arch"] = (statistics.mean(all_late_off)
                              if all_late_off else 0)

        conv_picks = [m.convergence_pick for m in all_metrics
                      if m.convergence_pick is not None]
        s["convergence_pick"] = (statistics.mean(conv_picks)
                                 if conv_picks else float("inf"))

        s["deck_concentration"] = statistics.mean(
            [m.deck_concentration for m in all_metrics])

        # Card overlap
        if len(all_metrics) >= 2:
            overlaps = []
            sample_size = min(200, len(all_metrics))
            sampled = random.sample(all_metrics, sample_size)
            for i in range(0, sample_size - 1, 2):
                s1 = set(sampled[i].drafted_card_ids)
                s2 = set(sampled[i + 1].drafted_card_ids)
                if s1 | s2:
                    overlaps.append(len(s1 & s2) / len(s1 | s2))
            s["card_overlap"] = statistics.mean(overlaps) if overlaps else 0
        else:
            s["card_overlap"] = 0

        # Power stats
        all_powers = [m.total_power / NUM_PICKS for m in all_metrics]
        s["avg_power"] = statistics.mean(all_powers)
        s["power_stdev"] = statistics.stdev(all_powers) if len(all_powers) > 1 else 0

        # Power variance between runs
        s["power_variance"] = statistics.variance(all_powers) if len(all_powers) > 1 else 0

        # Rarity in locked vs open slots
        total_rare_locked = sum(m.rare_in_locked_slot for m in all_metrics)
        total_rare_open = sum(m.rare_in_open_slot for m in all_metrics)
        total_rare = total_rare_locked + total_rare_open
        s["rare_in_locked_pct"] = (total_rare_locked / total_rare * 100
                                   if total_rare > 0 else 0)
        s["rare_in_open_pct"] = (total_rare_open / total_rare * 100
                                 if total_rare > 0 else 0)

        # Draft tension
        total_tension = sum(m.tension_moments for m in all_metrics)
        total_packs = sum(m.total_packs_after_commit for m in all_metrics)
        s["tension_rate"] = (total_tension / total_packs * 100
                             if total_packs > 0 else 0)

        # Rarity drafted breakdown
        rarity_totals = {r: 0 for r in Rarity}
        for m in all_metrics:
            for r, cnt in m.rarity_counts_drafted.items():
                rarity_totals[r] += cnt
        total_drafted = sum(rarity_totals.values())
        s["rarity_drafted"] = {
            RARITY_NAMES[r]: cnt / total_drafted * 100 if total_drafted else 0
            for r, cnt in rarity_totals.items()
        }

        agg[strategy] = s
    return agg


# ─── Archetype Frequency ────────────────────────────────────────────────────

def measure_archetype_frequency(pool: list, n: int = 1000) -> dict:
    arch_counts = Counter()
    for _ in range(n):
        m = run_single_draft(pool, "archetype_committed")
        drafted_cards = [next(c for c in pool if c.id == cid)
                         for cid in m.drafted_card_ids]
        scores = evaluate_archetype_strength(drafted_cards)
        best = max(scores, key=scores.get)
        arch_counts[best] += 1
    total = sum(arch_counts.values())
    return {k: v / total for k, v in arch_counts.items()}


# ─── Pool Statistics ────────────────────────────────────────────────────────

def pool_stats(pool: list, label: str) -> dict:
    """Compute and print pool statistics."""
    stats = {}
    rarity_counts = Counter(c.rarity for c in pool)
    stats["rarity_counts"] = {RARITY_NAMES[r]: rarity_counts.get(r, 0) for r in Rarity}
    stats["avg_power"] = statistics.mean(c.power for c in pool)
    stats["power_by_rarity"] = {}
    for r in Rarity:
        cards_of_r = [c for c in pool if c.rarity == r]
        if cards_of_r:
            stats["power_by_rarity"][RARITY_NAMES[r]] = statistics.mean(
                c.power for c in cards_of_r)

    sym_counts = Counter(len(c.symbols) for c in pool)
    stats["symbol_dist"] = dict(sym_counts)

    # Symbol counts by rarity
    stats["symbols_by_rarity"] = {}
    for r in Rarity:
        cards_of_r = [c for c in pool if c.rarity == r and c.symbols]
        if cards_of_r:
            stats["symbols_by_rarity"][RARITY_NAMES[r]] = statistics.mean(
                len(c.symbols) for c in cards_of_r)

    print(f"\n  Pool stats for {label}:")
    print(f"    Rarity: {stats['rarity_counts']}")
    print(f"    Avg power: {stats['avg_power']:.2f}")
    print(f"    Power by rarity: {', '.join(f'{k}={v:.2f}' for k, v in stats['power_by_rarity'].items())}")
    print(f"    Symbol dist: {stats['symbol_dist']}")
    print(f"    Avg symbols by rarity: {', '.join(f'{k}={v:.2f}' for k, v in stats['symbols_by_rarity'].items())}")

    return stats


# ─── Main ────────────────────────────────────────────────────────────────────

def print_model_results(model_name: str, agg: dict, arch_freq: dict):
    s = agg["archetype_committed"]
    p = agg.get("power_chaser", {})

    print(f"\n{'─'*60}")
    print(f"  {model_name}")
    print(f"{'─'*60}")

    # Standard 8 targets
    print(f"  Early unique archs w/ S/A:  {s['early_unique_archs']:.2f}  (>= 3)")
    print(f"  Early S/A for arch/pack:    {s['early_arch_fit']:.2f}  (<= 2)")
    print(f"  Late S/A for arch/pack:     {s['late_arch_fit']:.2f}  (>= 2)")
    print(f"  Late C/F cards/pack:        {s['late_off_arch']:.2f}  (>= 0.5)")
    print(f"  Convergence pick:           {s['convergence_pick']:.1f}  (5-8)")
    print(f"  Deck concentration:         {s['deck_concentration']*100:.1f}%  (60-80%)")
    print(f"  Card overlap:               {s['card_overlap']*100:.1f}%  (< 40%)")

    max_freq = max(arch_freq.values()) if arch_freq else 0
    min_freq = min(arch_freq.values()) if arch_freq else 0
    print(f"  Archetype freq range:       {min_freq*100:.1f}-{max_freq*100:.1f}%  (5-20%)")

    # Rarity-specific metrics
    print(f"\n  --- Rarity Metrics (archetype_committed) ---")
    print(f"  Avg power per card:         {s['avg_power']:.2f}")
    print(f"  Power stdev across runs:    {s['power_stdev']:.3f}")
    print(f"  Rare/Leg in locked slot:    {s['rare_in_locked_pct']:.1f}%")
    print(f"  Rare/Leg in open slot:      {s['rare_in_open_pct']:.1f}%")
    print(f"  Draft tension rate:         {s['tension_rate']:.1f}%")
    rd = s['rarity_drafted']
    print(f"  Rarity drafted (C/U/R/L):   {rd['C']:.1f}%/{rd['U']:.1f}%/{rd['R']:.1f}%/{rd['L']:.1f}%")

    if p:
        print(f"\n  --- Power Chaser ---")
        print(f"  Avg power per card:         {p['avg_power']:.2f}")
        print(f"  Deck concentration:         {p['deck_concentration']*100:.1f}%")
        print(f"  Draft tension rate:         {p['tension_rate']:.1f}%")


def count_passes(agg: dict, arch_freq: dict) -> int:
    s = agg["archetype_committed"]
    max_freq = max(arch_freq.values()) if arch_freq else 1
    min_freq = min(arch_freq.values()) if arch_freq else 0
    passes = 0
    if s["early_unique_archs"] >= 3: passes += 1
    if s["early_arch_fit"] <= 2: passes += 1
    if s["late_arch_fit"] >= 2: passes += 1
    if s["late_off_arch"] >= 0.5: passes += 1
    if 5 <= s["convergence_pick"] <= 8: passes += 1
    if 60 <= s["deck_concentration"] * 100 <= 80: passes += 1
    if s["card_overlap"] * 100 < 40: passes += 1
    if max_freq <= 0.20 and min_freq >= 0.05: passes += 1
    return passes


def main():
    random.seed(42)

    print("=" * 70)
    print("  RARITY x LANE LOCKING SIMULATION")
    print("  Testing 5 rarity models with Lane Locking (3/8 thresholds)")
    print("=" * 70)

    models = {
        "Model A: Flat Rarity (cosmetic, uniform power)": generate_pool_model_a,
        "Model B: Standard TCG (180C/100U/60R/20L, power scales)": generate_pool_model_b,
        "Model C: Roguelike-Skewed (120C/120U/80R/40L, power scales)": generate_pool_model_c,
        "Model D: Rarity-Symbol Correlation (rares=more symbols)": generate_pool_model_d,
        "Model E: Inverse Correlation (rares=fewer symbols)": generate_pool_model_e,
    }

    all_results = {}

    for model_name, gen_fn in models.items():
        print(f"\n{'='*70}")
        print(f"  Generating and simulating: {model_name}")
        print(f"{'='*70}")

        pool = gen_fn()
        pstats = pool_stats(pool, model_name)

        results = run_model_simulation(pool, NUM_SIMULATIONS)
        agg = compute_aggregate(results)

        arch_freq = measure_archetype_frequency(pool, 1000)

        print_model_results(model_name, agg, arch_freq)

        passes = count_passes(agg, arch_freq)
        print(f"\n  TARGETS PASSED: {passes}/8")

        all_results[model_name] = {
            "agg": agg,
            "arch_freq": arch_freq,
            "pool_stats": pstats,
            "passes": passes,
        }

    # ─── Summary Comparison Table ────────────────────────────────────────
    print("\n\n" + "=" * 70)
    print("  SUMMARY COMPARISON TABLE")
    print("=" * 70)

    headers = ["Metric", "Target"]
    short_names = ["A:Flat", "B:TCG", "C:Rogue", "D:SymCorr", "E:InvCorr"]
    model_keys = list(models.keys())

    print(f"\n  {'Metric':<35} {'Target':<10}", end="")
    for sn in short_names:
        print(f" {sn:>10}", end="")
    print()
    print(f"  {'-'*35} {'-'*10}", end="")
    for _ in short_names:
        print(f" {'-'*10}", end="")
    print()

    # Define metric rows
    metric_rows = [
        ("Early unique archs", ">= 3",
         lambda r: f"{r['agg']['archetype_committed']['early_unique_archs']:.2f}"),
        ("Early S/A for arch", "<= 2",
         lambda r: f"{r['agg']['archetype_committed']['early_arch_fit']:.2f}"),
        ("Late S/A for arch", ">= 2",
         lambda r: f"{r['agg']['archetype_committed']['late_arch_fit']:.2f}"),
        ("Late C/F cards", ">= 0.5",
         lambda r: f"{r['agg']['archetype_committed']['late_off_arch']:.2f}"),
        ("Convergence pick", "5-8",
         lambda r: f"{r['agg']['archetype_committed']['convergence_pick']:.1f}"),
        ("Deck concentration", "60-80%",
         lambda r: f"{r['agg']['archetype_committed']['deck_concentration']*100:.1f}%"),
        ("Card overlap", "< 40%",
         lambda r: f"{r['agg']['archetype_committed']['card_overlap']*100:.1f}%"),
        ("Archetype freq range", "5-20%",
         lambda r: f"{min(r['arch_freq'].values())*100:.0f}-{max(r['arch_freq'].values())*100:.0f}%"),
        ("Targets passed", "8/8",
         lambda r: f"{r['passes']}/8"),
        ("---", "---", lambda r: "---"),
        ("Avg power (committed)", "",
         lambda r: f"{r['agg']['archetype_committed']['avg_power']:.2f}"),
        ("Power stdev", "",
         lambda r: f"{r['agg']['archetype_committed']['power_stdev']:.3f}"),
        ("Rare in locked slot %", "",
         lambda r: f"{r['agg']['archetype_committed']['rare_in_locked_pct']:.1f}%"),
        ("Draft tension rate", "",
         lambda r: f"{r['agg']['archetype_committed']['tension_rate']:.1f}%"),
        ("Avg power (power chaser)", "",
         lambda r: f"{r['agg']['power_chaser']['avg_power']:.2f}"),
        ("Power gap (chaser-committed)", "",
         lambda r: f"{r['agg']['power_chaser']['avg_power'] - r['agg']['archetype_committed']['avg_power']:.2f}"),
    ]

    for name, target, fn in metric_rows:
        print(f"  {name:<35} {target:<10}", end="")
        for mk in model_keys:
            val = fn(all_results[mk])
            print(f" {val:>10}", end="")
        print()


if __name__ == "__main__":
    main()
