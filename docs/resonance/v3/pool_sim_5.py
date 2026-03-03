#!/usr/bin/env python3
"""
Pool Simulation 5: Lane Locking Threshold Tuning

Investigates: How should the Lane Locking thresholds be tuned relative to the
pool design, and what is the ideal lock timing curve for a 30-pick roguelike
draft?

Systematically explores:
- Threshold pairs: (2,5), (3,8), (4,10), (4,12), (5,12), (5,15), (6,15), (8,20)
- Primary weights: 1, 2, 3
- With and without a third threshold (at 2x the second threshold)
- Symbol distributions: 25/55/20 and 50/35/15

For each configuration, measures:
- Average pick for first lock
- Average pick for second lock (same resonance)
- Average pick when all 4 slots are locked
- % of drafts where first lock happens on pick 1
- % of drafts where all locks happen before pick 10
- S/A cards per pack at picks 5, 10, 15, 20, 25
- "Decision quality": how often the player faces 2+ S/A cards to choose between
- Full convergence curve
"""

import random
import statistics
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict
from typing import Optional
import itertools

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

ARCHETYPES = [
    ("Flash",        Resonance.ZEPHYR, Resonance.EMBER),
    ("Blink",        Resonance.EMBER,  Resonance.ZEPHYR),
    ("Storm",        Resonance.EMBER,  Resonance.STONE),
    ("Self-Discard", Resonance.STONE,  Resonance.EMBER),
    ("Self-Mill",    Resonance.STONE,  Resonance.TIDE),
    ("Sacrifice",    Resonance.TIDE,   Resonance.STONE),
    ("Warriors",     Resonance.TIDE,   Resonance.ZEPHYR),
    ("Ramp",         Resonance.ZEPHYR, Resonance.TIDE),
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
NUM_ARCHETYPES = 8
NUM_PICKS = 30
PACK_SIZE = 4

def circle_distance(i: int, j: int) -> int:
    d = abs(i - j)
    return min(d, NUM_ARCHETYPES - d)

def compute_fitness(card_arch_idx: int, player_arch_idx: int) -> Tier:
    if card_arch_idx == player_arch_idx:
        return Tier.S
    dist = circle_distance(card_arch_idx, player_arch_idx)
    card_primary = ARCHETYPES[card_arch_idx][1]
    player_primary = ARCHETYPES[player_arch_idx][1]
    card_secondary = ARCHETYPES[card_arch_idx][2]
    player_secondary = ARCHETYPES[player_arch_idx][2]
    if dist == 1:
        if card_primary == player_primary:
            return Tier.A
        return Tier.B
    elif dist == 2:
        card_res = {card_primary, card_secondary}
        player_res = {player_primary, player_secondary}
        if card_res & player_res:
            return Tier.B
        return Tier.C
    elif dist == 3:
        return Tier.C
    else:
        return Tier.F


@dataclass
class SimCard:
    id: int
    symbols: list
    archetype_idx: int
    fitness: dict = field(default_factory=dict)

    @property
    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None

    def weighted_symbols(self, primary_weight: int = 2) -> dict:
        counts = defaultdict(int)
        for i, sym in enumerate(self.symbols):
            counts[sym] += primary_weight if i == 0 else 1
        return dict(counts)


def is_sa_tier(card: SimCard, arch_idx: int) -> bool:
    return card.fitness.get(arch_idx, Tier.F) in (Tier.S, Tier.A)


# ---------------------------------------------------------------------------
# Card pool generation
# ---------------------------------------------------------------------------

def generate_pool(pct_1sym: float, pct_2sym: float, pct_3sym: float,
                  num_generic: int = 36, total: int = 360) -> list:
    cards = []
    card_id = 0

    for _ in range(num_generic):
        card = SimCard(id=card_id, symbols=[], archetype_idx=-1)
        for j in range(NUM_ARCHETYPES):
            card.fitness[j] = Tier.B
        cards.append(card)
        card_id += 1

    non_generic = total - num_generic
    per_archetype = non_generic // NUM_ARCHETYPES

    for arch_idx in range(NUM_ARCHETYPES):
        _, primary, secondary = ARCHETYPES[arch_idx]
        n1 = round(per_archetype * pct_1sym / 100)
        n3 = round(per_archetype * pct_3sym / 100)
        n2 = per_archetype - n1 - n3

        for _ in range(n1):
            card = SimCard(id=card_id, symbols=[primary], archetype_idx=arch_idx)
            cards.append(card)
            card_id += 1

        for i in range(n2):
            if i % 3 == 0:
                syms = [primary, primary]
            else:
                syms = [primary, secondary]
            card = SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx)
            cards.append(card)
            card_id += 1

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

    for card in cards:
        if card.archetype_idx >= 0:
            for j in range(NUM_ARCHETYPES):
                card.fitness[j] = compute_fitness(card.archetype_idx, j)

    return cards


# ---------------------------------------------------------------------------
# Lane Locking Algorithm (generalized for N thresholds)
# ---------------------------------------------------------------------------

@dataclass
class DraftState:
    counters: dict = field(default_factory=lambda: {r: 0 for r in Resonance})
    locked_slots: list = field(default_factory=list)
    picks: list = field(default_factory=list)
    target_archetype: int = -1
    primary_weight: int = 2

    # Tracking
    lock_picks: list = field(default_factory=list)

    @property
    def num_locked(self) -> int:
        return len(self.locked_slots)

    @property
    def num_open(self) -> int:
        return PACK_SIZE - min(len(self.locked_slots), PACK_SIZE)


def fill_pack(state: DraftState, pool: list) -> list:
    pack = []
    by_primary = defaultdict(list)
    for card in pool:
        if card.primary_resonance:
            by_primary[card.primary_resonance].append(card)

    active_locked = state.locked_slots[:PACK_SIZE]
    for res in active_locked:
        candidates = by_primary.get(res, [])
        if candidates:
            pack.append(random.choice(candidates))
        else:
            pack.append(random.choice(pool))

    for _ in range(PACK_SIZE - len(active_locked)):
        pack.append(random.choice(pool))

    return pack


def update_state(state: DraftState, card: SimCard, pick_num: int,
                 thresholds: list):
    """Update draft state after picking a card.

    thresholds: sorted list of threshold values, e.g. [3, 8] or [4, 8, 16].
    Each resonance can trigger at most one lock per threshold value.
    """
    state.picks.append(card)

    ws = card.weighted_symbols(state.primary_weight)
    for res, count in ws.items():
        state.counters[res] += count

    if state.num_locked >= PACK_SIZE:
        return

    # Check thresholds for all resonances
    for res in Resonance:
        count = state.counters[res]
        existing_locks_for_res = sum(1 for r in state.locked_slots if r == res)

        for t_idx, threshold in enumerate(thresholds):
            if state.num_locked >= PACK_SIZE:
                return
            # This resonance can lock at threshold t_idx only if it has
            # exactly t_idx locks already (i.e., it passed all prior thresholds)
            if existing_locks_for_res == t_idx and count >= threshold:
                state.locked_slots.append(res)
                state.lock_picks.append(pick_num)
                existing_locks_for_res += 1


def pick_card_committed(pack: list, state: DraftState, pick_num: int) -> SimCard:
    if pick_num <= 2 and state.target_archetype == -1:
        return random.choice(pack)
    arch = state.target_archetype
    tier_priority = [Tier.S, Tier.A, Tier.B, Tier.C, Tier.F]
    for tier in tier_priority:
        candidates = [c for c in pack if c.fitness.get(arch, Tier.F) == tier]
        if candidates:
            return random.choice(candidates)
    return random.choice(pack)


# ---------------------------------------------------------------------------
# Configuration and metrics
# ---------------------------------------------------------------------------

@dataclass
class Config:
    name: str
    thresholds: list        # e.g. [3, 8] or [4, 8, 16]
    primary_weight: int     # 1, 2, or 3
    pct_1sym: float
    pct_2sym: float
    pct_3sym: float

    @property
    def dist_label(self) -> str:
        return f"{int(self.pct_1sym)}/{int(self.pct_2sym)}/{int(self.pct_3sym)}"

    @property
    def thresh_label(self) -> str:
        return "/".join(str(t) for t in self.thresholds)

    @property
    def full_label(self) -> str:
        return f"T({self.thresh_label}) W{self.primary_weight} D{self.dist_label}"


@dataclass
class Metrics:
    config: Config
    first_lock_avg: float = 0.0
    second_lock_same_res_avg: float = 0.0
    all_locked_avg: float = 0.0
    pct_first_lock_pick1: float = 0.0
    pct_all_locked_before_10: float = 0.0
    sa_at_picks: dict = field(default_factory=dict)
    decision_quality: dict = field(default_factory=dict)  # pick -> % with 2+ SA
    convergence_curve: dict = field(default_factory=dict)
    late_sa: float = 0.0
    early_diversity: float = 0.0
    convergence_pick: int = 31

    # Draft arc quality metrics
    pct_first_lock_picks1_3: float = 0.0   # exploration phase
    pct_first_lock_picks3_6: float = 0.0   # commitment phase
    pct_second_lock_picks6_10: float = 0.0 # solidification phase

    # Rating for the ideal curve
    arc_rating: str = ""


# ---------------------------------------------------------------------------
# Simulation
# ---------------------------------------------------------------------------

def run_config(config: Config, num_drafts: int = 1000) -> Metrics:
    pool = generate_pool(config.pct_1sym, config.pct_2sym, config.pct_3sym)
    metrics = Metrics(config=config)

    first_locks = []
    second_lock_same_res = []
    all_locked_picks = []
    first_lock_on_pick1 = 0
    all_locked_before_10 = 0

    first_lock_in_1_3 = 0
    first_lock_in_3_6 = 0
    second_lock_in_6_10 = 0

    pick_sa_totals = defaultdict(list)
    pick_diversity_totals = defaultdict(list)
    pick_decision_quality = defaultdict(list)  # 1 if 2+ SA, 0 otherwise

    for _ in range(num_drafts):
        target_arch = random.randint(0, NUM_ARCHETYPES - 1)
        state = DraftState()
        state.target_archetype = target_arch
        state.primary_weight = config.primary_weight

        for pick_num in range(1, NUM_PICKS + 1):
            pack = fill_pack(state, pool)

            sa_count = sum(1 for c in pack if is_sa_tier(c, target_arch))
            pick_sa_totals[pick_num].append(sa_count)

            unique_sa_archs = set()
            for card in pack:
                for arch_idx in range(NUM_ARCHETYPES):
                    if is_sa_tier(card, arch_idx):
                        unique_sa_archs.add(arch_idx)
            pick_diversity_totals[pick_num].append(len(unique_sa_archs))

            pick_decision_quality[pick_num].append(1 if sa_count >= 2 else 0)

            card = pick_card_committed(pack, state, pick_num)
            update_state(state, card, pick_num, config.thresholds)

        # Analyze lock timing
        if state.lock_picks:
            first_pick = state.lock_picks[0]
            first_locks.append(first_pick)
            if first_pick == 1:
                first_lock_on_pick1 += 1
            if 1 <= first_pick <= 3:
                first_lock_in_1_3 += 1
            if 3 <= first_pick <= 6:
                first_lock_in_3_6 += 1

        # Find second lock for the SAME resonance as the first lock
        if len(state.locked_slots) >= 2:
            first_res = state.locked_slots[0]
            for i in range(1, len(state.locked_slots)):
                if state.locked_slots[i] == first_res:
                    second_lock_same_res.append(state.lock_picks[i])
                    if 6 <= state.lock_picks[i] <= 10:
                        second_lock_in_6_10 += 1
                    break

        if len(state.locked_slots) >= PACK_SIZE:
            # Find the pick when the 4th slot locked
            if len(state.lock_picks) >= PACK_SIZE:
                all_pick = state.lock_picks[PACK_SIZE - 1]
                all_locked_picks.append(all_pick)
                if all_pick < 10:
                    all_locked_before_10 += 1

    # Aggregate
    metrics.first_lock_avg = statistics.mean(first_locks) if first_locks else 31
    metrics.second_lock_same_res_avg = (
        statistics.mean(second_lock_same_res) if second_lock_same_res else 31
    )
    metrics.all_locked_avg = (
        statistics.mean(all_locked_picks) if all_locked_picks else 31
    )
    metrics.pct_first_lock_pick1 = first_lock_on_pick1 / num_drafts * 100
    metrics.pct_all_locked_before_10 = (
        all_locked_before_10 / num_drafts * 100 if all_locked_picks else 0
    )

    metrics.pct_first_lock_picks1_3 = first_lock_in_1_3 / num_drafts * 100
    metrics.pct_first_lock_picks3_6 = first_lock_in_3_6 / num_drafts * 100
    metrics.pct_second_lock_picks6_10 = (
        second_lock_in_6_10 / num_drafts * 100 if second_lock_same_res else 0
    )

    for pick_num in [5, 10, 15, 20, 25]:
        if pick_num in pick_sa_totals:
            metrics.sa_at_picks[pick_num] = statistics.mean(pick_sa_totals[pick_num])

    for pick_num in range(1, NUM_PICKS + 1):
        if pick_num in pick_sa_totals:
            metrics.convergence_curve[pick_num] = statistics.mean(pick_sa_totals[pick_num])
        if pick_num in pick_decision_quality:
            metrics.decision_quality[pick_num] = (
                statistics.mean(pick_decision_quality[pick_num]) * 100
            )

    early_div = []
    for p in range(1, 6):
        if p in pick_diversity_totals:
            early_div.extend(pick_diversity_totals[p])
    metrics.early_diversity = statistics.mean(early_div) if early_div else 0

    late_sa_vals = []
    for p in range(6, 31):
        if p in pick_sa_totals:
            late_sa_vals.extend(pick_sa_totals[p])
    metrics.late_sa = statistics.mean(late_sa_vals) if late_sa_vals else 0

    for pick_num in range(1, NUM_PICKS + 1):
        if metrics.convergence_curve.get(pick_num, 0) >= 2.0:
            metrics.convergence_pick = pick_num
            break

    # Rate the draft arc
    fl = metrics.first_lock_avg
    sl = metrics.second_lock_same_res_avg
    al = metrics.all_locked_avg
    p1 = metrics.pct_first_lock_pick1

    # Ideal: first lock 3-5, second lock 6-10, not too much pick-1 locking
    arc_score = 0
    if 2.5 <= fl <= 5.0:
        arc_score += 3
    elif 1.5 <= fl <= 6.0:
        arc_score += 2
    elif fl <= 8.0:
        arc_score += 1

    if 5.0 <= sl <= 10.0:
        arc_score += 3
    elif 4.0 <= sl <= 12.0:
        arc_score += 2
    elif sl <= 15.0:
        arc_score += 1

    if p1 <= 5.0:
        arc_score += 2
    elif p1 <= 15.0:
        arc_score += 1

    if al <= 20.0:
        arc_score += 1
    if al >= 8.0:
        arc_score += 1

    if arc_score >= 8:
        metrics.arc_rating = "EXCELLENT"
    elif arc_score >= 6:
        metrics.arc_rating = "GOOD"
    elif arc_score >= 4:
        metrics.arc_rating = "FAIR"
    else:
        metrics.arc_rating = "POOR"

    return metrics


# ---------------------------------------------------------------------------
# Build all configurations
# ---------------------------------------------------------------------------

def build_configs() -> list:
    configs = []

    threshold_pairs = [
        [2, 5], [3, 8], [4, 10], [4, 12], [5, 12], [5, 15], [6, 15], [8, 20]
    ]
    primary_weights = [1, 2, 3]
    distributions = [
        (25, 55, 20),
        (50, 35, 15),
    ]

    # Two-threshold configurations
    for thresholds in threshold_pairs:
        for pw in primary_weights:
            for d1, d2, d3 in distributions:
                name = f"T({thresholds[0]},{thresholds[1]}) W{pw} D{d1}/{d2}/{d3}"
                configs.append(Config(
                    name=name,
                    thresholds=thresholds,
                    primary_weight=pw,
                    pct_1sym=d1,
                    pct_2sym=d2,
                    pct_3sym=d3,
                ))

    # Three-threshold configurations (third threshold at 2x the second)
    for t1, t2 in threshold_pairs:
        t3 = t2 * 2
        for pw in primary_weights:
            for d1, d2, d3 in distributions:
                name = f"T({t1},{t2},{t3}) W{pw} D{d1}/{d2}/{d3}"
                configs.append(Config(
                    name=name,
                    thresholds=[t1, t2, t3],
                    primary_weight=pw,
                    pct_1sym=d1,
                    pct_2sym=d2,
                    pct_3sym=d3,
                ))

    return configs


# ---------------------------------------------------------------------------
# Output formatting
# ---------------------------------------------------------------------------

def print_table_header():
    print(f"{'Config':<38} {'1stLk':>5} {'2ndLk':>5} {'AllLk':>5} "
          f"{'P1%':>5} {'<10%':>5} "
          f"{'SA@5':>5} {'SA@10':>5} {'SA@15':>5} {'SA@20':>5} {'SA@25':>5} "
          f"{'LateSA':>6} {'Conv':>4} "
          f"{'DQ@5':>5} {'DQ@10':>5} {'DQ@15':>5} "
          f"{'EDiv':>5} {'Arc':>9}")
    print("-" * 155)


def print_metrics_row(m: Metrics):
    print(f"{m.config.name:<38} "
          f"{m.first_lock_avg:>5.1f} "
          f"{m.second_lock_same_res_avg:>5.1f} "
          f"{m.all_locked_avg:>5.1f} "
          f"{m.pct_first_lock_pick1:>4.0f}% "
          f"{m.pct_all_locked_before_10:>4.0f}% "
          f"{m.sa_at_picks.get(5, 0):>5.2f} "
          f"{m.sa_at_picks.get(10, 0):>5.2f} "
          f"{m.sa_at_picks.get(15, 0):>5.2f} "
          f"{m.sa_at_picks.get(20, 0):>5.2f} "
          f"{m.sa_at_picks.get(25, 0):>5.2f} "
          f"{m.late_sa:>6.2f} "
          f"{m.convergence_pick:>4} "
          f"{m.decision_quality.get(5, 0):>4.0f}% "
          f"{m.decision_quality.get(10, 0):>4.0f}% "
          f"{m.decision_quality.get(15, 0):>4.0f}% "
          f"{m.early_diversity:>5.1f} "
          f"{m.arc_rating:>9}")


def print_convergence_curve(m: Metrics, label: str = ""):
    name = label or m.config.name
    picks = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 15, 20, 25, 30]
    header = f"{'Config':<38} " + " ".join(f"{'P'+str(p):>5}" for p in picks)
    print(header)
    vals = " ".join(f"{m.convergence_curve.get(p, 0):>5.2f}" for p in picks)
    print(f"{name:<38} {vals}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    random.seed(42)

    configs = build_configs()
    print(f"Total configurations to test: {len(configs)}")
    print()

    all_results = []
    for i, config in enumerate(configs):
        if (i + 1) % 10 == 0:
            print(f"  Running config {i+1}/{len(configs)}: {config.name}")
        m = run_config(config, num_drafts=1000)
        all_results.append(m)

    # ======================================================================
    # SECTION 1: Full results table (two-threshold only)
    # ======================================================================
    two_thresh = [m for m in all_results if len(m.config.thresholds) == 2]
    three_thresh = [m for m in all_results if len(m.config.thresholds) == 3]

    print("\n" + "=" * 155)
    print("SECTION 1: TWO-THRESHOLD CONFIGURATIONS")
    print("=" * 155)
    print_table_header()
    for m in two_thresh:
        print_metrics_row(m)

    # ======================================================================
    # SECTION 2: Three-threshold results
    # ======================================================================
    print("\n" + "=" * 155)
    print("SECTION 2: THREE-THRESHOLD CONFIGURATIONS")
    print("=" * 155)
    print_table_header()
    for m in three_thresh:
        print_metrics_row(m)

    # ======================================================================
    # SECTION 3: Analysis by threshold pair (averaging over weights/dists)
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 3: THRESHOLD PAIR ANALYSIS (averaged over primary weights and distributions)")
    print("=" * 100)

    threshold_groups = defaultdict(list)
    for m in two_thresh:
        key = tuple(m.config.thresholds)
        threshold_groups[key].append(m)

    print(f"{'Thresholds':<12} {'1stLk':>6} {'2ndLk':>6} {'AllLk':>6} "
          f"{'P1%':>5} {'LateSA':>7} {'Conv':>5} {'DQ@10':>6} {'Arc':>10}")
    print("-" * 75)
    for key in sorted(threshold_groups.keys()):
        group = threshold_groups[key]
        fl = statistics.mean([m.first_lock_avg for m in group])
        sl = statistics.mean([m.second_lock_same_res_avg for m in group])
        al = statistics.mean([m.all_locked_avg for m in group])
        p1 = statistics.mean([m.pct_first_lock_pick1 for m in group])
        lsa = statistics.mean([m.late_sa for m in group])
        conv = statistics.mean([m.convergence_pick for m in group])
        dq10 = statistics.mean([m.decision_quality.get(10, 0) for m in group])
        arcs = [m.arc_rating for m in group]
        best_arc = max(set(arcs), key=arcs.count)
        label = f"({key[0]},{key[1]})"
        print(f"{label:<12} {fl:>6.1f} {sl:>6.1f} {al:>6.1f} "
              f"{p1:>4.0f}% {lsa:>7.2f} {conv:>5.1f} {dq10:>5.0f}% {best_arc:>10}")

    # ======================================================================
    # SECTION 4: Analysis by primary weight (averaging over thresholds/dists)
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 4: PRIMARY WEIGHT ANALYSIS (averaged over thresholds and distributions)")
    print("=" * 100)

    weight_groups = defaultdict(list)
    for m in two_thresh:
        weight_groups[m.config.primary_weight].append(m)

    print(f"{'Weight':>6} {'1stLk':>6} {'2ndLk':>6} {'P1%':>5} {'LateSA':>7} {'Conv':>5} {'EDiv':>6}")
    print("-" * 48)
    for w in sorted(weight_groups.keys()):
        group = weight_groups[w]
        fl = statistics.mean([m.first_lock_avg for m in group])
        sl = statistics.mean([m.second_lock_same_res_avg for m in group])
        p1 = statistics.mean([m.pct_first_lock_pick1 for m in group])
        lsa = statistics.mean([m.late_sa for m in group])
        conv = statistics.mean([m.convergence_pick for m in group])
        ediv = statistics.mean([m.early_diversity for m in group])
        print(f"    W{w} {fl:>6.1f} {sl:>6.1f} {p1:>4.0f}% {lsa:>7.2f} {conv:>5.1f} {ediv:>6.1f}")

    # ======================================================================
    # SECTION 5: Analysis by distribution (averaging over thresholds/weights)
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 5: DISTRIBUTION ANALYSIS (averaged over thresholds and weights)")
    print("=" * 100)

    dist_groups = defaultdict(list)
    for m in two_thresh:
        dist_groups[m.config.dist_label].append(m)

    print(f"{'Distribution':<12} {'1stLk':>6} {'2ndLk':>6} {'P1%':>5} {'LateSA':>7} {'Conv':>5} {'EDiv':>6}")
    print("-" * 48)
    for d in sorted(dist_groups.keys()):
        group = dist_groups[d]
        fl = statistics.mean([m.first_lock_avg for m in group])
        sl = statistics.mean([m.second_lock_same_res_avg for m in group])
        p1 = statistics.mean([m.pct_first_lock_pick1 for m in group])
        lsa = statistics.mean([m.late_sa for m in group])
        conv = statistics.mean([m.convergence_pick for m in group])
        ediv = statistics.mean([m.early_diversity for m in group])
        print(f"D{d:<11} {fl:>6.1f} {sl:>6.1f} {p1:>4.0f}% {lsa:>7.2f} {conv:>5.1f} {ediv:>6.1f}")

    # ======================================================================
    # SECTION 6: Two-threshold vs three-threshold comparison
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 6: TWO-THRESHOLD vs THREE-THRESHOLD COMPARISON")
    print("(matched configurations, showing what the third threshold adds)")
    print("=" * 100)

    # Match 2-thresh and 3-thresh by their first two thresholds + weight + dist
    two_map = {}
    for m in two_thresh:
        key = (tuple(m.config.thresholds), m.config.primary_weight, m.config.dist_label)
        two_map[key] = m

    print(f"{'Base Thresholds':<16} {'W':>2} {'Dist':<10} | "
          f"{'2T LateSA':>9} {'3T LateSA':>9} {'Delta':>6} | "
          f"{'2T Conv':>7} {'3T Conv':>7} | "
          f"{'2T AllLk':>8} {'3T AllLk':>8}")
    print("-" * 110)

    for m3 in three_thresh:
        # Find matching 2-threshold config
        base = tuple(m3.config.thresholds[:2])
        key = (list(base), m3.config.primary_weight, m3.config.dist_label)
        key2 = (base, m3.config.primary_weight, m3.config.dist_label)
        m2 = None
        for k, v in two_map.items():
            if (tuple(k[0]) == base and k[1] == m3.config.primary_weight
                    and k[2] == m3.config.dist_label):
                m2 = v
                break

        if m2:
            delta = m3.late_sa - m2.late_sa
            t_label = f"({base[0]},{base[1]})+{m3.config.thresholds[2]}"
            print(f"{t_label:<16} W{m3.config.primary_weight} {m3.config.dist_label:<10} | "
                  f"{m2.late_sa:>9.2f} {m3.late_sa:>9.2f} {delta:>+5.2f} | "
                  f"{m2.convergence_pick:>7} {m3.convergence_pick:>7} | "
                  f"{m2.all_locked_avg:>8.1f} {m3.all_locked_avg:>8.1f}")

    # ======================================================================
    # SECTION 7: Top 10 configurations by arc rating and late S/A
    # ======================================================================
    print("\n" + "=" * 155)
    print("SECTION 7: TOP 15 CONFIGURATIONS (sorted by arc rating then late S/A)")
    print("=" * 155)

    arc_order = {"EXCELLENT": 4, "GOOD": 3, "FAIR": 2, "POOR": 1}
    sorted_results = sorted(
        all_results,
        key=lambda m: (arc_order.get(m.arc_rating, 0), m.late_sa),
        reverse=True
    )

    print_table_header()
    for m in sorted_results[:15]:
        print_metrics_row(m)

    # ======================================================================
    # SECTION 8: Convergence curves for best configs
    # ======================================================================
    print("\n" + "=" * 120)
    print("SECTION 8: CONVERGENCE CURVES (S/A per pack at each pick) — TOP 5 CONFIGS")
    print("=" * 120)

    picks = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 15, 20, 25, 30]
    header = f"{'Config':<38} " + " ".join(f"{'P'+str(p):>5}" for p in picks)
    print(header)
    print("-" * len(header))
    for m in sorted_results[:5]:
        vals = " ".join(f"{m.convergence_curve.get(p, 0):>5.2f}" for p in picks)
        print(f"{m.config.name:<38} {vals}")

    # Also show some contrasting configs
    print("\nContrasting configurations:")
    # Find fastest and slowest
    fastest = min(all_results, key=lambda m: m.first_lock_avg)
    slowest = max(all_results, key=lambda m: m.first_lock_avg)
    print(f"\nFastest first lock (avg {fastest.first_lock_avg:.1f}):")
    vals = " ".join(f"{fastest.convergence_curve.get(p, 0):>5.2f}" for p in picks)
    print(f"{fastest.config.name:<38} {vals}")
    print(f"\nSlowest first lock (avg {slowest.first_lock_avg:.1f}):")
    vals = " ".join(f"{slowest.convergence_curve.get(p, 0):>5.2f}" for p in picks)
    print(f"{slowest.config.name:<38} {vals}")

    # ======================================================================
    # SECTION 9: Decision quality analysis
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 9: DECISION QUALITY (% of packs with 2+ S/A cards)")
    print("(Higher = more meaningful choices for the player)")
    print("=" * 100)

    picks_dq = [3, 5, 8, 10, 15, 20, 25]
    header = f"{'Config':<38} " + " ".join(f"{'P'+str(p):>6}" for p in picks_dq)
    print(header)
    print("-" * (38 + 7 * len(picks_dq)))
    for m in sorted_results[:10]:
        vals = " ".join(f"{m.decision_quality.get(p, 0):>5.0f}%" for p in picks_dq)
        print(f"{m.config.name:<38} {vals}")

    # ======================================================================
    # SECTION 10: Interaction analysis — threshold x distribution
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 10: THRESHOLD x DISTRIBUTION INTERACTION (W2 only, late S/A)")
    print("How thresholds interact with symbol distribution")
    print("=" * 100)

    w2_results = [m for m in two_thresh if m.config.primary_weight == 2]
    print(f"{'Thresholds':<12} {'D25/55/20':>10} {'D50/35/15':>10} {'Delta':>8}")
    print("-" * 45)
    for key in sorted(threshold_groups.keys()):
        d1_results = [m for m in w2_results
                      if tuple(m.config.thresholds) == key and m.config.dist_label == "25/55/20"]
        d2_results = [m for m in w2_results
                      if tuple(m.config.thresholds) == key and m.config.dist_label == "50/35/15"]
        if d1_results and d2_results:
            v1 = d1_results[0].late_sa
            v2 = d2_results[0].late_sa
            delta = v2 - v1
            label = f"({key[0]},{key[1]})"
            print(f"{label:<12} {v1:>10.2f} {v2:>10.2f} {delta:>+7.2f}")

    # ======================================================================
    # SECTION 11: The "ideal arc" analysis
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 11: IDEAL DRAFT ARC ANALYSIS")
    print("Target arc: 1st lock picks 3-5, 2nd lock picks 6-10, all locked picks 12-20")
    print("=" * 100)

    for m in sorted_results[:10]:
        fl = m.first_lock_avg
        sl = m.second_lock_same_res_avg
        al = m.all_locked_avg
        p1 = m.pct_first_lock_pick1

        # Characterize the arc
        if fl < 2:
            early = "INSTANT (too fast)"
        elif fl < 3.5:
            early = "Fast commitment"
        elif fl < 5.5:
            early = "IDEAL exploration"
        elif fl < 8:
            early = "Slow start"
        else:
            early = "Very slow"

        if sl < 5:
            mid = "INSTANT 2nd lock (too fast)"
        elif sl < 8:
            mid = "Fast solidify"
        elif sl < 11:
            mid = "IDEAL solidify"
        elif sl < 15:
            mid = "Slow solidify"
        else:
            mid = "Very slow solidify"

        if al < 8:
            late = "Locked too early"
        elif al < 14:
            late = "Fast endgame"
        elif al < 22:
            late = "IDEAL endgame"
        elif al >= 30:
            late = "Never fully locks"
        else:
            late = "Slow endgame"

        print(f"\n{m.config.name} [{m.arc_rating}]")
        print(f"  1st lock avg: {fl:.1f} — {early}")
        print(f"  2nd lock (same res) avg: {sl:.1f} — {mid}")
        print(f"  All 4 locked avg: {al:.1f} — {late}")
        print(f"  Pick-1 lock: {p1:.0f}% | Late S/A: {m.late_sa:.2f} | Conv: {m.convergence_pick}")


if __name__ == "__main__":
    main()
