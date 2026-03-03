# Comparison Agent 6: GF+PE (Guaranteed Floor) Perspective

## What I Learned From Failure

GF+PE was designed to solve the "dead pack" problem by guaranteeing pair-matched
slots from pick 3 onward. It failed because **the guarantee targets draws, not
quality.** Under realistic fitness, pair-matched draws from a pool split 50/50
between home and sibling archetype deliver sibling B/C cards half the time. The
floor prevents zero-match packs but does not prevent zero-quality packs.

This finding applies to every algorithm that uses pair-matching as its primary
quality mechanism: the floor of pair-matching is determined by the
home-archetype fraction of the pair-filtered subpool, not by the number of
pair-matched slots.

## Scorecard (Graduated Realistic, 40% Enriched)

| Algorithm            |  M3  | M10 | Worst-Arch | M6  |  M9  | Feel |
| -------------------- | :--: | :-: | :--------: | :-: | :--: | :--: |
| 7. CSCT              | 2.92 | 2.0 |    2.88    | 99% | 0.68 | 6/10 |
| 9. Narr. Gravity     | 2.75 | 3.3 |    2.40    | 85% | 1.21 | 8/10 |
| 5. Sym-Weighted (SR) | 2.50 | 4.3 |    1.88    | 83% | 1.18 | 7/10 |
| 2. Cont. Surge       | 2.48 | 3.8 |    1.55    | 85% | 0.78 | 5/10 |
| 4. GPE-45            | 2.25 | 8.2 |    1.92    | 67% | 0.51 | 4/10 |
| 1. Pair-Esc.         | 2.16 | 8.0 |    2.12    | 89% | 1.00 | 5/10 |
| 6. GF+PE             | 1.72 | 6.3 |    1.13    | 76% | 0.74 | 3/10 |
| 3. Esc. Pair Lock    | 1.50 | 8.8 |    0.97    | 64% | 1.23 | 1/10 |
| 8. Comp. Pair        | 1.45 | 6.9 |    1.30    | 62% | 0.83 | 2/10 |

Ranked by M3. The top three (CSCT, Narrative Gravity, Symbol-Weighted) are the
only credible contenders.

## Player Experience Rating

| Algorithm         | Rating | Key Insight                                                      |
| ----------------- | :----: | ---------------------------------------------------------------- |
| 9. Narr. Gravity  |   8    | Best feel: monotonic, high variance, all archetypes work         |
| 5. Sym-Weighted   |   7    | Good graduated ramp; Blink weakness is the only blemish          |
| 7. CSCT           |   6    | Smooth but sterile; M6=99% is the death of meaningful choice     |
| 2. Cont. Surge    |   5    | Unimodal is conceptually correct but probabilistic droughts hurt |
| 1. Pair-Esc.      |   5    | Reliable middle ground                                           |
| 4. GPE-45         |   4    | Strong late-game wasted by terrible early game                   |
| 6. GF+PE          |   3    | My own algorithm: floor concept is right, execution is wrong     |
| 8. Comp. Pair     |   2    | Alignment failure                                                |
| 3. Esc. Pair Lock |   1    | Broken                                                           |

## The Guaranteed Floor Concept: Salvageable or Not?

My key finding -- that guaranteed pair-matched draws do not equal guaranteed
quality -- has implications for every pair-matching algorithm. But the floor
concept itself is not dead. It needs to be applied differently:

**What works:** Guaranteeing that at least 1 slot per pack draws from a pool
with >= 80% home-archetype concentration. This requires either:

1. Pair-matched subpools that are dominated by home-archetype cards (not 50/50
   home/sibling), or
2. A contraction mechanism that removes sibling B/C cards from the pool before
   the floor draw

Narrative Gravity achieves option 2 naturally: pool contraction removes
low-relevance cards, so even early "random" draws increasingly hit
home-archetype cards. By pick 15, the pool is small enough that every draw is
effectively guaranteed quality.

**Hybrid proposal:** Apply my guaranteed floor concept to Narrative Gravity:
from pick 3+, ensure at least 1 slot is drawn from the contracted pool's
top-relevance subset. This addresses Narrative Gravity's M10 weakness (3.3 avg
bad streaks) by ensuring the floor draw always comes from the most concentrated
part of the surviving pool.

## KEY QUESTION 1: M3 >= 2.0 under harshest fitness?

Only Narrative Gravity achieves M3 >= 2.0 for all 8 archetypes under all four
fitness models. CSCT achieves better aggregate M3 but with M6 = 99%. The honest
answer is that **Narrative Gravity is the most complete solution.**

However, I want to flag that Narrative Gravity's mechanism (pool contraction) is
fundamentally different from slot-filling mechanisms. Slot-filling says "I will
put good cards in your pack." Contraction says "I will remove bad cards from the
pool." These have different failure modes:

- Slot-filling fails when the drawn card is wrong-archetype despite correct
  targeting
- Contraction fails when the wrong cards are removed (power-chaser with no
  commitment)

Contraction is more robust because it operates on the pool level, not the pack
level. Even when individual draws miss, the pool quality improves over time.

## KEY QUESTION 2: Best feel?

Narrative Gravity, and it is not close. The monotonic ramp, the "funnel"
metaphor, the intuitive explanation ("the game learns what you want"), the high
run-to-run variety (M7 = 5.3%) -- this is the algorithm that would make the
draft feel like a discovery process rather than a slot machine.

CSCT's smoothness is superficially appealing but creates a "solved game"
feeling. When every pack delivers 3 S/A cards, the player stops evaluating and
starts auto-drafting. Narrative Gravity's variance (M9 = 1.21) keeps the player
engaged because they never know exactly how good the next pack will be -- they
just know it will be *at least as good as the last one* on average.

## Proposed Best Algorithm

**Narrative Gravity with Relevance Floor** (hybrid of Agent 9's contraction +
Agent 6's floor concept):

- Pool contracts at 12% per pick from pick 4
- From pick 3+: 1 slot guaranteed from top-25% relevance subset of surviving
  pool
- Remaining 3 slots drawn from full surviving pool
- Pool: 40% Enriched (128 dual-res, 192 single-res, 40 generic)

Projected improvement over pure Narrative Gravity:

- M10: 3.3 -> ~1.5-2.0 (floor prevents early dead packs)
- M3: 2.75 -> ~2.5 (floor is a mild constraint, reducing late-game quality
  slightly)
- M6: 85% -> ~82% (floor prevents over-contraction on one axis)

**Minimum fitness: Graduated Realistic (36% avg).**

**Minimum pool change: 40% dual-resonance (128 cards, up from 54).**

**Set Design Specification:**

- 360 cards: 40 per archetype + 40 generic
- Each archetype: 24 home-only + 16 dual-res
- Dual-res cards carry (primary, secondary) of archetype pair
- Per-resonance R1: 80 cards; pair subpool: 16 cards per archetype
- Cross-archetype targets: Warriors/Sacrifice 50%, Self-Discard/Self-Mill 40%,
  Blink/Storm 30%, Flash/Ramp 25%

## Recommendations to the Card Designer

1. The 40% dual-res pool is the non-negotiable foundation. Every credible
   algorithm requires it.
2. The floor concept (guaranteed quality draws) only works when the pool has
   been pre-filtered. Narrative Gravity's contraction provides this
   pre-filtering naturally.
3. Do not try to guarantee quality through slot-filling alone. My algorithm's
   failure proves that pair-matched draws from a 50/50 home/sibling pool cannot
   guarantee quality under realistic fitness.
4. Focus card design effort on symbol assignment (ensuring every card carries
   correct primary and secondary resonance) rather than on cross-archetype
   fitness. The algorithm handles fitness through pool contraction.
5. For low-overlap pairs (Flash/Ramp), 5-8 intentional bridge cards improve the
   early-draft experience before contraction takes effect. This is a modest
   investment with high impact.
