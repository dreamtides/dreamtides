"""
V7 Agent 4: Aspiration Packs + Biased Random Simulation
========================================================

Algorithm: "After each pick, compute top resonance pair (R1, R2); if R2 >= 3
tokens AND R2 >= 50% of R1, one slot shows an R1 card, one shows an R2 card,
two slots draw from pool weighted 2x toward R1; otherwise all four
weighted-random (2x toward R1 once any counter >= 2)."

Fitness models:
  A (Optimistic): Cross-archetype = 100% A-tier
  B (Moderate):   Cross-archetype = 50%A/30%B/20%C  -> 75% S/A
  C (Pessimistic): Cross-archetype = 25%A/40%B/35%C -> 62.5% S/A

1000 drafts x 30 picks x 3 strategies, all 9 metrics at archetype level.
Parameter sensitivity on bias weight (1.5x, 2.0x, 3.0x) and gate thresholds.
"""

import random
import statistics
import math
from dataclasses import dataclass, field
from collections import Counter, defaultdict
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 4
NUM_ARCHETYPES = 8
NUM_GENERIC = 36
TOTAL_CARDS = 360
DUAL_CAP = 54

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

TIER_SCORE = {"S": 4.0, "A": 3.0, "B": 2.0, "C": 0.5, "F": 0.0}


# ---------------------------------------------------------------------------
# Fitness model helpers
# ---------------------------------------------------------------------------

def build_base_fitness_map():
    """Build the structural relationship map (home, sibling, secondary, distant)."""
    fm = {}
    for home_idx, (_, home_pri, home_sec) in enumerate(ARCHETYPES):
        fm[home_idx] = {}
        for eval_idx, (_, eval_pri, eval_sec) in enumerate(ARCHETYPES):
            if eval_idx == home_idx:
                fm[home_idx][eval_idx] = "home"
            elif eval_pri == home_pri:
                fm[home_idx][eval_idx] = "sibling"
            elif eval_sec == home_pri or eval_pri == home_sec:
                fm[home_idx][eval_idx] = "secondary"
            else:
                fm[home_idx][eval_idx] = "distant"
    return fm

BASE_FITNESS_MAP = build_base_fitness_map()


def assign_card_fitness(card_arch_idx, fitness_model, rng):
    """Assign per-card fitness tiers based on the fitness model.
    Returns dict: eval_archetype_idx -> tier string.
    Sibling tiers are rolled per card during pool construction."""
    fitness = {}
    for eval_idx in range(NUM_ARCHETYPES):
        relationship = BASE_FITNESS_MAP[card_arch_idx][eval_idx]
        if relationship == "home":
            fitness[eval_idx] = "S"
        elif relationship == "sibling":
            if fitness_model == "A":
                fitness[eval_idx] = "A"
            elif fitness_model == "B":
                r = rng.random()
                if r < 0.50:
                    fitness[eval_idx] = "A"
                elif r < 0.80:
                    fitness[eval_idx] = "B"
                else:
                    fitness[eval_idx] = "C"
            elif fitness_model == "C":
                r = rng.random()
                if r < 0.25:
                    fitness[eval_idx] = "A"
                elif r < 0.65:
                    fitness[eval_idx] = "B"
                else:
                    fitness[eval_idx] = "C"
        elif relationship == "secondary":
            fitness[eval_idx] = "B"
        else:
            fitness[eval_idx] = "C"
    return fitness


# ---------------------------------------------------------------------------
# SimCard
# ---------------------------------------------------------------------------

@dataclass
class SimCard:
    id: int
    symbols: list
    archetype_idx: int
    power: float
    fitness: dict = field(default_factory=dict)  # eval_arch_idx -> tier

    @property
    def primary_resonance(self) -> Optional[str]:
        return self.symbols[0] if self.symbols else None

    @property
    def resonance_types(self) -> set:
        return set(self.symbols)

    @property
    def is_generic(self) -> bool:
        return self.archetype_idx == -1

    def fitness_for(self, archetype_idx: int) -> str:
        if self.is_generic:
            return "B"
        return self.fitness.get(archetype_idx, "C")

    def fitness_score_for(self, archetype_idx: int) -> float:
        return TIER_SCORE[self.fitness_for(archetype_idx)]

    def is_sa_for(self, archetype_idx: int) -> bool:
        return self.fitness_for(archetype_idx) in ("S", "A")


# ---------------------------------------------------------------------------
# Card Pool Construction
# ---------------------------------------------------------------------------

def build_card_pool(rng, fitness_model="A"):
    """Build 360-card pool with configurable fitness model."""
    cards = []
    card_id = 0

    # Generic cards
    for _ in range(NUM_GENERIC):
        cards.append(SimCard(
            id=card_id, symbols=[], archetype_idx=-1,
            power=rng.uniform(3.0, 7.0),
        ))
        card_id += 1

    # Archetype cards: ~40 per archetype
    trimmed = rng.sample(range(NUM_ARCHETYPES), 2)
    boosted = set([i for i in range(NUM_ARCHETYPES) if i not in trimmed][:4])

    for arch_idx, (name, pri, sec) in enumerate(ARCHETYPES):
        is_trimmed = arch_idx in trimmed
        is_boosted = arch_idx in boosted
        n_dual_2 = 3 if is_trimmed else 4
        n_mono_1 = 11 if is_boosted else 10
        n_mono_2 = 18 if is_trimmed else 17

        symbol_configs = (
            [([pri], n) for n in range(n_mono_1)] +
            [([pri, pri], n) for n in range(n_mono_2)] +
            [([pri, pri, pri], n) for n in range(6)] +
            [([pri, sec], n) for n in range(n_dual_2)] +
            [([pri, pri, sec], n) for n in range(3)]
        )

        for symbols, _ in symbol_configs:
            power = rng.uniform(2.0, 9.0)
            card_fitness = assign_card_fitness(arch_idx, fitness_model, rng)
            cards.append(SimCard(
                id=card_id, symbols=list(symbols),
                archetype_idx=arch_idx, power=power,
                fitness=card_fitness,
            ))
            card_id += 1

    assert len(cards) == TOTAL_CARDS, f"Expected {TOTAL_CARDS}, got {len(cards)}"
    return cards


def build_resonance_pools(pool):
    """Index cards by primary resonance."""
    pools = {}
    for r in RESONANCES:
        pools[r] = [c for c in pool if c.primary_resonance == r]
    return pools


# ---------------------------------------------------------------------------
# Player Strategies
# ---------------------------------------------------------------------------

def _choose_card(pack, strategy, committed_archetype, pick_num,
                 commit_pick, resonance_signals, rng):
    if strategy == "committed":
        return _committed_choose(pack, committed_archetype, pick_num, rng)
    elif strategy == "power-chaser":
        return _power_chaser_choose(pack, rng)
    elif strategy == "signal-reader":
        return _signal_reader_choose(pack, committed_archetype, pick_num,
                                     resonance_signals, rng)
    raise ValueError(f"Unknown strategy: {strategy}")


def _committed_choose(pack, committed_archetype, pick_num, rng):
    if committed_archetype is None:
        symbol_cards = [c for c in pack if c.symbols]
        if symbol_cards:
            return max(symbol_cards, key=lambda c: c.power)
        return max(pack, key=lambda c: c.power)
    else:
        return max(pack, key=lambda c: (c.fitness_score_for(committed_archetype),
                                         c.power))


def _power_chaser_choose(pack, rng):
    return max(pack, key=lambda c: c.power)


def _signal_reader_choose(pack, committed_archetype, pick_num,
                           resonance_signals, rng):
    if committed_archetype is None:
        def score(c):
            if not c.symbols:
                return (0, c.power)
            pri_signal = resonance_signals.get(c.primary_resonance, 0)
            return (pri_signal, c.power)
        return max(pack, key=score)
    else:
        return max(pack, key=lambda c: (c.fitness_score_for(committed_archetype),
                                         c.power))


def _best_archetype_from_drafted(drafted):
    scores = [0.0] * NUM_ARCHETYPES
    for card in drafted:
        for a in range(NUM_ARCHETYPES):
            scores[a] += card.fitness_score_for(a)
    return scores.index(max(scores))


def _signal_read_commit(drafted, resonance_signals):
    sorted_res = sorted(RESONANCES, key=lambda r: resonance_signals[r],
                        reverse=True)
    top_pri = sorted_res[0]
    top_sec = sorted_res[1]
    for idx, (name, pri, sec) in enumerate(ARCHETYPES):
        if pri == top_pri and sec == top_sec:
            return idx
    for idx, (name, pri, sec) in enumerate(ARCHETYPES):
        if pri == top_pri:
            return idx
    return 0


def _get_top_resonance(res_counters):
    return max(RESONANCES, key=lambda r: res_counters[r])


def _get_top_two_resonances(res_counters):
    sorted_r = sorted(RESONANCES, key=lambda r: res_counters[r], reverse=True)
    return sorted_r[0], sorted_r[1]


def _update_resonance_counters(card, res_counters):
    """Add weighted symbol counts: +2 primary, +1 secondary/tertiary."""
    for i, sym in enumerate(card.symbols):
        weight = 2 if i == 0 else 1
        res_counters[sym] += weight


def _handle_commitment(strategy, committed_archetype, commit_pick, pick_num,
                       drafted, resonance_signals):
    if committed_archetype is None and pick_num >= 4:
        if strategy == "committed":
            committed_archetype = _best_archetype_from_drafted(drafted)
            commit_pick = pick_num
        elif strategy == "signal-reader":
            committed_archetype = _signal_read_commit(drafted, resonance_signals)
            commit_pick = pick_num
    return committed_archetype, commit_pick


# ---------------------------------------------------------------------------
# Biased random draw helper
# ---------------------------------------------------------------------------

def _biased_draw(pool, res_pools, top_resonance, bias_weight, rng):
    """Draw one card from pool with bias_weight multiplier on top_resonance cards.
    If top_resonance is None or bias_weight <= 1.0, draw uniformly."""
    if top_resonance is None or bias_weight <= 1.0:
        return rng.choice(pool)

    top_pool = res_pools.get(top_resonance, [])
    other_pool = [c for c in pool if c.primary_resonance != top_resonance]

    if not top_pool:
        return rng.choice(pool)

    # Weighted selection: top_resonance cards get bias_weight, others get 1.0
    total_top = len(top_pool) * bias_weight
    total_other = len(other_pool) * 1.0
    total = total_top + total_other

    if rng.random() < total_top / total:
        return rng.choice(top_pool)
    else:
        if other_pool:
            return rng.choice(other_pool)
        return rng.choice(pool)


# ---------------------------------------------------------------------------
# Algorithm: Aspiration Packs + Biased Random
# ---------------------------------------------------------------------------

def aspiration_biased_draft(pool, res_pools, rng, strategy,
                            bias_weight=2.0,
                            r2_min_tokens=3,
                            r2_min_ratio=0.50,
                            early_bias_threshold=2):
    """
    Aspiration Packs + Biased Random.

    After each pick, compute top resonance pair (R1, R2).
    If R2 >= r2_min_tokens AND R2 >= r2_min_ratio * R1:
        Slot 0: R1 card (from R1 pool)
        Slot 1: R2 card (from R2 pool)
        Slots 2-3: biased random (bias_weight toward R1)
    Otherwise:
        All 4 slots: biased random toward R1 (if any counter >= early_bias_threshold)
        or fully random if no counter has reached early_bias_threshold.
    """
    res_counters = {r: 0 for r in RESONANCES}
    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}

    for pick_num in range(NUM_PICKS):
        r1, r2 = _get_top_two_resonances(res_counters)

        # Determine if aspiration gate is open
        gate_open = (res_counters[r2] >= r2_min_tokens and
                     res_counters[r2] >= r2_min_ratio * res_counters[r1]
                     if res_counters[r1] > 0 else False)

        # Determine if early bias is active
        max_counter = max(res_counters[r] for r in RESONANCES)
        bias_active = max_counter >= early_bias_threshold

        pack = []
        if gate_open:
            # Aspiration mode: 1 R1 + 1 R2 + 2 biased-random toward R1
            pack.append(rng.choice(res_pools[r1]))
            pack.append(rng.choice(res_pools[r2]))
            for _ in range(PACK_SIZE - 2):
                pack.append(_biased_draw(pool, res_pools, r1, bias_weight, rng))
        elif bias_active:
            # Pre-aspiration biased mode: all 4 weighted toward R1
            for _ in range(PACK_SIZE):
                pack.append(_biased_draw(pool, res_pools, r1, bias_weight, rng))
        else:
            # Fully random early packs
            pack = [rng.choice(pool) for _ in range(PACK_SIZE)]

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              commit_pick, resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        _update_resonance_counters(chosen, res_counters)
        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals)

    return drafted, pack_records, committed_archetype


# ---------------------------------------------------------------------------
# Baseline: Surge Packs V6 (for comparison)
# ---------------------------------------------------------------------------

def surge_packs_draft(pool, res_pools, rng, strategy,
                      threshold=4, surge_slots=3):
    """Surge Packs V6 baseline: T=4, S=3."""
    token_counters = {r: 0 for r in RESONANCES}
    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}
    surge_resonance = None

    for pick_num in range(NUM_PICKS):
        pack = []
        if surge_resonance is not None:
            for slot_idx in range(PACK_SIZE):
                if slot_idx < surge_slots:
                    card = rng.choice(res_pools[surge_resonance])
                else:
                    card = rng.choice(pool)
                pack.append(card)
            surge_resonance = None
        else:
            pack = [rng.choice(pool) for _ in range(PACK_SIZE)]

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              commit_pick, resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        for i, sym in enumerate(chosen.symbols):
            weight = 2 if i == 0 else 1
            token_counters[sym] += weight
        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        max_res = max(RESONANCES, key=lambda r: token_counters[r])
        if token_counters[max_res] >= threshold:
            token_counters[max_res] -= threshold
            surge_resonance = max_res

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals)

    return drafted, pack_records, committed_archetype


# ---------------------------------------------------------------------------
# Baseline: Pure Aspiration Packs (Agent 7, no bias)
# ---------------------------------------------------------------------------

def pure_aspiration_draft(pool, res_pools, rng, strategy,
                          r2_min_tokens=3, r2_min_ratio=0.50):
    """Pure Aspiration Packs (Agent 7): 1 R1 + 1 R2 + 2 random, or all random."""
    res_counters = {r: 0 for r in RESONANCES}
    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}

    for pick_num in range(NUM_PICKS):
        r1, r2 = _get_top_two_resonances(res_counters)
        gate_open = (res_counters[r2] >= r2_min_tokens and
                     res_counters[r2] >= r2_min_ratio * res_counters[r1]
                     if res_counters[r1] > 0 else False)

        pack = []
        if gate_open:
            pack.append(rng.choice(res_pools[r1]))
            pack.append(rng.choice(res_pools[r2]))
            for _ in range(PACK_SIZE - 2):
                pack.append(rng.choice(pool))
        else:
            pack = [rng.choice(pool) for _ in range(PACK_SIZE)]

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              commit_pick, resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        _update_resonance_counters(chosen, res_counters)
        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals)

    return drafted, pack_records, committed_archetype


# ---------------------------------------------------------------------------
# Metrics Computation
# ---------------------------------------------------------------------------

def compute_metrics(all_results):
    """Compute all 9 required metrics from simulation results."""
    metrics = {}
    by_strategy = defaultdict(list)
    for strategy, drafted, pack_records, committed_arch in all_results:
        by_strategy[strategy].append((drafted, pack_records, committed_arch))

    # M1: Picks 1-5 unique archetypes with S/A cards per pack
    m1_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            for pick_num, pack, chosen in pack_records:
                if pick_num >= 5:
                    continue
                archs_with_sa = set()
                for card in pack:
                    for a in range(NUM_ARCHETYPES):
                        if card.is_sa_for(a):
                            archs_with_sa.add(a)
                m1_values.append(len(archs_with_sa))
    metrics["m1_unique_archetypes_early"] = (statistics.mean(m1_values)
                                              if m1_values else 0)

    # M2: Picks 1-5 S/A cards for emerging archetype per pack
    m2_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            if committed_arch is None:
                continue
            for pick_num, pack, chosen in pack_records:
                if pick_num >= 5:
                    continue
                sa_count = sum(1 for c in pack[:PACK_SIZE]
                               if c.is_sa_for(committed_arch))
                m2_values.append(sa_count)
    metrics["m2_sa_emerging_early"] = (statistics.mean(m2_values)
                                       if m2_values else 0)

    # M3: Picks 6+ S/A cards for committed archetype per pack
    m3_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            if committed_arch is None:
                continue
            for pick_num, pack, chosen in pack_records:
                if pick_num < 5:
                    continue
                sa_count = sum(1 for c in pack if c.is_sa_for(committed_arch))
                m3_values.append(sa_count)
    metrics["m3_sa_committed_late"] = (statistics.mean(m3_values)
                                       if m3_values else 0)

    # M4: Picks 6+ off-archetype (C/F) cards per pack
    m4_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            if committed_arch is None:
                continue
            for pick_num, pack, chosen in pack_records:
                if pick_num < 5:
                    continue
                cf_count = sum(1 for c in pack
                               if c.fitness_for(committed_arch) in ("C", "F"))
                m4_values.append(cf_count)
    metrics["m4_off_archetype_late"] = (statistics.mean(m4_values)
                                        if m4_values else 0)

    # M5: Convergence pick (first pick where 3 consecutive packs have >= 2 S/A)
    convergence_picks = []
    for strategy, results in by_strategy.items():
        if strategy == "power-chaser":
            continue
        for drafted, pack_records, committed_arch in results:
            if committed_arch is None:
                continue
            found = False
            for pick_num, pack, chosen in pack_records:
                if pick_num < 2:
                    continue
                if pick_num + 2 >= len(pack_records):
                    continue
                all_good = True
                for w in range(3):
                    _, wp, _ = pack_records[pick_num + w]
                    sa = sum(1 for c in wp if c.is_sa_for(committed_arch))
                    if sa < 2:
                        all_good = False
                        break
                if all_good:
                    convergence_picks.append(pick_num)
                    found = True
                    break
            if not found:
                convergence_picks.append(30)
    metrics["m5_convergence_pick"] = (statistics.mean(convergence_picks)
                                      if convergence_picks else 30)

    # M6: Deck archetype concentration
    m6_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            if committed_arch is None:
                continue
            sa_cards = sum(1 for c in drafted if c.is_sa_for(committed_arch))
            m6_values.append(sa_cards / len(drafted) * 100)
    metrics["m6_deck_concentration"] = (statistics.mean(m6_values)
                                        if m6_values else 0)

    # M7: Run-to-run card overlap
    archetype_runs = defaultdict(list)
    for strategy, drafted, pack_records, committed_arch in all_results:
        if committed_arch is not None:
            archetype_runs[committed_arch].append(set(c.id for c in drafted))

    overlaps = []
    overlap_rng = random.Random(99)
    for arch, run_sets in archetype_runs.items():
        if len(run_sets) < 2:
            continue
        n_pairs = min(200, len(run_sets) * (len(run_sets) - 1) // 2)
        for _ in range(n_pairs):
            i, j = overlap_rng.sample(range(len(run_sets)), 2)
            s1, s2 = run_sets[i], run_sets[j]
            if len(s1) == 0 or len(s2) == 0:
                continue
            overlap = len(s1 & s2) / min(len(s1), len(s2)) * 100
            overlaps.append(overlap)
    metrics["m7_card_overlap"] = statistics.mean(overlaps) if overlaps else 0

    # M8: Archetype frequency
    arch_counts = Counter()
    total_committed = 0
    for strategy, drafted, pack_records, committed_arch in all_results:
        if committed_arch is not None:
            arch_counts[committed_arch] += 1
            total_committed += 1

    arch_freqs = {}
    for a in range(NUM_ARCHETYPES):
        arch_freqs[ARCHETYPE_NAMES[a]] = (arch_counts[a] / total_committed * 100
                                           if total_committed > 0 else 0)
    metrics["m8_archetype_freq"] = arch_freqs
    metrics["m8_max_freq"] = max(arch_freqs.values()) if arch_freqs else 0
    metrics["m8_min_freq"] = min(arch_freqs.values()) if arch_freqs else 0

    # M9: StdDev of S/A cards per pack (picks 6+)
    metrics["m9_sa_stddev"] = (statistics.stdev(m3_values)
                                if len(m3_values) > 1 else 0)

    return metrics


def per_archetype_convergence(all_results):
    """Compute per-archetype average convergence pick."""
    arch_conv = defaultdict(list)
    for strategy, drafted, pack_records, committed_arch in all_results:
        if strategy == "power-chaser" or committed_arch is None:
            continue
        for pick_num, pack, chosen in pack_records:
            if pick_num < 2:
                continue
            if pick_num + 2 >= len(pack_records):
                continue
            all_good = True
            for w in range(3):
                _, wp, _ = pack_records[pick_num + w]
                sa = sum(1 for c in wp if c.is_sa_for(committed_arch))
                if sa < 2:
                    all_good = False
                    break
            if all_good:
                arch_conv[committed_arch].append(pick_num)
                break

    result = {}
    for a in range(NUM_ARCHETYPES):
        picks = arch_conv.get(a, [])
        result[ARCHETYPE_NAMES[a]] = (
            statistics.mean(picks) if picks else float('nan'),
            len(picks)
        )
    return result


def evaluate_pass_fail(metrics):
    results = {}
    results["M1"] = "PASS" if metrics["m1_unique_archetypes_early"] >= 3 else "FAIL"
    results["M2"] = "PASS" if metrics["m2_sa_emerging_early"] <= 2 else "FAIL"
    results["M3"] = "PASS" if metrics["m3_sa_committed_late"] >= 2.0 else "FAIL"
    results["M4"] = "PASS" if metrics["m4_off_archetype_late"] >= 0.5 else "FAIL"
    results["M5"] = "PASS" if 5 <= metrics["m5_convergence_pick"] <= 8 else "FAIL"
    results["M6"] = "PASS" if 60 <= metrics["m6_deck_concentration"] <= 90 else "FAIL"
    results["M7"] = "PASS" if metrics["m7_card_overlap"] < 40 else "FAIL"
    results["M8"] = "PASS" if (metrics["m8_min_freq"] >= 5 and
                                metrics["m8_max_freq"] <= 20) else "FAIL"
    results["M9"] = "PASS" if metrics["m9_sa_stddev"] >= 0.8 else "FAIL"
    return results


def print_metrics(metrics, name, pf):
    passed = sum(1 for v in pf.values() if v == "PASS")
    print(f"\n--- {name} ({passed}/9 PASS) ---")
    print(f"  M1 Unique archs early:    {metrics['m1_unique_archetypes_early']:.2f}  (>= 3)    [{pf['M1']}]")
    print(f"  M2 S/A early:             {metrics['m2_sa_emerging_early']:.2f}  (<= 2)    [{pf['M2']}]")
    print(f"  M3 S/A committed late:    {metrics['m3_sa_committed_late']:.2f}  (>= 2.0)  [{pf['M3']}]")
    print(f"  M4 Off-arch late:         {metrics['m4_off_archetype_late']:.2f}  (>= 0.5)  [{pf['M4']}]")
    print(f"  M5 Convergence pick:      {metrics['m5_convergence_pick']:.1f}  (5-8)     [{pf['M5']}]")
    print(f"  M6 Deck concentration:    {metrics['m6_deck_concentration']:.1f}%  (60-90%)  [{pf['M6']}]")
    print(f"  M7 Card overlap:          {metrics['m7_card_overlap']:.1f}%  (< 40%)   [{pf['M7']}]")
    print(f"  M8 Arch freq range:       {metrics['m8_min_freq']:.1f}%-{metrics['m8_max_freq']:.1f}%  (5-20%)   [{pf['M8']}]")
    print(f"  M9 S/A stddev:            {metrics['m9_sa_stddev']:.2f}  (>= 0.8)  [{pf['M9']}]")


# ---------------------------------------------------------------------------
# Draft Trace
# ---------------------------------------------------------------------------

def run_trace_draft(draft_fn, pool, res_pools, rng, strategy, label,
                    fitness_model="B"):
    """Run a single draft and print a detailed trace."""
    drafted, pack_records, committed_arch = draft_fn(pool, res_pools, rng,
                                                      strategy)
    print(f"\n  === TRACE: {label} (strategy={strategy}, "
          f"committed={ARCHETYPE_NAMES[committed_arch] if committed_arch is not None else 'None'}) ===")

    res_counters = {r: 0 for r in RESONANCES}
    for pick_num, pack, chosen in pack_records:
        if pick_num > 12:
            break
        sa_count = (sum(1 for c in pack if c.is_sa_for(committed_arch))
                    if committed_arch is not None else "?")
        chosen_tier = (chosen.fitness_for(committed_arch)
                       if committed_arch is not None else "?")
        print(f"    Pick {pick_num:2d}: pack_sa={sa_count}, "
              f"chosen={chosen.primary_resonance or 'generic'} "
              f"(tier={chosen_tier}, power={chosen.power:.1f}), "
              f"counters=[E:{res_counters['Ember']:.0f} "
              f"S:{res_counters['Stone']:.0f} "
              f"T:{res_counters['Tide']:.0f} "
              f"Z:{res_counters['Zephyr']:.0f}]")
        _update_resonance_counters(chosen, res_counters)

    # Show final deck composition
    if committed_arch is not None:
        sa = sum(1 for c in drafted if c.is_sa_for(committed_arch))
        s_count = sum(1 for c in drafted
                      if c.fitness_for(committed_arch) == "S")
        a_count = sum(1 for c in drafted
                      if c.fitness_for(committed_arch) == "A")
        print(f"    Final deck: {sa}/{len(drafted)} S/A "
              f"({s_count}S + {a_count}A), "
              f"concentration={sa/len(drafted)*100:.0f}%")


# ---------------------------------------------------------------------------
# Main Simulation
# ---------------------------------------------------------------------------

def run_algorithm(name, draft_fn, pool, res_pools, strategies, seed=42):
    """Run 1000 drafts for a single algorithm."""
    rng = random.Random(seed)
    all_results = []
    for run_idx in range(NUM_DRAFTS):
        strategy = strategies[run_idx % len(strategies)]
        drafted, pack_records, committed_arch = draft_fn(pool, res_pools,
                                                          rng, strategy)
        all_results.append((strategy, drafted, pack_records, committed_arch))
    return all_results


def main():
    strategies = ["committed", "power-chaser", "signal-reader"]

    fitness_models = ["A", "B", "C"]
    fitness_labels = {
        "A": "Optimistic (100% cross-archetype A)",
        "B": "Moderate (50%A/30%B/20%C)",
        "C": "Pessimistic (25%A/40%B/35%C)",
    }

    # Store all results for summary
    all_model_results = {}

    for fm in fitness_models:
        print(f"\n{'='*80}")
        print(f"FITNESS MODEL {fm}: {fitness_labels[fm]}")
        print(f"{'='*80}")

        # Build pool for this fitness model
        pool_rng = random.Random(42)
        pool = build_card_pool(pool_rng, fitness_model=fm)
        res_pools = build_resonance_pools(pool)

        dual_count = sum(1 for c in pool if len(c.resonance_types) >= 2)
        print(f"Pool: {len(pool)} cards, {dual_count} dual-type "
              f"({dual_count/len(pool)*100:.1f}%)")
        for r in RESONANCES:
            print(f"  {r} primary: {len(res_pools[r])} cards")

        model_results = {}

        # --- Algorithm variants ---
        algorithms = [
            # Main champion: Aspiration + Biased Random (2.0x)
            ("Aspiration+Bias (2.0x)",
             lambda p, rp, r, s: aspiration_biased_draft(
                 p, rp, r, s, bias_weight=2.0, r2_min_tokens=3,
                 r2_min_ratio=0.50)),

            # Parameter sensitivity: bias weights
            ("Aspiration+Bias (1.5x)",
             lambda p, rp, r, s: aspiration_biased_draft(
                 p, rp, r, s, bias_weight=1.5, r2_min_tokens=3,
                 r2_min_ratio=0.50)),
            ("Aspiration+Bias (3.0x)",
             lambda p, rp, r, s: aspiration_biased_draft(
                 p, rp, r, s, bias_weight=3.0, r2_min_tokens=3,
                 r2_min_ratio=0.50)),

            # Parameter sensitivity: gate thresholds
            ("Aspiration+Bias R2>=2,40%",
             lambda p, rp, r, s: aspiration_biased_draft(
                 p, rp, r, s, bias_weight=2.0, r2_min_tokens=2,
                 r2_min_ratio=0.40)),
            ("Aspiration+Bias R2>=4,60%",
             lambda p, rp, r, s: aspiration_biased_draft(
                 p, rp, r, s, bias_weight=2.0, r2_min_tokens=4,
                 r2_min_ratio=0.60)),

            # Baselines
            ("Pure Aspiration (no bias)",
             lambda p, rp, r, s: pure_aspiration_draft(
                 p, rp, r, s, r2_min_tokens=3, r2_min_ratio=0.50)),
            ("Surge V6 (T=4/S=3)",
             lambda p, rp, r, s: surge_packs_draft(
                 p, rp, r, s, threshold=4, surge_slots=3)),
        ]

        for algo_name, draft_fn in algorithms:
            print(f"\n  Running: {algo_name}")
            results = run_algorithm(algo_name, draft_fn, pool, res_pools,
                                     strategies, seed=42)
            metrics = compute_metrics(results)
            pf = evaluate_pass_fail(metrics)
            arch_conv = per_archetype_convergence(results)

            print_metrics(metrics, f"{algo_name} [{fm}]", pf)

            # Per-archetype convergence
            print(f"\n  Per-Archetype Convergence:")
            print(f"    {'Archetype':<15} {'Avg Pick':>10} {'N Runs':>8}")
            for aname, (avg, n) in arch_conv.items():
                avg_str = f"{avg:.1f}" if not math.isnan(avg) else "N/A"
                print(f"    {aname:<15} {avg_str:>10} {n:>8}")

            model_results[algo_name] = {
                "metrics": metrics,
                "pf": pf,
                "arch_conv": arch_conv,
            }

        all_model_results[fm] = model_results

        # --- Draft traces (only for Moderate fitness) ---
        if fm == "B":
            print(f"\n{'='*80}")
            print("DRAFT TRACES (Moderate fitness, Aspiration+Bias 2.0x)")
            print(f"{'='*80}")

            trace_rng = random.Random(777)
            trace_fn = lambda p, rp, r, s: aspiration_biased_draft(
                p, rp, r, s, bias_weight=2.0, r2_min_tokens=3,
                r2_min_ratio=0.50)
            run_trace_draft(trace_fn, pool, res_pools,
                           random.Random(100), "committed", "Early Committer",
                           fitness_model=fm)
            run_trace_draft(trace_fn, pool, res_pools,
                           random.Random(200), "power-chaser", "Power Chaser",
                           fitness_model=fm)
            run_trace_draft(trace_fn, pool, res_pools,
                           random.Random(300), "signal-reader", "Signal Reader",
                           fitness_model=fm)

    # ========================================================================
    # SUMMARY TABLES
    # ========================================================================
    print(f"\n\n{'='*120}")
    print("UNIFIED COMPARISON: Aspiration+Bias across fitness models")
    print(f"{'='*120}")

    for fm in fitness_models:
        mr = all_model_results[fm]
        print(f"\n  --- Fitness Model {fm}: {fitness_labels[fm]} ---")
        header = (f"  {'Algorithm':<30} {'M1':>5} {'M2':>5} {'M3':>6} "
                  f"{'M4':>5} {'M5':>6} {'M6':>6} {'M7':>6} {'M8':>10} "
                  f"{'M9':>5} {'Pass':>5}")
        print(header)
        print("  " + "-" * (len(header) - 2))

        for algo_name in mr:
            m = mr[algo_name]["metrics"]
            pf = mr[algo_name]["pf"]
            passed = sum(1 for v in pf.values() if v == "PASS")
            m8_str = f"{m['m8_min_freq']:.0f}-{m['m8_max_freq']:.0f}%"
            line = (f"  {algo_name:<30} "
                    f"{m['m1_unique_archetypes_early']:>5.1f} "
                    f"{m['m2_sa_emerging_early']:>5.2f} "
                    f"{m['m3_sa_committed_late']:>6.2f} "
                    f"{m['m4_off_archetype_late']:>5.2f} "
                    f"{m['m5_convergence_pick']:>6.1f} "
                    f"{m['m6_deck_concentration']:>5.1f}% "
                    f"{m['m7_card_overlap']:>5.1f}% "
                    f"{m8_str:>10} "
                    f"{m['m9_sa_stddev']:>5.2f} "
                    f"{passed:>4}/9")
            print(line)

    # ========================================================================
    # PASS/FAIL SUMMARY
    # ========================================================================
    print(f"\n\n{'='*120}")
    print("PASS/FAIL SUMMARY")
    print(f"{'='*120}")

    for fm in fitness_models:
        mr = all_model_results[fm]
        print(f"\n  --- Fitness Model {fm} ---")
        header = (f"  {'Algorithm':<30} {'M1':>5} {'M2':>5} {'M3':>5} "
                  f"{'M4':>5} {'M5':>5} {'M6':>5} {'M7':>5} {'M8':>5} "
                  f"{'M9':>5} {'Total':>6}")
        print(header)
        print("  " + "-" * (len(header) - 2))

        for algo_name in mr:
            pf = mr[algo_name]["pf"]
            passed = sum(1 for v in pf.values() if v == "PASS")
            line = (f"  {algo_name:<30} "
                    f"{pf['M1']:>5} {pf['M2']:>5} {pf['M3']:>5} "
                    f"{pf['M4']:>5} {pf['M5']:>5} {pf['M6']:>5} "
                    f"{pf['M7']:>5} {pf['M8']:>5} {pf['M9']:>5} "
                    f"{passed:>4}/9")
            print(line)

    # ========================================================================
    # FITNESS DEGRADATION CURVE
    # ========================================================================
    print(f"\n\n{'='*120}")
    print("FITNESS DEGRADATION CURVE (M3: S/A committed late)")
    print(f"{'='*120}")

    key_algos = ["Aspiration+Bias (2.0x)", "Pure Aspiration (no bias)",
                 "Surge V6 (T=4/S=3)"]
    print(f"\n  {'Algorithm':<30} {'Model A':>10} {'Model B':>10} "
          f"{'Model C':>10} {'A->B drop':>10} {'A->C drop':>10}")
    print("  " + "-" * 82)
    for algo in key_algos:
        vals = []
        for fm in fitness_models:
            vals.append(all_model_results[fm][algo]["metrics"]["m3_sa_committed_late"])
        drop_ab = vals[0] - vals[1]
        drop_ac = vals[0] - vals[2]
        print(f"  {algo:<30} {vals[0]:>10.2f} {vals[1]:>10.2f} "
              f"{vals[2]:>10.2f} {drop_ab:>10.2f} {drop_ac:>10.2f}")

    # ========================================================================
    # BIAS WEIGHT SENSITIVITY
    # ========================================================================
    print(f"\n\n{'='*120}")
    print("PARAMETER SENSITIVITY: Bias Weight (Model B only)")
    print(f"{'='*120}")

    bias_algos = ["Aspiration+Bias (1.5x)", "Aspiration+Bias (2.0x)",
                  "Aspiration+Bias (3.0x)", "Pure Aspiration (no bias)"]
    mr_b = all_model_results["B"]
    print(f"\n  {'Variant':<30} {'M3 (S/A)':>10} {'M4 (off)':>10} "
          f"{'M5 (conv)':>10} {'M6 (conc)':>10} {'M9 (std)':>10} {'Pass':>6}")
    print("  " + "-" * 88)
    for algo in bias_algos:
        if algo not in mr_b:
            continue
        m = mr_b[algo]["metrics"]
        p = sum(1 for v in mr_b[algo]["pf"].values() if v == "PASS")
        print(f"  {algo:<30} {m['m3_sa_committed_late']:>10.2f} "
              f"{m['m4_off_archetype_late']:>10.2f} "
              f"{m['m5_convergence_pick']:>10.1f} "
              f"{m['m6_deck_concentration']:>9.1f}% "
              f"{m['m9_sa_stddev']:>10.2f} {p:>4}/9")

    # ========================================================================
    # GATE THRESHOLD SENSITIVITY
    # ========================================================================
    print(f"\n\n{'='*120}")
    print("PARAMETER SENSITIVITY: Gate Thresholds (Model B only)")
    print(f"{'='*120}")

    gate_algos = ["Aspiration+Bias R2>=2,40%", "Aspiration+Bias (2.0x)",
                  "Aspiration+Bias R2>=4,60%"]
    print(f"\n  {'Variant':<30} {'M3 (S/A)':>10} {'M2 (early)':>10} "
          f"{'M5 (conv)':>10} {'M6 (conc)':>10} {'M9 (std)':>10} {'Pass':>6}")
    print("  " + "-" * 88)
    for algo in gate_algos:
        if algo not in mr_b:
            continue
        m = mr_b[algo]["metrics"]
        p = sum(1 for v in mr_b[algo]["pf"].values() if v == "PASS")
        print(f"  {algo:<30} {m['m3_sa_committed_late']:>10.2f} "
              f"{m['m2_sa_emerging_early']:>10.2f} "
              f"{m['m5_convergence_pick']:>10.1f} "
              f"{m['m6_deck_concentration']:>9.1f}% "
              f"{m['m9_sa_stddev']:>10.2f} {p:>4}/9")


if __name__ == "__main__":
    main()
