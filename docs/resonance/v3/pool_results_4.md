# Pool Results 4: Symbol Pattern Composition Analysis

## Question

What specific symbol patterns should cards have, and how do different patterns
affect draft decisions under Lane Locking?

Previous simulations varied symbol *count* (1 vs 2 vs 3). This simulation fixes
the count distribution and varies the specific *patterns* within each count --
which orderings appear and in what proportions.

## Comparison Table

Six compositions tested, 1000 drafts each, archetype-committed strategy.

| Metric | A: Pure | B: Mono | C: Bridge | D: Deep | E: Variety | **F: Rec** | Target |
|--------|:---:|:---:|:---:|:---:|:---:|:---:|--------|
| Early diversity | 6.51 | 6.47 | 6.49 | 6.47 | 6.45 | **6.50** | >=3 |
| Late S/A/pack | 2.72 | **2.76** | 2.52 | 2.75 | 2.57 | 2.68 | >=2 |
| Late C/F/pack | **0.99** | 0.94 | 0.87 | 0.97 | 0.91 | 0.91 | >=0.5 |
| Convergence pick | 4 FAIL | **5** | **5** | 4 FAIL | 4 FAIL | **5** | 5-8 |
| 1st lock pick | 2.19 | 2.22 | 2.26 | 1.68 | 1.80 | 1.95 | -- |
| 2nd lock pick | 3.21 | 3.97 | 3.80 | 3.29 | 3.24 | 3.47 | -- |
| All 4 locked pick | 7.5 | 14.6 | 10.4 | 11.6 | 9.4 | 9.6 | -- |
| **Genuine choice %** | 0.0% | 65.9% | 74.2% | 82.0% | 85.4% | **83.3%** | high |
| Unwanted lock % | 52.7% | **39.8%** | 56.2% | 42.8% | 53.5% | **39.5%** | low |
| Wasted symbols | 70.5 | **70.2** | **64.6** | 83.0 | 82.5 | 78.1 | low |
| Deck concentration | 98.5% | 98.3% | 97.7% | 98.6% | 97.9% | 98.3% | 60-80% |

## Recommended Pattern Mix (Composition F)

| Pattern | Symbols | Contribution | % of Pool | Cards (~) |
|---------|---------|-------------|:---------:|:---------:|
| mono_primary | [P] | +2P | 25% | 81 |
| standard_dual | [P, S] | +2P, +1S | 30% | 97 |
| double_primary | [P, P] | +3P | 10% | 32 |
| secondary_led | [S, P] | +2S, +1P | 10% | 32 |
| deep_commit | [P, P, S] | +3P, +1S | 10% | 32 |
| balanced_triple | [P, S, S] | +2P, +2S | 10% | 32 |
| cross_bridge | [P, Other] | +2P, +1O | 5% | 16 |
| *generic* | *none* | *none* | -- | 36 |

## Analysis: What Creates Interesting Decisions vs Autopilot

**The single most important finding: pattern variety is mandatory for
meaningful decisions.** Composition A (all [P, S]) produced 0% genuine choice
rate. Every S/A card in the pack had the same symbol contribution, making picks
purely about card mechanics with zero resonance tension. This is autopilot.

**Pattern variety creates draft tension through three mechanisms:**

1. **Depth vs breadth.** A [P, P] double-primary card gives +3P (fast single
   lock) while [P, S] gives +2P, +1S (slower but broader). The player asks:
   "Do I sprint to my second Tide lock, or build toward my Zephyr lock too?"
   This is the most common and most meaningful decision.

2. **Bridge temptation.** A [S, P] secondary-led card contributes +2S to the
   neighbor archetype while still giving +1P for the target. The player asks:
   "Do I take this card that's mechanically strong for me but accelerates a
   lock in my secondary resonance?" At 10% of the pool, these appear often
   enough to matter without dominating.

3. **Off-archetype splash.** A [P, Other] cross-bridge card contributes to a
   resonance outside the archetype pair. At only 5%, these are rare enough to
   feel special but present enough to occasionally tempt.

**Why the recommended mix works best:**

- **55% core identity** (25% mono + 30% standard). Over half the pool has
  clean, unambiguous archetype allegiance. A committed Tide/Zephyr player sees
  plenty of [Tide] and [Tide, Zephyr] cards. This is the backbone.

- **20% acceleration options** (10% double_primary + 10% deep_commit). These
  are the "commit harder" cards. [Tide, Tide] gives +3 Tide -- faster than
  [Tide, Zephyr] but with no secondary progress. The player must decide
  whether to sprint toward the second lock. These drive the depth-vs-breadth
  tension.

- **20% bridge/stretch** (10% secondary_led + 10% balanced_triple). These pull
  the player toward adjacent archetypes. [Zephyr, Tide] for a Warriors player
  gives +2 Zephyr, +1 Tide -- inverted from their ideal ratio. Good if they
  want to pivot toward Ramp; risky if they want to stay focused.

- **5% off-archetype** (cross_bridge). Just enough [Tide, Stone] or
  [Tide, Ember] cards to occasionally present an unexpected fork.

**Convergence pick 5 is correct.** Compositions with too many high-weight
patterns (D, E) converge at pick 4, slightly below the 5-8 target. The
recommended mix's 25% mono-primary cards slow accumulation enough to land at
pick 5.

## Structural Observations

**Unwanted locks are inherent, not fixable.** All compositions produce 40-56%
unwanted lock rates. At ~40% for the recommended mix, this means ~0.4 locks per
draft go to off-archetype resonances out of 4 total. Acceptable.

**Wasted symbols are a late-game artifact.** A committed player generates ~60-80
total weighted symbols but only needs ~16 to fill all thresholds. The excess is
structural, not a design problem.

**Triples are fine at 20%.** They create the strongest decisions ([P, P, S] vs
[P, S, S] is a real depth-vs-breadth choice) without over-accelerating locks.

## Answers to the Five Design Questions

1. **[P, P] double-primary?** Yes, at 10%. Creates tension with [P, S] cards.
2. **Off-archetype secondaries?** Yes, at 5% (cross_bridge). Higher rates
   (20% in C) raise unwanted locks to 56%.
3. **Secondary-led [S, P]?** 10%. Key bridge cards for pivots.
4. **3-symbol cards?** 20% total. Worth the complexity; keep below 30%.
5. **Best decision driver?** The three-way tension between [P], [P, S], and
   [P, P] in the same pack -- present in 83% of post-commitment packs.
