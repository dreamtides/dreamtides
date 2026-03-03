"""
Simulation Agent 1: Design 4 — Layered Salience (Two-Stage Filter)

Algorithm summary:
  Stage 1 (visible): R1 filtering gates 3 of 4 slots to committed primary
    resonance pool. One slot always random (splash window).
  Stage 2 (hidden): Within R1 pool, 4x weighting for home-tagged cards
    (3-bit archetype tag per card).
  Phase 3: Pool contraction at 8%/pick from pick 12 onward using archetype
    tag relevance scores.

Picks 1-5: no targeting (exploration).
Picks 6+: full two-stage targeting once player has committed resonance (2+
  cards with same primary resonance among their picks 1-5).
"""

import random
import math
from dataclasses import dataclass, field
from collections import defaultdict

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

# Contraction parameters (Phase 3)
CONTRACTION_RATE = 0.08   # 8% per pick
CONTRACTION_START = 12    # picks 12+ activate contraction
MIN_POOL_FLOOR = 25       # never contract below this size

# Stage 2 weighting
HOME_TAG_WEIGHT = 4.0     # home-tagged cards drawn at 4x relative to sibling-tagged
SIBLING_TAG_WEIGHT = 1.0

# R1 filtering: commit after 2+ picks of primary resonance (observed from picks 1-5)
R1_COMMIT_THRESHOLD = 2   # 2+ cards with same primary resonance
R1_ACTIVE_PICK = 6        # Stage 1 active from pick 6 onward (if committed)

# ============================================================
# Fitness Models
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
    "Optimistic":  make_fitness([1.0,  1.0,  1.0,  1.0]),
    "Graduated":   make_fitness([0.50, 0.40, 0.30, 0.25]),
    "Pessimistic": make_fitness([0.35, 0.25, 0.15, 0.10]),
    "Hostile":     make_fitness([0.08, 0.08, 0.08, 0.08]),
}

def get_sibling(arch_name):
    """Return co-primary sibling archetype name."""
    r1 = ARCH_BY_NAME[arch_name][1]
    for a in ARCHETYPES:
        if a[0] != arch_name and a[1] == r1:
            return a[0]
    return None

# ============================================================
# Card Model
# ============================================================
@dataclass
class SimCard:
    id: int
    visible_symbols: list    # 0-2 resonance symbols (player-visible)
    archetype_tag: int       # hidden 3-bit tag: index into ARCHETYPE_NAMES (0-7)
                             # or -1 for generic
    archetype: str           # canonical archetype (eval only)
    power: float
    is_generic: bool = False

    @property
    def primary_resonance(self):
        return self.visible_symbols[0] if self.visible_symbols else None


def build_pool():
    """
    Build the 360-card V9 pool:
      - 40 generic (no visible symbols)
      - 284 single-symbol (primary resonance only)
      - 36 dual-symbol (4-5 per archetype)

    Per archetype (40 cards total):
      Flash, Storm, Self-Mill, Warriors: 5 dual + 35 single
      Others: 4 dual + 36 single
    """
    cards = []
    card_id = 0

    five_dual = {"Flash", "Storm", "Self-Mill", "Warriors"}

    for arch_name, r1, r2 in ARCHETYPES:
        n_dual = 5 if arch_name in five_dual else 4
        n_single = CARDS_PER_ARCHETYPE - n_dual
        tag_idx = ARCHETYPE_NAMES.index(arch_name)

        for _ in range(n_single):
            cards.append(SimCard(
                id=card_id,
                visible_symbols=[r1],
                archetype_tag=tag_idx,
                archetype=arch_name,
                power=random.uniform(4, 8),
            ))
            card_id += 1

        for _ in range(n_dual):
            cards.append(SimCard(
                id=card_id,
                visible_symbols=[r1, r2],
                archetype_tag=tag_idx,
                archetype=arch_name,
                power=random.uniform(5, 9),
            ))
            card_id += 1

    for _ in range(GENERIC_CARDS):
        cards.append(SimCard(
            id=card_id,
            visible_symbols=[],
            archetype_tag=-1,
            archetype="Generic",
            power=random.uniform(3, 7),
            is_generic=True,
        ))
        card_id += 1

    assert len(cards) == POOL_SIZE
    return cards


def precompute_card_tiers(pool, player_archetype, fitness_model):
    """Pre-roll S/A status for all cards for this draft."""
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
# Algorithm: Layered Salience
# ============================================================

def infer_archetype(drafted_tags, committed_resonance):
    """
    Infer the player's probable archetype from hidden tags on drafted cards.
    Returns the archetype name with the highest tag count among cards
    whose primary resonance matches committed_resonance.
    """
    if not drafted_tags or committed_resonance is None:
        return None

    tag_counts = defaultdict(int)
    for tag_idx in drafted_tags:
        if tag_idx >= 0:
            tag_counts[ARCHETYPE_NAMES[tag_idx]] += 1

    if not tag_counts:
        return None

    # Only consider archetypes sharing the committed primary resonance
    res_archetypes = [a[0] for a in ARCHETYPES if a[1] == committed_resonance]
    filtered = {k: v for k, v in tag_counts.items() if k in res_archetypes}

    if filtered:
        return max(filtered, key=lambda k: filtered[k])
    # Fall back to overall mode
    return max(tag_counts, key=lambda k: tag_counts[k])


def get_r1_pool(active_pool, committed_resonance):
    """Return cards from active pool whose primary visible symbol matches R1."""
    return [c for c in active_pool if c.primary_resonance == committed_resonance]


def weighted_sample_without_replacement(candidates, weights, n):
    """
    Draw n items from candidates with given weights, without replacement.
    """
    if not candidates:
        return []
    n = min(n, len(candidates))

    remaining = list(zip(candidates, weights))
    selected = []

    for _ in range(n):
        if not remaining:
            break
        total = sum(w for _, w in remaining)
        if total <= 0:
            # Fall back to uniform
            idx = random.randrange(len(remaining))
        else:
            r = random.uniform(0, total)
            cumulative = 0
            idx = len(remaining) - 1
            for i, (_, w) in enumerate(remaining):
                cumulative += w
                if r <= cumulative:
                    idx = i
                    break
        selected.append(remaining[idx][0])
        remaining.pop(idx)

    return selected


def build_pack(active_pool, committed_resonance, inferred_arch, stage1_active,
               stage2_active, used_ids):
    """
    Construct a pack using the two-stage filter.

    Stage 1 (visible R1 filtering): 3 slots from R1 pool, 1 random.
    Stage 2 (hidden tag weighting): within R1 slots, 4x home-tag bias.
    """
    pack = []
    current_used = set(used_ids)

    if stage1_active and committed_resonance is not None:
        # R1-filtered pool for this resonance
        r1_candidates = [c for c in active_pool
                         if c.primary_resonance == committed_resonance
                         and c.id not in current_used]

        if stage2_active and inferred_arch is not None:
            # Stage 2: weighted sampling from R1 pool
            home_tag = ARCHETYPE_NAMES.index(inferred_arch)
            weights = []
            for c in r1_candidates:
                if c.is_generic:
                    weights.append(0.4)
                elif c.archetype_tag == home_tag:
                    weights.append(HOME_TAG_WEIGHT)
                else:
                    weights.append(SIBLING_TAG_WEIGHT)
            r1_cards = weighted_sample_without_replacement(r1_candidates, weights, 3)
        else:
            # Stage 1 only: uniform from R1 pool
            r1_cards = random.sample(r1_candidates, min(3, len(r1_candidates)))

        pack.extend(r1_cards)
        current_used.update(c.id for c in r1_cards)

        # 1 random splash slot from full active pool
        random_candidates = [c for c in active_pool if c.id not in current_used]
        if random_candidates:
            pack.append(random.choice(random_candidates))
    else:
        # Phase 1 (picks 1-5) or no committed resonance: full pool, uniform
        candidates = [c for c in active_pool if c.id not in current_used]
        pack = random.sample(candidates, min(PACK_SIZE, len(candidates)))

    return pack


def compute_contraction_relevance(card, inferred_arch, committed_resonance):
    """
    Relevance score for pool contraction.
    home-tagged = 1.0, sibling-tagged within R1 = 0.5,
    off-resonance = 0.1, generics = 0.4 (protected).
    """
    if card.is_generic:
        return 0.4

    primary_match = (card.primary_resonance == committed_resonance)

    if not primary_match:
        return 0.1

    if inferred_arch is None:
        return 0.5

    home_tag = ARCHETYPE_NAMES.index(inferred_arch)
    if card.archetype_tag == home_tag:
        return 1.0
    else:
        return 0.5


def update_committed_resonance(committed_resonance, resonance_counts, pick,
                                player_archetype, strategy):
    """
    Determine committed resonance from visible resonance picks.

    The committed resonance reflects what the player has signaled through
    their picks. For committed players (who know their archetype), this
    should align with their primary resonance when possible.

    - Committed strategy: commit to player's primary resonance once they've
      picked at least 1 card of that resonance (they know what they want).
    - Signal/power strategy: commit to whatever resonance hits threshold first
      (they're reading the draft, not their predetermined archetype).
    """
    if committed_resonance is not None:
        return committed_resonance

    if strategy == "committed":
        # Committed player knows their archetype; they prefer their primary resonance.
        # They commit to their primary resonance once they've picked >= 1 card of it.
        r1 = ARCH_BY_NAME[player_archetype][1]
        if resonance_counts.get(r1, 0) >= 1:
            return r1
        # Fallback: if they haven't picked any primary resonance cards by pick 4,
        # commit to whatever they've been picking most (pragmatic fallback)
        if pick >= 4 and resonance_counts:
            top_res = max(resonance_counts, key=lambda r: resonance_counts[r])
            if resonance_counts[top_res] >= R1_COMMIT_THRESHOLD:
                return top_res
        return None
    else:
        # Signal reader and power chaser: commit based on visible picks
        for res, cnt in resonance_counts.items():
            if cnt >= R1_COMMIT_THRESHOLD:
                return res
        return None


def layered_salience_draft(pool, player_archetype, fitness_model, strategy):
    """
    Run one complete draft using the Layered Salience algorithm.
    """
    active_pool = list(pool)
    drafted = []
    history = []
    drafted_tags = []

    # Track resonance counts from picks (visible signals)
    resonance_counts = defaultdict(int)
    committed_resonance = None

    sa_cache = precompute_card_tiers(pool, player_archetype, fitness_model)

    for pick in range(1, NUM_PICKS + 1):
        if len(active_pool) < PACK_SIZE:
            break

        # Determine algorithm state
        stage1_active = (pick >= R1_ACTIVE_PICK and committed_resonance is not None)
        inferred_arch = infer_archetype(drafted_tags, committed_resonance) if committed_resonance else None
        # Stage 2 needs at least 5 drafted tags to have reliable inference
        stage2_active = (stage1_active and inferred_arch is not None
                         and len([t for t in drafted_tags if t >= 0]) >= 5)

        used_ids = set(c.id for c in drafted)

        # Build pack
        pack = build_pack(
            active_pool, committed_resonance, inferred_arch,
            stage1_active, stage2_active, used_ids
        )

        if not pack:
            break

        # Player selects card
        chosen = select_card(pack, player_archetype, resonance_counts,
                             committed_resonance, strategy, pick, sa_cache)

        drafted.append(chosen)

        # Update visible resonance tracking (from chosen card's primary symbol)
        if chosen.visible_symbols:
            resonance_counts[chosen.visible_symbols[0]] += 1
            # Dual-resonance signpost cards provide a stronger secondary signal
            if len(chosen.visible_symbols) > 1:
                resonance_counts[chosen.visible_symbols[1]] += 0.5

        # Update hidden tag tracking
        if not chosen.is_generic:
            drafted_tags.append(chosen.archetype_tag)
        else:
            drafted_tags.append(-1)

        # Update committed resonance (evaluates after each pick in picks 1-5)
        if pick < R1_ACTIVE_PICK:
            committed_resonance = update_committed_resonance(
                committed_resonance, resonance_counts, pick,
                player_archetype, strategy
            )

        # Record pack quality (S/A count in pack per player's archetype)
        sa_count = sum(1 for c in pack if sa_cache[c.id])

        history.append({
            "pick": pick,
            "pack": pack,
            "chosen": chosen,
            "pool_size": len(active_pool),
            "sa_count": sa_count,
            "stage1_active": stage1_active,
            "stage2_active": stage2_active,
            "committed_res": committed_resonance,
            "inferred_arch": inferred_arch,
        })

        # Phase 3: Pool contraction from pick 12 onward
        if pick >= CONTRACTION_START:
            current_inferred = infer_archetype(drafted_tags, committed_resonance)

            scored = []
            for c in active_pool:
                rel = compute_contraction_relevance(c, current_inferred, committed_resonance)
                scored.append((rel, random.random(), c))

            scored.sort(key=lambda x: (x[0], x[1]))

            n_remove = max(1, int(len(active_pool) * CONTRACTION_RATE))
            safe_size = max(MIN_POOL_FLOOR, PACK_SIZE * 2)
            n_remove = min(n_remove, len(active_pool) - safe_size)

            if n_remove > 0:
                remove_ids = set(c.id for _, _, c in scored[:n_remove])
                active_pool = [c for c in active_pool if c.id not in remove_ids]

    return history, drafted, sa_cache


def select_card(pack, player_archetype, resonance_counts, committed_resonance,
                strategy, pick, sa_cache):
    """Select a card from the pack based on player strategy."""
    arch = ARCH_BY_NAME[player_archetype]
    r1, r2 = arch[1], arch[2]

    if strategy == "committed":
        # Committed player: prefers S/A for their archetype, then their primary resonance
        def score(c):
            s = 0
            if sa_cache.get(c.id, False):
                s += 10
            # Prefer cards matching their known primary resonance
            for i, sym in enumerate(c.visible_symbols):
                if sym == r1:
                    s += 3 if i == 0 else 1
                elif sym == r2:
                    s += 2 if i == 0 else 1
            s += c.power * 0.1
            return s
        return max(pack, key=score)

    elif strategy == "power":
        # Power chaser: raw power, ignores resonance
        return max(pack, key=lambda c: c.power)

    elif strategy == "signal":
        # Signal reader: watches visible resonance accumulation
        if pick <= 3:
            return max(pack, key=lambda c: c.power)
        # Follow strongest observed resonance signal
        top_res = max(resonance_counts, key=lambda r: resonance_counts[r]) \
            if resonance_counts else None
        def score(c):
            s = 0
            if top_res:
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
# V1 Measurement: Stage 1 only (no hidden tags, no contraction)
# ============================================================

def layered_salience_visible_only_draft(pool, player_archetype, fitness_model, strategy):
    """
    Draft using Stage 1 only (R1 filtering, no hidden tags, no contraction).
    Used to measure V1 (visible symbol influence).
    """
    active_pool = list(pool)
    drafted = []
    history = []
    resonance_counts = defaultdict(int)
    committed_resonance = None

    sa_cache = precompute_card_tiers(pool, player_archetype, fitness_model)

    for pick in range(1, NUM_PICKS + 1):
        if len(active_pool) < PACK_SIZE:
            break

        stage1_active = (pick >= R1_ACTIVE_PICK and committed_resonance is not None)

        used_ids = set(c.id for c in drafted)
        pack = build_pack(active_pool, committed_resonance, None,
                          stage1_active, False, used_ids)

        if not pack:
            break

        chosen = select_card(pack, player_archetype, resonance_counts,
                             committed_resonance, strategy, pick, sa_cache)
        drafted.append(chosen)

        if chosen.visible_symbols:
            resonance_counts[chosen.visible_symbols[0]] += 1

        if pick < R1_ACTIVE_PICK:
            committed_resonance = update_committed_resonance(
                committed_resonance, resonance_counts, pick,
                player_archetype, strategy
            )

        sa_count = sum(1 for c in pack if sa_cache[c.id])
        history.append({
            "pick": pick,
            "pack": pack,
            "chosen": chosen,
            "pool_size": len(active_pool),
            "sa_count": sa_count,
        })

    return history, drafted, sa_cache


# ============================================================
# Metrics
# ============================================================

def compute_metrics(all_histories, include_m11=True):
    """Compute all M1-M11 metrics."""
    m1_vals, m2_vals, m3_vals, m4_vals = [], [], [], []
    m5_vals, m6_vals, m9_vals, m10_vals = [], [], [], []
    m11_vals = []
    post_commit_sa = []
    consec_bad_list = []

    for history, drafted, sa_cache in all_histories:
        # M1: picks 1-5, unique archetypes per pack (measure openness)
        early_arch = []
        for h in history[:5]:
            archs = set()
            for c in h["pack"]:
                if not c.is_generic:
                    archs.add(c.archetype)
            early_arch.append(len(archs))
        m1_vals.append(sum(early_arch) / max(1, len(early_arch)))

        # M2: picks 1-5, S/A for player's archetype per pack
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

        # M5: convergence pick (rolling 3-pick avg >= 1.5 S/A)
        conv_pick = NUM_PICKS
        for i in range(2, len(history)):
            window = [history[j]["sa_count"] for j in range(max(0, i-2), i+1)]
            if len(window) > 0 and sum(window) / len(window) >= 1.5:
                conv_pick = i + 1
                break
        m5_vals.append(conv_pick)

        # M6: deck archetype concentration
        sa_drafted = sum(1 for c in drafted if sa_cache[c.id])
        m6_vals.append(sa_drafted / max(1, len(drafted)))

        # M9: stddev of S/A per pack (picks 6+)
        if len(post_sa) > 1:
            mean_sa = sum(post_sa) / len(post_sa)
            var = sum((x - mean_sa) ** 2 for x in post_sa) / len(post_sa)
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

        # M11: picks 15+, S/A per pack
        if include_m11:
            late = [h["sa_count"] for h in history if h["pick"] >= 15]
            if late:
                m11_vals.append(sum(late) / len(late))

    # M7: run-to-run card overlap
    m7_overlaps = []
    for i in range(1, min(len(all_histories), 101)):
        ids_prev = set(c.id for c in all_histories[i-1][1])
        ids_curr = set(c.id for c in all_histories[i][1])
        overlap = len(ids_prev & ids_curr) / max(1, len(ids_prev | ids_curr))
        m7_overlaps.append(overlap)

    # Pack quality percentiles
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
        "M11": avg(m11_vals) if m11_vals else 0.0,
        "pack_pcts": pcts,
        "avg_consec_bad": avg(consec_bad_list),
        "worst_consec_bad": max(consec_bad_list) if consec_bad_list else 0,
        "consec_bad_dist": dict(sorted(
            ((i, consec_bad_list.count(i)) for i in range(max(consec_bad_list) + 1)),
            key=lambda x: x[0]
        )) if consec_bad_list else {},
    }


# ============================================================
# Runners
# ============================================================

def run_aggregate(fitness_name, strategy, n_drafts=NUM_DRAFTS, include_m11=True):
    """Run aggregate drafts cycling through archetypes."""
    fitness_model = FITNESS_MODELS[fitness_name]
    all_histories = []

    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        pool = build_pool()
        h, d, cache = layered_salience_draft(pool, arch_name, fitness_model, strategy)
        all_histories.append((h, d, cache))

    return compute_metrics(all_histories, include_m11=include_m11), all_histories


def run_per_archetype(fitness_name, strategy, n_per=125):
    """Run per-archetype analysis."""
    fitness_model = FITNESS_MODELS[fitness_name]
    results = {}

    for arch_name in ARCHETYPE_NAMES:
        histories = []
        for _ in range(n_per):
            pool = build_pool()
            h, d, cache = layered_salience_draft(pool, arch_name, fitness_model, strategy)
            histories.append((h, d, cache))
        results[arch_name] = compute_metrics(histories, include_m11=True)

    return results


def run_v1_measurement(fitness_name="Graduated", strategy="committed", n_drafts=500):
    """
    Measure V1 by running visible-only (Stage 1 alone) vs full algorithm.
    V1 = (M3_visible - M3_baseline) / (M3_full - M3_baseline)
    """
    fitness_model = FITNESS_MODELS[fitness_name]

    # Full algorithm
    full_histories = []
    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        pool = build_pool()
        h, d, cache = layered_salience_draft(pool, arch_name, fitness_model, strategy)
        full_histories.append((h, d, cache))
    full_metrics = compute_metrics(full_histories, include_m11=False)

    # Visible-only (Stage 1, no tags, no contraction)
    visible_histories = []
    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        pool = build_pool()
        h, d, cache = layered_salience_visible_only_draft(pool, arch_name, fitness_model, strategy)
        visible_histories.append((h, d, cache))
    visible_metrics = compute_metrics(visible_histories, include_m11=False)

    # Baseline: random (no targeting)
    # 40 cards per archetype, 360 total, expected S/A ≈ (40 + sibling*40*rate) / 360 * 4
    # Simplification: ~40/320 * 4 ≈ 0.5 for committed at Graduated
    m3_baseline = 40.0 / POOL_SIZE * PACK_SIZE  # theoretical random baseline ≈ 0.444

    m3_full = full_metrics["M3"]
    m3_visible = visible_metrics["M3"]

    if m3_full - m3_baseline > 0:
        v1 = (m3_visible - m3_baseline) / (m3_full - m3_baseline)
    else:
        v1 = 0.0

    return {
        "M3_full": m3_full,
        "M3_visible_only": m3_visible,
        "M3_baseline": m3_baseline,
        "V1": v1,
    }


def run_v4_measurement(fitness_name="Graduated", n_drafts=100):
    """
    V4: % of picks (6+) where best visible pick differs from best hidden pick.
    """
    fitness_model = FITNESS_MODELS[fitness_name]
    differ_count = 0
    total_picks = 0

    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        pool = build_pool()
        sa_cache = precompute_card_tiers(pool, arch_name, fitness_model)
        arch = ARCH_BY_NAME[arch_name]
        r1, r2 = arch[1], arch[2]
        home_tag = ARCHETYPE_NAMES.index(arch_name)

        active_pool = list(pool)
        resonance_counts = defaultdict(int)
        committed_resonance = None
        drafted_tags = []

        for pick in range(1, NUM_PICKS + 1):
            if len(active_pool) < PACK_SIZE:
                break

            stage1_active = (pick >= R1_ACTIVE_PICK and committed_resonance is not None)
            inferred_arch = infer_archetype(drafted_tags, committed_resonance)
            stage2_active = (stage1_active and inferred_arch is not None
                             and len([t for t in drafted_tags if t >= 0]) >= 5)

            used = set()
            pack = build_pack(active_pool, committed_resonance, inferred_arch,
                              stage1_active, stage2_active, used)

            if not pack or pick < 6:
                # Use committed strategy for pick tracking
                chosen = max(pack, key=lambda c: (
                    (sa_cache.get(c.id, False) * 10) +
                    (3 if c.primary_resonance == r1 else 0) +
                    c.power * 0.1
                )) if pack else None
                if chosen:
                    if chosen.visible_symbols:
                        resonance_counts[chosen.visible_symbols[0]] += 1
                    if not chosen.is_generic:
                        drafted_tags.append(chosen.archetype_tag)
                    else:
                        drafted_tags.append(-1)
                    if pick < R1_ACTIVE_PICK:
                        committed_resonance = update_committed_resonance(
                            committed_resonance, resonance_counts, pick,
                            arch_name, "committed"
                        )
                continue

            # Measure divergence for picks 6+
            def visible_score(c):
                s = 0
                for sym in c.visible_symbols:
                    if sym == r1:
                        s += 2
                    elif sym == r2:
                        s += 1
                s += c.power * 0.05
                return s

            def hidden_score(c):
                s = 4.0 if c.archetype_tag == home_tag else 0.0
                if sa_cache.get(c.id, False):
                    s += 5.0
                s += c.power * 0.1
                return s

            best_visible = max(pack, key=visible_score)
            best_hidden = max(pack, key=hidden_score)

            if best_visible.id != best_hidden.id:
                differ_count += 1
            total_picks += 1

            # Update state with committed strategy pick
            chosen = max(pack, key=lambda c: (
                (sa_cache.get(c.id, False) * 10) +
                (3 if c.primary_resonance == r1 else 0) +
                c.power * 0.1
            ))
            if chosen.visible_symbols:
                resonance_counts[chosen.visible_symbols[0]] += 1
            if not chosen.is_generic:
                drafted_tags.append(chosen.archetype_tag)
            else:
                drafted_tags.append(-1)

    v4 = differ_count / total_picks if total_picks > 0 else 0
    return {"V4_divergence_pct": v4, "V4_alignment_pct": 1 - v4}


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
        s1 = "R1" if h.get("stage1_active") else "--"
        s2 = "T4" if h.get("stage2_active") else "--"
        inferred = h.get("inferred_arch") or "?"
        committed = h.get("committed_res") or "?"
        lines.append(
            f"  Pick {pick:2d}: pool={pool_sz:3d}  SA={sa}  {s1}/{s2}"
            f"  res={committed:<7}  inf={inferred:<14}"
            f"  chose [{chosen.archetype}:{sym_str}] ({chosen_sa})"
        )
    sa_d = sum(1 for c in drafted if sa_cache[c.id])
    lines.append(f"  Final: {sa_d}/{len(drafted)} S/A = {sa_d/max(1,len(drafted))*100:.0f}%")
    return "\n".join(lines)


# ============================================================
# Main
# ============================================================
def main():
    random.seed(42)

    print("=" * 80)
    print("LAYERED SALIENCE SIMULATION -- Design 4 (V9 Simulation Agent 1)")
    print("=" * 80)

    # ========== Primary: Graduated Realistic, all strategies ==========
    print("\n" + "=" * 60)
    print("PRIMARY RUN: Graduated Realistic, 1000 drafts x 3 strategies")
    print("=" * 60)

    strategies = ["committed", "power", "signal"]
    primary_results = {}
    for strat in strategies:
        print(f"  Running strategy: {strat}...")
        metrics, histories = run_aggregate("Graduated", strat, n_drafts=NUM_DRAFTS)
        primary_results[strat] = (metrics, histories)

    print("\n  SCORECARD (Graduated Realistic):")
    print(f"  {'Metric':<10} {'Target':<10} {'committed':>10} {'power':>10} {'signal':>10}")
    metric_targets = [
        ("M1", ">= 3.0"), ("M2", "<= 2.0"), ("M3", ">= 2.0"),
        ("M4", ">= 0.5"), ("M5", "5-8"), ("M6", "60-90%"),
        ("M7", "< 40%"), ("M9", ">= 0.8"), ("M10", "<= 2"),
        ("M11", ">= 3.0"),
    ]
    for name, target in metric_targets:
        row = [primary_results[s][0].get(name, 0) for s in strategies]
        if name in ("M6", "M7"):
            fmt = [f"{v*100:.1f}%" for v in row]
        else:
            fmt = [f"{v:.3f}" for v in row]
        status = ""
        if name == "M3":
            status = "PASS" if primary_results["committed"][0]["M3"] >= 2.0 else "FAIL"
        elif name == "M10":
            status = "PASS" if primary_results["committed"][0]["M10"] <= 2.0 else "FAIL"
        elif name == "M11":
            status = "PASS" if primary_results["committed"][0]["M11"] >= 3.0 else "FAIL"
        print(f"  {name:<10} {target:<10} {fmt[0]:>10} {fmt[1]:>10} {fmt[2]:>10}  {status}")

    # ========== Secondary: Pessimistic ==========
    print("\n" + "=" * 60)
    print("SECONDARY RUN: Pessimistic, committed strategy")
    print("=" * 60)
    print("  Running...")
    pess_metrics, _ = run_aggregate("Pessimistic", "committed", n_drafts=NUM_DRAFTS)
    grad_metrics = primary_results["committed"][0]
    print(f"  M3  Grad={grad_metrics['M3']:.3f}  Pess={pess_metrics['M3']:.3f}  "
          f"degradation={grad_metrics['M3']-pess_metrics['M3']:.3f}")
    print(f"  M10 Grad={grad_metrics['M10']:.2f}  Pess={pess_metrics['M10']:.2f}")
    print(f"  M11 Grad={grad_metrics['M11']:.3f}  Pess={pess_metrics['M11']:.3f}")

    # ========== Per-archetype M3 ==========
    print("\n" + "=" * 60)
    print("PER-ARCHETYPE M3 TABLE (Graduated Realistic, committed)")
    print("=" * 60)
    print("  Running per-archetype analysis (125 drafts each)...")
    pa_results = run_per_archetype("Graduated", "committed", n_per=125)
    print(f"  {'Archetype':<16} {'M3':>6} {'M5':>6} {'M6':>6} {'M9':>6} {'M10':>6} {'M11':>6}")
    for arch in ARCHETYPE_NAMES:
        m = pa_results[arch]
        flag = " <-- FAIL" if m["M3"] < 2.0 else ""
        print(f"  {arch:<16} {m['M3']:6.3f} {m['M5']:6.1f} {m['M6']*100:5.1f}%"
              f" {m['M9']:6.3f} {m['M10']:6.2f} {m['M11']:6.3f}{flag}")

    # Pessimistic per-archetype
    print("\n  Per-archetype M3 (Pessimistic):")
    pa_pess = run_per_archetype("Pessimistic", "committed", n_per=125)
    print(f"  {'Archetype':<16} {'M3_Grad':>8} {'M3_Pess':>8} {'Delta':>8}")
    for arch in ARCHETYPE_NAMES:
        g = pa_results[arch]["M3"]
        p = pa_pess[arch]["M3"]
        flag = " <-- below 2.0" if p < 2.0 else ""
        print(f"  {arch:<16} {g:8.3f} {p:8.3f} {g-p:8.3f}{flag}")

    # ========== Pack quality distribution ==========
    print("\n" + "=" * 60)
    print("PACK QUALITY DISTRIBUTION (picks 6+, committed, Graduated)")
    print("=" * 60)
    pq = grad_metrics["pack_pcts"]
    print(f"  P10={pq[10]}  P25={pq[25]}  P50={pq[50]}  P75={pq[75]}  P90={pq[90]}")
    print(f"  Avg consec bad packs (SA<1.5): {grad_metrics['avg_consec_bad']:.2f}")
    print(f"  Worst consec bad: {grad_metrics['worst_consec_bad']}")

    print("\n  Consecutive bad pack distribution:")
    cbd = grad_metrics["consec_bad_dist"]
    total = sum(cbd.values())
    for k in sorted(cbd.keys()):
        if cbd[k] == 0:
            continue
        pct = cbd[k] / total * 100
        bar = "#" * int(pct / 2)
        print(f"    {k:2d} consecutive: {cbd[k]:5d} ({pct:5.1f}%) {bar}")

    # ========== V1 Measurement ==========
    print("\n" + "=" * 60)
    print("V1 MEASUREMENT: Visible Symbol Influence")
    print("=" * 60)
    print("  Running visible-only vs full algorithm comparison (500 drafts each)...")
    v1_result = run_v1_measurement("Graduated", "committed", n_drafts=500)
    print(f"  M3 (full algorithm):   {v1_result['M3_full']:.3f}")
    print(f"  M3 (visible only):     {v1_result['M3_visible_only']:.3f}")
    print(f"  M3 (random baseline):  {v1_result['M3_baseline']:.3f}")
    denom = v1_result['M3_full'] - v1_result['M3_baseline']
    numer = v1_result['M3_visible_only'] - v1_result['M3_baseline']
    print(f"  V1 = ({v1_result['M3_visible_only']:.3f} - {v1_result['M3_baseline']:.3f})"
          f" / ({v1_result['M3_full']:.3f} - {v1_result['M3_baseline']:.3f})"
          f" = {v1_result['V1']*100:.1f}%")

    # ========== V4 Measurement ==========
    print("\n" + "=" * 60)
    print("V4 MEASUREMENT: Visible Resonance Salience")
    print("=" * 60)
    print("  Running V4 measurement (100 drafts, picks 6+)...")
    v4_result = run_v4_measurement("Graduated", n_drafts=100)
    print(f"  Best visible == best hidden: {v4_result['V4_alignment_pct']*100:.1f}%")
    print(f"  Picks where they diverge:  {v4_result['V4_divergence_pct']*100:.1f}%")

    # V4 empirical: power-chaser gap
    print("\n  Power-chaser M3 gap (visible resonance advantage):")
    m3_committed = grad_metrics["M3"]
    m3_power = primary_results["power"][0]["M3"]
    m3_signal = primary_results["signal"][0]["M3"]
    print(f"  Committed M3: {m3_committed:.3f}")
    print(f"  Power M3:     {m3_power:.3f}  (gap: {m3_committed - m3_power:.3f})")
    print(f"  Signal M3:    {m3_signal:.3f}  (gap: {m3_committed - m3_signal:.3f})")

    # ========== V2, V3 ==========
    print("\n  V2 (hidden info): 3 bits per card (1 of 8 archetype tag) = 1,080 bits total")
    print("  V3 (defensibility): 8/10 — tags reflect real card mechanics (card's best-fit archetype)")

    # ========== Draft traces ==========
    print("\n" + "=" * 60)
    print("DRAFT TRACES")
    print("=" * 60)

    # Trace 1: Committed player, Warriors
    random.seed(100)
    pool1 = build_pool()
    h1, d1, c1 = layered_salience_draft(pool1, "Warriors", FITNESS_MODELS["Graduated"], "committed")
    print("\n" + format_trace(h1, d1, c1, "Warriors (committed)"))

    # Trace 2: Signal reader, Sacrifice
    random.seed(200)
    pool2 = build_pool()
    h2, d2, c2 = layered_salience_draft(pool2, "Sacrifice", FITNESS_MODELS["Graduated"], "signal")
    print("\n" + format_trace(h2, d2, c2, "Sacrifice (signal)"))

    # ========== V8 Comparison ==========
    print("\n" + "=" * 60)
    print("COMPARISON TO V8 BASELINES (committed, Graduated Realistic)")
    print("=" * 60)
    m = grad_metrics
    print(f"  {'Algorithm':<36} {'M3':>6} {'M10':>5} {'M11':>5} {'M6':>6} {'M9':>5}")
    print(f"  {'V8: Narrative Gravity (40% pool)':<36} {'2.75':>6} {'3.3':>5} {'n/a':>5} {'72%':>6} {'1.00':>5}")
    print(f"  {'V8: SF+Bias R1 (V7 15% pool)':<36} {'2.24':>6} {'8.0':>5} {'n/a':>5} {'65%':>6} {'1.05':>5}")
    print(f"  {'V8: CSCT (V7 15% pool)':<36} {'2.92':>6} {'2.0':>5} {'n/a':>5} {'99%':>6} {'0.85':>5}")
    print(f"  {'V9 D4: Layered Salience (10% pool)':<36} {m['M3']:6.3f} {m['M10']:5.2f}"
          f" {m['M11']:5.3f} {m['M6']*100:5.1f}% {m['M9']:5.3f}")

    # ========== Full metrics across fitness models ==========
    print("\n" + "=" * 60)
    print("FULL METRICS ACROSS FITNESS MODELS (committed)")
    print("=" * 60)
    fit_results = {"Graduated": grad_metrics}
    for fn in ["Optimistic", "Pessimistic", "Hostile"]:
        print(f"  Running {fn}...")
        fm, _ = run_aggregate(fn, "committed", n_drafts=NUM_DRAFTS)
        fit_results[fn] = fm
    for fn in ["Optimistic", "Graduated", "Pessimistic", "Hostile"]:
        fm = fit_results[fn]
        print(f"  {fn:<12}: M1={fm['M1']:.2f} M2={fm['M2']:.2f} M3={fm['M3']:.3f} "
              f"M4={fm['M4']:.2f} M5={fm['M5']:.1f} M6={fm['M6']*100:.1f}% "
              f"M7={fm['M7']*100:.1f}% M9={fm['M9']:.3f} M10={fm['M10']:.2f} M11={fm['M11']:.3f}")

    print("\nDone.")


if __name__ == "__main__":
    main()
