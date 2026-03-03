# Domain 2: Probabilistic Slot Targeting (Soft Lane Locking)

## Key Takeaways

- **Pair-based slot targeting is the breakthrough path.** Single-resonance
  probabilistic approaches are structurally capped at ~1.7 S/A (V4 proven
  ceiling). But pair-based targeting achieves ~100% archetype precision for 2+
  symbol cards, potentially doubling the S/A yield of each targeted slot from
  ~50% to ~90%+. This may push probabilistic slot targeting above 2.0 without
  deterministic locking.

- **Graduated probability curves preserve variance while converging.** Unlike
  Lane Locking's binary on/off thresholds, a smooth probability function
  (e.g., P = min(pair_count / K, cap)) creates natural pack-to-pack variance:
  sometimes 3 targeted slots fire, sometimes 0. StdDev should comfortably
  exceed 0.8.

- **The "never 100%" principle matters.** Capping slot targeting probability
  below 1.0 (e.g., 0.80) ensures every pack retains randomness. This is the
  key experiential difference from Lane Locking -- no pack is ever fully
  determined.

- **Multi-slot independence creates binomial variance.** When 4 slots each
  independently roll against a probability, the number of targeted slots
  follows a binomial distribution. At P=0.6 across 4 slots, you get 0-4
  targeted slots with natural spread (mean 2.4, stddev ~0.98).

- **Pair-based matching requires heavy 2+ symbol card distributions.** If only
  25% of cards have 2+ symbols, there are too few pair-matchable cards to fill
  targeted slots. The algorithm needs 70-80% of cards to have 2+ symbols.

- **The champion algorithm (Pair-Escalation Slots) combines pair matching with
  smooth probability scaling**, avoiding both Lane Locking's rigidity and
  single-resonance dilution. Projected 2.0-2.4 S/A at archetype level.

- **Pivoting degrades gracefully.** Because probabilities scale with pair
  counts (not binary locks), a player who shifts archetype mid-draft sees
  their old pair's probability decrease as new pairs accumulate -- no permanent
  commitment, no wasted locks.

---

## Algorithm 1: Top-Resonance Probability Slots

**One-sentence description:** Each pack slot independently shows a card matching
your most-drafted resonance with probability equal to min(that resonance's
weighted symbol count / 15, 0.75), otherwise a random card.

**Technical description:** Track weighted symbol counts per resonance
(primary=2, secondary/tertiary=1). For each of 4 pack slots, roll against
P = min(top_resonance_count / 15, 0.75). On success, draw a card whose primary
resonance matches the top resonance. On failure, draw from the full pool. The
top resonance is recalculated after each pick.

**Assessment:** Serves convergence (Goal 6) moderately and variance (stddev
from binomial rolls). Fails the 2.0 S/A threshold due to the V4-proven
single-resonance dilution ceiling -- even at P=0.75, only ~50% of
resonance-matched cards are S/A for the specific archetype, yielding
~0.75 * 4 * 0.50 = 1.5 S/A. Simplicity is strong (Goal 1). No player
decisions (Goal 2). Not on rails since probability never hits 100% (Goal 3).

**Best symbol distribution:** Favors 1-symbol cards (simpler counting), but
distribution is not critical since only single resonance matters.

---

## Algorithm 2: Pair-Escalation Slots

**One-sentence description:** Track the resonance pair (first, second symbol)
of each 2+ symbol card you draft; each pack slot independently shows a card
matching your most common pair with probability min(that pair's count / 5,
0.80), otherwise a random card.

**Technical description:** Maintain a pair counter: each drafted card with 2+
symbols increments the count for its (primary, secondary) ordered pair. For
each of 4 pack slots, roll against P = min(top_pair_count / 5, 0.80). On
success, draw a card whose ordered pair matches the top pair. On failure, draw
from the full pool. Pair-matched cards have ~90%+ S/A precision for the target
archetype (since ordered pairs uniquely identify archetypes for 2+ symbol
cards). One-symbol and zero-symbol cards in the pool are never drawn as
pair-targeted cards but can appear in random slots.

**Assessment:** Best candidate for crossing 2.0 S/A. At P=0.80 across 4 slots,
expected targeted slots = 3.2. If ~90% of pair-matched cards are S/A, that is
3.2 * 0.90 = 2.88 S/A -- well above threshold. Variance is strong (binomial
with n=4, p=0.80 gives stddev ~0.80). Slightly more complex to explain (pairs
vs. single resonance) but still one sentence. Early openness preserved because
pair counts start at 0. Pivoting works because pair probabilities decay
relative to the new pair. Signal reading is weak (no pool manipulation).

**Best symbol distribution:** Needs 70-80% of cards with 2+ symbols to ensure
the pair-targeted pool is large enough. Recommended: 15% one-symbol, 60%
two-symbol, 25% three-symbol.

---

## Algorithm 3: Dual-Track Probability (Primary + Pair)

**One-sentence description:** Each pack slot rolls twice: first against your
top resonance pair's count / 6 (capped at 0.70) to show a pair-matched card,
and if that fails, against your top single resonance's count / 20 (capped at
0.40) to show a resonance-matched card; otherwise random.

**Technical description:** Two probability layers per slot. First, check
pair probability (higher precision, ~90% S/A). If that misses, check
single-resonance probability (lower precision, ~50% S/A). This creates a
fallback system: early in the draft when pair counts are low, the
single-resonance layer provides mild convergence; later, the pair layer
dominates. The combined probability of at least one layer firing grows
smoothly.

**Assessment:** Better early convergence than pure pair-matching (single
resonance kicks in first). But complexity fails the simplicity test -- two
probability layers with different caps and divisors in one sentence is too
dense. Convergence should be strong (projected 2.0-2.5 S/A). Variance is
good. The dual mechanism is hard to reason about as a player.

**Best symbol distribution:** Flexible; works with any distribution since both
1-symbol and 2+ symbol cards contribute to different layers.

---

## Algorithm 4: Threshold-Gated Soft Slots

**One-sentence description:** When your weighted symbol count in any resonance
first reaches 3, each pack slot gains a 50% chance of showing a card of that
resonance; at 8 symbols, the chance rises to 75%.

**Technical description:** Like Lane Locking but with probabilities instead of
deterministic locks. Track resonance counters. At threshold 3, set
P_resonance = 0.50. At threshold 8, set P_resonance = 0.75. Each slot
independently rolls against P for the highest-threshold resonance. This
preserves Lane Locking's milestone feel while adding variance. Multiple
resonances can have thresholds -- use the one with highest count for targeting.

**Assessment:** Very simple (Goal 1) -- mirrors Lane Locking's sentence
structure with probabilities replacing locks. No player decisions (Goal 2).
Maintains variance through probabilistic slots (Goal 5/variance target). But
inherits the single-resonance dilution problem: at P=0.75, expected S/A =
4 * 0.75 * 0.50 = 1.50. Below 2.0. Could be enhanced with pair-based
targeting at each threshold, but that complicates the sentence. The discrete
thresholds also create slight "on rails" feel similar to Lane Locking.

**Best symbol distribution:** Same as Lane Locking: heavy 2-symbol majority
to ensure fast threshold crossing.

---

## Algorithm 5: Proportional Pair Slots with Spillover

**One-sentence description:** Your pack's 4 slots are each assigned to your
most-common resonance pair with probability proportional to that pair's share
of your total drafted pairs, and assigned slots show a random card matching
that pair.

**Technical description:** After each pick, compute the proportion of each
pair relative to total pairs drafted. For each slot, roll against the top
pair's proportion. If a player has drafted 6 (Tide, Zephyr) pairs out of 10
total pairs, each slot has a 60% chance of being pair-targeted. Unlike
Algorithm 2's fixed divisor, the probability here is self-normalizing: it
grows as the player commits and shrinks if they diversify. A player with even
pair distribution (2 each across 5 pairs) gets P=0.20 per slot -- nearly
random. A committed player with 8/10 pairs concentrated gets P=0.80.

**Assessment:** Elegant self-scaling without tuning parameters (the divisor is
the player's own history). Convergence: at P=0.80 with pair precision ~90%,
expected 2.88 S/A. Early openness: automatic, because early pair distribution
is diffuse (P stays low). Pivoting: shifting pairs naturally dilutes the old
pair's proportion. Weakness: slower convergence than Algorithm 2 because the
denominator grows with every 2+ symbol card drafted, requiring strong
commitment to reach high proportions. Could feel sluggish -- at pick 10 with
7/10 pairs concentrated, P=0.70 is good, but diversified drafts might never
cross 2.0. No signal reading.

**Best symbol distribution:** Needs 70-80% cards with 2+ symbols, same as
Algorithm 2.

---

## Champion Selection: Algorithm 2 -- Pair-Escalation Slots

**Why this algorithm wins:**

1. **Crosses the 2.0 ceiling.** Pair-based matching at ~90% archetype precision
   eliminates the single-resonance dilution bottleneck that caps Algorithms 1
   and 4 at ~1.5 S/A. At the expected operating point (P=0.60-0.80 per slot
   after commitment), projected S/A is 2.0-2.9.

2. **One tuning knob.** The divisor K in min(pair_count / K, cap) controls
   convergence speed. Algorithm 3's two-layer system and Algorithm 5's
   self-normalizing proportion are harder to tune and reason about.

3. **Clean one-sentence description.** Despite using pairs (slightly more
   complex than single resonances), the description is concrete and
   implementable: count pairs, divide by K, cap at 0.80, roll per slot.

4. **Natural variance.** Four independent binomial rolls at moderate
   probability produce genuine pack-to-pack variance (sometimes 0 targeted
   slots, sometimes 4). StdDev projected 0.80-1.0.

5. **Graceful pivoting.** No permanent state changes. A player who shifts from
   (Tide, Zephyr) to (Tide, Stone) sees the old pair's probability hold steady
   while the new pair ramps up. The system tracks all pairs simultaneously.

Algorithm 5 (Proportional) is the closest competitor but its self-normalizing
denominator makes convergence sluggish for moderately committed players. A
player who drafts 2-3 different archetypes early (reasonable exploration) builds
a large denominator that's hard to overcome. Algorithm 2's fixed divisor K
creates a more predictable ramp.

---

## Champion Deep-Dive: Pair-Escalation Slots

### Example Draft Sequences

**Early committer (Warriors, Tide/Zephyr):**
- Picks 1-3: Drafts [Tide, Zephyr], [Tide], [Tide, Zephyr]. Pair counter:
  (Tide, Zephyr)=2. P = min(2/5, 0.80) = 0.40. Expected targeted slots: 1.6.
  Still mostly random packs with occasional Warriors cards appearing.
- Picks 4-6: Drafts more Tide/Zephyr cards. Pair counter reaches 4-5.
  P = min(5/5, 0.80) = 0.80. Expected targeted slots: 3.2. At 90% archetype
  precision, ~2.88 S/A per pack. Convergence achieved.
- Picks 7-30: Stable at P=0.80 (cap). Packs average 2.5-3.0 S/A with natural
  variance (some packs hit 4/4 targeted, some hit 1-2). The 20% random slot
  chance provides splash and off-archetype cards.

**Flexible player (explores 3 archetypes through pick 8):**
- Picks 1-4: Drafts across Warriors, Storm, Ramp. Pair counters: (Tide,
  Zephyr)=1, (Ember, Stone)=1, (Zephyr, Tide)=1. Top pair P = 0.20. Packs
  are nearly random -- 3.6+ unique archetypes visible.
- Picks 5-8: Gravitates toward Warriors. (Tide, Zephyr) climbs to 3-4.
  P = 0.60-0.80. Convergence begins pick 7-8.
- Picks 9+: Now committed. P stays at cap. Late convergence but still within
  the pick 5-8 target window.

**Pivot attempt (starts Storm, switches to Sacrifice at pick 8):**
- Picks 1-7: Builds (Ember, Stone) to 4. P = 0.80 for Storm cards.
- Pick 8: Decides to pivot to Sacrifice (Tide, Stone). Starts drafting Tide/
  Stone cards. (Ember, Stone) stays at 4, (Tide, Stone) starts climbing.
- Picks 9-12: (Tide, Stone) reaches 3-4. Now the TOP pair might be either
  (Ember, Stone)=4 or (Tide, Stone)=4 -- tied. System uses the one with more
  count (or random tiebreak). Both pairs share Stone secondary, so some
  crossover cards are useful. By pick 14-15, (Tide, Stone) overtakes if the
  player keeps committing. Pivot costs ~6 picks of degraded convergence but
  completes.

### Predicted Failure Modes

1. **One-symbol card dilution.** Cards with only 1 symbol contribute no pairs.
   If a player drafts several 1-symbol cards early, their pair count stays low
   and convergence stalls. Mitigation: ensure the card pool has 70%+ cards with
   2+ symbols so 1-symbol picks are the minority.

2. **Pair pool exhaustion.** The pool of cards matching a specific ordered pair
   is ~40 cards (one archetype's worth). With 4 slots potentially targeting
   this pool every pack, repeated draws could deplete interesting options by
   late draft. Mitigation: drawing with replacement from the full pool (not
   removing targeted cards), or the 360-card pool is large enough that 30 picks
   of 3-4 targeted draws never exhausts 40 cards meaningfully.

3. **Adjacent archetype confusion.** Warriors (Tide, Zephyr) and Ramp (Zephyr,
   Tide) share the same resonances but in opposite order. A player who drafts
   both types builds counts in two different pairs, splitting their probability.
   This is actually a feature -- it prevents accidental over-commitment -- but
   could feel frustrating for a player who wants "Tide and Zephyr cards"
   without caring about order.

4. **Cap reached too early.** If K=5 and a committed player drafts 5 pair
   cards by pick 7, they hit P=0.80 and stay there for 23 more picks. The
   remaining draft feels static. Mitigation: test K=7 or K=8 to slow the ramp.

### Parameter Variants Worth Testing

**Variant A: K=5, cap=0.80 (baseline).** Fast ramp, high cap. Convergence by
pick 5-6 for early committers. Risk: cap reached too early, static late draft.

**Variant B: K=7, cap=0.80 (slow ramp).** Convergence by pick 7-8. More
exploration time. Better for flexible players and signal reading. Risk: might
feel sluggish for players who know what they want immediately.

**Variant C: K=5, cap=0.65 (lower ceiling).** Faster ramp to a lower maximum.
More random slots remain even at full commitment (~1.4 random slots per pack).
Better splash and variance. Risk: at cap 0.65, expected S/A = 4 * 0.65 * 0.90
= 2.34 -- still above 2.0 but with less headroom.

### Proposed Symbol Distribution

| Symbol Count | % of Non-Generic | Cards |
|---|---|---|
| 0 (generic) | -- | 36 |
| 1 symbol | 15% | 49 |
| 2 symbols | 60% | 194 |
| 3 symbols | 25% | 81 |

**Rationale:** The 85% rate of 2+ symbol cards ensures most drafted cards
contribute pairs. The 60% two-symbol majority provides clear archetype identity
through ordered pairs. The 25% three-symbol cards provide richer pair data and
faster ramp for committed players (they still contribute exactly one pair each).
The 15% one-symbol cards exist as flexible "half-committed" options that provide
resonance signal without pair commitment -- useful for early exploration.
