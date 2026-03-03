#!/usr/bin/env python3
"""
Pool Distribution Simulation for Lane Locking Algorithm.

Investigates: What ratio of 1-symbol, 2-symbol, and 3-symbol cards produces
the best draft experience under the Lane Locking algorithm?

Lane Locking: Your pack has 4 slots; when your symbol count in a resonance
first reaches 3, one open slot locks to that resonance; when it reaches 8,
a second slot locks. Primary symbol counts as 2, secondary/tertiary as 1.
Max 4 locked slots.
"""

import random
import statistics
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict
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

# The 8 archetypes on a circle
ARCHETYPES = [
    ("Flash",        Resonance.ZEPHYR, Resonance.EMBER),   # 1
    ("Blink",        Resonance.EMBER,  Resonance.ZEPHYR),  # 2
    ("Storm",        Resonance.EMBER,  Resonance.STONE),   # 3
    ("Self-Discard", Resonance.STONE,  Resonance.EMBER),   # 4
    ("Self-Mill",    Resonance.STONE,  Resonance.TIDE),    # 5
    ("Sacrifice",    Resonance.TIDE,   Resonance.STONE),   # 6
    ("Warriors",     Resonance.TIDE,   Resonance.ZEPHYR),  # 7
    ("Ramp",         Resonance.ZEPHYR, Resonance.TIDE),    # 8
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
NUM_ARCHETYPES = 8


def circle_distance(i: int, j: int) -> int:
    """Minimum distance on the 8-archetype circle."""
    d = abs(i - j)
    return min(d, NUM_ARCHETYPES - d)


def compute_fitness(card_arch_idx: int, player_arch_idx: int) -> Tier:
    """Compute fitness tier of a card from archetype i for a player in archetype j."""
    if card_arch_idx == player_arch_idx:
        return Tier.S

    dist = circle_distance(card_arch_idx, player_arch_idx)

    # Adjacent archetypes sharing primary resonance
    card_primary = ARCHETYPES[card_arch_idx][1]
    player_primary = ARCHETYPES[player_arch_idx][1]
    card_secondary = ARCHETYPES[card_arch_idx][2]
    player_secondary = ARCHETYPES[player_arch_idx][2]

    if dist == 1:
        # Adjacent: check if they share primary resonance
        if card_primary == player_primary:
            return Tier.A
        # They share a resonance but through secondary
        return Tier.B
    elif dist == 2:
        # Check if any resonance overlap
        card_res = {card_primary, card_secondary}
        player_res = {player_primary, player_secondary}
        if card_res & player_res:
            return Tier.B
        return Tier.C
    elif dist == 3:
        return Tier.C
    else:  # dist == 4 (opposite)
        return Tier.F


@dataclass
class SimCard:
    id: int
    symbols: list  # list of Resonance, 0-3 elements
    archetype_idx: int  # index into ARCHETYPES, -1 for generic
    fitness: dict = field(default_factory=dict)  # arch_idx -> Tier

    @property
    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None

    @property
    def weighted_symbols(self) -> dict:
        """Return resonance -> weighted count for this card's symbols."""
        counts = defaultdict(int)
        for i, sym in enumerate(self.symbols):
            counts[sym] += 2 if i == 0 else 1
        return dict(counts)


# ---------------------------------------------------------------------------
# Card pool generation
# ---------------------------------------------------------------------------

def generate_pool(pct_1sym: float, pct_2sym: float, pct_3sym: float,
                  num_generic: int = 36, total: int = 360) -> list[SimCard]:
    """Generate a card pool with the given symbol distribution.

    pct_1sym, pct_2sym, pct_3sym: percentages of non-generic cards (should sum to ~100).
    """
    cards = []
    card_id = 0

    # Generic cards
    for _ in range(num_generic):
        card = SimCard(id=card_id, symbols=[], archetype_idx=-1)
        # Generic cards are B-tier in all archetypes
        for j in range(NUM_ARCHETYPES):
            card.fitness[j] = Tier.B
        cards.append(card)
        card_id += 1

    # Archetype cards
    non_generic = total - num_generic  # 324
    per_archetype = non_generic // NUM_ARCHETYPES  # ~40

    for arch_idx in range(NUM_ARCHETYPES):
        arch_name, primary, secondary = ARCHETYPES[arch_idx]

        # Determine how many 1/2/3-symbol cards for this archetype
        n1 = round(per_archetype * pct_1sym / 100)
        n3 = round(per_archetype * pct_3sym / 100)
        n2 = per_archetype - n1 - n3

        # Generate 1-symbol cards: [Primary]
        for _ in range(n1):
            card = SimCard(id=card_id, symbols=[primary], archetype_idx=arch_idx)
            cards.append(card)
            card_id += 1

        # Generate 2-symbol cards: mix of [Primary, Secondary] and [Primary, Primary]
        for i in range(n2):
            if i % 3 == 0:
                # 1/3 are [Primary, Primary]
                syms = [primary, primary]
            else:
                # 2/3 are [Primary, Secondary]
                syms = [primary, secondary]
            card = SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx)
            cards.append(card)
            card_id += 1

        # Generate 3-symbol cards: mix
        for i in range(n3):
            if i % 3 == 0:
                syms = [primary, primary, secondary]
            elif i % 3 == 1:
                syms = [primary, secondary, secondary]
            else:
                syms = [primary, secondary, primary]
            card = SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx)
            cards.append(card)
            card_id += 1

    # Compute fitness for all non-generic cards
    for card in cards:
        if card.archetype_idx >= 0:
            for j in range(NUM_ARCHETYPES):
                card.fitness[j] = compute_fitness(card.archetype_idx, j)

    return cards


# ---------------------------------------------------------------------------
# Lane Locking Algorithm
# ---------------------------------------------------------------------------

@dataclass
class DraftState:
    """State of one draft in progress."""
    counters: dict = field(default_factory=lambda: {r: 0 for r in Resonance})
    locked_slots: list = field(default_factory=list)  # list of (Resonance, threshold)
    picks: list = field(default_factory=list)
    target_archetype: int = -1

    # Tracking
    lock_picks: list = field(default_factory=list)  # pick numbers when locks happened
    first_lock_pick: int = -1
    second_lock_pick: int = -1
    all_locked_pick: int = -1

    @property
    def num_locked(self) -> int:
        return len(self.locked_slots)

    @property
    def num_open(self) -> int:
        return 4 - self.num_locked

    def locked_resonances(self) -> set:
        return {r for r, _ in self.locked_slots}

    def locked_resonance_counts(self) -> dict:
        counts = defaultdict(int)
        for r, _ in self.locked_slots:
            counts[r] += 1
        return dict(counts)


def fill_pack(state: DraftState, pool: list[SimCard]) -> list[SimCard]:
    """Fill a 4-card pack according to Lane Locking rules."""
    pack = []

    # Build index by primary resonance for locked slots
    by_primary = defaultdict(list)
    for card in pool:
        if card.primary_resonance:
            by_primary[card.primary_resonance].append(card)

    # Fill locked slots
    for res, _ in state.locked_slots:
        candidates = by_primary.get(res, [])
        if candidates:
            pack.append(random.choice(candidates))
        else:
            pack.append(random.choice(pool))

    # Fill open slots with random cards from full pool
    for _ in range(state.num_open):
        pack.append(random.choice(pool))

    return pack


def update_state(state: DraftState, card: SimCard, pick_num: int):
    """Update draft state after picking a card."""
    state.picks.append(card)

    # Update counters
    ws = card.weighted_symbols
    for res, count in ws.items():
        state.counters[res] += count

    # Check thresholds
    for res in Resonance:
        count = state.counters[res]

        # Check if this resonance already has locks
        existing_locks = sum(1 for r, _ in state.locked_slots if r == res)

        if state.num_locked >= 4:
            break

        if count >= 3 and existing_locks == 0:
            state.locked_slots.append((res, 3))
            state.lock_picks.append(pick_num)
            if state.first_lock_pick == -1:
                state.first_lock_pick = pick_num
            elif state.second_lock_pick == -1:
                state.second_lock_pick = pick_num

            if state.num_locked >= 4:
                if state.all_locked_pick == -1:
                    state.all_locked_pick = pick_num
                break

        if count >= 8 and existing_locks == 1:
            state.locked_slots.append((res, 8))
            state.lock_picks.append(pick_num)
            if state.second_lock_pick == -1:
                state.second_lock_pick = pick_num

            if state.num_locked >= 4:
                if state.all_locked_pick == -1:
                    state.all_locked_pick = pick_num
                break


def card_fitness_tier(card: SimCard, arch_idx: int) -> Tier:
    return card.fitness.get(arch_idx, Tier.F)


def is_sa_tier(card: SimCard, arch_idx: int) -> bool:
    tier = card_fitness_tier(card, arch_idx)
    return tier in (Tier.S, Tier.A)


# ---------------------------------------------------------------------------
# Archetype-committed player strategy
# ---------------------------------------------------------------------------

def pick_card_committed(pack: list[SimCard], state: DraftState, pick_num: int) -> SimCard:
    """Archetype-committed player: choose based on target archetype.

    For first 2 picks, choose randomly (haven't committed yet).
    After that, choose the card with highest fitness for target archetype.
    """
    if pick_num <= 2 and state.target_archetype == -1:
        # Pick randomly early
        return random.choice(pack)

    arch = state.target_archetype
    tier_priority = [Tier.S, Tier.A, Tier.B, Tier.C, Tier.F]
    for tier in tier_priority:
        candidates = [c for c in pack if card_fitness_tier(c, arch) == tier]
        if candidates:
            return random.choice(candidates)

    return random.choice(pack)


# ---------------------------------------------------------------------------
# Simulation
# ---------------------------------------------------------------------------

def simulate_draft(pool: list[SimCard], target_arch: int, num_picks: int = 30) -> DraftState:
    """Simulate a single draft with an archetype-committed player."""
    state = DraftState()
    state.target_archetype = target_arch

    for pick_num in range(1, num_picks + 1):
        pack = fill_pack(state, pool)
        card = pick_card_committed(pack, state, pick_num)
        update_state(state, card, pick_num)

    return state


@dataclass
class DistributionMetrics:
    """Aggregated metrics for one symbol distribution across many drafts."""
    name: str
    pct_1: float
    pct_2: float
    pct_3: float

    # Lock timing
    first_lock_avg: float = 0.0
    first_lock_median: float = 0.0
    second_lock_avg: float = 0.0
    second_lock_median: float = 0.0
    all_locked_avg: float = 0.0
    all_locked_pct: float = 0.0  # % of drafts where all 4 lock

    # S/A per pack at various picks
    sa_at_pick: dict = field(default_factory=dict)  # pick_num -> avg SA cards

    # Convergence curve (S/A per pack at each pick)
    convergence_curve: dict = field(default_factory=dict)

    # Double-lock: how often 2 slots locked to SAME resonance
    double_lock_pct: float = 0.0

    # Instant threshold on pick 1
    instant_threshold_pct: float = 0.0

    # Distinct locked resonances by pick 10
    distinct_res_at_10: float = 0.0

    # Wasted symbols
    wasted_symbols_avg: float = 0.0

    # Early diversity (unique archs with S/A in pack, picks 1-5)
    early_diversity: float = 0.0

    # Late S/A (picks 6+)
    late_sa: float = 0.0

    # Convergence pick (first pick where avg SA >= 2)
    convergence_pick: int = 30


def measure_pack_sa(pack: list[SimCard], arch_idx: int) -> int:
    """Count S/A tier cards in a pack for a given archetype."""
    return sum(1 for c in pack if is_sa_tier(c, arch_idx))


def measure_pack_unique_sa_archs(pack: list[SimCard]) -> int:
    """Count how many distinct archetypes have at least one S/A card in this pack."""
    archs_with_sa = set()
    for card in pack:
        for arch_idx in range(NUM_ARCHETYPES):
            if is_sa_tier(card, arch_idx):
                archs_with_sa.add(arch_idx)
    return len(archs_with_sa)


def run_distribution(name: str, pct_1: float, pct_2: float, pct_3: float,
                     num_drafts: int = 1000, num_picks: int = 30) -> DistributionMetrics:
    """Run full simulation for one distribution."""
    pool = generate_pool(pct_1, pct_2, pct_3)
    metrics = DistributionMetrics(name=name, pct_1=pct_1, pct_2=pct_2, pct_3=pct_3)

    first_locks = []
    second_locks = []
    all_locked_count = 0
    all_locked_picks = []
    double_lock_count = 0
    instant_threshold_count = 0

    # Per-pick SA tracking
    pick_sa_totals = defaultdict(list)  # pick_num -> list of SA counts
    pick_diversity_totals = defaultdict(list)

    distinct_res_at_10_list = []
    wasted_symbols_list = []

    for draft_num in range(num_drafts):
        target_arch = random.randint(0, NUM_ARCHETYPES - 1)

        # We need pick-by-pick data, so we run the draft manually
        state = DraftState()
        state.target_archetype = target_arch

        for pick_num in range(1, num_picks + 1):
            pack = fill_pack(state, pool)

            # Measure SA in this pack
            sa_count = measure_pack_sa(pack, target_arch)
            pick_sa_totals[pick_num].append(sa_count)

            # Measure diversity (unique archs with S/A)
            diversity = measure_pack_unique_sa_archs(pack)
            pick_diversity_totals[pick_num].append(diversity)

            # Pick a card
            card = pick_card_committed(pack, state, pick_num)
            update_state(state, card, pick_num)

        # Record lock timing
        if state.first_lock_pick > 0:
            first_locks.append(state.first_lock_pick)
        if state.second_lock_pick > 0:
            second_locks.append(state.second_lock_pick)
        if state.all_locked_pick > 0:
            all_locked_count += 1
            all_locked_picks.append(state.all_locked_pick)

        # Check double-lock (2 slots locked to same resonance)
        res_counts = state.locked_resonance_counts()
        if any(v >= 2 for v in res_counts.values()):
            double_lock_count += 1

        # Check instant threshold on pick 1
        if state.first_lock_pick == 1:
            instant_threshold_count += 1

        # Distinct resonances locked by pick 10
        locks_by_10 = set()
        for lp_idx, lp in enumerate(state.lock_picks):
            if lp <= 10 and lp_idx < len(state.locked_slots):
                locks_by_10.add(state.locked_slots[lp_idx][0])
        distinct_res_at_10_list.append(len(locks_by_10))

        # Wasted symbols: symbols in resonances that are already at 2 locks
        wasted = 0
        maxed_res = set()
        for res in Resonance:
            lock_count = sum(1 for r, _ in state.locked_slots if r == res)
            if lock_count >= 2:
                maxed_res.add(res)
        for card in state.picks:
            ws = card.weighted_symbols
            for res, count in ws.items():
                if res in maxed_res:
                    wasted += count
        wasted_symbols_list.append(wasted)

    # Compute aggregated metrics
    if first_locks:
        metrics.first_lock_avg = statistics.mean(first_locks)
        metrics.first_lock_median = statistics.median(first_locks)
    else:
        metrics.first_lock_avg = 31  # never locked
        metrics.first_lock_median = 31

    if second_locks:
        metrics.second_lock_avg = statistics.mean(second_locks)
        metrics.second_lock_median = statistics.median(second_locks)
    else:
        metrics.second_lock_avg = 31
        metrics.second_lock_median = 31

    metrics.all_locked_pct = all_locked_count / num_drafts * 100
    if all_locked_picks:
        metrics.all_locked_avg = statistics.mean(all_locked_picks)

    # SA at specific picks
    for pick_num in [5, 10, 15, 20, 25, 30]:
        if pick_num in pick_sa_totals:
            metrics.sa_at_pick[pick_num] = statistics.mean(pick_sa_totals[pick_num])

    # Convergence curve (every pick)
    for pick_num in range(1, num_picks + 1):
        if pick_num in pick_sa_totals:
            metrics.convergence_curve[pick_num] = statistics.mean(pick_sa_totals[pick_num])

    # Find convergence pick (first pick where avg SA >= 2.0)
    for pick_num in range(1, num_picks + 1):
        if metrics.convergence_curve.get(pick_num, 0) >= 2.0:
            metrics.convergence_pick = pick_num
            break

    metrics.double_lock_pct = double_lock_count / num_drafts * 100
    metrics.instant_threshold_pct = instant_threshold_count / num_drafts * 100

    metrics.distinct_res_at_10 = statistics.mean(distinct_res_at_10_list) if distinct_res_at_10_list else 0
    metrics.wasted_symbols_avg = statistics.mean(wasted_symbols_list) if wasted_symbols_list else 0

    # Early diversity (picks 1-5)
    early_div = []
    for p in range(1, 6):
        if p in pick_diversity_totals:
            early_div.extend(pick_diversity_totals[p])
    metrics.early_diversity = statistics.mean(early_div) if early_div else 0

    # Late S/A (picks 6-30)
    late_sa_vals = []
    for p in range(6, 31):
        if p in pick_sa_totals:
            late_sa_vals.extend(pick_sa_totals[p])
    metrics.late_sa = statistics.mean(late_sa_vals) if late_sa_vals else 0

    return metrics


# ---------------------------------------------------------------------------
# Main simulation
# ---------------------------------------------------------------------------

DISTRIBUTIONS = [
    ("All 1-sym (100/0/0)",        100,  0,  0),
    ("Heavy 1-sym (70/20/10)",      70, 20, 10),
    ("Moderate 1-sym (50/35/15)",   50, 35, 15),
    ("Balanced (33/34/33)",         33, 34, 33),
    ("Default (25/55/20)",          25, 55, 20),
    ("Heavy 2-sym (10/80/10)",      10, 80, 10),
    ("Heavy 3-sym (10/30/60)",      10, 30, 60),
    ("All 3-sym (0/0/100)",          0,  0, 100),
]


def main():
    random.seed(42)  # Reproducibility
    results = []

    for name, p1, p2, p3 in DISTRIBUTIONS:
        print(f"Running: {name}...")
        m = run_distribution(name, p1, p2, p3, num_drafts=1000, num_picks=30)
        results.append(m)

    # Print results
    print("\n" + "=" * 120)
    print("POOL DISTRIBUTION SIMULATION RESULTS — Lane Locking (3, 8)")
    print("=" * 120)

    # Table 1: Lock Timing
    print("\n### LOCK TIMING")
    print(f"{'Distribution':<30} {'1st Lock Avg':>12} {'1st Lock Med':>12} {'2nd Lock Avg':>12} {'2nd Lock Med':>12} {'All Locked %':>12} {'All Lock Avg':>12}")
    print("-" * 102)
    for m in results:
        all_lock_str = f"{m.all_locked_avg:.1f}" if m.all_locked_avg > 0 else "N/A"
        print(f"{m.name:<30} {m.first_lock_avg:>12.2f} {m.first_lock_median:>12.1f} {m.second_lock_avg:>12.2f} {m.second_lock_median:>12.1f} {m.all_locked_pct:>11.1f}% {all_lock_str:>12}")

    # Table 2: S/A Cards Per Pack at Key Picks
    print("\n### S/A CARDS PER PACK AT KEY PICKS")
    print(f"{'Distribution':<30} {'Pick 5':>8} {'Pick 10':>8} {'Pick 15':>8} {'Pick 20':>8} {'Pick 25':>8} {'Pick 30':>8} {'Late S/A':>10} {'Conv. Pick':>10}")
    print("-" * 118)
    for m in results:
        print(f"{m.name:<30} {m.sa_at_pick.get(5, 0):>8.2f} {m.sa_at_pick.get(10, 0):>8.2f} {m.sa_at_pick.get(15, 0):>8.2f} {m.sa_at_pick.get(20, 0):>8.2f} {m.sa_at_pick.get(25, 0):>8.2f} {m.sa_at_pick.get(30, 0):>8.2f} {m.late_sa:>10.2f} {m.convergence_pick:>10}")

    # Table 3: Special Metrics
    print("\n### SPECIAL METRICS")
    print(f"{'Distribution':<30} {'Double Lock%':>12} {'Instant T3%':>12} {'Res@10':>8} {'Wasted Sym':>10} {'Early Div':>10}")
    print("-" * 88)
    for m in results:
        print(f"{m.name:<30} {m.double_lock_pct:>11.1f}% {m.instant_threshold_pct:>11.1f}% {m.distinct_res_at_10:>8.2f} {m.wasted_symbols_avg:>10.1f} {m.early_diversity:>10.2f}")

    # Table 4: Convergence Curves (sampled every 3 picks)
    print("\n### CONVERGENCE CURVES (S/A cards per pack)")
    header = f"{'Distribution':<30}"
    for p in range(1, 31):
        if p % 2 == 1 or p <= 6:
            header += f" {'P'+str(p):>5}"
    print(header)
    print("-" * len(header))
    for m in results:
        line = f"{m.name:<30}"
        for p in range(1, 31):
            if p % 2 == 1 or p <= 6:
                val = m.convergence_curve.get(p, 0)
                line += f" {val:>5.2f}"
        print(line)

    # Table 5: SA by phase (early, mid, late)
    print("\n### S/A BY DRAFT PHASE")
    print(f"{'Distribution':<30} {'P1-5':>8} {'P6-10':>8} {'P11-15':>8} {'P16-20':>8} {'P21-25':>8} {'P26-30':>8} {'SA Trend':>10}")
    print("-" * 95)
    for m in results:
        phases = []
        for start, end in [(1,5), (6,10), (11,15), (16,20), (21,25), (26,30)]:
            vals = [m.convergence_curve.get(p, 0) for p in range(start, end+1)]
            phases.append(statistics.mean(vals))
        # Trend: difference between early-post-lock (P6-10) and late (P26-30)
        trend = phases[5] - phases[1]  # positive = climbing, negative = declining
        trend_str = f"+{trend:.2f}" if trend >= 0 else f"{trend:.2f}"
        print(f"{m.name:<30} {phases[0]:>8.2f} {phases[1]:>8.2f} {phases[2]:>8.2f} {phases[3]:>8.2f} {phases[4]:>8.2f} {phases[5]:>8.2f} {trend_str:>10}")

    # Summary analysis
    print("\n### ANALYSIS SUMMARY")
    print()
    for m in results:
        weighted_per_pick = (m.pct_1 / 100) * 2.0 + (m.pct_2 / 100) * 3.0 + (m.pct_3 / 100) * 4.0
        print(f"{m.name}: Avg weighted symbols/pick = {weighted_per_pick:.1f}")
        print(f"  First lock @ pick {m.first_lock_avg:.1f}, Second lock @ pick {m.second_lock_avg:.1f}")
        print(f"  Late S/A = {m.late_sa:.2f}, Convergence @ pick {m.convergence_pick}")
        print(f"  Double-lock = {m.double_lock_pct:.1f}%, Instant T3 = {m.instant_threshold_pct:.1f}%")
        print()


if __name__ == "__main__":
    main()
