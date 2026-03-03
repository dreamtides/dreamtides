# Comparison Agent 3: Escalating Pair Lock Perspective

## Honest Reckoning

My algorithm failed. The 55% pair alignment rate means nearly half of all drafts
deliver wrong-archetype cards. This is not a tuning problem -- it is a
fundamental flaw in the pair-detection mechanism. I am in the best position to
identify what went wrong and which other algorithms avoid this failure.

## Scorecard (Graduated Realistic, 40% Enriched)

| Algorithm             |  M3  |  M5  | M6  |  M9  | M10 | Pass Count |
| --------------------- | :--: | :--: | :-: | :--: | :-: | :--------: |
| 1. Pair-Esc. Baseline | 2.16 | 5.8  | 89% | 1.00 |  8  |    6/10    |
| 2. Continuous Surge   | 2.48 | 3.1  | 85% | 0.78 | 3.8 |    5/10    |
| 3. Esc. Pair Lock     | 1.50 | 16.8 | 64% | 1.23 | 8.8 |    3/10    |
| 4. GPE-45             | 2.25 | 12.5 | 67% | 0.51 | 8.2 |    4/10    |
| 5. Sym-Weighted       | 2.50 | 9.2  | 83% | 1.18 | 4.3 |    6/10    |
| 6. GF+PE              | 1.72 | 7.5  | 76% | 0.74 | 6.3 |    4/10    |
| 7. CSCT               | 2.92 | 5.0  | 99% | 0.68 | 2.0 |    5/10    |
| 8. Comp. Pair Alloc   | 1.45 | 11.4 | 62% | 0.83 | 6.9 |    4/10    |
| 9. Narr. Gravity      | 2.75 | 10.2 | 85% | 1.21 | 3.3 |    7/10    |

**Narrative Gravity passes 7/10 metrics -- the most of any algorithm.** It fails
only M5 (10.2 vs 5-8), M10 (3.3 vs 2), and M6 is marginal at 85%.

## The Alignment Problem: Lessons for Everyone

Agents 3, 6, and 8 all suffer from the same root cause: **pair-counter
misalignment.** When the algorithm must infer the player's archetype from
drafted cards, and the pair counter can lock onto the wrong pair, performance
collapses.

- Agent 3 (me): 55% alignment. M3 = 1.50.
- Agent 8: ~60% alignment. M3 = 1.45.
- Agent 6: Per-archetype M3 ranges from 1.13 to 2.31 -- alignment varies wildly
  by archetype.

Algorithms that **avoid** the alignment problem entirely:

- **CSCT (Agent 7):** Uses commitment *ratio* rather than absolute pair
  counters. The ratio naturally tracks the dominant archetype because it divides
  by total picks.
- **Narrative Gravity (Agent 9):** Uses resonance signature for pool contraction
  rather than pair detection. The signature is a continuous vector, not a
  discrete pair choice, so it degrades gracefully rather than failing
  catastrophically.
- **Continuous Surge (Agent 2):** Probabilistic slot assignment means
  misalignment reduces quality but does not create broken drafts.

**Lesson: Discrete pair-locking is fragile. Continuous methods are robust.**

## Player Experience Rating (1-10)

| Algorithm             | Rating | Justification                                                        |
| --------------------- | :----: | -------------------------------------------------------------------- |
| 3. Esc. Pair Lock     |   1    | I must rate my own algorithm honestly: broken 45% of the time        |
| 7. CSCT               |   6    | Smooth and reliable but sterile; the "autopilot" problem is real     |
| 9. Narr. Gravity      |   8    | Best feel: packs improve monotonically, the funnel is satisfying     |
| 5. Sym-Weighted       |   7    | Graduated ramp without bimodality; requires heavy symbol pool though |
| 2. Continuous Surge   |   5    | Probabilistic droughts are unpredictable and frustrating             |
| 1. Pair-Esc. Baseline |   5    | Solid but unexciting; the p10=1.0 floor is too low                   |
| 4. GPE-45             |   4    | Bootstrapping dead zone is painful; late-game is strong              |
| 6. GF+PE              |   3    | Concept was right (guaranteed floor) but implementation fails        |
| 8. Comp. Pair         |   2    | Same alignment flaw as mine, with worse M3                           |

## KEY QUESTION 1: M3 >= 2.0 under harshest fitness?

Under Hostile, only CSCT (2.85), Narrative Gravity (2.49), Symbol-Weighted
(2.34), Continuous Surge (2.25), Pair-Esc. baseline (2.08), and GPE-45 (2.05)
cross 2.0. But CSCT's M6 disqualifies it. The honest answer: **Narrative Gravity
is the only algorithm that achieves M3 >= 2.0 under Hostile fitness for all 8
archetypes without catastrophic failures in other metrics.**

## KEY QUESTION 2: Best feel?

Narrative Gravity. The monotonic quality ramp is the single most important
experiential property. Every other algorithm has some form of "things got worse"
moment -- surge valleys, probabilistic droughts, bootstrapping dead zones.
Narrative Gravity only gets better over time. The M10 failure is concentrated in
picks 6-10 before the pool contracts, which is acceptable because the player is
still in the "building" phase psychologically.

## Can Pair-Locking Be Rescued?

No, not with zero player decisions. The fundamental problem is that 8 competing
pair counters with noisy 4-5 pick exploration signals cannot reliably identify
the correct pair. The only fix is player input (archetype declaration at pick
5-6), which violates the zero-decision constraint.

However, the pair-locking *concept* (guaranteed slots filled with
archetype-specific cards) is valid. It just needs a detection mechanism that
does not fail 45% of the time. CSCT's commitment ratio and Narrative Gravity's
resonance signature are both superior detection mechanisms.

## Proposed Best Algorithm + Pool Composition

**Narrative Gravity (Ultra-Aggressive, 12% contraction) on 40% Enriched pool.**

This is the only algorithm that:

1. Crosses M3 >= 2.0 for all 8 archetypes under Graduated and Pessimistic
   fitness
2. Has a monotonic quality ramp (no surge/floor alternation)
3. Uses a simple, one-sentence mechanism
4. Achieves strong run-to-run variety (M7 = 5.3%)
5. Has moderate concentration (M6 = 85%)

Its M10 failure (3.3 avg) is the one remaining weakness, and I believe it can be
addressed by adding a 1-slot pair-matched floor from pick 3+ (as Agent 2
suggests). This is a two-sentence algorithm, which is an acceptable
explainability trade.

**Minimum fitness: Graduated Realistic (36% avg).** Under Pessimistic, Narrative
Gravity still delivers 2.59 M3.

**Minimum pool change: 40% dual-resonance.** 74 additional dual-res cards.
Non-negotiable for pair-matching viability.

## What the Pool Compensation Insight Saves

Agent 8's pool compensation idea (more dual-res cards for low-overlap pairs) is
the one salvageable finding from the pair-allocation family. Applied to any
algorithm:

| Pair                              | Standard Dual-Res | Compensated |
| --------------------------------- | :---------------: | :---------: |
| Warriors/Sacrifice (high overlap) |        16         |     14      |
| Flash/Ramp (low overlap)          |        16         |     18      |

This narrows the per-archetype M3 gap without changing the algorithm. The card
designer should implement this regardless of algorithm choice.

## Recommendations to the Card Designer

1. Raise dual-resonance cap to 40% (128 cards). This is the minimum viable
   change.
2. Compensate low-overlap pairs with 2-4 extra dual-res cards each. The marginal
   cost is small.
3. Do not attempt to solve the fitness problem through card design alone. The
   algorithm (Narrative Gravity's contraction) handles fitness by removing
   unplayable cards from the pool.
4. The mechanism is explainable: "As you draft, the game removes cards that do
   not match your style, so your future packs get better." Players will
   understand this intuitively.
