# Results 5: D2 Sentinel Draft Simulation

## Scorecard — Graduated Realistic (Sentinel = Level-0, see below)

| Metric | Committed | Power-Chaser | Signal-Reader | Target | Pass? |
|--------|:---------:|:------------:|:-------------:|--------|:-----:|
| M1     | 2.90      | 2.90         | 2.90          | >= 3   | FAIL  |
| M2     | 0.58      | 0.54         | 0.65          | <= 2   | PASS  |
| M3     | 0.39      | 0.72         | 0.42          | >= 2.0 | FAIL  |
| M4     | 3.13      | 2.72         | 3.08          | >= 0.5 | PASS  |
| M5     | 29.1      | 27.1         | 28.7          | 5-8    | FAIL  |
| M6     | 30.9%     | 21.9%        | 26.8%         | 60-90% | FAIL  |
| M7     | 3.3%      | 3.3%         | 3.1%          | < 40%  | PASS  |
| M8     | 11-15%    | 11-14%       | 11-15%        | 5-20%  | PASS  |
| M9     | 0.43      | 0.65         | 0.45          | >= 0.8 | FAIL  |
| M10    | 5.16      | 4.32         | 5.08          | <= 2   | FAIL  |
| M11    | 0.00      | 0.00         | 0.00          | >= 3.0 | FAIL  |

**Average picks completed: 11.0 / 30.** Pool exhausted by pick 12.

### Pessimistic (Sibling Rates -10pp)

| Variant  | Strategy      | M3   | M10  | M11  |
|----------|---------------|:----:|:----:|:----:|
| Sentinel | committed     | 0.39 | 5.15 | 0.00 |
| Sentinel | signal_reader | 0.42 | 5.07 | 0.00 |
| Level-0  | committed     | 0.39 | 5.15 | 0.00 |
| Level-0  | signal_reader | 0.42 | 5.07 | 0.00 |

## Per-Archetype M3 (Committed)

| Archetype    | M3   |
|--------------|:----:|
| Flash        | 0.31 |
| Blink        | 0.35 |
| Storm        | 0.34 |
| Self-Discard | 0.41 |
| Self-Mill    | 0.39 |
| Sacrifice    | 0.47 |
| Warriors     | 0.49 |
| Ramp         | 0.32 |

Tracks the sibling-rate gradient (Warriors/Sacrifice highest). All catastrophically below 2.0.

## Pack Quality Distribution (Picks 6+, Committed)

| p10 | p25 | p50 | p75 | p90 |
|:---:|:---:|:---:|:---:|:---:|
| 0   | 0   | 0   | 1   | 1   |

Median pack has zero S/A cards. Average max consecutive bad packs: 5.16.

## Draft Traces

**Trace 1 (committed, Sentinel):** AIs on Sacrifice/Warriors/Ramp/Blink/Flash/Self-Discard. Open: Storm, Self-Mill. Player commits to Blink (contested) at pick 5. Sees 0-1 S/A per pack. Pool exhausted at pick 12. Only 3 packs out of 11 contained any S/A card for Blink.

**Trace 2 (signal_reader, Sentinel):** Same seed. Player commits to Self-Discard (contested) at pick 5. Packs 7, 9, 10 show S/A hits via Self-Mill sibling cards. Still exhausted at pick 12. Late concentration is visible (3 S/A at pick 10) but the draft ends immediately after.

## Sentinel vs Level-0: Delta = Zero

**All metrics are numerically identical between Sentinel (reactive) and Level-0 (static).**

Phase 1 is identical by design (picks 1-7, same seed). Phase 2 kicks in at picks 8-10 but by then the pool has shrunk to 84/57/31 cards. With so few remaining, re-evaluating the pool produces the same top-4 picks as the predetermined list. The staggered transition never gets a chance to differentiate.

**Answer to the key question: Level 1 pool-pressure reactivity produces zero measurably better M10 than Level 0.** The experiment is conclusive but for the wrong reason -- reactivity cannot be tested when the pool is exhausted before Phase 2 activates meaningfully.

## Pool Depletion

| Pick | Pool | AI Removed | Culled | Contraction |
|:----:|:----:|:----------:|:------:|:-----------:|
| 1    | 320  | 24         | 16     | 11.1%       |
| 5    | 175  | 24         | 9      | 14.4%       |
| 8    | 84   | 24         | 4      | 21.2%       |
| 10   | 31   | 24         | 1      | 38.5%       |
| 12   | 0    | 4          | 0      | 100%        |

6 AIs x 4 picks = 24 fixed removals from a shrinking pool. The 360-card pool cannot survive 30 rounds at this removal rate.

## Comparison to V9

| Metric | V9 Hybrid B | Sentinel D2 | Delta  |
|--------|:-----------:|:-----------:|:------:|
| M3     | 2.70        | 0.39        | -2.31  |
| M10    | 3.8         | 5.16        | +1.36  |
| M11    | 3.25        | 0.00        | -3.25  |

## Self-Assessment

**Sentinel Draft fails comprehensively.** Passes only M2, M4, M7, M8. Fails M1, M3, M5, M6, M9, M10, M11.

**Root cause: pool math.** 24 AI picks + 5% cull per round exhausts the 360-card pool in 12 rounds. A 30-pick draft is impossible. The design document states "~29-35 cards removed per round" without confronting that 360/30 = 12 rounds maximum.

**The reactivity question is moot.** Phase 2 cannot be evaluated because the pool dies before it activates. To test reactivity, the design needs either a much larger pool (>1000), far fewer AI picks (1-2/round), or a different pacing model.

**Fixes:** (1) Reduce AI picks to 1-2/round (lasting 20+ rounds, matching V9's ~12% contraction). (2) Reduce to 3-4 AIs with 4 picks each. (3) Use 1 AI pick/round with heavier culling to control pacing. The 2-open-lane structure may have merit once pool math is viable, but cannot be assessed in its current form.
