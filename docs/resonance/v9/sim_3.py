"""
Simulation Agent 3: Design 2 — Tag-Gravity (60/40 blend)
V9 Round 4

Algorithm: Narrative Gravity with hidden 3-bit archetype tags.
- Relevance score: 60% hidden tag match + 40% visible resonance dot-product.
- Pool contraction: 12% of surviving pool removed per pick, from pick 4.
- Floor slot: from pick 3, one pack slot draws from top-quartile by relevance.
- Archetype inference: mode of hidden tags on drafted cards (pick 5+).
- Pre-pick-5: relevance is 100% visible dot-product.
- Generics protected at 0.5 baseline relevance.
- Pool: 360 cards, 10% visible dual-res, 79% single-symbol, 11% generic.
"""

import random
import math
from dataclasses import dataclass, field
from collections import defaultdict, Counter
from typing import Optional, List, Dict, Tuple

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
    ("Flash",       "Zephyr", "Ember"),
    ("Blink",       "Ember",  "Zephyr"),
    ("Storm",       "Ember",  "Stone"),
    ("Self-Discard","Stone",  "Ember"),
    ("Self-Mill",   "Stone",  "Tide"),
    ("Sacrifice",   "Tide",   "Stone"),
    ("Warriors",    "Tide",   "Zephyr"),
    ("Ramp",        "Zephyr", "Tide"),
]

ARCHETYPE_NAMES = [a[0] for a in ARCHETYPES]
ARCH_BY_NAME = {a[0]: a for a in ARCHETYPES}
ARCH_INDEX = {a[0]: i for i, a in enumerate(ARCHETYPES)}

# Graduated Realistic fitness per co-primary pair
FITNESS_GRADUATED = {
    ("Warriors", "Sacrifice"): 0.50,
    ("Sacrifice", "Warriors"): 0.50,
    ("Self-Discard", "Self-Mill"): 0.40,
    ("Self-Mill", "Self-Discard"): 0.40,
    ("Blink", "Storm"): 0.30,
    ("Storm", "Blink"): 0.30,
    ("Flash", "Ramp"): 0.25,
    ("Ramp", "Flash"): 0.25,
}

# Pessimistic fitness per co-primary pair
FITNESS_PESSIMISTIC = {
    ("Warriors", "Sacrifice"): 0.35,
    ("Sacrifice", "Warriors"): 0.35,
    ("Self-Discard", "Self-Mill"): 0.25,
    ("Self-Mill", "Self-Discard"): 0.25,
    ("Blink", "Storm"): 0.15,
    ("Storm", "Blink"): 0.15,
    ("Flash", "Ramp"): 0.10,
    ("Ramp", "Flash"): 0.10,
}

FITNESS_MODELS = {
    "Graduated": FITNESS_GRADUATED,
    "Pessimistic": FITNESS_PESSIMISTIC,
}


def get_sibling(arch_name: str) -> Optional[str]:
    """Return co-primary sibling archetype name (shares primary resonance)."""
    r1 = ARCH_BY_NAME[arch_name][1]
    for a in ARCHETYPES:
        if a[0] != arch_name and a[1] == r1:
            return a[0]
    return None


# ============================================================
# Card & Pool Construction
# ============================================================
@dataclass
class SimCard:
    id: int
    visible_symbols: List[str]   # 0-2 resonance symbols (player-facing)
    hidden_tag: int               # archetype index 0-7 (algorithm-only)
    archetype: str                # true archetype (evaluation only)
    power: float
    is_generic: bool = False


def build_pool() -> List[SimCard]:
    """
    Build 360-card pool:
    - 40 generics (no visible symbols, neutral hidden tag = 8)
    - 284 single-symbol cards (one visible symbol, tagged to archetype)
    - 36 dual-symbol cards (two visible symbols, 4-5 per archetype)

    Tag distribution: 40 cards per archetype (320 non-generic),
    with tags reflecting mechanical best-fit.
    """
    cards = []
    card_id = 0

    # Dual-res: 36 total = 4 or 5 per archetype pair.
    # We use 4 for the first 4 archetypes and 5 for the last 4
    # to get exactly 36 (4*4 + 4*5 = 16 + 20 = 36).
    dual_per_arch = [4, 4, 4, 4, 5, 5, 5, 5]

    for arch_idx, (arch_name, r1, r2) in enumerate(ARCHETYPES):
        n_dual = dual_per_arch[arch_idx]
        n_single = CARDS_PER_ARCHETYPE - n_dual  # 36 or 35

        for _ in range(n_single):
            cards.append(SimCard(
                id=card_id,
                visible_symbols=[r1],
                hidden_tag=arch_idx,
                archetype=arch_name,
                power=random.uniform(4, 8),
                is_generic=False,
            ))
            card_id += 1

        for _ in range(n_dual):
            cards.append(SimCard(
                id=card_id,
                visible_symbols=[r1, r2],
                hidden_tag=arch_idx,
                archetype=arch_name,
                power=random.uniform(4, 8),
                is_generic=False,
            ))
            card_id += 1

    # Generics: neutral hidden_tag = 8 (no archetype)
    for _ in range(GENERIC_CARDS):
        cards.append(SimCard(
            id=card_id,
            visible_symbols=[],
            hidden_tag=8,  # neutral tag
            archetype="Generic",
            power=random.uniform(3, 7),
            is_generic=True,
        ))
        card_id += 1

    assert len(cards) == POOL_SIZE, f"Pool size mismatch: {len(cards)}"
    return cards


def precompute_sa_tiers(
    pool: List[SimCard],
    player_archetype: str,
    fitness_model: Dict,
) -> Dict[int, bool]:
    """Pre-roll S/A status for all cards."""
    sibling = get_sibling(player_archetype)
    sa_map = {}
    for c in pool:
        if c.is_generic:
            sa_map[c.id] = False
        elif c.archetype == player_archetype:
            sa_map[c.id] = True
        elif sibling and c.archetype == sibling:
            rate = fitness_model.get((player_archetype, sibling), 0.0)
            sa_map[c.id] = (random.random() < rate)
        else:
            sa_map[c.id] = False
    return sa_map


# ============================================================
# Tag-Gravity Algorithm
# ============================================================
def compute_visible_dot(card: SimCard, signature: Dict[str, float], sig_magnitude: float) -> float:
    """Compute normalized visible resonance dot-product."""
    if not card.visible_symbols:
        return 0.0
    card_vec = {r: 0.0 for r in RESONANCE_TYPES}
    for i, sym in enumerate(card.visible_symbols):
        card_vec[sym] += 2.0 if i == 0 else 1.0
    card_mag = math.sqrt(sum(v**2 for v in card_vec.values()))
    if card_mag == 0 or sig_magnitude == 0:
        return 0.0
    dot = sum(card_vec[r] * signature[r] for r in RESONANCE_TYPES)
    return dot / (card_mag * sig_magnitude)


def infer_archetype(drafted_tags: List[int]) -> Optional[int]:
    """
    Infer committed archetype from mode of hidden tags on drafted cards.
    Returns None if fewer than 5 cards drafted (picks 1-4).
    Non-generic tags only (tag != 8).
    """
    if len(drafted_tags) < 5:
        return None
    non_generic = [t for t in drafted_tags if t != 8]
    if not non_generic:
        return None
    counts = Counter(non_generic)
    return counts.most_common(1)[0][0]


def compute_relevance(
    card: SimCard,
    signature: Dict[str, float],
    sig_magnitude: float,
    inferred_arch_idx: Optional[int],
) -> float:
    """
    Compute relevance score for pool contraction.

    From pick 4 (contraction active):
    - Generic cards: fixed 0.5 (protected baseline)
    - Pre-inference (pick < 5): 100% visible dot-product
    - Post-inference (pick >= 5): 60% tag match + 40% visible dot-product
    """
    if card.is_generic:
        return 0.5

    visible_score = compute_visible_dot(card, signature, sig_magnitude)

    if inferred_arch_idx is None:
        # Pre-pick-5: use only visible dot-product
        return visible_score

    # Post-pick-5: blend 40% visible + 60% tag match
    tag_score = 1.0 if card.hidden_tag == inferred_arch_idx else 0.0
    return 0.4 * visible_score + 0.6 * tag_score


def tag_gravity_draft(
    pool: List[SimCard],
    player_archetype: str,
    fitness_model: Dict,
    strategy: str,
    contraction_rate: float = 0.12,
    contraction_start: int = 4,
    floor_slot_start: int = 3,
) -> Tuple[List[Dict], List[SimCard], Dict[int, bool]]:
    """
    Run one draft using Tag-Gravity algorithm.

    Parameters:
      contraction_rate: fraction of pool removed per pick (default 12%)
      contraction_start: pick at which contraction begins (default 4)
      floor_slot_start: pick at which floor slot activates (default 3)
    """
    active_pool = list(pool)
    signature = {r: 0.0 for r in RESONANCE_TYPES}
    drafted: List[SimCard] = []
    drafted_tags: List[int] = []
    history: List[Dict] = []

    sa_cache = precompute_sa_tiers(pool, player_archetype, fitness_model)

    for pick in range(1, NUM_PICKS + 1):
        if len(active_pool) < PACK_SIZE:
            break

        # Compute sig magnitude once per pick
        sig_magnitude = math.sqrt(sum(v**2 for v in signature.values()))

        # Inferred archetype (None before pick 5)
        inferred = infer_archetype(drafted_tags)

        # --- Pack construction ---
        if pick >= floor_slot_start and len(active_pool) >= PACK_SIZE:
            # Floor slot: draw from top-quartile by relevance
            scored = [
                (compute_relevance(c, signature, sig_magnitude, inferred), c)
                for c in active_pool
            ]
            scored.sort(key=lambda x: -x[0])
            quartile_cutoff = max(1, len(scored) // 4)
            top_quartile = [c for _, c in scored[:quartile_cutoff]]

            # One floor card from top quartile
            floor_card = random.choice(top_quartile)

            # Remaining 3 random slots from the rest of active_pool
            remaining = [c for c in active_pool if c.id != floor_card.id]
            if len(remaining) >= PACK_SIZE - 1:
                random_slots = random.sample(remaining, PACK_SIZE - 1)
            else:
                random_slots = remaining
            pack = [floor_card] + random_slots
        else:
            # Early picks (before floor slot): pure random
            pack = random.sample(active_pool, min(PACK_SIZE, len(active_pool)))

        # --- Card selection ---
        chosen = select_card(pack, player_archetype, signature, strategy, pick, sa_cache)

        drafted.append(chosen)
        drafted_tags.append(chosen.hidden_tag)

        # Update visible resonance signature
        for i, sym in enumerate(chosen.visible_symbols):
            signature[sym] += 2.0 if i == 0 else 1.0

        sa_count = sum(1 for c in pack if sa_cache[c.id])
        history.append({
            "pick": pick,
            "pack": pack,
            "chosen": chosen,
            "pool_size": len(active_pool),
            "sa_count": sa_count,
            "inferred_arch": ARCHETYPE_NAMES[inferred] if inferred is not None else None,
            "signature": dict(signature),
        })

        # --- Pool contraction (from pick contraction_start) ---
        if pick >= contraction_start:
            sig_magnitude_post = math.sqrt(sum(v**2 for v in signature.values()))
            inferred_post = infer_archetype(drafted_tags)

            n_remove = max(1, int(len(active_pool) * contraction_rate))

            scored = []
            for c in active_pool:
                rel = compute_relevance(c, signature, sig_magnitude_post, inferred_post)
                scored.append((rel, c))
            scored.sort(key=lambda x: x[0])  # ascending: lowest relevance first

            to_remove = {c.id for _, c in scored[:n_remove]}
            active_pool = [c for c in active_pool if c.id not in to_remove]

    return history, drafted, sa_cache


def select_card(
    pack: List[SimCard],
    player_archetype: str,
    signature: Dict[str, float],
    strategy: str,
    pick: int,
    sa_cache: Dict[int, bool],
) -> SimCard:
    """Select a card from pack based on strategy."""
    arch = ARCH_BY_NAME[player_archetype]
    r1, r2 = arch[1], arch[2]

    if strategy == "committed":
        def score(c: SimCard) -> float:
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
        if pick <= 3:
            return max(pack, key=lambda c: c.power)
        top_res = max(RESONANCE_TYPES, key=lambda r: signature[r])
        def signal_score(c: SimCard) -> float:
            s = 0.0
            for i, sym in enumerate(c.visible_symbols):
                if sym == top_res:
                    s += 3.0 if i == 0 else 1.0
            s += c.power * 0.1
            if sa_cache.get(c.id, False):
                s += 5.0
            return s
        return max(pack, key=signal_score)

    return random.choice(pack)


# ============================================================
# Metrics Computation
# ============================================================
def compute_metrics(all_histories: List[Tuple]) -> Dict:
    """Compute M1-M11 from draft histories."""
    m1_vals, m2_vals, m3_vals, m4_vals = [], [], [], []
    m5_vals, m6_vals, m9_vals, m10_vals = [], [], [], []
    m11_vals = []
    post_commit_sa = []
    late_draft_sa = []  # picks 15+
    consec_bad_list = []
    all_drafted_ids: List[List[int]] = []

    for history, drafted, sa_cache in all_histories:
        all_drafted_ids.append([c.id for c in drafted])

        # M1: picks 1-5, avg unique archetypes per pack
        early_arch = []
        for h in history[:5]:
            archs = set(c.archetype for c in h["pack"] if not c.is_generic)
            early_arch.append(len(archs))
        m1_vals.append(sum(early_arch) / max(1, len(early_arch)))

        # M2: picks 1-5, avg S/A per pack
        early_sa = []
        for h in history[:5]:
            sa = sum(1 for c in h["pack"] if sa_cache[c.id])
            early_sa.append(sa)
        m2_vals.append(sum(early_sa) / max(1, len(early_sa)))

        # M3: picks 6+, avg S/A per pack
        post_sa = []
        for h in history[5:]:
            sa = h["sa_count"]
            post_sa.append(sa)
            post_commit_sa.append(sa)
        if post_sa:
            m3_vals.append(sum(post_sa) / len(post_sa))

        # M4: picks 6+, avg off-archetype (non-S/A) per pack
        post_off = []
        for h in history[5:]:
            off = sum(1 for c in h["pack"] if not sa_cache[c.id])
            post_off.append(off)
        if post_off:
            m4_vals.append(sum(post_off) / len(post_off))

        # M5: convergence pick (first pick where rolling 3-pick avg >= 1.5 S/A)
        conv_pick = NUM_PICKS
        for i in range(2, len(history)):
            window = [history[j]["sa_count"] for j in range(i - 2, i + 1)]
            if sum(window) / 3 >= 1.5:
                conv_pick = i + 1
                break
        m5_vals.append(conv_pick)

        # M6: deck concentration (S/A fraction of drafted cards)
        sa_drafted = sum(1 for c in drafted if sa_cache[c.id])
        m6_vals.append(sa_drafted / max(1, len(drafted)))

        # M9: stddev of S/A per pack (picks 6+)
        if len(post_sa) > 1:
            mean_sa = sum(post_sa) / len(post_sa)
            var = sum((x - mean_sa)**2 for x in post_sa) / len(post_sa)
            m9_vals.append(math.sqrt(var))

        # M10: max consecutive packs with <1.5 S/A (picks 6+)
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

        # M11: picks 15+, avg S/A per pack
        late_sa_picks = []
        for h in history[14:]:  # 0-indexed, pick 15 = index 14
            sa = h["sa_count"]
            late_sa_picks.append(sa)
            late_draft_sa.append(sa)
        if late_sa_picks:
            m11_vals.append(sum(late_sa_picks) / len(late_sa_picks))

    # M7: run-to-run card overlap
    m7_overlaps = []
    for i in range(1, len(all_drafted_ids)):
        ids_prev = set(all_drafted_ids[i - 1])
        ids_curr = set(all_drafted_ids[i])
        overlap = len(ids_prev & ids_curr) / max(1, len(ids_prev | ids_curr))
        m7_overlaps.append(overlap)

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
        "M10_max": max(m10_vals) if m10_vals else 0,
        "M11": avg(m11_vals),
        "pack_pcts": pcts,
        "avg_consec_bad": avg(consec_bad_list),
        "worst_consec_bad": max(consec_bad_list) if consec_bad_list else 0,
    }


# ============================================================
# Runners
# ============================================================
def run_aggregate(
    fitness_model: Dict,
    strategy: str,
    n_drafts: int = NUM_DRAFTS,
    contraction_rate: float = 0.12,
) -> Tuple[Dict, List[Tuple]]:
    """Run aggregate drafts cycling through archetypes."""
    all_histories = []
    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        pool = build_pool()
        h, d, cache = tag_gravity_draft(
            pool, arch_name, fitness_model, strategy,
            contraction_rate=contraction_rate,
        )
        all_histories.append((h, d, cache))
    return compute_metrics(all_histories), all_histories


def run_per_archetype(
    fitness_model: Dict,
    strategy: str,
    n_per: int = 125,
    contraction_rate: float = 0.12,
) -> Dict[str, Dict]:
    """Run per-archetype analysis."""
    results = {}
    for arch_name in ARCHETYPE_NAMES:
        histories = []
        for _ in range(n_per):
            pool = build_pool()
            h, d, cache = tag_gravity_draft(
                pool, arch_name, fitness_model, strategy,
                contraction_rate=contraction_rate,
            )
            histories.append((h, d, cache))
        results[arch_name] = compute_metrics(histories)
    return results


# ============================================================
# V1 Measurement: visible-only baseline
# ============================================================
def tag_gravity_visible_only_draft(
    pool: List[SimCard],
    player_archetype: str,
    fitness_model: Dict,
    strategy: str,
    contraction_rate: float = 0.12,
) -> Tuple[List[Dict], List[SimCard], Dict[int, bool]]:
    """
    Tag-Gravity with hidden tags stripped (V1 measurement).
    Relevance is 100% visible dot-product; no archetype inference.
    """
    active_pool = list(pool)
    signature = {r: 0.0 for r in RESONANCE_TYPES}
    drafted: List[SimCard] = []
    history: List[Dict] = []

    sa_cache = precompute_sa_tiers(pool, player_archetype, fitness_model)

    for pick in range(1, NUM_PICKS + 1):
        if len(active_pool) < PACK_SIZE:
            break

        sig_magnitude = math.sqrt(sum(v**2 for v in signature.values()))

        # Floor slot from pick 3 (uses visible relevance only)
        if pick >= 3 and len(active_pool) >= PACK_SIZE:
            scored = []
            for c in active_pool:
                if c.is_generic:
                    rel = 0.5
                else:
                    rel = compute_visible_dot(c, signature, sig_magnitude)
                scored.append((rel, c))
            scored.sort(key=lambda x: -x[0])
            quartile_cutoff = max(1, len(scored) // 4)
            top_quartile = [c for _, c in scored[:quartile_cutoff]]
            floor_card = random.choice(top_quartile)
            remaining = [c for c in active_pool if c.id != floor_card.id]
            if len(remaining) >= PACK_SIZE - 1:
                random_slots = random.sample(remaining, PACK_SIZE - 1)
            else:
                random_slots = remaining
            pack = [floor_card] + random_slots
        else:
            pack = random.sample(active_pool, min(PACK_SIZE, len(active_pool)))

        chosen = select_card(pack, player_archetype, signature, strategy, pick, sa_cache)
        drafted.append(chosen)

        for i, sym in enumerate(chosen.visible_symbols):
            signature[sym] += 2.0 if i == 0 else 1.0

        sa_count = sum(1 for c in pack if sa_cache[c.id])
        history.append({
            "pick": pick, "pack": pack, "chosen": chosen,
            "pool_size": len(active_pool), "sa_count": sa_count,
        })

        # Contraction using visible relevance only
        if pick >= 4:
            sig_magnitude_post = math.sqrt(sum(v**2 for v in signature.values()))
            n_remove = max(1, int(len(active_pool) * contraction_rate))
            scored = []
            for c in active_pool:
                if c.is_generic:
                    rel = 0.5
                else:
                    rel = compute_visible_dot(c, signature, sig_magnitude_post)
                scored.append((rel, c))
            scored.sort(key=lambda x: x[0])
            to_remove = {c.id for _, c in scored[:n_remove]}
            active_pool = [c for c in active_pool if c.id not in to_remove]

    return history, drafted, sa_cache


def run_visible_only(
    fitness_model: Dict,
    strategy: str,
    n_drafts: int = 500,
    contraction_rate: float = 0.12,
) -> Dict:
    """Run visible-only baseline for V1 measurement."""
    all_histories = []
    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        pool = build_pool()
        h, d, cache = tag_gravity_visible_only_draft(
            pool, arch_name, fitness_model, strategy,
            contraction_rate=contraction_rate,
        )
        all_histories.append((h, d, cache))
    return compute_metrics(all_histories)


# ============================================================
# Draft Trace Formatter
# ============================================================
def format_trace(
    history: List[Dict],
    drafted: List[SimCard],
    sa_cache: Dict[int, bool],
    player_archetype: str,
) -> str:
    lines = [f"=== Draft Trace: {player_archetype} ==="]
    for h in history:
        pick = h["pick"]
        sa = h["sa_count"]
        pool_sz = h["pool_size"]
        chosen = h["chosen"]
        chosen_sa = "S/A" if sa_cache[chosen.id] else "C/F"
        sym_str = "/".join(chosen.visible_symbols) if chosen.visible_symbols else "Generic"
        tag_str = ARCHETYPE_NAMES[chosen.hidden_tag] if chosen.hidden_tag < 8 else "Generic"
        inf = h.get("inferred_arch", None)
        inf_str = f" infer={inf}" if inf else ""
        lines.append(
            f"  Pick {pick:2d}: pool={pool_sz:3d}, S/A={sa}, "
            f"[{chosen.archetype}:{sym_str}|tag:{tag_str}] ({chosen_sa}){inf_str}"
        )
    sa_d = sum(1 for c in drafted if sa_cache[c.id])
    lines.append(f"  Final: {sa_d}/{len(drafted)} S/A = {sa_d / max(1, len(drafted)) * 100:.0f}%")
    return "\n".join(lines)


# ============================================================
# Main
# ============================================================
def main():
    random.seed(42)

    print("=" * 70)
    print("TAG-GRAVITY (60/40) SIMULATION — Design 2, V9 Round 4")
    print("=" * 70)

    fitness_grad = FITNESS_MODELS["Graduated"]
    fitness_pess = FITNESS_MODELS["Pessimistic"]

    # =====================================================
    # Primary: Graduated Realistic, all 3 strategies
    # =====================================================
    print("\n" + "=" * 70)
    print("GRADUATED REALISTIC FITNESS — All Strategies")
    print("=" * 70)

    results_grad = {}
    all_histories_grad = {}
    for strat in ["committed", "power", "signal"]:
        m, h = run_aggregate(fitness_grad, strat)
        results_grad[strat] = m
        all_histories_grad[strat] = h
        print(f"\nStrategy: {strat}")
        print(f"  M1={m['M1']:.2f}  M2={m['M2']:.2f}  M3={m['M3']:.2f}  "
              f"M4={m['M4']:.2f}  M5={m['M5']:.1f}")
        print(f"  M6={m['M6']:.2f}  M7={m['M7']:.3f}  M9={m['M9']:.2f}  "
              f"M10={m['M10']:.1f}  M11={m['M11']:.2f}")
        pq = m['pack_pcts']
        print(f"  Pack pcts (P10/P25/P50/P75/P90): "
              f"{pq[10]}/{pq[25]}/{pq[50]}/{pq[75]}/{pq[90]}")

    # =====================================================
    # Secondary: Pessimistic, committed strategy
    # =====================================================
    print("\n" + "=" * 70)
    print("PESSIMISTIC FITNESS — Committed Strategy")
    print("=" * 70)

    m_pess, _ = run_aggregate(fitness_pess, "committed")
    print(f"  M3={m_pess['M3']:.2f}  M10={m_pess['M10']:.1f}  M11={m_pess['M11']:.2f}  "
          f"M6={m_pess['M6']:.2f}")
    pq = m_pess['pack_pcts']
    print(f"  Pack pcts: P10={pq[10]} P25={pq[25]} P50={pq[50]} P75={pq[75]} P90={pq[90]}")

    # =====================================================
    # Per-archetype: Graduated, committed
    # =====================================================
    print("\n" + "=" * 70)
    print("PER-ARCHETYPE: Graduated Realistic, committed")
    print("=" * 70)

    pa_results = run_per_archetype(fitness_grad, "committed")
    print(f"  {'Archetype':<16} {'M3':>6} {'M5':>6} {'M6':>6} {'M9':>6} {'M10':>6} {'M11':>6}")
    m3_vals = []
    for arch in ARCHETYPE_NAMES:
        m = pa_results[arch]
        m3_vals.append(m['M3'])
        print(f"  {arch:<16} {m['M3']:6.2f} {m['M5']:6.1f} {m['M6']:6.2f} "
              f"{m['M9']:6.2f} {m['M10']:6.1f} {m['M11']:6.2f}")
    spread = max(m3_vals) - min(m3_vals)
    print(f"\n  M3 spread (max-min): {spread:.3f}")
    print(f"  M3 worst archetype: {ARCHETYPE_NAMES[m3_vals.index(min(m3_vals))]}: "
          f"{min(m3_vals):.2f}")
    print(f"  M3 best archetype:  {ARCHETYPE_NAMES[m3_vals.index(max(m3_vals))]}: "
          f"{max(m3_vals):.2f}")

    # =====================================================
    # V1 measurement: visible-only vs full Tag-Gravity
    # =====================================================
    print("\n" + "=" * 70)
    print("V1 MEASUREMENT: Visible-Only vs Tag-Gravity (Graduated, committed, 500 drafts)")
    print("=" * 70)

    m_vis = run_visible_only(fitness_grad, "committed", n_drafts=500)
    m_full_500, _ = run_aggregate(fitness_grad, "committed", n_drafts=500)
    baseline_random = 0.5  # random selection from uncontracted pool

    v1_numerator = m_vis['M3'] - baseline_random
    v1_denominator = m_full_500['M3'] - baseline_random
    v1_pct = v1_numerator / v1_denominator if v1_denominator > 0 else 0

    print(f"  Random baseline M3:        {baseline_random:.3f}")
    print(f"  Visible-only M3:           {m_vis['M3']:.3f}")
    print(f"  Full Tag-Gravity M3:       {m_full_500['M3']:.3f}")
    print(f"  V1 (visible contribution): {v1_pct * 100:.1f}%")
    print(f"  V2 (hidden info):          3 bits/card (1 of 8 archetype tags)")

    # V4: power-chaser gap
    m_committed_500 = m_full_500
    m_power_500, _ = run_aggregate(fitness_grad, "power", n_drafts=500)
    v4_gap = m_committed_500['M3'] - m_power_500['M3']
    print(f"\n  V4 power-chaser gap:       {v4_gap:.3f} (target >= 0.40)")
    print(f"  Committed M3: {m_committed_500['M3']:.3f}  Power M3: {m_power_500['M3']:.3f}")

    # =====================================================
    # Pack quality distribution (Graduated, committed)
    # =====================================================
    print("\n" + "=" * 70)
    print("PACK QUALITY DISTRIBUTION (picks 6+, Graduated, committed)")
    print("=" * 70)
    m = results_grad["committed"]
    pq = m["pack_pcts"]
    print(f"  P10={pq[10]}  P25={pq[25]}  P50={pq[50]}  P75={pq[75]}  P90={pq[90]}")
    print(f"  Avg consecutive bad packs (<1.5 S/A): {m['avg_consec_bad']:.2f}")
    print(f"  Worst consecutive bad: {m['worst_consec_bad']}")

    # Consecutive bad pack distribution
    consec_dist = Counter()
    for h, d, cache in all_histories_grad["committed"]:
        post_sa = [hh["sa_count"] for hh in h[5:]]
        max_c = 0
        cur_c = 0
        for sa in post_sa:
            if sa < 1.5:
                cur_c += 1
                max_c = max(max_c, cur_c)
            else:
                cur_c = 0
        consec_dist[max_c] += 1

    print(f"\n  Max consecutive bad pack distribution:")
    for k in sorted(consec_dist.keys())[:8]:
        pct = consec_dist[k] / sum(consec_dist.values()) * 100
        print(f"    {k}: {consec_dist[k]} drafts ({pct:.1f}%)")

    # =====================================================
    # Draft traces
    # =====================================================
    print("\n" + "=" * 70)
    print("DRAFT TRACES")
    print("=" * 70)

    # Trace 1: Committed player, Warriors, Graduated
    random.seed(100)
    pool_t1 = build_pool()
    h1, d1, c1 = tag_gravity_draft(
        pool_t1, "Warriors", fitness_grad, "committed"
    )
    print("\n" + format_trace(h1, d1, c1, "Warriors"))

    # Trace 2: Signal reader, Flash, Graduated
    random.seed(200)
    pool_t2 = build_pool()
    h2, d2, c2 = tag_gravity_draft(
        pool_t2, "Flash", fitness_grad, "signal"
    )
    print("\n" + format_trace(h2, d2, c2, "Flash"))

    # =====================================================
    # V8 comparison table
    # =====================================================
    print("\n" + "=" * 70)
    print("V8 COMPARISON")
    print("=" * 70)
    m_tag = results_grad["committed"]
    print(f"{'Algorithm':<40} {'M3':>6} {'M10':>6} {'M11':>6} {'M6':>6}")
    print(f"{'V8 Narrative Gravity (40% pool)':<40} {'2.75':>6} {'3.3':>6} {'~2.8':>6} {'0.85':>6}")
    print(f"{'V8 SF+Bias R1 (V7 15% pool)':<40} {'2.24':>6} {'8.0':>6} {'~2.4':>6} {'0.75':>6}")
    print(f"{'V8 CSCT (V7 pool, disqualified)':<40} {'2.92':>6} {'2.0':>6} {'~3.0':>6} {'0.99':>6}")
    print(f"{'Design 2 Tag-Gravity (10% pool)':<40} "
          f"{m_tag['M3']:6.2f} {m_tag['M10']:6.1f} {m_tag['M11']:6.2f} {m_tag['M6']:6.2f}")

    # =====================================================
    # Scorecard summary
    # =====================================================
    print("\n" + "=" * 70)
    print("SCORECARD SUMMARY")
    print("=" * 70)

    m = results_grad["committed"]
    targets = [
        ("M1 (early variety >= 3.0)",    m['M1'],  3.0,  ">="),
        ("M2 (early S/A <= 2.0)",         m['M2'],  2.0,  "<="),
        ("M3 (late S/A >= 2.0)",          m['M3'],  2.0,  ">="),
        ("M4 (off-arch >= 0.5)",          m['M4'],  0.5,  ">="),
        ("M5 (convergence pick 5-8)",     m['M5'],  8.0,  "<="),
        ("M6 (concentration 60-90%)",     m['M6'],  0.60, ">="),
        ("M7 (variety < 40% overlap)",    m['M7'],  0.40, "<="),
        ("M9 (stddev >= 0.8)",            m['M9'],  0.8,  ">="),
        ("M10 (consec bad <= 2)",         m['M10'], 2.0,  "<="),
        ("M11 (late S/A >= 3.0)",         m['M11'], 3.0,  ">="),
    ]

    for name, val, target, direction in targets:
        if direction == ">=":
            passed = val >= target
        else:
            passed = val <= target
        status = "PASS" if passed else "FAIL"
        print(f"  {name:<35} {val:6.2f}  {status}")

    print(f"\n  Pessimistic M3: {m_pess['M3']:.2f}  M10: {m_pess['M10']:.1f}  "
          f"M11: {m_pess['M11']:.2f}")

    print("\nDone.")


if __name__ == "__main__":
    main()
