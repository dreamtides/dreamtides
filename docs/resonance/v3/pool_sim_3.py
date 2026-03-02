#!/usr/bin/env python3
"""
Pool Distribution Simulation for the Lane Locking Draft Algorithm.

Investigates how cards should be distributed across archetypes, and how many
generic/bridge cards should exist. Tests 5 pool breakdown models against the
Lane Locking algorithm (thresholds 3/8, primary=2, secondary/tertiary=1).

Models:
  A: Equal flat — 40 per archetype, 36 generic, all cards [Primary] only
  B: Equal dual — 40 per archetype, 36 generic, mix of [P], [P,S], [P,P,S]
  C: Heavy generics — 30 per archetype (240), 120 generic (33%)
  D: Bridge-heavy — 30 core per archetype, 36 generic, 84 bridge cards
  E: Asymmetric — some archetypes 50 cards, some 30, 36 generic
"""

import random
import statistics
from collections import Counter
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ─── Constants ───────────────────────────────────────────────────────────────

PACK_SIZE = 4
NUM_PICKS = 30
NUM_SIMULATIONS = 1000

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

# Archetypes on the circle: (name, primary_resonance, secondary_resonance)
ARCHETYPES = [
    ("Flash",        "Zephyr", "Ember"),   # 0
    ("Blink",        "Ember",  "Zephyr"),  # 1
    ("Storm",        "Ember",  "Stone"),   # 2
    ("Self-Discard", "Stone",  "Ember"),   # 3
    ("Self-Mill",    "Stone",  "Tide"),    # 4
    ("Sacrifice",    "Tide",   "Stone"),   # 5
    ("Warriors",     "Tide",   "Zephyr"),  # 6
    ("Ramp",         "Zephyr", "Tide"),    # 7
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
NUM_ARCHETYPES = len(ARCHETYPES)

# Adjacent archetype pairs (index pairs on the circle, wrapping)
ADJACENT_PAIRS = [
    (0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 7), (7, 0)
]


# ─── Data Classes ────────────────────────────────────────────────────────────

class Rarity(Enum):
    COMMON = "common"
    UNCOMMON = "uncommon"
    RARE = "rare"
    LEGENDARY = "legendary"


@dataclass
class SimCard:
    id: int
    symbols: list          # ordered resonance strings, [] = generic
    archetype: Optional[str]       # home archetype (None for generic/bridge)
    bridge_archetypes: list        # list of archetype names if bridge card
    archetype_fitness: dict        # archetype_name -> tier (S/A/B/C/F)
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


def compute_fitness(card_archetype: Optional[str],
                    bridge_archetypes: list = None) -> dict:
    """
    Compute fitness tiers for a card given its home archetype(s).

    - S-tier: home archetype (and bridge archetypes)
    - A-tier: adjacent archetype sharing home's primary resonance
    - B-tier: archetype sharing home's secondary resonance (not already S/A)
    - C-tier: remaining archetypes sharing any resonance
    - F-tier: no shared resonance
    - Generic cards: B-tier in all archetypes
    """
    fitness = {}
    if card_archetype is None and not bridge_archetypes:
        # Generic card
        for name, _, _ in ARCHETYPES:
            fitness[name] = "B"
        return fitness

    # Collect all "home" archetypes (normal + bridge)
    home_names = []
    if card_archetype:
        home_names.append(card_archetype)
    if bridge_archetypes:
        for ba in bridge_archetypes:
            if ba not in home_names:
                home_names.append(ba)

    # Start with F everywhere
    for name, _, _ in ARCHETYPES:
        fitness[name] = "F"

    # Mark S-tier for all home archetypes
    for hn in home_names:
        fitness[hn] = "S"

    # For each home, compute A/B/C
    for hn in home_names:
        home_idx = get_archetype_index(hn)
        home_primary = ARCHETYPES[home_idx][1]
        home_secondary = ARCHETYPES[home_idx][2]

        for i, (name, arch_pri, arch_sec) in enumerate(ARCHETYPES):
            if fitness[name] == "S":
                continue  # already best
            if are_adjacent(home_idx, i) and home_primary in (arch_pri, arch_sec):
                if fitness[name] in ("F", "C", "B"):
                    fitness[name] = "A"
            elif home_secondary in (arch_pri, arch_sec):
                if fitness[name] in ("F", "C"):
                    fitness[name] = "B"
            elif home_primary in (arch_pri, arch_sec):
                if fitness[name] == "F":
                    fitness[name] = "C"

    return fitness


def assign_rarity() -> Rarity:
    r = random.random()
    if r < 0.50:
        return Rarity.COMMON
    elif r < 0.80:
        return Rarity.UNCOMMON
    elif r < 0.95:
        return Rarity.RARE
    else:
        return Rarity.LEGENDARY


# ─── Symbol Generation Helpers ──────────────────────────────────────────────

def make_mono_primary_symbol(primary: str) -> list:
    """Single symbol: [Primary]."""
    return [primary]


def make_mono_secondary_symbol(secondary: str) -> list:
    """Single symbol: [Secondary]."""
    return [secondary]


def make_dual_ps_symbol(primary: str, secondary: str) -> list:
    """Dual: [Primary, Secondary]."""
    return [primary, secondary]


def make_dual_pp_symbol(primary: str) -> list:
    """Dual: [Primary, Primary] -- deep mono."""
    return [primary, primary]


def make_dual_sp_symbol(secondary: str, primary: str) -> list:
    """Dual: [Secondary, Primary] -- secondary-led."""
    return [secondary, primary]


def make_triple_pps_symbol(primary: str, secondary: str) -> list:
    """Triple: [Primary, Primary, Secondary]."""
    return [primary, primary, secondary]


def make_triple_pss_symbol(primary: str, secondary: str) -> list:
    """Triple: [Primary, Secondary, Secondary]."""
    return [primary, secondary, secondary]


def generate_archetype_symbols_flat(primary: str, secondary: str) -> list:
    """Model A: all cards are [Primary] only."""
    return [primary]


def generate_archetype_symbols_dual(primary: str, secondary: str) -> list:
    """Model B: mix of [P], [P,S], [P,P,S] patterns."""
    r = random.random()
    if r < 0.25:
        return make_mono_primary_symbol(primary)
    elif r < 0.40:
        return make_mono_secondary_symbol(secondary)
    elif r < 0.65:
        return make_dual_ps_symbol(primary, secondary)
    elif r < 0.80:
        return make_dual_pp_symbol(primary)
    elif r < 0.90:
        return make_dual_sp_symbol(secondary, primary)
    else:
        return make_triple_pps_symbol(primary, secondary)


def generate_bridge_symbols(res_a: str, res_b: str) -> list:
    """
    Generate symbols for a bridge card shared between two adjacent archetypes.
    The card should carry symbols from both resonances.
    """
    r = random.random()
    if r < 0.35:
        return [res_a, res_b]
    elif r < 0.70:
        return [res_b, res_a]
    elif r < 0.85:
        return [res_a, res_b, res_a]
    else:
        return [res_b, res_a, res_b]


# ─── Card Pool Generators ──────────────────────────────────────────────────

def _make_generic_cards(start_id: int, count: int) -> list:
    """Create generic (no-symbol) cards."""
    cards = []
    for i in range(count):
        cards.append(SimCard(
            id=start_id + i,
            symbols=[],
            archetype=None,
            bridge_archetypes=[],
            archetype_fitness=compute_fitness(None),
            rarity=assign_rarity(),
            power=random.uniform(4.0, 9.5),
        ))
    return cards


def generate_pool_model_a() -> list:
    """
    Model A: Equal flat -- 40 per archetype, 36 generic, all [Primary] only.
    Total: 320 + 36 = 356; pad 4 more generics to reach 360.
    """
    cards = []
    card_id = 0
    cards_per_arch = 40

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        for _ in range(cards_per_arch):
            symbols = generate_archetype_symbols_flat(primary, secondary)
            cards.append(SimCard(
                id=card_id,
                symbols=symbols,
                archetype=arch_name,
                bridge_archetypes=[],
                archetype_fitness=compute_fitness(arch_name),
                rarity=assign_rarity(),
                power=random.uniform(3.0, 9.0),
            ))
            card_id += 1

    # 40 generics to reach 360
    generics = _make_generic_cards(card_id, 360 - len(cards))
    cards.extend(generics)
    random.shuffle(cards)
    return cards


def generate_pool_model_b() -> list:
    """
    Model B: Equal dual -- 40 per archetype, 36 generic, mixed symbol patterns.
    """
    cards = []
    card_id = 0
    cards_per_arch = 40

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        for _ in range(cards_per_arch):
            symbols = generate_archetype_symbols_dual(primary, secondary)
            cards.append(SimCard(
                id=card_id,
                symbols=symbols,
                archetype=arch_name,
                bridge_archetypes=[],
                archetype_fitness=compute_fitness(arch_name),
                rarity=assign_rarity(),
                power=random.uniform(3.0, 9.0),
            ))
            card_id += 1

    generics = _make_generic_cards(card_id, 360 - len(cards))
    cards.extend(generics)
    random.shuffle(cards)
    return cards


def generate_pool_model_c() -> list:
    """
    Model C: Heavy generics -- 30 per archetype (240), 120 generic (33%).
    Uses dual symbol patterns for archetype cards.
    """
    cards = []
    card_id = 0
    cards_per_arch = 30

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        for _ in range(cards_per_arch):
            symbols = generate_archetype_symbols_dual(primary, secondary)
            cards.append(SimCard(
                id=card_id,
                symbols=symbols,
                archetype=arch_name,
                bridge_archetypes=[],
                archetype_fitness=compute_fitness(arch_name),
                rarity=assign_rarity(),
                power=random.uniform(3.0, 9.0),
            ))
            card_id += 1

    generics = _make_generic_cards(card_id, 360 - len(cards))
    cards.extend(generics)
    random.shuffle(cards)
    return cards


def generate_pool_model_d() -> list:
    """
    Model D: Bridge-heavy -- 30 core per archetype (240), 36 generic,
    84 explicit bridge cards (shared between adjacent archetypes).
    Total: 240 + 36 + 84 = 360.

    Bridge cards: each of the 8 adjacent pairs gets ~10-11 bridge cards.
    A bridge card is S-tier in BOTH adjacent archetypes.
    """
    cards = []
    card_id = 0
    cards_per_arch = 30

    # Core archetype cards
    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        for _ in range(cards_per_arch):
            symbols = generate_archetype_symbols_dual(primary, secondary)
            cards.append(SimCard(
                id=card_id,
                symbols=symbols,
                archetype=arch_name,
                bridge_archetypes=[],
                archetype_fitness=compute_fitness(arch_name),
                rarity=assign_rarity(),
                power=random.uniform(3.0, 9.0),
            ))
            card_id += 1

    # Bridge cards: 84 total across 8 adjacent pairs
    bridge_per_pair = [11, 11, 10, 10, 11, 11, 10, 10]  # sums to 84
    for pair_idx, (i, j) in enumerate(ADJACENT_PAIRS):
        arch_a_name, pri_a, sec_a = ARCHETYPES[i]
        arch_b_name, pri_b, sec_b = ARCHETYPES[j]

        # Find the shared resonance(s) between adjacent archetypes
        shared = set()
        for r in RESONANCES:
            if r in (pri_a, sec_a) and r in (pri_b, sec_b):
                shared.add(r)

        # The two resonances we bridge across
        res_a = pri_a
        res_b = pri_b

        n_bridge = bridge_per_pair[pair_idx]
        for _ in range(n_bridge):
            symbols = generate_bridge_symbols(res_a, res_b)
            bridge_archs = [arch_a_name, arch_b_name]
            cards.append(SimCard(
                id=card_id,
                symbols=symbols,
                archetype=None,
                bridge_archetypes=bridge_archs,
                archetype_fitness=compute_fitness(
                    arch_a_name, bridge_archetypes=bridge_archs),
                rarity=assign_rarity(),
                power=random.uniform(4.0, 9.0),
            ))
            card_id += 1

    # Generics: 36
    generics = _make_generic_cards(card_id, 36)
    cards.extend(generics)

    assert len(cards) == 360, f"Model D has {len(cards)} cards, expected 360"
    random.shuffle(cards)
    return cards


def generate_pool_model_e() -> list:
    """
    Model E: Asymmetric -- some archetypes 50, some 30, 36 generic.
    Four "deep" archetypes (50 cards), four "shallow" (30 cards).
    50*4 + 30*4 = 200+120 = 320 + 36 = 356; pad 4 more generics.
    Uses dual symbol patterns.
    """
    # Alternate deep/shallow around the circle so adjacent archetypes differ
    arch_sizes = {
        "Flash": 50,        # deep
        "Blink": 30,        # shallow
        "Storm": 50,        # deep
        "Self-Discard": 30, # shallow
        "Self-Mill": 50,    # deep
        "Sacrifice": 30,    # shallow
        "Warriors": 50,     # deep
        "Ramp": 30,         # shallow
    }

    cards = []
    card_id = 0

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        n = arch_sizes[arch_name]
        for _ in range(n):
            symbols = generate_archetype_symbols_dual(primary, secondary)
            cards.append(SimCard(
                id=card_id,
                symbols=symbols,
                archetype=arch_name,
                bridge_archetypes=[],
                archetype_fitness=compute_fitness(arch_name),
                rarity=assign_rarity(),
                power=random.uniform(3.0, 9.0),
            ))
            card_id += 1

    generics = _make_generic_cards(card_id, 360 - len(cards))
    cards.extend(generics)
    random.shuffle(cards)
    return cards


# ─── Lane Locking Algorithm ─────────────────────────────────────────────────

@dataclass
class LaneLockState:
    symbol_counts: dict = field(
        default_factory=lambda: {r: 0 for r in RESONANCES})
    locked_slots: list = field(default_factory=list)
    thresholds_triggered: dict = field(
        default_factory=lambda: {r: set() for r in RESONANCES})
    max_locks: int = 4

    def add_card(self, card: SimCard,
                 threshold_low: int = 3, threshold_high: int = 8):
        new_locks = []
        if not card.symbols:
            return new_locks

        for i, sym in enumerate(card.symbols):
            weight = 2 if i == 0 else 1
            self.symbol_counts[sym] += weight

        effective_max = min(self.max_locks, PACK_SIZE)

        for res in RESONANCES:
            count = self.symbol_counts[res]
            if len(self.locked_slots) >= effective_max:
                break
            if count >= threshold_low and "low" not in self.thresholds_triggered[res]:
                self.thresholds_triggered[res].add("low")
                self.locked_slots.append(res)
                new_locks.append((res, "low"))
                if len(self.locked_slots) >= effective_max:
                    break
            if count >= threshold_high and "high" not in self.thresholds_triggered[res]:
                self.thresholds_triggered[res].add("high")
                self.locked_slots.append(res)
                new_locks.append((res, "high"))
                if len(self.locked_slots) >= effective_max:
                    break

        return new_locks


def build_pack(pool: list, state: LaneLockState) -> list:
    pack = []
    used_ids = set()

    slots_to_fill = state.locked_slots[:PACK_SIZE]
    for res in slots_to_fill:
        matching = [c for c in pool
                    if c.primary_resonance == res and c.id not in used_ids]
        if matching:
            chosen = random.choice(matching)
            pack.append(chosen)
            used_ids.add(chosen.id)
        else:
            matching = [c for c in pool
                        if res in c.symbols and c.id not in used_ids]
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


# ─── Player Strategies ───────────────────────────────────────────────────────

TIER_VALUES = {"S": 5, "A": 4, "B": 3, "C": 1, "F": 0}


def evaluate_archetype_strength(drafted: list) -> dict:
    scores = {name: 0 for name in ARCHETYPE_NAMES}
    for card in drafted:
        for arch, tier in card.archetype_fitness.items():
            scores[arch] += TIER_VALUES[tier]
    return scores


def pick_archetype_committed(pack: list, drafted: list,
                             committed_arch: Optional[str]) -> tuple:
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
            TIER_VALUES.get(c.archetype_fitness.get(best_arch, "F"), 0),
            c.power))
    else:
        best_card = max(pack, key=lambda c: (
            TIER_VALUES.get(c.archetype_fitness.get(committed_arch, "F"), 0),
            c.power))

    return best_card, committed_arch


def pick_power_chaser(pack: list, drafted: list) -> SimCard:
    return max(pack, key=lambda c: c.power)


def pick_signal_reader(pack: list, drafted: list, pick_num: int,
                       committed_arch: Optional[str]) -> tuple:
    if pick_num < 5:
        pack_res_count = Counter()
        for card in pack:
            for i, sym in enumerate(card.symbols):
                pack_res_count[sym] += 2 if i == 0 else 1
        if pack_res_count:
            best_res = max(pack_res_count, key=pack_res_count.get)
        else:
            best_res = random.choice(RESONANCES)

        def card_res_score(c):
            score = 0
            for i, sym in enumerate(c.symbols):
                if sym == best_res:
                    score += 2 if i == 0 else 1
            return score

        best_card = max(pack, key=lambda c: (card_res_score(c), c.power))
        return best_card, None
    else:
        if committed_arch is None:
            scores = evaluate_archetype_strength(drafted)
            committed_arch = max(scores, key=scores.get)
        best_card = max(pack, key=lambda c: (
            TIER_VALUES.get(c.archetype_fitness.get(committed_arch, "F"), 0),
            c.power))
        return best_card, committed_arch


# ─── Evaluation Helpers ─────────────────────────────────────────────────────

def card_fits_archetype(card: SimCard, arch_name: str) -> bool:
    return card.archetype_fitness.get(arch_name, "F") in ("S", "A")


def card_is_off_archetype(card: SimCard, arch_name: str) -> bool:
    return card.archetype_fitness.get(arch_name, "F") in ("C", "F")


def get_unique_archetypes_with_sa(pack: list) -> set:
    archetypes_present = set()
    for card in pack:
        for arch_name in ARCHETYPE_NAMES:
            if card.archetype_fitness.get(arch_name, "F") in ("S", "A"):
                archetypes_present.add(arch_name)
    return archetypes_present


# ─── Simulation Core ─────────────────────────────────────────────────────────

@dataclass
class DraftMetrics:
    early_unique_archetypes_per_pack: list = field(default_factory=list)
    early_archetype_fit_per_pack: list = field(default_factory=list)
    late_archetype_fit_per_pack: list = field(default_factory=list)
    late_off_archetype_per_pack: list = field(default_factory=list)
    convergence_pick: Optional[int] = None
    deck_concentration: float = 0.0
    drafted_card_ids: list = field(default_factory=list)
    final_archetype: Optional[str] = None


def run_single_draft(pool: list, strategy: str) -> DraftMetrics:
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

        if pick_num < 5:
            unique_archs = get_unique_archetypes_with_sa(pack)
            metrics.early_unique_archetypes_per_pack.append(len(unique_archs))
            if current_best_arch:
                fit_count = sum(1 for c in pack
                                if card_fits_archetype(c, current_best_arch))
                metrics.early_archetype_fit_per_pack.append(fit_count)

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

        if strategy == "archetype_committed":
            chosen, committed_arch = pick_archetype_committed(
                pack, drafted, committed_arch)
        elif strategy == "power_chaser":
            chosen = pick_power_chaser(pack, drafted)
        elif strategy == "signal_reader":
            chosen, committed_arch = pick_signal_reader(
                pack, drafted, pick_num, committed_arch)
        else:
            raise ValueError(f"Unknown strategy: {strategy}")

        state.add_card(chosen)
        drafted.append(chosen)
        metrics.drafted_card_ids.append(chosen.id)

    if drafted:
        scores = evaluate_archetype_strength(drafted)
        best_arch = max(scores, key=scores.get)
        sa_count = sum(1 for c in drafted if card_fits_archetype(c, best_arch))
        metrics.deck_concentration = sa_count / len(drafted)
        metrics.final_archetype = best_arch

    return metrics


# ─── Bridge Strategy ────────────────────────────────────────────────────────

def run_bridge_draft(pool: list) -> dict:
    """
    Simulate a player committing to TWO adjacent archetypes.
    After pick 5, they commit to whichever adjacent pair they have the most
    combined S/A fitness for. They pick the best card for either archetype.
    Returns metrics about bridge viability.
    """
    state = LaneLockState()
    drafted = []
    committed_pair = None
    late_sa_counts = []
    late_either_sa = []

    for pick_num in range(NUM_PICKS):
        pack = build_pack(pool, state)

        if committed_pair is None and len(drafted) >= 5:
            scores = evaluate_archetype_strength(drafted)
            # Find best adjacent pair
            best_pair = None
            best_pair_score = -1
            for i, j in ADJACENT_PAIRS:
                pair_score = scores[ARCHETYPE_NAMES[i]] + scores[ARCHETYPE_NAMES[j]]
                if pair_score > best_pair_score:
                    best_pair_score = pair_score
                    best_pair = (ARCHETYPE_NAMES[i], ARCHETYPE_NAMES[j])
            committed_pair = best_pair

        if committed_pair:
            a1, a2 = committed_pair
            # Count cards fitting either archetype at S/A
            either_sa = sum(1 for c in pack
                            if card_fits_archetype(c, a1) or
                            card_fits_archetype(c, a2))
            late_either_sa.append(either_sa)

            # Count cards fitting the primary archetype
            primary_sa = sum(1 for c in pack if card_fits_archetype(c, a1))
            late_sa_counts.append(primary_sa)

            # Pick best for either
            chosen = max(pack, key=lambda c: max(
                TIER_VALUES.get(c.archetype_fitness.get(a1, "F"), 0),
                TIER_VALUES.get(c.archetype_fitness.get(a2, "F"), 0),
            ))
        else:
            # Early: same as archetype committed, pick greedily
            if drafted:
                scores = evaluate_archetype_strength(drafted)
                best_arch = max(scores, key=scores.get)
            else:
                best_arch = random.choice(ARCHETYPE_NAMES)
            chosen = max(pack, key=lambda c: (
                TIER_VALUES.get(c.archetype_fitness.get(best_arch, "F"), 0),
                c.power))

        state.add_card(chosen)
        drafted.append(chosen)

    avg_either_sa = statistics.mean(late_either_sa) if late_either_sa else 0
    avg_primary_sa = statistics.mean(late_sa_counts) if late_sa_counts else 0

    # Deck stats
    if committed_pair:
        a1, a2 = committed_pair
        sa_either = sum(1 for c in drafted
                        if card_fits_archetype(c, a1) or
                        card_fits_archetype(c, a2))
        deck_bridge_concentration = sa_either / len(drafted) if drafted else 0
    else:
        deck_bridge_concentration = 0

    return {
        "avg_either_sa_per_pack": avg_either_sa,
        "avg_primary_sa_per_pack": avg_primary_sa,
        "deck_bridge_concentration": deck_bridge_concentration,
        "committed_pair": committed_pair,
    }


# ─── Aggregate Computation ──────────────────────────────────────────────────

def compute_aggregate(all_metrics: list) -> dict:
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
    s["convergence_pct"] = len(conv_picks) / len(all_metrics) * 100

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
                overlap = len(s1 & s2) / len(s1 | s2)
                overlaps.append(overlap)
        s["card_overlap"] = statistics.mean(overlaps) if overlaps else 0
    else:
        s["card_overlap"] = 0

    return s


def measure_archetype_frequency(all_metrics: list) -> dict:
    """Measure which archetypes committed players end up in."""
    arch_counts = Counter()
    for m in all_metrics:
        if m.final_archetype:
            arch_counts[m.final_archetype] += 1
    total = sum(arch_counts.values())
    if total == 0:
        return {n: 0 for n in ARCHETYPE_NAMES}
    return {n: arch_counts.get(n, 0) / total for n in ARCHETYPE_NAMES}


def pool_composition_summary(pool: list) -> dict:
    """Summarize a pool's composition."""
    generic_count = sum(1 for c in pool if not c.symbols)
    bridge_count = sum(1 for c in pool if c.bridge_archetypes)
    archetype_count = sum(1 for c in pool
                          if c.archetype is not None and not c.bridge_archetypes)

    # Per-archetype S/A counts
    sa_per_arch = {}
    for arch in ARCHETYPE_NAMES:
        sa_per_arch[arch] = sum(1 for c in pool
                                if card_fits_archetype(c, arch))

    # Symbol count distribution
    sym_dist = Counter()
    for c in pool:
        sym_dist[len(c.symbols)] += 1

    return {
        "total": len(pool),
        "generic": generic_count,
        "bridge": bridge_count,
        "archetype": archetype_count,
        "sa_per_arch": sa_per_arch,
        "sym_dist": dict(sym_dist),
    }


# ─── Run All Models ─────────────────────────────────────────────────────────

def run_model(model_name: str, pool_fn, n_sims: int = NUM_SIMULATIONS):
    """Run a complete model evaluation."""
    pool = pool_fn()
    comp = pool_composition_summary(pool)

    print(f"\n{'='*70}")
    print(f"  MODEL {model_name}")
    print(f"{'='*70}")
    print(f"  Pool: {comp['total']} cards | "
          f"{comp['archetype']} archetype | "
          f"{comp['bridge']} bridge | "
          f"{comp['generic']} generic")
    print(f"  Symbol counts: {comp['sym_dist']}")

    sa_vals = list(comp['sa_per_arch'].values())
    print(f"  S/A cards per archetype: min={min(sa_vals)}, "
          f"max={max(sa_vals)}, avg={statistics.mean(sa_vals):.1f}")
    for arch in ARCHETYPE_NAMES:
        print(f"    {arch:15s}: {comp['sa_per_arch'][arch]} S/A cards")

    # Run all 3 strategies
    results = {}
    for strategy in ["archetype_committed", "power_chaser", "signal_reader"]:
        all_metrics = []
        for _ in range(n_sims):
            m = run_single_draft(pool, strategy)
            all_metrics.append(m)
        results[strategy] = all_metrics

    # Aggregate per strategy
    agg = {}
    for strategy, metrics_list in results.items():
        agg[strategy] = compute_aggregate(metrics_list)

    # Print results per strategy
    for strategy, s in agg.items():
        print(f"\n  --- {strategy} ---")
        print(f"    Early unique archs w/ S/A:    {s['early_unique_archs']:.2f}  (>= 3)")
        print(f"    Early S/A for arch/pack:       {s['early_arch_fit']:.2f}  (<= 2)")
        print(f"    Late S/A for arch/pack:        {s['late_arch_fit']:.2f}  (>= 2)")
        print(f"    Late C/F cards/pack:           {s['late_off_arch']:.2f}  (>= 0.5)")
        print(f"    Convergence pick:              {s['convergence_pick']:.1f}  (5-8)")
        print(f"    Deck S/A concentration:        {s['deck_concentration']*100:.1f}%  (60-80%)")
        print(f"    Card overlap:                  {s['card_overlap']*100:.1f}%  (< 40%)")

    # Archetype frequency (committed strategy)
    freq = measure_archetype_frequency(results["archetype_committed"])
    print(f"\n  Archetype frequency (committed):")
    for arch in ARCHETYPE_NAMES:
        pct = freq.get(arch, 0)
        status = "PASS" if 0.05 <= pct <= 0.20 else "FAIL"
        print(f"    {arch:15s}: {pct*100:5.1f}%  [{status}]")

    # Bridge strategy viability
    print(f"\n  Bridge strategy viability (500 drafts):")
    bridge_results = []
    for _ in range(500):
        br = run_bridge_draft(pool)
        bridge_results.append(br)
    avg_bridge_sa = statistics.mean(
        [b["avg_either_sa_per_pack"] for b in bridge_results])
    avg_bridge_conc = statistics.mean(
        [b["deck_bridge_concentration"] for b in bridge_results])
    print(f"    Avg S/A for either archetype/pack: {avg_bridge_sa:.2f}")
    print(f"    Deck bridge concentration:         {avg_bridge_conc*100:.1f}%")

    # Committed player: how often S-tier vs A-tier in late packs
    committed_metrics = results["archetype_committed"]
    s_counts = []
    a_counts = []
    for m in committed_metrics:
        if m.final_archetype:
            # Re-simulate to count S vs A separately -- approximate by checking
            # drafted cards
            drafted_cards = [c for c in pool if c.id in set(m.drafted_card_ids)]
            s_count = sum(1 for c in drafted_cards
                          if c.archetype_fitness.get(m.final_archetype, "F") == "S")
            a_count = sum(1 for c in drafted_cards
                          if c.archetype_fitness.get(m.final_archetype, "F") == "A")
            s_counts.append(s_count)
            a_counts.append(a_count)
    avg_s = statistics.mean(s_counts) if s_counts else 0
    avg_a = statistics.mean(a_counts) if a_counts else 0
    print(f"\n  Committed player deck composition (30 cards):")
    print(f"    Avg S-tier (home archetype):  {avg_s:.1f}")
    print(f"    Avg A-tier (adjacent arch):   {avg_a:.1f}")
    print(f"    Avg B/C/F:                    {30 - avg_s - avg_a:.1f}")

    # Scorecard summary
    s = agg["archetype_committed"]
    max_freq = max(freq.values()) if freq else 0
    min_freq = min(freq.values()) if freq else 0

    print(f"\n  TARGET SCORECARD:")
    targets = [
        ("Early unique archs w/ S/A", ">= 3",
         f"{s['early_unique_archs']:.2f}",
         s["early_unique_archs"] >= 3),
        ("Early S/A for arch/pack", "<= 2",
         f"{s['early_arch_fit']:.2f}",
         s["early_arch_fit"] <= 2),
        ("Late S/A for arch/pack", ">= 2",
         f"{s['late_arch_fit']:.2f}",
         s["late_arch_fit"] >= 2),
        ("Late C/F cards/pack", ">= 0.5",
         f"{s['late_off_arch']:.2f}",
         s["late_off_arch"] >= 0.5),
        ("Convergence pick", "5-8",
         f"{s['convergence_pick']:.1f}",
         5 <= s["convergence_pick"] <= 8),
        ("Deck concentration", "60-80%",
         f"{s['deck_concentration']*100:.1f}%",
         60 <= s["deck_concentration"] * 100 <= 80),
        ("Card overlap", "< 40%",
         f"{s['card_overlap']*100:.1f}%",
         s["card_overlap"] * 100 < 40),
        ("Archetype freq", "5-20%",
         f"{min_freq*100:.0f}-{max_freq*100:.0f}%",
         max_freq <= 0.20 and min_freq >= 0.05),
    ]

    pass_count = 0
    for name, target, actual, passed in targets:
        result = "PASS" if passed else "FAIL"
        if passed:
            pass_count += 1
        print(f"    {name:<35} {target:<10} {actual:<10} {result}")
    print(f"    TOTAL: {pass_count}/{len(targets)} passed")

    return {
        "agg": agg,
        "freq": freq,
        "bridge_sa": avg_bridge_sa,
        "bridge_conc": avg_bridge_conc,
        "comp": comp,
        "targets_passed": pass_count,
        "avg_s_deck": avg_s,
        "avg_a_deck": avg_a,
    }


# ─── Main ────────────────────────────────────────────────────────────────────

def main():
    random.seed(42)

    print("="*70)
    print("  POOL DISTRIBUTION SIMULATION FOR LANE LOCKING")
    print("  Testing 5 pool breakdown models, 1000 drafts each")
    print("  Lane Locking: thresholds (3,8), primary=2, secondary/tertiary=1")
    print("="*70)

    models = {
        "A (Equal Flat)": generate_pool_model_a,
        "B (Equal Dual)": generate_pool_model_b,
        "C (Heavy Generics)": generate_pool_model_c,
        "D (Bridge-Heavy)": generate_pool_model_d,
        "E (Asymmetric)": generate_pool_model_e,
    }

    all_results = {}
    for model_name, pool_fn in models.items():
        all_results[model_name] = run_model(model_name, pool_fn)

    # ─── Summary Comparison Table ──────────────────────────────────────────
    print("\n\n" + "="*70)
    print("  COMPARISON SUMMARY")
    print("="*70)

    header = f"  {'Metric':<30}"
    for mn in models.keys():
        short = mn.split("(")[0].strip()
        header += f" {short:>10}"
    print(header)
    print("  " + "-" * (30 + len(models) * 11))

    metrics_to_show = [
        ("Late S/A/pack", lambda r: f"{r['agg']['archetype_committed']['late_arch_fit']:.2f}"),
        ("Early unique archs", lambda r: f"{r['agg']['archetype_committed']['early_unique_archs']:.2f}"),
        ("Early S/A/pack", lambda r: f"{r['agg']['archetype_committed']['early_arch_fit']:.2f}"),
        ("Late C/F/pack", lambda r: f"{r['agg']['archetype_committed']['late_off_arch']:.2f}"),
        ("Convergence pick", lambda r: f"{r['agg']['archetype_committed']['convergence_pick']:.1f}"),
        ("Deck concentration", lambda r: f"{r['agg']['archetype_committed']['deck_concentration']*100:.0f}%"),
        ("Card overlap", lambda r: f"{r['agg']['archetype_committed']['card_overlap']*100:.1f}%"),
        ("Targets passed", lambda r: f"{r['targets_passed']}/8"),
        ("Bridge S/A/pack", lambda r: f"{r['bridge_sa']:.2f}"),
        ("Bridge deck conc.", lambda r: f"{r['bridge_conc']*100:.0f}%"),
        ("Avg S in deck", lambda r: f"{r['avg_s_deck']:.1f}"),
        ("Avg A in deck", lambda r: f"{r['avg_a_deck']:.1f}"),
    ]

    for metric_name, extractor in metrics_to_show:
        row = f"  {metric_name:<30}"
        for mn in models.keys():
            row += f" {extractor(all_results[mn]):>10}"
        print(row)

    # ─── Per-archetype comparison for Model E ─────────────────────────────
    print("\n  Model E per-archetype S/A pool sizes:")
    comp_e = all_results["E (Asymmetric)"]["comp"]
    for arch in ARCHETYPE_NAMES:
        print(f"    {arch:15s}: {comp_e['sa_per_arch'][arch]} S/A cards in pool")

    print("\n  Model E archetype frequency:")
    freq_e = all_results["E (Asymmetric)"]["freq"]
    for arch in ARCHETYPE_NAMES:
        pct = freq_e.get(arch, 0)
        print(f"    {arch:15s}: {pct*100:.1f}%")


if __name__ == "__main__":
    main()
