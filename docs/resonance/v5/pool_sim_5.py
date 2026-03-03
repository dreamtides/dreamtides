#!/usr/bin/env python3
"""
Pool Simulation 5: Pair-Escalation Slots Parameter Tuning

Investigates: How should K (divisor), C (cap), and pack size be tuned relative
to the symbol distribution, and what is the ideal progression curve?

Algorithm: Pair-Escalation Slots
  Track the ordered symbol pair (first, second) of each 2+ symbol card you
  draft; each pack slot independently shows a card matching your most common
  pair with probability equal to that pair's count divided by K (capped at C),
  otherwise a random card.

Parameter matrix:
  K (divisor): 4, 6, 8, 10, 12
  C (cap):     0.35, 0.50, 0.65, 0.80
  Pack size:   4, 5
  Symbol distributions (1-sym/2-sym/3-sym among non-generic):
    Heavy 1-sym:   40/40/20
    V5 default:    15/60/25
    Heavy 2-sym:   5/75/20
"""

import random
import statistics
from dataclasses import dataclass, field
from collections import defaultdict, Counter
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPE_DEFS = [
    ("Flash",        "Zephyr", "Ember"),   # 0
    ("Blink",        "Ember",  "Zephyr"),  # 1
    ("Storm",        "Ember",  "Stone"),   # 2
    ("Self-Discard", "Stone",  "Ember"),   # 3
    ("Self-Mill",    "Stone",  "Tide"),    # 4
    ("Sacrifice",    "Tide",   "Stone"),   # 5
    ("Warriors",     "Tide",   "Zephyr"),  # 6
    ("Ramp",         "Zephyr", "Tide"),    # 7
]

NUM_ARCHETYPES = 8
NUM_CARDS = 360
GENERIC_COUNT = 36
CARDS_PER_ARCHETYPE = (NUM_CARDS - GENERIC_COUNT) // NUM_ARCHETYPES  # 40
NUM_PICKS = 30

# ---------------------------------------------------------------------------
# Card model and fitness
# ---------------------------------------------------------------------------

def circle_distance(a: int, b: int) -> int:
    d = abs(a - b)
    return min(d, NUM_ARCHETYPES - d)

@dataclass
class SimCard:
    id: int
    symbols: list       # list of resonance strings, 0-3 elements
    archetype_idx: int  # -1 for generic
    fitness: dict = field(default_factory=dict)  # arch_idx -> tier string

    @property
    def ordered_pair(self):
        if len(self.symbols) >= 2:
            return (self.symbols[0], self.symbols[1])
        return None

    @property
    def primary_resonance(self):
        return self.symbols[0] if self.symbols else None


def compute_fitness(card_arch_idx: int) -> dict:
    """Compute fitness tiers for a card from a given archetype index."""
    fitness = {}
    if card_arch_idx < 0:
        for j in range(NUM_ARCHETYPES):
            fitness[j] = "B"
        return fitness

    home_pri = ARCHETYPE_DEFS[card_arch_idx][1]
    home_sec = ARCHETYPE_DEFS[card_arch_idx][2]

    for j in range(NUM_ARCHETYPES):
        if j == card_arch_idx:
            fitness[j] = "S"
            continue
        target_pri = ARCHETYPE_DEFS[j][1]
        target_sec = ARCHETYPE_DEFS[j][2]
        dist = circle_distance(card_arch_idx, j)

        if dist == 1 and (target_pri == home_pri or target_sec == home_pri):
            fitness[j] = "A"
        elif home_sec in (target_pri, target_sec):
            fitness[j] = "B"
        elif dist <= 3:
            fitness[j] = "C"
        else:
            fitness[j] = "F"

    return fitness


def is_sa(card: SimCard, arch_idx: int) -> bool:
    return card.fitness.get(arch_idx, "F") in ("S", "A")


def is_cf(card: SimCard, arch_idx: int) -> bool:
    return card.fitness.get(arch_idx, "F") in ("C", "F")


# ---------------------------------------------------------------------------
# Pool generation
# ---------------------------------------------------------------------------

def generate_pool(pct_1sym: float, pct_2sym: float, pct_3sym: float,
                  rng: random.Random) -> list:
    """Generate a card pool with given symbol distribution percentages (0-1 fractions)."""
    cards = []
    card_id = 0

    # Generic cards
    for _ in range(GENERIC_COUNT):
        c = SimCard(id=card_id, symbols=[], archetype_idx=-1)
        c.fitness = compute_fitness(-1)
        cards.append(c)
        card_id += 1

    # Archetype cards
    for arch_idx in range(NUM_ARCHETYPES):
        _, primary, secondary = ARCHETYPE_DEFS[arch_idx]
        n = CARDS_PER_ARCHETYPE
        n1 = round(n * pct_1sym)
        n3 = round(n * pct_3sym)
        n2 = n - n1 - n3

        # 1-symbol cards
        for _ in range(n1):
            c = SimCard(id=card_id, symbols=[primary], archetype_idx=arch_idx)
            c.fitness = compute_fitness(arch_idx)
            cards.append(c)
            card_id += 1

        # 2-symbol cards: 85% [P,S], 15% [P,P]
        for _ in range(n2):
            if rng.random() < 0.85:
                syms = [primary, secondary]
            else:
                syms = [primary, primary]
            c = SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx)
            c.fitness = compute_fitness(arch_idx)
            cards.append(c)
            card_id += 1

        # 3-symbol cards: [P,S,P], [P,P,S], or [P,S,S]
        for _ in range(n3):
            pattern = rng.choice([
                [primary, secondary, primary],
                [primary, primary, secondary],
                [primary, secondary, secondary],
            ])
            c = SimCard(id=card_id, symbols=pattern, archetype_idx=arch_idx)
            c.fitness = compute_fitness(arch_idx)
            cards.append(c)
            card_id += 1

    return cards


# ---------------------------------------------------------------------------
# Pair-Escalation Slots algorithm
# ---------------------------------------------------------------------------

def gen_pack_pair_escalation(pool, pair_counts, rng, K, C, pack_size):
    """
    Each of `pack_size` slots independently:
      - With probability P = min(top_pair_count / K, C), draw from pair-matched subset
      - Otherwise draw from full pool
    """
    top_pair = None
    top_count = 0
    if pair_counts:
        top_pair, top_count = max(pair_counts.items(), key=lambda x: x[1])

    prob = min(top_count / K, C) if top_pair else 0.0
    pair_matched = [c for c in pool if c.ordered_pair == top_pair] if top_pair else []

    pack = []
    for _ in range(pack_size):
        if top_pair and pair_matched and rng.random() < prob:
            pack.append(rng.choice(pair_matched))
        else:
            pack.append(rng.choice(pool))
    return pack, prob


# ---------------------------------------------------------------------------
# Player strategy: committed to target archetype
# ---------------------------------------------------------------------------

def pick_card_committed(pack, target_arch):
    """Pick best card for target archetype by fitness tier then random."""
    tier_order = {"S": 5, "A": 4, "B": 3, "C": 1, "F": 0}
    return max(pack, key=lambda c: tier_order.get(c.fitness.get(target_arch, "F"), 0))


# ---------------------------------------------------------------------------
# Single draft simulation
# ---------------------------------------------------------------------------

@dataclass
class DraftRecord:
    """Collected data from a single draft."""
    sa_per_pack: list = field(default_factory=list)
    cf_per_pack: list = field(default_factory=list)
    prob_per_pack: list = field(default_factory=list)
    unique_sa_archs_per_pack: list = field(default_factory=list)
    deck_sa_count: int = 0
    target_archetype: int = 0
    card_ids: list = field(default_factory=list)


def run_single_draft(pool, target_arch, K, C, pack_size, rng) -> DraftRecord:
    """Run a single 30-pick draft with Pair-Escalation Slots."""
    pair_counts = {}
    record = DraftRecord(target_archetype=target_arch)

    for pick_num in range(NUM_PICKS):
        pack, prob = gen_pack_pair_escalation(pool, pair_counts, rng, K, C, pack_size)
        record.prob_per_pack.append(prob)

        # Measure S/A count for target archetype
        sa_count = sum(1 for c in pack if is_sa(c, target_arch))
        cf_count = sum(1 for c in pack if is_cf(c, target_arch))
        record.sa_per_pack.append(sa_count)
        record.cf_per_pack.append(cf_count)

        # Measure unique archetypes with S/A cards in pack
        unique_archs = set()
        for c in pack:
            for ai in range(NUM_ARCHETYPES):
                if is_sa(c, ai):
                    unique_archs.add(ai)
        record.unique_sa_archs_per_pack.append(len(unique_archs))

        # Player picks best card
        chosen = pick_card_committed(pack, target_arch)
        record.card_ids.append(chosen.id)

        # Update pair counter
        if chosen.ordered_pair:
            pair = chosen.ordered_pair
            pair_counts[pair] = pair_counts.get(pair, 0) + 1

    # Deck concentration
    record.deck_sa_count = sa_count  # placeholder, compute below
    return record


# ---------------------------------------------------------------------------
# Aggregate metrics
# ---------------------------------------------------------------------------

@dataclass
class ConfigResult:
    """Aggregated metrics for one parameter configuration."""
    label: str
    K: float
    C: float
    pack_size: int
    dist_label: str

    # Core metrics
    early_unique_archs: float = 0.0      # picks 1-5: unique archs with S/A
    early_sa_for_arch: float = 0.0       # picks 1-5: S/A for target
    late_sa: float = 0.0                 # picks 6+: avg S/A
    late_cf: float = 0.0                 # picks 6+: avg C/F
    convergence_pick: float = 0.0        # first pick where rolling avg >= 2.0
    deck_concentration: float = 0.0      # fraction of deck that is S/A
    overlap: float = 0.0                 # run-to-run card overlap
    sa_stddev: float = 0.0              # StdDev of S/A per pack (picks 6+)
    arch_freq_max: float = 0.0
    arch_freq_min: float = 0.0

    # Escalation curve: avg P at key picks
    p_at_5: float = 0.0
    p_at_10: float = 0.0
    p_at_15: float = 0.0
    p_at_20: float = 0.0
    p_at_25: float = 0.0
    p_at_30: float = 0.0

    # Expected targeted slots at key picks
    targeted_at_10: float = 0.0
    targeted_at_20: float = 0.0
    targeted_at_30: float = 0.0

    # Three-act draft arc detection
    exploration_p: float = 0.0   # avg P picks 1-5
    commitment_p: float = 0.0   # avg P picks 6-15
    refinement_p: float = 0.0   # avg P picks 16-30

    # Target passes
    passes: int = 0


def run_configuration(pool, K, C, pack_size, dist_label, num_drafts, rng_seed) -> ConfigResult:
    """Run num_drafts drafts for one parameter configuration and aggregate."""
    label = f"K={K:<2} C={C:.2f} PS={pack_size} {dist_label}"
    result = ConfigResult(label=label, K=K, C=C, pack_size=pack_size, dist_label=dist_label)

    all_records = []
    arch_freq = Counter()
    rng = random.Random(rng_seed)

    for _ in range(num_drafts):
        target_arch = rng.randint(0, NUM_ARCHETYPES - 1)
        arch_freq[target_arch] += 1
        record = run_single_draft(pool, target_arch, K, C, pack_size, rng)
        all_records.append(record)

    # Aggregate early metrics (picks 0-4, i.e. picks 1-5)
    early_unique = []
    early_sa = []
    for rec in all_records:
        for i in range(min(5, len(rec.unique_sa_archs_per_pack))):
            early_unique.append(rec.unique_sa_archs_per_pack[i])
            early_sa.append(rec.sa_per_pack[i])

    result.early_unique_archs = statistics.mean(early_unique) if early_unique else 0
    result.early_sa_for_arch = statistics.mean(early_sa) if early_sa else 0

    # Aggregate late metrics (picks 5-29, i.e. picks 6-30)
    late_sa_vals = []
    late_cf_vals = []
    for rec in all_records:
        for i in range(5, NUM_PICKS):
            late_sa_vals.append(rec.sa_per_pack[i])
            late_cf_vals.append(rec.cf_per_pack[i])

    result.late_sa = statistics.mean(late_sa_vals) if late_sa_vals else 0
    result.late_cf = statistics.mean(late_cf_vals) if late_cf_vals else 0
    result.sa_stddev = statistics.stdev(late_sa_vals) if len(late_sa_vals) > 1 else 0

    # Convergence pick: first pick index where 3-pack rolling average >= 2.0
    conv_picks = []
    for rec in all_records:
        conv = NUM_PICKS
        for i in range(2, NUM_PICKS):
            window = rec.sa_per_pack[max(0, i-2):i+1]
            if sum(window) / len(window) >= 2.0:
                conv = i + 1  # 1-indexed
                break
        conv_picks.append(conv)
    result.convergence_pick = statistics.mean(conv_picks)

    # Deck concentration
    deck_concs = []
    for rec in all_records:
        # Re-derive from sa_per_pack: count how many of the 30 picks were S/A
        # (This is an approximation; the player picks the best available)
        # Better: track actual picks' fitness. For now use the fact that committed
        # player picks the best card, so count S/A picked.
        # We'll count the number of packs that had at least 1 S/A card (player picked it)
        sa_picked = sum(1 for sa in rec.sa_per_pack if sa >= 1)
        deck_concs.append(sa_picked / NUM_PICKS)
    result.deck_concentration = statistics.mean(deck_concs)

    # Run-to-run overlap (same-archetype pairs)
    by_arch = defaultdict(list)
    for rec in all_records:
        by_arch[rec.target_archetype].append(set(rec.card_ids))
    overlaps = []
    for arch, decks in by_arch.items():
        for i in range(min(30, len(decks))):
            for j in range(i + 1, min(30, len(decks))):
                if decks[i] and decks[j]:
                    union = len(decks[i] | decks[j])
                    if union > 0:
                        overlaps.append(len(decks[i] & decks[j]) / union)
    result.overlap = statistics.mean(overlaps) if overlaps else 0

    # Archetype frequency
    total = sum(arch_freq.values())
    if total > 0:
        result.arch_freq_max = max(arch_freq.values()) / total
        result.arch_freq_min = min(arch_freq.get(i, 0) for i in range(NUM_ARCHETYPES)) / total

    # Escalation curve: avg P at key picks
    for pick_idx, attr in [(4, "p_at_5"), (9, "p_at_10"), (14, "p_at_15"),
                           (19, "p_at_20"), (24, "p_at_25"), (29, "p_at_30")]:
        vals = [rec.prob_per_pack[pick_idx] for rec in all_records if len(rec.prob_per_pack) > pick_idx]
        if vals:
            setattr(result, attr, statistics.mean(vals))

    # Expected targeted slots at key picks
    for pick_idx, attr in [(9, "targeted_at_10"), (19, "targeted_at_20"), (29, "targeted_at_30")]:
        vals = [rec.prob_per_pack[pick_idx] * pack_size for rec in all_records
                if len(rec.prob_per_pack) > pick_idx]
        if vals:
            setattr(result, attr, statistics.mean(vals))

    # Three-act draft arc
    exploration_p = []
    commitment_p = []
    refinement_p = []
    for rec in all_records:
        for i in range(min(5, len(rec.prob_per_pack))):
            exploration_p.append(rec.prob_per_pack[i])
        for i in range(5, min(15, len(rec.prob_per_pack))):
            commitment_p.append(rec.prob_per_pack[i])
        for i in range(15, min(30, len(rec.prob_per_pack))):
            refinement_p.append(rec.prob_per_pack[i])

    result.exploration_p = statistics.mean(exploration_p) if exploration_p else 0
    result.commitment_p = statistics.mean(commitment_p) if commitment_p else 0
    result.refinement_p = statistics.mean(refinement_p) if refinement_p else 0

    # Count passes
    passes = 0
    if result.early_unique_archs >= 3: passes += 1
    if result.early_sa_for_arch <= 2: passes += 1
    if result.late_sa >= 2.0: passes += 1
    if result.late_cf >= 0.5: passes += 1
    if 5 <= result.convergence_pick <= 8: passes += 1
    if 0.6 <= result.deck_concentration <= 0.9: passes += 1
    if result.overlap < 0.4: passes += 1
    if result.sa_stddev >= 0.8: passes += 1
    result.passes = passes

    return result


# ---------------------------------------------------------------------------
# Main simulation
# ---------------------------------------------------------------------------

def main():
    random.seed(42)

    K_values = [4, 6, 8, 10, 12]
    C_values = [0.35, 0.50, 0.65, 0.80]
    pack_sizes = [4, 5]
    distributions = {
        "heavy1": (0.40, 0.40, 0.20),
        "default": (0.15, 0.60, 0.25),
        "heavy2": (0.05, 0.75, 0.20),
    }

    total_configs = len(K_values) * len(C_values) * len(pack_sizes) * len(distributions)
    print(f"Total configurations: {total_configs}")
    print(f"Running 500 drafts per configuration for matrix sweep...\n")

    # Generate pools for each distribution
    pools = {}
    pool_rng = random.Random(42)
    for dist_name, (p1, p2, p3) in distributions.items():
        pools[dist_name] = generate_pool(p1, p2, p3, random.Random(42))
        n_pairs = sum(1 for c in pools[dist_name] if c.ordered_pair is not None)
        print(f"Pool '{dist_name}': {len(pools[dist_name])} cards, "
              f"{n_pairs} with ordered pairs ({100*n_pairs/len(pools[dist_name]):.0f}%)")

    # Run matrix sweep
    all_results = []
    config_num = 0
    base_seed = 1000

    for dist_name, (p1, p2, p3) in distributions.items():
        pool = pools[dist_name]
        for K in K_values:
            for C in C_values:
                for ps in pack_sizes:
                    config_num += 1
                    if config_num % 20 == 0:
                        print(f"  Config {config_num}/{total_configs}...", flush=True)
                    r = run_configuration(pool, K, C, ps, dist_name,
                                          num_drafts=500, rng_seed=base_seed)
                    base_seed += 500
                    all_results.append(r)

    # ======================================================================
    # SECTION 1: Full matrix summary (compact)
    # ======================================================================
    print("\n" + "=" * 140)
    print("SECTION 1: FULL MATRIX — KEY METRICS (500 drafts each)")
    print("=" * 140)
    print(f"{'Config':<35} {'LateSA':>6} {'StdDev':>6} {'Conv':>5} {'DeckC%':>6} "
          f"{'Olap%':>5} {'OffCF':>5} {'EDiv':>5} {'ESA':>5} {'Pass':>4}")
    print("-" * 105)
    for r in sorted(all_results, key=lambda x: (-x.passes, -x.late_sa)):
        print(f"{r.label:<35} {r.late_sa:>6.2f} {r.sa_stddev:>6.2f} "
              f"{r.convergence_pick:>5.1f} {r.deck_concentration*100:>5.1f}% "
              f"{r.overlap*100:>4.1f}% {r.late_cf:>5.2f} "
              f"{r.early_unique_archs:>5.1f} {r.early_sa_for_arch:>5.2f} {r.passes:>4}/8")

    # ======================================================================
    # SECTION 2: Escalation curves — P at key picks
    # ======================================================================
    print("\n" + "=" * 120)
    print("SECTION 2: ESCALATION CURVES — Avg P (pair-match probability) at key picks")
    print("=" * 120)
    print(f"{'Config':<35} {'P@5':>5} {'P@10':>5} {'P@15':>5} {'P@20':>5} "
          f"{'P@25':>5} {'P@30':>5} | {'T@10':>5} {'T@20':>5} {'T@30':>5}")
    print("-" * 110)
    for r in sorted(all_results, key=lambda x: (-x.passes, -x.late_sa)):
        print(f"{r.label:<35} {r.p_at_5:>5.2f} {r.p_at_10:>5.2f} {r.p_at_15:>5.2f} "
              f"{r.p_at_20:>5.2f} {r.p_at_25:>5.2f} {r.p_at_30:>5.2f} | "
              f"{r.targeted_at_10:>5.2f} {r.targeted_at_20:>5.2f} {r.targeted_at_30:>5.2f}")

    # ======================================================================
    # SECTION 3: Three-act draft arc analysis
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 3: THREE-ACT DRAFT ARC (Avg P in each phase)")
    print("  Exploration (picks 1-5) | Commitment (picks 6-15) | Refinement (picks 16-30)")
    print("=" * 100)
    print(f"{'Config':<35} {'Explore':>8} {'Commit':>8} {'Refine':>8} {'Ratio':>8} {'Arc':>10}")
    print("-" * 85)
    for r in sorted(all_results, key=lambda x: (-x.passes, -x.late_sa)):
        ratio = r.refinement_p / r.exploration_p if r.exploration_p > 0.001 else 999
        # Rate the arc
        if r.exploration_p < 0.15 and 0.20 <= r.commitment_p <= 0.50 and r.refinement_p >= 0.35:
            arc = "EXCELLENT"
        elif r.exploration_p < 0.20 and r.commitment_p >= 0.15 and r.refinement_p >= 0.25:
            arc = "GOOD"
        elif r.exploration_p < 0.25:
            arc = "FAIR"
        else:
            arc = "FAST"
        print(f"{r.label:<35} {r.exploration_p:>8.3f} {r.commitment_p:>8.3f} "
              f"{r.refinement_p:>8.3f} {ratio:>7.1f}x {arc:>10}")

    # ======================================================================
    # SECTION 4: K analysis (averaged across C, pack_size)
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 4: K (DIVISOR) ANALYSIS — averaged across C and pack size")
    print("=" * 100)

    for dist_name in distributions:
        print(f"\n  Distribution: {dist_name}")
        print(f"  {'K':>3} {'LateSA':>7} {'StdDev':>7} {'Conv':>6} {'P@10':>6} "
              f"{'P@20':>6} {'T@20':>6} {'DeckC%':>7} {'Passes':>7}")
        print("  " + "-" * 65)
        for K in K_values:
            group = [r for r in all_results if r.K == K and r.dist_label == dist_name]
            if group:
                print(f"  {K:>3} {statistics.mean([r.late_sa for r in group]):>7.2f} "
                      f"{statistics.mean([r.sa_stddev for r in group]):>7.2f} "
                      f"{statistics.mean([r.convergence_pick for r in group]):>6.1f} "
                      f"{statistics.mean([r.p_at_10 for r in group]):>6.3f} "
                      f"{statistics.mean([r.p_at_20 for r in group]):>6.3f} "
                      f"{statistics.mean([r.targeted_at_20 for r in group]):>6.2f} "
                      f"{statistics.mean([r.deck_concentration for r in group])*100:>6.1f}% "
                      f"{statistics.mean([r.passes for r in group]):>6.1f}/8")

    # ======================================================================
    # SECTION 5: C (cap) analysis
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 5: C (CAP) ANALYSIS — averaged across K and pack size")
    print("=" * 100)

    for dist_name in distributions:
        print(f"\n  Distribution: {dist_name}")
        print(f"  {'C':>5} {'LateSA':>7} {'StdDev':>7} {'Conv':>6} {'OffCF':>6} "
              f"{'DeckC%':>7} {'Olap%':>6} {'Passes':>7}")
        print("  " + "-" * 60)
        for C in C_values:
            group = [r for r in all_results if r.C == C and r.dist_label == dist_name]
            if group:
                print(f"  {C:>5.2f} {statistics.mean([r.late_sa for r in group]):>7.2f} "
                      f"{statistics.mean([r.sa_stddev for r in group]):>7.2f} "
                      f"{statistics.mean([r.convergence_pick for r in group]):>6.1f} "
                      f"{statistics.mean([r.late_cf for r in group]):>6.2f} "
                      f"{statistics.mean([r.deck_concentration for r in group])*100:>6.1f}% "
                      f"{statistics.mean([r.overlap for r in group])*100:>5.1f}% "
                      f"{statistics.mean([r.passes for r in group]):>6.1f}/8")

    # ======================================================================
    # SECTION 6: Pack size analysis
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 6: PACK SIZE ANALYSIS — averaged across K and C")
    print("=" * 100)

    for dist_name in distributions:
        print(f"\n  Distribution: {dist_name}")
        print(f"  {'PS':>3} {'LateSA':>7} {'StdDev':>7} {'Conv':>6} {'OffCF':>6} "
              f"{'DeckC%':>7} {'Olap%':>6} {'Passes':>7}")
        print("  " + "-" * 55)
        for ps in pack_sizes:
            group = [r for r in all_results if r.pack_size == ps and r.dist_label == dist_name]
            if group:
                print(f"  {ps:>3} {statistics.mean([r.late_sa for r in group]):>7.2f} "
                      f"{statistics.mean([r.sa_stddev for r in group]):>7.2f} "
                      f"{statistics.mean([r.convergence_pick for r in group]):>6.1f} "
                      f"{statistics.mean([r.late_cf for r in group]):>6.2f} "
                      f"{statistics.mean([r.deck_concentration for r in group])*100:>6.1f}% "
                      f"{statistics.mean([r.overlap for r in group])*100:>5.1f}% "
                      f"{statistics.mean([r.passes for r in group]):>6.1f}/8")

    # ======================================================================
    # SECTION 7: Distribution impact analysis
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 7: SYMBOL DISTRIBUTION IMPACT — averaged across K, C, pack size")
    print("=" * 100)

    print(f"{'Dist':<10} {'LateSA':>7} {'StdDev':>7} {'Conv':>6} {'P@10':>6} "
          f"{'P@20':>6} {'DeckC%':>7} {'Olap%':>6} {'Passes':>7}")
    print("-" * 70)
    for dist_name in distributions:
        group = [r for r in all_results if r.dist_label == dist_name]
        if group:
            print(f"{dist_name:<10} {statistics.mean([r.late_sa for r in group]):>7.2f} "
                  f"{statistics.mean([r.sa_stddev for r in group]):>7.2f} "
                  f"{statistics.mean([r.convergence_pick for r in group]):>6.1f} "
                  f"{statistics.mean([r.p_at_10 for r in group]):>6.3f} "
                  f"{statistics.mean([r.p_at_20 for r in group]):>6.3f} "
                  f"{statistics.mean([r.deck_concentration for r in group])*100:>6.1f}% "
                  f"{statistics.mean([r.overlap for r in group])*100:>5.1f}% "
                  f"{statistics.mean([r.passes for r in group]):>6.1f}/8")

    # ======================================================================
    # SECTION 8: Over/under convergence detection
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 8: CONVERGENCE CLASSIFICATION")
    print("  Over-converged: late S/A > 3.0 or StdDev < 0.8 or overlap >= 40%")
    print("  Under-converged: late S/A < 2.0")
    print("  Sweet spot: 2.0 <= late S/A <= 3.0 and StdDev >= 0.8 and overlap < 40%")
    print("=" * 100)

    over = [r for r in all_results if r.late_sa > 3.0 or r.sa_stddev < 0.8 or r.overlap >= 0.4]
    under = [r for r in all_results if r.late_sa < 2.0]
    sweet = [r for r in all_results
             if 2.0 <= r.late_sa <= 3.0 and r.sa_stddev >= 0.8 and r.overlap < 0.4]

    print(f"\nOver-converged: {len(over)} configs")
    print(f"Under-converged: {len(under)} configs")
    print(f"Sweet spot: {len(sweet)} configs")

    if sweet:
        print(f"\nSweet spot configurations (sorted by passes then late S/A):")
        print(f"{'Config':<35} {'LateSA':>6} {'StdDev':>6} {'Conv':>5} {'OffCF':>5} "
              f"{'DeckC%':>6} {'Olap%':>5} {'Pass':>4}")
        print("-" * 85)
        for r in sorted(sweet, key=lambda x: (-x.passes, -x.late_sa)):
            print(f"{r.label:<35} {r.late_sa:>6.2f} {r.sa_stddev:>6.2f} "
                  f"{r.convergence_pick:>5.1f} {r.late_cf:>5.2f} "
                  f"{r.deck_concentration*100:>5.1f}% {r.overlap*100:>4.1f}% {r.passes:>4}/8")

    # ======================================================================
    # SECTION 9: K x C interaction heatmap (default dist, pack_size=4)
    # ======================================================================
    print("\n" + "=" * 80)
    print("SECTION 9: K x C HEATMAP — Late S/A (default dist, pack_size=4)")
    print("=" * 80)

    print(f"{'K\\C':>5}", end="")
    for C in C_values:
        print(f"  {C:>6.2f}", end="")
    print()
    print("-" * 40)
    for K in K_values:
        print(f"{K:>5}", end="")
        for C in C_values:
            matches = [r for r in all_results
                       if r.K == K and r.C == C and r.pack_size == 4
                       and r.dist_label == "default"]
            if matches:
                v = matches[0].late_sa
                marker = "*" if 2.0 <= v <= 3.0 else " "
                print(f"  {v:>5.2f}{marker}", end="")
            else:
                print(f"  {'N/A':>6}", end="")
        print()

    print("\n  (* = sweet spot: 2.0-3.0 S/A)")

    # Also for pack_size=5
    print(f"\n{'K\\C':>5}", end="")
    for C in C_values:
        print(f"  {C:>6.2f}", end="")
    print("   (pack_size=5)")
    print("-" * 40)
    for K in K_values:
        print(f"{K:>5}", end="")
        for C in C_values:
            matches = [r for r in all_results
                       if r.K == K and r.C == C and r.pack_size == 5
                       and r.dist_label == "default"]
            if matches:
                v = matches[0].late_sa
                marker = "*" if 2.0 <= v <= 3.0 else " "
                print(f"  {v:>5.2f}{marker}", end="")
            else:
                print(f"  {'N/A':>6}", end="")
        print()

    # ======================================================================
    # SECTION 10: Top 5 configs — deep dive with 1000 drafts
    # ======================================================================
    print("\n" + "=" * 120)
    print("SECTION 10: TOP 5 CONFIGURATIONS — Re-run with 1000 drafts")
    print("=" * 120)

    # Pick top 5 from sweet spot (or highest passes)
    candidates = sorted(all_results, key=lambda x: (-x.passes, -x.late_sa))
    top5_params = []
    seen = set()
    for r in candidates[:20]:
        key = (r.K, r.C, r.pack_size, r.dist_label)
        if key not in seen:
            seen.add(key)
            top5_params.append(key)
        if len(top5_params) >= 5:
            break

    top5_results = []
    for K, C, ps, dist_name in top5_params:
        pool = pools[dist_name]
        print(f"  Re-running K={K} C={C} PS={ps} {dist_name} with 1000 drafts...")
        r = run_configuration(pool, K, C, ps, dist_name,
                              num_drafts=1000, rng_seed=99999)
        top5_results.append(r)

    print(f"\n{'Config':<35} {'LateSA':>6} {'StdDev':>6} {'Conv':>5} {'OffCF':>5} "
          f"{'DeckC%':>6} {'Olap%':>5} {'EDiv':>5} {'ESA':>5} {'Pass':>4}")
    print("-" * 100)
    for r in top5_results:
        print(f"{r.label:<35} {r.late_sa:>6.2f} {r.sa_stddev:>6.2f} "
              f"{r.convergence_pick:>5.1f} {r.late_cf:>5.2f} "
              f"{r.deck_concentration*100:>5.1f}% {r.overlap*100:>4.1f}% "
              f"{r.early_unique_archs:>5.1f} {r.early_sa_for_arch:>5.2f} {r.passes:>4}/8")

    print(f"\n  Escalation curves for top 5:")
    print(f"{'Config':<35} {'P@5':>5} {'P@10':>5} {'P@15':>5} {'P@20':>5} "
          f"{'P@25':>5} {'P@30':>5} | {'T@10':>5} {'T@20':>5} {'T@30':>5}")
    print("-" * 100)
    for r in top5_results:
        print(f"{r.label:<35} {r.p_at_5:>5.2f} {r.p_at_10:>5.2f} {r.p_at_15:>5.2f} "
              f"{r.p_at_20:>5.2f} {r.p_at_25:>5.2f} {r.p_at_30:>5.2f} | "
              f"{r.targeted_at_10:>5.2f} {r.targeted_at_20:>5.2f} {r.targeted_at_30:>5.2f}")

    print(f"\n  Three-act arc for top 5:")
    print(f"{'Config':<35} {'Explore':>8} {'Commit':>8} {'Refine':>8} {'Ratio':>8}")
    print("-" * 75)
    for r in top5_results:
        ratio = r.refinement_p / r.exploration_p if r.exploration_p > 0.001 else 999
        print(f"{r.label:<35} {r.exploration_p:>8.3f} {r.commitment_p:>8.3f} "
              f"{r.refinement_p:>8.3f} {ratio:>7.1f}x")

    # ======================================================================
    # SECTION 11: Scorecard for top 5
    # ======================================================================
    print("\n" + "=" * 120)
    print("SECTION 11: SCORECARD — TOP 5 vs ALL TARGETS")
    print("=" * 120)

    targets = [
        ("Picks 1-5: unique archs >= 3", lambda r: r.early_unique_archs >= 3,
         lambda r: f"{r.early_unique_archs:.1f}"),
        ("Picks 1-5: S/A <= 2", lambda r: r.early_sa_for_arch <= 2,
         lambda r: f"{r.early_sa_for_arch:.2f}"),
        ("Picks 6+: S/A >= 2.0 avg", lambda r: r.late_sa >= 2.0,
         lambda r: f"{r.late_sa:.2f}"),
        ("Picks 6+: C/F >= 0.5 avg", lambda r: r.late_cf >= 0.5,
         lambda r: f"{r.late_cf:.2f}"),
        ("Convergence pick 5-8", lambda r: 5 <= r.convergence_pick <= 8,
         lambda r: f"{r.convergence_pick:.1f}"),
        ("Deck concentration 60-90%", lambda r: 0.6 <= r.deck_concentration <= 0.9,
         lambda r: f"{r.deck_concentration*100:.1f}%"),
        ("Run-to-run overlap < 40%", lambda r: r.overlap < 0.4,
         lambda r: f"{r.overlap*100:.1f}%"),
        ("S/A StdDev >= 0.8", lambda r: r.sa_stddev >= 0.8,
         lambda r: f"{r.sa_stddev:.2f}"),
    ]

    header = f"{'Metric':<35}"
    for r in top5_results:
        header += f" {r.label[:20]:>22}"
    print(header)
    print("-" * (35 + 23 * len(top5_results)))

    for name, check_fn, fmt_fn in targets:
        row = f"{name:<35}"
        for r in top5_results:
            val_str = fmt_fn(r)
            passed = "PASS" if check_fn(r) else "FAIL"
            row += f" {val_str:>12} ({passed})"
        print(row)

    # Total passes row
    row = f"{'TOTAL PASSES':<35}"
    for r in top5_results:
        row += f" {r.passes:>17}/8"
    print(row)

    # ======================================================================
    # SECTION 12: Recommended configuration
    # ======================================================================
    print("\n" + "=" * 100)
    print("SECTION 12: RECOMMENDED CONFIGURATION")
    print("=" * 100)

    best = max(top5_results, key=lambda r: (r.passes, r.late_sa))
    print(f"\n  BEST: {best.label}")
    print(f"  K = {best.K}, C = {best.C}, Pack Size = {best.pack_size}")
    print(f"  Distribution: {best.dist_label}")
    print(f"")
    print(f"  Late S/A:          {best.late_sa:.2f}  (target >= 2.0)")
    print(f"  S/A StdDev:        {best.sa_stddev:.2f}  (target >= 0.8)")
    print(f"  Convergence pick:  {best.convergence_pick:.1f}  (target 5-8)")
    print(f"  Off-arch C/F:      {best.late_cf:.2f}  (target >= 0.5)")
    print(f"  Deck concentration:{best.deck_concentration*100:.1f}%  (target 60-90%)")
    print(f"  Run-to-run overlap:{best.overlap*100:.1f}%  (target < 40%)")
    print(f"  Early diversity:   {best.early_unique_archs:.1f}  (target >= 3)")
    print(f"  Early S/A:         {best.early_sa_for_arch:.2f}  (target <= 2)")
    print(f"  Passes:            {best.passes}/8")
    print(f"")
    print(f"  Escalation curve:")
    print(f"    P@5={best.p_at_5:.2f}, P@10={best.p_at_10:.2f}, P@15={best.p_at_15:.2f}, "
          f"P@20={best.p_at_20:.2f}, P@25={best.p_at_25:.2f}, P@30={best.p_at_30:.2f}")
    print(f"  Expected targeted slots:")
    print(f"    @10: {best.targeted_at_10:.2f}, @20: {best.targeted_at_20:.2f}, "
          f"@30: {best.targeted_at_30:.2f}")
    print(f"  Three-act arc:")
    print(f"    Exploration (1-5): P={best.exploration_p:.3f}")
    print(f"    Commitment (6-15): P={best.commitment_p:.3f}")
    print(f"    Refinement (16-30): P={best.refinement_p:.3f}")

    print("\n\nSimulation complete.")


if __name__ == "__main__":
    main()
