# Game Feel Audit Report

**Date:** 2026-03-14
**Auditor:** `game_feel_auditor.py`
**Seed:** 42 (all runs reproducible)

## Executive Summary

Quest-mode drafts consistently fail the frustration rate target: mean frustration rate is **0.30** (target: < 0.20) and P90 is **0.45** (target: < 0.35). Nearly one-third of post-commitment picks offer the player 0 or 1 on-plan options out of 4 shown cards. The problem is uniform across all 8 archetypes and follows a clear sawtooth pattern tied to pack boundaries, with on-plan density collapsing at the end of each 10-pick pack (picks 8-9, 17-19, 26-29).

## Methodology

The game feel auditor simulates quest-mode drafts using these parameters:
- **6 seats**, 20-card packs, show-N = 4, WITH_REPLACEMENT, 30 picks total
- **Sharpened preference** show-N strategy (w^4.0 exponent)
- **Adaptive** AI agent policy with 0.80 optimality
- **Frustrating pick** defined as: <= 1 on-plan option (fitness >= threshold) after resonance filtering
- **Frustration rate** = fraction of post-commitment picks that are frustrating
- **Targets:** mean frustration rate < 0.20, P90 < 0.35, mean max streak < 3, P90 max streak < 5

Runs performed:
1. Baseline: 200 runs, threshold 0.5
2. Per-archetype: 100 runs x 8 archetypes (800 total)
3. Comparison with v2 convergence metrics: 200 runs
4. Threshold sensitivity: 200 runs each at 0.4, 0.5, 0.6
5. Verbose worst-case: 200 runs with per-pick detail for worst 10%

## Key Findings

### 1. Overall Frustration Metrics

| Metric | Value | Target | Status |
|---|---|---|---|
| Mean frustration rate | 0.30 | < 0.20 | **WARN** |
| P90 frustration rate | 0.45 | < 0.35 | **WARN** |
| Mean max frustration streak | 2.6 | < 3 | PASS |
| P90 max frustration streak | 4.0 | < 5 | PASS |
| Mean zero-option picks | 3.3 | — | — |

The mean frustration rate of 0.30 means nearly 1 in 3 post-commitment picks
are frustrating (0-1 on-plan options). Players see 3.3 picks per draft with
**zero** on-plan options — not even a single viable card among the 4 shown.

### 2. Per-Pick On-Plan Density Curve (Sawtooth Pattern)

The on-plan density follows a clear **sawtooth pattern** aligned with 10-pick pack boundaries. Each pack starts with ~3 on-plan options and degrades to ~1.4 by the end:

```
Pack 1 (picks 1-9):   2.4  2.7  3.0  2.6  2.4  2.4  2.1  1.8  1.6
Pack 2 (picks 10-19):  3.2  3.1  3.1  2.8  2.6  2.4  2.0  1.9  1.9  1.4
Pack 3 (picks 20-29):  3.0  2.9  2.8  2.7  2.6  2.5  1.9  1.7  1.6  1.5
```

The worst picks are consistently picks **9** (1.6), **19** (1.4), and **29** (1.5) — the final picks of each pack. This is expected: by the end of a pack, most on-plan cards have been claimed by earlier picks.

The drop-off is sharpest in the **middle** of the draft (pack 2, picks 16-19), where on-plan density falls to 1.4-2.0. This coincides with the "feels random after pick ~6" complaint — by the time a player has made 6+ post-commitment picks, they've entered the depleted tail of a pack.

### 3. Per-Archetype Variation

All archetypes show remarkably similar frustration profiles (100 runs each, forced resonance filtering):

| Archetype | Resonance | Mean Frust. | P90 Frust. | Mean Max Streak | Res. Impact |
|---|---|---|---|---|---|
| Flash | Thunder/Tide | 0.33 | 0.45 | 2.9 | -0.2 |
| Awaken | Thunder/Flame | 0.33 | 0.48 | 3.0 | -0.3 |
| Flicker | Flame/Thunder | 0.33 | 0.48 | 3.0 | -0.3 |
| Ignite | Flame/Stone | 0.32 | 0.48 | 2.8 | -0.2 |
| Shatter | Stone/Flame | 0.32 | 0.48 | 2.8 | -0.2 |
| Endure | Stone/Tide | 0.32 | 0.47 | 2.9 | -0.2 |
| Submerge | Tide/Stone | 0.32 | 0.47 | 2.9 | -0.2 |
| Surge | Tide/Thunder | 0.33 | 0.45 | 2.9 | -0.2 |
| **Combined** | **all** | **0.33** | **0.48** | **2.9** | **-0.2** |

Key observations:
- **No archetype outlier.** The problem is systemic, not archetype-specific. All archetypes have mean frustration 0.32-0.33.
- Awaken and Flicker (sharing Thunder/Flame resonance pair) show the highest P90 (0.48) and highest resonance impact (-0.3), likely because Thunder and Flame archetypes compete more for the same card pool.
- With forced resonance filtering (vs auto-detection in baseline), frustration rises from 0.30 to 0.33, suggesting the auto-detected resonance pair sometimes differs from the forced pair, slightly masking the issue.

### 4. Resonance Filtering Impact

Resonance filtering removes an average of **0.1-0.3 on-plan cards per pick**, depending on the specific resonance pair. With auto-detected resonance (baseline), the impact is -0.1; with forced resonance (per-archetype), it's -0.2 to -0.3.

This is a **minor contributor** to frustration. Even without resonance filtering, the density at end-of-pack picks would only improve from ~1.5 to ~1.7 — still below the 2.0 threshold where picks feel comfortable.

### 5. Threshold Sensitivity: Fitness Values Are Binary

A critical finding: **all three threshold settings (0.4, 0.5, 0.6) produce identical results.** Investigation reveals that card fitness values are strictly binary — either 0.0 or 1.0, never intermediate:

```
Fitness value distribution across all cards x archetypes:
  0.0: 3141 (79.1%)
  1.0:  827 (20.9%)
```

This means the `strict_threshold` parameter is effectively meaningless — a card either fully belongs to an archetype or doesn't belong at all. There is no "somewhat on-plan" middle ground for the threshold to discriminate.

**Implication:** The current card-archetype fitness model is a hard binary assignment. Each card belongs to exactly ~2 archetypes (827/~500 unique cards / 8 archetypes ≈ 2 archetypes per card on average). A more graduated fitness model (e.g., 0.0 / 0.3 / 0.7 / 1.0) would allow "splash" cards to soften the frustration curve.

### 6. v2 Convergence Metrics vs Game-Feel Metrics (The Divergence)

The v2 convergence metrics **all pass** while game-feel metrics fail:

| v2 Metric | Value | Target | Status |
|---|---|---|---|
| Convergence (mid) | 2.4 ± 0.1 | >= 2.0 | PASS |
| Convergence (late) | 2.4 ± 0.1 | >= 2.0 | PASS |
| Choice richness | 2.0 ± 0.0 | >= 1.5 | PASS |
| Splashability | 0.51 ± 0.02 | >= 0.40 | PASS |
| Early openness | 7.6 ± 0.1 | >= 5.0 | PASS |

**Why they diverge:**

1. **v2 uses a permissive threshold (>= 0.3)** while game-feel uses strict (>= 0.5). Since fitness is binary (0 or 1), this shouldn't matter — and indeed doesn't. Both count the same cards.

2. **v2 measures convergence as the number of archetypes with high affinity**, not the per-pick on-plan density. A drafter can "converge" to 2 archetypes while still seeing 0 on-plan cards in a depleted pack.

3. **v2 doesn't model pack depletion.** Choice richness counts near-optimal options across the whole draft, averaging away the sawtooth depletion pattern. A draft with 4 great options for picks 1-5 and 0 for picks 6-9 averages to 2.0 — "passing" despite 4 frustrating picks.

4. **v2 doesn't track frustration streaks.** A player who sees zero on-plan cards for 3 consecutive picks experiences compounding frustration that no single-pick metric captures.

### 7. Worst-Case Analysis (P90 Tail)

The worst 10% of drafts (frustration rate >= 0.45) include extreme cases:

| Seed | Frust. Rate | Max Streak | Archetype | Pattern |
|---|---|---|---|---|
| 2510777721 | 0.72 | 18 | Flicker | 18-pick streak of 0 on-plan from pick 12-29 |
| 636057975 | 0.67 | 7 | Submerge | Scattered 0-option picks throughout |
| 3713453204 | 0.64 | 6 | Ignite | Late-draft collapse |
| 1699887270 | 0.62 | 6 | Awaken | Mid-draft dead zone |
| 965067727 | 0.59 | 9 | Endure | 9-pick streak, picks 14-22 |
| 2085812759 | 0.59 | 4 | Endure | Frequent short frustrating runs |

**Worst single run (seed 2510777721, Flicker):** After pick 12, the drafter sees **zero** on-plan cards for 18 of the remaining 19 picks — an essentially broken draft. Verbose output shows fitness = 0.00 (no archetype match at all) on most of these picks. The committed archetype's cards were exhausted from the pack, and subsequent packs didn't provide alternatives.

A common verbose pattern in bad runs: fitness values are **all-or-nothing**. When on-plan cards appear, they come in clusters of 2-4; when they don't, the player sees 0 with best_fit = 0.00. There is no middle ground.

## Root Cause Analysis

Ranked by measured impact on frustration:

### 1. Pack Depletion Dynamics (Primary Cause)

**Impact:** Directly responsible for the sawtooth pattern. On-plan density drops from ~3.0 at pack start to ~1.4 at pack end.

With 20-card packs, 6 seats, and show-N=4, each seat sees 4 cards per pick. By pick 8-9 of a pack, roughly 48 cards have been shown/drafted across all seats. Even with replacement, the sharpened preference algorithm (w^4.0) concentrates shown cards around popular archetypes, leaving late-pick players with the dregs.

### 2. Binary Fitness Model (Amplifier)

**Impact:** Eliminates the "soft landing" between fully on-plan and fully off-plan. A graduated fitness model would provide partially useful cards (0.3-0.7 fitness) even when perfect matches are exhausted.

The 79/21 split (off-plan/on-plan) means each card is on-plan for only ~2 of 8 archetypes. In a 4-card show-N, the expected number of on-plan cards from a random draw is 4 × 0.209 = 0.84. The show-N sharpening lifts this significantly early in a pack but can't overcome depletion.

### 3. Show-N Sharpening Exponent (w^4.0) (Contributor)

**Impact:** The w^4.0 exponent heavily concentrates shown cards around the player's top archetype preferences. This is good early (high on-plan density) but exacerbates depletion by consuming on-plan cards faster.

### 4. Resonance Filtering (Minor Contributor)

**Impact:** -0.1 to -0.3 cards/pick. The least impactful factor. Even removing resonance filtering entirely would not bring frustration below the 0.20 target.

### 5. Archetype Count (Structural)

**Impact:** With 8 archetypes and binary fitness, each card serves only ~2 archetypes. The card pool isn't dense enough in any single archetype to sustain 30 picks of on-plan options. This is structural but would require significant card design changes to address.

## Recommendations

### Priority 1: Introduce Pack Refresh or Larger Packs

The sawtooth depletion is the #1 frustration driver. Options:
- **Larger packs** (24-28 cards instead of 20) to extend the high-density window
- **Mid-pack refresh** adding 4-6 new cards halfway through each pack
- **Faster pack turnover** (3 packs of 10 picks instead of implicit 10-pick rounds from a single pack)

### Priority 2: Graduate the Fitness Model

Introducing intermediate fitness values (e.g., 0.3 for "splash-viable" cards) would:
- Create a soft landing when perfect matches run out
- Make the strict_threshold parameter actually meaningful
- Improve the "feel" without changing pack structure

### Priority 3: Soften Show-N Sharpening Late in Pack

Reduce the sharpening exponent dynamically: start at w^4.0 for early picks, decay to w^2.0 or w^1.0 for picks 7+ within a pack. This would spread shown cards more evenly across archetypes as the pack depletes, improving late-pick density.

### Priority 4: Reduce Pack-End Dead Zones

Consider a "mercy" mechanism: if the best fitness among shown cards is 0.0, reshuffle and try again (with a limit). This directly prevents the 0-option picks that drive the worst frustration streaks.

### Priority 5: Update v2 Convergence Metrics

Add a per-pick minimum density metric (e.g., "P10 on-plan density >= 1.0") to catch the depletion problem. The current metrics average away the issue, creating a false sense of quality.

## Reproducibility

All results can be reproduced with:
```bash
cd scripts/draft_simulator_v2/
python3 game_feel_auditor.py --runs 200 --seed 42                    # baseline
python3 game_feel_auditor.py --runs 100 --seed 42 --all-resonances   # per-archetype
python3 game_feel_auditor.py --runs 200 --seed 42 --compare          # v2 comparison
python3 game_feel_auditor.py --runs 200 --seed 42 --verbose          # worst-case detail
```
