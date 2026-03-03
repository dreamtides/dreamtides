#!/usr/bin/env python3
"""
Simulation Agent 6 (V7 Round 3): Dual-Counter Surge

One-sentence algorithm: "Each drafted symbol adds resonance tokens (+2 primary,
+1 others); maintain running average cost of drafted cards; when any resonance
counter reaches 4, spend 4 and fill 3 surge slots with cards matching that
resonance AND within +/-1 of the player's average cost (widening to +/-2, then
unfiltered if insufficient cards), fourth slot random."

No player decisions beyond picking 1 card from a pack of 4.
"""

import random
import statistics
import math
from collections import defaultdict, Counter
from dataclasses import dataclass, field
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
SEED = 42

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

# Cost distributions per archetype (mean, stddev)
# Aggro: Flash, Blink, Self-Discard center 2-3
# Midrange: Warriors, Sacrifice center 3-4
# Control/Ramp: Storm, Self-Mill, Ramp center 4-5
COST_PROFILES = {
    0: (2.5, 1.2),   # Flash - aggro/tempo
    1: (2.8, 1.3),   # Blink - aggro-ish ETB
    2: (4.2, 1.4),   # Storm - spell-heavy control
    3: (2.3, 1.1),   # Self-Discard - cheap discard enablers
    4: (4.5, 1.3),   # Self-Mill - expensive reanimator
    5: (3.2, 1.3),   # Sacrifice - midrange sac
    6: (3.8, 1.2),   # Warriors - midrange tribal
    7: (4.8, 1.2),   # Ramp - expensive payoffs
}


# ---------------------------------------------------------------------------
# Fitness Models
# ---------------------------------------------------------------------------

def build_optimistic_fitness():
    """Model A: All cross-archetype sharing primary = A-tier."""
    fm = {}
    for home_idx, (_, home_pri, home_sec) in enumerate(ARCHETYPES):
        fm[home_idx] = {}
        for eval_idx, (_, eval_pri, eval_sec) in enumerate(ARCHETYPES):
            if eval_idx == home_idx:
                fm[home_idx][eval_idx] = "S"
            elif eval_pri == home_pri:
                fm[home_idx][eval_idx] = "A"
            elif eval_sec == home_pri or eval_pri == home_sec:
                fm[home_idx][eval_idx] = "B"
            else:
                fm[home_idx][eval_idx] = "C"
    return fm


def build_moderate_fitness(rng):
    """Model B: Sibling cards (share primary) = 50%A/30%B/20%C per card.
    S/A precision ~75%."""
    fm = {}
    for home_idx, (_, home_pri, home_sec) in enumerate(ARCHETYPES):
        fm[home_idx] = {}
        for eval_idx, (_, eval_pri, eval_sec) in enumerate(ARCHETYPES):
            if eval_idx == home_idx:
                fm[home_idx][eval_idx] = "S"
            elif eval_pri == home_pri:
                # Roll: 50% A, 30% B, 20% C
                fm[home_idx][eval_idx] = "ROLL_MODERATE"
            elif eval_sec == home_pri or eval_pri == home_sec:
                fm[home_idx][eval_idx] = "B"
            else:
                fm[home_idx][eval_idx] = "C"
    return fm


def build_pessimistic_fitness(rng):
    """Model C: Sibling cards = 25%A/40%B/35%C per card.
    S/A precision ~62.5%."""
    fm = {}
    for home_idx, (_, home_pri, home_sec) in enumerate(ARCHETYPES):
        fm[home_idx] = {}
        for eval_idx, (_, eval_pri, eval_sec) in enumerate(ARCHETYPES):
            if eval_idx == home_idx:
                fm[home_idx][eval_idx] = "S"
            elif eval_pri == home_pri:
                fm[home_idx][eval_idx] = "ROLL_PESSIMISTIC"
            elif eval_sec == home_pri or eval_pri == home_sec:
                fm[home_idx][eval_idx] = "B"
            else:
                fm[home_idx][eval_idx] = "C"
    return fm


def resolve_roll(roll_type, rng):
    """Resolve a per-card fitness roll."""
    r = rng.random()
    if roll_type == "ROLL_MODERATE":
        if r < 0.50:
            return "A"
        elif r < 0.80:
            return "B"
        else:
            return "C"
    elif roll_type == "ROLL_PESSIMISTIC":
        if r < 0.25:
            return "A"
        elif r < 0.65:
            return "B"
        else:
            return "C"
    return roll_type


TIER_SCORE = {"S": 4.0, "A": 3.0, "B": 2.0, "C": 0.5, "F": 0.0}


# ---------------------------------------------------------------------------
# Card
# ---------------------------------------------------------------------------

@dataclass
class SimCard:
    id: int
    symbols: list
    archetype_idx: int  # -1 for generic
    power: float
    cost: int  # energy cost 0-7
    # Per-card resolved fitness (filled after pool construction)
    resolved_fitness: dict = field(default_factory=dict)

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
        return self.resolved_fitness.get(archetype_idx, "B")

    def fitness_score_for(self, archetype_idx: int) -> float:
        return TIER_SCORE[self.fitness_for(archetype_idx)]

    def is_sa_for(self, archetype_idx: int) -> bool:
        return self.fitness_for(archetype_idx) in ("S", "A")


# ---------------------------------------------------------------------------
# Card Pool Construction
# ---------------------------------------------------------------------------

def assign_cost(archetype_idx: int, rng: random.Random) -> int:
    """Assign energy cost based on archetype cost profile."""
    if archetype_idx == -1:
        # Generic: uniform 1-5
        return rng.randint(1, 5)
    mean, std = COST_PROFILES[archetype_idx]
    cost = round(rng.gauss(mean, std))
    return max(0, min(7, cost))


def build_card_pool(rng: random.Random, fitness_model: str = "optimistic") -> list:
    """Build the 360-card pool with energy costs and resolved fitness."""
    fitness_rng = random.Random(rng.randint(0, 2**32))

    if fitness_model == "optimistic":
        fm_template = build_optimistic_fitness()
    elif fitness_model == "moderate":
        fm_template = build_moderate_fitness(fitness_rng)
    elif fitness_model == "pessimistic":
        fm_template = build_pessimistic_fitness(fitness_rng)
    else:
        raise ValueError(f"Unknown fitness model: {fitness_model}")

    cards = []
    card_id = 0

    # Generic cards: 36
    for _ in range(NUM_GENERIC):
        cost = assign_cost(-1, rng)
        c = SimCard(id=card_id, symbols=[], archetype_idx=-1,
                    power=rng.uniform(3.0, 7.0), cost=cost)
        for ai in range(NUM_ARCHETYPES):
            c.resolved_fitness[ai] = "B"
        cards.append(c)
        card_id += 1

    # Archetype cards: ~40 per archetype
    trimmed = rng.sample(range(NUM_ARCHETYPES), 2)
    boosted = [i for i in range(NUM_ARCHETYPES) if i not in trimmed][:4]

    for arch_idx, (name, pri, sec) in enumerate(ARCHETYPES):
        is_trimmed = arch_idx in trimmed
        is_boosted = arch_idx in boosted
        n_dual = 3 if is_trimmed else 4
        n_mono_1sym = 11 if is_boosted else 10
        n_mono_2sym = 18 if is_trimmed else 17

        symbol_configs = (
            [([pri], ) for _ in range(n_mono_1sym)] +
            [([pri, pri], ) for _ in range(n_mono_2sym)] +
            [([pri, pri, pri], ) for _ in range(6)] +
            [([pri, sec], ) for _ in range(n_dual)] +
            [([pri, pri, sec], ) for _ in range(3)]
        )

        for (symbols,) in symbol_configs:
            cost = assign_cost(arch_idx, rng)
            power = rng.uniform(2.0, 9.0)
            c = SimCard(id=card_id, symbols=list(symbols),
                        archetype_idx=arch_idx, power=power, cost=cost)

            # Resolve fitness for all archetypes
            for eval_idx in range(NUM_ARCHETYPES):
                base = fm_template[arch_idx][eval_idx]
                if base.startswith("ROLL"):
                    c.resolved_fitness[eval_idx] = resolve_roll(base, fitness_rng)
                else:
                    c.resolved_fitness[eval_idx] = base

            cards.append(c)
            card_id += 1

    assert len(cards) == TOTAL_CARDS, f"Expected {TOTAL_CARDS}, got {len(cards)}"
    return cards


def build_resonance_pools(pool: list) -> dict:
    """Map resonance -> list of cards with that primary resonance."""
    pools = {}
    for r in RESONANCES:
        pools[r] = [c for c in pool if c.primary_resonance == r]
    return pools


def build_resonance_cost_index(pool: list) -> dict:
    """Map (resonance, cost) -> list of cards for efficient cost-filtered lookups."""
    idx = defaultdict(list)
    for c in pool:
        if c.primary_resonance is not None:
            idx[(c.primary_resonance, c.cost)].append(c)
    return idx


# ---------------------------------------------------------------------------
# Player Strategies
# ---------------------------------------------------------------------------

def _choose_card(pack, strategy, committed_archetype, pick_num,
                 resonance_signals, rng):
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
    return max(pack, key=lambda c: (c.fitness_score_for(committed_archetype), c.power))


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
    return max(pack, key=lambda c: (c.fitness_score_for(committed_archetype), c.power))


def _best_archetype_from_drafted(drafted):
    scores = [0.0] * NUM_ARCHETYPES
    for card in drafted:
        for a in range(NUM_ARCHETYPES):
            scores[a] += card.fitness_score_for(a)
    return scores.index(max(scores))


def _signal_read_commit(drafted, resonance_signals):
    sorted_res = sorted(RESONANCES, key=lambda r: resonance_signals.get(r, 0),
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
# Dual-Counter Surge Algorithm
# ---------------------------------------------------------------------------

def _get_cost_filtered_cards(res_pool, res_cost_idx, resonance, avg_cost, window):
    """Get cards matching resonance within cost window of avg_cost."""
    lo = int(math.floor(avg_cost - window))
    hi = int(math.ceil(avg_cost + window))
    lo = max(0, lo)
    hi = min(7, hi)
    candidates = []
    for cost_val in range(lo, hi + 1):
        candidates.extend(res_cost_idx.get((resonance, cost_val), []))
    return candidates


def dual_counter_surge_draft(pool, res_pools, res_cost_idx, rng, strategy,
                              threshold=4, surge_slots=3, cost_window=1.0):
    """Dual-Counter Surge: resonance tokens + cost-profile filtering on surges."""
    token_counters = {r: 0 for r in RESONANCES}
    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}
    surge_resonance = None
    total_cost = 0.0
    num_drafted = 0

    # Track cost filter effectiveness
    surge_slot_details = []  # list of (card, resonance, was_cost_filtered, card_archetype)

    for pick_num in range(NUM_PICKS):
        avg_cost = total_cost / num_drafted if num_drafted > 0 else 3.5

        pack = []
        is_surge = False
        if surge_resonance is not None:
            is_surge = True
            # Fill surge_slots with cost-filtered resonance cards
            for slot_idx in range(PACK_SIZE):
                if slot_idx < surge_slots:
                    # Try cost window +/-1, then +/-2, then unfiltered
                    card = None
                    was_filtered = False
                    for w in [cost_window, cost_window + 1.0]:
                        candidates = _get_cost_filtered_cards(
                            res_pools[surge_resonance], res_cost_idx,
                            surge_resonance, avg_cost, w)
                        if len(candidates) >= 3:
                            card = rng.choice(candidates)
                            was_filtered = True
                            break
                    if card is None:
                        # Fallback: unfiltered resonance match
                        card = rng.choice(res_pools[surge_resonance])
                        was_filtered = False
                    pack.append(card)
                    surge_slot_details.append({
                        "card": card,
                        "resonance": surge_resonance,
                        "was_cost_filtered": was_filtered,
                        "avg_cost": avg_cost,
                    })
                else:
                    # Random slot
                    pack.append(rng.choice(pool))
            surge_resonance = None
        else:
            pack = [rng.choice(pool) for _ in range(PACK_SIZE)]

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen, is_surge))
        drafted.append(chosen)

        # Update cost tracking
        total_cost += chosen.cost
        num_drafted += 1

        # Earn tokens: +2 primary, +1 others
        for i, sym in enumerate(chosen.symbols):
            weight = 2 if i == 0 else 1
            token_counters[sym] += weight

        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        # Check surge trigger
        max_res = max(RESONANCES, key=lambda r: token_counters[r])
        if token_counters[max_res] >= threshold:
            token_counters[max_res] -= threshold
            surge_resonance = max_res

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals)

    return drafted, pack_records, committed_archetype, surge_slot_details


# ---------------------------------------------------------------------------
# Surge Packs Baseline (for comparison)
# ---------------------------------------------------------------------------

def surge_packs_draft(pool, res_pools, rng, strategy, threshold=4, surge_slots=3):
    """Standard Surge Packs (V6 winner) for baseline comparison."""
    token_counters = {r: 0 for r in RESONANCES}
    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}
    surge_resonance = None

    for pick_num in range(NUM_PICKS):
        pack = []
        is_surge = False
        if surge_resonance is not None:
            is_surge = True
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
                              resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen, is_surge))
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

    return drafted, pack_records, committed_archetype, []


# ---------------------------------------------------------------------------
# Metrics Computation
# ---------------------------------------------------------------------------

def compute_metrics(all_results: list) -> dict:
    """Compute all 9 metrics from simulation results.
    all_results: list of (strategy, drafted, pack_records, committed_arch, surge_details)
    """
    metrics = {}
    by_strategy = defaultdict(list)
    for strategy, drafted, pack_records, committed_arch, surge_details in all_results:
        by_strategy[strategy].append((drafted, pack_records, committed_arch, surge_details))

    # M1: Picks 1-5 unique archetypes with S/A cards per pack
    m1_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch, _ in results:
            for pick_num, pack, chosen, is_surge in pack_records:
                if pick_num >= 5:
                    continue
                archs_with_sa = set()
                for card in pack:
                    for a in range(NUM_ARCHETYPES):
                        if card.is_sa_for(a):
                            archs_with_sa.add(a)
                m1_values.append(len(archs_with_sa))
    metrics["m1"] = statistics.mean(m1_values) if m1_values else 0

    # M2: Picks 1-5 S/A for emerging archetype per pack
    m2_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch, _ in results:
            if committed_arch is None:
                continue
            for pick_num, pack, chosen, is_surge in pack_records:
                if pick_num >= 5:
                    continue
                sa_count = sum(1 for c in pack[:PACK_SIZE] if c.is_sa_for(committed_arch))
                m2_values.append(sa_count)
    metrics["m2"] = statistics.mean(m2_values) if m2_values else 0

    # M3: Picks 6+ S/A for committed archetype per pack
    m3_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch, _ in results:
            if committed_arch is None:
                continue
            for pick_num, pack, chosen, is_surge in pack_records:
                if pick_num < 5:
                    continue
                sa_count = sum(1 for c in pack if c.is_sa_for(committed_arch))
                m3_values.append(sa_count)
    metrics["m3"] = statistics.mean(m3_values) if m3_values else 0

    # M4: Picks 6+ off-archetype (C/F) per pack
    m4_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch, _ in results:
            if committed_arch is None:
                continue
            for pick_num, pack, chosen, is_surge in pack_records:
                if pick_num < 5:
                    continue
                cf_count = sum(1 for c in pack
                               if c.fitness_for(committed_arch) in ("C", "F"))
                m4_values.append(cf_count)
    metrics["m4"] = statistics.mean(m4_values) if m4_values else 0

    # M5: Convergence pick (3-pack window where all have >= 2 S/A)
    convergence_picks = []
    for strategy, results in by_strategy.items():
        if strategy == "power-chaser":
            continue
        for drafted, pack_records, committed_arch, _ in results:
            if committed_arch is None:
                continue
            found = False
            for pick_num, pack, chosen, is_surge in pack_records:
                if pick_num < 2 or pick_num + 2 >= len(pack_records):
                    continue
                all_good = True
                for w in range(3):
                    _, wp, _, _ = pack_records[pick_num + w]
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
    metrics["m5"] = statistics.mean(convergence_picks) if convergence_picks else 30

    # M6: Deck archetype concentration
    m6_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch, _ in results:
            if committed_arch is None:
                continue
            sa_cards = sum(1 for c in drafted if c.is_sa_for(committed_arch))
            m6_values.append(sa_cards / len(drafted) * 100)
    metrics["m6"] = statistics.mean(m6_values) if m6_values else 0

    # M7: Run-to-run card overlap
    archetype_runs = defaultdict(list)
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch, _ in results:
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
    metrics["m7"] = statistics.mean(overlaps) if overlaps else 0

    # M8: Archetype frequency
    arch_counts = Counter()
    total_committed = 0
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch, _ in results:
            if committed_arch is not None:
                arch_counts[committed_arch] += 1
                total_committed += 1

    arch_freqs = {}
    for a in range(NUM_ARCHETYPES):
        arch_freqs[ARCHETYPE_NAMES[a]] = (arch_counts[a] / total_committed * 100
                                           if total_committed > 0 else 0)
    metrics["m8_freq"] = arch_freqs
    metrics["m8_max"] = max(arch_freqs.values()) if arch_freqs else 0
    metrics["m8_min"] = min(arch_freqs.values()) if arch_freqs else 0

    # M9: StdDev of S/A per pack (picks 6+)
    metrics["m9"] = statistics.stdev(m3_values) if len(m3_values) > 1 else 0

    return metrics


def per_archetype_convergence(all_results: list) -> dict:
    """Per-archetype convergence table."""
    arch_conv = defaultdict(list)
    for strategy, drafted, pack_records, committed_arch, _ in all_results:
        if strategy == "power-chaser" or committed_arch is None:
            continue
        found = False
        for pick_num, pack, chosen, is_surge in pack_records:
            if pick_num < 2 or pick_num + 2 >= len(pack_records):
                continue
            all_good = True
            for w in range(3):
                _, wp, _, _ = pack_records[pick_num + w]
                sa = sum(1 for c in wp if c.is_sa_for(committed_arch))
                if sa < 2:
                    all_good = False
                    break
            if all_good:
                arch_conv[committed_arch].append(pick_num)
                found = True
                break
        if not found:
            arch_conv[committed_arch].append(30)

    result = {}
    for a in range(NUM_ARCHETYPES):
        picks = arch_conv.get(a, [])
        if picks:
            result[ARCHETYPE_NAMES[a]] = {
                "mean": statistics.mean(picks),
                "median": statistics.median(picks),
                "count": len(picks),
            }
        else:
            result[ARCHETYPE_NAMES[a]] = {"mean": float("nan"), "median": float("nan"),
                                           "count": 0}
    return result


def evaluate_pass_fail(metrics):
    """Evaluate pass/fail for all 9 metrics."""
    results = {}
    results["M1"] = "PASS" if metrics["m1"] >= 3 else "FAIL"
    results["M2"] = "PASS" if metrics["m2"] <= 2 else "FAIL"
    results["M3"] = "PASS" if metrics["m3"] >= 2.0 else "FAIL"
    results["M4"] = "PASS" if metrics["m4"] >= 0.5 else "FAIL"
    results["M5"] = "PASS" if 5 <= metrics["m5"] <= 8 else "FAIL"
    results["M6"] = "PASS" if 60 <= metrics["m6"] <= 90 else "FAIL"
    results["M7"] = "PASS" if metrics["m7"] < 40 else "FAIL"
    results["M8"] = "PASS" if (metrics["m8_min"] >= 5 and
                                metrics["m8_max"] <= 20) else "FAIL"
    results["M9"] = "PASS" if metrics["m9"] >= 0.8 else "FAIL"
    return results


# ---------------------------------------------------------------------------
# Cost Filter Effectiveness Analysis
# ---------------------------------------------------------------------------

def analyze_cost_filter(all_results: list) -> dict:
    """Analyze what % of cost-filtered surge slots are home vs sibling archetype."""
    analysis = {
        "total_surge_slots": 0,
        "cost_filtered_slots": 0,
        "home_archetype": 0,
        "sibling_archetype": 0,
        "secondary_sharing": 0,
        "distant": 0,
        "generic": 0,
    }

    for strategy, drafted, pack_records, committed_arch, surge_details in all_results:
        if committed_arch is None:
            continue
        for detail in surge_details:
            analysis["total_surge_slots"] += 1
            card = detail["card"]
            if detail["was_cost_filtered"]:
                analysis["cost_filtered_slots"] += 1

            if card.is_generic:
                analysis["generic"] += 1
            elif card.archetype_idx == committed_arch:
                analysis["home_archetype"] += 1
            else:
                # Check relationship
                card_arch = ARCHETYPES[card.archetype_idx]
                target_arch = ARCHETYPES[committed_arch]
                if card_arch[1] == target_arch[1]:  # share primary
                    analysis["sibling_archetype"] += 1
                elif (card_arch[2] == target_arch[1] or
                      card_arch[1] == target_arch[2]):
                    analysis["secondary_sharing"] += 1
                else:
                    analysis["distant"] += 1

    return analysis


# ---------------------------------------------------------------------------
# Draft Traces
# ---------------------------------------------------------------------------

def format_trace(result_tuple, label):
    """Format a detailed draft trace."""
    strategy, drafted, pack_records, committed_arch, surge_details = result_tuple
    lines = [f"\n### {label}"]
    lines.append(f"Strategy: {strategy}")
    lines.append(f"Final archetype: {ARCHETYPE_NAMES[committed_arch] if committed_arch is not None else 'None'}")
    lines.append("")
    lines.append("| Pick | Surge? | Card Symbols | Cost | Pack S/A | Avg Cost |")
    lines.append("|------|--------|-------------|------|----------|----------|")

    total_cost = 0.0
    for idx, (pick_num, pack, chosen, is_surge) in enumerate(pack_records[:15]):
        total_cost += chosen.cost
        avg_c = total_cost / (idx + 1)
        syms = ",".join(chosen.symbols) if chosen.symbols else "generic"
        sa = sum(1 for c in pack if c.is_sa_for(committed_arch)) if committed_arch else "?"
        surge_mark = "YES" if is_surge else "no"
        lines.append(
            f"| {pick_num + 1:2d}   | {surge_mark:6s} | {syms:11s} | {chosen.cost:4d} "
            f"| {sa:>8} | {avg_c:>8.1f} |")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Printing
# ---------------------------------------------------------------------------

def print_metrics(metrics, name, pf):
    passed = sum(1 for v in pf.values() if v == "PASS")
    print(f"\n--- {name} ({passed}/9 PASS) ---")
    print(f"  M1 Unique archs early:    {metrics['m1']:.2f}  (>= 3)    [{pf['M1']}]")
    print(f"  M2 S/A early:             {metrics['m2']:.2f}  (<= 2)    [{pf['M2']}]")
    print(f"  M3 S/A committed late:    {metrics['m3']:.2f}  (>= 2.0)  [{pf['M3']}]")
    print(f"  M4 Off-arch late:         {metrics['m4']:.2f}  (>= 0.5)  [{pf['M4']}]")
    print(f"  M5 Convergence pick:      {metrics['m5']:.1f}  (5-8)     [{pf['M5']}]")
    print(f"  M6 Deck concentration:    {metrics['m6']:.1f}%  (60-90%)  [{pf['M6']}]")
    print(f"  M7 Card overlap:          {metrics['m7']:.1f}%  (< 40%)   [{pf['M7']}]")
    print(f"  M8 Arch freq range:       {metrics['m8_min']:.1f}%-{metrics['m8_max']:.1f}%  (5-20%)   [{pf['M8']}]")
    print(f"  M9 S/A stddev:            {metrics['m9']:.2f}  (>= 0.8)  [{pf['M9']}]")


# ---------------------------------------------------------------------------
# Main Simulation
# ---------------------------------------------------------------------------

def run_algorithm(draft_fn, pool, res_pools, res_cost_idx, rng, strategies,
                  uses_cost_idx=True, **kwargs):
    """Run 1000 drafts x 3 strategies for an algorithm."""
    all_results = []
    for strategy in strategies:
        strat_rng = random.Random(rng.randint(0, 2**32))
        for _ in range(NUM_DRAFTS):
            if uses_cost_idx:
                drafted, pack_records, committed_arch, surge_details = draft_fn(
                    pool, res_pools, res_cost_idx, strat_rng, strategy, **kwargs)
            else:
                drafted, pack_records, committed_arch, surge_details = draft_fn(
                    pool, res_pools, strat_rng, strategy, **kwargs)
            all_results.append((strategy, drafted, pack_records, committed_arch,
                                surge_details))
    return all_results


def main():
    rng = random.Random(SEED)

    strategies = ["committed", "power-chaser", "signal-reader"]
    fitness_models = ["optimistic", "moderate", "pessimistic"]

    all_algo_results = {}

    for fm_name in fitness_models:
        pool_rng = random.Random(SEED)
        pool = build_card_pool(pool_rng, fitness_model=fm_name)
        res_pools = build_resonance_pools(pool)
        res_cost_idx = build_resonance_cost_index(pool)

        dual_count = sum(1 for c in pool if len(c.resonance_types) >= 2)
        print(f"\n{'='*70}")
        print(f"FITNESS MODEL: {fm_name.upper()}")
        print(f"{'='*70}")
        print(f"Pool: {len(pool)} cards, {dual_count} dual-type")
        for r in RESONANCES:
            print(f"  {r}: {len(res_pools[r])} cards")

        # Print archetype cost profiles
        print("\nArchetype cost distributions in pool:")
        for ai in range(NUM_ARCHETYPES):
            arch_cards = [c for c in pool if c.archetype_idx == ai]
            if arch_cards:
                costs = [c.cost for c in arch_cards]
                print(f"  {ARCHETYPE_NAMES[ai]:14s}: mean={statistics.mean(costs):.1f}, "
                      f"std={statistics.stdev(costs):.1f}, range=[{min(costs)}-{max(costs)}]")

        # --- Dual-Counter Surge (primary champion) ---
        print(f"\n--- Running Dual-Counter Surge (T=4, W=+/-1) under {fm_name} ---")
        algo_rng = random.Random(SEED + 100)
        dcs_results = run_algorithm(
            dual_counter_surge_draft, pool, res_pools, res_cost_idx, algo_rng,
            strategies, uses_cost_idx=True, threshold=4, surge_slots=3,
            cost_window=1.0)

        dcs_metrics = compute_metrics(dcs_results)
        dcs_pf = evaluate_pass_fail(dcs_metrics)
        print_metrics(dcs_metrics, f"Dual-Counter Surge ({fm_name})", dcs_pf)

        dcs_conv = per_archetype_convergence(dcs_results)
        print(f"\n  Per-Archetype Convergence:")
        print(f"    {'Archetype':<15} {'Mean':>8} {'Median':>8} {'Count':>6}")
        for name in ARCHETYPE_NAMES:
            d = dcs_conv.get(name, {})
            m = d.get("mean", float("nan"))
            md = d.get("median", float("nan"))
            n = d.get("count", 0)
            m_str = f"{m:.1f}" if not math.isnan(m) else "N/A"
            md_str = f"{md:.1f}" if not math.isnan(md) else "N/A"
            print(f"    {name:<15} {m_str:>8} {md_str:>8} {n:>6}")

        # Cost filter analysis
        if fm_name != "optimistic":
            cost_analysis = analyze_cost_filter(dcs_results)
            total = cost_analysis["total_surge_slots"]
            if total > 0:
                print(f"\n  Cost Filter Effectiveness ({fm_name}):")
                print(f"    Total surge slots:   {total}")
                print(f"    Cost-filtered:       {cost_analysis['cost_filtered_slots']} "
                      f"({cost_analysis['cost_filtered_slots']/total*100:.1f}%)")
                print(f"    Home archetype:      {cost_analysis['home_archetype']} "
                      f"({cost_analysis['home_archetype']/total*100:.1f}%)")
                print(f"    Sibling archetype:   {cost_analysis['sibling_archetype']} "
                      f"({cost_analysis['sibling_archetype']/total*100:.1f}%)")
                print(f"    Secondary-sharing:   {cost_analysis['secondary_sharing']} "
                      f"({cost_analysis['secondary_sharing']/total*100:.1f}%)")
                print(f"    Distant:             {cost_analysis['distant']} "
                      f"({cost_analysis['distant']/total*100:.1f}%)")
                print(f"    Generic:             {cost_analysis['generic']} "
                      f"({cost_analysis['generic']/total*100:.1f}%)")
                home_rate = cost_analysis['home_archetype'] / total * 100
                sibling_rate = cost_analysis['sibling_archetype'] / total * 100
                print(f"    Home selection rate:  {home_rate:.1f}%")
                print(f"    Home+Sibling rate:   {home_rate + sibling_rate:.1f}%")

        all_algo_results[f"dcs_{fm_name}"] = {
            "metrics": dcs_metrics, "pf": dcs_pf, "conv": dcs_conv,
            "results": dcs_results,
        }

        # --- Surge Packs Baseline (for comparison) ---
        print(f"\n--- Running Surge Packs Baseline (T=4, S=3) under {fm_name} ---")
        baseline_rng = random.Random(SEED + 200)
        sp_results = run_algorithm(
            surge_packs_draft, pool, res_pools, res_cost_idx, baseline_rng,
            strategies, uses_cost_idx=False, threshold=4, surge_slots=3)

        sp_metrics = compute_metrics(sp_results)
        sp_pf = evaluate_pass_fail(sp_metrics)
        print_metrics(sp_metrics, f"Surge Packs Baseline ({fm_name})", sp_pf)

        all_algo_results[f"sp_{fm_name}"] = {
            "metrics": sp_metrics, "pf": sp_pf,
        }

    # ========== Parameter Sensitivity ==========
    print(f"\n\n{'='*70}")
    print("PARAMETER SENSITIVITY")
    print(f"{'='*70}")

    sensitivity_configs = [
        # (label, threshold, cost_window)
        ("T=3, W=1.0", 3, 1.0),
        ("T=3, W=1.5", 3, 1.5),
        ("T=3, W=2.0", 3, 2.0),
        ("T=4, W=1.0", 4, 1.0),
        ("T=4, W=1.5", 4, 1.5),
        ("T=4, W=2.0", 4, 2.0),
        ("T=5, W=1.0", 5, 1.0),
        ("T=5, W=1.5", 5, 1.5),
        ("T=5, W=2.0", 5, 2.0),
    ]

    # Run sensitivity under moderate fitness
    pool_rng = random.Random(SEED)
    pool = build_card_pool(pool_rng, fitness_model="moderate")
    res_pools = build_resonance_pools(pool)
    res_cost_idx = build_resonance_cost_index(pool)

    print(f"\n{'Label':<16} {'M3 S/A':>8} {'M5 Conv':>8} {'M6 Deck%':>9} {'M9 StdDev':>10} {'Pass':>6}")
    print("-" * 60)

    for label, threshold, cost_window in sensitivity_configs:
        sens_rng = random.Random(SEED + 300 + hash(label) % 10000)
        sens_results = []
        for strategy in strategies:
            strat_rng = random.Random(sens_rng.randint(0, 2**32))
            for _ in range(300):  # fewer drafts for sweep
                drafted, pack_records, committed_arch, surge_details = \
                    dual_counter_surge_draft(
                        pool, res_pools, res_cost_idx, strat_rng, strategy,
                        threshold=threshold, surge_slots=3, cost_window=cost_window)
                sens_results.append((strategy, drafted, pack_records,
                                     committed_arch, surge_details))

        m = compute_metrics(sens_results)
        pf = evaluate_pass_fail(m)
        passed = sum(1 for v in pf.values() if v == "PASS")
        print(f"{label:<16} {m['m3']:>8.2f} {m['m5']:>8.1f} {m['m6']:>8.1f}% "
              f"{m['m9']:>10.2f} {passed:>5}/9")

    # ========== Draft Traces ==========
    print(f"\n\n{'='*70}")
    print("DRAFT TRACES (Moderate Fitness, T=4, W=+/-1)")
    print(f"{'='*70}")

    dcs_moderate = all_algo_results.get("dcs_moderate", {})
    if "results" in dcs_moderate:
        results = dcs_moderate["results"]

        # Trace 1: Committed strategy that converged fast
        committed_results = [r for r in results if r[0] == "committed" and r[3] is not None]
        if committed_results:
            # Sort by convergence
            def find_conv(r):
                _, drafted, pack_records, committed_arch, _ = r
                for pick_num, pack, chosen, is_surge in pack_records:
                    if pick_num < 2 or pick_num + 2 >= len(pack_records):
                        continue
                    all_good = True
                    for w in range(3):
                        _, wp, _, _ = pack_records[pick_num + w]
                        sa = sum(1 for c in wp if c.is_sa_for(committed_arch))
                        if sa < 2:
                            all_good = False
                            break
                    if all_good:
                        return pick_num
                return 30
            committed_results.sort(key=find_conv)
            print(format_trace(committed_results[0], "Trace 1: Early Committer"))

        # Trace 2: Power chaser
        power_results = [r for r in results if r[0] == "power-chaser"]
        if power_results:
            # Find one with a committed arch (they get auto-assigned)
            for pr in power_results:
                if pr[3] is not None:
                    print(format_trace(pr, "Trace 2: Power Chaser"))
                    break

        # Trace 3: Signal reader
        signal_results = [r for r in results if r[0] == "signal-reader" and r[3] is not None]
        if signal_results:
            signal_results.sort(key=find_conv)
            print(format_trace(signal_results[len(signal_results)//2],
                               "Trace 3: Signal Reader (median)"))

    # ========== Fitness Degradation Curve ==========
    print(f"\n\n{'='*70}")
    print("FITNESS DEGRADATION CURVE")
    print(f"{'='*70}")

    print(f"\n{'Metric':<25} {'Optimistic':>12} {'Moderate':>12} {'Pessimistic':>12} {'Delta O->P':>12}")
    print("-" * 75)

    for metric_key, label, fmt in [
        ("m1", "M1 Unique Archs Early", ".2f"),
        ("m2", "M2 S/A Early", ".2f"),
        ("m3", "M3 S/A Late", ".2f"),
        ("m4", "M4 Off-Arch Late", ".2f"),
        ("m5", "M5 Convergence", ".1f"),
        ("m6", "M6 Deck Conc %", ".1f"),
        ("m7", "M7 Card Overlap %", ".1f"),
        ("m9", "M9 S/A StdDev", ".2f"),
    ]:
        vals = []
        for fm_name in fitness_models:
            key = f"dcs_{fm_name}"
            if key in all_algo_results:
                vals.append(all_algo_results[key]["metrics"][metric_key])
            else:
                vals.append(float("nan"))
        delta = vals[2] - vals[0] if not (math.isnan(vals[0]) or math.isnan(vals[2])) else float("nan")
        print(f"{label:<25} {vals[0]:>12{fmt}} {vals[1]:>12{fmt}} {vals[2]:>12{fmt}} "
              f"{delta:>+12{fmt}}")

    # Same for Surge Packs baseline
    print(f"\n{'Metric':<25} {'Optimistic':>12} {'Moderate':>12} {'Pessimistic':>12} {'Delta O->P':>12}")
    print("-" * 75)
    print("(Surge Packs Baseline)")
    for metric_key, label, fmt in [
        ("m3", "M3 S/A Late", ".2f"),
        ("m5", "M5 Convergence", ".1f"),
        ("m6", "M6 Deck Conc %", ".1f"),
        ("m9", "M9 S/A StdDev", ".2f"),
    ]:
        vals = []
        for fm_name in fitness_models:
            key = f"sp_{fm_name}"
            if key in all_algo_results:
                vals.append(all_algo_results[key]["metrics"][metric_key])
            else:
                vals.append(float("nan"))
        delta = vals[2] - vals[0] if not (math.isnan(vals[0]) or math.isnan(vals[2])) else float("nan")
        print(f"{label:<25} {vals[0]:>12{fmt}} {vals[1]:>12{fmt}} {vals[2]:>12{fmt}} "
              f"{delta:>+12{fmt}}")

    # ========== Comparison Table ==========
    print(f"\n\n{'='*70}")
    print("DCS vs SURGE PACKS COMPARISON")
    print(f"{'='*70}")

    print(f"\n{'Algorithm + Fitness':<35} {'M3':>6} {'M5':>6} {'M6':>7} {'M9':>6} {'Pass':>6}")
    print("-" * 68)

    for fm_name in fitness_models:
        for algo_prefix, algo_label in [("dcs", "Dual-Counter Surge"),
                                         ("sp", "Surge Packs")]:
            key = f"{algo_prefix}_{fm_name}"
            if key in all_algo_results:
                m = all_algo_results[key]["metrics"]
                pf = all_algo_results[key]["pf"]
                passed = sum(1 for v in pf.values() if v == "PASS")
                print(f"{algo_label+' ('+fm_name+')':35s} "
                      f"{m['m3']:>6.2f} {m['m5']:>6.1f} {m['m6']:>6.1f}% "
                      f"{m['m9']:>6.2f} {passed:>5}/9")

    # ========== Cost filter comparison across fitness models ==========
    print(f"\n\n{'='*70}")
    print("COST FILTER HOME-ARCHETYPE SELECTION RATE")
    print(f"{'='*70}")

    for fm_name in ["moderate", "pessimistic"]:
        key = f"dcs_{fm_name}"
        if key in all_algo_results and "results" in all_algo_results[key]:
            analysis = analyze_cost_filter(all_algo_results[key]["results"])
            total = analysis["total_surge_slots"]
            if total > 0:
                home_pct = analysis["home_archetype"] / total * 100
                sibling_pct = analysis["sibling_archetype"] / total * 100
                print(f"  {fm_name}: home={home_pct:.1f}%, sibling={sibling_pct:.1f}%, "
                      f"home+sibling={home_pct+sibling_pct:.1f}%")

    # For comparison, compute what unfiltered surge would give
    # (should be ~50% home, ~50% sibling for same-primary cards)
    print(f"\n  Expected unfiltered: ~50% home, ~50% sibling (same-resonance pool)")

    print(f"\n\n{'='*70}")
    print("SIMULATION COMPLETE")
    print(f"{'='*70}")


if __name__ == "__main__":
    main()
