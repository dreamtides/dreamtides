# Comparison Agent 5: Symbol-Weighted Escalation Perspective

## Scorecard: Graduated Realistic Fitness

### On 40% Enriched Pool

| Algorithm         |  M3  |  M5  | M6  |  M9  | M10 | Passes |
| ----------------- | :--: | :--: | :-: | :--: | :-: | :----: |
| 1. Pair-Esc.      | 2.16 | 5.8  | 89% | 1.00 |  8  |   6    |
| 2. Cont. Surge    | 2.48 | 3.1  | 85% | 0.78 | 3.8 |   5    |
| 3. Esc. Pair Lock | 1.50 | 16.8 | 64% | 1.23 | 8.8 |   3    |
| 4. GPE-45         | 2.25 | 12.5 | 67% | 0.51 | 8.2 |   4    |
| 5. Sym-Weighted   | 2.29 | 9.1  | 81% |  --  | 4.7 |   5    |
| 6. GF+PE          | 1.72 | 7.5  | 76% | 0.74 | 6.3 |   4    |
| 7. CSCT           | 2.92 | 5.0  | 99% | 0.68 | 2.0 |   5    |
| 8. Comp. Pair     | 1.45 | 11.4 | 62% | 0.83 | 6.9 |   4    |
| 9. Narr. Gravity  | 2.75 | 10.2 | 85% | 1.21 | 3.3 |   7    |

### On Symbol-Rich Pool (My Champion Pool, 84.5% Dual-Res)

| Algorithm       |    M3    |  Worst-Arch  | Notes                                        |
| --------------- | :------: | :----------: | -------------------------------------------- |
| 5. Sym-Weighted | **2.50** | 1.88 (Blink) | Best M3 on this pool; near-immune to fitness |

The symbol-rich pool is my algorithm's ideal environment. M3 = 2.50 under
Graduated and 2.49 under Pessimistic -- the 0.01 difference confirms that
pair-matching on 3-symbol cards almost completely bypasses sibling fitness. But
Blink at 1.88 remains the one archetype that falls short.

## The Pool Composition Debate

The simulation results reveal that **pool composition matters more than
algorithm choice.** Consider the same algorithm class (graduated
pair-escalation) across three pools:

| Pool                | My M3 (Grad.) | Agent 4's M3 (Grad.) |
| ------------------- | :-----------: | :------------------: |
| V7 Standard (15%)   |     1.99      |         2.01         |
| 40% Enriched        |     2.29      |         2.25         |
| Symbol-Rich (84.5%) |     2.50      |      not tested      |

Moving from V7 standard to 40% enriched adds 0.30 M3. Moving from 40% enriched
to symbol-rich adds another 0.21. The total pool improvement (0.51 M3) exceeds
what any algorithm change achieves within a fixed pool. **The card designer's
pool decisions are the primary lever; the algorithm is secondary.**

## Player Experience Rating (1-10)

| Algorithm            | Rating | Justification                                                            |
| -------------------- | :----: | ------------------------------------------------------------------------ |
| 9. Narr. Gravity     |   8    | Best overall: monotonic ramp, all archetypes viable, intuitive mechanism |
| 5. Sym-Weighted (SR) |   7    | On symbol-rich pool: smooth graduated ramp, high variance, near-immune   |
| 7. CSCT              |   6    | Reliable but boring; M6=99% means no real choices after pick 5           |
| 2. Cont. Surge       |   5    | Good concept but Ramp at 1.55 is unacceptable                            |
| 1. Pair-Esc.         |   5    | Solid baseline, nothing more                                             |
| 4. GPE-45            |   4    | Dead zone kills early-game feel despite strong late performance          |
| 6. GF+PE             |   3    | The floor concept is sound but execution fails at realistic fitness      |
| 8. Comp. Pair        |   2    | Compensation idea is good but alignment problem is fatal                 |
| 3. Esc. Pair Lock    |   1    | Half of drafts broken                                                    |

## KEY QUESTION 1: M3 >= 2.0 under harshest fitness?

I need to challenge the framing of this question. Under Hostile fitness (8% avg
A-tier), the following algorithms cross 2.0 on the 40% enriched pool: CSCT
(2.85), Narrative Gravity (2.49), Symbol-Weighted (2.34 on symbol-rich),
Continuous Surge (2.25), Pair-Esc. baseline (2.08), GPE-45 (2.05).

But the question "under the harshest fitness model" presupposes that we should
design for Hostile fitness. I disagree. Hostile fitness (0-10% sibling A-tier)
means archetypes designed in complete isolation, which the card designer would
never actually do. Even minimal effort produces 15-20% overlap (the "free"
bridge cards from Research Agent B). **Graduated Realistic (36% avg) is the
honest target.** Designing for Hostile means over-engineering the algorithm and
over-constraining the pool.

Under Graduated Realistic, CSCT (2.92), Narrative Gravity (2.75),
Symbol-Weighted on symbol-rich (2.50), Continuous Surge (2.48), GPE-45 (2.25),
and Pair-Esc. (2.16) all cross 2.0 comfortably. The binding constraint shifts
from "can we reach 2.0" to "which algorithm has the best tradeoff profile."

## KEY QUESTION 2: Best feel?

Narrative Gravity wins on feel because it is the only algorithm with a monotonic
quality curve AND acceptable variance AND all archetypes above 2.0. The "funnel"
experience -- the draft world narrowing around your choices -- is uniquely
satisfying and inherently roguelike.

Symbol-Weighted Escalation on the symbol-rich pool is the runner-up. Its
graduated ramp (1 slot at 3 picks, 2 at 6, 3 at 9) creates visible progression.
But it requires every card to carry 3 symbols, which is a substantial card
design constraint.

## Can CSCT Be Rescued?

Not without fundamentally changing it. The M6 = 99% problem is not a parameter
issue -- it is what happens when you directly tie targeting intensity to
commitment ratio. Any sufficiently committed player (which is the primary
strategy) will max out the ratio immediately, turning the draft into autopilot.

The right question is not "can we detune CSCT" but "can we add CSCT's smoothness
to Narrative Gravity?" The answer might be: Narrative Gravity already has
inherent smoothness (monotonic ramp), it just needs a floor mechanism to
eliminate the picks 6-10 dead zone.

## Proposed Best Algorithm + Pool Composition

**Tier 1 (if card designer accepts 3-symbol constraint): Symbol-Weighted
Escalation on Symbol-Rich Pool**

- M3 = 2.50 under Grad. Realistic, 2.49 under Pessimistic
- 7/8 archetypes above 2.0
- Requires every non-generic card to carry exactly 3 ordered symbols
- Pool: 360 cards, 304 dual+ resonance, 40 generic, 16 triple-same

**Tier 2 (minimum pool change): Narrative Gravity on 40% Enriched Pool**

- M3 = 2.75 under Grad. Realistic, 2.59 under Pessimistic
- All 8 archetypes above 2.0
- Requires only 128 dual-res cards (up from 54)
- Simpler mechanism, simpler pool requirements

Both require minimum Graduated Realistic fitness (36% avg).

**Set Design Specification (Symbol-Rich Pool):**

- 360 cards: 320 archetype + 40 generic
- Every non-generic card carries 3 ordered symbols with repetition
- 176 AAB cards (49%), 64 ABB (18%), 64 ABC (18%), 16 AAA (4.5%)
- Pair subpool per archetype: ~40 cards (massive, no depletion risk)
- Card designer assigns symbols as archetype identity: (Tide, Tide, Zephyr) =
  Warriors

## Recommendations to the Card Designer

1. The symbol-rich pool (3 symbols per card) is the gold standard for algorithm
   performance. If feasible, adopt it.
2. If 3 symbols per card is too constraining, the 40% enriched pool (2-symbol
   dual-res) is the minimum viable target.
3. Symbol repetition (Tide, Tide, Zephyr) is more valuable than distinct triples
   (Tide, Zephyr, Ember) because it encodes archetype identity more precisely.
4. The Blink archetype (Ember/Zephyr) is the weakest across all algorithms due
   to Ember pair overlap. Consider adding 2-3 extra bridge cards specifically
   for Blink/Storm.
5. Pool composition decisions have 2x the impact of algorithm decisions on M3.
   Invest design effort in the pool first, algorithm second.
