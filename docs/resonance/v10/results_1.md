# D1 Open Table — Simulation Results

## Scorecard: Graduated Realistic (Committed Player)

| Metric | Value | Target | Status |
|--------|------:|--------|:------:|
| M1 (unique archs S/A, picks 1-5) | 4.50 | >= 3 | PASS |
| M2 (emerging S/A, picks 1-5) | 1.01 | <= 2 | PASS |
| M3 (committed S/A, picks 6+) | 0.60 | >= 2.0 | FAIL |
| M4 (off-arch C/F, picks 6+) | 3.40 | >= 0.5 | PASS |
| M5 (convergence pick) | 26.7 | 5-8 | FAIL |
| M6 (deck concentration) | 48.4% | 60-90% | FAIL |
| M7 (run-to-run overlap) | 2.4% | < 40% | PASS |
| M8 (arch freq max/min) | 16.2%/10.3% | none>20%/none<5% | PASS |
| M9 (S/A StdDev, picks 6+) | 0.60 | >= 0.8 | FAIL |
| M10 (max consec < 1.5 S/A) | 5.3 | <= 2 | FAIL |
| M11 (committed S/A, picks 15+) | 0.00 | >= 3.0 | FAIL |

**Result: 5/11 passed. Structural failure.**

## Pessimistic Fitness (Committed Player)

| Metric | Graduated | Pessimistic |
|--------|----------:|------------:|
| M3 | 0.60 | 0.52 |
| M10 | 5.3 | 5.6 |
| M11 | 0.00 | 0.00 |

Pessimistic fitness worsens already-failing metrics. No additional insight.

## Per-Archetype M3 (Committed Player, Graduated Realistic)

| Archetype | M3 (picks 6+) |
|-----------|---------------:|
| Flash | 0.47 |
| Blink | 0.57 |
| Storm | 0.58 |
| Self-Discard | 0.67 |
| Self-Mill | 0.70 |
| Sacrifice | 0.67 |
| Warriors | 0.60 |
| Ramp | 0.51 |

Higher-sibling-rate pairs (Sacrifice/Warriors at 50%, Self-Mill/Self-Discard at
40%) show marginally higher M3, but all values are far below the 2.0 target.
The spread (0.47 to 0.70) is small relative to the deficit.

## Pack Quality Distribution (S/A per pack, picks 6+, Committed)

| Percentile | Value |
|:----------:|:-----:|
| p10 | 0 |
| p25 | 0 |
| p50 | 0 |
| p75 | 1 |
| p90 | 2 |

Most packs contain zero S/A cards for the committed archetype. 75th percentile
is 1 card. The player sees useful cards in fewer than 25% of late packs.

## Consecutive Bad Pack Analysis (Committed)

| Streak Length | Occurrences |
|:------------:|:-----------:|
| 1 | 300 |
| 2 | 236 |
| 3 | 181 |
| 4 | 138 |
| 5 | 119 |
| 6 | 72 |
| 7 (terminal) | 456 |

The "length 7" spike represents drafts where the pool was exhausted — all
remaining packs from pick 6 onward contained zero S/A cards, producing a
terminal streak that lasted until the pool ran dry.

## AI Drafter Behavior Summary

- **Total AI picks per draft:** 248 (5 AIs x 4 cards x ~12.4 productive rounds)
- **Card distribution:** Uniform across archetypes (11.3-11.5% per archetype,
  9.1% generic). AIs do not preferentially deplete their home archetype.
- **Pool depletion timeline:** 360 -> 329 -> 298 -> ... -> 50 (round 10) ->
  29 (round 11) -> 8 (round 12) -> pool exhausted at round 13.
- **Productive draft length:** ~12 rounds. Picks 13-30 draw from a pool of
  fewer than 8 cards; packs are mostly recycled remnants.

The fundamental issue: AIs remove 20 cards per round and culling removes 10,
totaling 31 per round. A 360-card pool survives only ~12 rounds at this rate.
The design spec assumed a 30-round draft, but the pool cannot support it.

## Example Draft Traces

### Trace 1: Committed Player

**Setup:** AIs on Storm, Self-Mill, Self-Discard, Flash, Sacrifice. Open lanes:
Blink, Warriors, Ramp. Player committed to Self-Mill (a contested lane).

| Pick | Pool | Pack (archetype, power, S/A?) | Chosen | Notes |
|------|------|-------------------------------|--------|-------|
| 1 | 360 | Sacrifice(8.4), Flash(2.8), Ramp(3.4), SD(3.1) | Sacrifice(8.4) | Pre-commit |
| 5 | 236 | Ramp(8.1), Ramp(6.8), Sacrifice(6.6), Generic(6.9) | Ramp(8.1,C/F) | Commits Self-Mill; no S/A in pack |
| 8 | 143 | Warriors(6.0), Warriors(8.0), Ramp(4.7), Warriors(6.2) | Warriors(8.0,C/F) | Zero Self-Mill cards |
| 10 | 81 | Generic(5.0), Ramp(7.3), Blink(5.2), Blink(7.5) | Blink(7.5,C/F) | Pool nearly empty |

Player committed to a contested lane (Self-Mill AI active) and found almost no
S/A cards after commitment. Pool at pick 10: only 1 Self-Mill card remained.

### Trace 2: Signal-Reader

**Setup:** Same AI composition. Player committed to Warriors (open lane).

| Pick | Pool | Pack highlights | Chosen | Notes |
|------|------|----------------|--------|-------|
| 5 | 236 | Sacrifice(6.6,S/A), others C/F | Sacrifice(6.6) | Commits Warriors (most remaining cards) |
| 7 | 174 | Warriors(8.5,S/A), Warriors(8.0,S/A) | Warriors(8.5) | Open lane delivers |
| 8 | 143 | Warriors(4.5,S/A), Warriors(8.0,S/A), Warriors(5.6,S/A) | Warriors(8.0) | 3 S/A in one pack |
| 10 | 81 | No Warriors cards | Blink(7.5,C/F) | Pool depleted |

Signal-reader found the open lane and got strong packs in rounds 7-9, but the
pool was exhausted by round 10. Even correct signal reading cannot overcome
structural pool depletion.

## V9 Hybrid B Comparison

| Metric | V9 Hybrid B | D1 Open Table | Delta |
|--------|:-----------:|:-------------:|:-----:|
| M3 | 2.70 | 0.60 | -2.10 |
| M5 | 9.6 | 26.7 | +17.1 |
| M6 | 86% | 48.4% | -37.6% |
| M10 | 3.8 | 5.3 | +1.5 |
| M11 | 3.25 | 0.00 | -3.25 |

D1 is catastrophically worse than V9 Hybrid B on every convergence metric. The
signal-reader strategy (M3=1.00) performs slightly better but still far below
V9 baseline.

## Root Cause Analysis

The D1 Open Table has a single structural flaw that causes all metric failures:

**The pool is exhausted by round 12.** With 5 AIs taking 4 cards each (20/round)
plus 10 culled plus 1 player pick, the pool loses 31 cards per round. A 360-card
pool survives only 11.6 rounds. Picks 12-30 have no meaningful pool to draw from.

V9's percentage-based contraction (12% per pick) naturally decelerates: it removes
~43 cards at pick 1 (pool=360) but only ~14 cards at pick 20 (pool=120). This
self-regulating behavior allows V9 to sustain the draft for 30 rounds.

D1's flat removal rate does not decelerate. It removes 31 cards at pick 1 and
still tries to remove 31 at pick 10. The pool minimum for culling (40 cards)
stops culling, but AIs still try to take 20 cards per round, exhausting the
remaining pool in 1-2 additional rounds.

A secondary issue: AIs pick cards uniformly across archetypes (11.3-11.5% each)
rather than concentrating on their home archetype. The pair-affinity scoring
provides only modest differentiation. A Storm AI takes its best Storm cards early,
then shifts to adjacent archetypes because Storm cards are exhausted.

## Self-Assessment: What Would Fix This

1. **Reduce flat removal to ~10 cards/round total** (3 AIs x 2 picks + 4 cull).
   This gives 360/11 = 32 rounds of pool life. But this likely produces M3 ~1.0,
   far below target.

2. **Switch to percentage-based AI removal** — each AI takes a percentage of the
   remaining pool rather than a fixed count. This replicates V9's self-regulating
   behavior. But this changes the AI drafter narrative: "each AI takes 3% of the
   remaining cards" is less intuitive than "each AI takes 4 cards."

3. **Increase pool size dramatically** (to 900+) so that 31 removals per round
   sustains 30 rounds. But this requires generating far more cards.

4. **Have AIs "draft a deck" of fixed size** (e.g., 30 cards total over 30
   rounds = 1 card per AI per round). Total removal: 5 + 10 cull + 1 = 16/round.
   Pool lasts 22 rounds. Still insufficient for M11 but closer to viable.

The honest conclusion: the D1 Open Table as designed cannot work with a 360-card
pool and 30-pick draft. The flat removal rate is structurally incompatible with
sustaining the draft. Any viable D1 variant must either (a) dramatically reduce
per-round removal, (b) adopt percentage-based scaling, or (c) accept a shorter
draft (~12 picks). Option (b) is the most promising but transforms D1 into V9
with an AI narrative layer — which, as the critic review noted, may be the right
answer.
