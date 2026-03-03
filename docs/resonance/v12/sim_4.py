#!/usr/bin/env python3
"""
V12 Simulation Agent 4: Design 1 Champion (N=4 Isolation Baseline)

Algorithm: Avoidance + Contraction, No Oversampling
- Starting pool: 120 cards (15 per archetype, 8 archetypes)
- 5 AIs, each assigned 1 of 5 archetypes (3 open lanes)
- AI avoidance: gradual ramp from pick 3 (30% at pick 3, 60% at pick 6, 85% at pick 10+)
- Refills: 60/0/0 (3 rounds of 10 picks), 2.0x open-lane bias
- Pack construction: N=4 (uniform random, NO oversampling)
- No floor slot

Purpose: Calibration baseline. Measures the M3 contribution of AI avoidance +
pool contraction WITHOUT any oversampling. Other V12 simulations measure their
oversampling contribution on top of this baseline.
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
PACK_SIZE = 4
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
# 60/0/0: single 60-card refill after R1, nothing after R2 or R3
ROUND_STRUCTURE = [
    (10, 60),   # Round 1: 10 picks, then refill 60 with 2.0x open-lane bias
    (10, 0),    # Round 2: 10 picks, no refill
    (10, 0),    # Round 3: 10 picks, no refill
]

# AI avoidance ramp: pick -> avoidance weight (0.0 = no avoidance, 1.0 = full)
# Gradual ramp: 30% at pick 3, 60% at pick 6, 85% at pick 10+
def get_avoidance_weight(pick_num):
    """Return avoidance weight for a given pick number."""
    if pick_num < 3:
        return 0.0
    elif pick_num < 6:
        # Linear ramp from 0.30 at pick 3 to 0.55 at pick 5
        return 0.30 + (pick_num - 3) * (0.60 - 0.30) / 3
    elif pick_num < 10:
        # Linear ramp from 0.60 at pick 6 to 0.82 at pick 9
        return 0.60 + (pick_num - 6) * (0.85 - 0.60) / 4
    else:
        return 0.85

# AI saturation threshold: stop taking from own lane when fewer than this many cards remain
AI_SATURATION_THRESHOLD = 5
# Fallback: take from adjacent open lane only if it has > this many cards
AI_FALLBACK_THRESHOLD = 8

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
def generate_cards(rng, count_per_archetype, card_id_start=0, arch_filter=None):
    """Generate cards for specified archetypes.

    arch_filter: if provided, dict of archetype_name -> count. Otherwise uses
    count_per_archetype for all archetypes.
    """
    cards = []
    card_id = card_id_start

    if arch_filter:
        arch_counts = arch_filter
    else:
        arch_counts = {a[0]: count_per_archetype for a in ARCHETYPES}

    for arch_name, r1, r2 in ARCHETYPES:
        n = arch_counts.get(arch_name, 0)
        if n == 0:
            continue
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


def generate_pool(rng, card_id_start=0):
    """Generate a 120-card starting pool: 15 per archetype."""
    return generate_cards(rng, CARDS_PER_ARCHETYPE, card_id_start)


def generate_biased_refill(rng, count, open_lanes, card_id_start):
    """Generate refill cards with 2.0x open-lane bias.

    Open lanes (archetypes with no assigned AI) receive 2.0x the per-card
    refill rate. Closed lanes receive the remaining budget proportionally.

    With 3 open lanes and 5 closed lanes:
    - Open lane weight = 2.0, Closed lane weight = 1.0
    - Total weight = 3*2.0 + 5*1.0 = 11.0
    - Open lane share each = 2.0/11.0 = 18.2% -> ~10.9 cards from 60
    - Closed lane share each = 1.0/11.0 = 9.1% -> ~5.5 cards from 60
    """
    n_open = len(open_lanes)
    n_closed = NUM_ARCHETYPES - n_open
    total_weight = n_open * 2.0 + n_closed * 1.0

    arch_counts = {}
    total_assigned = 0
    for arch_name in ARCHETYPE_NAMES:
        if arch_name in open_lanes:
            w = 2.0 / total_weight
        else:
            w = 1.0 / total_weight
        n = int(round(count * w))
        arch_counts[arch_name] = n
        total_assigned += n

    # Adjust rounding errors
    diff = count - total_assigned
    if diff != 0:
        # Distribute difference to open lanes first
        targets = list(open_lanes) if diff > 0 else ARCHETYPE_NAMES[:]
        rng.shuffle(targets)
        for i in range(abs(diff)):
            arch_counts[targets[i % len(targets)]] += (1 if diff > 0 else -1)

    return generate_cards(rng, 0, card_id_start, arch_filter=arch_counts)


# ============================================================
# AI Drafter with Avoidance
# ============================================================
class AIDrafter:
    def __init__(self, archetype_name, all_archetypes, open_lanes):
        self.archetype = archetype_name
        self.cards_drafted = []
        self.inferred_player_archetype = None
        self.open_lanes = open_lanes

        # Depletion tracking: per-archetype expected vs observed depletion
        self.depletion_history = {a: [] for a in all_archetypes}
        self.prev_pool_counts = None
        self.inference_confidence = 0.0

    def update_pool_observation(self, pool_dict, pick_num):
        """Observe pool state and update inference about player's archetype."""
        current_counts = Counter()
        for c in pool_dict.values():
            current_counts[c.archetype] += 1

        if self.prev_pool_counts is not None:
            # Calculate depletion per archetype since last observation
            for arch in ARCHETYPE_NAMES:
                prev = self.prev_pool_counts.get(arch, 0)
                curr = current_counts.get(arch, 0)
                depletion = prev - curr
                self.depletion_history[arch].append(depletion)

            # Infer player archetype from depletion patterns
            # Use sliding window of last 3 pick cycles
            window = 3
            if pick_num >= 3:
                arch_scores = {}
                for arch in ARCHETYPE_NAMES:
                    if arch == self.archetype:
                        continue  # Skip own archetype
                    recent = self.depletion_history[arch][-window:]
                    if recent:
                        avg_depletion = sum(recent) / len(recent)
                        # Expected depletion: ~1 card per cycle if 1 drafter
                        # is focused on this archetype. Higher = more drafters.
                        # Look for archetypes depleting faster than expected
                        # for an open lane (only 1 drafter should be taking them)
                        arch_scores[arch] = avg_depletion

                # The player's archetype should deplete at ~1 card per cycle
                # (only the player takes it). Archetypes with depletion >> 1
                # have multiple drafters. Archetypes with depletion ~1 are
                # likely the player's lane.
                if arch_scores:
                    # For open lanes, look for consistent ~1.0 depletion
                    # (player is the sole drafter). Higher depletion = AI lane.
                    open_lane_candidates = []
                    for arch in self.open_lanes:
                        if arch in arch_scores:
                            # Consistent moderate depletion suggests a single drafter
                            open_lane_candidates.append(
                                (arch, arch_scores[arch]))

                    if open_lane_candidates:
                        # The open lane with the highest depletion is most likely
                        # the player's archetype (player actively taking from it)
                        open_lane_candidates.sort(key=lambda x: x[1], reverse=True)
                        best = open_lane_candidates[0]
                        if best[1] > 0.3:  # Some evidence of depletion
                            self.inferred_player_archetype = best[0]
                            self.inference_confidence = min(best[1] / 1.5, 1.0)

        self.prev_pool_counts = dict(current_counts)

    def pick_card(self, pool_dict, pick_num, rng):
        """Pick one card from pool with avoidance logic."""
        if not pool_dict:
            return None

        available = list(pool_dict.values())
        avoidance = get_avoidance_weight(pick_num)

        # Count cards in own lane
        own_lane_count = sum(1 for c in available if c.archetype == self.archetype)

        # Check saturation
        if own_lane_count <= AI_SATURATION_THRESHOLD:
            # Saturated: try adjacent open lane, else take highest power
            # Find adjacent archetypes
            arch_idx = ARCHETYPE_NAMES.index(self.archetype)
            adjacent = [
                ARCHETYPE_NAMES[(arch_idx - 1) % NUM_ARCHETYPES],
                ARCHETYPE_NAMES[(arch_idx + 1) % NUM_ARCHETYPES],
            ]
            for adj in adjacent:
                if adj in self.open_lanes:
                    adj_cards = [c for c in available if c.archetype == adj]
                    if len(adj_cards) > AI_FALLBACK_THRESHOLD:
                        # Take from adjacent open lane (but not player's inferred arch)
                        if adj != self.inferred_player_archetype or avoidance < 0.5:
                            best = max(adj_cards, key=lambda c: c.fitness.get(adj, 0))
                            self.cards_drafted.append(best)
                            return best.id

            # No good adjacent option: take highest power from any non-player arch
            safe_cards = available
            if self.inferred_player_archetype and avoidance > 0.3:
                safe_cards = [c for c in available
                              if c.archetype != self.inferred_player_archetype]
            if not safe_cards:
                safe_cards = available

            best = max(safe_cards, key=lambda c: c.power)
            self.cards_drafted.append(best)
            return best.id

        # Normal picking: score by fitness for own archetype
        # Apply avoidance penalty if card is for player's inferred archetype
        def score_card(c):
            base_score = c.fitness.get(self.archetype, 0)
            # Penalty if this card is from the player's inferred archetype
            if (self.inferred_player_archetype and
                c.archetype == self.inferred_player_archetype):
                base_score *= (1.0 - avoidance)
            return base_score

        best = max(available, key=score_card)
        self.cards_drafted.append(best)
        return best.id


# ============================================================
# Pack Construction (N=4 Uniform Random)
# ============================================================
def build_pack(pool_list, rng, pack_size=PACK_SIZE):
    """Draw pack_size cards uniformly at random from pool. No oversampling."""
    if len(pool_list) <= pack_size:
        return pool_list[:]
    return rng.sample(pool_list, pack_size)


# ============================================================
# Player Strategies
# ============================================================
def committed_pick(pack, drafted, committed_arch, pick_num, rng):
    """Committed player: commits around pick 5-6, picks best S/A for archetype."""
    if committed_arch is None and pick_num >= 5:
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
            return max(sa_cards, key=lambda c: c.fitness.get(committed_arch, 0)), committed_arch
        return max(pack, key=lambda c: c.power), committed_arch
    else:
        best = None
        for c in pack:
            if any(c.is_sa_for(a) for a in ARCHETYPE_NAMES):
                if best is None or c.power > best.power:
                    best = c
        return (best or max(pack, key=lambda c: c.power)), committed_arch


def signal_reader_pick(pack, drafted, committed_arch, pick_num, pool_list, rng):
    """Signal reader: reads pool to find open lane (most remaining cards), commits at pick 5."""
    if committed_arch is None and pick_num >= 5:
        # Count S/A per archetype in pool to find the best open lane
        sa_counts = Counter()
        for c in pool_list:
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
            return max(sa_cards, key=lambda c: c.fitness.get(committed_arch, 0)), committed_arch
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
    """Run a single 30-pick draft with AI avoidance and biased refills."""
    # Generate starting pool
    pool_cards, next_card_id = generate_pool(rng)
    pool_dict = {c.id: c for c in pool_cards}

    # Assign 5 AIs to random archetypes
    ai_archetypes = rng.sample(ARCHETYPE_NAMES, NUM_AIS)
    open_lanes = [a for a in ARCHETYPE_NAMES if a not in ai_archetypes]
    ai_drafters = [AIDrafter(a, ARCHETYPE_NAMES, open_lanes) for a in ai_archetypes]

    player_picks = []
    committed_arch = None
    all_pack_data = []
    trace_data = [] if trace else None
    pool_snapshots = []

    # AI inference tracking
    ai_inference_log = []  # (pick, ai_idx, inferred_arch, actual_player_arch, correct)
    ai_avoidance_start_pick = {}  # ai_idx -> first pick where avoidance > 0.3

    # Pool contraction trajectory
    contraction_trajectory = []

    global_pick = 0

    for round_idx, (picks_in_round, refill_count) in enumerate(ROUND_STRUCTURE):
        round_num = round_idx + 1

        # Record pool composition at round start
        arch_counts = Counter()
        sa_counts_pool = Counter()
        for c in pool_dict.values():
            arch_counts[c.archetype] += 1
            if committed_arch and c.is_sa_for(committed_arch):
                sa_counts_pool["player_sa"] += 1
        pool_snapshots.append({
            "round": round_num,
            "pool_size": len(pool_dict),
            "arch_counts": dict(arch_counts),
            "event": "round_start",
        })

        for pick_in_round in range(picks_in_round):
            global_pick += 1
            pool_before = len(pool_dict)

            # Step 0: AIs observe pool state (before any picks this cycle)
            for ai_idx, ai in enumerate(ai_drafters):
                ai.update_pool_observation(pool_dict, global_pick)

            # Step 1: Each AI picks one card
            ai_removed = 0
            ai_order = list(range(len(ai_drafters)))
            rng.shuffle(ai_order)
            for ai_idx in ai_order:
                ai = ai_drafters[ai_idx]
                picked_id = ai.pick_card(pool_dict, global_pick, rng)
                if picked_id is not None and picked_id in pool_dict:
                    del pool_dict[picked_id]
                    ai_removed += 1

            # Track AI inference accuracy
            if committed_arch:
                for ai_idx, ai in enumerate(ai_drafters):
                    correct = (ai.inferred_player_archetype == committed_arch)
                    ai_inference_log.append({
                        "pick": global_pick,
                        "ai_idx": ai_idx,
                        "inferred": ai.inferred_player_archetype,
                        "actual": committed_arch,
                        "correct": correct,
                        "confidence": ai.inference_confidence,
                    })
                    # Track first pick where avoidance is active
                    avoidance_w = get_avoidance_weight(global_pick)
                    if (ai_idx not in ai_avoidance_start_pick and
                        avoidance_w > 0.3 and ai.inferred_player_archetype):
                        ai_avoidance_start_pick[ai_idx] = global_pick

            # Step 2: Build pack for player (N=4 uniform random)
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

            # Count S/A in pack for committed archetype
            sa_count = 0
            if committed_arch:
                sa_count = sum(1 for c in pack if c.is_sa_for(committed_arch))

            # Pool contraction trajectory
            arch_in_pool = Counter()
            sa_in_pool = 0
            player_arch_in_pool = 0
            for c in pool_dict.values():
                arch_in_pool[c.archetype] += 1
                if committed_arch:
                    if c.is_sa_for(committed_arch):
                        sa_in_pool += 1
                    if c.archetype == committed_arch:
                        player_arch_in_pool += 1

            contraction_trajectory.append({
                "pick": global_pick,
                "pool_size": len(pool_dict),
                "player_arch_count": player_arch_in_pool if committed_arch else 0,
                "player_arch_density": (player_arch_in_pool / max(len(pool_dict), 1))
                    if committed_arch else 0,
                "sa_in_pool": sa_in_pool if committed_arch else 0,
                "sa_density": (sa_in_pool / max(len(pool_dict), 1))
                    if committed_arch else 0,
            })

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
                trace_data.append({
                    "pick": global_pick,
                    "round": round_num,
                    "pool_size": len(pool_dict),
                    "ai_removed": ai_removed,
                    "pack_sa": sa_count,
                    "player_sa_in_pool": sa_in_pool,
                    "player_arch_in_pool": player_arch_in_pool,
                    "committed": committed_arch,
                    "chosen_arch": chosen.archetype,
                    "chosen_sa": chosen.is_sa_for(committed_arch) if committed_arch else None,
                    "chosen_power": chosen.power,
                    "arch_in_pool": dict(arch_in_pool),
                    "avoidance_weight": get_avoidance_weight(global_pick),
                    "ai_inferences": [(ai.inferred_player_archetype, ai.inference_confidence)
                                      for ai in ai_drafters],
                })

        # Refill between rounds
        if refill_count > 0:
            new_cards, next_card_id = generate_biased_refill(
                rng, refill_count, open_lanes, next_card_id)
            for c in new_cards:
                pool_dict[c.id] = c

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
        "contraction_trajectory": contraction_trajectory,
        "ai_inference_log": ai_inference_log,
        "ai_avoidance_start_pick": ai_avoidance_start_pick,
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

    # M5: Convergence pick
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

    # M11': Picks 20+, S/A for committed archetype per pack
    if committed_arch:
        m11_vals = []
        for d in all_pack_data[19:]:
            m11_vals.append(sum(1 for c in d["pack"]
                                if c.is_sa_for(committed_arch)))
        result["m11"] = sum(m11_vals) / max(len(m11_vals), 1)
    else:
        result["m11"] = 0

    # M13: AI avoidance detection pick (when AI behavior detectably changes)
    if ai_avoidance_start_pick:
        result["m13"] = min(ai_avoidance_start_pick.values())
    else:
        result["m13"] = NUM_PICKS + 1

    # M14: Player archetype visibility pick (when AI correctly infers)
    first_correct = NUM_PICKS + 1
    if committed_arch and ai_inference_log:
        for entry in ai_inference_log:
            if entry["correct"] and entry["confidence"] > 0.3:
                first_correct = min(first_correct, entry["pick"])
                break
    result["m14"] = first_correct

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
    for key in ["m1", "m2", "m3", "m4", "m5", "m6", "m9", "m10", "m11",
                "m13", "m14"]:
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

    # Pool contraction trajectory (averaged across drafts)
    contraction_by_pick = defaultdict(lambda: {
        "pool_sizes": [], "arch_counts": [], "arch_densities": [],
        "sa_counts": [], "sa_densities": [],
    })
    for r in results:
        for entry in r["contraction_trajectory"]:
            pick = entry["pick"]
            contraction_by_pick[pick]["pool_sizes"].append(entry["pool_size"])
            contraction_by_pick[pick]["arch_counts"].append(entry["player_arch_count"])
            contraction_by_pick[pick]["arch_densities"].append(entry["player_arch_density"])
            contraction_by_pick[pick]["sa_counts"].append(entry["sa_in_pool"])
            contraction_by_pick[pick]["sa_densities"].append(entry["sa_density"])

    agg_contraction = {}
    for pick in sorted(contraction_by_pick.keys()):
        data = contraction_by_pick[pick]
        agg_contraction[pick] = {
            "pool_size": sum(data["pool_sizes"]) / len(data["pool_sizes"]),
            "arch_count": sum(data["arch_counts"]) / len(data["arch_counts"]),
            "arch_density": sum(data["arch_densities"]) / len(data["arch_densities"]),
            "sa_count": sum(data["sa_counts"]) / len(data["sa_counts"]),
            "sa_density": sum(data["sa_densities"]) / len(data["sa_densities"]),
        }
    agg["contraction_trajectory"] = agg_contraction

    # AI inference accuracy over time
    inference_by_pick = defaultdict(lambda: {"correct": 0, "total": 0})
    for r in results:
        for entry in r["ai_inference_log"]:
            pick = entry["pick"]
            inference_by_pick[pick]["total"] += 1
            if entry["correct"]:
                inference_by_pick[pick]["correct"] += 1
    agg["ai_inference_accuracy"] = {
        pick: data["correct"] / max(data["total"], 1)
        for pick, data in inference_by_pick.items()
    }

    # Round compositions
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

    # Track how often committed arch is in open lane
    open_lane_rate = 0
    n_committed = 0
    for r in results:
        if r["committed_arch"]:
            n_committed += 1
            if r["committed_arch"] in r["open_lanes"]:
                open_lane_rate += 1
    agg["open_lane_commit_rate"] = open_lane_rate / max(n_committed, 1)

    agg["traces"] = traces
    agg["n_drafts"] = n_drafts
    return agg


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
        ("M12", "see below",     ">= 0.3",   False),
        ("M13", agg["m13_mean"],  "6-10",     6 <= agg["m13_mean"] <= 10),
        ("M14", agg["m14_mean"],  "4-7",      4 <= agg["m14_mean"] <= 7),
    ]

    n_pass = 0
    lines.append(f"  {'Metric':<7} {'Value':>10} {'Target':<12} Status")
    lines.append(f"  {'-'*7} {'-'*10} {'-'*12} {'-'*6}")
    for name, val, target, passed in metrics:
        if name == "M12":
            continue  # computed separately
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

    lines.append(f"\n  Total: {n_pass}/{len(metrics)-1} metrics passed (M12 computed separately)")
    return "\n".join(lines)


def format_per_arch_m3(agg):
    lines = []
    lines.append(f"\n  Per-Archetype M3 (picks 6+, S/A for committed arch):")
    lines.append(f"  {'Archetype':<16} {'M3':>6} {'Freq':>6}")
    lines.append(f"  {'-'*16} {'-'*6} {'-'*6}")
    for arch in ARCHETYPE_NAMES:
        m3 = agg["per_arch_m3"].get(arch, 0)
        freq = agg["m8_freq"].get(arch, 0)
        lines.append(f"  {arch:<16} {m3:6.2f} {freq:5.1%}")
    return "\n".join(lines)


def format_contraction_trajectory(agg):
    lines = []
    lines.append(f"\n  Pool Contraction Trajectory (committed player, averaged):")
    lines.append(f"  {'Pick':>4} {'Pool':>5} {'Arch#':>6} {'Arch%':>6} {'S/A':>5} {'S/A%':>6}")
    lines.append(f"  {'-'*4} {'-'*5} {'-'*6} {'-'*6} {'-'*5} {'-'*6}")
    trajectory = agg.get("contraction_trajectory", {})
    for pick in sorted(trajectory.keys()):
        t = trajectory[pick]
        lines.append(
            f"  {pick:4d} {t['pool_size']:5.0f} {t['arch_count']:6.1f} "
            f"{t['arch_density']:5.1%} {t['sa_count']:5.1f} {t['sa_density']:5.1%}"
        )
    return "\n".join(lines)


def format_ai_inference(agg):
    lines = []
    lines.append(f"\n  AI Inference Accuracy Over Time:")
    lines.append(f"  {'Pick':>4} {'Accuracy':>8}")
    lines.append(f"  {'-'*4} {'-'*8}")
    accuracy = agg.get("ai_inference_accuracy", {})
    for pick in sorted(accuracy.keys()):
        lines.append(f"  {pick:4d} {accuracy[pick]:7.1%}")
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

        avoidance_str = f"avoid={t['avoidance_weight']:.0%}"

        # Show AI inference summary
        ai_inf = t.get("ai_inferences", [])
        n_correct = sum(1 for inf, conf in ai_inf
                       if inf == t["committed"] and conf > 0.3) if t["committed"] else 0

        lines.append(
            f"  P{t['pick']:02d} pool={t['pool_size']:3d} "
            f"AI={t['ai_removed']} "
            f"packSA={t['pack_sa']} "
            f"poolSA={t['player_sa_in_pool']:2d} "
            f"archPool={t['player_arch_in_pool']:2d} "
            f"{avoidance_str} "
            f"aiCorr={n_correct}/{len(ai_inf)} "
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

    # Pool composition at key points
    for snap in r["pool_snapshots"]:
        if snap["event"] in ("round_start", "post_refill"):
            arch_str = ", ".join(
                f"{a[:4]}:{snap['arch_counts'].get(a, 0)}"
                for a in ARCHETYPE_NAMES)
            lines.append(f"  Pool R{snap['round']} {snap['event']}: "
                         f"total={snap['pool_size']} [{arch_str}]")

    return "\n".join(lines)


def format_round_compositions(agg):
    lines = []
    lines.append(f"\n  Round-by-Round Pool Composition:")
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


# ============================================================
# Main
# ============================================================
def main():
    print("=" * 70)
    print("V12 SIM-4: Design 1 Champion (N=4 Isolation Baseline)")
    print("3 rounds (10/10/10), refills 60/0/0 with 2.0x open-lane bias")
    print("AI avoidance: gradual ramp from pick 3 (30%->60%->85%)")
    print("Pack construction: N=4 uniform random (NO oversampling)")
    print(f"{NUM_DRAFTS} drafts x 30 picks x 3 strategies, 5 AIs, 3 open lanes")
    print("=" * 70)

    strategies = ["committed", "signal", "power"]
    all_results = {}

    for strat in strategies:
        print(f"\nRunning {strat} strategy ({NUM_DRAFTS} drafts)...")
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

    # Open lane commit rate
    print(f"\n  Open Lane Commit Rate:")
    for strat in strategies:
        rate = all_results[strat].get("open_lane_commit_rate", 0)
        print(f"    {strat}: {rate:.1%}")

    # Pool contraction trajectory (committed strategy)
    print(format_contraction_trajectory(all_results["committed"]))

    # AI inference accuracy
    print(format_ai_inference(all_results["committed"]))

    # Round compositions
    print(format_round_compositions(all_results["committed"]))

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

    # Comparison to V9 and V11
    cm = all_results["committed"]
    print(f"\n{'='*70}")
    print("  COMPARISON TO V9 AND V11 SIM-4")
    print(f"{'='*70}")
    print(f"  {'Metric':<7} {'V9 HybB':>10} {'V11 SIM4':>10} {'V12 D1':>10} {'v V9':>10} {'v V11':>10}")
    comparisons = [
        ("M3",  2.70, 0.83, cm["m3_mean"]),
        ("M5",  9.6,  None, cm["m5_mean"]),
        ("M6",  0.86, None, cm["m6_mean"]),
        ("M10", 3.8,  None, cm["m10_mean"]),
        ("M11'",3.25, None, cm["m11_mean"]),
    ]
    for name, v9, v11, sim in comparisons:
        v11_str = f"{v11:.2f}" if v11 is not None else "N/A"
        delta_v9 = sim - v9
        delta_v11 = (sim - v11) if v11 is not None else float('nan')
        delta_v11_str = f"{delta_v11:+.2f}" if v11 is not None else "N/A"
        print(f"  {name:<7} {v9:>10.2f} {v11_str:>10} {sim:>10.2f} {delta_v9:>+10.2f} {delta_v11_str:>10}")

    # Final self-assessment
    print(f"\n{'='*70}")
    print("  SELF-ASSESSMENT (Calibration Baseline)")
    print(f"{'='*70}")
    m3_val = cm["m3_mean"]
    m11_val = cm["m11_mean"]
    m10_val = cm["m10_mean"]
    m13_val = cm["m13_mean"]
    m14_val = cm["m14_mean"]

    print(f"  M3 (committed):  {m3_val:.3f}")
    print(f"  M11' (picks 20+): {m11_val:.3f}")
    print(f"  M10 (bad streaks): {m10_val:.2f}")
    print(f"  M12 (signal advantage): {m12:.3f}")
    print(f"  M13 (avoidance onset): {m13_val:.1f}")
    print(f"  M14 (inference pick): {m14_val:.1f}")

    print(f"\n  CALIBRATION BASELINE M3 = {m3_val:.3f}")
    print(f"  This is the M3 achieved by AI avoidance + pool contraction")
    print(f"  WITHOUT any oversampling (N=4 uniform random packs).")
    print(f"  Other V12 simulations should measure their oversampling")
    print(f"  contribution as delta above this baseline:")
    print(f"    N=8  needs to add {max(0, 2.0 - m3_val):.3f} to reach M3=2.0")
    print(f"    N=12 needs to add {max(0, 2.0 - m3_val):.3f} to reach M3=2.0")

    if m3_val >= 1.0:
        print(f"\n  VERDICT: Avoidance + contraction alone reaches M3 >= 1.0.")
        print(f"  Modest oversampling (N=8-12) should be sufficient to reach M3=2.0.")
    elif m3_val >= 0.5:
        print(f"\n  VERDICT: Avoidance + contraction produces modest concentration.")
        print(f"  N=12 oversampling is likely required; N=8 may be insufficient.")
    else:
        print(f"\n  VERDICT: Avoidance + contraction alone is weak (M3 < 0.5).")
        print(f"  Heavy oversampling or alternative mechanisms are needed.")

    print("\nDone.")


if __name__ == "__main__":
    main()
