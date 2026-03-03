#!/usr/bin/env python3
"""
Simulation Agent 1 (Baselines) — V8 Resonance Draft Investigation
Simulates: Surge+Floor, Pair-Escalation, Lane Locking, Surge+Floor+Bias
Under: Optimistic, Graduated Realistic, Pessimistic, Hostile fitness
Pools: V7 Standard (15% dual-res), 40% Enriched (40% dual-res)
"""

import random
import math
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional
from collections import defaultdict

SEED = 42
NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 4
POOL_SIZE = 360
GENERIC_COUNT = 40
ARCHETYPE_COUNT = 8
CARDS_PER_ARCHETYPE = 40

# ── Resonance and Archetype Definitions ──

class Resonance(Enum):
    EMBER = 0
    STONE = 1
    TIDE = 2
    ZEPHYR = 3

ARCHETYPES = [
    "Flash",         # 0: Zephyr/Ember
    "Blink",         # 1: Ember/Zephyr
    "Storm",         # 2: Ember/Stone
    "SelfDiscard",   # 3: Stone/Ember
    "SelfMill",      # 4: Stone/Tide
    "Sacrifice",     # 5: Tide/Stone
    "Warriors",      # 6: Tide/Zephyr
    "Ramp",          # 7: Zephyr/Tide
]

ARCHETYPE_RESONANCES = {
    0: (Resonance.ZEPHYR, Resonance.EMBER),
    1: (Resonance.EMBER, Resonance.ZEPHYR),
    2: (Resonance.EMBER, Resonance.STONE),
    3: (Resonance.STONE, Resonance.EMBER),
    4: (Resonance.STONE, Resonance.TIDE),
    5: (Resonance.TIDE, Resonance.STONE),
    6: (Resonance.TIDE, Resonance.ZEPHYR),
    7: (Resonance.ZEPHYR, Resonance.TIDE),
}

# Co-primary sibling pairs (share primary resonance)
# Flash(0)/Ramp(7): Zephyr primary
# Blink(1)/Storm(2): Ember primary
# SelfDiscard(3)/SelfMill(4): Stone primary
# Sacrifice(5)/Warriors(6): Tide primary
SIBLING_MAP = {0: 7, 7: 0, 1: 2, 2: 1, 3: 4, 4: 3, 5: 6, 6: 5}

# Per-pair fitness rates: (arch_a, arch_b) -> rate
# Ordered by pair index: Warriors/Sacrifice=50%, SelfDiscard/SelfMill=40%,
# Blink/Storm=30%, Flash/Ramp=25%
GRADUATED_REALISTIC = {
    (5, 6): 0.50, (6, 5): 0.50,  # Warriors/Sacrifice
    (3, 4): 0.40, (4, 3): 0.40,  # SelfDiscard/SelfMill
    (1, 2): 0.30, (2, 1): 0.30,  # Blink/Storm
    (0, 7): 0.25, (7, 0): 0.25,  # Flash/Ramp
}

PESSIMISTIC = {
    (5, 6): 0.35, (6, 5): 0.35,
    (3, 4): 0.25, (4, 3): 0.25,
    (1, 2): 0.15, (2, 1): 0.15,
    (0, 7): 0.10, (7, 0): 0.10,
}

FITNESS_MODELS = {
    "Optimistic":           lambda a, b: 1.00,
    "Graduated Realistic":  lambda a, b: GRADUATED_REALISTIC.get((a, b), 0.0),
    "Pessimistic":          lambda a, b: PESSIMISTIC.get((a, b), 0.0),
    "Hostile":              lambda a, b: 0.08,
}

# ── Card Model ──

@dataclass
class SimCard:
    card_id: int
    symbols: list  # list of Resonance, ordered [primary, secondary, ...]
    archetype: int  # primary archetype index (0-7), or -1 for generic
    power: float
    is_dual_res: bool = False  # carries 2 different resonance types

    def primary_resonance(self):
        if not self.symbols:
            return None
        return self.symbols[0]

    def resonance_pair(self):
        """Returns (R1, R2) if card has 2+ different resonances."""
        if len(self.symbols) < 2:
            return None
        r1 = self.symbols[0]
        for s in self.symbols[1:]:
            if s != r1:
                return (r1, s)
        return None

# ── Pool Construction ──

def build_pool(dual_res_pct=0.15, rng=None):
    """Build a 360-card pool with configurable dual-resonance percentage.

    Each archetype gets 40 cards. Generic gets 40 cards.
    dual_res_pct of non-generic cards carry both primary and secondary resonance symbols.
    """
    if rng is None:
        rng = random.Random(SEED)

    cards = []
    card_id = 0

    # 40 generic cards (no symbols)
    for _ in range(GENERIC_COUNT):
        cards.append(SimCard(
            card_id=card_id,
            symbols=[],
            archetype=-1,
            power=rng.uniform(3.0, 7.0),
            is_dual_res=False,
        ))
        card_id += 1

    non_generic = POOL_SIZE - GENERIC_COUNT  # 320
    dual_res_total = int(non_generic * dual_res_pct)
    dual_res_per_arch = dual_res_total // ARCHETYPE_COUNT
    single_res_per_arch = CARDS_PER_ARCHETYPE - dual_res_per_arch

    for arch_idx in range(ARCHETYPE_COUNT):
        r1, r2 = ARCHETYPE_RESONANCES[arch_idx]

        # Single-resonance cards for this archetype
        for _ in range(single_res_per_arch):
            # 70% have 1 symbol (primary), 30% have 2 of same res
            if rng.random() < 0.7:
                syms = [r1]
            else:
                syms = [r1, r1]
            cards.append(SimCard(
                card_id=card_id,
                symbols=syms,
                archetype=arch_idx,
                power=rng.uniform(3.0, 8.0),
                is_dual_res=False,
            ))
            card_id += 1

        # Dual-resonance cards for this archetype
        for _ in range(dual_res_per_arch):
            # Has both primary and secondary resonance
            if rng.random() < 0.5:
                syms = [r1, r2]
            else:
                syms = [r1, r2, r1]  # 3-symbol variant
            cards.append(SimCard(
                card_id=card_id,
                symbols=syms,
                archetype=arch_idx,
                power=rng.uniform(3.0, 8.0),
                is_dual_res=True,
            ))
            card_id += 1

    return cards

# ── Fitness Evaluation ──

def card_tier(card, player_arch, fitness_fn):
    """Determine S/A/B/C/F tier of a card for a given player archetype.

    Returns numeric value: S=1.0, A=0.8, B=0.4, C=0.2, F=0.0
    For M3 counting: S and A count as S/A (value >= 0.8)
    """
    if card.archetype == -1:
        # Generic: B-tier for everyone (some utility)
        return 0.4

    if card.archetype == player_arch:
        # Home archetype: always S-tier
        return 1.0

    # Sibling archetype?
    sibling = SIBLING_MAP.get(player_arch)
    if card.archetype == sibling:
        # Sibling: A-tier with probability = fitness rate
        rate = fitness_fn(player_arch, card.archetype)
        return 0.8 if random.random() < rate else 0.2

    # Non-sibling, non-home: check if shares any resonance
    player_r1, player_r2 = ARCHETYPE_RESONANCES[player_arch]
    card_r1, _ = ARCHETYPE_RESONANCES[card.archetype]

    # If card is from an archetype sharing secondary resonance, slight chance of B
    if card_r1 == player_r2:
        return 0.2  # C-tier: partially relevant

    return 0.0  # F-tier: irrelevant

def is_sa_tier(card, player_arch, fitness_fn):
    """Returns True if card is S or A tier for the player."""
    return card_tier(card, player_arch, fitness_fn) >= 0.8

# ── Precompute fitness tables for consistency ──

def precompute_fitness(pool, fitness_fn, rng):
    """Precompute per-card, per-archetype S/A status for deterministic evaluation."""
    # fitness_table[card_id][arch_idx] = True/False (is S/A)
    table = {}
    for card in pool:
        table[card.card_id] = {}
        for arch in range(ARCHETYPE_COUNT):
            if card.archetype == -1:
                table[card.card_id][arch] = False  # Generic = B tier, not S/A
            elif card.archetype == arch:
                table[card.card_id][arch] = True  # Home = S tier
            else:
                sibling = SIBLING_MAP.get(arch)
                if card.archetype == sibling:
                    rate = fitness_fn(arch, card.archetype)
                    table[card.card_id][arch] = rng.random() < rate
                else:
                    table[card.card_id][arch] = False
    return table

# ── Pool Indexing ──

class PoolIndex:
    """Pre-built indexes for efficient drawing."""

    def __init__(self, pool):
        self.pool = pool
        self.all_cards = list(pool)

        # Index by primary resonance
        self.by_primary_res = defaultdict(list)
        for card in pool:
            if card.symbols:
                self.by_primary_res[card.symbols[0]].append(card)

        # Index by ordered resonance pair
        self.by_pair = defaultdict(list)
        for card in pool:
            pair = card.resonance_pair()
            if pair:
                self.by_pair[pair].append(card)

        # Index by archetype
        self.by_archetype = defaultdict(list)
        for card in pool:
            self.by_archetype[card.archetype].append(card)

        # Index: cards NOT matching a given primary resonance (for biased draw)
        self.non_primary_res = defaultdict(list)
        r1_id_sets = {}
        for res in Resonance:
            r1_id_sets[res] = set(c.card_id for c in self.by_primary_res.get(res, []))
        for res in Resonance:
            self.non_primary_res[res] = [c for c in pool if c.card_id not in r1_id_sets[res]]

# ── Draft Algorithms ──

def draw_random(pool_index, rng):
    """Draw a random card from the full pool."""
    return rng.choice(pool_index.all_cards)

def draw_r1_filtered(pool_index, resonance, rng):
    """Draw from the R1-filtered pool (cards with matching primary resonance)."""
    candidates = pool_index.by_primary_res.get(resonance, [])
    if not candidates:
        return draw_random(pool_index, rng)
    return rng.choice(candidates)

def draw_pair_filtered(pool_index, r1, r2, rng):
    """Draw from cards matching the ordered resonance pair."""
    candidates = pool_index.by_pair.get((r1, r2), [])
    if not candidates:
        # Fallback to R1 filtering
        return draw_r1_filtered(pool_index, r1, rng)
    return rng.choice(candidates)

def draw_biased_random(pool_index, resonance, bias_weight, rng):
    """Draw with bias toward a specific resonance. bias_weight=2 means 2x likely."""
    r1_pool = pool_index.by_primary_res.get(resonance, [])
    non_r1_pool = pool_index.non_primary_res.get(resonance, [])

    r1_count = len(r1_pool)
    other_count = len(non_r1_pool)

    if r1_count == 0:
        return draw_random(pool_index, rng)

    # Weighted selection: bias_weight for r1 cards, 1 for others
    if rng.random() < (bias_weight * r1_count) / (bias_weight * r1_count + other_count):
        return rng.choice(r1_pool)
    else:
        return rng.choice(non_r1_pool) if non_r1_pool else draw_random(pool_index, rng)

# ── Algorithm: Surge+Floor (V7 champion) ──

def algo_surge_floor(pool_index, player_arch, fitness_table, rng,
                     threshold=3, surge_slots=3, floor_start=3):
    """Surge+Floor T=3: accumulate resonance tokens, surge at T, floor otherwise."""
    r1, r2 = ARCHETYPE_RESONANCES[player_arch]
    tokens = 0
    picks = []
    packs = []
    committed = False
    commit_pick = None

    for pick_num in range(NUM_PICKS):
        pack = []
        is_surge = False

        if pick_num >= floor_start and tokens >= threshold:
            # Surge pack
            is_surge = True
            tokens -= threshold
            for slot in range(surge_slots):
                pack.append(draw_r1_filtered(pool_index, r1, rng))
            for slot in range(PACK_SIZE - surge_slots):
                pack.append(draw_random(pool_index, rng))
        elif pick_num >= floor_start:
            # Floor pack: 1 R1-filtered + 3 random
            pack.append(draw_r1_filtered(pool_index, r1, rng))
            for _ in range(PACK_SIZE - 1):
                pack.append(draw_random(pool_index, rng))
        else:
            # Early picks: all random
            for _ in range(PACK_SIZE):
                pack.append(draw_random(pool_index, rng))

        # Player picks best card for their archetype
        best_card = max(pack, key=lambda c: (
            fitness_table[c.card_id][player_arch],
            c.power
        ))
        picks.append(best_card)

        # Count S/A cards in this pack for the player
        sa_count = sum(1 for c in pack if fitness_table[c.card_id][player_arch])
        packs.append({
            'pick_num': pick_num,
            'sa_count': sa_count,
            'pack': pack,
            'is_surge': is_surge,
        })

        # Accumulate tokens from picked card
        for sym in best_card.symbols:
            if sym == r1:
                tokens += 2
            else:
                tokens += 1

        # Track commitment
        if not committed and pick_num >= 4 and sa_count >= 2:
            committed = True
            commit_pick = pick_num

    if commit_pick is None:
        commit_pick = 5  # Default

    return picks, packs, commit_pick

# ── Algorithm: Pair-Escalation Slots (V5) ──

def algo_pair_escalation(pool_index, player_arch, fitness_table, rng,
                          cap=0.50, escalation_k=6):
    """V5 Pair-Escalation: probabilistic pair-matched slots, escalating probability."""
    r1, r2 = ARCHETYPE_RESONANCES[player_arch]
    pair_count = 0  # Number of pair-matching cards drafted
    picks = []
    packs = []
    committed = False
    commit_pick = None

    for pick_num in range(NUM_PICKS):
        # Calculate pair probability: ramps toward cap as pair_count increases
        if pair_count <= 0:
            pair_prob = 0.0
        else:
            pair_prob = min(cap, pair_count / escalation_k * cap)

        pack = []
        for slot in range(PACK_SIZE):
            if rng.random() < pair_prob:
                pack.append(draw_pair_filtered(pool_index, r1, r2, rng))
            else:
                pack.append(draw_random(pool_index, rng))

        # Player picks best card
        best_card = max(pack, key=lambda c: (
            fitness_table[c.card_id][player_arch],
            c.power
        ))
        picks.append(best_card)

        # Update pair count
        card_pair = best_card.resonance_pair()
        if card_pair and card_pair == (r1, r2):
            pair_count += 1
        elif best_card.symbols and best_card.symbols[0] == r1:
            pair_count += 0.5  # Partial credit for R1 match

        sa_count = sum(1 for c in pack if fitness_table[c.card_id][player_arch])
        packs.append({
            'pick_num': pick_num,
            'sa_count': sa_count,
            'pack': pack,
        })

        if not committed and pick_num >= 4 and sa_count >= 2:
            committed = True
            commit_pick = pick_num

    if commit_pick is None:
        commit_pick = 6

    return picks, packs, commit_pick

# ── Algorithm: Lane Locking (V3) ──

def algo_lane_locking(pool_index, player_arch, fitness_table, rng,
                       lock_threshold_1=5, lock_threshold_2=12,
                       use_pair=False, soft_prob=0.80):
    """Lane Locking: slots lock to player's resonance at thresholds.

    use_pair=True: locks use pair-filtering instead of R1
    soft_prob<1.0: locked slots show filtered cards with this probability
    """
    r1, r2 = ARCHETYPE_RESONANCES[player_arch]
    tokens = 0
    locked_slots = 0
    picks = []
    packs = []
    committed = False
    commit_pick = None

    for pick_num in range(NUM_PICKS):
        # Check lock thresholds
        if tokens >= lock_threshold_2:
            locked_slots = 2
        elif tokens >= lock_threshold_1:
            locked_slots = 1

        pack = []
        for slot in range(PACK_SIZE):
            if slot < locked_slots:
                # Locked slot
                if rng.random() < soft_prob:
                    if use_pair:
                        pack.append(draw_pair_filtered(pool_index, r1, r2, rng))
                    else:
                        pack.append(draw_r1_filtered(pool_index, r1, rng))
                else:
                    pack.append(draw_random(pool_index, rng))
            else:
                pack.append(draw_random(pool_index, rng))

        # Player picks best card
        best_card = max(pack, key=lambda c: (
            fitness_table[c.card_id][player_arch],
            c.power
        ))
        picks.append(best_card)

        # Accumulate tokens
        for sym in best_card.symbols:
            if sym == r1:
                tokens += 2
            else:
                tokens += 1

        sa_count = sum(1 for c in pack if fitness_table[c.card_id][player_arch])
        packs.append({
            'pick_num': pick_num,
            'sa_count': sa_count,
            'pack': pack,
        })

        if not committed and pick_num >= 4 and sa_count >= 2:
            committed = True
            commit_pick = pick_num

    if commit_pick is None:
        commit_pick = 5

    return picks, packs, commit_pick

# ── Algorithm: Surge+Floor+Bias ──

def algo_surge_floor_bias(pool_index, player_arch, fitness_table, rng,
                           threshold=3, surge_slots=3, floor_start=3,
                           bias_weight=2.0, use_pair_surge=False):
    """Surge+Floor with bias on random slots. Optionally pair-filtered surges."""
    r1, r2 = ARCHETYPE_RESONANCES[player_arch]
    tokens = 0
    picks = []
    packs = []
    committed = False
    commit_pick = None

    for pick_num in range(NUM_PICKS):
        pack = []
        is_surge = False

        if pick_num >= floor_start and tokens >= threshold:
            is_surge = True
            tokens -= threshold
            for slot in range(surge_slots):
                if use_pair_surge:
                    pack.append(draw_pair_filtered(pool_index, r1, r2, rng))
                else:
                    pack.append(draw_r1_filtered(pool_index, r1, rng))
            for slot in range(PACK_SIZE - surge_slots):
                pack.append(draw_biased_random(pool_index, r1, bias_weight, rng))
        elif pick_num >= floor_start:
            # Floor pack: 1 filtered + 3 biased random
            pack.append(draw_r1_filtered(pool_index, r1, rng))
            for _ in range(PACK_SIZE - 1):
                pack.append(draw_biased_random(pool_index, r1, bias_weight, rng))
        else:
            for _ in range(PACK_SIZE):
                pack.append(draw_random(pool_index, rng))

        best_card = max(pack, key=lambda c: (
            fitness_table[c.card_id][player_arch],
            c.power
        ))
        picks.append(best_card)

        for sym in best_card.symbols:
            if sym == r1:
                tokens += 2
            else:
                tokens += 1

        sa_count = sum(1 for c in pack if fitness_table[c.card_id][player_arch])
        packs.append({
            'pick_num': pick_num,
            'sa_count': sa_count,
            'pack': pack,
        })

        if not committed and pick_num >= 4 and sa_count >= 2:
            committed = True
            commit_pick = pick_num

    if commit_pick is None:
        commit_pick = 5

    return picks, packs, commit_pick

# ── Player Strategies ──

def strategy_committed(pool_index, algo_fn, player_arch, fitness_table, rng, **kwargs):
    """Archetype-committed player: always picks best for their archetype."""
    return algo_fn(pool_index, player_arch, fitness_table, rng, **kwargs)

def strategy_power_chaser(pool_index, algo_fn, fitness_table, rng, **kwargs):
    """Power-chaser: picks highest raw power regardless of archetype.
    We run the algorithm but override card selection to pick by power."""
    # Pick a random archetype for the algorithm (simulates an uncommitted player)
    player_arch = rng.randint(0, 7)
    r1, r2 = ARCHETYPE_RESONANCES[player_arch]

    # Use the algorithm but override picking to use power
    orig_picks, packs, commit_pick = algo_fn(
        pool_index, player_arch, fitness_table, rng, **kwargs
    )

    # Re-evaluate: pick highest power from each pack
    new_picks = []
    for p in packs:
        best_card = max(p['pack'], key=lambda c: c.power)
        new_picks.append(best_card)

    return new_picks, packs, commit_pick

def strategy_signal_reader(pool_index, algo_fn, fitness_table, rng, **kwargs):
    """Signal-reader: evaluates which archetype is most available and drafts toward it."""
    # Start with a random archetype, may switch in first 5 picks
    archetype_scores = [0.0] * ARCHETYPE_COUNT
    current_arch = rng.randint(0, 7)

    # We need to run the full draft; for simplicity, run committed to current_arch
    # but count signals
    picks, packs, commit_pick = algo_fn(
        pool_index, current_arch, fitness_table, rng, **kwargs
    )

    # Post-hoc: evaluate signal reading by checking if early packs suggested a
    # better archetype. This affects convergence timing.
    for p in packs[:5]:
        for card in p['pack']:
            for arch in range(ARCHETYPE_COUNT):
                if fitness_table[card.card_id][arch]:
                    archetype_scores[arch] += 1

    best_signal_arch = max(range(ARCHETYPE_COUNT), key=lambda a: archetype_scores[a])

    # If signal arch differs from initial, re-run with signal arch
    if best_signal_arch != current_arch:
        picks, packs, commit_pick = algo_fn(
            pool_index, best_signal_arch, fitness_table, rng, **kwargs
        )
        commit_pick = max(commit_pick, 6)  # Signal readers commit later

    return picks, packs, commit_pick

# ── Metrics Calculation ──

def compute_metrics(all_drafts_packs, all_commit_picks, all_picks, fitness_tables,
                    archetypes_used):
    """Compute M1-M10 from draft results."""

    # M1: Picks 1-5: unique archetypes with S/A cards per pack
    m1_values = []
    # M2: Picks 1-5: S/A cards for emerging archetype per pack
    m2_values = []
    # M3: Picks 6+: S/A cards for committed archetype per pack
    m3_values = []
    m3_per_arch = defaultdict(list)
    # M4: Picks 6+: off-archetype cards per pack
    m4_values = []
    # M5: Convergence pick
    m5_values = []
    # M6: Deck concentration
    m6_values = []
    # M9: StdDev of S/A per pack (picks 6+)
    m9_per_draft = []
    # M10: Max consecutive packs below 1.5 S/A
    m10_values = []

    # For pack quality distribution
    pack_sa_dist = []  # all per-pack S/A values for picks 6+

    # Consecutive bad pack tracking
    consec_bad_streaks = []

    for draft_idx, (packs, commit_pick, picks, ft, arch) in enumerate(
        zip(all_drafts_packs, all_commit_picks, all_picks, fitness_tables, archetypes_used)
    ):
        # M1: early packs (picks 0-4)
        for p in packs[:5]:
            archs_with_sa = set()
            sa_for_player = 0
            for card in p['pack']:
                for a in range(ARCHETYPE_COUNT):
                    if ft[card.card_id][a]:
                        archs_with_sa.add(a)
                if ft[card.card_id][arch]:
                    sa_for_player += 1
            m1_values.append(len(archs_with_sa))
            m2_values.append(sa_for_player)

        # M3, M4, M9, M10: post-commitment packs (picks 5+)
        post_commit_sa = []
        consecutive_bad = 0
        max_consecutive_bad = 0
        all_bad_streaks = []

        for p in packs[5:]:
            sa_count = p['sa_count']
            post_commit_sa.append(sa_count)
            m3_values.append(sa_count)
            m3_per_arch[arch].append(sa_count)
            pack_sa_dist.append(sa_count)

            # Off-archetype count
            off_arch = sum(1 for c in p['pack'] if not ft[c.card_id][arch])
            m4_values.append(off_arch)

            # Consecutive bad tracking
            if sa_count < 1.5:
                consecutive_bad += 1
            else:
                if consecutive_bad > 0:
                    all_bad_streaks.append(consecutive_bad)
                consecutive_bad = 0
            max_consecutive_bad = max(max_consecutive_bad, consecutive_bad)

        if consecutive_bad > 0:
            all_bad_streaks.append(consecutive_bad)

        m10_values.append(max_consecutive_bad)
        consec_bad_streaks.extend(all_bad_streaks)

        # M5: convergence pick
        m5_values.append(commit_pick)

        # M6: deck concentration
        sa_in_deck = sum(1 for c in picks if ft[c.card_id][arch])
        m6_values.append(sa_in_deck / len(picks) if picks else 0)

        # M9: stddev of per-pack S/A
        if len(post_commit_sa) >= 2:
            m9_per_draft.append(
                (sum((x - sum(post_commit_sa)/len(post_commit_sa))**2
                     for x in post_commit_sa) / len(post_commit_sa)) ** 0.5
            )

    # M7: Run-to-run variety (card overlap between consecutive drafts)
    m7_values = []
    for i in range(1, len(all_picks)):
        if archetypes_used[i] == archetypes_used[i-1]:
            ids_a = set(c.card_id for c in all_picks[i-1])
            ids_b = set(c.card_id for c in all_picks[i])
            overlap = len(ids_a & ids_b) / max(len(ids_a | ids_b), 1)
            m7_values.append(overlap)

    # M8: Archetype frequency
    arch_freq = defaultdict(int)
    for a in archetypes_used:
        arch_freq[a] += 1
    total_drafts = len(archetypes_used)
    m8_pcts = {a: arch_freq[a] / total_drafts * 100 for a in range(ARCHETYPE_COUNT)}

    # Pack quality distribution percentiles
    sorted_sa = sorted(pack_sa_dist)
    n = len(sorted_sa)

    def pctile(pct):
        if n == 0:
            return 0
        idx = int(n * pct / 100)
        return sorted_sa[min(idx, n-1)]

    # Per-archetype M3
    per_arch_m3 = {}
    for a in range(ARCHETYPE_COUNT):
        vals = m3_per_arch[a]
        per_arch_m3[a] = sum(vals) / len(vals) if vals else 0

    # Average consecutive bad streaks
    avg_consec_bad = sum(consec_bad_streaks) / len(consec_bad_streaks) if consec_bad_streaks else 0
    worst_consec_bad = max(m10_values) if m10_values else 0

    return {
        'M1': sum(m1_values) / len(m1_values) if m1_values else 0,
        'M2': sum(m2_values) / len(m2_values) if m2_values else 0,
        'M3': sum(m3_values) / len(m3_values) if m3_values else 0,
        'M4': sum(m4_values) / len(m4_values) if m4_values else 0,
        'M5': sum(m5_values) / len(m5_values) if m5_values else 0,
        'M6': sum(m6_values) / len(m6_values) if m6_values else 0,
        'M7': sum(m7_values) / len(m7_values) if m7_values else 0,
        'M8': m8_pcts,
        'M9': sum(m9_per_draft) / len(m9_per_draft) if m9_per_draft else 0,
        'M10': worst_consec_bad,
        'M10_avg': sum(m10_values) / len(m10_values) if m10_values else 0,
        'per_arch_m3': per_arch_m3,
        'pack_pctiles': {
            'p10': pctile(10),
            'p25': pctile(25),
            'p50': pctile(50),
            'p75': pctile(75),
            'p90': pctile(90),
        },
        'avg_consec_bad': avg_consec_bad,
        'worst_consec_bad': worst_consec_bad,
    }

# ── Run All Simulations ──

def run_simulation(algo_name, algo_fn, algo_kwargs, pool_name, pool, fitness_name, fitness_fn,
                   num_drafts=NUM_DRAFTS):
    """Run a full simulation suite: 1000 drafts x 30 picks x archetype-committed."""
    master_rng = random.Random(SEED)
    pool_index = PoolIndex(pool)

    all_packs = []
    all_commit_picks = []
    all_picks = []
    all_fitness_tables = []
    all_archetypes = []

    for draft_i in range(num_drafts):
        draft_rng = random.Random(master_rng.randint(0, 2**32))
        player_arch = draft_i % ARCHETYPE_COUNT  # Rotate through archetypes

        # Precompute fitness for this draft
        ft = precompute_fitness(pool, fitness_fn, random.Random(draft_rng.randint(0, 2**32)))

        picks, packs, commit_pick = algo_fn(
            pool_index, player_arch, ft, draft_rng, **algo_kwargs
        )

        all_packs.append(packs)
        all_commit_picks.append(commit_pick)
        all_picks.append(picks)
        all_fitness_tables.append(ft)
        all_archetypes.append(player_arch)

    metrics = compute_metrics(all_packs, all_commit_picks, all_picks,
                              all_fitness_tables, all_archetypes)

    return metrics

def generate_draft_trace(algo_name, algo_fn, algo_kwargs, pool, fitness_fn,
                          player_arch, trace_name, rng_seed=12345):
    """Generate a detailed draft trace for one draft."""
    pool_index = PoolIndex(pool)
    draft_rng = random.Random(rng_seed)
    ft = precompute_fitness(pool, fitness_fn, random.Random(rng_seed + 1))

    picks, packs, commit_pick = algo_fn(
        pool_index, player_arch, ft, draft_rng, **algo_kwargs
    )

    r1, r2 = ARCHETYPE_RESONANCES[player_arch]

    lines = [f"### Trace: {trace_name} ({algo_name}, {ARCHETYPES[player_arch]})"]
    lines.append(f"Commit pick: {commit_pick}")
    lines.append("")

    for i, (p, pick) in enumerate(zip(packs, picks)):
        sa = p['sa_count']
        pick_sa = "S/A" if ft[pick.card_id][player_arch] else "B/C/F"
        lines.append(f"Pick {i+1}: pack S/A={sa}, picked {pick_sa} (power={pick.power:.1f})")

    total_sa = sum(1 for c in picks if ft[c.card_id][player_arch])
    lines.append(f"\nFinal deck: {total_sa}/{len(picks)} S/A = {total_sa/len(picks)*100:.1f}% concentration")

    return "\n".join(lines)

# ── Main ──

def main():
    print("=" * 70)
    print("V8 RESONANCE DRAFT SIMULATION — AGENT 1 BASELINES")
    print("=" * 70)

    # Build pools
    v7_pool = build_pool(dual_res_pct=0.15, rng=random.Random(SEED))
    enriched_pool = build_pool(dual_res_pct=0.40, rng=random.Random(SEED + 1))

    # Verify pool composition
    v7_dual = sum(1 for c in v7_pool if c.is_dual_res)
    enr_dual = sum(1 for c in enriched_pool if c.is_dual_res)
    print(f"\nV7 pool: {len(v7_pool)} cards, {v7_dual} dual-res ({v7_dual/len(v7_pool)*100:.1f}%)")
    print(f"Enriched pool: {len(enriched_pool)} cards, {enr_dual} dual-res ({enr_dual/len(enriched_pool)*100:.1f}%)")

    # Verify pair pools
    v7_idx = PoolIndex(v7_pool)
    enr_idx = PoolIndex(enriched_pool)
    print(f"\nV7 pair pool sizes:")
    for r1 in Resonance:
        for r2 in Resonance:
            if r1 != r2:
                n = len(v7_idx.by_pair.get((r1, r2), []))
                if n > 0:
                    print(f"  ({r1.name}, {r2.name}): {n} cards")
    print(f"\nEnriched pair pool sizes:")
    for r1 in Resonance:
        for r2 in Resonance:
            if r1 != r2:
                n = len(enr_idx.by_pair.get((r1, r2), []))
                if n > 0:
                    print(f"  ({r1.name}, {r2.name}): {n} cards")

    # Define all algorithm configurations
    ALGORITHMS = {
        "Surge+Floor (T=3)": (algo_surge_floor, {"threshold": 3, "surge_slots": 3, "floor_start": 3}),
        "Pair-Escalation (cap=0.50)": (algo_pair_escalation, {"cap": 0.50, "escalation_k": 6}),
        "Lane Lock (hard, R1)": (algo_lane_locking, {"lock_threshold_1": 5, "lock_threshold_2": 12, "use_pair": False, "soft_prob": 1.0}),
        "Lane Lock (soft pair)": (algo_lane_locking, {"lock_threshold_1": 5, "lock_threshold_2": 12, "use_pair": True, "soft_prob": 0.80}),
        "Surge+Floor+Bias (R1)": (algo_surge_floor_bias, {"threshold": 3, "surge_slots": 3, "floor_start": 3, "bias_weight": 2.0, "use_pair_surge": False}),
        "Surge+Floor+Bias (pair)": (algo_surge_floor_bias, {"threshold": 3, "surge_slots": 3, "floor_start": 3, "bias_weight": 2.0, "use_pair_surge": True}),
    }

    POOLS = {
        "V7 Standard (15%)": v7_pool,
        "40% Enriched": enriched_pool,
    }

    results = {}

    for fitness_name, fitness_fn in FITNESS_MODELS.items():
        print(f"\n{'='*70}")
        print(f"FITNESS MODEL: {fitness_name}")
        print(f"{'='*70}")

        for pool_name, pool in POOLS.items():
            print(f"\n  Pool: {pool_name}")
            print(f"  {'-'*50}")

            for algo_name, (algo_fn, algo_kwargs) in ALGORITHMS.items():
                # Skip pair-dependent algorithms on V7 pool (too few pair cards)
                if "pair" in algo_name.lower() and "V7" in pool_name:
                    pair_idx = PoolIndex(pool)
                    max_pair = max(len(v) for v in pair_idx.by_pair.values()) if pair_idx.by_pair else 0
                    if max_pair < 8:
                        print(f"    {algo_name}: SKIPPED (pair pool too small, max={max_pair})")
                        key = (algo_name, pool_name, fitness_name)
                        results[key] = None
                        continue

                metrics = run_simulation(
                    algo_name, algo_fn, algo_kwargs,
                    pool_name, pool, fitness_name, fitness_fn
                )

                key = (algo_name, pool_name, fitness_name)
                results[key] = metrics

                print(f"    {algo_name}:")
                print(f"      M1={metrics['M1']:.2f} M2={metrics['M2']:.2f} M3={metrics['M3']:.2f} "
                      f"M4={metrics['M4']:.2f} M5={metrics['M5']:.1f}")
                print(f"      M6={metrics['M6']:.2f} M7={metrics['M7']:.3f} M9={metrics['M9']:.2f} "
                      f"M10_worst={metrics['M10']} M10_avg={metrics['M10_avg']:.1f}")

                # Per-archetype M3
                worst_arch = min(metrics['per_arch_m3'].values())
                best_arch = max(metrics['per_arch_m3'].values())
                worst_idx = min(metrics['per_arch_m3'], key=metrics['per_arch_m3'].get)
                print(f"      Worst-arch M3: {worst_arch:.2f} ({ARCHETYPES[worst_idx]}), "
                      f"Best: {best_arch:.2f}")

                # Pack quality distribution
                pp = metrics['pack_pctiles']
                print(f"      Pack S/A pctiles: p10={pp['p10']:.1f} p25={pp['p25']:.1f} "
                      f"p50={pp['p50']:.1f} p75={pp['p75']:.1f} p90={pp['p90']:.1f}")
                print(f"      Consec bad: avg={metrics['avg_consec_bad']:.2f} worst={metrics['worst_consec_bad']}")

    # ── Parameter Sensitivity Sweeps ──
    print(f"\n{'='*70}")
    print("PARAMETER SENSITIVITY SWEEPS")
    print(f"{'='*70}")

    # Sweep 1: Surge+Floor threshold T on enriched pool, Graduated Realistic
    print("\n  Sweep 1: Surge+Floor threshold T (Enriched, Grad. Realistic)")
    fitness_fn = FITNESS_MODELS["Graduated Realistic"]
    for t in [2, 3, 4, 5]:
        m = run_simulation(
            f"SF T={t}", algo_surge_floor, {"threshold": t, "surge_slots": 3, "floor_start": 3},
            "Enriched", enriched_pool, "GR", fitness_fn, num_drafts=500
        )
        print(f"    T={t}: M3={m['M3']:.2f}, M9={m['M9']:.2f}, M10_worst={m['M10']}, "
              f"worst-arch={min(m['per_arch_m3'].values()):.2f}")

    # Sweep 2: Pair-Escalation cap on enriched pool, Graduated Realistic
    print("\n  Sweep 2: Pair-Escalation cap (Enriched, Grad. Realistic)")
    for cap in [0.30, 0.40, 0.50, 0.60, 0.70]:
        m = run_simulation(
            f"PE cap={cap}", algo_pair_escalation, {"cap": cap, "escalation_k": 6},
            "Enriched", enriched_pool, "GR", fitness_fn, num_drafts=500
        )
        print(f"    cap={cap:.2f}: M3={m['M3']:.2f}, M6={m['M6']:.2f}, M4={m['M4']:.2f}, "
              f"M9={m['M9']:.2f}, worst-arch={min(m['per_arch_m3'].values()):.2f}")

    # Sweep 3: Lane Lock soft_prob on enriched pool, Graduated Realistic
    print("\n  Sweep 3: Soft Pair Lock probability (Enriched, Grad. Realistic)")
    for sp in [0.60, 0.70, 0.80, 0.90, 1.00]:
        m = run_simulation(
            f"SPL sp={sp}", algo_lane_locking,
            {"lock_threshold_1": 5, "lock_threshold_2": 12, "use_pair": True, "soft_prob": sp},
            "Enriched", enriched_pool, "GR", fitness_fn, num_drafts=500
        )
        print(f"    soft_prob={sp:.2f}: M3={m['M3']:.2f}, M9={m['M9']:.2f}, "
              f"M6={m['M6']:.2f}, M10_worst={m['M10']}")

    # ── Draft Traces ──
    print(f"\n{'='*70}")
    print("DRAFT TRACES")
    print(f"{'='*70}")

    fitness_fn = FITNESS_MODELS["Graduated Realistic"]

    # Trace 1: Early committer (Warriors, high-overlap pair)
    trace1 = generate_draft_trace(
        "Pair-Escalation", algo_pair_escalation, {"cap": 0.50, "escalation_k": 6},
        enriched_pool, fitness_fn, player_arch=6,
        trace_name="Early Committer (Warriors)", rng_seed=42
    )
    print(f"\n{trace1}")

    # Trace 2: Flexible player (Blink, medium-overlap pair)
    trace2 = generate_draft_trace(
        "Surge+Floor+Bias (pair)", algo_surge_floor_bias,
        {"threshold": 3, "surge_slots": 3, "floor_start": 3, "bias_weight": 2.0, "use_pair_surge": True},
        enriched_pool, fitness_fn, player_arch=1,
        trace_name="Flexible Player (Blink)", rng_seed=99
    )
    print(f"\n{trace2}")

    # Trace 3: Signal reader (Flash, worst-overlap pair)
    trace3 = generate_draft_trace(
        "Lane Lock (soft pair)", algo_lane_locking,
        {"lock_threshold_1": 5, "lock_threshold_2": 12, "use_pair": True, "soft_prob": 0.80},
        enriched_pool, fitness_fn, player_arch=0,
        trace_name="Signal Reader (Flash, worst pair)", rng_seed=777
    )
    print(f"\n{trace3}")

    # ── Summary table ──
    print(f"\n{'='*70}")
    print("SUMMARY: Graduated Realistic Fitness, All Algorithms")
    print(f"{'='*70}")
    print(f"{'Algorithm':<30} {'Pool':<18} {'M3':>5} {'Worst':>6} {'M4':>5} {'M5':>5} "
          f"{'M6':>5} {'M9':>5} {'M10':>4} {'p10':>4} {'p50':>4}")
    print("-" * 110)

    for key, m in sorted(results.items()):
        algo, pool, fitness = key
        if fitness != "Graduated Realistic" or m is None:
            continue
        wa = min(m['per_arch_m3'].values())
        pp = m['pack_pctiles']
        print(f"{algo:<30} {pool:<18} {m['M3']:5.2f} {wa:6.2f} {m['M4']:5.2f} "
              f"{m['M5']:5.1f} {m['M6']:5.2f} {m['M9']:5.2f} {m['M10']:4d} "
              f"{pp['p10']:4.1f} {pp['p50']:4.1f}")

    # ── Fitness Degradation Curve ──
    print(f"\n{'='*70}")
    print("FITNESS DEGRADATION CURVE (Enriched Pool)")
    print(f"{'='*70}")
    print(f"{'Algorithm':<30} {'Opt':>6} {'GradR':>6} {'Pess':>6} {'Host':>6} {'Degrad%':>8}")
    print("-" * 75)

    for algo_name in ALGORITHMS:
        opt = results.get((algo_name, "40% Enriched", "Optimistic"))
        gr = results.get((algo_name, "40% Enriched", "Graduated Realistic"))
        pess = results.get((algo_name, "40% Enriched", "Pessimistic"))
        host = results.get((algo_name, "40% Enriched", "Hostile"))

        if opt is None:
            continue

        opt_m3 = opt['M3'] if opt else 0
        gr_m3 = gr['M3'] if gr else 0
        pess_m3 = pess['M3'] if pess else 0
        host_m3 = host['M3'] if host else 0
        degrad = ((opt_m3 - host_m3) / opt_m3 * 100) if opt_m3 > 0 else 0

        print(f"{algo_name:<30} {opt_m3:6.2f} {gr_m3:6.2f} {pess_m3:6.2f} "
              f"{host_m3:6.2f} {degrad:7.1f}%")

    # ── Per-Archetype Convergence Table ──
    print(f"\n{'='*70}")
    print("PER-ARCHETYPE M3 (Graduated Realistic, Enriched Pool)")
    print(f"{'='*70}")

    header_algos = ["Surge+Floor (T=3)", "Pair-Escalation (cap=0.50)",
                    "Lane Lock (soft pair)", "Surge+Floor+Bias (pair)"]
    print(f"{'Archetype':<15}", end="")
    for a in header_algos:
        short = a[:22]
        print(f" {short:>22}", end="")
    print()
    print("-" * 105)

    for arch_idx in range(ARCHETYPE_COUNT):
        print(f"{ARCHETYPES[arch_idx]:<15}", end="")
        for algo_name in header_algos:
            key = (algo_name, "40% Enriched", "Graduated Realistic")
            m = results.get(key)
            if m and arch_idx in m['per_arch_m3']:
                print(f" {m['per_arch_m3'][arch_idx]:22.2f}", end="")
            else:
                print(f" {'N/A':>22}", end="")
        print()

    print("\nSimulation complete.")

if __name__ == "__main__":
    main()
