# Design Agent 7: Beyond Explainability

## Key Takeaways

1. **Relaxing explainability unlocks multi-mechanism hybrids that exceed any
   single-mechanism ceiling.** Combining surge, bias, pair-matching, and
   adaptive floors in a single system reaches projected M3 of 2.05-2.15 under
   Graduated Realistic fitness (36% weighted average), where no one-sentence
   algorithm exceeds 1.85.

2. **Phase-based architectures solve the bimodal quality problem.** By using
   different targeting strategies at different draft stages (exploration,
   commitment, refinement), pack quality follows a smooth ramp rather than
   surge/floor oscillation. This directly addresses M10 and the player
   experience criteria from Research Agent C.

3. **Adaptive algorithms that respond to drafting patterns produce better
   per-archetype equity.** A system that compensates for low-overlap pairs
   (Flash/Ramp at 25% fitness) by increasing targeting intensity for those
   archetypes narrows the per-archetype M3 gap from ~0.5 to ~0.15.

4. **The performance ceiling without explainability is approximately +0.25-0.35
   M3 over the best one-sentence algorithm at equivalent fitness.** This is
   meaningful (the difference between 1.85 and 2.15) but not transformative --
   pool composition changes (40% dual-resonance) contribute more than
   algorithmic complexity.

5. **Player experience improves when the mechanism is hidden but the feedback is
   legible.** Research Agent C's finding that "transparency of feedback, not
   mechanism" matters means complex algorithms that deliver smooth quality ramps
   feel better than simple algorithms with jarring alternation.

6. **The 40% dual-resonance pool is the critical enabler.** Every algorithm in
   this document assumes the enriched pool from Research Agent A. Without it,
   even unlimited algorithmic complexity cannot reach 2.0 under realistic
   fitness.

7. **The best "beyond explainability" algorithm is a continuous probability
   weighting system that avoids discrete surge/floor modes entirely.** This
   eliminates the bimodal distribution problem at its root.

______________________________________________________________________

## Five Algorithm Proposals

### 1. Continuous Resonance Gravity (CRG)

**One-sentence:** Not describable in one sentence.

**Technical description:** Every slot in every pack draws from a weighted pool
where the weight of each card is determined by a continuous function of its
resonance match to the player's accumulated profile. The weighting function is:
`w(card) = 1 + k * match_score(card, profile)`, where `match_score` returns 0-1
based on symbol overlap (pair-matching via the enriched pool's dual-resonance
symbols), and `k` starts at 0 (pick 1) and increases linearly to `k_max` by pick
10, then remains constant. There are no discrete "surge" or "floor" modes --
every pack is drawn from the same weighted distribution, with the weights
gradually shifting toward the player's committed resonances.

**Predicted behavior:**

| Fitness                   |  M3  |           M10           | Distribution Shape |
| ------------------------- | :--: | :---------------------: | :----------------: |
| Graduated Realistic (36%) | 1.95 | Pass (\<=1 consecutive) |  Unimodal, smooth  |
| Pessimistic (21%)         | 1.65 |          Pass           |  Unimodal, smooth  |
| Hostile (8%)              | 1.25 |          Pass           |  Unimodal, narrow  |

**Strengths:** Eliminates bimodal quality distribution entirely. Every pack
feels organic. M10 trivially passes because there are no "floor" packs.
**Weaknesses:** M9 variance may be too low (estimated 0.6-0.7) because every
pack converges to a similar weighted draw. The player may perceive less
progression because there are no "wow" surge moments.

### 2. Phased Pair-Surge Hybrid (PPS)

**One-sentence:** Not describable in one sentence.

**Technical description:** Three draft phases with different algorithms. Phase 1
(picks 1-5): All packs are random with mild bias (1.5x weight toward any
resonance the player has drafted). Phase 2 (picks 6-12): Pair-matching activates
-- 2 of 4 slots are drawn from the pair-matched subpool (ordered pair matching
the player's top archetype pair), 2 slots random. Phase 3 (picks 13-30): Surge
mechanism activates on top of pair-matching -- when token counter reaches 3, 3
of 4 slots are pair-matched; on non-surge packs, 2 of 4 slots are pair-matched.
The pair-matched subpool achieves ~85% S/A precision under Pessimistic fitness
(Research Agent A finding). Phase transitions are invisible to the player.

**Predicted behavior:**

| Fitness                   |  M3  |     M10     |        Distribution Shape        |
| ------------------------- | :--: | :---------: | :------------------------------: |
| Graduated Realistic (36%) | 2.10 | Pass (\<=2) | Ramping, mild bimodal in Phase 3 |
| Pessimistic (21%)         | 1.80 |  Marginal   |        Ramping, narrower         |
| Hostile (8%)              | 1.40 |    Pass     |               Flat               |

**Strengths:** Highest projected M3 because pair-matching's 85% precision under
Pessimistic exceeds R1-filtering's 62.5%. The phase structure creates a natural
quality ramp (Research Agent C's "7 Wonders principle"). **Weaknesses:** Phase 3
reintroduces mild bimodal distribution. Requires 40% dual-resonance pool. Three
distinct phases are complex to tune.

### 3. Adaptive Intensity Targeting (AIT)

**One-sentence:** Not describable in one sentence.

**Technical description:** The algorithm maintains a "targeting intensity" value
(0.0 to 1.0) per player that adjusts based on recent pack quality outcomes.
After each pack, the system evaluates how many S/A cards were present. If below
the running average, intensity increases by 0.1; if above, it decreases by 0.05.
The intensity determines how many slots are drawn from the pair-matched subpool
(intensity * 4, rounded): at 0.25, 1 slot is pair-matched; at 0.75, 3 slots are.
Intensity starts at 0.0 and is capped at 0.85 to preserve splash. This creates a
self-correcting feedback loop: bad packs increase future targeting, preventing
dread streaks. Good packs reduce targeting, preserving variety.

**Predicted behavior:**

| Fitness                   |  M3  |         M10          | Distribution Shape |
| ------------------------- | :--: | :------------------: | :----------------: |
| Graduated Realistic (36%) | 1.90 | Pass (0 consecutive) |   Tight unimodal   |
| Pessimistic (21%)         | 1.60 | Pass (0 consecutive) |   Tight unimodal   |
| Hostile (8%)              | 1.20 |         Pass         |   Tight unimodal   |

**Strengths:** Self-correcting eliminates dead packs structurally (M10 trivially
passes). Compensates for per-pair fitness differences -- Flash/Ramp (low
fitness) will have higher intensity, more targeting. **Weaknesses:** Lower M3
ceiling because the algorithm spends "intensity budget" recovering from bad
packs rather than maximizing good ones. The feedback loop may oscillate if
poorly tuned. The player cannot predict or anticipate pack quality.

### 4. Surge+Floor+Bias+PairFilter Quad-Hybrid (SFBP)

**One-sentence:** Not describable in one sentence.

**Technical description:** Extends V7's Surge+Floor with two additions. First,
all random slots (3 in floor packs, 1 in surge packs) are drawn with 2x bias
toward the player's primary resonance (the Bias layer from V7 Agent 4, projected
+0.12 M3). Second, all targeted slots (surge and floor) use pair-matching
instead of R1-only filtering when the player has sufficient pair data (drafted
3+ cards with dual-resonance symbols matching the leading pair). Pair-matching
raises targeted slot precision from 75% (R1 under Moderate) to ~85% (pair under
Pessimistic). The surge threshold remains T=3, surge slots remain 3/4, floor
starts at pick 3 with 1 slot. Parameters: T=3, S=3, floor_start=3, bias=2.0x,
pair_threshold=3 dual-res cards drafted.

**Predicted behavior:**

| Fitness                   |  M3  |       M10       |     Distribution Shape     |
| ------------------------- | :--: | :-------------: | :------------------------: |
| Graduated Realistic (36%) | 2.05 | Marginal (\<=2) | Bimodal but elevated floor |
| Pessimistic (21%)         | 1.75 |    Marginal     |  Bimodal, lower amplitude  |
| Hostile (8%)              | 1.35 |      Pass       |        Mild bimodal        |

**Strengths:** Directly extends the V7 champion, preserving all its validated
properties. Each addition is individually justified (Bias: +0.12 projected;
Pair-filter: +0.15-0.20 from precision upgrade). Combined effect is
multiplicative under favorable pool conditions. **Weaknesses:** Retains the
bimodal surge/floor distribution, though the elevated floor (pair-matched +
bias) raises the valley. Still "feels" like surge packs. Four interacting
mechanisms.

### 5. Commitment-Scaled Continuous Targeting (CSCT)

**One-sentence:** Not describable in one sentence.

**Technical description:** The number of pair-matched slots per pack is a
continuous function of the player's commitment level, defined as
`C = (top_pair_count) / (total_picks)`. At C=0 (no commitment), 0 slots are
targeted. At C=0.5 (half of all picks match the top pair), 2 slots are
pair-matched. At C=0.7+, 3 slots are pair-matched. The mapping is:
\`targeted_slots = floor(min(C

- 5, 3))\`. The remaining slots are drawn with mild bias (1.3x toward primary
  resonance). There is no surge/floor distinction, no token counting, no phase
  transitions. The algorithm simply reads the player's draft history and scales
  targeting proportionally. Early picks (low C) get random packs; committed
  drafters (high C) get heavily targeted packs. The transition is smooth and
  invisible.

**Predicted behavior:**

| Fitness                   |  M3  |     M10     |  Distribution Shape   |
| ------------------------- | :--: | :---------: | :-------------------: |
| Graduated Realistic (36%) | 2.00 | Pass (\<=1) | Smooth ramp, unimodal |
| Pessimistic (21%)         | 1.70 |    Pass     |      Smooth ramp      |
| Hostile (8%)              | 1.30 |    Pass     |      Gentle ramp      |

**Strengths:** Extremely smooth delivery -- no discrete modes, no oscillation.
The player experiences a gradual quality ramp that directly mirrors their
commitment level. Handles pivoting gracefully (C drops, targeting drops).
Pair-matching provides fitness robustness. **Weaknesses:** M9 variance may be
marginal (estimated 0.7-0.85) because pack structure changes gradually rather
than alternating. The commitment metric C is a lagging indicator -- it takes 3-4
picks for a pivot to register. No "surge moment" excitement.

______________________________________________________________________

## Champion Selection: Commitment-Scaled Continuous Targeting (CSCT)

**Justification:** CSCT best addresses V8's dual mandate of hitting M3 >= 2.0
and satisfying player experience criteria. It projects M3=2.00 under Graduated
Realistic fitness -- exactly at target -- while delivering the smoothest pack
quality distribution of all five proposals. It eliminates the bimodal
surge/floor problem that Research Agent C identified as the primary experiential
failure of V7's champion.

CSCT over PPS: PPS has higher projected M3 (2.10) but reintroduces bimodal
distribution in Phase 3. The +0.10 M3 advantage is not worth the experiential
regression.

CSCT over SFBP: SFBP is the safest engineering choice (extends V7's validated
champion) but inherits the surge/floor rhythm problem. CSCT starts from a
fundamentally different architecture that eliminates the rhythm problem.

CSCT over AIT: AIT's self-correcting mechanism is elegant but sacrifices M3
ceiling for consistency. CSCT achieves comparable smoothness at higher M3.

CSCT over CRG: CRG's per-slot weighting is too diffuse -- it may fail M9
variance. CSCT uses discrete pair-matched slots (which provide variance through
the binary "is this slot pair-matched or random?" question) within a smooth
scaling envelope.

______________________________________________________________________

## Champion Deep-Dive: CSCT

### How It Works

After each pick, compute the commitment ratio:
`C = pair_count(top_pair) / total_picks`, where `pair_count(top_pair)` counts
how many of the player's drafted cards have symbols matching the leading
archetype pair (e.g., cards with both Tide and Zephyr symbols for a Warriors
player, or cards with Tide as primary for the Tide-primary archetype detection).
The number of pair-matched slots is `S = floor(min(C * 5, 3))`. Remaining slots
(4 - S) are drawn with 1.3x bias toward primary resonance.

### Example Draft: Warriors (Tide/Zephyr) Under Graduated Realistic

- **Pick 1:** C=0.0, S=0. Pack is 4 random cards. Player takes a Tide/Zephyr
  character.
- **Pick 3:** C=0.33, S=1. Pack has 1 pair-matched Tide/Zephyr card + 3 mildly
  biased random. Player sees 1 likely-Warriors card.
- **Pick 6:** C=0.50, S=2. Pack has 2 pair-matched slots + 2 biased random.
  Player consistently sees 2 strong Warriors options. This is the commitment
  inflection point.
- **Pick 10:** C=0.60, S=3. Pack has 3 pair-matched slots + 1 biased random. At
  85% precision per pair-matched slot: expected 2.55 S/A + 0.35 from biased
  random = ~2.5 S/A. Splash preserved in the 1 random slot.
- **Pick 20:** C=0.65, S=3. Stable at maximum targeting. Expected ~2.5 S/A per
  pack.

**Post-commitment average (picks 6-30):** Weighted average across the commitment
curve, accounting for C increasing from 0.50 to 0.65: approximately 2.0-2.2 S/A
per pack under pair-matching precision of 85%.

### Example: Pivot at Pick 8

- **Picks 1-7:** Player drafts Flash (Zephyr/Ember). C=0.57, S=2 targeting
  Zephyr/Ember.
- **Pick 8:** Player takes a strong Tide card (off-pair). C drops to 0.50. S
  stays at 2 but pair identification may shift.
- **Picks 9-12:** Player commits to Warriors (Tide/Zephyr). C for the new pair
  climbs from 0.25 to 0.42. S transitions from 1 to 2.
- **Pick 15:** C=0.50, S=2. Full recovery.

The pivot costs approximately 5 picks of reduced targeting -- firm enough to
matter (signal reading is rewarded) but not so harsh that pivoting is punished
excessively.

### Failure Modes

1. **Slow starter problem.** If a player's first 5 picks are dispersed across
   archetypes, C stays below 0.20 and S=0 through pick 5. The algorithm provides
   no assistance until the player naturally concentrates. Mitigation: increase
   the multiplier from 5 to 6, so C=0.33 (2 of 6 picks on-pair) yields S=1.

2. **Variance deficit.** Because the targeting level changes gradually,
   consecutive packs have very similar structure. Estimated M9 stddev: 0.7-0.85,
   which is marginal against the >= 0.8 target. Mitigation: add stochastic
   jitter -- each slot has a 15% chance of being "promoted" (random to
   pair-matched) or "demoted" (pair-matched to random). This adds organic
   variance without changing the average.

3. **Pair-matched subpool exhaustion.** At 40% dual-resonance (18 pair-matched
   cards per archetype), drawing 3 pair-matched cards per pack for 25
   post-commitment packs requires 75 draws from an 18-card pool. Cards will be
   seen ~4 times each. This is acceptable but not ideal. At 50% dual-resonance
   (22 cards per pair), repetition drops to ~3.4 times.

### Parameter Variants

| Variant       | Multiplier |   Bias   | Jitter | Expected M3 (Grad. Real.) | M9 Est.  |
| ------------- | :--------: | :------: | :----: | :-----------------------: | :------: |
| **Base**      |   **5**    | **1.3x** | **0%** |         **2.00**          | **0.75** |
| High-mult     |     6      |   1.3x   |   0%   |           2.05            |   0.70   |
| Jittered      |     5      |   1.3x   |  15%   |           1.98            |   0.90   |
| Strong-bias   |     5      |   2.0x   |   0%   |           2.08            |   0.72   |
| Jittered+Bias |     5      |   1.5x   |  15%   |           2.05            |   0.88   |

**Recommended variant for simulation: Jittered+Bias** (multiplier=5, bias=1.5x,
jitter=15%). This balances M3 performance with M9 variance compliance.

______________________________________________________________________

## Set Design Specification (Champion: CSCT with 40% Dual-Resonance Pool)

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

Cross-archetype cards (18 per archetype) carry dual-resonance symbols and are
designed to be A-tier in the co-primary sibling archetype.

### 2. Symbol Distribution

|            Symbol Count             | Cards | % of Pool | Example                                                    |
| :---------------------------------: | :---: | :-------: | ---------------------------------------------------------- |
|             0 (generic)             |  40   |   11.1%   | No resonance symbols                                       |
|              1 symbol               |  56   |   15.6%   | (Tide) -- archetype-specific, no cross-pair utility        |
|     2 symbols (same resonance)      |   0   |    0%     | Not used                                                   |
| 2 symbols (different, ordered pair) |  144  |   40.0%   | (Tide, Zephyr) -- pair-matchable                           |
|      3 symbols (pair + splash)      |  120  |   33.3%   | (Tide, Zephyr, Ember) -- pair-matchable with splash signal |

### 3. Dual-Resonance Breakdown

| Type                                    | Cards | % of Pool | Filtering Implications                                       |
| --------------------------------------- | :---: | :-------: | ------------------------------------------------------------ |
| No resonance (generic)                  |  40   |   11.1%   | Not filterable                                               |
| Single-resonance                        |  56   |   15.6%   | Matches 2 archetypes on R1 filter                            |
| Dual-resonance (archetype-aligned pair) |  144  |   40.0%   | Matches 1 archetype on pair filter (~85% precision)          |
| Tri-resonance (pair + third)            |  120  |   33.3%   | Matches 1 archetype on pair filter; third symbol aids splash |

Total pair-matchable cards: 264 (73.3% of pool).

### 4. Per-Resonance Pool Sizes

| Resonance | Primary Symbol | Any Symbol | Pair-Matched Cards per Archetype Pair |
| --------- | :------------: | :--------: | :-----------------------------------: |
| Ember     |       80       |    ~200    |      ~33 per Ember-primary pair       |
| Stone     |       80       |    ~200    |      ~33 per Stone-primary pair       |
| Tide      |       80       |    ~200    |       ~33 per Tide-primary pair       |
| Zephyr    |       80       |    ~200    |      ~33 per Zephyr-primary pair      |

When CSCT filters by pair (Tide, Zephyr), the candidate pool contains ~33 cards.
Of these, ~85% (28) are S/A-tier for Warriors under Graduated Realistic fitness.
Over a 25-pack post-commitment draft drawing 3 pair-matched cards per pack, 75
draws from a 33-card pool means each card is seen ~2.3 times -- acceptable
repetition.

### 5. Cross-Archetype Requirements

Of each archetype's 40 cards, 18 (45%) must be at least A-tier in the co-primary
sibling archetype. Per-pair targets reflecting mechanical distance (Research
Agent B):

| Pair                             | Required A-tier | Natural Overlap | Design Gap | Difficulty |
| -------------------------------- | :-------------: | :-------------: | :--------: | :--------: |
| Warriors / Sacrifice (Tide)      | 18 of 40 (45%)  |    ~14 (35%)    |  4 cards   |    Low     |
| Self-Discard / Self-Mill (Stone) | 18 of 40 (45%)  |    ~12 (30%)    |  6 cards   |  Moderate  |
| Blink / Storm (Ember)            | 18 of 40 (45%)  |    ~8 (20%)     |  10 cards  |    High    |
| Flash / Ramp (Zephyr)            | 18 of 40 (45%)  |    ~6 (15%)     |  12 cards  |    High    |

For high-difficulty pairs (Blink/Storm, Flash/Ramp), the designer must create
10-12 intentional bridge cards per archetype. These are cards specifically
written to serve both archetypes mechanically (see Research Agent B's bridge
card examples).

### 6. What the Card Designer Must Do Differently

Compared to V7's assumptions:

1. **Increase dual-resonance cards from 54 to 264.** Every non-generic,
   non-singleton card should carry 2-3 resonance symbols with the first two
   forming an archetype-aligned ordered pair.

2. **Design 18 cross-archetype cards per archetype (45% of each archetype's
   pool).** These must be genuinely A-tier in both the home and sibling
   archetype. For mechanically close pairs (Warriors/Sacrifice), this emerges
   naturally. For distant pairs (Flash/Ramp), this requires deliberate bridge
   mechanics: cards that function differently depending on context (e.g., "Fast.
   Cost equals your current energy. Draw cards equal to cost paid" -- cheap draw
   in Flash, massive draw in Ramp).

3. **Assign ordered pair symbols to every non-generic card.** The first two
   symbols must align with an archetype pair. The optional third symbol signals
   splash potential. Symbol assignment is a design task, not arbitrary
   decoration -- the symbols must reflect the card's mechanical identity.

4. **Create 40 generic cards (up from 36).** These carry no resonance symbols
   and serve as neutral filler, power picks, and archetype-crossing utility
   (removal, unconditional draw).

5. **Maintain at least 33 pair-matchable cards per archetype pair** to prevent
   subpool exhaustion. This is automatically satisfied at 18 cross-archetype +
   15 single-resonance-with-pair-symbols per archetype.
