"""
Simulation Agent 2: Design 6 -- Anchor-Scaled Contraction
==========================================================

Algorithm per design_6.md (champion: Proposal A):

CONTRACTION RATES (pick-type-scaled):
  - 6%  per pick for generic pick (0 visible symbols)
  - 10% per pick for single-symbol pick
  - 18% per pick for dual-resonance anchor pick

RELEVANCE SCORE:
  - 60% visible dot-product (normalized cosine with player signature)
  - 40% hidden archetype tag match:
      * 1.0 if card's hidden_tag == resolved player archetype
      * 0.5 if player archetype unresolved (neutral)
      * 0.0 if mismatch
  - Generics: max(computed, 0.5) floor

ARCHETYPE INFERENCE:
  The design spec says "first time a pair accumulates >= 3 total weight" in the
  resonance signature. This is the visible signature (not hidden tags). The hidden
  tags are used in the *contraction relevance*, not for inference. The inference
  uses the dominant resonance pair from the visible signature.
  Disambiguation between same-primary siblings (e.g. Warriors vs Sacrifice, both
  Tide-primary) uses the hidden tags of picked dual-resonance cards as a
  tiebreaker: if the player has accumulated >= 2 dual-res picks, the majority
  hidden tag among them decides the home archetype within the Tide pair.

FLOOR SLOT: from pick 3, one of four pack slots draws from top-quartile
  relevance subset of surviving pool.

POOL: 360 cards per spec:
  - 36 dual-res (~10%): 5/5/4/4/4/4/5/5 per archetype
  - 284 single-symbol (~79%)
  - 40 generics (~11%)
  - Hidden tag: 0-7 (archetype index) for non-generics; 8 (neutral) for generics
"""

import random
import math
from dataclasses import dataclass
from collections import Counter
from typing import Optional

# ============================================================
# Constants
# ============================================================
NUM_DRAFTS = 1000
NUM_PICKS  = 30
PACK_SIZE  = 4
POOL_SIZE  = 360
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
ARCH_BY_NAME    = {a[0]: a for a in ARCHETYPES}
ARCH_INDEX      = {a[0]: i for i, a in enumerate(ARCHETYPES)}

# Map (primary, secondary) -> archetype index
PAIR_TO_ARCH_IDX = {(a[1], a[2]): ARCH_INDEX[a[0]] for a in ARCHETYPES}
# Map resonance -> list of archetype indices that have it as primary
PRIMARY_RES_TO_ARCHS = {r: [] for r in RESONANCE_TYPES}
for a in ARCHETYPES:
    PRIMARY_RES_TO_ARCHS[a[1]].append(ARCH_INDEX[a[0]])

# Graduated Realistic fitness (sibling A-tier rate per pair)
SIBLING_RATES = {
    ("Warriors",     "Sacrifice"):    0.50,
    ("Sacrifice",    "Warriors"):     0.50,
    ("Self-Discard", "Self-Mill"):    0.40,
    ("Self-Mill",    "Self-Discard"): 0.40,
    ("Blink",        "Storm"):        0.30,
    ("Storm",        "Blink"):        0.30,
    ("Flash",        "Ramp"):         0.25,
    ("Ramp",         "Flash"):        0.25,
}

PESSIMISTIC_RATES = {
    ("Warriors",     "Sacrifice"):    0.35,
    ("Sacrifice",    "Warriors"):     0.35,
    ("Self-Discard", "Self-Mill"):    0.25,
    ("Self-Mill",    "Self-Discard"): 0.25,
    ("Blink",        "Storm"):        0.15,
    ("Storm",        "Blink"):        0.15,
    ("Flash",        "Ramp"):         0.10,
    ("Ramp",         "Flash"):        0.10,
}


def get_sibling(arch_name):
    r1 = ARCH_BY_NAME[arch_name][1]
    for a in ARCHETYPES:
        if a[0] != arch_name and a[1] == r1:
            return a[0]
    return None


# ============================================================
# Card & Pool
# ============================================================
@dataclass
class SimCard:
    id: int
    visible_symbols: list   # 0, 1, or 2 resonance symbols (player-facing)
    archetype: str          # primary archetype label (evaluation only)
    hidden_tag: int         # 0-7 = ARCH_INDEX; 8 = neutral/generic
    power: float
    is_generic: bool = False


def build_pool():
    """Build 360-card V9 pool per design_6.md pool specification."""
    dual_res_per_arch = {
        "Flash": 5, "Blink": 5, "Storm": 4, "Self-Discard": 4,
        "Self-Mill": 4, "Sacrifice": 4, "Warriors": 5, "Ramp": 5,
    }
    cards = []
    card_id = 0
    for arch_name, r1, r2 in ARCHETYPES:
        n_dual   = dual_res_per_arch[arch_name]
        n_single = CARDS_PER_ARCHETYPE - n_dual
        tag      = ARCH_INDEX[arch_name]
        for _ in range(n_single):
            cards.append(SimCard(
                id=card_id, visible_symbols=[r1],
                archetype=arch_name, hidden_tag=tag,
                power=random.uniform(4, 8)))
            card_id += 1
        for _ in range(n_dual):
            cards.append(SimCard(
                id=card_id, visible_symbols=[r1, r2],
                archetype=arch_name, hidden_tag=tag,
                power=random.uniform(4, 8)))
            card_id += 1
    for _ in range(GENERIC_CARDS):
        cards.append(SimCard(
            id=card_id, visible_symbols=[],
            archetype="Generic", hidden_tag=8,
            power=random.uniform(3, 7), is_generic=True))
        card_id += 1
    return cards


def precompute_card_tiers(pool, player_archetype, fitness_rates):
    """Pre-roll S/A tier for all cards for this draft."""
    sa_map  = {}
    sibling = get_sibling(player_archetype)
    for c in pool:
        if c.is_generic:
            sa_map[c.id] = False
        elif c.archetype == player_archetype:
            sa_map[c.id] = True
        elif c.archetype == sibling:
            rate = fitness_rates.get((player_archetype, sibling), 0.0)
            sa_map[c.id] = (random.random() < rate)
        else:
            sa_map[c.id] = False
    return sa_map


# ============================================================
# Design 6 Core: Relevance + Archetype Inference
# ============================================================

def compute_visible_score(card, signature, sig_magnitude):
    """Normalized cosine similarity between card visible vector and signature."""
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


def compute_relevance(card, signature, sig_magnitude, resolved_arch_idx,
                      hidden_mode=True):
    """
    60% visible + 40% hidden relevance score.
    hidden_mode=False strips hidden contribution (V1 measurement).
    """
    if card.is_generic:
        return 0.5
    vis = compute_visible_score(card, signature, sig_magnitude)
    if not hidden_mode:
        hid = 0.5
    elif resolved_arch_idx is None:
        hid = 0.5
    else:
        hid = 1.0 if card.hidden_tag == resolved_arch_idx else 0.0
    return 0.6 * vis + 0.4 * hid


def infer_archetype(signature, dual_res_picked):
    """
    Resolve player archetype from visible signature + dual-res card tags.

    Step 1: Check if any resonance pair has accumulated >= 3 total weight.
            Use top-2 resonances in the signature as the candidate pair.
    Step 2: If both possible orderings give a valid archetype, use the
            secondary resonance to disambiguate via the most-secondary-signal.
    Step 3: If player has >= 2 dual-res picks, use majority hidden tag to
            disambiguate between same-primary siblings.

    Returns arch_idx (0-7) or None.
    """
    sorted_res = sorted(RESONANCE_TYPES, key=lambda r: signature[r], reverse=True)
    top1, top2 = sorted_res[0], sorted_res[1]

    pair_weight = signature[top1] + signature[top2]
    if pair_weight < 3.0:
        return None

    # Try (top1, top2) first -- top1 as primary
    if (top1, top2) in PAIR_TO_ARCH_IDX:
        candidate_a = PAIR_TO_ARCH_IDX[(top1, top2)]
    else:
        candidate_a = None

    # Try (top2, top1) -- top2 as primary
    if (top2, top1) in PAIR_TO_ARCH_IDX:
        candidate_b = PAIR_TO_ARCH_IDX[(top2, top1)]
    else:
        candidate_b = None

    # If unambiguous
    if candidate_a is not None and candidate_b is None:
        return candidate_a
    if candidate_b is not None and candidate_a is None:
        return candidate_b

    # Ambiguous (e.g. Tide+Zephyr could be Warriors or Ramp).
    # Use the stronger primary signal to pick
    if signature[top1] >= signature[top2]:
        if candidate_a is not None:
            primary_candidate = candidate_a
        else:
            primary_candidate = candidate_b
    else:
        if candidate_b is not None:
            primary_candidate = candidate_b
        else:
            primary_candidate = candidate_a

    # Disambiguate with dual-res picks' hidden tags if available
    if len(dual_res_picked) >= 2:
        counts = Counter(dual_res_picked)
        best_tag = counts.most_common(1)[0][0]
        # Check if best_tag matches one of the candidates
        if candidate_a is not None and best_tag == candidate_a:
            return candidate_a
        if candidate_b is not None and best_tag == candidate_b:
            return candidate_b

    return primary_candidate


def contraction_rate(card):
    """6% generic, 10% single-symbol, 18% dual-resonance."""
    n = len(card.visible_symbols)
    if n == 0:
        return 0.06
    elif n == 1:
        return 0.10
    else:
        return 0.18


# ============================================================
# Draft Engine
# ============================================================

def anchor_scaled_draft(pool, player_archetype, fitness_rates, strategy,
                        hidden_mode=True):
    """
    Run one draft using Design 6: Anchor-Scaled Contraction.
    hidden_mode=False: visible-only (no hidden tag influence) for V1 measurement.
    """
    active_pool     = list(pool)
    signature       = {r: 0.0 for r in RESONANCE_TYPES}
    resolved_arch   = None
    dual_res_tags   = []   # hidden tags of dual-res cards picked (for disambiguation)
    drafted         = []
    history         = []

    sa_cache = precompute_card_tiers(pool, player_archetype, fitness_rates)

    for pick in range(1, NUM_PICKS + 1):
        if len(active_pool) < PACK_SIZE:
            break

        sig_mag = math.sqrt(sum(v**2 for v in signature.values()))
        eff_arch = resolved_arch if hidden_mode else None

        # Build pack: floor slot from pick 3
        if pick >= 3 and len(active_pool) >= PACK_SIZE:
            scored_pool = sorted(
                active_pool,
                key=lambda c: compute_relevance(c, signature, sig_mag, eff_arch,
                                                hidden_mode),
                reverse=True,
            )
            top_q = max(1, len(scored_pool) // 4)
            floor_card = random.choice(scored_pool[:top_q])
            remaining  = [c for c in active_pool if c.id != floor_card.id]
            rand_cards = (random.sample(remaining, PACK_SIZE - 1)
                          if len(remaining) >= PACK_SIZE - 1
                          else list(remaining))
            pack = rand_cards + [floor_card]
        else:
            pack = random.sample(active_pool, PACK_SIZE)

        # Pick a card
        chosen = select_card(pack, player_archetype, signature, strategy,
                             pick, sa_cache)
        drafted.append(chosen)

        # Update visible signature
        for i, sym in enumerate(chosen.visible_symbols):
            signature[sym] += 2.0 if i == 0 else 1.0

        # Track dual-res picks for disambiguation (using hidden tags)
        if hidden_mode and len(chosen.visible_symbols) == 2:
            if not chosen.is_generic and chosen.hidden_tag < 8:
                dual_res_tags.append(chosen.hidden_tag)

        # Attempt archetype resolution each pick (idempotent once resolved)
        if resolved_arch is None:
            resolved_arch = infer_archetype(signature, dual_res_tags)

        sa_count = sum(1 for c in pack if sa_cache[c.id])
        history.append({
            "pick":         pick,
            "pack":         pack,
            "chosen":       chosen,
            "pool_size":    len(active_pool),
            "sa_count":     sa_count,
            "resolved_arch": resolved_arch,
        })

        # Contract pool
        rate     = contraction_rate(chosen)
        n_remove = max(1, int(len(active_pool) * rate))

        sig_mag2 = math.sqrt(sum(v**2 for v in signature.values()))
        scored   = sorted(
            active_pool,
            key=lambda c: compute_relevance(c, signature, sig_mag2,
                                            eff_arch if resolved_arch is None
                                            else (resolved_arch if hidden_mode else None),
                                            hidden_mode),
        )
        to_remove  = {c.id for c in scored[:n_remove]}
        active_pool = [c for c in active_pool if c.id not in to_remove]

    return history, drafted, sa_cache


def select_card(pack, player_archetype, signature, strategy, pick, sa_cache):
    """Card selection by player strategy."""
    arch    = ARCH_BY_NAME[player_archetype]
    r1, r2  = arch[1], arch[2]

    if strategy == "committed":
        def score(c):
            s = 10 if sa_cache.get(c.id, False) else 0
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
        def score_sig(c):
            s = 5 if sa_cache.get(c.id, False) else 0
            for i, sym in enumerate(c.visible_symbols):
                if sym == top_res:
                    s += 3 if i == 0 else 1
            s += c.power * 0.1
            return s
        return max(pack, key=score_sig)

    return random.choice(pack)


# ============================================================
# Metrics M1-M11
# ============================================================

def compute_m11(history, sa_cache):
    late = [sum(1 for c in h["pack"] if sa_cache[c.id])
            for h in history if h["pick"] >= 15]
    return sum(late) / len(late) if late else 0.0


def compute_metrics(all_histories):
    m1, m2, m3, m4 = [], [], [], []
    m5, m6, m9, m10, m11 = [], [], [], [], []
    post_sa_all = []

    for history, drafted, sa_cache in all_histories:
        # M1: picks 1-5 avg unique archetypes per pack
        early_arch = [len({c.archetype for c in h["pack"] if not c.is_generic})
                      for h in history[:5]]
        m1.append(sum(early_arch) / max(1, len(early_arch)))

        # M2: picks 1-5 avg S/A per pack
        early_sa = [sum(1 for c in h["pack"] if sa_cache[c.id])
                    for h in history[:5]]
        m2.append(sum(early_sa) / max(1, len(early_sa)))

        # M3: picks 6+ avg S/A per pack
        post_sa = [h["sa_count"] for h in history[5:]]
        post_sa_all.extend(post_sa)
        if post_sa:
            m3.append(sum(post_sa) / len(post_sa))

        # M4: picks 6+ avg off-archetype per pack
        post_off = [sum(1 for c in h["pack"] if not sa_cache[c.id])
                    for h in history[5:]]
        if post_off:
            m4.append(sum(post_off) / len(post_off))

        # M5: first pick where 3-pick rolling avg >= 1.5
        conv = NUM_PICKS
        for i in range(2, len(history)):
            w = [history[j]["sa_count"] for j in range(i - 2, i + 1)]
            if sum(w) / 3 >= 1.5:
                conv = history[i]["pick"]
                break
        m5.append(conv)

        # M6: deck S/A concentration
        sa_d = sum(1 for c in drafted if sa_cache[c.id])
        m6.append(sa_d / max(1, len(drafted)))

        # M9: StdDev of S/A picks 6+
        if len(post_sa) > 1:
            mu  = sum(post_sa) / len(post_sa)
            var = sum((x - mu)**2 for x in post_sa) / len(post_sa)
            m9.append(math.sqrt(var))

        # M10: avg max consecutive packs with S/A < 1.5 (picks 6+)
        max_c = cur_c = 0
        for sa in post_sa:
            cur_c = cur_c + 1 if sa < 1.5 else 0
            max_c = max(max_c, cur_c)
        m10.append(max_c)

        # M11
        m11.append(compute_m11(history, sa_cache))

    # M7: run-to-run card overlap
    m7 = []
    for i in range(1, len(all_histories)):
        a = {c.id for c in all_histories[i - 1][1]}
        b = {c.id for c in all_histories[i][1]}
        m7.append(len(a & b) / max(1, len(a | b)))

    pq = sorted(post_sa_all)
    n  = len(pq)
    pcts = {p: (pq[min(int(n * p / 100), n - 1)] if n > 0 else 0)
            for p in [10, 25, 50, 75, 90]}

    avg = lambda vs: sum(vs) / max(1, len(vs))
    return {
        "M1": avg(m1), "M2": avg(m2), "M3": avg(m3), "M4": avg(m4),
        "M5": avg(m5), "M6": avg(m6), "M7": avg(m7), "M9": avg(m9),
        "M10": avg(m10), "M10_max": max(m10) if m10 else 0,
        "M11": avg(m11), "pack_pcts": pcts,
    }


# ============================================================
# Runners
# ============================================================

def run_aggregate(fitness_rates, strategy, n_drafts=NUM_DRAFTS, hidden_mode=True):
    all_h = []
    for i in range(n_drafts):
        arch = ARCHETYPE_NAMES[i % 8]
        pool = build_pool()
        h, d, cache = anchor_scaled_draft(
            pool, arch, fitness_rates, strategy, hidden_mode=hidden_mode)
        all_h.append((h, d, cache))
    return compute_metrics(all_h), all_h


def run_per_archetype(fitness_rates, strategy, n_per=125, hidden_mode=True):
    results = {}
    for arch in ARCHETYPE_NAMES:
        histories = []
        for _ in range(n_per):
            pool = build_pool()
            h, d, cache = anchor_scaled_draft(
                pool, arch, fitness_rates, strategy, hidden_mode=hidden_mode)
            histories.append((h, d, cache))
        results[arch] = compute_metrics(histories)
    return results


# ============================================================
# Trace formatter
# ============================================================

def format_trace(history, drafted, sa_cache, player_archetype):
    lines = [f"=== Draft Trace: {player_archetype} ==="]
    for h in history:
        pick    = h["pick"]
        sa      = h["sa_count"]
        psz     = h["pool_size"]
        chosen  = h["chosen"]
        csa     = "S/A" if sa_cache.get(chosen.id, False) else "C/F"
        sym_str = "/".join(chosen.visible_symbols) if chosen.visible_symbols else "Generic"
        ra      = h.get("resolved_arch")
        alabel  = ARCHETYPE_NAMES[ra] if ra is not None else "?"
        lines.append(
            f"  Pick {pick:2d}: pool={psz:3d}, pack_SA={sa}, "
            f"[{chosen.archetype}:{sym_str}]({len(chosen.visible_symbols)}sym) "
            f"{csa} arch={alabel}"
        )
    sa_d = sum(1 for c in drafted if sa_cache.get(c.id, False))
    lines.append(f"  Final: {sa_d}/{len(drafted)} S/A = {sa_d / max(1, len(drafted)) * 100:.0f}%")
    return "\n".join(lines)


# ============================================================
# Main
# ============================================================

def main():
    random.seed(42)

    print("=" * 72)
    print("DESIGN 6: ANCHOR-SCALED CONTRACTION -- Simulation Agent 2")
    print("=" * 72)

    # Primary runs
    print("\n[1/6] Graduated Realistic, committed (1000 drafts)...")
    m_gc, hist_gc = run_aggregate(SIBLING_RATES, "committed")

    print("[2/6] Graduated Realistic, power (1000 drafts)...")
    m_gp, _ = run_aggregate(SIBLING_RATES, "power")

    print("[3/6] Graduated Realistic, signal (1000 drafts)...")
    m_gs, _ = run_aggregate(SIBLING_RATES, "signal")

    print("[4/6] Pessimistic, committed (1000 drafts)...")
    m_pc, _ = run_aggregate(PESSIMISTIC_RATES, "committed")

    print("[5/6] V1 baseline: visible-only (1000 drafts)...")
    m_vis, _ = run_aggregate(SIBLING_RATES, "committed", hidden_mode=False)

    print("[6/6] Per-archetype analysis (125 per archetype)...")
    per_arch = run_per_archetype(SIBLING_RATES, "committed", n_per=125)

    # Draft traces
    random.seed(101)
    pool_t1 = build_pool()
    h1, d1, c1 = anchor_scaled_draft(pool_t1, "Warriors", SIBLING_RATES, "committed")

    random.seed(303)
    pool_t2 = build_pool()
    h2, d2, c2 = anchor_scaled_draft(pool_t2, "Flash", SIBLING_RATES, "signal")

    # ==========================
    # PRINT RESULTS
    # ==========================

    print("\n" + "=" * 72)
    print("STRATEGY COMPARISON (Graduated Realistic, 1000 drafts each)")
    print("=" * 72)
    print(f"{'Strategy':<12} {'M3':>5} {'M11':>5} {'M5':>5} {'M6':>7} "
          f"{'M9':>5} {'M10':>6}")
    for label, m in [("committed", m_gc), ("power", m_gp), ("signal", m_gs)]:
        print(f"{label:<12} {m['M3']:5.2f} {m['M11']:5.2f} {m['M5']:5.1f} "
              f"{m['M6']:7.1%} {m['M9']:5.2f} {m['M10']:6.2f}")

    print("\n" + "=" * 72)
    print("PER-ARCHETYPE: M3, M11, M5, M6, M10 (Graduated, committed, 125 drafts ea.)")
    print("=" * 72)
    print(f"{'Archetype':<16} {'M3':>5} {'M11':>5} {'M5':>5} {'M6':>7} {'M10':>5}")
    for arch in ARCHETYPE_NAMES:
        m = per_arch[arch]
        print(f"{arch:<16} {m['M3']:5.2f} {m['M11']:5.2f} {m['M5']:5.1f} "
              f"{m['M6']:7.1%} {m['M10']:5.2f}")
    worst_m3 = min(per_arch[a]["M3"] for a in ARCHETYPE_NAMES)
    best_m3  = max(per_arch[a]["M3"] for a in ARCHETYPE_NAMES)
    print(f"  Range: worst={worst_m3:.2f}  best={best_m3:.2f}  "
          f"spread={best_m3 - worst_m3:.2f}")

    print("\n" + "=" * 72)
    print("PACK QUALITY DISTRIBUTION (picks 6+, Graduated, committed)")
    print("=" * 72)
    pq = m_gc["pack_pcts"]
    print(f"P10={pq[10]}  P25={pq[25]}  P50={pq[50]}  P75={pq[75]}  P90={pq[90]}")

    print("\n" + "=" * 72)
    print("V1 MEASUREMENT")
    print("=" * 72)
    m3_full = m_gc["M3"]
    m3_vis  = m_vis["M3"]
    m3_rand = 0.5
    if m3_full > m3_rand:
        v1_pct = (m3_vis - m3_rand) / (m3_full - m3_rand) * 100
    else:
        v1_pct = 0.0
    print(f"M3 full={m3_full:.2f}  M3 visible-only={m3_vis:.2f}  baseline={m3_rand}")
    print(f"M11 full={m_gc['M11']:.2f}  M11 visible-only={m_vis['M11']:.2f}")
    print(f"V1 estimate (M3-based): {v1_pct:.0f}%  "
          f"(positive = visible does most of the work)")
    v2_bits = 3  # 3 bits per card (8 archetypes)
    print(f"V2: {v2_bits} bits/card ({POOL_SIZE * v2_bits} bits total)")
    v3 = 8
    print(f"V3: {v3}/10 (tag reflects genuine mechanical best-fit)")

    print("\n" + "=" * 72)
    print("V4 POWER-CHASER GAP")
    print("=" * 72)
    gap = m_gc["M3"] - m_gp["M3"]
    print(f"Committed M3={m_gc['M3']:.2f}  Power-chaser M3={m_gp['M3']:.2f}  Gap={gap:.2f}")
    print(f"Gap >= 0.4 required: {'PASS' if gap >= 0.4 else 'FAIL'}")

    print("\n" + "=" * 72)
    print("DRAFT TRACES")
    print("=" * 72)
    print("\n" + format_trace(h1, d1, c1, "Warriors (committed)"))
    print("\n" + format_trace(h2, d2, c2, "Flash (signal)"))

    print("\n" + "=" * 72)
    print("FULL SCORECARD: Graduated Realistic, committed")
    print("=" * 72)
    m = m_gc
    rows = [
        ("M1",  m["M1"],  ">= 3.0",  m["M1"]  >= 3.0),
        ("M2",  m["M2"],  "<= 2.0",  m["M2"]  <= 2.0),
        ("M3",  m["M3"],  ">= 2.0",  m["M3"]  >= 2.0),
        ("M4",  m["M4"],  ">= 0.5",  m["M4"]  >= 0.5),
        ("M5",  m["M5"],  "5-8",     5 <= m["M5"] <= 8),
        ("M6",  m["M6"],  "60-90%",  0.60 <= m["M6"] <= 0.90),
        ("M7",  m["M7"],  "< 0.40",  m["M7"]  < 0.40),
        ("M9",  m["M9"],  ">= 0.8",  m["M9"]  >= 0.8),
        ("M10", m["M10"], "<= 2",    m["M10"] <= 2.0),
        ("M11", m["M11"], ">= 3.0",  m["M11"] >= 3.0),
    ]
    print(f"{'Metric':<6} {'Value':>8} {'Target':>10} {'Status':>7}")
    print("-" * 36)
    for name, val, tgt, ok in rows:
        status = "PASS" if ok else "FAIL"
        if name in ("M6", "M7"):
            print(f"{name:<6} {val:8.1%} {tgt:>10} {status:>7}")
        else:
            print(f"{name:<6} {val:8.2f} {tgt:>10} {status:>7}")

    print(f"\nPessimistic: M3={m_pc['M3']:.2f}  M10={m_pc['M10']:.2f}  "
          f"M11={m_pc['M11']:.2f}")

    print("\n" + "=" * 72)
    print("V8 BASELINE COMPARISON")
    print("=" * 72)
    print(f"{'Algorithm':<34} {'Pool':>6} {'M3':>5} {'M11':>5} {'M10':>5} {'M6':>7}")
    print(f"{'Design 6 (Graduated)':<34} {'10%':>6} {m_gc['M3']:5.2f} "
          f"{m_gc['M11']:5.2f} {m_gc['M10']:5.2f} {m_gc['M6']:7.1%}")
    print(f"{'Design 6 (Pessimistic)':<34} {'10%':>6} {m_pc['M3']:5.2f} "
          f"{m_pc['M11']:5.2f} {m_pc['M10']:5.2f}   ---")
    print(f"{'V8 Narrative Gravity':<34} {'40%':>6} {'2.75':>5} {'n/a':>5} "
          f"{'3.3':>5} {'~75%':>7}")
    print(f"{'V8 SF+Bias R1':<34} {'15%':>6} {'2.24':>5} {'n/a':>5} "
          f"{'8.0':>5}    ---")
    print(f"{'V8 CSCT':<34} {'15%':>6} {'2.92':>5} {'n/a':>5} "
          f"{'2.0':>5} {'99%':>7}")

    print("\nDone.")


if __name__ == "__main__":
    main()
