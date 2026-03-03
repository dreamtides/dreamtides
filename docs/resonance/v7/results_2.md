# V7 Agent 2 Results: Surge + Floor

## Algorithm Descriptions

**Pure Floor:** "Each drafted symbol adds tokens (+2 primary, +1 others); when any counter reaches T, spend T and fill 3 of 4 slots with that resonance; on non-surge packs (from pick 3+), 1 slot always shows the player's top resonance, 3 random."

**Floor+Pair:** Same tokens; surge packs use 2 R1 + 1 R2 + 1 random; floor packs use 1 R1 + 3 random.

## Scorecard (committed strategy, T=4, floor_start=3)

| Metric | Target | PFloor A | PFloor B | PFloor C | F+Pair A | F+Pair B | F+Pair C |
|--------|--------|----------|----------|----------|----------|----------|----------|
| M1 EarlyVar | >=3 | 4.43 | 3.76 | 3.34 | 4.83 | 4.05 | 3.55 |
| M2 EarlySA | <=2 | 1.39 | 1.05 | 0.89 | 1.29 | 0.95 | 0.84 |
| M3 PostSA | >=2 | **2.25** | **1.51 F** | **1.14 F** | **1.94 F** | **1.30 F** | **0.98 F** |
| M4 PostCF | >=0.5 | 0.63 | 0.88 | 1.08 | 0.83 | 1.04 | 1.21 |
| M5 Conv | 5-8 | 5.0 | 5.0 | 5.0 | 5.0 | 5.0 | 5.0 |
| M6 Conc% | 60-90 | 79.0 | 69.7 | 62.0 | 83.9 | 71.8 | 62.2 |
| M7 Overlap | <40% | 6.2 | 5.7 | 6.4 | 5.8 | 5.8 | 5.9 |
| M8 Freq | <20/>5 | 12.5/12.5 | 12.5/12.5 | 12.5/12.5 | 12.5/12.5 | 12.5/12.5 | 12.5/12.5 |
| M9 StdDev | >=0.8 | 1.31 | 1.18 | 1.06 | 1.00 | 0.98 | 0.90 |
| **Pass** | 9/9 | **9/9** | **8/9** | **8/9** | **8/9** | **8/9** | **8/9** |

Only Pure Floor under Optimistic achieves 9/9. All other configs fail M3 exclusively.

## Fitness Degradation Curve

| Variant | Fit | M3 S/A | M6 Conc | M9 Std | Delta |
|---------|-----|--------|---------|--------|-------|
| PureFloor | A | 2.25 | 79.0 | 1.31 | -- |
| PureFloor | B | 1.51 | 69.7 | 1.18 | -0.74 |
| PureFloor | C | 1.14 | 62.0 | 1.06 | -1.11 |
| Floor+Pair | A | 1.94 | 83.9 | 1.00 | -- |
| Floor+Pair | B | 1.30 | 71.8 | 0.98 | -0.64 |
| Floor+Pair | C | 0.98 | 62.2 | 0.90 | -0.96 |

Degradation is front-loaded: A-to-B drops 0.74 S/A because sibling A-tier rate halves from 100% to 50%. Pure Floor outperforms Floor+Pair at every fitness level -- the pair composition trades a primary-resonance surge slot for a secondary slot that delivers B-tier or worse. The disambiguation benefit is smaller than the slot-quality cost.

## Per-Archetype Convergence (pick for first 2+ S/A pack, picks 6+)

| Archetype | PFloor A | PFloor B | PFloor C |
|-----------|----------|----------|----------|
| Flash | 7.4 | 7.4 | 7.8 |
| Blink | 7.4 | 7.7 | 8.2 |
| Storm | 6.6 | 7.6 | 8.2 |
| SelfDiscard | 7.5 | 7.9 | 8.2 |
| SelfMill | 6.9 | 7.7 | 8.4 |
| Sacrifice | 7.8 | 7.9 | 9.2 |
| Warriors | 7.0 | 7.8 | 8.3 |
| Ramp | 6.5 | 8.0 | 8.3 |

Under Moderate, all archetypes converge within 7.4-8.0 (within target). Under Pessimistic, Sacrifice pushes to 9.2, outside the 5-8 window.

## Parameter Sensitivity

**Threshold (Pure Floor, committed):**

| T | M3(A) | M3(B) | M3(C) | M6(B) | M9(B) | Surges |
|---|-------|-------|-------|-------|-------|--------|
| 3 | 2.70 | 1.85 | 1.42 | 77.6 | 1.15 | 28.2 |
| 4 | 2.25 | 1.51 | 1.14 | 69.7 | 1.18 | 21.0 |
| 5 | 2.00 | 1.34 | 1.02 | 67.0 | 1.14 | 16.4 |

T=3 achieves the highest Moderate S/A (1.85) with 28 surges/draft. Concentration rises to 77.6-83.1% but remains within bounds.

**Floor activation pick (T=4, committed):**

| Start | M2(A) | M3(A) | M3(B) |
|-------|-------|-------|-------|
| 2 | 1.56 | 2.29 | 1.51 |
| 3 | 1.39 | 2.25 | 1.51 |
| 4 | 1.40 | 2.21 | 1.49 |

Minimal impact. Pick 3 is ideal for early-openness preservation.

## Draft Traces (Fitness B)

**Trace 1 -- Committed Warriors (Pure Floor):** Committed pick 5. Tide surges alternate with floor packs. Surge packs deliver Warriors(S) and Sacrifice(A) Tide cards. Avg S/A 6+: 1.92. Deck: 23/30 S/A (77%). Near top of distribution due to consistent Tide drafting.

**Trace 2 -- Signal Reader (Floor+Pair):** Committed SelfDiscard pick 5 after Stone token accumulation. Surge packs delivered Stone+Ember pairs but SelfDiscard cards rated F-tier for the signal reader's eventual archetype choice. Avg S/A 6+: 1.00. Deck: 22/30 S/A (73%). Illustrates Floor+Pair weakness: pair slot amplifies misalignment.

**Trace 3 -- Power Chaser (Pure Floor):** Never commits (pick 30). Picks raw power, ignoring resonance. Chaotic token pattern. Avg S/A 6+: 1.04. Deck: 8/30 S/A (27%). Power chasers are incompatible with resonance-tracking algorithms.

## Comparison to Agent 1 Baselines

Agent 1 estimated V6 Surge at 2.05 (A), 1.60 (B), 1.38 (C).

| Algorithm | Fit A | Fit B | Fit C |
|-----------|-------|-------|-------|
| V6 Surge (est.) | 2.05 | 1.60 | 1.38 |
| Pure Floor T=4 | 2.25 | 1.51 | 1.14 |
| Pure Floor T=3 | 2.70 | 1.85 | 1.42 |
| Floor+Pair T=4 | 1.94 | 1.30 | 0.98 |

Pure Floor T=4 outperforms V6 under Optimistic (+0.20) but slightly underperforms under Moderate (-0.09). The floor slot is subject to the same fitness degradation as surge slots, limiting its compensatory power.

Pure Floor T=3 outperforms V6 at all fitness levels (+0.65/+0.25/+0.04). The turbo threshold with floor is the strongest variant measured.

Floor+Pair underperforms V6 at all levels. Strictly worse.

## Self-Assessment

**Strengths:** Pure Floor T=3 achieves the highest Moderate S/A of any Surge variant (1.85). All non-M3 metrics pass at every fitness level. Floor mechanism adds minimal complexity. Pack-type variance (surge/floor/normal) maintains M9 stddev above 1.0.

**Weaknesses:** M3 still fails at 1.85 under Moderate (target 2.0). Floor+Pair hybrid is strictly worse -- disambiguation hypothesis failed. No configuration achieves 2.0 under realistic fitness.

**Recommendation:** If M3 is relaxed to 1.8 for realistic fitness assessment, Pure Floor T=3 with floor_start=3 passes all 9 metrics. The 2.0 target requires either 80%+ cross-archetype A-tier card design or a mechanism beyond resonance-level targeting.
