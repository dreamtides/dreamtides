# Results 6: SIM-6 -- Small Pool Concentrated Bias (Design 2 Modified)

**Agent 6, Round 4 -- V11 Simulation**

---

## Full Scorecard (Graduated Realistic, Committed)

| Metric | Value | Target | Status |
|--------|:-----:|:------:|--------|
| M1 (early diversity) | 3.51 | >= 3.0 | PASS |
| M2 (early S/A) | 0.90 | <= 2.0 | PASS |
| M3 (post-commit S/A) | 0.73 | >= 2.0 | **FAIL** |
| M4 (off-archetype) | 4.05 | >= 0.5 | PASS |
| M5 (convergence) | 12.9 | 5-8 | **FAIL** |
| M6 (deck concentration) | 54% | 60-90% | **FAIL** |
| M7 (run variety) | 22% | < 40% | PASS |
| M8 max (arch frequency) | 13.7% | < 20% | PASS |
| M8 min (arch frequency) | 11.3% | > 5% | PASS |
| M9 (S/A stddev) | 0.81 | >= 0.8 | PASS |
| M10 (consec bad) | 10.3 | <= 2 | **FAIL** |
| M11' (late S/A, picks 20+) | 0.15 | >= 2.5 | **FAIL** |
| M12 (signal advantage) | 0.11 | >= 0.3 | **FAIL** |

**Overall: 7/13 pass. Six critical failures.**

### Strategy Comparison

| Strategy | M3 | M5 | M6 | M10 | M11' |
|----------|:--:|:--:|:--:|:---:|:----:|
| Committed | 0.73 | 12.9 | 54% | 10.3 | 0.15 |
| Power-chaser | 1.32 | 8.1 | 33% | 5.2 | 0.92 |
| Signal-reader | 0.84 | 10.6 | 52% | 9.5 | 0.19 |

### Pessimistic Fitness

M3 = 0.62, M10 = 11.5, M11' = 0.12, M6 = 48%. Uniformly worse.

---

## Per-Archetype M3 Table

| Archetype | M3 | M5 | M6 | M9 | M10 | M11' |
|-----------|:--:|:--:|:--:|:--:|:---:|:----:|
| Flash | 0.67 | 13.1 | 51% | 0.80 | 11.0 | 0.08 |
| Blink | 0.78 | 10.8 | 56% | 0.82 | 9.7 | 0.19 |
| Storm | 0.71 | 13.7 | 53% | 0.81 | 10.5 | 0.20 |
| Self-Discard | 0.74 | 12.1 | 53% | 0.82 | 9.9 | 0.16 |
| Self-Mill | 0.78 | 11.6 | 55% | 0.85 | 9.6 | 0.18 |
| Sacrifice | 0.78 | 9.8 | 56% | 0.84 | 10.0 | 0.16 |
| Warriors | 0.79 | 10.0 | 56% | 0.85 | 9.8 | 0.15 |
| Ramp | 0.63 | 17.4 | 49% | 0.76 | 11.3 | 0.11 |

Spread: 0.16 (Warriors 0.79 best, Ramp 0.63 worst). High-sibling-rate pairs
(Warriors/Sacrifice at 50%) perform slightly better; low-rate pairs (Flash/Ramp
at 25%) are worst. No archetype reaches M3 >= 2.0.

---

## Round-by-Round Pool Composition

| Round Start | Avg Pool | Avg S/A | S/A Density |
|:-----------:|:--------:|:-------:|:-----------:|
| 1 | 66.0 | 9.9 | 15.0% |
| 2 | 58.0 | 12.8 | 22.0% |
| 3 | 46.0 | 6.1 | 13.3% |
| 4 | 30.0 | 1.9 | 6.4% |
| 5 | 0.0 | 0.0 | 0.0% |

The refill after Round 1 is the only moment where S/A density improves (15% to
22%), because the open-lane bias injects fresh S/A cards while AI lanes have
already depleted theirs. By Round 3 start, density collapses to 13.3% as the
declining refill volume cannot keep pace with 36-card-per-round depletion. Round
4 starts with only 30 cards at 6.4% S/A. Round 5 receives zero cards -- the
pool is completely exhausted after Round 4.

---

## S/A Density Trajectory

| Pick | Avg Pool | Avg S/A | S/A Density |
|:----:|:--------:|:-------:|:-----------:|
| 1 | 66.0 | 9.9 | 15.0% |
| 6 | 36.0 | 6.7 | 18.7% |
| 7 (R2) | 58.0 | 12.8 | 22.0% |
| 10 | 40.0 | 9.7 | 24.3% |
| 13 (R3) | 46.0 | 6.1 | 13.3% |
| 16 | 28.0 | 3.6 | 12.8% |
| 19 (R4) | 30.0 | 1.9 | 6.4% |
| 22 | 12.0 | 0.4 | 3.1% |
| 25+ | 0.0 | 0.0 | N/A |

S/A density peaks at pick 10 (24.3%) during the open-lane-biased Round 2 refill
window, then declines monotonically to zero. The "concentration ramp" that
Design 2 predicted does occur briefly in Round 2 but is overwhelmed by pool
exhaustion from Round 3 onward.

---

## Pack Quality Distribution

**Picks 6+:** P10=0, P25=0, P50=1, P75=1, P90=2

**Picks 20+:** P10=0, P25=0, P50=0, P75=0, P90=1

The median pack contains 1 S/A card post-commitment, dropping to 0 for picks
20+. The 90th percentile pack contains just 1 S/A in the late draft.

---

## Consecutive Bad Pack Analysis

| Streak Length | Count | Pct |
|:---:|:---:|:---:|
| 2 | 1 | 0.1% |
| 3 | 11 | 1.1% |
| 4 | 9 | 0.9% |
| 5 | 54 | 5.4% |
| 6 | 68 | 6.8% |
| 7 | 107 | 10.7% |
| 8 | 110 | 11.0% |
| 9-12 | 363 | 36.3% |
| 13+ | 277 | 27.7% |

98.8% of drafts have 3+ consecutive bad packs. The median worst streak is 9
packs. 27.7% of drafts experience 13+ consecutive bad packs. This is
catastrophic -- the player faces long stretches of zero-S/A packs through the
entire late draft.

### Rounds 4-5 Specifically

- Avg pool size: 12.0 cards
- Avg S/A per pack: 0.19
- Zero-S/A packs: 82.6%
- Worst consecutive bad streak: 5 (within rounds 4-5 alone)
- Players typically reach only 23 picks before pool exhaustion

---

## Draft Traces

### Trace 1: Warriors (Committed)

Round 1 (pool=66, SA=12): Strong start -- picks 3 S/A cards from Warriors and
Sacrifice. Pool depletes to 30 after AI removal. Refill adds 25 cards.

Round 2 (pool=58, SA=18): Best round. SA density peaks at 31%. Picks 4 more
S/A. Concentration from open-lane bias is visible -- Warriors/Sacrifice cards
are plentiful.

Round 3 (pool=46, SA=8): Quality drops sharply. 2 S/A picks, 4 C/F. Pool thins
to 10 cards by round end. Refill adds only 14.

Round 4 (pool=30, SA=2): Disaster. Zero S/A in every pack. Player picks pure
filler for 5 picks before pool exhausts at pick 23.

Round 5: Never reached -- no cards remain.

Final: 12/23 S/A (52%). All S/A cards came from Rounds 1-3.

### Trace 2: Sacrifice (Signal Reader)

Similar trajectory. Strong Rounds 1-2 (12 S/A picks), collapsing Round 3 (2
S/A), completely dead Round 4 (0 S/A). Final: 12/23 S/A (52%). Pool exhaustion
at pick 23.

---

## Comparison to V9 and V10

| Algorithm | M3 | M10 | M11' | M6 |
|-----------|:--:|:---:|:----:|:--:|
| V9 Hybrid B | 2.70 | 3.8 | 3.25 | 86% |
| V10 Hybrid X | 0.84 | -- | 0.69 | -- |
| **SIM-6** | **0.73** | **10.3** | **0.15** | **54%** |

SIM-6 is **worse than V10**. M3 = 0.73 is 73% below the V9 baseline and 13%
below V10's already-catastrophic result. M10 = 10.3 is among the worst ever
recorded. M11' = 0.15 is effectively zero -- the late draft has no S/A cards
at all.

---

## Self-Assessment: Can the 5-Round Fast-Cycle Structure Compete?

**No. SIM-6 is the worst-performing algorithm in V11 and performs below V10.**

The failure is structural and was predicted by Design 2's own analysis. Three
compounding mechanisms destroy the algorithm:

1. **Pool exhaustion dominates.** 36 cards removed per round from a 66-card
   pool means the pool is 55% depleted after each round. The declining refills
   (25/20/14/0) cannot recover fast enough. By Round 4, only 30 cards remain.
   By mid-Round 4, the pool is empty. Round 5 never happens. Players average
   only 23 picks instead of 30.

2. **The refill reset compounds 4 times.** Each refill partially restores the
   concentration gradient, but 4 reset events (Rounds 1-4 boundaries) erase
   more total concentration than 2 events would. The open-lane bias (70%) is
   not strong enough to compound -- each cold-lane refill adds cards but the
   AIs continue draining the pool of everything else, and the pool is too small
   for the bias to create meaningful selection advantage.

3. **S/A preferential depletion is extreme in small pools.** With only ~10 S/A
   cards in the starting pool, AIs drain S/A from their lanes in 2-3 picks.
   Refills add some S/A back, but the declining volume means fewer S/A cards
   enter per round. By Round 4, S/A density is 6.4% and dropping to zero.

**The Design 2 post-critique revision was correct:** the 5-round cadence offers
no unique upside that justifies its structural penalties. The open-lane bias
mechanism is worth preserving in a 3-round structure; the fast-cycle frame
destroys it.

**Recommended disposition: Eliminate. No parameter tuning can rescue this
structure. The pool math (66 cards - 36 per round = guaranteed exhaustion) is
fatal. A 3-round design with the same open-lane bias should be strictly
superior on every metric.**
