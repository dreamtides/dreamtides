# SIM-4 Results: Graduated 4-Round Decline, No Bias

## Algorithm

4 rounds (8/8/7/7 picks = 30 total), 120-card starting pool (8 archetypes x 15
cards). Declining balanced refills: 48/36/21/0 cards after rounds 1-4. No refill
bias -- all refills distribute cards equally across archetypes. 5 Level-0 AIs
with declining saturation thresholds (9/9/8/7). Tests whether declining refill
volume alone can build adequate concentration without open-lane bias.

## Full Scorecard (Committed Player, Graduated Realistic)

| Metric | Value | Target | Status |
|--------|------:|--------|--------|
| M1 (unique archs S/A, 1-5) | 5.66 | >= 3 | PASS |
| M2 (emerging S/A, 1-5) | 1.33 | <= 2 | PASS |
| M3 (committed S/A, 6+) | **0.83** | >= 2.0 | **FAIL** |
| M4 (off-arch C/F, 6+) | 4.17 | >= 0.5 | PASS |
| M5 (convergence pick) | 16.83 | 5-8 | FAIL |
| M6 (deck concentration) | 58.8% | 60-90% | FAIL |
| M7 (run-to-run overlap) | 12% | < 40% | PASS |
| M8 (arch freq max/min) | 15.2%/9.1% | <=20%/>=5% | PASS |
| M9 (S/A stddev, 6+) | 0.80 | >= 0.8 | FAIL |
| M10 (max consec bad) | **10.63** | <= 2 | **FAIL** |
| M11' (S/A, picks 20+) | **0.71** | >= 2.5 | **FAIL** |
| M12 (signal - committed) | 0.16 | >= 0.3 | FAIL |

**5/11 metrics passed. Total failure on all concentration metrics.**

## Per-Archetype M3 Table

| Archetype | M3 | Frequency |
|-----------|---:|----------:|
| Flash | 0.74 | 13.5% |
| Blink | 0.74 | 10.6% |
| Storm | 0.78 | 11.8% |
| Self-Discard | 0.89 | 13.9% |
| Self-Mill | 0.89 | 15.1% |
| Sacrifice | 0.99 | 15.2% |
| Warriors | 0.87 | 10.8% |
| Ramp | 0.68 | 9.1% |

All 8 archetypes fail M3. Higher-sibling-rate pairs (Sacrifice/Warriors at 50%,
Self-Discard/Self-Mill at 40%) show modestly better M3 (0.87-0.99) versus
low-rate pairs (Flash/Ramp at 0.68-0.74). The sibling rate effect is real but
insufficient -- even the best archetype (Sacrifice at 0.99) is half the 2.0
target.

## Round-by-Round Pool Composition

| Event | Pool Size | Per-Arch Avg | Open Lane Avg | AI Lane Avg | Gradient |
|-------|----------:|:------------:|:-------------:|:-----------:|:--------:|
| R1 start | 120 | 15.0 | 15.0 | 15.0 | 1.00x |
| R1 end (pre-refill) | ~72 | ~9.0 | ~10.5 | ~8.1 | ~1.30x |
| R2 start (post-refill) | 120 | 15.0 | 15.0 | 15.0 | **1.00x** |
| R2 end (pre-refill) | ~72 | ~9.0 | ~10.2 | ~8.3 | ~1.23x |
| R3 start (post-refill) | 108 | 13.5 | 13.6 | 13.4 | **1.01x** |
| R3 end (pre-refill) | ~66 | ~8.3 | ~9.5 | ~7.6 | ~1.25x |
| R4 start (post-refill) | 87 | 10.9 | 11.0 | 10.8 | **1.02x** |
| R4 end | ~45 | ~5.6 | ~6.5 | ~5.1 | ~1.27x |

**The refill reset is catastrophic.** Each balanced refill compresses the
gradient back to approximately 1.0x. The AIs build a modest 1.23-1.30x gradient
within each round, but the balanced refill completely erases it. Even with
declining refill volumes (48, 36, 21), the gradient never exceeds 1.30x at any
point in the draft.

The averaged pool composition data confirms this: at every round start, all
archetypes are within 0.3 cards of each other. The balanced refill treats all
archetypes identically, restocking AI-depleted lanes and open lanes equally.

## Pack Quality Distribution (picks 6+)

| Percentile | S/A Cards |
|:----------:|:---------:|
| p10 | 0 |
| p25 | 0 |
| p50 | 1 |
| p75 | 1 |
| p90 | 2 |

The median pack contains just 1 S/A card out of 5. The p25 is 0 -- one quarter
of all post-commitment packs contain zero playable cards for the committed
archetype. This is a catastrophically bad player experience.

## Consecutive Bad Pack Analysis

Mean max streak of packs with < 1.5 S/A: **10.63** (target: <= 2).

The average draft contains a streak of 10-11 consecutive bad packs. The
distribution is heavy-tailed: 26 drafts had a 25-pack streak (every post-
commitment pack was bad). This is effectively random -- the system provides
no meaningful concentration signal.

## S/A Density Trajectory

| Pick | Pool Size | Player S/A in Pool | Density |
|-----:|----------:|-------------------:|--------:|
| 5 | 90 | 20.5 | 22.8% |
| 8 | 72 | 17.0 | 23.6% |
| 9 (R2 start) | 114 | 28.5 | 25.0% |
| 16 (R2 end) | 72 | 12.0 | 16.7% |
| 17 (R3 start) | 102 | 19.0 | 18.6% |
| 23 (R3 end) | 66 | 11.0 | 16.7% |
| 24 (R4 start) | 81 | 14.5 | 17.9% |
| 30 (end) | 45 | 5.0 | 11.1% |

S/A density *declines* over the draft despite declining refill volumes. Each
refill adds fresh S/A cards (~36% rate), but the player and the AIs consuming
sibling-archetype S/A cards drains quality faster than refills replenish it.
The net trajectory is monotonically downward from 25% at pick 9 to 11% at pick
30. This is the opposite of what the system needs.

## Draft Traces

**Trace 1 (Committed, Self-Discard):** Player in open lane (AIs cover
Sacrifice/Flash/Self-Mill/Blink/Ramp). Despite correct open-lane commitment at
pick 5, packs average ~1 S/A card. Round 2 picks 9-16 show the strongest
stretch (2-3 S/A in some packs) but this is noise, not signal. The round-start
pool compositions show all archetypes within 1-3 cards of each other --
Storm/Warriors (open) at 19-21 vs Blink/Ramp (AI) at 12-13. The gradient is
barely 1.5x even in the best case and is erased by each refill.

**Trace 2 (Signal-reader, Storm):** Signal reader correctly identifies Storm
as open and commits. Despite aggressive archetype-focused picking, the median
pack offers 1 S/A card. The round-by-round pool shows the same pattern: open
lanes (Storm/Self-Discard/Warriors) accumulate modestly within each round, then
the balanced refill resets them.

## Comparison to V9 and V10

| Metric | V9 Hybrid B | V10 Hybrid X | SIM-4 | Delta vs V9 |
|--------|:-----------:|:------------:|:-----:|:-----------:|
| M3 | 2.70 | 0.84 | **0.83** | -1.87 |
| M5 | 9.6 | -- | 16.83 | +7.23 |
| M6 | 86% | -- | 58.8% | -27% |
| M10 | 3.8 | -- | 10.63 | +6.83 |
| M11 | 3.25 | -- | 0.71 | -2.54 |

SIM-4's M3 (0.83) is essentially identical to V10 Hybrid X (0.84) -- the
multi-round structure with balanced refills performs no better than V10's
single-pool approach. The declining refill volume provides zero measurable
benefit over V10. This is the strongest possible negative result: the refill
reset problem dominates the declining-volume mechanism completely.

## Self-Assessment: Does Declining Volume Alone Reach M3 >= 2.0?

**No. Definitively not.**

SIM-4's M3 of 0.83 is 59% below the 2.0 target and statistically
indistinguishable from V10's 0.84. Declining refill volumes (48/36/21/0) do
not overcome the fundamental problem: balanced refills reset the concentration
gradient at every round boundary.

The mechanism failure is mathematical, not parametric. Even with zero final
refill (Round 4 drafts from residual), the earlier balanced refills at rounds
1-2 erase any gradient the AIs build. The 1.25-1.30x within-round gradient
built by 5 AIs each depleting their lane is trivial against a 120-card pool
where each archetype has 15 cards.

**Key structural finding:** With a 120-card pool and balanced refills, the
player's committed archetype represents ~12.5% of the pool (15/120). A 5-card
pack drawn uniformly yields 0.625 expected cards of the player's archetype. At
the Graduated Realistic sibling rate (~36% weighted average), each such card has
a combined ~68% chance of being S/A (100% home + 36% sibling). Expected S/A per
pack: ~0.83. This matches the simulation result exactly -- the system is
performing at the theoretical random baseline.

Declining volume is necessary but not sufficient. Without refill bias (open-lane
multiplier, as in SIM-2/SIM-3), the refills neutralize any emergent
concentration from AI behavior. The critic's prediction (M3 1.3-1.5) was
optimistic; the actual result (0.83) shows that even the within-round gradient
is too small to produce measurable concentration in a 5-card pack drawn from a
120-card pool.

**Recommendation:** SIM-4 establishes the calibration floor for V11. Any design
that achieves M3 > 0.83 is doing so through its bias mechanism, not through
emergent AI-driven concentration. This makes SIM-4 invaluable as a null
hypothesis: it proves that open-lane bias (SIM-2, SIM-3) or structural
asymmetry (SIM-5) is required, not optional.
