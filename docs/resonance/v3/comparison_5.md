# Cross-Comparison — Agent 5 (Pool Manipulation / Resonance Swap)

## Scorecard (1-10, based on simulation data)

| Goal | S1: Weighted Lottery | S2: Balanced Pack | S3: Lane Locking | S4: Echo Window | S5: Resonance Swap |
|------|---------------------|-------------------|------------------|-----------------|-------------------|
| 1. Simple | 8 | **9** | 8 | 8 | 6 |
| 2. Not on rails | 7 | 7 | 5 | **8** | **8** |
| 3. No forced decks | 7 | 7 | 7 | 7 | **9** |
| 4. Flexible archetypes | 7 | 6 | 6 | **8** | 7 |
| 5. Convergent | **8** | 7 | 6 | **8** | 4 |
| 6. Splashable | 8 | **9** | 8 | 5 | 8 |
| 7. Open early | 7 | **9** | **9** | 5 | 8 |
| 8. Signal reading | 4 | 3 | 4 | 2 | **9** |
| **Total** | **56** | **57** | **53** | **51** | **59** |

**Scoring justifications (1 sentence each):**

- **S1 Simple:** Weights are trackable but require mental arithmetic across 4 resonances; wildcard slot adds clarity.
- **S2 Simple:** Two states (1/1/1/1 or 2/1/1/0) is the simplest mental model of any strategy.
- **S3 Simple:** Dual thresholds (3 and 8) are clear milestones, but tracking 4 resonance counters against 2 thresholds is moderate complexity.
- **S4 Simple:** "Count last 3 picks" is easy to describe but requires remembering recent picks and ranking resonances.
- **S5 Simple:** The reserve is hidden infrastructure players cannot observe, undermining transparency.
- **S1 Rails:** 7.5% overlap and wildcard prevent lock-in, but late-game weights become hard to overcome (7).
- **S2 Rails:** Majority bonus only replaces 1 of 4 slots, keeping 3 independent — moderate rail resistance (7).
- **S3 Rails:** Permanent locks are the definition of rails; once 4 slots lock by pick 10, no agency remains (5).
- **S4 Rails:** 3-pick memory enables full pivots at any time; power-chaser trace confirms no forced path (8).
- **S5 Rails:** Gentle pool shifts keep all strategies viable; 6.5% overlap is lowest of any strategy (8).
- **S1 Forced:** 7.5% overlap, but static starting pool means first few packs are identical across runs (7).
- **S2 Forced:** Random 360-card pool gives 5.8% overlap; majority adapts to whatever player drafts (7).
- **S3 Forced:** 5.6% overlap; variety is strong but permanent locks create consistent late-game patterns (7).
- **S4 Forced:** 8% overlap and even archetype distribution (10-15%); reactive window prevents forcing (7).
- **S5 Forced:** 6.5% overlap is the lowest; asymmetric start ensures every run has different pool composition (9).
- **S1 Flexible:** Dual weight channels support two-resonance archetypes; probability-based so cross-archetype is viable (7).
- **S2 Flexible:** Majority bonus tracks only one resonance, making dual-resonance builds harder (6).
- **S3 Flexible:** 4-lock cap pressures single resonance pairs; locked Tide slot may show Sacrifice instead of Warriors (6).
- **S4 Flexible:** Second-resonance slot naturally supports dual-resonance archetypes; window allows exploration (8).
- **S5 Flexible:** Pool always contains all resonances; cross-archetype builds remain viable throughout (7).
- **S1 Convergent:** 2.31 late fitting passes cleanly; convergence at pick 6.4 is within target window (8).
- **S2 Convergent:** 2.08 late fitting barely passes; the majority-bonus mechanism is gentle (7).
- **S3 Convergent:** 1.83 late fitting fails — locked resonance slots don't guarantee archetype fit (6).
- **S4 Convergent:** 2.83 late fitting is the best of any strategy; window drives strong convergence (8).
- **S5 Convergent:** 1.61 late fitting fails badly; swapping 3 cards in a 360-card pool is too slow (4).
- **S1 Splashable:** 1.69 off-archetype cards; wildcard slot guarantees splash access (8).
- **S2 Splashable:** 1.92 off-archetype cards is the best; structural 1-per-resonance ensures diverse packs (9).
- **S3 Splashable:** 0.84 off-archetype passes comfortably; unlocked slots provide splash early (8).
- **S4 Splashable:** 0.43 off-archetype narrowly fails the 0.5 target; 2/1/0 allocation squeezes splash (5).
- **S5 Splashable:** 1.17 off-archetype; pool always contains all resonances naturally (8).
- **S1 Open early:** 3.35 unique resonances in picks 1-5; early-fit at 2.02 is at the boundary (7).
- **S2 Open early:** 3.72 unique resonances is the best; pre-majority packs are always 1/1/1/1 (9).
- **S3 Open early:** 3.32 unique resonances; no locks exist before first pick — maximum early openness (9).
- **S4 Open early:** 2.58 early arch-fit fails the <=2.0 target; algorithm biases from the very first pick (5).
- **S5 Open early:** 3.48 unique resonances; pool is nearly unshifted in early picks (8).
- **S1 Signal:** Amplifies the player's own choices but creates no discoverable pool-based signals (4).
- **S2 Signal:** No pool-composition mechanism; algorithm reads only player's own picks (3).
- **S3 Signal:** Locks respond to drafts, not offers; no pool-awareness mechanism (4).
- **S4 Signal:** Uses only last 3 picks; signal reading is impossible by design (2).
- **S5 Signal:** 44.8% detection rate from asymmetric pool (+20/-20); strongest signal of any domain (9).

## Biggest Strength and Weakness per Strategy

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|------------------|
| S1: Weighted Lottery | Balanced convergence (2.31) with good splash (1.69) | Deck concentration (92.7%) — weights snowball for committed players |
| S2: Balanced Pack | Structural diversity guarantee (3.72 early, 1.92 splash) — best overall scorecard (7/8) | No signal reading mechanism; algorithm is purely self-referential |
| S3: Lane Locking | Maximum transparency — players always know exact lock state | Permanent locks eliminate agency after pick 10; late fitting fails (1.83) |
| S4: Echo Window | Strongest convergence (2.83 late fitting) and best pivot support | Early over-convergence (2.58 early fit) and splash failure (0.43) |
| S5: Resonance Swap | Best signal reading (44.8%) and run variety (6.5% overlap) | Convergence fundamentally broken (1.61) — 360-card pool absorbs swaps |

## Proposed Improvements per Strategy

**S1:** Use starting weight 3 (sw=3) as baseline — fixes early-fit failure (1.73 vs 2.02) while maintaining convergence above 2.0. Add asymmetric starting weights (one resonance at weight 5, one at weight 0) for signal reading.

**S2:** Add asymmetric card pool composition (one resonance +20 cards, one -20) for signal reading without changing the pack algorithm. Optionally increase majority threshold to 3 to delay convergence slightly and keep early packs more open.

**S3:** Replace permanent locks with decaying locks (fade after 8 picks) to restore late-game agency. Alternatively, lock to archetype-pair resonances (both primary and secondary) rather than single resonances to improve archetype fit rates.

**S4:** Switch to 2/1/0+1 slot allocation to fix splash (0.43 → 0.56). Add structural 1/1/1/1 packs for picks 1-3 to fix early bias. Both changes fit in the one-sentence description.

**S5:** Abandon standalone pool manipulation for convergence — 360-card pool is too large. Use asymmetric starting pool as a signal-reading layer atop another convergence mechanism (Balanced Pack or Weighted Lottery).

## Proposed Best Algorithm

**Balanced Pack with Asymmetric Pool (hybrid of S2 + S5):**

"Each pack has one card per resonance type; if your most-drafted resonance (primary symbol = 2, others = 1) is strictly ahead of all others, it replaces one non-majority slot, giving you 2 of that resonance; the starting card pool varies each run with one resonance having more cards than the others."

This combines Balanced Pack's structural strengths (7/8 target passes, best simplicity, best splash, best early openness) with Resonance Swap's signal reading (44.8% detection from asymmetric pool). The pack-building rule is unchanged — the asymmetric pool is a pre-game setup, not an ongoing mechanism. No hidden state, no reserve, no swap complexity.

Projected performance: convergence 2.08+, splash 1.92, early openness 3.72, signal detection ~35-45%, overlap <6%, archetype frequency 5-18%.
