"""
V7 Agent 1 — Round 3: Surge Packs V6 Baseline (T=4, S=3) under 3 Fitness Models

Algorithm: "Each drafted symbol adds tokens (+2 primary, +1 others); when any
counter reaches 4, spend 4 and fill 3 of the next pack's 4 slots with random
cards of that resonance, fourth slot random."

Fitness Models:
  A (Optimistic): Cross-archetype sibling = 100% A-tier. S/A precision ~100%.
  B (Moderate):   Cross-archetype sibling = 50%A/30%B/20%C. S/A precision ~75%.
  C (Pessimistic): Cross-archetype sibling = 25%A/40%B/35%C. S/A precision ~62.5%.
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

STRATEGIES = ["committed", "power-chaser", "signal-reader"]

# Fitness model distributions for sibling archetype (shares primary resonance)
FITNESS_MODELS = {
    "A": {"A": 1.00, "B": 0.00, "C": 0.00},  # Optimistic
    "B": {"A": 0.50, "B": 0.30, "C": 0.20},  # Moderate
    "C": {"A": 0.25, "B": 0.40, "C": 0.35},  # Pessimistic
}

TIER_SCORE = {"S": 4.0, "A": 3.0, "B": 2.0, "C": 0.5, "F": 0.0}


# ---------------------------------------------------------------------------
# Card and Fitness
# ---------------------------------------------------------------------------

def build_base_fitness_map():
    """Build the structural relationship map (home=S, sibling=variable,
    secondary-shared=B, distant=C). Returns dict[home_idx][eval_idx] -> category."""
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


@dataclass
class SimCard:
    id: int
    symbols: list
    archetype_idx: int
    power: float
    # Per-archetype fitness tier, rolled at pool creation under a fitness model
    archetype_tiers: dict = field(default_factory=dict)

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
        return self.archetype_tiers.get(archetype_idx, "C")

    def fitness_score_for(self, archetype_idx: int) -> float:
        return TIER_SCORE[self.fitness_for(archetype_idx)]

    def is_sa_for(self, archetype_idx: int) -> bool:
        return self.fitness_for(archetype_idx) in ("S", "A")


def assign_fitness_tiers(card: SimCard, fitness_model: str, rng: random.Random):
    """Assign per-archetype fitness tiers to a card based on the fitness model."""
    model = FITNESS_MODELS[fitness_model]
    card.archetype_tiers = {}

    if card.is_generic:
        for a in range(NUM_ARCHETYPES):
            card.archetype_tiers[a] = "B"
        return

    home = card.archetype_idx
    for eval_idx in range(NUM_ARCHETYPES):
        rel = BASE_FITNESS_MAP[home][eval_idx]
        if rel == "home":
            card.archetype_tiers[eval_idx] = "S"
        elif rel == "sibling":
            r = rng.random()
            if r < model["A"]:
                card.archetype_tiers[eval_idx] = "A"
            elif r < model["A"] + model["B"]:
                card.archetype_tiers[eval_idx] = "B"
            else:
                card.archetype_tiers[eval_idx] = "C"
        elif rel == "secondary":
            card.archetype_tiers[eval_idx] = "B"
        else:
            card.archetype_tiers[eval_idx] = "C"


# ---------------------------------------------------------------------------
# Card Pool Construction
# ---------------------------------------------------------------------------

def build_card_pool(rng: random.Random, fitness_model: str) -> list:
    cards = []
    card_id = 0

    # Generic cards
    for _ in range(NUM_GENERIC):
        c = SimCard(id=card_id, symbols=[], archetype_idx=-1,
                    power=rng.uniform(3.0, 7.0))
        assign_fitness_tiers(c, fitness_model, rng)
        cards.append(c)
        card_id += 1

    # Archetype cards: ~40 per archetype
    # Vary counts slightly to add natural variance
    trimmed = rng.sample(range(NUM_ARCHETYPES), 2)
    boosted = set([i for i in range(NUM_ARCHETYPES) if i not in trimmed][:4])

    for arch_idx, (name, pri, sec) in enumerate(ARCHETYPES):
        is_trimmed = arch_idx in trimmed
        is_boosted = arch_idx in boosted
        n_dual_2 = 3 if is_trimmed else 4
        n_mono_1 = 11 if is_boosted else 10
        n_mono_2 = 18 if is_trimmed else 17

        symbol_configs = (
            [([pri],) for _ in range(n_mono_1)] +
            [([pri, pri],) for _ in range(n_mono_2)] +
            [([pri, pri, pri],) for _ in range(6)] +
            [([pri, sec],) for _ in range(n_dual_2)] +
            [([pri, pri, sec],) for _ in range(3)]
        )

        for (symbols,) in symbol_configs:
            power = rng.uniform(2.0, 9.0)
            c = SimCard(id=card_id, symbols=list(symbols),
                        archetype_idx=arch_idx, power=power)
            assign_fitness_tiers(c, fitness_model, rng)
            cards.append(c)
            card_id += 1

    assert len(cards) == TOTAL_CARDS, f"Expected {TOTAL_CARDS}, got {len(cards)}"
    return cards


def build_resonance_pools(pool: list) -> dict:
    pools = {}
    for r in RESONANCES:
        pools[r] = [c for c in pool if c.primary_resonance == r]
    return pools


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


# ---------------------------------------------------------------------------
# Shared Draft Infrastructure
# ---------------------------------------------------------------------------

def _update_token_counters(card, token_counters):
    for i, sym in enumerate(card.symbols):
        weight = 2 if i == 0 else 1
        token_counters[sym] += weight


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
# Surge Packs Draft Algorithm
# ---------------------------------------------------------------------------

def surge_packs_draft(pool, res_pools, rng, strategy, threshold=4,
                      surge_slots=3):
    """Surge Packs V6 baseline: T=threshold, S=surge_slots."""
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
                              resonance_signals, rng)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        _update_token_counters(chosen, token_counters)
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
# Metrics Computation
# ---------------------------------------------------------------------------

def compute_metrics(all_results: list) -> dict:
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
    metrics["m1"] = statistics.mean(m1_values) if m1_values else 0

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
    metrics["m2"] = statistics.mean(m2_values) if m2_values else 0

    # M3: Picks 6+ S/A for committed archetype per pack
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
    metrics["m3"] = statistics.mean(m3_values) if m3_values else 0
    metrics["m3_values"] = m3_values

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
    metrics["m4"] = statistics.mean(m4_values) if m4_values else 0

    # M5: Convergence pick
    convergence_picks = []
    for strategy, results in by_strategy.items():
        if strategy == "power-chaser":
            continue
        for drafted, pack_records, committed_arch in results:
            if committed_arch is None:
                continue
            found = False
            for pick_num, pack, chosen in pack_records:
                if pick_num < 2 or pick_num + 2 >= len(pack_records):
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
    metrics["m5"] = statistics.mean(convergence_picks) if convergence_picks else 30

    # M6: Deck archetype concentration
    m6_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            if committed_arch is None:
                continue
            sa_cards = sum(1 for c in drafted if c.is_sa_for(committed_arch))
            m6_values.append(sa_cards / len(drafted) * 100)
    metrics["m6"] = statistics.mean(m6_values) if m6_values else 0

    # M7: Run-to-run card overlap
    archetype_runs = defaultdict(list)
    for strategy, drafted, pack_records, committed_arch in all_results:
        if committed_arch is not None:
            archetype_runs[committed_arch].append(set(c.id for c in drafted))

    overlaps = []
    overlap_rng = random.Random(999)
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
    for strategy, drafted, pack_records, committed_arch in all_results:
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

    # M9: StdDev of S/A cards per pack (picks 6+)
    metrics["m9"] = (statistics.stdev(m3_values)
                     if len(m3_values) > 1 else 0)

    return metrics


def evaluate_pass_fail(metrics):
    pf = {}
    pf["M1"] = "PASS" if metrics["m1"] >= 3 else "FAIL"
    pf["M2"] = "PASS" if metrics["m2"] <= 2 else "FAIL"
    pf["M3"] = "PASS" if metrics["m3"] >= 2.0 else "FAIL"
    pf["M4"] = "PASS" if metrics["m4"] >= 0.5 else "FAIL"
    pf["M5"] = "PASS" if 5 <= metrics["m5"] <= 8 else "FAIL"
    pf["M6"] = "PASS" if 60 <= metrics["m6"] <= 90 else "FAIL"
    pf["M7"] = "PASS" if metrics["m7"] < 40 else "FAIL"
    pf["M8"] = "PASS" if (metrics["m8_min"] >= 5 and
                           metrics["m8_max"] <= 20) else "FAIL"
    pf["M9"] = "PASS" if metrics["m9"] >= 0.8 else "FAIL"
    return pf


# ---------------------------------------------------------------------------
# Per-Archetype Convergence
# ---------------------------------------------------------------------------

def per_archetype_convergence(all_results: list) -> dict:
    arch_conv = defaultdict(list)
    for strategy, drafted, pack_records, committed_arch in all_results:
        if strategy == "power-chaser" or committed_arch is None:
            continue
        for pick_num, pack, chosen in pack_records:
            if pick_num < 2 or pick_num + 2 >= len(pack_records):
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
        avg = statistics.mean(picks) if picks else float('nan')
        result[ARCHETYPE_NAMES[a]] = (avg, len(picks))
    return result


# ---------------------------------------------------------------------------
# Draft Trace
# ---------------------------------------------------------------------------

def format_draft_trace(pack_records, committed_arch, strategy, draft_idx):
    lines = []
    lines.append(f"\n=== Draft Trace #{draft_idx} (strategy={strategy}, "
                 f"committed={ARCHETYPE_NAMES[committed_arch] if committed_arch is not None else 'None'}) ===")
    for pick_num, pack, chosen in pack_records[:15]:
        pack_desc = []
        for c in pack:
            tier = c.fitness_for(committed_arch) if committed_arch is not None else "?"
            res = c.primary_resonance or "Gen"
            arch = ARCHETYPE_NAMES[c.archetype_idx] if c.archetype_idx >= 0 else "Generic"
            pack_desc.append(f"{res[:2]}/{arch[:4]}({tier})")
        chosen_tier = chosen.fitness_for(committed_arch) if committed_arch is not None else "?"
        chosen_res = chosen.primary_resonance or "Gen"
        chosen_arch = ARCHETYPE_NAMES[chosen.archetype_idx] if chosen.archetype_idx >= 0 else "Generic"
        marker = " [SURGE]" if pick_num > 0 and len(pack) == PACK_SIZE else ""
        lines.append(f"  Pick {pick_num:2d}{marker}: [{', '.join(pack_desc)}] "
                     f"-> {chosen_res[:2]}/{chosen_arch[:4]}({chosen_tier})")
    lines.append(f"  ... (picks 15-29 omitted)")
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Parameter Sensitivity Sweep
# ---------------------------------------------------------------------------

def run_sensitivity_sweep(pool, res_pools, rng):
    """Sweep T=3,4,5 x S=2,3,4 for all three fitness models."""
    results = {}
    for model in ["A", "B", "C"]:
        # Rebuild pool with this fitness model
        sweep_rng = random.Random(42)
        sweep_pool = build_card_pool(sweep_rng, model)
        sweep_res_pools = build_resonance_pools(sweep_pool)

        for T in [3, 4, 5]:
            for S in [2, 3, 4]:
                key = (model, T, S)
                all_results = []
                draft_rng = random.Random(100 + T * 10 + S)
                for run_idx in range(NUM_DRAFTS):
                    strategy = STRATEGIES[run_idx % len(STRATEGIES)]
                    drafted, pack_records, committed_arch = surge_packs_draft(
                        sweep_pool, sweep_res_pools, draft_rng, strategy,
                        threshold=T, surge_slots=S)
                    all_results.append((strategy, drafted, pack_records,
                                        committed_arch))
                metrics = compute_metrics(all_results)
                pf = evaluate_pass_fail(metrics)
                passed = sum(1 for v in pf.values() if v == "PASS")
                results[key] = {
                    "m3": metrics["m3"],
                    "m5": metrics["m5"],
                    "m6": metrics["m6"],
                    "m7": metrics["m7"],
                    "m9": metrics["m9"],
                    "passed": passed,
                    "pf": pf,
                }
    return results


# ---------------------------------------------------------------------------
# Main Simulation
# ---------------------------------------------------------------------------

def main():
    print("=" * 70)
    print("V7 Agent 1: Surge Packs V6 Baseline (T=4, S=3)")
    print("=" * 70)

    all_model_metrics = {}
    all_model_pf = {}
    all_model_conv = {}
    all_model_traces = {}

    for model in ["A", "B", "C"]:
        model_name = {"A": "Optimistic", "B": "Moderate", "C": "Pessimistic"}[model]
        print(f"\n{'='*70}")
        print(f"Fitness Model {model}: {model_name}")
        print(f"{'='*70}")

        rng = random.Random(42)
        pool = build_card_pool(rng, model)
        res_pools = build_resonance_pools(pool)

        dual_count = sum(1 for c in pool if len(c.resonance_types) >= 2)
        print(f"Pool: {len(pool)} cards, {dual_count} dual-type "
              f"({dual_count/len(pool)*100:.1f}%)")
        for r in RESONANCES:
            print(f"  {r} primary: {len(res_pools[r])} cards")

        # Verify S/A precision for resonance-matched cards
        sa_counts = {r: 0 for r in RESONANCES}
        total_counts = {r: 0 for r in RESONANCES}
        for a in range(NUM_ARCHETYPES):
            pri = ARCHETYPES[a][1]
            for c in res_pools[pri]:
                total_counts[pri] += 1
                if c.is_sa_for(a):
                    sa_counts[pri] += 1
        overall_sa = sum(sa_counts.values())
        overall_total = sum(total_counts.values())
        print(f"Resonance-matched S/A precision: "
              f"{overall_sa/overall_total*100:.1f}% "
              f"({overall_sa}/{overall_total})")

        # Run drafts
        draft_rng = random.Random(123)
        all_results = []
        trace_results = []

        for run_idx in range(NUM_DRAFTS):
            strategy = STRATEGIES[run_idx % len(STRATEGIES)]
            drafted, pack_records, committed_arch = surge_packs_draft(
                pool, res_pools, draft_rng, strategy)
            all_results.append((strategy, drafted, pack_records, committed_arch))

            # Save traces for specific runs
            if run_idx in [0, 1, 2]:
                trace_results.append((strategy, drafted, pack_records,
                                      committed_arch))

        # Compute metrics
        metrics = compute_metrics(all_results)
        pf = evaluate_pass_fail(metrics)
        arch_conv = per_archetype_convergence(all_results)

        all_model_metrics[model] = metrics
        all_model_pf[model] = pf
        all_model_conv[model] = arch_conv
        all_model_traces[model] = trace_results

        # Print results
        passed = sum(1 for v in pf.values() if v == "PASS")
        print(f"\n--- Surge Packs V6 (T=4/S=3), Model {model} "
              f"({passed}/9 PASS) ---")
        print(f"  M1 Unique archs early:    {metrics['m1']:.2f}  "
              f"(>= 3)    [{pf['M1']}]")
        print(f"  M2 S/A early:             {metrics['m2']:.2f}  "
              f"(<= 2)    [{pf['M2']}]")
        print(f"  M3 S/A committed late:    {metrics['m3']:.2f}  "
              f"(>= 2.0)  [{pf['M3']}]")
        print(f"  M4 Off-arch late:         {metrics['m4']:.2f}  "
              f"(>= 0.5)  [{pf['M4']}]")
        print(f"  M5 Convergence pick:      {metrics['m5']:.1f}  "
              f"(5-8)     [{pf['M5']}]")
        print(f"  M6 Deck concentration:    {metrics['m6']:.1f}%  "
              f"(60-90%)  [{pf['M6']}]")
        print(f"  M7 Card overlap:          {metrics['m7']:.1f}%  "
              f"(< 40%)   [{pf['M7']}]")
        print(f"  M8 Arch freq range:       {metrics['m8_min']:.1f}%-"
              f"{metrics['m8_max']:.1f}%  (5-20%)   [{pf['M8']}]")
        print(f"  M9 S/A stddev:            {metrics['m9']:.2f}  "
              f"(>= 0.8)  [{pf['M9']}]")

        # Archetype frequency detail
        print(f"\n  Archetype Frequencies:")
        for name, freq in metrics["m8_freq"].items():
            print(f"    {name:<15} {freq:.1f}%")

        # Per-archetype convergence
        print(f"\n  Per-Archetype Convergence:")
        print(f"    {'Archetype':<15} {'Avg Pick':>10} {'N Runs':>8}")
        for name, (avg, n) in arch_conv.items():
            avg_str = f"{avg:.1f}" if not math.isnan(avg) else "N/A"
            print(f"    {name:<15} {avg_str:>10} {n:>8}")

        # Draft traces
        if model == "B":
            print("\n  Draft Traces (Model B, Moderate):")
            for i, (strat, drafted, pr, ca) in enumerate(trace_results):
                print(format_draft_trace(pr, ca, strat, i + 1))

    # ===== Fitness Degradation Curve =====
    print(f"\n\n{'='*70}")
    print("FITNESS DEGRADATION CURVE: A -> B -> C")
    print(f"{'='*70}")
    print(f"{'Metric':<25} {'Model A':>10} {'Model B':>10} {'Model C':>10} "
          f"{'A->B delta':>12} {'B->C delta':>12}")
    print("-" * 80)

    metric_keys = [
        ("M1 Unique archs", "m1"),
        ("M2 S/A early", "m2"),
        ("M3 S/A late", "m3"),
        ("M4 Off-arch", "m4"),
        ("M5 Convergence", "m5"),
        ("M6 Concentration", "m6"),
        ("M7 Overlap", "m7"),
        ("M9 S/A stddev", "m9"),
    ]
    for label, key in metric_keys:
        va = all_model_metrics["A"][key]
        vb = all_model_metrics["B"][key]
        vc = all_model_metrics["C"][key]
        d_ab = vb - va
        d_bc = vc - vb
        print(f"  {label:<23} {va:>10.2f} {vb:>10.2f} {vc:>10.2f} "
              f"{d_ab:>+12.2f} {d_bc:>+12.2f}")

    # Pass/fail summary
    print(f"\n{'Metric':<25} {'Model A':>10} {'Model B':>10} {'Model C':>10}")
    print("-" * 60)
    for m in ["M1", "M2", "M3", "M4", "M5", "M6", "M7", "M8", "M9"]:
        print(f"  {m:<23} {all_model_pf['A'][m]:>10} "
              f"{all_model_pf['B'][m]:>10} {all_model_pf['C'][m]:>10}")
    for model in ["A", "B", "C"]:
        passed = sum(1 for v in all_model_pf[model].values() if v == "PASS")
        print(f"  Model {model} total: {passed}/9 PASS")

    # ===== Parameter Sensitivity Sweep =====
    print(f"\n\n{'='*70}")
    print("PARAMETER SENSITIVITY SWEEP: T x S x Fitness Model")
    print(f"{'='*70}")

    sensitivity = run_sensitivity_sweep(None, None, None)

    print(f"\n{'Model':<8} {'T':>3} {'S':>3} {'M3':>6} {'M5':>6} "
          f"{'M6':>6} {'M7':>6} {'M9':>6} {'Pass':>5}")
    print("-" * 55)
    for model in ["A", "B", "C"]:
        for T in [3, 4, 5]:
            for S in [2, 3, 4]:
                r = sensitivity[(model, T, S)]
                print(f"  {model:<6} {T:>3} {S:>3} {r['m3']:>6.2f} "
                      f"{r['m5']:>6.1f} {r['m6']:>5.1f}% "
                      f"{r['m7']:>5.1f}% {r['m9']:>6.2f} {r['passed']:>4}/9")


if __name__ == "__main__":
    main()
