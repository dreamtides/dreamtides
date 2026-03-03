#!/usr/bin/env python3
"""
Agent 3: Archetype Breakdown Simulation for Pair-Escalation Slots.

Investigates: How should cards be distributed across archetypes, and how many
generic/bridge cards should exist?

Algorithm: Pair-Escalation Slots
"Track the ordered symbol pair (first, second) of each 2+ symbol card you draft;
each pack slot independently shows a card matching your most common pair with
probability equal to that pair's count divided by 6 (capped at 50%), otherwise
a random card."

Tests 5 breakdown models:
  1. Equal + Small Generic (~10%): 36 generic, ~40 per archetype
  2. Equal + Large Generic (~25%): 90 generic, ~34 per archetype
  3. Equal + Bridge Cards: 36 generic, 48 bridge, ~34 per archetype
  4. Asymmetric Sizes: varied sizes per archetype
  5. Pair-Pool-Optimized: ensure minimum pair pool size per pair

Symbol distribution: 15/60/25 (1-sym/2-sym/3-sym of non-generic cards).
K=6, C=0.50, 4 slots, 30 picks, 1200 drafts per model.
"""

import random
import statistics
from dataclasses import dataclass, field
from enum import Enum
from collections import defaultdict, Counter
from typing import Optional

# ============================================================================
# Core Types
# ============================================================================

class Resonance(Enum):
    EMBER = "Ember"
    STONE = "Stone"
    TIDE = "Tide"
    ZEPHYR = "Zephyr"

class Tier(Enum):
    S = "S"
    A = "A"
    B = "B"
    C = "C"
    F = "F"

ARCHETYPES = [
    ("Flash",        Resonance.ZEPHYR, Resonance.EMBER),
    ("Blink",        Resonance.EMBER,  Resonance.ZEPHYR),
    ("Storm",        Resonance.EMBER,  Resonance.STONE),
    ("Self-Discard", Resonance.STONE,  Resonance.EMBER),
    ("Self-Mill",    Resonance.STONE,  Resonance.TIDE),
    ("Sacrifice",    Resonance.TIDE,   Resonance.STONE),
    ("Warriors",     Resonance.TIDE,   Resonance.ZEPHYR),
    ("Ramp",         Resonance.ZEPHYR, Resonance.TIDE),
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
NUM_ARCHETYPES = 8

def circle_distance(i, j):
    d = abs(i - j)
    return min(d, NUM_ARCHETYPES - d)

def compute_fitness(card_arch_idx, player_arch_idx):
    if card_arch_idx == -1:
        return Tier.B
    if card_arch_idx == player_arch_idx:
        return Tier.S
    dist = circle_distance(card_arch_idx, player_arch_idx)
    card_primary = ARCHETYPES[card_arch_idx][1]
    player_primary = ARCHETYPES[player_arch_idx][1]
    if dist == 1:
        if card_primary == player_primary:
            return Tier.A
        return Tier.B
    elif dist == 2:
        card_res = {ARCHETYPES[card_arch_idx][1], ARCHETYPES[card_arch_idx][2]}
        player_res = {ARCHETYPES[player_arch_idx][1], ARCHETYPES[player_arch_idx][2]}
        if card_res & player_res:
            return Tier.B
        return Tier.C
    elif dist == 3:
        return Tier.C
    else:
        return Tier.F

def compute_bridge_fitness(card_arch1, card_arch2, player_arch_idx):
    if card_arch1 == player_arch_idx or card_arch2 == player_arch_idx:
        return Tier.S
    for home_idx in [card_arch1, card_arch2]:
        dist = circle_distance(home_idx, player_arch_idx)
        if dist == 1:
            if ARCHETYPES[home_idx][1] == ARCHETYPES[player_arch_idx][1]:
                return Tier.A
    card_res = set()
    for home_idx in [card_arch1, card_arch2]:
        card_res.add(ARCHETYPES[home_idx][1])
        card_res.add(ARCHETYPES[home_idx][2])
    player_res = {ARCHETYPES[player_arch_idx][1], ARCHETYPES[player_arch_idx][2]}
    if card_res & player_res:
        return Tier.B
    return Tier.C


@dataclass
class SimCard:
    id: int
    symbols: list
    archetype_idx: int
    bridge_arch_idx: int = -1
    fitness: dict = field(default_factory=dict)

    @property
    def ordered_pair(self):
        if len(self.symbols) >= 2:
            return (self.symbols[0], self.symbols[1])
        return None

    @property
    def is_generic(self):
        return len(self.symbols) == 0

    @property
    def is_bridge(self):
        return self.bridge_arch_idx >= 0

    def is_sa_for(self, arch_idx):
        return self.fitness.get(arch_idx, Tier.F) in (Tier.S, Tier.A)

    def is_s_for(self, arch_idx):
        return self.fitness.get(arch_idx, Tier.F) == Tier.S

    def is_cf_for(self, arch_idx):
        return self.fitness.get(arch_idx, Tier.F) in (Tier.C, Tier.F)


def assign_fitness(card):
    for j in range(NUM_ARCHETYPES):
        if card.is_generic:
            card.fitness[j] = Tier.B
        elif card.is_bridge:
            card.fitness[j] = compute_bridge_fitness(card.archetype_idx,
                                                      card.bridge_arch_idx, j)
        else:
            card.fitness[j] = compute_fitness(card.archetype_idx, j)


# ============================================================================
# Pool Generation
# ============================================================================

def make_symbols(arch_idx, num_symbols, rng):
    primary = ARCHETYPES[arch_idx][1]
    secondary = ARCHETYPES[arch_idx][2]
    if num_symbols == 1:
        return [primary]
    elif num_symbols == 2:
        return [primary, secondary]
    else:
        roll = rng.random()
        if roll < 0.33:
            return [primary, secondary, primary]
        elif roll < 0.67:
            return [primary, primary, secondary]
        else:
            return [primary, secondary, secondary]


def make_archetype_cards(arch_idx, count, card_id_start, rng,
                         one_pct=0.15, two_pct=0.60, three_pct=0.25):
    cards = []
    n1 = round(count * one_pct)
    n3 = round(count * three_pct)
    n2 = count - n1 - n3
    for _ in range(n1):
        syms = make_symbols(arch_idx, 1, rng)
        cards.append(SimCard(id=card_id_start, symbols=syms, archetype_idx=arch_idx))
        card_id_start += 1
    for _ in range(n2):
        syms = make_symbols(arch_idx, 2, rng)
        cards.append(SimCard(id=card_id_start, symbols=syms, archetype_idx=arch_idx))
        card_id_start += 1
    for _ in range(n3):
        syms = make_symbols(arch_idx, 3, rng)
        cards.append(SimCard(id=card_id_start, symbols=syms, archetype_idx=arch_idx))
        card_id_start += 1
    return cards, card_id_start


def generate_pool_equal_small_generic(total=360, rng=None):
    """Model 1: 36 generic, ~40 per archetype. 15/60/25 symbol split."""
    r = rng or random.Random()
    cards = []
    cid = 0
    for _ in range(36):
        cards.append(SimCard(id=cid, symbols=[], archetype_idx=-1))
        cid += 1
    per_arch = (total - 36) // NUM_ARCHETYPES
    for ai in range(NUM_ARCHETYPES):
        ac, cid = make_archetype_cards(ai, per_arch, cid, r)
        cards.extend(ac)
    while len(cards) < total:
        ai = r.randint(0, NUM_ARCHETYPES - 1)
        syms = make_symbols(ai, 2, r)
        cards.append(SimCard(id=cid, symbols=syms, archetype_idx=ai))
        cid += 1
    for c in cards:
        assign_fitness(c)
    return cards


def generate_pool_equal_large_generic(total=360, rng=None):
    """Model 2: 90 generic, ~33 per archetype. 15/60/25 symbol split."""
    r = rng or random.Random()
    cards = []
    cid = 0
    for _ in range(90):
        cards.append(SimCard(id=cid, symbols=[], archetype_idx=-1))
        cid += 1
    per_arch = (total - 90) // NUM_ARCHETYPES
    for ai in range(NUM_ARCHETYPES):
        ac, cid = make_archetype_cards(ai, per_arch, cid, r)
        cards.extend(ac)
    while len(cards) < total:
        ai = r.randint(0, NUM_ARCHETYPES - 1)
        syms = make_symbols(ai, 2, r)
        cards.append(SimCard(id=cid, symbols=syms, archetype_idx=ai))
        cid += 1
    for c in cards:
        assign_fitness(c)
    return cards


def generate_pool_bridge(total=360, rng=None):
    """Model 3: 36 generic, 48 bridge (6 per adjacent pair), ~34 per archetype.
    Bridge cards alternate ordered pair ownership."""
    r = rng or random.Random()
    cards = []
    cid = 0
    for _ in range(36):
        cards.append(SimCard(id=cid, symbols=[], archetype_idx=-1))
        cid += 1
    bridge_per_pair = 6
    for ai in range(NUM_ARCHETYPES):
        next_ai = (ai + 1) % NUM_ARCHETYPES
        for b in range(bridge_per_pair):
            if b % 2 == 0:
                syms = [ARCHETYPES[ai][1], ARCHETYPES[ai][2]]
                card = SimCard(id=cid, symbols=syms, archetype_idx=ai,
                               bridge_arch_idx=next_ai)
            else:
                syms = [ARCHETYPES[next_ai][1], ARCHETYPES[next_ai][2]]
                card = SimCard(id=cid, symbols=syms, archetype_idx=next_ai,
                               bridge_arch_idx=ai)
            cards.append(card)
            cid += 1
    num_bridge = bridge_per_pair * NUM_ARCHETYPES
    remaining = total - 36 - num_bridge
    per_arch = remaining // NUM_ARCHETYPES
    for ai in range(NUM_ARCHETYPES):
        ac, cid = make_archetype_cards(ai, per_arch, cid, r)
        cards.extend(ac)
    while len(cards) < total:
        ai = r.randint(0, NUM_ARCHETYPES - 1)
        syms = make_symbols(ai, 2, r)
        cards.append(SimCard(id=cid, symbols=syms, archetype_idx=ai))
        cid += 1
    for c in cards:
        assign_fitness(c)
    return cards


def generate_pool_asymmetric(total=360, rng=None):
    """Model 4: 36 generic. Varying archetype sizes (24-55)."""
    r = rng or random.Random()
    cards = []
    cid = 0
    for _ in range(36):
        cards.append(SimCard(id=cid, symbols=[], archetype_idx=-1))
        cid += 1
    arch_sizes = {0: 35, 1: 45, 2: 55, 3: 24, 4: 35, 5: 45, 6: 55, 7: 24}
    for ai, count in arch_sizes.items():
        ac, cid = make_archetype_cards(ai, count, cid, r)
        cards.extend(ac)
    while len(cards) < total:
        ai = r.randint(0, NUM_ARCHETYPES - 1)
        syms = make_symbols(ai, 2, r)
        cards.append(SimCard(id=cid, symbols=syms, archetype_idx=ai))
        cid += 1
    for c in cards:
        assign_fitness(c)
    return cards


def generate_pool_pair_optimized(total=360, rng=None):
    """Model 5: 36 generic. 5/70/25 symbol split to maximize pair pool size.
    Any pair pool < 30 gets extra 2-sym cards."""
    r = rng or random.Random()
    cards = []
    cid = 0
    for _ in range(36):
        cards.append(SimCard(id=cid, symbols=[], archetype_idx=-1))
        cid += 1
    per_arch = (total - 36) // NUM_ARCHETYPES
    for ai in range(NUM_ARCHETYPES):
        ac, cid = make_archetype_cards(ai, per_arch, cid, r,
                                        one_pct=0.05, two_pct=0.70, three_pct=0.25)
        cards.extend(ac)
    while len(cards) < total:
        ai = r.randint(0, NUM_ARCHETYPES - 1)
        syms = make_symbols(ai, 2, r)
        cards.append(SimCard(id=cid, symbols=syms, archetype_idx=ai))
        cid += 1
    for ai in range(NUM_ARCHETYPES):
        target_pair = (ARCHETYPES[ai][1], ARCHETYPES[ai][2])
        pair_count = sum(1 for c in cards if c.ordered_pair == target_pair)
        deficit = 30 - pair_count
        if deficit > 0:
            for _ in range(deficit):
                syms = [ARCHETYPES[ai][1], ARCHETYPES[ai][2]]
                cards.append(SimCard(id=cid, symbols=syms, archetype_idx=ai))
                cid += 1
    for c in cards:
        assign_fitness(c)
    return cards


# ============================================================================
# Pair-Escalation Slots Algorithm
# ============================================================================

def draft_pair_escalation(pool, target_arch, num_picks=30, K=6, C=0.50,
                          pack_size=4):
    """
    Pair-Escalation Slots algorithm.

    The player is archetype-committed: picks random for first 2 picks,
    then best fitness for target_arch thereafter.

    Returns (drafted, pack_results).
    pack_results: list of (pick_num, pack, chosen_card, sa_count, P, top_pair,
                            top_count, pair_pool_size)
    """
    pair_counter = Counter()
    drafted = []
    pack_results = []

    # Pre-index pair-matched pools for each possible pair
    pair_pools = {}
    for ai in range(NUM_ARCHETYPES):
        pair = (ARCHETYPES[ai][1], ARCHETYPES[ai][2])
        if pair not in pair_pools:
            pair_pools[pair] = [c for c in pool if c.ordered_pair == pair]

    for pick_num in range(1, num_picks + 1):
        # Step a: top pair and count
        if pair_counter:
            top_pair, top_count = pair_counter.most_common(1)[0]
        else:
            top_pair, top_count = None, 0

        # Step b: P
        P = min(top_count / K, C)

        # Step c: pair-matched subset
        pair_matched = pair_pools.get(top_pair, []) if top_pair else []

        # Step d: generate pack
        pack = []
        for _ in range(pack_size):
            roll = random.random()
            if roll < P and pair_matched:
                pack.append(random.choice(pair_matched))
            else:
                pack.append(random.choice(pool))

        # Step e: player picks (committed player)
        if pick_num <= 2:
            # First 2 picks: random
            chosen = random.choice(pack)
        else:
            # Pick best for target archetype
            def score(c):
                tier = c.fitness.get(target_arch, Tier.F)
                tier_val = {Tier.S: 5, Tier.A: 4, Tier.B: 2, Tier.C: 1, Tier.F: 0}
                return tier_val[tier] * 10 + random.random()
            chosen = max(pack, key=score)

        drafted.append(chosen)

        # Step 3: update pair counter
        if chosen.ordered_pair is not None:
            pair_counter[chosen.ordered_pair] += 1

        # Measure
        sa_count = sum(1 for c in pack if c.is_sa_for(target_arch))
        pair_pool_size = len(pair_matched)

        pack_results.append({
            'pick': pick_num,
            'pack': pack,
            'chosen': chosen,
            'sa': sa_count,
            'P': P,
            'top_pair': top_pair,
            'top_count': top_count,
            'pair_pool_size': pair_pool_size,
        })

    return drafted, pack_results


# ============================================================================
# Pool Analysis
# ============================================================================

def analyze_pair_pools(pool):
    results = {}
    for ai in range(NUM_ARCHETYPES):
        target_pair = (ARCHETYPES[ai][1], ARCHETYPES[ai][2])
        matched = [c for c in pool if c.ordered_pair == target_pair]
        pool_size = len(matched)
        s_count = sum(1 for c in matched if c.is_s_for(ai))
        a_count = sum(1 for c in matched if c.is_sa_for(ai) and not c.is_s_for(ai))
        sa_pct = (s_count + a_count) / pool_size if pool_size > 0 else 0
        results[ai] = {
            'pair': target_pair,
            'pool_size': pool_size,
            's_count': s_count,
            'a_count': a_count,
            's_pct': s_count / pool_size if pool_size > 0 else 0,
            'a_pct': a_count / pool_size if pool_size > 0 else 0,
            'sa_pct': sa_pct,
        }
    return results


# ============================================================================
# Metrics
# ============================================================================

def compute_draft_metrics(pack_results, drafted, target_arch):
    m = {}
    early = [r for r in pack_results if r['pick'] <= 5]
    late = [r for r in pack_results if r['pick'] >= 6]

    # Picks 1-5: unique archetypes with S/A per pack
    early_div = []
    for r in early:
        archs = set()
        for c in r['pack']:
            for aidx in range(NUM_ARCHETYPES):
                if c.is_sa_for(aidx):
                    archs.add(aidx)
        early_div.append(len(archs))
    m['early_diversity'] = statistics.mean(early_div) if early_div else 0

    # Picks 1-5: S/A for emerging arch per pack
    early_sa = [r['sa'] for r in early]
    m['early_sa_target'] = statistics.mean(early_sa) if early_sa else 0

    # Picks 6+: S/A for committed arch per pack
    late_sa = [r['sa'] for r in late]
    m['late_sa'] = statistics.mean(late_sa) if late_sa else 0

    # Picks 6+: off-arch (C/F) per pack
    late_cf = []
    for r in late:
        cf = sum(1 for c in r['pack'] if c.is_cf_for(target_arch))
        late_cf.append(cf)
    m['late_cf'] = statistics.mean(late_cf) if late_cf else 0

    # Convergence pick (3 consecutive packs with 2+ S/A)
    conv = 30
    hits = 0
    for r in pack_results:
        if r['sa'] >= 2:
            hits += 1
        else:
            hits = 0
        if hits >= 3:
            conv = r['pick'] - 2
            break
    m['convergence_pick'] = conv

    # Deck concentration
    sa_drafted = sum(1 for c in drafted if c.is_sa_for(target_arch))
    m['deck_concentration'] = sa_drafted / len(drafted) if drafted else 0

    # SA stddev picks 6+
    m['sa_stddev'] = statistics.stdev(late_sa) if len(late_sa) > 1 else 0

    # SA distribution
    m['sa_distribution'] = dict(Counter(late_sa))

    # Pair accumulation tracking
    pair_counts = Counter()
    first_activation = 30
    for r in pack_results:
        if r['chosen'].ordered_pair is not None:
            pair_counts[r['chosen'].ordered_pair] += 1
        if pair_counts and pair_counts.most_common(1)[0][1] >= 2 and first_activation == 30:
            first_activation = r['pick']
    m['first_activation'] = first_activation

    # Track P at various picks
    m['P_at_10'] = next((r['P'] for r in pack_results if r['pick'] == 10), 0)
    m['P_at_20'] = next((r['P'] for r in pack_results if r['pick'] == 20), 0)
    m['P_at_30'] = next((r['P'] for r in pack_results if r['pick'] == 30), 0)

    # Unique pair-matched cards seen in pack
    pair_card_ids = set()
    for r in pack_results:
        if r['top_pair']:
            for c in r['pack']:
                if c.ordered_pair == r['top_pair']:
                    pair_card_ids.add(c.id)
    m['pair_slot_variety'] = len(pair_card_ids)

    return m


# ============================================================================
# Bridge Strategy
# ============================================================================

def simulate_bridge_draft(pool, arch1, arch2, num_picks=30, K=6, C=0.50):
    pair_counter = Counter()
    drafted = []
    both_sa_packs = 0
    total_late_packs = 0
    sa1_total = 0
    sa2_total = 0

    # Pre-index
    pair_pools = {}
    for ai in range(NUM_ARCHETYPES):
        pair = (ARCHETYPES[ai][1], ARCHETYPES[ai][2])
        if pair not in pair_pools:
            pair_pools[pair] = [c for c in pool if c.ordered_pair == pair]

    for pick_num in range(1, num_picks + 1):
        if pair_counter:
            top_pair, top_count = pair_counter.most_common(1)[0]
        else:
            top_pair, top_count = None, 0
        P = min(top_count / K, C)
        pair_matched = pair_pools.get(top_pair, []) if top_pair else []

        pack = []
        for _ in range(4):
            if random.random() < P and pair_matched:
                pack.append(random.choice(pair_matched))
            else:
                pack.append(random.choice(pool))

        if pick_num >= 6:
            sa1 = sum(1 for c in pack if c.is_sa_for(arch1))
            sa2 = sum(1 for c in pack if c.is_sa_for(arch2))
            if sa1 >= 1 and sa2 >= 1:
                both_sa_packs += 1
            total_late_packs += 1
            sa1_total += sa1
            sa2_total += sa2

        if pick_num <= 2:
            chosen = random.choice(pack)
        else:
            def combined(c):
                sm = {Tier.S: 4, Tier.A: 3, Tier.B: 2, Tier.C: 1, Tier.F: 0}
                return sm[c.fitness.get(arch1, Tier.F)] + sm[c.fitness.get(arch2, Tier.F)]
            best = max(combined(c) for c in pack)
            cands = [c for c in pack if combined(c) == best]
            chosen = random.choice(cands)

        drafted.append(chosen)
        if chosen.ordered_pair is not None:
            pair_counter[chosen.ordered_pair] += 1

    both_pct = (both_sa_packs / total_late_packs * 100) if total_late_packs else 0
    avg1 = sa1_total / total_late_packs if total_late_packs else 0
    avg2 = sa2_total / total_late_packs if total_late_packs else 0
    return {"both_sa_pct": both_pct, "avg_sa1": avg1, "avg_sa2": avg2}


# ============================================================================
# Main Model Runner
# ============================================================================

def run_model(name, pool_gen_fn, num_drafts=1200, num_picks=30, K=6, C=0.50):
    rng = random.Random(42)
    pool = pool_gen_fn(rng=rng)
    pair_info = analyze_pair_pools(pool)

    all_metrics = []
    all_decks = []
    arch_target_counts = Counter()
    arch_card_sets = defaultdict(list)

    for draft_idx in range(num_drafts):
        target_arch = rng.randint(0, NUM_ARCHETYPES - 1)
        arch_target_counts[target_arch] += 1

        random.seed(42 + draft_idx * 13)
        drafted, pack_results = draft_pair_escalation(
            pool, target_arch, num_picks, K=K, C=C)

        m = compute_draft_metrics(pack_results, drafted, target_arch)
        all_metrics.append(m)
        all_decks.append(drafted)
        arch_card_sets[target_arch].append(frozenset(c.id for c in drafted))

    # Aggregate
    agg = {}
    for key in ['early_diversity', 'early_sa_target', 'late_sa', 'late_cf',
                'convergence_pick', 'deck_concentration', 'sa_stddev',
                'first_activation', 'P_at_10', 'P_at_20', 'P_at_30',
                'pair_slot_variety']:
        vals = [m[key] for m in all_metrics]
        agg[key] = statistics.mean(vals) if vals else 0

    # SA distribution
    total_dist = Counter()
    for m in all_metrics:
        for k, v in m.get('sa_distribution', {}).items():
            total_dist[k] += v
    total_count = sum(total_dist.values())
    agg['sa_distribution'] = {k: v / total_count for k, v in sorted(total_dist.items())} if total_count else {}

    # Card overlap
    overlaps = []
    for a in range(NUM_ARCHETYPES):
        sets = arch_card_sets[a]
        for i in range(len(sets)):
            for j in range(i + 1, min(i + 20, len(sets))):
                if len(sets[i] | sets[j]) > 0:
                    overlaps.append(len(sets[i] & sets[j]) / len(sets[i] | sets[j]))
    agg['overlap'] = statistics.mean(overlaps) * 100 if overlaps else 0

    # Archetype frequency
    agg['arch_freq'] = {a: arch_target_counts[a] / num_drafts * 100
                        for a in range(NUM_ARCHETYPES)}

    agg['pair_pools'] = pair_info

    # Bridge strategy
    bridge_results = []
    for ai in range(NUM_ARCHETYPES):
        next_ai = (ai + 1) % NUM_ARCHETYPES
        for trial in range(125):
            random.seed(42 + ai * 1000 + trial * 7)
            result = simulate_bridge_draft(pool, ai, next_ai, num_picks, K, C)
            bridge_results.append(result)
    agg['bridge_both_pct'] = statistics.mean([r["both_sa_pct"] for r in bridge_results])
    agg['bridge_avg_sa1'] = statistics.mean([r["avg_sa1"] for r in bridge_results])
    agg['bridge_avg_sa2'] = statistics.mean([r["avg_sa2"] for r in bridge_results])

    # Per-archetype convergence
    arch_conv = {}
    for ai in range(NUM_ARCHETYPES):
        conv_picks = []
        for run in range(200):
            random.seed(42 + ai * 10000 + run * 3)
            drafted_c, pr_c = draft_pair_escalation(pool, ai, num_picks, K=K, C=C)
            conv_p = 30
            hits = 0
            for r in pr_c:
                if r['sa'] >= 2:
                    hits += 1
                else:
                    hits = 0
                if hits >= 3:
                    conv_p = r['pick'] - 2
                    break
            conv_picks.append(conv_p)
        arch_conv[ARCHETYPE_NAMES[ai]] = statistics.mean(conv_picks)
    agg['arch_convergence'] = arch_conv

    return name, agg, pool


# ============================================================================
# Output
# ============================================================================

def print_results(results):
    print("\n" + "=" * 130)
    print("ARCHETYPE BREAKDOWN — Pair-Escalation Slots (K=6, C=0.50, 4 slots, 30 picks)")
    print("Symbol distribution: 15/60/25 (1-sym/2-sym/3-sym of non-generic)")
    print("=" * 130)

    # Table 1: Standard Targets
    print("\n### TABLE 1: STANDARD TARGETS")
    hdr = (f"{'Model':<35} {'EarlyDiv':>8} {'EarlySA':>8} {'LateSA':>8} "
           f"{'LateOff':>8} {'Conv':>6} {'DeckSA%':>8} {'Overlap%':>9} {'SA_SD':>6}")
    print(hdr)
    print("-" * len(hdr))
    for name, agg, _ in results:
        print(f"{name:<35} {agg['early_diversity']:>8.2f} {agg['early_sa_target']:>8.2f} "
              f"{agg['late_sa']:>8.2f} {agg['late_cf']:>8.2f} "
              f"{agg['convergence_pick']:>6.1f} {agg['deck_concentration']*100:>7.1f}% "
              f"{agg['overlap']:>8.1f}% {agg['sa_stddev']:>6.2f}")

    # Table 2: Pair-Escalation Specific Metrics
    print("\n### TABLE 2: PAIR-ESCALATION SPECIFIC METRICS")
    print(f"{'Model':<35} {'1stActiv':>9} {'P@10':>6} {'P@20':>6} {'P@30':>6} {'PairVar':>8}")
    print("-" * 75)
    for name, agg, _ in results:
        print(f"{name:<35} {agg['first_activation']:>9.1f} {agg['P_at_10']:>6.2f} "
              f"{agg['P_at_20']:>6.2f} {agg['P_at_30']:>6.2f} "
              f"{agg['pair_slot_variety']:>8.1f}")

    # Table 3: Pair-Matched Pool Analysis
    print("\n### TABLE 3: PAIR-MATCHED POOL SUMMARY (across 8 archetypes)")
    print(f"{'Model':<35} {'AvgPool':>8} {'MinPool':>8} {'MaxPool':>8} "
          f"{'AvgS%':>7} {'AvgSA%':>8}")
    print("-" * 82)
    for name, agg, _ in results:
        pp = agg['pair_pools']
        sizes = [pp[ai]['pool_size'] for ai in range(NUM_ARCHETYPES)]
        s_pcts = [pp[ai]['s_pct'] for ai in range(NUM_ARCHETYPES)]
        sa_pcts = [pp[ai]['sa_pct'] for ai in range(NUM_ARCHETYPES)]
        print(f"{name:<35} {statistics.mean(sizes):>8.1f} {min(sizes):>8} {max(sizes):>8} "
              f"{statistics.mean(s_pcts)*100:>6.1f}% {statistics.mean(sa_pcts)*100:>7.1f}%")

    # Table 4: Per-archetype pair pools (detailed for one model)
    print("\n### TABLE 4: PER-ARCHETYPE PAIR POOLS (Equal+SmallGeneric)")
    ref = results[0]
    pp = ref[1]['pair_pools']
    print(f"{'Archetype':<15} {'Pair':<20} {'PoolSize':>9} {'S-tier':>7} {'A-tier':>7} {'SA%':>6}")
    print("-" * 70)
    for ai in range(NUM_ARCHETYPES):
        info = pp[ai]
        pair_str = f"({info['pair'][0].value}, {info['pair'][1].value})"
        print(f"{ARCHETYPE_NAMES[ai]:<15} {pair_str:<20} {info['pool_size']:>9} "
              f"{info['s_count']:>7} {info['a_count']:>7} "
              f"{info['sa_pct']*100:>5.1f}%")

    # Table 5: Bridge Strategy
    print("\n### TABLE 5: BRIDGE STRATEGY VIABILITY")
    print(f"{'Model':<35} {'BothSA%':>8} {'AvgSA1':>8} {'AvgSA2':>8}")
    print("-" * 65)
    for name, agg, _ in results:
        print(f"{name:<35} {agg['bridge_both_pct']:>7.1f}% "
              f"{agg['bridge_avg_sa1']:>8.2f} {agg['bridge_avg_sa2']:>8.2f}")

    # Table 6: Per-Archetype Convergence
    print("\n### TABLE 6: PER-ARCHETYPE CONVERGENCE (avg pick for 2+ S/A)")
    hdr = f"{'Model':<35}"
    for an in ARCHETYPE_NAMES:
        hdr += f" {an[:8]:>9}"
    print(hdr)
    print("-" * len(hdr))
    for name, agg, _ in results:
        line = f"{name:<35}"
        for an in ARCHETYPE_NAMES:
            line += f" {agg['arch_convergence'][an]:>9.1f}"
        print(line)

    # Table 7: Per-Archetype Balance
    print("\n### TABLE 7: PER-ARCHETYPE BALANCE")
    for name, agg, _ in results:
        freqs = agg['arch_freq']
        vals = list(freqs.values())
        print(f"  {name:<35} min={min(vals):.1f}% max={max(vals):.1f}% "
              f"range={max(vals)-min(vals):.1f}pp")

    # Table 8: Target Comparison Scorecard
    print("\n### TABLE 8: TARGET COMPARISON SCORECARD")
    print(f"{'Metric':<50} {'Target':<15}", end="")
    for name, _, _ in results:
        print(f" {name[:14]:>14}", end="")
    print()
    print("-" * (65 + 15 * len(results)))

    targets = [
        ("Picks 1-5: unique archs w/ S/A per pack", ">= 3",
         lambda a: f"{a['early_diversity']:.1f}", lambda a: a['early_diversity'] >= 3),
        ("Picks 1-5: S/A for emerging arch per pack", "<= 2",
         lambda a: f"{a['early_sa_target']:.2f}", lambda a: a['early_sa_target'] <= 2),
        ("Picks 6+: S/A for committed arch per pack", ">= 2 avg",
         lambda a: f"{a['late_sa']:.2f}", lambda a: a['late_sa'] >= 2),
        ("Picks 6+: off-arch (C/F) per pack", ">= 0.5 avg",
         lambda a: f"{a['late_cf']:.2f}", lambda a: a['late_cf'] >= 0.5),
        ("Convergence pick", "5-8",
         lambda a: f"{a['convergence_pick']:.1f}", lambda a: 5 <= a['convergence_pick'] <= 8),
        ("Deck archetype concentration", "60-90% S/A",
         lambda a: f"{a['deck_concentration']*100:.1f}%",
         lambda a: 0.60 <= a['deck_concentration'] <= 0.90),
        ("Run-to-run variety", "< 40% overlap",
         lambda a: f"{a['overlap']:.1f}%", lambda a: a['overlap'] < 40),
        ("StdDev S/A per pack (picks 6+)", ">= 0.8",
         lambda a: f"{a['sa_stddev']:.2f}", lambda a: a['sa_stddev'] >= 0.8),
    ]

    for metric_name, target, extractor, check in targets:
        print(f"{metric_name:<50} {target:<15}", end="")
        for name, agg, _ in results:
            val = extractor(agg)
            passed = check(agg)
            mark = "PASS" if passed else "FAIL"
            print(f" {val+' '+mark:>14}", end="")
        print()

    # SA Distribution
    print("\n### TABLE 9: S/A DISTRIBUTION (picks 6+)")
    for name, agg, _ in results:
        dist = agg.get('sa_distribution', {})
        print(f"  {name}:")
        for k in sorted(dist.keys()):
            print(f"    {k} S/A: {dist[k]*100:.1f}%")


def main():
    random.seed(42)
    models = [
        ("Equal+SmallGeneric(10%)", generate_pool_equal_small_generic),
        ("Equal+LargeGeneric(25%)", generate_pool_equal_large_generic),
        ("Equal+BridgeCards", generate_pool_bridge),
        ("Asymmetric Sizes", generate_pool_asymmetric),
        ("Pair-Pool-Optimized", generate_pool_pair_optimized),
    ]

    results = []
    for name, gen_fn in models:
        print(f"Running: {name}...", flush=True)
        result = run_model(name, gen_fn, num_drafts=1200, num_picks=30, K=6, C=0.50)
        results.append(result)

    print_results(results)


if __name__ == "__main__":
    main()
