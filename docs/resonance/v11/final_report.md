# Resonance Draft System V11: Final Synthesis Report

## 1. Unified Comparison Table

All V11 results: 1000 drafts, 30 picks, Graduated Realistic fitness, committed
player strategy unless noted. V9 and V10 baselines included.

| Algorithm | M3 | M11/M11' | M10 | M6 | M5 | M9 | M12 | Pass |
|-----------|:--:|:-------:|:---:|:--:|:--:|:--:|:---:|:----:|
| **V9 Hybrid B** | **2.70** | **3.25** | 3.8 | 86% | 9.6 | 1.08 | -- | **8/10** |
| V10 Hybrid X (best) | 0.84 | 0.69 | 12.2 | 51% | 6.0 | 0.92 | -- | 7/11 |
| SIM-1 (balanced baseline) | 0.25 | 0.22 | 21.4 | 24% | 1.0 | 0.50 | 0.17 | 3/12 |
| SIM-2 (1.7x open-lane bias) | 0.87 | 0.84 | 9.49 | 61% | 15.9 | 0.82 | 0.09 | 5/11 |
| SIM-3 (graduated bias + declining) | 0.48 | 0.46 | 16.6 | 38% | 24.1 | 0.58 | 0.06 | 4/12 |
| SIM-3 (open lane only) | 0.89 | 0.89 | 9.1 | 60% | -- | -- | -- | -- |
| SIM-4 (declining volume, no bias) | 0.83 | 0.71 | 10.6 | 59% | 16.8 | 0.80 | 0.16 | 5/11 |
| SIM-5 (asymmetric replacement) | 0.57 | 0.56 | 14.2 | 46% | 28.1 | 0.68 | 0.09 | 6/15 |
| SIM-6 (small pool fast cycles) | 0.73 | 0.15 | 10.3 | 54% | 12.9 | 0.81 | 0.11 | 7/13 |

No V11 algorithm passes M3 >= 2.0. The best committed-player M3 across all six
simulations is 0.89 (SIM-3 forced open lane), which is 67% below V9's 2.70 and
comparable to V10's best result (0.84). This is a definitive negative result.

---

## 2. The Key Question

**Does multi-round drafting with refills produce V9-level metrics while
preserving the AI drafter narrative?**

**No.** V11 explored the complete parameter space of multi-round refill
mechanisms: balanced refills (SIM-1), static open-lane bias (SIM-2), graduated
bias with declining volume (SIM-3), declining volume without bias (SIM-4),
asymmetric replacement (SIM-5), and small-pool fast cycles (SIM-6). Every
algorithm failed M3 by wide margins. The best result (SIM-2/SIM-3 at M3 ~0.87-
0.89 for open-lane players) is structurally capped at less than half the 2.0
target.

The failure is not parametric. It is caused by a binding structural constraint:
the **pack-sampling bottleneck**.

### The Pack-Sampling Bottleneck

The core mechanism is straightforward. With a pool of 100-130 cards and 8
archetypes, the player's committed archetype represents approximately 12-21% of
the pool depending on refill bias. A 5-card pack drawn randomly from this pool
yields 0.6-1.05 expected cards from the player's archetype. With sibling
fitness (36% weighted average), expected S/A per pack reaches approximately
0.83-1.3.

To achieve M3 >= 2.0 with 5-card packs requires archetype density >= 40% in
the effective pool (5 x 0.40 = 2.0). No V11 mechanism produces this density.
Even the strongest bias (SIM-3, 2.0x multiplier + declining volume, producing a
3.17x gradient) yields only ~21% archetype density for the player's specific
archetype in a 119-card pool. The density gap between 21% and 40% cannot be
closed by any amount of refill bias, because refills maintain the pool at 100-
130 cards.

V9 achieved 40-60% archetype density by contracting the pool from 360 cards to
~17 by pick 30. This virtual contraction is the mechanism that produces M3 >=
2.0. V11's refill structure maintains pool size, preventing this concentration.
**Pool contraction is the only mechanism that converts early archetype
commitment into late-draft pack quality at the levels required by M3 >= 2.0.**

---

## 3. Round Structure Analysis

### Three-Round Structure (SIM-1, SIM-2, SIM-3)

The 3-round structure (10 picks per round, 60-card refills) is the cleanest V11
framework. It provides natural cognitive anchors (round-start recalibration
moments), familiar strategic pacing, and two refill events for information
updates. SIM-2's 1.7x open-lane bias achieves M3 = 0.87 in this structure,
the joint-best V11 result.

The refill reset is the primary limitation. Each balanced or biased refill
partially compresses the concentration gradient. Even biased refills cannot
prevent this: SIM-3's graduated bias produced a 3.17x pool-level gradient by
Round 3 but the per-pack S/A density never exceeded 1.3 because the pool
remained at 119 cards.

### Four-Round Structure (SIM-4)

The 4-round structure (8/8/7/7 picks, declining refills 48/36/21/0) tests
whether declining volume alone builds concentration without bias. SIM-4's
M3 = 0.83 is statistically indistinguishable from V10's Hybrid X (0.84). The
balanced refills completely reset the concentration gradient at every round
boundary. Each round, AIs build a modest 1.25-1.30x gradient that is erased by
the balanced refill. S/A density actually declines over the draft (25% at
pick 9 to 11% at pick 30), the opposite of the desired trajectory.

SIM-4 provides the calibration floor: M3 = 0.83 is the baseline achievable
through emergent AI-driven concentration with no bias. Any V11 design exceeding
this floor (SIM-2, SIM-3) is doing so through its bias mechanism.

### Five-Round Fast Cycles (SIM-6)

The 5-round structure (6 picks per round, 66-card starting pool) is
non-competitive. Pool exhaustion is catastrophic: 36-card-per-round depletion
from a 66-card pool means the pool is 55% depleted each round. By Round 4, only
30 cards remain at 6.4% S/A density. Round 5 never executes. Players average
only 23 picks instead of 30. M11' = 0.15 reflects total late-draft collapse.
Four refill reset events compound the concentration erasure problem. This
structure is eliminated.

### Structural Finding

Among multi-round designs, fewer rounds with larger refills outperform more
rounds with smaller refills. The refill reset is a fixed cost per round
boundary, and concentrating the budget into 2 events (3-round design) minimizes
this cost. No round count resolves the pack-sampling bottleneck.

---

## 4. Refill Bias Analysis

### Balanced Refills (SIM-1, SIM-4): Null Hypothesis

Balanced refills produce M3 = 0.25 (SIM-1) to 0.83 (SIM-4). The refill reset
completely erases any concentration gradient AIs build within a round. Balanced
refills restore pool uniformity, making AI-lane depletion invisible across
round boundaries. This confirms that AI-driven concentration alone, without
refill bias, cannot reach M3 >= 2.0. The finding is definitive.

### Static Open-Lane Bias (SIM-2): Best Simple Mechanism

The 1.7x open-lane multiplier achieves M3 = 0.87, a 3.5x improvement over
SIM-1 committed (0.25). The improvement comes from two sources: (1) committed
players always select open-lane archetypes, and (2) biased refills enrich open
lanes with more cards. The mechanism is genuinely Level 0 (bias determined
entirely by pre-draft AI assignment, independent of player behavior). SIM-2's
simplicity makes it the strongest calibration anchor in the field.

### Graduated Bias + Declining Volume (SIM-3): Strongest Pool-Level Gradient

SIM-3 combines graduated multipliers (1.4x after Round 1, 2.0x after Round 2)
with declining refill volumes (70/48 cards). This produces the strongest
pool-level gradient in V11: 3.17x by Round 3 start, with open lanes at 25
cards per lane versus AI lanes at 8 per lane. The mechanism works exactly as
designed at the pool level.

However, SIM-3's M3 (0.48 aggregate, 0.89 open-lane-only) is worse than
SIM-2's M3 (0.87). The graduated bias and declining volume reduce the absolute
number of cards available while failing to produce proportionally higher
per-pack density. The more aggressive late-round bias skews AI lane counts so
low that AIs struggle to draft, producing knock-on effects. Simpler is better:
SIM-2's static 1.7x outperforms SIM-3's graduated mechanism on M3.

### Asymmetric Replacement (SIM-5): Structural Failure

Design 6's asymmetric replacement (AI picks replaced immediately, player picks
permanent) produces M3 = 0.57, the worst among bias mechanisms. The prediction
of M3 = 2.2-2.5 was based on conflating pool-level depletion with pack-level
probability. In a 120-card pool, the player removes approximately 10 cards from
their archetype over 30 picks, producing a 2-3 card count differential between
open and AI lanes. This differential is invisible in a 5-card random pack. The
mechanism does not create concentration because the pool is maintained at
105-125 cards throughout.

### Is Any Bias Mechanism Contraction in Disguise?

No V11 bias mechanism is contraction in disguise. All mechanisms work by
adjusting pool-level composition (adding more open-lane cards) rather than
reducing pool size. The problem is precisely that they maintain pool size: a
120-card pool with any composition still yields low per-pack archetype density
because 5 cards is too small a sample to reliably hit 15-25 target cards in
a 120-card population. Only pool contraction (virtual or physical) converts
pool-level composition into pack-level quality.

---

## 5. Player Information Analysis: Design 5's Contribution

Design 5 (Bars + Round-Start Snapshot + Depletion Trends) is V11's strongest
contribution to the overall design. The three-layer information system:

1. **Availability bars** show archetype-level card counts (not quality),
   preserving the quality-uncertainty skill axis
2. **Round-start snapshots** provide a planning anchor that naturally
   becomes stale by mid-round, rewarding early-round action
3. **Depletion trend arrows** uniquely reward signal-readers by revealing
   which archetypes are being actively drafted

The critic review confirmed that this information system amplifies a
concentration mechanism rather than creating one. Under balanced refills
(SIM-1), M12 = 0.17 because there is insufficient signal to read. Under
biased refills (SIM-2), M12 = 0.09 because the open-lane advantage is so
obvious that committed players benefit equally. The information system is most
valuable when paired with moderate concentration mechanisms that create readable
but non-obvious gradients.

Design 5 carries forward unchanged into the recommended design. When paired
with V9's contraction engine, the concentration gradients will be substantial
enough for the trend indicators to create meaningful M12 differentiation.

---

## 6. Per-Archetype Convergence: Top Algorithms

### SIM-2 Per-Archetype M3

| Archetype | M3 | M11' | Sibling Rate |
|-----------|:--:|:----:|:------------:|
| Warriors | 0.61 | 0.58 | 50% |
| Sacrifice | 0.56 | 0.50 | 50% |
| Self-Discard | 0.56 | 0.49 | 40% |
| Self-Mill | 0.55 | 0.50 | 40% |
| Storm | 0.50 | 0.45 | 30% |
| Blink | 0.46 | 0.41 | 30% |
| Flash | 0.47 | 0.43 | 25% |
| Ramp | 0.46 | 0.42 | 25% |

All archetypes fail M3 by wide margins. The spread (0.15) is moderate, with
high-sibling-rate pairs (Warriors/Sacrifice at 50%) outperforming low-rate
pairs (Flash/Ramp at 25%) by approximately 0.10 M3. This gradient is correct
and expected but the absolute values are uniformly catastrophic. No V11
archetype achieves M3 >= 1.0 except in isolated forced-open-lane conditions.

### Comparison to V9 Hybrid B Per-Archetype

| Archetype | V9 M3 | Best V11 M3 | Gap |
|-----------|:-----:|:-----------:|:---:|
| Warriors | 2.78 | 0.93 | 1.85 |
| Sacrifice | 2.75 | 0.95 | 1.80 |
| Self-Discard | 2.76 | 0.91 | 1.85 |
| Self-Mill | 2.82 | 0.88 | 1.94 |
| Flash | 2.58 | 0.83 | 1.75 |
| Blink | 2.63 | 0.85 | 1.78 |
| Storm | 2.66 | 0.86 | 1.80 |
| Ramp | 2.66 | 0.87 | 1.79 |

Best V11 values taken from SIM-3 forced-open-lane configuration. The gap of
1.75-1.94 M3 per archetype is uniform, confirming the pack-sampling bottleneck
affects all archetypes equally and is not a fitness model artifact.

---

## 7. V11 vs V9 vs V10: What Each Version Proved

### V9: Pool Contraction Is the Concentration Engine

V9 proved that virtual pool contraction at 12% per pick, using a blended
relevance score (40% visible dot-product + 60% pair-affinity), produces M3 =
2.70 and M11 = 3.25 with 8 bits of hidden metadata per card. It established
that visible resonance symbols do 85% of targeting work (V1 = 84.8%) and that
pair-affinity encoding (8 bits) is the minimum sufficient hidden information
level. V9 remains the only version to achieve both M3 >= 2.0 and M11 >= 3.0.

### V10: AI Drafters Provide Narrative, Not Concentration

V10 proved that physical AI drafters cannot replicate V9's virtual contraction.
Physical removal depletes S/A (opposite of V9), exhausts the pool (non-self-
regulating), and dilutes targeting (general vs specific concentration). V10's
genuine contribution is the AI drafter narrative: "other players took those
cards" is more intuitive than "the game removed cards." The 5-AI / 3-open-lane
structure provides signal reading skill and game-to-game variety. V10
recommended V9's engine + AI narrative layer.

### V11: Multi-Round Refills Fix Pool Exhaustion But Cannot Replace Contraction

V11 proved three things and failed at its primary objective:

**Proved:** Multi-round refills completely solve V10's pool exhaustion problem
(root cause 1). No V11 algorithm exhausts its pool prematurely (except SIM-6,
where the design was structurally flawed). This finding is definitive.

**Proved:** Open-lane-biased refills are genuinely Level 0 and create real
pool-level gradients. SIM-3's 3.17x gradient demonstrates that open-lane bias
enriches the pool composition toward uncontested archetypes through a mechanism
that is static, pre-determined, and independent of player behavior. This
validates the Level 0 classification from the research phase.

**Proved:** The pack-sampling bottleneck is the binding constraint. With 100-130
cards in the pool and 5-card packs, per-pack archetype density is capped at
~21% regardless of pool-level composition. This is V11's most important
structural contribution: it identifies exactly why pool-level concentration
does not translate to pack-level quality, and establishes the mathematical
relationship between pool size, pack size, and M3 outcomes.

**Failed:** No multi-round refill mechanism achieves M3 >= 2.0. The pack-
sampling bottleneck cannot be overcome without pool contraction. V9's mechanism
(shrinking the effective pool from 360 to ~17 cards) is necessary, not merely
sufficient, for M3 >= 2.0 with 5-card packs.

---

## 8. Recommendation Tiers

### Simple: V9 Hybrid B + AI Narrative Layer

**Source:** V10 recommended this design. V11 validates it by eliminating all
alternatives.

**What it is:** V9's contraction engine (12% per pick, 40/60 visible/affinity
blend) runs unchanged. Five AI drafters are presented as opponents at the table.
Each AI is assigned one of 8 archetypes (3 uncontested per game). AIs have no
mechanical effect on pack construction. When V9's contraction removes cards,
removals are attributed to AI drafters as picks. The player reads which
archetypes are open from pack composition.

**Metrics:** M3 = 2.70, M11 = 3.25, M10 = 3.8, M6 = 86%, V1 = 84.8%.

**When to choose:** Default recommendation. Minimal implementation complexity
beyond V9 (add presentation layer only). No multi-round UI needed. The proven
engine with the proven narrative.

### Standard: V9 Hybrid B + AI Narrative Layer + Design 5 Information System

**What it adds:** Design 5's three-layer information system (availability bars,
round-start snapshots, depletion trend arrows) layered on top of the Simple
tier. The bars display archetype-level card availability (not quality). Trend
arrows show which archetypes are depleting fastest. Snapshots provide
round-start planning anchors.

**How the information system integrates with V9:** V9's contraction produces
concentration gradients that are visible through pack composition. The bars and
trends make these gradients explicit and readable, creating a genuine
signal-reading skill axis. Players who identify open archetypes from trend data
commit earlier and more accurately.

**Additional integration detail:** V9's draft does not have natural round
boundaries. Two options: (a) Introduce artificial round boundaries (every 10
picks) purely for information display, with snapshot resets at boundaries and
trend baselines recalculated. The contraction engine runs continuously. (b) Use
rolling 5-pick windows for trends and periodic snapshots as described in
Design 5's continuous-market adaptation. Option (a) is recommended because
round boundaries provide stronger cognitive anchors.

**Metrics:** M3 = 2.70, M11 = 3.25 (unchanged from V9 engine), plus enhanced
M12 from the information system (projected 0.35-0.50 based on Design 5's
analysis, assuming V9's concentration gradients provide sufficient signal for
trend indicators to differentiate).

**When to choose:** When signal-reading skill and player agency are design
priorities alongside concentration quality.

### Advanced: V9 Hybrid B + Lightweight Physical AI Removal + Design 5 Information + Multi-Round Presentation Structure

**What it adds:** A small mechanical contribution from AI drafters. Each AI
physically removes 1 card per round from its archetype (5 total physical
removals per round), before V9's contraction processes the remaining pool. This
strengthens the narrative: AI picks are real, not merely attributed. The draft
is presented in a multi-round format (3 rounds of 10 picks) with refill
presentation moments between rounds, even though V9's contraction runs
continuously underneath.

Design 5's full information system is displayed at round boundaries. Bars reset
at round starts. Trend arrows track within-round depletion. Snapshots show
post-refill pool state.

**Metrics:** M3 ~2.55-2.65 (slight degradation from physical S/A depletion),
M11 ~3.0-3.15 (earlier pool floor arrival), M10 ~3.5-4.0. The degradation is
modest because 5 physical removals per round are small relative to V9's virtual
contraction volume (~43 virtual removals in early picks).

**When to choose:** When the narrative integrity of "real" AI picks is a high
priority, and the team accepts a small metric cost for the presentation benefit.
Highest implementation complexity.

---

## 9. Complete Specification: Standard Recommended Algorithm

### Contraction Engine (V9 Hybrid B, Unchanged)

| Parameter | Value |
|-----------|-------|
| Pool size | 360 cards (40 generic + 284 single-symbol + 36 dual-symbol) |
| Contraction start | Pick 4 |
| Contraction rate | 12% per pick |
| Relevance blend | 40% visible dot-product + 60% pair-affinity score |
| Floor slot | 1 top-quartile slot from pick 3 |
| Generic protection | 0.5 baseline relevance |
| Signature weights | +2 primary, +1 secondary resonance per pick |
| Pool minimum | 17 cards (stop contraction) |
| Archetype inference | Mode of inferred archetype from drafted cards' higher-affinity label, from pick 5 |
| Hidden metadata | 8 bits/card: two 4-bit pair-affinity floats |
| Pack size | 4 cards (3 random + 1 floor slot from pick 3) |

### AI Narrative Layer

| Parameter | Value |
|-----------|-------|
| Number of AIs | 5 per game |
| Archetype assignment | Random 5 of 8, no duplicates |
| Open archetypes | 3 per game |
| Mechanical effect | None (presentation only) |
| Signal display | Show which archetypes are "contested" via UI |
| Narrative framing | Cards removed by contraction attributed to AI picks |
| Saturation display | AIs visually slow down after ~12 apparent picks |
| Per-game variety | C(8,5) = 56 possible compositions |
| AI personality | Optional: aggression/focus labels for flavor |

### Design 5 Information System

| Element | Specification |
|---------|---------------|
| Availability bars | 8 bars (one per archetype), grouped by resonance symbol. Relative height = card count as fraction of round-start count. Labels show resonance symbol only. Updated after each pick cycle. |
| Depletion trends | Directional arrow per bar. Fast (>1.5x avg depletion) = downward/orange. Stable (0.5-1.5x) = rightward/grey. Slow (<0.5x) = upward/green. Smoothed over 2 pick cycles. |
| Round-start snapshot | At each artificial round boundary (picks 1, 11, 21): approximate card counts per resonance symbol + brief quality descriptor. Dims progressively from pick 4 of each round onward. |
| Information not shown | Individual card counts, AI identity, AI archetype assignments, S/A vs C/F breakdown, refill preview |
| Artificial rounds | Every 10 picks. Purely for information display; contraction engine runs continuously. |

### Integration Architecture

```
Player pick
  -> Update resonance signature (+2 primary, +1 secondary)
  -> V9 contraction: remove bottom 12% by blended relevance
  -> Attribute removed cards to AI drafters (nearest archetype match)
  -> Update bars and trends from new pool state
  -> Construct next pack: 1 top-quartile + 3 random from surviving pool
  -> Display pack to player

Every 10 picks (artificial round boundary):
  -> Display round-start snapshot
  -> Reset trend baselines
  -> Show AI "round summary" (narrative: "AI drafters selected N cards")
```

---

## 10. Implementation Guide

### Phase 1: V9 Contraction Engine

Implement V9 Hybrid B exactly as specified in the V9 algorithm overview. This
is the mathematical foundation. The engine manages pool contraction, relevance
scoring, pack construction, and archetype inference. Test against V9's metrics
(M3 = 2.70, M11 = 3.25) before proceeding.

Key components:
- Resonance signature tracker (4-element vector, updated per pick)
- Relevance scorer (40/60 blend of visible dot-product and pair-affinity)
- Pool contraction logic (remove bottom 12%, respect generic protection floor)
- Pack constructor (1 top-quartile + 3 random slots)
- Archetype inference module (mode of higher-affinity labels from pick 5)

### Phase 2: AI Presentation Layer

1. At draft start, randomly select 5 of 8 archetypes for AI assignment
2. Create UI elements showing 5 AI "opponents" with archetype indicators
3. As V9's contraction removes cards each round, attribute removals to the AI
   whose archetype is closest to the removed card's pair-affinity
4. Display AI "draft picks" as a log or visual indicator
5. Show AI "saturation" (slowing apparent picks) after ~12 attributed picks

### Phase 3: Design 5 Information System

1. Implement availability bars grouped by resonance symbol
2. Implement depletion trend calculation with 2-cycle smoothing
3. Implement round-start snapshot display with progressive dimming
4. Add artificial round boundaries every 10 picks for information reset
5. Ensure information elements display on the same screen as pack selection

### Phase 4: Playtesting

1. Test V9 engine alone (no AI narrative, no information) to verify metrics
2. Add AI narrative and verify it does not alter V9's mechanical behavior
3. Add Design 5 information and measure M12 impact
4. Compare player satisfaction across Simple/Standard/Advanced tiers
5. Test whether artificial round boundaries improve or disrupt pacing
6. Measure cognitive load of three-layer information system
7. Test signal-reading skill expression: do players learn to use trends?

---

## 11. Open Questions

### Resolved by V11 (No Further Exploration Needed)

1. **Can multi-round refills replace V9's contraction?** No. The pack-sampling
   bottleneck makes this structurally impossible. Closed.

2. **Does open-lane refill bias achieve M3 >= 2.0?** No. Even 3.17x pool-level
   gradients produce only M3 = 0.89 in 5-card packs. Closed.

3. **Is asymmetric replacement a viable concentration mechanism?** No. Pool
   size maintenance prevents pack-level concentration. Closed.

4. **Does declining refill volume alone build concentration?** No. Balanced
   refills reset the gradient at every round boundary. Bias is required, and
   even bias is insufficient. Closed.

5. **Can fast cycles (5 rounds) compete with 3-round designs?** No. Pool
   exhaustion in thin rounds is fatal. Closed.

### Open for Future Investigation

1. **Can the pack-sampling bottleneck be addressed by increasing pack size?**
   With 10-card packs from a 119-card pool with 25 on-archetype cards, expected
   S/A per pack = ~2.1. This is a UI change, not a mechanism change. Worth
   investigating if the team considers larger packs acceptable.

2. **Can hybrid contraction within multi-round refills work?** Adding mild
   within-round virtual contraction (5-8% per pick) to a 3-round refill
   structure could contract the effective pool during each round while refills
   restore it between rounds. This preserves the multi-round narrative while
   using V9-style concentration within rounds.

3. **What is the optimal AI count for the narrative layer?** V10 tested 5-7 AIs
   in a physical-removal context. With V9's virtual contraction underneath, the
   optimal AI count for signal-reading quality may differ. Worth playtesting.

4. **Does the Design 5 information system improve M5 convergence?** V9's M5 =
   9.6 (target 5-8). The trend indicators and snapshots may help players commit
   earlier. Not tested in simulation.

5. **How does the attribution of V9 removals to AI picks feel in practice?**
   V9 removes cards by low relevance to the player. Attributing these to AIs
   whose archetypes differ from the player's is narratively consistent but
   requires careful UI design to avoid contradictions.

6. **Should V12 explore the multi-round presentation structure without multi-
   round mechanics?** The 3-round presentation format (with refill moments
   as narrative beats) may improve pacing even when V9's contraction runs
   continuously. This separates presentation structure from mechanical
   structure.
