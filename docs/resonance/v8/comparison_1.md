# Comparison Agent 1: Baselines Perspective

## Scorecard at Each Fitness Level (Algorithm x M3)

| Algorithm                 | Optimistic | Grad. Real. (40%) | Pessimistic | Hostile |
| ------------------------- | :--------: | :---------------: | :---------: | :-----: |
| 1. Baselines (Pair-Esc.)  |    2.34    |     **2.16**      |    2.12     |  2.08   |
| 2. Continuous Surge       |    3.10    |     **2.48**      |    2.43     |  2.25   |
| 3. Escalating Pair Lock   |    1.98    |     **1.50**      |    1.46     |  1.31   |
| 4. GPE-45                 |    2.73    |     **2.25**      |    2.21     |  2.05   |
| 5. Symbol-Weighted Esc.   |    2.88    |     **2.50**      |    2.49     |  2.34   |
| 6. GF+PE                  |    2.57    |     **1.72**      |    1.58     |  1.34   |
| 7. CSCT                   |    3.07    |     **2.92**      |    2.88     |  2.85   |
| 8. Compensated Pair Alloc |    2.16    |     **1.45**      |    1.40     |  1.29   |
| 9. Narrative Gravity      |    3.39    |     **2.75**      |    2.59     |  2.49   |

## Design Goal Scores (Graduated Realistic, 40% Enriched, 1-10)

| Algorithm         | Not On Rails | Convergent | Feels Good | Smooth | Splash | Variety |
| ----------------- | :----------: | :--------: | :--------: | :----: | :----: | :-----: |
| 1. Pair-Esc.      |      6       |     7      |     5      |   4    |   7    |    7    |
| 2. Cont. Surge    |      7       |     6      |     6      |   5    |   7    |    9    |
| 3. Esc. Pair Lock |      3       |     2      |     2      |   2    |   8    |    6    |
| 4. GPE-45         |      6       |     5      |     5      |   3    |   7    |    8    |
| 5. Sym-Weighted   |      7       |     6      |     6      |   4    |   7    |    8    |
| 6. GF+PE          |      5       |     5      |     3      |   3    |   8    |    9    |
| 7. CSCT           |      2       |     9      |     7      |   9    |   2    |    4    |
| 8. Comp. Pair     |      5       |     3      |     3      |   3    |   8    |    8    |
| 9. Narr. Gravity  |      5       |     7      |     7      |   6    |   5    |    8    |

## Player Experience Rating (1-10)

| Algorithm         | Rating | Justification                                                                 |
| ----------------- | :----: | ----------------------------------------------------------------------------- |
| 1. Pair-Esc.      |   5    | Solid average but p10=1.0 and worst streak=8 means periodic dead packs        |
| 2. Cont. Surge    |   5    | Unimodal distribution is nice but 10% zero-SA packs and avg streak 3.8 hurt   |
| 3. Esc. Pair Lock |   2    | 55% alignment means nearly half of all drafts are broken; catastrophic        |
| 4. GPE-45         |   4    | Strong late-game but 8-pick bootstrapping dead zone creates frustration       |
| 5. Sym-Weighted   |   6    | Best on symbol-rich pool (2.50), graduated ramp feels natural, but M10 at 4.3 |
| 6. GF+PE          |   3    | Floor guarantees draws not quality; Ramp at 1.13 M3 is unacceptable           |
| 7. CSCT           |   7    | Smoothest delivery, p10=2, no dead packs -- but sterile once converged        |
| 8. Comp. Pair     |   3    | 40% pair misalignment; signal-reader trace at 0.40 M3 is disqualifying        |
| 9. Narr. Gravity  |   7    | Monotonic quality ramp feels great; late-draft "funnel" is intuitive          |

## Biggest Strength and Weakness per Strategy

| Algorithm         | Biggest Strength                                  | Biggest Weakness                              |
| ----------------- | ------------------------------------------------- | --------------------------------------------- |
| 1. Pair-Esc.      | 11% fitness degradation (most robust baseline)    | M10 fail (worst streak = 8)                   |
| 2. Cont. Surge    | M3 = 2.48 with unimodal distribution              | Ramp archetype at 1.55 M3                     |
| 3. Esc. Pair Lock | High M9 variance (1.23)                           | 55% pair alignment (half of drafts broken)    |
| 4. GPE-45         | Clears M3 >= 2.0 at all fitness levels            | 10-pick bootstrapping dead zone (M5 = 12.5)   |
| 5. Sym-Weighted   | Nearly fitness-immune (2.50 to 2.49 Grad-to-Pess) | Requires all cards to carry 3 symbols         |
| 6. GF+PE          | Good concept (guaranteed quality floor)           | Floor guarantees draws not quality; M3 = 1.72 |
| 7. CSCT           | Only algorithm to pass M10 (\<= 2)                | M6 = 99% (completely on rails)                |
| 8. Comp. Pair     | Pool compensation narrows archetype gap           | 40% pair misalignment destroys performance    |
| 9. Narr. Gravity  | All 8 archetypes above 2.0 at Grad. Realistic     | M5 = 10.2 (slow convergence)                  |

## KEY QUESTION 1: Which algorithm achieves M3 >= 2.0 under harshest fitness?

Under Hostile fitness (8% average sibling A-tier), five algorithms cross 2.0:
CSCT (2.85), Narrative Gravity (2.49), Symbol-Weighted (2.34), Continuous Surge
(2.25), and GPE-45 (2.05). Pair-Escalation baseline barely clears (2.08). CSCT
is the clear leader but at severe cost to M6/M9. Among viable algorithms,
Narrative Gravity is the strongest under harsh fitness without sacrificing other
metrics catastrophically.

## KEY QUESTION 2: Which algorithm feels best to play?

CSCT and Narrative Gravity tie at 7/10. CSCT delivers the smoothest pack quality
(p10=2, no dead packs) but feels sterile once converged. Narrative Gravity
provides a satisfying "funnel" experience where the draft world narrows around
the player's choices, but the 6-10 pick transition period can feel sluggish.
Neither is perfect; a hybrid that provides CSCT's smoothness with Narrative
Gravity's variety is the target.

## Proposed Best Algorithm + Pool Composition

**Recommendation: Detuned CSCT with pair-matched slot cap of 2.** Reduce CSCT's
multiplier to 3 and cap pair-matched slots at 2/4. This projects M3 ~ 2.5-2.6
(still well above 2.0), M6 dropping from 99% to ~75-80%, and M9 recovering
toward 0.8. The M3 headroom CSCT has (2.92 at Grad. Realistic) provides
substantial budget to trade for M6 and M9.

**Pool: 40% Enriched.** 360 cards, 128 dual-resonance (16 per pair), 192
single-resonance, 40 generic. Each archetype: 24 home-only + 16 dual-res.

**Minimum fitness requirement:** Graduated Realistic (36% weighted average).
Even under Hostile, CSCT variants should remain above 2.0.

**Minimum pool change:** 15% to 40% dual-resonance (74 additional dual-res
cards). This is the critical inflection point identified in Round 1 research.

## Recommendations to the Card Designer

1. Create 128 dual-resonance cards (up from 54). Each archetype needs 16 cards
   carrying both primary and secondary resonance symbols.
2. The dual symbol is a filtering tag, not a fitness promise. A Warriors card
   with (Tide, Zephyr) need not be playable in Ramp.
3. For low-overlap pairs (Flash/Ramp, Blink/Storm), create 5-8 intentional
   bridge cards per pair that are genuinely A-tier in both archetypes. This
   lifts the worst-archetype experience.
4. The algorithm handles the rest. The card designer's job is symbol assignment
   and bridge cards, not ensuring universal cross-archetype fitness.
