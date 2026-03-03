# Domain 5: Curated Randomness / Filtered Sampling

## Key Takeaways

- **The two-phase structure (generate then filter) is the defining strength of this domain.** The randomness of the initial candidate pool ensures variance; the filter ensures convergence. Neither phase alone solves the problem -- their interaction is what produces natural-feeling packs.
- **Oversampling ratio is the master knob.** Drawing 6 candidates and keeping 4 is barely filtered; drawing 16 and keeping 4 is heavily filtered. This single parameter controls the convergence/variance tradeoff more than any other.
- **Filters that operate on individual cards (accept/reject per candidate) produce more natural variance than filters that rank-and-select the top N.** Rank-and-select degenerates into mechanical delivery because it always picks the best-fitting candidates. Stochastic per-card filters let mediocre packs through.
- **Symbol-overlap scoring (counting shared symbols between a candidate and the player's drafted deck) is a more archetype-accurate signal than raw resonance counting.** A player drafting Warriors (Tide/Zephyr) who scores candidates by overlap with their actual deck symbols naturally favors Tide/Zephyr cards over Tide/Stone cards, partially solving the resonance-vs-archetype problem that plagued V3.
- **The biggest risk in this domain is invisible complexity.** The player never sees rejected candidates, so the filtering step is inherently hidden. Algorithms must be describable in terms of what the player observes, not what happens behind the curtain.
- **Diminishing-returns filters (where filter strength grows sublinearly with commitment) naturally prevent over-convergence.** This addresses V3's 99% deck concentration problem without adding explicit caps.
- **Symbol distributions heavy on 2-symbol cards (50-60%) give filtering algorithms the best signal.** Single-symbol cards lack discriminating power; 3-symbol cards make filtering too precise too fast.

---

## Proposal 1: Oversample-and-Score

**One-sentence description:** To make each pack, deal 8 random cards face-down, score each by counting resonance symbols it shares with your drafted deck, then reveal the top 4 scoring cards (ties broken randomly).

**Technical description:** For each pack, draw 8 cards uniformly at random from the full pool. For each candidate, compute a score equal to the number of resonance symbols on that candidate that match any resonance symbol present in the player's drafted cards (primary symbols on drafted cards count double when checking matches). Sort candidates by score descending, break ties randomly, and present the top 4. Early in the draft when few cards have been drafted, most candidates score similarly, so packs are near-random.

**Assessment:** Strong on convergence (Goal 5) and open-ended early (Goal 7) since few drafted cards means weak filtering. Fails on natural variance -- rank-and-select always picks the best 4, suppressing pack-to-pack fluctuation. Also risks over-convergence (the "on rails" problem from Goal 2) once the player has many drafted cards. The scoring is based on visible symbols, which is good for transparency.

**Best symbol distribution:** 2-symbol heavy (55%) to give scoring enough signal without over-determining results.

---

## Proposal 2: Stochastic Sieve

**One-sentence description:** To make each pack, draw cards one at a time from the shuffled pool; each card passes through with probability 50% + 10% per resonance symbol it shares with your drafted deck, and the first 4 cards that pass through become your pack.

**Technical description:** Shuffle the full card pool. Iterate through cards one at a time. For each card, compute its acceptance probability: base 50% plus 10% for each resonance symbol on the card that matches a resonance present in the player's drafted deck (primary symbols in the drafted deck count double for matching purposes, capped at 100%). Flip a weighted coin; if the card is accepted, add it to the pack. Stop when the pack has 4 cards. If the pool is exhausted before 4 cards are accepted, fill remaining slots with random cards.

**Assessment:** Excellent natural variance because each card passes through an independent coin flip -- sometimes great cards are rejected and mediocre ones slip through. Good on Goals 1 (simple), 6 (splash -- low-affinity cards still have 50% base chance), and the variance target. Convergence (Goal 5) depends on parameter tuning; the +10% per shared symbol may be too gentle for early convergence but too aggressive late. Signal reading (Goal 8) is weak since the sieve operates on the player's own history, not the pool composition.

**Best symbol distribution:** Mixed (30% one-symbol, 50% two-symbol, 20% three-symbol) to create a range of acceptance probabilities.

---

## Proposal 3: Resonance Lens

**One-sentence description:** To make each pack, draw 12 random cards, then for each of the 4 pack slots independently, pick one of the 12 at random with each card's chance weighted by (1 + number of its resonance symbols matching your most-drafted resonance), removing picked cards from the candidates.

**Technical description:** Generate a candidate pool of 12 cards drawn uniformly at random. Determine the player's "lens resonance" -- the resonance with the highest weighted symbol count in their drafted deck. For each of the 4 pack slots sequentially, assign each remaining candidate a weight of (1 + count of lens-resonance symbols on that card). Sample one card proportional to these weights, add it to the pack, and remove it from the candidate pool. Generic cards (0 symbols) always have weight 1. Early in the draft when no resonance dominates, weights are near-uniform.

**Assessment:** Good convergence toward a single resonance, but operates at the resonance level rather than archetype level -- the fundamental V3 problem. A Warriors player's lens resonance is Tide, but half of Tide cards serve other archetypes. Natural variance is moderate: the 12-candidate random draw provides variety, but weighted sampling within that pool is still somewhat predictable. Simplicity is decent but the "lens resonance" concept adds mental overhead for players.

**Best symbol distribution:** 2-symbol heavy (60%) so the lens resonance appears frequently on candidates.

---

## Proposal 4: Affinity Threshold Filter

**One-sentence description:** To make each pack, draw cards one at a time from the shuffled pool; a card with no resonance symbols matching your drafted deck must beat a coin flip to enter the pack, but a card with any matching symbol always enters, and the first 4 cards that enter become your pack.

**Technical description:** Shuffle the pool. Iterate through cards. For each candidate, check if it shares at least one resonance symbol with any card in the player's drafted deck. If yes, the card automatically enters the pack. If no, the card enters with probability P (a tunable parameter, e.g., 40%). Stop when 4 cards fill the pack. Early in the draft, few resonances are represented in the deck so most cards are "matching" and packs are near-random. As the player commits to 1-2 resonances, non-matching cards (those from the opposite side of the resonance circle) are filtered at rate (1-P).

**Assessment:** Extremely simple -- the filter is binary (matching vs. non-matching) with one parameter. Strong on Goal 1 (simplicity) and Goal 6 (splash, since non-matching cards still enter at rate P). However, convergence is weak because "matching" is too broad: any shared resonance counts, so a Warriors (Tide/Zephyr) player's filter passes cards from 6 of 8 archetypes (all except those with zero Tide or Zephyr). This makes it hard to reach 2+ archetype-specific S/A cards per pack. Variance is natural but convergence is structurally limited.

**Best symbol distribution:** 3-symbol heavy (35%) to make the matching/non-matching distinction more meaningful.

---

## Proposal 5: Deck Echo Filter

**One-sentence description:** To make each pack, draw 10 random cards, then keep each card independently with probability equal to (2 + its symbol overlap with your drafted deck) / 6, and if fewer than 4 survive, fill the remaining slots randomly from the rejects.

**Technical description:** Draw 10 candidate cards uniformly at random from the pool. For each candidate, compute its "echo score": the number of resonance symbols on the candidate that match resonance symbols present in the player's drafted deck (counting primary symbols in drafted cards as double for matching). Each candidate independently survives the filter with probability (2 + echo_score) / 6, capped at 5/6. From the survivors, take up to 4 for the pack (randomly chosen if more than 4 survive). If fewer than 4 survive, fill remaining slots by randomly selecting from the rejected candidates. This guarantees 4-card packs while making high-affinity cards more likely to appear.

**Assessment:** This is the strongest design in the domain. The independent per-card filter creates genuine variance -- sometimes a high-affinity card is rejected (probability 1/6 even at max echo) and sometimes a zero-affinity card slips through (probability 2/6). The fallback-from-rejects mechanism guarantees pack size without biasing toward affinity. Early in the draft, echo scores are low so acceptance probabilities cluster around 2/6 to 3/6, producing near-random packs. After commitment, on-archetype cards have echo scores of 2-3 (probabilities 4/6 to 5/6) while off-archetype cards score 0-1 (probabilities 2/6 to 3/6). The overlap-based scoring partially addresses the resonance-vs-archetype gap because a Warriors player's deck has Tide AND Zephyr symbols, so candidates with both (Warriors cards) score higher than candidates with only Tide (Sacrifice cards). Splash is naturally maintained because the base acceptance probability (2/6) ensures off-archetype cards appear regularly.

**Best symbol distribution:** 50% two-symbol, 30% one-symbol, 20% three-symbol. Two-symbol cards give the echo scoring meaningful discrimination between on-archetype (2 matching symbols) and adjacent-archetype (1 matching symbol) candidates.

---

## Champion Selection: Deck Echo Filter

The Deck Echo Filter is the most promising algorithm in this domain for three reasons:

**1. It solves the resonance-vs-archetype gap better than any resonance-count approach.** By scoring candidates against the player's actual drafted symbols (not just counting which resonance is highest), it naturally distinguishes between cards that share the player's full symbol profile (on-archetype) and cards that share only one resonance (adjacent but different archetype). A Warriors player with a deck full of Tide/Zephyr symbols will see Warriors cards (Tide+Zephyr) score higher than Sacrifice cards (Tide+Stone), even though both carry Tide. This is not a complete solution -- it still operates on visible symbols, not hidden archetype fitness -- but it is structurally better than V3's resonance-level mechanisms.

**2. Independent per-card filtering produces genuinely natural variance.** Each of 10 candidates makes an independent coin flip. This means sometimes 7 candidates survive and sometimes 2 do, creating real pack-to-pack fluctuation. Rank-and-select approaches (Proposal 1) always pick the best 4, killing variance. The Deck Echo Filter lets mediocre packs happen, which is a V4 design goal.

**3. The fallback mechanism is elegant.** When fewer than 4 candidates survive, the remaining slots are filled from rejected candidates (randomly). This means even heavily filtered packs contain some surprise cards. It also guarantees pack size without adding a second algorithm for edge cases.

---

## Champion Deep-Dive: Deck Echo Filter

### Example Draft Sequences

**Early Committer (Warriors, picks Tide/Zephyr aggressively):**
- Picks 1-3: Drafted deck has symbols [Tide, Zephyr, Tide, Tide, Zephyr]. Echo scores are low across the board (0-2 for most candidates). Packs feel nearly random. Acceptance probabilities: Tide/Zephyr cards ~3/6 to 4/6, Ember/Stone cards ~2/6. Mild signal at best.
- Picks 4-6: Deck now has ~8-10 Tide/Zephyr symbols. Warriors candidates score 2-3 (acceptance 4/6 to 5/6). Sacrifice candidates (Tide/Stone) score 1-2 (3/6 to 4/6). Ember-primary candidates score 0-1 (2/6 to 3/6). Player starts seeing 1-2 Warriors-quality cards per pack on average. Some packs still have zero because filters are stochastic.
- Picks 7-15: Strong commitment. Warriors candidates have echo scores of 2-3 reliably (4/6 to 5/6 acceptance). Opposite-side cards (Storm, Self-Discard) score 0 (2/6 acceptance). Average pack has ~2 S/A Warriors cards, but ranges from 0 to 3+. The player feels the draft leaning their way without guarantees.

**Flexible Player (no clear commitment through pick 8):**
- Picks 1-5: Drafts best-power cards regardless of resonance. Deck has scattered symbols from 3-4 resonances. Echo scores are moderate for many candidates (1-2), high for few. Packs remain diverse -- many archetypes represented because no single resonance dominates matching.
- Picks 6-8: Still unfocused. Average echo scores are 1-2 for most candidates. Packs look similar to early packs. The algorithm does not punish flexibility -- it simply does not help either.
- Picks 9+: Eventually begins focusing. The filter starts differentiating, but convergence is delayed to picks 10-12. This player's deck concentration will be lower (maybe 55-65% S/A), reflecting genuine flexibility.

**Pivot Attempt (starts Ember, switches to Tide at pick 8):**
- Picks 1-7: Drafts Ember-primary cards. Deck has many Ember/Stone symbols. Filter favors Storm and Blink candidates. Working as expected.
- Pick 8: Sees a powerful Tide card, decides to pivot. Takes it. Deck now has ~12 Ember symbols and ~2 Tide symbols. Filter still heavily favors Ember cards.
- Picks 9-15: Each Tide pick adds Tide symbols and dilutes Ember dominance. By pick 12, deck might have 12 Ember, 8 Tide symbols. Tide candidates now score 1-2 (not as high as if the player had committed to Tide from the start). Ember candidates still score 1-2 as well. The filter does not actively prevent the pivot, but it also does not help it -- the player is fighting against their own history. By pick 18-20, if they keep picking Tide, Tide symbols may overtake Ember. The pivot is possible but costly -- about 5-7 "wasted" picks where the filter is unhelpful.

### Predicted Failure Modes

**1. Convergence may be too slow.** The stochastic filter is gentle by design. A committed player might not reliably see 2+ S/A cards until pick 8-10 instead of the target pick 5-8. The base acceptance probability of 2/6 means off-archetype cards have a 33% chance of appearing regardless of commitment level.

**2. 10 candidates may not always contain enough on-archetype cards to filter toward.** With 360 cards and ~40 per archetype, the expected number of S/A cards (home + adjacent archetypes, roughly 120 cards or 33% of the pool) among 10 random candidates is ~3.3. After filtering, maybe 2-3 of those survive. This might be fine for the average but creates a long tail of bad packs where the 10 candidates simply did not include many on-archetype options.

**3. The algorithm is invisible to the player.** The player sees 4 cards per pack and never knows that 10 were generated and filtered. This makes the algorithm difficult to reason about strategically. Unlike Lane Locking where the player sees slots lock, the Deck Echo Filter's influence is statistical and hidden.

### Parameter Variants Worth Testing

**Variant A: Aggressive Filter (candidate pool = 14, acceptance = (1 + echo) / 5)**
- More candidates to filter from, lower base acceptance, stronger convergence signal. Risk: over-convergence, suppressed variance. Tests whether the domain can match Lane Locking's 2.72 S/A average.

**Variant B: Gentle Filter (candidate pool = 8, acceptance = (3 + echo) / 7)**
- Fewer candidates, higher base acceptance. Packs are more random even late in the draft. Tests the floor of useful filtering -- at what point does the filter become negligible?

**Variant C: Scaled Echo (candidate pool = 10, acceptance = (2 + echo * primary_weight) / 6 where primary_weight = 1.5 for primary symbols, 1.0 for secondary)**
- Weights the echo score to favor candidates whose primary resonance matches the player's primary resonance. This strengthens archetype discrimination (a Warriors card with primary Tide scores higher than a Sacrifice card with primary Tide only if the player also has Zephyr). Tests whether weighting improves archetype-level convergence without adding player-facing complexity.

### Proposed Symbol Distribution for Simulation

| Symbol Count | % of non-generic cards | Cards |
|---|---|---|
| 0 (generic) | -- | 36 |
| 1 symbol | 30% | 97 |
| 2 symbols | 50% | 162 |
| 3 symbols | 20% | 65 |

Two-symbol cards at 50% ensure that echo scoring can distinguish between "shares both resonances" (on-archetype, score 2) and "shares one resonance" (adjacent archetype, score 1). One-symbol cards at 30% provide simple, strong-signal cards for early draft picks. Three-symbol cards at 20% allow ambitious multi-resonance strategies and provide the highest echo scores for deeply committed players.
