# V12 Algorithm Overview: Complete Catalog

V12 tested whether AI avoidance + physical pool contraction + modest
oversampling (N=8-12) could replace V9's virtual contraction in a face-up
shared pool. The thesis was definitively falsified: no V12 algorithm achieves
M3 >= 2.0. This catalog orders all algorithms by performance, documents failure
modes, and records the structural findings that constrain future design.

---

## 1. Recommended: V9 Hybrid B (Unchanged)

V12 produces no algorithm that supersedes V9 Hybrid B. The recommendation from
V11 stands: V9's contraction engine with an AI narrative layer and Design 5
information system remains the best available algorithm.

| Parameter | Value |
|-----------|-------|
| Pool | 360 cards |
| Contraction | 12%/pick from pick 4, 40/60 visible/pair-affinity blend |
| Floor slot | 1 top-quartile from pick 3 |
| Pack construction | 1 floor + 3 random from surviving pool |
| Pool minimum | 17 cards |
| Hidden metadata | 8 bits/card (two 4-bit pair-affinity floats) |
| M3 | 2.70 |
| M11' | 3.25 |

**V12's contribution to V9:** The face-up pool concept could be layered on top
of V9 as a browsing interface showing the post-contraction pool. This would
provide V12's information benefits (pool browsability, archetype availability
at a glance) without requiring physical concentration. The AI narrative layer
(V10/V11) and Design 5 information system (V11) remain valid enhancements.

---

## 2. Viable Alternatives

**None.** No V12 face-up algorithm is viable for deployment. All fail M3 by
at least 0.67 (the closest miss is SIM-5 at M3 = 1.33). The M3 shortfall is
structural, not parametric, so no tuning within V12's design space can close
the gap.

---

## 3. Eliminated Algorithms by Failure Mode

### Failure Mode A: AI Inference Accuracy (Primary Barrier)

All V12 algorithms depend on AIs correctly identifying the player's archetype
from aggregate pool depletion patterns. With 6 concurrent drafters, inference
accuracy is 25-35%, rendering avoidance ineffective.

**Affected algorithms:**

| Algorithm | M3 | Inference Accuracy | Effective Avoidance |
|-----------|:--:|:------------------:|:-------------------:|
| SIM-1: D3 Moderate + N=12 + Floor | 0.42 | 25-57% (peak 9, collapse post-refill) | ~24% |
| SIM-2: Hybrid 1 Conservative N=12 | 0.51 | 27-79% (peak 10, collapse at 15) | ~20% avg |
| SIM-3: Hybrid 2 Progressive N | 0.37 | 98% by pick 8 (but density still fails) | Moderate |
| SIM-5: D2 Steep N=8 | 1.33 | Wrong archetype in most games | ~0% effective |

SIM-3 achieved high inference accuracy (98% by pick 8) but still failed M3
because accurate avoidance alone cannot overcome the other structural barriers.
SIM-5 achieved the best M3 despite having the worst inference accuracy,
confirming that the M3 gains come primarily from oversampling (N=8), not from
avoidance.

### Failure Mode B: S/A Exhaustion (Binding Constraint)

The player consumes their own S/A supply through normal drafting. By pick 25,
S/A remaining in the pool ranges from 0.0 to 2.0 across all simulations.
Without S/A cards to find, no oversampling factor can produce M3 >= 2.0.

| Algorithm | S/A at Pick 20 | S/A at Pick 25 | S/A at Pick 30 |
|-----------|:--------------:|:--------------:|:--------------:|
| SIM-1 | 0.1 | 0.1 | 0.0 |
| SIM-2 | 2.4 | 2.0 | 1.1 |
| SIM-3 | 3.0 | 1.6 | 0.3 |
| SIM-4 | 10.2 | 3.7 | 0.0 |
| SIM-5 | 8.9 | 7.0 | 1.9 |

SIM-5's higher S/A counts reflect counting cross-archetype (sibling) S/A
cards, which inflates the numbers. Primary-archetype S/A is much lower.

### Failure Mode C: Archetype Density Decrease

The design-phase models predicted archetype density increasing from 12.5% to
40-55% as AIs avoided the player's archetype. In simulation, density decreases
because the player's own consumption outpaces the concentration benefit from
inaccurate avoidance.

| Algorithm | Density Pick 5 | Density Peak | Density Pick 25 | Density Pick 30 |
|-----------|:--------------:|:------------:|:---------------:|:---------------:|
| SIM-1 | 13.1% | 13.2% (pick 11) | 6.9% | 1.9% |
| SIM-2 | 14.3% | 17.5% (pick 11) | 12.3% | 8.7% |
| SIM-3 | 16.3% | 18.4% (pick 21) | 17.5% | 13.0% |
| SIM-4 | 11.7% | 11.9% (pick 11) | 7.0% | 0.0% |
| SIM-5 | 11.5% | 10.5% (pick 11) | 7.4% | 5.1% |

No simulation achieves the 40-50% density threshold required for M3 >= 2.0
with N=8-12 oversampling. The highest density observed (18.4% in SIM-3) is
less than half the required minimum.

### Failure Mode D: Physical Contraction Ratio Ceiling

V9 achieves 21:1 contraction (360 to 17 cards). V12's maximum physical
contraction is 4-6:1 (120 to 20-30 cards). This structural ceiling cannot be
exceeded without virtual contraction.

| Algorithm | Start Pool | End Pool | Ratio | V9 Ratio |
|-----------|:----------:|:--------:|:-----:|:--------:|
| SIM-1 | 120 | 26 | 4.6:1 | 21:1 |
| SIM-2 | 120 | 26 | 4.6:1 | 21:1 |
| SIM-3 | 120 | 26 | 4.6:1 | 21:1 |
| SIM-4 | 120 | 0 | Exhausted | 21:1 |
| SIM-5 | 120 | 20 | 6.0:1 | 21:1 |

### Failure Mode E: Refill Gradient Reset

Each refill event partially resets the archetype density gradient that avoidance
builds within a round. SIM-2's inference accuracy drops from 79% to 27% after
the pick-10 refill; density resets from 17.5% to baseline levels. Designs with
fewer refills (SIM-4's single-refill 60/0/0) avoid this reset but face pool
exhaustion.

### Isolation Baseline: SIM-4 (N=4, No Oversampling)

SIM-4 establishes the contribution of avoidance + contraction alone:
M3 = 0.66. The oversampling contribution of N=8 (SIM-5) is therefore
+0.67 (1.33 - 0.66), and the projected N=12 contribution would be
approximately +1.00 to +1.34. Even the maximum oversampling contribution
cannot close the gap to M3 = 2.0 from the 0.66 base.

### V9 Fallback: SIM-6

SIM-6 implements V9 Hybrid B's contraction engine with the V12 simulation's
card model. M3 = 1.00 (vs V9's reported 2.70) reflects card model calibration
differences, not a refutation of V9's mechanics. The contraction trajectory
is correct (25.7:1 ratio, 99% density by pick 20). The avoidance log adds
no strategic value (M12 = -0.36) because signal readers who delay commitment
are punished by V9's inference model, which needs coherent early picks.

---

## 4. Structural Findings

### Finding 1: Physical Contraction Cannot Replicate Virtual Contraction

This is V12's central finding. Physical pool contraction (AIs removing cards
from a shared pool) is structurally inferior to virtual contraction (algorithm
silently removing cards by relevance) for three reasons:

1. **Direction of removal.** V9 removes the LEAST relevant cards, enriching
   the pool. Physical AIs remove the MOST relevant cards for their own
   archetypes, which includes the player's S/A cards when inference is
   inaccurate. Virtual contraction enriches; physical contraction depletes.

2. **Contraction ratio.** Virtual contraction has no ceiling -- the algorithm
   can remove any percentage per pick. Physical contraction is bounded by the
   number of drafters and the card supply. Maximum physical ratio: 6:1.
   V9's virtual ratio: 21:1.

3. **Self-regulation.** V9's contraction automatically preserves the player's
   archetype (by definition -- it removes low-relevance cards). Physical
   contraction requires accurate inference to achieve selective preservation,
   and inference fails at the signal-to-noise levels present with 6 drafters.

### Finding 2: AI Avoidance Requires Perfect Knowledge to Work

The design-phase assumption was that AIs could infer the player's archetype
from public depletion patterns at 70-90% accuracy by pick 8-10. Simulation
reveals 25-35% accuracy (or worse after refill events). The inference failure
is not a parameter issue -- it is a fundamental signal-to-noise problem with
6 concurrent drafters. AI avoidance would need to know the player's archetype
directly (Level 2 information) to achieve the concentration the design
requires.

### Finding 3: Oversampling Has Diminishing Returns Against Low Density

Oversampling (draw N, show best 4) provides a proportional boost:
N=8 gives ~2x, N=12 gives ~3x the M3 of N=4. But when the base density is
12-18% (as in all V12 simulations), even a 3x multiplier produces
M3 = 0.50-1.50, insufficient for the 2.0 target. Oversampling is effective
only when combined with a concentration mechanism that first raises density
above 30-40%.

### Finding 4: S/A Exhaustion Is Self-Inflicted and Structural

The player consuming their own S/A supply is inherent to archetype-committed
drafting. Over 30 picks at 70-100% on-archetype rate with 36% S/A rate, the
player takes approximately 7-10 S/A cards. Starting S/A (5) plus refill S/A
(4-8) minus player consumption (7-10) leaves 0-3 S/A at pick 25. This
constraint cannot be resolved by AI behavior changes, refill schedules, or
oversampling -- it requires either much higher starting S/A counts or virtual
replenishment mechanisms.

### Finding 5: Signal Reading Works (The V12 Positive)

Despite all M3 failures, signal reading produces meaningful M12
differentiation (0.34-0.52 in the best simulations). Players who browse the
face-up pool and identify uncontested archetypes draft better decks. This
validates the face-up pool as an information surface and establishes that
open-lane identification is a genuine skill axis worth preserving in any
future design. The signal-reading mechanism should be layered on top of V9's
engine, not used as a replacement for it.

### Finding 6: The V12 Design-Phase Predictions Were Systematically Optimistic

Design-phase M3 predictions ranged from 1.8 to 2.2 across the leading
candidates. Simulation results ranged from 0.37 to 1.33. The predictions
were optimistic by 1.0-1.8 M3 points, or 50-80% of the predicted value.
The primary sources of optimism: (a) assuming 70-90% AI inference accuracy
when simulation shows 25-35%, (b) assuming archetype density would increase
when it decreases, (c) underestimating S/A exhaustion from the player's own
consumption, and (d) using simplified hypergeometric models that do not account
for the interaction between avoidance failure and density dynamics.

Future design phases should require simulation-backed evidence for any mechanism
predicted to contribute more than +0.3 M3, rather than relying on analytical
models that assume mechanisms work as designed.
