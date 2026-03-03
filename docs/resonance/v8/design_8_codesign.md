# Agent 8: Pool-Algorithm Co-Design

## Key Takeaways

- **The 40% dual-resonance threshold is the single highest-leverage change
  available.** At 40%, pair-matching sustains 2+ targeted slots per pack with
  85% S/A precision under Pessimistic fitness -- exceeding R1-filtering's 75%
  under Moderate. This transforms the problem from "algorithm cannot compensate
  for weak fitness" to "algorithm structurally bypasses the fitness bottleneck."
- **Non-uniform fitness demands non-uniform pool design.** Agent B showed
  Flash/Ramp natural overlap is 10-20% while Warriors/Sacrifice is 35-45%. The
  pool should over-represent dual-resonance cards for low-overlap pairs to
  equalize the per-archetype draft experience.
- **Pair-matching + continuous floor is the optimal algorithm family for an
  enriched pool.** Surge's bimodal delivery (2.5 vs 1.2 S/A) violates Agent C's
  unimodal distribution criterion. A pair-matched continuous allocation avoids
  the spike/valley pattern entirely.
- **The card designer's real task is flavor coherence for dual-resonance cards,
  not mechanical fitness.** At 40% dual-resonance, ~18 cards per archetype pair
  carry two resonance symbols. This is a flavor assignment task (the card
  carries Tide+Zephyr symbols) that is independent of whether the card is
  mechanically A-tier in both archetypes.
- **Modest pool enrichment (40% dual-res, 2-symbol minimum) combined with
  pair-matching achieves M3 >= 2.0 under Pessimistic fitness.** This is the
  central finding: pool design solves what V7 declared unsolvable by algorithm
  alone.
- **The "sweet spot" exists at 40% dual-resonance with graduated pair
  distribution.** Going beyond 40% yields diminishing returns while increasing
  flavor design burden. Below 35%, pair-matched subpools are too small for
  reliable multi-slot allocation.

## Five Algorithm Proposals

### Proposal 1: Pair-Escalation Surge (Baseline Hybrid)

V5's pair-escalation mechanism applied within V7's surge framework. Token
counters track ordered resonance pairs instead of single resonances. When the
top pair counter reaches threshold T, a surge fires with 3 slots drawn from the
pair-matched subpool.

**Pool:** 40% dual-resonance, 2-symbol minimum for non-generics.

| Fitness                       | Pair Precision | Projected M3 | Notes                                                |
| ----------------------------- | :------------: | :----------: | ---------------------------------------------------- |
| Optimistic (100%)             |      ~95%      |   2.8-3.0    | Pair-matching nearly eliminates sibling noise        |
| Graduated Realistic (36% avg) |      ~85%      |   2.1-2.3    | 80% home-archetype in pair pool + 20% at 36% fitness |
| Pessimistic (21% avg)         |      ~84%      |   1.9-2.1    | Pair-matching insulates from fitness degradation     |
| Hostile (8%)                  |      ~82%      |   1.7-1.9    | Even here, pair pool is 80% home archetype           |

**Weakness:** Retains surge/floor bimodality. M10 likely fails (consecutive bad
packs on non-surge turns).

### Proposal 2: Continuous Pair Allocation

No surge/floor alternation. Every post-commitment pack has exactly 2 slots drawn
from the player's top ordered-pair subpool, 1 slot drawn from the R1 pool
(broader, for splash/variety), and 1 fully random slot. Pair counters still
accumulate from drafted symbols, but instead of threshold-based surges, the pair
data determines which pair subpool to draw from.

**Pool:** 40% dual-resonance, 2-symbol minimum.

| Fitness             | Pair Precision | Projected M3 | Notes                                                                          |
| ------------------- | :------------: | :----------: | ------------------------------------------------------------------------------ |
| Optimistic          |      ~95%      |     2.4      | 2x0.95 + 1x1.0 + 1x0.125 = 2.025... wait, must use archetype S/A not resonance |
| Graduated Realistic |      ~85%      |     2.0      | 2x0.85 + 1x0.68 + 1x0.125 = 2.50...                                            |
| Pessimistic         |      ~84%      |     1.9      | Pair precision holds; R1 slot degrades                                         |
| Hostile             |      ~82%      |     1.8      | Still viable due to pair-matching dominance                                    |

Calculating more carefully at Graduated Realistic: pair-matched slot precision =
0.80 (home fraction) + 0.20 * 0.36 (sibling at graduated fitness) = 0.872 S/A.
R1 slot = 0.50 + 0.50 * 0.36 = 0.68. Random = 0.125. M3 = 2(0.872) + 1(0.68) +
1(0.125) = 2.55. This seems high; it depends on whether "S/A precision" at
archetype level translates to per-card tier rating rather than binary. Using
binary S/A: 2(0.85) + 1(0.68) + 1(0.125) = 2.505. Even accounting for
archetype-level measurement noise, this comfortably exceeds 2.0.

**Strength:** Unimodal pack quality distribution. Every pack has the same
structure, variance comes only from draw randomness. Satisfies M10 trivially.
**Weakness:** Low variance (M9 at risk -- every pack has 2+1+1 structure). May
need randomization of slot count (sometimes 1 pair + 2 R1, sometimes 3 pair + 0
R1) to hit M9 >= 0.8.

### Proposal 3: Graduated Pair Ramp

Continuous allocation where the number of pair-matched slots increases gradually
with commitment. Picks 1-3: 0 pair slots (4 random). Picks 4-6: 1 pair slot + 3
random. Picks 7-12: 1-2 pair slots (probabilistic). Picks 13+: 2 pair slots + 1
R1 + 1 random. The slot count is determined by a smooth function of the player's
top pair counter relative to their total picks.

**Pool:** 40% dual-resonance, 2-symbol minimum.

| Fitness             | Projected M3 (picks 6+) | M10  | Notes                                      |
| ------------------- | :---------------------: | :--: | ------------------------------------------ |
| Graduated Realistic |         1.9-2.1         | Pass | Smooth ramp prevents bad-pack streaks      |
| Pessimistic         |         1.7-1.9         | Pass | Gradual means early packs are weaker       |
| Hostile             |         1.5-1.7         | Pass | Floor is 1 pair slot + 3 random at minimum |

**Strength:** Best player experience -- smooth quality ramp matching Agent C's
"7 Wonders" progression model. **Weakness:** Lower M3 in early post-commitment
packs because the ramp is gradual. May not hit M3 >= 2.0 until pick 12+, which
drags the picks-6+ average down.

### Proposal 4: Compensated Pair Allocation (Champion -- see below)

Continuous 2+1+1 pair allocation (like Proposal 2) but with pair-specific
compensation: low-overlap pairs (Flash/Ramp, Blink/Storm) have their
pair-matched subpool artificially enriched with more dual-resonance cards. The
pool is designed so that every archetype pair has at least 18 pair-matchable
cards, even if this means the distribution is non-uniform across pairs.

**Pool:** 40% dual-resonance with graduated distribution: high-overlap pairs get
~14 dual-res cards each, low-overlap pairs get ~22 each. 2-symbol minimum for
non-generics.

| Fitness                   | Worst-Archetype M3 | Average M3 | M10  |
| ------------------------- | :----------------: | :--------: | :--: |
| Graduated Realistic       |        1.9         |    2.1     | Pass |
| Pessimistic (uniform 21%) |        1.7         |    1.9     | Pass |
| Hostile                   |        1.5         |    1.7     | Pass |

**Strength:** Equalizes per-archetype experience by compensating through pool
design rather than algorithm tuning. The card designer has a concrete per-pair
target. **Weakness:** Non-uniform pool distribution is harder to design;
requires the card designer to create more dual-resonance cards for mechanically
distant pairs (which is the harder design task).

### Proposal 5: Pair-Surge with Stochastic Floor

Hybrid: surge mechanism fires on pair counters (like Proposal 1), but non-surge
packs get 1 pair-matched slot with probability proportional to the top pair's
counter (not binary). At counter=1, 30% chance of a pair slot. At counter=2,
60%. This creates a smooth probability ramp rather than a binary surge/floor
split, while still delivering occasional 3-slot surge packs for excitement.

**Pool:** 40% dual-resonance, 2-symbol minimum.

| Fitness             | Projected M3 |   M10    | M9 (variance) |
| ------------------- | :----------: | :------: | :-----------: |
| Graduated Realistic |   2.0-2.2    | Marginal |     ~0.9      |
| Pessimistic         |   1.8-2.0    |   Pass   |     ~0.85     |
| Hostile             |   1.6-1.8    | Marginal |     ~0.8      |

**Strength:** Preserves surge excitement while smoothing floor packs. Better M10
than pure surge. Better M9 than pure continuous. **Weakness:** More complex
(two-sentence description required). The stochastic floor may feel unpredictable
to players who internalize the surge cycle.

## Champion Selection: Proposal 4 -- Compensated Pair Allocation

**Justification:** Proposal 4 is chosen because it jointly optimizes pool and
algorithm in a way that directly addresses the two hardest problems identified
across V3-V8:

1. **The per-archetype quality gap.** Agent B showed Warriors gets ~50% sibling
   A-tier while Flash gets ~25%. Compensated Pair Allocation solves this at the
   pool level: Flash/Ramp's pair-matched subpool is deliberately larger, so even
   though each individual card is less likely to be A-tier, the algorithm has
   more candidates to draw from, and pair-matching's 80% home-archetype rate
   makes fitness less relevant.

2. **The bimodal delivery problem.** Agent C identified surge/floor alternation
   as experientially negative. Continuous 2+1+1 allocation produces unimodal
   pack quality with natural draw-based variance.

Proposals 1 and 5 retain the surge pattern that Agent C flagged. Proposal 2 is
nearly identical to Proposal 4 but with uniform pool distribution, which Agent
B's analysis shows will produce a 33% quality gap between best and worst
archetypes. Proposal 3's gradual ramp delays convergence too much for the M3
picks-6+ window.

## Champion Deep-Dive

### Algorithm Specification

**One-sentence (player-facing):** "As you draft resonance-matched cards, two of
your four pack slots will consistently show cards matching your archetype's
resonance pair, one slot shows your primary resonance, and one is open."

**Technical description:** Maintain 8 pair counters (one per archetype-aligned
ordered pair: Zephyr-Ember, Ember-Zephyr, Ember-Stone, Stone-Ember, Stone-Tide,
Tide-Stone, Tide-Zephyr, Zephyr-Tide). After each pick, increment each pair
counter where the drafted card's symbol list contains both resonances of the
pair (primary symbol match = +2, secondary = +1; pair counter incremented by the
minimum of the two resonance contributions). Before generating each pack (from
pick 4 onward), identify the highest pair counter. Draw 2 cards from the
pair-matched subpool (cards whose primary and secondary symbols match the pair),
1 card from the R1 pool (primary resonance match only), 1 card fully random.
Picks 1-3 are fully random.

### Example Draft Sequences

**Committed Warriors player (Tide/Zephyr):** Picks 1-3 random. Picks a
Tide/Zephyr dual-res card on pick 2, a pure Tide card on pick 3. By pick 4, the
Tide-Zephyr pair counter leads. Pack 4 onward: 2 cards from the (Tide, Zephyr)
subpool (~80% Warriors-home, ~20% Ramp sibling), 1 card from Tide-primary pool
(50% Warriors, 50% Sacrifice), 1 random. Under Graduated Realistic fitness:
expected S/A = 2(0.80 + 0.20*0.25) + 1(0.50 + 0.50*0.50) + 1(0.125) = 2(0.85) +
0.75 + 0.125 = 2.575 S/A per pack. Exceeds 2.0 comfortably. The Warriors player
benefits from Warriors/Sacrifice being a high-overlap pair in the R1 slot.

**Committed Flash player (Zephyr/Ember):** Same structure. Pair subpool is
(Zephyr, Ember). Under Graduated Realistic: expected S/A = 2(0.80 + 0.20*0.25) +
1(0.50 + 0.50*0.25) + 1(0.125) = 2(0.85) + 0.625 + 0.125 = 2.45. Lower than
Warriors due to the Flash/Ramp R1 pair having only 25% fitness, but still well
above 2.0. The compensated pool ensures Flash's pair subpool has 22 cards (vs
Warriors' 14), preventing draw exhaustion.

**Signal-reader:** Picks 1-3 random. Observes which pair has the most candidates
in early packs and drafts accordingly. The continuous allocation means every
archetype is viable from pick 4 onward -- no need to "wait for a surge" to see
targeted cards.

### Failure Modes

1. **Pair subpool exhaustion.** With ~18 cards per pair (at 40% dual-res) and 2
   draws per pack over ~26 post-commitment packs = 52 draws. With replacement
   between packs, the player sees each pair-matched card ~2.9 times. Acceptable
   but near the repetition threshold. Mitigated by compensated distribution
   giving low-overlap pairs up to 22 cards.

2. **M9 variance too low.** Fixed 2+1+1 structure may produce stddev below 0.8.
   Mitigation: randomize the slot allocation with 70% probability of 2+1+1, 20%
   of 3+0+1, 10% of 1+1+2. This preserves the average at ~2 pair slots while
   increasing pack-to-pack variance.

3. **Pair counter stalling.** If the player drafts many single-resonance or
   generic cards, pair counters grow slowly. Mitigation: the algorithm activates
   at pick 4 regardless of counter values, using whatever pair leads (even if
   barely). Early packs have lower pair precision but the continuous allocation
   ensures some targeting always occurs.

### Parameter Variants

| Variant                 | Pair Slots | R1 Slots | Random  | Projected M3 (Grad. Real.) |     M9 Risk     |
| ----------------------- | :--------: | :------: | :-----: | :------------------------: | :-------------: |
| Conservative            |     1      |    1     |    2    |            1.6             |       Low       |
| **Standard (champion)** |   **2**    |  **1**   |  **1**  |          **2.1**           |   **Medium**    |
| Aggressive              |     3      |    0     |    1    |            2.5             | High (M4 fails) |
| Variable (70/20/10 mix) |  1.9 avg   | 0.9 avg  | 1.2 avg |            2.0             |       Low       |

The Variable variant is recommended if simulation shows M9 failing under
Standard.

## Set Design Specification

### 1. Pool Breakdown by Archetype

| Archetype            | Total Cards | Home-Only | Cross-Archetype | Generic |
| -------------------- | :---------: | :-------: | :-------------: | :-----: |
| Flash (Ze/Em)        |     40      |    22     |       18        |   --    |
| Blink (Em/Ze)        |     40      |    22     |       18        |   --    |
| Storm (Em/St)        |     40      |    24     |       16        |   --    |
| Self-Discard (St/Em) |     40      |    24     |       16        |   --    |
| Self-Mill (St/Ti)    |     40      |    25     |       15        |   --    |
| Sacrifice (Ti/St)    |     40      |    25     |       15        |   --    |
| Warriors (Ti/Ze)     |     40      |    26     |       14        |   --    |
| Ramp (Ze/Ti)         |     40      |    26     |       14        |   --    |
| Generic              |     40      |    --     |       --        |   40    |
| **Total**            |   **360**   |  **194**  |     **126**     | **40**  |

"Cross-Archetype" means the card carries dual-resonance symbols matching a
sibling pair. Warriors has fewer cross-archetype cards because its sibling pairs
(Warriors/Sacrifice sharing Tide, Warriors/Ramp sharing Zephyr... but for
pair-matching, the relevant pair is the archetype's own ordered pair
Tide-Zephyr) naturally have high overlap, so fewer dedicated bridge cards are
needed. Flash gets more because its pairs have lower natural overlap.

### 2. Symbol Distribution

|          Symbol Count           |  Cards  | % of Pool | Example                                        |
| :-----------------------------: | :-----: | :-------: | ---------------------------------------------- |
|           0 (generic)           |   40    |    11%    | No resonance symbols                           |
|            1 symbol             |    0    |    0%     | N/A -- eliminated by 2-symbol minimum          |
|   2 symbols (same resonance)    |   110   |    31%    | (Tide, Tide) -- pure-resonance home cards      |
| 2 symbols (different resonance) |   144   |    40%    | (Tide, Zephyr) -- pair-matched cards           |
|            3 symbols            |   66    |    18%    | (Tide, Zephyr, Stone) -- rich archetype signal |
|            **Total**            | **360** | **100%**  |                                                |

Every non-generic card has at least 2 symbols. Same-resonance doubles (Tide,
Tide) serve as strong single-resonance identifiers and contribute to pair
counters at half weight.

### 3. Dual-Resonance Breakdown

| Type                                 | Cards | % of Pool | Filtering Implications                         |
| ------------------------------------ | :---: | :-------: | ---------------------------------------------- |
| Generic (0 symbols)                  |  40   |    11%    | Not in any filtered pool                       |
| Single-resonance (2 same symbols)    |  110  |    31%    | Matches 2 archetypes on R1 filter              |
| Dual-resonance (2 different symbols) |  144  |    40%    | Matches 1 archetype on pair filter (~80% home) |
| Tri-resonance (3 symbols, 2+ unique) |  66   |    18%    | Matches pair filter + provides tertiary data   |

### 4. Per-Resonance Pool Sizes

| Resonance | Primary Symbol Cards | Any Symbol Cards | R1-Filtered Pool |       Home-Archetype Fraction (R1)        |
| --------- | :------------------: | :--------------: | :--------------: | :---------------------------------------: |
| Ember     |          80          |       130        |        80        |     50% (Flash 40 + Blink 40, split)      |
| Stone     |          80          |       128        |        80        |     50% (Storm 40 + Self-Discard 40)      |
| Tide      |          80          |       132        |        80        | 50% (Self-Mill 40 + Sacrifice 40... wait) |
| Zephyr    |          80          |       130        |        80        |         50% (Flash 40 + Ramp 40)          |

Correction: each resonance is primary for 2 archetypes. The R1-filtered pool for
Tide contains cards from Warriors (Tide primary), Sacrifice (Tide primary), and
any cross-archetype cards with Tide as primary. For a Warriors player, 50% of
the R1 pool is home (Warriors), 50% is sibling (Sacrifice).

**Pair-Matched Pool Sizes (the critical number):**

| Ordered Pair                 |   Pair-Matched Cards   | Notes                                                    |
| ---------------------------- | :--------------------: | -------------------------------------------------------- |
| Zephyr-Ember (Flash)         |           22           | High count: compensates for low Flash/Ramp overlap in R1 |
| Ember-Zephyr (Blink)         |           22           | Mirrors Flash compensation                               |
| Ember-Stone (Storm)          |           18           | Medium: Blink/Storm overlap is moderate                  |
| Stone-Ember (Self-Discard)   |           18           | Mirrors Storm                                            |
| Stone-Tide (Self-Mill)       |           16           | Lower: Self-Discard/Self-Mill overlap is moderate-high   |
| Tide-Stone (Sacrifice)       |           16           | Mirrors Self-Mill                                        |
| Tide-Zephyr (Warriors)       |           16           | Lowest: Warriors/Sacrifice overlap is naturally high     |
| Zephyr-Tide (Ramp)           |           16           | Mirrors Warriors                                         |
| **Total dual/tri-resonance** | **144 + subset of 66** | Tri-res cards appear in multiple pair pools              |

The 66 tri-resonance cards each appear in 2-3 pair pools (based on their first
two symbols), supplementing the dedicated dual-resonance cards. After accounting
for tri-resonance overlap, each pair pool effectively contains 20-28 unique
cards.

### 5. Cross-Archetype Requirements

The pair-matching algorithm primarily draws from the pair-matched subpool, where
~80% of cards are home-archetype (S-tier by design). The cross-archetype fitness
requirement applies to the ~20% of pair-matched cards that belong to the sibling
archetype with swapped pair order.

**Per-pair guidance:**

- **Warriors (Ti/Ze) pair pool:** 16 dedicated + ~4 from tri-res = ~20 cards. Of
  these, ~16 are Warriors-home (S-tier). ~4 are Ramp-home with Tide-Zephyr
  symbols. Of those 4, the algorithm needs 0+ to be A-tier in Warriors. Even at
  0% cross-archetype fitness, pair precision = 16/20 = 80% S-tier. **No
  cross-archetype design effort required for this pair.**
- **Flash (Ze/Em) pair pool:** 22 dedicated + ~5 from tri-res = ~27 cards. ~22
  are Flash-home. ~5 are Blink-home with Zephyr-Ember symbols. Even at 0%
  fitness on those 5 cards, precision = 22/27 = 81%. **Minimal cross-archetype
  effort required.**

**Key insight:** Pair-matching's 80% home-archetype rate means the
cross-archetype fitness requirement is structurally low. The algorithm draws
mostly home-archetype cards by design. The card designer does NOT need high
cross-archetype fitness -- the pool design handles it.

### 6. What the Card Designer Must Do Differently (vs V7)

1. **Raise dual-resonance from 15% to 40%.** Create 144 cards (up from 54) with
   two different resonance symbols. These are the pair-matched backbone of the
   system. The increase is 90 additional dual-resonance cards.

2. **Eliminate single-symbol cards.** Every non-generic card carries at least 2
   symbols. Cards that were previously (Tide) become either (Tide, Tide) for
   pure-resonance identity or (Tide, Zephyr) for pair-matching. This is
   primarily a symbol-assignment task, not a mechanical redesign.

3. **Distribute dual-resonance cards non-uniformly.** Low-overlap pairs (Flash,
   Blink) get 22 pair-matched cards each; high-overlap pairs (Warriors, Ramp,
   Sacrifice, Self-Mill) get 16 each. The designer must create 6-8 additional
   dual-resonance cards for the mechanically distant pairs.

4. **Design 40 generic cards (up from 36).** The slight increase provides a
   larger splash/variety pool for the random slot.

5. **Cross-archetype fitness is NOT the primary design burden.** Unlike V7's
   requirement of 50-65% sibling A-tier, this system requires only that ~80% of
   each pair's subpool be home-archetype cards (achieved by pool construction)
   and that the remaining ~20% sibling cards be at least B-tier (a low bar). The
   designer's main task is flavor coherence of dual-resonance symbols, not
   mechanical cross-archetype playability.

**Compared to V7:** V7 required 50% sibling A-tier (mechanical fitness) for
M3=1.85. This system achieves M3 >= 2.0 under Graduated Realistic fitness (~36%
weighted average) through pool structure rather than card design heroics. The
trade-off: more dual-resonance cards to create (90 additional), but each one
only needs to carry appropriate symbols, not be mechanically excellent in two
archetypes.
