#!/usr/bin/env python3
"""
V12 Simulation Agent 1: Design 3 Champion — Moderate Pressure + Floor (N=12)

Algorithm:
- Starting pool: 120 cards, 8 archetypes (15 per archetype)
- Fitness: Graduated Realistic (~36% weighted-average sibling A-tier rate)
- 5 AIs, each assigned 1 of 5 archetypes (3 open lanes)
- AI avoidance: delayed onset pick 8, sigmoid ramp to 80% by pick 15
- Refills: 50/30/0 (3 rounds of 10 picks), 1.7x open-lane bias
- Pack construction: Draw N=12 from pool, rank by fitness, show best 4
- Floor slot: guarantee 1 S/A in shown 4 when any S/A drawn
- Variants: B1 (pair-affinity ranking) and B2 (visible-symbol-only ranking)
"""

import random
import math
import statistics
from dataclasses import dataclass, field
from typing import Optional
from collections import defaultdict

# ─── Constants ───────────────────────────────────────────────────────────────

NUM_DRAFTS = 1000
NUM_PICKS = 30
NUM_AIS = 5
NUM_ARCHETYPES = 8
CARDS_PER_ARCHETYPE = 15
STARTING_POOL_SIZE = NUM_ARCHETYPES * CARDS_PER_ARCHETYPE  # 120
PACK_SHOW = 4
OVERSAMPLE_N_EARLY = 4   # picks 1-5
OVERSAMPLE_N_LATE = 12   # picks 6-30

REFILL_SCHEDULE = {10: 50, 20: 30}  # after pick X, add Y cards
OPEN_LANE_BIAS = 1.7
PICKS_PER_ROUND = 10

# AI avoidance sigmoid: avoidance_strength(pick) = 0 for pick<8, then sigmoid
AVOIDANCE_ONSET = 8
AVOIDANCE_MAX = 0.80

# AI saturation threshold
AI_SATURATION = 12

# Archetype definitions with sibling A-tier rates
ARCHETYPES = [
    "Flash/Tempo",        # 0: Zephyr/Ember
    "Blink/Flicker",      # 1: Ember/Zephyr
    "Storm/Spellslinger", # 2: Ember/Stone
    "Self-Discard",       # 3: Stone/Ember
    "Self-Mill/Reanimator",# 4: Stone/Tide
    "Sacrifice/Abandon",  # 5: Tide/Stone
    "Warriors/Midrange",  # 6: Tide/Zephyr
    "Ramp/Spirit Animals",# 7: Zephyr/Tide
]

RESONANCE_SYMBOLS = ["Zephyr", "Ember", "Stone", "Tide"]

ARCHETYPE_SYMBOLS = {
    0: ("Zephyr", "Ember"),
    1: ("Ember", "Zephyr"),
    2: ("Ember", "Stone"),
    3: ("Stone", "Ember"),
    4: ("Stone", "Tide"),
    5: ("Tide", "Stone"),
    6: ("Tide", "Zephyr"),
    7: ("Zephyr", "Tide"),
}

# Sibling A-tier rates by pair (adjacent archetypes share a symbol)
PAIR_SA_RATES = {
    (6, 5): 0.50,  # Warriors/Sacrifice (Tide)
    (5, 6): 0.50,
    (3, 4): 0.40,  # Self-Discard/Self-Mill (Stone)
    (4, 3): 0.40,
    (1, 2): 0.30,  # Blink/Storm (Ember)
    (2, 1): 0.30,
    (0, 7): 0.25,  # Flash/Ramp (Zephyr)
    (7, 0): 0.25,
}

# Each archetype's own S/A rate (based on shared pair rate with neighbors)
# Use the higher of the two pair rates the archetype participates in
def get_archetype_sa_rate(arch_id):
    """Get the S/A rate for cards in this archetype."""
    left = (arch_id - 1) % NUM_ARCHETYPES
    right = (arch_id + 1) % NUM_ARCHETYPES
    pair_left = PAIR_SA_RATES.get((arch_id, left), 0.36)
    pair_right = PAIR_SA_RATES.get((arch_id, right), 0.36)
    # Use average of the two pair rates this archetype participates in
    return (pair_left + pair_right) / 2.0

# Precompute
ARCHETYPE_SA_RATES = {}
for i in range(NUM_ARCHETYPES):
    left = (i - 1) % NUM_ARCHETYPES
    right = (i + 1) % NUM_ARCHETYPES
    rl = PAIR_SA_RATES.get((i, left), 0.36)
    rr = PAIR_SA_RATES.get((i, right), 0.36)
    ARCHETYPE_SA_RATES[i] = (rl + rr) / 2.0


# ─── Card Model ──────────────────────────────────────────────────────────────

@dataclass
class SimCard:
    card_id: int
    archetype: int  # primary archetype index
    is_sa: bool     # S-tier or A-tier for its primary archetype
    visible_symbols: list  # list of resonance symbol strings
    pair_affinity: dict = field(default_factory=dict)  # arch_id -> float [0,1]

    def fitness_for_archetype_pair_affinity(self, target_arch):
        """Return pair-affinity fitness for a target archetype."""
        return self.pair_affinity.get(target_arch, 0.0)

    def fitness_for_archetype_symbol(self, target_arch):
        """Return symbol-match fitness for a target archetype."""
        target_primary, target_secondary = ARCHETYPE_SYMBOLS[target_arch]
        score = 0.0
        for sym in self.visible_symbols:
            if sym == target_primary:
                score += 0.5
            elif sym == target_secondary:
                score += 0.3
        # Bonus for being S/A of own archetype if matching
        if self.archetype == target_arch and self.is_sa:
            score += 0.2
        return score


_card_id_counter = 0

def make_card(archetype_id, is_sa):
    """Create a card for a given archetype."""
    global _card_id_counter
    _card_id_counter += 1

    primary_sym, secondary_sym = ARCHETYPE_SYMBOLS[archetype_id]

    # Visible symbol distribution: ~11% generic, ~79% single, ~10% dual
    r = random.random()
    if r < 0.11:
        visible_symbols = []
    elif r < 0.90:
        # Single symbol: 70% primary, 30% secondary
        if random.random() < 0.70:
            visible_symbols = [primary_sym]
        else:
            visible_symbols = [secondary_sym]
    else:
        visible_symbols = [primary_sym, secondary_sym]

    # Pair affinity: high for own archetype, moderate for siblings, low for others
    pair_affinity = {}
    for a in range(NUM_ARCHETYPES):
        if a == archetype_id:
            pair_affinity[a] = 0.85 + random.random() * 0.15 if is_sa else 0.30 + random.random() * 0.20
        elif abs(a - archetype_id) == 1 or abs(a - archetype_id) == NUM_ARCHETYPES - 1:
            # Sibling archetype
            pair_affinity[a] = 0.50 + random.random() * 0.20 if is_sa else 0.20 + random.random() * 0.15
        else:
            pair_affinity[a] = random.random() * 0.15

    return SimCard(
        card_id=_card_id_counter,
        archetype=archetype_id,
        is_sa=is_sa,
        visible_symbols=visible_symbols,
        pair_affinity=pair_affinity,
    )


def generate_pool(count_per_archetype=CARDS_PER_ARCHETYPE):
    """Generate a starting pool of cards."""
    pool = []
    for arch_id in range(NUM_ARCHETYPES):
        sa_rate = ARCHETYPE_SA_RATES[arch_id]
        for _ in range(count_per_archetype):
            is_sa = random.random() < sa_rate
            pool.append(make_card(arch_id, is_sa))
    return pool


def generate_refill_cards(count, open_lanes, ai_lanes):
    """Generate refill cards with open-lane bias."""
    cards = []
    # Calculate weighted distribution
    total_weight = len(open_lanes) * OPEN_LANE_BIAS + len(ai_lanes) * 1.0
    open_weight = OPEN_LANE_BIAS / total_weight
    ai_weight = 1.0 / total_weight

    for _ in range(count):
        r = random.random()
        cumulative = 0.0
        chosen_arch = 0
        for lane in open_lanes:
            cumulative += open_weight
            if r < cumulative:
                chosen_arch = lane
                break
        else:
            for lane in ai_lanes:
                cumulative += ai_weight
                if r < cumulative:
                    chosen_arch = lane
                    break
            else:
                chosen_arch = random.choice(list(range(NUM_ARCHETYPES)))

        sa_rate = ARCHETYPE_SA_RATES[chosen_arch]
        is_sa = random.random() < sa_rate
        cards.append(make_card(chosen_arch, is_sa))
    return cards


# ─── AI Drafter ──────────────────────────────────────────────────────────────

@dataclass
class AIDrafter:
    drafter_id: int
    assigned_archetype: int
    cards_drafted: list = field(default_factory=list)
    inferred_player_archetype: Optional[int] = None
    avoidance_strength: float = 0.0

    def get_avoidance(self, pick_num):
        """Sigmoid avoidance ramp: 0 before pick 8, ramp to 80% by pick 15."""
        if pick_num < AVOIDANCE_ONSET:
            return 0.0
        # Sigmoid ramp: 30% at pick 8, 60% at pick 10, 80% at pick 14+
        x = pick_num - AVOIDANCE_ONSET
        # f(0)=0.30, f(2)=0.60, f(6)=0.80
        strength = AVOIDANCE_MAX / (1.0 + math.exp(-0.8 * (x - 2.5)))
        return min(strength, AVOIDANCE_MAX)

    def infer_player_archetype(self, pool_history, pick_num):
        """Infer player archetype from pool depletion patterns."""
        if pick_num < 5 or len(pool_history) < 3:
            return None

        # Count cards per archetype across last 3 snapshots
        recent = pool_history[-3:]
        depletion_rates = {}
        for arch_id in range(NUM_ARCHETYPES):
            counts = []
            for snapshot in recent:
                counts.append(sum(1 for c in snapshot if c.archetype == arch_id))
            if len(counts) >= 2 and counts[0] > 0:
                rate = (counts[0] - counts[-1]) / max(counts[0], 1)
                depletion_rates[arch_id] = rate

        if not depletion_rates:
            return None

        avg_rate = sum(depletion_rates.values()) / len(depletion_rates)
        # Flag archetype depleting at >= 1.8x expected rate
        for arch_id, rate in depletion_rates.items():
            if arch_id == self.assigned_archetype:
                continue  # Don't infer own archetype
            if avg_rate > 0 and rate >= 1.8 * avg_rate:
                return arch_id

        # Fallback: return archetype with highest depletion rate (excluding own)
        candidates = {k: v for k, v in depletion_rates.items()
                      if k != self.assigned_archetype}
        if candidates:
            return max(candidates, key=candidates.get)
        return None

    def pick_card(self, pool, pick_num, pool_history):
        """AI picks a card from the pool."""
        if not pool:
            return None

        self.inferred_player_archetype = self.infer_player_archetype(pool_history, pick_num)
        avoidance = self.get_avoidance(pick_num)

        primary_count = sum(1 for c in self.cards_drafted if c.archetype == self.assigned_archetype)
        saturated = primary_count >= AI_SATURATION

        best_card = None
        best_score = -1.0

        for card in pool:
            score = card.pair_affinity.get(self.assigned_archetype, 0.0)

            if saturated:
                # Reduce primary weight, expand to sibling
                if card.archetype == self.assigned_archetype:
                    score *= 0.5
                sibling_left = (self.assigned_archetype - 1) % NUM_ARCHETYPES
                sibling_right = (self.assigned_archetype + 1) % NUM_ARCHETYPES
                if card.archetype in (sibling_left, sibling_right):
                    score *= 1.3

            # Apply avoidance: reduce score for cards in inferred player archetype
            if (self.inferred_player_archetype is not None and
                card.archetype == self.inferred_player_archetype):
                score *= (1.0 - avoidance)

            if score > best_score:
                best_score = score
                best_card = card

        if best_card:
            pool.remove(best_card)
            self.cards_drafted.append(best_card)
        return best_card


# ─── Pack Construction ───────────────────────────────────────────────────────

def construct_pack_b1(pool, player_arch, pick_num):
    """B1: Pair-affinity ranking with floor slot."""
    n = OVERSAMPLE_N_EARLY if pick_num <= 5 else OVERSAMPLE_N_LATE
    n = min(n, len(pool))
    if n == 0:
        return []

    sample = random.sample(pool, n)

    if pick_num <= 5 or player_arch is None:
        # Exploration: rank by power (use pair_affinity average)
        sample.sort(key=lambda c: max(c.pair_affinity.values()), reverse=True)
        return sample[:PACK_SHOW]

    # Rank by pair-affinity for player's archetype
    sample.sort(key=lambda c: c.fitness_for_archetype_pair_affinity(player_arch), reverse=True)

    # Floor slot: guarantee 1 S/A in shown 4 if any S/A drawn
    sa_in_sample = [c for c in sample if c.is_sa and c.archetype == player_arch]
    sibling_left = (player_arch - 1) % NUM_ARCHETYPES
    sibling_right = (player_arch + 1) % NUM_ARCHETYPES
    sibling_sa = [c for c in sample if c.is_sa and c.archetype in (sibling_left, sibling_right)]
    all_sa = sa_in_sample + sibling_sa

    top4 = sample[:PACK_SHOW]

    if all_sa and not any(c.is_sa and (c.archetype == player_arch or c.archetype in (sibling_left, sibling_right)) for c in top4):
        # Floor slot fires: replace worst card in top4 with best S/A
        top4[-1] = all_sa[0]

    return top4


def construct_pack_b2(pool, player_arch, pick_num):
    """B2: Visible-symbol-only ranking with floor slot."""
    n = OVERSAMPLE_N_EARLY if pick_num <= 5 else OVERSAMPLE_N_LATE
    n = min(n, len(pool))
    if n == 0:
        return []

    sample = random.sample(pool, n)

    if pick_num <= 5 or player_arch is None:
        sample.sort(key=lambda c: max(c.pair_affinity.values()), reverse=True)
        return sample[:PACK_SHOW]

    # Rank by visible symbol match
    sample.sort(key=lambda c: c.fitness_for_archetype_symbol(player_arch), reverse=True)

    # Floor slot: same as B1
    sibling_left = (player_arch - 1) % NUM_ARCHETYPES
    sibling_right = (player_arch + 1) % NUM_ARCHETYPES
    sa_in_sample = [c for c in sample if c.is_sa and c.archetype == player_arch]
    sibling_sa = [c for c in sample if c.is_sa and c.archetype in (sibling_left, sibling_right)]
    all_sa = sa_in_sample + sibling_sa

    top4 = sample[:PACK_SHOW]

    if all_sa and not any(c.is_sa and (c.archetype == player_arch or c.archetype in (sibling_left, sibling_right)) for c in top4):
        top4[-1] = all_sa[0]

    return top4


# ─── Player Strategies ───────────────────────────────────────────────────────

def committed_player_pick(pack, player_arch, pick_num):
    """Commits to archetype at pick 5-6, takes best on-arch card."""
    if player_arch is None or pick_num <= 5:
        # Take best overall card
        return max(pack, key=lambda c: max(c.pair_affinity.values())) if pack else None

    # Take best card for committed archetype
    on_arch = [c for c in pack if c.archetype == player_arch]
    sibling_left = (player_arch - 1) % NUM_ARCHETYPES
    sibling_right = (player_arch + 1) % NUM_ARCHETYPES
    sibling = [c for c in pack if c.archetype in (sibling_left, sibling_right)]

    if on_arch:
        return max(on_arch, key=lambda c: c.pair_affinity.get(player_arch, 0))
    elif sibling:
        return max(sibling, key=lambda c: c.pair_affinity.get(player_arch, 0))
    else:
        return max(pack, key=lambda c: c.pair_affinity.get(player_arch, 0)) if pack else None


def power_chaser_pick(pack, player_arch, pick_num):
    """Ignores archetype, takes highest raw power card."""
    if not pack:
        return None
    return max(pack, key=lambda c: max(c.pair_affinity.values()))


def signal_reader_pick(pack, player_arch, pick_num, pool):
    """Reads pool state to choose best archetype, commits earlier (pick 3-4)."""
    if player_arch is None:
        return max(pack, key=lambda c: max(c.pair_affinity.values())) if pack else None

    # Like committed but reads signals earlier
    on_arch = [c for c in pack if c.archetype == player_arch]
    sibling_left = (player_arch - 1) % NUM_ARCHETYPES
    sibling_right = (player_arch + 1) % NUM_ARCHETYPES
    sibling = [c for c in pack if c.archetype in (sibling_left, sibling_right)]

    if on_arch:
        return max(on_arch, key=lambda c: c.pair_affinity.get(player_arch, 0))
    elif sibling:
        return max(sibling, key=lambda c: c.pair_affinity.get(player_arch, 0))
    else:
        return max(pack, key=lambda c: c.pair_affinity.get(player_arch, 0)) if pack else None


def choose_open_archetype(pool, open_lanes, strategy, pick_num):
    """Choose which open lane to commit to based on strategy."""
    if strategy == "signal_reader" and pick_num >= 3:
        # Signal reader evaluates pool density earlier
        best_lane = None
        best_sa_count = -1
        for lane in open_lanes:
            sa_count = sum(1 for c in pool if c.archetype == lane and c.is_sa)
            if sa_count > best_sa_count:
                best_sa_count = sa_count
                best_lane = lane
        return best_lane
    else:
        # Committed player picks at random from open lanes (pick 5-6)
        # Actually, pick the lane with most S/A remaining
        best_lane = None
        best_sa_count = -1
        for lane in open_lanes:
            sa_count = sum(1 for c in pool if c.archetype == lane and c.is_sa)
            if sa_count > best_sa_count:
                best_sa_count = sa_count
                best_lane = lane
        return best_lane


# ─── Draft Simulation ────────────────────────────────────────────────────────

@dataclass
class DraftMetrics:
    """Metrics tracked per draft."""
    m3_per_pack: list = field(default_factory=list)  # S/A in pack for committed arch, picks 6+
    m1_per_pack: list = field(default_factory=list)  # unique archetypes with S/A, picks 1-5
    m2_per_pack: list = field(default_factory=list)  # S/A for emerging arch, picks 1-5
    m4_per_pack: list = field(default_factory=list)  # off-arch C/F in pack, picks 6+
    m11_per_pack: list = field(default_factory=list)  # S/A for committed arch, picks 20+
    convergence_pick: int = 30
    deck_concentration: float = 0.0
    pool_sizes: list = field(default_factory=list)
    arch_densities: list = field(default_factory=list)
    sa_counts: list = field(default_factory=list)
    floor_fired: dict = field(default_factory=lambda: {"6-10": [0, 0], "11-20": [0, 0], "21-30": [0, 0]})
    sa_at_picks: dict = field(default_factory=dict)  # pick -> S/A count in pool
    ai_avoidance_timeline: list = field(default_factory=list)
    ai_inference_correct: list = field(default_factory=list)
    pack_qualities: list = field(default_factory=list)  # S/A count per pack for picks 6+
    player_deck: list = field(default_factory=list)


def get_floor_band(pick_num):
    if pick_num <= 10:
        return "6-10"
    elif pick_num <= 20:
        return "11-20"
    else:
        return "21-30"


def run_single_draft(strategy, variant="B1", seed=None):
    """Run a single draft and collect metrics."""
    if seed is not None:
        random.seed(seed)

    metrics = DraftMetrics()

    # Setup
    pool = generate_pool()
    ai_archetypes = random.sample(range(NUM_ARCHETYPES), NUM_AIS)
    open_lanes = [a for a in range(NUM_ARCHETYPES) if a not in ai_archetypes]

    ais = [AIDrafter(drafter_id=i, assigned_archetype=ai_archetypes[i]) for i in range(NUM_AIS)]

    player_arch = None
    player_deck = []
    pool_history = [list(pool)]

    # Commitment pick depends on strategy
    commit_pick = 3 if strategy == "signal_reader" else 5

    for pick_num in range(1, NUM_PICKS + 1):
        # Track pool state
        metrics.pool_sizes.append(len(pool))
        sa_for_player = sum(1 for c in pool if player_arch is not None and c.archetype == player_arch and c.is_sa)
        metrics.sa_counts.append(sa_for_player)
        arch_density = sum(1 for c in pool if player_arch is not None and c.archetype == player_arch) / max(len(pool), 1)
        metrics.arch_densities.append(arch_density)

        # Track S/A at specific picks
        if pick_num in (20, 25, 30):
            metrics.sa_at_picks[pick_num] = sa_for_player

        # Player commits to archetype
        if player_arch is None and pick_num >= commit_pick:
            player_arch = choose_open_archetype(pool, open_lanes, strategy, pick_num)
            metrics.convergence_pick = pick_num

        # Construct pack
        if variant == "B1":
            pack = construct_pack_b1(pool, player_arch, pick_num)
        else:
            pack = construct_pack_b2(pool, player_arch, pick_num)

        if not pack:
            break

        # Track floor slot firing (picks 6+)
        if pick_num >= 6 and player_arch is not None:
            n = min(OVERSAMPLE_N_LATE, len(pool))
            band = get_floor_band(pick_num)
            # Check if S/A was available in the full oversample draw
            # (We approximate: did the pack contain any S/A?)
            sibling_left = (player_arch - 1) % NUM_ARCHETYPES
            sibling_right = (player_arch + 1) % NUM_ARCHETYPES
            has_sa = any(c.is_sa and c.archetype in (player_arch, sibling_left, sibling_right) for c in pack)
            metrics.floor_fired[band][0] += (1 if has_sa else 0)
            metrics.floor_fired[band][1] += 1

        # Player picks
        if strategy == "committed":
            chosen = committed_player_pick(pack, player_arch, pick_num)
        elif strategy == "power_chaser":
            chosen = power_chaser_pick(pack, player_arch, pick_num)
        else:  # signal_reader
            chosen = signal_reader_pick(pack, player_arch, pick_num, pool)

        if chosen and chosen in pool:
            pool.remove(chosen)
            player_deck.append(chosen)

        # Track pack metrics
        if pick_num <= 5:
            # M1: unique archetypes with S/A in pack
            sa_archs = set(c.archetype for c in pack if c.is_sa)
            metrics.m1_per_pack.append(len(sa_archs))
            # M2: S/A for emerging archetype (use best-represented)
            if player_arch is not None:
                m2 = sum(1 for c in pack if c.is_sa and c.archetype == player_arch)
                metrics.m2_per_pack.append(m2)
        else:
            if player_arch is not None:
                # M3: S/A for committed archetype in pack
                sibling_left = (player_arch - 1) % NUM_ARCHETYPES
                sibling_right = (player_arch + 1) % NUM_ARCHETYPES
                sa_count = sum(1 for c in pack if c.is_sa and c.archetype in (player_arch, sibling_left, sibling_right))
                metrics.m3_per_pack.append(sa_count)
                metrics.pack_qualities.append(sa_count)

                # M4: off-archetype C/F
                off_arch_cf = sum(1 for c in pack if c.archetype != player_arch and not c.is_sa)
                metrics.m4_per_pack.append(off_arch_cf)

                if pick_num >= 20:
                    metrics.m11_per_pack.append(sa_count)

        # AIs pick
        for ai in ais:
            ai.pick_card(pool, pick_num, pool_history)

        # Track AI inference
        if player_arch is not None:
            for ai in ais:
                if ai.inferred_player_archetype is not None:
                    correct = ai.inferred_player_archetype == player_arch
                    metrics.ai_inference_correct.append((pick_num, correct))
                    avoidance = ai.get_avoidance(pick_num)
                    metrics.ai_avoidance_timeline.append((pick_num, avoidance, correct))

        # Save pool snapshot for AI inference
        pool_history.append(list(pool))

        # Refills
        if pick_num in REFILL_SCHEDULE:
            refill_count = REFILL_SCHEDULE[pick_num]
            refill_cards = generate_refill_cards(refill_count, open_lanes, ai_archetypes)
            pool.extend(refill_cards)

    # Deck metrics
    metrics.player_deck = player_deck
    if player_arch is not None and player_deck:
        sibling_left = (player_arch - 1) % NUM_ARCHETYPES
        sibling_right = (player_arch + 1) % NUM_ARCHETYPES
        on_arch_sa = sum(1 for c in player_deck if c.is_sa and c.archetype in (player_arch, sibling_left, sibling_right))
        metrics.deck_concentration = on_arch_sa / len(player_deck) if player_deck else 0

    return metrics, player_arch


# ─── Aggregate Results ───────────────────────────────────────────────────────

@dataclass
class AggregateResults:
    strategy: str
    variant: str
    m1: float = 0.0
    m2: float = 0.0
    m3: float = 0.0
    m4: float = 0.0
    m5: float = 0.0  # convergence pick
    m6: float = 0.0  # deck concentration
    m7: float = 0.0  # run-to-run variety (card overlap)
    m8: dict = field(default_factory=dict)  # archetype frequencies
    m9: float = 0.0  # StdDev of S/A per pack
    m10: int = 0      # max consecutive packs below 1.5 S/A
    m11: float = 0.0  # S/A per pack picks 20+
    m12: float = 0.0  # signal-reader M3 - committed M3
    m13: float = 0.0  # AI avoidance detection pick
    m14: float = 0.0  # player archetype visibility pick
    pool_trajectory: list = field(default_factory=list)
    sa_trajectory: list = field(default_factory=list)
    density_trajectory: list = field(default_factory=list)
    pack_quality_percentiles: dict = field(default_factory=dict)
    consecutive_bad_packs: list = field(default_factory=list)
    floor_rates: dict = field(default_factory=dict)
    sa_at_picks: dict = field(default_factory=dict)
    per_archetype_m3: dict = field(default_factory=dict)
    ai_inference_accuracy_by_pick: dict = field(default_factory=dict)
    ai_avoidance_by_pick: dict = field(default_factory=dict)
    draft_traces: list = field(default_factory=list)


def run_simulations(strategy, variant, num_drafts=NUM_DRAFTS):
    """Run multiple drafts and aggregate results."""
    all_m3 = []
    all_m1 = []
    all_m2 = []
    all_m4 = []
    all_m5 = []
    all_m6 = []
    all_m9_packs = []
    all_m10 = []
    all_m11 = []
    all_pool_sizes = defaultdict(list)
    all_sa_counts = defaultdict(list)
    all_densities = defaultdict(list)
    all_pack_qualities = []
    all_consecutive_bad = []
    floor_agg = {"6-10": [0, 0], "11-20": [0, 0], "21-30": [0, 0]}
    sa_at_pick_agg = defaultdict(list)
    per_arch_m3 = defaultdict(list)
    ai_inference_by_pick = defaultdict(lambda: [0, 0])
    ai_avoidance_by_pick = defaultdict(list)
    arch_chosen_count = defaultdict(int)
    draft_traces = []
    all_deck_cards = []

    for draft_idx in range(num_drafts):
        metrics, player_arch = run_single_draft(strategy, variant)

        if metrics.m3_per_pack:
            m3_avg = statistics.mean(metrics.m3_per_pack)
            all_m3.append(m3_avg)
            all_m9_packs.extend(metrics.m3_per_pack)

            # Per-archetype M3
            if player_arch is not None:
                per_arch_m3[player_arch].append(m3_avg)
                arch_chosen_count[player_arch] += 1

        if metrics.m1_per_pack:
            all_m1.append(statistics.mean(metrics.m1_per_pack))
        if metrics.m2_per_pack:
            all_m2.append(statistics.mean(metrics.m2_per_pack))
        if metrics.m4_per_pack:
            all_m4.append(statistics.mean(metrics.m4_per_pack))
        all_m5.append(metrics.convergence_pick)
        all_m6.append(metrics.deck_concentration)
        if metrics.m11_per_pack:
            all_m11.append(statistics.mean(metrics.m11_per_pack))

        # M10: max consecutive packs below 1.5 S/A
        if metrics.m3_per_pack:
            max_consec = 0
            current = 0
            for sa in metrics.m3_per_pack:
                if sa < 1.5:
                    current += 1
                    max_consec = max(max_consec, current)
                else:
                    current = 0
            all_m10.append(max_consec)
            all_consecutive_bad.append(max_consec)

        # Pool trajectory
        for i, size in enumerate(metrics.pool_sizes):
            all_pool_sizes[i].append(size)
        for i, sa in enumerate(metrics.sa_counts):
            all_sa_counts[i].append(sa)
        for i, d in enumerate(metrics.arch_densities):
            all_densities[i].append(d)

        # Floor slot
        for band in floor_agg:
            floor_agg[band][0] += metrics.floor_fired[band][0]
            floor_agg[band][1] += metrics.floor_fired[band][1]

        # S/A at picks
        for pick, count in metrics.sa_at_picks.items():
            sa_at_pick_agg[pick].append(count)

        # AI inference
        for pick_num, correct in metrics.ai_inference_correct:
            ai_inference_by_pick[pick_num][0] += (1 if correct else 0)
            ai_inference_by_pick[pick_num][1] += 1

        for pick_num, avoidance, correct in metrics.ai_avoidance_timeline:
            ai_avoidance_by_pick[pick_num].append(avoidance)

        # Pack qualities
        all_pack_qualities.extend(metrics.pack_qualities)

        # Deck cards for variety
        all_deck_cards.append(set(c.card_id for c in metrics.player_deck))

        # Save first 2 draft traces
        if draft_idx < 2:
            trace = {
                "strategy": strategy,
                "variant": variant,
                "player_arch": player_arch,
                "player_arch_name": ARCHETYPES[player_arch] if player_arch is not None else "None",
                "pool_sizes": metrics.pool_sizes[:],
                "sa_counts": metrics.sa_counts[:],
                "m3_per_pack": metrics.m3_per_pack[:],
                "convergence_pick": metrics.convergence_pick,
                "deck_size": len(metrics.player_deck),
                "deck_sa": sum(1 for c in metrics.player_deck if c.is_sa),
            }
            draft_traces.append(trace)

    # Aggregate
    results = AggregateResults(strategy=strategy, variant=variant)
    results.m1 = statistics.mean(all_m1) if all_m1 else 0
    results.m2 = statistics.mean(all_m2) if all_m2 else 0
    results.m3 = statistics.mean(all_m3) if all_m3 else 0
    results.m4 = statistics.mean(all_m4) if all_m4 else 0
    results.m5 = statistics.mean(all_m5) if all_m5 else 0
    results.m6 = statistics.mean(all_m6) if all_m6 else 0
    results.m9 = statistics.stdev(all_m9_packs) if len(all_m9_packs) > 1 else 0
    results.m10 = statistics.mean(all_m10) if all_m10 else 0
    results.m11 = statistics.mean(all_m11) if all_m11 else 0

    # M7: card overlap between consecutive runs
    if len(all_deck_cards) >= 2:
        overlaps = []
        for i in range(min(100, len(all_deck_cards) - 1)):
            overlap = len(all_deck_cards[i] & all_deck_cards[i + 1])
            total = len(all_deck_cards[i] | all_deck_cards[i + 1])
            if total > 0:
                overlaps.append(overlap / total)
        results.m7 = statistics.mean(overlaps) if overlaps else 0

    # M8: archetype frequency
    total_chosen = sum(arch_chosen_count.values())
    for arch_id in range(NUM_ARCHETYPES):
        results.m8[ARCHETYPES[arch_id]] = arch_chosen_count.get(arch_id, 0) / max(total_chosen, 1)

    # Pool trajectory (mean at each pick)
    for pick_idx in sorted(all_pool_sizes.keys()):
        results.pool_trajectory.append(statistics.mean(all_pool_sizes[pick_idx]))
    for pick_idx in sorted(all_sa_counts.keys()):
        results.sa_trajectory.append(statistics.mean(all_sa_counts[pick_idx]))
    for pick_idx in sorted(all_densities.keys()):
        results.density_trajectory.append(statistics.mean(all_densities[pick_idx]))

    # Pack quality percentiles
    if all_pack_qualities:
        sorted_pq = sorted(all_pack_qualities)
        n = len(sorted_pq)
        results.pack_quality_percentiles = {
            "p10": sorted_pq[int(n * 0.10)],
            "p25": sorted_pq[int(n * 0.25)],
            "p50": sorted_pq[int(n * 0.50)],
            "p75": sorted_pq[int(n * 0.75)],
            "p90": sorted_pq[int(n * 0.90)],
        }

    # Floor rates
    for band in floor_agg:
        if floor_agg[band][1] > 0:
            results.floor_rates[band] = floor_agg[band][0] / floor_agg[band][1]
        else:
            results.floor_rates[band] = 0

    # S/A at picks
    for pick, counts in sa_at_pick_agg.items():
        results.sa_at_picks[pick] = statistics.mean(counts)

    # Per-archetype M3
    for arch_id in range(NUM_ARCHETYPES):
        if per_arch_m3[arch_id]:
            results.per_archetype_m3[ARCHETYPES[arch_id]] = statistics.mean(per_arch_m3[arch_id])

    # AI inference accuracy by pick
    for pick in sorted(ai_inference_by_pick.keys()):
        correct, total = ai_inference_by_pick[pick]
        results.ai_inference_accuracy_by_pick[pick] = correct / total if total > 0 else 0

    # AI avoidance by pick
    for pick in sorted(ai_avoidance_by_pick.keys()):
        results.ai_avoidance_by_pick[pick] = statistics.mean(ai_avoidance_by_pick[pick])

    results.consecutive_bad_packs = all_consecutive_bad
    results.draft_traces = draft_traces

    # M13: pick at which AIs detectably change behavior (avoidance > 0.1)
    for pick in sorted(results.ai_avoidance_by_pick.keys()):
        if results.ai_avoidance_by_pick[pick] > 0.1:
            results.m13 = pick
            break

    # M14: pick at which AI correctly infers player archetype > 50% of the time
    for pick in sorted(results.ai_inference_accuracy_by_pick.keys()):
        if results.ai_inference_accuracy_by_pick[pick] > 0.5:
            results.m14 = pick
            break

    return results


# ─── Main ────────────────────────────────────────────────────────────────────

def format_results(results_by_strategy, variant):
    """Format results into a readable report section."""
    lines = []
    lines.append(f"\n{'='*70}")
    lines.append(f"  VARIANT {variant} RESULTS")
    lines.append(f"{'='*70}\n")

    for strat, r in results_by_strategy.items():
        lines.append(f"--- Strategy: {strat} ---")
        lines.append(f"  M1  (unique archs w/ S/A, picks 1-5):  {r.m1:.2f}  (target >= 3)")
        lines.append(f"  M2  (S/A for emerging arch, picks 1-5): {r.m2:.2f}  (target <= 2)")
        lines.append(f"  M3  (S/A for committed arch, picks 6+): {r.m3:.2f}  (target >= 2.0)")
        lines.append(f"  M4  (off-arch C/F per pack, picks 6+):  {r.m4:.2f}  (target >= 0.5)")
        lines.append(f"  M5  (convergence pick):                 {r.m5:.1f}  (target 5-8)")
        lines.append(f"  M6  (deck S/A concentration):           {r.m6:.1%}  (target 60-90%)")
        lines.append(f"  M7  (run-to-run card overlap):          {r.m7:.1%}  (target < 40%)")
        lines.append(f"  M9  (StdDev S/A per pack):              {r.m9:.2f}  (target >= 0.8)")
        lines.append(f"  M10 (avg max consec packs < 1.5 S/A):   {r.m10:.1f}  (target <= 2)")
        lines.append(f"  M11'(S/A per pack, picks 20+):          {r.m11:.2f}  (target >= 2.5)")
        lines.append(f"  M13 (AI avoidance detection pick):      {r.m13}    (target 6-10)")
        lines.append(f"  M14 (AI inference correct >50% pick):   {r.m14}    (target 4-7)")
        lines.append("")

    # Per-archetype M3 (from committed strategy)
    committed_r = results_by_strategy.get("committed", list(results_by_strategy.values())[0])
    lines.append("--- Per-Archetype M3 (committed strategy) ---")
    for arch_name, m3 in committed_r.per_archetype_m3.items():
        lines.append(f"  {arch_name:30s}: {m3:.2f}")
    lines.append("")

    # M8: archetype frequency
    lines.append("--- Archetype Frequency (M8) ---")
    for arch_name, freq in committed_r.m8.items():
        lines.append(f"  {arch_name:30s}: {freq:.1%}")
    lines.append("")

    # AI avoidance timeline
    lines.append("--- AI Avoidance Timeline ---")
    for pick in sorted(committed_r.ai_avoidance_by_pick.keys()):
        avoidance = committed_r.ai_avoidance_by_pick[pick]
        accuracy = committed_r.ai_inference_accuracy_by_pick.get(pick, 0)
        lines.append(f"  Pick {pick:2d}: avoidance={avoidance:.2f}, inference_accuracy={accuracy:.1%}")
    lines.append("")

    # Pool contraction trajectory
    lines.append("--- Pool Contraction Trajectory ---")
    for i, (size, sa, density) in enumerate(zip(
        committed_r.pool_trajectory,
        committed_r.sa_trajectory,
        committed_r.density_trajectory,
    )):
        pick = i + 1
        if pick in (1, 5, 8, 10, 11, 15, 20, 21, 25, 30) or pick <= 3:
            lines.append(f"  Pick {pick:2d}: pool={size:.0f}, S/A={sa:.1f}, density={density:.1%}")
    lines.append("")

    # Floor slot firing rates
    lines.append("--- Floor Slot Firing Rate ---")
    for band, rate in committed_r.floor_rates.items():
        lines.append(f"  Picks {band}: {rate:.1%}")
    lines.append("")

    # S/A at key picks
    lines.append("--- S/A Count at Key Picks ---")
    for pick, sa in sorted(committed_r.sa_at_picks.items()):
        lines.append(f"  Pick {pick}: {sa:.1f} S/A remaining in pool")
    lines.append("")

    # Pack quality distribution
    lines.append("--- Pack Quality Distribution (picks 6+) ---")
    for k, v in committed_r.pack_quality_percentiles.items():
        lines.append(f"  {k}: {v}")
    lines.append("")

    # Consecutive bad pack analysis
    if committed_r.consecutive_bad_packs:
        lines.append("--- Consecutive Bad Pack Analysis (max run < 1.5 S/A) ---")
        sorted_bad = sorted(committed_r.consecutive_bad_packs)
        n = len(sorted_bad)
        lines.append(f"  Mean: {statistics.mean(sorted_bad):.1f}")
        lines.append(f"  Median: {sorted_bad[n//2]}")
        lines.append(f"  p90: {sorted_bad[int(n*0.90)]}")
        lines.append(f"  p99: {sorted_bad[int(n*0.99)]}")
        lines.append(f"  Max: {max(sorted_bad)}")
        lines.append("")

    # Draft traces
    lines.append("--- Draft Traces ---")
    for i, trace in enumerate(committed_r.draft_traces):
        lines.append(f"\n  Trace {i+1}: {trace['strategy']} / {trace['variant']}")
        lines.append(f"    Player archetype: {trace['player_arch_name']}")
        lines.append(f"    Convergence pick: {trace['convergence_pick']}")
        lines.append(f"    Deck size: {trace['deck_size']}, S/A in deck: {trace['deck_sa']}")
        lines.append(f"    Pool sizes: {[trace['pool_sizes'][i] for i in range(0, len(trace['pool_sizes']), 5)]}")
        lines.append(f"    S/A counts: {[trace['sa_counts'][i] for i in range(0, len(trace['sa_counts']), 5)]}")
        if trace['m3_per_pack']:
            lines.append(f"    M3 by pack (picks 6+): {[f'{x:.0f}' for x in trace['m3_per_pack']]}")
    lines.append("")

    return "\n".join(lines)


def main():
    print("V12 Simulation Agent 1: Design 3 Champion")
    print(f"Running {NUM_DRAFTS} drafts x 30 picks x 3 strategies x 2 variants")
    print(f"Total simulation runs: {NUM_DRAFTS * 3 * 2}")
    print("")

    strategies = ["committed", "power_chaser", "signal_reader"]
    variants = ["B1", "B2"]

    all_results = {}

    for variant in variants:
        all_results[variant] = {}
        for strategy in strategies:
            print(f"  Running {variant}/{strategy}...", flush=True)
            results = run_simulations(strategy, variant, NUM_DRAFTS)
            all_results[variant][strategy] = results
            print(f"    M3={results.m3:.2f}, M11'={results.m11:.2f}, M6={results.m6:.1%}")

    # Print full results
    for variant in variants:
        output = format_results(all_results[variant], variant)
        print(output)

    # M12: signal-reader M3 - committed M3
    for variant in variants:
        sr_m3 = all_results[variant]["signal_reader"].m3
        cm_m3 = all_results[variant]["committed"].m3
        m12 = sr_m3 - cm_m3
        print(f"  {variant} M12 (signal-reader M3 - committed M3): {m12:.2f}  (target >= 0.3)")

    # B1 vs B2 delta
    b1_m3 = all_results["B1"]["committed"].m3
    b2_m3 = all_results["B2"]["committed"].m3
    print(f"\n  Pair-affinity M3 delta (B1 - B2): {b1_m3 - b2_m3:.2f}")
    print(f"  B1 committed M3: {b1_m3:.2f}")
    print(f"  B2 committed M3: {b2_m3:.2f}")

    # V9 comparison
    print(f"\n--- Comparison to V9 ---")
    print(f"  V9 Hybrid B M3: 2.70")
    print(f"  V12 B1 committed M3: {b1_m3:.2f} ({b1_m3/2.70*100:.0f}% of V9)")
    print(f"  V12 B2 committed M3: {b2_m3:.2f} ({b2_m3/2.70*100:.0f}% of V9)")
    print(f"  V11 SIM-4 M3: 0.83")
    print(f"  V12 B1 improvement over V11: {b1_m3 - 0.83:+.2f}")

    return all_results


if __name__ == "__main__":
    results = main()
