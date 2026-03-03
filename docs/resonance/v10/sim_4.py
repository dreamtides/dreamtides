"""
Simulation Agent 4: Hybrid Y -- Escalating Open Table (D1 + D4)

Algorithm specification (from critic_review.md Section 4, Hybrid Y):
- 5 AI drafters randomly selected from 8 archetypes (3 lanes open)
- Escalating picks per AI per round:
    Picks 1-5:   2 cards/AI/round
    Picks 6-10:  3 cards/AI/round
    Picks 11-15: 4 cards/AI/round
    Picks 16+:   5 cards/AI/round
- 85% archetype focus throughout, 15% off-archetype power picks
- Market culling: 8 lowest-power cards removed per round
- Level 0 reactivity (fully predetermined)

Pack construction: After AI picks and market culling, the pack is drawn from
the surviving pool using V9-equivalent weighted sampling:
- Pre-commitment: random 4-card pack (picks 1 until commitment)
- Post-commitment: 3 slots weighted by pair_affinity for inferred archetype
  from surviving pool, 1 random splash slot
- Floor slot: from pick 3+, one slot draws from top-quartile power

This reproduces V9 Hybrid B's convergence mechanism on top of the AI drafter
pool. The design documents confirm this is the intended approach: "V10 is V9's
contraction engine with an AI drafter narrative layer on top."
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
NUM_AI_DRAFTERS = 5
MARKET_CULL_COUNT = 8
ARCHETYPE_FOCUS = 0.85
POOL_FLOOR = 30
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

ESCALATION_SCHEDULE = {1: 2, 2: 3, 3: 4, 4: 5}

DUAL_PER_ARCH = 4
SINGLE_PER_ARCH = CARDS_PER_ARCHETYPE - DUAL_PER_ARCH

# Commitment
COMMIT_THRESHOLD = 4.0  # resonance signal needed

# ============================================================
# Sibling Lookup & Fitness Models
# ============================================================
def get_sibling(arch_name):
    r1 = ARCH_BY_NAME[arch_name][1]
    for a in ARCHETYPES:
        if a[0] != arch_name and a[1] == r1:
            return a[0]
    return None


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
    visible_symbols: list
    archetype: str
    power: float
    pair_affinity: dict
    is_generic: bool = False

# ============================================================
# Pool Construction
# ============================================================
def build_pool():
    cards = []
    card_id = 0

    for arch_idx, (arch_name, r1, r2) in enumerate(ARCHETYPES):
        sibling = get_sibling(arch_name)
        for _ in range(DUAL_PER_ARCH):
            affinities = _make_affinities(arch_name, sibling)
            cards.append(SimCard(
                id=card_id, visible_symbols=[r1, r2],
                archetype=arch_name, power=random.uniform(5.0, 8.5),
                pair_affinity=affinities,
            ))
            card_id += 1
        for _ in range(SINGLE_PER_ARCH):
            affinities = _make_affinities(arch_name, sibling)
            cards.append(SimCard(
                id=card_id, visible_symbols=[r1],
                archetype=arch_name, power=random.uniform(4.0, 8.0),
                pair_affinity=affinities,
            ))
            card_id += 1

    for _ in range(GENERIC_CARDS):
        affinities = {a: random.uniform(0.1, 0.4) for a in ARCHETYPE_NAMES}
        cards.append(SimCard(
            id=card_id, visible_symbols=[],
            archetype="Generic", power=random.uniform(3.0, 7.0),
            pair_affinity=affinities, is_generic=True,
        ))
        card_id += 1

    return cards


def _make_affinities(home_arch, sibling):
    home_r1 = ARCH_BY_NAME[home_arch][1]
    affinities = {}
    for a_name in ARCHETYPE_NAMES:
        if a_name == home_arch:
            affinities[a_name] = random.uniform(0.7, 1.0)
        elif a_name == sibling:
            affinities[a_name] = random.uniform(0.3, 0.6)
        else:
            a_r1 = ARCH_BY_NAME[a_name][1]
            a_r2 = ARCH_BY_NAME[a_name][2]
            if a_r1 == home_r1 or a_r2 == home_r1:
                affinities[a_name] = random.uniform(0.15, 0.35)
            else:
                affinities[a_name] = random.uniform(0.0, 0.15)
    return affinities


# ============================================================
# S/A Tier Precomputation
# ============================================================
def precompute_card_tiers(pool, player_archetype, fitness_model):
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
# AI Drafter Model
# ============================================================
class AIDrafter:
    def __init__(self, archetype_name):
        self.archetype = archetype_name
        self.drafted = []

    def get_picks_for_phase(self, pick_num):
        if pick_num <= 5:   return ESCALATION_SCHEDULE[1]
        elif pick_num <= 10: return ESCALATION_SCHEDULE[2]
        elif pick_num <= 15: return ESCALATION_SCHEDULE[3]
        else:                return ESCALATION_SCHEDULE[4]

    def score_card(self, card):
        if card.is_generic:
            return 0.3 * card.power / 10.0
        return card.pair_affinity.get(self.archetype, 0.0)

    def pick_cards(self, pool_dict, pick_num):
        n_picks = self.get_picks_for_phase(pick_num)
        pool_size = len(pool_dict)
        if pool_size <= POOL_FLOOR:
            return set()
        max_take = pool_size - POOL_FLOOR
        n_picks = min(n_picks, max_take)
        if n_picks <= 0:
            return set()

        available = list(pool_dict.values())
        picked_ids = set()
        for i in range(n_picks):
            if not available:
                break
            if random.random() < ARCHETYPE_FOCUS:
                best = max(available, key=lambda c: self.score_card(c))
            else:
                best = max(available, key=lambda c: c.power)
            picked_ids.add(best.id)
            self.drafted.append(best)
            available = [c for c in available if c.id != best.id]
        return picked_ids


# ============================================================
# Pack Construction
# ============================================================
def weighted_draw(candidates, weight_fn, used_ids):
    """Draw one card from candidates using weight_fn, excluding used_ids."""
    avail = [c for c in candidates if c.id not in used_ids]
    if not avail:
        return None
    weights = [max(0.01, weight_fn(c)) for c in avail]
    total = sum(weights)
    r = random.random() * total
    cum = 0
    for c, w in zip(avail, weights):
        cum += w
        if cum >= r:
            return c
    return avail[-1]


def build_pack(pool_list, committed_res, inferred_arch, pick, sa_cache):
    """
    Build 4-card pack from surviving pool.

    Pre-commitment: random 4-card pack.
    Post-commitment:
      - 1 floor slot (top-quartile by power, from pick 3+)
      - 2 archetype-weighted slots: weight = pair_affinity[inferred_arch]
        Cards with matching resonance get 4x weight boost.
      - 1 random splash slot
    """
    if len(pool_list) < PACK_SIZE:
        return pool_list[:PACK_SIZE]

    pack = []
    used_ids = set()

    # Floor slot: from pick 3+
    if pick >= 3:
        sorted_by_power = sorted(pool_list, key=lambda c: c.power, reverse=True)
        top_q = sorted_by_power[:max(1, len(sorted_by_power) // 4)]
        floor_card = None
        for c in top_q:
            if c.id not in used_ids:
                floor_card = c
                break
        if floor_card:
            pack.append(floor_card)
            used_ids.add(floor_card.id)

    if committed_res is None or inferred_arch is None:
        # Pre-commitment: fill remaining slots randomly
        remaining = PACK_SIZE - len(pack)
        avail = [c for c in pool_list if c.id not in used_ids]
        if len(avail) >= remaining:
            for c in random.sample(avail, remaining):
                pack.append(c)
                used_ids.add(c.id)
        else:
            pack.extend(avail)
        return pack[:PACK_SIZE]

    # Post-commitment: archetype-weighted pack construction
    def archetype_weight(c):
        """Weight for archetype-focused slots."""
        base = 1.0
        if c.is_generic:
            return 0.5
        # Resonance match: big boost
        if committed_res in c.visible_symbols:
            base = 4.0
        # Pair affinity boost for inferred archetype
        aff = c.pair_affinity.get(inferred_arch, 0.0)
        return base * (1.0 + aff * 3.0)

    # 2 archetype-weighted slots
    arch_slots = min(2, PACK_SIZE - len(pack) - 1)
    for _ in range(arch_slots):
        card = weighted_draw(pool_list, archetype_weight, used_ids)
        if card:
            pack.append(card)
            used_ids.add(card.id)

    # Fill remaining with random (splash)
    while len(pack) < PACK_SIZE:
        avail = [c for c in pool_list if c.id not in used_ids]
        if not avail:
            break
        c = random.choice(avail)
        pack.append(c)
        used_ids.add(c.id)

    return pack[:PACK_SIZE]


# ============================================================
# Draft
# ============================================================
def get_phase(pick_num):
    if pick_num <= 5: return 1
    elif pick_num <= 10: return 2
    elif pick_num <= 15: return 3
    else: return 4


def get_committed_resonance(signature):
    best_res = max(RESONANCE_TYPES, key=lambda r: signature[r])
    if signature[best_res] >= COMMIT_THRESHOLD:
        return best_res
    return None


def infer_archetype(drafted, committed_res):
    if committed_res is None:
        return None
    arch_counts = defaultdict(int)
    for c in drafted:
        if not c.is_generic and committed_res in c.visible_symbols:
            arch_counts[c.archetype] += 1
    if not arch_counts or sum(arch_counts.values()) < 2:
        return None
    return max(arch_counts, key=arch_counts.get)


def run_draft(pool, player_archetype, fitness_model, strategy, ai_archetypes):
    active_pool = {c.id: c for c in pool}
    sa_cache = precompute_card_tiers(pool, player_archetype, fitness_model)

    ai_drafters = [AIDrafter(arch) for arch in ai_archetypes]
    open_lanes = [a for a in ARCHETYPE_NAMES if a not in ai_archetypes]

    drafted = []
    history = []
    signature = {r: 0.0 for r in RESONANCE_TYPES}
    ai_total_removed = 0
    cull_total_removed = 0

    for pick in range(1, NUM_PICKS + 1):
        if len(active_pool) < PACK_SIZE:
            break

        pool_before = len(active_pool)

        # Step 1: AI drafters pick
        ai_removed_this_round = set()
        if len(active_pool) > POOL_FLOOR:
            order = list(range(len(ai_drafters)))
            random.shuffle(order)
            for idx in order:
                if len(active_pool) <= POOL_FLOOR:
                    break
                ai = ai_drafters[idx]
                picked = ai.pick_cards(active_pool, pick)
                ai_removed_this_round |= picked
                for cid in picked:
                    if cid in active_pool:
                        del active_pool[cid]
        ai_total_removed += len(ai_removed_this_round)

        # Step 2: Market culling
        cull_removed = 0
        if len(active_pool) > POOL_FLOOR + MARKET_CULL_COUNT:
            pool_sorted = sorted(active_pool.values(), key=lambda c: c.power)
            n_to_cull = min(MARKET_CULL_COUNT,
                            len(pool_sorted) - POOL_FLOOR)
            for c in pool_sorted[:n_to_cull]:
                del active_pool[c.id]
                cull_removed += 1
        cull_total_removed += cull_removed

        # Step 3: Build pack
        pool_list = list(active_pool.values())
        if len(pool_list) < PACK_SIZE:
            break

        committed_res = get_committed_resonance(signature)
        inferred_arch = infer_archetype(drafted, committed_res)

        pack = build_pack(pool_list, committed_res, inferred_arch, pick,
                          sa_cache)

        sa_count = sum(1 for c in pack if sa_cache.get(c.id, False))

        # Step 4: Player picks
        chosen = select_card(pack, player_archetype, signature, strategy,
                             pick, sa_cache, open_lanes)
        drafted.append(chosen)

        for i, sym in enumerate(chosen.visible_symbols):
            signature[sym] += 2.0 if i == 0 else 1.0

        if chosen.id in active_pool:
            del active_pool[chosen.id]

        # Pool composition
        arch_counts_in_pool = defaultdict(int)
        for c in active_pool.values():
            if not c.is_generic:
                arch_counts_in_pool[c.archetype] += 1
        total_non_generic = sum(arch_counts_in_pool.values())
        player_arch_in_pool = arch_counts_in_pool.get(player_archetype, 0)
        concentration = player_arch_in_pool / max(1, total_non_generic)

        history.append({
            "pick": pick,
            "pack": pack,
            "chosen": chosen,
            "pool_size": pool_before,
            "pool_after": len(active_pool),
            "sa_count": sa_count,
            "ai_removed": len(ai_removed_this_round),
            "cull_removed": cull_removed,
            "phase": get_phase(pick),
            "player_arch_in_pool": player_arch_in_pool,
            "concentration": concentration,
            "committed_res": committed_res,
            "inferred_arch": inferred_arch,
        })

    return history, drafted, sa_cache, {
        "ai_archetypes": ai_archetypes,
        "open_lanes": open_lanes,
        "ai_total_removed": ai_total_removed,
        "cull_total_removed": cull_total_removed,
    }


def select_card(pack, player_archetype, signature, strategy, pick, sa_cache,
                open_lanes):
    arch = ARCH_BY_NAME[player_archetype]
    r1, r2 = arch[1], arch[2]

    if strategy == "committed":
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
        return max(pack, key=lambda c: c.power)

    elif strategy == "signal":
        if pick <= 4:
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
# Metrics
# ============================================================
def compute_metrics(all_results):
    m1_vals, m2_vals, m3_vals, m4_vals = [], [], [], []
    m5_vals, m6_vals, m9_vals, m10_vals = [], [], [], []
    m11_vals = []
    post_commit_sa = []
    consec_bad_list = []

    for history, drafted, sa_cache, meta in all_results:
        early_arch = []
        for h in history[:5]:
            archs = set()
            for c in h["pack"]:
                if not c.is_generic:
                    archs.add(c.archetype)
            early_arch.append(len(archs))
        m1_vals.append(sum(early_arch) / max(1, len(early_arch)))

        early_sa = []
        for h in history[:5]:
            sa = sum(1 for c in h["pack"] if sa_cache.get(c.id, False))
            early_sa.append(sa)
        m2_vals.append(sum(early_sa) / max(1, len(early_sa)))

        post_sa = []
        for h in history[5:]:
            sa = h["sa_count"]
            post_sa.append(sa)
            post_commit_sa.append(sa)
        if post_sa:
            m3_vals.append(sum(post_sa) / len(post_sa))

        post_off = []
        for h in history[5:]:
            off = sum(1 for c in h["pack"] if not sa_cache.get(c.id, False))
            post_off.append(off)
        if post_off:
            m4_vals.append(sum(post_off) / len(post_off))

        conv_pick = NUM_PICKS
        all_sa = [h["sa_count"] for h in history]
        for i in range(2, len(all_sa)):
            window = all_sa[i-2:i+1]
            if sum(window) / 3 >= 1.5:
                conv_pick = i + 1
                break
        m5_vals.append(conv_pick)

        sa_drafted = sum(1 for c in drafted if sa_cache.get(c.id, False))
        m6_vals.append(sa_drafted / max(1, len(drafted)))

        if len(post_sa) > 1:
            mean_sa = sum(post_sa) / len(post_sa)
            var = sum((x - mean_sa)**2 for x in post_sa) / len(post_sa)
            m9_vals.append(math.sqrt(var))

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

        late_sa = [h["sa_count"] for h in history[14:]]
        if late_sa:
            m11_vals.append(sum(late_sa) / len(late_sa))

    m7_overlaps = []
    for i in range(1, len(all_results)):
        ids_prev = set(c.id for c in all_results[i-1][1])
        ids_curr = set(c.id for c in all_results[i][1])
        overlap = len(ids_prev & ids_curr) / max(1, len(ids_prev | ids_curr))
        m7_overlaps.append(overlap)

    arch_counts = defaultdict(int)
    for _, drafted, _, _ in all_results:
        dominant = defaultdict(int)
        for c in drafted:
            if not c.is_generic:
                dominant[c.archetype] += 1
        if dominant:
            arch_counts[max(dominant, key=dominant.get)] += 1
    total_drafts = len(all_results)

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
# Runners
# ============================================================
def run_aggregate(fitness_name, strategy, n_drafts=NUM_DRAFTS):
    fitness_model = FITNESS_MODELS[fitness_name]
    all_results = []
    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        pool = build_pool()
        ai_archetypes = random.sample(ARCHETYPE_NAMES, NUM_AI_DRAFTERS)
        h, d, cache, meta = run_draft(
            pool, arch_name, fitness_model, strategy, ai_archetypes
        )
        all_results.append((h, d, cache, meta))
    return compute_metrics(all_results), all_results


def run_per_archetype(fitness_name, strategy, n_per=125):
    fitness_model = FITNESS_MODELS[fitness_name]
    results = {}
    for arch_name in ARCHETYPE_NAMES:
        histories = []
        for _ in range(n_per):
            pool = build_pool()
            ai_archetypes = random.sample(ARCHETYPE_NAMES, NUM_AI_DRAFTERS)
            h, d, cache, meta = run_draft(
                pool, arch_name, fitness_model, strategy, ai_archetypes
            )
            histories.append((h, d, cache, meta))
        results[arch_name] = compute_metrics(histories)
    return results


def analyze_open_vs_contested(all_results):
    open_results = []
    contested_results = []
    for result in all_results:
        history, drafted, sa_cache, meta = result
        # Infer player arch from the archetype cycling
        # The run_aggregate cycles through archetypes, but we can use
        # committed resonance from history
        arch_counts = defaultdict(int)
        for c in drafted:
            if not c.is_generic and sa_cache.get(c.id, False):
                arch_counts[c.archetype] += 1
        if not arch_counts:
            arch_counts2 = defaultdict(int)
            for c in drafted:
                if not c.is_generic:
                    arch_counts2[c.archetype] += 1
            if not arch_counts2:
                continue
            player_arch = max(arch_counts2, key=arch_counts2.get)
        else:
            player_arch = max(arch_counts, key=arch_counts.get)

        if player_arch in meta["ai_archetypes"]:
            contested_results.append(result)
        else:
            open_results.append(result)

    open_m = compute_metrics(open_results) if open_results else None
    cont_m = compute_metrics(contested_results) if contested_results else None
    return open_m, cont_m, len(open_results), len(contested_results)


def analyze_ai_behavior(all_results):
    total_ai_removed = 0
    total_cull_removed = 0
    pool_sizes_by_pick = defaultdict(list)
    conc_by_pick = defaultdict(list)

    for history, _, _, meta in all_results:
        total_ai_removed += meta["ai_total_removed"]
        total_cull_removed += meta["cull_total_removed"]
        for h in history:
            pool_sizes_by_pick[h["pick"]].append(h["pool_size"])
            conc_by_pick[h["pick"]].append(h["concentration"])

    n = len(all_results)
    avg_ai = total_ai_removed / max(1, n)
    avg_cull = total_cull_removed / max(1, n)

    avg_pool = {}
    for pick in sorted(pool_sizes_by_pick.keys()):
        avg_pool[pick] = sum(pool_sizes_by_pick[pick]) / len(pool_sizes_by_pick[pick])

    avg_conc = {}
    for pick in sorted(conc_by_pick.keys()):
        avg_conc[pick] = sum(conc_by_pick[pick]) / len(conc_by_pick[pick])

    return avg_ai, avg_cull, avg_pool, avg_conc


def format_trace(history, drafted, sa_cache, meta, player_archetype,
                 n_picks=30):
    lines = []
    lines.append(f"=== Draft Trace: Player={player_archetype} ===")
    lines.append(f"  AI drafters: {', '.join(meta['ai_archetypes'])}")
    lines.append(f"  Open lanes:  {', '.join(meta['open_lanes'])}")
    contested = player_archetype in meta['ai_archetypes']
    lines.append(f"  Player lane: {'CONTESTED' if contested else 'OPEN'}")
    lines.append("")

    for h in history[:n_picks]:
        pick = h["pick"]
        sa = h["sa_count"]
        pool_sz = h["pool_size"]
        pool_after = h["pool_after"]
        chosen = h["chosen"]
        chosen_sa = "S/A" if sa_cache.get(chosen.id, False) else "C/F"
        sym_str = "/".join(chosen.visible_symbols) if chosen.visible_symbols else "Generic"
        ai_rem = h["ai_removed"]
        cull_rem = h["cull_removed"]
        phase = h["phase"]
        conc = h["concentration"]
        arch_in_pool = h["player_arch_in_pool"]
        c_res = h.get("committed_res") or "none"
        i_arch = h.get("inferred_arch") or "none"
        lines.append(
            f"  P{pick:02d} [Ph{phase}] pool={pool_sz:3d}->{pool_after:3d} "
            f"AI={ai_rem:2d} cull={cull_rem} "
            f"SA={sa} conc={conc:.0%}({arch_in_pool:2d}) "
            f"res={c_res:<7s} "
            f"-> [{chosen.archetype}:{sym_str}] ({chosen_sa})"
        )

    sa_d = sum(1 for c in drafted if sa_cache.get(c.id, False))
    lines.append(f"\n  Final: {sa_d}/{len(drafted)} S/A = "
                 f"{sa_d/max(1,len(drafted))*100:.0f}%")
    return "\n".join(lines)


# ============================================================
# Main
# ============================================================
def main():
    random.seed(42)

    print("=" * 80)
    print("HYBRID Y: ESCALATING OPEN TABLE (D1 + D4)")
    print("5 AIs, escalating picks (2/3/4/5), 3 open lanes, 8-card cull")
    print("Affinity-weighted pack construction | 1000 drafts x 30 picks")
    print("=" * 80)

    strategies = ["committed", "power", "signal"]
    grad_results = {}
    grad_all = {}

    print("\n" + "=" * 60)
    print("SECTION 1: Graduated Realistic -- All Strategies")
    print("=" * 60)
    for strategy in strategies:
        metrics, all_res = run_aggregate("Graduated", strategy)
        grad_results[strategy] = metrics
        grad_all[strategy] = all_res
        m = metrics
        print(f"\n  Strategy: {strategy}")
        print(f"    M1={m['M1']:.2f}  M2={m['M2']:.2f}  M3={m['M3']:.2f}  "
              f"M4={m['M4']:.2f}  M5={m['M5']:.1f}")
        print(f"    M6={m['M6']:.2f}  M7={m['M7']:.3f}  M9={m['M9']:.2f}  "
              f"M10={m['M10']:.2f}  M11={m['M11']:.2f}")
        print(f"    M10_worst={m['M10_worst']}  "
              f"worst_consec_bad={m['worst_consec_bad']}")

    print("\n" + "=" * 60)
    print("SECTION 2: Pessimistic Fitness")
    print("=" * 60)
    pess_metrics, _ = run_aggregate("Pessimistic", "committed")
    m = pess_metrics
    print(f"\n  Pessimistic / committed:")
    print(f"    M3={m['M3']:.2f}  M10={m['M10']:.2f}  "
          f"M11={m['M11']:.2f}  M6={m['M6']:.2f}")

    print("\n" + "=" * 60)
    print("SECTION 3: Per-Archetype M3 (Graduated, committed)")
    print("=" * 60)
    per_arch = run_per_archetype("Graduated", "committed")
    print(f"\n  {'Archetype':<16} {'M3':>6} {'M10':>6} {'M11':>6} {'M6':>6}")
    print(f"  {'-'*16} {'-'*6} {'-'*6} {'-'*6} {'-'*6}")
    worst_m3 = 10.0
    worst_arch = ""
    for arch in ARCHETYPE_NAMES:
        m = per_arch[arch]
        if m['M3'] < worst_m3:
            worst_m3 = m['M3']
            worst_arch = arch
        print(f"  {arch:<16} {m['M3']:6.2f} {m['M10']:6.2f} "
              f"{m['M11']:6.2f} {m['M6']:6.2f}")
    print(f"\n  Worst: {worst_arch} (M3={worst_m3:.2f})")

    print("\n" + "=" * 60)
    print("SECTION 4: Open vs Contested Lane (committed)")
    print("=" * 60)
    open_m, cont_m, n_open, n_cont = analyze_open_vs_contested(
        grad_all["committed"])
    print(f"\n  Open lane drafts:     {n_open}")
    print(f"  Contested lane drafts: {n_cont}")
    if open_m:
        print(f"\n  Open lane:     M3={open_m['M3']:.2f}  "
              f"M11={open_m['M11']:.2f}  M10={open_m['M10']:.2f}  "
              f"M6={open_m['M6']:.2f}")
    if cont_m:
        print(f"  Contested:     M3={cont_m['M3']:.2f}  "
              f"M11={cont_m['M11']:.2f}  M10={cont_m['M10']:.2f}  "
              f"M6={cont_m['M6']:.2f}")

    print("\n" + "=" * 60)
    print("SECTION 5: AI Behavior Summary")
    print("=" * 60)
    avg_ai, avg_cull, avg_pool, avg_conc = analyze_ai_behavior(
        grad_all["committed"])
    print(f"\n  Avg AI cards removed per draft:   {avg_ai:.1f}")
    print(f"  Avg market cull removed per draft: {avg_cull:.1f}")
    print(f"  Total removed per draft:           {avg_ai + avg_cull:.1f}")
    print(f"\n  Pool size trajectory:")
    print(f"  {'Pick':>6} {'Phase':>5} {'Pool':>6} {'ArchConc':>8}")
    for pick in sorted(avg_pool.keys()):
        phase = get_phase(pick)
        pool_sz = avg_pool[pick]
        conc = avg_conc.get(pick, 0)
        print(f"  {pick:6d} {phase:5d} {pool_sz:6.0f} {conc:8.1%}")

    print("\n" + "=" * 60)
    print("SECTION 6: Pack Quality Distribution (picks 6+, committed)")
    print("=" * 60)
    pq = grad_results["committed"]["pack_pcts"]
    m = grad_results["committed"]
    print(f"\n    P10={pq[10]}  P25={pq[25]}  P50={pq[50]}  "
          f"P75={pq[75]}  P90={pq[90]}")
    print(f"    Avg consec bad: {m['avg_consec_bad']:.2f}")
    print(f"    Worst consec bad: {m['worst_consec_bad']}")

    print("\n" + "=" * 60)
    print("SECTION 7: Archetype Frequency M8")
    print("=" * 60)
    arch_freq = grad_results["committed"]["arch_freq"]
    print(f"\n  {'Archetype':<16} {'Freq%':>6} {'M8':>6}")
    for arch in ARCHETYPE_NAMES:
        freq = arch_freq.get(arch, 0.0) * 100
        passes = 5.0 <= freq <= 20.0
        print(f"  {arch:<16} {freq:6.1f}%  {'PASS' if passes else 'FAIL'}")

    print("\n" + "=" * 60)
    print("SECTION 8: Draft Traces")
    print("=" * 60)

    random.seed(100)
    pool = build_pool()
    ai_t1 = ["Flash", "Blink", "Storm", "Self-Mill", "Sacrifice"]
    h1, d1, c1, m1 = run_draft(
        pool, "Warriors", FITNESS_MODELS["Graduated"], "committed", ai_t1)
    print("\n" + format_trace(h1, d1, c1, m1, "Warriors"))

    random.seed(200)
    pool = build_pool()
    ai_t2 = ["Flash", "Storm", "Self-Discard", "Sacrifice", "Ramp"]
    h2, d2, c2, m2 = run_draft(
        pool, "Warriors", FITNESS_MODELS["Graduated"], "signal", ai_t2)
    print("\n" + format_trace(h2, d2, c2, m2, "Warriors"))

    print("\n" + "=" * 80)
    print("FULL SCORECARD: Hybrid Y, Graduated Realistic, committed")
    print("=" * 80)

    cm = grad_results["committed"]
    targets = [
        ("M1",  cm["M1"],  ">=3",       cm["M1"] >= 3.0),
        ("M2",  cm["M2"],  "<=2",       cm["M2"] <= 2.0),
        ("M3",  cm["M3"],  ">=2.0",     cm["M3"] >= 2.0),
        ("M4",  cm["M4"],  ">=0.5",     cm["M4"] >= 0.5),
        ("M5",  cm["M5"],  "5-8",       5 <= cm["M5"] <= 8),
        ("M6",  cm["M6"],  "0.60-0.90", 0.60 <= cm["M6"] <= 0.90),
        ("M7",  cm["M7"],  "<0.40",     cm["M7"] < 0.40),
        ("M9",  cm["M9"],  ">=0.8",     cm["M9"] >= 0.8),
        ("M10", cm["M10"], "<=2",       cm["M10"] <= 2.0),
        ("M11", cm["M11"], ">=3.0",     cm["M11"] >= 3.0),
    ]

    print(f"\n  {'Metric':<6} {'Value':>8} {'Target':<12} Status")
    print(f"  {'-'*6} {'-'*8} {'-'*12} {'-'*6}")
    n_pass = 0
    for name, val, target, passes in targets:
        status = "PASS" if passes else "FAIL"
        if passes: n_pass += 1
        print(f"  {name:<6} {val:8.3f} {target:<12} {status}")

    print(f"\n  {n_pass}/{len(targets)} metrics pass")

    print(f"\n  V9 Hybrid B: M3=2.70, M11=3.25, M10=3.8")
    print(f"  Hybrid Y:    M3={cm['M3']:.2f}, M11={cm['M11']:.2f}, "
          f"M10={cm['M10']:.2f}")
    m3_d = cm["M3"] - 2.70
    m11_d = cm["M11"] - 3.25
    m10_d = cm["M10"] - 3.8
    print(f"  Delta:       M3={m3_d:+.2f}, M11={m11_d:+.2f}, M10={m10_d:+.2f}")

    print(f"\n  Pessimistic: M3={pess_metrics['M3']:.2f}  "
          f"M10={pess_metrics['M10']:.2f}  M11={pess_metrics['M11']:.2f}")

    print("\nDone.")


if __name__ == "__main__":
    main()
