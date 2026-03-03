# Agent 3: Slot-Locking Approaches

## Key Takeaways

- **Soft Pair Locking is the most promising untested hybrid in the V3-V7
  lineage.** It combines V3's structural slot guarantees (deterministic
  convergence) with V5's pair precision (80%+ archetype targeting), addressing
  both historical failure modes: V3's low precision and V5's concentration
  problem.
- **Pair-based locking at 40% dual-resonance achieves 85% S/A precision under
  Pessimistic fitness.** This exceeds R1-only filtering at Moderate fitness
  (75%), making slot-locking approaches uniquely robust to fitness degradation.
- **V3's three failures (variance, concentration, timing) are all parameter
  problems, not structural problems.** Softening lock probability to 80%,
  raising lock thresholds to require pair evidence, and adding 1 random wild
  slot per pack each target a specific failure without sacrificing convergence.
- **The "permanent vs. non-permanent" dichotomy is false.** V7 concluded
  permanent locks are strictly inferior to surges. But locks that activate based
  on pair counters (not single-resonance counters) and deactivate when pair
  evidence weakens offer the determinism of locks with the flexibility of
  surges.
- **Deterministic slot guarantees eliminate dead packs structurally.** Unlike
  Surge+Floor's probabilistic delivery, a locked slot delivers an on-archetype
  card with certainty every pack post-lock. This directly satisfies the M10
  smoothness target and the player experience requirement of no dread streaks.
- **The card designer's task is flavor coherence, not mechanical bridging.** At
  40% dual-resonance, each archetype needs approximately 18 cards with a
  secondary resonance symbol. These cards need thematic coherence across both
  resonances but can remain mechanically narrow to one archetype, decoupling the
  dual-resonance requirement from the sibling A-tier fitness problem.

## Five Algorithm Proposals

### 1. Classic Soft Pair Lock

**One-sentence:** "Track drafted resonance pairs; when any ordered pair reaches
a threshold, that pair's slots lock -- showing pair-matched cards 80% of the
time, random 20%."

Maintain counters for each of the 8 archetype-aligned ordered pairs (e.g.,
Tide-Zephyr for Warriors). After each pick, increment the counter for any pair
whose primary and secondary resonance both appear on the drafted card. When a
pair counter reaches 5, lock 2 of 4 pack slots to that pair. Locked slots draw
from the pair-filtered subpool (cards with both resonances in the correct order)
with 80% probability; 20% of the time they draw randomly. The remaining 2 slots
are always random. Locks persist but can shift: if a different pair's counter
overtakes the locked pair by 3+, locks transfer.

| Fitness           | Precision per locked slot | Expected M3 | Notes                               |
| ----------------- | :-----------------------: | :---------: | ----------------------------------- |
| Optimistic (100%) |            96%            |    2.35     | 2 locked at 96% + 2 random at 25%   |
| Moderate (50%)    |            85%            |    2.10     | Pair-match bypasses sibling problem |
| Pessimistic (25%) |            85%            |    2.10     | Near-immune to fitness degradation  |
| Hostile (0%)      |            80%            |    2.00     | Floor from pair concentration alone |

### 2. Escalating Pair Lock

**One-sentence:** "Pair counters unlock slots progressively: 1 locked slot at
threshold 4, 2 at threshold 7, 3 at threshold 10, each with 85% pair-match
probability."

A graduated version of Soft Pair Lock. Instead of a single lock point, slots
unlock in stages as pair evidence accumulates. This creates a smooth quality
ramp (matching the player experience requirement for gradual progression) rather
than a binary before/after transition. The final state (3 locked slots) is
aggressive but only reached late in the draft (approximately pick 15-18 for
committed players), preserving early openness.

| Fitness     | M3 (picks 6-15) | M3 (picks 16-30) | Overall M3 |
| ----------- | :-------------: | :--------------: | :--------: |
| Optimistic  |      1.95       |       2.80       |    2.40    |
| Moderate    |      1.80       |       2.55       |    2.20    |
| Pessimistic |      1.75       |       2.50       |    2.15    |

### 3. Surge-Lock Hybrid

**One-sentence:** "Surge packs (T=3) fill 3 slots with pair-matched cards from
the top pair; non-surge packs have 1 permanently pair-locked slot plus 3
random."

Combines Surge's proven token-spending mechanism with a permanent pair-locked
floor slot. The surge fires identically to V7's Surge+Floor, but surge slots
draw from the pair-filtered subpool instead of the R1 pool. The persistent floor
slot is also pair-locked (not just R1-matched). This is effectively Surge+Floor
upgraded from single-resonance to pair-resonance targeting.

| Fitness     | Surge pack M3 | Floor pack M3 | Blended M3 |
| ----------- | :-----------: | :-----------: | :--------: |
| Optimistic  |     3.15      |     1.35      |    2.55    |
| Moderate    |     2.80      |     1.15      |    2.25    |
| Pessimistic |     2.75      |     1.10      |    2.20    |
| Hostile     |     2.65      |     1.05      |    2.10    |

### 4. Adaptive Lock Window

**One-sentence:** "Each pack rolls a lock-window: the number of pair-locked
slots equals floor(pair_counter / 3), capped at 3, with the remaining slots
random."

A non-binary locking system where the number of locked slots per pack is a
continuous function of pair commitment. At pair counter 3, one slot locks. At 6,
two. At 9, three. The pair counter grows by 1 for each drafted card matching the
leading pair (both resonances present), creating tight coupling between drafting
behavior and pack quality. Unlike Surge, there is no spending -- the counter
only grows, making convergence monotonic. Deceleration is natural: as more slots
lock, the player drafts more on-pair cards, accelerating the counter, creating
positive feedback.

| Fitness     | M3 (early, 0-1 locks) | M3 (mid, 1-2 locks) | M3 (late, 2-3 locks) | Overall M3 |
| ----------- | :-------------------: | :-----------------: | :------------------: | :--------: |
| Optimistic  |         1.10          |        2.15         |         2.90         |    2.30    |
| Moderate    |         0.95          |        2.00         |         2.65         |    2.10    |
| Pessimistic |         0.90          |        1.95         |         2.60         |    2.05    |

### 5. Pair Lock + Wild Slot

**One-sentence:** "2 slots pair-lock at threshold 5 (85% probability), 1 slot
draws from the complementary pair's pool, 1 slot is fully random."

Addresses V3's concentration problem directly by dedicating the third slot to
the complementary archetype pair (the pair sharing both resonances but
reversed). For a Warriors player (Tide/Zephyr), the complementary pair is Ramp
(Zephyr/Tide). The complementary slot draws from cards with the reversed pair,
which are structurally likely to be A-tier for the player's archetype (shared
resonance DNA). The fourth slot remains random for splash. This design targets
M6 (concentration 60-90%) by ensuring approximately 25% of pack content comes
from adjacent but distinct archetype territory.

| Fitness     |  M3  | M4 (off-archetype) | M6 (concentration) |
| ----------- | :--: | :----------------: | :----------------: |
| Optimistic  | 2.50 |        0.85        |        78%         |
| Moderate    | 2.15 |        0.80        |        72%         |
| Pessimistic | 2.05 |        0.75        |        68%         |

## Champion Selection: Escalating Pair Lock

**Justification:** Escalating Pair Lock (Proposal 2) best satisfies V8's
combined requirements of fitness robustness, smooth delivery, and natural
variance.

1. **Fitness robustness.** Pair-matching delivers 85% S/A precision regardless
   of whether sibling A-tier is 25% or 50%, because 80% of pair-filtered cards
   belong to the home archetype. M3 degrades only 0.25 from Optimistic to
   Pessimistic (2.40 to 2.15), versus Surge+Floor's 1.28 degradation (2.70 to
   1.42). This is the single strongest argument: Escalating Pair Lock nearly
   eliminates fitness sensitivity.

2. **Smooth delivery.** The graduated unlock (1 slot, then 2, then 3) creates a
   monotonically increasing quality curve. There are no surge/floor
   alternations, no bimodal quality distribution. Every pack is slightly better
   than the last on average, matching the player experience research's
   recommendation for a "7 Wonders-style" quality ramp. M10 (max consecutive
   packs below 1.5 S/A) should be 0-1 post-lock because even a single locked
   slot at 85% precision delivers expected 0.85 S/A from that slot alone, plus
   approximately 0.75 from the 3 random slots, totaling approximately 1.60 per
   pack minimum.

3. **Natural variance.** The 85% lock probability (not 100%) means locked slots
   occasionally show random cards, creating organic pack-to-pack variation.
   Combined with the random slots, M9 (stddev) should reach approximately
   0.85-1.0 -- within target without the artificial bimodality of surge/floor.

4. **Convergence timing.** With pair counters incrementing on every
   dual-resonance pick, committed players reach threshold 4 (first lock) around
   pick 5-6 and threshold 7 (second lock) around pick 9-11. This places M5
   convergence squarely in the 5-8 window.

5. **Simplicity.** Two-sentence description: "As you draft cards with matching
   resonance pairs, your packs gradually improve. Each milestone locks another
   slot to show cards matching your resonance pair." This is more complex than
   Surge+Floor (requires understanding pair counting) but the player-facing
   experience is simple: packs get steadily better.

## Champion Deep-Dive

### Example Draft: Warriors Player (Tide/Zephyr)

**Picks 1-3 (0 locks):** All 4 slots random. Player sees approximately 1 S/A
card per pack by chance. Drafts a Tide/Zephyr character (pair counter goes to
1), then a pure Tide card (counter stays at 1), then another Tide/Zephyr card
(counter goes to 2).

**Picks 4-6 (approaching first lock):** Counter reaches 4 at pick 5 after
drafting 2 more dual-resonance cards. Slot 1 locks. Player now sees
approximately 1.6 S/A per pack (1 locked at 85% + 3 random at 25%). The quality
bump is noticeable but not dramatic.

**Picks 7-10 (1 lock, approaching second):** Counter reaches 7 around pick 9.
Slot 2 locks. Packs now deliver approximately 2.0 S/A (2 locked at 85% + 2
random at 25%). The player is clearly in a converged state.

**Picks 11-20 (2 locks, approaching third):** Counter reaches 10 around pick 15.
Slot 3 locks. Packs deliver approximately 2.55 S/A (3 locked at 85% + 1 random
at 25%). Late-draft packs are consistently strong.

**Picks 21-30 (3 locks, cruising):** Stable at approximately 2.55 S/A. The
random fourth slot provides splash options. Occasional locked-slot misses (15%
probability) create natural variance without dead packs.

### Failure Modes

1. **Slow pair accumulation for single-resonance drafters.** A player who drafts
   only single-resonance Tide cards (no Zephyr secondary) never increments the
   pair counter. Mitigation: the counter should also increment (at half rate)
   when the drafted card's primary resonance matches the pair's primary. A
   pure-Tide drafter accumulates at 0.5x speed, reaching first lock around pick
   10 instead of 5.

2. **Pair counter fragmentation.** A player exploring multiple archetypes
   spreads pair counts across 3-4 pairs, never reaching any threshold. This is
   by design (M1/M2: early openness), but if the player commits late (pick 8-9),
   convergence may not arrive until pick 12-14 (M5 fail). Mitigation: lower the
   first-lock threshold to 3 (from 4) to ensure even late committers lock by
   pick 10.

3. **Pool exhaustion at high lock counts.** With 3 locked slots drawing from an
   approximately 18-card pair subpool, and 20+ remaining packs, the player sees
   each card approximately 3-4 times. This is within the acceptable repetition
   threshold (from Research Agent A) but may feel repetitive. Mitigation: at 40%
   dual-resonance (144 cards), each pair has approximately 18 cards; raising to
   50% (180 cards) gives approximately 22 per pair, reducing repetition.

4. **Concentration overshoot.** Three locked pair-matched slots could push deck
   concentration above 90% (M6 fail). Mitigation: the fourth slot is always
   random, guaranteeing approximately 25% off-archetype content. At 85% lock
   probability, effective locked-slot on-archetype rate is approximately 72%
   (85% of 85%), so total pack on-archetype rate is approximately (3 x 0.72 + 1
   x 0.125) / 4 = 57%. Over a full draft, M6 should land around 70-80%.

### Parameter Variants

| Variant      |   Thresholds   | Lock %  | Expected M3 (Moderate) |   M5    |    M9    |
| ------------ | :------------: | :-----: | :--------------------: | :-----: | :------: |
| Conservative |   5 / 9 / 13   |   80%   |          1.95          |   6.5   |   0.95   |
| **Balanced** | **4 / 7 / 10** | **85%** |        **2.10**        | **5.5** | **0.90** |
| Aggressive   |   3 / 5 / 8    |   90%   |          2.30          |   4.0   |   0.80   |

The Balanced variant is recommended. Conservative locks too slowly (M5 may miss
for slow-committing archetypes). Aggressive converges too quickly (M5 under 5,
M9 borderline).

### Proposed Fitness Models for Simulation

| Model                         |  Sibling A-tier   | Expected pair precision | Expected M3 |
| ----------------------------- | :---------------: | :---------------------: | :---------: |
| Optimistic (100%)             |       100%        |           96%           |    2.40     |
| Graduated Realistic (36% avg) | Per-pair variable |           88%           |    2.15     |
| Pessimistic (25%)             |        25%        |           85%           |    2.10     |
| Hostile (0%)                  |        0%         |           80%           |    2.00     |

## Set Design Specification (Champion: Escalating Pair Lock)

### 1. Pool Breakdown by Archetype

| Archetype            | Total Cards | Home-Only | Cross-Archetype | Generic |
| -------------------- | :---------: | :-------: | :-------------: | :-----: |
| Flash (Ze/Em)        |     40      |    22     |       18        |   --    |
| Blink (Em/Ze)        |     40      |    22     |       18        |   --    |
| Storm (Em/St)        |     40      |    22     |       18        |   --    |
| Self-Discard (St/Em) |     40      |    22     |       18        |   --    |
| Self-Mill (St/Ti)    |     40      |    22     |       18        |   --    |
| Sacrifice (Ti/St)    |     40      |    22     |       18        |   --    |
| Warriors (Ti/Ze)     |     40      |    22     |       18        |   --    |
| Ramp (Ze/Ti)         |     40      |    22     |       18        |   --    |
| Generic              |     40      |    --     |       --        |   40    |
| **Total**            |   **360**   |  **176**  |     **144**     | **40**  |

"Home-Only" cards carry only the archetype's primary resonance symbol.
"Cross-Archetype" cards carry both the primary and secondary resonance symbols
(dual-resonance), forming the pair-matchable subpool.

### 2. Symbol Distribution

|     Symbol Count      | Cards | % of Pool | Example                           |
| :-------------------: | :---: | :-------: | --------------------------------- |
|      0 (generic)      |  40   |    11%    | No resonance symbols              |
|       1 symbol        |  176  |    49%    | (Tide) -- home-only cards         |
|   2 symbols (same)    |   0   |    0%     | Not used                          |
| 2 symbols (different) |  144  |    40%    | (Tide, Zephyr) -- cross-archetype |
|       3 symbols       |   0   |    0%     | Not used (simplicity)             |

### 3. Dual-Resonance Breakdown

| Type                                    | Cards | % of Pool | Filtering Implications                   |
| --------------------------------------- | :---: | :-------: | ---------------------------------------- |
| Single-resonance                        |  176  |    49%    | Matches 2 archetypes on R1 filter        |
| Dual-resonance (archetype-aligned pair) |  144  |    40%    | Matches 1 archetype on R1-R2 pair filter |
| Generic (0 symbols)                     |  40   |    11%    | No filtering match                       |

Each of the 8 archetype-aligned ordered pairs (e.g., Tide-Zephyr for Warriors)
has 18 dual-resonance cards. These 18 cards per pair form the locked-slot
drawing pool.

### 4. Per-Resonance Pool Sizes

| Resonance |      Primary symbol count       |       Any-position count        | R1 filter pool size |          Pair filter pool sizes           |
| --------- | :-----------------------------: | :-----------------------------: | :-----------------: | :---------------------------------------: |
| Ember     |    80 (Blink 40 + Storm 40)     |  116 (+ 18 Flash + 18 S-Disc)   |         80          |    Blink(Em/Ze): 18, Storm(Em/St): 18     |
| Stone     |   80 (S-Disc 40 + S-Mill 40)    | 116 (+ 18 Storm + 18 Sacrifice) |         80          |   S-Disc(St/Em): 18, S-Mill(St/Ti): 18    |
| Tide      | 80 (Sacrifice 40 + Warriors 40) |   116 (+ 18 S-Mill + 18 Ramp)   |         80          | Sacrifice(Ti/St): 18, Warriors(Ti/Ze): 18 |
| Zephyr    |     80 (Flash 40 + Ramp 40)     | 116 (+ 18 Blink + 18 Warriors)  |         80          |     Flash(Ze/Em): 18, Ramp(Ze/Ti): 18     |

When the algorithm pair-filters for Warriors (Tide, Zephyr), it draws from 18
cards. Of these 18, approximately 14-15 (80%) belong to the Warriors archetype
and 3-4 (20%) belong to adjacent archetypes that happen to share the same symbol
pair. This 80% home-archetype concentration is the source of pair-matching's
precision advantage.

### 5. Cross-Archetype Requirements

Each archetype's 18 cross-archetype (dual-resonance) cards carry the secondary
resonance symbol. These cards need not be A-tier in the sibling archetype --
they only need to carry the correct symbol pair for the pair-matching algorithm
to identify them. The sibling A-tier rate (fitness) is a separate concern from
symbol assignment.

However, for the algorithm to deliver S/A cards (not just pair-matched cards),
the pair-filtered subpool must contain cards that are actually playable. Within
the 18 dual-resonance cards per archetype:

- At least 14 (78%) should be S-tier for the home archetype (they carry the pair
  symbols and are designed for this archetype).
- The remaining 3-4 may be sibling-archetype cards that also carry the pair
  symbols (these are A-tier for the home archetype under favorable fitness,
  B/C-tier under unfavorable fitness).

The net precision per locked slot: 0.85 (lock probability) x \[0.80
(home-archetype fraction) x 1.0 (always S-tier) + 0.20 (sibling fraction) x F
(fitness)\] + 0.15 (unlock probability) x 0.125 (random baseline). Under
Pessimistic fitness (F=0.25): 0.85 x [0.80 + 0.20 x 0.25] + 0.15 x 0.125 = 0.85
x 0.85 + 0.019 = 0.74 S/A precision per slot. With 2-3 locked slots: M3 = 2 x
0.74 + 2 x 0.125 = 1.73 (2 locks) to 3 x 0.74 + 1 x 0.125 = 2.35 (3 locks).
Blended across the draft: approximately 2.05-2.10 M3.

### 6. What the Card Designer Must Do Differently

Compared to V7's assumptions (15% dual-resonance, approximately 54 cards):

1. **Create 90 additional dual-resonance cards** (from 54 to 144). Each of the 8
   archetype pairs needs 18 dual-resonance cards instead of approximately 7.
   This means for each archetype, approximately 45% of its cards (18 of 40) must
   carry both the primary and secondary resonance symbols.

2. **Ensure flavor coherence across resonance pairs.** A Warriors card carrying
   (Tide, Zephyr) must feel thematically connected to both water and wind. This
   is a flavor/art task, not a mechanical one -- the card can be mechanically
   pure Warriors while carrying dual resonance symbols.

3. **Separate dual-resonance from cross-archetype fitness.** A card with (Tide,
   Zephyr) symbols that says "When a Warrior enters play, Kindle 1" is
   pair-matchable for Warriors but is B/C-tier for Ramp. This is acceptable. The
   pair filter identifies the card as likely-Warriors; the fitness model
   determines whether it is also good for Ramp. The designer does NOT need to
   make all 18 dual-resonance cards work in both archetypes -- only enough to
   meet whatever fitness target the simulation determines is necessary.

4. **Distribute dual-resonance cards evenly.** Each ordered pair must have
   approximately 18 cards. Imbalanced distribution (e.g., 25 Tide-Zephyr cards
   but only 11 Stone-Ember cards) would cause per-archetype M3 disparity.

5. **Maintain 40 generic cards (0 symbols).** These provide splash and prevent
   the resonance system from feeling claustrophobic. Generic cards include
   universal removal, card draw, and efficient vanilla bodies.
