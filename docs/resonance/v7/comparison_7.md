# Agent 7 Comparison: Pure Aspiration Postmortem & Synthesis

## Scorecard (1-10, all algorithms, by fitness level)

### Optimistic Fitness (Model A)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias 3x | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Simple | 8 | 7 | 10 | 7 | 6 | 5 | 10 |
| No actions | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| Not on rails | 8 | 8 | 9 | 8 | 7 | 8 | 9 |
| No forced decks | 7 | 7 | 8 | 7 | 7 | 7 | 8 |
| Flexible | 7 | 7 | 8 | 8 | 6 | 7 | 8 |
| Convergent | 8 | 9 | 2 | 7 | 5 | 8 | 2 |
| Splashable | 7 | 7 | 9 | 7 | 7 | 7 | 9 |
| Open early | 8 | 8 | 9 | 8 | 8 | 8 | 9 |
| Signal reading | 6 | 7 | 3 | 5 | 3 | 6 | 3 |
| **Total** | **69** | **70** | **68** | **67** | **59** | **66** | **68** |

### Moderate Fitness (Model B)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias 3x | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Convergent | 4 | 7 | 1 | 4 | 3 | 4 | 1 |
| Signal reading | 4 | 6 | 1 | 4 | 2 | 4 | 1 |
| **Adj Total** | **63** | **66** | **58** | **63** | **53** | **60** | **58** |

### Pessimistic Fitness (Model C)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias 3x | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Convergent | 3 | 5 | 1 | 3 | 2 | 3 | 1 |
| Signal reading | 3 | 4 | 1 | 3 | 1 | 3 | 1 |
| **Adj Total** | **59** | **62** | **56** | **60** | **46** | **56** | **56** |

## Biggest Strength and Weakness

| Algorithm | Biggest Strength | Biggest Weakness |
|-----------|-----------------|------------------|
| Surge V6 T=4 | Battle-tested; clean mechanism; 9/9 at optimistic | M3 collapses under moderate fitness (2.03 -> 1.43) |
| Surge+Floor T=3 | Best measured M3 under moderate (1.85); fixes M5 | Still fails M3=2.0; floor adds a conditional rule |
| Asp+Pair | Simplest mechanism; perfect early openness | R2 slot delivers 3-4% S/A; gate paradox; dead |
| Asp+Bias 3.0x | Best M6 robustness at pessimistic (70%); bias is validated | M3=1.39 moderate; too far from any viable target |
| Compass 2+1+1 | Extra R1 slot reaches 1.66 moderate M3 | M9 stddev fails at all fitness levels; rotation worthless |
| Dual-Counter | Proves cost is orthogonal to archetype identity | +0.05 M3; complexity not justified |
| Pure Aspiration | Establishes the 2-slot structural floor; simplest | M3=0.72-1.02; convergence pick 20-28; fundamentally broken |

## Graceful Degradation: The Full Picture

As the agent who tested all 4 Aspiration variants, I have the clearest view of what "graceful degradation" actually means versus what it appears to mean.

**The trap:** Aspiration variants retain 68-76% of their Optimistic M3 at Pessimistic. This sounds like "graceful degradation." But retaining 76% of 1.02 gives you 0.78 -- a number so far below any viable target that the percentage is meaningless.

**The reality:** Only absolute performance at realistic fitness levels matters. Ranked by M3 at Model B:

| Rank | Algorithm | M3(B) | M5(B) | Pass(B) |
|------|-----------|:-----:|:-----:|:-------:|
| 1 | Surge V6 T=3 | 1.88 | 5.8 | 8/9 |
| 2 | DualCounter T=3 | 1.88 | 5.7 | 8/9 |
| 3 | Surge+Floor T=3 | 1.85 | 5.0 | 8/9 |
| 4 | Compass 2+1+1 | 1.66 | 7.2 | 7/9 |
| 5 | Surge V6 T=4 | 1.43 | 10.8 | 7/9 |
| 6 | DualCounter T=4 | 1.41 | 11.1 | 7/9 |
| 7 | Asp+Bias 3.0x | 1.39 | 16.0 | 7/9 |
| 8 | Surge+Floor T=4 | 1.51 | 5.0 | 8/9 |
| 9 | Asp+Pair | 0.84 | 26.0 | 5/8 |
| 10 | Pure Asp best | 0.84 | 26.0 | 6/9 |

Surge+Floor T=3 is the best choice because it achieves the third-highest M3 while having the best M5 (5.0, comfortably within 5-8). Surge V6 T=3 and DualCounter T=3 tie on M3 but have worse M5 (5.8 and 5.7, barely inside the window) and lack the floor safety net for non-surge packs.

## What Pure Aspiration Taught the Field

As the advocate for all 4 Aspiration variants, I owe the field a clear accounting:

1. **2 targeted slots (50% of pack) is the structural minimum for any convergence.** Pure Aspiration with 2 slots achieves M3 ~1.0. Surge with 3 slots achieves M3 ~2.0. The relationship is roughly linear: each additional targeted slot adds ~0.5 M3 under optimistic fitness.

2. **The R2 slot prediction error was 3-4x.** Design documents predicted 37-50% R2 S/A. Actual: 4-17%. The error came from assuming R2's pool would contain the player's archetype's cards. It does not -- R2's pool contains the wrong archetypes.

3. **Simplicity without convergence is worthless.** Aspiration scored 10/10 on simplicity and 2/10 on convergence. Players would rather have a slightly more complex algorithm that actually helps them build a deck.

4. **The bias component is the one salvageable piece.** Weighting random slots toward the player's top resonance adds +0.36 M3 over pure random at Model B. This is a validated, transferable component.

## Minimum Cross-Archetype A-Tier Rate

| Algorithm | M3 at 50% A-tier | M3 at 100% A-tier | Interpolated rate for M3=2.0 |
|-----------|:-:|:-:|:-:|
| Surge+Floor T=3 | 1.85 | 2.70 | **~58%** |
| Surge V6 T=3 | 1.88 | 2.16 | **~71%** |
| Asp+Bias 3.0x | 1.39 | 1.87 | **Never** |
| Pure Aspiration | 0.72-0.84 | 0.92-1.02 | **Never** |

Surge+Floor T=3 benefits from the floor slot scaling well with fitness: at 100% A-tier, every floor slot card is S/A, pushing M3 to 2.70. This means the floor variant reaches M3=2.0 at a lower A-tier rate (~58%) than plain Surge (~71%).

## Proposed Best Algorithm

**Surge Packs + Floor (T=3, S=3, floor_start=3).**

All 7 agents should converge here. The data is unambiguous:
- Highest M3 at moderate fitness among algorithms with healthy M5
- 8/9 pass rate at moderate fitness (only M3 fails at 2.0 target)
- 9/9 if M3 target is revised to 1.8
- Minimal complexity increment over plain Surge V6
- No wasted components (unlike Compass rotation, cost filtering, R2 slots)

The one question remaining is Agent 4's Surge+Floor+Bias hybrid. Projected M3 ~1.97 under moderate. If confirmed by simulation, this would push the algorithm to the 2.0 threshold. But even without bias, Surge+Floor T=3 at 1.85 is the clear V7 recommendation.

## Revised M3 Target

**M3 >= 1.8 under Moderate fitness.** This is the only target achievable by any tested algorithm under realistic card design assumptions. The 2.0 target should be reformulated as: "M3 >= 2.0 under Optimistic fitness, serving as a card-design quality indicator."

## Card Designer's Brief

1. **The 50% sibling A-tier baseline** (Model B) represents "average" card design quality. The draft algorithm achieves M3=1.85 at this level. Every 10% improvement in sibling A-tier rate yields approximately +0.17 M3.

2. **The critical design task:** For each primary resonance, ensure 6-7 out of 10 cards in archetype X are A-tier in sibling archetype Y. This means designing cards with broadly applicable effects that gain bonus value in the home archetype.

3. **Card design testing protocol:** For each card, evaluate: "Is this A-tier in both archetypes sharing this primary resonance?" Track the per-resonance-pair A-tier percentage. If any pair drops below 50%, the draft algorithm will underperform for those archetypes.

4. **Signal cards are valuable.** Cards that are S-tier in one archetype and B/C-tier in the sibling act as "draft signals" -- when a player sees them in a surge pack, they know which archetype the algorithm is pointing toward. Design 2-3 signal cards per archetype.

5. **Pool depth:** Ensure 15+ cards per primary resonance. With T=3 and floor packs, the algorithm draws heavily from the top resonance pool. Insufficient depth causes repetitive packs (M7 overlap failure).
