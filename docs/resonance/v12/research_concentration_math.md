# Research: Concentration Math for Pool Contraction + Oversampling

## Setup and Key Formula

**Goal:** Determine which pool contraction trajectories achieve M3 >= 2.0 with
N = 8-12 oversampling.

**M3 definition:** Average S/A cards for the committed archetype per pack,
picks 6-30. Pack construction: draw N cards from pool, show best 4 by fitness
for inferred archetype. S/A cards rank highest (~0.9 fitness), so the shown 4
are drawn preferentially from the N-sample.

**Core formula (hypergeometric):** When drawing N cards from a pool of P cards
containing S S/A cards for the player's archetype:

```
E[S/A in drawn N] = N * (S / P)
E[S/A shown in pack] ≈ min(N * S/P, 4)
```

This approximation holds when N << P. When N approaches P (late-draft, small
pool), the hypergeometric is exact and the approximation slightly overstates.
For the "show best 4" step: since S/A cards rank highest, all drawn S/A cards
appear in the shown pack unless more than 4 are drawn. So M3 ≈ min(N * S/P, 4)
for uniform S/A ranking.

**Baseline parameters:**
- Pool: 120 cards, 8 archetypes, 15 per archetype
- S/A rate: 36% weighted average sibling A-tier → ~5.4 S/A per archetype (use 5)
- Player's archetype fraction: 15/120 = 12.5%
- S/A fraction: 5/120 = 4.2%

---

## Scenario 1: Baseline — No Contraction, No Avoidance, N = 4

Pool stays at ~120 throughout (with balanced refills). Player's archetype = 15
cards, S/A = 5. No oversampling (N = 4 = uniform random).

```
M3 = 4 * (5/120) = 0.167
```

This matches V11 SIM-1 committed result (M3 = 0.25 is slightly higher due to
AIs taking cards from other archetypes, leaving player's archetype slightly
enriched). The theoretical floor is **M3 ≈ 0.17**.

---

## Scenario 2: Avoidance Only — No Contraction, N = 4

Static 120-card pool. 5 AIs avoid the player's archetype. Only the player takes
their own S/A. Model: 6 pick cycles per pick number (player + 5 AIs). AIs take
cards from non-player archetypes. Pool composition changes as other archetypes
deplete while player's archetype accumulates as a larger fraction.

**Without refills (static pool):** After pick 30, player has taken ~30 cards
from pool. AIs have taken ~150 cards total from non-player archetypes. But with
5 non-player S/A archetypes of 5 cards each = 25 S/A for AIs, and AIs
preferentially taking S/A... The pool shrinks but player's archetype count only
depletes via player's own picks.

**Pick-by-pick archetype density (avoidance only, no refills, no contraction):**

| Pick | Pool Size | Player Arch Cards | Arch Density | S/A | M3 (N=4) |
|:----:|:---------:|:-----------------:|:------------:|:---:|:--------:|
| 1 | 120 | 15 | 12.5% | 5 | 0.17 |
| 6 | 90 | 12 | 13.3% | 4 | 0.18 |
| 11 | 60 | 10 | 16.7% | 3 | 0.20 |
| 16 | 40 | 8 | 20.0% | 3 | 0.30 |
| 21 | 25 | 6 | 24.0% | 2 | 0.32 |
| 26 | 14 | 4 | 28.6% | 2 | 0.57 |

Notes: Pool shrinks 6 per pick (5 AIs + player). Player arch depletes at 1 per
player pick. AIs avoid player's arch entirely. By pick 26, archetype density
reaches ~29% but pool is very small. With no refills and no oversampling,
avoidance alone without contraction plateaus at M3 ≈ 0.3-0.5 in the mid-to-late
draft. **Far below 2.0.** The pool contracts naturally (no refills) which helps,
but avoidance without declining refills produces organic contraction already —
this scenario collapses into the contraction case.

**Insight:** Avoidance alone, with uniform packs (N = 4), cannot approach M3 =
2.0 because even at 29% archetype density, E[S/A per pack] = 4 * 2/14 = 0.57.
The absolute S/A count (2-3 cards) is the binding limit, not just density.

---

## Scenario 3: Contraction Only — V11 SIM-4 Reference Case

V11 SIM-4: 4 rounds (8/8/7/7 picks), declining balanced refills (48/36/21/0),
Level 0 AIs (no avoidance), N = 4. Achieved **M3 = 0.83**.

**Pool trajectory (SIM-4, reconstructed from V11 data):**

| Round | Picks | Start Pool | Consumed | Refill | End Pool | Arch % | S/A |
|:-----:|:-----:|:----------:|:--------:|:------:|:--------:|:------:|:---:|
| R1 | 1-8 | 120 | 48 | 48 | 120 | 12.5% | 5 |
| R2 | 9-16 | 120 | 48 | 36 | 108 | ~13% | 5 |
| R3 | 17-23 | 108 | 42 | 21 | 87 | ~14% | 5 |
| R4 | 24-30 | 87 | 42 | 0 | 45 | ~16% | 4 |

Density barely increases because balanced refills restore all archetypes equally.
M3 = 0.83 is near the theoretical prediction: 4 * (5/87) * ~36% sibling = 0.83.

**Why SIM-4 fails:** Refills reset concentration. End pool at 45 cards, archetype
at 16% density, S/A at 4 cards → M3 = 4 * 4/45 = 0.36. The late-round M3 is
actually *lower* than early round (confirmed by V11: S/A density declines from
25% pick 9 to 11% pick 30 — the refills are adding S/A from other archetypes).

---

## Scenario 4: Avoidance + Contraction — N = 4

The key combination. 5 AIs avoid player's archetype (from pick 6), declining
refills (60/36/0 over 3 rounds). No oversampling.

**Pool model (30 picks, 3 rounds of 10, refills 60/36/0):**

Starting pool: 120 cards. Player arch: 15 cards, S/A: 5.
Per pick cycle: player takes 1 card (own arch), AIs take 5 cards (non-player
arch). Net: 6 removed per cycle, mostly from non-player archetypes.

**R1 (picks 1-10), refill 60 at pick 10:**
- Consumed: 60 cards (5 player-arch by player, 55 non-player-arch by AIs with
  avoidance starting pick 6; picks 1-5 AIs take randomly, so ~6 player-arch
  cards taken picks 1-5 estimated 1 from player arch, 29 from non-player).
- More precisely: picks 1-5 no avoidance → 5 AIs take randomly, ~0.6 player
  arch cards per cycle → ~3 player-arch taken. Picks 6-10 full avoidance →
  0 player-arch taken by AIs.
- Player takes 10 cards from own arch in R1.
- Net player arch remaining at end R1: 15 - 3 (AI random) - 10 (player) = 2
  ... but player picks 10 out of 30 in round 1 is too many. Let me re-model.

**Corrected model:** Player picks 1 card per pick cycle. Over 10 picks in R1:
player takes 10 cards total. But player's arch has 15 cards. After 10 player
picks: 5 remain in player arch (assuming all 10 player picks are from own arch
— realistic if avoidance means player always finds own-arch cards).

Wait — the player has only committed by pick 6. Picks 1-5 = exploration, may
take some off-arch cards. Assume ~7 of 10 picks in R1 are on-arch.

**Simplified model using realistic assumptions:**

- Player on-arch pick rate: 70% R1, 90% R2, 100% R3
- AI avoidance start: pick 6 (zero player-arch picks by AIs from pick 6+)
- Picks 1-5: AIs take ~0.5 player-arch per cycle → 2-3 player-arch taken
- Refills: balanced, 60/36/0

| Event | Pool | P-Arch Cards | P-Arch S/A | Arch % | M3 (N=4) |
|-------|:----:|:------------:|:----------:|:------:|:---------:|
| Start | 120 | 15 | 5 | 12.5% | 0.17 |
| Pick 5 end | 90 | 13 | 4 | 14.4% | 0.18 |
| R1 end (pre-refill) | 60 | 7 | 3 | 11.7% | 0.20 |
| R2 start (post-60 refill) | 120 | 14 | 4.5 | 11.7% | 0.15 |
| R2 end (pre-refill) | 60 | 7 | 3 | 11.7% | 0.20 |
| R3 start (post-36 refill) | 96 | 11.5 | 3.5 | 12.0% | 0.15 |
| R3 end (no refill) | 36 | 4 | 2 | 11.1% | 0.22 |

**Critical problem:** Balanced refills restore all archetypes equally. The
60-card refill adds ~7.5 cards to the player's arch. But the player has only
removed ~8 from their arch, so the refill nearly fully restores the player's
arch count. The density stays stuck at ~12% throughout. **M3 ≈ 0.20 with N=4.**

This matches the SIM-4 pattern: balanced refills erase the concentration gradient.

**To escape this trap, refills must be biased OR smaller than the rate of
non-player-arch depletion.** With 5 AIs depleting non-player archetypes fast
(5 cards per cycle from 7 other archetypes), and refills restoring all 8
archetypes equally, the player's arch fraction doesn't grow.

**The real mechanism:** The pool must contract *faster than refills restore it*.
With 60 cards consumed and 60 refilled, the pool doesn't contract at all.
The declining schedule (60/36/0) only helps in R3 (no refill). By R3 with
36-card starting pool and no refill, the player's arch is ~4 cards of 36 = 11%.

**Conclusion for N = 4:** Avoidance + balanced refills ≈ 0.20-0.25 M3. The
avoidance helps preserve the player's S/A count (AIs don't take them), but with
a pool that stays large, density doesn't build. **N = 4 cannot reach M3 = 2.0
regardless of avoidance, given balanced refills.**

---

## Scenario 5: The Critical Mechanism — Why Pool Size Matters

The fundamental equation for M3 at N = 4:

```
M3 = 4 * (S / P)
```

For M3 = 2.0: S/P must = 0.50. With S = 5, P must = 10.
For M3 = 1.0: S/P = 0.25. With S = 5, P must = 20.

**Pool contraction target table (N = 4, S = 5 S/A remaining):**

| Target M3 | Required S/P | Required Pool Size (S=5) |
|:---------:|:------------:|:------------------------:|
| 0.5 | 12.5% | 40 |
| 1.0 | 25.0% | 20 |
| 1.5 | 37.5% | 13 |
| 2.0 | 50.0% | 10 |
| 2.5 | 62.5% | 8 |

At N = 4, reaching M3 = 2.0 requires contracting to ~10 cards with 5 S/A
remaining. This is essentially impossible with a 30-pick draft and reasonable
pool dynamics.

---

## The Primary Analysis: M3 vs Pool Size at N = 8 and N = 12

With oversampling (draw N, show best 4 by fitness), the formula becomes:

```
E[S/A shown] = min(N * S/P, 4)
```

**Table: M3 at various pool sizes, S/A counts, and N values**

| Pool (P) | S/A (S) | S/P% | N=4 | N=8 | N=12 |
|:--------:|:-------:|:----:|:---:|:---:|:----:|
| 120 | 5 | 4.2% | 0.17 | 0.33 | 0.50 |
| 80 | 5 | 6.3% | 0.25 | 0.50 | 0.75 |
| 60 | 5 | 8.3% | 0.33 | 0.67 | 1.00 |
| 40 | 5 | 12.5% | 0.50 | 1.00 | 1.50 |
| 30 | 5 | 16.7% | 0.67 | 1.33 | 2.00 |
| 25 | 5 | 20.0% | 0.80 | 1.60 | 2.40 |
| 20 | 5 | 25.0% | 1.00 | 2.00 | 3.00 |
| 15 | 5 | 33.3% | 1.33 | 2.67 | 4.00 (cap 4) |
| 20 | 4 | 20.0% | 0.80 | 1.60 | 2.40 |
| 20 | 3 | 15.0% | 0.60 | 1.20 | 1.80 |
| 15 | 4 | 26.7% | 1.07 | 2.13 | 3.20 |
| 15 | 3 | 20.0% | 0.80 | 1.60 | 2.40 |

**Reading the table:**
- N = 8 reaches M3 = 2.0 at pool = 20 cards with 5 S/A remaining
- N = 12 reaches M3 = 2.0 at pool = 30 cards with 5 S/A remaining
- N = 12 reaches M3 = 2.0 at pool = 25 cards with 4 S/A remaining (4 * 4/25 * 3 = wait: 12 * 4/25 = 1.92, just below 2.0)

**The M3 = 2.0 boundary line:**
- N = 8: Pool must reach 20 cards (5 S/A) or equivalently 16 cards (4 S/A)
- N = 12: Pool must reach 30 cards (5 S/A) or 25 cards (4 S/A)

---

## Scenario 6: S/A Trajectory — Does the Player Run Out?

With avoidance, only the player depletes their own S/A. Starting S/A: 5.
Refills add S/A proportionally (5/15 = 33% of refilled arch cards are S/A).

**S/A trajectory with avoidance + declining refills (60/36/0):**

| Event | S/A Before Refill | Refill Adds | S/A After |
|-------|:-----------------:|:-----------:|:---------:|
| Start | 5 | — | 5 |
| After R1 (10 picks, ~7 on-arch) | 5 - 7*(5/15) ≈ 2.7 | 60*(5/120) = 2.5 | 5.2 |
| After R2 (10 picks, ~9 on-arch) | 5.2 - 9*(5.2/14) ≈ 1.5 | 36*(5/120) = 1.5 | 3.0 |
| After R3 (10 picks, ~10 on-arch) | 3.0 - 10*(3/11) ≈ 0.3 | no refill | 0.3 |

**S/A exhaustion is a real risk by pick 25-30.** With the player picking mostly
on-arch cards (which include both S/A and C/F), S/A depletes faster than total
arch count. The player's own picks consume S/A at the same rate as their arch
fraction (since they take best cards first).

**Critical insight:** Refills must add enough S/A to replenish what the player
consumes. With declining refills, the R3 refill adds only 1.5 S/A on average.
By late draft (picks 25-30), S/A count may be 1-3 cards — the binding constraint
for M3, not pool size.

**Adjusted M3 estimates accounting for S/A depletion:**

| Pick | Pool | S/A Remaining | N=4 | N=8 | N=12 |
|:----:|:----:|:-------------:|:---:|:---:|:----:|
| 6 | 90 | 4.5 | 0.20 | 0.40 | 0.60 |
| 11 | 65 | 4.0 | 0.25 | 0.49 | 0.74 |
| 16 | 45 | 3.5 | 0.31 | 0.62 | 0.93 |
| 21 | 30 | 3.0 | 0.40 | 0.80 | 1.20 |
| 26 | 20 | 2.0 | 0.40 | 0.80 | 1.20 |

These numbers are still below 2.0 even at N = 12, picks 21-30.

**The problem:** S/A depletion from player's own picks limits M3 even when
pool contraction is aggressive.

**Required S/A to reach M3 = 2.0:**
- At pool = 20, N = 8: need S/A = 20/(N/M3_target) = 20*2/8 = 5 S/A
- At pool = 20, N = 12: need S/A = 20*2/12 = 3.3 S/A

So at N = 12, 3-4 S/A remaining in a 20-card pool achieves M3 ≈ 2.0.
At N = 8, 5 S/A remaining in a 20-card pool is required.

**Maintaining S/A supply:** The refill schedule must be tuned so that by pick
20, at least 4-5 S/A remain in the player's arch. This means the refills must
add enough S/A to offset player consumption. With the player consuming ~0.33
S/A per pick on-arch, over 30 picks the player takes ~10 S/A total. Starting
with 5 and adding via refills: need to add ~5-8 S/A via refills to end with
3-5 S/A in the late pool.

---

## Scenario 7: Pool Size Sensitivity at N = 8 and N = 12

Assuming S/A count is maintained at ~5 (via appropriate refill design):

| Final Pool Size | N=8 M3 | N=12 M3 | Feasibility |
|:---------------:|:------:|:-------:|:-----------:|
| 15 | 2.67 | 4.00 (capped) | Aggressive but possible |
| 20 | 2.00 | 3.00 | Target for N=8 |
| 25 | 1.60 | 2.40 | Target for N=12 |
| 30 | 1.33 | 2.00 | Minimum target for N=12 |
| 40 | 1.00 | 1.50 | Insufficient for either |
| 50 | 0.80 | 1.20 | Far below target |

**Key thresholds:**
- **N = 8 requires pool <= 20 cards (with 5 S/A)** to reach M3 = 2.0
- **N = 12 requires pool <= 30 cards (with 5 S/A)** to reach M3 = 2.0
- Pool of 40 cards is insufficient for M3 = 2.0 at any N <= 12

---

## Scenario 8: Comparison to V9

V9 Hybrid B: pool contracts from 360 to ~17 cards by pick 30. At pick 30, pool
= 17 with 60%+ archetype density from blended relevance scoring. Pack = 3 random
+ 1 floor slot (top-quartile guaranteed).

V9 achieves M3 = 2.70 through:
- Pool contraction to 17 cards (extreme contraction)
- ~60% archetype density in surviving pool
- 1 guaranteed floor slot (equivalent to N ≈ 8 for the floor slot alone)
- Floor slot: 1 top-quartile card = guaranteed S/A if any S/A remain

**Structural equivalence analysis:**

| Parameter | V9 | V12 Target |
|-----------|:--:|:----------:|
| Starting pool | 360 | 120 |
| Final pool | ~17 | ~20-30 |
| Contraction ratio | 21:1 | 4-6:1 |
| Archetype density at end | 60%+ | 45-55% |
| Pack mechanism | 3 random + 1 floor | N=8-12, show best 4 |
| Transparency | Invisible removal | Physical AI drafting |

V9's contraction ratio (21:1) is far more extreme than what V12 can achieve
via physical AI drafting. V12's 120 → 20-30 contraction is only 4-6:1.
However, V12's avoidance mechanism preserves S/A specifically (AIs don't take
player's S/A), while V9's contraction targets cards by relevance scoring.

**Are they structurally equivalent?** Partially. Both achieve concentration by
reducing pool size. V9 achieves higher density through more aggressive
contraction. V12 requires oversampling (N = 8-12) as a partial substitute for
the density gap. The combination of V12's 5:1 contraction + N = 8-12 oversampling
can approach (but not exceed) V9's M3 = 2.70, targeting M3 = 2.0-2.4.

**V9's floor slot advantage:** The guaranteed top-quartile floor slot in V9 is
a strong mechanism equivalent to unlimited N for S/A. V12 can incorporate a
floor slot within the oversampling framework: "draw 12, guarantee 1 S/A in the
4 shown if any S/A were drawn." This is compatible with the face-up pool.

---

## Scenario 9: Exploration Phase (Picks 1-5)

Before commitment, the player needs diverse packs to evaluate archetypes. Options:

**Option A — N = 4 (uniform) during exploration:**
- Picks 1-5: random 4 from 120-card pool. Avg 0.5 archetypes with S/A in each
  pack (M1 metric, target >= 3). Uniform random from 120 cards yields
  ~1.5-2.0 archetypes per pack with any S/A → M1 ≈ 1.5-2.0. This underperforms
  the M1 >= 3 target.

**Option B — Pool browser serves exploration:**
- Picks 1-5: player browses the full 120-card face-up pool to identify all
  available archetypes, then receives a random pack of N = 4.
- Since the player can see all 120 cards, M1 is effectively satisfied by browsing
  (all 8 archetypes are visible). The pack is for execution, not discovery.
- This satisfies the exploration need without inflating N during exploration.

**Option C — N = 4 during exploration, N ramps post-commitment:**
- Picks 1-5: N = 4 uniform.
- Picks 6-10: N = 6 (light oversampling begins).
- Picks 11+: N = 8-12 (full oversampling).
- Advantage: pool starts larger during exploration, so uniform N = 4 still shows
  variety. The ramp aligns oversampling with pool contraction.

**Recommendation:** Option B (pool browser for exploration, N = 8-12 for
execution) is the natural V12 mechanism given the face-up pool. The pool browser
replaces the need for high N during exploration — the player can see everything.
N should be fixed at the target value throughout picks 6-30. Picks 1-5 can use
N = 4 (uniform) since the player is browsing, not relying on packs for discovery.

---

## Summary: Required Conditions for M3 >= 2.0

To achieve M3 >= 2.0 in V12, all three conditions must hold simultaneously:

1. **Pool contracts to 20-30 cards** by picks 20-30 (depending on N)
2. **S/A count remains at 4-5** in the player's arch through those picks
3. **N = 8 (pool <= 20)** or **N = 12 (pool <= 30)** oversampling is applied

Avoidance alone (no contraction) cannot achieve M3 = 2.0 — density stays at
12-30% max with a static or slowly contracting pool.

Contraction without avoidance (V11 SIM-4) produces M3 = 0.83 — balanced refills
erase the gradient.

**The winning combination:** Aggressive declining refills (net pool shrinkage
each round) + AI avoidance (S/A preservation) + N = 8-12 oversampling.

**Refill schedule to achieve pool <= 20-30 cards by pick 25:**
Starting at 120, removing 6 per pick cycle:
- Without refills: pool = 120 - 6*25 = -30 (exhausted by pick 20) — too aggressive
- Refill schedule needed: consume 180 total (6*30 picks), add ~100-110 via refills
  to reach final pool of 20-30
- Example: 120 start + 45 refill R1 + 30 refill R2 + 0 refill R3 = 195 total
  supply - 180 consumed = 15 remaining ← too thin
- Safer: 120 + 60 + 36 + 0 = 216 - 180 = 36 remaining ← pool ≈ 36 at pick 30
  → N = 12 achieves M3 ≈ 1.9 (close but may need slightly higher N or smaller pool)

**The math points to a constraint:** With 30 picks and 6 removals per pick,
the total removal is 180 cards. Starting at 120 and targeting a 20-card final
pool means supplying 80 additional cards via refills (120 + 80 - 180 = 20).
This is achievable with a refill schedule like 50/30/0 (80 total refills).
Or 60/20/0. The key is keeping total refills to ~80 cards.

**Concrete refill schedule recommendation for M3 >= 2.0 at N = 12:**
- Start: 120 cards
- Refill after R1 (10 picks): 50 cards → pool at R2 start: 60 + 50 = 110
- Refill after R2 (10 picks): 30 cards → pool at R3 start: 50 + 30 = 80
- No R3 refill, 10 picks to finish → final pool: ~20 cards
- With avoidance preserving player's arch: archetype fraction grows from 12.5%
  to ~25% (5 of 20 cards). S/A count preserved at ~4 by avoidance.
- M3 at picks 21-30: N=12, pool=20-40, S/A=4-5 → M3 = 1.6-2.4

This is the mathematical basis for V12 algorithm design.
