# Agent 5 Comparison: Compass Postmortem Perspective

## Scorecard (1-10, all algorithms, by fitness level)

### Optimistic Fitness (Model A)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias 3x | Compass 2+1+1 | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Simple | 8 | 7 | 10 | 7 | 6 | 5 | 10 |
| No actions | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| Not on rails | 8 | 8 | 9 | 8 | 7 | 8 | 9 |
| No forced decks | 7 | 7 | 8 | 7 | 7 | 7 | 8 |
| Flexible | 7 | 7 | 8 | 7 | 5 | 7 | 8 |
| Convergent | 8 | 9 | 2 | 7 | 5 | 8 | 2 |
| Splashable | 7 | 7 | 9 | 7 | 6 | 7 | 9 |
| Open early | 8 | 8 | 9 | 8 | 8 | 8 | 9 |
| Signal reading | 6 | 7 | 3 | 5 | 3 | 6 | 3 |
| **Total** | **69** | **70** | **68** | **66** | **57** | **66** | **68** |

### Moderate Fitness (Model B)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias 3x | Compass 2+1+1 | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Convergent | 4 | 7 | 1 | 4 | 3 | 4 | 1 |
| Signal reading | 4 | 6 | 1 | 4 | 2 | 4 | 1 |
| **Adj Total** | **63** | **66** | **58** | **62** | **51** | **60** | **58** |

### Pessimistic Fitness (Model C)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias 3x | Compass 2+1+1 | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Convergent | 3 | 5 | 1 | 3 | 2 | 3 | 1 |
| Signal reading | 3 | 4 | 1 | 3 | 1 | 3 | 1 |
| **Adj Total** | **59** | **62** | **56** | **59** | **46** | **56** | **56** |

## Biggest Strength and Weakness

| Algorithm | Biggest Strength | Biggest Weakness |
|-----------|-----------------|------------------|
| Surge V6 T=4 | Clean, proven, well-understood | T=4 threshold is too high for moderate fitness |
| Surge+Floor T=3 | 1.85 M3 moderate; floor smooths the entire draft curve | Still fails M3=2.0; floor is a band-aid |
| Asp+Pair | Demonstrates the R2 structural problem definitively | Gate paradox + 3% R2 S/A = comprehensive failure |
| Asp+Bias 3.0x | Bias validated as +0.36 M3 over pure aspiration | Base mechanism too weak; bias cannot fully compensate |
| Compass 2+1+1 | Proves extra R1 slots matter; accidentally a good data point | M9 stddev fails everywhere (0.44-0.78); neighbor slot worthless |
| Dual-Counter | Confirms archetype cost profiles overlap too much for cost-based disambiguation | Effectively identical to Surge V6 |
| Pure Aspiration | Establishes the 2-slot floor as provably insufficient | M3 never exceeds 1.02 |

## Graceful Degradation: Compass Perspective

Compass 2+1+1 degrades from 2.21 (A) to 1.32 (C), retaining 60%. But the Compass failure teaches a broader lesson: **algorithms that waste a slot on a non-functional mechanism degrade worse than algorithms where every slot contributes.** The neighbor slot contributes 1.2% S/A. This means Compass 2+1+1 is functionally "2 R1 + 2 random" -- a 50% targeting rate, between Surge's 75% and Aspiration's 25-50%.

The degradation ranking by "absolute M3 at Model B" (the only metric that matters):

1. **Surge V6 T=3**: 1.88
2. **Surge+Floor T=3**: 1.85
3. **DualCounter T=3**: 1.88
4. **Compass 2+1+1**: 1.66
5. **Asp+Bias 3.0x**: 1.39
6. **Asp+Pair**: 0.84
7. **Pure Aspiration**: 0.72-0.84

All three T=3 Surge variants cluster at 1.85-1.88. The floor variant trades ~0.03 M3 for dramatically better M5 convergence. This is a clear win.

## What Compass Taught Us

1. **R2 neighbor slots are worthless.** Neighbor resonances on the archetype circle do not share archetypes with the player's primary resonance. The "compass" concept of exploring adjacent resonances fails because adjacency on the circle does not equal archetype overlap.

2. **Rotation adds zero value.** Alternating between two neighbors produces the same 1.2% S/A as picking one. Neither neighbor contributes.

3. **The 2+1+1 variant's success comes entirely from the extra R1 slot.** This confirms the general principle: more R1 slots = more S/A. Surge Packs with 3 R1 slots outperforms Compass with 2.

4. **M9 stddev is a Compass-specific failure.** Because every pack has the same structure (2+1+1 always), there is no pack-type variance. Surge Packs' alternation between surge and non-surge packs naturally creates stddev. Any algorithm with always-on pack modification will struggle with M9.

## Minimum Cross-Archetype A-Tier Rate

| Algorithm | Rate for M3=2.0 |
|-----------|:---:|
| Surge+Floor T=3 | ~65% |
| Surge V6 T=3 | ~63% |
| Compass 2+1+1 | ~85% (interpolating from 1.66 at 50%) |
| Asp+Bias 3.0x | Impossible |

Compass would need 85%+ sibling A-tier to reach M3=2.0. This is impractical and confirms Compass should not be considered.

## Proposed Best Algorithm

**Surge Packs + Floor (T=3, S=3, floor_start=3).** I agree with the emerging consensus.

The Compass investigation taught me that targeting the wrong resonance is worse than random. Surge+Floor targets the right resonance (R1) in both surge and floor packs, with no wasted slots. The floor mechanism provides exactly what non-surge packs need: a single guaranteed R1 card that acts as a "minimum viable signal" even when the surge counter has not fired.

Compass could be seen as an "always-on floor with 2 R1 slots" -- and indeed Compass 2+1+1 at 1.66 moderate is close to what you would get from "always-floor with 2 R1 + 2 random." But Surge+Floor beats it because surge packs deliver 3 R1 slots in concentrated bursts, which is better for M5 convergence than a steady 2 R1 per pack.

## Revised M3 Target

**M3 >= 1.8 under Moderate fitness.** Unanimous across agents. The 2.0 target is unachievable under realistic assumptions and should be reframed as a card-design quality gate rather than an algorithm metric.

## Card Designer's Brief

1. **Archetype circle adjacency does not create cross-archetype playability.** Do not assume that archetypes "near" each other on the resonance circle share card synergies. Only primary-resonance siblings matter.
2. **Each primary resonance pair needs 60-70% mutual A-tier fitness.** Design cards to be good in both archetypes of a resonance pair.
3. **Avoid "parasitic" mechanics** that lock a card to exactly one archetype. Cards with "When you sacrifice a creature" are A-tier for Sacrifice but C-tier for Warriors, even though both share Tide. Prefer "When a creature leaves play" which works for both.
4. **The algorithm delivers cards; the designer must ensure those cards are playable.** No amount of algorithmic cleverness compensates for cards that are only playable in their home archetype.
5. **Pack variance requires surge/non-surge alternation.** Always-on pack modification kills M9 stddev. The designer should expect players to see a mix of focused (surge) and exploratory (floor) packs.
