# V8 Simulation Results — Agent 1: Baselines

**Algorithms tested:** V7 Surge+Floor (T=3, S=3, floor_start=3), V5
Pair-Escalation (cap=0.50, K=6), V3 Lane Locking (hard R1-filter and soft
pair-filter variants), Surge+Floor+Bias (R1 and pair-filtered variants).

______________________________________________________________________

## Complete Set Design Specification: V7 Standard Pool (15% Dual-Res)

**Pool Breakdown:** 360 cards total. 40 cards per archetype (320) plus 40
generic. Each archetype has 34 home-only cards and 6 cross-archetype
(dual-resonance) cards. **Symbol Distribution:** 40 (11%) zero-symbol generic,
49 (14%) single-symbol, 194 (54%) two-symbol, 77 (21%) three-symbol.
**Dual-Resonance:** 48 cards (13.3%) carry two different resonance types, 6 per
archetype pair. **Per-Resonance:** 80 cards per resonance at primary position.
When filtering "primary=Tide," pool is 80 cards: 50% Warriors, 50% Sacrifice.
**Cross-Archetype Requirement:** Under Graduated Realistic: Warriors/Sacrifice
need 50% A-tier (20/40), SelfDiscard/SelfMill 40% (16/40), Blink/Storm 30%
(12/40), Flash/Ramp 25% (10/40). Total: 58 cross-archetype A-tier cards.

## Complete Set Design Specification: 40% Enriched Pool

**Pool Breakdown:** 360 cards. 40 per archetype (320) plus 40 generic. Each
archetype has 24 home-only and 16 dual-resonance cards. **Symbol Distribution:**
40 (11%) zero-symbol, 32 (9%) single-symbol, 194 (54%) two-symbol different-res,
94 (26%) three-symbol. **Dual-Resonance:** 128 cards (35.6%) carry two different
resonance types, 16 per archetype pair. **Per-Resonance:** 80 cards per
resonance at primary. Pair-matched subpool: 16 cards per ordered pair (e.g.,
Tide/Zephyr for Warriors). **Cross-Archetype Requirement:** Same fitness targets
as V7 pool, but 16 dual-res cards per pair provide more natural cross-archetype
overlap. **Card Designer Change from V7:** Create 80 additional dual-resonance
cards (48 to 128). Each archetype needs 16 cards carrying both primary and
secondary resonance. Example for Warriors (Tide/Zephyr): 16 cards with both
symbols, at least 8 A-tier in Ramp.

______________________________________________________________________

## Scorecard: Graduated Realistic Fitness (Primary Target)

| Algorithm             | Pool    |  M3  |  Worst-Arch  |  M4  | M5  | M6  |  M9  | M10 | Pass? |
| :-------------------- | :------ | :--: | :----------: | :--: | :-: | :-: | :--: | :-: | :---: |
| Surge+Floor (T=3)     | V7 15%  | 2.12 | 1.94 (Flash) | 1.88 | 4.3 | 91% | 0.88 |  8  | 5/10  |
| Surge+Floor (T=3)     | 40% Enr | 2.17 | 1.99 (Ramp)  | 1.83 | 4.3 | 92% | 0.86 |  7  | 5/10  |
| Pair-Escalation       | 40% Enr | 2.16 | 2.12 (Flash) | 1.84 | 5.8 | 89% | 1.00 |  8  | 6/10  |
| Lane Lock (hard R1)   | 40% Enr | 1.65 | 1.51 (Ramp)  | 2.35 | 5.3 | 88% | 0.81 | 11  | 3/10  |
| Lane Lock (soft pair) | 40% Enr | 1.96 | 1.92 (Flash) | 2.04 | 4.7 | 93% | 0.71 |  6  | 5/10  |
| SF+Bias (R1)          | V7 15%  | 2.24 | 2.06 (Ramp)  | 1.76 | 4.3 | 92% | 0.91 |  8  | 6/10  |
| SF+Bias (R1)          | 40% Enr | 2.27 | 2.07 (Ramp)  | 1.73 | 4.3 | 92% | 0.89 |  6  | 6/10  |
| SF+Bias (pair)        | 40% Enr | 3.25 | 3.23 (Ramp)  | 0.75 | 4.0 | 95% | 0.42 |  1  | 4/10  |

**Metric pass criteria:** M1>=3, M2\<=2, M3>=2.0, M4>=0.5, M5 in 5-8, M6 60-90%,
M7\<40%, M9>=0.8, M10\<=2. SF+Bias (pair) achieves the highest M3 (3.25) but
fails M4 (insufficient splash at 0.75), M6 (95% too concentrated), M9 (0.42, far
too uniform), and M5 (converges too early). Every Surge-family algorithm fails
M10 (worst consecutive bad packs of 6-8). Pair-Escalation and Soft Pair Lock
have the best M10 performance among algorithms that still pass M3.

## Scorecard: All Fitness Levels, 40% Enriched Pool

| Algorithm             | Optimistic | Grad. Real. | Pessimistic | Hostile | Degradation |
| :-------------------- | :--------: | :---------: | :---------: | :-----: | :---------: |
| Surge+Floor (T=3)     |    3.19    |    2.17     |    1.93     |  1.72   |    46.2%    |
| Pair-Escalation       |    2.34    |    2.16     |    2.12     |  2.08   |    11.0%    |
| Lane Lock (hard R1)   |    2.43    |    1.65     |    1.47     |  1.31   |    46.1%    |
| Lane Lock (soft pair) |    2.13    |    1.96     |    1.92     |  1.88   |    11.6%    |
| SF+Bias (R1)          |    3.33    |    2.27     |    2.03     |  1.81   |    45.7%    |
| SF+Bias (pair)        |    3.36    |    3.25     |    3.22     |  3.20   |    5.0%     |

## Fitness Degradation Curve

The critical finding: **pair-matching algorithms degrade only 11% from
Optimistic to Hostile, while R1-filtering algorithms degrade 46%.**
Pair-Escalation maintains M3 >= 2.08 even under Hostile fitness. Soft Pair Lock
holds at 1.88. R1-based algorithms (Surge+Floor, Lane Lock hard) lose nearly
half their performance. This confirms Research Agent A's prediction:
pair-matching is structurally immune to fitness variation because ~80% of
pair-matched draws are home-archetype cards (always S-tier), making the sibling
fitness rate nearly irrelevant.

SF+Bias (pair) shows only 5% degradation, but this is illusory: it is so
aggressive in pair-filtering that nearly every card drawn is home-archetype,
eliminating the fitness question entirely at the cost of all splash and variety.

## Pack Quality Distribution (Picks 6+, Grad. Realistic, 40% Enriched)

| Algorithm             | p10 | p25 | p50 | p75 | p90 |
| :-------------------- | :-: | :-: | :-: | :-: | :-: |
| Surge+Floor           | 1.0 | 2.0 | 2.0 | 3.0 | 3.0 |
| Pair-Escalation       | 1.0 | 1.0 | 2.0 | 3.0 | 3.0 |
| Lane Lock (soft pair) | 1.0 | 2.0 | 2.0 | 2.0 | 3.0 |
| SF+Bias (R1)          | 1.0 | 2.0 | 2.0 | 3.0 | 3.0 |
| SF+Bias (pair)        | 3.0 | 3.0 | 3.0 | 3.0 | 4.0 |

All algorithms except SF+Bias (pair) have p10 = 1.0, meaning 10% of
post-commitment packs contain only 1 S/A card. Pair-Escalation has the widest
spread (p25=1.0, p90=3.0), reflecting its high variance (M9=1.00). Lane Lock
(soft pair) has the tightest useful distribution (p25=2.0, p75=2.0, p90=3.0) but
at lower average. SF+Bias (pair) is artificially concentrated at 3.0.

## Consecutive Bad Pack Analysis (S/A < 1.5, Picks 6+, Grad. Realistic, 40% Enriched)

| Algorithm             | Avg Streak | Worst Streak | M10 Pass? |
| :-------------------- | :--------: | :----------: | :-------: |
| Surge+Floor           |    1.31    |      7       |   FAIL    |
| Pair-Escalation       |    1.44    |      8       |   FAIL    |
| Lane Lock (soft pair) |    1.28    |      6       |   FAIL    |
| SF+Bias (R1)          |    1.27    |      6       |   FAIL    |
| SF+Bias (pair)        |    1.00    |      1       |   PASS    |

**No algorithm except the over-concentrated SF+Bias (pair) passes M10 \<= 2.**
The worst-case streaks of 6-8 consecutive sub-1.5 packs represent genuine player
experience failures. Average streaks are modest (1.3-1.4), meaning most bad
streaks are just 1-2 packs, but the tail is long. The M10 target of \<= 2 is
extremely demanding; even with pair-filtering, the probabilistic nature of
slot-filling produces occasional long dry spells.

## Per-Archetype Convergence Table (Grad. Realistic, 40% Enriched)

| Archetype           | Sibling A-Tier | Surge+Floor | Pair-Esc. | Soft Pair Lock | SF+Bias (R1) |
| :------------------ | :------------: | :---------: | :-------: | :------------: | :----------: |
| Flash (Ze/Em)       |      25%       |    2.00     |   2.12    |      1.92      |     2.07     |
| Blink (Em/Ze)       |      30%       |    2.10     |   2.16    |      1.92      |     2.15     |
| Storm (Em/St)       |      30%       |    2.04     |   2.15    |      1.94      |     2.18     |
| SelfDiscard (St/Em) |      40%       |    2.19     |   2.19    |      1.95      |     2.25     |
| SelfMill (St/Ti)    |      40%       |    2.22     |   2.13    |      1.97      |     2.30     |
| Sacrifice (Ti/St)   |      50%       |    2.39     |   2.22    |      2.01      |     2.46     |
| Warriors (Ti/Ze)    |      50%       |    2.40     |   2.20    |      1.99      |     2.52     |
| Ramp (Ze/Ti)        |      25%       |    1.99     |   2.12    |      1.95      |     2.07     |

The worst-best archetype gap reveals each algorithm's fitness sensitivity.
**Surge+Floor has the widest gap: 2.40 (Warriors) vs 1.99 (Ramp), a 21%
spread.** Pair-Escalation compresses this to 10% (2.22 vs 2.12) because
pair-matching bypasses sibling fitness. Soft Pair Lock has the tightest gap (5%,
from 2.01 to 1.92) but at a lower overall level. SF+Bias (R1) has the widest
absolute gap (2.52 vs 2.07, 22%) because bias amplifies the fitness
differential.

## Parameter Sensitivity

**Surge threshold T:** T=2 and T=3 perform similarly (2.18 vs 2.16 M3), with T=2
slightly faster surges. T=4 and T=5 drop sharply (1.93, 1.76). **T=2-3 is the
optimal range.**

**Pair-Escalation cap:** Each +0.10 cap adds ~0.32 M3. At cap=0.50, M3=2.16 with
M6=89%. At cap=0.60, M3=2.48 but M6=91% (borderline). At cap=0.70, M3=2.80 but
M6=92% (fails 60-90% target). **Cap=0.50 is the sweet spot for balanced
metrics.**

**Soft Lock probability:** Linear relationship. sp=0.80 gives M3=1.96, M9=0.72.
sp=1.00 gives M3=2.30, M9=0.50 (fails M9). **sp=0.80 balances convergence and
variance.**

## Draft Traces

**Trace 1 (Warriors, Pair-Escalation, Enriched, Grad. Realistic):** Committed
pick 4. Post-commitment packs ranged 1-3 S/A. Final deck 28/30 S/A (93.3%). The
high-overlap Warriors pair provides consistently strong packs, with
pair-matching delivering 2+ S/A in most picks after 6. One stretch (picks 7, 10,
18\) delivered only 1 S/A -- acceptable single-pack dips.

**Trace 2 (Blink, SF+Bias pair, Enriched, Grad. Realistic):** Committed pick 4.
Every post-commitment pack delivered 3-4 S/A. Final deck 30/30 (100%). This
demonstrates SF+Bias (pair)'s over-convergence problem: the player never sees a
non-S/A card, eliminating meaningful choice and variety.

**Trace 3 (Flash, Soft Pair Lock, Enriched, Grad. Realistic):** Committed pick 4
(early; algorithm uses higher thresholds). Picks 1-3 showed 0 S/A -- the
worst-overlap pair struggles early. Post-lock (pick 5+), packs stabilize at 1-3
S/A. Final deck 27/30 (90.0%). The pair lock provides steady but not spectacular
delivery for the hardest archetype pair.

## Self-Assessment

**Pair-Escalation on 40% Enriched is the strongest baseline:** M3=2.16 under
Graduated Realistic with only 11% fitness degradation. It crosses 2.0 under all
fitness models tested. However, it fails M10 (worst streak = 8) and has a wide
pack quality distribution (p25=1.0). It needs a guaranteed floor mechanism to
eliminate dead pack streaks.

**Surge+Floor+Bias (R1) is the strongest V7-pool algorithm:** M3=2.24 on V7, but
fails M10 (streak=8) and has a 22% archetype gap.

**The binding constraint is M10.** No probabilistic algorithm reliably prevents
long streaks of sub-1.5 packs. A structural floor (guaranteeing at least 1
pair-matched slot in every pack from pick 3+) would address this at modest M3
cost. This is the key modification for other agents to test.
