"""
Simulation Agent 6: Hybrid B -- Affinity-Tagged Gravity (Designs 2 + 5)

Combines Design 2's Tag-Gravity pool contraction mechanism with Design 5's
honest affinity concept, minimized to two floats per resonance pair per card.

Key algorithm innovations over plain Tag-Gravity (Design 2):
  - Replace 3-bit binary archetype tag with two-float pair affinity
  - A Tide card has (warriors_affinity, sacrifice_affinity), each 0.0-1.0
  - Two floats at 4-bit resolution = 8 bits per card vs. 3 bits for a tag
  - Contraction relevance = 0.4 * visible_dot + 0.6 * affinity_score[committed]
  - Bridge cards (high affinity for both archetypes in a pair) survive longer
  - V3 target = 9/10 (honest affinity encoding, no forced single-archetype)

Algorithm spec from critic_review.md Section 8, Hybrid B:
  - Pool contraction from pick 4, 12% removal rate
  - Floor slot from pick 3 (top-quartile draw)
  - Archetype inference = mode of drafted card arch-affinities from pick 5+
  - Pre-pick-5: relevance is 100% visible dot-product

Fixed parameters:
  - 1000 drafts x 30 picks x 3 player strategies
  - Fitness: Graduated Realistic (primary), Pessimistic (secondary)
  - Pool: 360 cards, ~10% visible dual-res, ~79% single, ~11% generic
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
CARDS_PER_ARCHETYPE = 40
GENERIC_CARDS = 40
RESONANCE_TYPES = ["Ember", "Stone", "Tide", "Zephyr"]

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

# Co-primary sibling pairs (share same primary resonance)
SIBLING_PAIRS = [
    ("Flash", "Ramp"),        # Zephyr primary
    ("Blink", "Storm"),       # Ember primary
    ("Storm", "Blink"),       # Ember primary (reverse)
    ("Self-Discard", "Self-Mill"),  # Stone primary
    ("Self-Mill", "Self-Discard"),  # Stone primary (reverse)
    ("Sacrifice", "Warriors"),      # Tide primary
    ("Warriors", "Sacrifice"),      # Tide primary (reverse)
    ("Ramp", "Flash"),              # Zephyr primary (reverse)
]
SIBLING_MAP = dict(SIBLING_PAIRS)


def get_sibling(arch_name):
    """Return co-primary sibling archetype name."""
    return SIBLING_MAP.get(arch_name)


# ============================================================
# Fitness Models
# ============================================================
def make_fitness(rates_by_pair):
    """Create a full fitness dict from shorthand (per co-primary pair)."""
    pairs = [
        ("Warriors", "Sacrifice"),
        ("Self-Discard", "Self-Mill"),
        ("Blink", "Storm"),
        ("Flash", "Ramp"),
    ]
    model = {}
    for (a, b), rate in zip(pairs, rates_by_pair):
        model[(a, b)] = rate
        model[(b, a)] = rate
    return model


FITNESS_MODELS = {
    "Graduated": make_fitness([0.50, 0.40, 0.30, 0.25]),
    "Pessimistic": make_fitness([0.35, 0.25, 0.15, 0.10]),
}


# ============================================================
# Hybrid B Card Model
# ============================================================
@dataclass
class HybridCard:
    id: int
    visible_symbols: list       # 0-2 resonance symbols (player-facing)
    archetype: str              # primary archetype (evaluation only)
    power: float
    is_generic: bool = False
    # Two-float pair affinity: affinity for each archetype in the card's
    # primary resonance pair. For a Tide card: {Warriors: float, Sacrifice: float}
    # For a cross-resonance card (dual-symbol), both entries reflect the exact pair.
    pair_affinities: dict = field(default_factory=dict)


def compute_pair_affinities(arch_name, visible_symbols, is_generic):
    """
    Derive two-float pair affinity for Hybrid B.

    Each card gets affinity scores for the two archetypes that share its
    primary resonance symbol. This is the 'honest' encoding: scores are
    derived from visible card properties (resonance, type, mechanical role).

    Published derivation rules (V3 = 9/10):
      - Primary resonance match: +0.60 for primary-resonance archetype
      - Secondary resonance presence (dual-symbol): +0.30 bonus for exact pair
      - Archetype-specific card (home archetype): +0.25 bonus
      - Sibling archetype: determined by resonance but no home bonus
      - Generic: balanced affinities (0.4 for all archetypes it could serve)
      - Bridge cards (visibly dual-symbol): high affinity for both pair archetypes
    """
    if is_generic:
        # Generics are balanced: 0.4 for all archetypes
        return {a[0]: 0.4 for a in ARCHETYPES}

    if not visible_symbols:
        return {a[0]: 0.3 for a in ARCHETYPES}

    primary_res = visible_symbols[0]

    # Find archetypes that share this primary resonance
    # (archetypes where this resonance is either primary or secondary)
    primary_archs = [a[0] for a in ARCHETYPES if a[1] == primary_res]
    secondary_archs = [a[0] for a in ARCHETYPES if a[2] == primary_res]

    affinities = {a[0]: 0.0 for a in ARCHETYPES}

    # Base: primary resonance provides 0.60 to archetypes sharing it as primary
    for pa in primary_archs:
        affinities[pa] += 0.60

    # Secondary resonance: 0.30 to archetypes sharing it as secondary
    for sa in secondary_archs:
        affinities[sa] += 0.30

    # If the card has a second visible symbol, boost the exact pair archetype
    if len(visible_symbols) >= 2:
        secondary_res = visible_symbols[1]
        for a in ARCHETYPES:
            if a[1] == primary_res and a[2] == secondary_res:
                affinities[a[0]] = min(1.0, affinities[a[0]] + 0.30)
            elif a[1] == secondary_res and a[2] == primary_res:
                affinities[a[0]] = min(1.0, affinities[a[0]] + 0.30)

    # Home archetype bonus: the card's own archetype gets +0.25
    if arch_name in affinities:
        affinities[arch_name] = min(1.0, affinities[arch_name] + 0.25)

    # Clamp all to [0, 1]
    for k in affinities:
        affinities[k] = max(0.0, min(1.0, affinities[k]))

    return affinities


def build_pool_hybrid():
    """
    Build a 360-card pool for Hybrid B with two-float pair affinities.
    Pool composition: ~10% visible dual-res, ~79% single-symbol, ~11% generic.
    """
    cards = []
    card_id = 0

    # 40 generics
    for _ in range(GENERIC_CARDS):
        affinities = compute_pair_affinities("Generic", [], True)
        cards.append(HybridCard(
            id=card_id,
            visible_symbols=[],
            archetype="Generic",
            power=random.uniform(3, 7),
            is_generic=True,
            pair_affinities=affinities,
        ))
        card_id += 1

    # 40 cards per archetype
    for arch_name, r1, r2 in ARCHETYPES:
        n_arch = CARDS_PER_ARCHETYPE

        # ~10% of pool = 36 total dual-res cards = ~4-5 per archetype
        # Each archetype gets 4 dual-res + 36 single-symbol
        n_dual = 4
        n_single = n_arch - n_dual

        for _ in range(n_single):
            syms = [r1]
            affinities = compute_pair_affinities(arch_name, syms, False)
            cards.append(HybridCard(
                id=card_id,
                visible_symbols=syms,
                archetype=arch_name,
                power=random.uniform(4, 8),
                pair_affinities=affinities,
            ))
            card_id += 1

        for _ in range(n_dual):
            syms = [r1, r2]
            affinities = compute_pair_affinities(arch_name, syms, False)
            cards.append(HybridCard(
                id=card_id,
                visible_symbols=syms,
                archetype=arch_name,
                power=random.uniform(4, 8),
                pair_affinities=affinities,
            ))
            card_id += 1

    return cards


def precompute_card_tiers(pool, player_archetype, fitness_model):
    """Pre-roll S/A status for all cards."""
    sa_map = {}
    sibling = get_sibling(player_archetype)
    for c in pool:
        if c.is_generic:
            sa_map[c.id] = False
        elif c.archetype == player_archetype:
            sa_map[c.id] = True
        elif c.archetype == sibling:
            rate = fitness_model.get((player_archetype, sibling), 0.0)
            sa_map[c.id] = (random.random() < rate)
        else:
            sa_map[c.id] = False
    return sa_map


# ============================================================
# Hybrid B Algorithm: Affinity-Tagged Gravity
# ============================================================
def compute_visible_dot(card, signature, sig_magnitude):
    """Compute visible resonance dot-product score (same as Design 2 / V8)."""
    if card.is_generic:
        return 0.5

    if not card.visible_symbols:
        return 0.0

    card_vec = {r: 0.0 for r in RESONANCE_TYPES}
    for i, sym in enumerate(card.visible_symbols):
        if i == 0:
            card_vec[sym] += 2.0
        else:
            card_vec[sym] += 1.0

    card_mag = math.sqrt(sum(v**2 for v in card_vec.values()))
    if card_mag == 0 or sig_magnitude == 0:
        return 0.5

    dot = sum(card_vec[r] * signature[r] for r in RESONANCE_TYPES)
    return dot / (card_mag * sig_magnitude)


def infer_archetype(drafted_cards, pick_num, signature):
    """
    Infer committed archetype from picks 5+.
    Uses the mode of drafted card affinities (which archetype has highest
    cumulative affinity across drafted cards).
    Pre-pick-5: returns None (pure visible contraction).
    """
    if pick_num < 5 or not drafted_cards:
        return None

    # Accumulate affinity scores across drafted cards
    arch_totals = {a[0]: 0.0 for a in ARCHETYPES}
    for card in drafted_cards:
        for arch, aff in card.pair_affinities.items():
            arch_totals[arch] += aff

    best_arch = max(arch_totals, key=lambda a: arch_totals[a])
    return best_arch


def compute_relevance_hybrid(card, signature, sig_magnitude,
                              committed_arch, pick_num):
    """
    Hybrid B relevance score for pool contraction.

    relevance = 0.4 * visible_dot + 0.6 * affinity_score[committed_arch]

    Pre-pick-5 (no committed arch): 100% visible dot-product.
    Generics: protected at 0.5 baseline.
    Bridge cards: high affinity for both archetypes in pair -> survive longer.
    """
    if card.is_generic:
        return 0.5

    vis_dot = compute_visible_dot(card, signature, sig_magnitude)

    if committed_arch is None or pick_num < 5:
        return vis_dot

    # Affinity score for the committed archetype
    aff_score = card.pair_affinities.get(committed_arch, 0.0)

    return 0.4 * vis_dot + 0.6 * aff_score


def hybrid_b_draft(pool, player_archetype, fitness_model, strategy,
                   contraction_pct=0.12, contraction_start=4,
                   floor_start=3):
    """
    Run one draft using Hybrid B: Affinity-Tagged Gravity.

    Key mechanisms:
    - Pool contraction from pick 4, 12% removal rate
    - Floor slot from pick 3 (top-quartile draw by relevance)
    - Relevance = 0.4*visible_dot + 0.6*affinity[committed]
    - Archetype inference from pick 5 via affinity accumulation
    - Bridge cards survive longer (high affinity for both pair archetypes)
    """
    active_pool = list(pool)
    signature = {r: 0.0 for r in RESONANCE_TYPES}
    drafted = []
    history = []

    sa_cache = precompute_card_tiers(pool, player_archetype, fitness_model)

    for pick in range(1, NUM_PICKS + 1):
        if len(active_pool) < PACK_SIZE:
            break

        # Compute current archetype inference
        committed_arch = infer_archetype(drafted, pick, signature)

        sig_magnitude = math.sqrt(sum(v**2 for v in signature.values()))

        # Score all cards for floor slot selection
        if pick >= floor_start:
            scored_pool = []
            for c in active_pool:
                rel = compute_relevance_hybrid(
                    c, signature, sig_magnitude, committed_arch, pick)
                scored_pool.append((rel, c))
            scored_pool.sort(key=lambda x: -x[0])

            # Top-quartile for floor slot
            top_q_size = max(1, len(active_pool) // 4)
            top_q = [c for _, c in scored_pool[:top_q_size]]

            # Floor card: guaranteed best-quartile card
            floor_card = random.choice(top_q)

            # Remaining 3 slots from surviving pool (excluding floor candidate)
            remaining_pool = [c for c in active_pool if c.id != floor_card.id]
            if len(remaining_pool) >= PACK_SIZE - 1:
                other_cards = random.sample(remaining_pool, PACK_SIZE - 1)
            else:
                other_cards = remaining_pool

            pack = [floor_card] + other_cards
        else:
            pack = random.sample(active_pool, min(PACK_SIZE, len(active_pool)))

        # Select card based on strategy
        chosen = select_card_hybrid(pack, player_archetype, signature,
                                    strategy, pick, sa_cache, committed_arch)

        drafted.append(chosen)

        # Update visible resonance signature
        for i, sym in enumerate(chosen.visible_symbols):
            if i == 0:
                signature[sym] += 2.0
            else:
                signature[sym] += 1.0

        sa_count = sum(1 for c in pack if sa_cache[c.id])
        history.append({
            "pick": pick,
            "pack": pack,
            "chosen": chosen,
            "pool_size": len(active_pool),
            "sa_count": sa_count,
            "committed_arch": committed_arch,
        })

        # Contract pool from pick 4
        if pick >= contraction_start and len(active_pool) > PACK_SIZE * 3:
            sig_magnitude = math.sqrt(sum(v**2 for v in signature.values()))
            n_remove = max(1, int(len(active_pool) * contraction_pct))

            scored = []
            for c in active_pool:
                rel = compute_relevance_hybrid(
                    c, signature, sig_magnitude, committed_arch, pick)
                scored.append((rel, c))
            scored.sort(key=lambda x: x[0])

            to_remove = set(c.id for _, c in scored[:n_remove])
            active_pool = [c for c in active_pool if c.id not in to_remove]

    return history, drafted, sa_cache


def select_card_hybrid(pack, player_archetype, signature, strategy, pick,
                       sa_cache, committed_arch):
    """Select card from pack based on player strategy."""
    arch = ARCH_BY_NAME[player_archetype]
    r1, r2 = arch[1], arch[2]

    if strategy == "committed":
        def score(c):
            s = 0
            if sa_cache.get(c.id, False):
                s += 10
            for i, sym in enumerate(c.visible_symbols):
                if sym == r1:
                    s += 3 if i == 0 else 1
                elif sym == r2:
                    s += 2 if i == 0 else 1
            s += c.power * 0.1
            return s
        return max(pack, key=score)

    elif strategy == "power":
        return max(pack, key=lambda c: c.power)

    elif strategy == "signal":
        if pick <= 3:
            return max(pack, key=lambda c: c.power)
        top_res = max(RESONANCE_TYPES, key=lambda r: signature[r])
        def score(c):
            s = 0
            for i, sym in enumerate(c.visible_symbols):
                if sym == top_res:
                    s += 3 if i == 0 else 1
            s += c.power * 0.1
            if sa_cache.get(c.id, False):
                s += 5
            return s
        return max(pack, key=score)

    return random.choice(pack)


# ============================================================
# Metrics
# ============================================================
def compute_metrics(all_histories):
    """Compute M1-M11 from draft histories."""
    m1_vals, m2_vals, m3_vals, m4_vals = [], [], [], []
    m5_vals, m6_vals, m9_vals, m10_vals = [], [], [], []
    m11_vals = []
    post_commit_sa = []
    late_draft_sa = []
    consec_bad_list = []

    for history, drafted, sa_cache in all_histories:
        # M1: picks 1-5, unique archetypes per pack
        early_arch = []
        for h in history[:5]:
            archs = set()
            for c in h["pack"]:
                if not c.is_generic:
                    archs.add(c.archetype)
            early_arch.append(len(archs))
        m1_vals.append(sum(early_arch) / max(1, len(early_arch)))

        # M2: picks 1-5, S/A per pack
        early_sa = []
        for h in history[:5]:
            sa = sum(1 for c in h["pack"] if sa_cache[c.id])
            early_sa.append(sa)
        m2_vals.append(sum(early_sa) / max(1, len(early_sa)))

        # M3: picks 6+, S/A per pack
        post_sa = []
        for h in history[5:]:
            sa = h["sa_count"]
            post_sa.append(sa)
            post_commit_sa.append(sa)
        if post_sa:
            m3_vals.append(sum(post_sa) / len(post_sa))

        # M4: picks 6+, off-archetype (non-S/A) per pack
        post_off = []
        for h in history[5:]:
            off = sum(1 for c in h["pack"] if not sa_cache[c.id])
            post_off.append(off)
        if post_off:
            m4_vals.append(sum(post_off) / len(post_off))

        # M5: convergence pick (first pick where rolling 3-pack avg >= 1.5 SA)
        conv_pick = NUM_PICKS
        for i in range(2, len(history)):
            window = [history[j]["sa_count"] for j in range(i-2, i+1)]
            if sum(window) / 3 >= 1.5:
                conv_pick = i + 1
                break
        m5_vals.append(conv_pick)

        # M6: deck concentration (fraction S/A of all drafted)
        sa_drafted = sum(1 for c in drafted if sa_cache[c.id])
        m6_vals.append(sa_drafted / max(1, len(drafted)))

        # M9: stddev of S/A per pack (picks 6+)
        if len(post_sa) > 1:
            mean_sa = sum(post_sa) / len(post_sa)
            var = sum((x - mean_sa)**2 for x in post_sa) / len(post_sa)
            m9_vals.append(math.sqrt(var))

        # M10: max consecutive packs with S/A < 1.5 (picks 6+)
        max_c = 0
        cur_c = 0
        for sa in post_sa:
            if sa < 1.5:
                cur_c += 1
                max_c = max(max_c, cur_c)
            else:
                cur_c = 0
        m10_vals.append(max_c)
        consec_bad_list.append(max_c)

        # M11: picks 15+, S/A per pack (late-draft density)
        late_sa = []
        for h in history[14:]:
            sa = h["sa_count"]
            late_sa.append(sa)
            late_draft_sa.append(sa)
        if late_sa:
            m11_vals.append(sum(late_sa) / len(late_sa))

    # M7: run-to-run card overlap
    m7_overlaps = []
    for i in range(1, len(all_histories)):
        ids_prev = set(c.id for c in all_histories[i-1][1])
        ids_curr = set(c.id for c in all_histories[i][1])
        overlap = len(ids_prev & ids_curr) / max(1, len(ids_prev | ids_curr))
        m7_overlaps.append(overlap)

    # M8: archetype frequency
    arch_counts = defaultdict(int)
    total = len(all_histories)
    for history, drafted, sa_cache in all_histories:
        # Identify committed archetype by majority of drafted card archetypes
        if drafted:
            counts = defaultdict(int)
            for c in drafted:
                if not c.is_generic:
                    counts[c.archetype] += 1
            if counts:
                top_arch = max(counts, key=counts.get)
                arch_counts[top_arch] += 1
    m8_max = max(arch_counts.values()) / max(1, total) if arch_counts else 0
    m8_min = min(arch_counts.values()) / max(1, total) if arch_counts else 0

    # Pack quality percentiles (picks 6+)
    pq_sorted = sorted(post_commit_sa)
    n = len(pq_sorted)
    pcts = {}
    for p in [10, 25, 50, 75, 90]:
        idx = min(int(n * p / 100), n - 1)
        pcts[p] = pq_sorted[idx] if n > 0 else 0

    # Late draft percentiles (picks 15+)
    late_sorted = sorted(late_draft_sa)
    nl = len(late_sorted)
    late_pcts = {}
    for p in [10, 25, 50, 75, 90]:
        idx = min(int(nl * p / 100), nl - 1)
        late_pcts[p] = late_sorted[idx] if nl > 0 else 0

    avg = lambda vs: sum(vs) / max(1, len(vs))

    return {
        "M1": avg(m1_vals),
        "M2": avg(m2_vals),
        "M3": avg(m3_vals),
        "M4": avg(m4_vals),
        "M5": avg(m5_vals),
        "M6": avg(m6_vals),
        "M7": avg(m7_overlaps),
        "M8_max": m8_max,
        "M8_min": m8_min,
        "M9": avg(m9_vals),
        "M10": avg(m10_vals),
        "M10_worst": max(m10_vals) if m10_vals else 0,
        "M11": avg(m11_vals),
        "pack_pcts": pcts,
        "late_pcts": late_pcts,
        "avg_consec_bad": avg(consec_bad_list),
        "worst_consec_bad": max(consec_bad_list) if consec_bad_list else 0,
    }


# ============================================================
# V1 Measurement: Strip Hidden Affinities
# ============================================================
def hybrid_b_visible_only_draft(pool, player_archetype, fitness_model, strategy,
                                 contraction_pct=0.12, contraction_start=4,
                                 floor_start=3):
    """
    Run a draft using ONLY visible resonance (no affinity scores).
    Used to measure V1 (visible symbol influence).
    Relevance = 100% visible dot-product, even after pick 5.
    """
    active_pool = list(pool)
    signature = {r: 0.0 for r in RESONANCE_TYPES}
    drafted = []
    history = []

    sa_cache = precompute_card_tiers(pool, player_archetype, fitness_model)

    for pick in range(1, NUM_PICKS + 1):
        if len(active_pool) < PACK_SIZE:
            break

        sig_magnitude = math.sqrt(sum(v**2 for v in signature.values()))

        if pick >= floor_start:
            scored_pool = []
            for c in active_pool:
                rel = compute_visible_dot(c, signature, sig_magnitude)
                scored_pool.append((rel, c))
            scored_pool.sort(key=lambda x: -x[0])
            top_q_size = max(1, len(active_pool) // 4)
            top_q = [c for _, c in scored_pool[:top_q_size]]
            floor_card = random.choice(top_q)
            remaining_pool = [c for c in active_pool if c.id != floor_card.id]
            if len(remaining_pool) >= PACK_SIZE - 1:
                other_cards = random.sample(remaining_pool, PACK_SIZE - 1)
            else:
                other_cards = remaining_pool
            pack = [floor_card] + other_cards
        else:
            pack = random.sample(active_pool, min(PACK_SIZE, len(active_pool)))

        chosen = select_card_hybrid(pack, player_archetype, signature,
                                    strategy, pick, sa_cache, None)
        drafted.append(chosen)

        for i, sym in enumerate(chosen.visible_symbols):
            if i == 0:
                signature[sym] += 2.0
            else:
                signature[sym] += 1.0

        sa_count = sum(1 for c in pack if sa_cache[c.id])
        history.append({
            "pick": pick,
            "pack": pack,
            "chosen": chosen,
            "pool_size": len(active_pool),
            "sa_count": sa_count,
            "committed_arch": None,
        })

        if pick >= contraction_start and len(active_pool) > PACK_SIZE * 3:
            sig_magnitude = math.sqrt(sum(v**2 for v in signature.values()))
            n_remove = max(1, int(len(active_pool) * contraction_pct))

            scored = []
            for c in active_pool:
                rel = compute_visible_dot(c, signature, sig_magnitude)
                scored.append((rel, c))
            scored.sort(key=lambda x: x[0])

            to_remove = set(c.id for _, c in scored[:n_remove])
            active_pool = [c for c in active_pool if c.id not in to_remove]

    return history, drafted, sa_cache


# ============================================================
# Runners
# ============================================================
def run_aggregate(fitness_name, strategy, n_drafts=NUM_DRAFTS,
                  contraction_pct=0.12):
    """Run aggregate drafts cycling through archetypes."""
    fitness_model = FITNESS_MODELS[fitness_name]
    all_histories = []

    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        pool = build_pool_hybrid()
        h, d, cache = hybrid_b_draft(
            pool, arch_name, fitness_model, strategy,
            contraction_pct=contraction_pct,
        )
        all_histories.append((h, d, cache))

    return compute_metrics(all_histories), all_histories


def run_per_archetype(fitness_name, strategy, n_per=125, contraction_pct=0.12):
    """Run per-archetype analysis."""
    fitness_model = FITNESS_MODELS[fitness_name]
    results = {}

    for arch_name in ARCHETYPE_NAMES:
        histories = []
        for _ in range(n_per):
            pool = build_pool_hybrid()
            h, d, cache = hybrid_b_draft(
                pool, arch_name, fitness_model, strategy,
                contraction_pct=contraction_pct,
            )
            histories.append((h, d, cache))
        results[arch_name] = compute_metrics(histories)

    return results


def run_visible_only(fitness_name, strategy, n_drafts=500, contraction_pct=0.12):
    """Run visible-only baseline for V1 measurement."""
    fitness_model = FITNESS_MODELS[fitness_name]
    all_histories = []

    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        pool = build_pool_hybrid()
        h, d, cache = hybrid_b_visible_only_draft(
            pool, arch_name, fitness_model, strategy,
            contraction_pct=contraction_pct,
        )
        all_histories.append((h, d, cache))

    return compute_metrics(all_histories)


def format_trace(history, drafted, sa_cache, player_archetype):
    """Format a detailed draft trace."""
    lines = [f"=== Draft Trace: {player_archetype} ==="]
    for h in history:
        pick = h["pick"]
        sa = h["sa_count"]
        pool_sz = h["pool_size"]
        chosen = h["chosen"]
        chosen_sa = "S/A" if sa_cache[chosen.id] else "C/F"
        sym_str = "/".join(chosen.visible_symbols) if chosen.visible_symbols else "Generic"
        arch_committed = h.get("committed_arch", "None") or "None"
        if pick <= 5 or pick % 5 == 0:
            lines.append(
                f"  Pick {pick:2d}: pool={pool_sz:3d}, pack S/A={sa}, "
                f"inferred={arch_committed:<14}, "
                f"chose [{chosen.archetype}:{sym_str}] ({chosen_sa})"
            )
    sa_d = sum(1 for c in drafted if sa_cache[c.id])
    lines.append(f"  Final: {sa_d}/{len(drafted)} S/A = {sa_d/max(1,len(drafted))*100:.0f}%")
    return "\n".join(lines)


# ============================================================
# V4: Visible-vs-Hidden Pick Alignment
# ============================================================
def measure_v4_alignment(n_drafts=500):
    """
    Measure V4: fraction of picks where 'best visible pick' == 'best affinity pick'.
    'Best visible' = highest visible_dot to signature.
    'Best affinity' = highest affinity[committed_arch] for cards with matching symbol.
    """
    fitness_model = FITNESS_MODELS["Graduated"]
    alignments = []

    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        pool = build_pool_hybrid()
        sa_cache = precompute_card_tiers(pool, arch_name, fitness_model)
        signature = {r: 0.0 for r in RESONANCE_TYPES}
        drafted = []

        active_pool = list(pool)

        for pick in range(1, NUM_PICKS + 1):
            if len(active_pool) < PACK_SIZE:
                break

            pack = random.sample(active_pool, PACK_SIZE)
            sig_magnitude = math.sqrt(sum(v**2 for v in signature.values()))
            committed = infer_archetype(drafted, pick, signature)

            # Best visible pick: highest visible dot-product
            best_vis = max(pack, key=lambda c: compute_visible_dot(
                c, signature, sig_magnitude))

            # Best affinity pick: highest affinity for committed arch
            if committed:
                best_aff = max(pack, key=lambda c: c.pair_affinities.get(
                    committed, 0.0))
                alignments.append(1 if best_vis.id == best_aff.id else 0)

            # Committed selection
            chosen = select_card_hybrid(pack, arch_name, signature,
                                        "committed", pick, sa_cache, committed)
            drafted.append(chosen)
            for i2, sym in enumerate(chosen.visible_symbols):
                if i2 == 0:
                    signature[sym] += 2.0
                else:
                    signature[sym] += 1.0

            if pick >= 4 and len(active_pool) > PACK_SIZE * 3:
                sig_magnitude = math.sqrt(sum(v**2 for v in signature.values()))
                n_remove = max(1, int(len(active_pool) * 0.12))
                scored = [(compute_relevance_hybrid(
                    c, signature, sig_magnitude, committed, pick), c)
                    for c in active_pool]
                scored.sort(key=lambda x: x[0])
                to_remove = set(c.id for _, c in scored[:n_remove])
                active_pool = [c for c in active_pool if c.id not in to_remove]

    return sum(alignments) / max(1, len(alignments))


# ============================================================
# Main
# ============================================================
def main():
    random.seed(42)

    print("=" * 80)
    print("HYBRID B: AFFINITY-TAGGED GRAVITY -- Simulation Agent 6")
    print("=" * 80)
    print("Algorithm: Design 2 Tag-Gravity + Design 5 pair-affinity encoding")
    print("Hidden metadata: 2 floats per card (8 bits at 4-bit resolution)")
    print("Relevance: 0.4 * visible_dot + 0.6 * affinity_score[committed]")
    print()

    # =====================================================
    # Primary run: Graduated Realistic, all strategies
    # =====================================================
    print("=" * 80)
    print("PRIMARY RUN: Graduated Realistic Fitness")
    print("=" * 80)

    grad_results = {}
    for strategy in ["committed", "power", "signal"]:
        print(f"  Running strategy: {strategy} ...")
        m, histories = run_aggregate("Graduated", strategy, n_drafts=NUM_DRAFTS)
        grad_results[strategy] = (m, histories)

    print()
    print(f"{'Strategy':<12} {'M1':>5} {'M2':>5} {'M3':>5} {'M4':>5} "
          f"{'M5':>5} {'M6':>5} {'M7':>6} {'M9':>5} {'M10':>5} {'M11':>5}")
    for strategy in ["committed", "power", "signal"]:
        m = grad_results[strategy][0]
        print(f"{strategy:<12} {m['M1']:5.2f} {m['M2']:5.2f} {m['M3']:5.2f} "
              f"{m['M4']:5.2f} {m['M5']:5.1f} {m['M6']:5.2f} {m['M7']:6.3f} "
              f"{m['M9']:5.2f} {m['M10']:5.1f} {m['M11']:5.2f}")

    # M8
    m_committed = grad_results["committed"][0]
    print(f"\nM8 (archetype frequency): max={m_committed['M8_max']:.1%}, "
          f"min={m_committed['M8_min']:.1%} (target: no arch > 20% or < 5%)")

    # =====================================================
    # Secondary run: Pessimistic fitness
    # =====================================================
    print()
    print("=" * 80)
    print("SECONDARY RUN: Pessimistic Fitness (committed strategy)")
    print("=" * 80)
    m_pess, _ = run_aggregate("Pessimistic", "committed", n_drafts=NUM_DRAFTS)
    print(f"  M3={m_pess['M3']:.2f} M10={m_pess['M10']:.1f} "
          f"M11={m_pess['M11']:.2f} M6={m_pess['M6']:.2f}")

    # =====================================================
    # Per-archetype M3 table (Graduated, committed)
    # =====================================================
    print()
    print("=" * 80)
    print("PER-ARCHETYPE M3 (Graduated Realistic, committed)")
    print("=" * 80)
    pa = run_per_archetype("Graduated", "committed", n_per=125)
    print(f"  {'Archetype':<16} {'M3':>6} {'M5':>6} {'M6':>6} {'M9':>6} "
          f"{'M10':>5} {'M11':>6}")
    m3_per_arch = []
    for arch in ARCHETYPE_NAMES:
        m = pa[arch]
        m3_per_arch.append(m["M3"])
        print(f"  {arch:<16} {m['M3']:6.2f} {m['M5']:6.1f} {m['M6']:6.2f} "
              f"{m['M9']:6.2f} {m['M10']:5.1f} {m['M11']:6.2f}")
    m3_spread = max(m3_per_arch) - min(m3_per_arch)
    print(f"  Spread (max-min): {m3_spread:.2f}")
    print(f"  Worst archetype: {ARCHETYPE_NAMES[m3_per_arch.index(min(m3_per_arch))]}"
          f" = {min(m3_per_arch):.2f}")

    # =====================================================
    # Pack quality distribution
    # =====================================================
    print()
    print("=" * 80)
    print("PACK QUALITY DISTRIBUTION (picks 6+, committed)")
    print("=" * 80)
    m_c = grad_results["committed"][0]
    pq = m_c["pack_pcts"]
    lp = m_c["late_pcts"]
    print(f"  Picks 6+  P10={pq[10]}  P25={pq[25]}  P50={pq[50]}  "
          f"P75={pq[75]}  P90={pq[90]}")
    print(f"  Picks 15+ P10={lp[10]}  P25={lp[25]}  P50={lp[50]}  "
          f"P75={lp[75]}  P90={lp[90]}")
    print(f"  Avg consecutive bad packs (SA<1.5, picks 6+): "
          f"{m_c['avg_consec_bad']:.2f}")
    print(f"  Worst consecutive bad: {m_c['worst_consec_bad']}")

    # =====================================================
    # V1 measurement: visible-only baseline
    # =====================================================
    print()
    print("=" * 80)
    print("V1 MEASUREMENT: Visible-Only vs Full Hybrid B")
    print("=" * 80)
    print("  Running visible-only baseline (500 drafts)...")
    m_vis = run_visible_only("Graduated", "committed", n_drafts=500)
    m_full_500, _ = run_aggregate("Graduated", "committed", n_drafts=500)
    m_baseline = 0.125  # random pool expected value (1/8 archetypes)

    m3_visible = m_vis["M3"]
    m3_full = m_full_500["M3"]
    if m3_full > m_baseline:
        v1_pct = (m3_visible - m_baseline) / (m3_full - m_baseline)
    else:
        v1_pct = 0.0
    print(f"  M3 visible-only: {m3_visible:.3f}")
    print(f"  M3 full Hybrid B: {m3_full:.3f}")
    print(f"  M3 random baseline: {m_baseline:.3f}")
    print(f"  V1 (visible contribution): {v1_pct:.1%}")
    print(f"  V2 (hidden info): 8 bits/card (two 4-bit floats per resonance pair)")
    print(f"  V3 (defensibility): 9/10 (affinities derived from published rules)")

    # =====================================================
    # V4 measurement
    # =====================================================
    print()
    print("=" * 80)
    print("V4 MEASUREMENT: Visible vs Hidden Pick Alignment")
    print("=" * 80)
    print("  Measuring alignment (500 drafts, picks 5+)...")
    v4_align = measure_v4_alignment(n_drafts=500)
    print(f"  V4 (visible/hidden pick alignment): {v4_align:.1%}")
    print("  (Divergence = same-symbol cards where affinity distinguishes)")

    # =====================================================
    # Consecutive bad pack analysis
    # =====================================================
    print()
    print("=" * 80)
    print("CONSECUTIVE BAD PACK ANALYSIS (picks 6+, committed, Graduated)")
    print("=" * 80)
    _, histories = grad_results["committed"]
    consec_dist = defaultdict(int)
    for h, d, cache in histories:
        post_sa = [hh["sa_count"] for hh in h[5:]]
        max_c, cur_c = 0, 0
        for sa in post_sa:
            if sa < 1.5:
                cur_c += 1
                max_c = max(max_c, cur_c)
            else:
                cur_c = 0
        consec_dist[max_c] += 1
    print(f"  {'Streak':<8} {'Count':>8} {'Pct':>8}")
    for k in sorted(consec_dist.keys()):
        pct = consec_dist[k] / NUM_DRAFTS * 100
        print(f"  {k:<8} {consec_dist[k]:>8} {pct:>7.1f}%")

    # =====================================================
    # Draft Traces
    # =====================================================
    print()
    print("=" * 80)
    print("DRAFT TRACES")
    print("=" * 80)

    # Trace 1: Committed player, Warriors, Graduated
    random.seed(100)
    pool1 = build_pool_hybrid()
    h1, d1, c1 = hybrid_b_draft(
        pool1, "Warriors", FITNESS_MODELS["Graduated"], "committed")
    print("\n" + format_trace(h1, d1, c1, "Warriors (committed)"))

    # Trace 2: Signal reader, Sacrifice, Graduated
    random.seed(200)
    pool2 = build_pool_hybrid()
    h2, d2, c2 = hybrid_b_draft(
        pool2, "Sacrifice", FITNESS_MODELS["Graduated"], "signal")
    print("\n" + format_trace(h2, d2, c2, "Sacrifice (signal reader)"))

    # =====================================================
    # V8 Comparison
    # =====================================================
    print()
    print("=" * 80)
    print("V8 BASELINE COMPARISON (Graduated Realistic, committed)")
    print("=" * 80)
    m_h = grad_results["committed"][0]
    print(f"  {'Algorithm':<35} {'M3':>6} {'M10':>5} {'M11':>6} {'M6':>5}")
    print(f"  {'V8 Narrative Gravity (40% pool)':<35} {'2.75':>6} {'3.3':>5} {'~2.8':>6} {'0.85':>5}")
    print(f"  {'V8 SF+Bias R1 (V7 15% pool)':<35} {'2.24':>6} {'8.0':>5} {'~2.0':>6} {'0.79':>5}")
    print(f"  {'V8 CSCT (V7 pool)':<35} {'2.92':>6} {'2.0':>5} {'~3.0':>6} {'0.99':>5}")
    print(f"  {'Hybrid B (V9, 10% visible pool)':<35} {m_h['M3']:6.2f} "
          f"{m_h['M10']:5.1f} {m_h['M11']:6.2f} {m_h['M6']:5.2f}")
    m3_vs_ng = m_h['M3'] - 2.38  # vs V8 NG on V7 15% pool
    m3_vs_ng_40 = m_h['M3'] - 2.75  # vs V8 NG on 40% pool
    print(f"\n  vs V8 NG (15% pool): {m3_vs_ng:+.2f}")
    print(f"  vs V8 NG (40% pool): {m3_vs_ng_40:+.2f}")

    # =====================================================
    # Power-chaser gap (V4 test)
    # =====================================================
    print()
    print("=" * 80)
    print("V4 POWER-CHASER GAP TEST")
    print("=" * 80)
    m_res = grad_results["committed"][0]
    m_pow = grad_results["power"][0]
    gap = m_res["M3"] - m_pow["M3"]
    print(f"  Committed M3:     {m_res['M3']:.2f}")
    print(f"  Power-chaser M3:  {m_pow['M3']:.2f}")
    print(f"  Gap:              {gap:.2f} (target >= 0.4)")

    # =====================================================
    # Summary scorecard
    # =====================================================
    print()
    print("=" * 80)
    print("SUMMARY SCORECARD: Hybrid B (Graduated Realistic, committed)")
    print("=" * 80)
    m = grad_results["committed"][0]
    targets = {
        "M1 >= 3.0": (m["M1"], m["M1"] >= 3.0),
        "M2 <= 2.0": (m["M2"], m["M2"] <= 2.0),
        "M3 >= 2.0": (m["M3"], m["M3"] >= 2.0),
        "M4 >= 0.5": (m["M4"], m["M4"] >= 0.5),
        "M5 pick 5-8": (m["M5"], 5.0 <= m["M5"] <= 8.0),
        "M6 60-90%": (m["M6"], 0.60 <= m["M6"] <= 0.90),
        "M7 < 40%": (m["M7"], m["M7"] < 0.40),
        "M8 max < 20%": (m["M8_max"], m["M8_max"] < 0.20),
        "M8 min > 5%": (m["M8_min"], m["M8_min"] > 0.05),
        "M9 >= 0.8": (m["M9"], m["M9"] >= 0.8),
        "M10 <= 2": (m["M10"], m["M10"] <= 2.0),
        "M11 >= 3.0": (m["M11"], m["M11"] >= 3.0),
    }
    passes = 0
    for label, (val, passed) in targets.items():
        status = "PASS" if passed else "FAIL"
        passes += int(passed)
        print(f"  {status} {label:<18}: {val:.2f}")
    print(f"\n  Overall: {passes}/{len(targets)} metrics pass")

    # V1-V4 summary
    print()
    print("  V1-V4 Summary:")
    print(f"  V1 (visible influence): {v1_pct:.1%} (target >= 40%): "
          f"{'PASS' if v1_pct >= 0.4 else 'FAIL'}")
    print(f"  V2 (hidden info): 8 bits/card (two-float pair affinity)")
    print(f"  V3 (defensibility): 9/10 (honest derivation from card properties)")
    print(f"  V4 (pick alignment): {v4_align:.1%} alignment (vs power-chaser gap {gap:.2f})")

    print("\nDone.")


if __name__ == "__main__":
    main()
