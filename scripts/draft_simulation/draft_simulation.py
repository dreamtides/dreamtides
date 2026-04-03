#!/usr/bin/env python3
"""Monte Carlo simulation for the Tide Current draft algorithm.

Validates convergence behavior against the real card pool.
See notes/draft_algorithm.md for the full algorithm specification.

Explores pool bias as a mechanism for creating readable external signals
that reward players who notice which tides are overrepresented.
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
PACK_SIZE = 4
NUM_PICKS = 25


class Variant:
    def __init__(self, name, description, **overrides):
        self.name = name
        self.description = description
        self.base_affinity = overrides.get("base_affinity", 1.0)
        self.focus_rate = overrides.get("focus_rate", 0.35)
        self.focus_cap = overrides.get("focus_cap", None)
        self.neutral_draft_contribution = overrides.get(
            "neutral_draft_contribution", 0.4
        )
        self.neutral_affinity_factor = overrides.get("neutral_affinity_factor", 0.5)
        self.trim_start_pick = overrides.get("trim_start_pick", 999)
        self.trim_rate = overrides.get("trim_rate", 1.5)
        self.trim_threshold = overrides.get("trim_threshold", 2.0)
        self.affinity_mode = overrides.get("affinity_mode", "decay")
        self.decay_factor = overrides.get("decay_factor", 0.85)
        # Tide exclusion
        self.initial_tide_exclusion = overrides.get("initial_tide_exclusion", 2)
        # Pool bias (physical removal)
        self.featured_tide_count = overrides.get("featured_tide_count", 0)
        self.non_featured_removal_rate = overrides.get(
            "non_featured_removal_rate", 0.30
        )
        # Pool bias (weight multiplier, no removal)
        self.featured_weight = overrides.get("featured_weight", 1.0)


def load_card_pool():
    with open(RENDERED_CARDS, "rb") as f:
        data = tomllib.load(f)
    return [c["tide"] for c in data["cards"] if c.get("tide")]


def circle_distance(t1, t2):
    i1 = CORE_TIDES.index(t1)
    i2 = CORE_TIDES.index(t2)
    d = abs(i1 - i2)
    return min(d, 7 - d)


def compute_affinities(drafted_tides, variant, pick_number=None):
    affinities = {}
    neutral_count = drafted_tides.count(NEUTRAL)
    core_drafted = [t for t in drafted_tides if t != NEUTRAL]

    for t in CORE_TIDES:
        a = variant.base_affinity

        if variant.affinity_mode == "decay":
            factor = variant.decay_factor or 0.85
            for i, c in enumerate(reversed(core_drafted)):
                weight = factor**i
                if c != NEUTRAL:
                    a += SIMILARITY[circle_distance(t, c)] * weight
            for i in range(neutral_count):
                idx_from_end = len(drafted_tides) - 1
                for j, dt in enumerate(reversed(drafted_tides)):
                    if dt == NEUTRAL:
                        idx_from_end = j
                        break
                a += variant.neutral_draft_contribution * (factor**idx_from_end)
        else:
            for c in core_drafted:
                a += SIMILARITY[circle_distance(t, c)]
            a += neutral_count * variant.neutral_draft_contribution

        affinities[t] = a

    max_core = (
        max(affinities[t] for t in CORE_TIDES) if CORE_TIDES else variant.base_affinity
    )
    affinities[NEUTRAL] = max(
        variant.base_affinity + neutral_count * 1.0,
        variant.neutral_affinity_factor * max_core,
    )
    return affinities


def compute_focus(pick_number, variant):
    f = max(0.0, (pick_number - 2) * variant.focus_rate)
    if variant.focus_cap is not None:
        f = min(f, variant.focus_cap)
    return f


def weighted_sample_indices(pool, affinities, focus, n, tide_multipliers=None):
    if len(pool) <= n:
        return list(range(len(pool)))

    weights = []
    for tide in pool:
        a = affinities.get(tide, 1.0)
        w = a**focus if focus > 0 else 1.0
        if tide_multipliers and tide in tide_multipliers:
            w *= tide_multipliers[tide]
        weights.append(w)

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


def pick_featured_tides(excluded_tides):
    """Pick a random adjacent pair of core tides (both available after exclusion)."""
    available_core = [t for t in CORE_TIDES if t not in excluded_tides]
    if len(available_core) < 2:
        return []
    attempts = 0
    while attempts < 50:
        t1 = random.choice(available_core)
        idx = CORE_TIDES.index(t1)
        t2 = CORE_TIDES[(idx + 1) % 7]
        if t2 in available_core:
            return [t1, t2]
        t2 = CORE_TIDES[(idx - 1) % 7]
        if t2 in available_core:
            return [t1, t2]
        attempts += 1
    return random.sample(available_core, 2)


def apply_pool_bias(pool, variant, excluded_tides):
    """Apply pool bias: remove a fraction of non-featured tide cards.

    Featured tides are always an adjacent pair on the circle.
    """
    if variant.featured_tide_count == 0 and variant.featured_weight <= 1.0:
        return pool, []

    featured = pick_featured_tides(excluded_tides)
    if not featured:
        return pool, []

    # Weight-only bias: no physical removal, just return featured list
    if variant.featured_tide_count == 0:
        return pool, featured

    # Physical removal bias
    biased_pool = []
    for tide in pool:
        if tide in featured or tide == NEUTRAL:
            biased_pool.append(tide)
        elif tide not in excluded_tides:
            if random.random() > variant.non_featured_removal_rate:
                biased_pool.append(tide)
        else:
            biased_pool.append(tide)

    return biased_pool, featured


def simulate_draft(
    card_pool,
    pick_strategy,
    dominant_tides_fn,
    allied_tides_fn,
    variant,
    num_picks=NUM_PICKS,
):
    excluded = random.sample(CORE_TIDES, variant.initial_tide_exclusion)
    pool = [t for t in card_pool if t not in excluded]

    pool, featured_tides = apply_pool_bias(pool, variant, excluded)

    # Build tide multipliers for weighted bias (no physical removal)
    tide_multipliers = None
    if featured_tides and variant.featured_weight > 1.0:
        tide_multipliers = {t: variant.featured_weight for t in featured_tides}

    drafted = []
    metrics = []

    for pick in range(1, num_picks + 1):
        if len(pool) < PACK_SIZE:
            metrics.append(None)
            continue

        affinities = compute_affinities(drafted, variant, pick)
        focus = compute_focus(pick, variant)
        pack_indices = weighted_sample_indices(
            pool, affinities, focus, PACK_SIZE, tide_multipliers
        )
        pack_tides = [pool[i] for i in pack_indices]

        current_dominant = set(dominant_tides_fn(drafted))
        current_allied = set(allied_tides_fn(drafted))

        available_dominant = [t for t in current_dominant if t not in excluded]
        if not available_dominant and drafted:
            core_drafted = [t for t in drafted if t != NEUTRAL and t not in excluded]
            if core_drafted:
                most_common = Counter(core_drafted).most_common(1)[0][0]
                available_dominant = [most_common]
                current_dominant = set(available_dominant)
                current_allied = set(get_allied_tides(most_common))

        classification = classify_pack(pack_tides, current_dominant, current_allied)
        classification["pool_size"] = len(pool)

        pool_tide_counts = Counter(pool)
        chosen_idx = pick_strategy(pack_tides, drafted, pick, pool_tide_counts)
        drafted.append(pack_tides[chosen_idx])

        for i in sorted(pack_indices, reverse=True):
            pool.pop(i)

        metrics.append(classification)

    core_drafted = [t for t in drafted if t != NEUTRAL]
    final_dominant = (
        Counter(core_drafted).most_common(1)[0][0] if core_drafted else None
    )
    featured_aligned = final_dominant in featured_tides if featured_tides else None

    return metrics, featured_aligned, featured_tides


# ---- Player Strategies ----


def mono_tide_strategy(target_tide):
    """Hard-commits to a pre-chosen tide regardless of what's offered."""

    def strategy(pack_tides, drafted, pick, pool_counts):
        for i, t in enumerate(pack_tides):
            if t == target_tide:
                return i
        return random.randrange(len(pack_tides))

    return strategy


def pivot_strategy(tide_a, tide_b, pivot_pick=8):
    def strategy(pack_tides, drafted, pick, pool_counts):
        target = tide_a if pick <= pivot_pick else tide_b
        for i, t in enumerate(pack_tides):
            if t == target:
                return i
        return random.randrange(len(pack_tides))

    return strategy


def signal_reader_strategy():
    """Reads pack signals rather than pre-committing.

    Picks 1-3: Takes whichever tide appears most in the pack (reading signal).
    Picks 4-7: Blends drafted history with pack composition.
    Picks 8+: Committed to dominant tide, falls back to allied.
    """

    def strategy(pack_tides, drafted, pick, pool_counts):
        core_pack = [t for t in pack_tides if t != NEUTRAL]
        drafted_counts = Counter(t for t in drafted if t != NEUTRAL)

        if pick <= 3:
            pack_counts = Counter(core_pack)
            if pack_counts:
                best_tide = pack_counts.most_common(1)[0][0]
                for i, t in enumerate(pack_tides):
                    if t == best_tide:
                        return i
            return random.randrange(len(pack_tides))

        elif pick <= 7:
            pack_counts = Counter(core_pack)
            best_score = -1
            best_idx = 0
            for i, t in enumerate(pack_tides):
                if t == NEUTRAL:
                    score = 0.3
                else:
                    score = drafted_counts.get(t, 0) + 0.5 * pack_counts.get(t, 0)
                    for ally in get_allied_tides(t):
                        score += 0.3 * drafted_counts.get(ally, 0)
                if score > best_score:
                    best_score = score
                    best_idx = i
            return best_idx

        else:
            if drafted_counts:
                dominant = drafted_counts.most_common(1)[0][0]
                allied = get_allied_tides(dominant)
                for i, t in enumerate(pack_tides):
                    if t == dominant:
                        return i
                for i, t in enumerate(pack_tides):
                    if t in allied:
                        return i
            return random.randrange(len(pack_tides))

    return strategy


def pool_reader_strategy():
    """Reads pool composition to identify overrepresented tides.

    Uses the pool tide counts (which pool bias makes unequal) to identify
    the most abundant tide, then drafts toward it. This models a player who
    notices "I keep seeing Bloom cards" and leans in.

    Picks 1-4: Picks the card whose tide is most abundant in the remaining pool.
    Picks 5-7: Blends pool abundance with drafted history.
    Picks 8+: Committed to dominant tide.
    """

    def strategy(pack_tides, drafted, pick, pool_counts):
        drafted_counts = Counter(t for t in drafted if t != NEUTRAL)

        if pick <= 4:
            # Pick the card from the most abundant tide in the pool
            best_score = -1
            best_idx = 0
            for i, t in enumerate(pack_tides):
                if t == NEUTRAL:
                    score = 0
                else:
                    score = pool_counts.get(t, 0)
                if score > best_score:
                    best_score = score
                    best_idx = i
            return best_idx

        elif pick <= 7:
            # Blend pool abundance with drafted history
            best_score = -1
            best_idx = 0
            for i, t in enumerate(pack_tides):
                if t == NEUTRAL:
                    score = 0.3
                else:
                    pool_score = pool_counts.get(t, 0) / max(
                        1, sum(pool_counts.get(ct, 0) for ct in CORE_TIDES)
                    )
                    draft_score = drafted_counts.get(t, 0)
                    for ally in get_allied_tides(t):
                        draft_score += 0.3 * drafted_counts.get(ally, 0)
                    score = pool_score * 2 + draft_score
                if score > best_score:
                    best_score = score
                    best_idx = i
            return best_idx

        else:
            if drafted_counts:
                dominant = drafted_counts.most_common(1)[0][0]
                allied = get_allied_tides(dominant)
                for i, t in enumerate(pack_tides):
                    if t == dominant:
                        return i
                for i, t in enumerate(pack_tides):
                    if t in allied:
                        return i
            return random.randrange(len(pack_tides))

    return strategy


def two_tide_strategy(tide_a, tide_b):
    targets = {tide_a, tide_b}

    def strategy(pack_tides, drafted, pick, pool_counts):
        for i, t in enumerate(pack_tides):
            if t in targets:
                return i
        return random.randrange(len(pack_tides))

    return strategy


# ---- Factories ----


def make_mono_factory():
    def factory():
        target = random.choice(CORE_TIDES)
        allied = get_allied_tides(target)
        strat = mono_tide_strategy(target)
        return strat, lambda d: [target], lambda d: allied

    return factory


def make_pivot_factory():
    def factory():
        tide_a = random.choice(CORE_TIDES)
        allies_a = get_allied_tides(tide_a)
        distant = [t for t in CORE_TIDES if t != tide_a and t not in allies_a]
        tide_b = random.choice(distant) if distant else random.choice(CORE_TIDES)
        allied_b = get_allied_tides(tide_b)
        strat = pivot_strategy(tide_a, tide_b)
        return strat, lambda d: [tide_b], lambda d: allied_b

    return factory


def _dynamic_dominant(drafted):
    core = [t for t in drafted if t != NEUTRAL]
    if not core:
        return CORE_TIDES[:1]
    return [Counter(core).most_common(1)[0][0]]


def _dynamic_allied(drafted):
    dom = _dynamic_dominant(drafted)
    return get_allied_tides(dom[0]) if dom else []


def make_signal_reader_factory():
    def factory():
        strat = signal_reader_strategy()
        return strat, _dynamic_dominant, _dynamic_allied

    return factory


def make_pool_reader_factory():
    def factory():
        strat = pool_reader_strategy()
        return strat, _dynamic_dominant, _dynamic_allied

    return factory


def make_two_tide_factory():
    def factory():
        t1 = random.choice(CORE_TIDES)
        idx = CORE_TIDES.index(t1)
        t2 = CORE_TIDES[(idx + 1) % 7]
        strat = two_tide_strategy(t1, t2)
        targets = [t1, t2]
        all_allied = set(get_allied_tides(t1) + get_allied_tides(t2)) - {t1, t2}
        return strat, lambda d: targets, lambda d: list(all_allied)

    return factory


STRATEGIES = {
    "Mono-Tide": make_mono_factory,
    "Signal Reader": make_signal_reader_factory,
    "Pool Reader": make_pool_reader_factory,
    "Pivot@8": make_pivot_factory,
}


# ---- Scenario Running ----


def run_scenario(card_pool, strategy_factory, variant, num_trials, num_picks=NUM_PICKS):
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
        for p in range(1, num_picks + 1)
    }

    featured_alignments = []
    valid_trials = 0
    attempts = 0
    max_attempts = num_trials * 3

    while valid_trials < num_trials and attempts < max_attempts:
        attempts += 1
        strategy, dominant_fn, allied_fn = strategy_factory()
        result = simulate_draft(
            card_pool, strategy, dominant_fn, allied_fn, variant, num_picks
        )
        if result is None:
            continue

        trial_metrics, featured_aligned, featured = result
        if trial_metrics is None or all(m is None for m in trial_metrics):
            continue

        valid_trials += 1
        if featured_aligned is not None:
            featured_alignments.append(featured_aligned)

        for pick, m in enumerate(trial_metrics, 1):
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

    alignment_rate = (
        sum(featured_alignments) / len(featured_alignments)
        if featured_alignments
        else None
    )
    return per_pick, valid_trials, alignment_rate


def avg(lst):
    return sum(lst) / len(lst) if lst else 0.0


def print_results(name, per_pick, num_trials, alignment_rate=None, picks_to_show=None):
    if picks_to_show is None:
        picks_to_show = sorted(per_pick.keys())
    print(
        f"{'Pick':>4}  {'Dom':>5}  {'Ally':>5}  {'Neut':>5}  {'Dist':>5}"
        f"  {'Pool':>5}  {'P>=1':>5}  {'P>=2':>5}  {'P>=3':>5}"
    )
    print(f"{'-' * 55}")

    for pick in picks_to_show:
        b = per_pick[pick]
        if b["count"] == 0:
            continue
        n = b["count"]
        print(
            f"{pick:>4}  {avg(b['dominant']):>5.2f}  {avg(b['allied']):>5.2f}"
            f"  {avg(b['neutral']):>5.2f}  {avg(b['distant']):>5.2f}"
            f"  {avg(b['pool_size']):>5.0f}"
            f"  {b['ge1']/n:>5.2f}  {b['ge2']/n:>5.2f}  {b['ge3']/n:>5.2f}"
        )

    if alignment_rate is not None:
        print(f"  Featured-tide alignment: {alignment_rate:.1%}")


def run_variant(card_pool, variant, num_trials, key_picks, strategies=None):
    if strategies is None:
        strategies = STRATEGIES
    print(f"\n{'=' * 70}")
    print(f"  {variant.name}")
    print(f"  {variant.description}")
    n_exc = variant.initial_tide_exclusion
    fw = variant.featured_weight
    removal = (
        variant.non_featured_removal_rate if variant.featured_tide_count > 0 else 0
    )
    fr = variant.focus_rate
    print(f"  N={n_exc} | weight={fw}x | removal={removal} | focus_rate={fr}")
    print(f"{'=' * 70}")

    for strat_name, factory_fn in strategies.items():
        print(f"\n  --- {strat_name} ({num_trials} trials) ---")
        results, trials, alignment = run_scenario(
            card_pool, factory_fn(), variant, num_trials
        )
        print_results(strat_name, results, trials, alignment, key_picks)


def main():
    print("Loading card pool...")
    card_pool = load_card_pool()
    tide_counts = Counter(card_pool)
    print(f"Loaded {len(card_pool)} cards: {dict(tide_counts)}")

    NUM_TRIALS = 3000
    KEY_PICKS = [1, 3, 5, 7, 10, 15, 20, 25]

    variants = [
        # Baseline
        Variant(
            "BASELINE",
            "Current algorithm: no bias",
        ),
        # Weighted bias sweep: featured_weight multiplier (no physical removal)
        Variant(
            "WEIGHT 1.5x",
            "Featured pair gets 1.5x sampling weight",
            featured_weight=1.5,
        ),
        Variant(
            "WEIGHT 2.0x",
            "Featured pair gets 2.0x sampling weight",
            featured_weight=2.0,
        ),
        Variant(
            "WEIGHT 2.5x",
            "Featured pair gets 2.5x sampling weight",
            featured_weight=2.5,
        ),
        Variant(
            "WEIGHT 3.0x",
            "Featured pair gets 3.0x sampling weight",
            featured_weight=3.0,
        ),
        Variant(
            "WEIGHT 4.0x",
            "Featured pair gets 4.0x sampling weight",
            featured_weight=4.0,
        ),
        # Weighted bias with higher focus
        Variant(
            "WEIGHT 2.5x FR=0.40",
            "Featured 2.5x + focus_rate 0.40",
            featured_weight=2.5,
            focus_rate=0.40,
        ),
        Variant(
            "WEIGHT 3.0x FR=0.40",
            "Featured 3.0x + focus_rate 0.40",
            featured_weight=3.0,
            focus_rate=0.40,
        ),
        # Compare: physical removal 40% vs weight equivalent
        Variant(
            "REMOVAL 40% (reference)",
            "Physical removal for comparison",
            featured_tide_count=2,
            non_featured_removal_rate=0.40,
        ),
    ]

    for variant in variants:
        run_variant(card_pool, variant, NUM_TRIALS, KEY_PICKS)


if __name__ == "__main__":
    main()
