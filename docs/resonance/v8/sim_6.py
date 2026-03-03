#!/usr/bin/env python3
"""
Simulation Agent 6: Guaranteed Floor + Pair Escalation (GF+PE)

Algorithm: Monotonic permanent pair-escalation with a guaranteed floor.
- Picks 1-2: fully random packs (4 cards).
- Pick 3+: 1 pair-matched slot guaranteed (floor).
- When pair counter >= T1: 2 pair-matched slots (permanent).
- When pair counter >= T2: 3 pair-matched slots (permanent).
- 20% jitter: each pair-matched slot has 20% chance of being random instead.
- Quality never drops -- escalation is permanent once thresholds are crossed.

Pair counter: each pick adds +2 if card's primary resonance matches leading
pair's R1, +1 if secondary resonance matches R2.
"""

import random
import math
from collections import defaultdict
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

# ─── Constants ───────────────────────────────────────────────────────────

NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 4
NUM_ARCHETYPES = 8
POOL_SIZE = 360
JITTER_RATE = 0.20  # 20% chance a pair-matched slot becomes random

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

ARCHETYPES = [
    {"name": "Flash/Tempo",         "primary": "Zephyr", "secondary": "Ember",  "index": 0},
    {"name": "Blink/Flicker",       "primary": "Ember",  "secondary": "Zephyr", "index": 1},
    {"name": "Storm/Spellslinger",  "primary": "Ember",  "secondary": "Stone",  "index": 2},
    {"name": "Self-Discard",        "primary": "Stone",  "secondary": "Ember",  "index": 3},
    {"name": "Self-Mill/Reanimator","primary": "Stone",  "secondary": "Tide",   "index": 4},
    {"name": "Sacrifice/Abandon",   "primary": "Tide",   "secondary": "Stone",  "index": 5},
    {"name": "Warriors/Midrange",   "primary": "Tide",   "secondary": "Zephyr", "index": 6},
    {"name": "Ramp/Spirit Animals", "primary": "Zephyr", "secondary": "Tide",   "index": 7},
]

# Co-primary sibling pairs (share primary resonance)
SIBLING_MAP = {
    0: 7,  # Flash <-> Ramp (Zephyr)
    7: 0,
    1: 2,  # Blink <-> Storm (Ember)
    2: 1,
    3: 4,  # Self-Discard <-> Self-Mill (Stone)
    4: 3,
    5: 6,  # Sacrifice <-> Warriors (Tide)
    6: 5,
}

# ─── Fitness Models ──────────────────────────────────────────────────────

# Per-pair sibling A-tier rates for graduated models
# Index by frozenset of archetype indices
PAIR_FITNESS = {
    "optimistic": {
        frozenset({5, 6}): 1.00,  # Warriors/Sacrifice
        frozenset({3, 4}): 1.00,  # Self-Discard/Self-Mill
        frozenset({1, 2}): 1.00,  # Blink/Storm
        frozenset({0, 7}): 1.00,  # Flash/Ramp
    },
    "graduated_realistic": {
        frozenset({5, 6}): 0.50,  # Warriors/Sacrifice
        frozenset({3, 4}): 0.40,  # Self-Discard/Self-Mill
        frozenset({1, 2}): 0.30,  # Blink/Storm
        frozenset({0, 7}): 0.25,  # Flash/Ramp
    },
    "pessimistic": {
        frozenset({5, 6}): 0.35,  # Warriors/Sacrifice
        frozenset({3, 4}): 0.25,  # Self-Discard/Self-Mill
        frozenset({1, 2}): 0.15,  # Blink/Storm
        frozenset({0, 7}): 0.10,  # Flash/Ramp
    },
    "hostile": {
        frozenset({5, 6}): 0.08,
        frozenset({3, 4}): 0.08,
        frozenset({1, 2}): 0.08,
        frozenset({0, 7}): 0.08,
    },
}


def get_sibling_fitness(fitness_model: str, arch_idx: int) -> float:
    """Get the sibling A-tier rate for a given archetype under a fitness model."""
    sibling_idx = SIBLING_MAP[arch_idx]
    pair_key = frozenset({arch_idx, sibling_idx})
    return PAIR_FITNESS[fitness_model][pair_key]


# ─── Card and Pool ───────────────────────────────────────────────────────

@dataclass
class Card:
    card_id: int
    symbols: list  # ordered: [primary, secondary, ...] resonance strings
    home_archetype: int  # index into ARCHETYPES
    power: float  # 0-10

    def pair_key(self):
        """Return (R1, R2) pair if card has 2+ symbols, else (R1, None)."""
        if len(self.symbols) >= 2:
            return (self.symbols[0], self.symbols[1])
        elif len(self.symbols) == 1:
            return (self.symbols[0], None)
        return (None, None)


def build_pool(pool_type: str) -> list:
    """
    Build a 360-card pool.
    pool_type: "v7_standard" (15% dual-res) or "enriched_40" (40% dual-res)
    """
    cards = []
    card_id = 0

    if pool_type == "v7_standard":
        # V7: 40 cards per archetype, 36 generic. ~15% dual-res (54 cards).
        # Single-res: 270, Dual-res: 54, Generic: 36
        for arch in ARCHETYPES:
            arch_idx = arch["index"]
            r1 = arch["primary"]
            r2 = arch["secondary"]
            # 33 single-res cards (R1 only)
            for _ in range(33):
                cards.append(Card(card_id, [r1], arch_idx, random.uniform(3, 8)))
                card_id += 1
            # ~7 dual-res cards per archetype (56 total ~15%)
            for _ in range(7):
                cards.append(Card(card_id, [r1, r2], arch_idx, random.uniform(3, 8)))
                card_id += 1
        # 36 generic
        for _ in range(36):
            cards.append(Card(card_id, [], random.randint(0, 7), random.uniform(2, 6)))
            card_id += 1

    elif pool_type == "enriched_40":
        # 40% Enriched: 40 cards per archetype, 40 generic.
        # 22 home-only (single-res), 18 cross-archetype (dual/tri-res), 40 generic
        for arch in ARCHETYPES:
            arch_idx = arch["index"]
            r1 = arch["primary"]
            r2 = arch["secondary"]
            # Determine a third resonance for tri-symbol cards
            others = [r for r in RESONANCES if r != r1 and r != r2]
            r3 = others[0]  # deterministic third

            # 4 single-res cards
            for _ in range(4):
                cards.append(Card(card_id, [r1], arch_idx, random.uniform(3, 8)))
                card_id += 1
            # 18 dual-res (R1, R2) -- cross-archetype
            for _ in range(18):
                cards.append(Card(card_id, [r1, r2], arch_idx, random.uniform(3, 8)))
                card_id += 1
            # 18 tri-res (R1, R2, R3)
            for _ in range(18):
                cards.append(Card(card_id, [r1, r2, r3], arch_idx, random.uniform(3, 8)))
                card_id += 1
        # 40 generic
        for _ in range(40):
            cards.append(Card(card_id, [], random.randint(0, 7), random.uniform(2, 6)))
            card_id += 1

    return cards


def card_tier_for_archetype(card: Card, target_arch_idx: int, fitness_model: str) -> str:
    """
    Determine tier of a card for a target archetype.
    Returns 'S', 'A', 'B', 'C', or 'F'.
    """
    if card.home_archetype == target_arch_idx:
        return 'S'

    # Check if card's home archetype is a co-primary sibling
    if SIBLING_MAP.get(card.home_archetype) == target_arch_idx:
        fitness_rate = get_sibling_fitness(fitness_model, target_arch_idx)
        if random.random() < fitness_rate:
            return 'A'
        else:
            # Low quality for non-fit sibling card
            return random.choice(['B', 'C'])

    # Not home, not sibling -> generic or off-archetype
    # Generic cards (no symbols) get B tier sometimes
    if len(card.symbols) == 0:
        return random.choice(['B', 'C'])

    return random.choice(['C', 'F'])


def is_sa_tier(tier: str) -> bool:
    return tier in ('S', 'A')


# ─── Pre-compute card fitness assignments ────────────────────────────────

def precompute_fitness(pool: list, fitness_model: str) -> dict:
    """
    Pre-compute tier assignments for all card-archetype pairs.
    Returns dict: (card_id, arch_idx) -> tier
    """
    assignments = {}
    for card in pool:
        for arch_idx in range(NUM_ARCHETYPES):
            assignments[(card.card_id, arch_idx)] = card_tier_for_archetype(
                card, arch_idx, fitness_model
            )
    return assignments


# ─── GF+PE Algorithm ─────────────────────────────────────────────────────

def generate_pack_gfpe(
    pool: list,
    pair_counters: dict,
    max_pair_reached: dict,
    pick_number: int,
    t1: int,
    t2: int,
    rng: random.Random,
) -> list:
    """
    Generate a pack using GF+PE algorithm.

    pair_counters: {(R1, R2): int} -- accumulated pair affinity
    max_pair_reached: {"pair": (R1,R2), "level": int} -- highest level reached
    pick_number: 1-indexed
    """
    pack = []

    if pick_number <= 2:
        # Fully random
        pack = rng.sample(pool, PACK_SIZE)
        return pack

    # Find leading pair
    leading_pair = max(pair_counters, key=pair_counters.get) if pair_counters else None
    leading_count = pair_counters.get(leading_pair, 0) if leading_pair else 0

    # Compute current escalation level (monotonic)
    current_level = 1  # guaranteed floor: 1 pair-matched slot
    if leading_count >= t1:
        current_level = 2
    if leading_count >= t2:
        current_level = 3

    # Update max level (monotonic -- never decreases)
    if current_level > max_pair_reached.get("level", 0):
        max_pair_reached["level"] = current_level
        max_pair_reached["pair"] = leading_pair

    active_level = max_pair_reached.get("level", 1)
    active_pair = max_pair_reached.get("pair", leading_pair)

    # Build pair-filtered subpool
    if active_pair:
        r1_target, r2_target = active_pair
        pair_pool = [
            c for c in pool
            if len(c.symbols) >= 2 and c.symbols[0] == r1_target and c.symbols[1] == r2_target
        ]
        # Fallback: if pair pool too small, use R1-only filter
        if len(pair_pool) < 4:
            pair_pool = [c for c in pool if len(c.symbols) >= 1 and c.symbols[0] == r1_target]
    else:
        pair_pool = pool

    # Fill pair-matched slots (with jitter)
    used_ids = set()
    pair_slots = 0
    for _ in range(active_level):
        if rng.random() < JITTER_RATE:
            # Jitter: use random card instead
            candidates = [c for c in pool if c.card_id not in used_ids]
            if candidates:
                chosen = rng.choice(candidates)
                pack.append(chosen)
                used_ids.add(chosen.card_id)
        else:
            candidates = [c for c in pair_pool if c.card_id not in used_ids]
            if candidates:
                chosen = rng.choice(candidates)
                pack.append(chosen)
                used_ids.add(chosen.card_id)
                pair_slots += 1
            else:
                # Pool exhaustion fallback
                candidates = [c for c in pool if c.card_id not in used_ids]
                if candidates:
                    chosen = rng.choice(candidates)
                    pack.append(chosen)
                    used_ids.add(chosen.card_id)

    # Fill remaining slots randomly
    remaining = PACK_SIZE - len(pack)
    random_candidates = [c for c in pool if c.card_id not in used_ids]
    if len(random_candidates) >= remaining:
        fills = rng.sample(random_candidates, remaining)
    else:
        fills = random_candidates
    pack.extend(fills)

    return pack[:PACK_SIZE]


def update_pair_counters(pair_counters: dict, card: Card):
    """Update pair counters based on the drafted card's symbols."""
    if len(card.symbols) >= 1:
        r1 = card.symbols[0]
        # Find all pairs where R1 matches
        for (pr1, pr2) in list(pair_counters.keys()):
            if pr1 == r1:
                pair_counters[(pr1, pr2)] += 2
            elif pr2 == r1:
                pair_counters[(pr1, pr2)] += 1
        if len(card.symbols) >= 2:
            r2 = card.symbols[1]
            for (pr1, pr2) in list(pair_counters.keys()):
                if pr2 == r2 and pr1 != r1:
                    pair_counters[(pr1, pr2)] += 1


# ─── Player Strategies ───────────────────────────────────────────────────

def pick_archetype_committed(pack: list, target_arch: int, fitness: dict) -> Card:
    """Pick the card with highest fitness for target archetype."""
    tier_order = {'S': 4, 'A': 3, 'B': 2, 'C': 1, 'F': 0}
    best = max(pack, key=lambda c: (
        tier_order.get(fitness[(c.card_id, target_arch)], 0),
        c.power
    ))
    return best


def pick_power_chaser(pack: list, fitness: dict) -> Card:
    """Pick highest raw power regardless of archetype."""
    return max(pack, key=lambda c: c.power)


def pick_signal_reader(pack: list, pair_counters: dict, fitness: dict) -> Card:
    """Pick best card for the currently leading pair's archetype."""
    leading_pair = max(pair_counters, key=pair_counters.get) if pair_counters else None
    if leading_pair is None:
        return max(pack, key=lambda c: c.power)

    # Find archetype matching the leading pair
    r1, r2 = leading_pair
    target_arch = None
    for arch in ARCHETYPES:
        if arch["primary"] == r1 and arch["secondary"] == r2:
            target_arch = arch["index"]
            break
    if target_arch is None:
        return max(pack, key=lambda c: c.power)

    tier_order = {'S': 4, 'A': 3, 'B': 2, 'C': 1, 'F': 0}
    return max(pack, key=lambda c: (
        tier_order.get(fitness[(c.card_id, target_arch)], 0),
        c.power
    ))


# ─── Draft Simulation ────────────────────────────────────────────────────

@dataclass
class DraftResult:
    picks: list  # list of (card, pack, pick_number)
    pack_sa_counts: list  # per-pack S/A count for committed archetype
    target_arch: int
    convergence_pick: int
    deck_concentration: float
    pack_details: list  # list of dicts with per-pack info


def simulate_draft(
    pool: list,
    fitness: dict,
    strategy: str,
    t1: int,
    t2: int,
    seed: int,
    target_arch: Optional[int] = None,
) -> DraftResult:
    rng = random.Random(seed)

    # Initialize pair counters for all 8 archetype pairs
    pair_counters = {}
    for arch in ARCHETYPES:
        pair_counters[(arch["primary"], arch["secondary"])] = 0

    max_pair_reached = {"level": 0, "pair": None}
    picks = []
    pack_sa_counts = []
    pack_details = []

    # For archetype-committed, choose target archetype
    if strategy == "committed":
        if target_arch is None:
            target_arch = rng.randint(0, 7)
    elif strategy == "power":
        target_arch = None  # determined later
    elif strategy == "signal":
        target_arch = None  # evolves during draft

    committed_arch = target_arch
    convergence_pick = 30  # default: never converged

    for pick_num in range(1, NUM_PICKS + 1):
        pack = generate_pack_gfpe(pool, pair_counters, max_pair_reached, pick_num, t1, t2, rng)

        # Choose card based on strategy
        if strategy == "committed":
            chosen = pick_archetype_committed(pack, committed_arch, fitness)
        elif strategy == "power":
            chosen = pick_power_chaser(pack, fitness)
        elif strategy == "signal":
            chosen = pick_signal_reader(pack, pair_counters, fitness)
            # After pick 5, commit to leading pair's archetype
            if pick_num >= 5 and committed_arch is None:
                leading_pair = max(pair_counters, key=pair_counters.get)
                if leading_pair:
                    r1, r2 = leading_pair
                    for arch in ARCHETYPES:
                        if arch["primary"] == r1 and arch["secondary"] == r2:
                            committed_arch = arch["index"]
                            break

        picks.append((chosen, pack, pick_num))
        update_pair_counters(pair_counters, chosen)

        # For power-chaser: determine most-drafted archetype after pick 5
        if strategy == "power" and pick_num == 5:
            arch_counts = defaultdict(int)
            for c, _, _ in picks:
                arch_counts[c.home_archetype] += 1
            committed_arch = max(arch_counts, key=arch_counts.get)

        # Compute pack S/A for evaluation archetype
        eval_arch = committed_arch if committed_arch is not None else 0
        sa_count = sum(1 for c in pack if is_sa_tier(fitness[(c.card_id, eval_arch)]))
        pack_sa_counts.append(sa_count)

        # Track convergence: first pick where 2+ S/A in pack
        if pick_num >= 3 and sa_count >= 2 and convergence_pick == 30:
            convergence_pick = pick_num

        pack_details.append({
            "pick": pick_num,
            "sa_count": sa_count,
            "level": max_pair_reached.get("level", 0),
            "pack_cards": [(c.card_id, fitness[(c.card_id, eval_arch)]) for c in pack],
        })

    # Compute deck concentration
    if committed_arch is not None:
        sa_in_deck = sum(
            1 for c, _, _ in picks
            if is_sa_tier(fitness[(c.card_id, committed_arch)])
        )
        deck_concentration = sa_in_deck / NUM_PICKS
    else:
        deck_concentration = 0.0

    return DraftResult(
        picks=picks,
        pack_sa_counts=pack_sa_counts,
        target_arch=committed_arch if committed_arch is not None else 0,
        convergence_pick=convergence_pick,
        deck_concentration=deck_concentration,
        pack_details=pack_details,
    )


# ─── Metrics Computation ─────────────────────────────────────────────────

def compute_metrics(results: list, pool: list, fitness: dict) -> dict:
    """Compute all 10 metrics from a list of DraftResults."""
    m1_vals = []
    m2_vals = []
    m3_vals = []
    m4_vals = []
    m5_vals = []
    m6_vals = []
    m9_vals = []
    m10_vals = []
    pack_sa_post6 = []
    consec_bad_streaks = []

    for dr in results:
        arch = dr.target_arch
        # M1: Picks 1-5: unique archetypes with S/A cards per pack
        early_archs_per_pack = []
        for pd in dr.pack_details[:5]:
            archs_with_sa = set()
            for cid, tier in pd["pack_cards"]:
                if tier in ('S', 'A'):
                    # find the card's home archetype
                    card_obj = None
                    for c in pool:
                        if c.card_id == cid:
                            card_obj = c
                            break
                    if card_obj:
                        archs_with_sa.add(card_obj.home_archetype)
            early_archs_per_pack.append(len(archs_with_sa))
        m1_vals.append(sum(early_archs_per_pack) / max(len(early_archs_per_pack), 1))

        # M2: Picks 1-5: S/A cards for emerging archetype per pack
        emerging_sa = []
        for pd in dr.pack_details[:5]:
            sa = sum(1 for _, t in pd["pack_cards"] if t in ('S', 'A'))
            emerging_sa.append(sa)
        m2_vals.append(sum(emerging_sa) / max(len(emerging_sa), 1))

        # M3: Picks 6+: avg S/A per pack for committed archetype
        post6_sa = [pd["sa_count"] for pd in dr.pack_details[5:]]
        if post6_sa:
            m3_vals.append(sum(post6_sa) / len(post6_sa))
            pack_sa_post6.extend(post6_sa)
        else:
            m3_vals.append(0)

        # M4: Picks 6+: off-archetype (C/F) per pack
        post6_cf = []
        for pd in dr.pack_details[5:]:
            cf = sum(1 for _, t in pd["pack_cards"] if t in ('C', 'F'))
            post6_cf.append(cf)
        m4_vals.append(sum(post6_cf) / max(len(post6_cf), 1))

        # M5: Convergence pick
        m5_vals.append(dr.convergence_pick)

        # M6: Deck concentration
        m6_vals.append(dr.deck_concentration)

        # M9: StdDev of S/A per pack (picks 6+)
        if len(post6_sa) > 1:
            mean_sa = sum(post6_sa) / len(post6_sa)
            var = sum((x - mean_sa) ** 2 for x in post6_sa) / len(post6_sa)
            m9_vals.append(math.sqrt(var))
        else:
            m9_vals.append(0)

        # M10: Max consecutive packs below 1.5 S/A (picks 6+)
        max_streak = 0
        current_streak = 0
        all_streaks = []
        for sa in post6_sa:
            if sa < 1.5:
                current_streak += 1
            else:
                if current_streak > 0:
                    all_streaks.append(current_streak)
                current_streak = 0
            max_streak = max(max_streak, current_streak)
        if current_streak > 0:
            all_streaks.append(current_streak)
        m10_vals.append(max_streak)
        consec_bad_streaks.append((
            sum(all_streaks) / max(len(all_streaks), 1) if all_streaks else 0,
            max_streak
        ))

    # M7: Run-to-run variety (card overlap between consecutive drafts)
    m7_overlaps = []
    for i in range(len(results) - 1):
        ids_a = set(c.card_id for c, _, _ in results[i].picks)
        ids_b = set(c.card_id for c, _, _ in results[i + 1].picks)
        overlap = len(ids_a & ids_b) / max(len(ids_a | ids_b), 1)
        m7_overlaps.append(overlap)
    m7 = sum(m7_overlaps) / max(len(m7_overlaps), 1) if m7_overlaps else 0

    # M8: Archetype frequency
    arch_counts = defaultdict(int)
    for dr in results:
        arch_counts[dr.target_arch] += 1
    total = sum(arch_counts.values())
    arch_freqs = {a: arch_counts[a] / total for a in range(8)}
    m8_max = max(arch_freqs.values()) if arch_freqs else 0
    m8_min = min(arch_freqs.values()) if arch_freqs else 0

    # Pack quality distribution percentiles (picks 6+)
    pack_sa_post6_sorted = sorted(pack_sa_post6)
    n = len(pack_sa_post6_sorted)

    def percentile(data, p):
        if not data:
            return 0
        k = (len(data) - 1) * p / 100
        f = math.floor(k)
        c = math.ceil(k)
        if f == c:
            return data[int(k)]
        return data[f] * (c - k) + data[c] * (k - f)

    pct = {}
    for p in [10, 25, 50, 75, 90]:
        pct[p] = percentile(pack_sa_post6_sorted, p)

    # Consecutive bad pack analysis
    avg_bad_streak = sum(s[0] for s in consec_bad_streaks) / max(len(consec_bad_streaks), 1)
    worst_bad_streak = max(s[1] for s in consec_bad_streaks) if consec_bad_streaks else 0

    # Per-archetype M3
    per_arch_m3 = defaultdict(list)
    for dr in results:
        post6 = [pd["sa_count"] for pd in dr.pack_details[5:]]
        if post6:
            per_arch_m3[dr.target_arch].append(sum(post6) / len(post6))

    return {
        "M1": sum(m1_vals) / len(m1_vals),
        "M2": sum(m2_vals) / len(m2_vals),
        "M3": sum(m3_vals) / len(m3_vals),
        "M4": sum(m4_vals) / len(m4_vals),
        "M5": sum(m5_vals) / len(m5_vals),
        "M6": sum(m6_vals) / len(m6_vals),
        "M7": m7,
        "M8_max": m8_max,
        "M8_min": m8_min,
        "M9": sum(m9_vals) / len(m9_vals),
        "M10": sum(m10_vals) / len(m10_vals),
        "M10_worst": max(m10_vals) if m10_vals else 0,
        "pack_pct": pct,
        "avg_bad_streak": avg_bad_streak,
        "worst_bad_streak": worst_bad_streak,
        "per_arch_m3": {
            a: sum(vals) / len(vals) if vals else 0
            for a, vals in per_arch_m3.items()
        },
        "arch_freqs": arch_freqs,
    }


# ─── Baseline: Surge+Floor (for comparison) ──────────────────────────────

def generate_pack_surge_floor(
    pool: list,
    surge_counter: int,
    pick_number: int,
    target_pair: tuple,
    rng: random.Random,
    surge_threshold: int = 3,
    surge_slots: int = 3,
    floor_start: int = 3,
) -> tuple:
    """Surge+Floor baseline (V7 algorithm)."""
    pack = []

    if pick_number < floor_start:
        return rng.sample(pool, PACK_SIZE), surge_counter

    # Check surge
    is_surge = surge_counter >= surge_threshold
    if is_surge:
        surge_counter = 0
    else:
        surge_counter += 1

    r1_target = target_pair[0] if target_pair else None

    if is_surge and r1_target:
        # Fill surge_slots with R1-filtered cards
        r1_pool = [c for c in pool if len(c.symbols) >= 1 and c.symbols[0] == r1_target]
        used = set()
        for _ in range(surge_slots):
            cands = [c for c in r1_pool if c.card_id not in used]
            if cands:
                ch = rng.choice(cands)
                pack.append(ch)
                used.add(ch.card_id)
        # Fill remainder
        rem = PACK_SIZE - len(pack)
        rcands = [c for c in pool if c.card_id not in used]
        pack.extend(rng.sample(rcands, min(rem, len(rcands))))
    else:
        # Floor: 1 R1 slot
        if r1_target:
            r1_pool = [c for c in pool if len(c.symbols) >= 1 and c.symbols[0] == r1_target]
            if r1_pool:
                pack.append(rng.choice(r1_pool))
                used = {pack[0].card_id}
                rcands = [c for c in pool if c.card_id not in used]
                pack.extend(rng.sample(rcands, min(PACK_SIZE - 1, len(rcands))))
            else:
                pack = rng.sample(pool, PACK_SIZE)
        else:
            pack = rng.sample(pool, PACK_SIZE)

    return pack[:PACK_SIZE], surge_counter


def simulate_draft_baseline(
    pool: list,
    fitness: dict,
    t1: int = 3,
    seed: int = 0,
    target_arch: Optional[int] = None,
) -> DraftResult:
    """Simulate Surge+Floor baseline for comparison."""
    rng = random.Random(seed)
    if target_arch is None:
        target_arch = rng.randint(0, 7)

    arch = ARCHETYPES[target_arch]
    target_pair = (arch["primary"], arch["secondary"])
    surge_counter = 0
    picks = []
    pack_sa_counts = []
    pack_details = []
    convergence_pick = 30

    for pick_num in range(1, NUM_PICKS + 1):
        pack, surge_counter = generate_pack_surge_floor(
            pool, surge_counter, pick_num, target_pair, rng,
            surge_threshold=t1, surge_slots=3, floor_start=3,
        )
        chosen = pick_archetype_committed(pack, target_arch, fitness)
        picks.append((chosen, pack, pick_num))

        sa_count = sum(1 for c in pack if is_sa_tier(fitness[(c.card_id, target_arch)]))
        pack_sa_counts.append(sa_count)

        if pick_num >= 3 and sa_count >= 2 and convergence_pick == 30:
            convergence_pick = pick_num

        pack_details.append({
            "pick": pick_num,
            "sa_count": sa_count,
            "level": 0,
            "pack_cards": [(c.card_id, fitness[(c.card_id, target_arch)]) for c in pack],
        })

    sa_in_deck = sum(1 for c, _, _ in picks if is_sa_tier(fitness[(c.card_id, target_arch)]))

    return DraftResult(
        picks=picks,
        pack_sa_counts=pack_sa_counts,
        target_arch=target_arch,
        convergence_pick=convergence_pick,
        deck_concentration=sa_in_deck / NUM_PICKS,
        pack_details=pack_details,
    )


# ─── Main Simulation ─────────────────────────────────────────────────────

def run_all():
    random.seed(42)

    fitness_models = ["optimistic", "graduated_realistic", "pessimistic", "hostile"]
    pool_types = ["v7_standard", "enriched_40"]
    strategies = ["committed", "power", "signal"]

    # Default parameters
    default_t1 = 4
    default_t2 = 8

    all_results = {}

    print("=" * 80)
    print("GF+PE SIMULATION — Agent 6")
    print("=" * 80)

    for pool_type in pool_types:
        pool = build_pool(pool_type)
        pool_id_map = {c.card_id: c for c in pool}

        for fm in fitness_models:
            fitness = precompute_fitness(pool, fm)

            for strat in strategies:
                key = (pool_type, fm, strat)
                print(f"\nRunning: pool={pool_type}, fitness={fm}, strategy={strat}")

                results = []
                for d in range(NUM_DRAFTS):
                    seed = d * 1000 + hash(key) % 10000
                    # For committed, cycle through archetypes evenly
                    if strat == "committed":
                        tgt = d % 8
                    else:
                        tgt = None
                    dr = simulate_draft(pool, fitness, strat, default_t1, default_t2, seed, tgt)
                    results.append(dr)

                metrics = compute_metrics(results, pool, fitness)
                all_results[key] = metrics

                print(f"  M1={metrics['M1']:.2f}  M2={metrics['M2']:.2f}  "
                      f"M3={metrics['M3']:.2f}  M4={metrics['M4']:.2f}  "
                      f"M5={metrics['M5']:.1f}  M6={metrics['M6']:.2f}  "
                      f"M7={metrics['M7']:.2f}  M9={metrics['M9']:.2f}  "
                      f"M10={metrics['M10']:.2f}")
                print(f"  Pack pct: {metrics['pack_pct']}")
                print(f"  Consec bad: avg={metrics['avg_bad_streak']:.2f}, "
                      f"worst={metrics['worst_bad_streak']}")

    # ─── Baseline comparison (Surge+Floor, committed only) ──────────
    print("\n" + "=" * 80)
    print("BASELINE COMPARISON: Surge+Floor (T=3)")
    print("=" * 80)

    baseline_results = {}
    for pool_type in pool_types:
        pool = build_pool(pool_type)
        for fm in fitness_models:
            fitness = precompute_fitness(pool, fm)
            bkey = (pool_type, fm, "baseline_sf")
            results = []
            for d in range(NUM_DRAFTS):
                seed = d * 2000 + 9999
                tgt = d % 8
                dr = simulate_draft_baseline(pool, fitness, t1=3, seed=seed, target_arch=tgt)
                results.append(dr)
            metrics = compute_metrics(results, pool, fitness)
            baseline_results[bkey] = metrics
            print(f"\nBaseline: pool={pool_type}, fitness={fm}")
            print(f"  M3={metrics['M3']:.2f}  M10={metrics['M10']:.2f}  "
                  f"worst_streak={metrics['worst_bad_streak']}")

    # ─── Parameter sensitivity sweep ──────────────────────────────────
    print("\n" + "=" * 80)
    print("PARAMETER SENSITIVITY SWEEP")
    print("=" * 80)

    sweep_results = {}
    pool = build_pool("enriched_40")
    fm = "graduated_realistic"
    fitness = precompute_fitness(pool, fm)

    for t1_val in [3, 4, 5]:
        for t2_val in [6, 7, 8, 9]:
            if t2_val <= t1_val:
                continue
            results = []
            for d in range(NUM_DRAFTS):
                seed = d * 3000 + t1_val * 100 + t2_val
                tgt = d % 8
                dr = simulate_draft(pool, fitness, "committed", t1_val, t2_val, seed, tgt)
                results.append(dr)
            metrics = compute_metrics(results, pool, fitness)
            sweep_results[(t1_val, t2_val)] = metrics
            print(f"  T1={t1_val}, T2={t2_val}: M3={metrics['M3']:.2f}, "
                  f"M5={metrics['M5']:.1f}, M9={metrics['M9']:.2f}, "
                  f"M10={metrics['M10']:.2f}")

    # Jitter sweep
    print("\n  Jitter sweep (T1=4, T2=8):")
    jitter_results = {}
    for jitter_pct in [0.0, 0.10, 0.15, 0.20, 0.25, 0.30]:
        global JITTER_RATE
        JITTER_RATE = jitter_pct
        results = []
        for d in range(NUM_DRAFTS):
            seed = d * 4000 + int(jitter_pct * 100)
            tgt = d % 8
            dr = simulate_draft(pool, fitness, "committed", 4, 8, seed, tgt)
            results.append(dr)
        metrics = compute_metrics(results, pool, fitness)
        jitter_results[jitter_pct] = metrics
        print(f"    Jitter={jitter_pct:.0%}: M3={metrics['M3']:.2f}, "
              f"M9={metrics['M9']:.2f}, M10={metrics['M10']:.2f}")
    JITTER_RATE = 0.20  # Reset

    # ─── Per-archetype convergence table ──────────────────────────────
    print("\n" + "=" * 80)
    print("PER-ARCHETYPE CONVERGENCE TABLE (Enriched 40%, Grad. Realistic, Committed)")
    print("=" * 80)

    pool = build_pool("enriched_40")
    fitness = precompute_fitness(pool, "graduated_realistic")
    per_arch_results = defaultdict(list)

    for d in range(NUM_DRAFTS):
        tgt = d % 8
        seed = d * 5000
        dr = simulate_draft(pool, fitness, "committed", default_t1, default_t2, seed, tgt)
        per_arch_results[tgt].append(dr)

    print(f"{'Archetype':<25} {'M3':>6} {'M5':>6} {'M6':>6} {'M9':>6} {'M10':>6}")
    for arch_idx in range(8):
        drafts = per_arch_results[arch_idx]
        if not drafts:
            continue
        m3s = []
        m5s = []
        m6s = []
        m9s = []
        m10s = []
        for dr in drafts:
            post6 = [pd["sa_count"] for pd in dr.pack_details[5:]]
            if post6:
                m3s.append(sum(post6) / len(post6))
                mean_sa = sum(post6) / len(post6)
                var = sum((x - mean_sa) ** 2 for x in post6) / len(post6)
                m9s.append(math.sqrt(var))
                max_streak = 0
                streak = 0
                for sa in post6:
                    if sa < 1.5:
                        streak += 1
                        max_streak = max(max_streak, streak)
                    else:
                        streak = 0
                m10s.append(max_streak)
            m5s.append(dr.convergence_pick)
            m6s.append(dr.deck_concentration)

        name = ARCHETYPES[arch_idx]["name"]
        print(f"{name:<25} {sum(m3s)/len(m3s):6.2f} {sum(m5s)/len(m5s):6.1f} "
              f"{sum(m6s)/len(m6s):6.2f} {sum(m9s)/len(m9s):6.2f} "
              f"{sum(m10s)/len(m10s):6.2f}")

    # ─── Fitness degradation curve ────────────────────────────────────
    print("\n" + "=" * 80)
    print("FITNESS DEGRADATION CURVE (Enriched 40%, Committed)")
    print("=" * 80)

    pool = build_pool("enriched_40")
    print(f"{'Fitness Model':<25} {'M3':>6} {'M10_avg':>8} {'M10_worst':>10} {'Worst Streak':>12}")
    for fm in fitness_models:
        fitness = precompute_fitness(pool, fm)
        results = []
        for d in range(NUM_DRAFTS):
            seed = d * 6000
            tgt = d % 8
            dr = simulate_draft(pool, fitness, "committed", default_t1, default_t2, seed, tgt)
            results.append(dr)
        metrics = compute_metrics(results, pool, fitness)
        print(f"{fm:<25} {metrics['M3']:6.2f} {metrics['M10']:8.2f} "
              f"{metrics['M10_worst']:10d} {metrics['worst_bad_streak']:12d}")

    # ─── Draft traces ─────────────────────────────────────────────────
    print("\n" + "=" * 80)
    print("DRAFT TRACES (Enriched 40%, Grad. Realistic)")
    print("=" * 80)

    pool = build_pool("enriched_40")
    fitness = precompute_fitness(pool, "graduated_realistic")

    # Trace 1: Early committer (Warriors, committed)
    print("\n--- Trace 1: Early Committer (Warriors, committed) ---")
    dr = simulate_draft(pool, fitness, "committed", default_t1, default_t2, seed=77777, target_arch=6)
    for pd in dr.pack_details:
        tiers = [t for _, t in pd["pack_cards"]]
        sa = sum(1 for t in tiers if t in ('S', 'A'))
        print(f"  Pick {pd['pick']:2d}: Level={pd['level']}, S/A={sa}, Tiers={tiers}")

    # Trace 2: Flexible player (signal reader)
    print("\n--- Trace 2: Flexible Player (signal reader) ---")
    dr = simulate_draft(pool, fitness, "signal", default_t1, default_t2, seed=88888)
    for pd in dr.pack_details:
        tiers = [t for _, t in pd["pack_cards"]]
        sa = sum(1 for t in tiers if t in ('S', 'A'))
        print(f"  Pick {pd['pick']:2d}: Level={pd['level']}, S/A={sa}, Tiers={tiers}")
    print(f"  Converged to archetype: {ARCHETYPES[dr.target_arch]['name']}")

    # Trace 3: Flash player (worst-case pair)
    print("\n--- Trace 3: Flash Player (worst-case pair, committed) ---")
    dr = simulate_draft(pool, fitness, "committed", default_t1, default_t2, seed=99999, target_arch=0)
    for pd in dr.pack_details:
        tiers = [t for _, t in pd["pack_cards"]]
        sa = sum(1 for t in tiers if t in ('S', 'A'))
        print(f"  Pick {pd['pick']:2d}: Level={pd['level']}, S/A={sa}, Tiers={tiers}")

    # ─── Summary output ───────────────────────────────────────────────
    print("\n" + "=" * 80)
    print("SUMMARY TABLE: GF+PE vs Baseline (Committed Strategy)")
    print("=" * 80)

    print(f"\n{'Config':<45} {'Algo':<10} {'M3':>6} {'M5':>6} {'M9':>6} "
          f"{'M10':>6} {'Worst':>6} {'M6':>6}")
    for pool_type in pool_types:
        for fm in fitness_models:
            gfpe_key = (pool_type, fm, "committed")
            bl_key = (pool_type, fm, "baseline_sf")
            gm = all_results.get(gfpe_key, {})
            bm = baseline_results.get(bl_key, {})
            config = f"{pool_type}/{fm}"
            if gm:
                print(f"{config:<45} {'GF+PE':<10} {gm.get('M3',0):6.2f} "
                      f"{gm.get('M5',0):6.1f} {gm.get('M9',0):6.2f} "
                      f"{gm.get('M10',0):6.2f} {gm.get('worst_bad_streak',0):6d} "
                      f"{gm.get('M6',0):6.2f}")
            if bm:
                print(f"{'':<45} {'S+F':<10} {bm.get('M3',0):6.2f} "
                      f"{bm.get('M5',0):6.1f} {bm.get('M9',0):6.2f} "
                      f"{bm.get('M10',0):6.2f} {bm.get('worst_bad_streak',0):6d} "
                      f"{bm.get('M6',0):6.2f}")

    # Print per-archetype M3 for key configs
    print("\n" + "=" * 80)
    print("PER-ARCHETYPE M3 (GF+PE, Enriched 40%, Committed)")
    print("=" * 80)
    for fm in fitness_models:
        key = ("enriched_40", fm, "committed")
        m = all_results.get(key, {})
        if m:
            print(f"\n{fm}:")
            pa = m.get("per_arch_m3", {})
            for ai in range(8):
                name = ARCHETYPES[ai]["name"]
                val = pa.get(ai, 0)
                print(f"  {name:<25} M3={val:.2f}")

    print("\n\nSimulation complete.")


if __name__ == "__main__":
    run_all()
