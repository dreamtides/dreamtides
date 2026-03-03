# V8 Algorithm Overview: Complete Catalog

This document catalogs every algorithm simulated in V8, ordered by
recommendation strength. V8's defining contribution over V7 is treating pool
composition as a design variable, introducing per-pair fitness calibration, and
elevating player experience to a first-class constraint.

______________________________________________________________________

## Gold: Narrative Gravity (Pool Contraction)

**Agent 9 champion. Recommended algorithm.**

**One sentence:** "After each pick, permanently remove the least relevant cards
from the pool based on the player's accumulated resonance signature, so future
packs draw from an increasingly concentrated pool."

**Mechanism:** Maintain a 4-element resonance signature vector updated by
drafted card symbols (+2 primary, +1 secondary). From pick 4, compute
dot-product relevance for every pool card; remove the bottom 12% per pick.
Generics protected at 0.5 baseline. One slot from pick 3 onward draws from the
top-quartile relevance subset (floor).

### Full Metrics (40% Enriched Compensated Pool)

| Fitness             |  M3  |  M5  | M6  |  M9  | M10 |  Worst Arch  |
| ------------------- | :--: | :--: | :-: | :--: | :-: | :----------: |
| Optimistic          | 3.39 | 7.5  | 92% | 1.01 | 1.9 | 2.92 (Ramp)  |
| Graduated Realistic | 2.75 | 10.2 | 85% | 1.21 | 3.3 | 2.40 (Flash) |
| Pessimistic         | 2.59 | 11.0 | 83% | 1.25 | 3.9 | 2.13 (Flash) |
| Hostile             | 2.49 | 11.7 | 80% | 1.31 | 4.3 | 2.09 (Blink) |

Passes: M1, M2, M3, M4, M6, M7, M9 (7/10). Fails: M5 (10.2 vs 5-8), M10 (3.3 vs
2). M6 marginal at 85%.

### Set Design Specification

**Pool Breakdown (Compensated):**

| Archetype                    |  Total  | Home-Only |     Dual-Res     |
| ---------------------------- | :-----: | :-------: | :--------------: |
| Flash / Blink / Storm / Ramp | 40 each |    22     |        18        |
| Self-Discard / Self-Mill     | 40 each |    24     |        16        |
| Warriors / Sacrifice         | 40 each |    26     |        14        |
| Generic                      |   40    |    --     |        --        |
| **Total**                    | **360** |  **188**  | **132 + 40 gen** |

**Symbol Distribution:** 40 generic (11%), 188 single-res (52%), 132 dual-res
(37%).

**Dual-Res per Pair:** Flash/Ramp 18, Blink/Storm 18, Self-Disc./Self-Mill 16,
Warriors/Sacrifice 14. Low-overlap pairs get more dual-res to compensate.

**Per-Resonance R1 Pool:** 80 cards each. Pair subpool: 14-18 per archetype.

**Cross-Archetype A-Tier Targets:** Warriors/Sacrifice 50% (7/14),
Self-Disc./Self-Mill 40% (6/16), Blink/Storm 30% (5/18), Flash/Ramp 25% (4/18).
Plus 4-5 intentional bridge cards per low-overlap pair.

**Worked Example (Warriors, Tide/Zephyr):** 40 cards total. 26 home-only:
single-symbol (Tide), mechanically specific to Warriors (tribal synergies,
Kindle effects, Warrior subtype). 14 dual-res: carry (Tide, Zephyr), 7 designed
as bridge cards A-tier in both Warriors and Sacrifice (characters with
Materialized/Dissolved triggers), 7 mechanically narrow to Warriors. Pool
contraction keeps Warriors' 40 cards longest; by pick 20, the surviving pool is
predominantly Warriors + some Sacrifice + generics.

**Card Designer Guidance:** Create 132 dual-res cards (up from 54). The
secondary symbol is a filtering tag. A Warriors (Tide, Zephyr) card need not be
mechanically playable in Ramp. For each low-overlap pair, design 4-5 bridge
cards with universal effects (draw, removal, efficient bodies) carrying the
correct pair symbols. Total new work: 78 additional symbol assignments plus ~18
bridge cards.

### Why Gold

Narrative Gravity is the only algorithm where all 8 archetypes exceed M3 >= 2.0
under Graduated Realistic, Pessimistic, AND Hostile fitness without catastrophic
failures in M6 or M9. Its monotonic quality ramp is the most satisfying player
experience tested. The mechanism is genuinely novel: instead of placing good
cards into packs (slot-filling), it removes bad cards from the pool, so every
slot -- including "random" ones -- draws from increasingly concentrated pools.
This explains its superior M3: at 2.75 it exceeds all slot-filling algorithms
except the disqualified CSCT (2.92).

______________________________________________________________________

## Silver: CSCT (Commitment-Scaled Continuous Targeting)

**Agent 7 champion. Best raw M3, best M10, but disqualified by M6 = 99%.**

**One sentence:** "The number of pair-matched pack slots scales continuously
with the player's commitment ratio, with a minimum floor from pick 3."

### Full Metrics (40% Enriched Pool)

| Fitness             |  M3  | M5  | M6  |  M9  | M10 | Worst Arch |
| ------------------- | :--: | :-: | :-: | :--: | :-: | :--------: |
| Optimistic          | 3.07 | 5.0 | 99% | 0.68 |  2  |    2.88    |
| Graduated Realistic | 2.92 | 5.0 | 99% | 0.68 |  2  |    2.88    |
| Pessimistic         | 2.88 | 5.0 | 99% | 0.67 |  2  |    2.85    |
| Hostile             | 2.85 | 5.0 | 99% | 0.67 |  2  |    2.80    |

Passes: M1, M3, M5, M7, M10 (5/10). Fails: M2 (2.61), M4 (0.47), M6 (99%), M9
(0.68).

### Set Design Specification

Same 40% Enriched Pool as Narrative Gravity. 360 cards: 176 home-only + 144
dual-res + 40 generic. Each archetype: 22 home-only + 18 dual-res. Per-resonance
R1: 80 cards. Pair subpool: ~18 per archetype. Cross-archetype targets: same
Graduated Realistic rates.

**Card Designer Guidance:** Identical to Narrative Gravity. The dual-res symbol
is a filtering tag. 128-144 dual-res cards required.

### Why Silver (Not Gold)

CSCT is the most fitness-immune algorithm tested (only 0.22 M3 degradation from
Optimistic to Hostile) and the only algorithm to pass M10 \<= 2. Its
per-archetype equity is extraordinary (0.08 spread). However, M6 = 99% means the
committed player ends up with a mono-archetype deck. This is "on rails" -- the
algorithm is so effective at finding on-archetype cards that the player stops
making meaningful choices after pick 5. Every pack looks identical (p10=2,
p25=3, p50=3, p75=3). M9 = 0.68 confirms sterility.

Detuning CSCT (capping at 2 pair-matched slots, forcing 1 random slot) projects
M3 ~2.3-2.4 with M6 ~75-80% and M9 ~0.82. But this sacrifices M10 (estimated
~3-4), removing CSCT's unique advantage. A detuned CSCT resembles a less
effective Narrative Gravity. CSCT is Silver because its raw M3 and smoothness
demonstrate the theoretical ceiling, but it cannot be deployed as designed.

**When to choose CSCT:** If playtesting reveals that M6 = 99% concentration and
M9 < 0.8 variance are acceptable in a roguelike context -- if players welcome
"autopilot convergence" as supportive rather than constraining. This is possible
but contradicts V8's design goals.

______________________________________________________________________

## Bronze: Symbol-Weighted Graduated Escalation

**Agent 5 champion. Best M3 on the symbol-rich pool.**

**One sentence:** "Track weighted resonance counters from 3-symbol cards;
pair-matched pack slots unlock progressively at thresholds 3/6/9."

### Full Metrics (Symbol-Rich Pool, 84.5% Dual-Res)

| Fitness             |  M3  |  M5  | M6  |  M9  | M10 |  Worst Arch  |
| ------------------- | :--: | :--: | :-: | :--: | :-: | :----------: |
| Optimistic          | 2.88 | 3.4  | 91% | 1.02 | 2.3 |      --      |
| Graduated Realistic | 2.50 | 9.2  | 83% | 1.18 | 4.3 | 1.88 (Blink) |
| Pessimistic         | 2.49 | 11.7 | 78% | 1.19 | 5.8 |      --      |
| Hostile             | 2.34 | 11.8 | 68% | 1.32 | 9.1 |      --      |

Passes: M1, M2, M3, M4, M6, M7, M9 (7/10 on symbol-rich). Fails: M5 (9.2), M10
(4.3).

On 40% Enriched Pool: M3 = 2.29 (Graduated Realistic), 5 passes.

### Set Design Specification (Symbol-Rich Pool)

Every non-generic card carries exactly 3 ordered symbols with repetition. 360
cards: 176 AAB (49%), 64 ABB (18%), 64 ABC (18%), 16 AAA (4.5%), 40 generic
(11%). Per-archetype: 40 cards, 22 home-only, 18 cross-archetype. Pair subpool
per archetype: ~40 cards.

**Card Designer Guidance:** Assign 3 ordered symbols to every non-generic card.
Use AAB pattern (55%) for core archetype cards: (Tide, Tide, Zephyr) = Warriors.
ABB (20%) for secondary-leaning cards. ABC (20%) for splash. AAA (5%) for
pure-resonance utility. The third symbol is the hardest assignment -- it must be
a meaningful resonance for cards that naturally belong to only two.

### Why Bronze

Symbol-Weighted achieves the best fitness insulation on its dedicated pool (M3
drops only 0.01 from Graduated to Pessimistic). Its graduated ramp eliminates
surge/floor bimodality. 7/8 archetypes pass M3 >= 2.0 under Graduated Realistic.
However, it requires the symbol-rich pool (3 symbols per card, 84.5% dual-res),
which is a substantially more demanding card design constraint than the 40%
enriched pool. Blink at 1.88 remains the one archetype that falls short. It
earns Bronze as the best option for designers willing to commit to a fully
symbol-driven identity system.

______________________________________________________________________

## Honorable Mentions

### Continuous Surge (Agent 2)

**M3 = 2.48 (Graduated Realistic, 40% Enriched).** Replaces Surge+Floor's binary
surge/floor modes with per-slot pair-matching probability, creating a unimodal
quality distribution. Strong aggregate M3 and works even on V7 standard pool (M3
= 2.49). However, per-archetype disparity is severe: Warriors 2.60 vs Ramp 1.55
(68% gap). Three archetypes fall below 2.0 under Graduated Realistic. M10 fails
(3.8 avg, 25 worst). The algorithm concept is sound but the per-archetype
inequity disqualifies it.

**What would make it viable:** A compensation mechanism that boosts targeting
for low-overlap archetypes. Alternatively, the compensated pool (more dual-res
for Flash/Ramp) might narrow the gap enough.

### GPE-45 Graduated Pair-Escalation (Agent 4)

**M3 = 2.25 (Graduated Realistic, 40% Enriched).** Pair-matching probability
ramps smoothly across a pick 11-15 transition, with R1-resonance fallback on
non-pair slots. The R1 fallback is its most valuable innovation, adding 0.75 M3
over no-fallback variants. Clears M3 >= 2.0 at all four fitness levels. However,
the 10-pick bootstrapping dead zone (M5 = 12.5, M10 = 8.2) creates a terrible
early experience. Worst archetype Flash at 1.92 fails. M9 = 0.51 (too consistent
once ramped).

**What would make it viable:** A mechanism to provide value during the
bootstrapping phase. The R1-fallback component should be adopted by any
pair-matching algorithm.

### V5 Pair-Escalation Baseline (Agent 1)

**M3 = 2.16 (Graduated Realistic, 40% Enriched).** V5's algorithm retested under
V8 conditions. Only 11% degradation from Optimistic to Hostile -- the most
fitness-robust slot-filling algorithm. All archetypes above 2.0. However, M10
fails (worst streak = 8), M6 = 89% (high end), and the pack quality distribution
is wide (p25 = 1.0). A reliable workhorse that lacks the experiential qualities
of Narrative Gravity.

### Surge+Floor+Bias (Agent 1, R1 variant)

**M3 = 2.27 (Graduated Realistic, 40% Enriched).** V7's untested hybrid finally
simulated. Adds R1 bias to random slots on floor packs. Highest M3 among
Surge-family algorithms. Passes M3 at all fitness levels. However, M10 fails
(worst streak = 6) and per-archetype gap is 22% (Warriors 2.52 vs Ramp 2.07).
The Surge rhythm problem (bimodal distribution) persists.

______________________________________________________________________

## Eliminated

### Discrete Pair-Counter Mechanisms (Agents 3, 6, 8)

**Failure mode: Pair alignment catastrophe.**

- **Escalating Pair Lock (Agent 3):** M3 = 1.50. 55% pair alignment means 45% of
  drafts are functionally broken. The algorithm locks onto whichever pair
  accumulates counters first, which is only the player's intended pair slightly
  more than half the time.
- **GF+PE (Agent 6):** M3 = 1.72. The "guaranteed floor" guarantees pair-matched
  draws but not quality. Under realistic fitness, pair-matched sibling cards
  frequently fail the A-tier check. Worst archetype Ramp at 1.13 --
  catastrophically broken.
- **Compensated Pair Allocation (Agent 8):** M3 = 1.45. Same alignment failure
  as Agent 3. Signal-reader trace at 0.40 M3 demonstrates complete system
  breakdown. Pool compensation insight (more dual-res for low-overlap pairs) is
  valuable and was adopted into the recommended pool.

**Structural lesson:** Discrete pair counters tracking 8 competing pairs with
noisy early signals cannot reliably identify the player's intended archetype.
Continuous methods (commitment ratio, resonance signature, pool contraction) are
structurally more robust because they degrade gracefully rather than failing
catastrophically.

### SF+Bias Pair-Filtered (Agent 1)

**Failure mode: Over-concentration.** M3 = 3.25 but M6 = 95%, M9 = 0.42, M4 =
0.75. Pair-filtering every slot eliminates all splash and variety. Same problem
as CSCT but more extreme.

### V7 Surge+Floor on V7 Pool (Agent 1, V7 Baseline)

**Not eliminated conceptually but superseded.** M3 = 2.12 (Graduated Realistic,
V7 pool), but M10 fails (worst streak = 8) and per-archetype gap is 21%.
Narrative Gravity on the same V7 pool achieves M3 = 2.38 with better M9 and
monotonic delivery. Surge+Floor remains viable for Tier 1 (minimal change) but
is no longer recommended.

### V3 Lane Locking (Agent 1, retested)

**Failure mode: Variance collapse.** Hard R1 variant: M3 = 1.65, M9 = 0.81, M10
= 11. Soft pair variant: M3 = 1.96, M9 = 0.71 (fails). Lane Locking's
deterministic slots produce identical pack structures, killing M9 variance. The
soft-pair variant improves M3 but cannot reach 0.8 variance without sacrificing
convergence. Permanently locked slots remain structurally inferior to
non-permanent mechanisms.

### V7 Mechanism Classes (Previously Eliminated)

The following were eliminated in V7 and not re-tested in V8, as their structural
limitations remain:

- **Aspiration Packs:** 2-slot ceiling, gate paradox. Dead.
- **Compass Packs:** Neighbor rotation delivers 1.2% S/A. Dead.
- **Cost-based filtering:** +0.05 M3 -- noise. Dead.
- **R2 slot filling:** 3-17% S/A precision. Dead.

______________________________________________________________________

## Structural Findings

### 1. Pool Composition Dominates Algorithm Choice

The same algorithm class (graduated pair-escalation) yields M3 = 1.99 on V7
standard, 2.29 on 40% enriched, and 2.50 on symbol-rich. The pool improvement
(0.51 M3) exceeds what any algorithm change achieves within a fixed pool. The
card designer's pool decisions are the primary lever; the algorithm is
secondary.

### 2. Pair-Matching Is the Key to Fitness Robustness

Pair-matched algorithms degrade only 11-27% from Optimistic to Hostile.
R1-filtered algorithms degrade 46%. This is because pair-matching concentrates
draws on home-archetype cards (~80%), making sibling fitness nearly irrelevant.
V7's finding that "R2 is worthless" applied to R2 slots, not R2 as a filter.
Using R2 as a pair-filter on R1-primary cards is the single most important
targeting improvement.

### 3. Continuous Methods Beat Discrete Counters

Three of nine agents independently discovered that discrete pair-counter
mechanisms fail due to alignment catastrophe. The 55% alignment rate for
pair-locking means half of all drafts are broken. Continuous methods (commitment
ratio, resonance signature, pool contraction) degrade gracefully and track the
player's intent more reliably.

### 4. The M3-M10-M6 Triangle Has No Perfect Solution

No algorithm simultaneously maximizes M3, M10, and M6. High M3 + good M10 (CSCT)
requires M6 = 99%. High M3 + good M6 (Narrative Gravity) produces M10 = 3.3. The
recommended approach: prioritize M3 and M6 (achievable with Narrative Gravity)
and address M10 through a floor mechanism, accepting a marginal failure.

### 5. Pool Contraction Is a New Mechanism Class

V3-V7 only explored slot-filling (placing targeted cards into specific pack
slots). Narrative Gravity introduces pool contraction: removing bad cards from
the draw pool so all slots improve. This bypasses the per-slot precision ceiling
(75-85% for slot-filling) by raising the entire pool's quality. It is the
structural reason Narrative Gravity outperforms all slot-filling algorithms
except the over-concentrated CSCT.

### 6. Per-Pair Fitness Varies 2-3x Across the Archetype Circle

Uniform fitness models hide critical per-archetype failures. Warriors/Sacrifice
(50% natural overlap) gets 2-3x better draft quality than Flash/Ramp (10-20%
natural overlap) under R1-filtering algorithms. Pool compensation (more dual-res
cards for low-overlap pairs) narrows this gap. Per-archetype M3 reporting should
be mandatory for any future simulation.

### 7. Explainability Is Less Important Than Experience

V8 relaxed the one-sentence explainability constraint and found that the
best-performing algorithm (Narrative Gravity) is also the simplest to explain to
players. The mechanism is hidden; the experience is intuitive: "your packs get
better as you commit." Research Agent C's prediction was correct: transparency
of feedback matters more than transparency of mechanism.

### 8. The 40% Dual-Resonance Threshold Is the Critical Inflection Point

Below 40% dual-res, pair-matching algorithms lack sufficient subpool depth
(fewer than 14-18 cards per archetype pair) and cannot sustain 2+ pair-matched
slots per pack over a 25-pick post-commitment draft. Above 40%, returns
diminish. The jump from 15% to 40% is the single highest-value pool change,
adding 0.30-0.50 M3 across all algorithm classes.
