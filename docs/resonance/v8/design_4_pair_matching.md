# Agent 4: Pair-Matching Under Realistic Fitness

## Key Takeaways

- **Pair-matching is partially immune to fitness degradation.** Ordered-pair
  filtering concentrates draws in the home archetype (~80%), so precision under
  Pessimistic fitness is 0.80 + 0.20 * 0.25 = 85% S/A -- exceeding R1-filtering
  under Moderate (75%).
- **The binding constraint is subpool size, not precision.** Pair-matched
  subpools need 18+ cards per archetype pair for sustained 2+ slots/pack. This
  requires >= 40% dual-resonance cards.
- **V5 was never stress-tested.** Under Graduated Realistic (36% avg sibling
  A-tier), pair precision degrades unevenly: Tide pairs ~90%, Zephyr pairs ~82%.
  Concentration drops from 96.2% to ~80% -- actually fixing V5's failed metric.
- **Pair accumulation stalls under harsh fitness.** When pair-matched cards are
  B-tier, players choose power over pair contribution, pushing convergence from
  pick 6.3 to 9-11.
- **Smooth delivery is pair-matching's hidden advantage.** Per-slot probability
  rolls produce unimodal pack quality distributions, avoiding Surge+Floor's
  bimodal alternation.
- **M3 >= 2.0 under Pessimistic is mathematically achievable** with 40%+
  dual-res pool and pair-escalation at 2+ targeted slots/pack.

## Five Algorithm Proposals

### 1. V5 Pair-Escalation on Enriched Pool (PE-40)

Each slot independently shows a pair-matched card with P = min(pair_count/6,
0.50). Pool: 40% dual-resonance. At cap=0.50, 2.0 expected targeted slots
post-commitment.

| Fitness                   | Pair Precision | Projected M3 |
| ------------------------- | :------------: | :----------: |
| Optimistic                |      90%       |     2.05     |
| Graduated Realistic (36%) |      87%       |     1.94     |
| Pessimistic (25%)         |      85%       |     1.89     |
| Hostile (0%)              |      80%       |     1.79     |

Weakness: 18 cards/pair is marginal. M3 misses 2.0 under Pessimistic.

### 2. Pair-Escalation with Raised Cap (PE-60)

Cap=0.60, pool at 50% dual-res (~20 cards/pair). Expected targeted slots: 2.4.

| Fitness             | Projected M3 | Concentration |
| ------------------- | :----------: | :-----------: |
| Optimistic          |     2.58     |     ~94%      |
| Graduated Realistic |     2.30     |     ~87%      |
| Pessimistic         |     2.24     |     ~82%      |
| Hostile             |     2.08     |     ~76%      |

Clears 2.0 everywhere. Weakness: cap=0.60 leaves only 1.6 random slots; M4
splash marginal under Optimistic.

### 3. Pair-Surge Hybrid (PS-40)

At pair_count=4, spend 4 and fill 2 slots pair-matched + 2 random. Non-surge: 1
pair-matched floor slot. Pool: 40% dual-res.

| Fitness     | Blended M3 |
| ----------- | :--------: |
| Optimistic  |    1.69    |
| Pessimistic |    1.59    |

Weakness: Reintroduces bimodal delivery. Pairs accumulate slower than symbols,
so surges fire less often. Not competitive.

### 4. Graduated Pair-Escalation (GPE-45)

Two-phase probability: P = min(pair_count/8, 0.35) for picks 1-12, then P =
min(pair_count/5, 0.55) for picks 13+. Pool: 45% dual-res.

| Fitness             | Early M3 (6-12) | Late M3 (13+) | Blended M3 |
| ------------------- | :-------------: | :-----------: | :--------: |
| Optimistic          |      1.72       |     2.60      |    2.25    |
| Graduated Realistic |      1.60       |     2.38      |    2.05    |
| Pessimistic         |      1.54       |     2.31      |    1.99    |

Best experience profile: smooth ramp, no bimodal alternation. Two-sentence
description required.

### 5. Pair-Escalation + Symbol-Weight Boost (PEW-40)

P = min(weighted_score/8, 0.55), where primary-position pair match scores +2,
secondary +1. Pool: 40% dual-res, minimum 2 symbols per non-generic.

| Fitness     | Projected M3 | Convergence |
| ----------- | :----------: | :---------: |
| Optimistic  |     2.40     |     5.8     |
| Pessimistic |     2.08     |     7.2     |
| Hostile     |     1.90     |     8.5     |

Clears 2.0 under Pessimistic. Adds complexity via weighted scoring.

## Champion: Graduated Pair-Escalation (GPE-45)

**Why GPE-45:** (1) Best player experience -- smooth quality ramp satisfies
Research Agent C's criteria for unimodal distribution and gradual progression.
(2) Robust across fitness: only 12% M3 degradation from Optimistic to
Pessimistic vs. Surge+Floor's 47%. (3) Concentration naturally falls to 75-85%
under realistic fitness, fixing V5's failed metric. The two-sentence description
is justified by the experience improvement -- the player perceives only "my
packs gradually improve."

## Champion Deep-Dive

### Mechanism

**Phase 1 (picks 1-12):** Each drafted 2+ symbol card increments its ordered
pair counter. Per slot: P = min(top_pair_count/8, 0.35). Success = pair-matched
card; failure = random.

**Phase 2 (picks 13-30):** P = min(top_pair_count/5, 0.55). The divisor and cap
both increase, creating a step-up in targeting intensity.

### Example Draft: Warriors Player, Graduated Realistic

- **Picks 1-5:** Random packs, ~1.0 S/A. Drafts 3 (Tide,Zephyr) cards, count=3.
- **Picks 6-12:** P=0.35 (capped). ~1.4 targeted slots, ~1.7 S/A. Warriors
  forming with Sacrifice splashes.
- **Picks 13-20:** Phase 2. P=0.55 (capped). ~2.2 targeted slots, ~2.3 S/A.
  Packs have 2-3 playable cards.
- **Picks 21-30:** Steady 2.2 slots. Dead pack probability = 0.45^4 = 4.1%.

**Worst case -- Flash player, Pessimistic:** Pair precision = 80%*1.0 + 20%*0.10
= 82%. Late M3 = 2.2*0.82 + 1.8*0.125 = 2.03. Clears 2.0 even for the worst
pair.

### Failure Modes

1. **Subpool depletion:** ~24-26 pair-matched cards (18 dual-res + 6-8 tri-res).
   Drawing 2.2/pack over 18 packs = ~40 draws with replacement; each card seen
   ~1.7 times. Manageable.
2. **Stalled accumulators:** Exploratory players reaching pick 13 with no
   dominant pair see weaker phase-2 benefits. By design -- rewards commitment.
3. **Phase transition spike:** P jumps from 0.35 to 0.55 at pick 13. Smoothable
   via linear interpolation over picks 12-16.

### Parameter Variants

| Variant      | Phase 1 (div/cap) | Phase 2 (div/cap) | M3 (Pess.) |
| ------------ | :---------------: | :---------------: | :--------: |
| **Champion** |      8/0.35       |      5/0.55       |    1.99    |
| Aggressive   |      6/0.40       |      4/0.60       |   ~2.15    |
| Conservative |      10/0.30      |      6/0.50       |   ~1.82    |

## Set Design Specification

### 1. Pool Breakdown by Archetype

| Archetype            |  Total  | Home-Only | Cross-Archetype | Generic |
| -------------------- | :-----: | :-------: | :-------------: | :-----: |
| Each of 8 archetypes |   40    |    22     |       18        |   --    |
| Generic              |   40    |    --     |       --        |   40    |
| **Total**            | **360** |  **176**  |     **144**     | **40**  |

"Cross-Archetype" = cards with both the archetype's primary and secondary
resonance symbols (the pair-matchable cards).

### 2. Symbol Distribution

|     Symbol Count      | Cards | % of Pool | Role                                                |
| :-------------------: | :---: | :-------: | --------------------------------------------------- |
|      0 (generic)      |  40   |    11%    | No resonance                                        |
|       1 symbol        |  76   |    21%    | Archetype-defining; no pair data                    |
| 2 symbols (different) |  144  |    40%    | Cross-archetype dual-res; primary pair data source  |
| 3 symbols (different) |  100  |    28%    | Home-only with extra pair data from first 2 symbols |

Total pair-data-generating cards: 244 (68% of pool), ensuring pair data on most
picks.

### 3. Dual-Resonance Breakdown

| Type                          | Cards |  %  | Filtering Effect                          |
| ----------------------------- | :---: | :-: | ----------------------------------------- |
| Single-resonance              |  76   | 21% | R1 filter only; no pair matching          |
| Dual-resonance (ordered pair) |  144  | 40% | ~80% precision on pair filter             |
| Tri-resonance                 |  100  | 28% | Pair from first 2 symbols; ~80% precision |
| Generic                       |  40   | 11% | Random slots only                         |

### 4. Per-Resonance Pool Sizes

Each resonance has 80 cards as primary (2 archetypes x 40). Per ordered pair
(e.g., Tide,Zephyr for Warriors): 18 dual-res + ~6-8 tri-res whose first two
symbols match = **24-26 pair-matched cards**. Sufficient for 2.2 draws/pack over
18+ post-commitment packs.

### 5. Cross-Archetype Requirements

| Pair                           | Required Sibling A-Tier (Grad. Realistic) | Of 18 Dual-Res Cards | Difficulty |
| ------------------------------ | :---------------------------------------: | :------------------: | ---------- |
| Warriors/Sacrifice (Tide)      |                    50%                    |          9           | Low        |
| Self-Discard/Self-Mill (Stone) |                    40%                    |          7           | Medium     |
| Blink/Storm (Ember)            |                    30%                    |          5           | High       |
| Flash/Ramp (Zephyr)            |                    25%                    |          4           | High       |

Pair-matching's 80% home-archetype concentration means sibling fitness affects
only 20% of pair-matched draws, making even 25% sibling A-tier sufficient.

### 6. What the Card Designer Must Do Differently

1. **Create 144 dual-resonance cards (up from 54).** Each archetype needs 18
   cards with both primary and secondary resonance symbols. These need flavor
   coherence in both resonance families but can be mechanically narrow to the
   home archetype.
2. **Ensure 68% of cards have 2+ symbols** (244 of 360). Single-symbol cards
   become the minority (21%), reserved for archetype-defining effects.
3. **Per-pair fitness targets are modest.** Flash/Ramp (worst pair) needs only 4
   of 18 dual-res cards A-tier in the sibling -- achievable via generic utility
   (removal, draw) with zero deliberate bridging.
4. **The main cost is flavor design**, not mechanical design. A (Tide,Zephyr)
   card must feel thematically coherent but can be mechanically a pure Warriors
   card.
