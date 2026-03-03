#!/usr/bin/env python3
"""
Pool Design Agent 1: Symbol Count Distribution for Pair-Escalation Slots

Investigates what ratio of 1/2/3-symbol cards (among non-generic cards)
produces the best draft experience with the Pair-Escalation Slots algorithm
(D2 with K=6, C=0.50).

Key insight: Only 2+ symbol cards contribute pair data. The symbol count
distribution directly controls pair accumulation rate, which drives the
algorithm's convergence probability P = min(top_pair_count / K, C).

All metrics evaluated at ARCHETYPE level, 1000+ drafts per configuration.
"""

import random
import statistics
from dataclasses import dataclass, field
from collections import defaultdict, Counter
from typing import Optional

# ============================================================
# Constants
# ============================================================

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    {"name": "Flash/Tempo/Prison",   "primary": "Zephyr", "secondary": "Ember",  "pos": 0},
    {"name": "Blink/Flicker",        "primary": "Ember",  "secondary": "Zephyr", "pos": 1},
    {"name": "Storm/Spellslinger",   "primary": "Ember",  "secondary": "Stone",  "pos": 2},
    {"name": "Self-Discard",         "primary": "Stone",  "secondary": "Ember",  "pos": 3},
    {"name": "Self-Mill/Reanimator", "primary": "Stone",  "secondary": "Tide",   "pos": 4},
    {"name": "Sacrifice/Abandon",    "primary": "Tide",   "secondary": "Stone",  "pos": 5},
    {"name": "Warriors/Midrange",    "primary": "Tide",   "secondary": "Zephyr", "pos": 6},
    {"name": "Ramp/Spirit Animals",  "primary": "Zephyr", "secondary": "Tide",   "pos": 7},
]

ARCHETYPE_NAMES = [a["name"] for a in ARCHETYPES]
NUM_ARCHETYPES = 8
NUM_CARDS = 360
GENERIC_COUNT = 36
CARDS_PER_ARCHETYPE = (NUM_CARDS - GENERIC_COUNT) // NUM_ARCHETYPES  # 40
PACK_SIZE = 4
NUM_PICKS = 30
NUM_DRAFTS = 1200

# Pair-Escalation parameters
K_PARAM = 6
CAP_PARAM = 0.50

# ============================================================
# Helpers
# ============================================================

def circle_distance(pos1, pos2):
    d = abs(pos1 - pos2)
    return min(d, 8 - d)


TIER_VALUES = {"S": 4, "A": 3, "B": 2, "C": 1, "F": 0}


# ============================================================
# Card Model
# ============================================================

@dataclass
class SimCard:
    id: int
    symbols: list       # list of resonance strings, ordered
    archetype: Optional[str]
    archetype_idx: Optional[int]
    fitness: dict = field(default_factory=dict)  # archetype_name -> tier_str
    power: float = 5.0

    def ordered_pair(self):
        if len(self.symbols) >= 2:
            return (self.symbols[0], self.symbols[1])
        return None

    def is_sa_for(self, arch_name):
        return self.fitness.get(arch_name, "F") in ("S", "A")

    def is_cf_for(self, arch_name):
        return self.fitness.get(arch_name, "F") in ("C", "F")


def assign_fitness(card):
    """Assign fitness tiers for all 8 archetypes."""
    card.fitness = {}
    if card.archetype is None:
        for a in ARCHETYPES:
            card.fitness[a["name"]] = "B"
        return

    home = next(a for a in ARCHETYPES if a["name"] == card.archetype)
    for a in ARCHETYPES:
        if a["name"] == card.archetype:
            card.fitness[a["name"]] = "S"
        elif a["primary"] == home["primary"] and a["name"] != card.archetype:
            card.fitness[a["name"]] = "A"
        elif a["primary"] == home["secondary"] or a["secondary"] == home["primary"]:
            card.fitness[a["name"]] = "B"
        else:
            dist = circle_distance(home["pos"], a["pos"])
            if dist <= 2:
                card.fitness[a["name"]] = "C"
            else:
                card.fitness[a["name"]] = "F"


def generate_symbols(arch, num_symbols, rng):
    """Generate symbol list for a card of given archetype and symbol count."""
    pri, sec = arch["primary"], arch["secondary"]
    if num_symbols == 1:
        # 70% primary, 30% secondary
        return [pri] if rng.random() < 0.70 else [sec]
    elif num_symbols == 2:
        # Ordered pair: [primary, secondary] is the canonical archetype pair
        # But allow some variation for realism
        first = pri if rng.random() < 0.75 else sec
        other = sec if first == pri else pri
        second = other if rng.random() < 0.80 else first
        return [first, second]
    else:
        # 3-symbol: variations of primary/secondary
        patterns = [
            [pri, sec, pri],
            [pri, pri, sec],
            [pri, sec, sec],
        ]
        return rng.choice(patterns)


def build_pool(rng, pct_1sym=0.15, pct_2sym=0.60, pct_3sym=0.25):
    """Build a 360-card pool with given symbol distribution."""
    cards = []
    card_id = 0

    non_generic = NUM_CARDS - GENERIC_COUNT  # 324
    base_per_arch = non_generic // NUM_ARCHETYPES  # 40
    remainder = non_generic % NUM_ARCHETYPES  # 4

    for arch_idx, arch in enumerate(ARCHETYPES):
        n = base_per_arch + (1 if arch_idx < remainder else 0)
        n1 = round(n * pct_1sym)
        n3 = round(n * pct_3sym)
        n2 = n - n1 - n3

        for count, num in [(1, n1), (2, n2), (3, n3)]:
            for _ in range(num):
                syms = generate_symbols(arch, count, rng)
                c = SimCard(id=card_id, symbols=syms,
                            archetype=arch["name"],
                            archetype_idx=arch_idx,
                            power=rng.uniform(3, 8))
                assign_fitness(c)
                cards.append(c)
                card_id += 1

    for _ in range(GENERIC_COUNT):
        c = SimCard(id=card_id, symbols=[], archetype=None,
                    archetype_idx=None, power=rng.uniform(4, 9))
        assign_fitness(c)
        cards.append(c)
        card_id += 1

    return cards


# ============================================================
# Pair-Escalation Slots Algorithm
# ============================================================

def gen_pack_pair_escalation(pool, pair_counts, rng, K=K_PARAM, cap=CAP_PARAM):
    """
    Each slot independently pair-matched with probability min(top_pair/K, cap).
    """
    top_pair, top_count = None, 0
    if pair_counts:
        top_pair = max(pair_counts, key=pair_counts.get)
        top_count = pair_counts[top_pair]

    prob = min(top_count / K, cap) if top_pair else 0.0
    pair_matched = [c for c in pool if c.ordered_pair() == top_pair] if top_pair else []

    pack = []
    used_ids = set()
    for _ in range(PACK_SIZE):
        if top_pair and pair_matched and rng.random() < prob:
            chosen = rng.choice(pair_matched)
        else:
            candidates = [c for c in pool if c.id not in used_ids]
            if not candidates:
                candidates = pool
            chosen = rng.choice(candidates)
        pack.append(chosen)
        used_ids.add(chosen.id)

    return pack, prob


def update_pair_counts(card, pair_counts):
    pair = card.ordered_pair()
    if pair:
        pair_counts[pair] = pair_counts.get(pair, 0) + 1


# ============================================================
# Player Strategy
# ============================================================

def pick_card_committed(pack, target_arch, pick_num):
    """Committed player: picks highest fitness for target archetype."""
    if pick_num < 5:
        # Early: mix fitness and power
        def score(c):
            tier_v = TIER_VALUES.get(c.fitness.get(target_arch, "F"), 0)
            return tier_v * 0.5 + c.power * 0.1
        return max(pack, key=score)
    else:
        def score(c):
            tier_v = TIER_VALUES.get(c.fitness.get(target_arch, "F"), 0)
            return tier_v * 0.9 + c.power * 0.1
        return max(pack, key=score)


def best_archetype_from_pairs(pair_counts):
    """Determine best archetype from pair counter."""
    if not pair_counts:
        return None
    pair_to_arch = {}
    for a in ARCHETYPES:
        pair_to_arch[(a["primary"], a["secondary"])] = a["name"]

    arch_scores = {a["name"]: 0.0 for a in ARCHETYPES}
    for (p1, p2), cnt in pair_counts.items():
        arch = pair_to_arch.get((p1, p2))
        if arch:
            arch_scores[arch] += cnt
        # Adjacent bonus
        for a in ARCHETYPES:
            if a["primary"] == p1 and a["name"] != arch:
                arch_scores[a["name"]] += cnt * 0.3

    best = max(arch_scores, key=arch_scores.get)
    if arch_scores[best] == 0:
        return None
    return best


# ============================================================
# Single Draft
# ============================================================

@dataclass
class DraftResult:
    picks: list = field(default_factory=list)
    packs_seen: list = field(default_factory=list)
    target_archetype: str = ""
    pair_counts_by_pick: list = field(default_factory=list)
    prob_by_pick: list = field(default_factory=list)
    top_pair_count_by_pick: list = field(default_factory=list)
    one_sym_picks: int = 0
    two_plus_sym_picks: int = 0


def run_single_draft(pool, rng, forced_arch=None):
    """Run one draft with committed strategy."""
    pair_counts = {}
    result = DraftResult()
    target_arch = forced_arch

    for pick_num in range(NUM_PICKS):
        # Generate pack
        pack, prob = gen_pack_pair_escalation(pool, pair_counts, rng)

        # Determine target if not yet committed
        if target_arch is None and pick_num >= 5:
            target_arch = best_archetype_from_pairs(pair_counts)
            if target_arch is None:
                target_arch = rng.choice(ARCHETYPE_NAMES)

        if target_arch is None:
            # Pre-commitment: pick highest power
            chosen = max(pack, key=lambda c: c.power)
        else:
            chosen = pick_card_committed(pack, target_arch, pick_num)

        result.picks.append(chosen)
        result.packs_seen.append(list(pack))

        # Track pair probability
        top_count = max(pair_counts.values()) if pair_counts else 0
        result.top_pair_count_by_pick.append(top_count)
        result.prob_by_pick.append(prob)

        # Update pair counts
        update_pair_counts(chosen, pair_counts)
        result.pair_counts_by_pick.append(dict(pair_counts))

        # Track 1-sym vs 2+sym picks
        if len(chosen.symbols) == 1:
            result.one_sym_picks += 1
        elif len(chosen.symbols) >= 2:
            result.two_plus_sym_picks += 1

    if target_arch is None:
        target_arch = rng.choice(ARCHETYPE_NAMES)
    result.target_archetype = target_arch
    return result


# ============================================================
# Metrics Computation
# ============================================================

@dataclass
class DistributionMetrics:
    name: str
    pct_1sym: float
    pct_2sym: float
    pct_3sym: float

    # Pair accumulation rate: top pair count at various picks
    top_pair_at_pick: dict = field(default_factory=dict)  # pick -> mean count

    # First meaningful probability: pick where P first exceeds thresholds
    first_p_25: float = 0.0
    first_p_50: float = 0.0

    # Effective convergence rate: fraction of slots pair-targeted
    eff_conv_at_10: float = 0.0
    eff_conv_at_20: float = 0.0
    eff_conv_at_30: float = 0.0

    # S/A per pack at various picks
    sa_at_pick: dict = field(default_factory=dict)

    # 1-symbol trap metrics
    one_sym_pick_fraction: float = 0.0
    avg_stall_length: float = 0.0  # consecutive picks without pair contribution

    # Standard metrics
    early_unique_archs: float = 0.0
    early_sa_target: float = 0.0
    late_sa_mean: float = 0.0
    late_sa_stddev: float = 0.0
    late_cf_mean: float = 0.0
    convergence_pick: float = 0.0
    deck_concentration: float = 0.0
    run_overlap: float = 0.0
    arch_freq_max: float = 0.0
    arch_freq_min: float = 0.0

    # Average probability at each pick (for ramp curve)
    avg_prob_at_pick: dict = field(default_factory=dict)

    # Pass/fail counts
    targets_passed: int = 0


def compute_distribution_metrics(name, pool, n_drafts, rng,
                                  pct_1sym, pct_2sym, pct_3sym):
    """Run n_drafts and compute all metrics for one distribution."""
    m = DistributionMetrics(name=name, pct_1sym=pct_1sym,
                            pct_2sym=pct_2sym, pct_3sym=pct_3sym)

    # Accumulators
    top_pair_by_pick = {p: [] for p in range(NUM_PICKS)}
    prob_by_pick = {p: [] for p in range(NUM_PICKS)}
    sa_by_pick = {p: [] for p in range(NUM_PICKS)}

    early_unique_all = []
    early_sa_all = []
    late_sa_all = []
    late_cf_all = []
    conv_picks = []
    deck_concs = []
    all_decks = []
    arch_freq = defaultdict(int)
    one_sym_fracs = []
    stall_lengths = []

    first_p25_picks = []
    first_p50_picks = []

    for _ in range(n_drafts):
        dr = run_single_draft(pool, rng, forced_arch=None)
        tgt = dr.target_archetype
        arch_freq[tgt] += 1

        # Track top pair counts and probabilities per pick
        for pn in range(NUM_PICKS):
            top_pair_by_pick[pn].append(dr.top_pair_count_by_pick[pn])
            prob_by_pick[pn].append(dr.prob_by_pick[pn])

        # Track S/A per pack at each pick
        for pn, pack in enumerate(dr.packs_seen):
            sa_count = sum(1 for c in pack if c.is_sa_for(tgt))
            sa_by_pick[pn].append(sa_count)

        # First meaningful probability
        found_25, found_50 = False, False
        for pn in range(NUM_PICKS):
            p = dr.prob_by_pick[pn]
            if not found_25 and p >= 0.25:
                first_p25_picks.append(pn + 1)
                found_25 = True
            if not found_50 and p >= 0.50:
                first_p50_picks.append(pn + 1)
                found_50 = True
        if not found_25:
            first_p25_picks.append(NUM_PICKS + 1)
        if not found_50:
            first_p50_picks.append(NUM_PICKS + 1)

        # Early metrics (picks 1-5)
        for pn in range(min(5, len(dr.packs_seen))):
            pack = dr.packs_seen[pn]
            unique_archs = set()
            for c in pack:
                for aname in ARCHETYPE_NAMES:
                    if c.is_sa_for(aname):
                        unique_archs.add(aname)
            early_unique_all.append(len(unique_archs))
            early_sa_all.append(sum(1 for c in pack if c.is_sa_for(tgt)))

        # Late metrics (picks 6+)
        sa_streak = 0
        conv = NUM_PICKS
        for pn in range(5, len(dr.packs_seen)):
            pack = dr.packs_seen[pn]
            sa_count = sum(1 for c in pack if c.is_sa_for(tgt))
            cf_count = sum(1 for c in pack if c.is_cf_for(tgt))
            late_sa_all.append(sa_count)
            late_cf_all.append(cf_count)

            if sa_count >= 2:
                sa_streak += 1
            else:
                sa_streak = 0
            if sa_streak >= 3 and conv == NUM_PICKS:
                conv = pn - 2

        conv_picks.append(conv)

        # Deck concentration
        sa_in_deck = sum(1 for c in dr.picks if c.is_sa_for(tgt))
        deck_concs.append(sa_in_deck / len(dr.picks) if dr.picks else 0)

        # Run variety tracking
        all_decks.append(set(c.id for c in dr.picks))

        # 1-symbol trap
        total_nongen = dr.one_sym_picks + dr.two_plus_sym_picks
        if total_nongen > 0:
            one_sym_fracs.append(dr.one_sym_picks / total_nongen)
        else:
            one_sym_fracs.append(0)

        # Stall detection: longest consecutive picks without pair contribution
        max_stall = 0
        current_stall = 0
        for pn in range(5, NUM_PICKS):
            card = dr.picks[pn]
            if len(card.symbols) < 2:
                current_stall += 1
                max_stall = max(max_stall, current_stall)
            else:
                current_stall = 0
        stall_lengths.append(max_stall)

    # Aggregate top pair counts and probabilities at all picks
    for p in range(NUM_PICKS):
        m.top_pair_at_pick[p + 1] = statistics.mean(top_pair_by_pick[p])
        m.avg_prob_at_pick[p + 1] = statistics.mean(prob_by_pick[p])

    # S/A at key picks
    for p in [4, 9, 14, 19, 24, 29]:
        m.sa_at_pick[p + 1] = statistics.mean(sa_by_pick[p])

    # First meaningful probability
    m.first_p_25 = statistics.mean(first_p25_picks)
    m.first_p_50 = statistics.mean(first_p50_picks)

    # Effective convergence rate
    m.eff_conv_at_10 = statistics.mean(prob_by_pick[9])
    m.eff_conv_at_20 = statistics.mean(prob_by_pick[19])
    m.eff_conv_at_30 = statistics.mean(prob_by_pick[29])

    # Standard metrics
    m.early_unique_archs = statistics.mean(early_unique_all) if early_unique_all else 0
    m.early_sa_target = statistics.mean(early_sa_all) if early_sa_all else 0
    m.late_sa_mean = statistics.mean(late_sa_all) if late_sa_all else 0
    m.late_sa_stddev = statistics.stdev(late_sa_all) if len(late_sa_all) > 1 else 0
    m.late_cf_mean = statistics.mean(late_cf_all) if late_cf_all else 0
    m.convergence_pick = statistics.mean(conv_picks) if conv_picks else NUM_PICKS
    m.deck_concentration = statistics.mean(deck_concs) if deck_concs else 0

    # Run overlap
    overlaps = []
    by_arch = defaultdict(list)
    for i in range(len(all_decks)):
        # Use archetype freq for grouping
        pass
    # Simpler: sample random pairs
    sample_size = min(200, len(all_decks))
    sample_indices = rng.sample(range(len(all_decks)), sample_size)
    for i in range(0, len(sample_indices), 2):
        if i + 1 < len(sample_indices):
            s1 = all_decks[sample_indices[i]]
            s2 = all_decks[sample_indices[i + 1]]
            if s1 and s2:
                overlaps.append(len(s1 & s2) / max(len(s1 | s2), 1))
    m.run_overlap = statistics.mean(overlaps) if overlaps else 0

    # Archetype frequency
    total = n_drafts
    af = {a: arch_freq.get(a, 0) / total for a in ARCHETYPE_NAMES}
    m.arch_freq_max = max(af.values()) if af else 0
    m.arch_freq_min = min(af.values()) if af else 0

    # 1-symbol trap
    m.one_sym_pick_fraction = statistics.mean(one_sym_fracs) if one_sym_fracs else 0
    m.avg_stall_length = statistics.mean(stall_lengths) if stall_lengths else 0

    # Count passes
    passes = 0
    if m.early_unique_archs >= 3:
        passes += 1
    if m.early_sa_target <= 2:
        passes += 1
    if m.late_sa_mean >= 2:
        passes += 1
    if m.late_cf_mean >= 0.5:
        passes += 1
    if 5 <= m.convergence_pick <= 8:
        passes += 1
    if 0.60 <= m.deck_concentration <= 0.90:
        passes += 1
    if m.run_overlap < 0.40:
        passes += 1
    if m.late_sa_stddev >= 0.8:
        passes += 1
    m.targets_passed = passes

    return m


# ============================================================
# Configurations
# ============================================================

DISTRIBUTIONS = [
    ("V5 Recommended (15/60/25)", 0.15, 0.60, 0.25),
    ("All 1-sym (100/0/0)",       1.00, 0.00, 0.00),
    ("Heavy 1-sym (60/30/10)",    0.60, 0.30, 0.10),
    ("Moderate 1-sym (40/40/20)", 0.40, 0.40, 0.20),
    ("Balanced (33/34/33)",       0.33, 0.34, 0.33),
    ("Heavy 2-sym (10/80/10)",    0.10, 0.80, 0.10),
    ("Heavy 3-sym (10/30/60)",    0.10, 0.30, 0.60),
    ("All 2+3-sym (0/70/30)",     0.00, 0.70, 0.30),
    ("Minimal 1-sym (5/70/25)",   0.05, 0.70, 0.25),
    ("Quarter 1-sym (25/50/25)",  0.25, 0.50, 0.25),
]


# ============================================================
# Main
# ============================================================

def main():
    print("=" * 100)
    print(" Pool Design Agent 1: Symbol Count Distribution for Pair-Escalation Slots")
    print(f" {NUM_DRAFTS} drafts per configuration, {NUM_PICKS} picks per draft, K={K_PARAM}, C={CAP_PARAM}")
    print("=" * 100)

    all_metrics = []
    base_seed = 42

    for name, p1, p2, p3 in DISTRIBUTIONS:
        print(f"\nSimulating: {name}...", flush=True)
        rng = random.Random(base_seed)
        pool = build_pool(rng, pct_1sym=p1, pct_2sym=p2, pct_3sym=p3)

        # Validate pool composition
        n1 = sum(1 for c in pool if len(c.symbols) == 1)
        n2 = sum(1 for c in pool if len(c.symbols) == 2)
        n3 = sum(1 for c in pool if len(c.symbols) == 3)
        n0 = sum(1 for c in pool if len(c.symbols) == 0)
        print(f"  Pool: {len(pool)} cards ({n0} generic, {n1} 1-sym, {n2} 2-sym, {n3} 3-sym)")

        sim_rng = random.Random(base_seed + 1000)
        m = compute_distribution_metrics(name, pool, NUM_DRAFTS, sim_rng,
                                          pct_1sym=p1, pct_2sym=p2, pct_3sym=p3)
        all_metrics.append(m)
        print(f"  Late S/A: {m.late_sa_mean:.2f}, Conv: {m.convergence_pick:.1f}, "
              f"StdDev: {m.late_sa_stddev:.2f}, Passes: {m.targets_passed}/8")

    # ================================================================
    # TABLE 1: Pair Accumulation Rate
    # ================================================================
    print("\n\n" + "=" * 120)
    print(" TABLE 1: Pair Accumulation Rate (avg top pair count at pick N)")
    print("=" * 120)
    header = f"{'Distribution':<30}"
    for p in [5, 10, 15, 20, 25, 30]:
        header += f" {'P'+str(p):>7}"
    print(header)
    print("-" * (30 + 7 * 6))
    for m in all_metrics:
        row = f"{m.name:<30}"
        for p in [5, 10, 15, 20, 25, 30]:
            row += f" {m.top_pair_at_pick.get(p, 0):>7.2f}"
        print(row)

    # ================================================================
    # TABLE 2: First Meaningful Probability
    # ================================================================
    print("\n\n" + "=" * 120)
    print(" TABLE 2: First Meaningful Probability & Effective Convergence Rate")
    print("=" * 120)
    header = f"{'Distribution':<30} {'1st P>=.25':>10} {'1st P>=.50':>10} {'P@10':>7} {'P@20':>7} {'P@30':>7}"
    print(header)
    print("-" * 71)
    for m in all_metrics:
        p25 = f"{m.first_p_25:.1f}" if m.first_p_25 <= NUM_PICKS else "never"
        p50 = f"{m.first_p_50:.1f}" if m.first_p_50 <= NUM_PICKS else "never"
        print(f"{m.name:<30} {p25:>10} {p50:>10} {m.eff_conv_at_10:>7.1%} "
              f"{m.eff_conv_at_20:>7.1%} {m.eff_conv_at_30:>7.1%}")

    # ================================================================
    # TABLE 3: S/A Cards Per Pack Convergence Curve
    # ================================================================
    print("\n\n" + "=" * 120)
    print(" TABLE 3: S/A Cards Per Pack at Key Picks")
    print("=" * 120)
    header = f"{'Distribution':<30}"
    for p in [5, 10, 15, 20, 25, 30]:
        header += f" {'P'+str(p):>7}"
    print(header)
    print("-" * (30 + 7 * 6))
    for m in all_metrics:
        row = f"{m.name:<30}"
        for p in [5, 10, 15, 20, 25, 30]:
            row += f" {m.sa_at_pick.get(p, 0):>7.2f}"
        print(row)

    # ================================================================
    # TABLE 4: 1-Symbol Card Trap Analysis
    # ================================================================
    print("\n\n" + "=" * 120)
    print(" TABLE 4: 1-Symbol Card Trap (fraction of non-generic picks that are 1-sym)")
    print("=" * 120)
    header = f"{'Distribution':<30} {'1-Sym Frac':>10} {'Avg Max Stall':>14} {'Pool 1-Sym%':>11}"
    print(header)
    print("-" * 65)
    for m in all_metrics:
        print(f"{m.name:<30} {m.one_sym_pick_fraction:>10.1%} {m.avg_stall_length:>14.1f} "
              f"{m.pct_1sym:>11.0%}")

    # ================================================================
    # TABLE 5: Full Scorecard
    # ================================================================
    print("\n\n" + "=" * 140)
    print(" TABLE 5: Full Scorecard (Archetype Level, All Targets)")
    print("=" * 140)
    header = (f"{'Distribution':<30} {'EarlyU':>7} {'EarlySA':>8} {'LateSA':>7} "
              f"{'LateCF':>7} {'Conv':>6} {'DkConc':>7} {'Overlap':>8} {'StdDev':>7} {'Pass':>5}")
    print(header)
    print("-" * 92)

    targets_row = (f"{'TARGET':<30} {'>=3':>7} {'<=2':>8} {'>=2':>7} "
                   f"{'>=0.5':>7} {'5-8':>6} {'60-90%':>7} {'<40%':>8} {'>=0.8':>7}")
    print(targets_row)
    print("-" * 92)

    for m in all_metrics:
        eu_pf = "P" if m.early_unique_archs >= 3 else "F"
        es_pf = "P" if m.early_sa_target <= 2 else "F"
        ls_pf = "P" if m.late_sa_mean >= 2 else "F"
        lc_pf = "P" if m.late_cf_mean >= 0.5 else "F"
        cv_pf = "P" if 5 <= m.convergence_pick <= 8 else "F"
        dk_pf = "P" if 0.60 <= m.deck_concentration <= 0.90 else "F"
        ov_pf = "P" if m.run_overlap < 0.40 else "F"
        sd_pf = "P" if m.late_sa_stddev >= 0.8 else "F"

        print(f"{m.name:<30} {m.early_unique_archs:>6.1f}{eu_pf} "
              f"{m.early_sa_target:>6.2f}{es_pf}  "
              f"{m.late_sa_mean:>5.2f}{ls_pf} "
              f"{m.late_cf_mean:>5.2f}{lc_pf} "
              f"{m.convergence_pick:>5.1f}{cv_pf} "
              f"{m.deck_concentration:>5.1%}{dk_pf} "
              f"{m.run_overlap:>6.1%}{ov_pf} "
              f"{m.late_sa_stddev:>5.2f}{sd_pf} "
              f"{m.targets_passed:>4}/8")

    # ================================================================
    # TABLE 6: Pair Accumulation vs Convergence Summary
    # ================================================================
    print("\n\n" + "=" * 120)
    print(" TABLE 6: Summary — Pair Accumulation Speed vs Draft Quality")
    print("=" * 120)
    header = (f"{'Distribution':<30} {'TopPair@15':>11} {'P@15':>6} {'LateSA':>7} "
              f"{'Conv':>6} {'StdDev':>7} {'1-Sym Trap':>10} {'Rating':>8}")
    print(header)
    print("-" * 86)

    for m in all_metrics:
        tp15 = m.top_pair_at_pick.get(15, 0)
        p15 = min(tp15 / K_PARAM, CAP_PARAM)

        # Rating
        score = 0
        if m.late_sa_mean >= 2.0:
            score += 3
        elif m.late_sa_mean >= 1.8:
            score += 2
        elif m.late_sa_mean >= 1.5:
            score += 1

        if 5 <= m.convergence_pick <= 8:
            score += 2
        elif m.convergence_pick <= 12:
            score += 1

        if m.late_sa_stddev >= 0.8:
            score += 1

        if m.early_unique_archs >= 3:
            score += 1

        if m.avg_stall_length <= 2:
            score += 1

        labels = {0: "Poor", 1: "Fair", 2: "Fair+", 3: "OK", 4: "Good",
                  5: "Good+", 6: "Great", 7: "Excellent", 8: "Perfect"}
        rating = labels.get(score, "Good")

        print(f"{m.name:<30} {tp15:>11.2f} {p15:>6.1%} {m.late_sa_mean:>7.2f} "
              f"{m.convergence_pick:>6.1f} {m.late_sa_stddev:>7.2f} "
              f"{m.one_sym_pick_fraction:>10.1%} {rating:>8}")

    # ================================================================
    # KEY FINDING: The Ramp Curve
    # ================================================================
    print("\n\n" + "=" * 120)
    print(" KEY FINDING: Pair Probability Ramp (avg P at each pick)")
    print("=" * 120)
    print(f"  For the Pair-Escalation algorithm, P = min(top_pair_count / {K_PARAM}, {CAP_PARAM})")
    print(f"  P reaches cap ({CAP_PARAM}) when top_pair_count = {int(K_PARAM * CAP_PARAM)}")
    print()

    # Show average probability at every 3 picks for key configurations
    key_configs = ["V5 Recommended (15/60/25)", "Heavy 2-sym (10/80/10)",
                   "All 2+3-sym (0/70/30)", "Heavy 1-sym (60/30/10)",
                   "Balanced (33/34/33)", "All 1-sym (100/0/0)"]

    show_picks = [1, 3, 5, 7, 9, 12, 15, 18, 21, 24, 27, 30]
    header = f"{'Distribution':<30}"
    for p in show_picks:
        header += f" {'P'+str(p):>6}"
    print(header)
    print("-" * (30 + 7 * len(show_picks)))

    for m in all_metrics:
        if m.name in key_configs:
            row = f"{m.name:<30}"
            for p in show_picks:
                prob = m.avg_prob_at_pick.get(p, 0)
                row += f" {prob:>6.1%}"
            print(row)

    print("\n\nSimulation complete.")


if __name__ == "__main__":
    main()
