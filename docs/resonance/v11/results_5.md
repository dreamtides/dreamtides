# Results 5: Asymmetric Replacement (Design 6, Timing-Adjusted)

**SIM-5 — V11 Round 4**

## Full Scorecard

| Metric | Committed | Signal | Power | Target | Status |
|--------|:---------:|:------:|:-----:|:------:|--------|
| M1 | 3.80 | 3.84 | 3.84 | >= 3 | PASS |
| M2 | 0.62 | 0.72 | 0.68 | <= 2 | PASS |
| M3 | 0.57 | 0.67 | 0.58 | >= 2.0 | **FAIL** |
| M4 | 4.43 | 4.33 | 4.42 | >= 0.5 | PASS |
| M5 | 28.1 | 26.9 | 28.0 | 5-8 | **FAIL** |
| M6 | 46.1% | 46.0% | 15.1% | 60-90% | **FAIL** |
| M7 | 12.4% | 13.6% | 12.0% | < 40% | PASS |
| M8 | 11.3-13.5% | 4.4-22.7% | 11.2-14.3% | 5-20% | PASS |
| M9 | 0.68 | 0.73 | 0.68 | >= 0.8 | **FAIL** |
| M10 | 14.21 | 12.40 | 14.44 | <= 2 | **FAIL** |
| M11' | 0.56 | 0.62 | 0.60 | >= 2.5 | **FAIL** |
| M12 | — | 0.09 | — | >= 0.3 | **FAIL** |

**Score: 6/15 checks passed. Catastrophic failure on M3, M5, M6, M10, M11', M12.**

Pessimistic fitness: committed M3=0.49, signal M3=0.57 — even worse.

---

## Per-Archetype M3 Table

| Archetype | M3 (Committed) | M3 (Signal) |
|-----------|:--------------:|:-----------:|
| Flash | 0.50 | 0.57 |
| Blink | 0.51 | 0.60 |
| Storm | 0.53 | 0.59 |
| Self-Discard | 0.60 | 0.67 |
| Self-Mill | 0.61 | 0.67 |
| Sacrifice | 0.65 | 0.70 |
| Warriors | 0.66 | 0.72 |
| Ramp | 0.52 | 0.58 |

Range: 0.50-0.66 (committed). No archetype reaches even M3=1.0. The graduated
fitness model is visible: Warriors/Sacrifice (50% sibling rate) are highest;
Flash/Ramp (25%) are lowest. But the absolute values are uniformly catastrophic.

---

## Pool Composition Snapshots

| Moment | Archetype Avg Total | Archetype Avg S/A | Total Pool |
|--------|:-------------------:|:-----------------:|:----------:|
| Pick 1 | 15.0 | 9.8 | 120 |
| Pick 10 | 13.9 | 7.3 | 110 |
| Pick 16 (pre-refill) | 13.1 | 6.1 | 105 |
| Pick 16 (post-refill) | 15.6 | 7.7 | 125 |
| Pick 20 | 15.1 | 7.7 | 120 |
| Pick 30 | 13.5 | 7.6 | 109 |

**The critical finding:** All archetypes maintain roughly equal counts throughout
the draft. AI lanes do not deplete because replacement immediately restocks them.
Open lanes do not concentrate because the player removes only ~10 cards from
their lane over 30 picks (pool starts at 15 per archetype, ends at ~12-13 for
open lanes vs. ~14 for AI lanes). The asymmetry exists but is negligible: a
2-card count difference produces no meaningful pack-level signal.

---

## Pack Quality Distribution (picks 6+, committed)

| p10 | p25 | p50 | p75 | p90 |
|:---:|:---:|:---:|:---:|:---:|
| 0 | 0 | 0 | 1 | 2 |

The median pack contains zero S/A cards for the committed archetype. 75% of
packs have 0-1 S/A. This is indistinguishable from random draws.

---

## Consecutive Bad Pack Analysis

| Max Streak | % of Drafts |
|:----------:|:-----------:|
| <= 8 | 16.2% |
| 9-15 | 41.3% |
| 16-20 | 19.7% |
| 21-25 | 22.8% |

Nearly 43% of drafts have a 16+ pack streak without adequate S/A. The worst
case (25 consecutive bad packs) occurs in 9.8% of drafts — meaning 1 in 10
drafts has essentially zero S/A cards for the entire post-commitment period.

---

## S/A Density Trajectory

S/A density for committed archetype starts at 12.4% and declines steadily to
7.5% by pick 15. The pick 16 refill boosts it back to 15.0%, but it then
resumes declining to 7.1% by pick 30. At no point does density meaningfully
exceed the starting value. The asymmetric replacement mechanism does not create
a concentration gradient — it merely prevents AI-lane depletion while allowing
all-lane S/A to drain equally from random pack draws.

**Why the density never rises:** AI picks are highest-fitness (preferentially
S/A), and their replacements are random reserve cards with the same S/A
distribution as the starting pool (~65% S/A rate within-archetype). But
the pool is large enough (105-125 cards) that the pack's 5-card sample rarely
hits the committed archetype regardless of density. The fundamental constraint
is pool size relative to pack size, not replacement asymmetry.

---

## Draft Traces

### Trace 1: Committed (Blink)

Open lanes: Warriors, Blink, Storm. AI lanes: Self-Mill, Sacrifice, Ramp,
Flash, Self-Discard.

Picks 1-10: Only 3 S/A packs in 10 picks. Player takes what's available —
many off-archetype cards. Pool steady at 119-110. Pick 10 snapshot shows no
differentiation between AI and open lanes.

Picks 11-20: Sporadic S/A (picks 11, 13, 14, 15 hit, then 16-18 dry). Refill
at 16 boosts pool to 124 but player sees no immediate S/A benefit. Final
deck after 30 picks: 11/30 S/A = 37%.

### Trace 2: Signal Reader (Ramp)

Open lanes: Ramp, Self-Discard, Self-Mill. Player explores picks 1-4 (takes
high power), commits to Ramp at pick 5 based on S/A count in pool.

Picks 5-15: Gets 4 Ramp S/A cards across 10 picks. Many packs contain zero
Ramp cards entirely. Picks 16-20: Refill helps briefly (picks 16-18 each have
1-2 S/A). Picks 21-30: Zero S/A in 9 of 10 packs. Final: 11/30 S/A = 37%.

---

## Comparison to V9 and V10

| Algorithm | M3 | M10 | M11 | M6 |
|-----------|:--:|:---:|:---:|:--:|
| V9 Hybrid B | 2.70 | 3.8 | 3.25 | 86% |
| V10 Hybrid X | 0.84 | — | 0.69 | — |
| **SIM-5** | **0.57** | **14.2** | **0.56** | **46%** |

SIM-5 performs **worse than V10**. M3 = 0.57 is 32% below V10's 0.84 and 79%
below V9's 2.70. M10 = 14.2 is catastrophic (target <= 2). This is the worst
result in the V11 simulation field.

---

## Root Cause Analysis

The design predicted M3 = 2.2-2.5 based on the assumption that "player takes
all 5 original S/A with zero AI competition." This prediction was based on a
fundamental error: it assumed the player could target their open lane
efficiently from a 120-card pool through 5-card packs.

**The math:** With 120 cards in 8 archetypes at ~15 each, a random 5-card pack
contains an expected 0.94 cards from any specific archetype (15/120 x 5 = 0.63
home + 0.63 sibling x sibling_rate). With ~10 S/A cards per archetype, the
expected S/A for the committed archetype per pack is ~0.5 (10/120 x 5 x
fitness_adjustment). This matches M3 = 0.57 almost exactly. The prediction of
2.2-2.5 was off by 4x.

**Why asymmetric replacement fails as a concentration mechanism:** Concentration
requires that the player's target cards become a larger fraction of what they
see. Design 6 asymmetric replacement maintains pool size (~110-125) and maintains
archetype balance (~13-16 per archetype). The player takes 30 cards from 120 —
too few to create visible depletion of their lane, and the pool is too large for
the 5-card pack to reliably sample their target archetype.

V9 achieved concentration by shrinking the pool. A 360-card pool contracted to
~60 cards means the player's target archetype goes from 5% to 30%+ of the
visible pool. Design 6 maintains the pool at ~120 throughout — the target
archetype never exceeds 13% of the pool. The pack construction cannot overcome
this dilution.

**The structural lesson:** Asymmetric replacement preserves AI-lane supply but
does not create player-lane concentration. These are different mechanisms. The
design document conflated "AI lanes cycle while open lanes deplete" with "open
lane cards concentrate in the player's packs." The first is true; the second
does not follow because pack construction samples from the entire pool, not
just the open lanes.

---

## Self-Assessment: Does SIM-5 Reach "Advanced" Tier?

**No.** SIM-5 fails every core metric. M3 = 0.57 is below V10's already-failed
0.84. The asymmetric replacement mechanism produces no meaningful concentration
because the pool remains too large and too balanced for 5-card random packs to
capture the modest depletion differential. The scheduled refill at pick 16 works
as designed (restocks depleted archetypes) but cannot compensate for the
fundamental pool-size-to-pack-size ratio problem.

Design 6's prediction of M3 2.2-2.5 was based on reasoning about pool-level
archetype counts rather than pack-level sampling probabilities. The design
correctly identified that open lanes would deplete monotonically while AI lanes
cycle — but this 2-3 card count differential across 120 cards is invisible in
a 5-card pack.

For asymmetric replacement to work, it would need either: (a) a much smaller
pool (~40-60 cards) where player picks create proportionally larger depletion,
or (b) a pack construction mechanism that preferentially samples from the
player's target archetype (which would be V9 contraction by another name), or
(c) substantially more player picks relative to pool size.

**Tier: Eliminated. Not viable as presented.**
