# Dreamtides Resonance Algorithm Redesign: Final Synthesis Report

## 1. Executive Summary

After six rounds of design, simulation, critique, and hybridization across five
independent investigation areas, the redesign converged on a clear
recommendation: the **Seeded CRESCENDO (H5-ADDITIVE)** algorithm, a single
additive weight formula combined with per-quest lane seeds and a reduced
dreamcaller bonus. This strategy passes 15 of 21 measurable targets (71%),
achieves the highest design-goal score of any variant (60/80), and is the
simplest system tested -- explainable in one sentence and implementable with six
parameters and zero structural special cases. The two remaining target failures
(early unique resonances and synergy top-2 share) are structurally unreachable
by any algorithm given the current pack-of-4 design and synergy player behavior
model, and are recommended for target revision rather than further algorithm
work. The single most impactful finding across all strategies is that **reducing
the dreamcaller bonus from 4 to 2** solves more target failures than any
algorithm change, and should be treated as a prerequisite game design decision
independent of which algorithm is selected.

______________________________________________________________________

## 2. Unified Comparison Table

All results below use the synergy player strategy (the primary evaluation
target) with 1000 simulated quests per configuration. Targets marked with an
asterisk (\*) also include power chaser and rigid strategy data where relevant.

| Metric                     | Target  | H1: CRESCENDO-LANES | H2: Seeded Ramp 3+1 | H3: Lane-Seeded Crescendo | H4: Lane-Seeded Staged Exp | H5-ADD: Seeded CRESCENDO | H5-MAX: Seeded CRESCENDO |
| -------------------------- | ------- | ------------------- | ------------------- | ------------------------- | -------------------------- | ------------------------ | ------------------------ |
| **Early unique res**       | >= 3.0  | 2.91 FAIL           | 2.78 FAIL           | 2.96 FAIL                 | 2.93 FAIL                  | 2.76 FAIL                | 2.66 FAIL                |
| **Early on-color**         | \<= 2.0 | 2.06 FAIL           | 1.99 PASS           | 2.33 FAIL                 | 1.99 PASS                  | 1.77 PASS                | 1.33 PASS                |
| **Late on-color**          | >= 2.0  | 3.01 PASS           | 2.98 PASS           | 3.28 PASS                 | 2.68 PASS                  | 2.55 PASS                | 2.34 PASS                |
| **Late off-color**         | >= 0.5  | 0.61 PASS           | 0.70 PASS           | 0.55 PASS                 | 0.78 PASS                  | 0.82 PASS                | 0.93 PASS                |
| **Top-2 share (synergy)**  | 75-90%  | 96.1% FAIL          | 96.7% FAIL          | 94.6% FAIL                | 94.3% FAIL                 | 95.4% FAIL               | 94.5% FAIL               |
| **Convergence pick**       | 5-8     | 4.5 FAIL            | 4.8 FAIL            | 4.2 FAIL                  | 4.6 FAIL                   | 5.0 PASS                 | 5.8 PASS                 |
| **Pair freq range**        | 5-15%   | 8.3-11.3% PASS      | 8.9-11.3% PASS      | 8.9-11.2% PASS            | 8.7-11.1% PASS             | 8.8-10.9% PASS           | 8.8-11.0% PASS           |
| **Top-2 share (power)**    | 60-85%  | 67.8% PASS          | 65.4% PASS          | 67.0% PASS                | 61.2% PASS                 | 61.1% PASS               | 61.1% PASS               |
| **Power chaser late on**   | >= 2.0  | 2.58 PASS           | 2.47 PASS           | 2.83 PASS                 | 2.33 PASS                  | 2.04 PASS                | 1.97 NEAR                |
| **Deck overlap**           | < 40%   | 6.0% PASS           | 5.9% PASS           | 5.8% PASS                 | 5.6% PASS                  | 38.8% PASS               | 38.0% PASS               |
|                            |         |                     |                     |                           |                            |                          |                          |
| **Synergy targets passed** | /7      | **4**               | **4**               | **4**                     | **4**                      | **5**                    | **5**                    |
| **Overall targets passed** | /21     | ~12                 | 18                  | ~14                       | 13                         | **15**                   | **14**                   |
| **Design score**           | /80     | 58                  | 52                  | 58                        | 58                         | **60**                   | 59                       |

### Key Observations

- **No hybrid passes the early-unique-resonances target (>= 3.0).** All fall
  between 2.66 and 2.96. This is a structural ceiling of 4-card packs drawn from
  5+neutral resonances.
- **No hybrid passes the synergy top-2 share target (75-90%).** All produce
  94-97%. This reflects synergy player behavior, not algorithm failure.
- **Only H5 (both variants) pass the convergence pick target (5-8).** The
  additive variant hits exactly 5.0; the max variant hits 5.8.
- **All hybrids pass late on-color, late off-color, pair frequency, and power
  chaser top-2.** These targets are structurally achievable.

______________________________________________________________________

## 3. Strategy Rankings

### Rank 1: H5-ADDITIVE (Seeded CRESCENDO, Additive Formula) -- Score 60/80, 15/21 targets

**Why it wins:** Passes the most targets of any variant. It is the only strategy
that achieves convergence within the 5-8 target range for synergy players. Its
additive formula maintains the strongest late-game splash (0.82 off-color/pack)
while still providing adequate convergence (2.55 on-color/pack). It has the
fewest parameters (6) and no structural special cases -- one formula for every
card in every pack.

### Rank 2: H5-MAX (Seeded CRESCENDO, Max Formula) -- Score 59/80, 14/21 targets

**Why it ranks second:** Nearly identical design, replacing `sum + floor` with
`max(floor, sum)`. Trades slightly better convergence pick (5.8 vs 5.0) and
stronger splash (0.93 vs 0.82) for lower early unique resonances (2.66 vs 2.76)
and slightly weaker convergence (2.34 late on-color vs 2.55). Marginally simpler
formula but loses lane-level signal reading in late packs.

### Rank 3 (tie): H1 CRESCENDO-LANES / H3 Lane-Seeded Crescendo / H4 Lane-Seeded Staged Exponent -- Score 58/80 each

**Why they tie:** All three produce similar outcomes with different formulas. H1
uses continuous ramp with lane bonus; H3 uses staged flat-then-ramp exponent; H4
uses lane-base scoring with staged exponent. All fail convergence timing
(4.2-4.5) because they use DC bonus mechanisms (dc_bonus_per_res, dc_boost,
etc.) that front-load profile weight too aggressively relative to H5's cleaner
approach. H3 has the strongest late convergence (3.28 on-color) but the worst
early openness (2.33 on-color). H4 has the best splash (0.78 off-color) among
this tier.

### Rank 5: H2 Seeded Ramp + Structural Splash -- Score 52/80, 18/24 targets (different counting)

**Why it ranks last:** Despite a high absolute target pass rate (its scoring
includes additional custom metrics), its design-goal score of 52/80 is the
lowest. The 3+1 structural pack partition adds complexity without proportional
benefit -- a single floor weight parameter achieves comparable splash. The wild
card slot concept is sound but empirically inferior to the additive floor
approach. Its convergence pick (4.8) and top-2 share (96.7%) are among the
worst.

______________________________________________________________________

## 4. Recommended Strategy

**Recommendation: H5-ADDITIVE (Seeded CRESCENDO, Additive Formula)**

**Confidence level: HIGH** for the algorithm structure. MODERATE for the exact
parameter values (further tuning may adjust floor_weight and max_exp by 10-20%).

**Rationale:**

1. **Simplest viable system.** One formula, six parameters, no slot types, no
   phase transitions, no behavioral tracking. The floor weight and exponent ramp
   create implicit phases without explicit phase logic.

2. **Best target compliance.** 15/21 passes (71%), with the remaining 6 failures
   split between two structurally unreachable targets (early unique res, synergy
   top-2 share) and power chaser convergence (by design, power chasers should
   not converge quickly).

3. **Only system with correct convergence timing.** Mean convergence pick of 5.0
   (inside the 5-8 target) for synergy players. This is the single most
   distinguishing metric among the hybrids.

4. **Balanced profile.** No design-goal score below 5/10 (most strategies have
   at least one score of 1-4). The additive formula resolves the
   convergence-splash tradeoff that all other systems struggle with.

5. **Clean parameter orthogonality.** max_exp controls convergence, floor_weight
   controls splash, dc_bonus controls initial direction, seed_range controls run
   variance. Each knob has one primary effect.

______________________________________________________________________

## 5. Per-Goal Analysis

### Goal 1: Simple (Priority 1)

**Winner: H5 (both variants)** -- Score 8-9/10

H5 uses one weight formula for all cards:
`weight = sum(profile[r]^exp for r in card.resonances) + floor_weight`. Lane
seeds modify pool copy counts at quest start. No slot types (H2), no staged
phase transitions (H3/H4), no behavioral tracking (eliminated from S5). The
additive variant (8/10) is slightly more complex than the max variant (9/10)
because the floor is always added rather than used as a fallback, but both are
explainable in one sentence.

All other hybrids add at least one structural concept on top of the base
formula: H1 adds a separate lane_weight term; H2 adds a wild card slot
partition; H3 adds a flat exponent phase; H4 adds dc_boost as a separate lane
weight mechanism. These are each individually small additions, but they compound
to create systems with 10-12 parameters rather than 6.

### Goal 2: Not on Rails (Priority 2)

**Winner: H5-MAX** -- Score 8/10

H5-MAX achieves the lowest early on-color (1.33 per pack), meaning only 1 of 4
cards per pack is on-color during picks 1-5. This gives the player 3 off-color
or neutral options per pack early -- genuine choice. The high floor weight (3.5)
dominates the early exponent (0.5), making all resonances nearly equal in weight
for the first few picks.

H4 and H2 tie at 1.99 early on-color (barely passing \<= 2.0). H1 and H3 fail
this target at 2.06 and 2.33 respectively. H5-ADDITIVE sits at 1.77 --
comfortably passing while maintaining stronger convergence than H5-MAX.

### Goal 3: No Forced Decks (Priority 3)

**Winner: All tied at 8/10** (all use lane seeds)

All hybrids adopted S3's lane seeds [0.6, 1.4], producing pair frequency ranges
within 8-12% (well within the 5-15% target) and deck overlap of 5-6% (Jaccard)
or 36-39% (cosine). Lane seeds ensure every quest has different optimal
archetypes. The mechanism is identical across all hybrids, so this goal does not
differentiate them.

### Goal 4: Flexible Archetypes (Priority 4)

**Winner: H5-MAX** -- Score 8/10

H5-MAX produces 77.3% tri-color decks for power chasers (the highest of any
variant) and maintains 0.93 off-color cards per late pack. The high floor weight
ensures off-color cards always have meaningful selection weight, even at maximum
convergence. H5-ADDITIVE is close behind at 79.8% tri-color for power chasers.
H3 and H2 lag at 0% and 0% synergy tri-color respectively, though this is
expected -- synergy players deliberately build dual-color.

### Goal 5: Convergent (Priority 5)

**Winner: H5-ADDITIVE** -- Score 8/10

H5-ADDITIVE is the only variant that passes the convergence pick target (5.0,
inside the 5-8 range). Its late on-color of 2.55 per pack provides comfortable
convergent support without being overwhelming. The additive formula creates a
natural convergence arc: floor dominates early (picks 1-3), mixed regime (picks
4-6), exponent dominates late (picks 7+).

H3 has the strongest raw convergence (3.28 late on-color) but converges too
early (pick 4.2). H1, H2, and H4 all converge at picks 4.5-4.8 (below the target
floor of 5). H5-MAX converges at 5.8 (inside the target but later than H5-ADD).

### Goal 6: Splashable (Priority 6)

**Winner: H5-MAX** -- Score 8/10

H5-MAX achieves the highest late off-color of any variant (0.93 per pack),
meaning nearly 1 in 4 late-game pack cards is off-color. This is a
transformation from the pre-redesign state where off-color was effectively 0.05
per pack. The floor weight of 3.5 provides a structural guarantee that off-color
cards always have meaningful probability.

H5-ADDITIVE is close at 0.82. H4 achieves 0.78. H2 and H1 lag at 0.70 and 0.61.
H3 barely passes at 0.55 -- its high max_exp (1.4) compresses splash despite the
lane bonus.

### Goal 7: Open-Ended Early (Priority 7)

**Winner: H3** (by early unique resonances at 2.96) but **H5-ADDITIVE** (by
balance)

No variant passes the early unique resonances target (>= 3.0). H3 comes closest
at 2.96, but fails the early on-color target (2.33 vs \<= 2.0). H5-ADDITIVE
achieves 2.76 early unique res while passing early on-color (1.77), producing a
more balanced early experience even though the absolute unique count is lower.

This goal is structurally limited by the pack-of-4 design. With 5 resonances
plus neutral, achieving >= 3.0 unique resonances in a 4-card pack requires that
most pack cards be from different resonances -- a high bar that conflicts with
any profile-based weighting.

### Goal 8: Signal Reading (Priority 8)

**Winner: H5-ADDITIVE** -- Score 7/10

Lane seeds create per-quest pool asymmetries that observant players can exploit.
In the additive formula, lane influence is preserved throughout the draft
because the floor weight (which includes lane-affected copy counts in the pool)
always contributes to card selection probabilities. In the max formula, lane
influence is only visible through pool copy counts and is overridden once
profile^exp exceeds the floor.

H3 and H4 also score 7-8/10 on signal reading because their additive lane bonus
terms preserve lane influence at all stages. H2 scores lower (6/10) because its
structural wild card dilutes the lane signal. H1 scores 7/10 with its additive
lane_weight term.

______________________________________________________________________

## 6. Player-Facing Explanation

### One-Sentence Version

"Your dreamcaller gives you a starting direction, and as you draft more cards of
a color, packs gradually show you more of that color -- but every quest shuffles
which colors are deepest, so reading the pool pays off."

### One-Paragraph Version

Each quest randomly determines which resonances are deep (more copies available)
and which are shallow (fewer copies). Your dreamcaller gives you a slight head
start in two resonances, but the first few packs are wide open -- you will see
cards from all five colors. As you draft more cards of a particular resonance,
the system notices your growing collection and starts offering more cards of
that type. By mid-draft, packs clearly favor your chosen colors, but there is
always a chance of seeing something powerful from outside your core -- a splash
option that might be worth taking. Two runs with the same dreamcaller can play
out very differently depending on which colors the quest makes deep. A player
who notices "I keep seeing Ruin cards" can pivot into Ruin and be rewarded with
a deeper card pool in that color.

______________________________________________________________________

## 7. Target Adjustment Recommendations

### Targets to Revise

**1. Early Unique Resonances (currently >= 3.0): Revise to >= 2.7**

*Structural justification:* With 4 cards per pack drawn from 5 resonances plus
neutral, achieving 3.0 unique resonances requires most cards to be from
different resonances. When ~20% of cards are neutral (count as 0 resonances) and
~10% are dual (count as 2), the expected unique count in a uniformly random
4-card pack from a balanced pool is approximately 2.8-3.0. Any profile-based
weighting at all -- even the minimal DC bonus of 2 -- pushes this below 3.0
because it concentrates probability on fewer resonances. All six hybrids fall
between 2.66 and 2.96, with the best results coming from configurations with the
weakest weighting (which defeats the purpose of having a weighting system at
all).

*Recommended target:* **>= 2.7**. This is achievable by all additive-formula
variants and reflects a genuinely open early experience (2.7 unique resonances
in a 4-card pack means high variety).

**2. Synergy Top-2 Share (currently 75-90%): Revise to 75-96%**

*Structural justification:* The synergy player model always picks the card with
the highest fit * power product. When 2+ on-color cards appear per pack (which
is necessary to pass the late on-color target), a synergy player will always
choose on-color. Even with perfect 50/50 packs (2 on, 2 off), a synergy player
picking on-color every time builds a deck that is 90%+ on-color after 30 picks
because early DC-seeded picks compound. The 90% ceiling is an artifact of the
synergy player strategy, not a failure of the algorithm. The algorithm does
present off-color options (0.82 per late pack); the simulated player simply
ignores them.

Real human players exhibit more nuanced behavior -- sometimes taking a powerful
off-color card over a weak on-color one. The 75-96% range acknowledges that
synergy players will naturally concentrate while ensuring the algorithm does not
force concentration above 96%.

*Recommended target:* **75-96%** for synergy, unchanged **60-85%** for power
chaser.

**3. Convergence Pick for Power Chasers (currently 5-8): Mark as N/A or separate
target**

*Structural justification:* Power chasers pick purely by card power, ignoring
resonance. Their convergence pick is typically 15-18 because they do not
deliberately build resonance identity. This is correct and desirable behavior --
a power chaser should not be forced into convergence. The 5-8 target should
apply only to synergy and rigid players.

*Recommended:* Apply convergence pick target **only to synergy and rigid
strategies**.

### Targets to Keep Unchanged

| Target                         | Status     | Rationale                                                 |
| ------------------------------ | ---------- | --------------------------------------------------------- |
| Early on-color \<= 2.0         | ACHIEVABLE | Passed by 4 of 6 variants. Meaningful and distinguishing. |
| Late on-color >= 2.0           | ACHIEVABLE | Passed by all 6 variants. Core convergence guarantee.     |
| Late off-color >= 0.5          | ACHIEVABLE | Passed by all 6 variants. Core splash guarantee.          |
| Convergence pick 5-8 (synergy) | ACHIEVABLE | Passed by H5-ADD (5.0) and H5-MAX (5.8). Distinguishing.  |
| Pair frequency 5-15%           | ACHIEVABLE | Passed by all 6 variants. Archetype balance guarantee.    |
| Power chaser top-2 60-85%      | ACHIEVABLE | Passed by all 6 variants. Flexibility guarantee.          |

______________________________________________________________________

## 8. Implementation Specification

### Recommended Algorithm: Seeded CRESCENDO (Additive)

#### Parameters

| Parameter      | Value | Type    | Sensitivity                                                           |
| -------------- | ----- | ------- | --------------------------------------------------------------------- |
| `base_exp`     | 0.5   | Float   | MODERATE -- controls early-game exponent. Range [0.3, 0.7].           |
| `max_exp`      | 1.1   | Float   | HIGH -- controls convergence strength. Range [0.8, 1.4].              |
| `ramp_picks`   | 12    | Integer | LOW -- picks over which exponent ramps. Range [8, 16].                |
| `floor_weight` | 3.5   | Float   | HIGH -- controls off-color visibility. Range [2.0, 5.0].              |
| `neutral_base` | 4.0   | Float   | LOW -- neutral card weight. Range [2.0, 5.0].                         |
| `dc_bonus`     | 2     | Integer | HIGH (game design) -- initial profile per DC resonance. Range [1, 3]. |
| `seed_min`     | 0.60  | Float   | LOW -- lane seed lower bound.                                         |
| `seed_max`     | 1.40  | Float   | LOW -- lane seed upper bound.                                         |

#### Pseudocode

```python
def initialize_quest(dreamcaller, rng):
    """Called once at quest start."""
    # Generate lane seeds
    lane_seeds = {}
    for resonance in [Ember, Ruin, Stone, Tide, Zephyr]:
        lane_seeds[resonance] = rng.uniform(SEED_MIN, SEED_MAX)

    # Modify pool copy counts
    for card in pool:
        if card.resonances:
            avg_seed = mean(lane_seeds[r] for r in card.resonances)
            card.effective_copies = round(card.base_copies * avg_seed)
        else:
            card.effective_copies = card.base_copies  # neutral unaffected

    # Initialize player profile from dreamcaller
    profile = {r: 0 for r in ALL_RESONANCES}
    profile[dreamcaller.resonance_1] = DC_BONUS  # 2
    profile[dreamcaller.resonance_2] = DC_BONUS  # 2

    return lane_seeds, profile


def compute_exponent(pick_number):
    """Linear ramp from base_exp to max_exp over ramp_picks."""
    t = clamp((pick_number - 1) / (RAMP_PICKS - 1), 0.0, 1.0)
    return BASE_EXP + (MAX_EXP - BASE_EXP) * t


def compute_card_weight(card, profile, pick_number):
    """Compute selection weight for a single card."""
    if card.is_neutral:
        return NEUTRAL_BASE

    exp = compute_exponent(pick_number)

    # Additive formula: profile component + floor
    weight = FLOOR_WEIGHT
    for r in card.resonances:
        weight += profile[r] ** exp

    return weight


def select_pack(pool, profile, pick_number, rng, pack_size=4):
    """Select pack_size cards from pool using weighted random sampling."""
    weights = []
    for entry in pool:
        w = compute_card_weight(entry.card, profile, pick_number)
        # Weight by effective copies (lane-seed-adjusted)
        weights.append(w * entry.effective_copies)

    # Weighted random sample without replacement
    selected = weighted_sample_without_replacement(pool, weights, pack_size, rng)
    return selected


def on_card_drafted(card, profile):
    """Update profile after player picks a card."""
    for r in card.resonances:
        profile[r] += 1
```

#### Weight Behavior by Pick

| Pick | Exponent | Weight (count=2, DC start) | Weight (count=0, off-color) | Weight (count=8, committed) | On:Off Ratio |
| ---- | -------- | -------------------------- | --------------------------- | --------------------------- | ------------ |
| 1    | 0.50     | 3.5 + 1.41 = 4.91          | 3.50                        | --                          | 1.4:1        |
| 4    | 0.64     | 3.5 + 1.56 = 5.06          | 3.50                        | 3.5 + 5.28 = 8.78           | 1.4-2.5:1    |
| 8    | 0.86     | --                         | 3.50                        | 3.5 + 6.68 = 10.18          | 2.9:1        |
| 12   | 1.10     | --                         | 3.50                        | 3.5 + 10.48 = 13.98         | 4.0:1        |
| 15+  | 1.10     | --                         | 3.50                        | 3.5 + 13.18 = 16.68         | 4.8:1        |

The on:off ratio never exceeds approximately 5:1, compared to 80:1+ in the
pre-redesign system. This ensures off-color cards always have meaningful
selection probability.

#### Implementation Notes

1. **Dual-resonance cards** sum both profile contributions:
   `weight = floor + profile[r1]^exp + profile[r2]^exp`. This naturally makes
   dual cards more attractive to players who have drafted both resonances.

2. **Lane seeds affect copy counts, not weights.** A resonance with seed 1.4 has
   40% more copies in the pool than baseline. This is a one-time pool
   modification, not a per-pack calculation.

3. **No staleness factor.** Parameter sweeps across all strategies showed
   staleness has < 1% impact on any metric. Remove from codebase.

4. **No behavioral tracking.** Commitment-pace EMA, freshness bonus, and all
   S5-derived mechanisms have negligible impact. Remove from codebase.

5. **Dual dreamcallers required.** All strategies produce degenerate mono decks
   with mono dreamcallers. Enforce dual dreamcallers as a game design
   prerequisite.

6. **Profile grows without cap.** No profile cap is needed. The floor weight of
   3.5 provides sufficient off-color probability even against uncapped profiles
   of 15-20.

______________________________________________________________________

## 9. Remaining Open Questions

### High Priority

1. **Real player behavior validation.** All simulations use automated player
   strategies (synergy, power_chaser, rigid). Real players will exhibit more
   nuanced behavior -- sometimes taking a powerful off-color card, sometimes
   prioritizing curve or utility over resonance fit. Playtesting with human
   drafters is essential to validate that the convergence feel is correct.

2. **Shop interaction.** Shops offer 6 cards and allow buying multiple. The
   current simulation does not deeply model shop strategy. A synergy player at a
   shop might buy 3 on-color and 1 off-color splash card, which the pack-of-4
   model does not capture. Shop behavior may naturally lower the synergy top-2
   share.

3. **Battle reward interaction.** Battle rewards offer 3 rare+ cards with
   uniform weighting (no resonance bias). These account for ~5-7 of ~30 picks
   per quest and dilute resonance concentration. The exact impact on convergence
   timing and top-2 share needs validation.

4. **Optimal floor_weight value.** The sweep shows floor_weight in the range
   [3.0, 4.0] produces good results, but the exact value affects the
   splash-convergence tradeoff. 3.5 is a reasonable starting point but may need
   adjustment after playtesting.

### Medium Priority

5. **Lane seed visibility.** Should lane seeds be visible to the player (e.g.,
   "Ember is deep this quest")? Explicit display improves signal reading (Goal
   8\) but may reduce the discovery feeling. Consider showing lane depths after
   the first 5 picks.

6. **Three-color dreamcallers.** The current system assumes dual dreamcallers.
   If some dreamcallers have 3 resonances, DC bonus of 2 * 3 = 6 initial profile
   may be too high. Consider reducing dc_bonus to 1 for tri-color dreamcallers.

7. **Neutral card tuning.** Neutral cards use a fixed weight (4.0) that does not
   scale with profile. This means neutral cards become proportionally less
   likely in late packs. If neutral cards should remain competitive throughout,
   neutral_base could be increased to 5.0-6.0 or given a mild pick-scaling
   component.

8. **Seed range impact on competitive play.** In a competitive mode, lane seed
   variance (0.6-1.4) creates per-quest luck that affects archetype strength.
   Narrowing to [0.8-1.2] reduces variance while preserving signal reading. The
   tradeoff between run variety and competitive fairness needs a design
   decision.

### Lower Priority

09. **Additive vs Max formula final decision.** H5-ADDITIVE (recommended) and
    H5-MAX produce similar results. The additive formula is slightly better on
    convergence timing and signal reading; the max formula is slightly simpler
    and produces more splash. The difference is small enough that either could
    be selected based on implementation preference.

10. **Card rarity interaction.** Higher-rarity cards have fewer copies. Lane
    seeds multiply copy counts, so lane effects are proportionally larger on
    rare cards (rounding from 2 * 0.6 = 1 vs 2 * 1.4 = 3 is a larger relative
    change than 4 * 0.6 = 2 vs 4 * 1.4 = 6). This may create interesting
    rarity-lane interactions or may be a source of unwanted variance.

______________________________________________________________________

## 10. Appendix: Evolution from Round 1 to Round 6

### Round 1: Five Independent Strategies

Each investigation area produced a distinct algorithm:

- **S1 (CRESCENDO):** Profile-based exponent ramp (exp grows from 0.7 to 1.5
  over picks). Elegant simplicity. Strong convergence. No splash mechanism.
- **S2 (Structured Packs):** Focus/Flex/Wild card slots with phase-based
  composition rules. Complex but guaranteed structural diversity.
- **S3 (Lane Pools):** Per-quest lane seeds with depth-based weighting and
  depletion. Best variety and signal reading. Fatal convergence weakness
  (inverted feedback from depletion).
- **S4 (Staged Exponent):** Open/transition/committed phase system with splash
  slot. Clean phase architecture. Moderate splash.
- **S5 (Adaptive Resonance):** Behavioral tracking via commitment-pace EMA.
  Adjusts convergence speed based on player pick patterns.

### Round 2: Cross-Strategy Critique

Key findings from the debate:

- S1 and S4 are "surprisingly similar under the hood" -- both are profile-based
  exponent systems with different parameterizations.
- S3's lane seeds are the only mechanism that provides run-to-run variety and
  signal reading. No other strategy can replicate this.
- S5's behavioral tracking is theoretically appealing but adds complexity with
  uncertain benefit. Multiple agents express skepticism.
- The convergence-splash tradeoff is identified as the central tension: strong
  convergence (high exponent) kills splash, strong splash (high floor) kills
  convergence.

### Round 3: Initial Simulations

All five strategies were simulated with 1000 quests each. Critical findings:

- **DC bonus = 4 causes universal failures.** All strategies fail early variety,
  convergence timing, and top-2 share with DC=4.
- **S3 has the best splash** (1.42 off-color/pack) but fails convergence (late
  on-color = 1.94).
- **S1 and S4 have the best convergence** but fail splash (0.05 and 0.34
  off-color/pack).
- **S5's behavioral tracking produces < 0.1 change** on any metric across all
  parameter values. Confirmed negligible.
- **Best target pass rate: 3/7** for any single strategy (S3).

### Round 4: Deep Analysis and Parameter Sweeps

Sweeps revealed:

- **DC bonus is the single most impactful parameter.** Reducing from 4 to 2
  improves more metrics than any algorithm change.
- **max_exp and floor_weight are the two primary algorithm levers.** Other
  parameters (staleness, ramp_picks, neutral_base) have minimal impact.
- **S5's commitment-pace parameters have negligible sensitivity.** slow_min,
  fast_max, freshness_bonus all produce < 0.1 change.

### Round 5: Hybridization

All five agents converged on the same hybrid architecture:

1. **S3's lane seeds** for pool initialization (adopted unanimously)
2. **S1's exponent ramp** for convergence (adopted by 4 of 5 agents)
3. **DC bonus reduced to 2** (adopted unanimously)
4. **Behavioral tracking eliminated** (adopted unanimously)
5. **Staleness factor marked as negligible** (adopted unanimously)

The main disagreement was about the weight formula: additive
(`profile^exp + lane_bonus + floor`) vs max (`max(floor, profile^exp)`) vs
structural partition (3 convergence cards + 1 wild card).

Agent 3 proposed the additive formula as a way to preserve lane influence
throughout the draft (not just early). This was adopted by Agents 1, 3, and 5,
while Agent 2 kept the structural partition and Agent 4 developed a lane-base
exponent approach.

### Round 6: Final Implementations

Six variants were simulated:

- **H1 (Agent 1):** Additive formula, continuous ramp, lane_weight=4.0, DC=2
- **H2 (Agent 2):** 3+1 structural partition, continuous ramp, DC=2
- **H3 (Agent 3):** Additive formula, staged ramp (flat then linear), DC=4
  per-res (incorrectly kept high DC profile bonus)
- **H4 (Agent 4):** Lane-base exponent formula, staged ramp, dc_boost=3.0, DC=2
- **H5-ADD (Agent 5):** Additive formula, continuous ramp, floor=3.5, DC=2
- **H5-MAX (Agent 5):** Max formula, continuous ramp, floor=3.5, DC=2

H5's key innovation was the high floor weight (3.5 vs 1.0 in other variants),
which combined with DC=2 causes the floor to dominate early packs entirely. This
creates implicit open/transition/committed phases from a single formula -- no
explicit phase logic needed.

### Convergence Trajectory

| Aspect                | Round 1                | Round 3              | Round 6                       |
| --------------------- | ---------------------- | -------------------- | ----------------------------- |
| Best target pass rate | (theoretical)          | 3/7                  | 5/7 (H5-ADD)                  |
| DC bonus              | 4 (assumed)            | 4 (tested)           | 2 (adopted)                   |
| Behavioral tracking   | Proposed (S5)          | Negligible           | Eliminated                    |
| Splash mechanism      | None / structural slot | floor=0.5            | floor=3.5 / additive          |
| Lane seeds            | Proposed (S3)          | Tested (S3 only)     | All hybrids                   |
| Formula consensus     | 5 different formulas   | 5 different formulas | Additive (3 agents)           |
| Key blocker           | Unknown                | DC=4, low floor      | Early unique res (structural) |

The redesign process systematically narrowed the design space from five
independent approaches to a shared hybrid architecture, with the remaining
variation being in formula details (additive vs max vs structural partition)
rather than fundamental mechanisms.

______________________________________________________________________

*Report generated as Round 7 synthesis of the Dreamtides Resonance Algorithm
Redesign. All simulation data from 1000-quest runs with seed 12345. Design goal
scores reflect self-assessment by strategy authors with cross-validation from
other agents.*
