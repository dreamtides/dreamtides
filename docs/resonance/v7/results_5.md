# V7 Agent 5 Results: Archetype Compass Packs

## Algorithm

"Each pack (from pick 2) contains one card from the player's top resonance pool, one from an adjacent resonance on the archetype circle (alternating between the two neighbors each pick), and two random cards."

## Scorecard

| Metric | Target | 1+1+2 (A) | 1+1+2 (B) | 1+1+2 (C) | 2+1+1 (A) | 2+1+1 (B) | 2+1+1 (C) |
|--------|--------|:---------:|:---------:|:---------:|:---------:|:---------:|:---------:|
| M1 Unique archs | >= 3 | 5.38 P | 4.38 P | 3.69 P | 4.98 P | 4.18 P | 3.57 P |
| M2 S/A early | <= 2 | 1.36 P | 1.09 P | 0.90 P | 1.79 P | 1.40 P | 1.17 P |
| M3 S/A late | >= 2.0 | 1.45 F | 1.10 F | 0.87 F | 2.21 P | 1.66 F | 1.32 F |
| M4 Off-arch | >= 0.5 | 0.92 P | 1.06 P | 1.20 P | 0.59 P | 0.80 P | 1.01 P |
| M5 Convergence | 5-8 | 17.1 F | 26.0 F | 28.6 F | 2.4 F | 7.2 P | 15.5 F |
| M6 Concentration | 60-90% | 95.4 F | 79.3 P | 67.0 P | 95.9 F | 89.0 P | 80.1 P |
| M7 Overlap | < 40% | 30.8 P | 28.5 P | 26.0 P | 36.8 P | 36.4 P | 34.5 P |
| M8 Arch freq | 5-20% | 9-20 P | 9-19 P | 10-18 P | 10-15 P | 11-15 P | 10-15 P |
| M9 Stddev | >= 0.8 | 0.60 F | 0.69 F | 0.69 F | 0.44 F | 0.73 F | 0.78 F |
| **Total** | | **5/9** | **6/9** | **6/9** | **6/9** | **7/9** | **6/9** |

## R2 Neighbor Slot Breakdown

The critical finding. The neighbor slot contributes nearly zero S/A.

| Model | S% | A% | B% | C% | S/A% |
|-------|---:|---:|---:|---:|-----:|
| Optimistic | 0.7 | 0.5 | 73.7 | 25.1 | **1.2** |
| Moderate | 1.1 | 0.4 | 73.1 | 25.5 | **1.4** |
| Pessimistic | 1.4 | 0.2 | 72.3 | 26.0 | **1.7** |

The neighbor resonance's cards belong to archetypes that do not share a primary resonance with the player. For Warriors (Tide primary), neighbors are Stone and Zephyr -- their primary cards are Self-Mill/Self-Discard and Flash/Ramp, all B/C-tier for Warriors. The R2 slot provides deck filler, not S/A convergence.

## Fitness Degradation Curve

| Algorithm | Model A | Model B | Model C | A-to-B | A-to-C |
|-----------|--------:|--------:|--------:|-------:|-------:|
| Compass 1+1+2 | 1.45 | 1.10 | 0.87 | -0.36 | -0.58 |
| Compass 2+1+1 | 2.21 | 1.66 | 1.32 | -0.55 | -0.90 |
| Aspiration Packs | 0.96 | 0.77 | 0.65 | -0.19 | -0.30 |

Compass 2+1+1 stays highest at every level but degrades fastest (-0.55 A-to-B) because its extra R1 slot amplifies the sibling-archetype fitness roll. Aspiration degrades least but starts lowest.

## Per-Archetype Convergence (Model B)

| Archetype | Avg Pick | N Converged / ~165 |
|-----------|:--------:|:------------------:|
| Flash | 13.5 | 32 |
| Blink | 13.6 | 15 |
| Storm | 14.4 | 31 |
| Self-Discard | 15.5 | 20 |
| Self-Mill | 11.4 | 18 |
| Sacrifice | 11.7 | 19 |
| Warriors | 14.9 | 15 |
| Ramp | 14.9 | 15 |

Under Moderate fitness, fewer than 25% of drafts ever achieve 3 consecutive packs with 2+ S/A. The 1+1+2 structure provides only 1 effective R1 slot.

## Parameter Sensitivity

**Structure (1+1+2 vs 2+1+1):** 2+1+1 is strictly superior on M3 (+0.56 Model B). It passes M5 under Moderate (7.2). The cost: reduced M4 (0.80 vs 1.06) and lower stddev. The 2+1+1 variant passes 7/9 under Moderate -- the best Compass result.

**Activation pick (2, 3, 4):** Negligible impact. M3 varies by only 0.02 because the R2 slot contributes nothing regardless of activation timing.

## Draft Traces (Model B)

**Committed (Flash):** R1 delivers Zephyr cards (~50% S/A Moderate). R2 alternates Tide/Ember, both distant from Flash. Typical late pack: ~1.1 S/A.

**Signal-reader (Self-Mill):** R1 delivers Stone. R2 alternates Ember/Tide. Tide-neighbor packs occasionally show secondary-resonance cards but B-tier. Typical: ~1.1 S/A.

**Power-chaser:** Ignores fitness, picks highest power. Worst performance.

## Compass vs Aspiration (M4/M8)

| Metric | Model | Compass | Aspiration |
|--------|-------|--------:|-----------:|
| M4 | A | 0.92 | **1.30** |
| M4 | B | 1.06 | **1.34** |
| M8 range | B | 9-19% | **9-16%** |

Aspiration beats Compass on both M4 and M8 at every fitness level. The rotation feature provides no M4/M8 advantage.

## Self-Assessment

**Compass Packs fails its core hypothesis.** The neighbor slot contributes 1.2% S/A -- functionally zero. The reason is structural: neighbor resonances share no archetypes with the player's primary. The R2 slot would need to target the player's *secondary resonance* (what Aspiration does), not an arbitrary circle neighbor.

**The 2+1+1 variant succeeds only through its extra R1 slot**, not the R2 mechanism. It is functionally "2 R1 + 1 wasted + 1 random."

**The rotation feature adds no value.** Alternating between two equally useless neighbors does not produce meaningful variance.

**Recommendation:** Abandon Compass Packs. Algorithms should target R1 or the player's actual secondary resonance. More R1 slots drive M3; the neighbor concept is a dead end.
