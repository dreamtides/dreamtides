# Discussion 5: Curated Randomness / Filtered Sampling

## Simplicity Ranking (Most to Least Simple)

1. **Pack Widening** (Economic) — The cleanest one-sentence description. Every
   word maps to a concrete operation: earn tokens from symbols, spend tokens to
   add a card. A programmer implements it in 20 minutes. Zero hidden complexity.

2. **Multiple Phantoms** (Phantom) — "Three phantoms each remove 1 card per
   round, preferring their assigned resonance; you draft from what's left."
   Nearly as clean, but "preferring" hand-waves over the phantom's card
   selection scoring function.

3. **Square-Root Affinity Sampling** (Soft Prob) — One mathematical sentence,
   fully implementable. But "square root" is an abstract concept that players
   can't intuit. The behavioral implication (diminishing returns) is simple; the
   mechanism description is not. A capped linear function would achieve similar
   results with more transparency.

4. **Deck Echo Filter** (Filtered) — Three concrete operations (draw
   candidates, filter probabilistically, fill from rejects), each independently
   simple. But the full sentence is long, and "symbol overlap with your drafted
   deck" requires implicit context. Moderate on both programmer and player
   simplicity.

5. **Exile Pressure** (Rejection) — Three interacting subsystems packed into
   one sentence: counter increment from passing, probabilistic pack-generation
   skip, and per-pick counter decay. Raises implementation questions (decay
   timing, reroll mechanics, multi-card passing) that the sentence doesn't
   answer. Fails the simplicity test as written.

## Scorecard Table

| Goal | Exile Pressure | Sq-Root Affinity | Pack Widening | Multiple Phantoms | Deck Echo Filter |
|------|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 4 | 7 | 9 | 8 | 6 |
| 2. Not on rails | 7 | 5 | 8 | 9 | 6 |
| 3. No forced decks | 6 | 5 | 7 | 9 | 6 |
| 4. Flexible archetypes | 7 | 8 | 7 | 4 | 7 |
| 5. Convergent | 5 | 8 | 3 | 5 | 6 |
| 6. Splashable | 6 | 5 | 8 | 7 | 8 |
| 7. Open early | 7 | 8 | 8 | 5 | 8 |
| 8. Signal reading | 4 | 2 | 3 | 10 | 2 |
| **Total** | **46** | **48** | **53** | **57** | **49** |

**Key tensions identified:**
- Goals 4+5 (flexible + convergent) oppose each other for phantom approaches.
  Automatic convergence (Sq-Root) enables flexibility; signal-based convergence
  (Phantoms) constrains it.
- Goals 5+6 (convergent + splashable) are in tension for all approaches.
  Stronger convergence crowds out off-archetype cards. Only Deck Echo Filter
  partially resolves this via its base acceptance floor (2/6 for all cards).
- Pack Widening's simplicity and convergence are inversely linked: it's simple
  *because* it does too little.

## Key Discussion Insights

- **Per-card symbol-overlap scoring is strictly superior to resonance-level
  suppression** (Agent 1's observation). Overlap naturally distinguishes
  [Tide, Zephyr] Warriors cards from [Tide, Stone] Sacrifice cards. Any final
  recommendation should use overlap-based scoring regardless of domain.
- **Deck Echo and Sq-Root Affinity are the same core idea** — both compute
  per-card affinity via symbol overlap and bias sampling probabilistically. The
  structural difference is variance profile: Deck Echo has double variance
  (candidate pool randomness × filter randomness); Sq-Root has single variance
  (weighted sampling only).
- **The Stochastic Sieve challenge:** Agents 1, 3, and 4 argued my P2 (Sieve)
  is simpler than Deck Echo. I disagree: the Sieve is SEQUENTIAL (stops after
  4 pass), so early high-affinity cards fill the pack before off-archetype
  cards are examined. Deck Echo's PARALLEL filtering (all 12 candidates
  independently tested, then 4 selected from survivors) structurally preserves
  splash. The fallback mechanism is complexity worth paying for.

## Final Champion: Deck Echo Filter (Refined)

I am keeping the Deck Echo Filter. It does not dominate any single goal, but
it avoids catastrophic failure on every goal. Its unique structural advantages:

1. **Guaranteed splash floor.** The base acceptance probability (2/6) ensures
   off-archetype cards appear in every pack regardless of commitment depth. No
   other champion offers this structural guarantee.

2. **Double variance source.** Variance comes from both the random candidate
   draw (which 12 cards?) AND the independent per-card filter (which survive?).
   This should comfortably hit the stddev >= 0.8 target.

3. **Archetype-aware filtering.** Symbol-overlap scoring naturally distinguishes
   on-archetype cards (matching both resonances) from adjacent-archetype cards
   (matching one). This partially addresses the resonance-vs-archetype gap that
   plagued V3.

## Specific Modifications for Round 3

1. **Increase candidate pool from 10 to 12.** With ~11% of cards being S/A for
   any target archetype, 12 candidates gives ~1.3 expected S/A cards in the
   candidate pool vs. ~1.1 for 10. Reduces worst-case empty-pack scenarios.

2. **Weight primary symbol overlap at 1.5x.** Echo score becomes:
   `sum(1.5 if candidate_symbol matches a primary symbol in drafted deck, else
   1.0 if it matches a secondary/tertiary symbol)`. Sharpens the distinction
   between [Tide, Zephyr] Warriors cards and [Zephyr, Tide] Ramp cards.

3. **Parameter variant: progressive denominator.** Test denominator = 6 for
   picks 1-10, 5 for picks 11-20, 4 for picks 21-30. This tightens the filter
   as commitment deepens. Include as sensitivity test, not core design.

**Revised one-sentence description:** "To make each pack, draw 12 random cards,
then keep each independently with probability (2 + its weighted symbol overlap
with your drafted deck) / 6, and fill any remaining pack slots randomly from
the rejects."

**Parameter sweeps planned:**
- Candidate pool size: 8 / 10 / 12 / 14
- Acceptance formula: (2+echo)/6 vs (1+echo)/4 vs (2+echo)/5
- Primary weight: 1.0x vs 1.5x vs 2.0x
- Symbol distribution sensitivity

## Proposed Symbol Distribution

| Symbol Count | % of Non-Generic | Cards |
|---|---|---|
| 0 (generic) | -- | 36 |
| 1 symbol | 30% | 97 |
| 2 symbols | 50% | 162 |
| 3 symbols | 20% | 65 |

Two-symbol cards at 50% are essential for the echo scoring to discriminate
between "shares both resonances" (echo 2, on-archetype) and "shares one
resonance" (echo 1, adjacent archetype). One-symbol cards at 30% provide clean
single-resonance signals for early picks. Three-symbol cards at 20% give deeply
committed players high echo scores and create the strongest convergence signal.
