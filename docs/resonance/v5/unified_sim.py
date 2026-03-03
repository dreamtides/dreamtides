#!/usr/bin/env python3
"""
Resonance V5 — Round 5 Unified Simulation

Runs all 5 V5 algorithms plus Lane Locking and Pack Widening baselines
on the same card pool, producing a unified comparison table.
"""

import random
import statistics
import math
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional
from collections import defaultdict, Counter

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    "Flash/Tempo/Prison",   # 0: Zephyr primary, Ember secondary
    "Blink/Flicker",        # 1: Ember primary, Zephyr secondary
    "Storm/Spellslinger",   # 2: Ember primary, Stone secondary
    "Self-Discard",         # 3: Stone primary, Ember secondary
    "Self-Mill/Reanimator", # 4: Stone primary, Tide secondary
    "Sacrifice/Abandon",    # 5: Tide primary, Stone secondary
    "Warriors/Midrange",    # 6: Tide primary, Zephyr secondary
    "Ramp/Spirit Animals",  # 7: Zephyr primary, Tide secondary
]

ARCHETYPE_RESONANCES = {
    0: ("Zephyr", "Ember"),
    1: ("Ember", "Zephyr"),
    2: ("Ember", "Stone"),
    3: ("Stone", "Ember"),
    4: ("Stone", "Tide"),
    5: ("Tide", "Stone"),
    6: ("Tide", "Zephyr"),
    7: ("Zephyr", "Tide"),
}

NUM_ARCHETYPES = 8
NUM_CARDS = 360
GENERIC_COUNT = 36
CARDS_PER_ARCHETYPE = (NUM_CARDS - GENERIC_COUNT) // NUM_ARCHETYPES  # 40
PACK_SIZE = 4
NUM_PICKS = 30
NUM_DRAFTS = 1000
PRIMARY_WEIGHT = 2

# ---------------------------------------------------------------------------
# Card model
# ---------------------------------------------------------------------------

class Tier(Enum):
    S = 4
    A = 3
    B = 2
    C = 1
    F = 0

@dataclass
class SimCard:
    id: int
    symbols: list
    archetype: Optional[str]
    archetype_idx: Optional[int]
    archetype_fitness: dict = field(default_factory=dict)
    power: float = 5.0

def _circle_distance(a: int, b: int) -> int:
    return min(abs(a - b), NUM_ARCHETYPES - abs(a - b))

def _assign_fitness(card: SimCard):
    if card.archetype_idx is None:
        for i in range(NUM_ARCHETYPES):
            card.archetype_fitness[i] = Tier.B
        return
    home = card.archetype_idx
    home_pri, home_sec = ARCHETYPE_RESONANCES[home]
    for i in range(NUM_ARCHETYPES):
        if i == home:
            card.archetype_fitness[i] = Tier.S
            continue
        dist = _circle_distance(home, i)
        target_pri, target_sec = ARCHETYPE_RESONANCES[i]
        if dist == 1 and (target_pri == home_pri or target_sec == home_pri):
            card.archetype_fitness[i] = Tier.A
        elif home_sec in (target_pri, target_sec):
            card.archetype_fitness[i] = Tier.B
        elif dist <= 3:
            card.archetype_fitness[i] = Tier.C
        else:
            card.archetype_fitness[i] = Tier.F

def _generate_symbols(archetype_idx: int, num_symbols: int, rng: random.Random) -> list:
    primary_res, secondary_res = ARCHETYPE_RESONANCES[archetype_idx]
    if num_symbols == 1:
        return [primary_res] if rng.random() < 0.70 else [secondary_res]
    elif num_symbols == 2:
        first = primary_res if rng.random() < 0.75 else secondary_res
        other = secondary_res if first == primary_res else primary_res
        second = other if rng.random() < 0.80 else first
        return [first, second]
    else:
        patterns = [
            [primary_res, secondary_res, primary_res],
            [primary_res, primary_res, secondary_res],
            [primary_res, secondary_res, secondary_res],
        ]
        return rng.choice(patterns)

def build_card_pool(rng: random.Random,
                    pct_1sym: float = 0.15,
                    pct_2sym: float = 0.60,
                    pct_3sym: float = 0.25) -> list:
    cards = []
    card_id = 0
    for arch_idx in range(NUM_ARCHETYPES):
        n = CARDS_PER_ARCHETYPE
        n1 = round(n * pct_1sym)
        n3 = round(n * pct_3sym)
        n2 = n - n1 - n3
        for count, num in [(1, n1), (2, n2), (3, n3)]:
            for _ in range(num):
                syms = _generate_symbols(arch_idx, count, rng)
                c = SimCard(id=card_id, symbols=syms,
                            archetype=ARCHETYPES[arch_idx],
                            archetype_idx=arch_idx,
                            power=rng.uniform(3, 8))
                _assign_fitness(c)
                cards.append(c)
                card_id += 1
    for _ in range(GENERIC_COUNT):
        c = SimCard(id=card_id, symbols=[], archetype=None, archetype_idx=None,
                    power=rng.uniform(4, 9))
        _assign_fitness(c)
        cards.append(c)
        card_id += 1
    return cards

def get_card_pair(card: SimCard):
    if len(card.symbols) >= 2:
        return (card.symbols[0], card.symbols[1])
    return None

# ---------------------------------------------------------------------------
# Utilities
# ---------------------------------------------------------------------------

def _update_resonance_counters(card: SimCard, player_counts: dict):
    if not card.symbols:
        return
    player_counts[card.symbols[0]] = player_counts.get(card.symbols[0], 0) + PRIMARY_WEIGHT
    for s in card.symbols[1:]:
        player_counts[s] = player_counts.get(s, 0) + 1

def _update_pair_counters(card: SimCard, pair_counts: dict):
    p = get_card_pair(card)
    if p:
        pair_counts[p] = pair_counts.get(p, 0) + 1

def _get_top_pair(pair_counts: dict):
    if not pair_counts:
        return None, 0
    top = max(pair_counts.items(), key=lambda x: x[1])
    return top[0], top[1]

def _get_top_resonance(res_counts: dict):
    if not res_counts:
        return None, 0
    top = max(res_counts.items(), key=lambda x: x[1])
    return top[0], top[1]

# ---------------------------------------------------------------------------
# Draft Algorithms
# ---------------------------------------------------------------------------

# D1: Pair-Based Threshold Auto-Spend
def gen_pack_d1(pool, pair_counts, rng, threshold=3, bonus=1, scaling=True):
    """Pair threshold auto-spend: when top pair reaches threshold, add bonus card(s)."""
    top_pair, top_count = _get_top_pair(pair_counts)
    pack = rng.sample(pool, min(PACK_SIZE, len(pool)))

    if top_pair and top_count >= threshold:
        pair_pool = [c for c in pool if get_card_pair(c) == top_pair]
        if pair_pool:
            # Check if triggered before (for scaling)
            trigger_count = top_count // threshold
            actual_bonus = bonus
            if scaling and trigger_count > 1:
                actual_bonus = min(bonus + 1, 2)  # scale up to 2
            for _ in range(actual_bonus):
                pack.append(rng.choice(pair_pool))
        # Reset after trigger
        pair_counts[top_pair] = top_count % threshold
    return pack

# D2: Pair-Escalation Slots
def gen_pack_d2(pool, pair_counts, rng, K=6, cap=0.65):
    """Each slot independently pair-matched with probability min(top_pair/K, cap)."""
    top_pair, top_count = _get_top_pair(pair_counts)
    prob = min(top_count / K, cap) if top_pair else 0.0
    pair_matched = [c for c in pool if get_card_pair(c) == top_pair] if top_pair else []

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
    return pack

# D3: Top-Pair Pool Seeding
def gen_pack_d3(pool, pair_counts, rng, inject_rate=4, inject_threshold=2):
    """Pool seeding: after threshold, inject pair-matched cards into pool. Pack is random 4."""
    # Injection happens after pick (handled in draft loop). Pack generation is just random.
    return rng.sample(pool, min(PACK_SIZE, len(pool)))

def d3_inject(pool, pair_counts, reserve, rng, inject_rate=4, inject_threshold=2):
    """Inject cards into the pool after a pick."""
    top_pair, top_count = _get_top_pair(pair_counts)
    if top_pair and top_count >= inject_threshold:
        candidates = [c for c in reserve if get_card_pair(c) == top_pair]
        n = min(inject_rate, len(candidates))
        if n > 0:
            injected = rng.sample(candidates, n)
            for c in injected:
                pool.append(c)
                reserve.remove(c)

# D4: Dual-Threshold Pair Guarantee
def gen_pack_d4(pool, pair_counts, rng, threshold1=3, threshold2=7):
    """At threshold1: 1 pair-matched slot. At threshold2: 2 pair-matched slots."""
    top_pair, top_count = _get_top_pair(pair_counts)
    guaranteed = 0
    if top_pair:
        if top_count >= threshold2:
            guaranteed = 2
        elif top_count >= threshold1:
            guaranteed = 1

    pack = []
    used_ids = set()
    for i in range(PACK_SIZE):
        if i < guaranteed and top_pair:
            pair_pool = [c for c in pool if get_card_pair(c) == top_pair]
            if pair_pool:
                chosen = rng.choice(pair_pool)
            else:
                candidates = [c for c in pool if c.id not in used_ids]
                chosen = rng.choice(candidates if candidates else pool)
        else:
            candidates = [c for c in pool if c.id not in used_ids]
            chosen = rng.choice(candidates if candidates else pool)
        pack.append(chosen)
        used_ids.add(chosen.id)
    return pack

# D5: Hybrid Resonance-Triggered Pair Bonus
def gen_pack_d5(pool, pair_counts, res_counts, rng):
    """Draw 4 random; if any card's primary matches top resonance, add 1 bonus pair-matched."""
    pack = rng.sample(pool, min(PACK_SIZE, len(pool)))
    top_res, top_res_count = _get_top_resonance(res_counts)
    top_pair, top_pair_count = _get_top_pair(pair_counts)

    if top_res and top_pair:
        trigger = any(c.symbols and c.symbols[0] == top_res for c in pack)
        if trigger:
            pair_pool = [c for c in pool if get_card_pair(c) == top_pair]
            if pair_pool:
                pack.append(rng.choice(pair_pool))
    return pack

# Lane Locking baseline
def gen_pack_lane_locking(pool, res_counts, rng, threshold1=3, threshold2=8):
    locked_resonances = []
    for res in RESONANCES:
        cnt = res_counts.get(res, 0)
        if cnt >= threshold2:
            locked_resonances.append(res)
            locked_resonances.append(res)
        elif cnt >= threshold1:
            locked_resonances.append(res)

    pack = []
    used_ids = set()
    for res in locked_resonances[:PACK_SIZE]:
        candidates = [c for c in pool if c.symbols and c.symbols[0] == res and c.id not in used_ids]
        if candidates:
            chosen = rng.choice(candidates)
            pack.append(chosen)
            used_ids.add(chosen.id)

    remaining = PACK_SIZE - len(pack)
    if remaining > 0:
        candidates = [c for c in pool if c.id not in used_ids]
        if candidates:
            chosen = rng.sample(candidates, min(remaining, len(candidates)))
            pack.extend(chosen)

    return pack[:PACK_SIZE]

# Pack Widening baseline (auto-spend)
def gen_pack_pack_widening(pool, res_counts, rng, cost=3, bonus_count=1):
    """Auto-spend: deduct tokens when top resonance >= cost."""
    pack = rng.sample(pool, min(PACK_SIZE, len(pool)))
    top_res, top_count = _get_top_resonance(res_counts)
    if top_res and top_count >= cost:
        res_pool = [c for c in pool if c.symbols and c.symbols[0] == top_res]
        if res_pool:
            for _ in range(bonus_count):
                pack.append(rng.choice(res_pool))
        res_counts[top_res] -= cost
    return pack

# Random baseline
def gen_pack_random(pool, rng):
    return rng.sample(pool, min(PACK_SIZE, len(pool)))

# ---------------------------------------------------------------------------
# Player strategies
# ---------------------------------------------------------------------------

def _best_arch_from_pairs(pair_counts):
    if not pair_counts:
        return None
    pair_to_arch = {}
    for ai, (pri, sec) in ARCHETYPE_RESONANCES.items():
        pair_to_arch[(pri, sec)] = ai
    arch_scores = [0] * NUM_ARCHETYPES
    for (p, s), cnt in pair_counts.items():
        arch = pair_to_arch.get((p, s))
        if arch is not None:
            arch_scores[arch] += cnt
        for ai, (pri, sec) in ARCHETYPE_RESONANCES.items():
            if pri == p and ai != arch:
                arch_scores[ai] += cnt * 0.3
    if max(arch_scores) == 0:
        return None
    return arch_scores.index(max(arch_scores))

def _best_arch_from_res(res_counts):
    if not res_counts or sum(res_counts.values()) == 0:
        return None
    scores = []
    for arch_idx in range(NUM_ARCHETYPES):
        pri, sec = ARCHETYPE_RESONANCES[arch_idx]
        score = res_counts.get(pri, 0) * 2 + res_counts.get(sec, 0)
        scores.append(score)
    return scores.index(max(scores))

def pick_card(pack, pair_counts, res_counts, target_arch, pick_num, strategy):
    if strategy == "committed":
        if target_arch is not None:
            best = max(pack, key=lambda c: (
                c.archetype_fitness.get(target_arch, Tier.F).value, c.power))
            return best, target_arch
        if pick_num < 5:
            tgt = _best_arch_from_pairs(pair_counts) or _best_arch_from_res(res_counts)
            if tgt is not None:
                sa = [c for c in pack if c.archetype_fitness.get(tgt, Tier.F).value >= Tier.A.value]
                if sa:
                    return max(sa, key=lambda c: c.power), None
            return max(pack, key=lambda c: c.power), None
        else:
            tgt = _best_arch_from_pairs(pair_counts) or _best_arch_from_res(res_counts) or 0
            return max(pack, key=lambda c: (
                c.archetype_fitness.get(tgt, Tier.F).value, c.power)), tgt

    elif strategy == "power":
        chosen = max(pack, key=lambda c: c.power)
        tgt = _best_arch_from_pairs(pair_counts) or _best_arch_from_res(res_counts)
        return chosen, tgt

    elif strategy == "signal":
        if pick_num < 5:
            pack_res = defaultdict(int)
            for c in pack:
                for s in c.symbols:
                    pack_res[s] += 1
            if pack_res:
                best_res = max(pack_res, key=pack_res.get)
                res_cards = [c for c in pack if best_res in c.symbols]
                return max(res_cards, key=lambda c: c.power), None
            return max(pack, key=lambda c: c.power), None
        else:
            if target_arch is None:
                target_arch = _best_arch_from_pairs(pair_counts) or _best_arch_from_res(res_counts) or 0
            return max(pack, key=lambda c: (
                c.archetype_fitness.get(target_arch, Tier.F).value, c.power)), target_arch

    return pack[0], 0

# ---------------------------------------------------------------------------
# Draft Result
# ---------------------------------------------------------------------------

@dataclass
class DraftResult:
    picks: list = field(default_factory=list)
    packs_seen: list = field(default_factory=list)
    target_archetype: int = 0
    strategy: str = ""

# ---------------------------------------------------------------------------
# Single Draft
# ---------------------------------------------------------------------------

def run_single_draft(pool_orig, strategy, rng, algorithm,
                     forced_arch=None, params=None):
    if params is None:
        params = {}

    pair_counts = {}
    res_counts = defaultdict(int)
    result = DraftResult(strategy=strategy)
    target_arch = forced_arch

    # D3 needs a mutable pool and reserve
    if algorithm == "d3":
        pool = list(pool_orig)
        reserve = build_reserve(rng)
    else:
        pool = pool_orig

    # D1 needs mutable pair counts for reset
    d1_pair_counts = {}

    for pick_num in range(NUM_PICKS):
        if algorithm == "d1":
            pack = gen_pack_d1(pool, d1_pair_counts, rng,
                               threshold=params.get("threshold", 3),
                               bonus=params.get("bonus", 1),
                               scaling=params.get("scaling", True))
        elif algorithm == "d2":
            pack = gen_pack_d2(pool, pair_counts, rng,
                               K=params.get("K", 6),
                               cap=params.get("cap", 0.65))
        elif algorithm == "d3":
            pack = gen_pack_d3(pool, pair_counts, rng)
        elif algorithm == "d4":
            pack = gen_pack_d4(pool, pair_counts, rng,
                               threshold1=params.get("threshold1", 3),
                               threshold2=params.get("threshold2", 7))
        elif algorithm == "d5":
            pack = gen_pack_d5(pool, pair_counts, res_counts, rng)
        elif algorithm == "lane_locking":
            pack = gen_pack_lane_locking(pool, res_counts, rng)
        elif algorithm == "pack_widening":
            pack = gen_pack_pack_widening(pool, res_counts, rng,
                                          cost=params.get("cost", 3),
                                          bonus_count=params.get("bonus_count", 1))
        elif algorithm == "random":
            pack = gen_pack_random(pool, rng)
        else:
            pack = gen_pack_random(pool, rng)

        if not pack:
            break

        result.packs_seen.append(list(pack))

        chosen, target_arch = pick_card(pack, pair_counts, res_counts,
                                        target_arch, pick_num, strategy)
        result.picks.append(chosen)
        _update_pair_counters(chosen, pair_counts)
        _update_resonance_counters(chosen, res_counts)

        # D1 uses its own pair counter (with reset)
        if algorithm == "d1":
            _update_pair_counters(chosen, d1_pair_counts)

        # D3 pool injection after pick
        if algorithm == "d3":
            d3_inject(pool, pair_counts, reserve, rng,
                      inject_rate=params.get("inject_rate", 4),
                      inject_threshold=params.get("inject_threshold", 2))

    if target_arch is None:
        target_arch = _best_arch_from_pairs(pair_counts) or _best_arch_from_res(res_counts) or 0
    result.target_archetype = target_arch
    return result


def build_reserve(rng):
    """Build a reserve pool for D3 pool seeding."""
    reserve = []
    card_id = 10000
    for arch_idx in range(NUM_ARCHETYPES):
        for _ in range(30):
            syms = _generate_symbols(arch_idx, 2, rng)
            c = SimCard(id=card_id, symbols=syms,
                        archetype=ARCHETYPES[arch_idx],
                        archetype_idx=arch_idx,
                        power=rng.uniform(3, 8))
            _assign_fitness(c)
            reserve.append(c)
            card_id += 1
    return reserve

# ---------------------------------------------------------------------------
# Metrics
# ---------------------------------------------------------------------------

def compute_metrics(results):
    early_unique = []
    early_sa = []
    late_sa = []
    late_off = []
    conv_picks = []
    deck_concs = []
    all_decks = []
    arch_freq = defaultdict(int)

    for dr in results:
        tgt = dr.target_archetype
        arch_freq[tgt] += 1
        sa_streak = 0
        conv = NUM_PICKS

        for pn, pack in enumerate(dr.packs_seen):
            sa_count = sum(1 for c in pack
                           if c.archetype_fitness.get(tgt, Tier.F).value >= Tier.A.value)
            cf_count = sum(1 for c in pack
                           if c.archetype_fitness.get(tgt, Tier.F).value <= Tier.C.value)

            if pn < 5:
                unique_archs = set()
                for c in pack:
                    for ai in range(NUM_ARCHETYPES):
                        if c.archetype_fitness.get(ai, Tier.F).value >= Tier.A.value:
                            unique_archs.add(ai)
                early_unique.append(len(unique_archs))
                early_sa.append(sa_count)
            else:
                late_sa.append(sa_count)
                late_off.append(cf_count)

            if pn >= 5:
                if sa_count >= 2:
                    sa_streak += 1
                else:
                    sa_streak = 0
                if sa_streak >= 3 and conv == NUM_PICKS:
                    conv = pn - 2

        conv_picks.append(conv)
        sa_deck = sum(1 for c in dr.picks
                      if c.archetype_fitness.get(tgt, Tier.F).value >= Tier.A.value)
        deck_concs.append(sa_deck / len(dr.picks) if dr.picks else 0)
        all_decks.append(set(c.id for c in dr.picks))

    # overlap
    overlaps = []
    by_arch = defaultdict(list)
    for i, dr in enumerate(results):
        by_arch[dr.target_archetype].append(i)
    for arch, indices in by_arch.items():
        for i in range(min(50, len(indices))):
            for j in range(i + 1, min(50, len(indices))):
                s1 = all_decks[indices[i]]
                s2 = all_decks[indices[j]]
                if s1 and s2:
                    overlaps.append(len(s1 & s2) / max(len(s1 | s2), 1))

    total = len(results)
    af = {a: arch_freq.get(a, 0) / total for a in range(NUM_ARCHETYPES)}
    stddev = statistics.stdev(late_sa) if len(late_sa) > 1 else 0

    # S/A distribution
    dist = defaultdict(int)
    for v in late_sa:
        dist[v] += 1
    total_late = len(late_sa) if late_sa else 1

    return {
        "early_unique": statistics.mean(early_unique) if early_unique else 0,
        "early_sa": statistics.mean(early_sa) if early_sa else 0,
        "late_sa": statistics.mean(late_sa) if late_sa else 0,
        "late_off": statistics.mean(late_off) if late_off else 0,
        "conv_pick": statistics.mean(conv_picks) if conv_picks else 30,
        "deck_conc": statistics.mean(deck_concs) if deck_concs else 0,
        "overlap": statistics.mean(overlaps) if overlaps else 0,
        "arch_freq_max": max(af.values()) if af else 0,
        "arch_freq_min": min(af.values()) if af else 0,
        "stddev": stddev,
        "sa_dist": {k: dist[k] / total_late for k in sorted(dist.keys())},
    }


def compute_per_arch_conv(pool, rng, algorithm, params=None, n_drafts=200):
    conv_table = {}
    for arch_idx in range(NUM_ARCHETYPES):
        picks_list = []
        for _ in range(n_drafts):
            dr = run_single_draft(pool, "committed", rng, algorithm,
                                  forced_arch=arch_idx, params=params)
            sa_streak = 0
            conv = NUM_PICKS
            for pn in range(5, len(dr.packs_seen)):
                pack = dr.packs_seen[pn]
                sa = sum(1 for c in pack
                         if c.archetype_fitness.get(arch_idx, Tier.F).value >= Tier.A.value)
                if sa >= 2:
                    sa_streak += 1
                else:
                    sa_streak = 0
                if sa_streak >= 3 and conv == NUM_PICKS:
                    conv = pn - 2
            picks_list.append(conv)
        conv_table[arch_idx] = statistics.mean(picks_list)
    return conv_table

# ---------------------------------------------------------------------------
# Run full simulation for one algorithm
# ---------------------------------------------------------------------------

def run_full(pool, algorithm, rng_seed, params=None, n_drafts=1000, label=""):
    all_results = []
    strategies = ["committed", "power", "signal"]
    for strat in strategies:
        for _ in range(n_drafts):
            rng = random.Random(rng_seed)
            rng_seed += 1
            dr = run_single_draft(pool, strat, rng, algorithm, params=params)
            all_results.append(dr)

    committed = [r for r in all_results if r.strategy == "committed"]
    metrics = compute_metrics(committed)
    return metrics

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    print("=" * 80)
    print(" Resonance V5 — Round 5 Unified Simulation")
    print("=" * 80)

    pool_rng = random.Random(42)
    pool = build_card_pool(pool_rng, pct_1sym=0.15, pct_2sym=0.60, pct_3sym=0.25)
    print(f"Pool: {len(pool)} cards")
    print(f"  1-sym: {sum(1 for c in pool if len(c.symbols)==1)}")
    print(f"  2-sym: {sum(1 for c in pool if len(c.symbols)==2)}")
    print(f"  3-sym: {sum(1 for c in pool if len(c.symbols)==3)}")
    print(f"  generic: {sum(1 for c in pool if not c.symbols)}")

    # Pair precision check
    pair_to_arch = {}
    for ai, (pri, sec) in ARCHETYPE_RESONANCES.items():
        pair_to_arch[(pri, sec)] = ai
    total_pair = 0
    s_count = 0
    for c in pool:
        p = get_card_pair(c)
        if p and p in pair_to_arch:
            total_pair += 1
            arch = pair_to_arch[p]
            if c.archetype_fitness.get(arch, Tier.F) == Tier.S:
                s_count += 1
    print(f"  Pair precision (S-tier): {s_count}/{total_pair} = {s_count/total_pair:.1%}")

    algorithms = {
        "D1 PairThresh": ("d1", {"threshold": 3, "bonus": 1, "scaling": True}),
        "D2 PairEsc(6,.65)": ("d2", {"K": 6, "cap": 0.65}),
        "D2 PairEsc(6,.50)": ("d2", {"K": 6, "cap": 0.50}),
        "D3 PoolSeed": ("d3", {"inject_rate": 4, "inject_threshold": 2}),
        "D4 DualThr(3,7)": ("d4", {"threshold1": 3, "threshold2": 7}),
        "D4 DualThr(2,5)": ("d4", {"threshold1": 2, "threshold2": 5}),
        "D5 HybridTrig": ("d5", {}),
        "LaneLocking": ("lane_locking", {}),
        "PackWidening": ("pack_widening", {"cost": 3, "bonus_count": 1}),
        "Random": ("random", {}),
    }

    all_metrics = {}
    base_seed = 1000

    for label, (algo, params) in algorithms.items():
        print(f"\nRunning {label}...", flush=True)
        m = run_full(pool, algo, base_seed, params=params, n_drafts=NUM_DRAFTS, label=label)
        all_metrics[label] = m
        print(f"  Late S/A: {m['late_sa']:.2f}, Conv: {m['conv_pick']:.1f}, "
              f"StdDev: {m['stddev']:.2f}, DeckConc: {m['deck_conc']:.1%}, "
              f"Off-arch: {m['late_off']:.2f}")

    # Comparison table
    print("\n\n" + "=" * 130)
    print(" UNIFIED COMPARISON TABLE (Archetype-Level, Committed Strategy)")
    print("=" * 130)

    header = f"{'Metric':40s} | {'Target':8s}"
    for label in all_metrics:
        header += f" | {label:>16s}"
    print(header)
    print("-" * len(header))

    defs = [
        ("Early unique archs", "early_unique", ">=3", lambda v: v >= 3),
        ("Early S/A emerging", "early_sa", "<=2", lambda v: v <= 2),
        ("Late S/A committed", "late_sa", ">=2", lambda v: v >= 2),
        ("Late off-arch (C/F)", "late_off", ">=0.5", lambda v: v >= 0.5),
        ("Convergence pick", "conv_pick", "5-8", lambda v: 5 <= v <= 8),
        ("Deck concentration", "deck_conc", "60-90%", lambda v: 0.60 <= v <= 0.90),
        ("Run-to-run overlap", "overlap", "<40%", lambda v: v < 0.40),
        ("S/A StdDev (late)", "stddev", ">=0.8", lambda v: v >= 0.8),
    ]

    for name, key, target, check in defs:
        row = f"{name:40s} | {target:8s}"
        for label in all_metrics:
            v = all_metrics[label][key]
            if "conc" in key or "overlap" in key:
                vs = f"{v:.1%}"
            elif "pick" in key:
                vs = f"{v:.1f}"
            else:
                vs = f"{v:.2f}"
            pf = "P" if check(v) else "F"
            row += f" | {vs:>12s}({pf})"
        print(row)

    # Pass counts
    print()
    row = f"{'Targets Passed':40s} | {'':8s}"
    for label in all_metrics:
        passes = sum(1 for _, key, _, check in defs if check(all_metrics[label][key]))
        row += f" | {passes:>14d}/8"
    print(row)

    # Per-archetype convergence for key algorithms
    print("\n\n" + "=" * 130)
    print(" PER-ARCHETYPE CONVERGENCE TABLE")
    print("=" * 130)

    key_algos = ["D2 PairEsc(6,.50)", "D4 DualThr(2,5)", "D4 DualThr(3,7)",
                 "LaneLocking", "PackWidening"]

    conv_results = {}
    for label in key_algos:
        algo, params = algorithms[label]
        print(f"Computing per-archetype convergence for {label}...", flush=True)
        conv_results[label] = compute_per_arch_conv(pool, random.Random(2000),
                                                     algo, params, n_drafts=200)

    header = f"{'Archetype':30s}"
    for label in key_algos:
        header += f" | {label:>16s}"
    print(header)
    print("-" * len(header))

    for ai in range(NUM_ARCHETYPES):
        row = f"{ARCHETYPES[ai]:30s}"
        for label in key_algos:
            v = conv_results[label][ai]
            row += f" | {v:>16.1f}"
        print(row)

    # Averages
    row = f"{'AVERAGE':30s}"
    for label in key_algos:
        avg = statistics.mean(conv_results[label].values())
        row += f" | {avg:>16.1f}"
    print(row)

    print("\n\nSimulation complete.")
