# Discussion Output — Agent 5 (Pool Manipulation)

## Simplicity Ranking (Most to Least Simple to PREDICT)

1. **Echo Window** — Player looks at last 3 drafted cards, counts symbols,
   knows pack structure (2/1/1). Zero hidden state. A 12-year-old can do this.
2. **Lane Locking** — Player counts symbols toward threshold of 3, knows which
   slots are locked. Permanent locks are visible. Minor edge cases (simultaneous
   threshold crossings, which slot gets assigned) are not in the one-sentence
   description but are manageable.
3. **Weighted Lottery** — Conceptually clean ("add weight, sample
   proportionally") but prediction requires computing probability = weight /
   total. Reframing as "Token Bag with starting tokens" improves intuition but
   doesn't eliminate the math. A player can feel the trend but not predict
   specific pack composition.
4. **Rotating Wheel** — Deterministic rotation is learnable, but requires
   tracking cycle position AND remembering opposite-slot pairings (E<->T,
   S<->Z). The oscillation between focused and unfocused packs is confusing:
   "Why did my last pack have 2 Tide cards but this one has zero?"
5. **Resonance Swap** — Simple to describe ("swap 2 out, 2 in") but the effect
   is statistical drift in a 360-card pool. Players cannot predict pack
   composition from pool state. The shift from 25% to 33% over 15 picks is
   real but invisible on a per-pack basis.

## Scorecard (1-10, higher is better)

| Algorithm | G1 Simple | G2 Not Rails | G3 No Force | G4 Flexible | G5 Converge | G6 Splash | G7 Open Early | G8 Signal | Avg |
|---|---|---|---|---|---|---|---|---|---|
| Weighted Lottery | 7 | 6 | 5 | 6 | 9 | 8 | 8 | 3 | 6.5 |
| Rotating Wheel | 5 | 6 | 5 | 5 | 4 | 7 | 8 | 6 | 5.8 |
| Lane Locking | 8 | 5 | 6 | 5 | 7 | 7 | 6 | 5 | 6.1 |
| Echo Window | 9 | 8 | 4 | 9 | 7 | 5 | 9 | 2 | 6.6 |
| Resonance Swap | 6 | 7 | 8 | 7 | 5 | 6 | 7 | 9 | 6.9 |

**Scoring rationale for key differentiators:**
- Echo Window scores 9 on Simple because pack structure is fully determinable
  from the last 3 cards. Scores 2 on Signal because it uses zero pool
  information.
- Resonance Swap scores 9 on Signal because pool manipulation is the only
  domain where the pool itself changes observably. Scores 6 on Simple because
  the action is simple but prediction is opaque.
- Weighted Lottery scores 9 on Convergence because weights directly and
  monotonically determine pack composition. Scores 3 on Signal because weights
  are private state.
- Rotating Wheel scores 4 on Convergence because the oscillation problem
  (majority resonance only appears in 1 of 4 wheel positions) averages to only
  1.0 archetype cards per pack, below the 2+ target.
- Lane Locking scores 8 on Simple because threshold triggers and locked slots
  are visible and predictable.

## Critical Finding: Pool Manipulation Cannot Standalone

Agent 1's convergence math exposed a fundamental structural limitation. With a
360-card pool, individual swaps shift resonance probability by ~0.3%. To reach
50% Tide (needed for 2+ expected Tide cards per 4-card pack) from 25% baseline
by pick 8, you would need ~15 swaps per pick — reconstructing 4% of the pool
every turn. This is not "subtle pool shifting," it's wholesale pool
replacement, and it defeats the purpose of pool manipulation.

The math: 3 swaps/pick over 6 committed picks = 18 swaps. Pool goes from 90
to 108 Tide of 360 = 30%. Expected Tide per pack: 1.2. Even 5 swaps/pick only
reaches 33% = 1.33 expected. The pool size of 360 is the binding constraint.

**Implication:** Pool manipulation is a COMPLEMENTARY mechanism, not a
standalone convergence engine. Its unique value is signal reading (Goal 8) and
run-to-run variety (Goal 3). It should be layered on top of a primary
convergence mechanism like Weighted Lottery or Echo Window.

## Final Championed Algorithm: Resonance Swap (Modified)

I am keeping Resonance Swap as my champion for Round 3 simulation, with the
explicit goal of quantifying its limitations and measuring its complementary
value (signal reading, variety). I acknowledge it ranks 5th on simplicity of
prediction and will not hit the convergence target as a standalone mechanism.

Pool manipulation is the only domain that addresses Goal 8 (signal reading)
and Goal 3 (no forced decks via pool variance). Every other champion operates
on a static pool with player-side state, meaning the pool has no influence on
pack composition and every run with the same player strategy is statistically
identical. Only pool manipulation creates emergent run-to-run variety and
rewards players who read the pool's state.

If forced to recommend a single best algorithm for V3, I would choose Echo
Window or Weighted Lottery (with wildcard slot). Resonance Swap's value is as
a secondary layer that provides signal reading and variety.

## Modifications for Round 3 Simulation

1. **Swap count: 3 per pick** (up from 2). After 15 committed picks, 45 swaps
   shift the committed resonance from 25% to ~37% of the pool. Expected
   archetype cards per 4-card pack rises to ~1.5 from primary resonance alone,
   plus secondary-resonance cards could push past the 2.0 target.

2. **Reserve recycling.** Removed cards are added to the reserve of their
   primary resonance type. This prevents reserve exhaustion (the original
   design's failure mode with a finite 200-card reserve) and keeps pool size
   at exactly 360 forever.

3. **Asymmetric starting pool.** Each quest run randomly moves 20 cards of one
   resonance from the reserve into the pool and removes 20 cards of another
   resonance from the pool into the reserve. This creates a detectable signal
   in early packs (the boosted resonance appears ~5% more often) and directly
   serves Goals 3 and 8.

4. **Parameter sweeps:** Test swap counts of 2, 3, and 4. Test symmetric vs
   asymmetric starting pools. Test reserve sizes of 150 vs 200 vs 300.

5. **Signal detection measurement.** Explicitly simulate whether a
   signal-reading player can correctly identify the boosted resonance from
   pack contents in the first 5 picks.

## Proposed Symbol Distribution

| Symbols | Percentage | Cards | Notes |
|---|---|---|---|
| 0 | 10% | 36 | Generic, no swap effect |
| 1 | 25% | 81 | [Primary], clear resonance signal |
| 2 | 50% | 162 | [Primary, Secondary], archetype backbone |
| 3 | 15% | 81 | [P,P,S] or [P,S,T], deep commitment |

The 50% 2-symbol concentration ensures each pick provides a clear primary
resonance signal for swapping. The 25/50/15 split among non-generic cards
(slightly adjusted from Round 1's 30/50/20) reduces 3-symbol cards to prevent
too-rapid cross-resonance contamination in the pool, since each swap operates
on primary resonance only.
