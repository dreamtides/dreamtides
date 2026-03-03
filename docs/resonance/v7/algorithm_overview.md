# V7 Algorithm Overview: Complete Catalog

This document catalogs every algorithm proposed, simulated, and evaluated across all 7 agents in the V7 resonance draft investigation, with fitness-model-adjusted results. V7's defining contribution over V6 is testing under realistic card design assumptions.

## Fitness Models

All algorithms tested under three fitness models:

| Model | Sibling A-tier Rate | Resonance-Matched S/A Precision | Purpose |
|-------|:-------------------:|:-------------------------------:|---------|
| A (Optimistic) | 100% | 100% | V6 backward compatibility |
| B (Moderate) | 50% | 75% | Primary realistic target |
| C (Pessimistic) | 25% | 62.5% | Stress test for specialized archetypes |

"Sibling" = the adjacent archetype sharing a primary resonance (e.g., Warriors and Sacrifice both have Tide primary). Under Model B, when the algorithm draws a Tide card for a Warriors player, 50% of the sibling (Sacrifice) cards are A-tier and 50% are B/C-tier.

---

## Surge Variants

### Surge Packs V6 (T=4/S=3) -- Agent 1 Baseline

**One-sentence:** "Each drafted symbol adds tokens (+2 primary, +1 others); when any counter reaches 4, spend 4 and fill 3 of 4 slots with that resonance, fourth random."

| Model | M3 | M5 | M9 | Pass |
|-------|:--:|:--:|:--:|:----:|
| A | 2.03 | 6.2 | 1.42 | 9/9 |
| B | 1.43 | 10.8 | 1.23 | 7/9 |
| C | 1.09 | 17.9 | 1.10 | 6/9 |

**Status: Superseded.** V6 champion. Passes 9/9 under Optimistic but degrades rapidly: M3 drops 0.60 from A to B, M5 explodes to 10.8 (FAIL). T=4 fires too infrequently under realistic fitness to maintain convergence. Replaced by T=3 variants.

### Surge Packs T=3/S=3 -- Agent 1 Parameter Variant

**One-sentence:** Same as V6 but threshold lowered to 3.

| Model | M3 | M5 | M9 | Pass |
|-------|:--:|:--:|:--:|:----:|
| A | 2.66 | 3.9 | ~1.2 | 6/9 |
| B | 1.88 | 5.8 | 1.21 | 8/9 |
| C | 1.49 | 9.1 | ~1.1 | 7/9 |

**Status: Viable.** Highest raw M3 under Moderate (1.88). Fails M5 at Optimistic (3.9, too fast) and M6 at Optimistic (concentration too high). Under Moderate, achieves 8/9 with revised M3 target of 1.8. Slightly worse M5 than Surge+Floor (5.8 vs 5.0).

### Surge Packs + Floor (T=3, S=3, floor_start=3) -- Agent 2 Champion

**One-sentence:** "Each drafted symbol adds tokens (+2 primary, +1 others); when any counter reaches 3, spend 3 and fill 3 of 4 slots with that resonance (4th random); on non-surge packs from pick 3 onward, 1 slot shows top resonance, 3 random."

| Model | M3 | M5 | M9 | Pass |
|-------|:--:|:--:|:--:|:----:|
| A | 2.70 | 5.0 | 1.31 | 9/9 |
| B | 1.85 | 5.0 | 1.15 | 8/9 |
| C | 1.42 | 5.0 | 1.06 | 8/9 |

**Status: RECOMMENDED.** Best overall algorithm. Floor slot fixes M5 convergence (constant 5.0 across all fitness levels). Minimal complexity increase over plain Surge. M3 of 1.85 under Moderate is the highest among algorithms with healthy convergence. Passes 9/9 with revised M3 >= 1.8 target. All 7 agents unanimously endorsed this algorithm.

**Why championed:** The floor mechanism directly addresses the "dead zone" problem on non-surge packs, where V6 Surge delivered fully random packs. One guaranteed R1 card per non-surge pack raises the minimum per-pack S/A from ~0.5 to ~0.75, which is enough to maintain M5 convergence even under degraded fitness.

### Floor+Pair (T=4, S=3) -- Agent 2 Variant

**One-sentence:** "Surge packs use 2 R1 + 1 R2 + 1 random composition; floor packs use 1 R1 + 3 random."

| Model | M3 | M5 | M9 | Pass |
|-------|:--:|:--:|:--:|:----:|
| A | 1.94 | 5.0 | 1.00 | 8/9 |
| B | 1.30 | 5.0 | 0.98 | 8/9 |
| C | 0.98 | 5.0 | 0.90 | 8/9 |

**Status: Eliminated.** Replacing one R1 surge slot with an R2 slot costs ~0.55 M3 under Moderate. The R2 slot delivers B-tier cards (wrong archetypes), not S/A. Strictly worse than Pure Floor at every fitness level.

**Why rejected:** Demonstrates that trading primary-resonance slots for secondary-resonance slots destroys convergence. The disambiguation benefit (targeting R2) is far smaller than the slot-quality cost.

### Surge Packs + Biased Floor (Hypothetical) -- Agent 4 Proposal

**One-sentence:** "Surge+Floor with 2x weight toward top resonance on the 3 random slots of floor packs."

**Not simulated.** Projected M3 ~1.97 under Moderate based on Agent 4's analytical estimate (+0.12 over plain Floor). Would add one parameter (bias weight). Recommended as the first variant to test if playtesting shows 1.85 M3 is insufficient.

**Status: Open hybrid.** Tantalizingly close to 2.0 M3 target but unvalidated by simulation.

### Turbo Surge (T=3/S=4) -- Agent 1 Parameter Variant

**One-sentence:** Same as Surge V6 but T=3 and all 4 slots filled with resonance cards.

| Model | M3 | M5 | M9 | Pass |
|-------|:--:|:--:|:--:|:----:|
| B | 1.88 | 5.4 | ~1.2 | 6/9 |

**Status: Eliminated.** 4/4 targeted slots eliminates splash (M4 fails). M6 concentration too high. 3/4 is the correct surge composition.

---

## Dual-Resonance Targeting

### Aspiration Packs + Pair Preference -- Agent 3 Champion

**One-sentence:** "After each pick, compute top resonance pair (R1, R2); if R2 >= 3 tokens AND R2 >= 50% of R1, one slot shows an R1 card preferring those with R2 symbols, one slot shows an R2 card, two random; otherwise all four random."

| Model | M3 | M5 | M9 | Pass |
|-------|:--:|:--:|:--:|:----:|
| A | 1.02 | 20.3 | 0.81 | 6/8 |
| B | 0.84 | 26.0 | 0.75 | 5/8 |
| C | 0.78 | 27.6 | 0.72 | 5/8 |

**Status: DEAD.** Comprehensive failure. R2 slot delivers 3-4% S/A (predicted 37-50%). Gate paradox: committed players push R1 far above R2, preventing the gate from opening. The algorithm works best for power-chasers who least need it. Pair preference actually reduces R2 quality further by constraining the candidate pool.

**Why rejected:** Three structural failures -- (1) R2 maps to wrong archetypes for S/A, (2) gate punishes commitment, (3) only 2 targeted slots vs Surge's 3.

### Pure Aspiration Packs -- Agent 7 Champion (4 Variants)

**One-sentence (Variant A):** "Compute top resonance pair; if gate opens, 1 R1 slot + 1 R2 slot + 2 random; otherwise all random."

| Variant | M3(A) | M3(B) | M3(C) | Best Pass |
|---------|:-----:|:-----:|:-----:|:---------:|
| A (Pure) | 0.92 | 0.72 | 0.63 | 7/9 |
| B (+Pair Pref) | 1.02 | 0.84 | 0.75 | 7/9 |
| C (+Bias 2x) | 0.97 | 0.77 | 0.67 | 7/9 |
| D (+Floor: 2R1+1R2+1rand) | 0.95 | 0.76 | 0.66 | 7/9 |

**Status: DEAD (all variants).** Establishes the structural lower bound: 2 targeted slots (50% of pack) cannot compete with 3 targeted slots (75%). The R2 slot prediction error was 3-4x (predicted 37-50% S/A, actual 4-17%). The Pair Preference variant is the strongest but still less than half of Surge's M3.

**Structural lesson:** Slot count dominates slot precision. Three R1 slots at 75% S/A beat 1 R1 slot at 100% + 1 R2 slot at 4%.

### Compass Packs -- Agent 5 Champion

**One-sentence:** "Each pack has 1 card from top resonance, 1 from an adjacent resonance on the archetype circle alternating each pick, 2 random."

| Variant | M3(A) | M3(B) | M3(C) | Best Pass |
|---------|:-----:|:-----:|:-----:|:---------:|
| 1+1+2 | 1.45 | 1.10 | 0.87 | 6/9 |
| 2+1+1 | 2.21 | 1.66 | 1.32 | 7/9 |

**Status: DEAD.** Neighbor slot delivers 1.2% S/A -- functionally zero. Neighbor resonances share no archetypes with the player's primary resonance. The 2+1+1 variant succeeds only through its extra R1 slot, not the rotation mechanism. M9 stddev fails at all fitness levels (0.44-0.78) because every pack has identical structure.

**Why rejected:** Neighbor rotation adds zero value. The useful component (extra R1 slot) is already present in Surge Packs at higher concentration.

---

## Supplementary Layers

### Aspiration + Bias 3.0x -- Agent 4 Champion

**One-sentence:** "Aspiration Packs base with 2x or 3x weight toward top resonance on all random slots."

| Bias | M3(A) | M3(B) | M3(C) | Pass(B) |
|------|:-----:|:-----:|:-----:|:-------:|
| 2.0x | 1.48 | 1.11 | 0.91 | 7/9 |
| 3.0x | 1.87 | 1.39 | 1.13 | 7/9 |

**Status: Eliminated (as standalone). Bias validated as component.** The bias layer adds +0.36 M3 over pure Aspiration at 2.0x -- a real and transferable effect. At 3.0x, Asp+Bias nearly matches Surge V6 T=4 under Moderate (1.39 vs 1.43). But the base mechanism (Aspiration) is too weak. Best degradation profile (retains 61% from A to C) but from a lower starting point.

**Why championed then abandoned:** Bias works; the Aspiration base does not. Agent 4 acknowledged bias should augment the winning algorithm (Surge+Floor+Bias hybrid), not stand alone.

### Dual-Counter Surge -- Agent 6 Champion

**One-sentence:** "Surge Packs plus cost-band filtering: surge slots filtered to player's average cost +/-1, widening to +/-2 then unfiltered if insufficient."

| Model | M3 | M5 | M9 | Pass |
|-------|:--:|:--:|:--:|:----:|
| A | 2.02 | 6.5 | 1.43 | 9/9 |
| B | 1.41 | 11.1 | 1.23 | 7/9 |
| C | 1.13 | 16.4 | 1.14 | 6/9 |

At T=3: M3(B)=1.88, M5(B)=5.7, Pass(B)=8/9.

**Status: Eliminated.** Cost filtering provides +0.05 M3 over standard Surge -- indistinguishable from noise. The improvement comes entirely from lowering the threshold to T=3, not from cost filtering. Home archetype selection improves from 50% to 53% -- imperceptible.

**Why rejected:** Archetype cost profiles overlap too much (Warriors 3.7 vs Sacrifice 3.0 = 0.7 separation). Running average is noisy. Fallback widening defeats the filter. Cost is orthogonal to archetype fitness.

---

## Other Mechanisms

### Phase-Shift (from V7 design discussions)

Concept: Alternate between "exploration phases" (all random) and "commitment phases" (targeted packs) based on pick number rather than token accumulation.

**Status: Not simulated.** Recognized during discussion as a less adaptive version of Surge Packs. Fixed phase timing cannot respond to player behavior; token-based surge timing naturally adapts.

### Echo Accumulator (V6 Agent 7 proposal)

**One-sentence:** "Earn resonance points; at 5, reset to 0 and inject one resonance-matched slot into the next pack."

**Status: Eliminated in V6.** Single injection every 2 picks yields ~1.25 S/A. Insufficient slot count.

### Negative Sculpting (V7 discussion concept)

Concept: Instead of adding targeted cards, remove off-resonance cards from the draw pool over time.

**Status: Not simulated.** Recognized as equivalent to Pool Sculpting (V6 Agent 4), which confirmed the probabilistic ceiling at ~2.0 S/A. Pool manipulation cannot reliably cross 2.0 because draws remain probabilistic.

### Momentum Injection (V6 Agent 7 proposal)

**One-sentence:** "If your current pick shares a primary resonance with your previous pick, one slot in the next pack shows a resonance-matched card."

**Status: Eliminated in V6.** Binary on/off provides only 1 guaranteed slot at ~0.5 S/A.

### Resonance Gravity (V6 Agent 7 proposal)

**One-sentence:** "Each slot draws from a weighted pool: 3x for top resonance, 2x for second, with anti-monotony rerolls."

**Status: Eliminated in V6.** Probabilistic with weight boosting. Estimated ~1.6-1.8 S/A. Anti-monotony reroll further reduces convergence.

---

## V7 Structural Findings

1. **No zero-decision algorithm reaches M3 >= 2.0 under Moderate fitness (50% sibling A-tier).** This is a hard structural limit of resonance-level targeting, not a failure of any specific algorithm.

2. **The R2 slot is structurally worthless for S/A.** Secondary-resonance slots deliver 3-17% S/A because R2's primary pool contains cards from archetypes that do not share primary resonance with the player. Predicted 37-50%; actual 3-17%. The prediction error was 3-4x.

3. **Cost-based disambiguation provides +0.05 S/A.** Archetype cost profiles overlap too much. Cost is orthogonal to archetype fitness.

4. **Threshold T=3 dominates T=4 under realistic fitness.** More frequent surges compensate for reduced per-slot precision. T=3 fires ~28 surges per draft vs T=4's ~21.

5. **Slot count dominates slot precision.** Three R1 slots at 75% S/A (Surge) beat 1 R1 + 1 R2 at any precision (Aspiration). Each additional primary-resonance slot adds ~0.5 M3 under Optimistic fitness.

6. **The floor mechanism fixes M5 convergence.** Adding 1 guaranteed R1 card to non-surge packs raises the minimum per-pack S/A, keeping all 8 archetypes within the 5-8 convergence window.

7. **All degradation is structural to resonance-level targeting.** Every algorithm's M3 is linearly proportional to sibling A-tier rate. The algorithm cannot compensate for weak card design.

8. **Permanent locks are strictly inferior to non-permanent surges.** Lane Locking (V3/V6) achieves higher raw S/A but fails variance, concentration, and timing. Surge's non-permanent tracking allows pivoting.

9. **Gate conditions that punish commitment are fatal.** Aspiration's R2 >= 50% * R1 gate closes for committed drafters whose R1 grows much faster than R2.

10. **Variance requires state alternation.** Always-on pack modification (Compass, Aspiration) kills M9 stddev. Surge/non-surge alternation naturally creates pack-type variance.

## Mechanism Class Ceiling Analysis

| Class | Best Algorithm | M3(A) | M3(B) | M3(C) | Theoretical Ceiling | Binding Constraint |
|-------|---------------|:-----:|:-----:|:-----:|:-------------------:|-------------------|
| Slot-Filling Surge | Surge+Floor T=3 | 2.70 | 1.85 | 1.42 | ~3.0 (4/4 slots, no floor) | M4 splash; M6 concentration |
| Additive Injection | Double Enhancement T=1 (V6) | 2.13 | ~1.5 | ~1.1 | ~2.5 | Variable pack size; trigger rate |
| Deterministic Placement | Lane Locking (V3/V6) | 2.22 | ~1.6 | ~1.2 | Unlimited | M5 timing; M9 variance; M6 concentration |
| Dual-Resonance Targeting | Asp+PairPref | 1.02 | 0.84 | 0.78 | ~1.3 | R2 structural S/A limit (3-17%) |
| Probabilistic Weighting | Asp+Bias 3.0x | 1.87 | 1.39 | 1.13 | ~2.0 | Pool Sculpting ceiling confirmed in V6 |
| Secondary Signal (Cost) | DualCounter T=3 | 2.02 | 1.88 | 1.48 | = Surge (cost adds nothing) | Cost profiles overlap |

Slot-Filling Surge has the highest practical ceiling because it combines ADD-mechanism power (placing targeted cards directly) with non-permanent state (preserving variance and flexibility). The only mechanism class with higher raw S/A potential (Deterministic Placement) fails too many variance and timing metrics to be viable.

## Cross-Agent Consensus Points

1. **All 7 agents** recommend Surge+Floor T=3 as the V7 winner.
2. **All 7 agents** agree M3 should be revised to >= 1.8 under Moderate fitness.
3. **All 7 agents** agree that R2/secondary-resonance slots are structurally worthless for S/A convergence.
4. **All 7 agents** agree that cost-based disambiguation does not justify its complexity.
5. **All 7 agents** agree that sibling A-tier rate is the binding constraint and must be >= 50% for any algorithm to function.
6. **6 of 7 agents** agree that the Surge+Floor+Bias hybrid (~1.97 projected) deserves future simulation if 1.85 proves insufficient.
7. **All 7 agents** agree that the gap between 1.85 (achievable) and 2.0 (target) is a card design problem, not an algorithm problem.

## Final Recommendation

**Surge Packs + Floor (T=3, S=3, floor_start=3).**

**One-sentence (player-facing):** "As you draft, the game tracks which resonance types you're collecting; every few picks you'll get a focused pack of 3 cards matching your top resonance, and in between, one card in each pack always matches your strongest resonance."

**One-sentence (technical):** "Each drafted symbol adds tokens (+2 primary, +1 others); when any counter reaches 3, spend 3 and fill 3 of 4 slots with that resonance (4th random); on non-surge packs from pick 3 onward, 1 slot shows top resonance, 3 random."

**Card designer brief:** Target 50-65% sibling A-tier rate. Each resonance pair's two archetypes should share complementary mechanics where most cards work in both decks. The algorithm delivers cards; the designer ensures those cards are playable. Every 10% improvement in sibling A-tier yields ~+0.17 M3. At 65%, the algorithm crosses M3=2.0.
