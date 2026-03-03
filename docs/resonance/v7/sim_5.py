"""
V7 Agent 5: Archetype Compass Packs Simulation
================================================
Algorithm: "Each pack (from pick 2) contains one card from the player's top
resonance pool, one from an adjacent resonance on the archetype circle
(alternating between the two neighbors each pick), and two random cards."

Three fitness models (Optimistic A, Moderate B, Pessimistic C).
1000 drafts x 30 picks x 3 strategies.
Parameter sensitivity: structure (1+1+2 vs 2+1+1), activation pick (2, 3, 4).
Aspiration Packs baseline for M4/M8 comparison.
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
    ("Flash",        "Zephyr", "Ember"),   # 0
    ("Blink",        "Ember",  "Zephyr"),  # 1
    ("Storm",        "Ember",  "Stone"),   # 2
    ("Self-Discard", "Stone",  "Ember"),   # 3
    ("Self-Mill",    "Stone",  "Tide"),    # 4
    ("Sacrifice",    "Tide",   "Stone"),   # 5
    ("Warriors",     "Tide",   "Zephyr"),  # 6
    ("Ramp",         "Zephyr", "Tide"),    # 7
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]

# Neighbor map on the archetype circle.
# Each resonance's neighbors are the resonances that share an archetype with it.
# Ember neighbors: Zephyr (Flash/Blink share E+Z) and Stone (Storm/Self-Discard share E+S)
# Stone neighbors: Ember (Storm/Self-Discard) and Tide (Self-Mill/Sacrifice share S+T)
# Tide neighbors: Stone (Self-Mill/Sacrifice) and Zephyr (Warriors/Ramp share T+Z)
# Zephyr neighbors: Tide (Warriors/Ramp) and Ember (Flash/Blink)
RESONANCE_NEIGHBORS = {
    "Ember":  ("Zephyr", "Stone"),
    "Stone":  ("Ember",  "Tide"),
    "Tide":   ("Stone",  "Zephyr"),
    "Zephyr": ("Tide",   "Ember"),
}

TIER_SCORE = {"S": 4.0, "A": 3.0, "B": 2.0, "C": 0.5, "F": 0.0}

# ---------------------------------------------------------------------------
# Fitness Models
# ---------------------------------------------------------------------------

def build_fitness_map_optimistic():
    """Model A: Cross-archetype sharing primary resonance = always A-tier."""
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


def build_fitness_map_realistic(rng, a_rate, b_rate, c_rate):
    """
    Models B/C: Cross-archetype sharing primary resonance = rolled per card.
    Returns a function that takes (card_id, home_arch, eval_arch) -> tier.
    The roll is per-card, assigned once during pool construction.
    """
    base_fm = {}
    for home_idx, (_, home_pri, home_sec) in enumerate(ARCHETYPES):
        base_fm[home_idx] = {}
        for eval_idx, (_, eval_pri, eval_sec) in enumerate(ARCHETYPES):
            if eval_idx == home_idx:
                base_fm[home_idx][eval_idx] = "S"
            elif eval_pri == home_pri:
                base_fm[home_idx][eval_idx] = "ROLL"  # to be assigned per card
            elif eval_sec == home_pri or eval_pri == home_sec:
                base_fm[home_idx][eval_idx] = "B"
            else:
                base_fm[home_idx][eval_idx] = "C"
    return base_fm, (a_rate, b_rate, c_rate)


@dataclass
class SimCard:
    id: int
    symbols: list
    archetype_idx: int
    power: float
    # Per-card fitness rolls for realistic models (card_id -> {eval_arch -> tier})
    realistic_fitness_b: dict = field(default_factory=dict)
    realistic_fitness_c: dict = field(default_factory=dict)

    @property
    def primary_resonance(self) -> Optional[str]:
        return self.symbols[0] if self.symbols else None

    @property
    def is_generic(self) -> bool:
        return self.archetype_idx == -1

    def fitness_for(self, archetype_idx: int, model: str) -> str:
        if self.is_generic:
            return "B"
        if model == "A":
            return OPTIMISTIC_FM[self.archetype_idx][archetype_idx]
        elif model == "B":
            return self.realistic_fitness_b.get(archetype_idx,
                        OPTIMISTIC_FM[self.archetype_idx][archetype_idx])
        elif model == "C":
            return self.realistic_fitness_c.get(archetype_idx,
                        OPTIMISTIC_FM[self.archetype_idx][archetype_idx])
        return "C"

    def is_sa_for(self, archetype_idx: int, model: str) -> bool:
        return self.fitness_for(archetype_idx, model) in ("S", "A")

    def fitness_score_for(self, archetype_idx: int, model: str) -> float:
        return TIER_SCORE[self.fitness_for(archetype_idx, model)]


# Build global optimistic map
OPTIMISTIC_FM = build_fitness_map_optimistic()


# ---------------------------------------------------------------------------
# Card Pool Construction
# ---------------------------------------------------------------------------

def _roll_tier(rng, rates):
    """Roll a tier given (a_rate, b_rate, c_rate)."""
    r = rng.random()
    if r < rates[0]:
        return "A"
    elif r < rates[0] + rates[1]:
        return "B"
    else:
        return "C"


def build_card_pool(rng: random.Random) -> list:
    cards = []
    card_id = 0

    # Generic cards
    for _ in range(NUM_GENERIC):
        cards.append(SimCard(
            id=card_id, symbols=[], archetype_idx=-1,
            power=rng.uniform(3.0, 7.0),
        ))
        card_id += 1

    # Distribute archetype cards: ~40 per archetype
    trimmed = rng.sample(range(NUM_ARCHETYPES), 2)
    non_trimmed = [i for i in range(NUM_ARCHETYPES) if i not in trimmed]
    boosted = set(non_trimmed[:4])

    b_rates = (0.50, 0.30, 0.20)  # Moderate
    c_rates = (0.25, 0.40, 0.35)  # Pessimistic

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
            card = SimCard(
                id=card_id, symbols=list(symbols),
                archetype_idx=arch_idx, power=power,
            )

            # Assign realistic fitness rolls for sibling archetypes
            for eval_idx in range(NUM_ARCHETYPES):
                if OPTIMISTIC_FM[arch_idx][eval_idx] == "A":
                    card.realistic_fitness_b[eval_idx] = _roll_tier(rng, b_rates)
                    card.realistic_fitness_c[eval_idx] = _roll_tier(rng, c_rates)

            cards.append(card)
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

def _best_archetype_from_drafted(drafted, model):
    scores = [0.0] * NUM_ARCHETYPES
    for card in drafted:
        for a in range(NUM_ARCHETYPES):
            scores[a] += card.fitness_score_for(a, model)
    return scores.index(max(scores))


def _signal_read_commit(drafted, resonance_signals):
    sorted_res = sorted(RESONANCES, key=lambda r: resonance_signals[r], reverse=True)
    top_pri = sorted_res[0]
    top_sec = sorted_res[1]
    for idx, (name, pri, sec) in enumerate(ARCHETYPES):
        if pri == top_pri and sec == top_sec:
            return idx
    for idx, (name, pri, sec) in enumerate(ARCHETYPES):
        if pri == top_pri:
            return idx
    return 0


def _choose_card(pack, strategy, committed_archetype, pick_num,
                 resonance_signals, rng, model):
    if strategy == "committed":
        return _committed_choose(pack, committed_archetype, pick_num, rng, model)
    elif strategy == "power-chaser":
        return _power_chaser_choose(pack, rng)
    elif strategy == "signal-reader":
        return _signal_reader_choose(pack, committed_archetype, pick_num,
                                     resonance_signals, rng, model)
    raise ValueError(f"Unknown strategy: {strategy}")


def _committed_choose(pack, committed_archetype, pick_num, rng, model):
    if committed_archetype is None:
        symbol_cards = [c for c in pack if c.symbols]
        if symbol_cards:
            return max(symbol_cards, key=lambda c: c.power)
        return max(pack, key=lambda c: c.power)
    else:
        return max(pack, key=lambda c: (c.fitness_score_for(committed_archetype, model),
                                         c.power))


def _power_chaser_choose(pack, rng):
    return max(pack, key=lambda c: c.power)


def _signal_reader_choose(pack, committed_archetype, pick_num,
                           resonance_signals, rng, model):
    if committed_archetype is None:
        def score(c):
            if not c.symbols:
                return (0, c.power)
            pri_signal = resonance_signals.get(c.primary_resonance, 0)
            return (pri_signal, c.power)
        return max(pack, key=score)
    else:
        return max(pack, key=lambda c: (c.fitness_score_for(committed_archetype, model),
                                         c.power))


def _update_resonance_counters(card, res_counters):
    for i, sym in enumerate(card.symbols):
        weight = 2 if i == 0 else 1
        res_counters[sym] += weight


def _get_top_resonance(res_counters):
    return max(RESONANCES, key=lambda r: res_counters[r])


def _get_top_two_resonances(res_counters):
    sorted_r = sorted(RESONANCES, key=lambda r: res_counters[r], reverse=True)
    return sorted_r[0], sorted_r[1]


def _handle_commitment(strategy, committed_archetype, commit_pick, pick_num,
                       drafted, resonance_signals, model):
    if committed_archetype is None and pick_num >= 4:
        if strategy == "committed":
            committed_archetype = _best_archetype_from_drafted(drafted, model)
            commit_pick = pick_num
        elif strategy == "signal-reader":
            committed_archetype = _signal_read_commit(drafted, resonance_signals)
            commit_pick = pick_num
    return committed_archetype, commit_pick


# ---------------------------------------------------------------------------
# Algorithm: Archetype Compass Packs
# ---------------------------------------------------------------------------

def compass_packs_draft(pool, res_pools, rng, strategy, model,
                        activation_pick=2, r1_slots=1, r2_slots=1):
    """
    Compass Packs: From activation_pick onward, each pack has:
      r1_slots cards from top resonance pool,
      r2_slots cards from the rotating neighbor resonance pool,
      remaining slots random.
    Neighbor alternates each pick: odd picks use neighbor A, even use neighbor B.
    """
    res_counters = {r: 0 for r in RESONANCES}
    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}
    # Track which slot is R2 for neighbor-slot analysis
    r2_slot_cards = []  # list of (card, committed_archetype_at_time)

    for pick_num in range(NUM_PICKS):
        top_res = _get_top_resonance(res_counters)

        if pick_num >= activation_pick and res_counters[top_res] > 0:
            # Determine neighbor: alternate between the two
            neighbor_a, neighbor_b = RESONANCE_NEIGHBORS[top_res]
            if pick_num % 2 == 0:
                neighbor = neighbor_a
            else:
                neighbor = neighbor_b

            pack = []
            # Fill R1 slots
            for _ in range(r1_slots):
                pack.append(rng.choice(res_pools[top_res]))
            # Fill R2 (neighbor) slots
            r2_cards_this_pack = []
            for _ in range(r2_slots):
                card = rng.choice(res_pools[neighbor])
                pack.append(card)
                r2_cards_this_pack.append(card)
            # Fill remaining random slots
            remaining = PACK_SIZE - r1_slots - r2_slots
            for _ in range(remaining):
                pack.append(rng.choice(pool))

            # Track R2 cards for analysis
            for c in r2_cards_this_pack:
                r2_slot_cards.append((c, committed_archetype))
        else:
            # Fully random pack
            pack = [rng.choice(pool) for _ in range(PACK_SIZE)]

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              resonance_signals, rng, model)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        _update_resonance_counters(chosen, res_counters)
        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals, model)

    return drafted, pack_records, committed_archetype, r2_slot_cards


# ---------------------------------------------------------------------------
# Algorithm: Aspiration Packs (for comparison)
# ---------------------------------------------------------------------------

def aspiration_packs_draft(pool, res_pools, rng, strategy, model,
                            r2_min=3, r2_ratio=0.50):
    """
    Aspiration Packs: If R2 >= r2_min AND R2 >= r2_ratio * R1, next pack
    has 1 R1 card + 1 R2 card + 2 random. Otherwise all 4 random.
    """
    res_counters = {r: 0 for r in RESONANCES}
    drafted = []
    pack_records = []
    committed_archetype = None
    commit_pick = None
    resonance_signals = {r: 0 for r in RESONANCES}

    for pick_num in range(NUM_PICKS):
        r1, r2 = _get_top_two_resonances(res_counters)
        r1_val = res_counters[r1]
        r2_val = res_counters[r2]

        if r2_val >= r2_min and r1_val > 0 and r2_val >= r2_ratio * r1_val:
            # Aspiration pack
            pack = [
                rng.choice(res_pools[r1]),
                rng.choice(res_pools[r2]),
                rng.choice(pool),
                rng.choice(pool),
            ]
        else:
            pack = [rng.choice(pool) for _ in range(PACK_SIZE)]

        chosen = _choose_card(pack, strategy, committed_archetype, pick_num,
                              resonance_signals, rng, model)
        pack_records.append((pick_num, list(pack), chosen))
        drafted.append(chosen)

        _update_resonance_counters(chosen, res_counters)
        if chosen.symbols:
            for sym in chosen.symbols:
                resonance_signals[sym] += 1

        committed_archetype, commit_pick = _handle_commitment(
            strategy, committed_archetype, commit_pick, pick_num,
            drafted, resonance_signals, model)

    return drafted, pack_records, committed_archetype, []


# ---------------------------------------------------------------------------
# Metrics Computation
# ---------------------------------------------------------------------------

def compute_metrics(all_results: list, model: str) -> dict:
    metrics = {}
    by_strategy = defaultdict(list)
    for strategy, drafted, pack_records, committed_arch, _ in all_results:
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
                        if card.is_sa_for(a, model):
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
                               if c.is_sa_for(committed_arch, model))
                m2_values.append(sa_count)
    metrics["m2"] = statistics.mean(m2_values) if m2_values else 0

    # M3: Picks 6+ S/A cards for committed archetype per pack
    m3_values = []
    for strategy, results in by_strategy.items():
        for drafted, pack_records, committed_arch in results:
            if committed_arch is None:
                continue
            for pick_num, pack, chosen in pack_records:
                if pick_num < 5:
                    continue
                sa_count = sum(1 for c in pack if c.is_sa_for(committed_arch, model))
                m3_values.append(sa_count)
    metrics["m3"] = statistics.mean(m3_values) if m3_values else 0

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
                               if c.fitness_for(committed_arch, model) in ("C", "F"))
                m4_values.append(cf_count)
    metrics["m4"] = statistics.mean(m4_values) if m4_values else 0

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
                    sa = sum(1 for c in wp if c.is_sa_for(committed_arch, model))
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
            sa_cards = sum(1 for c in drafted if c.is_sa_for(committed_arch, model))
            m6_values.append(sa_cards / len(drafted) * 100)
    metrics["m6"] = statistics.mean(m6_values) if m6_values else 0

    # M7: Run-to-run card overlap
    archetype_runs = defaultdict(list)
    for strategy, drafted, pack_records, committed_arch, _ in all_results:
        if committed_arch is not None:
            archetype_runs[committed_arch].append(set(c.id for c in drafted))

    overlaps = []
    olap_rng = random.Random(999)
    for arch, run_sets in archetype_runs.items():
        if len(run_sets) < 2:
            continue
        n_pairs = min(200, len(run_sets) * (len(run_sets) - 1) // 2)
        for _ in range(n_pairs):
            i, j = olap_rng.sample(range(len(run_sets)), 2)
            s1, s2 = run_sets[i], run_sets[j]
            if len(s1) == 0 or len(s2) == 0:
                continue
            overlap = len(s1 & s2) / min(len(s1), len(s2)) * 100
            overlaps.append(overlap)
    metrics["m7"] = statistics.mean(overlaps) if overlaps else 0

    # M8: Archetype frequency
    arch_counts = Counter()
    total_committed = 0
    for strategy, drafted, pack_records, committed_arch, _ in all_results:
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
    metrics["m9"] = statistics.stdev(m3_values) if len(m3_values) > 1 else 0

    return metrics


def per_archetype_convergence(all_results: list, model: str) -> dict:
    arch_conv = defaultdict(list)
    for strategy, drafted, pack_records, committed_arch, _ in all_results:
        if strategy == "power-chaser" or committed_arch is None:
            continue
        for pick_num, pack, chosen in pack_records:
            if pick_num < 2 or pick_num + 2 >= len(pack_records):
                continue
            all_good = True
            for w in range(3):
                _, wp, _ = pack_records[pick_num + w]
                sa = sum(1 for c in wp if c.is_sa_for(committed_arch, model))
                if sa < 2:
                    all_good = False
                    break
            if all_good:
                arch_conv[committed_arch].append(pick_num)
                break

    result = {}
    for a in range(NUM_ARCHETYPES):
        picks = arch_conv.get(a, [])
        result[ARCHETYPE_NAMES[a]] = (statistics.mean(picks) if picks else float('nan'),
                                       len(picks))
    return result


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
# R2 Slot Analysis
# ---------------------------------------------------------------------------

def analyze_r2_slots(all_results: list, model: str) -> dict:
    """Analyze the tier distribution of cards in the R2 (neighbor) slot."""
    tier_counts = {"S": 0, "A": 0, "B": 0, "C": 0}
    total = 0
    for strategy, drafted, pack_records, committed_arch, r2_cards in all_results:
        if committed_arch is None:
            continue
        for card, arch_at_time in r2_cards:
            arch = committed_arch  # use final committed archetype
            tier = card.fitness_for(arch, model)
            tier_counts[tier] = tier_counts.get(tier, 0) + 1
            total += 1

    if total == 0:
        return {"S%": 0, "A%": 0, "B%": 0, "C%": 0, "SA%": 0, "total": 0}

    result = {}
    for t in ["S", "A", "B", "C"]:
        result[f"{t}%"] = tier_counts[t] / total * 100
    result["SA%"] = (tier_counts["S"] + tier_counts["A"]) / total * 100
    result["total"] = total
    return result


# ---------------------------------------------------------------------------
# Draft Traces
# ---------------------------------------------------------------------------

def print_draft_trace(drafted, pack_records, committed_arch, model, label):
    print(f"\n  === Draft Trace: {label} ===")
    print(f"  Committed archetype: {ARCHETYPE_NAMES[committed_arch] if committed_arch is not None else 'None'}")
    for pick_num, pack, chosen in pack_records[:15]:
        sa_count = 0
        if committed_arch is not None:
            sa_count = sum(1 for c in pack if c.is_sa_for(committed_arch, model))
        chosen_tier = chosen.fitness_for(committed_arch, model) if committed_arch is not None else "?"
        chosen_res = chosen.primary_resonance or "generic"
        pack_res = [c.primary_resonance or "gen" for c in pack]
        print(f"    Pick {pick_num+1:2d}: pack=[{','.join(pack_res)}] "
              f"chose={chosen_res}({chosen_tier}) S/A={sa_count}")


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

def run_algorithm(draft_fn, pool, res_pools, rng, strategies, model, **kwargs):
    all_results = []
    for run_idx in range(NUM_DRAFTS):
        strategy = strategies[run_idx % len(strategies)]
        drafted, pack_records, committed_arch, r2_cards = draft_fn(
            pool, res_pools, rng, strategy, model, **kwargs)
        all_results.append((strategy, drafted, pack_records, committed_arch, r2_cards))
    return all_results


def main():
    rng = random.Random(42)
    pool = build_card_pool(rng)
    res_pools = build_resonance_pools(pool)

    dual_count = sum(1 for c in pool if len(set(c.symbols)) >= 2)
    print(f"Pool: {len(pool)} cards, {dual_count} dual-type ({dual_count/len(pool)*100:.1f}%)")
    for r in RESONANCES:
        print(f"  {r} primary: {len(res_pools[r])} cards")

    strategies = ["committed", "power-chaser", "signal-reader"]
    models = ["A", "B", "C"]
    model_names = {"A": "Optimistic", "B": "Moderate", "C": "Pessimistic"}

    # ====================================================================
    # 1. Primary algorithm: Compass Packs (1+1+2, activation=2)
    # ====================================================================
    print(f"\n{'='*70}")
    print("PRIMARY: Compass Packs (1R1 + 1R2-neighbor + 2 random, from pick 2)")
    print(f"{'='*70}")

    compass_metrics = {}
    compass_pf = {}
    compass_r2 = {}
    compass_results = {}

    for model in models:
        print(f"\n  ---- Fitness Model {model} ({model_names[model]}) ----")
        results = run_algorithm(compass_packs_draft, pool, res_pools,
                                random.Random(42), strategies, model,
                                activation_pick=2, r1_slots=1, r2_slots=1)
        compass_results[model] = results
        metrics = compute_metrics(results, model)
        pf = evaluate_pass_fail(metrics)
        r2_analysis = analyze_r2_slots(results, model)

        compass_metrics[model] = metrics
        compass_pf[model] = pf
        compass_r2[model] = r2_analysis

        print_metrics(metrics, f"Compass ({model})", pf)
        print(f"\n  R2 Neighbor Slot Breakdown:")
        print(f"    S={r2_analysis['S%']:.1f}%, A={r2_analysis['A%']:.1f}%, "
              f"B={r2_analysis['B%']:.1f}%, C={r2_analysis['C%']:.1f}% "
              f"| S/A={r2_analysis['SA%']:.1f}% (n={r2_analysis['total']})")

    # Per-archetype convergence for primary model
    print(f"\n  Per-Archetype Convergence (Moderate B):")
    arch_conv = per_archetype_convergence(compass_results["B"], "B")
    print(f"    {'Archetype':<15} {'Avg Pick':>10} {'N Runs':>8}")
    for name, (avg, n) in arch_conv.items():
        avg_str = f"{avg:.1f}" if not math.isnan(avg) else "N/A"
        print(f"    {name:<15} {avg_str:>10} {n:>8}")

    # Draft traces (model B)
    print(f"\n  Draft Traces (Moderate B):")
    b_results = compass_results["B"]
    # Find a committed, signal-reader, and power-chaser trace
    for target_strategy in ["committed", "signal-reader", "power-chaser"]:
        for strategy, drafted, pack_records, committed_arch, r2_cards in b_results:
            if strategy == target_strategy and committed_arch is not None:
                print_draft_trace(drafted, pack_records, committed_arch, "B",
                                  target_strategy)
                break

    # ====================================================================
    # 2. Parameter sensitivity: Structure 2+1+1
    # ====================================================================
    print(f"\n{'='*70}")
    print("VARIANT: Compass 2+1+1 (2R1 + 1R2-neighbor + 1 random, from pick 2)")
    print(f"{'='*70}")

    compass_211_metrics = {}
    compass_211_pf = {}

    for model in models:
        results = run_algorithm(compass_packs_draft, pool, res_pools,
                                random.Random(42), strategies, model,
                                activation_pick=2, r1_slots=2, r2_slots=1)
        metrics = compute_metrics(results, model)
        pf = evaluate_pass_fail(metrics)
        compass_211_metrics[model] = metrics
        compass_211_pf[model] = pf
        print_metrics(metrics, f"Compass 2+1+1 ({model})", pf)

    # ====================================================================
    # 3. Parameter sensitivity: Activation pick 3
    # ====================================================================
    print(f"\n{'='*70}")
    print("VARIANT: Compass 1+1+2, activation pick 3")
    print(f"{'='*70}")

    compass_p3_metrics = {}
    compass_p3_pf = {}

    for model in models:
        results = run_algorithm(compass_packs_draft, pool, res_pools,
                                random.Random(42), strategies, model,
                                activation_pick=3, r1_slots=1, r2_slots=1)
        metrics = compute_metrics(results, model)
        pf = evaluate_pass_fail(metrics)
        compass_p3_metrics[model] = metrics
        compass_p3_pf[model] = pf
        print_metrics(metrics, f"Compass act=3 ({model})", pf)

    # ====================================================================
    # 4. Parameter sensitivity: Activation pick 4
    # ====================================================================
    print(f"\n{'='*70}")
    print("VARIANT: Compass 1+1+2, activation pick 4")
    print(f"{'='*70}")

    compass_p4_metrics = {}
    compass_p4_pf = {}

    for model in models:
        results = run_algorithm(compass_packs_draft, pool, res_pools,
                                random.Random(42), strategies, model,
                                activation_pick=4, r1_slots=1, r2_slots=1)
        metrics = compute_metrics(results, model)
        pf = evaluate_pass_fail(metrics)
        compass_p4_metrics[model] = metrics
        compass_p4_pf[model] = pf
        print_metrics(metrics, f"Compass act=4 ({model})", pf)

    # ====================================================================
    # 5. Comparison: Aspiration Packs (Agent 7)
    # ====================================================================
    print(f"\n{'='*70}")
    print("COMPARISON: Aspiration Packs (1R1 + 1R2 + 2 random, gate R2>=3, >=50%)")
    print(f"{'='*70}")

    aspiration_metrics = {}
    aspiration_pf = {}

    for model in models:
        results = run_algorithm(aspiration_packs_draft, pool, res_pools,
                                random.Random(42), strategies, model,
                                r2_min=3, r2_ratio=0.50)
        metrics = compute_metrics(results, model)
        pf = evaluate_pass_fail(metrics)
        aspiration_metrics[model] = metrics
        aspiration_pf[model] = pf
        print_metrics(metrics, f"Aspiration ({model})", pf)

    # ====================================================================
    # 6. Summary tables
    # ====================================================================
    print(f"\n\n{'='*90}")
    print("COMPARATIVE SUMMARY")
    print(f"{'='*90}")

    all_algos = [
        ("Compass 1+1+2 act=2", compass_metrics, compass_pf),
        ("Compass 2+1+1 act=2", compass_211_metrics, compass_211_pf),
        ("Compass 1+1+2 act=3", compass_p3_metrics, compass_p3_pf),
        ("Compass 1+1+2 act=4", compass_p4_metrics, compass_p4_pf),
        ("Aspiration Packs",    aspiration_metrics, aspiration_pf),
    ]

    for model in models:
        print(f"\n  --- Model {model} ({model_names[model]}) ---")
        header = f"  {'Algorithm':<25} {'M1':>5} {'M2':>5} {'M3':>6} {'M4':>5} {'M5':>6} {'M6':>6} {'M7':>6} {'M8':>8} {'M9':>5} {'Pass':>5}"
        print(header)
        print("  " + "-" * (len(header) - 2))
        for name, met_dict, pf_dict in all_algos:
            m = met_dict[model]
            pf = pf_dict[model]
            passed = sum(1 for v in pf.values() if v == "PASS")
            m8_str = f"{m['m8_min']:.0f}-{m['m8_max']:.0f}%"
            print(f"  {name:<25} {m['m1']:>5.1f} {m['m2']:>5.2f} {m['m3']:>6.2f} "
                  f"{m['m4']:>5.2f} {m['m5']:>6.1f} {m['m6']:>5.1f}% "
                  f"{m['m7']:>5.1f}% {m8_str:>8} {m['m9']:>5.2f} {passed:>4}/9")

    # Fitness degradation curve
    print(f"\n\n{'='*90}")
    print("FITNESS DEGRADATION CURVE (M3: S/A committed late)")
    print(f"{'='*90}")
    print(f"  {'Algorithm':<25} {'Model A':>8} {'Model B':>8} {'Model C':>8} {'A->B drop':>10} {'A->C drop':>10}")
    for name, met_dict, _ in all_algos:
        ma = met_dict["A"]["m3"]
        mb = met_dict["B"]["m3"]
        mc = met_dict["C"]["m3"]
        drop_ab = ma - mb
        drop_ac = ma - mc
        print(f"  {name:<25} {ma:>8.2f} {mb:>8.2f} {mc:>8.2f} {drop_ab:>10.2f} {drop_ac:>10.2f}")

    # R2 slot analysis summary
    print(f"\n\n{'='*90}")
    print("R2 NEIGHBOR SLOT TIER BREAKDOWN (Compass 1+1+2 primary)")
    print(f"{'='*90}")
    print(f"  {'Model':<15} {'S%':>6} {'A%':>6} {'B%':>6} {'C%':>6} {'S/A%':>7} {'N':>8}")
    for model in models:
        r2 = compass_r2[model]
        print(f"  {model_names[model]:<15} {r2['S%']:>6.1f} {r2['A%']:>6.1f} "
              f"{r2['B%']:>6.1f} {r2['C%']:>6.1f} {r2['SA%']:>7.1f} {r2['total']:>8}")

    # Compass vs Aspiration on M4/M8
    print(f"\n\n{'='*90}")
    print("COMPASS vs ASPIRATION: M4 (off-arch) and M8 (arch freq)")
    print(f"{'='*90}")
    for model in models:
        cm = compass_metrics[model]
        am = aspiration_metrics[model]
        print(f"\n  Model {model}:")
        print(f"    M4 (off-arch, >=0.5): Compass={cm['m4']:.2f}, Aspiration={am['m4']:.2f}")
        print(f"    M8 range (5-20%):     Compass={cm['m8_min']:.1f}-{cm['m8_max']:.1f}%, "
              f"Aspiration={am['m8_min']:.1f}-{am['m8_max']:.1f}%")

    print("\n\nDone.")


if __name__ == "__main__":
    main()
