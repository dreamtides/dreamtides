#!/usr/bin/env python3
"""Explore parameter adjustments to hit convergence targets with N=0.

Tests several approaches:
A) Baseline focus_rate=0.35 (reference, known too slow)
B) Higher focus_rate=0.50
C) Higher focus_rate=0.55
D) Moderate focus_rate=0.45 + tighter similarity (dist2=0.05, dist3=0.0)
E) focus_rate=0.45 + earlier start (pick 2)
F) focus_rate=0.50 + tighter similarity
"""

import math
import random
import tomllib
from bisect import bisect_left
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
RENDERED_CARDS = (
    ROOT / "client" / "Assets" / "StreamingAssets" / "Tabula" / "rendered-cards.toml"
)

CORE_TIDES = ["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"]
NEUTRAL = "Neutral"
PACK_SIZE = 4
NUM_PICKS = 25
N_EXCLUSION = 0

BASE_AFFINITY = 1.0
DECAY_FACTOR = 0.85
NEUTRAL_DRAFT_CONTRIBUTION = 0.4
NEUTRAL_AFFINITY_FACTOR = 0.5


def load_card_pool():
    with open(RENDERED_CARDS, "rb") as f:
        data = tomllib.load(f)
    return [c["tide"] for c in data["cards"] if c.get("tide")]


def circle_distance(t1, t2):
    i1 = CORE_TIDES.index(t1)
    i2 = CORE_TIDES.index(t2)
    d = abs(i1 - i2)
    return min(d, 7 - d)


_dist_cache = {}


def cached_distance(t1, t2):
    key = (t1, t2)
    if key not in _dist_cache:
        _dist_cache[key] = circle_distance(t1, t2)
    return _dist_cache[key]


def compute_affinities(drafted_tides, similarity):
    affinities = {}
    neutral_count = drafted_tides.count(NEUTRAL)

    for t in CORE_TIDES:
        a = BASE_AFFINITY
        pos = 0
        for c in reversed(drafted_tides):
            if c == NEUTRAL:
                a += NEUTRAL_DRAFT_CONTRIBUTION * (DECAY_FACTOR**pos)
            else:
                a += similarity[cached_distance(t, c)] * (DECAY_FACTOR**pos)
            pos += 1
        affinities[t] = a

    max_core = max(affinities[t] for t in CORE_TIDES)
    affinities[NEUTRAL] = max(
        BASE_AFFINITY + neutral_count * 1.0,
        NEUTRAL_AFFINITY_FACTOR * max_core,
    )
    return affinities


def weighted_sample_indices(pool, affinities, focus, n):
    if len(pool) <= n:
        return list(range(len(pool)))

    weights = []
    for tide in pool:
        a = affinities.get(tide, BASE_AFFINITY)
        weights.append(a**focus if focus > 0 else 1.0)

    selected = []
    remaining_indices = list(range(len(pool)))
    remaining_weights = list(weights)

    for _ in range(n):
        total = sum(remaining_weights)
        if total <= 0:
            idx = random.randrange(len(remaining_indices))
        else:
            cumulative = []
            s = 0.0
            for w in remaining_weights:
                s += w
                cumulative.append(s)
            r = random.random() * total
            idx = bisect_left(cumulative, r)
            if idx >= len(remaining_indices):
                idx = len(remaining_indices) - 1
        selected.append(remaining_indices[idx])
        remaining_indices.pop(idx)
        remaining_weights.pop(idx)

    return selected


def get_allied_tides(tide):
    idx = CORE_TIDES.index(tide)
    return [CORE_TIDES[(idx - 1) % 7], CORE_TIDES[(idx + 1) % 7]]


def classify_pack(pack_tides, dominant_tides, allied_tides):
    dominant = sum(1 for t in pack_tides if t in dominant_tides)
    allied = sum(1 for t in pack_tides if t in allied_tides and t not in dominant_tides)
    neutral = sum(
        1 for t in pack_tides if t == NEUTRAL and NEUTRAL not in dominant_tides
    )
    distant = len(pack_tides) - dominant - allied - neutral
    return {
        "dominant": dominant,
        "allied": allied,
        "neutral": neutral,
        "distant": distant,
    }


def mono_tide_strategy(target_tide):
    def strategy(pack_tides, drafted, pick):
        for i, t in enumerate(pack_tides):
            if t == target_tide:
                return i
        return random.randrange(len(pack_tides))

    return strategy


def pivot_strategy(tide_a, tide_b, pivot_pick=8):
    def strategy(pack_tides, drafted, pick):
        target = tide_a if pick <= pivot_pick else tide_b
        for i, t in enumerate(pack_tides):
            if t == target:
                return i
        return random.randrange(len(pack_tides))

    return strategy


def simulate_draft(
    card_pool,
    pick_strategy,
    dominant_tides,
    allied_tides,
    similarity,
    focus_rate,
    focus_start,
):
    pool = list(card_pool)
    drafted = []
    metrics = []

    for pick in range(1, NUM_PICKS + 1):
        if len(pool) < PACK_SIZE:
            metrics.append(None)
            continue

        affinities = compute_affinities(drafted, similarity)
        focus = (
            max(0.0, (pick - focus_start + 1) * focus_rate)
            if pick >= focus_start
            else 0.0
        )
        pack_indices = weighted_sample_indices(pool, affinities, focus, PACK_SIZE)
        pack_tides = [pool[i] for i in pack_indices]

        classification = classify_pack(
            pack_tides, set(dominant_tides), set(allied_tides)
        )
        classification["pool_size"] = len(pool)

        chosen_idx = pick_strategy(pack_tides, drafted, pick)
        drafted.append(pack_tides[chosen_idx])

        for i in sorted(pack_indices, reverse=True):
            pool.pop(i)

        metrics.append(classification)

    return metrics


def run_scenario(
    card_pool, strategy_factory, similarity, focus_rate, focus_start, num_trials
):
    per_pick = {
        p: {
            "dominant": [],
            "ge1": 0,
            "ge2": 0,
            "ge3": 0,
            "allied": [],
            "neutral": [],
            "distant": [],
            "pool_size": [],
            "count": 0,
        }
        for p in range(1, NUM_PICKS + 1)
    }

    for _ in range(num_trials):
        strategy, dominant_tides, allied_tides = strategy_factory()
        result = simulate_draft(
            card_pool,
            strategy,
            dominant_tides,
            allied_tides,
            similarity,
            focus_rate,
            focus_start,
        )

        for pick, m in enumerate(result, 1):
            if m is None:
                continue
            b = per_pick[pick]
            b["dominant"].append(m["dominant"])
            b["allied"].append(m["allied"])
            b["neutral"].append(m["neutral"])
            b["distant"].append(m["distant"])
            b["pool_size"].append(m["pool_size"])
            b["count"] += 1
            if m["dominant"] >= 1:
                b["ge1"] += 1
            if m["dominant"] >= 2:
                b["ge2"] += 1
            if m["dominant"] >= 3:
                b["ge3"] += 1

    return per_pick


def avg(lst):
    return sum(lst) / len(lst) if lst else 0.0


def print_compact(per_pick, picks):
    print(
        f"  {'Pick':>4}  {'Dom':>5}  {'Ally':>5}  {'Neut':>5}  {'Dist':>5}  {'Pool':>5}  {'P≥1':>5}  {'P≥2':>5}  {'P≥3':>5}"
    )
    print(f"  {'-' * 54}")
    for pick in picks:
        b = per_pick[pick]
        if b["count"] == 0:
            continue
        n = b["count"]
        print(
            f"  {pick:>4}  {avg(b['dominant']):>5.2f}  {avg(b['allied']):>5.2f}"
            f"  {avg(b['neutral']):>5.2f}  {avg(b['distant']):>5.2f}"
            f"  {avg(b['pool_size']):>5.0f}"
            f"  {b['ge1']/n:>5.2f}  {b['ge2']/n:>5.2f}  {b['ge3']/n:>5.2f}"
        )


def main():
    print("Loading card pool...")
    card_pool = load_card_pool()
    print(f"Loaded {len(card_pool)} cards, N=0 (all tides present)\n")

    SIM = {0: 1.0, 1: 0.5, 2: 0.15, 3: 0.05}
    SIM_TIGHT = {0: 1.0, 1: 0.5, 2: 0.05, 3: 0.0}

    TRIALS = 5000
    KEY_PICKS = [1, 5, 10, 15, 20, 25]

    variants = [
        ("A: Baseline (focus=0.35, standard sim, start=3)", SIM, 0.35, 3),
        ("B: Higher focus (0.50)", SIM, 0.50, 3),
        ("C: Higher focus (0.55)", SIM, 0.55, 3),
        ("D: Moderate focus (0.45) + tight similarity", SIM_TIGHT, 0.45, 3),
        ("E: Moderate focus (0.45) + earlier start (pick 2)", SIM, 0.45, 2),
        ("F: Higher focus (0.50) + tight similarity", SIM_TIGHT, 0.50, 3),
    ]

    def mono_factory():
        target = random.choice(CORE_TIDES)
        allied = get_allied_tides(target)
        return mono_tide_strategy(target), [target], allied

    def pivot_factory():
        tide_a = random.choice(CORE_TIDES)
        allies = get_allied_tides(tide_a)
        distant = [t for t in CORE_TIDES if t != tide_a and t not in allies]
        tide_b = random.choice(distant)
        allied_b = get_allied_tides(tide_b)
        return pivot_strategy(tide_a, tide_b), [tide_b], allied_b

    for name, sim, fr, fs in variants:
        print(f"{'=' * 62}")
        print(f"  {name}")
        sim_desc = "standard" if sim[2] == 0.15 else f"tight (d2={sim[2]}, d3={sim[3]})"
        print(f"  focus_rate={fr}, focus_start={fs}, similarity={sim_desc}")
        print(f"{'=' * 62}")

        print(f"\n  Mono-Tide ({TRIALS} trials):")
        results = run_scenario(card_pool, mono_factory, sim, fr, fs, TRIALS)
        print_compact(results, KEY_PICKS)

        print(f"\n  Pivot ({TRIALS} trials):")
        results = run_scenario(card_pool, pivot_factory, sim, fr, fs, TRIALS)
        print_compact(results, KEY_PICKS)
        print()


if __name__ == "__main__":
    main()
