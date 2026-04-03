#!/usr/bin/env python3
"""Sweep initial_tide_exclusion (N=0..4) to see how it affects draft behavior.

Uses the final algorithm: decay=0.85, focus_rate=0.35, no trimming.
Runs mono-tide, two-tide, and pivot scenarios for each N value.
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
SIMILARITY = {0: 1.0, 1: 0.5, 2: 0.15, 3: 0.05}

# Final algorithm parameters
BASE_AFFINITY = 1.0
FOCUS_RATE = 0.35
DECAY_FACTOR = 0.85
NEUTRAL_DRAFT_CONTRIBUTION = 0.4
NEUTRAL_AFFINITY_FACTOR = 0.5
PACK_SIZE = 4
NUM_PICKS = 25


def load_card_pool():
    with open(RENDERED_CARDS, "rb") as f:
        data = tomllib.load(f)
    return [c["tide"] for c in data["cards"] if c.get("tide")]


def circle_distance(t1, t2):
    i1 = CORE_TIDES.index(t1)
    i2 = CORE_TIDES.index(t2)
    d = abs(i1 - i2)
    return min(d, 7 - d)


def compute_affinities(drafted_tides):
    affinities = {}
    neutral_count = drafted_tides.count(NEUTRAL)
    core_drafted_reversed = [t for t in reversed(drafted_tides) if t != NEUTRAL]

    for t in CORE_TIDES:
        a = BASE_AFFINITY
        for i, c in enumerate(core_drafted_reversed):
            a += SIMILARITY[circle_distance(t, c)] * (DECAY_FACTOR**i)
        # Neutral contributions with decay
        neutral_positions = [
            j for j, dt in enumerate(reversed(drafted_tides)) if dt == NEUTRAL
        ]
        for pos in neutral_positions:
            a += NEUTRAL_DRAFT_CONTRIBUTION * (DECAY_FACTOR**pos)
        affinities[t] = a

    max_core = max(affinities[t] for t in CORE_TIDES)
    affinities[NEUTRAL] = max(
        BASE_AFFINITY + neutral_count * 1.0,
        NEUTRAL_AFFINITY_FACTOR * max_core,
    )
    return affinities


def compute_focus(pick_number):
    return max(0.0, (pick_number - 2) * FOCUS_RATE)


def weighted_sample_indices(pool, affinities, n):
    focus = affinities["_focus"]
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
    if tide == NEUTRAL:
        return []
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


def two_tide_strategy(tide_a, tide_b):
    targets = {tide_a, tide_b}

    def strategy(pack_tides, drafted, pick):
        for i, t in enumerate(pack_tides):
            if t in targets:
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


def simulate_draft(card_pool, pick_strategy, dominant_tides, allied_tides, n_exclusion):
    if n_exclusion > 0:
        excluded = random.sample(CORE_TIDES, min(n_exclusion, len(CORE_TIDES)))
    else:
        excluded = []
    pool = [t for t in card_pool if t not in excluded]

    available_dominant = [t for t in dominant_tides if t not in excluded]
    available_allied = [t for t in allied_tides if t not in excluded]

    if not available_dominant:
        return None

    drafted = []
    metrics = []

    for pick in range(1, NUM_PICKS + 1):
        if len(pool) < PACK_SIZE:
            metrics.append(None)
            continue

        affinities = compute_affinities(drafted)
        affinities["_focus"] = compute_focus(pick)
        pack_indices = weighted_sample_indices(pool, affinities, PACK_SIZE)
        pack_tides = [pool[i] for i in pack_indices]

        classification = classify_pack(
            pack_tides, set(available_dominant), set(available_allied)
        )
        classification["pool_size"] = len(pool)

        chosen_idx = pick_strategy(pack_tides, drafted, pick)
        drafted.append(pack_tides[chosen_idx])

        for i in sorted(pack_indices, reverse=True):
            pool.pop(i)

        metrics.append(classification)

    return metrics


def run_scenario(card_pool, strategy_factory, n_exclusion, num_trials):
    per_pick = {
        p: {
            "dominant": [],
            "allied": [],
            "neutral": [],
            "distant": [],
            "pool_size": [],
            "ge1": 0,
            "ge2": 0,
            "ge3": 0,
            "count": 0,
        }
        for p in range(1, NUM_PICKS + 1)
    }

    valid_trials = 0
    attempts = 0
    max_attempts = num_trials * 5

    while valid_trials < num_trials and attempts < max_attempts:
        attempts += 1
        strategy, dominant_tides, allied_tides = strategy_factory()
        result = simulate_draft(
            card_pool, strategy, dominant_tides, allied_tides, n_exclusion
        )
        if result is None:
            continue
        valid_trials += 1

        for pick, m in enumerate(result, 1):
            if m is None:
                continue
            bucket = per_pick[pick]
            bucket["dominant"].append(m["dominant"])
            bucket["allied"].append(m["allied"])
            bucket["neutral"].append(m["neutral"])
            bucket["distant"].append(m["distant"])
            bucket["pool_size"].append(m["pool_size"])
            bucket["count"] += 1
            if m["dominant"] >= 1:
                bucket["ge1"] += 1
            if m["dominant"] >= 2:
                bucket["ge2"] += 1
            if m["dominant"] >= 3:
                bucket["ge3"] += 1

    return per_pick, valid_trials


def avg(lst):
    return sum(lst) / len(lst) if lst else 0.0


def print_compact(per_pick, num_trials, picks):
    print(
        f"  {'Pick':>4}  {'Dom':>5}  {'Ally':>5}  {'Neut':>5}  {'Dist':>5}  {'Pool':>5}  {'P≥1':>5}  {'P≥2':>5}  {'P≥3':>5}"
    )
    print(f"  {'-' * 54}")
    for pick in picks:
        b = per_pick[pick]
        if b["count"] == 0:
            print(f"  {pick:>4}  {'N/A':>5}")
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
    tide_counts = Counter(card_pool)
    print(f"Loaded {len(card_pool)} cards")
    print(f"Per tide: {dict(sorted(tide_counts.items(), key=lambda x: -x[1]))}")

    TRIALS = 5000
    KEY_PICKS = [1, 5, 10, 15, 20, 25]

    for n_excl in range(0, 5):
        remaining_tides = 7 - n_excl
        pool_size = (
            sum(v for k, v in tide_counts.items() if k != NEUTRAL)
            * remaining_tides
            // 7
            + tide_counts[NEUTRAL]
        )
        alliances = max(0, remaining_tides - 1) if remaining_tides >= 2 else 0

        print(f"\n{'=' * 62}")
        print(
            f"  N = {n_excl} ({remaining_tides} core tides + Neutral, ~{pool_size} cards)"
        )
        print(f"  {alliances} possible tide alliances available")
        print(f"{'=' * 62}")

        # Mono-tide
        def mono_factory():
            target = random.choice(CORE_TIDES)
            allied = get_allied_tides(target)
            return mono_tide_strategy(target), [target], allied

        print(f"\n  Mono-Tide ({TRIALS} trials):")
        results, trials = run_scenario(card_pool, mono_factory, n_excl, TRIALS)
        print(f"  (completed {trials} valid trials)")
        print_compact(results, trials, KEY_PICKS)

        # Two-tide (skip for N>=5 where only 2 core tides remain)
        if remaining_tides >= 3:

            def two_tide_factory():
                target = random.choice(CORE_TIDES)
                allies = get_allied_tides(target)
                partner = random.choice(allies)
                dominant = [target, partner]
                outer_allies = set(
                    get_allied_tides(target) + get_allied_tides(partner)
                ) - set(dominant)
                return two_tide_strategy(target, partner), dominant, list(outer_allies)

            print(f"\n  Two-Tide ({TRIALS} trials):")
            results, trials = run_scenario(card_pool, two_tide_factory, n_excl, TRIALS)
            print(f"  (completed {trials} valid trials)")
            print_compact(results, trials, KEY_PICKS)

        # Pivot
        def pivot_factory():
            tide_a = random.choice(CORE_TIDES)
            allies = get_allied_tides(tide_a)
            distant = [t for t in CORE_TIDES if t != tide_a and t not in allies]
            if not distant:
                distant = [t for t in CORE_TIDES if t != tide_a]
            tide_b = random.choice(distant)
            allied_b = get_allied_tides(tide_b)
            return pivot_strategy(tide_a, tide_b), [tide_b], allied_b

        print(f"\n  Pivot at pick 8 ({TRIALS} trials, dominant = post-pivot tide):")
        results, trials = run_scenario(card_pool, pivot_factory, n_excl, TRIALS)
        print(f"  (completed {trials} valid trials)")
        print_compact(results, trials, KEY_PICKS)

        # Additional stats for this N
        # How many cards can the player feasibly draft from their dominant tide?
        avg_dom_cards = tide_counts.get("Bloom", 71)  # representative
        max_draft_picks = 25
        cards_leaving_per_pick = PACK_SIZE
        total_cards_leaving = max_draft_picks * cards_leaving_per_pick
        print(
            f"\n  Pool notes: ~{avg_dom_cards} cards per tide, "
            f"{total_cards_leaving} cards leave pool over {max_draft_picks} picks"
        )
        if n_excl > 0:
            combos = math.comb(7, n_excl)
            print(f"  C(7,{n_excl}) = {combos} possible tide exclusion configurations")


if __name__ == "__main__":
    main()
