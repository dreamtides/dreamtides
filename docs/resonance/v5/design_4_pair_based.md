# Domain 4: Pair-Based Pack Construction -- Round 1 Design

## Key Takeaways

- **Ordered resonance pairs (primary, secondary) uniquely identify archetypes for 2+ symbol cards.** This is the breakthrough that V3 and V4 never exploited. A card with [Tide, Zephyr] is ~100% a Warriors card; a card with just [Tide] is only ~50% likely to be useful to a Warriors player. Pair-based matching doubles effective precision.
- **Pair matching can break through V4's 1.7 S/A probabilistic ceiling.** V4 proved single-resonance probabilistic approaches cap at 1.26-1.74 S/A because each resonance maps to 4 archetypes. Pair matching eliminates this dilution for 2+ symbol cards (~75% of the pool), potentially pushing probabilistic methods past 2.0 without needing to add bonus cards or lock slots.
- **The champion algorithm is Pair Slot Guarantee:** one pack slot is deterministically filled with a card matching the player's most-drafted ordered pair once they have drafted 3+ cards of that pair. This is simple, fully automatic, and projected to cross 2.0 S/A because pair-matched cards are ~100% S/A for the target archetype (vs ~50% for resonance-matched cards).
- **The main simplicity challenge is explaining "ordered pair" to players.** "Match on first two symbols" is concrete but slightly more complex than "match on first symbol." All five proposals must earn their complexity through measurably better archetype precision.
- **Symbol distribution heavily favors 2-symbol majority (60%+).** Pair-based algorithms are inert on 0- and 1-symbol cards. The more 2+ symbol cards in the pool, the stronger pair matching becomes. A distribution of 15% one-symbol / 65% two-symbol / 20% three-symbol maximizes pair availability.
- **1-symbol cards create a "pair blind spot" that preserves early openness.** Because pair tracking only activates on 2+ symbol cards, early picks of 1-symbol cards contribute to resonance identity without committing to an archetype pair, naturally keeping the early draft open.
- **Pair-based algorithms are inherently archetype-balanced.** Each of the 8 archetypes corresponds to exactly one ordered pair. Unlike resonance-based systems where some resonances appear more (due to adjacent archetypes), pair counts are uniformly distributed -- no archetype has a structural advantage.

---

## Algorithm 1: Pair-Weighted Sampling

**One-sentence description:** Each card in the pool is drawn with weight 1 + (3 x match), where match is 1 if the card's ordered pair equals your most-drafted pair, 0 otherwise; all 4 pack slots use this weighting.

**Technical description:** Track the player's pair profile (counts of each ordered (primary, secondary) pair from drafted 2+ symbol cards). Identify the leading pair. When generating each pack slot, sample from the full pool with weight 4 for cards whose ordered pair matches the leading pair and weight 1 for all others. All 4 slots are independently sampled.

**Assessment:** Serves convergence (Goal 6) moderately -- pair-matched cards are ~100% archetype S/A, so even modest weighting produces high-quality hits. Serves natural variance (Goal 5) well -- probabilistic sampling means pack composition varies naturally. Fails the 2.0 S/A threshold likely -- V4 showed pure weighting caps around 1.7 S/A even with perfect archetype targeting. The 4x weight boost for a pair that is ~1/12 of the pool (roughly 30 cards out of 360) would raise pair density from ~8% to ~28% per slot, yielding ~1.1 pair-matched cards per pack. Adding ~0.5 from adjacent-pair partial matches gives ~1.6 total S/A. Still below 2.0.

**Best symbol distribution:** 65% two-symbol, 20% three-symbol, 15% one-symbol. Maximizes the pool of pair-matchable cards.

---

## Algorithm 2: Pair Slot Guarantee

**One-sentence description:** Track the ordered symbol pair (first, second) of each card you draft; once you have drafted 3+ cards with the same pair, one pack slot always shows a random card with that pair, and the other 3 slots are random.

**Technical description:** Maintain a pair counter for each of the 12 possible ordered pairs. When any pair count reaches the activation threshold (3), one slot in every subsequent pack is filled by sampling uniformly from cards in the pool whose ordered pair matches the player's leading pair. The remaining 3 slots are filled randomly from the full pool. If the leading pair changes (a different pair overtakes the current leader), the guaranteed slot switches to the new leader. Only 1 slot is ever guaranteed.

**Assessment:** Strongly serves convergence (Goal 6) -- a pair-matched card is ~100% S/A for the target archetype, so the guaranteed slot delivers ~1.0 S/A cards reliably. Combined with the ~0.75 baseline S/A from the 3 random slots, total late S/A is projected at ~1.75-2.1. Serves simplicity (Goal 1) well -- "match on first two symbols" is concrete. Serves flexibility (Goal 5) -- the slot follows the leading pair, so pivots are possible by shifting drafting. Serves variance well -- only 1 of 4 slots is guaranteed, so 3 remain fully random. May fail the 2.0 target marginally without a second guaranteed slot.

**Best symbol distribution:** 65% two-symbol, 20% three-symbol, 15% one-symbol.

---

## Algorithm 3: Pair Cascade

**One-sentence description:** Each card you draft adds 1 to its ordered pair counter; at pair count 2 one slot is pair-matched, at 5 a second slot is pair-matched, and at 9 a third slot is pair-matched.

**Technical description:** Track pair counters as in Algorithm 2. At threshold 2, one pack slot is permanently assigned to draw from the player's leading pair. At threshold 5, a second slot is assigned. At threshold 9, a third. Only 1 slot ever remains random. Thresholds apply to the highest-count pair. Slots remain locked to whichever pair triggered them; they do not follow if the leading pair changes.

**Assessment:** Strongest convergence in this domain -- 3 pair-matched slots at ~100% S/A precision yields ~3.0+ S/A late-game. Convergence is comparable to Lane Locking (2.72) but with better archetype precision. However, this is essentially "Lane Locking but with pairs" -- it inherits the mechanical feel (Goal 3 fail) and "on rails" problem. After 3 slots lock, there is 1 random slot left. Deck concentration will approach 99%. Pivoting is nearly impossible after threshold 5.

**Best symbol distribution:** 50% two-symbol, 30% three-symbol, 20% one-symbol. Higher three-symbol count accelerates pair accumulation.

---

## Algorithm 4: Pair Echo Replacement

**One-sentence description:** After you pick a card with 2+ symbols, one random card in the next pack is replaced by a card whose ordered pair matches the card you just picked.

**Technical description:** No counters or state beyond the most recently drafted card. If the player's last pick had 2+ symbols, forming an ordered pair (P, S), then one of the 4 cards in the next pack is replaced: instead of being drawn randomly, it is drawn from the subset of pool cards with ordered pair (P, S). If the last pick had 0-1 symbols, no replacement occurs and all 4 slots are random. This is entirely reactive -- only the most recent pick matters.

**Assessment:** Excellent simplicity (Goal 1) -- no counters, no thresholds, just "your last pick echoes." Excellent flexibility (Goal 5) -- every pick can change the echo, so pivoting is instant. Good variance -- replacement only occurs when the last card had 2+ symbols (~75% of picks), creating natural on/off. Mediocre convergence -- only 1 slot is pair-matched, and only ~75% of the time, giving ~0.75 guaranteed S/A cards per pack plus ~0.75 baseline = ~1.5 total. Below 2.0. Fails to reward building toward a specific archetype over time (no accumulation).

**Best symbol distribution:** 70% two-symbol, 15% three-symbol, 15% one-symbol. Maximizes echo activation frequency.

---

## Algorithm 5: Pair Bonus Injection

**One-sentence description:** Each 2+ symbol card you draft adds 1 to its ordered pair's counter; when any pair counter reaches 3, a bonus 5th card matching that pair is added to your next pack and the counter resets to 0.

**Technical description:** Track pair counters (only 2+ symbol cards contribute). When any counter hits 3, the next pack is generated as 4 random cards plus 1 bonus card drawn from the pool subset matching that pair. Then the counter resets to 0. If multiple pairs hit 3 simultaneously, one is chosen (highest count, ties broken randomly). This is the pair-based analog of V4's Pack Widening, but fully automatic -- no spending decision, just a threshold trigger.

**Assessment:** Strong convergence -- the bonus card is ~100% S/A for the target archetype (pair precision), compared to ~50% for Pack Widening's resonance-matched bonus. Auto-trigger at threshold 3 fires roughly every 3-4 picks for a committed player (since ~75% of picks contribute 1 pair count). This yields bonus packs ~25-33% of the time. Combined with baseline random S/A, projected late S/A is ~1.5 (baseline) + ~0.3 (bonus contribution averaged across all packs) = ~1.8. Close to 2.0 but may not clear it. Simplicity is excellent -- very close to the passing example in the orchestration plan. Flexibility is good -- counter resets allow pivots. Variance is natural -- bonus packs alternate with normal packs.

**Best symbol distribution:** 60% two-symbol, 25% three-symbol, 15% one-symbol.

---

## Champion Selection: Algorithm 2 -- Pair Slot Guarantee

**Why this is the champion:**

Algorithm 2 occupies the sweet spot in this domain. It is the simplest algorithm that plausibly crosses the 2.0 S/A threshold, while avoiding the mechanical over-commitment of Algorithm 3 (Pair Cascade) and the under-convergence of Algorithms 1, 4, and 5.

The core reasoning: V4 proved that single-resonance probabilistic approaches cap at ~1.7 S/A because ~50% of resonance-matched cards belong to the wrong archetype. Pair matching eliminates this dilution for 2+ symbol cards. A single guaranteed pair-matched slot delivers ~1.0 S/A card per pack (vs ~0.5 for a resonance-matched slot). Combined with ~0.75 baseline S/A from 3 random slots, the projected total is ~1.75-2.1.

The critical question is whether 1 guaranteed slot is enough. If the baseline random contribution is generous (which depends on symbol distribution and pool composition), 2.0 is achievable. If not, a variant with a second slot at a higher threshold (pick 7-8) would push it over decisively -- bridging toward Algorithm 3 but with less rigidity.

Why not the others:
- **Algorithm 1 (Weighted Sampling):** Pure weighting is too weak. V4's structural finding applies -- even with pair precision, weighting alone likely caps below 2.0.
- **Algorithm 3 (Pair Cascade):** Too mechanical. It is Lane Locking with better archetype targeting but the same "on rails" feel. Passes convergence easily but fails Goals 3, 5, and produces 99% concentration.
- **Algorithm 4 (Echo Replacement):** Too reactive. No accumulation means no reward for commitment. Convergence ~1.5 is too low.
- **Algorithm 5 (Pair Bonus Injection):** Promising but the periodic bonus (~25-33% of packs) averages out to weak per-pack contribution. Projected ~1.8 S/A is close but likely short.

---

## Champion Deep-Dive: Pair Slot Guarantee

### Example Draft Sequences

**Early Committer (Warriors -- Tide/Zephyr):**
- Picks 1-2: Drafts [Tide, Zephyr] Warriors card and [Tide] card. Pair counter: {(Tide, Zephyr): 1}. Only pick 1 contributed a pair (the 1-symbol card does not). All packs fully random.
- Picks 3-4: Drafts [Tide, Zephyr] and [Zephyr, Tide]. Pair counter: {(Tide, Zephyr): 2, (Zephyr, Tide): 1}. Still below threshold.
- Pick 5: Drafts [Tide, Zephyr, Tide]. Pair counter: {(Tide, Zephyr): 3}. Threshold reached. Starting with pick 6, 1 slot shows a (Tide, Zephyr) card.
- Picks 6-30: 1 guaranteed Warriors-precision slot + 3 random. Player sees ~1.0 archetype S/A from the guaranteed slot plus ~0.75 from random = ~1.75-2.0 S/A per pack.

**Flexible Player (undecided through pick 8):**
- Picks 1-4: Drafts across multiple archetypes based on power. Pair counts: {(Tide, Zephyr): 1, (Ember, Stone): 1, (Stone, Ember): 1, (Tide, Stone): 1}. No pair reaches 3. All packs fully random -- maximum early openness.
- Picks 5-8: Starts leaning toward Sacrifice (Tide/Stone). Pair counter: {(Tide, Stone): 3} by pick 8. Guaranteed slot activates.
- Picks 9-30: Late activation but now receives consistent pair-matched Sacrifice cards. Convergence is delayed but functional.

**Pivot Attempt (starts Warriors, switches to Ramp at pick 10):**
- Picks 1-5: Commits to Warriors. Pair counter: {(Tide, Zephyr): 3}. Guaranteed slot shows (Tide, Zephyr) = Warriors cards.
- Picks 6-10: Sees appealing Ramp (Zephyr/Tide) cards in random slots. Starts drafting them. Pair counter: {(Tide, Zephyr): 4, (Zephyr, Tide): 3}.
- Pick 11+: (Zephyr, Tide) now at 3 but does not overtake (Tide, Zephyr) at 4. Player needs 1-2 more Ramp picks to make (Zephyr, Tide) the leader. By pick 13, the guaranteed slot switches to Ramp. Pivot succeeds but costs ~3 picks of mismatched guarantee.

### Predicted Failure Modes

1. **Marginal convergence.** The single guaranteed slot may not push total S/A past 2.0 reliably. If baseline random S/A averages 0.70 (rather than 0.75), the total is ~1.70 -- below target. Mitigation: add a second guaranteed slot at a higher threshold (e.g., pair count 7).

2. **1-symbol card blind spot.** Players who draft many 1-symbol cards accumulate no pair counts. An unlucky run with mostly 1-symbol offerings could delay activation past pick 8. Mitigation: symbol distribution with 65%+ two-symbol cards ensures most picks contribute pairs.

3. **Adjacent archetype confusion.** Warriors is (Tide, Zephyr) and Ramp is (Zephyr, Tide). A player drafting both Warriors and Ramp cards has split pair counts and may never reach threshold 3 in either. This is partially by design (it penalizes unfocused drafting) but could frustrate players trying to blend adjacent archetypes. Mitigation: could count (Tide, Zephyr) and (Zephyr, Tide) as partially contributing to each other (0.5 cross-credit), but this adds complexity.

4. **Generic card dilution.** The ~10% generic cards in the pool contribute no pairs. In the guaranteed slot, generics cannot appear (they have no pair). In random slots, they are noise. This is fine -- generics serve as flexible picks with no archetype commitment.

### Parameter Variants Worth Testing

**Variant A -- Single threshold, low (threshold 2):**
Faster activation (by pick 3-4 for a committed player). Risk: may activate before the player has truly committed, producing a guaranteed slot in a pair the player abandons. Benefit: stronger early convergence signal.

**Variant B -- Dual threshold (threshold 3 / threshold 7):**
First slot at pair count 3, second slot at pair count 7. This is the "bridge toward Cascade" variant. Projected late S/A: ~2.5-2.8 (two pair-matched slots at ~100% precision). Risk: second slot at 7 may trigger around pick 10-12, leaving open random from picks 6-10 that may feel under-supported. Benefit: decisively crosses 2.0 while keeping 2 random slots for variance.

**Variant C -- Dynamic threshold (threshold = total drafted cards / 5):**
Activation point scales with draft progress. At 10 total picks, threshold is 2. At 20 picks, threshold is 4. This rewards consistent pair-drafting proportionally. Risk: harder to explain, more complex mental model. Benefit: naturally adapts activation to different player speeds.

### Proposed Symbol Distribution for Simulation

| Symbol Count | % of Non-Generic | Cards | Rationale |
|---|---|---|---|
| 0 (generic) | -- | 36 | Standard 10% |
| 1 symbol | 15% | 49 | Minimal -- these cards are invisible to pair tracking |
| 2 symbols | 65% | 211 | Majority -- each contributes exactly 1 pair, ideal for pair matching |
| 3 symbols | 20% | 64 | Moderate -- contribute 1 pair plus extra resonance info |

This distribution ensures ~85% of non-generic cards contribute a pair per pick. A committed player drafting on-pair 2-symbol cards reaches threshold 3 by pick 4-5 (accounting for occasional off-pair or 1-symbol picks). The 65% two-symbol majority also means the guaranteed slot has a large pool to draw from, maintaining variance within the pair-matched subset.

Each archetype's ~40 cards break down as: ~6 one-symbol, ~26 two-symbol, ~8 three-symbol. The two-symbol subset of 26 per archetype ensures the guaranteed slot has meaningful variety (not the same 5 cards every time).
