#!/usr/bin/env python3
"""
Echo Window Draft Simulation — Agent 4 (Reactive/Immediate Domain)
CORRECTED: All metrics measured at ARCHETYPE level, not resonance level.

Algorithm: "Count the resonance symbols across your last 3 picks (primary
symbols count as 2, others as 1); your top resonance fills 2 pack slots,
your second resonance fills 1, and the last slot is random."

Critical distinction: a resonance (e.g., Tide) is shared by multiple
archetypes (Warriors=Tide/Zephyr, Sacrifice=Tide/Stone). Metrics must
measure archetype-level fitness (S/A tier for the player's specific target
archetype), not resonance matching.
"""

import random
import statistics
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ──────────────────────────────────────────────────────────────────────────
# Core types
# ──────────────────────────────────────────────────────────────────────────

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

TIER_POWER = {Tier.S: 10, Tier.A: 8, Tier.B: 6, Tier.C: 3, Tier.F: 1}

# Archetypes on the circle (index, name, primary resonance, secondary resonance)
ARCHETYPES = [
    ("Flash",        Resonance.ZEPHYR, Resonance.EMBER),    # 0
    ("Blink",        Resonance.EMBER,  Resonance.ZEPHYR),   # 1
    ("Storm",        Resonance.EMBER,  Resonance.STONE),    # 2
    ("SelfDiscard",  Resonance.STONE,  Resonance.EMBER),    # 3
    ("SelfMill",     Resonance.STONE,  Resonance.TIDE),     # 4
    ("Sacrifice",    Resonance.TIDE,   Resonance.STONE),    # 5
    ("Warriors",     Resonance.TIDE,   Resonance.ZEPHYR),   # 6
    ("Ramp",         Resonance.ZEPHYR, Resonance.TIDE),     # 7
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
ARCHETYPE_INDEX = {a[0]: i for i, a in enumerate(ARCHETYPES)}

def adjacent_indices(idx):
    """Return indices of archetypes adjacent on the circle."""
    return [(idx - 1) % 8, (idx + 1) % 8]


@dataclass
class SimCard:
    id: int
    symbols: list  # list[Resonance], ordered, 0-3 elements
    archetype: str  # primary archetype name
    archetype_fitness: dict = field(default_factory=dict)  # archetype_name -> Tier
    power: float = 5.0

    @property
    def primary_resonance(self) -> Optional[Resonance]:
        return self.symbols[0] if self.symbols else None

    def sa_tier_for(self, arch_name):
        """Return True if this card is S or A tier for the given archetype."""
        t = self.archetype_fitness.get(arch_name, Tier.F)
        return t in (Tier.S, Tier.A)

    def cf_tier_for(self, arch_name):
        """Return True if this card is C or F tier for the given archetype."""
        t = self.archetype_fitness.get(arch_name, Tier.F)
        return t in (Tier.C, Tier.F)


# ──────────────────────────────────────────────────────────────────────────
# Card pool construction
# ──────────────────────────────────────────────────────────────────────────

def compute_fitness(arch_idx, card_symbols):
    """
    Compute archetype fitness for a card belonging to archetype at arch_idx.
    Uses the circle adjacency model:
    - S-tier: home archetype
    - A-tier: adjacent archetype sharing primary resonance
    - B-tier: archetypes sharing secondary resonance
    - C/F-tier: distant archetypes
    - Generic cards: B-tier in all
    """
    home_name, home_pri, home_sec = ARCHETYPES[arch_idx]
    fitness = {}

    for other_idx, (other_name, other_pri, other_sec) in enumerate(ARCHETYPES):
        if other_idx == arch_idx:
            fitness[other_name] = Tier.S
        elif other_idx in adjacent_indices(arch_idx) and other_pri == home_pri:
            # Adjacent archetype sharing PRIMARY resonance -> A-tier
            fitness[other_name] = Tier.A
        elif other_pri == home_sec or other_sec == home_pri or other_sec == home_sec:
            # Shares secondary resonance in some direction -> B-tier
            fitness[other_name] = Tier.B
        else:
            # Check for any resonance overlap at all
            card_res = set(card_symbols)
            other_res = {other_pri, other_sec}
            if card_res & other_res:
                fitness[other_name] = Tier.C
            else:
                fitness[other_name] = Tier.F

    return fitness


def build_card_pool(pct_1sym=0.20, pct_2sym=0.60, pct_3sym=0.20, num_generic=36):
    """Build 360 cards: 8 archetypes x ~40 + 36 generic."""
    cards = []
    card_id = 0
    num_archetype_cards = 360 - num_generic
    per_archetype = num_archetype_cards // 8  # 40 each

    for arch_idx, (arch_name, primary, secondary) in enumerate(ARCHETYPES):
        n1 = round(per_archetype * pct_1sym)
        n3 = round(per_archetype * pct_3sym)
        n2 = per_archetype - n1 - n3

        for i in range(per_archetype):
            if i < n1:
                symbols = [primary]
            elif i < n1 + n2:
                symbols = [primary, secondary]
            else:
                symbols = [primary, primary, secondary]

            fitness = compute_fitness(arch_idx, symbols)
            power = random.uniform(3.0, 9.0)
            cards.append(SimCard(
                id=card_id,
                symbols=symbols,
                archetype=arch_name,
                archetype_fitness=fitness,
                power=power,
            ))
            card_id += 1

    # Generic cards: B-tier in all archetypes
    for _ in range(num_generic):
        fitness = {name: Tier.B for name in ARCHETYPE_NAMES}
        cards.append(SimCard(
            id=card_id,
            symbols=[],
            archetype="Generic",
            archetype_fitness=fitness,
            power=random.uniform(4.0, 8.0),
        ))
        card_id += 1

    return cards


# ──────────────────────────────────────────────────────────────────────────
# Echo Window Algorithm (UNCHANGED — operates on resonance symbols)
# ──────────────────────────────────────────────────────────────────────────

def count_window_symbols(recent_picks, primary_weight=2):
    """
    Count resonance symbols across recent picks.
    Primary symbol counts as primary_weight, secondary/tertiary count as 1.
    """
    counts = Counter()
    for card in recent_picks:
        for i, sym in enumerate(card.symbols):
            if i == 0:
                counts[sym] += primary_weight
            else:
                counts[sym] += 1
    return counts


def build_pack_echo_window(pool, recent_picks, window_size=3,
                           primary_weight=2, slot_alloc="2/1/1"):
    """
    Build a 4-card pack using the Echo Window algorithm.
    The algorithm uses ONLY visible card properties (resonance symbols).
    """
    window = recent_picks[-window_size:] if len(recent_picks) >= window_size else recent_picks

    num_picks = len(recent_picks)
    if num_picks == 0:
        return random.sample(pool, min(4, len(pool)))

    counts = count_window_symbols(window, primary_weight)
    ranked = sorted(counts.keys(), key=lambda r: (-counts[r], random.random()))

    top_res = ranked[0] if len(ranked) >= 1 else None
    second_res = ranked[1] if len(ranked) >= 2 else None

    # Determine slot allocation based on number of picks
    if num_picks == 1:
        top_slots, second_slots, random_slots = 1, 0, 3
    elif num_picks == 2:
        top_slots = 2
        second_slots = 1 if second_res and counts[second_res] > 0 else 0
        random_slots = 4 - top_slots - second_slots
    else:
        if slot_alloc == "2/1/1":
            top_slots = 2
            second_slots = 1 if second_res and counts[second_res] > 0 else 0
            random_slots = 4 - top_slots - second_slots
        elif slot_alloc == "3/1/0/0":
            top_slots = 3
            second_slots = 1 if second_res and counts[second_res] > 0 else 0
            random_slots = 4 - top_slots - second_slots
        elif slot_alloc == "2/1/0+1":
            top_slots, second_slots, random_slots = 2, 0, 2
        else:
            top_slots, second_slots, random_slots = 2, 1, 1

    def cards_with_resonance(res):
        return [c for c in pool if res in c.symbols]

    pack = []
    used_ids = set()

    def pick_from(candidate_list):
        available = [c for c in candidate_list if c.id not in used_ids]
        if not available:
            available = [c for c in pool if c.id not in used_ids]
        if not available:
            return None
        chosen = random.choice(available)
        used_ids.add(chosen.id)
        return chosen

    if top_res:
        top_cards = cards_with_resonance(top_res)
        for _ in range(top_slots):
            c = pick_from(top_cards)
            if c:
                pack.append(c)

    if second_res and second_slots > 0:
        second_cards = cards_with_resonance(second_res)
        for _ in range(second_slots):
            c = pick_from(second_cards)
            if c:
                pack.append(c)

    for _ in range(random_slots):
        available = [c for c in pool if c.id not in used_ids]
        if available:
            c = random.choice(available)
            used_ids.add(c.id)
            pack.append(c)

    while len(pack) < 4:
        available = [c for c in pool if c.id not in used_ids]
        if not available:
            break
        c = random.choice(available)
        used_ids.add(c.id)
        pack.append(c)

    return pack[:4]


# ──────────────────────────────────────────────────────────────────────────
# Player strategies
# ──────────────────────────────────────────────────────────────────────────

def archetype_committed_pick(pack, drafted, pick_num):
    """Picks highest fitness in strongest archetype, commits ~pick 5-6."""
    if pick_num < 5 or not drafted:
        def best_score(card):
            return max(TIER_POWER[t] for t in card.archetype_fitness.values())
        return max(pack, key=best_score)

    arch_scores = Counter()
    for card in drafted:
        for arch, tier in card.archetype_fitness.items():
            arch_scores[arch] += TIER_POWER[tier]
    committed_arch = arch_scores.most_common(1)[0][0]

    def fitness_in_arch(card):
        return TIER_POWER.get(card.archetype_fitness.get(committed_arch, Tier.F), 1)
    return max(pack, key=lambda c: (fitness_in_arch(c), c.power))


def power_chaser_pick(pack, drafted, pick_num):
    """Picks highest raw power regardless of archetype."""
    return max(pack, key=lambda c: c.power)


def signal_reader_pick(pack, drafted, pick_num):
    """Evaluates which resonance is most available, drafts toward open archetype."""
    if pick_num < 3:
        return max(pack, key=lambda c: c.power)

    res_counts = Counter()
    for card in pack:
        for sym in card.symbols:
            res_counts[sym] += 1

    own_counts = Counter()
    for card in drafted:
        for i, sym in enumerate(card.symbols):
            if i == 0:
                own_counts[sym] += 2
            else:
                own_counts[sym] += 1

    def signal_score(card):
        if not card.symbols:
            return card.power
        primary = card.symbols[0]
        pack_signal = res_counts.get(primary, 0) * 2
        own_signal = own_counts.get(primary, 0)
        return pack_signal + own_signal + card.power * 0.3

    return max(pack, key=signal_score)


# ──────────────────────────────────────────────────────────────────────────
# Archetype-level metric helpers
# ──────────────────────────────────────────────────────────────────────────

def determine_committed_archetype(drafted):
    """Determine committed archetype from drafted cards by accumulated fitness."""
    arch_scores = Counter()
    for card in drafted:
        for arch, tier in card.archetype_fitness.items():
            arch_scores[arch] += TIER_POWER[tier]
    return arch_scores.most_common(1)[0][0]


def count_sa_cards_in_pack(pack, arch_name):
    """Count cards with S or A tier fitness for given archetype in a pack."""
    return sum(1 for c in pack if c.sa_tier_for(arch_name))


def count_cf_cards_in_pack(pack, arch_name):
    """Count cards with C or F tier fitness for given archetype in a pack."""
    return sum(1 for c in pack if c.cf_tier_for(arch_name))


def count_unique_archetypes_with_sa(pack):
    """Count how many distinct archetypes have at least one S/A card in pack."""
    archs_with_sa = set()
    for card in pack:
        for arch_name in ARCHETYPE_NAMES:
            if card.sa_tier_for(arch_name):
                archs_with_sa.add(arch_name)
    return len(archs_with_sa)


# ──────────────────────────────────────────────────────────────────────────
# Simulation engine
# ──────────────────────────────────────────────────────────────────────────

def run_single_draft(pool, strategy_fn, window_size=3, primary_weight=2,
                     slot_alloc="2/1/1", trace=False):
    """Run a single 30-pick draft. Returns drafted cards and trace data."""
    drafted = []
    trace_data = []

    for pick_num in range(30):
        pack = build_pack_echo_window(
            pool, drafted, window_size=window_size,
            primary_weight=primary_weight, slot_alloc=slot_alloc
        )
        choice = strategy_fn(pack, drafted, pick_num)
        drafted.append(choice)

        if trace:
            window = drafted[-window_size:] if len(drafted) >= window_size else drafted
            counts = count_window_symbols(window, primary_weight)
            ranked = sorted(counts.keys(), key=lambda r: (-counts[r], random.random()))

            # Archetype-level pack analysis
            current_arch = determine_committed_archetype(drafted) if len(drafted) >= 3 else "N/A"
            sa_in_pack = count_sa_cards_in_pack(pack, current_arch) if current_arch != "N/A" else 0
            unique_archs = count_unique_archetypes_with_sa(pack)

            trace_data.append({
                "pick": pick_num + 1,
                "pack_symbols": [c.symbols for c in pack],
                "pack_archetypes": [c.archetype for c in pack],
                "pack_fitness": [
                    c.archetype_fitness.get(current_arch, Tier.F).value if current_arch != "N/A" else "?"
                    for c in pack
                ],
                "chosen_symbols": choice.symbols,
                "chosen_archetype": choice.archetype,
                "chosen_fitness": choice.archetype_fitness.get(current_arch, Tier.F).value if current_arch != "N/A" else "?",
                "window_counts": dict(counts),
                "top_resonance": ranked[0].value if ranked else "None",
                "committed_arch": current_arch,
                "sa_in_pack": sa_in_pack,
                "unique_archs_sa": unique_archs,
            })

    return drafted, trace_data


def compute_pack_metrics(pool, strategy_fn, n_runs=1000, window_size=3,
                         primary_weight=2, slot_alloc="2/1/1"):
    """
    Run n_runs drafts and compute all metrics at the ARCHETYPE level.

    Key distinction from old version:
    - "fitting" = S or A tier fitness for player's committed archetype
    - "off-archetype" = C or F tier fitness
    - "unique archetypes" = distinct archetypes with at least one S/A card in pack
    """
    all_early_unique_archs = []    # picks 1-5: unique archetypes with S/A cards per pack
    all_early_arch_fit = []        # picks 1-5: S/A cards for emerging archetype per pack
    all_late_arch_fit = []         # picks 6+: S/A cards for committed archetype per pack
    all_late_cf = []               # picks 6+: C/F-tier cards per pack
    convergence_picks = []         # pick where player first sees 2+ S/A arch cards
    all_sa_concentrations = []     # deck S/A concentration
    all_drafted_ids = []           # for overlap calculation
    all_committed_archs = []       # for archetype frequency

    for run in range(n_runs):
        drafted = []
        run_early_unique = []
        run_early_fit = []
        run_late_fit = []
        run_late_cf = []
        convergence_pick = None

        for pick_num in range(30):
            pack = build_pack_echo_window(
                pool, drafted, window_size=window_size,
                primary_weight=primary_weight, slot_alloc=slot_alloc
            )
            choice = strategy_fn(pack, drafted, pick_num)
            drafted.append(choice)

            # Determine current committed archetype
            current_arch = determine_committed_archetype(drafted)

            # ARCHETYPE-LEVEL metrics
            sa_count = count_sa_cards_in_pack(pack, current_arch)
            cf_count = count_cf_cards_in_pack(pack, current_arch)
            unique_archs = count_unique_archetypes_with_sa(pack)

            if pick_num < 5:
                run_early_unique.append(unique_archs)
                run_early_fit.append(sa_count)
            else:
                run_late_fit.append(sa_count)
                run_late_cf.append(cf_count)

                if convergence_pick is None and sa_count >= 2:
                    convergence_pick = pick_num + 1

        all_early_unique_archs.append(
            statistics.mean(run_early_unique) if run_early_unique else 0)
        all_early_arch_fit.append(
            statistics.mean(run_early_fit) if run_early_fit else 0)
        all_late_arch_fit.append(
            statistics.mean(run_late_fit) if run_late_fit else 0)
        all_late_cf.append(
            statistics.mean(run_late_cf) if run_late_cf else 0)
        convergence_picks.append(convergence_pick if convergence_pick else 30)

        # Final deck S/A concentration
        committed = determine_committed_archetype(drafted)
        all_committed_archs.append(committed)
        sa = sum(1 for c in drafted if c.sa_tier_for(committed))
        all_sa_concentrations.append(sa / 30.0)

        all_drafted_ids.append(set(c.id for c in drafted))

    # Run-to-run overlap
    overlaps = []
    sample_pairs = min(500, n_runs * (n_runs - 1) // 2)
    for _ in range(sample_pairs):
        i, j = random.sample(range(n_runs), 2)
        overlap = len(all_drafted_ids[i] & all_drafted_ids[j]) / 30.0
        overlaps.append(overlap)

    # Archetype frequency
    arch_freq = Counter(all_committed_archs)
    arch_pcts = {k: v / n_runs for k, v in arch_freq.items()}

    results = {
        "early_unique_archs": statistics.mean(all_early_unique_archs),
        "early_arch_fit": statistics.mean(all_early_arch_fit),
        "late_arch_fit": statistics.mean(all_late_arch_fit),
        "late_cf": statistics.mean(all_late_cf),
        "convergence_pick": statistics.mean(convergence_picks),
        "sa_concentration": statistics.mean(all_sa_concentrations),
        "overlap": statistics.mean(overlaps) if overlaps else 0,
        "arch_freq": arch_pcts,
        "arch_max": max(arch_pcts.values()),
        "arch_min": min(arch_pcts.values()) if arch_pcts else 0,
    }
    return results


# ──────────────────────────────────────────────────────────────────────────
# Output formatting
# ──────────────────────────────────────────────────────────────────────────

def print_separator():
    print("=" * 72)


def print_results_table(label, results):
    print(f"\n--- {label} ---")
    targets = [
        ("Picks 1-5: unique archetypes w/ S/A per pack",
         results["early_unique_archs"], ">= 3.0",
         results["early_unique_archs"] >= 3.0),
        ("Picks 1-5: S/A cards for emerging arch/pack",
         results["early_arch_fit"], "<= 2.0",
         results["early_arch_fit"] <= 2.0),
        ("Picks 6+: S/A cards for committed arch/pack",
         results["late_arch_fit"], ">= 2.0",
         results["late_arch_fit"] >= 2.0),
        ("Picks 6+: C/F-tier cards per pack",
         results["late_cf"], ">= 0.5",
         results["late_cf"] >= 0.5),
        ("Convergence pick",
         results["convergence_pick"], "5-8",
         5 <= results["convergence_pick"] <= 8),
        ("S/A concentration",
         results["sa_concentration"], "60-80%",
         0.60 <= results["sa_concentration"] <= 0.80),
        ("Run-to-run overlap",
         results["overlap"], "< 40%",
         results["overlap"] < 0.40),
        ("Arch freq max",
         results["arch_max"], "<= 20%",
         results["arch_max"] <= 0.20),
        ("Arch freq min",
         results["arch_min"], ">= 5%",
         results["arch_min"] >= 0.05),
    ]
    print(f"  {'Metric':<48} {'Actual':>8} {'Target':>10} {'Pass':>6}")
    print(f"  {'-'*48} {'-'*8} {'-'*10} {'-'*6}")
    for name, actual, target, passed in targets:
        fmt = f"{actual:.2f}" if isinstance(actual, float) else str(actual)
        status = "PASS" if passed else "FAIL"
        print(f"  {name:<48} {fmt:>8} {target:>10} {status:>6}")


def print_draft_trace(trace_data, label):
    print(f"\n{'='*72}")
    print(f"DRAFT TRACE: {label}")
    print(f"{'='*72}")
    for t in trace_data:
        pack_syms = [
            "/".join(s.value[0] for s in syms) if syms else "Gen"
            for syms in t["pack_symbols"]
        ]
        chosen_sym = "/".join(s.value[0] for s in t["chosen_symbols"]) if t["chosen_symbols"] else "Gen"
        wc = {k.value: v for k, v in t["window_counts"].items()} if t["window_counts"] else {}
        fitness_str = "/".join(t["pack_fitness"])
        print(f"  Pick {t['pick']:2d}: Pack=[{', '.join(pack_syms):>20s}] Fit=[{fitness_str:>8s}]  "
              f"Chose={chosen_sym:<6s} ({t['chosen_archetype']:<12s} {t['chosen_fitness']})  "
              f"Arch={t['committed_arch']:<12s} SA={t['sa_in_pack']} UniArch={t['unique_archs_sa']}  "
              f"Window={wc}  Top={t['top_resonance']}")


def main():
    random.seed(42)
    pool = build_card_pool(pct_1sym=0.20, pct_2sym=0.60, pct_3sym=0.20)

    print_separator()
    print("ECHO WINDOW DRAFT SIMULATION — Agent 4 (CORRECTED)")
    print("All metrics now measured at ARCHETYPE level, not resonance level.")
    print()
    print("Algorithm: Count resonance symbols across last 3 picks")
    print("(primary=2, others=1); top resonance fills 2 slots,")
    print("second fills 1, last slot is random.")
    print()
    print("Key: S/A-tier = cards from player's archetype or adjacent")
    print("     C/F-tier = cards from distant archetypes")
    print_separator()

    # ── Main results for all 3 strategies ────────────────────────────────
    strategies = [
        ("Archetype-Committed", archetype_committed_pick),
        ("Power-Chaser", power_chaser_pick),
        ("Signal-Reader", signal_reader_pick),
    ]

    print("\n" + "=" * 72)
    print("MAIN RESULTS (window=3, primary_weight=2, alloc=2/1/1, 1000 runs)")
    print("=" * 72)

    for name, fn in strategies:
        results = compute_pack_metrics(pool, fn, n_runs=1000,
                                       window_size=3, primary_weight=2,
                                       slot_alloc="2/1/1")
        print_results_table(name, results)
        print(f"  Archetype frequencies: ", end="")
        for arch, pct in sorted(results["arch_freq"].items()):
            print(f"{arch}={pct:.1%} ", end="")
        print()

    # ── Parameter sensitivity: window size ───────────────────────────────
    print("\n" + "=" * 72)
    print("PARAMETER SENSITIVITY: Window Size (Archetype-Committed, 500 runs)")
    print("=" * 72)

    for ws in [3, 4, 5]:
        results = compute_pack_metrics(pool, archetype_committed_pick, n_runs=500,
                                       window_size=ws, primary_weight=2,
                                       slot_alloc="2/1/1")
        print(f"\n  Window={ws}: late_SA={results['late_arch_fit']:.2f}, "
              f"late_CF={results['late_cf']:.2f}, "
              f"convergence={results['convergence_pick']:.1f}, "
              f"early_uni_arch={results['early_unique_archs']:.2f}, "
              f"sa_conc={results['sa_concentration']:.2f}, "
              f"overlap={results['overlap']:.2f}")

    # ── Parameter sensitivity: slot allocation ───────────────────────────
    print("\n" + "=" * 72)
    print("PARAMETER SENSITIVITY: Slot Allocation (Archetype-Committed, 500 runs)")
    print("=" * 72)

    for alloc in ["2/1/1", "3/1/0/0", "2/1/0+1"]:
        results = compute_pack_metrics(pool, archetype_committed_pick, n_runs=500,
                                       window_size=3, primary_weight=2,
                                       slot_alloc=alloc)
        print(f"\n  Alloc={alloc}: late_SA={results['late_arch_fit']:.2f}, "
              f"late_CF={results['late_cf']:.2f}, "
              f"convergence={results['convergence_pick']:.1f}, "
              f"sa_conc={results['sa_concentration']:.2f}, "
              f"overlap={results['overlap']:.2f}")

    # ── Parameter sensitivity: primary weight ────────────────────────────
    print("\n" + "=" * 72)
    print("PARAMETER SENSITIVITY: Primary Weight (Archetype-Committed, 500 runs)")
    print("=" * 72)

    for pw in [2, 3]:
        results = compute_pack_metrics(pool, archetype_committed_pick, n_runs=500,
                                       window_size=3, primary_weight=pw,
                                       slot_alloc="2/1/1")
        print(f"\n  Primary_weight={pw}: late_SA={results['late_arch_fit']:.2f}, "
              f"late_CF={results['late_cf']:.2f}, "
              f"convergence={results['convergence_pick']:.1f}, "
              f"sa_conc={results['sa_concentration']:.2f}")

    # ── Draft traces ─────────────────────────────────────────────────────
    print("\n" + "=" * 72)
    print("DRAFT TRACES")
    print("=" * 72)

    # Trace 1: Early committer
    random.seed(100)
    drafted1, trace1 = run_single_draft(pool, archetype_committed_pick,
                                         window_size=3, primary_weight=2,
                                         slot_alloc="2/1/1", trace=True)
    print_draft_trace(trace1, "Early Committer (Archetype-Committed)")
    committed1 = determine_committed_archetype(drafted1)
    sa1 = sum(1 for c in drafted1 if c.sa_tier_for(committed1))
    print(f"\n  Final: committed={committed1}, S/A cards={sa1}/30 ({sa1/30:.0%})")

    # Trace 2: Flexible player
    random.seed(200)
    drafted2, trace2 = run_single_draft(pool, power_chaser_pick,
                                         window_size=3, primary_weight=2,
                                         slot_alloc="2/1/1", trace=True)
    print_draft_trace(trace2, "Flexible Player (Power-Chaser)")
    committed2 = determine_committed_archetype(drafted2)
    sa2 = sum(1 for c in drafted2 if c.sa_tier_for(committed2))
    print(f"\n  Final: committed={committed2}, S/A cards={sa2}/30 ({sa2/30:.0%})")

    # Trace 3: Signal reader
    random.seed(300)
    drafted3, trace3 = run_single_draft(pool, signal_reader_pick,
                                         window_size=3, primary_weight=2,
                                         slot_alloc="2/1/1", trace=True)
    print_draft_trace(trace3, "Signal Reader")
    committed3 = determine_committed_archetype(drafted3)
    sa3 = sum(1 for c in drafted3 if c.sa_tier_for(committed3))
    print(f"\n  Final: committed={committed3}, S/A cards={sa3}/30 ({sa3/30:.0%})")

    print("\n" + "=" * 72)
    print("SIMULATION COMPLETE")
    print("=" * 72)


if __name__ == "__main__":
    main()
