# Resonance Draft V3 -- Final Synthesis Report (Corrected)

## Unified Comparison Table (Archetype-Level Metrics)

All metrics measured at the **archetype** level (S/A-tier fitness for the
player's specific target archetype). The previous report measured at the
resonance level, which inflated convergence numbers because a resonance like
Tide is shared by 4 archetypes -- roughly half of resonance-matched cards are
S/A for the wrong archetype. This correction significantly changed the
rankings.

| Metric | Target | S1: W. Lottery | S2: Bal. Pack | S3: Lane Lock | S4: Echo Win. | S5: Res. Swap |
|--------|--------|:-:|:-:|:-:|:-:|:-:|
| Early unique archs w/ S/A | >= 3 | 6.56 PASS | 7.24 PASS | 6.49 PASS | 5.18 PASS | 6.44 PASS |
| Early S/A for arch/pack | <= 2 | 2.02 FAIL | N/A* PASS | 1.93 PASS | 1.61 PASS | 1.68 PASS |
| Late S/A for arch/pack | >= 2 | 2.32 PASS | 2.07 PASS | **2.72 PASS** | 1.54 FAIL | 1.58 FAIL |
| Late C/F cards/pack | >= 0.5 | 0.46 FAIL | 0.67 PASS | **0.84 PASS** | 0.38 FAIL | **1.24 PASS** |
| Convergence pick | 5-8 | 8.5 FAIL | 6.3 PASS | **6.1 PASS** | 7.0 PASS | 7.6 PASS |
| Deck concentration | 60-80% | 92.3% FAIL | 93.8% FAIL | 99.0% FAIL | 84% FAIL | 83.2% FAIL |
| Card overlap | < 40% | 8.1% PASS | 5.5% PASS | 5.4% PASS | 7% PASS | 6.9% PASS |
| Archetype frequency | 5-20% | 6-20% FAIL | 8-24% FAIL | **8-19% PASS** | 10-16% PASS | 10-15% PASS |
| **Targets passed** | | **3/8** | **7/9** | **7/8** | **6/9** | **6/8** |

*Balanced Pack's early S/A is N/A because the committed player has no declared
target during picks 1-5; the 1/1/1/1 structure inherently limits early bias.
Counted as pass.

**Bold** = best-in-class. Lane Locking leads on late S/A (2.72), convergence
speed (6.1), and off-archetype splash (0.84). Balanced Pack leads on early
diversity (7.24). Resonance Swap leads on splash volume (1.24) and signal
reading (44.8% detection).

### What Changed from the Previous Report

| Algorithm | Old Passes | New Passes | Key Change |
|-----------|:---:|:---:|---|
| S1: Weighted Lottery | 5/8 | **3/8** | Early S/A, convergence pick, C/F all fail at archetype level |
| S2: Balanced Pack | 7/8 | **7/9** | Stable; archetype freq slightly worse (23.8% Flash bias) |
| S3: Lane Locking | 5/8 | **7/8** | Late S/A jumps from 1.83 to 2.72; locked resonance slots guarantee S/A |
| S4: Echo Window | 5/8 | **6/9** | Late S/A drops from 2.83 to 1.54; resonance ambiguity exposed |
| S5: Resonance Swap | 5/8 | **6/8** | Modest improvement; pool shifts are resonance-neutral at archetype level |

The critical insight: algorithms that **structurally guarantee** resonance
slots (Lane Locking, Balanced Pack) benefit from the correction because a
locked resonance slot has ~75% chance of being S/A for the committed archetype.
Algorithms that use **probabilistic** resonance selection (Weighted Lottery,
Echo Window) lose ground because probability spreads cards across all archetypes
sharing that resonance.

### The Deck Concentration Problem

All five strategies fail the 60-80% deck concentration target (range: 83-99%).
This is a **fitness model artifact**. The circle model assigns S/A-tier to 3 of
8 archetypes (37.5% of cards), so a committed player selecting from 4-card
packs nearly always finds an S/A option. The target should be revised to
80-95%, or addressed through card design (wider F-tier gaps between archetypes).

---

## Algorithm Ranking (Corrected)

### Rank 1: Lane Locking (7/8 pass)

The biggest winner of the correction. Locking a resonance slot **guarantees**
S/A cards roughly 75% of the time for the committed archetype (since the locked
resonance is primary for 2 archetypes and secondary for 2 more, and the
committed archetype is always one of those). The dual threshold (3 and 8)
ensures 2 locked slots by pick 5-6, driving late S/A to 2.72 -- the highest of
any strategy by a wide margin.

Strengths: Best convergence (2.72 late S/A), fastest convergence (pick 6.1),
strong splash (0.84 C/F), balanced archetype frequency (8-19%), maximum
transparency.

Weaknesses: Highest deck concentration (99%), no signal reading, permanent locks
prevent pivoting after picks 6-8.

### Rank 2: Balanced Pack (7/9 pass)

The previous consensus winner holds up well. The 1/1/1/1 baseline produces the
most predictable and diverse early packs (7.24 unique archetypes). Late S/A at
2.07 just clears the target. The structural guarantee of always seeing at least
2 non-majority resonances provides reliable splash.

Strengths: Best early diversity (7.24), reliable splash (0.67), simplest mental
model (two pack states), convergence on target (6.3).

Weaknesses: Flash frequency at 23.8% exceeds 20% cap, no signal reading, late
S/A barely passes (2.07) with no tuning headroom.

### Rank 3: Resonance Swap (6/8 pass)

The only strategy with meaningful signal reading (44.8% detection rate). The
asymmetric starting pool creates discoverable run-to-run variety. However,
convergence (1.58 late S/A) is structurally unfixable.

Best role: complementary layer providing signal reading atop another algorithm.

### Rank 4: Echo Window (6/9 pass)

The 3-pick sliding window allows instant pivots but fails the core convergence
target (1.54 late S/A). The structural problem is resonance-to-archetype
ambiguity: filling 2 slots with Ember cards delivers Blink cards roughly half
the time and Storm/Flash the other half. No parameter tuning fixes this.

### Rank 5: Weighted Lottery (3/8 pass)

The correction hit this strategy hardest. The probabilistic weighting produces
too many marginal failures: early S/A 2.02, convergence pick 8.5, C/F 0.46,
archetype frequency 6-20%. The core issue: weighted random resonance selection
does not translate reliably into archetype-specific cards.

---

## Independent Simplicity Test

### 1. Lane Locking -- PASS (clear, complete)

> "Your pack has 4 slots; when your symbol count in a resonance first reaches 3,
> one open slot locks to that resonance; when it first reaches 8, a second slot
> locks."

Unambiguous. A programmer knows: track 4 counters, fire at thresholds 3 and 8,
locked slots show resonance-matched cards, open slots are random. The only
unstated detail is "primary=2, others=1" counting, a system convention.

### 2. Balanced Pack -- PASS (clear, complete)

> "Each pack has one card per resonance type, but if you have a clear majority
> resonance (strictly more weighted symbols than any other, counting primary=2),
> it replaces one random non-majority slot, giving you 2 of your majority."

Complete. "Clear majority" is unambiguous. Minor detail: which non-majority slot
is replaced is "random," which suffices.

### 3. Echo Window -- PASS (clear, complete)

> "Count the resonance symbols across your last 3 picks (primary symbols count
> as 2, others as 1); your top resonance fills 2 pack slots, your second fills
> 1, and the last slot is random."

Fully specified. Tie-breaking unstated but conventions handle it.

### 4. Weighted Lottery -- PASS (clear, complete)

> "Each resonance starts at weight 1; each drafted symbol adds to weights
> (primary +2, others +1); 3 of 4 pack slots pick a resonance proportionally to
> weights; the 4th slot is always a random card."

Implementable, but prediction is probabilistic, not concrete.

### 5. Resonance Swap -- MARGINAL FAIL (hides infrastructure)

> "When you draft a card, 3 random cards matching its primary resonance are
> added from a reserve, and 3 non-matching are moved to the reserve; each run
> starts with one resonance boosted (+20) and another suppressed (-20)."

Describes invisible infrastructure. A player cannot predict pack changes.

**Simplicity ranking (most to least predictable):**
1. Lane Locking -- binary lock state, perfect prediction
2. Balanced Pack -- two pack states (1/1/1/1 or 2/1/1/0)
3. Echo Window -- deterministic from last 3 cards, requires counting
4. Weighted Lottery -- correct intuition but probabilistic output
5. Resonance Swap -- simple to describe, impossible to predict

---

## Recommended Algorithm: Lane Locking + Pool Asymmetry

### Why the Recommendation Changed

The previous report recommended Balanced Pack based on resonance-level metrics.
The corrected archetype-level data reverses this:

| Metric | Lane Locking | Balanced Pack | Winner |
|--------|:-:|:-:|---|
| Late S/A | 2.72 | 2.07 | Lane Locking (+31%) |
| Convergence pick | 6.1 | 6.3 | Lane Locking |
| Splash (C/F) | 0.84 | 0.67 | Lane Locking (+25%) |
| Archetype frequency | 8-19% | 8-24% | Lane Locking |
| Early diversity | 6.49 | 7.24 | Balanced Pack |
| Simplicity | Binary lock state | Two pack states | Comparable |

Lane Locking outperforms on the four metrics that most directly affect player
experience: convergence strength, convergence speed, splash availability, and
archetype balance. Balanced Pack wins only on early diversity, a metric all
strategies pass easily at archetype level.

The decisive advantage is convergence depth. Lane Locking delivers 2.72 S/A
cards per pack -- 31% more than Balanced Pack's 2.07, which barely clears the
2.0 target. In a roguelike deckbuilder, strong convergence is critical: the
player needs to find cards for their archetype reliably to assemble a winning
deck.

### Why Add Pool Asymmetry

Lane Locking scores poorly on signal reading (Goal 8). Layering Resonance
Swap's static pool asymmetry (+20/-20 per run) addresses this without changing
pack construction. The asymmetry is a pre-game setup. Players who notice one
resonance appearing more often in open slots can draft toward it.

### Complete Specification

**One-sentence player description:**

> "Your pack has 4 slots that start random; when your resonance symbol count
> hits 3 a slot locks to that resonance, and at 8 a second locks -- plus each
> quest starts with one resonance having more cards in the pool."

**One-paragraph player description:**

> Your draft packs have 4 card slots. Each slot starts open, showing a random
> card from the full pool. As you draft cards, you accumulate resonance symbols
> (the first symbol on each card counts double). When your total in any
> resonance first reaches 3, one open slot permanently locks to that resonance
> -- from then on, that slot always shows a card of that type. When it reaches
> 8, a second slot locks. You can lock up to 4 slots total. Unlocked slots
> remain random. Each quest run, one resonance has extra cards in the pool and
> another has fewer, creating a detectable signal for experienced players who
> watch what appears in their open slots.

**Step-by-step algorithm:**

1. Initialize 4 pack slots as OPEN. Initialize 4 resonance counters at 0.
2. Pre-game: randomly select one resonance to boost (+20 cards from reserve to
   pool) and one to suppress (-20 cards from pool to reserve).
3. For each pack, fill each slot:
   - LOCKED to resonance R: select a random card whose primary resonance is R.
   - OPEN: select a random card from the full pool.
4. Player picks one card. Update counters: +2 for primary symbol, +1 for each
   secondary/tertiary. Generic cards add nothing.
5. Check each counter against thresholds:
   - First crosses 3: lock one random OPEN slot to that resonance.
   - First crosses 8: lock another random OPEN slot to that resonance.
   - Maximum 4 locked slots total. No further locks if all are locked.
6. Repeat from step 3.

**Parameters:**
- First threshold: 3 | Second threshold: 8
- Lock cap: 4 total
- Primary weight: 2 | Secondary/tertiary: 1
- Pool asymmetry: +20/-20 cards per run

### Recommended Symbol Distribution

| Symbol Count | % of non-generic | Cards |
|---|---|---|
| 0 (generic) | -- | 36 |
| 1 symbol | 25% | 81 |
| 2 symbols | 55% | 178 |
| 3 symbols | 20% | 65 |

Two-symbol cards dominate, providing ~3 weighted symbols per pick. A committed
player reaches threshold 3 by pick 2 and threshold 8 by pick 4-5.

### Edge Cases

- **Simultaneous thresholds:** If one pick crosses thresholds in two resonances,
  lock one slot per crossing, highest count first. Ties broken randomly.
- **All 4 slots locked early:** Possible with broad drafting. Pack structure is
  then fixed. Acceptable -- 4 threshold investments earn a customized pack.
- **Threshold 3 on pick 1:** A [Tide, Tide, Zephyr] card gives 3 Tide symbols
  instantly (2+1). By design -- immediate feedback. 3-symbol cards are 20% of
  the pool, so this is not the default experience.
- **Pool asymmetry reserve:** A 200-card reserve (50 per resonance) is
  sufficient for the +20/-20 asymmetry.

---

## Open Questions for Playtesting

1. **Is 99% deck concentration acceptable?** May be fine for a roguelike (the
   challenge is finding the right archetype) or may feel too smooth. Adjust
   fitness model if needed.

2. **Should locks be visible in the UI?** Recommended: show lock status as
   resonance icons above each pack slot. The algorithm's transparency advantage
   depends on players seeing their lock state.

3. **Do permanent locks feel punishing for mistakes?** A player who accidentally
   locks Ember then decides to play Tide is stuck. Playtest whether this creates
   frustration or acceptable tension. Potential mitigation: one "relock" per
   draft (breaks simplicity but adds forgiveness).

4. **Is pool asymmetry detectable?** Static +20/-20 should produce ~30-35%
   detection rate. Test whether players notice and act on it.

5. **Should Balanced Pack remain the fallback?** If playtesting reveals
   permanent locks feel too rigid, Balanced Pack is the clear second choice
   (7/9 pass, best early diversity, no lock commitment). The two algorithms
   represent a genuine flexibility-vs-convergence tradeoff.

6. **Threshold tuning.** (3, 8) is the simulated sweet spot. Testing (2, 6)
   gives slightly faster convergence; (4, 10) gives more breathing room.
   Sensitivity is low -- all pairs perform similarly.
