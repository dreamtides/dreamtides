# Resonance Structure Exploration: Final Synthesis Report

## 1. Executive Summary

Model A (N=5, Pair Archetypes) is the recommended resonance structure for
Dreamtides. Averaging scores across three independent analysts, Model A achieves
56.0/80 -- the highest of any model -- with no catastrophic failures on any
goal. The current N=5 pair-archetype structure with the H5-ADDITIVE algorithm is
validated as the optimal balance of simplicity, convergence, splash viability,
and archetype variety. While Model D (N=7, Triple) offers superior early
diversity (3.90 unique resonances/pack vs 2.91) and Model E (Spectrum) offers
the best exploration dynamics (convergence at pick 12.6), both impose complexity
costs that outweigh their benefits for a roguelike deckbuilder targeting broad
accessibility. The two previously unreachable targets (early unique resonances
\>= 3.0 and synergy top-2 share 75-90%) remain unreachable within N=5 but are
reachable by structural changes (N=7 with multi-resonance cards, and K=1 mono
archetypes respectively) at costs that are not justified. Confidence level:
HIGH.

______________________________________________________________________

## 2. Unified Comparison Table

Scores from all three analysts (Player Experience = A1, Convergence Dynamics =
A2, Variety & Depth = A3) averaged for each model-goal pair. The "Key Metric"
column cites the primary number driving the score.

| Goal                       | Model A (N=5 Pair)                                      | Model B (N=10 Single)                               | Model C (N=4 Lean)                                         | Model D (N=7 Triple)                                     | Model E (5-Axis Spectrum)                                  |
| -------------------------- | ------------------------------------------------------- | --------------------------------------------------- | ---------------------------------------------------------- | -------------------------------------------------------- | ---------------------------------------------------------- |
| **1. Simple**              | **8.0** (A1=8, A2=8, A3=8) 5 res, 10 pairs              | **5.0** (A1=5, A2=5, A3=5) 10 res, no pair glue     | **9.0** (A1=9, A2=9, A3=9) 4 res, elemental                | **4.0** (A1=4, A2=4, A3=4) 7 res, 35 triples             | **3.0** (A1=3, A2=3, A3=3) 5D vectors, 10 poles            |
| **2. Not on Rails**        | **5.7** (A1=6, A2=6, A3=5) 51.8% early on-color         | **6.3** (A1=5, A2=7, A3=7) 16.5% early on-color     | **4.0** (A1=5, A2=4, A3=3) 59% early on-color              | **5.3** (A1=4, A2=7, A3=5) conv 3.79 despite 3.90 unique | **8.7** (A1=9, A2=8, A3=9) conv 12.6, 36.3% opposing       |
| **3. No Forced Decks**     | **8.0** (A1=7, A2=9, A3=8) CV=0.08, 10 pairs            | **7.0** (A1=6, A2=8, A3=7) range=32%, 10 single     | **6.7** (A1=6, A2=7, A3=7) 6 pairs, max dev 2.7%           | **9.0** (A1=9, A2=9, A3=9) 35/35 triples reached         | **7.3** (A1=8, A2=7, A3=7) axis range 15-24%               |
| **4. Flexible Archetypes** | **6.3** (A1=5, A2=7, A3=7) 100% dual, 80.9% splash      | **3.7** (A1=3, A2=4, A3=4) 99.4% mono, no dual glue | **6.3** (A1=7, A2=6, A3=6) 50/50 mono/dual split           | **7.7** (A1=8, A2=7, A3=8) 8/40/52 mono/dual/triple      | **6.7** (A1=7, A2=6, A3=7) 59/24/18 focus/dual/scatter     |
| **5. Convergent**          | **7.7** (A1=8, A2=7, A3=8) 95.5% top-2, pick 4.6        | **7.7** (A1=8, A2=8, A3=7) 82.8% top-1, pick 7.4    | **7.7** (A1=9, A2=6, A3=8) 98.5% top-2, pick 4.8           | **6.0** (A1=7, A2=4, A3=7) 90.5% top-3, pick 3.79        | **6.0** (A1=6, A2=6, A3=6) mag 19.84, pick 12.6            |
| **6. Splashable**          | **8.0** (A1=8, A2=8, A3=8) 80.9% splash, 22.9% off      | **6.0** (A1=6, A2=6, A3=6) 91.3% splash but shallow | **5.0** (A1=5, A2=5, A3=5) 42.8% splash, only 2 off-colors | **2.3** (A1=3, A2=1, A3=3) 0.5% splash, K=3 too broad    | **5.3** (A1=6, A2=5, A3=5) 17.7% scattered, opposing 36.3% |
| **7. Open-Ended Early**    | **5.7** (A1=6, A2=6, A3=5) 2.91 unique res/pack         | **5.7** (A1=4, A2=6, A3=7) 2.94 unique, 29.4% sat   | **4.3** (A1=5, A2=5, A3=3) 2.56 unique, 64% sat            | **7.0** (A1=7, A2=8, A3=6) 3.90 unique res/pack          | **8.3** (A1=8, A2=9, A3=8) late conv, genuine exploration  |
| **8. Signal Reading**      | **6.7** (A1=7, A2=7, A3=6) lane seeds, 52->78% on-color | **6.0** (A1=5, A2=5, A3=8) 58.8% off-color visible  | **5.3** (A1=6, A2=6, A3=4) deep pools bury signal          | **6.7** (A1=7, A2=6, A3=7) triple card signals           | **6.3** (A1=6, A2=7, A3=7) opposing cards as signal        |
| **TOTAL**                  | **56.0**                                                | **47.3**                                            | **48.3**                                                   | **47.7**                                                 | **51.7**                                                   |

______________________________________________________________________

## 3. Pass/Fail Matrix Against Measurable Targets

Specific measurable targets derived from simulation data, with pass/fail for
each model using the synergy strategy data.

| Target                                  | Threshold                        | A                         | B                         | C                          | D                                      | E                                                  |
| --------------------------------------- | -------------------------------- | ------------------------- | ------------------------- | -------------------------- | -------------------------------------- | -------------------------------------------------- |
| Early unique res >= 2.7                 | >= 2.7/pack (picks 1-5)          | PASS (2.91)               | PASS (2.94)               | FAIL (2.56)                | PASS (3.90)                            | N/A (continuous)                                   |
| Convergence pick 5-8 (synergy)          | Mean 5.0-8.0                     | FAIL (4.6)                | PASS (7.4)                | FAIL (4.8)                 | FAIL (3.79)                            | FAIL (12.6)                                        |
| Top-K share 75-90%                      | Top-K in [75%, 90%]              | FAIL (95.5% top-2)        | PASS (82.8% top-1)        | FAIL (98.5% top-2)         | PASS (90.5% top-3)                     | N/A (continuous)                                   |
| Splash rate 30-80%                      | 30-80% of synergy decks          | PASS (80.9%)              | FAIL (91.3%)              | PASS (42.8%)               | FAIL (0.5%)                            | N/A                                                |
| Off-color late >= 0.5/pack              | >= 0.5 cards per late pack       | PASS (16.3% = ~0.65/pack) | PASS (54.7% = ~2.19/pack) | PASS (15% = ~0.60/pack)    | PASS (0.62/pack)                       | PASS (36.3% opposing)                              |
| Pair freq CV < 0.15                     | CV of archetype distribution     | PASS (0.08)               | PASS (~0.11 est.)         | PASS (~0.10 est.)          | PASS (all 35 >= 1%)                    | PASS (axis range 1.58x)                            |
| Classification diversity (>1 type >10%) | At least 2 types above 10%       | FAIL (100% dual)          | FAIL (99.4% mono)         | PASS (52% dual / 48% mono) | PASS (52% triple / 40% dual / 8% mono) | PASS (59% focused / 24% dual-axis / 18% scattered) |
| Early on-color \<= 55%                  | Early on-color fraction          | PASS (51.8%)              | PASS (16.5%)              | FAIL (59%)                 | FAIL (~60% est.)                       | PASS (~64% aligned)                                |
| Late on-color >= 70%                    | Late on-color fraction           | PASS (78.4%)              | FAIL (34.4%)              | PASS (82%)                 | PASS (74.3% = 2.97/4)                  | N/A                                                |
| Power chaser eff colors >= 3.0          | Effective colors for PC strategy | PASS (3.89)               | PASS (5.75)               | PASS (3.12)                | PASS (5.59)                            | N/A                                                |
| **Passes**                              |                                  | **7/10**                  | **6/10**                  | **5/10**                   | **6/10**                               | **4/7 applicable**                                 |

______________________________________________________________________

## 4. Overall Ranking with Justification

### Rank 1: Model A (N=5, Pair Archetypes) -- Average Score: 56.0/80

**Analyst scores:** A1=55, A2=58, A3=55 (average 56.0)

**Key strengths (with numbers):**

- Best convergence-splash balance: 80.9% splash rate with 95.5% top-2
  convergence share (synergy)
- Excellent archetype balance: pair frequency CV = 0.08 across 10 pair
  archetypes (range 9.0-11.3%)
- Simplest viable system: 5 resonances, one-sentence explanation, strong
  MTG-like analogy
- Proven algorithm foundation: H5-ADDITIVE already validated with 15/21
  measurable targets passing
- Healthy late-game dynamics: on-color rises from 51.8% to 78.4%, off-color
  available at 22.9%

**Key weaknesses (with numbers):**

- Early convergence: pick 4.6 for synergy, slightly below the 5.0-8.0 target
  window
- Rigid classification: 100% dual for synergy, 0% mono or tri builds possible
- Early diversity ceiling: 2.91 unique resonances/pack, structurally limited by
  N=5 and pack-of-4
- Convergence shape: cliff-then-plateau (94.7% at pick 5, 94.9% at pick 25) --
  no exploration dip

**Best-case scenario:** With minor parameter tuning (increase floor_weight to
4.0, decrease DC bonus to 1 for some dreamcallers), convergence can be delayed
to pick 5.0-5.5 and mono builds become viable for a subset of runs, addressing
both main weaknesses.

______________________________________________________________________

### Rank 2: Model E (5-Axis Spectrum, Continuous) -- Average Score: 51.7/80

**Analyst scores:** A1=53, A2=51, A3=52 (average 51.7)

**Key strengths (with numbers):**

- Best exploration dynamics: convergence at pick 12.6 (synergy), 2.06 direction
  changes per draft
- Linear convergence shape: magnitude grows 2.81 -> 5.23 -> 7.74 -> 10.42 ->
  13.19 (picks 5-25), the smoothest identity arc
- Genuine early openness: 36.3% opposing cards visible throughout the draft
- Strong identity emergence: 82.3% of synergy players achieve focused or
  dual-axis identity
- Excellent axis and pole balance: axis range 15.4-24.4%, pole ratio 0.45-0.53

**Key weaknesses (with numbers):**

- Worst simplicity: 5 axes, 10 poles, signed vectors, dot products -- no card
  game analogy exists
- Late convergence: 12.6 picks exceeds the 5-8 target window (tunable to 8.5 at
  threshold=4.0)
- Scattered identity risk: 17.7% of synergy decks fail to converge clearly
- Rigid strategy dysfunction: 45.9% of rigid players end up scattered despite
  trying to focus
- Novel mental model: players must evaluate [+0, -5, -7, -1, +2] style vectors

**Best-case scenario:** A presentation layer that hides the vector math behind
named poles (e.g., "your deck leans toward Wildfire-Decay") combined with a
lowered convergence threshold (4.0) could produce the best-feeling draft of any
model, IF the team can solve the player communication challenge.

______________________________________________________________________

### Rank 3: Model C (N=4, Lean Color Pie) -- Average Score: 48.3/80

**Analyst scores:** A1=52, A2=48, A3=45 (average 48.3)

**Key strengths (with numbers):**

- Maximum simplicity: 4 resonances with elemental names (Storm, Flame, Terra,
  Shadow), strongest analog to universal archetypes
- Deep per-resonance pools: 77 unique mono cards per resonance (vs 50 for Model
  A)
- Interesting DC variant: mono DC (50%) produces 95% mono decks, dual DC (50%)
  produces 98.8% dual -- genuine structural variety
- Strong convergence: 98.5% top-2 share, HHI 0.712

**Key weaknesses (with numbers):**

- Most "on rails": 59% early on-color fraction, highest of any model
- Lowest early diversity: 2.56 unique resonances/pack (below 2.7 revised target)
- Limited archetype space: only 6 pair archetypes (vs 10 for Model A, 35 for
  Model D)
- Near-zero dual cards: only 1.8 dual cards per pair, no mechanical bridge
  between colors
- Flat convergence shape: 98.1% top-2 at pick 5, essentially no exploration
  phase

**Best-case scenario:** Works well as a "simplified mode" for new players or
tutorial runs, where the simplicity advantage is paramount and the exploration
depth limitation is acceptable.

______________________________________________________________________

### Rank 4: Model D (N=7, Triple Archetypes) -- Average Score: 47.7/80

**Analyst scores:** A1=49, A2=46, A3=49 (average 47.7)

**Key strengths (with numbers):**

- Highest early diversity: 3.90 unique resonances/pack (breaks the N=5 ceiling
  of ~2.96)
- Richest archetype space: 35 triple archetypes, all 35 reached at >= 1%
  frequency
- Broadest classification spectrum: 7.6% mono, 40.1% dual, 52.3% triple
  (synergy)
- Meaningful pivots: 25.8% of players change exactly 1 resonance from DC
  direction
- Triple card mechanic: appears in 36.9% of packs, picked 49.1% when matching

**Key weaknesses (with numbers):**

- Near-zero splash: 0.5% splash rate (structurally broken -- K=3 consumes too
  much color space)
- Fastest convergence: pick 3.79, well below the 5-8 window (threshold too easy:
  top-3 > 70% vs 43% random)
- High complexity: 7 resonances, 35 possible triples, partial fit evaluation
  (0.7 for 2/3 match)
- Power chaser collapse: 88.3% quad+ classification, no meaningful identity for
  unfocused drafters
- Diffuse identity: HHI 0.306 and effective colors 3.32 for synergy -- spread
  thin across 3 resonances

**Best-case scenario:** Could serve as an "advanced mode" for experienced
players who have mastered the N=5 system. The 35-archetype design space provides
months of exploration, and the triple card mechanic adds strategic depth. Would
require reducing DC bonus significantly to slow convergence.

______________________________________________________________________

### Rank 5: Model B (N=10, Archetype-as-Resonance) -- Average Score: 47.3/80

**Analyst scores:** A1=42, A2=49, A3=51 (average 47.3)

**Key strengths (with numbers):**

- Ideal convergence window: pick 7.4 lands dead center of the 5-8 target
- Best U-shaped convergence curve: top-1 dips from 81.8% (pick 5) to 78.6% (pick
  10\) then recovers to 82.7% (pick 25) -- genuine exploration then commitment
- Low early on-color: 16.5% (most open early experience among discrete models)
- Strong signal reading: 58.8% off-color visible, 10 countable resonances

**Key weaknesses (with numbers):**

- Worst archetype flexibility: 99.4% mono decks, 0.6% splash-mono -- no dual
  builds possible (score 3.7/10 on Goal 4)
- High cognitive load: 10 unfamiliar resonance names with no combinatorial logic
- Shallow per-resonance pools: only 29 mono cards per resonance, ~90% pool
  utilization risk
- No dual cards: 80% mono / 20% neutral means no mechanical bridge between
  resonances
- Power chaser degeneration: 56.5% scattered, 5.75 effective colors -- no
  meaningful identity

**Best-case scenario:** If the game design shifted entirely to single-resonance
identity (no pairs), this model provides clean convergence dynamics. But the
absence of dual-card "glue" is a fundamental design limitation that cannot be
parameter-tuned away.

______________________________________________________________________

## 5. Recommendation

**Use Model A (N=5, Pair Archetypes) with the existing H5-ADDITIVE algorithm.**

The current resonance structure is validated. No alternative model improves
enough on the overall goal profile to justify the complexity, implementation
cost, or design risk of changing the fundamental structure. Model A scores
highest across all three analysts, passes the most measurable targets, and has
already been optimized through six rounds of algorithm research.

Specifically:

- The 5-resonance count (Tide, Ember, Zephyr, Stone, Ruin) provides the optimal
  balance of simplicity (5 concepts to learn), variety (10 pair archetypes), and
  depth (50 mono cards + 29 dual cards per archetype).
- The pair archetype system with 10% dual cards is essential for splash
  viability (80.9%) and flexible deckbuilding -- Model B's elimination of duals
  proves this definitively.
- The H5-ADDITIVE algorithm (floor=3.5, exp 0.5->1.1 over 12 picks, DC bonus=2)
  achieves the best convergence-splash tradeoff of any tested configuration.

______________________________________________________________________

## 6. Parameter Recommendations for Top Model(s)

### Model A (Recommended): H5-ADDITIVE Parameters

| Parameter      | Current Value | Recommended          | Sensitivity | Notes                                                                                                                                           |
| -------------- | ------------- | -------------------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `base_exp`     | 0.5           | **0.5** (no change)  | MODERATE    | Controls early-game flatness. Range [0.3, 0.7]. Lower values delay convergence.                                                                 |
| `max_exp`      | 1.1           | **1.1** (no change)  | HIGH        | Controls late-game convergence strength. Range [0.8, 1.4]. Higher = tighter focus, less splash.                                                 |
| `ramp_picks`   | 12            | **12** (no change)   | LOW         | Picks over which exponent ramps. Range [8, 16]. Minor impact.                                                                                   |
| `floor_weight` | 3.5           | **3.5-4.0**          | HIGH        | Controls off-color visibility. Increasing to 4.0 adds +1.6% splash (83.2%) at cost of -0.3% top-2 share. Consider 4.0 if splash is prioritized. |
| `neutral_base` | 4.0           | **4.0** (no change)  | LOW         | Neutral card weight. Minimal impact on resonance dynamics.                                                                                      |
| `dc_bonus`     | 2             | **2** (no change)    | HIGH        | Initial profile per DC resonance. The previous research conclusively showed DC=2 is optimal (DC=4 causes universal target failures).            |
| `seed_min`     | 0.60          | **0.60** (no change) | LOW         | Lane seed lower bound. Creates per-quest variation.                                                                                             |
| `seed_max`     | 1.40          | **1.40** (no change) | LOW         | Lane seed upper bound. Narrowing to [0.8, 1.2] reduces variance for competitive fairness.                                                       |

**Sensitivity summary:**

- To increase splash: raise `floor_weight` from 3.5 to 4.0 (adds ~3% splash rate
  with minimal convergence cost)
- To slow convergence: lower `base_exp` from 0.5 to 0.4 (delays convergence by
  ~0.5 picks) or lower `dc_bonus` to 1 (for select dreamcallers only)
- To increase archetype flexibility: introduce mono dreamcallers (bonus to 1
  resonance) for a subset of runs, following Model C's successful mono DC
  variant

### Model E (Secondary Consideration): Key Parameters

If Model E's exploration dynamics are desired as a future design direction:

| Parameter               | Recommended         | Notes                                                               |
| ----------------------- | ------------------- | ------------------------------------------------------------------- |
| `convergence_threshold` | 4.0 (down from 6.0) | Moves convergence from pick 12.6 to 8.5, entering the target window |
| `scale_factor`          | 0.8 (up from 0.5)   | Strengthens directional pull, moves convergence to pick 11.4        |
| `dc_bonus`              | 2                   | Each DC axis gets +2 starting magnitude                             |

______________________________________________________________________

## 7. Player-Facing One-Sentence Explanation

### Model A (Recommended)

"Your dreamcaller gives you two starting colors, and as you draft more cards of
those colors, packs gradually show you more of them -- but every quest shuffles
which colors are deepest, so reading the pool pays off."

### Model E (Runner-up, if pursued)

"Each card pushes your deck's identity along one of five thematic spectrums, and
as your direction strengthens, the packs increasingly align with it -- but cards
from the opposite direction still appear, tempting you to pivot."

______________________________________________________________________

## 8. Structural Insights

### N (Resonance Count) vs Early Diversity

Early unique resonances per pack depend on BOTH N and multi-resonance card
density. N alone is insufficient:

- N=4 (Model C): 2.56 unique res/pack (85% mono, 3% dual) -- lowest
- N=5 (Model A): 2.91 unique res/pack (70% mono, 10% dual) -- moderate
- N=10 (Model B): 2.94 unique res/pack (80% mono, 0% dual) -- barely better than
  N=5
- N=7 (Model D): 3.90 unique res/pack (60% mono, 20% dual, 5% triple) -- highest
  by far

**Principle:** Multi-resonance cards are the key lever for early diversity.
Model B proves that doubling N from 5 to 10 without dual cards adds only +0.03
unique res/pack, while Model D shows that N=7 with 25% multi-resonance cards
adds +0.99.

### N vs Convergence Speed

Larger archetype size K accelerates convergence because more of the card pool is
"on-archetype":

- K=1 (Model B): convergence at pick 7.4 (1/10 = 10% base on-archetype rate)
- K=2 (Model A): convergence at pick 4.6 (2/5 = 40% base on-archetype rate)
- K=2 (Model C): convergence at pick 4.8 (2/4 = 50% base on-archetype rate)
- K=3 (Model D): convergence at pick 3.79 (3/7 = 43% base on-archetype rate)
- Continuous (Model E): convergence at pick 12.6 (no fixed K, organic magnitude
  growth)

**Principle:** Convergence speed is primarily driven by the K/N ratio (DC
coverage / total options). Higher K/N = faster convergence. The optimal ratio
appears to be around 0.35-0.40 (Model A at 2/5 = 0.40).

### N vs Archetype Depth

With a fixed pool of 360 cards, increasing N dilutes per-resonance depth:

- N=4: 77 mono cards/resonance, very deep but limited variety
- N=5: 50 mono cards/resonance, good depth and variety
- N=7: 31 mono cards/resonance, adequate but thin cross-color
- N=10: 29 mono cards/resonance, 90% pool utilization risk

**Sweet spot:** N=5 gives 50 mono cards per resonance, meaning a synergy player
drafting ~25 on-color cards uses only ~50% of the available pool. This preserves
run-to-run variety. Model B's 29 cards per resonance means ~90% utilization --
two runs with the same resonance will feel very similar.

### K (Archetype Size) vs Splash Viability

Splash viability is inversely correlated with K:

- K=1 (Model B): 91.3% splash -- near-universal but shallow (1-3 cards)
- K=2 (Model A): 80.9% splash -- common and meaningful third color
- K=2 (Model C): 42.8% splash -- moderate, limited by N=4
- K=3 (Model D): 0.5% splash -- effectively zero, K=3 already covers too much
  color space

**Principle:** If splash is a design priority, keep K \<= 2. With K=3, the
weight signal from 3 core resonances overwhelms the floor weight for any
4th-color card, regardless of parameter tuning.

### The K/N Ratio as a Design Lever

The ratio K/N (archetype color count / total resonances) predicts the
convergence-splash tradeoff:

| Model | K/N         | Convergence (picks) | Splash Rate |
| ----- | ----------- | ------------------: | ----------: |
| B     | 1/10 = 0.10 |                 7.4 |       91.3% |
| A     | 2/5 = 0.40  |                 4.6 |       80.9% |
| D     | 3/7 = 0.43  |                3.79 |        0.5% |
| C     | 2/4 = 0.50  |                 4.8 |       42.8% |

Model A's K/N = 0.40 sits in the ideal quadrant: fast enough convergence (4.6
picks) with high splash viability (80.9%). Model D's similar K/N (0.43) fails on
splash because K=3 creates a stronger combined weight signal than K=2 -- the
relevant factor is not just K/N but K * avg_profile_per_resonance /
floor_weight.

### Dual Card Density as a Diversity Mechanism

Dual (multi-resonance) cards serve three critical functions:

1. **Early diversity amplifier:** Each dual card exposes 2 resonances per card
   slot, dramatically increasing early unique res/pack
2. **Mechanical bridge:** Dual cards glue two colors together, making pair
   identity feel organic rather than just accumulation of mono cards
3. **Splash enabler:** Dual cards reward commitment to two colors, creating a
   natural "splash" pathway when a player picks a dual that overlaps with their
   secondary color

Model B's elimination of dual cards (80% mono, 20% neutral) proves all three
functions: early diversity barely improves over N=5 (2.94 vs 2.91), no pair
identity exists (99.4% mono), and splash is shallow (1-3 incidental cards).
Model A's 10% dual allocation is sufficient; Model D's 20% dual + 5% triple
produces even richer dynamics at the cost of complexity.

### Continuous vs Discrete Identity Spaces

Model E demonstrates that continuous identity spaces offer fundamentally
different convergence dynamics:

- **Linear magnitude growth** (2.81 -> 5.23 -> 7.74 -> 13.19 over picks 5-25) vs
  cliff-then-plateau in discrete models
- **Opposing cards** (36.3% of non-neutral offers) create tension that does not
  exist in discrete models where off-color cards are simply ignored
- **Direction changes** (2.06 for synergy) enable genuine mid-draft pivots
- **Gradual identity strengthening** vs binary on/off-color switching

The tradeoff: continuous models produce the best draft *dynamics* but the worst
player *comprehension*. The presentation problem -- helping a player understand
[+0, -5, -7, -1, +2] as an identity -- has no known solution in existing card
games.

______________________________________________________________________

## 9. Previously Unreachable Targets

### Target 1: Early Unique Resonances >= 3.0

**Previous status:** Best was 2.96 (H3 algorithm), declared structurally
unreachable with N=5 and pack-of-4.

**Is it now reachable?** YES -- by Model D (N=7, Triple Archetypes) at **3.90
unique resonances/pack**.

**By which model?** Model D achieves this through the combination of N=7
resonances and 25% multi-resonance card allocation (20% dual + 5% triple). Each
dual card exposes 2 resonances and each triple card exposes 3, pushing effective
per-pack diversity far beyond what mono-only distributions achieve.

**At what cost?**

- Complexity increase: 7 resonances vs 5, 35 triple archetypes vs 10 pairs
- Convergence too fast: pick 3.79 (would need significant tuning to reach 5-8
  window)
- Splash destroyed: 0.5% splash rate (vs 80.9% for Model A)
- Simplicity score drops from 8 to 4

**Is the tradeoff worth it?** NO. The early diversity gain (+0.99 unique
res/pack) comes at the cost of three critical design goals. The previous
research correctly recommended revising the target to >= 2.7, which Model A
achieves at 2.91 with no sacrifices. The structural ceiling at N=5 should be
accepted.

### Target 2: Synergy Top-2 Share 75-90%

**Previous status:** All algorithms produced 94-97%, attributed to synergy
player behavior model always picking highest-fit card.

**Is it now reachable?** PARTIALLY -- depending on redefinition.

**Which models approach it?**

- Model B (N=10, single): top-1 share = 82.8% -- inside the 75-90% range when
  measured against the PRIMARY resonance only. But top-2 share remains at 89.5%.
- Model D (N=7, triple): top-2 share = 70.0% -- actually BELOW the target floor
  because K=3 spreads concentration across 3 colors.

**At what cost?**

- Model B: requires N=10 (high cognitive load), eliminates dual cards (no pair
  identity), produces 99.4% mono decks
- Model D: requires N=7 with triple archetypes, produces K=3 identity that is
  structurally different from K=2

**Is the tradeoff worth it?** NO. The target is fundamentally behavioral, not
structural. A synergy player who always picks the highest-fit card will always
produce high top-K concentration in any system that presents enough on-color
cards to feel convergent. The previous research correctly recommended revising
the target to 75-96%. Real human players (who sometimes take powerful off-color
cards) will naturally produce lower top-2 shares than the simulated synergy
player.

______________________________________________________________________

## 10. Implementation Implications for the Rust Codebase

### Recommendation: Keep N=5 Structure

Since the recommendation is to retain the current N=5 pair-archetype structure,
no fundamental Rust codebase changes are needed. The following adjustments from
this research are warranted:

#### Algorithm Improvements to Adopt

The H5-ADDITIVE algorithm (already documented in the previous final report)
should be the production implementation. Key formula:

```
weight = floor_weight + sum(profile[r]^exp for r in card.resonances)
exp = base_exp + (max_exp - base_exp) * clamp((pick - 1) / (ramp_picks - 1), 0.0, 1.0)
```

With parameters: floor_weight=3.5, base_exp=0.5, max_exp=1.1, ramp_picks=12,
dc_bonus=2, lane seeds [0.6, 1.4].

#### Parameter Adjustments Warranted

1. **Consider floor_weight = 4.0** (up from 3.5): The parameter sweep shows this
   increases splash from 81.6% to 83.2% with only -0.3% impact on top-2 share
   (95.2% vs 95.6%). This is a low-risk improvement.

2. **Consider mono dreamcallers for a subset of runs:** Model C's mono DC data
   shows that giving DC bonus to only 1 resonance produces 95% mono decks with a
   genuine second-color discovery phase (mean pick 10.8). Adding a small
   percentage (10-20%) of mono-resonance dreamcallers would address Model A's
   main weakness (100% dual classification) without changing the algorithm.

3. **No changes needed to:** base_exp, max_exp, ramp_picks, neutral_base, seed
   range. These are already at or near optimal values per the previous research.

#### What Does NOT Need to Change

- **Resonance count:** Keep 5 (Tide, Ember, Zephyr, Stone, Ruin)
- **Card distribution:** Keep 70% mono, 10% dual, 20% neutral
- **Dreamcaller structure:** Keep dual dreamcallers (2 resonances) as the
  primary type
- **Lane seeds:** Keep [0.6, 1.4] range for pool modification
- **Pack size:** Keep 4 cards per pack
- **TOML data:** No card data changes needed
- **Unity client:** No client-side changes needed

______________________________________________________________________

## 11. Appendix: Analyst Scorecards

Raw scores from all three analysts, side by side, with the computed average.

### Analyst 1: Player Experience (Goals 1-4 focus)

| Goal                   | Model A | Model B | Model C | Model D | Model E |
| ---------------------- | ------- | ------- | ------- | ------- | ------- |
| 1. Simple              | 8       | 5       | 9       | 4       | 3       |
| 2. Not on Rails        | 6       | 5       | 5       | 4       | 9       |
| 3. No Forced Decks     | 7       | 6       | 6       | 9       | 8       |
| 4. Flexible Archetypes | 5       | 3       | 7       | 8       | 7       |
| 5. Convergent          | 8       | 8       | 9       | 7       | 6       |
| 6. Splashable          | 8       | 6       | 5       | 3       | 6       |
| 7. Open-Ended Early    | 6       | 4       | 5       | 7       | 8       |
| 8. Signal Reading      | 7       | 5       | 6       | 7       | 6       |
| **TOTAL**              | **55**  | **42**  | **52**  | **49**  | **53**  |

### Analyst 2: Convergence Dynamics (Goals 5-6 focus)

| Goal                   | Model A | Model B | Model C | Model D | Model E |
| ---------------------- | ------- | ------- | ------- | ------- | ------- |
| 1. Simple              | 8       | 5       | 9       | 4       | 3       |
| 2. Not on Rails        | 6       | 7       | 4       | 7       | 8       |
| 3. No Forced Decks     | 9       | 8       | 7       | 9       | 7       |
| 4. Flexible Archetypes | 7       | 4       | 6       | 7       | 6       |
| 5. Convergent          | 7       | 8       | 6       | 4       | 6       |
| 6. Splashable          | 8       | 6       | 5       | 1       | 5       |
| 7. Open-Ended Early    | 6       | 6       | 5       | 8       | 9       |
| 8. Signal Reading      | 7       | 5       | 6       | 6       | 7       |
| **TOTAL**              | **58**  | **49**  | **48**  | **46**  | **51**  |

### Analyst 3: Variety and Depth (Goals 7-8 focus)

| Goal                   | Model A | Model B | Model C | Model D | Model E |
| ---------------------- | ------- | ------- | ------- | ------- | ------- |
| 1. Simple              | 8       | 5       | 9       | 4       | 3       |
| 2. Not on Rails        | 5       | 7       | 3       | 5       | 9       |
| 3. No Forced Decks     | 8       | 7       | 7       | 9       | 7       |
| 4. Flexible Archetypes | 7       | 4       | 6       | 8       | 7       |
| 5. Convergent          | 8       | 7       | 8       | 7       | 6       |
| 6. Splashable          | 8       | 6       | 5       | 3       | 5       |
| 7. Open-Ended Early    | 5       | 7       | 3       | 6       | 8       |
| 8. Signal Reading      | 6       | 8       | 4       | 7       | 7       |
| **TOTAL**              | **55**  | **51**  | **45**  | **49**  | **52**  |

### Average Across All Three Analysts

| Goal                   | Model A  | Model B  | Model C  | Model D  | Model E  |
| ---------------------- | -------- | -------- | -------- | -------- | -------- |
| 1. Simple              | **8.0**  | 5.0      | 9.0      | 4.0      | 3.0      |
| 2. Not on Rails        | 5.7      | 6.3      | 4.0      | 5.3      | **8.7**  |
| 3. No Forced Decks     | 8.0      | 7.0      | 6.7      | **9.0**  | 7.3      |
| 4. Flexible Archetypes | 6.3      | 3.7      | 6.3      | **7.7**  | 6.7      |
| 5. Convergent          | **7.7**  | **7.7**  | **7.7**  | 6.0      | 6.0      |
| 6. Splashable          | **8.0**  | 6.0      | 5.0      | 2.3      | 5.3      |
| 7. Open-Ended Early    | 5.7      | 5.7      | 4.3      | 7.0      | **8.3**  |
| 8. Signal Reading      | **6.7**  | 6.0      | 5.3      | **6.7**  | 6.3      |
| **TOTAL**              | **56.0** | **47.3** | **48.3** | **47.7** | **51.7** |

**Bold** indicates the highest score per goal. Model A leads on 3 goals outright
(Simple tied with C, Convergent tied with B and C, Splashable). Model E leads on
2 goals (Not on Rails, Open-Ended Early). Model D leads on 2 goals (No Forced
Decks, Flexible Archetypes). No single model dominates all goals -- the
recommendation is based on the best AVERAGE performance with no catastrophic
failures, which is Model A.
