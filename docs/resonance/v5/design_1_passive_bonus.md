# Domain 1: Passive Resonance Bonus (Auto-Widening) -- Round 1 Design

## Key Takeaways

- **Auto-spending on the highest resonance closely mimics optimal manual play.** V4 noted that Pack Widening's dominant strategy is "always spend on primary resonance ASAP." Automating this removes a decision that was rarely interesting while preserving convergence behavior. The loss of player agency is negligible because the agency was largely illusory.
- **Pair-based auto-spending is the most promising enhancement.** Spending based on the player's most common ordered resonance pair rather than their highest single resonance narrows the bonus card pool from ~80 cards (resonance-matched) to ~40 cards (pair-matched), roughly doubling archetype precision from ~50% to ~75-90% for bonus cards.
- **Threshold-reset vs. continuous-drain creates the core variance tradeoff.** A threshold system (spend 3, get 1 bonus, reset to 0) produces bursty delivery with natural variance. A continuous-drain system (every N tokens accumulated, probability of bonus increases) produces smoother delivery. The threshold approach is simpler and more transparent.
- **The V4 structural ceiling does NOT apply to auto-widening.** V4 proved probabilistic resonance-matching caps at ~1.7 S/A. Auto-widening sidesteps this because it ADDS bonus cards rather than merely filtering/weighting existing ones. The 2.0 threshold is reachable through the same mechanism as manual Pack Widening.
- **Over-concentration remains the primary risk.** V4's Pack Widening at cost 2 produced 98.6% deck concentration. Auto-spending removes the human brake (choosing not to spend). Higher thresholds (cost 4-5) and bonus-1 configurations are needed to keep concentration in the 60-90% target range.
- **Pack size variation (4 vs 5 cards) is the simplest UI signal.** When the player sees 5 cards instead of 4, they know the bonus system fired. This provides transparency without requiring counter displays.
- **Early-game bonus suppression is unnecessary if the threshold is right.** At cost 4+ with primary=2 weighting, a committed player cannot trigger a bonus until pick 2 at earliest (requiring a 3-symbol card). With cost 5, first bonus arrives around pick 3-4, naturally preserving early openness.

---

## Five Algorithm Proposals

### 1. Simple Threshold Auto-Spend

**Player-facing description:** "Each symbol you draft adds a matching token (primary counts as 2, others as 1); when any resonance reaches 4 tokens, your next pack gets a bonus card of that resonance and 4 tokens are deducted."

**Technical description:** Maintain 4 resonance counters. After each pick, increment counters based on drafted symbols (primary +2, secondary/tertiary +1). Before generating the next pack, check each counter in descending order; if any counter >= 4, deduct 4 tokens from the highest counter and add 1 bonus card drawn randomly from cards whose primary resonance matches. Pack becomes 5 cards (pick 1). If multiple counters >= 4, only one fires per pack.

**Assessment:** Serves convergence (Goal 6), simplicity (Goal 1), no-actions (Goal 2) very well. May struggle with not-on-rails (Goal 3) since auto-spending the highest resonance locks in quickly. Splash (Goal 7) depends on the random 4 base cards. Variance is naturally high because bonus cards come from the full resonance pool (~50% archetype hit rate). Signal reading (Goal 9) is weak -- the algorithm is self-referential, not pool-aware.

**Best symbol distribution:** Majority 2-symbol cards (55%). At cost 4 with ~3 tokens/pick, bonus fires every ~1.3 picks once committed, yielding roughly 0.75 bonus cards per pack on average for picks 6+.

### 2. Pair-Based Threshold Auto-Spend

**Player-facing description:** "Each card you draft with 2+ symbols adds 1 to its ordered pair count; when any pair reaches 3, your next pack gets a bonus card sharing that pair and the count resets to 0."

**Technical description:** Maintain a dictionary of 12 possible ordered resonance pairs mapping to counts. When a drafted card has 2+ symbols, extract its ordered pair (primary, secondary) and increment that pair's counter by 1. Before generating the next pack, check the highest pair count; if >= 3, reset it to 0 and add 1 bonus card drawn from cards whose ordered pair (primary, secondary) matches. 1-symbol and 0-symbol cards contribute nothing. Pack becomes 5 cards when triggered.

**Assessment:** Potentially the strongest convergence in this domain because pair-matched bonus cards have ~75-90% archetype precision (vs ~50% for single-resonance). Serves convergence (Goal 6) extremely well. Simplicity (Goal 1) is good but slightly more complex than single-resonance -- the pair concept requires explanation. Flexible archetypes (Goal 5) are supported because pair matching naturally targets the correct archetype. Weakness: 1-symbol cards (~20-25% of pool) don't contribute to pair counts, slowing accumulation. May need lower threshold (2 instead of 3) to compensate.

**Best symbol distribution:** High proportion of 2-symbol cards (65%+) to maximize pair generation. 1-symbol cards slow this algorithm disproportionately.

### 3. Fractional Probability Bonus

**Player-facing description:** "Each symbol you draft adds to your resonance total (primary counts as 2); each pack, there is a chance equal to your highest resonance total divided by 12 that the pack includes a bonus card of that resonance."

**Technical description:** Maintain resonance counters as in algorithm 1. Before generating each pack, compute p = min(highest_counter / 12, 0.85). Roll a random number; if < p, add 1 bonus card of the highest resonance to the pack. Counters never reset -- they grow monotonically. Pack is 4 or 5 cards depending on the roll. This produces gradually increasing bonus frequency: at 3 symbols (pick 2-3), ~25% bonus chance; at 6 symbols (pick 4-5), ~50%; at 10+ symbols (pick 7+), ~80%.

**Assessment:** Excellent variance (Goal implied) because the bonus is probabilistic, not guaranteed. Simplicity (Goal 1) passes but the probability formula is harder to intuit than a threshold. Not-on-rails (Goal 3) is moderate -- the system commits to the highest resonance but the player can still draft off-resonance. Convergence (Goal 6) scales gradually, likely reaching 2.0+ by pick 8-10 as probability climbs. Weakness: non-resetting counters mean late-game packs are nearly always 5 cards, reducing variance in the endgame.

**Best symbol distribution:** Standard distribution (25% 1-sym, 55% 2-sym, 20% 3-sym). Monotonic growth tolerates varied symbol counts.

### 4. Dual-Track Auto-Spend

**Player-facing description:** "Each symbol you draft adds a matching token (primary counts as 2); when your two highest resonances each have 3+ tokens, your next pack gets two bonus cards -- one from each of those resonances -- and 3 tokens are deducted from each."

**Technical description:** Track 4 resonance counters. Before each pack, sort counters descending. If the top two counters are both >= 3, deduct 3 from each and add 2 bonus cards (one from each resonance). If only one counter >= 3, deduct 3 from it and add 1 bonus card. Pack size is 4, 5, or 6. This naturally supports the dual-resonance structure of archetypes -- a committed Warriors player accumulates both Tide and Zephyr, triggering dual bonuses that are highly archetype-targeted.

**Assessment:** Strong convergence (Goal 6) because dual bonuses from the player's primary+secondary resonances are very likely to hit the target archetype. Flexible archetypes (Goal 5) are well-served since adjacent archetypes share resonances. Splash (Goal 7) is excellent when only one resonance triggers. Weakness: complexity (Goal 1) -- the dual-track logic is harder to explain in one sentence. Pack size variance (4/5/6) may feel chaotic. Early openness (Goal 8) could suffer if both resonances trigger simultaneously early.

**Best symbol distribution:** Majority 2-symbol (55%) to ensure both primary and secondary resonances accumulate. 3-symbol cards accelerate secondary accumulation.

### 5. Cooldown Auto-Spend

**Player-facing description:** "Each symbol you draft adds a matching token (primary counts as 2); after every 3 picks, your next pack gets a bonus card from your highest resonance and that resonance loses 3 tokens."

**Technical description:** Maintain resonance counters. Every 3rd pick (picks 3, 6, 9, ...), automatically add 1 bonus card of the highest resonance to the next pack and deduct min(3, counter) from that resonance's counter. If the highest counter is 0, no bonus is added. The fixed cadence creates a predictable rhythm: every 3rd pack is potentially enhanced. This is simpler than threshold-based systems because the trigger is temporal (every 3 picks) rather than counter-based.

**Assessment:** Strongest simplicity (Goal 1) -- the cadence is trivially predictable. No-actions (Goal 2) is perfect. Not-on-rails (Goal 3) is moderate because the system commits to the highest resonance every 3 picks, but the player can steer which resonance is highest. Convergence (Goal 6) is modest -- only ~33% of packs get bonuses, limiting late S/A to perhaps 1.5-1.8 (below 2.0 target). Weakness: the fixed cadence may feel mechanical (like Lane Locking), and the low bonus frequency likely fails the convergence target.

**Best symbol distribution:** Any distribution works. The temporal cadence decouples bonus frequency from symbol density.

---

## Champion Selection: Pair-Based Threshold Auto-Spend (Algorithm 2)

**Why this algorithm:** It combines two proven mechanisms with a V5-specific innovation:

1. **Proven: Bonus card injection.** V4 established that adding cards to packs is the only probabilistic-compatible mechanism crossing 2.0 S/A. This algorithm inherits that structural advantage.
2. **Proven: Threshold-reset delivery.** The threshold-and-reset pattern from V4's Pack Widening creates natural variance -- bonus packs and non-bonus packs alternate organically based on drafting pace.
3. **New: Pair-based precision.** The V5 orchestration plan identifies ordered pair matching as a potential breakthrough. When the bonus card is drawn from the pair-matched pool (e.g., [Tide, Zephyr] for Warriors), the archetype precision jumps from ~50% (single resonance) to ~75-90% (ordered pair). This means fewer wasted bonus cards and faster effective convergence.

The pair-based approach also has an elegant side effect: it naturally supports the dual-resonance structure of archetypes without the complexity of Algorithm 4's dual-track system. A player committed to Warriors accumulates (Tide, Zephyr) pairs, and bonus cards drawn from [Tide, Zephyr] are overwhelmingly Warriors cards. The pair IS the archetype signal, compressed into the simplest possible matching unit.

**Why not the others:**
- Algorithm 1 (Simple Threshold) is the fallback -- it works but wastes ~50% of bonus cards on wrong-archetype hits.
- Algorithm 3 (Fractional Probability) has good variance but monotonically increasing probability eliminates late-game variance and the formula is harder to explain.
- Algorithm 4 (Dual-Track) has strong convergence but the dual-trigger logic is too complex for the one-sentence test.
- Algorithm 5 (Cooldown) is the simplest but likely cannot reach 2.0 S/A with only 33% bonus frequency.

---

## Champion Deep-Dive: Pair-Based Threshold Auto-Spend

### Example Draft Sequences

**Early committer (Warriors, commits pick 4):**
- Picks 1-3: Drafts [Tide, Zephyr] Warriors card, [Tide] card, [Zephyr, Tide] Ramp card. Pair counts: (Tide, Zephyr)=1, (Zephyr, Tide)=1. No bonuses yet.
- Pick 4: Drafts [Tide, Zephyr] Warriors card. Pair counts: (Tide, Zephyr)=2. Still no bonus.
- Pick 5: Drafts [Tide, Zephyr, Tide] Warriors card. Pair counts: (Tide, Zephyr)=3. Threshold reached! Next pack gets a bonus [Tide, Zephyr] card. Reset to 0.
- Picks 6-8: Cycle continues. Each 2-symbol Warriors pick adds 1 to (Tide, Zephyr). Bonus fires every ~3 picks. Late packs: 4 random + 1 pair-matched bonus. The bonus card has ~80% chance of being a Warriors card (S/A-tier).

**Flexible player (uncommitted through pick 8):**
- Picks 1-8: Drafts across archetypes. Pair profile spreads across 4-5 pairs, none reaching 3. No bonuses fire until pick 7-9, when one pair finally accumulates. Packs remain pure random (4 cards) through the flexible phase, preserving openness. Once committed, bonus packs begin within 2-3 picks.

**Pivot attempt (starts Ember, switches to Tide at pick 8):**
- Picks 1-5: Drafts Ember-primary cards. (Ember, Stone)=2, (Ember, Zephyr)=1. One bonus fires around pick 5-6 for (Ember, Stone).
- Pick 8: Starts drafting Tide cards. Old Ember pair counts are stranded (wasted tokens). New (Tide, Zephyr) pair starts at 0. Takes 3 more picks to reach threshold 3. Effective convergence delayed to pick 11. The pivot is possible but costly -- exactly the right incentive structure.

### Predicted Failure Modes

1. **1-symbol card dilution.** Cards with only 1 symbol contribute NO pairs. If 25% of the pool is 1-symbol, a committed player drafting ~7 of 30 cards as 1-symbol gets zero pair accumulation from those picks. This slows bonus frequency. Mitigation: reduce 1-symbol cards to 15% or lower the threshold to 2.

2. **Pair pool exhaustion in small archetypes.** If the pool has only ~25-30 cards per ordered pair, drawing bonus cards without replacement could deplete the pair pool by pick 20. Mitigation: draw with replacement or from a reserve; in simulation, the pool of 360 cards with 40 per archetype should have ~25-30 cards per pair, which is sufficient for 8-10 bonus draws.

3. **Adjacent archetype confusion.** Warriors (Tide, Zephyr) and Ramp (Zephyr, Tide) share symbols but have opposite pair orders. A player who drafts a mix of both accumulates both pairs slowly rather than one quickly. This is actually desirable -- it naturally penalizes unfocused drafting -- but could frustrate players who don't understand pair order matters.

4. **Generic card dead picks.** Generic cards (10% of pool) contribute no pairs. If a player drafts 3 generics in a row (possible in early game), pair accumulation stalls entirely. This is fine for early openness but could feel bad mid-draft.

### Parameter Variants Worth Testing

1. **Threshold 3 / Bonus 1 (baseline).** Bonus fires every ~3-4 picks for a committed player with 65% 2-symbol cards. Expected late S/A: ~2.0-2.3 (4 base random giving ~1.2 S/A on average, plus ~0.75 bonus cards/pack with ~80% archetype hit rate = +0.6).

2. **Threshold 2 / Bonus 1 (aggressive).** Bonus fires every ~2-3 picks. Compensates for 1-symbol card dilution. Expected late S/A: ~2.3-2.6. Risk: may over-concentrate and trigger too early, hurting early openness.

3. **Threshold 3 / Bonus 1 / Pair fallback to single-resonance.** When no pair has reached threshold, fall back to single-resonance counting (primary=2, secondary=1). This ensures 1-symbol cards still contribute to SOME bonus mechanism. Adds complexity but addresses the 1-symbol dilution problem. The one-sentence description becomes longer but remains concrete.

### Proposed Symbol Distribution

| Symbol Count | % of Non-Generic | Cards |
|---|---|---|
| 0 (generic) | -- | 36 |
| 1 symbol | 15% | 49 |
| 2 symbols | 65% | 211 |
| 3 symbols | 20% | 64 |

Rationale: 65% 2-symbol cards maximizes pair generation. The pair-based algorithm needs most cards to have 2+ symbols to function. 1-symbol cards are reduced to 15% to minimize dead picks for pair accumulation. 3-symbol cards at 20% provide diversity and faster secondary resonance accumulation. This distribution gives a committed player ~0.85 pairs per pick (65% of picks are 2+ symbol), meaning threshold 3 fires roughly every 3.5 picks -- close to once every 3 packs.
