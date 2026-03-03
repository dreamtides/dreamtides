#!/usr/bin/env python3
"""
V5 Pool Design Synthesis Simulation: Pair-Escalation Slots

Reconciles findings from all 5 Round 1 agents into a unified pool specification
and tests it against 4 configurations:

  Baseline:  V5 default pool (mixed patterns, K=6, C=0.50) -- the starting point
  Config A:  Reconciled best (high-PS patterns + bridges, K=10, C=0.65)
  Config B:  Reconciled pool with V5 defaults (K=6, C=0.50)
  Config C:  Reconciled pool, moderate params (K=8, C=0.50)

Reconciled pool design:
  - 360 cards total: 36 generic, 276 archetype, 48 bridge
  - 8 archetypes x 34.5 cards each (rounded: alternating 34/35)
  - 48 bridge cards: 6 per adjacent pair, alternating pair ownership
  - Symbol distribution: 15% 1-sym, 60% 2-sym, 25% 3-sym (Agent 1 validated)
  - Pattern composition: 85% [P,S] for 2-sym, 100% [P,S,X] for 3-sym (Agent 4)
  - Rarity: Standard TCG (cosmetic, no symbol correlation) (Agent 2)
  - Algorithm params: K=10, C=0.65, Pack Size=4 (Agent 5)

Three player strategies:
  - Committed: picks best fitness for target archetype
  - Power Chaser: picks highest power card regardless of archetype
  - Signal Reader: starts flexible, commits to archetype with most S/A seen

1000 drafts per configuration per strategy.
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
    # (name, primary, secondary, id)
    ("Flash",        "Zephyr", "Ember"),   # 0
    ("Blink",        "Ember",  "Zephyr"),  # 1
    ("Storm",        "Ember",  "Stone"),   # 2
    ("Self-Discard", "Stone",  "Ember"),   # 3
    ("Self-Mill",    "Stone",  "Tide"),    # 4
    ("Sacrifice",    "Tide",   "Stone"),   # 5
    ("Warriors",     "Tide",   "Zephyr"),  # 6
    ("Ramp",         "Zephyr", "Tide"),    # 7
]

# Adjacent pairs on the circle (each archetype shares a resonance with neighbors)
ADJACENT_PAIRS = [
    (0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 7), (7, 0)
]

NUM_ARCHETYPES = 8
NUM_CARDS = 360
GENERIC_COUNT = 36
NUM_PICKS = 30
PACK_SIZE = 4
NUM_DRAFTS = 1000

# Pair-to-archetype mapping
PAIR_TO_ARCH = {}
for _i, (_name, _pri, _sec) in enumerate(ARCHETYPE_DEFS):
    PAIR_TO_ARCH[(_pri, _sec)] = _i


# ---------------------------------------------------------------------------
# Card model
# ---------------------------------------------------------------------------

@dataclass
class SimCard:
    id: int
    symbols: list           # ordered resonance symbols
    archetype_idx: int      # -1 for generic
    power: float
    is_bridge: bool = False
    bridge_archetypes: tuple = ()  # (arch_a, arch_b) if bridge
    fitness: dict = field(default_factory=dict)  # arch_idx -> tier string

    @property
    def ordered_pair(self):
        if len(self.symbols) >= 2:
            return (self.symbols[0], self.symbols[1])
        return None


def circle_distance(a: int, b: int) -> int:
    d = abs(a - b)
    return min(d, NUM_ARCHETYPES - d)


def compute_fitness(card_arch_idx: int, is_bridge: bool = False,
                    bridge_archs: tuple = ()) -> dict:
    """Compute fitness tiers for a card."""
    fitness = {}
    if card_arch_idx < 0:
        # Generic: B-tier for all
        for j in range(NUM_ARCHETYPES):
            fitness[j] = "B"
        return fitness

    # Bridge cards are S-tier for both adjacent archetypes
    if is_bridge and bridge_archs:
        for j in range(NUM_ARCHETYPES):
            if j in bridge_archs:
                fitness[j] = "S"
            elif circle_distance(j, bridge_archs[0]) == 1 or circle_distance(j, bridge_archs[1]) == 1:
                # Adjacent to either bridge archetype
                home_pri = ARCHETYPE_DEFS[card_arch_idx][1]
                target_pri = ARCHETYPE_DEFS[j][1]
                target_sec = ARCHETYPE_DEFS[j][2]
                if target_pri == home_pri or target_sec == home_pri:
                    fitness[j] = "A"
                else:
                    fitness[j] = "B"
            elif circle_distance(j, card_arch_idx) <= 2:
                fitness[j] = "C"
            else:
                fitness[j] = "F"
        return fitness

    # Normal archetype card
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
# Pool construction: Reconciled design
# ---------------------------------------------------------------------------

def build_reconciled_pool(rng, use_high_ps=True, use_bridges=True):
    """
    Build the reconciled pool design:
    - 36 generic, 48 bridge (if enabled), remainder split across 8 archetypes
    - Symbol distribution: 15/60/25
    - Pattern composition: 85% [P,S] for 2-sym if use_high_ps, else mixed
    """
    cards = []
    card_id = 0

    bridge_count = 48 if use_bridges else 0
    archetype_total = NUM_CARDS - GENERIC_COUNT - bridge_count
    cards_per_arch = archetype_total // NUM_ARCHETYPES  # 34 with bridges, 40 without
    remainder = archetype_total - (cards_per_arch * NUM_ARCHETYPES)

    # Build archetype cards
    for arch_idx in range(NUM_ARCHETYPES):
        _name, primary, secondary = ARCHETYPE_DEFS[arch_idx]
        n = cards_per_arch + (1 if arch_idx < remainder else 0)

        n1 = round(n * 0.15)
        n3 = round(n * 0.25)
        n2 = n - n1 - n3

        # 1-symbol cards: 70% primary, 30% secondary
        for _ in range(n1):
            sym = primary if rng.random() < 0.70 else secondary
            c = SimCard(id=card_id, symbols=[sym], archetype_idx=arch_idx,
                        power=rng.uniform(3, 8))
            c.fitness = compute_fitness(arch_idx)
            cards.append(c)
            card_id += 1

        # 2-symbol cards
        for _ in range(n2):
            if use_high_ps:
                # 85% [P,S], 10% [P,P], 5% [S,P] -- Agent 4 recommendation
                roll = rng.random()
                if roll < 0.85:
                    syms = [primary, secondary]
                elif roll < 0.95:
                    syms = [primary, primary]
                else:
                    syms = [secondary, primary]
            else:
                # Mixed baseline: 50% [P,S], 25% [P,P], 25% [S,P]
                roll = rng.random()
                if roll < 0.50:
                    syms = [primary, secondary]
                elif roll < 0.75:
                    syms = [primary, primary]
                else:
                    syms = [secondary, primary]
            c = SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx,
                        power=rng.uniform(3, 8))
            c.fitness = compute_fitness(arch_idx)
            cards.append(c)
            card_id += 1

        # 3-symbol cards: all start [P,S,...] to produce home pair
        for _ in range(n3):
            if use_high_ps:
                # All produce home pair: [P,S,P], [P,S,S], [P,S,O]
                third = rng.choice([primary, secondary,
                                     rng.choice([r for r in RESONANCES
                                                 if r != primary and r != secondary])])
                syms = [primary, secondary, third]
            else:
                # Mixed: some [P,P,S], [P,S,P], etc.
                pat = rng.choice([
                    [primary, secondary, primary],
                    [primary, primary, secondary],
                    [primary, secondary, secondary],
                ])
                syms = pat
            c = SimCard(id=card_id, symbols=syms, archetype_idx=arch_idx,
                        power=rng.uniform(3, 8))
            c.fitness = compute_fitness(arch_idx)
            cards.append(c)
            card_id += 1

    # Bridge cards: 6 per adjacent pair
    if use_bridges:
        for a_idx, b_idx in ADJACENT_PAIRS:
            _name_a, pri_a, sec_a = ARCHETYPE_DEFS[a_idx]
            _name_b, pri_b, sec_b = ARCHETYPE_DEFS[b_idx]
            # Shared resonance between adjacent archetypes
            shared = set([pri_a, sec_a]) & set([pri_b, sec_b])
            shared_res = list(shared)[0] if shared else pri_a

            for i in range(6):
                # Alternate ownership: 3 cards owned by arch_a, 3 by arch_b
                if i < 3:
                    owner = a_idx
                    # Bridge card uses owner's pair: [P_owner, S_owner]
                    syms = [ARCHETYPE_DEFS[owner][1], ARCHETYPE_DEFS[owner][2]]
                else:
                    owner = b_idx
                    syms = [ARCHETYPE_DEFS[owner][1], ARCHETYPE_DEFS[owner][2]]

                c = SimCard(id=card_id, symbols=syms, archetype_idx=owner,
                            power=rng.uniform(4, 8), is_bridge=True,
                            bridge_archetypes=(a_idx, b_idx))
                c.fitness = compute_fitness(owner, is_bridge=True,
                                            bridge_archs=(a_idx, b_idx))
                cards.append(c)
                card_id += 1

    # Generic cards
    for _ in range(GENERIC_COUNT):
        c = SimCard(id=card_id, symbols=[], archetype_idx=-1,
                    power=rng.uniform(4, 9))
        c.fitness = compute_fitness(-1)
        cards.append(c)
        card_id += 1

    return cards


def build_baseline_pool(rng):
    """Build V5 default pool: no bridges, mixed patterns, 40 cards per arch."""
    return build_reconciled_pool(rng, use_high_ps=False, use_bridges=False)


# ---------------------------------------------------------------------------
# Pair-Escalation Slots algorithm
# ---------------------------------------------------------------------------

def get_top_pair(pair_counts):
    if not pair_counts:
        return None, 0
    top_pair, top_count = max(pair_counts.items(), key=lambda x: x[1])
    return top_pair, top_count


def gen_pack(pool, pair_counts, rng, K, C):
    """Generate a pack using Pair-Escalation Slots."""
    top_pair, top_count = get_top_pair(pair_counts)
    prob = min(top_count / K, C) if top_pair else 0.0
    pair_matched = [c for c in pool if c.ordered_pair == top_pair] if top_pair else []

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


# ---------------------------------------------------------------------------
# Player strategies
# ---------------------------------------------------------------------------

TIER_ORDER = {"S": 5, "A": 4, "B": 3, "C": 2, "F": 1}


def pick_committed(pack, target_arch):
    """Pick best fitness for target archetype."""
    return max(pack, key=lambda c: (
        TIER_ORDER.get(c.fitness.get(target_arch, "F"), 0), c.power))


def pick_power_chaser(pack, _target_arch):
    """Pick highest power card regardless of archetype."""
    return max(pack, key=lambda c: c.power)


def pick_signal_reader(pack, target_arch, pick_num, arch_sa_seen):
    """
    Flexible early, then commit to archetype with most cumulative S/A cards seen.
    Before pick 6: pick highest power among S/A cards for any top-3 archetype.
    After pick 5: commit to best archetype by total S/A seen, then pick as committed.
    """
    # Update tracking: count S/A cards per archetype in this pack
    for c in pack:
        for ai in range(NUM_ARCHETYPES):
            if is_sa(c, ai):
                arch_sa_seen[ai] = arch_sa_seen.get(ai, 0) + 1

    if pick_num < 5:
        # Flexible: pick highest power among cards that are S/A for any archetype
        sa_cards = [c for c in pack
                    if any(is_sa(c, ai) for ai in range(NUM_ARCHETYPES))]
        if sa_cards:
            return max(sa_cards, key=lambda c: c.power)
        return max(pack, key=lambda c: c.power)
    else:
        # Commit to the archetype with most S/A cards seen so far
        if target_arch is None:
            target_arch = max(range(NUM_ARCHETYPES),
                              key=lambda ai: arch_sa_seen.get(ai, 0))
        return pick_committed(pack, target_arch), target_arch


# ---------------------------------------------------------------------------
# Draft simulation
# ---------------------------------------------------------------------------

@dataclass
class DraftRecord:
    sa_per_pack: list = field(default_factory=list)
    cf_per_pack: list = field(default_factory=list)
    prob_per_pack: list = field(default_factory=list)
    unique_sa_archs_per_pack: list = field(default_factory=list)
    target_archetype: int = 0
    card_ids: list = field(default_factory=list)
    pair_counts_history: list = field(default_factory=list)


def run_single_draft(pool, target_arch, K, C, rng, strategy="committed"):
    """Run a single 30-pick draft."""
    pair_counts = {}
    record = DraftRecord(target_archetype=target_arch)
    arch_sa_seen = {}  # for signal reader
    committed_arch = target_arch if strategy == "committed" else None

    for pick_num in range(NUM_PICKS):
        pack, prob = gen_pack(pool, pair_counts, rng, K, C)
        record.prob_per_pack.append(prob)

        # Measure S/A and C/F for eventual target
        effective_target = committed_arch if committed_arch is not None else target_arch
        sa_count = sum(1 for c in pack if is_sa(c, effective_target))
        cf_count = sum(1 for c in pack if is_cf(c, effective_target))
        record.sa_per_pack.append(sa_count)
        record.cf_per_pack.append(cf_count)

        # Unique archetypes with S/A
        unique_archs = set()
        for c in pack:
            for ai in range(NUM_ARCHETYPES):
                if is_sa(c, ai):
                    unique_archs.add(ai)
        record.unique_sa_archs_per_pack.append(len(unique_archs))

        # Pick card based on strategy
        if strategy == "committed":
            chosen = pick_committed(pack, target_arch)
        elif strategy == "power_chaser":
            chosen = pick_power_chaser(pack, target_arch)
        elif strategy == "signal_reader":
            result = pick_signal_reader(pack, committed_arch, pick_num, arch_sa_seen)
            if isinstance(result, tuple):
                chosen, committed_arch = result
            else:
                chosen = result
        else:
            chosen = pick_committed(pack, target_arch)

        record.card_ids.append(chosen.id)

        # Update pair counter
        if chosen.ordered_pair:
            pair = chosen.ordered_pair
            pair_counts[pair] = pair_counts.get(pair, 0) + 1

        # Record pair count snapshot
        top_pair, top_count = get_top_pair(pair_counts)
        record.pair_counts_history.append(top_count)

    if committed_arch is not None:
        record.target_archetype = committed_arch
    return record


# ---------------------------------------------------------------------------
# Metrics computation
# ---------------------------------------------------------------------------

def compute_metrics(records, pool):
    """Compute all measurable targets from a batch of draft records."""
    early_unique = []
    early_sa = []
    late_sa_vals = []
    late_cf_vals = []
    conv_picks = []
    deck_concs = []
    all_card_sets = []
    arch_freq = Counter()
    late_sa_per_draft = []

    # Pair economy tracking
    pair_count_at_picks = defaultdict(list)  # pick_num -> list of top pair counts

    for rec in records:
        tgt = rec.target_archetype
        arch_freq[tgt] += 1

        # Early metrics (picks 1-5)
        for i in range(min(5, len(rec.sa_per_pack))):
            early_unique.append(rec.unique_sa_archs_per_pack[i])
            early_sa.append(rec.sa_per_pack[i])

        # Late metrics (picks 6-30)
        draft_late_sa = []
        for i in range(5, NUM_PICKS):
            late_sa_vals.append(rec.sa_per_pack[i])
            late_cf_vals.append(rec.cf_per_pack[i])
            draft_late_sa.append(rec.sa_per_pack[i])
        if draft_late_sa:
            late_sa_per_draft.append(statistics.mean(draft_late_sa))

        # Convergence: first pick where 3-pack rolling avg >= 2.0
        conv = NUM_PICKS
        for i in range(2, NUM_PICKS):
            window = rec.sa_per_pack[max(0, i - 2):i + 1]
            if sum(window) / len(window) >= 2.0:
                conv = i + 1  # 1-indexed
                break
        conv_picks.append(conv)

        # Deck concentration
        # Count picks where at least 1 S/A was available (committed player picks it)
        sa_picked = sum(1 for sa in rec.sa_per_pack if sa >= 1)
        deck_concs.append(sa_picked / NUM_PICKS)

        all_card_sets.append(set(rec.card_ids))

        # Pair economy
        for pn, top_count in enumerate(rec.pair_counts_history):
            pair_count_at_picks[pn].append(top_count)

    # Run-to-run overlap
    by_arch = defaultdict(list)
    for i, rec in enumerate(records):
        by_arch[rec.target_archetype].append(i)
    overlaps = []
    for arch, indices in by_arch.items():
        for i in range(min(30, len(indices))):
            for j in range(i + 1, min(30, len(indices))):
                s1 = all_card_sets[indices[i]]
                s2 = all_card_sets[indices[j]]
                if s1 and s2:
                    union = len(s1 | s2)
                    if union > 0:
                        overlaps.append(len(s1 & s2) / union)

    total = len(records)
    af = {a: arch_freq.get(a, 0) / total for a in range(NUM_ARCHETYPES)}

    # Pair economy: average top pair count at key picks
    pair_at = {}
    for pn in [4, 7, 9, 14, 19, 24, 29]:
        vals = pair_count_at_picks.get(pn, [0])
        pair_at[pn + 1] = statistics.mean(vals) if vals else 0

    # Probability at key picks
    prob_at = {}
    for pick_idx in [4, 9, 14, 19, 29]:
        vals = [rec.prob_per_pack[pick_idx] for rec in records
                if len(rec.prob_per_pack) > pick_idx]
        prob_at[pick_idx + 1] = statistics.mean(vals) if vals else 0

    return {
        "early_unique": statistics.mean(early_unique) if early_unique else 0,
        "early_sa": statistics.mean(early_sa) if early_sa else 0,
        "late_sa": statistics.mean(late_sa_vals) if late_sa_vals else 0,
        "late_cf": statistics.mean(late_cf_vals) if late_cf_vals else 0,
        "conv_pick": statistics.mean(conv_picks) if conv_picks else 30,
        "deck_conc": statistics.mean(deck_concs) if deck_concs else 0,
        "overlap": statistics.mean(overlaps) if overlaps else 0,
        "arch_freq_max": max(af.values()) if af else 0,
        "arch_freq_min": min(af.values()) if af else 0,
        "stddev": statistics.stdev(late_sa_vals) if len(late_sa_vals) > 1 else 0,
        "pair_at": pair_at,
        "prob_at": prob_at,
    }


def count_passes(m):
    """Count how many of the 8 measurable targets pass."""
    passes = 0
    if m["early_unique"] >= 3: passes += 1
    if m["early_sa"] <= 2: passes += 1
    if m["late_sa"] >= 2.0: passes += 1
    if m["late_cf"] >= 0.5: passes += 1
    if 5 <= m["conv_pick"] <= 8: passes += 1
    if 0.60 <= m["deck_conc"] <= 0.90: passes += 1
    if m["overlap"] < 0.40: passes += 1
    if m["stddev"] >= 0.8: passes += 1
    return passes


# ---------------------------------------------------------------------------
# Pool analysis
# ---------------------------------------------------------------------------

def analyze_pool(pool):
    """Print pool composition analysis."""
    total = len(pool)
    generic = sum(1 for c in pool if c.archetype_idx < 0)
    bridge = sum(1 for c in pool if c.is_bridge)
    archetype = total - generic - bridge

    sym_counts = Counter(len(c.symbols) for c in pool if c.archetype_idx >= 0)

    # Pair pool analysis per archetype
    pair_pool_sizes = []
    pair_pool_s_pct = []
    for arch_idx in range(NUM_ARCHETYPES):
        _name, pri, sec = ARCHETYPE_DEFS[arch_idx]
        home_pair = (pri, sec)
        matched = [c for c in pool if c.ordered_pair == home_pair]
        pair_pool_sizes.append(len(matched))
        if matched:
            s_count = sum(1 for c in matched if c.fitness.get(arch_idx, "F") == "S")
            sa_count = sum(1 for c in matched if is_sa(c, arch_idx))
            pair_pool_s_pct.append(sa_count / len(matched))
        else:
            pair_pool_s_pct.append(0)

    # Pattern breakdown for 2-sym non-generic cards
    ps_count = 0
    pp_count = 0
    sp_count = 0
    other_2sym = 0
    for c in pool:
        if c.archetype_idx >= 0 and len(c.symbols) == 2:
            _name, pri, sec = ARCHETYPE_DEFS[c.archetype_idx]
            if c.symbols == [pri, sec]:
                ps_count += 1
            elif c.symbols == [pri, pri]:
                pp_count += 1
            elif c.symbols == [sec, pri]:
                sp_count += 1
            else:
                other_2sym += 1

    total_2sym = ps_count + pp_count + sp_count + other_2sym

    return {
        "total": total,
        "generic": generic,
        "bridge": bridge,
        "archetype": archetype,
        "sym_counts": dict(sym_counts),
        "pair_pool_sizes": pair_pool_sizes,
        "pair_pool_s_pct": pair_pool_s_pct,
        "avg_pair_pool": statistics.mean(pair_pool_sizes),
        "avg_pair_sa_pct": statistics.mean(pair_pool_s_pct),
        "pattern_2sym": {"PS": ps_count, "PP": pp_count, "SP": sp_count,
                         "other": other_2sym, "total": total_2sym},
        "ps_pct": ps_count / total_2sym if total_2sym > 0 else 0,
    }


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    print("=" * 120)
    print("  V5 POOL DESIGN SYNTHESIS SIMULATION: Pair-Escalation Slots")
    print(f"  {NUM_DRAFTS} drafts per configuration per strategy, {NUM_PICKS} picks per draft")
    print("=" * 120)

    # Define configurations
    configs = {
        "Baseline": {
            "desc": "V5 default: mixed patterns, no bridges, K=6, C=0.50",
            "K": 6, "C": 0.50, "high_ps": False, "bridges": False,
        },
        "Config A (Reconciled)": {
            "desc": "Best: high-PS + bridges, K=10, C=0.65",
            "K": 10, "C": 0.65, "high_ps": True, "bridges": True,
        },
        "Config B (Pool+V5)": {
            "desc": "Reconciled pool, V5 params: K=6, C=0.50",
            "K": 6, "C": 0.50, "high_ps": True, "bridges": True,
        },
        "Config C (Moderate)": {
            "desc": "Reconciled pool, moderate: K=8, C=0.50",
            "K": 8, "C": 0.50, "high_ps": True, "bridges": True,
        },
    }

    strategies = ["committed", "power_chaser", "signal_reader"]

    # ======================================================================
    # SECTION 1: Pool Composition Analysis
    # ======================================================================
    print("\n" + "=" * 100)
    print("  SECTION 1: POOL COMPOSITION ANALYSIS")
    print("=" * 100)

    pools = {}
    for config_name, cfg in configs.items():
        pool_rng = random.Random(42)
        pool = build_reconciled_pool(pool_rng, use_high_ps=cfg["high_ps"],
                                     use_bridges=cfg["bridges"])
        pools[config_name] = pool
        pa = analyze_pool(pool)

        print(f"\n  {config_name}: {cfg['desc']}")
        print(f"    Total: {pa['total']} cards | Generic: {pa['generic']} | "
              f"Bridge: {pa['bridge']} | Archetype: {pa['archetype']}")
        print(f"    Symbol counts: {pa['sym_counts']}")
        print(f"    2-sym patterns: PS={pa['pattern_2sym']['PS']} ({pa['ps_pct']:.0%}), "
              f"PP={pa['pattern_2sym']['PP']}, SP={pa['pattern_2sym']['SP']}, "
              f"other={pa['pattern_2sym']['other']}")
        print(f"    Avg pair pool size: {pa['avg_pair_pool']:.1f} cards, "
              f"S/A precision: {pa['avg_pair_sa_pct']:.1%}")
        print(f"    Per-archetype pair pools: {pa['pair_pool_sizes']}")

    # ======================================================================
    # SECTION 2: Run Simulations
    # ======================================================================
    print("\n\n" + "=" * 100)
    print("  SECTION 2: SIMULATION RESULTS")
    print("=" * 100)

    all_results = {}  # (config_name, strategy) -> metrics

    for config_name, cfg in configs.items():
        pool = pools[config_name]
        K, C = cfg["K"], cfg["C"]

        for strat in strategies:
            label = f"{config_name} | {strat}"
            print(f"  Running: {label}...", flush=True)

            records = []
            for i in range(NUM_DRAFTS):
                rng = random.Random(1000 + i)
                target_arch = rng.randint(0, NUM_ARCHETYPES - 1)
                rec = run_single_draft(pool, target_arch, K, C, rng, strategy=strat)
                records.append(rec)

            metrics = compute_metrics(records, pool)
            metrics["passes"] = count_passes(metrics)
            all_results[(config_name, strat)] = metrics

    # ======================================================================
    # SECTION 3: Committed Strategy Comparison (Primary)
    # ======================================================================
    print("\n\n" + "=" * 120)
    print("  SECTION 3: COMMITTED STRATEGY -- FULL SCORECARD")
    print("=" * 120)

    targets = [
        ("Early unique archs (>=3)",   "early_unique", lambda v: v >= 3,     "{:.1f}"),
        ("Early S/A (<=2)",            "early_sa",     lambda v: v <= 2,     "{:.2f}"),
        ("Late S/A (>=2)",             "late_sa",      lambda v: v >= 2,     "{:.2f}"),
        ("Late off-arch C/F (>=0.5)",  "late_cf",      lambda v: v >= 0.5,   "{:.2f}"),
        ("Convergence pick (5-8)",     "conv_pick",    lambda v: 5 <= v <= 8, "{:.1f}"),
        ("Deck concentration (60-90%)","deck_conc",    lambda v: 0.60 <= v <= 0.90, "{:.1%}"),
        ("Run-to-run overlap (<40%)",  "overlap",      lambda v: v < 0.40,   "{:.1%}"),
        ("S/A StdDev (>=0.8)",         "stddev",       lambda v: v >= 0.8,   "{:.2f}"),
    ]

    config_names = list(configs.keys())
    header = f"  {'Metric':<32}"
    for cn in config_names:
        header += f" | {cn[:22]:>24}"
    print(header)
    print("  " + "-" * (32 + 27 * len(config_names)))

    for name, key, check, fmt in targets:
        row = f"  {name:<32}"
        for cn in config_names:
            m = all_results[(cn, "committed")]
            v = m[key]
            vs = fmt.format(v)
            pf = "PASS" if check(v) else "FAIL"
            row += f" | {vs:>16} {pf:>4}"
        print(row)

    # Pass totals
    print()
    row = f"  {'TOTAL PASSES':<32}"
    for cn in config_names:
        m = all_results[(cn, "committed")]
        row += f" | {m['passes']:>19}/8"
    print(row)

    # ======================================================================
    # SECTION 4: Strategy Comparison
    # ======================================================================
    print("\n\n" + "=" * 120)
    print("  SECTION 4: STRATEGY COMPARISON (Late S/A, Conv Pick, Deck Conc)")
    print("=" * 120)

    header = f"  {'Config + Strategy':<45} {'Late S/A':>9} {'Conv':>6} {'Deck%':>7} {'StdDev':>7} {'Off-CF':>7} {'Pass':>5}"
    print(header)
    print("  " + "-" * 90)

    for cn in config_names:
        for strat in strategies:
            m = all_results[(cn, strat)]
            label = f"{cn[:28]} | {strat}"
            print(f"  {label:<45} {m['late_sa']:>9.2f} {m['conv_pick']:>6.1f} "
                  f"{m['deck_conc']:>6.1%} {m['stddev']:>7.2f} {m['late_cf']:>7.2f} "
                  f"{m['passes']:>4}/8")
        print()

    # ======================================================================
    # SECTION 5: Pair Economy Analysis
    # ======================================================================
    print("\n" + "=" * 120)
    print("  SECTION 5: PAIR ECONOMY ANALYSIS (Committed Strategy)")
    print("  Average top pair count and P (probability) at key draft picks")
    print("=" * 120)

    print(f"\n  {'Config':<30} {'Pairs@5':>8} {'Pairs@8':>8} {'Pairs@10':>9} "
          f"{'Pairs@15':>9} {'Pairs@20':>9} {'Pairs@30':>9}")
    print("  " + "-" * 85)
    for cn in config_names:
        m = all_results[(cn, "committed")]
        pa = m["pair_at"]
        print(f"  {cn:<30} {pa.get(5,0):>8.1f} {pa.get(8,0):>8.1f} {pa.get(10,0):>9.1f} "
              f"{pa.get(15,0):>9.1f} {pa.get(20,0):>9.1f} {pa.get(30,0):>9.1f}")

    K_for_config = {cn: cfg["K"] for cn, cfg in configs.items()}
    C_for_config = {cn: cfg["C"] for cn, cfg in configs.items()}

    print(f"\n  {'Config':<30} {'P@5':>6} {'P@10':>7} {'P@15':>7} {'P@20':>7} {'P@30':>7} "
          f"{'Tgt@10':>8} {'Tgt@20':>8} {'Tgt@30':>8}")
    print("  " + "-" * 85)
    for cn in config_names:
        m = all_results[(cn, "committed")]
        pa = m["prob_at"]
        K = K_for_config[cn]
        C_val = C_for_config[cn]
        print(f"  {cn:<30} {pa.get(5,0):>6.2f} {pa.get(10,0):>7.2f} {pa.get(15,0):>7.2f} "
              f"{pa.get(20,0):>7.2f} {pa.get(30,0):>7.2f} "
              f"{pa.get(10,0)*PACK_SIZE:>8.2f} {pa.get(20,0)*PACK_SIZE:>8.2f} "
              f"{pa.get(30,0)*PACK_SIZE:>8.2f}")

    # ======================================================================
    # SECTION 6: Three-Act Arc Analysis
    # ======================================================================
    print("\n\n" + "=" * 100)
    print("  SECTION 6: THREE-ACT DRAFT ARC (Committed Strategy)")
    print("  Exploration (1-5) | Commitment (6-15) | Refinement (16-30)")
    print("=" * 100)

    print(f"\n  {'Config':<30} {'Explore P':>10} {'Commit P':>10} {'Refine P':>10} {'Ratio':>8}")
    print("  " + "-" * 72)
    for cn in config_names:
        m = all_results[(cn, "committed")]
        pa = m["prob_at"]
        # Average P in each phase from prob_at data points
        explore_p = pa.get(5, 0)
        commit_p = pa.get(10, 0)
        refine_p = (pa.get(20, 0) + pa.get(30, 0)) / 2 if pa.get(20, 0) else pa.get(15, 0)
        ratio = refine_p / explore_p if explore_p > 0.001 else 999
        print(f"  {cn:<30} {explore_p:>10.3f} {commit_p:>10.3f} {refine_p:>10.3f} {ratio:>7.1f}x")

    # ======================================================================
    # SECTION 7: Per-Archetype Balance
    # ======================================================================
    print("\n\n" + "=" * 100)
    print("  SECTION 7: PER-ARCHETYPE CONVERGENCE (Config A, Committed)")
    print("=" * 100)

    # Re-run Config A with archetype-specific tracking
    cfg_a = configs["Config A (Reconciled)"]
    pool_a = pools["Config A (Reconciled)"]
    per_arch_conv = defaultdict(list)
    per_arch_late_sa = defaultdict(list)

    for i in range(NUM_DRAFTS):
        rng = random.Random(1000 + i)
        target_arch = i % NUM_ARCHETYPES  # Ensure equal distribution
        rec = run_single_draft(pool_a, target_arch, cfg_a["K"], cfg_a["C"], rng,
                               strategy="committed")

        # Convergence for this archetype
        conv = NUM_PICKS
        for pi in range(2, NUM_PICKS):
            window = rec.sa_per_pack[max(0, pi - 2):pi + 1]
            if sum(window) / len(window) >= 2.0:
                conv = pi + 1
                break
        per_arch_conv[target_arch].append(conv)

        # Late S/A
        late_sa = statistics.mean(rec.sa_per_pack[5:])
        per_arch_late_sa[target_arch].append(late_sa)

    print(f"\n  {'Archetype':<25} {'Conv Pick':>10} {'Late S/A':>10} {'N':>5}")
    print("  " + "-" * 55)
    for ai in range(NUM_ARCHETYPES):
        name = ARCHETYPE_DEFS[ai][0]
        conv = statistics.mean(per_arch_conv[ai]) if per_arch_conv[ai] else 30
        lsa = statistics.mean(per_arch_late_sa[ai]) if per_arch_late_sa[ai] else 0
        n = len(per_arch_conv[ai])
        print(f"  {name:<25} {conv:>10.1f} {lsa:>10.2f} {n:>5}")

    all_convs = [c for vals in per_arch_conv.values() for c in vals]
    all_lsa = [l for vals in per_arch_late_sa.values() for l in vals]
    print(f"  {'AVERAGE':<25} {statistics.mean(all_convs):>10.1f} "
          f"{statistics.mean(all_lsa):>10.2f} {len(all_convs):>5}")

    conv_range = max(statistics.mean(per_arch_conv[ai]) for ai in range(NUM_ARCHETYPES)) - \
                 min(statistics.mean(per_arch_conv[ai]) for ai in range(NUM_ARCHETYPES))
    print(f"\n  Convergence range: {conv_range:.1f} picks (target: < 3.0)")

    # ======================================================================
    # SECTION 8: Before/After Summary
    # ======================================================================
    print("\n\n" + "=" * 100)
    print("  SECTION 8: BEFORE/AFTER COMPARISON (Committed Strategy)")
    print("  Baseline (V5 default) vs Config A (Reconciled Best)")
    print("=" * 100)

    baseline = all_results[("Baseline", "committed")]
    reconciled = all_results[("Config A (Reconciled)", "committed")]

    metrics_compare = [
        ("Late S/A",           "late_sa",     "{:.2f}"),
        ("Late Off-Arch C/F",  "late_cf",     "{:.2f}"),
        ("Convergence Pick",   "conv_pick",   "{:.1f}"),
        ("Deck Concentration", "deck_conc",   "{:.1%}"),
        ("S/A StdDev",         "stddev",      "{:.2f}"),
        ("Early Diversity",    "early_unique", "{:.1f}"),
        ("Early S/A",          "early_sa",    "{:.2f}"),
        ("Run-to-Run Overlap", "overlap",     "{:.1%}"),
    ]

    print(f"\n  {'Metric':<25} {'Baseline':>12} {'Reconciled':>12} {'Delta':>10}")
    print("  " + "-" * 62)
    for name, key, fmt in metrics_compare:
        bv = baseline[key]
        rv = reconciled[key]
        delta = rv - bv
        bvs = fmt.format(bv)
        rvs = fmt.format(rv)
        if "%" in fmt:
            ds = f"{delta:+.1%}"
        else:
            ds = f"{delta:+.2f}"
        print(f"  {name:<25} {bvs:>12} {rvs:>12} {ds:>10}")

    print(f"\n  Baseline passes: {baseline['passes']}/8")
    print(f"  Reconciled passes: {reconciled['passes']}/8")

    # ======================================================================
    # SECTION 9: Probability Escalation Curve Detail
    # ======================================================================
    print("\n\n" + "=" * 100)
    print("  SECTION 9: PROBABILITY ESCALATION CURVES (Config A, Committed)")
    print("  Expected pair-matched slots at each pick")
    print("=" * 100)

    # Re-run for detailed per-pick tracking
    prob_by_pick = defaultdict(list)
    sa_by_pick = defaultdict(list)

    for i in range(NUM_DRAFTS):
        rng = random.Random(1000 + i)
        target_arch = rng.randint(0, NUM_ARCHETYPES - 1)
        rec = run_single_draft(pool_a, target_arch, cfg_a["K"], cfg_a["C"], rng,
                               strategy="committed")
        for pn in range(NUM_PICKS):
            prob_by_pick[pn].append(rec.prob_per_pack[pn])
            sa_by_pick[pn].append(rec.sa_per_pack[pn])

    print(f"\n  {'Pick':>5} {'Avg P':>8} {'Tgt Slots':>10} {'Avg S/A':>9} {'Phase'}")
    print("  " + "-" * 50)
    for pn in range(NUM_PICKS):
        avg_p = statistics.mean(prob_by_pick[pn])
        avg_sa = statistics.mean(sa_by_pick[pn])
        tgt = avg_p * PACK_SIZE
        phase = "Explore" if pn < 5 else ("Commit" if pn < 15 else "Refine")
        print(f"  {pn+1:>5} {avg_p:>8.3f} {tgt:>10.2f} {avg_sa:>9.2f} {phase}")

    print("\n\nSynthesis simulation complete.")


if __name__ == "__main__":
    main()
