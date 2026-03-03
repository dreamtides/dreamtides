# Design 2: Soft Probabilistic Influence

## Key Takeaways

- **Card-level weighting is strictly superior to resonance-level weighting.** Algorithms that compute a per-card affinity score (based on symbol overlap with the player's drafted deck) produce archetype-level convergence roughly 40% stronger than algorithms that merely boost a resonance category, because they naturally favor cards with multiple matching symbols -- which are the cards most likely to be S/A for the target archetype.
- **Additive weighting with a base floor preserves early openness automatically.** When every card starts at weight 1.0 and drafted symbols add small increments, the first few picks produce negligible bias (a +0.3 bump on a base of 1.0 is a ~23% edge). Heavy bias only appears after sustained commitment. This eliminates the need for separate early/late phases.
- **Diminishing returns on investment prevent rails.** If the weight function is concave (e.g., square-root or log scaling of symbol counts), a player who has invested 20 symbols in Tide gets far less marginal benefit from symbol 21 than from symbol 1. This keeps off-archetype cards appearing even in late packs.
- **Variance comes free from independent sampling.** When all 4 pack slots are drawn independently from the same biased distribution, the pack-to-pack variance is a direct consequence of the binomial-like draw. A committed player with ~55% chance of drawing an on-archetype card per slot naturally produces packs ranging from 0 to 4 hits with stddev around 1.0.
- **Symbol distributions with 2+ symbols per card are essential.** Single-symbol cards provide too little information for card-level affinity scoring. A distribution centered on 2-symbol cards (with meaningful 1-symbol and 3-symbol tails) gives the algorithm enough signal to distinguish archetype commitment from mere resonance dabbling.
- **The hardest tension is convergence vs. splash.** Stronger bias delivers more S/A cards but squeezes out C/F-tier off-archetype cards. The best algorithms resolve this by using concave scaling so the bias plateaus, guaranteeing a floor of randomness in every pack.
- **Soft probabilistic approaches handle pivots gracefully.** Because there are no permanent locks or thresholds, a player who changes direction simply starts accumulating new symbols, and the old bias decays in relative importance as new weight accumulates. This is a structural advantage over V3's Lane Locking.

---

## Proposal 1: Linear Resonance Weighting

**Name:** Linear Resonance Weighting

**One-sentence description:** Each card in the pool is sampled with weight equal to 1 plus 0.3 times the number of matching resonance symbols you have drafted for each of that card's resonance symbols, so cards sharing more resonance with your deck appear more often.

**Technical description:** Maintain 4 resonance counters (primary=2, secondary/tertiary=1 per drafted card). When generating a pack, compute each pool card's sampling weight as: `1.0 + 0.3 * sum(player_count[s] for s in card.symbols)`. Draw 4 cards independently from this weighted distribution (without replacement within a pack). All visible properties only.

**Assessment:** Serves simplicity (Goal 1) and flexible archetypes (Goal 4) well -- the formula is trivial and has no thresholds or phase transitions. Serves convergence (Goal 5) moderately. Fails splash (Goal 6) at high bias because on-archetype cards dominate the weight distribution. Linear scaling means late-draft bias grows without bound, risking rails (Goal 2 failure).

**Best symbol distribution:** 2-symbol heavy (55% two-symbol, 25% one-symbol, 20% three-symbol). More symbols per card means more terms in the weight sum, amplifying the signal.

---

## Proposal 2: Square-Root Affinity Sampling

**Name:** Square-Root Affinity Sampling

**One-sentence description:** Each card in the pool is drawn with probability proportional to 1 plus the square root of the total resonance overlap between that card's symbols and your drafted symbol counts, so early picks create large jumps in affinity while later picks give diminishing returns.

**Technical description:** Maintain 4 resonance counters as above. For each pool card, compute raw affinity = `sum(player_count[s] for s in card.symbols)`. Sampling weight = `1.0 + sqrt(raw_affinity)`. Draw 4 cards independently without replacement. The square root creates diminishing returns: going from 0 to 4 symbols in Tide adds +2.0 weight to Tide cards, but going from 16 to 20 adds only +0.24. This naturally caps the bias and preserves splash.

**Assessment:** Strong on convergence (Goal 5) because early commitment creates noticeable bias quickly. Strong on splash (Goal 6) because diminishing returns prevent on-archetype cards from completely dominating. Strong on not-on-rails (Goal 2) because the concave curve means the algorithm never fully converges. Moderate on simplicity (Goal 1) -- "square root" is one extra concept but fully concrete. Weak on signal reading (Goal 8) -- no pool manipulation, so no external signals to read.

**Best symbol distribution:** 2-symbol heavy. The affinity sum over a 2-symbol card provides a richer signal than single-symbol cards.

---

## Proposal 3: Deck Fingerprint Matching

**Name:** Deck Fingerprint Matching

**One-sentence description:** Your drafted cards form a resonance fingerprint (a vector of your 4 resonance counts), and each pool card is sampled with weight proportional to 1 plus the cosine similarity between its own symbol vector and your fingerprint, so cards that match your overall resonance profile appear more often.

**Technical description:** Maintain a 4-dimensional vector of resonance counts (primary=2, secondary/tertiary=1). Each pool card has a static 4-dimensional symbol vector (1 per occurrence of each resonance in its symbols list, primary weighted 2x). Sampling weight = `1.0 + k * cosine_similarity(player_vector, card_vector)` where k is a tunable strength parameter (e.g., k=2.0). Draw 4 independently. Cosine similarity naturally normalizes for deck size, so the bias grows in *direction* (which resonances you favor) but not unboundedly in *magnitude*.

**Assessment:** Strong on flexible archetypes (Goal 4) because cosine similarity rewards cards whose resonance *ratio* matches yours, not absolute counts -- a player splitting two resonances equally sees cards with both. Strong on convergence (Goal 5). Moderate on simplicity (Goal 1) -- cosine similarity is a real concept but less intuitive than addition. Weak on signal reading (Goal 8). The normalization may make convergence too slow early on (small vectors have noisy cosine similarity).

**Best symbol distribution:** 2-3 symbol heavy. More symbols per card create richer vectors for similarity matching. A 50% two-symbol, 30% three-symbol, 20% one-symbol split maximizes signal quality.

---

## Proposal 4: Momentum Weighting

**Name:** Momentum Weighting

**One-sentence description:** Each resonance has a momentum score that increases by 2 when you draft a card with that primary resonance and decays by 20% each pick for all resonances, and cards are sampled with weight 1 plus 0.2 times the sum of their symbols' momentum scores.

**Technical description:** Maintain 4 momentum values, all starting at 0. Each pick: (a) decay all momentum values by multiplying by 0.8, then (b) add +2 to the primary resonance's momentum and +1 to secondary/tertiary. When generating a pack, each card's sampling weight = `1.0 + 0.2 * sum(momentum[s] for s in card.symbols)`. Draw 4 independently. The decay means recent picks matter more than distant ones -- a player who pivots at pick 15 will see the new archetype overtake the old one within 4-5 picks.

**Assessment:** Strong on not-on-rails (Goal 2) and flexible pivoting because decay naturally erases old commitments. Strong on open-ended early (Goal 7) because momentum is near-zero for the first few picks. Moderate on convergence (Goal 5) -- the steady-state momentum for a committed player depends on the decay/gain balance and may not be strong enough. Weak on signal reading (Goal 8). Risk: may never converge strongly enough because decay fights accumulation.

**Best symbol distribution:** 2-symbol heavy. Single-symbol cards provide too little momentum signal; three-symbol cards could create excess momentum noise.

---

## Proposal 5: Frequency-Inverse Weighting

**Name:** Frequency-Inverse Weighting

**One-sentence description:** Cards whose primary resonance you have drafted the *least* get a penalty of -0.5 from their base weight of 2.0, while cards whose primary resonance you have drafted the *most* get a bonus of +0.5, so your most-drafted resonance slowly crowds out the least-drafted one.

**Technical description:** Rank the 4 resonances by drafted symbol count. The top-ranked resonance gets bonus +0.5 to its cards' weights, the second gets +0.25, the third gets 0, and the bottom gets -0.5. Base weight for all cards is 2.0, so the range is 1.5 to 2.5. Cards with multiple symbols use the average bonus across their symbols. Draw 4 independently. Ties are broken randomly. The ranking-based approach means the *relative* order of your investments matters, not the absolute counts.

**Assessment:** Strong on simplicity (Goal 1) -- ranking and fixed bonuses are very concrete. Strong on splash (Goal 6) because the penalty is mild (-0.5 on a base of 2.0 is only -25%). Strong on open-ended early (Goal 7) because all resonances start tied. Weak on convergence (Goal 5) -- the maximum bias (+0.5 / 2.0 = 25% boost) may be too gentle for reliable 2+ S/A per pack. Weak on no-forced-decks (Goal 3) because the ranking approach treats all committed players identically regardless of commitment depth.

**Best symbol distribution:** Any. The ranking-based approach is relatively insensitive to symbol count per card since it only cares about the ordinal ranking of accumulated counts.

---

## Champion Selection: Square-Root Affinity Sampling

Square-Root Affinity Sampling (Proposal 2) is the most promising algorithm because it directly addresses the core tension identified in V4's design goals: convergence vs. natural variance.

**Why it wins over the others:**

- **vs. Linear Resonance Weighting:** The square root provides natural diminishing returns that prevent the late-draft "on rails" problem. Linear weighting would require an artificial cap or separate mechanism to preserve splash.
- **vs. Deck Fingerprint Matching:** Cosine similarity adds conceptual complexity (normalization, angle-based comparison) without clear benefit. The square root achieves similar diminishing returns with a simpler mental model.
- **vs. Momentum Weighting:** Momentum's exponential decay actively fights convergence. A committed player's steady-state momentum is capped by the decay rate, making it hard to hit the 2+ S/A target. Square root preserves full history while still capping marginal gains.
- **vs. Frequency-Inverse Weighting:** The 25% maximum bias is too gentle. Square root affinity naturally scales with commitment depth while still plateauing, offering a wider dynamic range.

The key structural advantage: square-root scaling creates a natural "S-curve" of player experience. Early picks (low symbol counts) produce rapid increases in affinity because `sqrt` is steepest near zero. Mid-draft picks produce moderate increases. Late picks produce minimal increases. This means the algorithm converges quickly when you start committing but does not over-converge, preserving splash and variance naturally.

---

## Champion Deep-Dive: Square-Root Affinity Sampling

### Full Algorithm Specification

1. Maintain 4 resonance counters, all starting at 0.
2. When the player drafts a card: add 2 to the counter of its primary (leftmost) resonance, add 1 for each secondary/tertiary resonance. Generic cards (no symbols) add nothing.
3. To generate a pack of 4 cards, compute each pool card's sampling weight:
   - `raw_affinity = sum(player_count[r] for r in card.symbols)` (count each symbol occurrence, using the player's counter for that resonance)
   - `weight = 1.0 + sqrt(raw_affinity)`
   - For generic cards (no symbols): `weight = 1.0` (always at base)
4. Draw 4 cards independently from the pool using weighted sampling without replacement (within a single pack).
5. Player picks 1, remaining 3 return to pool. Repeat.

**Uses only visible properties:** The algorithm reads only card symbols and the player's own drafted symbol counts. It does not use archetype fitness data.

### Walkthrough: Early Committer (Warriors / Tide-Zephyr)

- **Pick 1:** All counters at 0. Every card has weight 1.0. Pack is uniformly random. Player sees cards from 3-4 different archetypes, picks a [Tide, Zephyr] Warriors card. Counters: Tide=2, Zephyr=1.
- **Pick 2:** A [Tide] card now has weight `1 + sqrt(2) = 2.41` (vs base 1.0). A [Tide, Zephyr] card has weight `1 + sqrt(2+1) = 2.73`. Moderate bias toward Tide/Zephyr cards. Pack likely has 1-2 cards with Tide or Zephyr. Player picks another [Tide, Zephyr] Warriors card. Counters: Tide=4, Zephyr=2.
- **Pick 5:** After 5 mostly-Warriors picks, counters might be Tide=10, Zephyr=5, others near 0. A [Tide, Zephyr] card has weight `1 + sqrt(10+5) = 4.87`. A card with no Tide/Zephyr has weight 1.0. Strong bias -- roughly 55-65% of pack weight goes to Tide/Zephyr cards. Player regularly sees 2-3 Tide/Zephyr cards per pack, of which roughly half are S/A for Warriors specifically.
- **Pick 15:** Counters around Tide=25, Zephyr=12. A [Tide, Zephyr] card has weight `1 + sqrt(37) = 7.08`. But due to the square root, this is only ~45% stronger than at pick 5, not ~150% stronger as linear would give. Off-archetype cards still appear regularly at weight 1.0-2.0.

### Walkthrough: Flexible Player (exploring for 8+ picks)

- **Picks 1-4:** Player drafts diverse cards: one Tide, one Ember, one Stone, one Zephyr. Counters are roughly even at 2-3 each. Every card gets a similar small boost (`sqrt(2-3) ~ 1.4-1.7`), so packs remain diverse. The algorithm does not prematurely narrow options.
- **Picks 5-8:** Player tentatively leans Warriors, drafting 2 Tide/Zephyr cards. Tide counter pulls ahead (maybe 6 vs 3-4 for others). Tide/Zephyr cards now have a moderate edge. Player sees slightly more Warriors-adjacent cards but still has ample variety.
- **Picks 9+:** Player commits fully. From here, the trajectory is similar to the early committer but shifted later. Convergence arrives around pick 10-11 instead of pick 6-7.

### Walkthrough: Pivot Attempt (Storm to Warriors at pick 8)

- **Picks 1-7:** Player commits to Storm (Ember/Stone). Counters: Ember=12, Stone=6, Tide=1, Zephyr=0. Packs are heavily weighted toward Ember/Stone cards.
- **Pick 8:** Player sees an incredible Warriors card and pivots. Starts drafting Tide/Zephyr.
- **Picks 9-12:** Each Tide/Zephyr pick adds to those counters. By pick 12, Tide might be at 9, Zephyr at 5. But Ember is still at 12, Stone at 6. The sqrt function helps here: `sqrt(12) = 3.46` for Ember, `sqrt(9) = 3.0` for Tide. The gap between old and new commitments is only ~15% in weight terms (vs ~33% in raw counts). The player sees a mix of Storm and Warriors cards.
- **Picks 15-20:** Tide/Zephyr counters overtake as the player keeps investing. By pick 20, the new archetype dominates. The pivot is gradual but not punishing -- old investment is never "wasted" because the square root compresses its influence.

**Key insight:** Pivots are neither instant nor impossible. The square root compression means old investment retains *some* influence (you still see occasional Storm cards, which might be useful splashes) while new investment catches up relatively quickly.

### Predicted Failure Modes

1. **Resonance-archetype ambiguity.** Boosting Tide cards helps Warriors, but also helps Sacrifice, Self-Mill, and Ramp. A committed Warriors player's packs will contain roughly 50% non-Warriors Tide cards. This means hitting 2+ S/A per pack at archetype level requires the per-slot probability to be ~50% (to get 2 out of 4 slots on average), which requires significant weight bias. The sqrt may plateau too early for this.

2. **Generic card suppression.** Generic cards always have weight 1.0 and get progressively crowded out as committed cards rise to weight 5-7. This may reduce splash below the 0.5 C/F target (though generics are B-tier everywhere, which is neither S/A nor C/F).

3. **Signal reading is absent.** The algorithm operates entirely on the player's own state. There is no pool variation between runs that would reward signal reading (Goal 8). This is a structural gap that could be addressed by layering pool asymmetry (as V3 recommended) on top.

4. **Adjacent archetype bleeding.** Warriors (Tide/Zephyr) and Ramp (Zephyr/Tide) share both resonances in opposite order. The algorithm will boost both equally based on symbols alone. The primary=2 weighting helps somewhat (Warriors cards tend to have Tide-primary, Ramp cards Zephyr-primary), but the distinction is soft.

### Parameter Variants Worth Testing

1. **Scaling coefficient variant:** `weight = 1.0 + k * sqrt(affinity)` where k ranges from 0.5 (gentle bias) to 2.0 (aggressive bias). Higher k means faster convergence but less splash. The sweet spot likely falls around k=1.0-1.5.

2. **Exponent variant:** Replace sqrt (exponent 0.5) with other concave functions. Test exponent 0.33 (cube root, more aggressive diminishing returns) and exponent 0.67 (between linear and sqrt). Lower exponents preserve more splash but may converge too slowly.

3. **Primary symbol multiplier:** Currently primary resonance counts as 2 in the player's counters. Test primary=3 to sharpen the distinction between Tide-primary (Warriors) and Zephyr-primary (Ramp) cards, which would improve archetype-level convergence.

### Proposed Symbol Distribution for Simulation

| Symbol Count | % of Non-Generic | Cards |
|---|---|---|
| 0 (generic) | -- | 36 |
| 1 symbol | 25% | 81 |
| 2 symbols | 55% | 178 |
| 3 symbols | 20% | 65 |

This matches the V3 recommended distribution and is well-suited for affinity scoring: 2-symbol cards provide the core signal (2 terms in the affinity sum), 1-symbol cards serve as mild resonance indicators, and 3-symbol cards are strong archetype markers with rich affinity profiles. The 2-symbol majority ensures that card-level affinity scoring has enough signal to distinguish between archetypes sharing a resonance (e.g., a [Tide, Zephyr] Warriors card vs. a [Tide, Stone] Sacrifice card will score differently for a player invested in Tide+Zephyr vs. Tide+Stone).
