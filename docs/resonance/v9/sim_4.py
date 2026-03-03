"""
Simulation Agent 4: Hybrid A - Visible-First Anchor Gravity
Combines Design 4 (Layered Salience) + Design 6 (Anchor-Scaled Contraction).

Algorithm specification (from critic_review.md Section 8):
- Stage 1 (visible): R1 filtering gates 3 slots to the committed primary
  resonance pool. One slot always random (splash window).
- Stage 2 (hidden): Within R1-filtered slots, 4x weighting for home-tagged
  cards (archetype_tag matches inferred player archetype).
- Anchor-scaled contraction: pool contraction rate 6%/10%/18% by pick type
  (generic/single-symbol/dual-resonance), from Design 6.
- Contraction relevance: 60% visible dot-product + 40% hidden tag match.
  Applies from pick 5 (earlier than Design 4's pick 12).
- Floor slot from pick 3: one slot draws from top-quartile of surviving pool.
- 3-bit archetype tag per card.

Archetype inference:
- Primary resonance commitment: once player has >= R1_THRESHOLD signal in
  any resonance, that resonance is committed (Stage 1 activates).
- Within-resonance archetype inference: mode of archetype_tag on drafted
  cards that share the committed primary resonance.
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
ARCH_IDX_BY_NAME = {a[0]: i for i, a in enumerate(ARCHETYPES)}

# Map: primary resonance -> list of archetype indices with that primary
RES_TO_ARCHS = defaultdict(list)
for idx, (name, r1, r2) in enumerate(ARCHETYPES):
    RES_TO_ARCHS[r1].append(idx)

# Pool config: 10% visible dual-res per archetype
DUAL_PER_ARCH = 4    # 4/40 = 10% dual per archetype
SINGLE_PER_ARCH = CARDS_PER_ARCHETYPE - DUAL_PER_ARCH  # 36 single

# Contraction rates by pick type (Design 6)
CONTRACTION_GENERIC = 0.06
CONTRACTION_SINGLE  = 0.10
CONTRACTION_DUAL    = 0.18

# Minimum pool floor
MIN_POOL_SIZE = 25

# Primary resonance R1 commitment threshold (total signal accumulated)
R1_COMMIT_THRESHOLD = 4.0  # Require 4 points: e.g. 2 single picks (2x2=4)

# 4x home-tag weighting within R1 slots (Design 4 Stage 2)
HOME_TAG_WEIGHT = 4.0

# Contraction relevance split (Design 6)
VIS_WEIGHT = 0.60
HIDDEN_WEIGHT = 0.40

# Contraction starts at pick 5 (Hybrid A: earlier than Design 4's pick 12)
CONTRACTION_START = 5

# Floor slot from pick 3
FLOOR_START = 3

# ============================================================
# Sibling lookup
# ============================================================
def get_sibling(arch_name):
    """Return co-primary sibling archetype name."""
    r1 = ARCH_BY_NAME[arch_name][1]
    for a in ARCHETYPES:
        if a[0] != arch_name and a[1] == r1:
            return a[0]
    return None

# ============================================================
# Fitness Models (Graduated Realistic = primary, Pessimistic = secondary)
# ============================================================
def make_fitness(rates_by_pair):
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
# Card Model
# ============================================================
@dataclass
class SimCard:
    id: int
    visible_symbols: list      # what player sees (0-2 resonance symbols)
    archetype: str             # primary archetype (evaluation only)
    archetype_tag: int         # hidden 3-bit tag (index into ARCHETYPES; -1=generic)
    power: float
    is_generic: bool = False

# ============================================================
# Pool Construction
# ============================================================
def build_pool():
    """
    Build 360-card pool:
    - 8 archetypes x 40 cards = 320 archetype cards
      - 4 dual-resonance (10%) per archetype
      - 36 single-resonance (90%) per archetype
    - 40 generic cards (no visible resonance)
    Hidden: each card tagged with its primary archetype index (3 bits)
    """
    cards = []
    card_id = 0

    for arch_idx, (arch_name, r1, r2) in enumerate(ARCHETYPES):
        # Dual-resonance signpost cards (slightly above average power per design spec)
        for _ in range(DUAL_PER_ARCH):
            cards.append(SimCard(
                id=card_id,
                visible_symbols=[r1, r2],
                archetype=arch_name,
                archetype_tag=arch_idx,
                power=random.uniform(5.0, 8.5),
            ))
            card_id += 1

        # Single-resonance cards
        for _ in range(SINGLE_PER_ARCH):
            cards.append(SimCard(
                id=card_id,
                visible_symbols=[r1],
                archetype=arch_name,
                archetype_tag=arch_idx,
                power=random.uniform(4.0, 8.0),
            ))
            card_id += 1

    # Generic cards
    for _ in range(GENERIC_CARDS):
        cards.append(SimCard(
            id=card_id,
            visible_symbols=[],
            archetype="Generic",
            archetype_tag=-1,
            power=random.uniform(3.0, 7.0),
            is_generic=True,
        ))
        card_id += 1

    return cards


def precompute_card_tiers(pool, player_archetype, fitness_model):
    """
    Pre-roll S/A status for all cards relative to player's archetype.
    S-tier: cards from player's own archetype.
    A-tier: cards from co-primary sibling, at probability = fitness rate.
    F-tier: everything else.
    """
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
# Hybrid A Algorithm Support Functions
# ============================================================

def get_committed_resonance(signature):
    """
    Return the primary resonance the player has committed to, or None.
    Committed when any resonance accumulates >= R1_COMMIT_THRESHOLD signal.
    """
    best_res = max(RESONANCE_TYPES, key=lambda r: signature[r])
    if signature[best_res] >= R1_COMMIT_THRESHOLD:
        return best_res
    return None


def infer_archetype_from_drafts(drafted_cards, committed_res):
    """
    Infer player's specific archetype (within their committed resonance)
    using the mode of hidden archetype_tag on drafted cards.

    Only considers cards whose primary resonance matches committed_res.
    Returns archetype index, or -1 if insufficient signal.
    """
    if committed_res is None:
        return -1

    # Count tag votes from cards with matching primary resonance
    tag_counts = defaultdict(int)
    for c in drafted_cards:
        if not c.is_generic and c.visible_symbols:
            if c.visible_symbols[0] == committed_res:
                if c.archetype_tag >= 0:
                    tag_counts[c.archetype_tag] += 1

    if not tag_counts:
        return -1

    # Need at least 3 cards to infer (from Design 4 spec: inference at pick 5)
    total = sum(tag_counts.values())
    if total < 3:
        return -1

    return max(tag_counts, key=tag_counts.get)


def compute_visible_relevance(card, signature, sig_magnitude):
    """
    Visible dot-product relevance score (V8 Narrative Gravity formula).
    """
    if card.is_generic:
        return 0.5
    if not card.visible_symbols:
        return 0.0

    card_vec = {r: 0.0 for r in RESONANCE_TYPES}
    for i, sym in enumerate(card.visible_symbols):
        card_vec[sym] += 2.0 if i == 0 else 1.0

    card_mag = math.sqrt(sum(v**2 for v in card_vec.values()))
    if card_mag == 0 or sig_magnitude == 0:
        return 0.5

    dot = sum(card_vec[r] * signature[r] for r in RESONANCE_TYPES)
    return dot / (card_mag * sig_magnitude)


def compute_full_relevance(card, signature, sig_magnitude, inferred_arch_idx):
    """
    Combined relevance for pool contraction:
    60% visible dot-product + 40% hidden tag match.
    """
    vis = compute_visible_relevance(card, signature, sig_magnitude)

    if card.is_generic:
        hidden = 0.5
    elif inferred_arch_idx < 0:
        hidden = 0.5  # neutral before archetype resolved
    else:
        hidden = 1.0 if card.archetype_tag == inferred_arch_idx else 0.0

    combined = VIS_WEIGHT * vis + HIDDEN_WEIGHT * hidden
    # Generics protected at 0.5 floor
    if card.is_generic:
        combined = max(combined, 0.5)
    return combined


def build_pack_hybrid_a(active_pool, committed_res, inferred_arch_idx, pick, use_hidden_weighting):
    """
    Hybrid A pack construction:
    - Pick 1-2: 4 slots from full pool (open draft).
    - Pick 3+: 1 floor slot from top-quartile by power (visible quality floor).
    - If R1 committed: 3 slots from R1-filtered pool + 1 random slot.
      Within R1 slots: 4x home-tag weighting if archetype inferred.
    - If not committed: 4 slots from full pool.
    """
    pack = []
    used_ids = set()

    def draw_from(pool_subset, exclude_ids):
        avail = [c for c in pool_subset if c.id not in exclude_ids]
        if not avail:
            return None
        return random.choice(avail)

    def draw_weighted_r1(r1_candidates, arch_idx, exclude_ids):
        """Draw from R1 pool with 4x home-tag weighting."""
        avail = [c for c in r1_candidates if c.id not in exclude_ids]
        if not avail:
            return None
        if not use_hidden_weighting or arch_idx < 0:
            return random.choice(avail)

        home = [c for c in avail if c.archetype_tag == arch_idx]
        other = [c for c in avail if c.archetype_tag != arch_idx]
        # Build weighted sample list
        weighted = home * int(HOME_TAG_WEIGHT) + other
        if not weighted:
            return random.choice(avail)
        return random.choice(weighted)

    # Floor slot: pick 3+ (top-quartile by power for quality guarantee)
    if pick >= FLOOR_START and len(active_pool) >= PACK_SIZE:
        sorted_by_power = sorted(active_pool, key=lambda c: c.power, reverse=True)
        top_q = sorted_by_power[:max(1, len(sorted_by_power) // 4)]
        floor_card = draw_from(top_q, used_ids)
        if floor_card:
            pack.append(floor_card)
            used_ids.add(floor_card.id)

    slots_needed = PACK_SIZE - len(pack)

    if committed_res is not None:
        # Stage 1: 3 R1-filtered slots (minus what floor already provided)
        r1_pool = [c for c in active_pool
                   if c.visible_symbols and c.visible_symbols[0] == committed_res]

        r1_slots = min(3, slots_needed)
        random_slots = slots_needed - r1_slots

        for _ in range(r1_slots):
            c = draw_weighted_r1(r1_pool, inferred_arch_idx, used_ids)
            if c:
                pack.append(c)
                used_ids.add(c.id)

        # 1 random slot (splash window)
        for _ in range(random_slots):
            c = draw_from(active_pool, used_ids)
            if c:
                pack.append(c)
                used_ids.add(c.id)
    else:
        # No commitment yet: all slots from full pool
        for _ in range(slots_needed):
            c = draw_from(active_pool, used_ids)
            if c:
                pack.append(c)
                used_ids.add(c.id)

    # Pad if still short
    while len(pack) < PACK_SIZE:
        c = draw_from(active_pool, used_ids)
        if c is None:
            break
        pack.append(c)
        used_ids.add(c.id)

    return pack[:PACK_SIZE]


# ============================================================
# Main Draft Simulation
# ============================================================
def hybrid_a_draft(pool, player_archetype, fitness_model, strategy,
                   use_hidden_weighting=True):
    """
    Run one draft using Hybrid A: Visible-First Anchor Gravity.
    use_hidden_weighting=False: V1 measurement mode (visible-only)
    """
    active_pool = list(pool)
    signature = {r: 0.0 for r in RESONANCE_TYPES}
    drafted = []
    history = []

    sa_cache = precompute_card_tiers(pool, player_archetype, fitness_model)

    for pick in range(1, NUM_PICKS + 1):
        if len(active_pool) < PACK_SIZE:
            break

        # Determine committed resonance (visible layer - Stage 1)
        committed_res = get_committed_resonance(signature)

        # Infer specific archetype (hidden layer - Stage 2)
        if use_hidden_weighting:
            inferred_arch_idx = infer_archetype_from_drafts(drafted, committed_res)
        else:
            inferred_arch_idx = -1

        # Build pack
        pack = build_pack_hybrid_a(
            active_pool, committed_res, inferred_arch_idx,
            pick, use_hidden_weighting
        )

        # Select card based on player strategy
        chosen = select_card(pack, player_archetype, signature, strategy, pick, sa_cache)

        drafted.append(chosen)

        # Update resonance signature from visible symbols
        for i, sym in enumerate(chosen.visible_symbols):
            signature[sym] += 2.0 if i == 0 else 1.0

        sa_count = sum(1 for c in pack if sa_cache.get(c.id, False))

        history.append({
            "pick": pick,
            "pack": pack,
            "chosen": chosen,
            "pool_size": len(active_pool),
            "sa_count": sa_count,
            "committed_res": committed_res,
            "inferred_arch": inferred_arch_idx,
        })

        # Anchor-scaled pool contraction (from pick CONTRACTION_START)
        if pick >= CONTRACTION_START and len(active_pool) > MIN_POOL_SIZE:
            n_symbols = len(chosen.visible_symbols)
            if n_symbols == 0:
                rate = CONTRACTION_GENERIC
            elif n_symbols == 1:
                rate = CONTRACTION_SINGLE
            else:
                rate = CONTRACTION_DUAL

            sig_magnitude = math.sqrt(sum(v**2 for v in signature.values()))

            scored = []
            for c in active_pool:
                if use_hidden_weighting:
                    rel = compute_full_relevance(c, signature, sig_magnitude, inferred_arch_idx)
                else:
                    # Visible-only: neutral hidden score
                    vis = compute_visible_relevance(c, signature, sig_magnitude)
                    rel = VIS_WEIGHT * vis + HIDDEN_WEIGHT * 0.5
                    if c.is_generic:
                        rel = max(rel, 0.5)
                scored.append((rel, c))

            scored.sort(key=lambda x: x[0])

            n_remove = max(1, int(len(active_pool) * rate))
            n_remove = min(n_remove, len(active_pool) - MIN_POOL_SIZE)

            if n_remove > 0:
                remove_ids = set(c.id for _, c in scored[:n_remove])
                active_pool = [c for c in active_pool if c.id not in remove_ids]

    return history, drafted, sa_cache


def select_card(pack, player_archetype, signature, strategy, pick, sa_cache):
    """Select a card from the pack based on strategy."""
    arch = ARCH_BY_NAME[player_archetype]
    r1, r2 = arch[1], arch[2]

    if strategy == "committed":
        # Prefer S/A cards, then resonance match, then power
        def score(c):
            s = 0.0
            if sa_cache.get(c.id, False):
                s += 10.0
            for i, sym in enumerate(c.visible_symbols):
                if sym == r1:
                    s += 3.0 if i == 0 else 1.0
                elif sym == r2:
                    s += 2.0 if i == 0 else 1.0
            s += c.power * 0.1
            return s
        return max(pack, key=score)

    elif strategy == "power":
        # Pure power-chaser: ignores resonance
        return max(pack, key=lambda c: c.power)

    elif strategy == "signal":
        # Reads visible resonance signals; picks power early then commits
        if pick <= 3:
            return max(pack, key=lambda c: c.power)
        top_res = max(RESONANCE_TYPES, key=lambda r: signature[r])
        def score(c):
            s = 0.0
            for i, sym in enumerate(c.visible_symbols):
                if sym == top_res:
                    s += 3.0 if i == 0 else 1.0
            s += c.power * 0.1
            if sa_cache.get(c.id, False):
                s += 5.0
            return s
        return max(pack, key=score)

    return random.choice(pack)

# ============================================================
# Metrics Computation
# ============================================================
def compute_metrics(all_histories):
    """Compute M1-M11 from draft histories."""
    m1_vals, m2_vals, m3_vals, m4_vals = [], [], [], []
    m5_vals, m6_vals, m9_vals, m10_vals = [], [], [], []
    m11_vals = []
    post_commit_sa = []
    consec_bad_list = []

    for history, drafted, sa_cache in all_histories:
        # M1: picks 1-5, unique archetypes per pack (target >= 3)
        early_arch = []
        for h in history[:5]:
            archs = set()
            for c in h["pack"]:
                if not c.is_generic:
                    archs.add(c.archetype)
            early_arch.append(len(archs))
        m1_vals.append(sum(early_arch) / max(1, len(early_arch)))

        # M2: picks 1-5, S/A for emerging archetype per pack (target <= 2)
        early_sa = []
        for h in history[:5]:
            sa = sum(1 for c in h["pack"] if sa_cache.get(c.id, False))
            early_sa.append(sa)
        m2_vals.append(sum(early_sa) / max(1, len(early_sa)))

        # M3: picks 6+, S/A per pack (target >= 2.0)
        post_sa = []
        for h in history[5:]:
            sa = h["sa_count"]
            post_sa.append(sa)
            post_commit_sa.append(sa)
        if post_sa:
            m3_vals.append(sum(post_sa) / len(post_sa))

        # M4: picks 6+, off-archetype per pack (target >= 0.5)
        post_off = []
        for h in history[5:]:
            off = sum(1 for c in h["pack"] if not sa_cache.get(c.id, False))
            post_off.append(off)
        if post_off:
            m4_vals.append(sum(post_off) / len(post_off))

        # M5: convergence pick -- first pick where rolling 3-pack avg >= 1.5
        conv_pick = NUM_PICKS
        for i in range(2, len(history)):
            window = [history[j]["sa_count"] for j in range(i-2, i+1)]
            if sum(window) / 3 >= 1.5:
                conv_pick = i + 1
                break
        m5_vals.append(conv_pick)

        # M6: deck archetype concentration (target 60-90%)
        sa_drafted = sum(1 for c in drafted if sa_cache.get(c.id, False))
        m6_vals.append(sa_drafted / max(1, len(drafted)))

        # M9: stddev of S/A per pack (picks 6+, target >= 0.8)
        if len(post_sa) > 1:
            mean_sa = sum(post_sa) / len(post_sa)
            var = sum((x - mean_sa)**2 for x in post_sa) / len(post_sa)
            m9_vals.append(math.sqrt(var))

        # M10: max consecutive packs below 1.5 S/A, picks 6+ (target <= 2)
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

        # M11: picks 15+, S/A per pack (target >= 3.0)
        late_sa = [h["sa_count"] for h in history[14:]]
        if late_sa:
            m11_vals.append(sum(late_sa) / len(late_sa))

    # M7: run-to-run card overlap (target < 40%)
    m7_overlaps = []
    for i in range(1, len(all_histories)):
        ids_prev = set(c.id for c in all_histories[i-1][1])
        ids_curr = set(c.id for c in all_histories[i][1])
        overlap = len(ids_prev & ids_curr) / max(1, len(ids_prev | ids_curr))
        m7_overlaps.append(overlap)

    # M8: archetype frequency (target: no archetype > 20% or < 5%)
    arch_counts = defaultdict(int)
    for _, drafted, _ in all_histories:
        dominant = defaultdict(int)
        for c in drafted:
            if not c.is_generic:
                dominant[c.archetype] += 1
        if dominant:
            arch_counts[max(dominant, key=dominant.get)] += 1
    total_drafts = len(all_histories)

    # Pack quality percentiles (picks 6+)
    pq_sorted = sorted(post_commit_sa)
    n = len(pq_sorted)
    pcts = {}
    for p in [10, 25, 50, 75, 90]:
        idx = min(int(n * p / 100), n - 1)
        pcts[p] = pq_sorted[idx] if n > 0 else 0

    avg = lambda vs: sum(vs) / max(1, len(vs))

    return {
        "M1": avg(m1_vals),
        "M2": avg(m2_vals),
        "M3": avg(m3_vals),
        "M4": avg(m4_vals),
        "M5": avg(m5_vals),
        "M6": avg(m6_vals),
        "M7": avg(m7_overlaps),
        "M9": avg(m9_vals),
        "M10": avg(m10_vals),
        "M10_worst": max(m10_vals) if m10_vals else 0,
        "M11": avg(m11_vals),
        "pack_pcts": pcts,
        "avg_consec_bad": avg(consec_bad_list),
        "worst_consec_bad": max(consec_bad_list) if consec_bad_list else 0,
        "arch_freq": {k: v / total_drafts for k, v in arch_counts.items()},
    }

# ============================================================
# Aggregate and Per-Archetype Runners
# ============================================================
def run_aggregate(fitness_name, strategy, n_drafts=NUM_DRAFTS,
                  use_hidden_weighting=True):
    fitness_model = FITNESS_MODELS[fitness_name]
    all_histories = []
    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        pool = build_pool()
        h, d, cache = hybrid_a_draft(
            pool, arch_name, fitness_model, strategy,
            use_hidden_weighting=use_hidden_weighting
        )
        all_histories.append((h, d, cache))
    return compute_metrics(all_histories), all_histories


def run_per_archetype(fitness_name, strategy, n_per=125,
                      use_hidden_weighting=True):
    fitness_model = FITNESS_MODELS[fitness_name]
    results = {}
    for arch_name in ARCHETYPE_NAMES:
        histories = []
        for _ in range(n_per):
            pool = build_pool()
            h, d, cache = hybrid_a_draft(
                pool, arch_name, fitness_model, strategy,
                use_hidden_weighting=use_hidden_weighting
            )
            histories.append((h, d, cache))
        results[arch_name] = compute_metrics(histories)
    return results


def format_trace(history, drafted, sa_cache, player_archetype, n_picks=20):
    """Format a draft trace."""
    lines = [f"=== Draft Trace: {player_archetype} ==="]
    for h in history[:n_picks]:
        pick = h["pick"]
        sa = h["sa_count"]
        pool_sz = h["pool_size"]
        chosen = h["chosen"]
        chosen_sa = "S/A" if sa_cache.get(chosen.id, False) else "C/F"
        sym_str = "/".join(chosen.visible_symbols) if chosen.visible_symbols else "Generic"
        cres = h.get("committed_res") or "none"
        iarch = h.get("inferred_arch", -1)
        iarch_str = ARCHETYPES[iarch][0] if iarch >= 0 else "none"
        lines.append(
            f"  P{pick:02d} pool={pool_sz:3d} res={cres:<7} arch={iarch_str:<12} "
            f"SA={sa} -> [{chosen.archetype}:{sym_str}] ({chosen_sa})"
        )
    sa_d = sum(1 for c in drafted if sa_cache.get(c.id, False))
    lines.append(f"  Final: {sa_d}/{len(drafted)} S/A = {sa_d/max(1,len(drafted))*100:.0f}%")
    return "\n".join(lines)

# ============================================================
# Main
# ============================================================
def main():
    random.seed(42)

    print("=" * 80)
    print("HYBRID A: VISIBLE-FIRST ANCHOR GRAVITY")
    print("Design 4 (Layered Salience) + Design 6 (Anchor-Scaled Contraction)")
    print("1000 drafts x 30 picks x 3 strategies | Graduated Realistic (primary)")
    print("Pool: 360 cards | 10% dual-res | 3-bit archetype tag | 1,080 bits hidden")
    print("=" * 80)

    # =====================================================
    # Primary: Graduated Realistic, all 3 strategies
    # =====================================================
    print("\n" + "=" * 60)
    print("SECTION 1: Graduated Realistic -- All Strategies")
    print("=" * 60)

    strategies = ["committed", "power", "signal"]
    grad_results = {}

    for strategy in strategies:
        metrics, _ = run_aggregate("Graduated", strategy)
        grad_results[strategy] = metrics
        m = metrics
        print(f"\n  Strategy: {strategy}")
        print(f"    M1={m['M1']:.2f}  M2={m['M2']:.2f}  M3={m['M3']:.2f}  "
              f"M4={m['M4']:.2f}  M5={m['M5']:.1f}")
        print(f"    M6={m['M6']:.2f}  M7={m['M7']:.3f}  M9={m['M9']:.2f}  "
              f"M10={m['M10']:.2f}  M11={m['M11']:.2f}")
        print(f"    M10_worst={m['M10_worst']}  "
              f"worst_consec_bad={m['worst_consec_bad']}")

    # =====================================================
    # Pessimistic (robustness check)
    # =====================================================
    print("\n" + "=" * 60)
    print("SECTION 2: Pessimistic Fitness (robustness check)")
    print("=" * 60)

    pess_metrics, _ = run_aggregate("Pessimistic", "committed")
    m = pess_metrics
    print(f"\n  Pessimistic / committed:")
    print(f"    M3={m['M3']:.2f}  M10={m['M10']:.2f}  "
          f"M11={m['M11']:.2f}  M6={m['M6']:.2f}")

    # =====================================================
    # Per-Archetype M3
    # =====================================================
    print("\n" + "=" * 60)
    print("SECTION 3: Per-Archetype M3 (Graduated Realistic, committed)")
    print("=" * 60)

    per_arch = run_per_archetype("Graduated", "committed")
    print(f"\n  {'Archetype':<16} {'M3':>6} {'M10':>6} {'M11':>6} {'M6':>6}")
    print(f"  {'-'*16} {'-'*6} {'-'*6} {'-'*6} {'-'*6}")
    worst_m3 = 10.0
    worst_arch = ""
    for arch in ARCHETYPE_NAMES:
        m = per_arch[arch]
        mark = " <--" if m['M3'] < worst_m3 else ""
        if m['M3'] < worst_m3:
            worst_m3 = m['M3']
            worst_arch = arch
        print(f"  {arch:<16} {m['M3']:6.2f} {m['M10']:6.2f} {m['M11']:6.2f} {m['M6']:6.2f}")
    print(f"\n  Worst: {worst_arch} (M3={worst_m3:.2f})")

    # =====================================================
    # V1 Measurement: visible-only vs full algorithm
    # =====================================================
    print("\n" + "=" * 60)
    print("SECTION 4: V1 -- Visible Symbol Influence")
    print("=" * 60)

    vis_metrics, _ = run_aggregate("Graduated", "committed", use_hidden_weighting=False)
    m3_full   = grad_results["committed"]["M3"]
    m3_vis    = vis_metrics["M3"]
    m3_random = 0.5

    denom = m3_full - m3_random
    v1_pct = (m3_vis - m3_random) / max(0.001, denom) * 100

    print(f"\n  M3 (full Hybrid A):       {m3_full:.3f}")
    print(f"  M3 (visible-only):        {m3_vis:.3f}")
    print(f"  M3 (random baseline):     {m3_random:.3f}")
    print(f"  V1 = ({m3_vis:.3f} - {m3_random}) / ({m3_full:.3f} - {m3_random})"
          f" = {v1_pct:.1f}%")
    print(f"  V2 = 3 bits/card  (1,080 bits total for 360-card pool)")
    print(f"  V3 = 8/10 (archetype tags derived from card mechanics)")

    # =====================================================
    # V4: Power-chaser gap (visible resonance salience)
    # =====================================================
    print("\n" + "=" * 60)
    print("SECTION 5: V4 -- Visible Resonance Salience")
    print("=" * 60)

    m3_committed = grad_results["committed"]["M3"]
    m3_power     = grad_results["power"]["M3"]
    m3_signal    = grad_results["signal"]["M3"]
    gap_vs_power = m3_committed - m3_power

    print(f"\n  M3 committed player:   {m3_committed:.3f}")
    print(f"  M3 signal reader:      {m3_signal:.3f}")
    print(f"  M3 power-chaser:       {m3_power:.3f}")
    print(f"  Committed - power gap: {gap_vs_power:.3f}")
    print(f"  V4 gap >= 0.4: {'PASS' if gap_vs_power >= 0.4 else 'FAIL'}")

    # =====================================================
    # Pack Quality Distribution
    # =====================================================
    print("\n" + "=" * 60)
    print("SECTION 6: Pack Quality Distribution (picks 6+, committed)")
    print("=" * 60)

    pq = grad_results["committed"]["pack_pcts"]
    m = grad_results["committed"]
    print(f"\n  Graduated Realistic:")
    print(f"    P10={pq[10]}  P25={pq[25]}  P50={pq[50]}  "
          f"P75={pq[75]}  P90={pq[90]}")
    print(f"    Avg consec bad: {m['avg_consec_bad']:.2f}")
    print(f"    Worst consec bad: {m['worst_consec_bad']}")

    # =====================================================
    # V8 Baseline Comparison
    # =====================================================
    print("\n" + "=" * 60)
    print("SECTION 7: Comparison to V8 Baselines")
    print("=" * 60)

    cm = grad_results["committed"]
    print(f"\n  {'Algorithm':<35} {'M3':>5} {'M10':>5} {'M11':>5} {'M6':>5}")
    print(f"  {'-'*35} {'-'*5} {'-'*5} {'-'*5} {'-'*5}")
    print(f"  {'Hybrid A (this, 10% dual-res)':<35} "
          f"{cm['M3']:5.2f} {cm['M10']:5.2f} {cm['M11']:5.2f} {cm['M6']:5.2f}")
    print(f"  {'V8 Narrative Gravity (40% dual)':<35}  2.75  3.30   n/a  0.65")
    print(f"  {'V8 SF+Bias R1 (V7 15% pool)':<35}  2.24  8.00   n/a  0.70")
    print(f"  {'V8 CSCT (V7 pool)':<35}  2.92  2.00   n/a  0.99")

    # =====================================================
    # M8: Archetype Frequency
    # =====================================================
    print("\n" + "=" * 60)
    print("SECTION 8: Archetype Frequency M8 (target: 5-20%)")
    print("=" * 60)

    arch_freq = grad_results["committed"]["arch_freq"]
    print(f"\n  {'Archetype':<16} {'Freq%':>6} {'M8':>6}")
    for arch in ARCHETYPE_NAMES:
        freq = arch_freq.get(arch, 0.0) * 100
        passes = 5.0 <= freq <= 20.0
        print(f"  {arch:<16} {freq:6.1f}%  {'PASS' if passes else 'FAIL'}")

    # =====================================================
    # Draft Traces
    # =====================================================
    print("\n" + "=" * 60)
    print("SECTION 9: Draft Traces")
    print("=" * 60)

    random.seed(100)
    pool = build_pool()
    h1, d1, c1 = hybrid_a_draft(
        pool, "Warriors", FITNESS_MODELS["Graduated"], "committed"
    )
    print("\n" + format_trace(h1, d1, c1, "Warriors"))

    random.seed(200)
    pool = build_pool()
    h2, d2, c2 = hybrid_a_draft(
        pool, "Flash", FITNESS_MODELS["Graduated"], "signal"
    )
    print("\n" + format_trace(h2, d2, c2, "Flash"))

    # =====================================================
    # Full Scorecard
    # =====================================================
    print("\n" + "=" * 80)
    print("FULL SCORECARD: Hybrid A, Graduated Realistic, committed strategy")
    print("=" * 80)

    m = grad_results["committed"]
    targets = [
        ("M1",  m["M1"],  ">=3",      m["M1"] >= 3.0),
        ("M2",  m["M2"],  "<=2",      m["M2"] <= 2.0),
        ("M3",  m["M3"],  ">=2.0",    m["M3"] >= 2.0),
        ("M4",  m["M4"],  ">=0.5",    m["M4"] >= 0.5),
        ("M5",  m["M5"],  "5-8",      5 <= m["M5"] <= 8),
        ("M6",  m["M6"],  "0.60-0.90",0.60 <= m["M6"] <= 0.90),
        ("M7",  m["M7"],  "<0.40",    m["M7"] < 0.40),
        ("M9",  m["M9"],  ">=0.8",    m["M9"] >= 0.8),
        ("M10", m["M10"], "<=2",      m["M10"] <= 2.0),
        ("M11", m["M11"], ">=3.0",    m["M11"] >= 3.0),
    ]

    print(f"\n  {'Metric':<6} {'Value':>8} {'Target':<12} Status")
    print(f"  {'-'*6} {'-'*8} {'-'*12} {'-'*6}")
    n_pass = 0
    for name, val, target, passes in targets:
        status = "PASS" if passes else "FAIL"
        if passes:
            n_pass += 1
        print(f"  {name:<6} {val:8.3f} {target:<12} {status}")

    print(f"\n  {n_pass}/{len(targets)} metrics pass")
    print(f"\n  V1 = {v1_pct:.1f}%  (visible layer contribution to M3 gain)")
    print(f"  V2 = 3 bits/card  (1,080 bits total)")
    print(f"  V3 = 8/10  (archetype tags = mechanically-derived)")
    print(f"  V4 gap = {gap_vs_power:.3f}  (committed vs power-chaser M3)")

    print(f"\n  Pessimistic M3={pess_metrics['M3']:.2f}  M10={pess_metrics['M10']:.2f}  "
          f"M11={pess_metrics['M11']:.2f}")

    print("\nDone.")


if __name__ == "__main__":
    main()
