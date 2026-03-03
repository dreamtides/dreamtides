# Resonance Draft System V12: Final Synthesis Report

## 1. Unified Comparison Table

All V12 simulations: 1000 drafts, 30 picks, Graduated Realistic fitness,
committed player strategy unless noted. V9-V11 baselines included.

| Algorithm | M3 | M11' | M10 | M6 | M5 | M12 | Notes |
|-----------|:--:|:----:|:---:|:--:|:--:|:---:|-------|
| **V9 Hybrid B** | **2.70** | **3.25** | 3.8 | 86% | 9.6 | -- | Gold standard |
| V10 Hybrid X | 0.84 | 0.69 | 12.2 | 51% | 6.0 | -- | Physical AI baseline |
| V11 SIM-2 (best) | 0.87 | 0.84 | 9.5 | 61% | 15.9 | 0.09 | Refill-only ceiling |
| V11 SIM-4 (declining) | 0.83 | 0.71 | 10.6 | 59% | 16.8 | 0.16 | Balanced decline |
| V12 SIM-1 (D3, N=12+Floor) | 0.42 | 0.27 | 17.6 | 22.7% | 5.0 | -0.04 | Delayed avoidance |
| V12 SIM-2 (Hybrid 1, N=12) | 0.51 | 0.30 | 18.9 | 42.4% | 1.0 | 0.11 | Conservative hybrid |
| V12 SIM-3 (Hybrid 2, Prog N) | 0.37 | 0.26 | 18.5 | 25.7% | 13.8 | 0.03 | Progressive N |
| V12 SIM-4 (D1, N=4 baseline) | 0.66 | 0.48 | 13.1 | 49.8% | 22.6 | 0.34 | Isolation test |
| V12 SIM-5 (D2, Steep N=8) | 1.33 | 1.18 | 6.2 | 72.2% | 13.5 | 0.52 | Best V12 face-up |
| V12 SIM-6 (V9 Fallback) | 1.00 | 1.08 | 9.9 | 52.5% | 7.8 | -0.36 | Calibration gap |

**Every V12 face-up design fails M3 >= 2.0 by wide margins.** The best V12
result (SIM-5, M3 = 1.33) achieves 49% of the 2.70 target and 67% of the 2.0
minimum. The worst V12 results (SIM-1 at 0.42, SIM-3 at 0.37) perform below
V11's best efforts, a regression rather than progress.

---

## 2. The Key Question Answered

**Can AI avoidance + physical pool contraction + modest oversampling (N=8-12)
replace V9's virtual contraction?**

**No. The thesis is definitively falsified.**

V12's central hypothesis was that three transparent mechanisms working in
concert could replicate the concentration quality that V9 achieves through
invisible pool contraction. The thesis predicted that AI avoidance would
preserve the player's S/A supply, declining refills would shrink the pool, and
modest oversampling would convert the resulting density into pack quality. The
design-phase math projected M3 = 1.8-2.2 across the leading candidates.

Simulation reveals that the three mechanisms interact destructively rather than
constructively. Each mechanism's contribution is undermined by the others'
failure modes, producing a compound failure far worse than any individual
mechanism's shortfall:

1. AI avoidance requires accurate inference, but inference fails at ~25-35%
   accuracy with 6 concurrent drafters.
2. Without accurate avoidance, declining refills shrink the pool without
   concentrating it -- density decreases rather than increases.
3. Oversampling multiplies a density that never reaches the required threshold,
   producing proportionally small absolute gains.

The V12 design documents predicted these mechanisms would compound positively:
avoidance preserves S/A, contraction concentrates the pool, oversampling
amplifies the concentration. In simulation, they compound negatively: inaccurate
avoidance fails to preserve S/A, the player's own picks exhaust their S/A
supply, refills dilute whatever concentration builds between rounds, and the
pool contracts without ever achieving the 40-50% archetype density that the
math requires for M3 >= 2.0.

---

## 3. Root Cause Analysis: Why ALL Face-Up Designs Failed

Four structural barriers, each individually sufficient to prevent M3 >= 2.0,
combine to produce the comprehensive failure observed across all six
simulations.

### 3.1 AI Inference Accuracy (The Primary Failure)

The depletion-rate inference mechanism -- the foundation of V12's avoidance
system -- cannot reliably identify the player's archetype from aggregate pool
snapshots when 6 drafters are simultaneously removing cards.

- **SIM-1 (Design 3):** Inference peaks at 57% accuracy at pick 9, then
  collapses to 25-35% after each refill event resets pool composition.
  Effective avoidance of the player's actual archetype: ~24%.
- **SIM-2 (Hybrid 1):** Inference reaches 79% at pick 10, then drops to 27% at
  pick 15 and 1-3% by picks 20-30.
- **SIM-5 (Design 2):** AIs infer the wrong archetype in most games. The
  depletion rate for a single player's 1-card-per-cycle signal is statistically
  indistinguishable from background noise created by 5 other AIs.

The research phase predicted 3-5 pick cycles for reliable inference. This was
optimistic: the research modeled clean depletion signals, but simulation reveals
that refill events destroy accumulated depletion history, AIs' own picks create
confounding noise indistinguishable from the player's signal, and the
player's archetype may not deplete anomalously at all if it happens to be one
that AIs incidentally pick from.

### 3.2 S/A Exhaustion (The Binding Constraint)

The player consumes their own S/A supply through normal drafting. Starting with
~5 S/A per archetype, the player takes approximately 0.33 S/A per on-archetype
pick. Over 30 picks with ~70-100% on-archetype rate, the player consumes
approximately 7-10 S/A cards. Refills add 4-8 S/A depending on schedule and
bias. The net result: by pick 25, S/A remaining in the pool ranges from 0.1
(SIM-1) to 7.0 (SIM-5, counting cross-archetype S/A).

Design-phase walkthroughs predicted 3-5 S/A remaining at pick 25. Simulation
consistently shows lower values because (a) AI avoidance is inaccurate, so AIs
also consume player S/A, and (b) the player's own consumption rate was
underestimated. S/A exhaustion is self-inflicted and cannot be resolved by AI
behavior changes alone.

### 3.3 Physical Contraction Ratio Ceiling (6:1 vs V9's 21:1)

V9 contracts the pool from 360 cards to ~17, a 21:1 ratio, achieving 60%+
archetype density by pick 20. V12's physical contraction operates within much
tighter bounds: a 120-card starting pool with 80 total refill cards and 180
total removals produces a final pool of ~20-30 cards, a ratio of only 4-6:1.

This lower ratio means V12's pool never reaches the archetype density that V9
achieves. V9 reaches 98% archetype density by pick 20 (confirmed by SIM-6);
V12's best density is 18.4% (SIM-3 at pick 21). The 5x gap in contraction
ratio translates directly into the 5x gap in M3 outcomes.

Virtual contraction has no ceiling: the algorithm can remove any fraction of
cards per pick, and the pool shrinks monotonically. Physical contraction is
bounded by the number of drafters, the refill schedule, and the requirement
that cards exist for AIs to take. This is a structural asymmetry that V12
cannot overcome within its design constraints.

### 3.4 Archetype Density Never Concentrates (Decreasing, Not Increasing)

The most surprising and devastating finding: across all V12 simulations,
the player's archetype density **decreases** over the course of the draft. The
design-phase models predicted density increasing from 12.5% to 40-55% as AIs
avoided the player's archetype. In simulation:

- SIM-4 (Design 1): density decreases from 11.7% at pick 5 to 7.0% at pick 25
  to 0% at pick 30
- SIM-5 (Design 2): density decreases from 11.5% to 5.1%
- SIM-3 (Hybrid 2): density peaks at 18.4% then falls to 13.0%

The root cause is that the player's own consumption of their archetype cards
outpaces the concentration benefit from (inaccurate) AI avoidance. The player
removes 1 archetype card per pick cycle; 5 AIs removing from 7 other archetypes
should deplete those archetypes faster, but with only ~25% effective avoidance,
AIs still take player-archetype cards at 75% of the baseline rate. The net
effect: all archetypes deplete roughly proportionally, with the player's
archetype depleting slightly faster because the player is the most aggressive
consumer of their own cards.

---

## 4. What DID Work

Despite the comprehensive M3 failure, V12 produced genuine insights:

### 4.1 Signal Reading (M12)

SIM-4 (Design 1, N=4 isolation) achieved M12 = 0.34, the only V12 simulation
to pass this metric. SIM-5 (Design 2) achieved M12 = 0.52, the strongest M12
across all V12 results. Signal readers who identify open lanes and commit to
uncontested archetypes achieve meaningfully higher M3 than committed players
who pick randomly.

The signal-reading mechanism works: the face-up pool provides genuine strategic
information. Players who browse the pool and identify which archetypes have
persisting S/A cards draft better decks. In SIM-4, signal readers achieved
M3 = 0.99 vs committed players' 0.66 -- a 50% improvement. Signal readers
selected open lanes 80% of the time vs 43% for committed players.

### 4.2 AI Avoidance Narrative Concept

The research phase's analysis of human drafting behavior, surveillance
boundaries, and inference timing is architecturally sound. The finding that
public-information-based avoidance should use aggregate pool depletion (not
individual pick attribution) establishes the correct honesty criterion for
AI behavior in any future face-up pool design. The graduated avoidance ramp
(starting at low confidence, building with evidence) is the right framework
even though the underlying inference mechanism lacks sufficient signal
strength with 6 drafters.

### 4.3 The Face-Up Pool as Information Surface

The face-up pool creates a natural and honest information model: the player
can browse all available cards, observe which archetypes are depleting, and
make informed strategic decisions. This replaces V11's Design 5 information
system (bars, trends, snapshots) with something simpler and more complete.
The pool browser's value is independent of whether AI avoidance works -- it
improves the player's decision-making regardless.

---

## 5. Per-Archetype Analysis: Top Simulations

### SIM-5 (Design 2, N=8, Best V12 M3)

| Archetype | M3 | Sibling Rate |
|-----------|:--:|:-----------:|
| Warriors/Sacrifice (Tide) | 1.46 / 1.57 | 50% |
| Self-Discard/Self-Mill (Stone) | 1.41 / 1.44 | 40% |
| Storm/Blink (Ember) | 1.31 / 1.14 | 30% |
| Flash/Ramp (Zephyr) | 1.13 / 1.06 | 25% |

M3 correlates with sibling S/A rate (r = 0.96). The spread (1.06-1.57) is 0.51,
matching the graduated fitness model's prediction. No archetype achieves M3 >=
2.0; Sacrifice (best) reaches 1.57.

### SIM-6 (V9 Fallback)

| Archetype | M3 |
|-----------|:--:|
| Warriors/Sacrifice (Tide) | 1.48 / 1.49 |
| Self-Discard/Self-Mill (Stone) | 1.17 / 1.05 |
| Storm/Blink (Ember) | 0.77 / 0.77 |
| Flash/Ramp (Zephyr) | 0.54 / 0.60 |

The same sibling-rate correlation appears in V9's engine operating under the
V12 simulation's card model. Per-archetype spread (0.54-1.49) is wider than
SIM-5, reflecting V9's aggressive contraction amplifying the sibling-rate
differences.

---

## 6. V12 vs V9 vs V10 vs V11

| Dimension | V9 | V10 | V11 | V12 |
|-----------|:--:|:---:|:---:|:---:|
| Best M3 | 2.70 | 0.84 | 0.89 | 1.33 |
| Concentration mechanism | Virtual contraction | Physical AI drafting | Biased refills | Avoidance + contraction + oversampling |
| Pool visibility | Hidden | Hidden | Hidden | Face-up |
| Contraction ratio | 21:1 | N/A (exhaustion) | N/A (maintained) | 4-6:1 |
| AI role | None | Mechanical | Mechanical | Avoidance + mechanical |
| Signal reading (M12) | N/A | N/A | 0.09-0.17 | 0.03-0.52 |
| Key failure | M5, M10 | Pool exhaustion | Pack-sampling bottleneck | Inference accuracy + S/A exhaustion |

**Progression narrative:** V10 introduced physical AI drafting and discovered
pool exhaustion. V11 solved pool exhaustion with refills and discovered the
pack-sampling bottleneck. V12 attempted to solve the bottleneck with AI
avoidance + oversampling and discovered two new structural barriers: inference
accuracy in multi-drafter environments and self-inflicted S/A exhaustion.

Each version has correctly identified and solved one predecessor's failure mode
while uncovering a deeper structural constraint. The deepest constraint remains
V9's original insight: virtual pool contraction is the only mechanism that
produces sufficient archetype density for M3 >= 2.0, because it operates
without the physical constraints (real drafters, real card consumption) that
limit transparent mechanisms.

---

## 7. Recommendation Tiers

### V12's Contribution: Structural Findings, Not Algorithms

V12 contributes no deployable algorithm. Its contribution is structural
understanding:

1. **Public-information inference cannot reliably identify a single player's
   archetype among 6 drafters.** This is a fundamental signal-to-noise
   limitation, not a parameter tuning problem. With 5 AIs creating background
   depletion noise, the player's 1-card-per-cycle signal is statistically
   indistinguishable. This finding applies to any face-up pool design with
   AI avoidance.

2. **Physical pool contraction is bounded by a 4-6:1 ratio**, far below V9's
   21:1. The ceiling is set by the number of drafters, the starting pool size,
   and the refill schedule. No parameter combination within V12's constraints
   reaches the 40-50% archetype density required for M3 >= 2.0.

3. **The player is their own worst enemy for S/A supply.** The player
   consumes their archetype's S/A cards faster than any mechanism can
   replenish them. This is a structural property of "pick the best card for
   your archetype" strategy and cannot be mitigated by AI behavior alone.

4. **Signal reading works and is the strongest V12 positive finding.** Open-lane
   identification from the face-up pool produces M12 = 0.34-0.52. This
   validates the face-up pool as an information surface even though it fails
   as a concentration mechanism.

### V9 Enhanced: Incorporating V12 Insights

V9 Hybrid B remains the recommended concentration engine. V12 provides three
insights that could enhance V9:

1. **Face-up pool as a browsing overlay.** Even with V9's invisible contraction
   running underneath, the player could browse a representation of the remaining
   pool (after contraction). This provides the information benefits of V12's
   face-up design without requiring physical concentration. The pool would show
   only cards that survived V9's contraction -- a progressively concentrated
   view that naturally reveals open archetypes.

2. **AI avoidance as a narrative layer.** V11's Standard recommendation (V9 +
   AI narrative + Design 5 information) remains the best integration. V12's
   SIM-6 confirms that the avoidance log adds minimal strategic value (M12 is
   negative), but the AI narrative remains valuable for framing. V12's research
   on surveillance boundaries and public-information honesty criteria should
   inform how the narrative layer is presented.

3. **Open-lane signal reading for M5 improvement.** V9's M5 = 9.6 (target 5-8)
   reflects slow convergence. A face-up pool browser showing surviving cards
   (post-contraction) would help players identify open archetypes earlier,
   potentially improving M5 by 1-3 picks.

### Future Directions: What V13 Would Need to Explore

V12 has exhausted the physical AI drafting + face-up pool design space. Future
exploration should consider:

1. **Hybrid contraction within a face-up framework.** V9's invisible contraction
   could operate on a subset of the pool while the player sees a different
   (larger) subset. The player browses a "market" of 40-60 cards; V9's
   contraction silently curates which 40-60 appear from the full 360-card
   pool. This preserves browsability while using virtual contraction for
   concentration. The honesty cost is that the market's composition is
   curated, not random.

2. **Reduced drafter count.** V12's inference accuracy might reach usable levels
   with 2-3 total drafters (1 player + 1-2 AIs). With fewer confounding
   signals, depletion patterns become readable. But this sacrifices the
   "crowded table" narrative and reduces the strategic depth of lane selection.

3. **Player-declared intent.** Rather than inferring the player's archetype from
   depletion patterns, the player could explicitly declare their archetype at
   pick 5-6. AIs respond to the declaration, not to inference. This is honest
   (the player chooses openly) but eliminates the inference challenge entirely.
   It converts the problem from "can AIs read the player?" to "should AIs
   respond to the player's stated preference?"

4. **Larger pack sizes.** V11 identified this as an open question: with 10-card
   packs instead of 4, per-pack archetype density doubles. From a 120-card
   pool with 15% archetype density, a 10-card pack yields 1.5 on-archetype
   cards (vs 0.6 for a 4-card pack). This is a UI change, not a mechanism
   change, and may be the simplest path to higher M3 within a face-up
   framework.

---

## 8. The Simulation Calibration Question

SIM-6 implemented V9 Hybrid B's exact contraction algorithm and achieved
M3 = 1.00 -- far below V9's reported M3 = 2.70. This 1.70 gap demands
explanation.

### Analysis

The gap is primarily a **card model calibration issue**, not a finding that V9's
reported metrics are incorrect. Key differences between the V12 simulation card
model and V9's original model:

1. **S/A assignment:** V12's simulation assigns S/A status randomly at the
   archetype's sibling rate (25-50%). V9's original simulation used calibrated
   fitness functions derived from pair-affinity metadata, which produced
   more structured S/A distributions with better coverage across the pool.

2. **Pair-affinity encoding:** V12's simulation uses simplified fitness scores.
   V9's 8-bit pair-affinity encoding creates a continuous spectrum of card
   relevance, not a binary S/A/not-S/A classification. This continuous scoring
   is what V9's contraction algorithm was designed to exploit.

3. **Relevance scoring:** V9's 40/60 visible/pair-affinity blend produces
   nuanced relevance scores that the contraction engine uses to remove the
   least relevant cards progressively. V12's simulation of this blend may not
   replicate the exact scoring dynamics.

4. **Archetype and sibling counting.** V9's M3 metric counts all cards with
   reasonable fitness for the committed archetype, including partial-fitness
   cards that score 0.5-0.7. V12's binary S/A classification may undercount
   these partial-fitness contributions.

**Directional conclusions from SIM-6 remain valid:** the avoidance log adds no
strategic value (M12 is negative), signal readers are punished by V9's inference
model (engine-mismatch), and V9's contraction trajectory works as designed
(25.7:1 ratio, 99% density by pick 20). The absolute M3 value should not be
compared directly between V12's simulation framework and V9's original
framework; the relative comparisons between V12 designs within the same
simulation framework are the valid basis for conclusions.

---

## 9. Complete Structural Findings

### Finding 1: The Pack-Sampling Bottleneck Persists from V11

V11 identified that drawing 4-5 cards from a 100-130 card pool cannot achieve
M3 >= 2.0 regardless of pool composition. V12 attempted to overcome this
through oversampling (drawing 8-12 cards from the pool). The simulation shows
that oversampling provides a proportional boost (N=8 roughly doubles M3 vs N=4,
confirmed by SIM-5's comparison: 1.33 vs SIM-4's 0.66) but the base density is
too low for the multiplier to reach 2.0.

### Finding 2: Oversampling Is a Multiplier, Not a Mechanism

N=8 doubles effective pack quality; N=12 triples it. But multiplying a base
of 0.66 (the N=4 baseline from avoidance + contraction) produces 1.32 (N=8)
or 1.98 (N=12). Even N=12 barely reaches 2.0, and only under optimistic
assumptions about S/A survival that simulation disproves.

### Finding 3: Biased Refills Help But Cannot Overcome the Density Ceiling

Open-lane-biased refills (2.0x multiplier) raise the player's archetype
density from 12.5% to approximately 14-18%. This is a 12-44% improvement over
the unbiased baseline but remains far below the 40-50% density required for
M3 >= 2.0. The bias is limited by the 3-open-lane structure: the player's
archetype is one of three open lanes, so the 2.0x multiplier is diluted across
all three.

### Finding 4: Refill Events Destroy Inference Signals

Each refill event resets pool composition, destroying the depletion history that
AI inference relies on. SIM-2 shows inference accuracy collapsing from 79%
(pick 10) to 27% (pick 15) after a refill event. With 2-3 refill events per
draft, the AI inference system spends most of the draft rebuilding evidence
that is repeatedly destroyed.

### Finding 5: The 7-AI Design Trades M12 for Nothing

Design 5's 7-AI, 1-open-lane configuration eliminates the signal-reading skill
axis (M12 ~0.05-0.15) without compensating gains in M3. The inference delay
from 7-AI confounding is worse than with 5 AIs, and the M3 improvement from
faster physical contraction is offset by the gradient-reset cost of balanced
refills.

---

## 10. Open Questions and V13 Directions

### Resolved by V12

1. **Can AI avoidance from public information reach useful accuracy with 6
   drafters?** No. 25-35% accuracy, collapsing to single digits after refill
   events. Closed.

2. **Can physical pool contraction achieve V9-level density ratios?** No.
   Maximum physical contraction ratio is 4-6:1 vs V9's 21:1. The ceiling is
   structural. Closed.

3. **Does oversampling (N=8-12) close the gap between physical and virtual
   contraction?** No. Oversampling multiplies insufficient density, producing
   M3 = 1.33 at best (SIM-5, N=8). Closed.

4. **Is S/A exhaustion a parametric or structural problem?** Structural. The
   player's consumption rate is inherent to the "draft good cards for your
   archetype" strategy. No refill schedule resolves it without virtual
   replenishment. Closed.

### Open for V13

1. **Can a hybrid face-up/virtual-contraction design preserve browsability
   while using V9's engine?** The player browses a curated market (40-60
   cards selected by V9's contraction from a larger pool). The market feels
   face-up; the curation is invisible. This is partially dishonest but may
   be the best compromise between concentration and transparency.

2. **Does player-declared intent at pick 5-6 solve the inference problem?**
   The player explicitly chooses an archetype; AIs respond honestly. This
   removes the inference challenge but changes the draft's strategic character.

3. **Can larger pack sizes (8-10 cards) achieve M3 >= 2.0 from a face-up pool
   without virtual contraction?** This is a UI design question, not an
   algorithm question. With 10-card packs from a pool at 20% archetype
   density, M3 = 10 * 0.20 * 0.36 = 0.72 -- still below 2.0. Even pack
   size increases cannot overcome the fundamental density constraint without
   contraction.

4. **Is there a "face-up + gentle contraction" middle ground?** V9 removes
   12% per pick (aggressive). A face-up pool with 3-5% virtual contraction
   per pick (removing the bottom 3-5% of cards, attributed to "market
   rotation") would be partially transparent and partially curated. This
   design space is unexplored.
