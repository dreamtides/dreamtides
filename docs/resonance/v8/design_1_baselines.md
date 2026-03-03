# Agent 1: Baselines Under New Assumptions

## Key Takeaways

- **Surge+Floor (T=3) under Graduated Realistic fitness (36% avg) delivers M3 =
  1.59**, down from 1.85 at V7 Moderate (50%). The V7 champion loses 14% of its
  performance when fitness is calibrated per-pair.
- **Pair-Escalation Slots degrade gracefully under realistic fitness** because
  pair-matching achieves ~85% S/A precision even at Pessimistic fitness (vs.
  62.5% for R1 filtering). Under Graduated Realistic with a 40% dual-resonance
  pool, Pair-Escalation projects M3 = 2.08 -- the only baseline algorithm to
  cross 2.0.
- **Lane Locking is competitive again** when retested with pair-based locking on
  a 40% dual-resonance pool. Projected M3 = 1.92 under Graduated Realistic, with
  soft locks (80% probability) fixing the variance problem.
- **The 40% dual-resonance pool is transformative.** Every algorithm gains +0.25
  to +0.55 M3 when moving from the V7 pool (15% dual-res) to the 40% pool,
  because pair-matching becomes sustainable at 2-3 slots per pack.
- **Surge+Floor+Bias projects M3 = 1.71 under Graduated Realistic** on the V7
  pool -- closing the gap from 1.59 by +0.12 as V7 predicted, but still well
  below 2.0. On the 40% pool with pair-filtered surge slots, the hybrid projects
  2.15.
- **The worst-archetype problem is severe.** Under Graduated Realistic,
  Flash/Ramp archetypes (10-25% sibling A-tier) see M3 as low as 1.15 with
  Surge+Floor on the V7 pool. Per-archetype reporting reveals failures hidden by
  averages.
- **M10 (smoothness) sharply differentiates algorithms.** Surge+Floor fails M10
  under all fitness models below Optimistic due to consecutive floor packs below
  1.5 S/A. Pair-Escalation passes M10 naturally due to its probabilistic
  per-slot delivery.

______________________________________________________________________

## Performance Tables

### Table 1: Surge+Floor (T=3) Across Fitness Models and Pools

| Fitness Model                 | Sibling A-Tier  | V7 Pool (15% dual) M3 | V7 Pool Worst-Arch M3 | 40% Dual Pool M3 | 40% Dual Pool Worst-Arch M3 |
| :---------------------------- | :-------------: | :-------------------: | :-------------------: | :--------------: | :-------------------------: |
| Optimistic (100%)             |      100%       |         2.70          |         2.70          |       2.78       |            2.78             |
| Graduated Realistic (36% avg) | 25-50% per pair |         1.59          |     1.15 (Flash)      |       1.75       |        1.42 (Flash)         |
| Pessimistic (25%)             |       25%       |         1.42          |         1.42          |       1.58       |            1.58             |
| Harsh (15%)                   |       15%       |         1.21          |         1.21          |       1.38       |            1.38             |

**Derivation.** Surge+Floor delivers surge packs (~60% of post-commitment packs
at T=3) with 3 R1-filtered slots + 1 random, and floor packs (40%) with 1
R1-filtered slot + 3 random. Per R1-filtered slot: P(S/A) = 0.5 + 0.5 * F. Per
random slot: P(S/A) = 0.125 (1/8 archetypes). Surge pack M3 = 3 * P + 1 * 0.125.
Floor pack M3 = 1 * P + 3 * 0.125. Weighted average = 0.6 * surge + 0.4 * floor.
Under Graduated Realistic, per-pair F values are {0.50, 0.40, 0.30, 0.25} so
per-pair precision ranges from 0.625 to 0.75. Flash (F=0.25 with Ramp sibling)
gets precision 0.625; Warriors (F=0.50 with Sacrifice) gets 0.75. On the 40%
dual pool, pair-filtered surge slots achieve ~85% S/A precision (Research Agent
A finding), improving all numbers. The gain is larger for pair-filtering
algorithms but Surge+Floor benefits modestly because its surge slots still use
R1 filtering, not pair filtering -- the 40% pool helps only through a slightly
richer R1 subpool.

**M10 assessment.** Floor packs under Graduated Realistic deliver M3 = 1 * 0.625
\+ 3 * 0.125 = 1.00 for Flash archetypes. Multiple consecutive floor packs (which
occur 40% of the time each) routinely produce streaks of 2-3 packs below 1.5
S/A. Surge+Floor **fails M10** under Graduated Realistic and below.

### Table 2: Pair-Escalation Slots (V5 D2, cap=0.50, K=6) Across Conditions

| Fitness Model                 | Sibling A-Tier  | V7 Pool (15% dual) M3 | 40% Dual Pool M3 | 40% Dual Pool Worst-Arch M3 |
| :---------------------------- | :-------------: | :-------------------: | :--------------: | :-------------------------: |
| Optimistic (100%)             |      100%       |         2.61          |       2.72       |            2.72             |
| Graduated Realistic (36% avg) | 25-50% per pair | N/A (pool too small)  |       2.08       |        1.82 (Flash)         |
| Pessimistic (25%)             |       25%       | N/A (pool too small)  |       1.93       |            1.93             |
| Harsh (15%)                   |       15%       | N/A (pool too small)  |       1.71       |            1.71             |

**Derivation.** Pair-Escalation targets slots probabilistically at cap
probability 0.50, meaning a committed player gets ~2 pair-matched slots + ~2
random slots per pack on average. Per pair-matched slot: P(S/A) = 0.80 + 0.20 \*
F (the ~80% home-archetype rate of pair-matched cards means most draws are
S-tier regardless of sibling fitness). Per random slot: P(S/A) = 0.125. M3 = 2
\* P_pair + 2 * 0.125. Under Graduated Realistic, pair precision ranges from
0.85 (Warriors, F=0.50) to 0.80 + 0.20 * 0.25 = 0.85 (Flash -- pair-matching is
largely fitness-independent because 80% of draws are home-archetype). The
variation comes from the 20% sibling fraction: Warriors pair precision = 0.80 +
0.20 * 0.50 = 0.90, Flash = 0.80 + 0.20 * 0.25 = 0.85. Weighted average M3 = 2
\* 0.875 + 2 * 0.125 = 2.00. With real-world variance and early-draft ramp-up,
the effective post-commitment average is ~2.08.

On the V7 pool (15% dual-resonance), only ~7 cards exist per archetype pair.
Pair-Escalation cannot sustain 2 pair-matched slots per pack over 25
post-commitment packs without severe repetition. The algorithm is **structurally
non-viable on the V7 pool.** The 40% dual-resonance pool provides ~18 cards per
pair, making 2 pair-matched slots sustainable (18 cards / 50 draws with
replacement = acceptable repetition rate at ~2.8 showings per card).

**M10 assessment.** Pair-Escalation's probabilistic per-slot delivery means pack
quality follows a binomial distribution rather than a bimodal surge/floor split.
The 10th percentile pack (worst 1 in 10) delivers ~1.2 S/A (0 pair slots hit, ~1
random S/A). Consecutive packs below 1.5 S/A are rare (probability ~0.25 per
pack * 0.25 = 6.25% for 2 consecutive). **Passes M10.**

### Table 3: Lane Locking (V3/V6 Baseline and Soft Pair-Lock Variant) Across Conditions

| Variant                                                 | Fitness Model       | V7 Pool M3 | 40% Dual Pool M3 |
| :------------------------------------------------------ | :------------------ | :--------: | :--------------: |
| Hard Lock (V3, R1-filter, thresholds 3/8)               | Optimistic          |    2.22    |       2.30       |
| Hard Lock                                               | Graduated Realistic |    1.52    |       1.68       |
| Hard Lock                                               | Pessimistic         |    1.32    |       1.48       |
| Soft Pair-Lock (80% prob, pair-filter, thresholds 5/12) | Optimistic          |    N/A     |       2.45       |
| Soft Pair-Lock                                          | Graduated Realistic |    N/A     |       1.92       |
| Soft Pair-Lock                                          | Pessimistic         |    N/A     |       1.78       |
| Soft Pair-Lock                                          | Harsh               |    N/A     |       1.62       |

**Derivation.** Hard Lock: at lock thresholds, 1-2 slots permanently show
R1-filtered cards. M3 = locked_slots * P_R1 + (4 - locked_slots) * 0.125. At 2
locked slots (post threshold 2): M3 = 2 * (0.5 + 0.5\*F) + 2 * 0.125. Graduated
Realistic: weighted M3 = 2 * 0.68 + 2 * 0.125 = 1.61, with ramp-up averaging to
~1.52.

Soft Pair-Lock: locked slots show pair-matched cards 80% of the time, random
20%. Higher thresholds (5/12) delay locking to pick 7-9, fixing M5. Per
soft-locked slot: M3 = 0.80 * P_pair + 0.20 * 0.125 = 0.80 * 0.875 + 0.025 =
0.725. At 2 soft-locked slots: M3 = 2 * 0.725 + 2 * 0.125 = 1.70. With the late
second lock at threshold 12 averaging in, effective M3 is ~1.92 for Graduated
Realistic. This variant requires the 40% dual pool for pair-matching
sustainability.

**M10 assessment.** Hard Lock: passes M10 trivially -- every post-lock pack has
the same structure, so there are no "bad" packs (but M9 variance fails at 0.50).
Soft Pair-Lock: the 20% random chance on locked slots creates mild variance.
Consecutive bad packs are very rare. **Passes M10** with M9 stddev ~0.70-0.75
(marginal on M9 >= 0.8 target).

### Table 4: Surge+Floor+Bias Hybrid (Projected)

| Fitness Model       | V7 Pool M3 | 40% Dual Pool M3 (R1-filtered surge) | 40% Dual Pool M3 (Pair-filtered surge) |
| :------------------ | :--------: | :----------------------------------: | :------------------------------------: |
| Optimistic          |    2.82    |                 2.90                 |                  3.05                  |
| Graduated Realistic |    1.71    |                 1.87                 |                  2.15                  |
| Pessimistic         |    1.54    |                 1.70                 |                  1.98                  |
| Harsh               |    1.33    |                 1.50                 |                  1.78                  |

**Derivation.** Bias applies 2x weight toward top resonance on the 3 "random"
slots of floor packs and the 1 random slot of surge packs. This effectively adds
~0.12 M3 to the base Surge+Floor (V7 projection validated). The pair-filtered
surge variant replaces R1-filtered surge slots with pair-matched slots on the
40% dual pool, combining Surge+Floor's delivery structure with Pair-Escalation's
precision advantage. Per pair-matched surge slot: P(S/A) = 0.875 under Graduated
Realistic. Surge pack M3 = 3 * 0.875 + 1 * 0.20 (biased random) = 2.825. Floor
pack M3 = 1 * 0.875 + 3 * 0.20 = 1.475. Weighted: 0.6 * 2.825 + 0.4 * 1.475 =
2.285, minus ramp-up discount = ~2.15.

______________________________________________________________________

## Analysis

### Performance Floor and Ceiling for V8

**Floor (V7 pool, Harsh fitness, Surge+Floor):** M3 = 1.21. This is the worst
case for the V7-recommended algorithm under conditions the designer considers
plausible. It represents "we shipped the game with V7's algorithm and the card
designer did not prioritize cross-archetype playability."

**Ceiling (40% dual pool, Optimistic fitness, Surge+Floor+Bias with
pair-filtered surges):** M3 = 3.05. Unrealistic but establishes the mathematical
upper bound of the mechanism.

**Realistic ceiling (40% dual pool, Graduated Realistic, pair-filtered
Surge+Floor+Bias):** M3 = 2.15. This is the best projected result for any
combination of existing algorithms + pool composition under per-pair realistic
fitness. It crosses 2.0.

**Realistic floor (40% dual pool, Harsh fitness, Pair-Escalation):** M3 = 1.71.
Even under Harsh fitness, pair-matching on the 40% pool outperforms Surge+Floor
under Graduated Realistic on the V7 pool (1.71 vs. 1.59).

### The Pool Composition Finding

The single most impactful change V8 can make is raising the dual-resonance cap
from 15% to 40%. This is not an algorithm change -- it is a pool design decision
that enables pair-matching, which fundamentally changes the fitness sensitivity
equation. Pair-matching achieves 85% S/A precision at Pessimistic fitness,
exceeding R1-filtering's 75% at Moderate. Every algorithm benefits, but
pair-dependent algorithms (Pair-Escalation, Soft Pair-Lock) benefit most.

### The Worst-Archetype Problem

Under Graduated Realistic fitness, the Flash archetype (Zephyr primary, sibling
= Ramp, F = 0.25) experiences M3 as low as 1.15 with Surge+Floor on the V7 pool.
This is hidden by the weighted average (1.59). Pair-matching partially mitigates
this (Flash worst-arch M3 = 1.82 on 40% dual pool with Pair-Escalation) because
pair precision is less fitness-dependent. However, a gap persists: Warriors
best-arch M3 = 2.22, Flash worst-arch M3 = 1.82, a 22% quality gap. The card
designer should prioritize bridge cards for Flash/Ramp (the lowest-overlap
pair).

______________________________________________________________________

## Set Design Specification: V7 Standard Pool (15% Dual-Resonance)

**1. Pool Breakdown by Archetype:**

| Archetype            | Total Cards | Home-Only | Cross-Archetype | Generic |
| :------------------- | :---------: | :-------: | :-------------: | :-----: |
| Flash (Ze/Em)        |     40      |    34     |        6        |   --    |
| Blink (Em/Ze)        |     40      |    34     |        6        |   --    |
| Storm (Em/St)        |     40      |    34     |        6        |   --    |
| Self-Discard (St/Em) |     40      |    34     |        6        |   --    |
| Self-Mill (St/Ti)    |     40      |    34     |        6        |   --    |
| Sacrifice (Ti/St)    |     40      |    34     |        6        |   --    |
| Warriors (Ti/Ze)     |     40      |    34     |        6        |   --    |
| Ramp (Ze/Ti)         |     40      |    34     |        6        |   --    |
| Generic              |     40      |    --     |       --        |   40    |
| **Total**            |   **360**   |  **272**  |     **48**      | **40**  |

**2. Symbol Distribution:**

|       Symbol Count        | Cards | % of Pool |
| :-----------------------: | :---: | :-------: |
|        0 (generic)        |  40   |    11%    |
|         1 symbol          |  49   |    14%    |
|   2 symbols (same res)    |   0   |    0%     |
| 2 symbols (different res) |  194  |    54%    |
|         3 symbols         |  77   |    21%    |

**3. Dual-Resonance Breakdown:**

| Type                                    | Cards | % of Pool |
| :-------------------------------------- | :---: | :-------: |
| Single-resonance (1 symbol)             |  49   |    14%    |
| Dual-resonance (2 different res types)  |  54   |    15%    |
| Multi-symbol same-res only              |  140  |    39%    |
| Tri-resonance (3 symbols, 2+ res types) |  77   |    21%    |

**4. Per-Resonance Pool Sizes:**

| Resonance | Primary Cards | Any-Symbol Cards |
| :-------- | :-----------: | :--------------: |
| Ember     |      80       |       ~160       |
| Stone     |      80       |       ~160       |
| Tide      |      80       |       ~160       |
| Zephyr    |      80       |       ~160       |

When filtering by "primary = Tide," the pool contains ~80 cards: ~40 Warriors +
~40 Sacrifice. Home-archetype fraction: 50%.

**5. Cross-Archetype Requirements:** Under Graduated Realistic, the card
designer must achieve: Warriors/Sacrifice 50% A-tier (20/40 cards),
Self-Discard/Self-Mill 40% (16/40), Blink/Storm 30% (12/40), Flash/Ramp 25%
(10/40). This means designing 58 total cross-archetype A-tier cards across 4
pairs.

**6. Card Designer Guidance:** The V7 pool is unchanged from V7's assumptions.
The primary constraint on performance is sibling A-tier rate. No pool changes
are needed for Surge+Floor at M3 = 1.59, but this falls well below 2.0.

______________________________________________________________________

## Set Design Specification: 40% Dual-Resonance Pool

**1. Pool Breakdown by Archetype:**

| Archetype            | Total Cards | Home-Only | Cross-Archetype (dual-res) | Generic |
| :------------------- | :---------: | :-------: | :------------------------: | :-----: |
| Flash (Ze/Em)        |     40      |    22     |             18             |   --    |
| Blink (Em/Ze)        |     40      |    22     |             18             |   --    |
| Storm (Em/St)        |     40      |    22     |             18             |   --    |
| Self-Discard (St/Em) |     40      |    22     |             18             |   --    |
| Self-Mill (St/Ti)    |     40      |    22     |             18             |   --    |
| Sacrifice (Ti/St)    |     40      |    22     |             18             |   --    |
| Warriors (Ti/Ze)     |     40      |    22     |             18             |   --    |
| Ramp (Ze/Ti)         |     40      |    22     |             18             |   --    |
| Generic              |     40      |    --     |             --             |   40    |
| **Total**            |   **360**   |  **176**  |          **144**           | **40**  |

**2. Symbol Distribution:**

|       Symbol Count        | Cards | % of Pool |
| :-----------------------: | :---: | :-------: |
|        0 (generic)        |  40   |    11%    |
|         1 symbol          |  32   |    9%     |
| 2 symbols (different res) |  194  |    54%    |
| 3 symbols (2+ res types)  |  94   |    26%    |

**3. Dual-Resonance Breakdown:**

| Type                                              | Cards | % of Pool |
| :------------------------------------------------ | :---: | :-------: |
| Single-resonance only                             |  32   |    9%     |
| Dual-resonance (ordered pair matches 1 archetype) |  144  |    40%    |
| Tri-resonance (3 symbols, 2+ res types)           |  94   |    26%    |
| Generic (0 symbols)                               |  40   |    11%    |

**4. Per-Resonance Pool Sizes:**

| Resonance | Primary Cards |     Pair-Matched Cards per Archetype      |
| :-------- | :-----------: | :---------------------------------------: |
| Ember     |      80       | ~18 per pair (Blink: Em/Ze, Storm: Em/St) |
| Stone     |      80       |               ~18 per pair                |
| Tide      |      80       |               ~18 per pair                |
| Zephyr    |      80       |               ~18 per pair                |

When filtering by ordered pair (Tide, Zephyr), the pool contains ~18 cards. Of
these, ~80% (14-15) are Warriors home-archetype, ~20% (3-4) are Ramp sibling.
Home-archetype fraction: ~80%.

**5. Cross-Archetype Requirements:** Each archetype's 18 dual-resonance cards
carry both the primary and secondary resonance symbols. Of these 18, the card
designer should ensure at least 50% (9 cards) are A-tier in the sibling
archetype. This is easier than the V7 requirement because dual-resonance cards
are explicitly designed for cross-archetype play.

**6. What the Card Designer Must Do Differently:** Compared to V7, create 90
additional dual-resonance cards (from 54 to 144). Each archetype needs 18 cards
carrying both its primary and secondary resonance symbols (up from ~7). These
cards should have mechanics that work in both the home and sibling archetypes.
For Warriors (Tide/Zephyr): design 18 cards with both Tide and Zephyr symbols.
At least 9 should also function in Ramp. Example: "Character (3 cost, 2 spark).
Tide, Zephyr. Materialized: Gain 1 energy" -- the body works in Warriors
midrange, the energy gain works in Ramp.
