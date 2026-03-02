# Discussion Output — Agent 2 (Structural/Guaranteed)

## Simplicity Ranking (Most to Least Simple)

1. **Echo Window** — "Count symbols across last 3 picks; top gets 2 slots, second
   gets 1, last is random." Deterministic output (2/1/1) from a small, visible
   input (3 cards). A 12-year-old can hold 3 cards in memory and predict pack
   structure. Tie-breaking is the only hidden complexity.

2. **Balanced Pack with Majority Bonus** (my revised champion, was Rotating Wheel)
   — "Each pack has one card per resonance; your majority resonance takes one
   extra slot if you have a clear leader." Output is always 1/1/1/1 or 2/1/1/0.
   Maximally predictable. Requires counting accumulated symbols, which is more
   mental load than a 3-card window but produces a simpler output structure.

3. **Lane Locking** — "When your count reaches 3, one slot permanently locks."
   Clear threshold moments, but requires tracking 4 symbol counts and remembering
   which slots are locked. "Which random slot locks" is an unnecessary detail
   players will fixate on.

4. **Weighted Lottery** — Genuinely simple to implement but probabilistic output
   fails the prediction test. A player with weight 15 Tide vs weight 3 Ember
   cannot predict their pack — only estimate probabilities. "I'll probably see
   Tide" is less transparent than "I will see 2 Tide."

5. **Resonance Swap** — The one-sentence description is clear, but the effect is
   invisible. Pool changes cannot be observed or predicted. The player experience
   is "packs somehow get better" — indistinguishable from the opaque V2 systems
   this exercise aims to replace.

## Scorecard

| Algorithm | Simple | Not Rails | No Force | Flex Arch | Converge | Splash | Open Early | Signal |
|-----------|--------|-----------|----------|-----------|----------|--------|------------|--------|
| Weighted Lottery | 7 | 5 | 6 | 6 | 9 | 4 | 8 | 2 |
| Balanced Pack w/ Majority | 8 | 7 | 7 | 6 | 7 | 8 | 9 | 3 |
| Lane Locking | 7 | 5 | 6 | 7 | 6 | 7 | 8 | 4 |
| Echo Window | 9 | 8 | 7 | 9 | 7 | 6 | 8 | 2 |
| Resonance Swap | 5 | 6 | 7 | 5 | 5 | 6 | 7 | 8 |

**Scoring rationale for key cells:**
- Weighted Lottery Splash=4: Late-game dominant weight overwhelms baseline;
  splash probability drops below 5% by pick 20.
- Balanced Pack Converge=7: Guaranteed 2 majority cards once committed (meets
  target exactly) but cannot exceed 2 without the dual-resonance variant.
- Echo Window Flex=9: 3-pick pivot window is the fastest reset of any algorithm.
  Commitment is cheap, which is both a strength (flexibility) and weakness (no
  "weight" to decisions).
- Resonance Swap Signal=8: Observable pool frequency shifts are the strongest
  signal-reading mechanism. However, the player can't distinguish "I'm seeing
  more Tide because I drafted Tide" from "more Tide was in the pool to begin
  with" — signal is present but ambiguous.

## Final Championed Algorithm: Balanced Pack with Majority Bonus

**One-sentence:** "Each pack has one card per resonance type, but if you have a
clear majority resonance (strictly more weighted symbols than any other, counting
primary=2), it replaces one random non-majority slot, giving you 2 of your
majority resonance."

During this discussion I realized my original Rotating Wheel was hiding this
simpler algorithm. The 4-slot wheel cycling through 4 resonances always produces
one of each — the rotation is cosmetic. Stripping it away reveals the true
mechanism: one-of-each baseline with majority override.

**Why this beats the original Rotating Wheel:**
- No wheel order to memorize, no opposite-slot mapping
- Pack structure is always exactly 1/1/1/1 or 2/1/1/0
- "Clear majority" threshold emerges naturally around pick 3-5

**Why I'm not switching to Echo Window (which I rank highest on simplicity):**
Echo Window's 3-pick memory means commitment is essentially free to abandon.
Structural algorithms should provide *structural guarantees* — the one-of-each
baseline is a guarantee no probabilistic algorithm can match. My algorithm
ensures the player ALWAYS sees at least 2 resonances as splash, by structure.

## Modifications for Round 3 Simulation

1. **Core algorithm:** One-of-each baseline + majority override (as described).
2. **Variant A — Dual override:** If both top-2 resonances have ≥5 symbols each,
   pack becomes 2/2/0/0. Rewards archetype-pair commitment. Test whether the
   added complexity is worth the archetype support.
3. **Variant B — Threshold majority:** Majority override only activates after the
   top resonance reaches 5+ symbols (not just "strictly more than second").
   Delays convergence slightly, keeping picks 1-3 fully balanced.
4. **Parameter sweep:** Test majority threshold at 0 (any lead), 3, and 5
   symbols ahead of second place. Also sweep symbol distribution.

## Proposed Symbol Distribution

| Symbols | % of non-generic | Count |
|---------|-----------------|-------|
| 1 symbol | 30% | 97 |
| 2 symbols | 55% | 178 |
| 3 symbols | 15% | 49 |
| Generic (0) | — | 36 |

The 1-of-each structure makes symbol distribution matter primarily for how fast
majority is established. With 55% 2-symbol cards (adding 3 weighted symbols per
pick), a committed player reaches "clear majority" by pick 3-4. The 30%
1-symbol cards provide focused mono-resonance signal; the 15% 3-symbol cards
(reduced from 20% in Round 1 to prevent instant-majority on pick 1) support
cross-archetype exploration.
