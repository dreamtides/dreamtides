#!/usr/bin/env python3
"""
Pool Symbol Pattern Simulation for Lane Locking Algorithm.

Investigates: What SPECIFIC symbol patterns should cards have, and how do
different pattern compositions affect draft decisions?

Previous simulations varied the count of symbols (1 vs 2 vs 3). This simulation
fixes the symbol count distribution at the recommended 25/55/20 and varies the
SPECIFIC patterns within each count -- which orderings appear and in what
proportions.

Lane Locking: Your pack has 4 slots; when your symbol count in a resonance
first reaches 3, one open slot locks to that resonance; when it reaches 8,
a second slot locks. Primary symbol counts as 2, secondary/tertiary as 1.
Max 4 locked slots.
"""

import random
import statistics
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict, Counter
from typing import Optional

# ---------------------------------------------------------------------------
# Core types
# ---------------------------------------------------------------------------

class Resonance(Enum):
    EMBER = "Ember"
    STONE = "Stone"
    TIDE = "Tide"
    ZEPHYR = "Zephyr"

class Tier(Enum):
    S = "S"
    A = "A"
    B = "B"
    C = "C"
    F = "F"

# The 8 archetypes on a circle: (name, primary, secondary)
ARCHETYPES = [
    ("Flash",        Resonance.ZEPHYR, Resonance.EMBER),   # 0
    ("Blink",        Resonance.EMBER,  Resonance.ZEPHYR),  # 1
    ("Storm",        Resonance.EMBER,  Resonance.STONE),   # 2
    ("Self-Discard", Resonance.STONE,  Resonance.EMBER),   # 3
    ("Self-Mill",    Resonance.STONE,  Resonance.TIDE),    # 4
    ("Sacrifice",    Resonance.TIDE,   Resonance.STONE),   # 5
    ("Warriors",     Resonance.TIDE,   Resonance.ZEPHYR),  # 6
    ("Ramp",         Resonance.ZEPHYR, Resonance.TIDE),    # 7
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
NUM_ARCHETYPES = 8
PACK_SIZE = 4
NUM_PICKS = 30
RESONANCES = list(Resonance)

# ---------------------------------------------------------------------------
# Fitness model
# ---------------------------------------------------------------------------

def circle_distance(i: int, j: int) -> int:
    d = abs(i - j)
    return min(d, NUM_ARCHETYPES - d)

def are_adjacent(i: int, j: int) -> bool:
    return circle_distance(i, j) == 1

def compute_fitness(card_arch_idx: int) -> dict:
    """Compute fitness tiers for a card from a given archetype."""
    fitness = {}
    home_primary = ARCHETYPES[card_arch_idx][1]
    home_secondary = ARCHETYPES[card_arch_idx][2]

    for j in range(NUM_ARCHETYPES):
        arch_pri = ARCHETYPES[j][1]
        arch_sec = ARCHETYPES[j][2]
        if j == card_arch_idx:
            fitness[j] = Tier.S
        elif are_adjacent(card_arch_idx, j) and home_primary in (arch_pri, arch_sec):
            fitness[j] = Tier.A
        elif home_secondary in (arch_pri, arch_sec) and j not in fitness:
            fitness[j] = Tier.B
        elif home_primary in (arch_pri, arch_sec):
            fitness[j] = Tier.C
        elif home_secondary in (arch_pri, arch_sec):
            fitness[j] = Tier.B
        else:
            fitness[j] = Tier.F
    return fitness

def compute_generic_fitness() -> dict:
    return {j: Tier.B for j in range(NUM_ARCHETYPES)}


# ---------------------------------------------------------------------------
# Card data
# ---------------------------------------------------------------------------

@dataclass
class SimCard:
    id: int
    symbols: list           # list of Resonance, 0-3 elements
    archetype_idx: int      # index into ARCHETYPES, -1 for generic
    pattern_name: str       # human-readable pattern type
    fitness: dict = field(default_factory=dict)  # arch_idx -> Tier

    @property
    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None

    @property
    def weighted_symbols(self) -> dict:
        counts = defaultdict(int)
        for i, sym in enumerate(self.symbols):
            counts[sym] += 2 if i == 0 else 1
        return dict(counts)


# ---------------------------------------------------------------------------
# Pattern definitions
#
# For an archetype with primary P and secondary S, the possible patterns are:
#
# 1-symbol patterns:
#   [P]         -> +2P          ("mono_primary")
#   [S]         -> +2S          ("mono_secondary")
#
# 2-symbol patterns:
#   [P, S]      -> +2P, +1S    ("standard_dual")     The canonical archetype card.
#   [P, P]      -> +3P         ("double_primary")    Fast single-resonance accumulation.
#   [S, P]      -> +2S, +1P    ("secondary_led")     Bridges to neighbor archetype.
#   [P, Other]  -> +2P, +1O    ("cross_bridge")      Bridges to non-adjacent archetype.
#
# 3-symbol patterns:
#   [P, P, S]   -> +3P, +1S    ("deep_commit")       Heavy primary investment.
#   [P, S, S]   -> +2P, +2S    ("balanced_triple")   Equal dual investment.
#   [P, S, O]   -> +2P, +1S, +1O ("triple_splash")   Three-way contribution.
#   [P, P, P]   -> +4P         ("triple_primary")    Maximum single-resonance.
# ---------------------------------------------------------------------------

def get_other_resonance(primary, secondary):
    """Get a resonance that is neither primary nor secondary for the archetype.
    Picks a resonance that would bridge to a non-adjacent archetype."""
    others = [r for r in Resonance if r != primary and r != secondary]
    return random.choice(others)


def generate_symbols_for_pattern(pattern_name: str, primary: Resonance,
                                  secondary: Resonance) -> list:
    """Generate the symbol list for a given pattern name."""
    if pattern_name == "mono_primary":
        return [primary]
    elif pattern_name == "mono_secondary":
        return [secondary]
    elif pattern_name == "standard_dual":
        return [primary, secondary]
    elif pattern_name == "double_primary":
        return [primary, primary]
    elif pattern_name == "secondary_led":
        return [secondary, primary]
    elif pattern_name == "cross_bridge":
        other = get_other_resonance(primary, secondary)
        return [primary, other]
    elif pattern_name == "deep_commit":
        return [primary, primary, secondary]
    elif pattern_name == "balanced_triple":
        return [primary, secondary, secondary]
    elif pattern_name == "triple_splash":
        other = get_other_resonance(primary, secondary)
        return [primary, secondary, other]
    elif pattern_name == "triple_primary":
        return [primary, primary, primary]
    else:
        raise ValueError(f"Unknown pattern: {pattern_name}")


# ---------------------------------------------------------------------------
# Pattern compositions (5 configurations)
# Each maps pattern_name -> fraction of non-generic cards.
# Fractions must sum to ~1.0.
# ---------------------------------------------------------------------------

COMPOSITIONS = {
    "A: Pure Archetype": {
        # All 2-symbol [P, S] cards. Perfectly aligned to archetypes.
        "standard_dual": 1.0,
    },
    "B: Mono-Heavy": {
        # 60% mono primary, 30% standard dual, 10% deep commit triple
        "mono_primary": 0.60,
        "standard_dual": 0.30,
        "deep_commit": 0.10,
    },
    "C: Bridge-Heavy": {
        # 30% mono, 30% standard, 20% secondary-led, 20% cross-bridge
        "mono_primary": 0.30,
        "standard_dual": 0.30,
        "secondary_led": 0.20,
        "cross_bridge": 0.20,
    },
    "D: Deep Commitment": {
        # 20% mono, 40% standard, 20% double-primary, 20% deep commit
        "mono_primary": 0.20,
        "standard_dual": 0.40,
        "double_primary": 0.20,
        "deep_commit": 0.20,
    },
    "E: Maximum Variety": {
        # Roughly equal distribution across all 10 pattern types
        "mono_primary": 0.10,
        "mono_secondary": 0.05,
        "standard_dual": 0.15,
        "double_primary": 0.10,
        "secondary_led": 0.10,
        "cross_bridge": 0.10,
        "deep_commit": 0.10,
        "balanced_triple": 0.10,
        "triple_splash": 0.10,
        "triple_primary": 0.10,
    },
    "F: Recommended": {
        # Optimized blend: strong core identity (mono+standard=55%),
        # meaningful variety (double_primary+secondary_led=25%),
        # small splash of depth (deep_commit+balanced_triple=20%)
        "mono_primary": 0.25,
        "standard_dual": 0.30,
        "double_primary": 0.10,
        "secondary_led": 0.10,
        "deep_commit": 0.10,
        "balanced_triple": 0.10,
        "cross_bridge": 0.05,
    },
}


# ---------------------------------------------------------------------------
# Pool generation
# ---------------------------------------------------------------------------

def generate_pool(composition: dict, num_generic: int = 36,
                  total: int = 360) -> list:
    """Generate a card pool with the specified pattern composition."""
    cards = []
    card_id = 0

    # Generic cards
    for _ in range(num_generic):
        card = SimCard(id=card_id, symbols=[], archetype_idx=-1,
                       pattern_name="generic")
        card.fitness = compute_generic_fitness()
        cards.append(card)
        card_id += 1

    # Archetype cards
    non_generic = total - num_generic  # 324
    per_archetype = non_generic // NUM_ARCHETYPES  # 40
    leftover = non_generic - per_archetype * NUM_ARCHETYPES

    for arch_idx in range(NUM_ARCHETYPES):
        _, primary, secondary = ARCHETYPES[arch_idx]
        n_cards = per_archetype + (1 if arch_idx < leftover else 0)

        # Distribute cards across patterns according to composition
        pattern_counts = {}
        remaining = n_cards
        pattern_names = list(composition.keys())
        for i, pname in enumerate(pattern_names):
            if i == len(pattern_names) - 1:
                # Last pattern gets whatever is left
                pattern_counts[pname] = remaining
            else:
                count = round(n_cards * composition[pname])
                count = min(count, remaining)
                pattern_counts[pname] = count
                remaining -= count

        for pname, count in pattern_counts.items():
            for _ in range(count):
                symbols = generate_symbols_for_pattern(pname, primary, secondary)
                card = SimCard(id=card_id, symbols=symbols,
                               archetype_idx=arch_idx, pattern_name=pname)
                card.fitness = compute_fitness(arch_idx)
                cards.append(card)
                card_id += 1

    return cards


# ---------------------------------------------------------------------------
# Lane Locking Algorithm
# ---------------------------------------------------------------------------

@dataclass
class DraftState:
    counters: dict = field(default_factory=lambda: {r: 0 for r in Resonance})
    locked_slots: list = field(default_factory=list)  # list of (Resonance, threshold)
    picks: list = field(default_factory=list)
    target_archetype: int = -1

    # Tracking
    lock_picks: list = field(default_factory=list)
    first_lock_pick: int = -1
    second_lock_pick: int = -1
    all_locked_pick: int = -1

    @property
    def num_locked(self) -> int:
        return len(self.locked_slots)

    @property
    def num_open(self) -> int:
        return PACK_SIZE - self.num_locked

    def locked_resonances(self) -> set:
        return {r for r, _ in self.locked_slots}

    def locked_resonance_counts(self) -> dict:
        counts = defaultdict(int)
        for r, _ in self.locked_slots:
            counts[r] += 1
        return dict(counts)


def fill_pack(state: DraftState, pool: list) -> list:
    """Fill a 4-card pack according to Lane Locking rules."""
    pack = []
    by_primary = defaultdict(list)
    for card in pool:
        if card.primary_resonance:
            by_primary[card.primary_resonance].append(card)

    # Fill locked slots
    for res, _ in state.locked_slots[:PACK_SIZE]:
        candidates = by_primary.get(res, [])
        if candidates:
            pack.append(random.choice(candidates))
        else:
            pack.append(random.choice(pool))

    # Fill open slots with random cards
    for _ in range(state.num_open):
        pack.append(random.choice(pool))

    return pack


def update_state(state: DraftState, card: SimCard, pick_num: int):
    """Update draft state after picking a card."""
    state.picks.append(card)
    ws = card.weighted_symbols
    for res, count in ws.items():
        state.counters[res] += count

    # Check thresholds
    for res in Resonance:
        count = state.counters[res]
        existing_locks = sum(1 for r, _ in state.locked_slots if r == res)

        if state.num_locked >= PACK_SIZE:
            break

        if count >= 3 and existing_locks == 0:
            state.locked_slots.append((res, 3))
            state.lock_picks.append(pick_num)
            if state.first_lock_pick == -1:
                state.first_lock_pick = pick_num
            elif state.second_lock_pick == -1:
                state.second_lock_pick = pick_num
            if state.num_locked >= PACK_SIZE:
                if state.all_locked_pick == -1:
                    state.all_locked_pick = pick_num
                break

        if count >= 8 and existing_locks == 1:
            state.locked_slots.append((res, 8))
            state.lock_picks.append(pick_num)
            if state.second_lock_pick == -1:
                state.second_lock_pick = pick_num
            if state.num_locked >= PACK_SIZE:
                if state.all_locked_pick == -1:
                    state.all_locked_pick = pick_num
                break


# ---------------------------------------------------------------------------
# Fitness helpers
# ---------------------------------------------------------------------------

def is_sa_tier(card: SimCard, arch_idx: int) -> bool:
    tier = card.fitness.get(arch_idx, Tier.F)
    return tier in (Tier.S, Tier.A)

def is_cf_tier(card: SimCard, arch_idx: int) -> bool:
    tier = card.fitness.get(arch_idx, Tier.F)
    return tier in (Tier.C, Tier.F)

def pack_sa_count(pack: list, arch_idx: int) -> int:
    return sum(1 for c in pack if is_sa_tier(c, arch_idx))

def pack_unique_sa_archs(pack: list) -> int:
    archs = set()
    for card in pack:
        for j in range(NUM_ARCHETYPES):
            if is_sa_tier(card, j):
                archs.add(j)
    return len(archs)


# ---------------------------------------------------------------------------
# Player strategy: archetype-committed
# ---------------------------------------------------------------------------

TIER_VALUES = {Tier.S: 5, Tier.A: 4, Tier.B: 3, Tier.C: 1, Tier.F: 0}

def pick_card_committed(pack: list, state: DraftState, pick_num: int) -> SimCard:
    """Committed player: picks for target archetype, commits by pick 5."""
    arch = state.target_archetype
    if arch < 0:
        return random.choice(pack)

    # Sort by fitness tier, then by how many weighted symbols contribute to
    # the archetype's primary resonance (to simulate meaningful pattern choices)
    target_primary = ARCHETYPES[arch][1]
    target_secondary = ARCHETYPES[arch][2]

    def card_value(c):
        tier_val = TIER_VALUES.get(c.fitness.get(arch, Tier.F), 0)
        # Within same tier, prefer cards whose symbols align with target
        ws = c.weighted_symbols
        alignment = ws.get(target_primary, 0) * 2 + ws.get(target_secondary, 0)
        return (tier_val, alignment)

    return max(pack, key=card_value)


# ---------------------------------------------------------------------------
# Simulation metrics
# ---------------------------------------------------------------------------

@dataclass
class DraftMetrics:
    """Metrics from a single draft."""
    first_lock_pick: int = -1
    second_lock_pick: int = -1
    all_locked_pick: int = -1

    # Accidental locks: resonances locked that don't match target archetype
    unwanted_locks: int = 0

    # Per-pack S/A counts at each pick (for convergence curve)
    sa_per_pack: list = field(default_factory=list)

    # Number of distinct resonances that end up locked
    distinct_locked_res: int = 0

    # How many picks had a "genuine choice" - 2+ S/A cards with different patterns
    genuine_choice_count: int = 0
    total_post_commit_picks: int = 0

    # Wasted symbols: symbols added to a resonance that already has 2 locks
    wasted_symbols: int = 0

    # Early diversity (unique archs with S/A, picks 1-5)
    early_diversity_vals: list = field(default_factory=list)

    # Late S/A (picks 6+)
    late_sa_vals: list = field(default_factory=list)

    # Late C/F (picks 6+)
    late_cf_vals: list = field(default_factory=list)

    # Deck concentration
    deck_sa_concentration: float = 0.0


def simulate_draft(pool: list, target_arch: int, num_picks: int = NUM_PICKS) -> DraftMetrics:
    """Simulate a single draft with an archetype-committed player."""
    state = DraftState()
    state.target_archetype = target_arch
    metrics = DraftMetrics()

    target_primary = ARCHETYPES[target_arch][1]
    target_secondary = ARCHETYPES[target_arch][2]
    target_resonances = {target_primary, target_secondary}

    for pick_num in range(1, num_picks + 1):
        pack = fill_pack(state, pool)

        sa_count = pack_sa_count(pack, target_arch)
        metrics.sa_per_pack.append(sa_count)

        if pick_num <= 5:
            metrics.early_diversity_vals.append(pack_unique_sa_archs(pack))

        if pick_num >= 6:
            metrics.late_sa_vals.append(sa_count)
            cf_count = sum(1 for c in pack if is_cf_tier(c, target_arch))
            metrics.late_cf_vals.append(cf_count)

        # Measure genuine choice: 2+ S/A cards with different pattern_names
        if pick_num >= 3:  # after initial commitment
            metrics.total_post_commit_picks += 1
            sa_cards = [c for c in pack if is_sa_tier(c, target_arch)]
            if len(sa_cards) >= 2:
                patterns_in_sa = set(c.pattern_name for c in sa_cards)
                if len(patterns_in_sa) >= 2:
                    metrics.genuine_choice_count += 1

        card = pick_card_committed(pack, state, pick_num)
        update_state(state, card, pick_num)

    # Record lock timing
    metrics.first_lock_pick = state.first_lock_pick
    metrics.second_lock_pick = state.second_lock_pick
    metrics.all_locked_pick = state.all_locked_pick

    # Count unwanted locks (locked resonances not in target archetype)
    for res, _ in state.locked_slots:
        if res not in target_resonances:
            metrics.unwanted_locks += 1

    # Distinct locked resonances
    metrics.distinct_locked_res = len(state.locked_resonances())

    # Wasted symbols: symbols going to resonances with 2 locks already
    maxed_res = set()
    for res in Resonance:
        lock_count = sum(1 for r, _ in state.locked_slots if r == res)
        if lock_count >= 2:
            maxed_res.add(res)
    for card in state.picks:
        ws = card.weighted_symbols
        for res, count in ws.items():
            if res in maxed_res:
                metrics.wasted_symbols += count

    # Deck S/A concentration
    sa_in_deck = sum(1 for c in state.picks if is_sa_tier(c, target_arch))
    metrics.deck_sa_concentration = sa_in_deck / len(state.picks) if state.picks else 0

    return metrics


# ---------------------------------------------------------------------------
# Aggregate metrics
# ---------------------------------------------------------------------------

@dataclass
class CompositionResults:
    """Aggregated results for one pattern composition."""
    name: str

    # Lock timing
    first_lock_avg: float = 0.0
    second_lock_avg: float = 0.0
    all_locked_avg: float = 0.0
    all_locked_pct: float = 0.0

    # Unwanted locks
    unwanted_lock_rate: float = 0.0  # avg unwanted locks per draft
    drafts_with_unwanted_pct: float = 0.0  # % of drafts with any unwanted lock

    # S/A per pack
    early_diversity: float = 0.0
    late_sa: float = 0.0
    late_cf: float = 0.0

    # Convergence pick (first where avg SA >= 2)
    convergence_pick: int = 30

    # Distinct locked resonances
    distinct_locked_res_avg: float = 0.0

    # Genuine choice rate
    genuine_choice_rate: float = 0.0

    # Wasted symbols
    wasted_symbols_avg: float = 0.0

    # Deck concentration
    deck_concentration: float = 0.0

    # Convergence curve (pick -> avg SA)
    convergence_curve: dict = field(default_factory=dict)


def run_composition(name: str, composition: dict,
                    num_drafts: int = 1000) -> CompositionResults:
    """Run full simulation for one pattern composition."""
    pool = generate_pool(composition)
    results = CompositionResults(name=name)

    first_locks = []
    second_locks = []
    all_locked_count = 0
    all_locked_picks = []
    unwanted_lock_counts = []
    drafts_with_unwanted = 0

    early_div_all = []
    late_sa_all = []
    late_cf_all = []
    distinct_res_all = []
    genuine_choice_all = []
    total_post_commit_all = []
    wasted_all = []
    concentration_all = []

    # Per-pick SA for convergence curve
    pick_sa = defaultdict(list)

    for _ in range(num_drafts):
        target_arch = random.randint(0, NUM_ARCHETYPES - 1)
        m = simulate_draft(pool, target_arch)

        if m.first_lock_pick > 0:
            first_locks.append(m.first_lock_pick)
        if m.second_lock_pick > 0:
            second_locks.append(m.second_lock_pick)
        if m.all_locked_pick > 0:
            all_locked_count += 1
            all_locked_picks.append(m.all_locked_pick)

        unwanted_lock_counts.append(m.unwanted_locks)
        if m.unwanted_locks > 0:
            drafts_with_unwanted += 1

        early_div_all.extend(m.early_diversity_vals)
        late_sa_all.extend(m.late_sa_vals)
        late_cf_all.extend(m.late_cf_vals)
        distinct_res_all.append(m.distinct_locked_res)
        genuine_choice_all.append(m.genuine_choice_count)
        total_post_commit_all.append(m.total_post_commit_picks)
        wasted_all.append(m.wasted_symbols)
        concentration_all.append(m.deck_sa_concentration)

        for i, sa in enumerate(m.sa_per_pack):
            pick_sa[i + 1].append(sa)

    # Aggregate
    results.first_lock_avg = statistics.mean(first_locks) if first_locks else 31
    results.second_lock_avg = statistics.mean(second_locks) if second_locks else 31
    results.all_locked_pct = all_locked_count / num_drafts * 100
    results.all_locked_avg = statistics.mean(all_locked_picks) if all_locked_picks else 0

    results.unwanted_lock_rate = statistics.mean(unwanted_lock_counts)
    results.drafts_with_unwanted_pct = drafts_with_unwanted / num_drafts * 100

    results.early_diversity = statistics.mean(early_div_all) if early_div_all else 0
    results.late_sa = statistics.mean(late_sa_all) if late_sa_all else 0
    results.late_cf = statistics.mean(late_cf_all) if late_cf_all else 0

    results.distinct_locked_res_avg = statistics.mean(distinct_res_all) if distinct_res_all else 0

    total_genuine = sum(genuine_choice_all)
    total_post = sum(total_post_commit_all)
    results.genuine_choice_rate = total_genuine / total_post * 100 if total_post > 0 else 0

    results.wasted_symbols_avg = statistics.mean(wasted_all) if wasted_all else 0
    results.deck_concentration = statistics.mean(concentration_all) if concentration_all else 0

    # Convergence curve
    for p in range(1, NUM_PICKS + 1):
        if p in pick_sa:
            results.convergence_curve[p] = statistics.mean(pick_sa[p])

    # Find convergence pick (first where avg SA >= 2.0)
    for p in range(1, NUM_PICKS + 1):
        if results.convergence_curve.get(p, 0) >= 2.0:
            results.convergence_pick = p
            break

    return results


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    random.seed(42)

    print("=" * 100)
    print("  SYMBOL PATTERN COMPOSITION SIMULATION — Lane Locking (3, 8)")
    print("  Testing how specific symbol patterns affect draft decisions")
    print("=" * 100)

    all_results = []

    for name, composition in COMPOSITIONS.items():
        print(f"\nRunning: {name}...")
        # Show what patterns this uses
        for pname, frac in composition.items():
            print(f"  {pname}: {frac*100:.0f}%")
        r = run_composition(name, composition, num_drafts=1000)
        all_results.append(r)

    # -----------------------------------------------------------------------
    # Table 1: Lock Timing
    # -----------------------------------------------------------------------
    print("\n" + "=" * 100)
    print("  TABLE 1: LOCK TIMING")
    print("=" * 100)
    print(f"{'Composition':<25} {'1st Lock':>10} {'2nd Lock':>10} "
          f"{'All Locked%':>12} {'All Lock Avg':>12}")
    print("-" * 75)
    for r in all_results:
        all_str = f"{r.all_locked_avg:.1f}" if r.all_locked_avg > 0 else "N/A"
        print(f"{r.name:<25} {r.first_lock_avg:>10.2f} {r.second_lock_avg:>10.2f} "
              f"{r.all_locked_pct:>11.1f}% {all_str:>12}")

    # -----------------------------------------------------------------------
    # Table 2: Core Metrics
    # -----------------------------------------------------------------------
    print("\n" + "=" * 100)
    print("  TABLE 2: CORE METRICS (archetype-committed, 1000 drafts)")
    print("=" * 100)
    print(f"{'Composition':<25} {'Early Div':>10} {'Late S/A':>10} "
          f"{'Late C/F':>10} {'Conv Pick':>10} {'Deck Conc%':>10}")
    print("-" * 80)
    for r in all_results:
        print(f"{r.name:<25} {r.early_diversity:>10.2f} {r.late_sa:>10.2f} "
              f"{r.late_cf:>10.2f} {r.convergence_pick:>10} "
              f"{r.deck_concentration * 100:>9.1f}%")

    # -----------------------------------------------------------------------
    # Table 3: Decision Quality Metrics
    # -----------------------------------------------------------------------
    print("\n" + "=" * 100)
    print("  TABLE 3: DECISION QUALITY METRICS")
    print("=" * 100)
    print(f"{'Composition':<25} {'Unwanted%':>10} {'Unwanted/d':>10} "
          f"{'DistRes':>8} {'Choice%':>10} {'Wasted':>8}")
    print("-" * 78)
    for r in all_results:
        print(f"{r.name:<25} {r.drafts_with_unwanted_pct:>9.1f}% "
              f"{r.unwanted_lock_rate:>10.2f} "
              f"{r.distinct_locked_res_avg:>8.2f} "
              f"{r.genuine_choice_rate:>9.1f}% "
              f"{r.wasted_symbols_avg:>8.1f}")

    # -----------------------------------------------------------------------
    # Table 4: Convergence Curves
    # -----------------------------------------------------------------------
    print("\n" + "=" * 100)
    print("  TABLE 4: CONVERGENCE CURVES (S/A per pack at each pick)")
    print("=" * 100)
    header = f"{'Comp':<6}"
    for p in [1, 2, 3, 4, 5, 6, 8, 10, 12, 15, 20, 25, 30]:
        header += f" {'P' + str(p):>5}"
    print(header)
    print("-" * len(header))
    for r in all_results:
        line = f"{r.name[:5]:<6}"
        for p in [1, 2, 3, 4, 5, 6, 8, 10, 12, 15, 20, 25, 30]:
            val = r.convergence_curve.get(p, 0)
            line += f" {val:>5.2f}"
        print(line)

    # -----------------------------------------------------------------------
    # Table 5: Target Scorecard
    # -----------------------------------------------------------------------
    print("\n" + "=" * 100)
    print("  TABLE 5: TARGET SCORECARD")
    print("=" * 100)
    print(f"{'Metric':<35} {'Target':<10}", end="")
    for r in all_results:
        print(f" {r.name[:8]:>10}", end="")
    print()
    print("-" * (55 + 11 * len(all_results)))

    def check_print(metric_name, target_str, values, check_fn):
        print(f"{metric_name:<35} {target_str:<10}", end="")
        for v in values:
            passed = check_fn(v)
            marker = "PASS" if passed else "FAIL"
            print(f" {v:>6.2f} {marker}", end="")
        print()

    check_print("Early diversity (>=3)",
                ">=3",
                [r.early_diversity for r in all_results],
                lambda v: v >= 3)
    check_print("Late S/A per pack (>=2)",
                ">=2",
                [r.late_sa for r in all_results],
                lambda v: v >= 2)
    check_print("Late C/F per pack (>=0.5)",
                ">=0.5",
                [r.late_cf for r in all_results],
                lambda v: v >= 0.5)
    check_print("Convergence pick (5-8)",
                "5-8",
                [float(r.convergence_pick) for r in all_results],
                lambda v: 5 <= v <= 8)
    check_print("Genuine choice rate",
                "high",
                [r.genuine_choice_rate for r in all_results],
                lambda v: v >= 30)  # 30%+ is good
    check_print("Unwanted lock rate",
                "low",
                [r.drafts_with_unwanted_pct for r in all_results],
                lambda v: v <= 20)
    check_print("Wasted symbols",
                "low",
                [r.wasted_symbols_avg for r in all_results],
                lambda v: v <= 15)

    # -----------------------------------------------------------------------
    # Analysis: Per-pattern symbol contribution summary
    # -----------------------------------------------------------------------
    print("\n" + "=" * 100)
    print("  PATTERN SYMBOL CONTRIBUTION REFERENCE")
    print("=" * 100)
    patterns = [
        ("mono_primary",    "[P]",            "+2P"),
        ("mono_secondary",  "[S]",            "+2S"),
        ("standard_dual",   "[P, S]",         "+2P, +1S"),
        ("double_primary",  "[P, P]",         "+3P"),
        ("secondary_led",   "[S, P]",         "+2S, +1P"),
        ("cross_bridge",    "[P, O]",         "+2P, +1O"),
        ("deep_commit",     "[P, P, S]",      "+3P, +1S"),
        ("balanced_triple", "[P, S, S]",      "+2P, +2S"),
        ("triple_splash",   "[P, S, O]",      "+2P, +1S, +1O"),
        ("triple_primary",  "[P, P, P]",      "+4P"),
    ]
    print(f"{'Pattern':<20} {'Symbols':<15} {'Contribution':<20} {'Total Weight':>12}")
    print("-" * 70)
    for pname, sym_str, contrib in patterns:
        total = sum(2 if i == 0 else 1 for i in range(len(sym_str.split(","))))
        # Count actual symbols
        if "P, P, S" in sym_str:
            total = 4
        elif "P, S, S" in sym_str:
            total = 4
        elif "P, S, O" in sym_str:
            total = 4
        elif "P, P, P" in sym_str:
            total = 4
        elif "P, P" in sym_str:
            total = 3
        elif "P, S" in sym_str or "S, P" in sym_str or "P, O" in sym_str:
            total = 3
        elif sym_str in ("[P]", "[S]"):
            total = 2
        print(f"{pname:<20} {sym_str:<15} {contrib:<20} {total:>12}")

    # -----------------------------------------------------------------------
    # Summary
    # -----------------------------------------------------------------------
    print("\n" + "=" * 100)
    print("  SUMMARY: KEY FINDINGS")
    print("=" * 100)

    # Rank by a combined score
    for r in all_results:
        score = 0
        if r.early_diversity >= 3: score += 1
        if r.late_sa >= 2: score += 2
        if r.late_cf >= 0.5: score += 1
        if 5 <= r.convergence_pick <= 8: score += 1
        # Genuine choice is important
        score += r.genuine_choice_rate / 100 * 2
        # Penalize unwanted locks
        if r.drafts_with_unwanted_pct > 30: score -= 1
        print(f"  {r.name}: combined_score={score:.2f}, "
              f"genuine_choice={r.genuine_choice_rate:.1f}%, "
              f"late_SA={r.late_sa:.2f}, unwanted_locks={r.drafts_with_unwanted_pct:.1f}%")

    print()
    print("  Genuine choice rate = % of post-commitment picks where the pack")
    print("  contains 2+ S/A cards with DIFFERENT pattern types (meaning the")
    print("  player must evaluate which symbol contribution is better).")


if __name__ == "__main__":
    main()
