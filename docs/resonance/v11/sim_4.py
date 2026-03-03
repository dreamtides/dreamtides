#!/usr/bin/env python3
"""
Simulation Agent 4: SIM-4 -- Graduated 4-Round Decline, No Bias

Algorithm specification (from critic_review.md, modified Design 3):
- 4 rounds (8/8/7/7 picks = 30 total), 120-card starting pool
- Declining balanced refills: 48, 36, 21, 0 cards after each round
- No refill bias -- all refills are balanced (equal per archetype)
- Tests whether declining volume alone can build adequate concentration

Pool: 120 cards, 8 archetypes x 15 cards each
5 AI drafters (Level 0), each assigned one unique archetype (3 open lanes)
AI saturation thresholds decline: R1-R2: 9 cards, R3: 8 cards, R4: 7 cards
After saturation, pick highest-power generic

Card model:
- Each card has archetype, fitness scores per archetype, power, visible symbols, tier
- Graduated Realistic sibling A-tier rates
- Visible symbols: ~11% generic, ~79% single, ~10% dual
"""

import random
import math
from dataclasses import dataclass, field
from collections import defaultdict, Counter
from typing import Optional

# ============================================================
# Constants
# ============================================================
NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 5
NUM_ARCHETYPES = 8
CARDS_PER_ARCHETYPE = 15
STARTING_POOL = NUM_ARCHETYPES * CARDS_PER_ARCHETYPE  # 120
NUM_AIS = 5

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
ARCH_BY_NAME = {a[0]: a for a in ARCHETYPES}
RESONANCES = ["Zephyr", "Ember", "Stone", "Tide"]

# Round structure: (picks_in_round, refill_after_round)
ROUND_STRUCTURE = [
    (8, 48),   # Round 1: 8 picks, then refill 48 balanced
    (8, 36),   # Round 2: 8 picks, then refill 36 balanced
    (7, 21),   # Round 3: 7 picks, then refill 21 balanced
    (7, 0),    # Round 4: 7 picks, no refill
]

# AI saturation thresholds per round
SATURATION_THRESHOLDS = {1: 9, 2: 9, 3: 8, 4: 7}

# Graduated Realistic sibling A-tier rates
SIBLING_PAIRS = {
    frozenset({"Warriors", "Sacrifice"}): 0.50,
    frozenset({"Self-Discard", "Self-Mill"}): 0.40,
    frozenset({"Blink", "Storm"}): 0.30,
    frozenset({"Flash", "Ramp"}): 0.25,
    # Cross-resonance siblings (share secondary)
    frozenset({"Flash", "Blink"}): 0.30,
    frozenset({"Storm", "Self-Discard"}): 0.40,
    frozenset({"Self-Mill", "Sacrifice"}): 0.50,
    frozenset({"Warriors", "Ramp"}): 0.25,
}


def get_sibling(arch_name):
    """Return co-primary sibling archetype name."""
    r1 = ARCH_BY_NAME[arch_name][1]
    for a in ARCHETYPES:
        if a[0] != arch_name and a[1] == r1:
            return a[0]
    return None


def get_sibling_rate(arch1, arch2):
    """Return the A-tier rate for a sibling pair."""
    key = frozenset({arch1, arch2})
    return SIBLING_PAIRS.get(key, 0.0)


# ============================================================
# Card Model
# ============================================================
@dataclass
class SimCard:
    id: int
    archetype: str
    visible_symbols: list
    power: float
    fitness: dict          # archetype -> float (0.0-1.0)
    tier: dict             # archetype -> 'S'|'A'|'C'|'F'
    is_generic: bool = False

    def is_sa_for(self, arch):
        return self.tier.get(arch, 'F') in ('S', 'A')


# ============================================================
# Pool Generation
# ============================================================
def generate_pool(rng, card_id_start=0):
    """Generate a 120-card pool: 15 per archetype, no generics in starting pool.

    Visible symbols: ~11% generic (0 symbols), ~79% single, ~10% dual.
    For a 120-card pool: ~13 generic, ~95 single, ~12 dual.
    However, since we have 8 archetypes x 15 cards, we distribute:
    - Per archetype: ~1-2 dual-symbol cards, rest single-symbol
    - Generic cards: we embed ~13 cards with 0 symbols spread across archetypes
    """
    cards = []
    card_id = card_id_start

    # Target: ~13 generic-symbol, ~12 dual-symbol, ~95 single-symbol out of 120
    # Distribute dual: 1-2 per archetype (total ~12)
    # Distribute generic-symbol: ~1-2 per archetype (total ~13)
    generic_per_arch = [2, 2, 2, 1, 1, 2, 1, 2]  # = 13
    dual_per_arch =    [2, 1, 2, 1, 2, 1, 2, 1]  # = 12

    for arch_idx, (arch_name, r1, r2) in enumerate(ARCHETYPES):
        sibling = get_sibling(arch_name)
        n_generic = generic_per_arch[arch_idx]
        n_dual = dual_per_arch[arch_idx]
        n_single = CARDS_PER_ARCHETYPE - n_generic - n_dual

        for card_type_idx in range(CARDS_PER_ARCHETYPE):
            if card_type_idx < n_dual:
                vis = [r1, r2]
            elif card_type_idx < n_dual + n_single:
                vis = [r1]
            else:
                vis = []  # generic-symbol card (still belongs to archetype)

            power = rng.uniform(1.0, 10.0)

            # Fitness scores
            fitness = {}
            tier = {}
            for other_name in ARCHETYPE_NAMES:
                if other_name == arch_name:
                    fitness[other_name] = rng.uniform(0.7, 1.0)
                    tier[other_name] = rng.choice(['S', 'A'])
                elif other_name == sibling:
                    rate = get_sibling_rate(arch_name, other_name)
                    fitness[other_name] = rng.uniform(0.3, 0.6)
                    tier[other_name] = rng.choice(['S', 'A']) if rng.random() < rate else rng.choice(['C', 'F'])
                else:
                    # Check cross-resonance affinity
                    cross_rate = get_sibling_rate(arch_name, other_name)
                    if cross_rate > 0:
                        fitness[other_name] = rng.uniform(0.15, 0.35)
                        tier[other_name] = rng.choice(['S', 'A']) if rng.random() < cross_rate else rng.choice(['C', 'F'])
                    else:
                        fitness[other_name] = rng.uniform(0.0, 0.15)
                        tier[other_name] = rng.choice(['C', 'F'])

            cards.append(SimCard(
                id=card_id,
                archetype=arch_name,
                visible_symbols=vis,
                power=power,
                fitness=fitness,
                tier=tier,
                is_generic=(len(vis) == 0),
            ))
            card_id += 1

    return cards, card_id


def generate_refill(rng, count, card_id_start):
    """Generate balanced refill cards: equal distribution across 8 archetypes.

    count / 8 cards per archetype. Remainder distributed randomly.
    Same symbol distribution ratios as starting pool.
    """
    per_arch = count // NUM_ARCHETYPES
    remainder = count % NUM_ARCHETYPES
    # Distribute remainder randomly
    extra = [0] * NUM_ARCHETYPES
    indices = list(range(NUM_ARCHETYPES))
    rng.shuffle(indices)
    for i in range(remainder):
        extra[indices[i]] = 1

    cards = []
    card_id = card_id_start

    for arch_idx, (arch_name, r1, r2) in enumerate(ARCHETYPES):
        n = per_arch + extra[arch_idx]
        sibling = get_sibling(arch_name)

        for i in range(n):
            # Symbol distribution: ~10% dual, ~11% generic, ~79% single
            roll = rng.random()
            if roll < 0.10:
                vis = [r1, r2]
            elif roll < 0.21:
                vis = []
            else:
                vis = [r1]

            power = rng.uniform(1.0, 10.0)

            fitness = {}
            tier = {}
            for other_name in ARCHETYPE_NAMES:
                if other_name == arch_name:
                    fitness[other_name] = rng.uniform(0.7, 1.0)
                    tier[other_name] = rng.choice(['S', 'A'])
                elif other_name == sibling:
                    rate = get_sibling_rate(arch_name, other_name)
                    fitness[other_name] = rng.uniform(0.3, 0.6)
                    tier[other_name] = rng.choice(['S', 'A']) if rng.random() < rate else rng.choice(['C', 'F'])
                else:
                    cross_rate = get_sibling_rate(arch_name, other_name)
                    if cross_rate > 0:
                        fitness[other_name] = rng.uniform(0.15, 0.35)
                        tier[other_name] = rng.choice(['S', 'A']) if rng.random() < cross_rate else rng.choice(['C', 'F'])
                    else:
                        fitness[other_name] = rng.uniform(0.0, 0.15)
                        tier[other_name] = rng.choice(['C', 'F'])

            cards.append(SimCard(
                id=card_id,
                archetype=arch_name,
                visible_symbols=vis,
                power=power,
                fitness=fitness,
                tier=tier,
                is_generic=(len(vis) == 0),
            ))
            card_id += 1

    return cards, card_id


# ============================================================
# AI Drafter
# ============================================================
class AIDrafter:
    def __init__(self, archetype_name):
        self.archetype = archetype_name
        self.arch_cards_taken = 0

    def pick_card(self, pool_dict, round_num):
        """Pick one card from pool. Returns card id or None."""
        if not pool_dict:
            return None

        threshold = SATURATION_THRESHOLDS.get(round_num, 9)
        available = list(pool_dict.values())

        if self.arch_cards_taken < threshold:
            # Pick best fitness for assigned archetype
            arch_cards = [c for c in available
                          if c.fitness.get(self.archetype, 0) > 0.2]
            if arch_cards:
                best = max(arch_cards,
                           key=lambda c: c.fitness.get(self.archetype, 0))
                self.arch_cards_taken += 1
                return best.id

        # Saturated or no good archetype cards: pick highest power generic
        best = max(available, key=lambda c: c.power)
        return best.id


# ============================================================
# Pack Construction
# ============================================================
def build_pack(pool_list, rng, pack_size=PACK_SIZE):
    """Draw pack_size cards from pool weighted by archetype pool presence.

    Weighting: each card's draw probability is proportional to
    1 / (count of its archetype in pool). This makes underrepresented
    archetypes more likely to appear in packs, reflecting natural sampling.
    Actually, the spec says "weighted by archetype pool presence" which means
    more common archetypes are more likely. We use simple random sampling
    (uniform) as the baseline -- pool composition naturally determines what
    appears.
    """
    if len(pool_list) <= pack_size:
        return pool_list[:]

    # Weighted sampling: probability proportional to archetype's share of pool
    # This naturally means archetypes with more cards in pool appear more often
    return rng.sample(pool_list, pack_size)


# ============================================================
# Player Strategies
# ============================================================
def committed_pick(pack, drafted, committed_arch, pick_num, rng):
    """Committed player: commits around pick 5, picks best S/A for archetype."""
    if committed_arch is None and pick_num >= 5:
        # Commit to archetype with most S/A cards drafted
        sa_counts = Counter()
        for c in drafted:
            for a in ARCHETYPE_NAMES:
                if c.is_sa_for(a):
                    sa_counts[a] += 1
        if sa_counts:
            committed_arch = sa_counts.most_common(1)[0][0]
        else:
            committed_arch = ARCHETYPE_NAMES[rng.randint(0, 7)]

    if committed_arch:
        sa_cards = [c for c in pack if c.is_sa_for(committed_arch)]
        if sa_cards:
            return max(sa_cards, key=lambda c: c.power), committed_arch
        return max(pack, key=lambda c: c.power), committed_arch
    else:
        # Pre-commit: pick best S/A for any archetype (preferring high power)
        best = None
        for c in pack:
            if any(c.is_sa_for(a) for a in ARCHETYPE_NAMES):
                if best is None or c.power > best.power:
                    best = c
        return (best or max(pack, key=lambda c: c.power)), committed_arch


def signal_reader_pick(pack, drafted, committed_arch, pick_num, pool_list, rng):
    """Signal reader: reads pool to find open lane, commits at pick 5."""
    if committed_arch is None and pick_num >= 5:
        # Count cards per archetype in pool -- commit to most available
        arch_counts = Counter()
        for c in pool_list:
            arch_counts[c.archetype] += 1
        if arch_counts:
            committed_arch = arch_counts.most_common(1)[0][0]
        else:
            committed_arch = ARCHETYPE_NAMES[rng.randint(0, 7)]

    if committed_arch:
        sa_cards = [c for c in pack if c.is_sa_for(committed_arch)]
        if sa_cards:
            return max(sa_cards, key=lambda c: c.power), committed_arch
        return max(pack, key=lambda c: c.power), committed_arch
    else:
        best = None
        for c in pack:
            if any(c.is_sa_for(a) for a in ARCHETYPE_NAMES):
                if best is None or c.power > best.power:
                    best = c
        return (best or max(pack, key=lambda c: c.power)), committed_arch


def power_chaser_pick(pack, drafted, committed_arch, pick_num, rng):
    """Always pick highest power card."""
    return max(pack, key=lambda c: c.power), None


# ============================================================
# Single Draft
# ============================================================
def run_single_draft(rng, strategy, trace=False):
    """Run a single 30-pick multi-round draft."""
    # Generate starting pool
    pool_cards, next_card_id = generate_pool(rng)
    pool_dict = {c.id: c for c in pool_cards}

    # Assign 5 AIs to random archetypes
    ai_archetypes = rng.sample(ARCHETYPE_NAMES, NUM_AIS)
    open_lanes = [a for a in ARCHETYPE_NAMES if a not in ai_archetypes]
    ai_drafters = [AIDrafter(a) for a in ai_archetypes]

    player_picks = []
    committed_arch = None
    all_pack_data = []  # (pack, sa_count_for_committed_arch) per pick
    trace_data = [] if trace else None

    # Pool composition snapshots at round boundaries
    pool_snapshots = []

    global_pick = 0

    for round_idx, (picks_in_round, refill_count) in enumerate(ROUND_STRUCTURE):
        round_num = round_idx + 1

        # Record pool composition at round start
        arch_counts = Counter()
        for c in pool_dict.values():
            arch_counts[c.archetype] += 1
        pool_snapshots.append({
            "round": round_num,
            "pool_size": len(pool_dict),
            "arch_counts": dict(arch_counts),
            "event": "round_start",
        })

        for pick_in_round in range(picks_in_round):
            global_pick += 1
            pool_before = len(pool_dict)

            # Step 1: Each AI picks one card
            ai_removed = 0
            ai_order = list(range(len(ai_drafters)))
            rng.shuffle(ai_order)
            for ai_idx in ai_order:
                ai = ai_drafters[ai_idx]
                picked_id = ai.pick_card(pool_dict, round_num)
                if picked_id is not None and picked_id in pool_dict:
                    del pool_dict[picked_id]
                    ai_removed += 1

            # Step 2: Build pack for player
            pool_list = list(pool_dict.values())
            if len(pool_list) < PACK_SIZE:
                pack = pool_list[:]
            else:
                pack = build_pack(pool_list, rng)

            if not pack:
                break

            # Step 3: Player picks
            if strategy == "committed":
                chosen, committed_arch = committed_pick(
                    pack, player_picks, committed_arch, global_pick, rng)
            elif strategy == "signal":
                chosen, committed_arch = signal_reader_pick(
                    pack, player_picks, committed_arch, global_pick,
                    pool_list, rng)
            elif strategy == "power":
                chosen, committed_arch = power_chaser_pick(
                    pack, player_picks, committed_arch, global_pick, rng)
            else:
                raise ValueError(f"Unknown strategy: {strategy}")

            if chosen.id in pool_dict:
                del pool_dict[chosen.id]
            player_picks.append(chosen)

            # Record pack data
            sa_count = 0
            if committed_arch:
                sa_count = sum(1 for c in pack if c.is_sa_for(committed_arch))
            all_pack_data.append({
                "pick": global_pick,
                "round": round_num,
                "pack": pack,
                "sa_count": sa_count,
                "committed_arch": committed_arch,
                "pool_before": pool_before,
                "pool_after": len(pool_dict),
                "ai_removed": ai_removed,
                "chosen": chosen,
            })

            if trace:
                arch_in_pool = Counter()
                for c in pool_dict.values():
                    arch_in_pool[c.archetype] += 1
                sa_in_pool = Counter()
                for c in pool_dict.values():
                    if committed_arch and c.is_sa_for(committed_arch):
                        sa_in_pool["player_sa"] += 1
                trace_data.append({
                    "pick": global_pick,
                    "round": round_num,
                    "pool_size": len(pool_dict),
                    "ai_removed": ai_removed,
                    "pack_sa": sa_count,
                    "arch_in_pool": dict(arch_in_pool),
                    "player_sa_in_pool": sa_in_pool.get("player_sa", 0),
                    "committed": committed_arch,
                    "chosen_arch": chosen.archetype,
                    "chosen_sa": chosen.is_sa_for(committed_arch) if committed_arch else None,
                    "chosen_power": chosen.power,
                })

        # Refill between rounds
        if refill_count > 0:
            new_cards, next_card_id = generate_refill(
                rng, refill_count, next_card_id)
            for c in new_cards:
                pool_dict[c.id] = c

            # Record post-refill composition
            arch_counts = Counter()
            for c in pool_dict.values():
                arch_counts[c.archetype] += 1
            pool_snapshots.append({
                "round": round_num,
                "pool_size": len(pool_dict),
                "arch_counts": dict(arch_counts),
                "event": "post_refill",
                "refill_count": refill_count,
            })

    # ============================================================
    # Compute Metrics
    # ============================================================
    result = {
        "committed_arch": committed_arch,
        "ai_archetypes": ai_archetypes,
        "open_lanes": open_lanes,
        "player_picks": player_picks,
        "pool_snapshots": pool_snapshots,
    }

    # M1: Picks 1-5, unique archetypes with S/A per pack
    m1_vals = []
    for d in all_pack_data[:5]:
        archs_with_sa = set()
        for c in d["pack"]:
            for a in ARCHETYPE_NAMES:
                if c.is_sa_for(a):
                    archs_with_sa.add(a)
        m1_vals.append(len(archs_with_sa))
    result["m1"] = sum(m1_vals) / max(len(m1_vals), 1)

    # M2: Picks 1-5, S/A for emerging archetype per pack
    if committed_arch:
        m2_vals = []
        for d in all_pack_data[:5]:
            m2_vals.append(sum(1 for c in d["pack"]
                               if c.is_sa_for(committed_arch)))
        result["m2"] = sum(m2_vals) / max(len(m2_vals), 1)
    else:
        result["m2"] = 0

    # M3: Picks 6+, S/A for committed archetype per pack
    m3_per_pack = []
    if committed_arch:
        for d in all_pack_data[5:]:
            sa = sum(1 for c in d["pack"] if c.is_sa_for(committed_arch))
            m3_per_pack.append(sa)
    result["m3"] = sum(m3_per_pack) / max(len(m3_per_pack), 1) if m3_per_pack else 0
    result["m3_per_pack"] = m3_per_pack

    # M4: Picks 6+, off-archetype C/F per pack
    if committed_arch:
        m4_vals = []
        for d in all_pack_data[5:]:
            m4_vals.append(sum(1 for c in d["pack"]
                               if not c.is_sa_for(committed_arch)))
        result["m4"] = sum(m4_vals) / max(len(m4_vals), 1)
    else:
        result["m4"] = 0

    # M5: Convergence pick (first pick where 3-pick rolling avg S/A >= 2.0)
    if committed_arch:
        sa_seq = []
        for d in all_pack_data:
            sa_seq.append(sum(1 for c in d["pack"]
                              if c.is_sa_for(committed_arch)))
        convergence = None
        for i in range(2, len(sa_seq)):
            if sum(sa_seq[i-2:i+1]) / 3.0 >= 2.0:
                convergence = i + 1
                break
        result["m5"] = convergence if convergence else NUM_PICKS + 1
    else:
        result["m5"] = NUM_PICKS + 1

    # M6: Deck archetype concentration
    if committed_arch:
        sa_deck = sum(1 for c in player_picks if c.is_sa_for(committed_arch))
        result["m6"] = sa_deck / max(len(player_picks), 1)
    else:
        result["m6"] = 0

    # M9: StdDev of S/A per pack, picks 6+
    if m3_per_pack:
        mean = result["m3"]
        var = sum((x - mean) ** 2 for x in m3_per_pack) / len(m3_per_pack)
        result["m9"] = math.sqrt(var)
    else:
        result["m9"] = 0

    # M10: Max consecutive packs below 1.5 S/A, picks 6+
    max_streak = 0
    cur_streak = 0
    all_streaks = []
    for sa in m3_per_pack:
        if sa < 1.5:
            cur_streak += 1
            max_streak = max(max_streak, cur_streak)
        else:
            if cur_streak > 0:
                all_streaks.append(cur_streak)
            cur_streak = 0
    if cur_streak > 0:
        all_streaks.append(cur_streak)
    result["m10"] = max_streak
    result["m10_streaks"] = all_streaks

    # M11': Picks 20+, S/A for committed archetype per pack (target >= 2.5)
    if committed_arch:
        m11_vals = []
        for d in all_pack_data[19:]:  # picks 20+
            m11_vals.append(sum(1 for c in d["pack"]
                                if c.is_sa_for(committed_arch)))
        result["m11"] = sum(m11_vals) / max(len(m11_vals), 1)
    else:
        result["m11"] = 0

    result["deck_card_ids"] = set(c.id for c in player_picks)
    result["all_pack_data"] = all_pack_data

    if trace:
        result["trace"] = trace_data

    return result


# ============================================================
# Aggregate Simulation
# ============================================================
def run_simulation(strategy, n_drafts=NUM_DRAFTS, collect_traces=0):
    """Run n_drafts and aggregate metrics."""
    results = []
    traces = []

    for i in range(n_drafts):
        rng = random.Random(42 + i)
        do_trace = (i < collect_traces)
        r = run_single_draft(rng, strategy, trace=do_trace)
        results.append(r)
        if do_trace:
            traces.append(r)

    agg = {}

    # Simple means
    for key in ["m1", "m2", "m3", "m4", "m5", "m6", "m9", "m10", "m11"]:
        vals = [r[key] for r in results]
        agg[f"{key}_mean"] = sum(vals) / len(vals)

    # Per-archetype M3
    arch_m3 = defaultdict(list)
    for r in results:
        if r["committed_arch"]:
            arch_m3[r["committed_arch"]].append(r["m3"])
    agg["per_arch_m3"] = {a: (sum(v)/len(v) if v else 0)
                          for a, v in arch_m3.items()}

    # M7: Run-to-run card overlap
    arch_runs = defaultdict(list)
    for r in results:
        if r["committed_arch"]:
            arch_runs[r["committed_arch"]].append(r["deck_card_ids"])
    overlaps = []
    for arch, decks in arch_runs.items():
        for i in range(len(decks) - 1):
            union = decks[i] | decks[i+1]
            inter = decks[i] & decks[i+1]
            if union:
                overlaps.append(len(inter) / len(union))
    agg["m7_overlap"] = sum(overlaps) / max(len(overlaps), 1)

    # M8: Archetype frequency
    arch_freq = Counter()
    for r in results:
        if r["committed_arch"]:
            arch_freq[r["committed_arch"]] += 1
    total = sum(arch_freq.values())
    agg["m8_freq"] = {a: arch_freq.get(a, 0) / max(total, 1)
                      for a in ARCHETYPE_NAMES}
    agg["m8_max"] = max(agg["m8_freq"].values()) if agg["m8_freq"] else 0
    agg["m8_min"] = min(agg["m8_freq"].values()) if agg["m8_freq"] else 0

    # M10 streak distribution
    all_streaks = []
    for r in results:
        all_streaks.extend(r["m10_streaks"])
    agg["m10_streak_dist"] = Counter(all_streaks)

    # Pack quality distribution (S/A per pack, picks 6+)
    all_late_sa = []
    for r in results:
        all_late_sa.extend(r["m3_per_pack"])
    all_late_sa.sort()
    n = len(all_late_sa)
    if n > 0:
        agg["pq_p10"] = all_late_sa[int(n * 0.10)]
        agg["pq_p25"] = all_late_sa[int(n * 0.25)]
        agg["pq_p50"] = all_late_sa[int(n * 0.50)]
        agg["pq_p75"] = all_late_sa[int(n * 0.75)]
        agg["pq_p90"] = all_late_sa[min(int(n * 0.90), n - 1)]
    else:
        for p in ["pq_p10", "pq_p25", "pq_p50", "pq_p75", "pq_p90"]:
            agg[p] = 0

    # M12: Signal-reader M3 minus Committed M3 (computed externally)

    # S/A density trajectory: average S/A in pool at each pick
    # (from trace data, computed separately)

    # Pool composition at round boundaries (averaged)
    round_compositions = defaultdict(lambda: defaultdict(list))
    for r in results:
        for snap in r["pool_snapshots"]:
            key = (snap["round"], snap["event"])
            round_compositions[key]["pool_size"].append(snap["pool_size"])
            for arch in ARCHETYPE_NAMES:
                round_compositions[key][arch].append(
                    snap["arch_counts"].get(arch, 0))

    agg_compositions = {}
    for key, data in round_compositions.items():
        agg_comp = {"pool_size": sum(data["pool_size"]) / len(data["pool_size"])}
        for arch in ARCHETYPE_NAMES:
            if data[arch]:
                agg_comp[arch] = sum(data[arch]) / len(data[arch])
            else:
                agg_comp[arch] = 0
        agg_compositions[key] = agg_comp
    agg["round_compositions"] = agg_compositions

    agg["traces"] = traces
    agg["n_drafts"] = n_drafts
    return agg


# ============================================================
# S/A Density Trajectory (from traces)
# ============================================================
def compute_sa_trajectory(traces):
    """Compute average S/A density in pool at each pick from trace data."""
    if not traces:
        return {}

    pick_sa = defaultdict(list)
    pick_pool = defaultdict(list)
    for r in traces:
        if "trace" not in r:
            continue
        for t in r["trace"]:
            pick_sa[t["pick"]].append(t["player_sa_in_pool"])
            pick_pool[t["pick"]].append(t["pool_size"])

    trajectory = {}
    for pick in sorted(pick_sa.keys()):
        avg_sa = sum(pick_sa[pick]) / len(pick_sa[pick])
        avg_pool = sum(pick_pool[pick]) / len(pick_pool[pick])
        density = avg_sa / max(avg_pool, 1)
        trajectory[pick] = {
            "avg_sa": avg_sa,
            "avg_pool": avg_pool,
            "density": density,
        }
    return trajectory


# ============================================================
# Output Formatting
# ============================================================
def format_scorecard(label, agg):
    lines = []
    lines.append(f"\n{'='*70}")
    lines.append(f"  {label}")
    lines.append(f"{'='*70}")

    metrics = [
        ("M1",  agg["m1_mean"],   ">= 3",     agg["m1_mean"] >= 3.0),
        ("M2",  agg["m2_mean"],   "<= 2",     agg["m2_mean"] <= 2.0),
        ("M3",  agg["m3_mean"],   ">= 2.0",   agg["m3_mean"] >= 2.0),
        ("M4",  agg["m4_mean"],   ">= 0.5",   agg["m4_mean"] >= 0.5),
        ("M5",  agg["m5_mean"],   "5-8",      5 <= agg["m5_mean"] <= 8),
        ("M6",  agg["m6_mean"],   "60-90%",   0.60 <= agg["m6_mean"] <= 0.90),
        ("M7",  agg["m7_overlap"],"< 40%",    agg["m7_overlap"] < 0.40),
        ("M8",  f"{agg['m8_max']:.1%}/{agg['m8_min']:.1%}",
         "<=20%/>=5%", agg["m8_max"] <= 0.20 and agg["m8_min"] >= 0.05),
        ("M9",  agg["m9_mean"],   ">= 0.8",   agg["m9_mean"] >= 0.8),
        ("M10", agg["m10_mean"],  "<= 2",     agg["m10_mean"] <= 2.0),
        ("M11'",agg["m11_mean"],  ">= 2.5",   agg["m11_mean"] >= 2.5),
    ]

    n_pass = 0
    lines.append(f"  {'Metric':<7} {'Value':>10} {'Target':<12} Status")
    lines.append(f"  {'-'*7} {'-'*10} {'-'*12} {'-'*6}")
    for name, val, target, passed in metrics:
        status = "PASS" if passed else "FAIL"
        if passed:
            n_pass += 1
        if isinstance(val, float):
            if name == "M6":
                lines.append(f"  {name:<7} {val:>9.1%} {target:<12} {status}")
            else:
                lines.append(f"  {name:<7} {val:>10.2f} {target:<12} {status}")
        else:
            lines.append(f"  {name:<7} {val:>10} {target:<12} {status}")

    lines.append(f"\n  Total: {n_pass}/{len(metrics)} metrics passed")
    return "\n".join(lines)


def format_per_arch_m3(agg):
    lines = []
    lines.append(f"\n  Per-Archetype M3 (picks 6+, S/A for committed arch):")
    lines.append(f"  {'Archetype':<16} {'M3':>6} {'Freq':>6}")
    lines.append(f"  {'-'*16} {'-'*6} {'-'*6}")
    for arch in ARCHETYPE_NAMES:
        m3 = agg["per_arch_m3"].get(arch, 0)
        freq = agg["m8_freq"].get(arch, 0)
        flag = " <-- below 2.0" if m3 < 2.0 else ""
        lines.append(f"  {arch:<16} {m3:6.2f} {freq:5.1%}{flag}")
    return "\n".join(lines)


def format_round_compositions(agg, ai_label=""):
    lines = []
    lines.append(f"\n  Round-by-Round Pool Composition{ai_label}:")
    lines.append(f"  {'Event':<20} {'Pool':>5} " +
                 " ".join(f"{a[:6]:>7}" for a in ARCHETYPE_NAMES))
    lines.append(f"  {'-'*20} {'-'*5} " + " ".join(['-'*7]*8))

    comps = agg["round_compositions"]
    for key in sorted(comps.keys()):
        round_num, event = key
        data = comps[key]
        label = f"R{round_num} {event}"
        pool_sz = data["pool_size"]
        vals = [data.get(a, 0) for a in ARCHETYPE_NAMES]
        lines.append(f"  {label:<20} {pool_sz:5.0f} " +
                     " ".join(f"{v:7.1f}" for v in vals))
    return "\n".join(lines)


def format_pack_quality(agg):
    lines = []
    lines.append(f"\n  Pack Quality Distribution (S/A per pack, picks 6+):")
    lines.append(f"    p10={agg['pq_p10']}  p25={agg['pq_p25']}  "
                 f"p50={agg['pq_p50']}  p75={agg['pq_p75']}  "
                 f"p90={agg['pq_p90']}")
    return "\n".join(lines)


def format_bad_streaks(agg):
    lines = []
    lines.append(f"\n  Consecutive Bad Pack Analysis (< 1.5 S/A, picks 6+):")
    lines.append(f"    Mean max streak: {agg['m10_mean']:.2f}")
    if agg["m10_streak_dist"]:
        lines.append(f"    Streak distribution:")
        for length in sorted(agg["m10_streak_dist"].keys()):
            count = agg["m10_streak_dist"][length]
            lines.append(f"      Length {length}: {count} occurrences")
    return "\n".join(lines)


def format_trace(r, label=""):
    lines = []
    lines.append(f"\n  --- Draft Trace{' (' + label + ')' if label else ''} ---")
    lines.append(f"  AI archetypes: {', '.join(r['ai_archetypes'])}")
    lines.append(f"  Open lanes: {', '.join(r['open_lanes'])}")
    lines.append(f"  Committed to: {r['committed_arch']}")
    in_open = r['committed_arch'] in r['open_lanes'] if r['committed_arch'] else 'N/A'
    lines.append(f"  In open lane: {in_open}")
    lines.append("")

    if "trace" not in r:
        lines.append("  (no trace data)")
        return "\n".join(lines)

    current_round = 0
    for t in r["trace"]:
        if t["round"] != current_round:
            current_round = t["round"]
            lines.append(f"  --- Round {current_round} ---")

        sa_str = ""
        if t["chosen_sa"] is not None:
            sa_str = "S/A" if t["chosen_sa"] else "C/F"
        else:
            sa_str = "?"

        lines.append(
            f"  P{t['pick']:02d} pool={t['pool_size']:3d} "
            f"AI={t['ai_removed']} "
            f"packSA={t['pack_sa']} "
            f"poolSA={t['player_sa_in_pool']:2d} "
            f"-> {t['chosen_arch']}"
            f"({t['chosen_power']:.1f},{sa_str}) "
            f"arch={t['committed'] or 'none'}"
        )

    # Final deck summary
    if r["committed_arch"]:
        sa_deck = sum(1 for c in r["player_picks"]
                      if c.is_sa_for(r["committed_arch"]))
        lines.append(f"\n  Final: {sa_deck}/{len(r['player_picks'])} S/A = "
                     f"{sa_deck/max(1,len(r['player_picks']))*100:.0f}%")

    # Show pool composition at key points
    for snap in r["pool_snapshots"]:
        if snap["event"] in ("round_start", "post_refill"):
            arch_str = ", ".join(
                f"{a[:4]}:{snap['arch_counts'].get(a, 0)}"
                for a in ARCHETYPE_NAMES)
            lines.append(f"  Pool R{snap['round']} {snap['event']}: "
                         f"total={snap['pool_size']} [{arch_str}]")

    return "\n".join(lines)


# ============================================================
# Main
# ============================================================
def main():
    print("=" * 70)
    print("SIM-4: Graduated 4-Round Decline, No Bias")
    print("4 rounds (8/8/7/7), declining balanced refills (48/36/21/0)")
    print(f"1000 drafts x 30 picks x 3 strategies, 5 AIs, 3 open lanes")
    print("=" * 70)

    # Run all three strategies
    strategies = ["committed", "signal", "power"]
    all_results = {}

    for strat in strategies:
        print(f"\nRunning {strat} strategy...")
        agg = run_simulation(strat, n_drafts=NUM_DRAFTS, collect_traces=2)
        all_results[strat] = agg
        print(format_scorecard(f"{strat.capitalize()} Player", agg))
        print(format_per_arch_m3(agg))
        print(format_pack_quality(agg))
        print(format_bad_streaks(agg))

    # M12: Signal-reader M3 minus Committed M3
    m12 = all_results["signal"]["m3_mean"] - all_results["committed"]["m3_mean"]
    print(f"\n{'='*70}")
    print(f"  M12 (Signal M3 - Committed M3): {m12:.3f}  (target >= 0.3)")
    print(f"    Signal M3:    {all_results['signal']['m3_mean']:.3f}")
    print(f"    Committed M3: {all_results['committed']['m3_mean']:.3f}")
    print(f"    Power M3:     {all_results['power']['m3_mean']:.3f}")
    m12_pass = m12 >= 0.3
    print(f"    M12 Status: {'PASS' if m12_pass else 'FAIL'}")

    # Round-by-round pool composition (committed strategy)
    print(format_round_compositions(all_results["committed"]))

    # S/A density trajectory from traces
    print(f"\n  S/A Density Trajectory (from committed traces):")
    if all_results["committed"]["traces"]:
        trajectory = compute_sa_trajectory(all_results["committed"]["traces"])
        if trajectory:
            print(f"  {'Pick':>4} {'Pool':>5} {'SA':>5} {'Density':>8}")
            for pick in sorted(trajectory.keys()):
                t = trajectory[pick]
                print(f"  {pick:4d} {t['avg_pool']:5.0f} "
                      f"{t['avg_sa']:5.1f} {t['density']:8.3f}")

    # Draft traces
    print(f"\n{'='*70}")
    print("  DRAFT TRACES")
    print(f"{'='*70}")
    if all_results["committed"]["traces"]:
        print(format_trace(
            all_results["committed"]["traces"][0], "Committed #1"))
    if len(all_results["committed"]["traces"]) > 1:
        print(format_trace(
            all_results["committed"]["traces"][1], "Committed #2"))
    if all_results["signal"]["traces"]:
        print(format_trace(
            all_results["signal"]["traces"][0], "Signal-reader #1"))

    # Comparison to V9 and V10
    cm = all_results["committed"]
    print(f"\n{'='*70}")
    print("  COMPARISON TO V9 AND V10")
    print(f"{'='*70}")
    print(f"  {'Metric':<7} {'V9 HybB':>10} {'V10 HybX':>10} {'SIM-4':>10} {'Delta/V9':>10}")
    comparisons = [
        ("M3",  2.70, 0.84, cm["m3_mean"]),
        ("M5",  9.6,  None, cm["m5_mean"]),
        ("M6",  0.86, None, cm["m6_mean"]),
        ("M10", 3.8,  None, cm["m10_mean"]),
        ("M11", 3.25, None, cm["m11_mean"]),
    ]
    for name, v9, v10, sim4 in comparisons:
        v10_str = f"{v10:.2f}" if v10 is not None else "N/A"
        delta = sim4 - v9
        print(f"  {name:<7} {v9:>10.2f} {v10_str:>10} {sim4:>10.2f} {delta:>+10.2f}")

    # Final self-assessment
    print(f"\n{'='*70}")
    print("  SELF-ASSESSMENT")
    print(f"{'='*70}")
    m3_pass = cm["m3_mean"] >= 2.0
    m11_pass = cm["m11_mean"] >= 2.5
    m10_pass = cm["m10_mean"] <= 2.0
    print(f"  M3 >= 2.0:  {'YES' if m3_pass else 'NO'} ({cm['m3_mean']:.2f})")
    print(f"  M11'>= 2.5: {'YES' if m11_pass else 'NO'} ({cm['m11_mean']:.2f})")
    print(f"  M10 <= 2.0: {'YES' if m10_pass else 'NO'} ({cm['m10_mean']:.2f})")
    print(f"  M12 >= 0.3: {'YES' if m12_pass else 'NO'} ({m12:.2f})")

    if m3_pass and m11_pass:
        print("\n  VERDICT: Declining volume alone (without bias) reaches M3 >= 2.0")
        print("  and M11' >= 2.5. This validates Design 3's hypothesis that")
        print("  graduated declining refills are sufficient without open-lane bias.")
    elif m3_pass:
        print("\n  VERDICT: Declining volume reaches M3 >= 2.0 but falls short on")
        print("  M11'. Late-draft concentration is insufficient without bias.")
    else:
        print("\n  VERDICT: Declining volume alone does NOT reach M3 >= 2.0.")
        print("  This confirms that balanced refills -- even declining ones --")
        print("  cannot overcome the refill reset problem without bias assistance.")
        print("  Open-lane bias (SIM-2/SIM-3) is likely required.")

    print("\nDone.")


if __name__ == "__main__":
    main()
