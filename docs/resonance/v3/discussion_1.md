# Discussion Output: Agent 1 (Accumulation-Based)

## Simplicity Ranking (Most to Least Simple)

1. **Lane Locking** — State is static and visible: "I have 1 Tide slot, 3
   random." A 12-year-old can predict their next pack perfectly.
2. **Echo Window** — "My last 3 picks were Tide, Tide, Zephyr, so I get 2
   Tide, 1 Zephyr, 1 random." Requires remembering 3 cards.
3. **Weighted Lottery** — "I've drafted lots of Tide, so I'll probably see
   mostly Tide." Fuzzy but correct intuition. Can't compute exact weights
   mentally, but the direction is always clear.
4. **Rotating Wheel** — Three simultaneous state variables: wheel position
   (pick number mod 4), accumulated symbol counts for majority, and the
   opposition map (E↔T, S↔Z). Elegant on paper, mentally demanding to track.
5. **Resonance Swap** — Mechanism is simple to *describe* ("swap 2 cards") but
   impossible to *predict from*. The pool is 360 cards; a player cannot observe
   its composition or estimate probabilities. Descriptively simple, experientially
   opaque.

**Key insight:** Description simplicity and prediction simplicity are different
things. The Simplicity Test should privilege prediction — can the player
anticipate what happens next?

## Scorecard (1-10, higher is better)

| Algorithm | Simple | Not Rails | No Forced | Flexible | Convergent | Splash | Open Early | Signal | **Avg** |
|-----------|--------|-----------|-----------|----------|------------|--------|------------|--------|---------|
| Weighted Lottery (modified) | 8 | 8 | 6 | 8 | 7 | 7 | 8 | 3 | **6.9** |
| Rotating Wheel | 5 | 6 | 6 | 4 | 5 | 6 | 7 | 7 | **5.8** |
| Lane Locking | 9 | 5 | 6 | 7 | 6 | 8 | 9 | 4 | **6.8** |
| Echo Window | 8 | 5 | 4 | 7 | 9 | 4 | 5 | 3 | **5.6** |
| Resonance Swap | 7 | 7 | 8 | 5 | 4 | 7 | 7 | 9 | **6.8** |

**Score justifications for key divergences:**
- Echo Window's Convergent (9): guarantees 2 archetype slots after just 3
  picks, the fastest convergence. But its Splash (4) and Open Early (5) suffer
  because the rigid 2/1/1 allocation starts immediately and leaves only 1
  random slot.
- Resonance Swap's Signal (9): pool manipulation is the only domain where
  pack composition directly reveals shared state. But Convergent (4) because
  2 swaps in a 360-card pool shifts probability only ~0.5% per pick.
- Rotating Wheel's Flexible (4): only the single majority resonance gets
  duplication. A Warriors player's Zephyr (secondary) gets zero structural
  support despite being architecturally essential.

## Final Champion: Weighted Lottery with Wildcard Slot

**One-sentence description:** "Each resonance starts at weight 1; each drafted
symbol adds to weights (primary +2, others +1); 3 of 4 pack slots pick a
resonance proportionally to weights; the 4th slot is always a random card."

I retain Weighted Lottery as my champion with one modification: reserving 1 of
4 slots as permanently random ("wildcard"). This addresses the critical late-game
splash death problem. Without it, accumulated weights of 40+ reduce off-archetype
probability to ~2%, producing near-zero splash by pick 25.

**Why the wildcard slot works:**
- Splash floor: the random slot has ~50% chance of off-archetype, giving a
  permanent floor of ~0.5 off-archetype cards per pack — exactly the target.
- Convergence preserved: 3 proportional slots deliver 2+ archetype cards by
  pick 8-10 for committed players.
- Not more complex: "3 proportional + 1 random" is one sentence.

**Why not switch to Lane Locking?** It scored nearly as high (6.8 vs 6.9) and
is simpler. But Lane Locking has two structural problems: (1) convergence caps
at ~1.75 archetype cards per pack (1 locked + 0.75 random), missing the 2+
target, and (2) diverse drafters who cross thresholds in all 4 resonances lock
all slots permanently — the system punishes exploration more than
specialization.

## Modifications for Round 3 Simulation

1. **Wildcard slot:** 1 of 4 slots is always fully random. Remaining 3 use
   proportional weight sampling.
2. **Parameter sweep on starting weight:** Test 1, 3, and 5. Higher starting
   weights slow convergence but improve early openness and splash.
3. **Primary multiplier sweep:** Test 2x (base), 3x, and 1x. At 3x, primary
   resonance dominates faster; at 1x, dual-resonance archetypes are more
   balanced.
4. **Test with and without wildcard slot** to quantify the splash improvement.

## Proposed Symbol Distribution

| Symbols | % of non-generic | Cards | Examples |
|---------|------------------|-------|---------|
| 0 | — | 36 | Generic/neutral |
| 1 | 20% | 65 | [Tide] |
| 2 | 60% | 194 | [Tide, Zephyr] |
| 3 | 20% | 65 | [Tide, Tide, Zephyr] |

Two-symbol cards dominate. Each pick contributes ~2.7 weight on average (2 for
primary + 0.7 for secondary, accounting for 10% generics adding 0). After 6
committed picks, dominant weight ≈ 13 of total ≈ 20, giving ~65% pack share for
the proportional slots — right in the convergence zone. The 20% three-symbol
cards reward deep commitment; the 20% one-symbol cards provide focused signals.
