# Discussion 4: Phantom Drafter / Competitive Scarcity

## Simplicity Ranking (Most to Least Simple)

1. **Pack Widening** (9/10) — The most honestly complete one-sentence description. Every operation is concrete: earn rates, spend cost, effect. A programmer can implement it from the sentence alone with only one minor ambiguity (primary vs. any-position resonance on the bonus card).

2. **Multiple Phantoms, Ecosystem Competition** (8/10) — Structurally complete in one sentence. The word "preferring" is vague (should specify "removing the card with the most matching symbols"), and tiebreaking needs a rule, but the algorithm's shape is fully determined by the description. No hidden parameters or decay rules.

3. **Square-Root Affinity Sampling** (7/10) — The core concept is clear and implementable, but "resonance overlap" needs precise definition (sum of player counters for each card symbol? count of matching resonance types?). The sqrt function is doing heavy lifting — changing it to log or cube-root fundamentally alters the algorithm's character, yet the one-sentence treats it as a fixed detail.

4. **Deck Echo Filter** (6/10) — Hides a load-bearing fallback mechanism. The filter (draw 10, keep probabilistically) is one algorithm; the backfill (fill from rejects when fewer than 4 survive) is a second algorithm glued on. Without the backfill, packs sometimes have 0-2 cards. Two algorithms pretending to be one sentence.

5. **Exile Pressure** (5/10) — Missing the decay rule entirely from the one-sentence description. Without decay (-1 per pick), exile counters grow monotonically and reach 90% skip probability by pick 27, breaking the algorithm. The decay is essential but absent. Additionally, the reroll behavior on skip is ambiguous: does the rerolled card also face the exile check? If yes, potential infinite loops; if no, the reroll bypasses the mechanism.

## Scorecard Table

| Goal | Exile Pressure | Sqrt Affinity | Pack Widening | Multiple Phantoms | Deck Echo Filter |
|------|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 4 | 7 | 9 | 8 | 6 |
| 2. Not on rails | 6 | 7 | 9 | 9 | 7 |
| 3. No forced decks | 5 | 5 | 6 | 9 | 5 |
| 4. Flexible archetypes | 6 | 8 | 7 | 4 | 7 |
| 5. Convergent | 6 | 8 | 4 | 5 | 7 |
| 6. Splashable | 5 | 6 | 9 | 7 | 6 |
| 7. Open early | 5 | 7 | 8 | 9 | 7 |
| 8. Signal reading | 6 | 2 | 3 | 10 | 2 |
| **Total** | **43** | **50** | **55** | **61** | **47** |

**Scoring rationale for key entries:**
- Multiple Phantoms gets 10 on signal reading because it is the only champion where signal reading is a first-class feature — phantom consumption patterns are directly observable.
- Multiple Phantoms gets 4 on flexible archetypes because 3 contested resonances may limit viable primary archetypes to 2 per run (the two sharing the open resonance).
- Pack Widening gets 4 on convergent because its own analysis estimates only 0.75 S/A per pack, far below the 2.0 target.
- Exile Pressure gets 4 on simplicity because its one-sentence description is critically incomplete (missing decay rule).

## Final Championed Algorithm

**Multiple Phantoms, Ecosystem Competition (Refined)**

I am keeping my Round 1 champion with two modifications:

**Modification 1 — Variable phantom count with overlap.** Instead of always 3 phantoms on 3 distinct resonances, randomize the configuration each run: sometimes 2 phantoms (2 resonances open, 4+ viable primary archetypes), sometimes 3 phantoms with possible resonance overlap (1 heavily contested + 2 open resonances). This widens the viable archetype space from 2 to 3-4 per run, addressing the Goal 4 weakness.

**Modification 2 — Static 2 phantoms as default.** After discussion, escalating aggressiveness (1→2→3 cards/round) adds time-dependent complexity. The cleaner design is static 2 phantoms with overlap as the default configuration, tested against escalating variants in simulation.

**Updated one-sentence description:** "Two phantom drafters, each assigned a random resonance (sometimes the same one), each remove the best-matching card from the pool each round; you draft from what remains."

**Honest convergence assessment (updated after discussion):** Agent 1 forced me to confront the convergence math: even if phantoms remove 25% of the pool concentrated in 3 resonances, the open resonance rises from ~25% to ~33% of the remaining pool. Only ~50% of open-resonance cards are S/A for a specific archetype, giving ~16% archetype S/A rate per card, or ~0.65 S/A per 4-card pack. This is well below the 2.0 target. Pure suppression-based mechanisms likely cannot hit 2.0 S/A at archetype level — per-card overlap scoring (Domains 2 and 5) appears to be the winning ingredient for convergence. Round 3 simulation will confirm this ceiling honestly.

**Why I'm not switching despite the convergence gap:** Multiple Phantoms is the only champion that makes signal reading a first-class feature. It creates a narrative ("someone else is taking those cards") that is thematically rich and strategically deep. The honest simulation will show the convergence ceiling, which informs whether a hybrid (Phantoms + convergence layer) should be explored in Round 4.

## Specific Modifications for Round 3

1. **Phantom configuration randomization:** Implement 3 configurations — (a) 2 phantoms on distinct resonances, (b) 3 phantoms on distinct resonances, (c) 3 phantoms with 2 sharing a resonance — with equal probability. Measure archetype viability per configuration.

2. **Static vs. escalating removal:** Test static 2 phantoms removing 1 card/round each vs. escalating 1→2→3 cards/round across draft thirds. Agent 1's feedback suggests static is simpler and may suffice.

3. **Phantom selection rule:** Phantoms always remove the card with the highest weighted symbol match to their assigned resonance (primary=2, secondary/tertiary=1), ties broken randomly.

4. **Honest convergence testing:** Specifically measure whether a signal-reading player who identifies the open resonance by pick 3-4 achieves 2+ S/A per pack by pick 6-8. Also measure what happens for a player who ignores signals and commits to a contested resonance — this should be noticeably worse, confirming that signal reading is rewarded.

## Proposed Symbol Distribution for Simulation

| Symbol Count | % of Non-Generic | Cards |
|---|---|---|
| 0 (generic) | — | 36 |
| 1 symbol | 35% | 113 |
| 2 symbols | 45% | 146 |
| 3 symbols | 20% | 65 |

Higher 1-symbol proportion (35%) sharpens phantom preferences — when a phantom assigned to Ember picks from the pool, 1-symbol [Ember] cards are unambiguous targets, making the scarcity signal cleaner. Two-symbol cards at 45% provide cross-archetype competition (phantom taking [Tide, Zephyr] hurts both Warriors and Ramp). Three-symbol cards at 20% provide strong resonance signals for players building multi-resonance decks.
