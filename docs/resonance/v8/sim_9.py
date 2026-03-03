"""
Simulation Agent 9: Narrative Gravity
Progressive pool contraction -- removes cards whose resonance profile is distant
from the player's emerging identity. Subtractive approach borrowed from roguelike
shop systems.

Key insight from initial run: standard contraction (2-3% per pick) is far too
slow. The pool must contract aggressively to concentrate archetype-level S/A
cards. Testing standard, aggressive, and ultra-aggressive variants with
proper archetype-level evaluation.
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

# Co-primary sibling pairs (share same primary resonance)
def get_sibling(arch_name):
    """Return co-primary sibling archetype name."""
    r1 = ARCH_BY_NAME[arch_name][1]
    for a in ARCHETYPES:
        if a[0] != arch_name and a[1] == r1:
            return a[0]
    return None

# ============================================================
# Fitness Models (per co-primary pair)
# ============================================================
def make_fitness(rates_by_pair):
    """Create a full fitness dict from shorthand."""
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
    "Optimistic": make_fitness([1.0, 1.0, 1.0, 1.0]),
    "Graduated":  make_fitness([0.50, 0.40, 0.30, 0.25]),
    "Pessimistic": make_fitness([0.35, 0.25, 0.15, 0.10]),
    "Hostile":    make_fitness([0.08, 0.08, 0.08, 0.08]),
}

# ============================================================
# Card & Pool
# ============================================================
@dataclass
class SimCard:
    id: int
    symbols: list  # ordered resonance symbols
    archetype: str  # primary archetype (for evaluation)
    power: float
    is_generic: bool = False


def build_pool(dual_res_pct=0.15):
    """Build a 360-card pool."""
    cards = []
    card_id = 0

    for arch_name, r1, r2 in ARCHETYPES:
        n_arch = CARDS_PER_ARCHETYPE
        n_dual = int(n_arch * dual_res_pct)
        n_single = n_arch - n_dual

        for _ in range(n_single):
            cards.append(SimCard(
                id=card_id, symbols=[r1],
                archetype=arch_name,
                power=random.uniform(4, 8),
            ))
            card_id += 1

        for _ in range(n_dual):
            cards.append(SimCard(
                id=card_id, symbols=[r1, r2],
                archetype=arch_name,
                power=random.uniform(4, 8),
            ))
            card_id += 1

    for _ in range(GENERIC_CARDS):
        cards.append(SimCard(
            id=card_id, symbols=[],
            archetype="Generic",
            power=random.uniform(3, 7),
            is_generic=True,
        ))
        card_id += 1

    return cards


def precompute_card_tiers(pool, player_archetype, fitness_model):
    """Pre-roll S/A status for all cards (for consistency within a draft)."""
    sa_map = {}
    sibling = get_sibling(player_archetype)
    for c in pool:
        if c.is_generic:
            sa_map[c.id] = False  # B-tier generics are NOT S/A
        elif c.archetype == player_archetype:
            sa_map[c.id] = True  # S-tier
        elif c.archetype == sibling:
            rate = fitness_model.get((player_archetype, sibling), 0.0)
            sa_map[c.id] = (random.random() < rate)  # A-tier with probability
        else:
            sa_map[c.id] = False  # F-tier
    return sa_map


# ============================================================
# Narrative Gravity Algorithm
# ============================================================
def compute_relevance(card, signature, sig_magnitude):
    """Compute relevance score for pool contraction."""
    if card.is_generic:
        return 0.5  # Protected baseline

    if not card.symbols:
        return 0.0

    card_vec = {r: 0.0 for r in RESONANCE_TYPES}
    for i, sym in enumerate(card.symbols):
        if i == 0:
            card_vec[sym] += 2.0
        else:
            card_vec[sym] += 1.0

    card_mag = math.sqrt(sum(v**2 for v in card_vec.values()))
    if card_mag == 0 or sig_magnitude == 0:
        return 0.5

    dot = sum(card_vec[r] * signature[r] for r in RESONANCE_TYPES)
    return dot / (card_mag * sig_magnitude)


def narrative_gravity_draft(pool, player_archetype, fitness_model, strategy,
                            contraction_pct=0.05, contraction_start=4,
                            ramp_phase_end=8, ramp_pct=0.07):
    """
    Run one draft using Narrative Gravity.
    contraction_pct: fraction of pool removed per post-ramp pick
    ramp_pct: fraction of pool removed per ramp-phase pick (picks 4-8)
    """
    active_pool = list(pool)
    signature = {r: 0.0 for r in RESONANCE_TYPES}
    drafted = []
    history = []

    sa_cache = precompute_card_tiers(pool, player_archetype, fitness_model)

    for pick in range(1, NUM_PICKS + 1):
        if len(active_pool) < PACK_SIZE:
            break

        # Draw pack
        pack = random.sample(active_pool, min(PACK_SIZE, len(active_pool)))

        # Select card
        chosen = select_card(pack, player_archetype, signature, strategy,
                             pick, sa_cache)

        drafted.append(chosen)

        # Update signature
        for i, sym in enumerate(chosen.symbols):
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
        })

        # Contract pool
        if pick >= contraction_start:
            sig_magnitude = math.sqrt(sum(v**2 for v in signature.values()))

            # Adaptive: committed players contract slightly faster
            max_sig = max(signature.values())
            total_sig = sum(signature.values())
            commitment = max_sig / total_sig if total_sig > 0 else 0

            if pick <= ramp_phase_end:
                rate = ramp_pct + 0.02 * commitment
            else:
                rate = contraction_pct + 0.02 * commitment

            n_remove = max(1, int(len(active_pool) * rate))

            scored = []
            for c in active_pool:
                rel = compute_relevance(c, signature, sig_magnitude)
                scored.append((rel, c))
            scored.sort(key=lambda x: x[0])

            to_remove = set(c.id for _, c in scored[:n_remove])
            active_pool = [c for c in active_pool if c.id not in to_remove]

    return history, drafted, sa_cache


def select_card(pack, player_archetype, signature, strategy, pick, sa_cache):
    """Select a card from the pack based on strategy."""
    arch = ARCH_BY_NAME[player_archetype]
    r1, r2 = arch[1], arch[2]

    if strategy == "committed":
        def score(c):
            s = 0
            if sa_cache.get(c.id, False):
                s += 10
            for i, sym in enumerate(c.symbols):
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
            for i, sym in enumerate(c.symbols):
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
    """Compute M1-M10 from draft histories."""
    m1_vals, m2_vals, m3_vals, m4_vals = [], [], [], []
    m5_vals, m6_vals, m9_vals, m10_vals = [], [], [], []
    post_commit_sa = []
    consec_bad_list = []

    for history, drafted, sa_cache in all_histories:
        # M1: picks 1-5, unique archetypes represented per pack
        early_arch = []
        for h in history[:5]:
            archs = set()
            for c in h["pack"]:
                if not c.is_generic:
                    archs.add(c.archetype)
            early_arch.append(len(archs))
        m1_vals.append(sum(early_arch) / max(1, len(early_arch)))

        # M2: picks 1-5, S/A for emerging archetype per pack
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

        # M5: convergence pick
        conv_pick = NUM_PICKS
        for i in range(2, len(history)):
            window = [history[j]["sa_count"] for j in range(i-2, i+1)]
            if sum(window) / 3 >= 1.5:
                conv_pick = i + 1
                break
        m5_vals.append(conv_pick)

        # M6: deck concentration
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

    # M7: run-to-run card overlap
    m7_overlaps = []
    for i in range(1, len(all_histories)):
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
        "pack_pcts": pcts,
        "avg_consec_bad": avg(consec_bad_list),
        "worst_consec_bad": max(consec_bad_list) if consec_bad_list else 0,
    }


# ============================================================
# Runners
# ============================================================
def run_aggregate(pool_cfg, fitness_name, strategy, n_drafts=NUM_DRAFTS,
                  contraction_pct=0.05, ramp_pct=0.07):
    """Run aggregate drafts cycling through archetypes."""
    fitness_model = FITNESS_MODELS[fitness_name]
    all_histories = []

    for i in range(n_drafts):
        arch_name = ARCHETYPE_NAMES[i % 8]
        pool = build_pool(dual_res_pct=pool_cfg["dual_res_pct"])
        h, d, cache = narrative_gravity_draft(
            pool, arch_name, fitness_model, strategy,
            contraction_pct=contraction_pct,
            ramp_pct=ramp_pct,
        )
        all_histories.append((h, d, cache))

    return compute_metrics(all_histories), all_histories


def run_per_archetype(pool_cfg, fitness_name, strategy, n_per=125,
                      contraction_pct=0.05, ramp_pct=0.07):
    """Run per-archetype analysis."""
    fitness_model = FITNESS_MODELS[fitness_name]
    results = {}

    for arch_name in ARCHETYPE_NAMES:
        histories = []
        for _ in range(n_per):
            pool = build_pool(dual_res_pct=pool_cfg["dual_res_pct"])
            h, d, cache = narrative_gravity_draft(
                pool, arch_name, fitness_model, strategy,
                contraction_pct=contraction_pct,
                ramp_pct=ramp_pct,
            )
            histories.append((h, d, cache))
        results[arch_name] = compute_metrics(histories)

    return results


def format_trace(history, drafted, sa_cache, player_archetype):
    """Format a detailed draft trace."""
    lines = [f"=== Draft Trace: {player_archetype} ==="]
    for h in history:
        pick = h["pick"]
        sa = h["sa_count"]
        pool_sz = h["pool_size"]
        chosen = h["chosen"]
        chosen_sa = "S/A" if sa_cache[chosen.id] else "C/F"
        sym_str = "/".join(chosen.symbols) if chosen.symbols else "Generic"
        lines.append(
            f"  Pick {pick:2d}: pool={pool_sz:3d}, pack S/A={sa}, "
            f"chose [{chosen.archetype}:{sym_str}] ({chosen_sa})"
        )
    sa_d = sum(1 for c in drafted if sa_cache[c.id])
    lines.append(f"  Final: {sa_d}/{len(drafted)} S/A = {sa_d/max(1,len(drafted))*100:.0f}%")
    return "\n".join(lines)


# ============================================================
# Main
# ============================================================
def main():
    random.seed(42)

    POOLS = [
        {"name": "V7_15pct", "dual_res_pct": 0.15},
        {"name": "Enriched_40pct", "dual_res_pct": 0.40},
    ]

    FITNESS_NAMES = ["Optimistic", "Graduated", "Pessimistic", "Hostile"]
    STRATEGIES = ["committed", "power", "signal"]

    # Test multiple contraction rates
    VARIANTS = [
        ("Standard",        0.05, 0.07),
        ("Aggressive",      0.08, 0.10),
        ("Ultra-Aggressive", 0.12, 0.15),
    ]

    print("=" * 80)
    print("NARRATIVE GRAVITY SIMULATION -- Agent 9")
    print("=" * 80)

    # =====================================================
    # Full results: all combos
    # =====================================================
    all_results = {}

    for variant_name, c_pct, r_pct in VARIANTS:
        print(f"\n{'='*60}")
        print(f"VARIANT: {variant_name} (contraction={c_pct}, ramp={r_pct})")
        print(f"{'='*60}")

        for pool_cfg in POOLS:
            for fitness_name in FITNESS_NAMES:
                for strategy in STRATEGIES:
                    label = f"{variant_name}|{pool_cfg['name']}|{fitness_name}|{strategy}"
                    metrics, _ = run_aggregate(
                        pool_cfg, fitness_name, strategy,
                        contraction_pct=c_pct, ramp_pct=r_pct,
                    )
                    all_results[label] = metrics

    # =====================================================
    # Print summary tables
    # =====================================================
    print("\n" + "=" * 80)
    print("SUMMARY: M3 by Variant x Pool x Fitness (committed strategy)")
    print("=" * 80)
    print(f"{'Variant':<18} {'Pool':<16} {'Optimistic':>10} {'Graduated':>10} "
          f"{'Pessimistic':>11} {'Hostile':>10}")
    for vn, _, _ in VARIANTS:
        for pool_cfg in POOLS:
            row = []
            for fn in FITNESS_NAMES:
                label = f"{vn}|{pool_cfg['name']}|{fn}|committed"
                row.append(f"{all_results[label]['M3']:.2f}")
            print(f"{vn:<18} {pool_cfg['name']:<16} {row[0]:>10} {row[1]:>10} "
                  f"{row[2]:>11} {row[3]:>10}")

    # Full metric table for best variant
    print("\n" + "=" * 80)
    print("FULL METRICS: Ultra-Aggressive, committed strategy")
    print("=" * 80)
    best_var = "Ultra-Aggressive"
    print(f"{'Pool':<16} {'Fitness':<12} {'M1':>5} {'M2':>5} {'M3':>5} {'M4':>5} "
          f"{'M5':>5} {'M6':>5} {'M7':>6} {'M9':>5} {'M10':>5}")
    for pool_cfg in POOLS:
        for fn in FITNESS_NAMES:
            label = f"{best_var}|{pool_cfg['name']}|{fn}|committed"
            m = all_results[label]
            print(f"{pool_cfg['name']:<16} {fn:<12} {m['M1']:5.2f} {m['M2']:5.2f} "
                  f"{m['M3']:5.2f} {m['M4']:5.2f} {m['M5']:5.1f} {m['M6']:5.2f} "
                  f"{m['M7']:6.3f} {m['M9']:5.2f} {m['M10']:5.1f}")

    # All strategies for best variant
    print("\n" + "=" * 80)
    print("ALL STRATEGIES: Ultra-Aggressive, Enriched_40pct")
    print("=" * 80)
    for fn in FITNESS_NAMES:
        for strat in STRATEGIES:
            label = f"{best_var}|Enriched_40pct|{fn}|{strat}"
            m = all_results[label]
            print(f"  {fn:<12} {strat:<12}: M3={m['M3']:.2f} M4={m['M4']:.2f} "
                  f"M5={m['M5']:.1f} M6={m['M6']:.2f} M10={m['M10']:.1f}")

    # =====================================================
    # Per-archetype convergence (Ultra-Aggressive)
    # =====================================================
    print("\n" + "=" * 80)
    print("PER-ARCHETYPE: Ultra-Aggressive, Enriched_40pct, committed")
    print("=" * 80)

    for fn in ["Graduated", "Pessimistic", "Hostile"]:
        print(f"\n  Fitness: {fn}")
        pa = run_per_archetype(
            {"name": "Enriched_40pct", "dual_res_pct": 0.40},
            fn, "committed", contraction_pct=0.12, ramp_pct=0.15,
        )
        print(f"  {'Archetype':<16} {'M3':>6} {'M5':>6} {'M6':>6} {'M9':>6} {'M10':>5}")
        for arch in ARCHETYPE_NAMES:
            m = pa[arch]
            print(f"  {arch:<16} {m['M3']:6.2f} {m['M5']:6.1f} {m['M6']:6.2f} "
                  f"{m['M9']:6.2f} {m['M10']:5.1f}")

    # V7 pool per-archetype
    print("\n  Per-archetype: Ultra-Aggressive, V7_15pct, committed, Graduated")
    pa_v7 = run_per_archetype(
        {"name": "V7_15pct", "dual_res_pct": 0.15},
        "Graduated", "committed", contraction_pct=0.12, ramp_pct=0.15,
    )
    print(f"  {'Archetype':<16} {'M3':>6} {'M5':>6} {'M6':>6} {'M9':>6} {'M10':>5}")
    for arch in ARCHETYPE_NAMES:
        m = pa_v7[arch]
        print(f"  {arch:<16} {m['M3']:6.2f} {m['M5']:6.1f} {m['M6']:6.2f} "
              f"{m['M9']:6.2f} {m['M10']:5.1f}")

    # =====================================================
    # Pack quality distribution
    # =====================================================
    print("\n" + "=" * 80)
    print("PACK QUALITY DISTRIBUTION (picks 6+, Ultra-Aggressive, committed)")
    print("=" * 80)
    for pool_cfg in POOLS:
        for fn in ["Graduated", "Pessimistic"]:
            label = f"{best_var}|{pool_cfg['name']}|{fn}|committed"
            m = all_results[label]
            pq = m["pack_pcts"]
            print(f"\n  {pool_cfg['name']} / {fn}:")
            print(f"    P10={pq[10]}  P25={pq[25]}  P50={pq[50]}  "
                  f"P75={pq[75]}  P90={pq[90]}")
            print(f"    Avg consec bad (SA<1.5): {m['avg_consec_bad']:.1f}")
            print(f"    Worst consec bad: {m['worst_consec_bad']}")

    # =====================================================
    # Parameter sensitivity
    # =====================================================
    print("\n" + "=" * 80)
    print("PARAMETER SENSITIVITY: contraction_pct (Graduated, Enriched_40pct, committed)")
    print("=" * 80)
    for rate in [0.02, 0.04, 0.06, 0.08, 0.10, 0.12, 0.15, 0.20]:
        metrics, _ = run_aggregate(
            {"name": "Enriched_40pct", "dual_res_pct": 0.40},
            "Graduated", "committed", n_drafts=500,
            contraction_pct=rate, ramp_pct=rate + 0.03,
        )
        print(f"  rate={rate:.2f}: M3={metrics['M3']:.2f} M6={metrics['M6']:.2f} "
              f"M7={metrics['M7']:.3f} M9={metrics['M9']:.2f} M10={metrics['M10']:.1f} "
              f"consec_bad={metrics['avg_consec_bad']:.1f}")

    print("\n  SENSITIVITY: ramp_pct with fixed contraction=0.12 (Graduated, 40%, committed)")
    for ramp in [0.08, 0.10, 0.12, 0.15, 0.18, 0.20]:
        metrics, _ = run_aggregate(
            {"name": "Enriched_40pct", "dual_res_pct": 0.40},
            "Graduated", "committed", n_drafts=500,
            contraction_pct=0.12, ramp_pct=ramp,
        )
        print(f"  ramp={ramp:.2f}: M3={metrics['M3']:.2f} M6={metrics['M6']:.2f} "
              f"M7={metrics['M7']:.3f} M9={metrics['M9']:.2f}")

    print("\n  SENSITIVITY: contraction_start pick (Graduated, 40%, committed)")
    # Test by modifying the algorithm start
    for start in [2, 3, 4, 5, 6]:
        histories = []
        fitness_model = FITNESS_MODELS["Graduated"]
        for i in range(500):
            arch_name = ARCHETYPE_NAMES[i % 8]
            pool = build_pool(dual_res_pct=0.40)
            h, d, cache = narrative_gravity_draft(
                pool, arch_name, fitness_model, "committed",
                contraction_pct=0.12, ramp_pct=0.15,
                contraction_start=start,
            )
            histories.append((h, d, cache))
        metrics = compute_metrics(histories)
        print(f"  start={start}: M3={metrics['M3']:.2f} M6={metrics['M6']:.2f} "
              f"M9={metrics['M9']:.2f}")

    # =====================================================
    # Fitness degradation curve
    # =====================================================
    print("\n" + "=" * 80)
    print("FITNESS DEGRADATION (Ultra-Aggressive, committed)")
    print("=" * 80)
    for pool_cfg in POOLS:
        print(f"\n  Pool: {pool_cfg['name']}")
        for fn in FITNESS_NAMES:
            label = f"{best_var}|{pool_cfg['name']}|{fn}|committed"
            m = all_results[label]
            print(f"    {fn:<14}: M3={m['M3']:.2f}  M6={m['M6']:.2f}  "
                  f"M10avg={m['M10']:.1f}  consec_bad={m['avg_consec_bad']:.1f}")

    # =====================================================
    # Draft traces
    # =====================================================
    print("\n" + "=" * 80)
    print("DRAFT TRACES (Ultra-Aggressive, Enriched_40pct)")
    print("=" * 80)

    # Trace 1: Early committer, Warriors, Graduated
    random.seed(100)
    pool = build_pool(dual_res_pct=0.40)
    h1, d1, c1 = narrative_gravity_draft(
        pool, "Warriors", FITNESS_MODELS["Graduated"], "committed",
        contraction_pct=0.12, ramp_pct=0.15,
    )
    print("\n" + format_trace(h1, d1, c1, "Warriors"))

    # Trace 2: Signal reader, Blink, Graduated
    random.seed(200)
    pool = build_pool(dual_res_pct=0.40)
    h2, d2, c2 = narrative_gravity_draft(
        pool, "Blink", FITNESS_MODELS["Graduated"], "signal",
        contraction_pct=0.12, ramp_pct=0.15,
    )
    print("\n" + format_trace(h2, d2, c2, "Blink"))

    # Trace 3: Power chaser, Flash, Pessimistic
    random.seed(300)
    pool = build_pool(dual_res_pct=0.40)
    h3, d3, c3 = narrative_gravity_draft(
        pool, "Flash", FITNESS_MODELS["Pessimistic"], "power",
        contraction_pct=0.12, ramp_pct=0.15,
    )
    print("\n" + format_trace(h3, d3, c3, "Flash"))

    # =====================================================
    # Comparison table
    # =====================================================
    print("\n" + "=" * 80)
    print("FINAL COMPARISON: M3 across all conditions (committed)")
    print("=" * 80)
    print(f"{'Variant':<18} {'Pool':<16} {'Opt':>6} {'Grad':>6} {'Pess':>6} {'Host':>6}")
    for vn, _, _ in VARIANTS:
        for pool_cfg in POOLS:
            vals = []
            for fn in FITNESS_NAMES:
                label = f"{vn}|{pool_cfg['name']}|{fn}|committed"
                vals.append(f"{all_results[label]['M3']:.2f}")
            print(f"{vn:<18} {pool_cfg['name']:<16} {vals[0]:>6} {vals[1]:>6} "
                  f"{vals[2]:>6} {vals[3]:>6}")

    print("\nDone.")


if __name__ == "__main__":
    main()
