#!/usr/bin/env python3
"""
Simulation Agent 8: Compensated Pair Allocation

Algorithm: Every post-commitment pack contains pair-matched slots, R1 slots,
and random slots. Graduated ramp: picks 1-2 fully random, picks 3-5: 1 pair +
1 R1 + 2 random, picks 6+: 2 pair + 1 R1 + 1 random (with jitter).
Pool and algorithm co-designed: non-uniform dual-res distribution compensates
for per-pair fitness asymmetry.
"""

import random
import math
from dataclasses import dataclass, field
from collections import defaultdict
from typing import Optional

# ============================================================
# Constants
# ============================================================

NUM_DRAFTS = 1000
NUM_PICKS = 30
PACK_SIZE = 4
POOL_SIZE = 360
NUM_ARCHETYPES = 8
GENERIC_COUNT = 40  # generic cards with 0 symbols

ARCHETYPES = [
    "Flash",        # 0: Zephyr/Ember
    "Blink",        # 1: Ember/Zephyr
    "Storm",        # 2: Ember/Stone
    "SelfDiscard",  # 3: Stone/Ember
    "SelfMill",     # 4: Stone/Tide
    "Sacrifice",    # 5: Tide/Stone
    "Warriors",     # 6: Tide/Zephyr
    "Ramp",         # 7: Zephyr/Tide
]

RESONANCES = ["Ember", "Stone", "Tide", "Zephyr"]

# Archetype -> (primary, secondary) resonance
ARCHETYPE_RES = {
    "Flash":       ("Zephyr", "Ember"),
    "Blink":       ("Ember", "Zephyr"),
    "Storm":       ("Ember", "Stone"),
    "SelfDiscard": ("Stone", "Ember"),
    "SelfMill":    ("Stone", "Tide"),
    "Sacrifice":   ("Tide", "Stone"),
    "Warriors":    ("Tide", "Zephyr"),
    "Ramp":        ("Zephyr", "Tide"),
}

# Co-primary siblings (share primary resonance)
CO_PRIMARY = {
    "Flash": "Ramp",       "Ramp": "Flash",
    "Blink": "Storm",      "Storm": "Blink",
    "SelfDiscard": "SelfMill", "SelfMill": "SelfDiscard",
    "Sacrifice": "Warriors",   "Warriors": "Sacrifice",
}

# ============================================================
# Fitness Models
# ============================================================

# Per co-primary-pair sibling A-tier rates
FITNESS_MODELS = {
    "optimistic": {
        ("Flash", "Ramp"): 1.0, ("Ramp", "Flash"): 1.0,
        ("Blink", "Storm"): 1.0, ("Storm", "Blink"): 1.0,
        ("SelfDiscard", "SelfMill"): 1.0, ("SelfMill", "SelfDiscard"): 1.0,
        ("Sacrifice", "Warriors"): 1.0, ("Warriors", "Sacrifice"): 1.0,
    },
    "graduated_realistic": {
        ("Warriors", "Sacrifice"): 0.50, ("Sacrifice", "Warriors"): 0.50,
        ("SelfDiscard", "SelfMill"): 0.40, ("SelfMill", "SelfDiscard"): 0.40,
        ("Blink", "Storm"): 0.30, ("Storm", "Blink"): 0.30,
        ("Flash", "Ramp"): 0.25, ("Ramp", "Flash"): 0.25,
    },
    "pessimistic": {
        ("Warriors", "Sacrifice"): 0.35, ("Sacrifice", "Warriors"): 0.35,
        ("SelfDiscard", "SelfMill"): 0.25, ("SelfMill", "SelfDiscard"): 0.25,
        ("Blink", "Storm"): 0.15, ("Storm", "Blink"): 0.15,
        ("Flash", "Ramp"): 0.10, ("Ramp", "Flash"): 0.10,
    },
    "hostile": {
        ("Flash", "Ramp"): 0.08, ("Ramp", "Flash"): 0.08,
        ("Blink", "Storm"): 0.08, ("Storm", "Blink"): 0.08,
        ("SelfDiscard", "SelfMill"): 0.08, ("SelfMill", "SelfDiscard"): 0.08,
        ("Sacrifice", "Warriors"): 0.08, ("Warriors", "Sacrifice"): 0.08,
    },
}

# ============================================================
# Card Pool Construction
# ============================================================

@dataclass
class SimCard:
    id: int
    symbols: list  # list of resonance strings, ordered
    archetype: str  # primary archetype
    power: float
    tier_cache: dict = field(default_factory=dict)

    def primary_res(self):
        if not self.symbols:
            return None
        return self.symbols[0]

    def resonance_pair(self):
        """Return (primary, secondary) resonance if card has 2+ symbols."""
        if len(self.symbols) < 2:
            return None
        return (self.symbols[0], self.symbols[1])


def get_sibling_a_tier(archetype, card_archetype, fitness_model):
    """Return probability that card from card_archetype is A-tier for archetype."""
    if archetype == card_archetype:
        return 1.0  # Home archetype is always S-tier
    if card_archetype == "Generic":
        return 0.25  # Generic cards: 25% chance of being useful for any archetype
    # Check if co-primary siblings
    model = FITNESS_MODELS[fitness_model]
    key = (archetype, card_archetype)
    if key in model:
        return model[key]
    # Non-sibling archetypes: very low fitness
    return 0.05


def card_tier_for_archetype(card, archetype, fitness_model):
    """
    Determine if a card is S-tier, A-tier, or below for a given archetype.
    Returns 'S', 'A', 'B', or 'F'.
    """
    cache_key = (archetype, fitness_model)
    if cache_key in card.tier_cache:
        return card.tier_cache[cache_key]

    if card.archetype == archetype:
        tier = 'S'
    elif card.archetype == "Generic":
        tier = 'A' if random.random() < 0.25 else 'B'
    else:
        a_rate = get_sibling_a_tier(archetype, card.archetype, fitness_model)
        if random.random() < a_rate:
            tier = 'A'
        else:
            tier = 'B' if random.random() < 0.4 else 'F'
    card.tier_cache[cache_key] = tier
    return tier


def is_sa_tier(card, archetype, fitness_model):
    """Is the card S or A tier for the given archetype?"""
    t = card_tier_for_archetype(card, archetype, fitness_model)
    return t in ('S', 'A')


def build_pool(pool_type="enriched_40", compensated=True):
    """
    Build a 360-card pool.

    pool_type:
      - "v7_standard": 15% dual-res, V7 baseline
      - "enriched_40": 40% dual-res, 2-symbol minimum

    compensated: if True, distribute dual-res non-uniformly
                 (more for low-overlap pairs)
    """
    cards = []
    card_id = 0

    if pool_type == "v7_standard":
        # V7 baseline: 360 cards, 36 generic, 324 archetype
        # ~40 per archetype, 15% dual-res (54 dual-res cards)
        generic_count = 36
        arch_cards = 324
        per_arch = arch_cards // NUM_ARCHETYPES  # 40 (with rounding)
        dual_res_total = 54
        dual_per_pair = dual_res_total // NUM_ARCHETYPES  # ~7 per pair

        for arch in ARCHETYPES:
            primary, secondary = ARCHETYPE_RES[arch]
            # dual-res cards for this archetype's pair
            for _ in range(dual_per_pair):
                cards.append(SimCard(
                    id=card_id,
                    symbols=[primary, secondary],
                    archetype=arch,
                    power=random.uniform(4, 8),
                ))
                card_id += 1
            # single-res cards
            remaining = per_arch - dual_per_pair
            for _ in range(remaining):
                cards.append(SimCard(
                    id=card_id,
                    symbols=[primary],
                    archetype=arch,
                    power=random.uniform(3, 9),
                ))
                card_id += 1
        # Generic
        for _ in range(generic_count):
            cards.append(SimCard(
                id=card_id,
                symbols=[],
                archetype="Generic",
                power=random.uniform(3, 7),
            ))
            card_id += 1
        # Pad to 360 if needed
        while len(cards) < 360:
            arch = random.choice(ARCHETYPES)
            primary, _ = ARCHETYPE_RES[arch]
            cards.append(SimCard(
                id=card_id,
                symbols=[primary],
                archetype=arch,
                power=random.uniform(3, 8),
            ))
            card_id += 1

    elif pool_type == "enriched_40":
        # 40% dual-res pool, 2-symbol minimum for non-generics
        # 360 cards: 40 generic + 320 archetype
        # 144 dual-res (40%), 110 same-res double (31%), 66 tri-res (18%)

        # Per-pair dual-res distribution
        if compensated:
            # Non-uniform: more for low-overlap pairs
            pair_dual_counts = {
                "Flash": 22, "Blink": 22,
                "Storm": 18, "SelfDiscard": 18,
                "SelfMill": 16, "Sacrifice": 16,
                "Warriors": 14, "Ramp": 14,
            }
        else:
            pair_dual_counts = {a: 18 for a in ARCHETYPES}

        per_arch_total = 40

        for arch in ARCHETYPES:
            primary, secondary = ARCHETYPE_RES[arch]
            dual_count = pair_dual_counts[arch]

            # Tri-res cards (have primary, secondary, and a third resonance)
            tri_count = max(0, min(8, per_arch_total - dual_count - 12))
            same_res_count = per_arch_total - dual_count - tri_count

            # Dual-resonance cards (primary, secondary)
            for _ in range(dual_count):
                cards.append(SimCard(
                    id=card_id,
                    symbols=[primary, secondary],
                    archetype=arch,
                    power=random.uniform(4, 8),
                ))
                card_id += 1

            # Same-resonance double cards (primary, primary)
            for _ in range(same_res_count):
                cards.append(SimCard(
                    id=card_id,
                    symbols=[primary, primary],
                    archetype=arch,
                    power=random.uniform(3, 9),
                ))
                card_id += 1

            # Tri-resonance cards
            other_res = [r for r in RESONANCES if r != primary and r != secondary]
            for i in range(tri_count):
                third = other_res[i % len(other_res)]
                cards.append(SimCard(
                    id=card_id,
                    symbols=[primary, secondary, third],
                    archetype=arch,
                    power=random.uniform(4, 8),
                ))
                card_id += 1

        # Generic cards
        for _ in range(GENERIC_COUNT):
            cards.append(SimCard(
                id=card_id,
                symbols=[],
                archetype="Generic",
                power=random.uniform(3, 7),
            ))
            card_id += 1

    return cards[:POOL_SIZE]


# ============================================================
# Pool Indexing
# ============================================================

def build_indices(pool):
    """Build lookup indices for efficient card drawing."""
    r1_index = defaultdict(list)     # resonance -> [card]
    pair_index = defaultdict(list)   # (res1, res2) -> [card], only valid archetype pairs

    for card in pool:
        if card.symbols:
            r1_index[card.symbols[0]].append(card)
            pair = card.resonance_pair()
            if pair and pair in VALID_PAIRS:
                pair_index[pair].append(card)
            # For tri-res cards, add to all valid archetype pair indices
            if len(card.symbols) >= 3:
                for i in range(len(card.symbols)):
                    for j in range(len(card.symbols)):
                        if i != j:
                            p = (card.symbols[i], card.symbols[j])
                            if p in VALID_PAIRS and p != pair and card not in pair_index[p]:
                                pair_index[p].append(card)

    return r1_index, pair_index


# ============================================================
# Draft Algorithm: Compensated Pair Allocation
# ============================================================

def generate_pack(pick_num, pair_counters, pool, r1_index, pair_index, rng):
    """
    Generate a pack of 4 cards using Compensated Pair Allocation.

    Graduated ramp:
      - Picks 1-2: 4 random
      - Picks 3-5: 1 pair + 1 R1 + 2 random
      - Picks 6+: jittered (70% 2+1+1, 20% 3+0+1, 10% 1+1+2)
    """
    pack = []

    if pick_num <= 2:
        # Fully random
        pack = rng.sample(pool, PACK_SIZE)
        return pack

    # Find top pair among valid archetype pairs only
    valid_counters = {k: v for k, v in pair_counters.items() if k in VALID_PAIRS}
    if valid_counters:
        top_pair = max(valid_counters, key=valid_counters.get)
    else:
        arch = rng.choice(ARCHETYPES)
        top_pair = ARCHETYPE_RES[arch]
    top_pair_res = top_pair  # (primary, secondary) resonance tuple
    primary_res = top_pair_res[0]

    if pick_num <= 5:
        # 1 pair + 1 R1 + 2 random
        n_pair, n_r1, n_random = 1, 1, 2
    else:
        # Jittered allocation
        roll = rng.random()
        if roll < 0.70:
            n_pair, n_r1, n_random = 2, 1, 1
        elif roll < 0.90:
            n_pair, n_r1, n_random = 3, 0, 1
        else:
            n_pair, n_r1, n_random = 1, 1, 2

    used_ids = set()

    # Draw pair-matched cards
    pair_pool = pair_index.get(top_pair_res, [])
    if pair_pool:
        available = [c for c in pair_pool if c.id not in used_ids]
        if len(available) >= n_pair:
            drawn = rng.sample(available, n_pair)
        else:
            drawn = available[:]
        for c in drawn:
            pack.append(c)
            used_ids.add(c.id)

    # Draw R1-filtered cards
    r1_pool = r1_index.get(primary_res, [])
    if r1_pool and n_r1 > 0:
        available = [c for c in r1_pool if c.id not in used_ids]
        if available:
            drawn = rng.sample(available, min(n_r1, len(available)))
            for c in drawn:
                pack.append(c)
                used_ids.add(c.id)

    # Fill remaining with random cards
    remaining = PACK_SIZE - len(pack)
    if remaining > 0:
        available = [c for c in pool if c.id not in used_ids]
        if available:
            drawn = rng.sample(available, min(remaining, len(available)))
            pack.extend(drawn)

    return pack[:PACK_SIZE]


VALID_PAIRS = set(ARCHETYPE_RES[a] for a in ARCHETYPES)


def get_top_pair(pair_counters, rng, default_arch=None):
    """Get the top valid archetype pair from counters."""
    valid = {k: v for k, v in pair_counters.items() if k in VALID_PAIRS}
    if valid:
        return max(valid, key=valid.get)
    if default_arch:
        return ARCHETYPE_RES[default_arch]
    return ARCHETYPE_RES[rng.choice(ARCHETYPES)]


def update_pair_counters(card, pair_counters):
    """Update pair counters based on drafted card's symbols.
    Only tracks the 8 valid archetype pairs."""
    if len(card.symbols) < 2:
        # Single symbol or generic: contribute to pairs where this is primary
        if card.symbols:
            res = card.symbols[0]
            for arch in ARCHETYPES:
                p, s = ARCHETYPE_RES[arch]
                if p == res:
                    pair_counters[(p, s)] += 1
        return

    # Card has 2+ symbols: check all symbol combinations against valid pairs
    primary = card.symbols[0]
    secondary = card.symbols[1]

    # Strong match if (primary, secondary) is a valid archetype pair
    if (primary, secondary) in VALID_PAIRS:
        pair_counters[(primary, secondary)] += 2

    # For tri-res cards, also check other valid pair combinations
    if len(card.symbols) >= 3:
        for i in range(len(card.symbols)):
            for j in range(len(card.symbols)):
                if i != j:
                    candidate = (card.symbols[i], card.symbols[j])
                    if candidate in VALID_PAIRS and candidate != (primary, secondary):
                        pair_counters[candidate] += 1

    # Weakly increment any valid pair using just the primary resonance
    for arch in ARCHETYPES:
        p, s = ARCHETYPE_RES[arch]
        if p == primary and (p, s) != (primary, secondary):
            pair_counters[(p, s)] += 1


# ============================================================
# Player Strategies
# ============================================================

def archetype_committed_pick(pack, committed_arch, fitness_model):
    """Pick the best card for the committed archetype."""
    best = None
    best_score = -1
    for card in pack:
        if is_sa_tier(card, committed_arch, fitness_model):
            score = card.power + 10  # Prioritize S/A cards
        else:
            score = card.power
        if score > best_score:
            best_score = score
            best = card
    return best


def power_chaser_pick(pack):
    """Pick the highest raw power card."""
    return max(pack, key=lambda c: c.power)


def signal_reader_pick(pack, pair_counters, pick_num, fitness_model):
    """
    Evaluate which archetype seems most available and draft toward it.
    Before pick 6, explore. After pick 6, commit to top pair's archetype.
    """
    if pick_num <= 5:
        # Explore: pick the card that contributes most to any pair
        best = None
        best_score = -1
        for card in pack:
            score = card.power
            pair = card.resonance_pair()
            if pair and pair in pair_counters:
                score += pair_counters[pair] * 0.5
            if score > best_score:
                best_score = score
                best = card
        return best
    else:
        # Commit to top pair's archetype
        top_pair = get_top_pair(pair_counters, rng=random.Random())
        # Find which archetype matches this pair
        committed = None
        for arch in ARCHETYPES:
            p, s = ARCHETYPE_RES[arch]
            if (p, s) == top_pair:
                committed = arch
                break
        if committed is None:
            committed = ARCHETYPES[0]
        return archetype_committed_pick(pack, committed, fitness_model)


# ============================================================
# Single Draft Simulation
# ============================================================

@dataclass
class DraftResult:
    picks: list = field(default_factory=list)
    packs: list = field(default_factory=list)
    committed_archetype: str = ""
    pack_sa_counts: list = field(default_factory=list)  # S/A count per pack
    per_arch_pack_sa: dict = field(default_factory=dict)  # arch -> [sa_count per pack]
    convergence_pick: int = 0


def simulate_draft(pool, r1_index, pair_index, strategy, fitness_model, rng,
                   forced_archetype=None):
    """
    Simulate a single 30-pick draft.

    strategy: "committed", "power", "signal"
    """
    result = DraftResult()
    pair_counters = defaultdict(int)

    # For committed strategy, pick a random archetype
    if forced_archetype:
        committed_arch = forced_archetype
    elif strategy == "committed":
        committed_arch = rng.choice(ARCHETYPES)
    else:
        committed_arch = None

    # Track when we consider committed (for signal reader)
    signal_committed_arch = None

    for pick_num in range(1, NUM_PICKS + 1):
        # Generate pack
        pack = generate_pack(pick_num, pair_counters, pool, r1_index,
                             pair_index, rng)
        result.packs.append(pack)

        # Player picks
        if strategy == "committed":
            card = archetype_committed_pick(pack, committed_arch, fitness_model)
        elif strategy == "power":
            card = power_chaser_pick(pack)
        elif strategy == "signal":
            card = signal_reader_pick(pack, pair_counters, pick_num,
                                      fitness_model)
            if pick_num == 6:
                top_pair = get_top_pair(pair_counters, rng)
                for arch in ARCHETYPES:
                    p, s = ARCHETYPE_RES[arch]
                    if (p, s) == top_pair:
                        signal_committed_arch = arch
                        break
        else:
            card = pack[0]

        result.picks.append(card)
        update_pair_counters(card, pair_counters)

    # Determine committed archetype for evaluation
    if strategy == "committed":
        result.committed_archetype = committed_arch
    elif strategy == "signal":
        result.committed_archetype = signal_committed_arch or rng.choice(ARCHETYPES)
    else:
        # Power chaser: evaluate against the archetype with most S/A cards picked
        arch_counts = defaultdict(int)
        for card in result.picks:
            for arch in ARCHETYPES:
                if is_sa_tier(card, arch, fitness_model):
                    arch_counts[arch] += 1
        result.committed_archetype = max(arch_counts, key=arch_counts.get) if arch_counts else ARCHETYPES[0]

    # Calculate pack S/A counts for committed archetype
    ca = result.committed_archetype
    for i, pack in enumerate(result.packs):
        sa_count = sum(1 for c in pack if is_sa_tier(c, ca, fitness_model))
        result.pack_sa_counts.append(sa_count)

    # Calculate per-archetype pack S/A (for all archetypes)
    for arch in ARCHETYPES:
        arch_sa = []
        for pack in result.packs:
            sa = sum(1 for c in pack if is_sa_tier(c, arch, fitness_model))
            arch_sa.append(sa)
        result.per_arch_pack_sa[arch] = arch_sa

    # Convergence pick: first pick where trailing 3-pack average >= 1.5
    for i in range(2, len(result.pack_sa_counts)):
        avg = sum(result.pack_sa_counts[i-2:i+1]) / 3
        if avg >= 1.5:
            result.convergence_pick = i + 1  # 1-indexed
            break
    else:
        result.convergence_pick = NUM_PICKS

    return result


# ============================================================
# Metrics Calculation
# ============================================================

def compute_metrics(results, fitness_model):
    """Compute all 10 metrics from a list of DraftResults."""
    metrics = {}

    # M1: Picks 1-5: unique archetypes with S/A cards per pack (avg)
    m1_values = []
    for r in results:
        for pick_idx in range(min(5, len(r.packs))):
            pack = r.packs[pick_idx]
            archs_with_sa = set()
            for arch in ARCHETYPES:
                for c in pack:
                    if is_sa_tier(c, arch, fitness_model):
                        archs_with_sa.add(arch)
                        break
            m1_values.append(len(archs_with_sa))
    metrics["M1"] = sum(m1_values) / len(m1_values) if m1_values else 0

    # M2: Picks 1-5: S/A cards for any single archetype per pack (max across archetypes)
    m2_values = []
    for r in results:
        for pick_idx in range(min(5, len(r.packs))):
            pack = r.packs[pick_idx]
            max_sa = 0
            for arch in ARCHETYPES:
                sa = sum(1 for c in pack if is_sa_tier(c, arch, fitness_model))
                max_sa = max(max_sa, sa)
            m2_values.append(max_sa)
    metrics["M2"] = sum(m2_values) / len(m2_values) if m2_values else 0

    # M3: Picks 6+: S/A cards for committed archetype per pack (avg)
    m3_values = []
    for r in results:
        ca = r.committed_archetype
        for pick_idx in range(5, len(r.packs)):
            sa = sum(1 for c in r.packs[pick_idx]
                     if is_sa_tier(c, ca, fitness_model))
            m3_values.append(sa)
    metrics["M3"] = sum(m3_values) / len(m3_values) if m3_values else 0

    # M3 per archetype
    m3_per_arch = {}
    for arch in ARCHETYPES:
        vals = []
        for r in results:
            if r.committed_archetype == arch:
                for pick_idx in range(5, len(r.packs)):
                    sa = sum(1 for c in r.packs[pick_idx]
                             if is_sa_tier(c, arch, fitness_model))
                    vals.append(sa)
        m3_per_arch[arch] = sum(vals) / len(vals) if vals else 0
    metrics["M3_per_arch"] = m3_per_arch

    # M4: Picks 6+: off-archetype (not S/A) cards per pack
    m4_values = []
    for r in results:
        ca = r.committed_archetype
        for pick_idx in range(5, len(r.packs)):
            off = sum(1 for c in r.packs[pick_idx]
                      if not is_sa_tier(c, ca, fitness_model))
            m4_values.append(off)
    metrics["M4"] = sum(m4_values) / len(m4_values) if m4_values else 0

    # M5: Convergence pick (average)
    metrics["M5"] = sum(r.convergence_pick for r in results) / len(results)

    # M6: Deck archetype concentration (% S/A cards in final deck)
    m6_values = []
    for r in results:
        ca = r.committed_archetype
        sa_count = sum(1 for c in r.picks if is_sa_tier(c, ca, fitness_model))
        m6_values.append(sa_count / len(r.picks) if r.picks else 0)
    metrics["M6"] = sum(m6_values) / len(m6_values) if m6_values else 0

    # M7: Run-to-run variety (card overlap between pairs of runs with same archetype)
    m7_per_arch = defaultdict(list)
    for r in results:
        m7_per_arch[r.committed_archetype].append(set(c.id for c in r.picks))
    overlap_values = []
    for arch, id_sets in m7_per_arch.items():
        for i in range(min(50, len(id_sets))):
            for j in range(i + 1, min(50, len(id_sets))):
                intersection = len(id_sets[i] & id_sets[j])
                union = len(id_sets[i] | id_sets[j])
                if union > 0:
                    overlap_values.append(intersection / union)
    metrics["M7"] = sum(overlap_values) / len(overlap_values) if overlap_values else 0

    # M8: Archetype frequency across runs
    arch_freq = defaultdict(int)
    for r in results:
        arch_freq[r.committed_archetype] += 1
    total = len(results)
    metrics["M8_max"] = max(arch_freq.values()) / total if arch_freq else 0
    metrics["M8_min"] = min(arch_freq.values()) / total if arch_freq else 0

    # M9: StdDev of S/A cards per pack (picks 6+)
    m9_per_draft = []
    for r in results:
        ca = r.committed_archetype
        sa_list = []
        for pick_idx in range(5, len(r.packs)):
            sa = sum(1 for c in r.packs[pick_idx]
                     if is_sa_tier(c, ca, fitness_model))
            sa_list.append(sa)
        if sa_list:
            mean = sum(sa_list) / len(sa_list)
            var = sum((x - mean) ** 2 for x in sa_list) / len(sa_list)
            m9_per_draft.append(math.sqrt(var))
    metrics["M9"] = sum(m9_per_draft) / len(m9_per_draft) if m9_per_draft else 0

    # M10: Max consecutive packs below 1.5 S/A (picks 6+)
    m10_values = []
    for r in results:
        ca = r.committed_archetype
        max_consec = 0
        current = 0
        for pick_idx in range(5, len(r.packs)):
            sa = sum(1 for c in r.packs[pick_idx]
                     if is_sa_tier(c, ca, fitness_model))
            if sa < 1.5:
                current += 1
                max_consec = max(max_consec, current)
            else:
                current = 0
        m10_values.append(max_consec)
    metrics["M10_avg"] = sum(m10_values) / len(m10_values) if m10_values else 0
    metrics["M10_max"] = max(m10_values) if m10_values else 0

    # Pack quality distribution (picks 6+)
    all_sa = []
    for r in results:
        ca = r.committed_archetype
        for pick_idx in range(5, len(r.packs)):
            sa = sum(1 for c in r.packs[pick_idx]
                     if is_sa_tier(c, ca, fitness_model))
            all_sa.append(sa)
    all_sa.sort()
    n = len(all_sa)
    if n > 0:
        metrics["PQ_p10"] = all_sa[int(n * 0.10)]
        metrics["PQ_p25"] = all_sa[int(n * 0.25)]
        metrics["PQ_p50"] = all_sa[int(n * 0.50)]
        metrics["PQ_p75"] = all_sa[int(n * 0.75)]
        metrics["PQ_p90"] = all_sa[int(n * 0.90)]
    else:
        metrics["PQ_p10"] = metrics["PQ_p25"] = metrics["PQ_p50"] = 0
        metrics["PQ_p75"] = metrics["PQ_p90"] = 0

    # Consecutive bad pack analysis (avg and worst case)
    consec_bad = []
    for r in results:
        ca = r.committed_archetype
        current = 0
        worst = 0
        total_bad_runs = []
        for pick_idx in range(5, len(r.packs)):
            sa = sum(1 for c in r.packs[pick_idx]
                     if is_sa_tier(c, ca, fitness_model))
            if sa < 1.5:
                current += 1
            else:
                if current > 0:
                    total_bad_runs.append(current)
                current = 0
            worst = max(worst, current)
        if current > 0:
            total_bad_runs.append(current)
        consec_bad.append({
            "worst": worst,
            "avg_run": sum(total_bad_runs) / len(total_bad_runs) if total_bad_runs else 0,
            "total_bad_packs": sum(total_bad_runs),
        })
    metrics["bad_consec_avg_worst"] = sum(c["worst"] for c in consec_bad) / len(consec_bad)
    metrics["bad_consec_global_worst"] = max(c["worst"] for c in consec_bad) if consec_bad else 0

    return metrics


# ============================================================
# Parameter Sensitivity
# ============================================================

def run_sensitivity(pool_type, fitness_model, compensated, rng_seed=42):
    """Run parameter sensitivity sweeps."""
    results = {}

    # Sweep 1: Pair slot count (1, 2, 3 fixed — no jitter)
    for n_pair_slots in [1, 2, 3]:
        rng = random.Random(rng_seed)
        pool = build_pool(pool_type, compensated)
        r1_index, pair_index = build_indices(pool)

        draft_results = []
        for _ in range(200):  # fewer drafts for sensitivity
            # Override generate_pack behavior via monkey-patching approach:
            # Instead, just run with committed strategy and measure M3
            dr = simulate_draft_with_fixed_slots(
                pool, r1_index, pair_index, "committed", fitness_model,
                rng, n_pair_slots)
            draft_results.append(dr)

        m = compute_metrics(draft_results, fitness_model)
        results[f"pair_slots={n_pair_slots}"] = {
            "M3": m["M3"], "M4": m["M4"], "M9": m["M9"], "M10_avg": m["M10_avg"]
        }

    # Sweep 2: Jitter distribution (no jitter vs standard vs aggressive)
    for jitter_name, jitter_dist in [
        ("none", (1.0, 0.0, 0.0)),
        ("standard", (0.7, 0.2, 0.1)),
        ("aggressive", (0.4, 0.4, 0.2)),
    ]:
        rng = random.Random(rng_seed)
        pool = build_pool(pool_type, compensated)
        r1_index, pair_index = build_indices(pool)

        draft_results = []
        for _ in range(200):
            dr = simulate_draft_with_jitter(
                pool, r1_index, pair_index, "committed", fitness_model,
                rng, jitter_dist)
            draft_results.append(dr)

        m = compute_metrics(draft_results, fitness_model)
        results[f"jitter={jitter_name}"] = {
            "M3": m["M3"], "M9": m["M9"], "M10_avg": m["M10_avg"]
        }

    # Sweep 3: Ramp start pick (3, 4, 5, 6)
    for ramp_start in [3, 4, 5, 6]:
        rng = random.Random(rng_seed)
        pool = build_pool(pool_type, compensated)
        r1_index, pair_index = build_indices(pool)

        draft_results = []
        for _ in range(200):
            dr = simulate_draft_with_ramp_start(
                pool, r1_index, pair_index, "committed", fitness_model,
                rng, ramp_start)
            draft_results.append(dr)

        m = compute_metrics(draft_results, fitness_model)
        results[f"ramp_start={ramp_start}"] = {
            "M3": m["M3"], "M5": m["M5"], "M9": m["M9"]
        }

    return results


def simulate_draft_with_fixed_slots(pool, r1_index, pair_index, strategy,
                                     fitness_model, rng, n_pair_slots):
    """Draft with fixed pair slot count (no jitter, for sensitivity)."""
    result = DraftResult()
    pair_counters = defaultdict(int)
    committed_arch = rng.choice(ARCHETYPES)

    for pick_num in range(1, NUM_PICKS + 1):
        if pick_num <= 2:
            pack = rng.sample(pool, PACK_SIZE)
        else:
            top_pair = get_top_pair(pair_counters, rng, committed_arch)
            primary_res = top_pair[0]
            pack = []
            used_ids = set()

            n_p = n_pair_slots if pick_num >= 6 else min(1, n_pair_slots)
            n_r = 1 if (PACK_SIZE - n_p) >= 2 else 0
            n_rand = PACK_SIZE - n_p - n_r

            pair_pool = pair_index.get(top_pair, [])
            if pair_pool:
                avail = [c for c in pair_pool if c.id not in used_ids]
                drawn = rng.sample(avail, min(n_p, len(avail)))
                for c in drawn:
                    pack.append(c)
                    used_ids.add(c.id)

            if n_r > 0:
                r1_pool = r1_index.get(primary_res, [])
                avail = [c for c in r1_pool if c.id not in used_ids]
                if avail:
                    drawn = rng.sample(avail, min(n_r, len(avail)))
                    for c in drawn:
                        pack.append(c)
                        used_ids.add(c.id)

            remaining = PACK_SIZE - len(pack)
            if remaining > 0:
                avail = [c for c in pool if c.id not in used_ids]
                drawn = rng.sample(avail, min(remaining, len(avail)))
                pack.extend(drawn)

            pack = pack[:PACK_SIZE]

        result.packs.append(pack)
        card = archetype_committed_pick(pack, committed_arch, fitness_model)
        result.picks.append(card)
        update_pair_counters(card, pair_counters)

    result.committed_archetype = committed_arch
    ca = committed_arch
    for pack in result.packs:
        sa = sum(1 for c in pack if is_sa_tier(c, ca, fitness_model))
        result.pack_sa_counts.append(sa)

    for i in range(2, len(result.pack_sa_counts)):
        avg = sum(result.pack_sa_counts[i-2:i+1]) / 3
        if avg >= 1.5:
            result.convergence_pick = i + 1
            break
    else:
        result.convergence_pick = NUM_PICKS

    return result


def simulate_draft_with_jitter(pool, r1_index, pair_index, strategy,
                                fitness_model, rng, jitter_dist):
    """Draft with configurable jitter distribution."""
    result = DraftResult()
    pair_counters = defaultdict(int)
    committed_arch = rng.choice(ARCHETYPES)

    p_211, p_301, p_112 = jitter_dist

    for pick_num in range(1, NUM_PICKS + 1):
        if pick_num <= 2:
            pack = rng.sample(pool, PACK_SIZE)
        elif pick_num <= 5:
            top_pair = get_top_pair(pair_counters, rng, committed_arch)
            pack = _build_structured_pack(pool, r1_index, pair_index,
                                          top_pair, 1, 1, 2, rng)
        else:
            top_pair = get_top_pair(pair_counters, rng, committed_arch)
            roll = rng.random()
            if roll < p_211:
                n_p, n_r, n_rand = 2, 1, 1
            elif roll < p_211 + p_301:
                n_p, n_r, n_rand = 3, 0, 1
            else:
                n_p, n_r, n_rand = 1, 1, 2
            pack = _build_structured_pack(pool, r1_index, pair_index,
                                          top_pair, n_p, n_r, n_rand, rng)

        result.packs.append(pack)
        card = archetype_committed_pick(pack, committed_arch, fitness_model)
        result.picks.append(card)
        update_pair_counters(card, pair_counters)

    result.committed_archetype = committed_arch
    ca = committed_arch
    for pack in result.packs:
        sa = sum(1 for c in pack if is_sa_tier(c, ca, fitness_model))
        result.pack_sa_counts.append(sa)
    for i in range(2, len(result.pack_sa_counts)):
        avg = sum(result.pack_sa_counts[i-2:i+1]) / 3
        if avg >= 1.5:
            result.convergence_pick = i + 1
            break
    else:
        result.convergence_pick = NUM_PICKS

    return result


def simulate_draft_with_ramp_start(pool, r1_index, pair_index, strategy,
                                    fitness_model, rng, ramp_start):
    """Draft with configurable ramp start pick."""
    result = DraftResult()
    pair_counters = defaultdict(int)
    committed_arch = rng.choice(ARCHETYPES)

    for pick_num in range(1, NUM_PICKS + 1):
        if pick_num <= ramp_start - 1:
            pack = rng.sample(pool, PACK_SIZE)
        elif pick_num <= ramp_start + 2:
            top_pair = get_top_pair(pair_counters, rng, committed_arch)
            pack = _build_structured_pack(pool, r1_index, pair_index,
                                          top_pair, 1, 1, 2, rng)
        else:
            top_pair = get_top_pair(pair_counters, rng, committed_arch)
            roll = rng.random()
            if roll < 0.7:
                n_p, n_r, n_rand = 2, 1, 1
            elif roll < 0.9:
                n_p, n_r, n_rand = 3, 0, 1
            else:
                n_p, n_r, n_rand = 1, 1, 2
            pack = _build_structured_pack(pool, r1_index, pair_index,
                                          top_pair, n_p, n_r, n_rand, rng)

        result.packs.append(pack)
        card = archetype_committed_pick(pack, committed_arch, fitness_model)
        result.picks.append(card)
        update_pair_counters(card, pair_counters)

    result.committed_archetype = committed_arch
    ca = committed_arch
    for pack in result.packs:
        sa = sum(1 for c in pack if is_sa_tier(c, ca, fitness_model))
        result.pack_sa_counts.append(sa)
    for i in range(2, len(result.pack_sa_counts)):
        avg = sum(result.pack_sa_counts[i-2:i+1]) / 3
        if avg >= 1.5:
            result.convergence_pick = i + 1
            break
    else:
        result.convergence_pick = NUM_PICKS

    return result


def _build_structured_pack(pool, r1_index, pair_index, top_pair,
                           n_pair, n_r1, n_random, rng):
    """Build a pack with specified slot structure."""
    pack = []
    used_ids = set()
    primary_res = top_pair[0]

    pair_pool = pair_index.get(top_pair, [])
    if pair_pool:
        avail = [c for c in pair_pool if c.id not in used_ids]
        drawn = rng.sample(avail, min(n_pair, len(avail)))
        for c in drawn:
            pack.append(c)
            used_ids.add(c.id)

    if n_r1 > 0:
        r1_pool = r1_index.get(primary_res, [])
        avail = [c for c in r1_pool if c.id not in used_ids]
        if avail:
            drawn = rng.sample(avail, min(n_r1, len(avail)))
            for c in drawn:
                pack.append(c)
                used_ids.add(c.id)

    remaining = PACK_SIZE - len(pack)
    if remaining > 0:
        avail = [c for c in pool if c.id not in used_ids]
        drawn = rng.sample(avail, min(remaining, len(avail)))
        pack.extend(drawn)

    return pack[:PACK_SIZE]


# ============================================================
# Draft Traces
# ============================================================

def generate_trace(pool, r1_index, pair_index, strategy, fitness_model,
                   rng, label, forced_archetype=None):
    """Generate a detailed draft trace."""
    result = simulate_draft(pool, r1_index, pair_index, strategy,
                            fitness_model, rng, forced_archetype)
    ca = result.committed_archetype

    lines = [f"\n### Trace: {label} ({ca})"]
    lines.append(f"Strategy: {strategy}, Fitness: {fitness_model}")
    lines.append("")
    lines.append("| Pick | Pack S/A | Picked Card | Card Arch | Tier |")
    lines.append("|:----:|:-------:|-------------|-----------|:----:|")

    for i, (pack, card) in enumerate(zip(result.packs, result.picks)):
        pick_num = i + 1
        sa_count = sum(1 for c in pack if is_sa_tier(c, ca, fitness_model))
        tier = card_tier_for_archetype(card, ca, fitness_model)
        symbols = "/".join(card.symbols) if card.symbols else "none"
        lines.append(
            f"| {pick_num} | {sa_count} | [{symbols}] | {card.archetype} | {tier} |"
        )

    total_sa = sum(1 for c in result.picks if is_sa_tier(c, ca, fitness_model))
    post_sa = [sum(1 for c in p if is_sa_tier(c, ca, fitness_model))
               for p in result.packs[5:]]
    avg_post = sum(post_sa) / len(post_sa) if post_sa else 0
    lines.append(f"\nTotal S/A picked: {total_sa}/{NUM_PICKS}")
    lines.append(f"Post-commitment avg S/A per pack: {avg_post:.2f}")
    lines.append(f"Convergence pick: {result.convergence_pick}")

    return "\n".join(lines)


# ============================================================
# Baseline Comparison (V7 Surge+Floor simulation)
# ============================================================

def simulate_surge_floor_draft(pool, r1_index, fitness_model, rng,
                                threshold=3, surge_slots=3, floor_start=3):
    """Simulate V7's Surge+Floor algorithm for baseline comparison."""
    result = DraftResult()
    res_counters = defaultdict(int)
    committed_arch = rng.choice(ARCHETYPES)
    primary_res = ARCHETYPE_RES[committed_arch][0]

    surge_counter = 0

    for pick_num in range(1, NUM_PICKS + 1):
        pack = []
        used_ids = set()

        if pick_num < floor_start:
            pack = rng.sample(pool, PACK_SIZE)
        else:
            surge_counter += 1
            if surge_counter >= threshold:
                # Surge pack: surge_slots from R1, rest random
                surge_counter = 0
                r1_pool = r1_index.get(primary_res, [])
                if r1_pool:
                    avail = [c for c in r1_pool if c.id not in used_ids]
                    drawn = rng.sample(avail, min(surge_slots, len(avail)))
                    for c in drawn:
                        pack.append(c)
                        used_ids.add(c.id)
                remaining = PACK_SIZE - len(pack)
                if remaining > 0:
                    avail = [c for c in pool if c.id not in used_ids]
                    drawn = rng.sample(avail, min(remaining, len(avail)))
                    pack.extend(drawn)
            else:
                # Floor pack: 1 R1 + 3 random
                r1_pool = r1_index.get(primary_res, [])
                if r1_pool:
                    avail = [c for c in r1_pool if c.id not in used_ids]
                    drawn = rng.sample(avail, min(1, len(avail)))
                    for c in drawn:
                        pack.append(c)
                        used_ids.add(c.id)
                remaining = PACK_SIZE - len(pack)
                if remaining > 0:
                    avail = [c for c in pool if c.id not in used_ids]
                    drawn = rng.sample(avail, min(remaining, len(avail)))
                    pack.extend(drawn)

        pack = pack[:PACK_SIZE]
        result.packs.append(pack)
        card = archetype_committed_pick(pack, committed_arch, fitness_model)
        result.picks.append(card)

    result.committed_archetype = committed_arch
    ca = committed_arch
    for pack in result.packs:
        sa = sum(1 for c in pack if is_sa_tier(c, ca, fitness_model))
        result.pack_sa_counts.append(sa)
    for i in range(2, len(result.pack_sa_counts)):
        avg = sum(result.pack_sa_counts[i-2:i+1]) / 3
        if avg >= 1.5:
            result.convergence_pick = i + 1
            break
    else:
        result.convergence_pick = NUM_PICKS

    return result


# ============================================================
# Main Simulation
# ============================================================

def run_full_simulation():
    """Run the complete simulation suite."""
    output_lines = []

    def log(s=""):
        output_lines.append(s)
        print(s)

    log("=" * 70)
    log("COMPENSATED PAIR ALLOCATION — FULL SIMULATION")
    log("=" * 70)

    fitness_models = ["optimistic", "graduated_realistic", "pessimistic", "hostile"]
    pool_configs = [
        ("v7_standard", False, "V7 Standard (15% dual-res)"),
        ("enriched_40", True, "40% Enriched (compensated)"),
        ("enriched_40", False, "40% Enriched (uniform)"),
    ]
    strategies = ["committed", "power", "signal"]

    all_results = {}

    for pool_type, compensated, pool_label in pool_configs:
        for fm in fitness_models:
            log(f"\n{'='*60}")
            log(f"Pool: {pool_label} | Fitness: {fm}")
            log(f"{'='*60}")

            rng = random.Random(42)
            pool = build_pool(pool_type, compensated)
            r1_index, pair_index = build_indices(pool)

            for strategy in strategies:
                strat_rng = random.Random(rng.randint(0, 2**32))
                draft_results = []

                for d in range(NUM_DRAFTS):
                    # Clear tier caches for each draft
                    for card in pool:
                        card.tier_cache = {}

                    dr = simulate_draft(pool, r1_index, pair_index,
                                        strategy, fm, strat_rng)
                    draft_results.append(dr)

                metrics = compute_metrics(draft_results, fm)
                key = (pool_label, fm, strategy)
                all_results[key] = metrics

                log(f"\n  Strategy: {strategy}")
                log(f"    M1 (early variety):     {metrics['M1']:.2f} (target >= 3)")
                log(f"    M2 (early focus):       {metrics['M2']:.2f} (target <= 2)")
                log(f"    M3 (post-commit S/A):   {metrics['M3']:.2f} (target >= 2.0)")
                log(f"    M4 (off-archetype):     {metrics['M4']:.2f} (target >= 0.5)")
                log(f"    M5 (convergence pick):  {metrics['M5']:.1f} (target 5-8)")
                log(f"    M6 (concentration):     {metrics['M6']:.1%} (target 60-90%)")
                log(f"    M7 (card overlap):      {metrics['M7']:.1%} (target < 40%)")
                log(f"    M8 (arch freq):         {metrics['M8_min']:.1%}-{metrics['M8_max']:.1%} (target 5-20%)")
                log(f"    M9 (S/A stddev):        {metrics['M9']:.2f} (target >= 0.8)")
                log(f"    M10 avg consec bad:     {metrics['M10_avg']:.1f} (target <= 2)")
                log(f"    M10 worst consec bad:   {metrics['M10_max']}")
                log(f"    Pack quality: p10={metrics['PQ_p10']} p25={metrics['PQ_p25']} "
                    f"p50={metrics['PQ_p50']} p75={metrics['PQ_p75']} p90={metrics['PQ_p90']}")
                log(f"    Bad consec avg worst:   {metrics['bad_consec_avg_worst']:.1f}")
                log(f"    Bad consec global worst:{metrics['bad_consec_global_worst']}")

                # Per-archetype M3
                if strategy == "committed":
                    log(f"\n    Per-Archetype M3:")
                    for arch in ARCHETYPES:
                        v = metrics['M3_per_arch'].get(arch, 0)
                        log(f"      {arch:15s}: {v:.2f}")

    # ============================================================
    # Baseline comparison: Surge+Floor on same pools
    # ============================================================
    log(f"\n{'='*60}")
    log("BASELINE: V7 Surge+Floor (T=3, S=3)")
    log(f"{'='*60}")

    baseline_results = {}
    for pool_type, compensated, pool_label in pool_configs[:2]:  # V7 and enriched
        for fm in fitness_models:
            rng = random.Random(42)
            pool = build_pool(pool_type, compensated)
            r1_index, pair_index = build_indices(pool)

            draft_results = []
            for d in range(NUM_DRAFTS):
                for card in pool:
                    card.tier_cache = {}
                dr = simulate_surge_floor_draft(pool, r1_index, fm, rng)
                draft_results.append(dr)

            metrics = compute_metrics(draft_results, fm)
            baseline_results[(pool_label, fm)] = metrics

            log(f"\n  {pool_label} | {fm} | committed")
            log(f"    M3: {metrics['M3']:.2f} | M9: {metrics['M9']:.2f} | "
                f"M10 avg: {metrics['M10_avg']:.1f} | M10 worst: {metrics['M10_max']}")
            log(f"    Pack quality: p10={metrics['PQ_p10']} p25={metrics['PQ_p25']} "
                f"p50={metrics['PQ_p50']} p75={metrics['PQ_p75']} p90={metrics['PQ_p90']}")

    # ============================================================
    # Parameter Sensitivity
    # ============================================================
    log(f"\n{'='*60}")
    log("PARAMETER SENSITIVITY (Graduated Realistic, 40% Enriched)")
    log(f"{'='*60}")

    sens = run_sensitivity("enriched_40", "graduated_realistic", True, 42)
    for param, vals in sens.items():
        log(f"\n  {param}:")
        for k, v in vals.items():
            log(f"    {k}: {v:.2f}" if isinstance(v, float) else f"    {k}: {v}")

    # ============================================================
    # Draft Traces
    # ============================================================
    log(f"\n{'='*60}")
    log("DRAFT TRACES (Graduated Realistic, 40% Enriched Compensated)")
    log(f"{'='*60}")

    rng = random.Random(123)
    pool = build_pool("enriched_40", True)
    r1_index, pair_index = build_indices(pool)
    for card in pool:
        card.tier_cache = {}

    trace1 = generate_trace(pool, r1_index, pair_index, "committed",
                            "graduated_realistic", random.Random(100),
                            "Early Committer (Warriors)", "Warriors")
    log(trace1)

    for card in pool:
        card.tier_cache = {}
    trace2 = generate_trace(pool, r1_index, pair_index, "power",
                            "graduated_realistic", random.Random(200),
                            "Power Chaser")
    log(trace2)

    for card in pool:
        card.tier_cache = {}
    trace3 = generate_trace(pool, r1_index, pair_index, "signal",
                            "graduated_realistic", random.Random(300),
                            "Signal Reader")
    log(trace3)

    # ============================================================
    # Fitness Degradation Curve
    # ============================================================
    log(f"\n{'='*60}")
    log("FITNESS DEGRADATION CURVE")
    log(f"{'='*60}")

    log("\nCompensated Pair Allocation (committed, 40% Enriched compensated):")
    log("| Fitness Model | M3 | M10 avg | Worst Consec Bad |")
    log("|---------------|:--:|:-------:|:----------------:|")
    for fm in fitness_models:
        key = ("40% Enriched (compensated)", fm, "committed")
        m = all_results[key]
        log(f"| {fm:25s} | {m['M3']:.2f} | {m['M10_avg']:.1f} | {m['M10_max']} |")

    log("\nSurge+Floor Baseline (committed, V7 Standard):")
    log("| Fitness Model | M3 | M10 avg | Worst Consec Bad |")
    log("|---------------|:--:|:-------:|:----------------:|")
    for fm in fitness_models:
        key = ("V7 Standard (15% dual-res)", fm)
        m = baseline_results[key]
        log(f"| {fm:25s} | {m['M3']:.2f} | {m['M10_avg']:.1f} | {m['M10_max']} |")

    # ============================================================
    # Summary Comparison Table
    # ============================================================
    log(f"\n{'='*60}")
    log("COMPARISON: CPA vs Surge+Floor")
    log(f"{'='*60}")

    log("\n| Config | M3 CPA | M3 S+F | Delta | M10 CPA | M10 S+F |")
    log("|--------|:------:|:------:|:-----:|:-------:|:-------:|")
    for fm in fitness_models:
        cpa_key = ("40% Enriched (compensated)", fm, "committed")
        sf_v7_key = ("V7 Standard (15% dual-res)", fm)
        sf_40_key = ("40% Enriched (compensated)", fm)

        cpa_m3 = all_results[cpa_key]["M3"]
        sf_m3 = baseline_results[sf_v7_key]["M3"]
        delta = cpa_m3 - sf_m3

        cpa_m10 = all_results[cpa_key]["M10_avg"]
        sf_m10 = baseline_results[sf_v7_key]["M10_avg"]

        log(f"| {fm:25s} | {cpa_m3:.2f} | {sf_m3:.2f} | {delta:+.2f} | "
            f"{cpa_m10:.1f} | {sf_m10:.1f} |")

    return "\n".join(output_lines)


if __name__ == "__main__":
    output = run_full_simulation()
    # Write output to file as well
    with open("/Users/dthurn/Documents/GoogleDrive/dreamtides/docs/resonance/v8/sim_8_output.txt", "w") as f:
        f.write(output)
    print("\nSimulation complete. Output also saved to sim_8_output.txt")
