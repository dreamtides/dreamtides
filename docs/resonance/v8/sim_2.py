#!/usr/bin/env python3
"""
Simulation Agent 2: Continuous Surge

Algorithm: Maintain 4 resonance counters. Each drafted card adds +2 to primary
resonance, +1 to secondary. Per slot, P(targeted) = min(max_counter / K, P_max).
Targeted slots draw from pair-matched subpool. Decay of -0.5/pick prevents runaway.
Floor of 1 targeted slot from pick 3+.

Parameters: K=6, P_max=0.75, decay=0.5, floor_start=3
"""

import random
import math
from collections import defaultdict
from dataclasses import dataclass, field
from typing import Optional

# ============================================================
# Configuration
# ============================================================

NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 4
POOL_SIZE = 360
CARDS_PER_ARCHETYPE = 40
GENERIC_CARDS = 40
RESONANCE_CARDS = POOL_SIZE - GENERIC_CARDS  # 320

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    {"name": "Flash",        "primary": "Zephyr", "secondary": "Ember"},
    {"name": "Blink",        "primary": "Ember",  "secondary": "Zephyr"},
    {"name": "Storm",        "primary": "Ember",  "secondary": "Stone"},
    {"name": "SelfDiscard",  "primary": "Stone",  "secondary": "Ember"},
    {"name": "SelfMill",     "primary": "Stone",  "secondary": "Tide"},
    {"name": "Sacrifice",    "primary": "Tide",   "secondary": "Stone"},
    {"name": "Warriors",     "primary": "Tide",   "secondary": "Zephyr"},
    {"name": "Ramp",         "primary": "Zephyr", "secondary": "Tide"},
]

ARCH_NAMES = [a["name"] for a in ARCHETYPES]
ARCH_INFO = {a["name"]: a for a in ARCHETYPES}

CO_PRIMARY_PAIRS = {}
for i, a in enumerate(ARCHETYPES):
    for j, b in enumerate(ARCHETYPES):
        if i != j and a["primary"] == b["primary"]:
            CO_PRIMARY_PAIRS[a["name"]] = b["name"]

FITNESS_MODELS = {
    "Optimistic": {
        ("Warriors", "Sacrifice"): 1.0, ("Sacrifice", "Warriors"): 1.0,
        ("SelfDiscard", "SelfMill"): 1.0, ("SelfMill", "SelfDiscard"): 1.0,
        ("Blink", "Storm"): 1.0, ("Storm", "Blink"): 1.0,
        ("Flash", "Ramp"): 1.0, ("Ramp", "Flash"): 1.0,
    },
    "Graduated Realistic": {
        ("Warriors", "Sacrifice"): 0.50, ("Sacrifice", "Warriors"): 0.50,
        ("SelfDiscard", "SelfMill"): 0.40, ("SelfMill", "SelfDiscard"): 0.40,
        ("Blink", "Storm"): 0.30, ("Storm", "Blink"): 0.30,
        ("Flash", "Ramp"): 0.25, ("Ramp", "Flash"): 0.25,
    },
    "Pessimistic": {
        ("Warriors", "Sacrifice"): 0.35, ("Sacrifice", "Warriors"): 0.35,
        ("SelfDiscard", "SelfMill"): 0.25, ("SelfMill", "SelfDiscard"): 0.25,
        ("Blink", "Storm"): 0.15, ("Storm", "Blink"): 0.15,
        ("Flash", "Ramp"): 0.10, ("Ramp", "Flash"): 0.10,
    },
    "Hostile": {
        ("Warriors", "Sacrifice"): 0.08, ("Sacrifice", "Warriors"): 0.08,
        ("SelfDiscard", "SelfMill"): 0.08, ("SelfMill", "SelfDiscard"): 0.08,
        ("Blink", "Storm"): 0.08, ("Storm", "Blink"): 0.08,
        ("Flash", "Ramp"): 0.08, ("Ramp", "Flash"): 0.08,
    },
}


@dataclass
class SimCard:
    id: int
    symbols: list
    archetype: str
    power: float
    is_generic: bool = False
    # Pre-computed tiers for each player archetype (assigned at pool creation)
    tiers: dict = field(default_factory=dict)  # archetype_name -> tier string


def assign_tiers(card: SimCard, fitness_model: dict):
    """Pre-assign a card's tier for every archetype. Called once at pool creation."""
    for arch in ARCHETYPES:
        arch_name = arch["name"]
        if card.is_generic:
            card.tiers[arch_name] = "B" if random.random() < 0.25 else "C"
            continue

        if card.archetype == arch_name:
            card.tiers[arch_name] = "S"
            continue

        sibling = CO_PRIMARY_PAIRS.get(arch_name)
        if card.archetype == sibling:
            rate = fitness_model.get((arch_name, sibling), 0.0)
            if random.random() < rate:
                card.tiers[arch_name] = "A"
            else:
                card.tiers[arch_name] = "B" if random.random() < 0.3 else "C"
            continue

        # Check resonance overlap
        a_info = ARCH_INFO[arch_name]
        c_info = ARCH_INFO[card.archetype]
        player_res = {a_info["primary"], a_info["secondary"]}
        card_res = {c_info["primary"], c_info["secondary"]}

        if player_res & card_res:
            card.tiers[arch_name] = "B" if random.random() < 0.15 else "C"
        else:
            card.tiers[arch_name] = "F" if random.random() < 0.3 else "C"


def is_sa(tier: str) -> bool:
    return tier in ("S", "A")


# ============================================================
# Pool Construction
# ============================================================

def build_pool(dual_res_pct: float, fitness_model: dict) -> list:
    """Build a 360-card pool with pre-assigned fitness tiers."""
    cards = []
    card_id = 0

    for _ in range(GENERIC_CARDS):
        c = SimCard(id=card_id, symbols=[], archetype="Generic",
                    power=random.uniform(3, 7), is_generic=True)
        assign_tiers(c, fitness_model)
        cards.append(c)
        card_id += 1

    total_resonance = RESONANCE_CARDS
    num_dual = int(total_resonance * dual_res_pct)
    dual_per_archetype = num_dual // 8
    single_per_archetype = CARDS_PER_ARCHETYPE - dual_per_archetype

    for arch in ARCHETYPES:
        for _ in range(single_per_archetype):
            c = SimCard(id=card_id, symbols=[arch["primary"]],
                        archetype=arch["name"], power=random.uniform(3, 8))
            assign_tiers(c, fitness_model)
            cards.append(c)
            card_id += 1

        for _ in range(dual_per_archetype):
            c = SimCard(id=card_id, symbols=[arch["primary"], arch["secondary"]],
                        archetype=arch["name"], power=random.uniform(3, 8))
            assign_tiers(c, fitness_model)
            cards.append(c)
            card_id += 1

    return cards


def build_indices(pool: list) -> dict:
    indices = {
        "by_primary": defaultdict(list),
        "by_pair": defaultdict(list),
        "by_archetype": defaultdict(list),
        "all": pool,
    }
    for card in pool:
        if card.is_generic:
            continue
        if card.symbols:
            indices["by_primary"][card.symbols[0]].append(card)
        if len(card.symbols) >= 2:
            indices["by_pair"][(card.symbols[0], card.symbols[1])].append(card)
        indices["by_archetype"][card.archetype].append(card)
    return indices


# ============================================================
# Continuous Surge Algorithm
# ============================================================

@dataclass
class SurgeState:
    counters: dict = field(default_factory=lambda: {r: 0.0 for r in RESONANCES})
    K: float = 6.0
    P_max: float = 0.75
    decay: float = 0.5
    floor_start: int = 3
    floor_slots: int = 1


def targeting_prob(state: SurgeState) -> float:
    max_c = max(state.counters.values())
    return min(max_c / state.K, state.P_max)


def dominant_pair(state: SurgeState) -> Optional[tuple]:
    dom = max(state.counters, key=state.counters.get)
    if state.counters[dom] == 0:
        return None
    best_pair = None
    best_score = -1
    for res in RESONANCES:
        if res != dom:
            score = state.counters[dom] * 2 + state.counters[res]
            if score > best_score:
                best_score = score
                best_pair = (dom, res)
    return best_pair


def update_counters(state: SurgeState, card: SimCard):
    if card.is_generic or not card.symbols:
        return
    state.counters[card.symbols[0]] += 2.0
    for sym in card.symbols[1:]:
        state.counters[sym] += 1.0
    for res in RESONANCES:
        state.counters[res] = max(0, state.counters[res] - state.decay)


def generate_pack(state: SurgeState, pick_num: int,
                  indices: dict, pool: list) -> list:
    p = targeting_prob(state)
    pair = dominant_pair(state)
    dom = max(state.counters, key=state.counters.get) if pair else None

    # Determine targeted slot count via independent Bernoulli trials
    targeted = sum(1 for _ in range(PACK_SIZE) if random.random() < p) if pair else 0

    # Apply floor
    if pick_num >= state.floor_start and pair is not None:
        targeted = max(targeted, state.floor_slots)

    targeted = min(targeted, PACK_SIZE)
    pack = []
    used_ids = set()

    # Fill targeted slots: prefer pair-matched, fallback to R1
    for _ in range(targeted):
        candidates = [c for c in indices["by_pair"].get(pair, [])
                       if c.id not in used_ids] if pair else []
        if not candidates and dom:
            candidates = [c for c in indices["by_primary"].get(dom, [])
                           if c.id not in used_ids]
        if not candidates:
            candidates = [c for c in pool if c.id not in used_ids]
        if candidates:
            card = random.choice(candidates)
            pack.append(card)
            used_ids.add(card.id)

    # Fill remaining slots randomly
    remaining = PACK_SIZE - len(pack)
    for _ in range(remaining):
        candidates = [c for c in pool if c.id not in used_ids]
        if candidates:
            card = random.choice(candidates)
            pack.append(card)
            used_ids.add(card.id)

    return pack


# ============================================================
# Player Strategies
# ============================================================

def pick_committed(pack, committed_arch, pick_num, arch_scores):
    """Archetype-committed: pre-commit tracks scores, post-commit picks best for arch."""
    if committed_arch is None:
        # Pre-commit: pick highest power, track arch affinity
        best = max(pack, key=lambda c: c.power)
        for a in ARCH_NAMES:
            if is_sa(best.tiers.get(a, "C")):
                arch_scores[a] += 1.0
        return best, None

    def score(c):
        t = c.tiers.get(committed_arch, "C")
        return {"S": 10, "A": 7, "B": 3, "C": 1, "F": 0}.get(t, 0) + c.power * 0.01

    return max(pack, key=score), committed_arch


def pick_power(pack):
    return max(pack, key=lambda c: c.power)


def pick_signal(pack, arch_scores, pick_num):
    best_card = None
    best_score = -1
    best_arch = None
    for card in pack:
        for a in ARCH_NAMES:
            t = card.tiers.get(a, "C")
            tv = {"S": 10, "A": 7, "B": 3, "C": 1, "F": 0}.get(t, 0)
            combined = tv * (1 + arch_scores.get(a, 0) * 0.3)
            if combined > best_score:
                best_score = combined
                best_card = card
                best_arch = a
    return best_card, best_arch


# ============================================================
# Simulation Core
# ============================================================

def simulate_draft(pool, indices, strategy, surge_params, force_arch=None, trace=False):
    """Run one draft. Returns dict of metrics. If trace=True, also returns trace lines."""
    state = SurgeState(**surge_params)
    committed_arch = force_arch
    arch_scores = defaultdict(float)
    drafted = []
    pack_sa_all = []
    trace_lines = [] if trace else None

    for pick_num in range(1, NUM_PICKS + 1):
        pack = generate_pack(state, pick_num, indices, pool)

        # Pick
        if strategy == "archetype_committed":
            if committed_arch is None and pick_num >= 5:
                committed_arch = max(arch_scores, key=arch_scores.get) if arch_scores else random.choice(ARCH_NAMES)
            picked, _ = pick_committed(pack, committed_arch, pick_num, arch_scores)
        elif strategy == "power_chaser":
            picked = pick_power(pack)
            if pick_num <= 6:
                for a in ARCH_NAMES:
                    if is_sa(picked.tiers.get(a, "C")):
                        arch_scores[a] += 1.0
            if pick_num == 6 and committed_arch is None:
                committed_arch = max(arch_scores, key=arch_scores.get) if arch_scores else random.choice(ARCH_NAMES)
        elif strategy == "signal_reader":
            picked, best_arch = pick_signal(pack, arch_scores, pick_num)
            if best_arch:
                arch_scores[best_arch] += 1.0
            if committed_arch is None and pick_num >= 6:
                committed_arch = max(arch_scores, key=arch_scores.get) if arch_scores else random.choice(ARCH_NAMES)

        update_counters(state, picked)
        drafted.append(picked)

        # Count S/A for committed archetype in this pack
        eval_arch = committed_arch if committed_arch else (
            max(arch_scores, key=arch_scores.get) if arch_scores else None
        )
        if eval_arch:
            sa_count = sum(1 for c in pack if is_sa(c.tiers.get(eval_arch, "C")))
        else:
            # Count max S/A across any archetype
            sa_count = max(
                (sum(1 for c in pack if is_sa(c.tiers.get(a, "C"))) for a in ARCH_NAMES),
                default=0
            )

        off_count = 0
        if committed_arch:
            off_count = sum(1 for c in pack if c.tiers.get(committed_arch, "C") in ("C", "F"))

        pack_sa_all.append(sa_count)

        if trace:
            p = targeting_prob(state)
            pair = dominant_pair(state)
            trace_lines.append(
                f"  Pick {pick_num:2d}: P={p:.2f} pair={pair} SA={sa_count} "
                f"picked={picked.archetype}({','.join(picked.symbols)}) pwr={picked.power:.1f}"
            )

    # Compute metrics
    early_sa = pack_sa_all[:5]
    late_sa = pack_sa_all[5:]
    m3 = sum(late_sa) / len(late_sa) if late_sa else 0

    # Convergence: first pick i where rolling 3-pick avg >= 2.0
    m5 = NUM_PICKS
    for i in range(len(pack_sa_all) - 2):
        if sum(pack_sa_all[i:i+3]) / 3 >= 2.0:
            m5 = i + 1
            break

    # Deck concentration
    if committed_arch:
        sa_deck = sum(1 for c in drafted if is_sa(c.tiers.get(committed_arch, "C")))
        m6 = sa_deck / len(drafted)
    else:
        m6 = 0

    # M9: stddev of late pack SA
    if len(late_sa) >= 2:
        mean = sum(late_sa) / len(late_sa)
        m9 = math.sqrt(sum((x - mean)**2 for x in late_sa) / len(late_sa))
    else:
        m9 = 0

    # Early: unique archetypes with S/A
    unique_early = []
    for pick_idx in range(min(5, len(pack_sa_all))):
        # Recount from pack data -- we need the pack. Approximate: use pack_sa_all as proxy.
        unique_early.append(pack_sa_all[pick_idx])  # placeholder

    result = {
        "m3": m3,
        "m5": m5,
        "m6": m6,
        "m9": m9,
        "late_sa": late_sa,
        "early_sa": early_sa,
        "pack_sa_all": pack_sa_all,
        "committed_arch": committed_arch,
        "drafted_ids": set(c.id for c in drafted),
    }

    if trace:
        return result, trace_lines
    return result


# ============================================================
# Full M1/M2 measurement (needs pack-level archetype tracking)
# ============================================================

def simulate_draft_full(pool, indices, strategy, surge_params, force_arch=None):
    """Like simulate_draft but also tracks M1, M2, M4 precisely."""
    state = SurgeState(**surge_params)
    committed_arch = force_arch
    arch_scores = defaultdict(float)
    drafted = []
    pack_sa_all = []
    m1_values = []
    m2_values = []
    m4_values = []

    for pick_num in range(1, NUM_PICKS + 1):
        pack = generate_pack(state, pick_num, indices, pool)

        # M1/M2: for early packs
        if pick_num <= 5:
            archs_with_sa = set()
            for c in pack:
                for a in ARCH_NAMES:
                    if is_sa(c.tiers.get(a, "C")):
                        archs_with_sa.add(a)
            m1_values.append(len(archs_with_sa))

            # M2: best single-archetype SA count
            best_sa = max(sum(1 for c in pack if is_sa(c.tiers.get(a, "C"))) for a in ARCH_NAMES)
            m2_values.append(best_sa)

        # Pick
        if strategy == "archetype_committed":
            if committed_arch is None and pick_num >= 5:
                committed_arch = max(arch_scores, key=arch_scores.get) if arch_scores else random.choice(ARCH_NAMES)
            picked, _ = pick_committed(pack, committed_arch, pick_num, arch_scores)
        elif strategy == "power_chaser":
            picked = pick_power(pack)
            if pick_num <= 6:
                for a in ARCH_NAMES:
                    if is_sa(picked.tiers.get(a, "C")):
                        arch_scores[a] += 1.0
            if pick_num == 6 and committed_arch is None:
                committed_arch = max(arch_scores, key=arch_scores.get) if arch_scores else random.choice(ARCH_NAMES)
        elif strategy == "signal_reader":
            picked, best_arch = pick_signal(pack, arch_scores, pick_num)
            if best_arch:
                arch_scores[best_arch] += 1.0
            if committed_arch is None and pick_num >= 6:
                committed_arch = max(arch_scores, key=arch_scores.get) if arch_scores else random.choice(ARCH_NAMES)

        update_counters(state, picked)
        drafted.append(picked)

        eval_arch = committed_arch
        if eval_arch:
            sa_count = sum(1 for c in pack if is_sa(c.tiers.get(eval_arch, "C")))
            off_count = sum(1 for c in pack if c.tiers.get(eval_arch, "C") in ("C", "F"))
        else:
            sa_count = max((sum(1 for c in pack if is_sa(c.tiers.get(a, "C"))) for a in ARCH_NAMES), default=0)
            off_count = 0

        pack_sa_all.append(sa_count)
        if pick_num > 5 and committed_arch:
            m4_values.append(off_count)

    late_sa = pack_sa_all[5:]
    m3 = sum(late_sa) / len(late_sa) if late_sa else 0

    m5 = NUM_PICKS
    for i in range(len(pack_sa_all) - 2):
        if sum(pack_sa_all[i:i+3]) / 3 >= 2.0:
            m5 = i + 1
            break

    sa_deck = sum(1 for c in drafted if is_sa(c.tiers.get(committed_arch, "C"))) if committed_arch else 0
    m6 = sa_deck / len(drafted) if drafted else 0

    if len(late_sa) >= 2:
        mean = sum(late_sa) / len(late_sa)
        m9 = math.sqrt(sum((x - mean)**2 for x in late_sa) / len(late_sa))
    else:
        m9 = 0

    return {
        "m1": sum(m1_values) / len(m1_values) if m1_values else 0,
        "m2": sum(m2_values) / len(m2_values) if m2_values else 0,
        "m3": m3,
        "m4": sum(m4_values) / len(m4_values) if m4_values else 0,
        "m5": m5,
        "m6": m6,
        "m9": m9,
        "late_sa": late_sa,
        "committed_arch": committed_arch,
        "drafted_ids": set(c.id for c in drafted),
    }


# ============================================================
# Aggregation
# ============================================================

def compute_m10(all_late_sa: list) -> dict:
    max_consec_list = []
    avg_streak_list = []
    for draft_sa in all_late_sa:
        consec = 0
        max_c = 0
        streaks = []
        for sa in draft_sa:
            if sa < 1.5:
                consec += 1
            else:
                if consec > 0:
                    streaks.append(consec)
                consec = 0
            max_c = max(max_c, consec)
        if consec > 0:
            streaks.append(consec)
        max_consec_list.append(max_c)
        avg_streak_list.append(sum(streaks) / len(streaks) if streaks else 0)

    return {
        "avg_max": sum(max_consec_list) / len(max_consec_list) if max_consec_list else 0,
        "worst": max(max_consec_list) if max_consec_list else 0,
        "avg_streak": sum(avg_streak_list) / len(avg_streak_list) if avg_streak_list else 0,
    }


def pack_quality_pctiles(all_late_sa: list) -> dict:
    flat = [sa for draft in all_late_sa for sa in draft]
    if not flat:
        return {p: 0 for p in [10, 25, 50, 75, 90]}
    flat.sort()
    n = len(flat)
    return {p: flat[min(int(n * p / 100), n - 1)] for p in [10, 25, 50, 75, 90]}


def run_full_sim(pool_config, fitness_name, surge_params, strategy, n_drafts=1000):
    """Run n_drafts and return aggregated metrics."""
    dual_pct = 0.15 if pool_config == "V7 Standard" else 0.40
    fm = FITNESS_MODELS[fitness_name]

    all_m1, all_m2, all_m3, all_m4, all_m5, all_m6, all_m9 = [], [], [], [], [], [], []
    all_late_sa = []
    all_ids = []
    arch_freq = defaultdict(int)

    for _ in range(n_drafts):
        pool = build_pool(dual_pct, fm)
        idx = build_indices(pool)
        r = simulate_draft_full(pool, idx, strategy, surge_params)

        all_m1.append(r["m1"])
        all_m2.append(r["m2"])
        all_m3.append(r["m3"])
        all_m4.append(r["m4"])
        all_m5.append(r["m5"])
        all_m6.append(r["m6"])
        all_m9.append(r["m9"])
        all_late_sa.append(r["late_sa"])
        all_ids.append(r["drafted_ids"])
        if r["committed_arch"]:
            arch_freq[r["committed_arch"]] += 1

    avg = lambda lst: sum(lst) / len(lst) if lst else 0

    # M7: pairwise card overlap (sample)
    sample_n = min(200, len(all_ids))
    sample = random.sample(all_ids, sample_n)
    overlaps = []
    for i in range(len(sample)):
        for j in range(i+1, min(i+20, len(sample))):
            inter = len(sample[i] & sample[j])
            overlaps.append(inter / NUM_PICKS)
    m7 = avg(overlaps)

    # M8
    total_runs = sum(arch_freq.values())
    m8 = {k: v / total_runs for k, v in arch_freq.items()} if total_runs > 0 else {}

    m10 = compute_m10(all_late_sa)
    pq = pack_quality_pctiles(all_late_sa)

    return {
        "M1": avg(all_m1), "M2": avg(all_m2), "M3": avg(all_m3),
        "M4": avg(all_m4), "M5": avg(all_m5), "M6": avg(all_m6),
        "M7": m7, "M8": m8, "M9": avg(all_m9),
        "M10": m10, "PQ": pq,
    }


def run_per_arch(pool_config, fitness_name, surge_params, n_per_arch=125):
    """Run simulation forcing each archetype. Returns per-arch metrics."""
    dual_pct = 0.15 if pool_config == "V7 Standard" else 0.40
    fm = FITNESS_MODELS[fitness_name]

    results = {}
    for arch in ARCHETYPES:
        a = arch["name"]
        all_m3, all_m5, all_m6, all_m9 = [], [], [], []
        all_late = []

        for _ in range(n_per_arch):
            pool = build_pool(dual_pct, fm)
            idx = build_indices(pool)
            r = simulate_draft(pool, idx, "archetype_committed", surge_params, force_arch=a)
            all_m3.append(r["m3"])
            all_m5.append(r["m5"])
            all_m6.append(r["m6"])
            all_m9.append(r["m9"])
            all_late.append(r["late_sa"])

        avg = lambda lst: sum(lst) / len(lst) if lst else 0
        m10 = compute_m10(all_late)
        pq = pack_quality_pctiles(all_late)

        results[a] = {
            "M3": avg(all_m3), "M5": avg(all_m5), "M6": avg(all_m6),
            "M9": avg(all_m9), "M10": m10, "PQ": pq,
        }

    return results


def parameter_sweep(pool_config, fitness_name, n_drafts=200):
    """Sweep K, P_max, decay."""
    rows = []
    for K in [4, 6, 8]:
        for P_max in [0.65, 0.75, 0.85]:
            for decay in [0.3, 0.5, 0.7]:
                params = {"K": K, "P_max": P_max, "decay": decay,
                          "floor_start": 3, "floor_slots": 1}
                r = run_full_sim(pool_config, fitness_name, params,
                                 "archetype_committed", n_drafts)
                rows.append({"K": K, "P_max": P_max, "decay": decay,
                             "M3": r["M3"], "M5": r["M5"], "M9": r["M9"],
                             "M10": r["M10"]["avg_max"]})
    return rows


def run_trace(pool_config, fitness_name, surge_params, strategy, label):
    dual_pct = 0.15 if pool_config == "V7 Standard" else 0.40
    fm = FITNESS_MODELS[fitness_name]
    pool = build_pool(dual_pct, fm)
    idx = build_indices(pool)
    r, lines = simulate_draft(pool, idx, strategy, surge_params, trace=True)
    header = [f"\n--- {label} ({pool_config}, {fitness_name}) ---"]
    header.append(f"  M3={r['m3']:.2f}  M5={r['m5']}  committed={r['committed_arch']}")
    return header + lines


# ============================================================
# Main
# ============================================================

def main():
    random.seed(42)

    default_params = {"K": 6.0, "P_max": 0.75, "decay": 0.5,
                      "floor_start": 3, "floor_slots": 1}

    pools = ["V7 Standard", "40% Enriched"]
    fitnesses = ["Optimistic", "Graduated Realistic", "Pessimistic", "Hostile"]
    strategies = ["archetype_committed", "power_chaser", "signal_reader"]

    print("=" * 80)
    print("CONTINUOUS SURGE SIMULATION - Agent 2")
    print("=" * 80)

    # ---- Main scorecard ----
    print("\n### MAIN SCORECARD (archetype-committed)")
    all_results = {}
    for pool in pools:
        for fit in fitnesses:
            key = f"{pool} / {fit}"
            print(f"\nRunning: {key}...")
            r = run_full_sim(pool, fit, default_params, "archetype_committed", 1000)
            all_results[key] = r
            pq = r["PQ"]
            m10 = r["M10"]
            print(f"  M1={r['M1']:.2f}  M2={r['M2']:.2f}  M3={r['M3']:.2f}  "
                  f"M4={r['M4']:.2f}  M5={r['M5']:.1f}  M6={r['M6']:.1%}")
            print(f"  M7={r['M7']:.1%}  M9={r['M9']:.2f}  "
                  f"M10_avg={m10['avg_max']:.1f}  M10_worst={m10['worst']}")
            print(f"  PQ: p10={pq[10]} p25={pq[25]} p50={pq[50]} p75={pq[75]} p90={pq[90]}")

    # ---- Multi-strategy ----
    print("\n\n### MULTI-STRATEGY (40% Enriched, Graduated Realistic)")
    for strat in strategies:
        r = run_full_sim("40% Enriched", "Graduated Realistic",
                         default_params, strat, 500)
        print(f"  {strat:<22} M3={r['M3']:.2f}  M5={r['M5']:.1f}  "
              f"M6={r['M6']:.1%}  M9={r['M9']:.2f}")

    # ---- Per-archetype convergence ----
    print("\n\n### PER-ARCHETYPE CONVERGENCE (40% Enriched)")
    for fit in ["Graduated Realistic", "Pessimistic"]:
        print(f"\nFitness: {fit}")
        per_arch = run_per_arch("40% Enriched", fit, default_params, 200)
        print(f"  {'Archetype':<15} {'M3':>6} {'M5':>6} {'M6':>8} {'M9':>6} {'M10avg':>7}")
        for a in ARCH_NAMES:
            r = per_arch[a]
            print(f"  {a:<15} {r['M3']:6.2f} {r['M5']:6.1f} "
                  f"{r['M6']:7.1%} {r['M9']:6.2f} {r['M10']['avg_max']:7.1f}")

    # ---- M7/M8 (already in main scorecard) ----
    print("\n\n### M7/M8 detail (40% Enriched, Graduated Realistic)")
    r = all_results["40% Enriched / Graduated Realistic"]
    print(f"  M7 (overlap): {r['M7']:.1%}")
    print(f"  M8 distribution: {r['M8']}")

    # ---- Parameter sweep ----
    print("\n\n### PARAMETER SENSITIVITY (40% Enriched, Graduated Realistic)")
    sweep = parameter_sweep("40% Enriched", "Graduated Realistic", 200)
    print(f"  {'K':>3} {'Pmax':>5} {'dec':>5} {'M3':>6} {'M5':>6} {'M9':>6} {'M10':>6}")
    for s in sweep:
        print(f"  {s['K']:3.0f} {s['P_max']:5.2f} {s['decay']:5.2f} "
              f"{s['M3']:6.2f} {s['M5']:6.1f} {s['M9']:6.2f} {s['M10']:6.1f}")

    # ---- Fitness degradation ----
    print("\n\n### FITNESS DEGRADATION (40% Enriched)")
    for fit in fitnesses:
        key = f"40% Enriched / {fit}"
        print(f"  {fit:<25} M3={all_results[key]['M3']:.2f}")

    # ---- Draft traces ----
    print("\n\n### DRAFT TRACES")
    for label, strat in [("Early Committer", "archetype_committed"),
                          ("Signal Reader", "signal_reader"),
                          ("Power Chaser", "power_chaser")]:
        lines = run_trace("40% Enriched", "Graduated Realistic",
                          default_params, strat, label)
        for l in lines:
            print(l)

    print("\n### SIMULATION COMPLETE")


if __name__ == "__main__":
    main()
