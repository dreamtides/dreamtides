# Agent 9: Open Exploration -- Narrative Gravity

## Key Takeaways

1. **V3-V7 all fight the same battle: improving precision of slot-level
   targeting.** Every algorithm asks "how do I put the right card in a slot?"
   This framing is inherently limited because precision is bounded by pool
   composition and fitness. A fundamentally different question: "how do I shape
   the pool the slots draw from?"

2. **Roguelike deckbuilders solve this problem differently.** Slay the Spire,
   Monster Train, and Inscryption do not manipulate individual offers -- they
   shrink or reshape the candidate pool itself over time. The player's choices
   progressively narrow what future offers can contain.

3. **Low cross-archetype fitness is not a bug if the pool shrinks fast enough.**
   If sibling cards are mostly B/C-tier, the answer is not to present them more
   precisely but to present fewer of them. A pool that contracts toward the home
   archetype as the draft progresses sidesteps the fitness problem entirely.

4. **"Draft quality" can be reframed as pool relevance rate.** Instead of
   measuring how many S/A cards the algorithm places in slots, measure what
   fraction of the drawable pool is S/A for the player. An algorithm that raises
   pool relevance delivers smooth, organic quality without bimodal surge/floor
   patterns.

5. **Progressive pool contraction produces naturally smooth quality curves.**
   Because every pack draws from the same (shrinking, increasingly relevant)
   pool, there is no structural distinction between "surge" and "floor" packs.
   Variance comes from random draw within the pool, not from algorithmic
   mode-switching.

6. **The mechanism class is Pool Sculpting -- but continuous, not
   probabilistic.** V6 tested pool sculpting as weight-boosting (still drawing
   from the full pool with bias). The approach here is actual removal: cards
   outside the player's emerging identity become ineligible, reducing noise at
   the source.

7. **This approach works WITH low fitness rather than against it.** When 75% of
   sibling cards are B/C-tier, removing them from the pool is more valuable than
   trying to present the 25% that are A-tier.

______________________________________________________________________

## Five Algorithm Proposals

### 1. Narrative Gravity (Pool Contraction via Resonance Focus)

**One sentence:** After each pick, the drawable pool permanently shrinks by
removing cards whose resonance profile is most distant from the player's
accumulated resonance signature.

**Technical description:** Maintain a resonance signature vector (4 floats, one
per resonance, updated by drafted card symbols with +2 primary / +1 secondary).
After each pick, compute a "relevance score" for every card in the pool: dot
product of the card's symbol vector with the player's normalized signature.
Remove the bottom P% of the pool (by relevance score), where P starts at 0%
(picks 1-3), ramps to 3% per pick (picks 4-8), then stabilizes at 2% per pick
(picks 9-30). All 4 pack slots draw uniformly from the surviving pool. No
targeted slots, no surges, no modes.

**Predicted behavior:** Optimistic (F=100%): M3 ~2.3. The pool contracts to
mostly R1 cards by pick 10; all 4 slots draw from a pool that is ~70%
home+sibling S/A. Pessimistic (F=25%): M3 ~1.7. Contraction removes sibling B/C
cards, concentrating the pool on home-archetype cards. Hostile (F=0%): M3 ~1.5.
Pool shrinks to ~50% home-only cards; every draw has 50% S-tier chance across
all 4 slots.

### 2. Archetype Crystallization (Discrete Pool Phases)

**One sentence:** The pool starts as the full 360-card set and transitions
through 3 discrete phases -- Open (full pool), Focused (R1-filtered, ~180
cards), Crystallized (pair-filtered, ~80 cards) -- triggered by resonance
commitment thresholds.

**Technical description:** Phase 1 (picks 1-5): draw from full pool. When any
resonance counter reaches 6, enter Phase 2: pool narrows to cards containing the
dominant resonance as any symbol (~180 cards, 4 archetypes represented). When
the top two resonances each reach 8, enter Phase 3: pool narrows to cards
containing both the top two resonances (~80 cards, primarily 1-2 archetypes).
All 4 pack slots draw uniformly from the current phase pool.

**Predicted behavior:** Optimistic: M3 ~2.5 (Phase 3 pool is ~80% S/A).
Pessimistic: M3 ~1.9 (Phase 3 pool is ~65% home archetype, sibling cards present
but small fraction are A-tier; net ~55% S/A across 4 slots = 2.2, minus some
phase transition timing loss). Harsh (F=15%): M3 ~1.6.

### 3. Drift Lanes (Soft Pool Partitioning)

**One sentence:** The pool is partitioned into 4 resonance "lanes"; each pack
draws 1 card from each lane, but lane sizes shift dynamically based on the
player's drafting -- the dominant resonance lane grows while others shrink.

**Technical description:** Maintain 4 lane weights (initially equal at 25%
each). After each pick, increase the drafted card's primary resonance lane by 5%
and decrease all others proportionally. Each pack draws 1 card from each lane
(the lane weight determines what fraction of the full pool that lane contains,
implemented as a weighted random draw biased toward that resonance). This is NOT
slot-filling (cards are not guaranteed to match the lane resonance) -- it is a
soft pool partitioning where larger lanes have more candidates.

**Predicted behavior:** Optimistic: M3 ~2.0. Pessimistic: M3 ~1.5. Hostile: M3
~1.2. Smooth delivery but modest ceiling because the partitioning is soft.

### 4. Echo Draft (History-Based Pool Weighting)

**One sentence:** Each card in the pool accumulates an "echo score" based on how
similar it is to previously drafted cards (shared resonance symbols, shared
subtypes); packs are drawn with probability proportional to echo score.

**Technical description:** Every card starts with echo score 1.0. After each
pick, increase the echo score of every pool card that shares at least 1
resonance symbol with the picked card by +0.5 (shared primary: +1.0). Cards that
share 2+ symbols get +1.5. Pack slots draw weighted-random from the pool using
echo scores as weights. No cards are removed; low-echo cards become increasingly
unlikely but never impossible.

**Predicted behavior:** Optimistic: M3 ~2.1 (echo scores concentrate draws on
home+sibling cards). Pessimistic: M3 ~1.6 (echo weighting helps but sibling B/C
cards still drawn occasionally). Hostile: M3 ~1.3. Very smooth delivery; no
mode-switching. The weighting is continuous, not discrete.

### 5. Slay the Spire Shop Model (Curated Micro-Pools)

**One sentence:** Instead of drawing from the full pool, each pack draws from a
procedurally generated "shop" of 12-16 cards pre-filtered for relevance to the
player's resonance profile, with 4 cards shown from the shop.

**Technical description:** Before generating each pack, construct a micro-pool:
50% of the micro-pool is drawn from cards matching the player's top resonance
(R1 filter), 25% from cards matching the second resonance, 25% random. Then draw
4 cards uniformly from this micro-pool. The micro-pool is regenerated each pack.
This is equivalent to Surge's slot-filling but with an intermediate pooling step
that creates natural variance.

**Predicted behavior:** Optimistic: M3 ~2.3. Pessimistic: M3 ~1.7. Harsh: M3
~1.5. Smooth because the micro-pool composition varies naturally. Essentially a
softer version of slot-filling with built-in randomness.

______________________________________________________________________

## Champion Selection: Narrative Gravity (Proposal 1)

**Justification:** Narrative Gravity is the only proposal that introduces a
genuinely novel mechanism class -- continuous pool contraction. It addresses the
core V8 problems simultaneously:

- **Smooth delivery:** No surge/floor modes. Every pack draws from the same
  pool, producing a unimodal quality distribution. M10 is structurally
  satisfied.
- **Fitness robustness:** Instead of trying to identify the 25% of sibling cards
  that are A-tier (Pessimistic), it removes the 75% that are B/C/F. The result
  is the same (player sees mostly S/A cards) but the mechanism is subtractive
  rather than selective.
- **Intuitive experience:** The player perceives "my packs keep getting better"
  without understanding the mechanism. The feedback loop is legible: draft Tide
  cards, see more Tide cards. The pool contraction is invisible.
- **Not on rails:** Early picks (1-5) use the full pool. Contraction is gradual.
  The player can pivot until pick 8-10, after which the pool has narrowed enough
  that pivoting is costly but not impossible (the pool still contains ~60% of
  original cards at pick 10).

Archetype Crystallization (Proposal 2) is the runner-up but suffers from
discrete phase transitions that create perceptible quality jumps -- the opposite
of smooth delivery.

______________________________________________________________________

## Champion Deep-Dive: Narrative Gravity

### Mechanism Detail

**Resonance signature:** A 4-element vector [Ember, Stone, Tide, Zephyr],
starting at [0,0,0,0]. After drafting a card with symbols (Tide, Zephyr), add
[0, 0, 2, 1] (primary +2, secondary +1). After 6 picks of Warriors-aligned
cards, signature might be [0, 0, 10, 5].

**Relevance scoring:** For each pool card, compute relevance =
dot(card_symbols_normalized, player_signature_normalized). A card with (Tide,
Zephyr) scores high for a Warriors drafter. A card with (Ember, Stone) scores
near zero.

**Contraction schedule:**

- Picks 1-3: No contraction. Full pool (360 cards).
- Picks 4-8: Remove bottom 3% per pick. Pool at pick 8: ~310 cards.
- Picks 9-30: Remove bottom 2% per pick. Pool at pick 30: ~170 cards.

**What gets removed:** Cards with zero resonance overlap with the player's
signature are removed first. By pick 10, most cards from the two resonances
opposite the player's primary are gone. By pick 20, the pool is dominated by
cards with the player's primary resonance, plus generics.

### Example Draft: Warriors Player (Tide/Zephyr)

Pick 1-3: Full pool. Sees a mix of all 8 archetypes. Drafts 2 Tide cards and 1
generic. Pick 4: Signature = [0, 0, 5, 1]. Bottom 3% removed: ~11 cards, mostly
pure Ember or pure Stone cards with no Tide/Zephyr symbols. Pool: 349. Pick 5-6:
Continues drafting Tide/Zephyr. Signature grows. Pool contracts to ~330. Pure
Ember and Stone cards increasingly absent. Pick 8: Pool ~310. Cards remaining
are predominantly Tide-bearing, Zephyr-bearing, Tide/Zephyr dual, and generics.
Some Stone/Tide and Ember/Zephyr cards survive (they share one resonance with
the player). Pick 15: Pool ~250. Dominated by Tide and Zephyr cards. Packs feel
consistently on-theme. Pick 25: Pool ~190. Almost entirely Tide-primary,
Zephyr-primary, and dual Tide/Zephyr cards plus generics. S/A rate per pack:
~2.0-2.5 depending on fitness.

### Example Draft: Flash Player (Zephyr/Ember) Under Pessimistic Fitness

Flash/Ramp is the worst sibling pair (10-20% A-tier). Under old algorithms, this
player suffers most.

Under Narrative Gravity: By pick 15, the pool has contracted to ~250 cards. Ramp
cards (Zephyr/Tide) that are B/C-tier for Flash have been partially removed
(their Tide component scores low against the Flash player's \[Ember-heavy,
Zephyr-heavy\] signature). The surviving Zephyr cards are disproportionately
Flash-aligned or generic. S/A rate: ~1.6-1.8. Not as high as Warriors (~2.0-2.2)
but the gap is smaller than under Surge+Floor because contraction removes the
worst sibling cards rather than presenting them.

### Failure Modes

1. **Pivot penalty.** A player who drafts Tide for 8 picks then tries to pivot
   to Ember finds a contracted pool with few Ember cards remaining. Mitigation:
   contraction rate is slow enough that at pick 8, ~85% of the pool survives.
   Pivoting is costly but possible.

2. **Generic card depletion.** Generic cards (0 symbols) have relevance score 0
   against any signature and get removed early. Mitigation: Assign generic cards
   a baseline relevance of 0.3, ensuring they survive until late-draft
   contraction.

3. **Signal reading weakness.** Because the pool contracts based on the player's
   own drafting (not on what is "open"), signal reading provides less benefit
   than under Surge. The player shapes their own pool rather than reading the
   environment. Mitigation: this is a design choice, not a failure. In a
   single-player roguelike, signal reading is less important than in competitive
   drafts.

4. **Under-contraction (early draft feels random).** Picks 1-5 draw from the
   full pool with no contraction, potentially delivering low S/A. Mitigation:
   this matches the M1/M2 targets (early openness). The player is exploring, not
   converging.

### Parameter Variants

| Variant          | Contraction Rate | Pool at Pick 15 | Pool at Pick 30 | M3 (est. Pessimistic) |
| ---------------- | :--------------: | :-------------: | :-------------: | :-------------------: |
| Conservative     |    1.5%/pick     |       290       |       220       |         ~1.4          |
| Standard         |    2-3%/pick     |       250       |       170       |         ~1.7          |
| Aggressive       |     4%/pick      |       200       |       120       |         ~1.9          |
| Ultra-Aggressive |     5%/pick      |       170       |       90        |  ~2.0+ (M6/M7 risk)   |

The standard variant balances convergence against flexibility. Aggressive is the
highest-performing variant that likely passes M6/M7; ultra-aggressive risks
excessive concentration and low run-to-run variety.

### Fitness Model Predictions

|       Model       | Sibling A-Tier | Est. M3 (Standard) | Est. M3 (Aggressive) | Rationale                                              |
| :---------------: | :------------: | :----------------: | :------------------: | ------------------------------------------------------ |
| Optimistic (100%) |      100%      |        ~2.3        |         ~2.6         | Pool contraction concentrates an already-good pool     |
|  Moderate (50%)   |      50%       |        ~2.0        |         ~2.3         | B/C siblings removed; surviving pool mostly S/A        |
| Pessimistic (25%) |      25%       |        ~1.7        |         ~1.9         | More siblings to remove, but contraction handles it    |
|    Harsh (15%)    |      15%       |        ~1.5        |         ~1.7         | Contraction removes most siblings; home cards dominate |
|   Hostile (0%)    |       0%       |        ~1.3        |         ~1.5         | Pool shrinks to home+generic only; 4 slots x ~35% S/A  |

**Critical comparison to Surge+Floor:** Under Pessimistic fitness, Surge+Floor
achieves M3 ~1.42. Narrative Gravity (Standard) is projected at ~1.7 -- a +0.28
improvement. The improvement comes from removing B/C sibling cards rather than
presenting them in targeted slots. Under Hostile fitness, Surge+Floor would
achieve ~1.0-1.1; Narrative Gravity projects ~1.3, still meaningfully better.

______________________________________________________________________

## Set Design Specification (Champion: Narrative Gravity, 40% Dual-Resonance Pool)

### 1. Pool Breakdown by Archetype

| Archetype            |  Total  | Home-Only | Cross-Archetype | Generic |
| -------------------- | :-----: | :-------: | :-------------: | :-----: |
| Flash (Ze/Em)        |   40    |    24     |       16        |   --    |
| Blink (Em/Ze)        |   40    |    24     |       16        |   --    |
| Storm (Em/St)        |   40    |    24     |       16        |   --    |
| Self-Discard (St/Em) |   40    |    24     |       16        |   --    |
| Self-Mill (St/Ti)    |   40    |    24     |       16        |   --    |
| Sacrifice (Ti/St)    |   40    |    24     |       16        |   --    |
| Warriors (Ti/Ze)     |   40    |    24     |       16        |   --    |
| Ramp (Ze/Ti)         |   40    |    24     |       16        |   --    |
| Generic              |   40    |    --     |       --        |   40    |
| **Total**            | **360** |  **192**  |     **128**     | **40**  |

Cross-archetype cards are designed to be A-tier in both the home archetype and
its co-primary sibling. With 16 cross-archetype cards per archetype out of 40
total, the base cross-archetype A-tier rate is 40% -- but the algorithm does not
depend on this rate being high because it removes low-fitness siblings from the
pool.

### 2. Symbol Distribution

|     Symbol Count      | Cards | % of Pool | Example                            |
| :-------------------: | :---: | :-------: | ---------------------------------- |
|      0 (generic)      |  40   |    11%    | No resonance symbols               |
|       1 symbol        |  48   |    13%    | (Tide) -- archetype-specific cards |
|   2 symbols (same)    |   0   |    0%     | Not used                           |
| 2 symbols (different) |  192  |    53%    | (Tide, Zephyr) -- dual-resonance   |
|       3 symbols       |  80   |    22%    | (Tide, Zephyr, Stone)              |

### 3. Dual-Resonance Breakdown

| Type                                    | Cards |  %  | Filtering Implications                         |
| --------------------------------------- | :---: | :-: | ---------------------------------------------- |
| Single-resonance (1 symbol)             |  48   | 13% | Matches 2 archetypes on R1 filter              |
| Dual-resonance (archetype-aligned pair) |  192  | 53% | Matches 1 archetype on R1+R2 filter            |
| Tri-resonance                           |  80   | 22% | Matches 1 archetype on full triple filter      |
| Generic (0 symbols)                     |  40   | 11% | Matches no filter; assigned baseline relevance |

### 4. Per-Resonance Pool Sizes

| Resonance | As Primary (pos 1) | As Any Symbol | Cards Available on "Has R" Filter |
| --------- | :----------------: | :-----------: | :-------------------------------: |
| Ember     |         80         |      160      |                160                |
| Stone     |         80         |      160      |                160                |
| Tide      |         80         |      160      |                160                |
| Zephyr    |         80         |      160      |                160                |

Each resonance appears as primary symbol on 80 cards (2 archetypes x 40 cards).
Each resonance appears as any symbol on ~160 cards (primary on 80 + secondary on
~80 more via dual/tri-resonance cards from adjacent archetypes). Narrative
Gravity uses the relevance dot product rather than binary filtering, so these
numbers define the gradient, not a hard boundary.

### 5. Cross-Archetype Requirements

Of each archetype's 40 cards, 16 (40%) must be at least A-tier in the co-primary
sibling. This is achievable because:

- Warriors/Sacrifice (Tide): 16 cards is conservative for this high-overlap pair
  (35-45% natural A-tier).
- Self-Discard/Self-Mill (Stone): 16 cards requires moderate effort (25-35%
  natural, need to push to 40%).
- Blink/Storm (Ember): 16 cards requires deliberate bridging (15-25% natural,
  need 8-10 intentional bridge cards).
- Flash/Ramp (Zephyr): 16 cards requires significant design investment (10-20%
  natural, need 12+ bridge cards).

However, Narrative Gravity is uniquely tolerant of missing this target. If
Flash/Ramp achieves only 20% cross-archetype A-tier (8 cards instead of 16), the
algorithm compensates by removing the 32 B/C-tier Ramp cards from the Flash
player's pool more aggressively. M3 for Flash drops from ~1.7 to ~1.5 under
Pessimistic fitness -- still better than Surge+Floor's 1.42 average across all
archetypes.

### 6. What the Card Designer Must Do Differently

Compared to V7 assumptions:

1. **Increase dual-resonance cards from 15% to 53%.** Most non-generic cards
   should carry 2 resonance symbols corresponding to their archetype pair. A
   Warriors card gets (Tide, Zephyr). A Storm card gets (Ember, Stone). This is
   a flavor requirement, not a mechanical one -- the card's mechanics can be
   narrow, but its symbol pair must reflect its archetype.
2. **Add 40 generic cards (up from 36).** These serve as universal filler that
   survives pool contraction regardless of player direction.
3. **Design 16 cross-archetype cards per archetype** that are genuinely A-tier
   in both siblings. For low-overlap pairs (Flash/Ramp), this requires
   intentional bridge card design (e.g., cards that scale with energy -- cheap
   and efficient for Flash, powerful and expensive for Ramp).
4. **The key difference from V7:** The algorithm does not depend on high
   cross-archetype fitness to deliver good results. Even if the designer falls
   short on bridge cards, Narrative Gravity compensates by removing the
   unplayable siblings. The designer's target is "make dual-resonance symbols
   accurate" rather than "make every card cross-archetype playable."
