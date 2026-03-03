# Discussion 2: Soft Probabilistic Influence — Round 2 Output

## Simplicity Ranking (Most to Least Simple)

Ranking considers both **programmer simplicity** (can you code it from the
sentence?) and **player explainability** (can a player predict how choices
affect future packs?), a distinction surfaced during discussion with Agent 5.

1. **Pack Widening (Domain 3):** "Each symbol you draft earns 1 matching token
   (primary earns 2); before seeing a pack, you may spend 3 tokens to add a 5th
   card of that resonance." Highest on both programmer and player simplicity.
   Every operation is concrete. The player has full agency and transparency.

2. **Multiple Phantoms (Domain 4):** "Three phantom drafters each remove 1 card
   per round from the pool, each preferring a different randomly-assigned
   resonance; you draft from whatever is left." Structurally complete, one
   ambiguity (phantom selection heuristic — "preferring" is vague). Simple to
   explain but strategic implications require sophisticated reasoning.

3. **Square-Root Affinity Sampling (Domain 2, mine):** "Each card in the pool
   is drawn with probability proportional to (1 + sqrt(S)), where S is the sum
   of your drafted symbol counts for each resonance on that card, counting
   primary symbols as 2." High programmer simplicity (one formula, fully
   determined). Lower player explainability — "square root" does all the heavy
   lifting and players can't intuit the diminishing returns curve. Agent 3's
   suggested one-sentence wording adopted here for precision.

4. **Deck Echo Filter (Domain 5):** "Draw 12 random cards, keep each
   independently with probability (2 + weighted symbol overlap) / 6, fill
   remaining spots randomly from rejects." Fully specified but dense: three
   operations, a formula, and a fallback. Agent 5 argues the mental model
   ("sorting through a pile, keeping what catches your eye") is more intuitive
   than my weighted sampling. Fair point — but the two-phase structure is still
   more complex than a single formula.

5. **Exile Pressure (Domain 1):** Agent 1 acknowledged the one-sentence
   description was incomplete — decay (-1/pick) was missing. Revised version
   includes multi-symbol exile and decay but is now a paragraph, not a sentence.
   Three interacting systems (increment, probability, decay) make this the most
   opaque champion.

## Scorecard Table

| Goal | Sq-Root Affinity | Exile Pressure | Pack Widening | Multiple Phantoms | Deck Echo Filter |
|------|:---:|:---:|:---:|:---:|:---:|
| 1. Simple | 5 | 4 | 8 | 8 | 5 |
| 2. Not on rails | 5 | 7 | 8 | 6 | 5 |
| 3. No forced decks | 4 | 5 | 5 | 9 | 4 |
| 4. Flexible archetypes | 8 | 6 | 7 | 3 | 8 |
| 5. Convergent | 8 | 6 | 4 | 4 | 7 |
| 6. Splashable | 7 | 7 | 9 | 7 | 7 |
| 7. Open early | 8 | 7 | 7 | 7 | 8 |
| 8. Signal reading | 2 | 5 | 3 | 10 | 2 |
| **Total** | **47** | **47** | **51** | **54** | **46** |

Score adjustments from discussion: Pack Widening Goal 1 raised to 8 (consensus
simplest); Phantoms Goal 7 raised to 7 (Agent 4 correctly argued that 3 cards
removed from 360 is negligible early); Splash raised to 7 for Sqrt Affinity
(base weight increase to 1.5 fixes the suppression concern).

**Key structural finding (consensus across all agents):** Algorithms drawing
randomly from an unmodified pool (Phantoms, Pack Widening, Exile Pressure) face
a convergence ceiling of ~1.0-1.2 S/A per pack because each archetype is only
~11% of the pool. Only algorithms that modify per-card sampling probabilities
(Sqrt Affinity, Deck Echo Filter) can structurally push per-slot S/A
probability above 50% to reach the 2.0 target. Agent 4 explicitly conceded
this, planning to position Phantoms as a signal-reading layer rather than a
standalone convergence solution.

## Final Championed Algorithm: Square-Root Affinity Sampling (Refined)

I am **keeping** Square-Root Affinity Sampling as my champion, with refinements
from the cross-agent discussion:

**Refined formula:** `weight = 1.5 + min(sqrt(affinity), 4.5)`

Where:
- `affinity = sum(player_count[r] for r in card.symbols)` — the sum of the
  player's resonance counters for each of the card's symbols
- Player counters: primary symbol of drafted card adds 2, secondary/tertiary
  add 1
- The `min(..., 4.5)` cap prevents over-convergence after deep commitment
  (Agent 5 confirmed this works elegantly with the base weight increase)
- The `1.5` base (increased from 1.0) ensures off-archetype cards maintain
  ~25% relative probability even at maximum commitment

**Revised one-sentence description:** "Each card in the pool is drawn with
probability proportional to (1.5 + sqrt(S)), where S is the sum of your drafted
symbol counts for each resonance on that card (primary symbols count as 2,
bonus capped at 4.5), so cards matching your deck appear more often with
diminishing returns."

**Why not switch to Deck Echo Filter?** Agent 5 and I agree our algorithms are
"the same core idea with different variance profiles." Deck Echo has a double
variance source (candidate pool randomness × filter randomness) that may
produce higher stddev. Sqrt Affinity has cleaner single-phase design with no
fallback mechanism needed. Simulation will determine which variance profile
better serves V4 goals. I believe single-phase weighted sampling is the cleaner
primitive — if more variance is needed, tuning k is simpler than tuning the
oversampling ratio + acceptance formula.

**Why not Momentum Weighting?** Agents 1, 3, and 4 all flagged Momentum (P4)
as underrated for its pivot-friendliness (exponential decay erases old
commitment in 5-8 picks). This is a real advantage over Sqrt, which never
forgets. However, Momentum's convergence ceiling is structural — decay fights
accumulation, so steady-state bias is capped. Agent 4 suggested hybridizing
(sqrt of momentum-weighted affinity), but this adds complexity. I'll test
Momentum as a parameter variant in Round 3 (decay rates 0.8 vs 0.9 vs 0.95).

## Specific Modifications for Round 3

1. **Base weight 1.0 → 1.5:** Stronger splash floor. Zero-overlap cards have
   25% relative probability instead of 18% at max commitment. Directly hits
   the ≥0.5 C/F per pack target.
2. **Affinity bonus cap at 4.5:** Prevents runaway bias. Agent 5 confirmed the
   cap and base weight work together elegantly (max weight 6.0, giving 1.5/6.0
   = 25% floor for zero-overlap cards).
3. **Pool asymmetry layer:** Remove 15-20% of one random resonance's cards
   before draft start to enable signal reading (Goal 8). Team consensus: this
   is orthogonal and should apply to all champions. Agent 5 flagged a risk:
   test interaction with sqrt weighting — if pool already lacks one resonance,
   does the algorithm over-converge toward remaining resonances?
4. **Parameter sweep:** Test k in `1.5 + k*sqrt(affinity)` with
   k = {0.8, 1.0, 1.2, 1.5} and exponent p in `1.5 + affinity^p` with
   p = {0.33, 0.5, 0.67}. Agent 4 flagged that by pick 15, the 7:1 weight
   ratio might suppress variance — the k sweep will find the sweet spot.
5. **Primary symbol multiplier sweep:** Test primary = {2, 3} to sharpen
   archetype vs. resonance discrimination.
6. **Momentum variant:** Test exponential decay rates {0.8, 0.9, 0.95} as an
   alternative to cumulative counters, per suggestions from Agents 1, 3, 4.

## Proposed Symbol Distribution

| Symbols | % of non-generic | Cards |
|---------|:---:|:---:|
| 0 (generic) | — | 36 |
| 1 | 25% | 81 |
| 2 | 55% | 178 |
| 3 | 20% | 65 |

The 2-symbol majority gives affinity scoring enough signal to distinguish
archetypes sharing a resonance (e.g., Warriors [Tide, Zephyr] vs. Sacrifice
[Tide, Stone] score differently for a Tide+Zephyr-heavy deck). Will also sweep
1-symbol-heavy (40/40/20) and 3-symbol-heavy (20/40/40) distributions.
