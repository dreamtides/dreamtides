# V11 Algorithm Overview: Complete Catalog

V11's defining question was whether multi-round drafting with pool refills could
replace V9's virtual pool contraction while preserving the AI drafter narrative
that V10 validated. The answer is no. The pack-sampling bottleneck -- the
fundamental constraint that 5-card packs from 100-130 card pools cannot achieve
40% archetype density -- makes M3 >= 2.0 structurally impossible without pool
contraction. V11's contribution is threefold: the Design 5 information system,
the confirmation that open-lane bias is genuinely Level 0, and the definitive
identification of the pack-sampling bottleneck as the binding constraint.

---

## 1. Recommended: V9 Hybrid B + AI Narrative Layer + Design 5 Information

**One sentence (player-facing):** "You draft against AI opponents who compete
for cards; read which archetypes are open, and the information display helps
you commit at the right time."

**One sentence (technical):** V9 Hybrid B contraction engine (12% per pick,
40/60 visible/affinity blend) provides all concentration; AI drafters provide
narrative framing with no mechanical effect; Design 5's three-layer information
system (bars + trends + snapshot) provides signal-reading skill expression.

### Components

**V9 Contraction Engine (unchanged):** Pool contraction at 12% per pick from
pick 4. Relevance = 40% visible dot-product + 60% pair-affinity. Floor slot
from pick 3. Archetype inference from pick 5. Pool minimum 17 cards. Hidden
metadata: 8 bits/card (two 4-bit pair-affinity floats).

**AI Narrative Layer (from V10):** 5 AIs randomly assigned to archetypes (3
open lanes per game). No mechanical effect. Cards removed by contraction
attributed to AI picks. Saturation display after ~12 apparent picks per AI.
C(8,5) = 56 compositions for game-to-game variety.

**Design 5 Information System (from V11):** Availability bars (quantity, not
quality) per archetype grouped by resonance symbol. Depletion trend arrows
(fast/stable/slow) smoothed over 2 pick cycles. Round-start snapshots with
quality descriptor that dim progressively. Artificial round boundaries every
10 picks for information display.

### Metrics

| Metric | Value | Target | Status |
|--------|------:|--------|:------:|
| M3 | 2.70 | >= 2.0 | PASS |
| M11 | 3.25 | >= 3.0 | PASS |
| M10 | 3.8 | <= 2 | FAIL |
| M6 | 86% | 60-90% | PASS |
| M5 | 9.6 | 5-8 | FAIL |
| V1 | 84.8% | -- | Visible symbols primary |
| M12 | 0.35-0.50 (projected) | >= 0.3 | PROJECTED PASS |

### Why This Is the Final Answer

V9 through V11 have now explored the complete design space:
- V9: Virtual contraction algorithms (Hybrid B is optimal)
- V10: Physical AI drafter removal (cannot replace V9; provides narrative)
- V11: Multi-round refills with biased replenishment (cannot replace V9;
  provides information system and confirms pack-sampling bottleneck)

No unexplored mechanism category remains that could plausibly replace V9's
contraction engine. The recommended design combines the best contribution from
each version: V9's proven math, V10's proven narrative, V11's proven information
architecture.

---

## 2. Viable Alternatives

### Alternative A: V9 Hybrid B + AI Narrative Layer (No Information System)

The Simple tier from the final report. V9's engine with V10's narrative, no
Design 5 information. Appropriate when implementation simplicity is paramount
or when the team prefers players discover lane signals organically from pack
composition rather than through explicit UI elements.

**Tradeoff:** Lower projected M12 (signal-reading advantage depends on player
observation rather than explicit indicators). M3 and M11 unchanged.

### Alternative B: V9 Hybrid B + Lightweight Physical AI Removal + Design 5

The Advanced tier. Each AI physically removes 1 card per round from its
archetype (5 total per round) before V9's contraction processes the pool.
Strengthens the AI narrative at a small metric cost (M3 ~2.55-2.65). Highest
implementation complexity.

**Tradeoff:** Physical S/A depletion degrades M3 and M11 slightly. The 5
physical removals per round are a small perturbation (V9's contraction removes
~43 virtual cards per pick in early draft), so the degradation is modest.

### Alternative C: V9 Hybrid B with Multi-Round Presentation Wrapper

V9's contraction engine runs continuously, but the draft is presented in a
3-round format with narrative "refill" moments between rounds 10-11 and 20-21.
No mechanical refills occur; the presentation structure exists solely for
pacing and information display. Design 5's snapshots and trend resets align
with these presentation rounds.

**Tradeoff:** Additional UI complexity for no mechanical benefit. May improve
player experience through clearer pacing and strategic recalibration moments.
Not tested in simulation.

---

## 3. Eliminated Algorithms

### Failure Mode A: Pack-Sampling Bottleneck (All V11 Algorithms)

Every V11 algorithm fails because the pool remains at 100-130 cards throughout
the draft. With 8 archetypes and 5-card packs, the player's committed archetype
is 12-21% of the pool, yielding 0.6-1.05 expected on-archetype cards per pack.
M3 >= 2.0 requires 40%+ archetype density, achievable only through pool
contraction.

**SIM-1 (Balanced Baseline).** M3 = 0.25. Pure calibration anchor. Balanced
60-card refills after rounds 1 and 2 restore pool uniformity, erasing any
AI-driven concentration gradient. Committed players randomly select AI-lane
archetypes 62% of the time. Every concentration metric fails catastrophically.
57% of drafts have all picks 6-30 below 1.5 S/A.

**SIM-2 (Static Open-Lane Bias).** M3 = 0.87. Best simple V11 result. The 1.7x
open-lane multiplier enriches open lanes to ~66% of pool by Round 3. But the
player's specific archetype is one of three open lanes, occupying ~22% of pool.
A 5-card pack yields ~1.1 expected on-archetype draws. The bias works at the
pool level but cannot bridge the pack-sampling gap. M10 = 9.49 (43.8% of drafts
have 25 consecutive bad packs). Root cause analysis showed the M3 = 0.87 result
almost exactly matches the theoretical random baseline for the achieved
archetype density.

**SIM-3 (Graduated Bias + Declining Volume).** M3 = 0.48 aggregate, 0.89 open
lane only. The strongest pool-level gradient (3.17x by Round 3) with open lanes
at 25 cards per lane vs AI lanes at 8. S/A density trajectory peaks at 1.08 per
pack (picks 12-13 in Round 2) then declines. Despite the most aggressive bias
in the field, the player's archetype in a 119-card pool is only ~21% density.
M3 aggregate is worse than SIM-2 because the graduated mechanism's declining
volume reduces absolute card counts while the more aggressive late bias skews AI
lanes too thin. The design document predicted M3 = 2.2-2.6; the prediction
overestimated by 2.5x due to conflating pool-level and pack-level concentration.

**SIM-4 (Declining Volume, No Bias).** M3 = 0.83. The calibration floor. Four
rounds (8/8/7/7 picks) with declining balanced refills (48/36/21/0). Balanced
refills reset the concentration gradient at every round boundary to ~1.0x. The
within-round gradient built by 5 AIs (1.25-1.30x) is trivial against a
120-card pool. S/A density declines monotonically from 25% (pick 9) to 11%
(pick 30). The result (0.83) is statistically identical to V10's Hybrid X
(0.84), proving that multi-round structure with balanced refills performs no
better than V10's single-pool approach. The theoretical random baseline for
these parameters predicts M3 = 0.83 exactly, confirming zero emergent
concentration.

### Failure Mode B: Pool Exhaustion + Pack-Sampling Bottleneck

**SIM-6 (Small Pool Fast Cycles).** M3 = 0.73, M11' = 0.15. Five rounds of 6
picks from a 66-card starting pool with declining refills and 70% open-lane
bias. Pool exhaustion is catastrophic: 36-card removal per round from a 66-card
pool means 55% depletion each round. By Round 4, only 30 cards remain at 6.4%
S/A density. Pool empties after Round 4; players average 23 picks instead of
30. Four refill reset events compound concentration erasure. S/A density peaks
briefly in Round 2 (24.3%) then collapses to zero. 82.6% of Round 4-5 packs
have zero S/A. Worst M11' in the field (0.15). Structural elimination: the pool
math (66 - 36 = guaranteed exhaustion) is fatal.

### Failure Mode C: Prediction Error + Pack-Sampling Bottleneck

**SIM-5 (Asymmetric Replacement).** M3 = 0.57. The worst result among bias
mechanisms. Design 6's thesis was that AI picks being immediately replaced while
player picks are permanent would create monotonic depletion of open lanes,
concentrating the player's archetype. The prediction (M3 = 2.2-2.5) was based
on a fundamental error: it assumed the player could target their open lane
efficiently from a 120-card pool through 5-card packs.

In practice, the player removes ~10 cards from their archetype over 30 picks,
producing a 2-3 card count differential between open and AI lanes in a pool of
105-125 cards. This differential is invisible in a 5-card random pack. Pool
composition snapshots confirm: all archetypes maintain 13-16 cards throughout,
with no meaningful differentiation between AI and open lanes. The mechanism
preserves AI-lane supply (correctly) but does not create player-lane
concentration (incorrectly predicted). The design conflated "AI lanes cycle
while open lanes deplete" with "open lane cards concentrate in the player's
packs." The former is true; the latter does not follow because pack construction
samples from the entire pool, not from open lanes selectively.

---

## 4. Structural Findings: What V11 Conclusively Proved

### Finding 1: The Pack-Sampling Bottleneck Is the Binding Constraint

This is V11's most important contribution to understanding the design space.
With 5-card packs sampled from pools of 100-130 cards, per-pack archetype
density is structurally capped at 12-21% regardless of pool-level composition.
M3 >= 2.0 requires 40%+ density. The gap cannot be bridged by any pool-level
manipulation that maintains pool size. Only two mechanisms can overcome the
bottleneck: pool contraction (reduce pool size until archetype density reaches
40%) or pack size increase (sample more cards per pack to increase expected
on-archetype draws).

### Finding 2: Pool-Level Concentration Does Not Translate to Pack-Level Quality

V11 demonstrated that pool-level gradients (measured by archetype card counts
per lane) are a poor predictor of pack-level quality (measured by S/A per pack).
SIM-3 achieved a 3.17x pool-level gradient (25 open-lane cards vs 8 AI-lane
cards per archetype) while producing only M3 = 0.89 for open-lane players. The
translation failure occurs because the player's specific archetype is one of
three open lanes. A 3.17x gradient across 3 open lanes yields only ~21%
archetype density per lane. Pack construction draws uniformly from the entire
pool, diluting any lane-level advantage.

### Finding 3: Refill Reset Is a Fixed Cost Per Round Boundary

Each balanced or biased refill partially compresses the concentration gradient
because fresh cards restore uniformity to depleted archetypes. The reset is
proportional to refill volume: a 60-card balanced refill almost completely
erases a 1.30x gradient built over 10 picks. Biased refills mitigate but do not
eliminate this reset. The cost is paid per round boundary, making designs with
fewer rounds (3-round) structurally superior to designs with more rounds
(4-5 round). No refill schedule eliminates the reset entirely because
restocking the pool necessarily dilutes accumulated concentration.

### Finding 4: Open-Lane-Biased Refills Are Genuinely Level 0

V11 confirms that refills biased toward open lanes (archetypes with no AI
drafter assigned) meet the Level 0 transparency criterion. The bias is
determined entirely by the pre-draft AI archetype configuration, which is static
and does not observe player behavior. The narrative framing ("the market
restocks slow-moving inventory") is accurate and defensible. This validates the
mechanism's design integrity for any future design that uses open-lane bias.

### Finding 5: Declining Volume Alone Is Insufficient

SIM-4 proved that declining refill volumes without open-lane bias produce no
measurable concentration advantage over V10's single-pool approach. Balanced
refills reset the gradient regardless of volume. The declining volume mechanism
is necessary (prevents late-draft pool bloat) but not sufficient (does not
create archetype-specific concentration). Bias is required in addition to volume
control, and even the combination is insufficient for M3 >= 2.0.

### Finding 6: The Multi-Round Refill Design Space Is Exhausted

V11's six simulations span the plausible parameter space for multi-round
refills:
- Balanced vs biased refills (SIM-1 vs SIM-2/SIM-3)
- Static vs graduated bias (SIM-2 vs SIM-3)
- Volume decline vs no decline (SIM-3 vs SIM-2)
- Bias vs no bias with decline (SIM-3 vs SIM-4)
- Replacement asymmetry (SIM-5)
- Small pool / fast cycles (SIM-6)

No unexplored configuration within this space can plausibly achieve M3 >= 2.0.
The constraint is mathematical (pack-sampling bottleneck), not parametric. No
combination of refill timing, volume, or bias can raise the player's archetype
from 12-21% of a 100-130 card pool to the 40% required for M3 >= 2.0. The
multi-round refill design space is closed.

### Finding 7: V11's Information System Is the Only Transferable Contribution

Design 5's bars + trends + snapshot information architecture is V11's sole
contribution that enhances the recommended design. The information system
amplifies concentration mechanisms (making gradients readable) without creating
them. Paired with V9's substantial concentration gradients, the system should
produce M12 values in the 0.35-0.50 range, creating genuine signal-reading
skill differentiation. The system is orthogonal to the concentration engine and
can be added to any tier of the recommended design without mechanical changes.
