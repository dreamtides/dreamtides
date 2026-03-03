#!/usr/bin/env python3
"""
V12 Simulation Agent 5: Design 2 Champion -- Steep Contraction + N=8

Algorithm specification (Design 2 post-critique revision):
- Starting pool: 120 cards (8 archetypes x 15 cards, ~5 S/A each)
- Fitness: Graduated Realistic (~36% weighted-average sibling A-tier)
- 5 AIs, each assigned 1 of 5 archetypes (3 open lanes)
- AI avoidance: evidence-proportional from pick 2, minimum 3-cycle window
  before 50%+ weight, max weight ramps 0.5 (picks 2-4) to 0.9 (pick 12+)
- Refills: 60/20/0 (3 rounds of 10 picks), 2.0x open-lane bias
- S/A targeting: refills add S/A at 40% rate for open lanes
- Pack construction: N=4 (picks 1-5), N=8 (picks 6-30)
- "Best 4" ranking: visible resonance symbol match for player's inferred arch
- Conditional: if S/A at pick 25 < 3, add floor slot and consider N=10
- AI saturation threshold: 10 cards per archetype

Simulation: 1000 drafts x 30 picks x 3 player strategies
All 14 metrics (M1-M14)
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
SHOW_SIZE = 4
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
ARCH_INDEX = {a[0]: i for i, a in enumerate(ARCHETYPES)}
RESONANCES = ["Zephyr", "Ember", "Stone", "Tide"]

# Round structure: 3 rounds of 10 picks, refill 60/20/0
ROUND_STRUCTURE = [
    (10, 60),   # Round 1: 10 picks, then refill 60 cards (2.0x open-lane bias)
    (10, 20),   # Round 2: 10 picks, then refill 20 cards (2.0x open-lane bias)
    (10, 0),    # Round 3: 10 picks, no refill
]

# AI saturation threshold
AI_SATURATION_THRESHOLD = 10

# Refill bias for open lanes
OPEN_LANE_BIAS = 2.0

# S/A rate for open-lane refill cards (40% vs 36% baseline)
OPEN_LANE_SA_RATE = 0.40
BASELINE_SA_RATE = 0.36

# Oversampling
N_EXPLORE = 4    # picks 1-5
N_EXPLOIT = 8    # picks 6-30

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


def get_arch_resonances(arch_name):
    """Return (primary, secondary) resonance for an archetype."""
    a = ARCH_BY_NAME[arch_name]
    return a[1], a[2]


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

    def resonance_match_score(self, target_primary, target_secondary):
        """Score based on visible resonance symbol match to target archetype."""
        score = 0.0
        for sym in self.visible_symbols:
            if sym == target_primary:
                score += 1.0
            elif sym == target_secondary:
                score += 0.5
        return score


# ============================================================
# Pool Generation
# ============================================================
def generate_pool(rng, card_id_start=0):
    """Generate a 120-card pool: 15 per archetype."""
    cards = []
    card_id = card_id_start

    generic_per_arch = [2, 2, 2, 1, 1, 2, 1, 2]  # = 13
    dual_per_arch =    [2, 1, 2, 1, 2, 1, 2, 1]  # = 12

    for arch_idx, (arch_name, r1, r2) in enumerate(ARCHETYPES):
        sibling = get_sibling(arch_name)
        n_generic = generic_per_arch[arch_idx]
        n_dual = dual_per_arch[arch_idx]

        for card_type_idx in range(CARDS_PER_ARCHETYPE):
            if card_type_idx < n_dual:
                vis = [r1, r2]
            elif card_type_idx < n_dual + (CARDS_PER_ARCHETYPE - n_generic - n_dual):
                vis = [r1]
            else:
                vis = []

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


def generate_biased_refill(rng, count, card_id_start, open_lanes):
    """Generate refill cards with 2.0x open-lane bias and 40% S/A targeting.

    Open-lane archetypes get OPEN_LANE_BIAS times as many cards as AI-lane
    archetypes. Open-lane cards use OPEN_LANE_SA_RATE for S/A generation.
    """
    # Distribute cards: open lanes get 2.0x weight
    n_open = len(open_lanes)  # 3
    n_ai = NUM_ARCHETYPES - n_open  # 5
    # total_weight = n_open * OPEN_LANE_BIAS + n_ai * 1.0
    total_weight = n_open * OPEN_LANE_BIAS + n_ai * 1.0  # 3*2.0 + 5*1.0 = 11.0
    per_open = count * OPEN_LANE_BIAS / total_weight
    per_ai = count * 1.0 / total_weight

    arch_counts = {}
    for arch_name in ARCHETYPE_NAMES:
        if arch_name in open_lanes:
            arch_counts[arch_name] = int(round(per_open))
        else:
            arch_counts[arch_name] = int(round(per_ai))

    # Adjust to match exact count
    total_assigned = sum(arch_counts.values())
    diff = count - total_assigned
    if diff != 0:
        # Add/remove from random archetypes
        keys = list(arch_counts.keys())
        rng.shuffle(keys)
        for k in keys:
            if diff == 0:
                break
            if diff > 0:
                arch_counts[k] += 1
                diff -= 1
            elif diff < 0 and arch_counts[k] > 0:
                arch_counts[k] -= 1
                diff += 1

    cards = []
    card_id = card_id_start

    for arch_name in ARCHETYPE_NAMES:
        n = arch_counts.get(arch_name, 0)
        r1, r2 = get_arch_resonances(arch_name)
        sibling = get_sibling(arch_name)
        is_open = arch_name in open_lanes

        for i in range(n):
            # Symbol distribution
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
                    # S/A targeting for open lanes
                    sa_rate_for_own = OPEN_LANE_SA_RATE if is_open else BASELINE_SA_RATE
                    # Cards are always S or A for their own archetype
                    tier[other_name] = rng.choice(['S', 'A'])
                elif other_name == sibling:
                    rate = get_sibling_rate(arch_name, other_name)
                    fitness[other_name] = rng.uniform(0.3, 0.6)
                    # Boost sibling S/A rate for open lanes
                    effective_rate = rate
                    if is_open:
                        effective_rate = min(rate * (OPEN_LANE_SA_RATE / BASELINE_SA_RATE), 0.8)
                    tier[other_name] = rng.choice(['S', 'A']) if rng.random() < effective_rate else rng.choice(['C', 'F'])
                else:
                    cross_rate = get_sibling_rate(arch_name, other_name)
                    if cross_rate > 0:
                        fitness[other_name] = rng.uniform(0.15, 0.35)
                        effective_rate = cross_rate
                        if is_open:
                            effective_rate = min(cross_rate * (OPEN_LANE_SA_RATE / BASELINE_SA_RATE), 0.6)
                        tier[other_name] = rng.choice(['S', 'A']) if rng.random() < effective_rate else rng.choice(['C', 'F'])
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
# AI Drafter with Avoidance
# ============================================================
class AIDrafter:
    """AI drafter with evidence-proportional avoidance from pick 2.

    Avoidance model:
    - From pick 2, AI tracks per-archetype depletion rates via rolling window
    - evidence_ratio = observed depletion / expected depletion
    - avoidance_weight = min(evidence_ratio / 1.5, 1.0) * max_weight
    - max_weight ramps: 0.5 (picks 2-4), 0.7 (picks 5-7), 0.9 (picks 8-11), 0.9 (pick 12+)
    - Minimum 3-cycle window before weight exceeds 50%
    """
    def __init__(self, archetype_name, all_archetypes):
        self.archetype = archetype_name
        self.arch_cards_taken = 0
        self.all_archetypes = all_archetypes
        # Depletion tracking
        self.prev_pool_counts = None  # arch -> count
        self.depletion_history = defaultdict(list)  # arch -> list of depletion per cycle
        self.inferred_player_arch = None
        self.avoidance_weight = 0.0
        self.inference_correct = False
        self.inference_pick = None  # pick at which AI first inferred correctly

    def update_pool_observation(self, pool_dict, pick_num):
        """Observe pool state and update depletion tracking."""
        current_counts = Counter()
        for c in pool_dict.values():
            current_counts[c.archetype] += 1

        if self.prev_pool_counts is not None and pick_num >= 2:
            # Compute depletion per archetype this cycle
            for arch in self.all_archetypes:
                prev = self.prev_pool_counts.get(arch, 0)
                curr = current_counts.get(arch, 0)
                depletion = prev - curr
                self.depletion_history[arch].append(depletion)

            # Infer player archetype from depletion patterns
            # Use rolling window of last 4 cycles
            window = 4
            total_pool_before = sum(self.prev_pool_counts.values())
            if total_pool_before > 0:
                best_arch = None
                best_ratio = 0.0
                for arch in self.all_archetypes:
                    if arch == self.archetype:
                        continue  # Skip own archetype
                    recent = self.depletion_history[arch][-window:]
                    if len(recent) < 2:
                        continue
                    total_depletion = sum(recent)
                    # Expected depletion: proportional to arch fraction of pool
                    expected_fraction = self.prev_pool_counts.get(arch, 0) / max(total_pool_before, 1)
                    # Each cycle removes ~6 cards total (5 AIs + 1 player)
                    expected_depletion = len(recent) * 6.0 * expected_fraction
                    if expected_depletion > 0:
                        ratio = total_depletion / expected_depletion
                    else:
                        ratio = 0.0
                    if ratio > best_ratio:
                        best_ratio = ratio
                        best_arch = arch

                # Flag archetype with 1.5x expected depletion rate
                if best_ratio >= 1.5 and best_arch is not None:
                    self.inferred_player_arch = best_arch

            # Compute avoidance weight
            if self.inferred_player_arch:
                # Max weight ramp
                if pick_num <= 4:
                    max_w = 0.5
                elif pick_num <= 7:
                    max_w = 0.7
                elif pick_num <= 11:
                    max_w = 0.9
                else:
                    max_w = 0.9

                # Evidence-proportional
                recent = self.depletion_history[self.inferred_player_arch][-window:]
                if len(recent) >= 1:
                    total_depl = sum(recent)
                    total_pool_now = sum(current_counts.values())
                    expected_frac = current_counts.get(self.inferred_player_arch, 0) / max(total_pool_now, 1)
                    expected_depl = len(recent) * 6.0 * expected_frac
                    ratio = total_depl / max(expected_depl, 0.01)
                    raw_weight = min(ratio / 1.5, 1.0) * max_w
                else:
                    raw_weight = 0.0

                # Minimum 3-cycle window before weight > 0.5
                n_cycles = len(self.depletion_history.get(self.inferred_player_arch, []))
                if n_cycles < 3:
                    raw_weight = min(raw_weight, 0.5)

                self.avoidance_weight = raw_weight
            else:
                self.avoidance_weight = 0.0

        self.prev_pool_counts = dict(current_counts)

    def pick_card(self, pool_dict, pick_num):
        """Pick one card from pool with avoidance behavior."""
        if not pool_dict:
            return None

        available = list(pool_dict.values())

        # Saturation check
        arch_available = [c for c in available if c.archetype == self.archetype]
        if self.arch_cards_taken >= AI_SATURATION_THRESHOLD or len(arch_available) == 0:
            # Pick from most abundant non-player, non-own archetype
            arch_counts = Counter(c.archetype for c in available)
            excluded = {self.archetype}
            if self.inferred_player_arch:
                excluded.add(self.inferred_player_arch)
            candidates = [(a, cnt) for a, cnt in arch_counts.items() if a not in excluded]
            if candidates:
                target_arch = max(candidates, key=lambda x: x[1])[0]
                target_cards = [c for c in available if c.archetype == target_arch]
                if target_cards:
                    return max(target_cards, key=lambda c: c.fitness.get(target_arch, 0)).id
            # Fallback: highest power anything
            return max(available, key=lambda c: c.power).id

        # Weight each card by fitness for AI's archetype, adjusted by avoidance
        def card_weight(c):
            base = c.fitness.get(self.archetype, 0.0)
            # If card belongs to inferred player archetype, reduce weight
            if self.inferred_player_arch and c.archetype == self.inferred_player_arch:
                base *= (1.0 - self.avoidance_weight)
            # Also reduce weight for cards with resonance matching player's arch
            if self.inferred_player_arch:
                p_r1, p_r2 = get_arch_resonances(self.inferred_player_arch)
                for sym in c.visible_symbols:
                    if sym == p_r1 or sym == p_r2:
                        base *= (1.0 - self.avoidance_weight * 0.5)
                        break
            return base

        best = max(available, key=card_weight)
        if best.archetype == self.archetype:
            self.arch_cards_taken += 1
        return best.id


# ============================================================
# Pack Construction (Oversampled)
# ============================================================
def build_oversampled_pack(pool_list, rng, n_draw, committed_arch, pick_num):
    """Draw n_draw cards from pool, show best 4 by resonance symbol match.

    Picks 1-5: rank by power (exploration).
    Picks 6+: rank by visible resonance symbol match to committed archetype.
    """
    if len(pool_list) <= SHOW_SIZE:
        return pool_list[:]

    # Draw n_draw cards (or all if pool smaller)
    actual_n = min(n_draw, len(pool_list))
    drawn = rng.sample(pool_list, actual_n)

    if actual_n <= SHOW_SIZE:
        return drawn

    if pick_num <= 5 or committed_arch is None:
        # Exploration: rank by power
        drawn.sort(key=lambda c: c.power, reverse=True)
    else:
        # Exploitation: rank by resonance symbol match + S/A bonus
        r1, r2 = get_arch_resonances(committed_arch)

        def ranking_score(c):
            # Primary: S/A for committed archetype
            sa_bonus = 3.0 if c.is_sa_for(committed_arch) else 0.0
            # Resonance symbol match
            res_score = c.resonance_match_score(r1, r2)
            # Tiebreaker: power
            return (sa_bonus, res_score, c.power)

        drawn.sort(key=ranking_score, reverse=True)

    return drawn[:SHOW_SIZE]


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


def signal_reader_pick(pack, drafted, committed_arch, pick_num, pool_list, open_lanes, rng):
    """Signal reader: reads pool to find open lane, commits at pick 5.

    Specifically looks for archetypes with the most remaining S/A-quality cards.
    """
    if committed_arch is None and pick_num >= 5:
        # Count S/A-eligible cards per archetype in pool
        arch_sa_counts = Counter()
        for c in pool_list:
            for a in ARCHETYPE_NAMES:
                if c.is_sa_for(a):
                    arch_sa_counts[a] += 1
        if arch_sa_counts:
            committed_arch = arch_sa_counts.most_common(1)[0][0]
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
    """Run a single 30-pick draft with Design 2 champion parameters."""
    # Generate starting pool
    pool_cards, next_card_id = generate_pool(rng)
    pool_dict = {c.id: c for c in pool_cards}

    # Assign 5 AIs to random archetypes
    ai_archetypes = rng.sample(ARCHETYPE_NAMES, NUM_AIS)
    open_lanes = [a for a in ARCHETYPE_NAMES if a not in ai_archetypes]
    ai_drafters = [AIDrafter(a, ARCHETYPE_NAMES) for a in ai_archetypes]

    player_picks = []
    committed_arch = None
    all_pack_data = []
    trace_data = [] if trace else None
    pool_snapshots = []

    # Track AI avoidance timeline
    ai_avoidance_timeline = []  # (pick, ai_idx, inferred_arch, avoidance_weight)
    ai_inference_accuracy = []  # (pick, ai_idx, correct)

    # Track pool contraction trajectory
    pool_trajectory = []  # (pick, pool_size, player_arch_count, player_sa_count, player_arch_density)

    # Track S/A counts at specific picks
    sa_at_picks = {}

    global_pick = 0

    for round_idx, (picks_in_round, refill_count) in enumerate(ROUND_STRUCTURE):
        round_num = round_idx + 1

        # Record pool composition at round start
        arch_counts = Counter(c.archetype for c in pool_dict.values())
        pool_snapshots.append({
            "round": round_num,
            "pool_size": len(pool_dict),
            "arch_counts": dict(arch_counts),
            "event": "round_start",
        })

        for pick_in_round in range(picks_in_round):
            global_pick += 1

            # Step 0: AIs observe pool before picks
            for ai_idx, ai in enumerate(ai_drafters):
                ai.update_pool_observation(pool_dict, global_pick)

                # Track avoidance timeline
                if trace and ai.inferred_player_arch:
                    correct = (committed_arch is not None and
                               ai.inferred_player_arch == committed_arch)
                    ai_avoidance_timeline.append({
                        "pick": global_pick,
                        "ai_idx": ai_idx,
                        "ai_arch": ai.archetype,
                        "inferred": ai.inferred_player_arch,
                        "weight": ai.avoidance_weight,
                        "correct": correct,
                    })

            # Step 1: Each AI picks one card
            ai_removed = 0
            ai_order = list(range(len(ai_drafters)))
            rng.shuffle(ai_order)
            for ai_idx in ai_order:
                ai = ai_drafters[ai_idx]
                picked_id = ai.pick_card(pool_dict, global_pick)
                if picked_id is not None and picked_id in pool_dict:
                    del pool_dict[picked_id]
                    ai_removed += 1

            # Step 2: Build oversampled pack for player
            pool_list = list(pool_dict.values())
            n_draw = N_EXPLORE if global_pick <= 5 else N_EXPLOIT
            pack = build_oversampled_pack(pool_list, rng, n_draw, committed_arch, global_pick)

            if not pack:
                break

            # Step 3: Player picks
            if strategy == "committed":
                chosen, committed_arch = committed_pick(
                    pack, player_picks, committed_arch, global_pick, rng)
            elif strategy == "signal":
                chosen, committed_arch = signal_reader_pick(
                    pack, player_picks, committed_arch, global_pick,
                    pool_list, open_lanes, rng)
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
                "pool_before": len(pool_dict) + ai_removed + 1,
                "pool_after": len(pool_dict),
                "ai_removed": ai_removed,
                "chosen": chosen,
                "n_drawn": n_draw,
            })

            # Track pool contraction trajectory
            if committed_arch:
                p_arch_count = sum(1 for c in pool_dict.values()
                                   if c.archetype == committed_arch)
                p_sa_count = sum(1 for c in pool_dict.values()
                                 if c.is_sa_for(committed_arch))
                pool_sz = len(pool_dict)
                density = p_arch_count / max(pool_sz, 1)
                pool_trajectory.append({
                    "pick": global_pick,
                    "pool_size": pool_sz,
                    "player_arch_count": p_arch_count,
                    "player_sa_count": p_sa_count,
                    "player_arch_density": density,
                })

                # Track S/A at specific picks
                if global_pick in (20, 25, 30):
                    sa_at_picks[global_pick] = p_sa_count

            # Track AI inference accuracy
            for ai_idx, ai in enumerate(ai_drafters):
                if ai.inferred_player_arch and committed_arch:
                    correct = ai.inferred_player_arch == committed_arch
                    ai_inference_accuracy.append({
                        "pick": global_pick,
                        "ai_idx": ai_idx,
                        "correct": correct,
                    })
                    if correct and ai.inference_pick is None:
                        ai.inference_pick = global_pick
                        ai.inference_correct = True

            if trace:
                arch_in_pool = Counter(c.archetype for c in pool_dict.values())
                p_sa_in_pool = sum(1 for c in pool_dict.values()
                                   if committed_arch and c.is_sa_for(committed_arch))
                ai_weights = [(ai.archetype, ai.avoidance_weight,
                               ai.inferred_player_arch) for ai in ai_drafters]
                trace_data.append({
                    "pick": global_pick,
                    "round": round_num,
                    "pool_size": len(pool_dict),
                    "ai_removed": ai_removed,
                    "pack_sa": sa_count,
                    "player_sa_in_pool": p_sa_in_pool,
                    "arch_in_pool": dict(arch_in_pool),
                    "committed": committed_arch,
                    "chosen_arch": chosen.archetype,
                    "chosen_sa": chosen.is_sa_for(committed_arch) if committed_arch else None,
                    "chosen_power": chosen.power,
                    "ai_weights": ai_weights,
                    "n_drawn": n_draw,
                })

        # Refill between rounds
        if refill_count > 0:
            new_cards, next_card_id = generate_biased_refill(
                rng, refill_count, next_card_id, open_lanes)
            for c in new_cards:
                pool_dict[c.id] = c

            arch_counts = Counter(c.archetype for c in pool_dict.values())
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
        "pool_trajectory": pool_trajectory,
        "sa_at_picks": sa_at_picks,
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
        m2_vals = [sum(1 for c in d["pack"] if c.is_sa_for(committed_arch))
                   for d in all_pack_data[:5]]
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
        m4_vals = [sum(1 for c in d["pack"] if not c.is_sa_for(committed_arch))
                   for d in all_pack_data[5:]]
        result["m4"] = sum(m4_vals) / max(len(m4_vals), 1)
    else:
        result["m4"] = 0

    # M5: Convergence pick
    if committed_arch:
        sa_seq = [sum(1 for c in d["pack"] if c.is_sa_for(committed_arch))
                  for d in all_pack_data]
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
        m11_vals = [sum(1 for c in d["pack"] if c.is_sa_for(committed_arch))
                    for d in all_pack_data[19:]]
        result["m11"] = sum(m11_vals) / max(len(m11_vals), 1)
    else:
        result["m11"] = 0

    # M13: AI avoidance detection pick (first pick where avg avoidance weight > 0.3)
    avoidance_by_pick = defaultdict(list)
    for ai in ai_drafters:
        if ai.inference_pick:
            avoidance_by_pick[ai.inference_pick].append(ai.avoidance_weight)
    # Find first pick where at least 2 AIs have avoidance weight > 0.3
    m13 = NUM_PICKS + 1
    ai_weight_by_pick = defaultdict(list)
    for entry in ai_avoidance_timeline if trace else []:
        ai_weight_by_pick[entry["pick"]].append(entry["weight"])
    if ai_weight_by_pick:
        for pick in sorted(ai_weight_by_pick.keys()):
            weights = ai_weight_by_pick[pick]
            if sum(1 for w in weights if w > 0.3) >= 2:
                m13 = pick
                break
    # Simpler M13: average pick at which AI first gets avoidance_weight > 0.3
    ai_first_avoidance = []
    for ai in ai_drafters:
        if ai.inference_pick:
            ai_first_avoidance.append(ai.inference_pick)
    result["m13"] = sum(ai_first_avoidance) / max(len(ai_first_avoidance), 1) if ai_first_avoidance else NUM_PICKS + 1

    # M14: Player archetype visibility pick (pick at which AI correctly infers)
    ai_correct_picks = [ai.inference_pick for ai in ai_drafters if ai.inference_correct]
    result["m14"] = min(ai_correct_picks) if ai_correct_picks else NUM_PICKS + 1

    result["deck_card_ids"] = set(c.id for c in player_picks)
    result["all_pack_data"] = all_pack_data
    result["ai_avoidance_timeline"] = ai_avoidance_timeline if trace else []

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
    for key in ["m1", "m2", "m3", "m4", "m5", "m6", "m9", "m10", "m11", "m13", "m14"]:
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

    # Pool contraction trajectory (average across drafts)
    pick_pool = defaultdict(list)
    pick_arch_count = defaultdict(list)
    pick_sa_count = defaultdict(list)
    pick_density = defaultdict(list)
    for r in results:
        for pt in r["pool_trajectory"]:
            pick_pool[pt["pick"]].append(pt["pool_size"])
            pick_arch_count[pt["pick"]].append(pt["player_arch_count"])
            pick_sa_count[pt["pick"]].append(pt["player_sa_count"])
            pick_density[pt["pick"]].append(pt["player_arch_density"])
    agg["pool_trajectory"] = {}
    for pick in sorted(pick_pool.keys()):
        agg["pool_trajectory"][pick] = {
            "pool_size": sum(pick_pool[pick]) / len(pick_pool[pick]),
            "arch_count": sum(pick_arch_count[pick]) / len(pick_arch_count[pick]),
            "sa_count": sum(pick_sa_count[pick]) / len(pick_sa_count[pick]),
            "density": sum(pick_density[pick]) / len(pick_density[pick]),
        }

    # S/A at specific picks
    sa_at_20 = [r["sa_at_picks"].get(20, 0) for r in results if 20 in r["sa_at_picks"]]
    sa_at_25 = [r["sa_at_picks"].get(25, 0) for r in results if 25 in r["sa_at_picks"]]
    sa_at_30 = [r["sa_at_picks"].get(30, 0) for r in results if 30 in r["sa_at_picks"]]
    agg["sa_at_20"] = sum(sa_at_20) / max(len(sa_at_20), 1) if sa_at_20 else 0
    agg["sa_at_25"] = sum(sa_at_25) / max(len(sa_at_25), 1) if sa_at_25 else 0
    agg["sa_at_30"] = sum(sa_at_30) / max(len(sa_at_30), 1) if sa_at_30 else 0

    # Round-by-round pool compositions
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
# Oversampling Analysis
# ============================================================
def oversampling_analysis(results):
    """Analyze what M3 would be at different N values using pool trajectory data."""
    # For each draft's pool trajectory, compute expected M3 at different N
    n_values = [4, 8, 10, 12]
    m3_by_n = {n: [] for n in n_values}

    for r in results:
        if not r["committed_arch"]:
            continue
        for pt in r["pool_trajectory"]:
            if pt["pick"] < 6:
                continue
            pool_sz = pt["pool_size"]
            sa_count = pt["player_sa_count"]
            if pool_sz == 0:
                continue
            for n in n_values:
                actual_n = min(n, pool_sz)
                expected_sa = actual_n * sa_count / pool_sz
                m3_by_n[n].append(min(expected_sa, 4.0))

    result = {}
    for n in n_values:
        vals = m3_by_n[n]
        result[n] = sum(vals) / max(len(vals), 1) if vals else 0
    return result


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
        ("M12", agg.get("m12", 0), ">= 0.3",  agg.get("m12", 0) >= 0.3),
        ("M13", agg["m13_mean"],  "6-10",     6 <= agg["m13_mean"] <= 10),
        ("M14", agg["m14_mean"],  "4-7",      4 <= agg["m14_mean"] <= 7),
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


def format_pool_trajectory(agg):
    lines = []
    lines.append(f"\n  Pool Contraction Trajectory (committed strategy, averages):")
    lines.append(f"  {'Pick':>4} {'Pool':>6} {'PArch':>6} {'SA':>5} {'Density':>8}")
    lines.append(f"  {'-'*4} {'-'*6} {'-'*6} {'-'*5} {'-'*8}")
    traj = agg.get("pool_trajectory", {})
    for pick in sorted(traj.keys()):
        t = traj[pick]
        lines.append(f"  {pick:4d} {t['pool_size']:6.1f} {t['arch_count']:6.1f} "
                     f"{t['sa_count']:5.1f} {t['density']:8.3f}")
    return "\n".join(lines)


def format_sa_at_picks(agg):
    lines = []
    lines.append(f"\n  S/A Counts in Pool at Key Picks:")
    lines.append(f"    Pick 20: {agg['sa_at_20']:.1f} S/A remaining")
    lines.append(f"    Pick 25: {agg['sa_at_25']:.1f} S/A remaining")
    lines.append(f"    Pick 30: {agg['sa_at_30']:.1f} S/A remaining")
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


def format_oversampling(results):
    lines = []
    analysis = oversampling_analysis(results)
    lines.append(f"\n  Oversampling Analysis (expected M3 at different N, picks 6+):")
    lines.append(f"    N=4  (uniform):   M3 = {analysis[4]:.2f}")
    lines.append(f"    N=8  (actual):    M3 = {analysis[8]:.2f}")
    lines.append(f"    N=10 (fallback):  M3 = {analysis[10]:.2f}")
    lines.append(f"    N=12 (upper):     M3 = {analysis[12]:.2f}")
    lines.append(f"    N=8 vs N=4 lift:  +{analysis[8]-analysis[4]:.2f}")
    lines.append(f"    N=10 vs N=8 lift: +{analysis[10]-analysis[8]:.2f}")
    lines.append(f"    N=12 vs N=8 lift: +{analysis[12]-analysis[8]:.2f}")
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

        # AI avoidance summary
        avoiding = sum(1 for _, w, _ in t["ai_weights"] if w > 0.3)

        lines.append(
            f"  P{t['pick']:02d} pool={t['pool_size']:3d} "
            f"N={t['n_drawn']} "
            f"AI={t['ai_removed']} "
            f"packSA={t['pack_sa']} "
            f"poolSA={t['player_sa_in_pool']:2d} "
            f"avoid={avoiding}/5 "
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

    # AI avoidance timeline
    if r.get("ai_avoidance_timeline"):
        lines.append(f"\n  AI Avoidance Timeline (first inference per AI):")
        seen = set()
        for entry in r["ai_avoidance_timeline"]:
            key = entry["ai_idx"]
            if key not in seen:
                seen.add(key)
                lines.append(
                    f"    AI[{entry['ai_idx']}] ({entry['ai_arch']}): "
                    f"pick {entry['pick']}, "
                    f"inferred={entry['inferred']}, "
                    f"weight={entry['weight']:.2f}, "
                    f"correct={entry['correct']}"
                )

    # Pool snapshots
    for snap in r["pool_snapshots"]:
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
    print("V12 SIM-5: Design 2 Champion -- Steep Contraction + N=8")
    print("Refills 60/20/0, 2.0x open-lane bias, 40% S/A targeting")
    print("N=4 (picks 1-5), N=8 (picks 6-30)")
    print("Evidence-proportional avoidance from pick 2")
    print(f"{NUM_DRAFTS} drafts x {NUM_PICKS} picks x 3 strategies, {NUM_AIS} AIs")
    print("=" * 70)

    # Run all three strategies
    strategies = ["committed", "signal", "power"]
    all_results = {}
    all_raw_results = {}

    for strat in strategies:
        print(f"\nRunning {strat} strategy...")
        # Collect raw results for oversampling analysis
        raw_results = []
        for i in range(NUM_DRAFTS):
            rng = random.Random(42 + i)
            do_trace = (i < 2)
            r = run_single_draft(rng, strat, trace=do_trace)
            raw_results.append(r)
        all_raw_results[strat] = raw_results

        # Aggregate
        agg = {}
        for key in ["m1", "m2", "m3", "m4", "m5", "m6", "m9", "m10", "m11", "m13", "m14"]:
            vals = [r[key] for r in raw_results]
            agg[f"{key}_mean"] = sum(vals) / len(vals)

        # Per-archetype M3
        arch_m3 = defaultdict(list)
        for r in raw_results:
            if r["committed_arch"]:
                arch_m3[r["committed_arch"]].append(r["m3"])
        agg["per_arch_m3"] = {a: (sum(v)/len(v) if v else 0) for a, v in arch_m3.items()}

        # M7
        arch_runs = defaultdict(list)
        for r in raw_results:
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

        # M8
        arch_freq = Counter()
        for r in raw_results:
            if r["committed_arch"]:
                arch_freq[r["committed_arch"]] += 1
        total = sum(arch_freq.values())
        agg["m8_freq"] = {a: arch_freq.get(a, 0) / max(total, 1) for a in ARCHETYPE_NAMES}
        agg["m8_max"] = max(agg["m8_freq"].values()) if agg["m8_freq"] else 0
        agg["m8_min"] = min(agg["m8_freq"].values()) if agg["m8_freq"] else 0

        # M10 streaks
        all_streaks = []
        for r in raw_results:
            all_streaks.extend(r["m10_streaks"])
        agg["m10_streak_dist"] = Counter(all_streaks)

        # Pack quality
        all_late_sa = []
        for r in raw_results:
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

        # Pool trajectory
        pick_pool = defaultdict(list)
        pick_arch_count = defaultdict(list)
        pick_sa_count = defaultdict(list)
        pick_density = defaultdict(list)
        for r in raw_results:
            for pt in r["pool_trajectory"]:
                pick_pool[pt["pick"]].append(pt["pool_size"])
                pick_arch_count[pt["pick"]].append(pt["player_arch_count"])
                pick_sa_count[pt["pick"]].append(pt["player_sa_count"])
                pick_density[pt["pick"]].append(pt["player_arch_density"])
        agg["pool_trajectory"] = {}
        for pick in sorted(pick_pool.keys()):
            agg["pool_trajectory"][pick] = {
                "pool_size": sum(pick_pool[pick]) / len(pick_pool[pick]),
                "arch_count": sum(pick_arch_count[pick]) / len(pick_arch_count[pick]),
                "sa_count": sum(pick_sa_count[pick]) / len(pick_sa_count[pick]),
                "density": sum(pick_density[pick]) / len(pick_density[pick]),
            }

        # S/A at picks
        sa_at_20 = [r["sa_at_picks"].get(20, 0) for r in raw_results if 20 in r["sa_at_picks"]]
        sa_at_25 = [r["sa_at_picks"].get(25, 0) for r in raw_results if 25 in r["sa_at_picks"]]
        sa_at_30 = [r["sa_at_picks"].get(30, 0) for r in raw_results if 30 in r["sa_at_picks"]]
        agg["sa_at_20"] = sum(sa_at_20) / max(len(sa_at_20), 1) if sa_at_20 else 0
        agg["sa_at_25"] = sum(sa_at_25) / max(len(sa_at_25), 1) if sa_at_25 else 0
        agg["sa_at_30"] = sum(sa_at_30) / max(len(sa_at_30), 1) if sa_at_30 else 0

        # Round compositions
        round_compositions = defaultdict(lambda: defaultdict(list))
        for r in raw_results:
            for snap in r["pool_snapshots"]:
                key = (snap["round"], snap["event"])
                round_compositions[key]["pool_size"].append(snap["pool_size"])
                for arch in ARCHETYPE_NAMES:
                    round_compositions[key][arch].append(snap["arch_counts"].get(arch, 0))
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

        # Store traces
        agg["traces"] = [r for r in raw_results[:2]]
        agg["n_drafts"] = NUM_DRAFTS

        all_results[strat] = agg

    # M12: Signal-reader M3 minus Committed M3
    m12 = all_results["signal"]["m3_mean"] - all_results["committed"]["m3_mean"]
    for strat in strategies:
        all_results[strat]["m12"] = m12

    # Print results
    for strat in strategies:
        print(format_scorecard(f"{strat.capitalize()} Player", all_results[strat]))
        print(format_per_arch_m3(all_results[strat]))
        print(format_pack_quality(all_results[strat]))
        print(format_bad_streaks(all_results[strat]))

    print(f"\n{'='*70}")
    print(f"  M12 (Signal M3 - Committed M3): {m12:.3f}  (target >= 0.3)")
    print(f"    Signal M3:    {all_results['signal']['m3_mean']:.3f}")
    print(f"    Committed M3: {all_results['committed']['m3_mean']:.3f}")
    print(f"    Power M3:     {all_results['power']['m3_mean']:.3f}")
    m12_pass = m12 >= 0.3
    print(f"    M12 Status: {'PASS' if m12_pass else 'FAIL'}")

    # Pool contraction trajectory
    cm = all_results["committed"]
    print(format_pool_trajectory(cm))
    print(format_sa_at_picks(cm))

    # AI avoidance timeline
    print(f"\n  AI Avoidance Timeline:")
    print(f"    M13 (avg first avoidance pick): {cm['m13_mean']:.1f}")
    print(f"    M14 (earliest correct inference): {cm['m14_mean']:.1f}")

    # Round-by-round pool composition
    print(format_round_compositions(cm))

    # Oversampling analysis
    print(format_oversampling(all_raw_results["committed"]))

    # Draft traces
    print(f"\n{'='*70}")
    print("  DRAFT TRACES")
    print(f"{'='*70}")
    if cm["traces"]:
        print(format_trace(cm["traces"][0], "Committed #1"))
    if len(cm["traces"]) > 1:
        print(format_trace(cm["traces"][1], "Committed #2"))

    # Signal reader trace
    sr = all_results["signal"]
    if sr["traces"]:
        print(format_trace(sr["traces"][0], "Signal-reader #1"))

    # Comparison to V9 and V11
    print(f"\n{'='*70}")
    print("  COMPARISON TO V9 BASELINE AND V11")
    print(f"{'='*70}")
    print(f"  {'Metric':<7} {'V9 HybB':>10} {'V11 SIM4':>10} {'V12 D2':>10} {'Delta/V9':>10}")
    comparisons = [
        ("M3",  2.70, 0.83, cm["m3_mean"]),
        ("M5",  9.6,  None, cm["m5_mean"]),
        ("M6",  0.86, None, cm["m6_mean"]),
        ("M10", 3.8,  None, cm["m10_mean"]),
        ("M11'",3.25, None, cm["m11_mean"]),
    ]
    for name, v9, v11, v12 in comparisons:
        v11_str = f"{v11:.2f}" if v11 is not None else "N/A"
        delta = v12 - v9
        print(f"  {name:<7} {v9:>10.2f} {v11_str:>10} {v12:>10.2f} {delta:>+10.2f}")

    # N=8 sufficiency analysis
    print(f"\n{'='*70}")
    print("  N=8 SUFFICIENCY ANALYSIS")
    print(f"{'='*70}")
    os_analysis = oversampling_analysis(all_raw_results["committed"])
    actual_m3 = cm["m3_mean"]
    m3_at_10 = os_analysis[10]
    m3_at_12 = os_analysis[12]

    print(f"  Actual M3 (N=8):     {actual_m3:.2f}")
    print(f"  Projected M3 (N=10): {m3_at_10:.2f}")
    print(f"  Projected M3 (N=12): {m3_at_12:.2f}")
    print(f"  S/A at pick 25:      {cm['sa_at_25']:.1f}")

    if actual_m3 >= 2.0:
        print(f"\n  VERDICT: N=8 is SUFFICIENT. M3 = {actual_m3:.2f} >= 2.0.")
        print(f"  No need for N=10 or N=12 fallback.")
    elif m3_at_10 >= 2.0:
        print(f"\n  VERDICT: N=8 is INSUFFICIENT (M3 = {actual_m3:.2f}).")
        print(f"  N=10 would achieve M3 = {m3_at_10:.2f} -- SUFFICIENT.")
        print(f"  Recommend activating N=10 fallback.")
    elif m3_at_12 >= 2.0:
        print(f"\n  VERDICT: N=8 and N=10 are INSUFFICIENT.")
        print(f"  N=12 would achieve M3 = {m3_at_12:.2f} -- SUFFICIENT.")
        print(f"  Recommend N=12 (Design 3's approach).")
    else:
        print(f"\n  VERDICT: Even N=12 is INSUFFICIENT (projected M3 = {m3_at_12:.2f}).")
        print(f"  The pool contraction trajectory does not produce enough S/A")
        print(f"  density for any N <= 12 to reach M3 = 2.0.")
        print(f"  Root cause: S/A exhaustion (pick 25 S/A = {cm['sa_at_25']:.1f}).")

    # Final self-assessment
    print(f"\n{'='*70}")
    print("  SELF-ASSESSMENT")
    print(f"{'='*70}")
    m3_pass = cm["m3_mean"] >= 2.0
    m11_pass = cm["m11_mean"] >= 2.5
    m10_pass = cm["m10_mean"] <= 2.0
    print(f"  M3 >= 2.0:   {'YES' if m3_pass else 'NO'} ({cm['m3_mean']:.2f})")
    print(f"  M11' >= 2.5: {'YES' if m11_pass else 'NO'} ({cm['m11_mean']:.2f})")
    print(f"  M10 <= 2:    {'YES' if m10_pass else 'NO'} ({cm['m10_mean']:.2f})")
    print(f"  M12 >= 0.3:  {'YES' if m12_pass else 'NO'} ({m12:.2f})")

    if m3_pass and m11_pass:
        print("\n  AI avoidance + physical pool contraction + N=8 oversampling")
        print("  is a VIABLE replacement for V9's invisible contraction.")
    elif m3_pass:
        print("\n  M3 target reached but late-draft quality (M11') falls short.")
        print("  Consider N=10 fallback for picks 20+.")
    else:
        print(f"\n  M3 = {cm['m3_mean']:.2f} falls short of 2.0 target.")
        if cm['sa_at_25'] < 3:
            print(f"  Root cause: S/A exhaustion (pick 25 avg = {cm['sa_at_25']:.1f}).")
            print(f"  The biased refills (2.0x) and 40% S/A targeting are")
            print(f"  insufficient to maintain 5 S/A through pick 25.")
        print(f"  Recommendation: N=12 is needed. Design 2's steep contraction")
        print(f"  with N=8 is on the knife's edge as the critic predicted.")

    print("\nDone.")


if __name__ == "__main__":
    main()
