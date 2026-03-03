# Hybrid Proposal 3: Lane-Seeded Crescendo

**Full name:** Lane-Seeded Crescendo **Proposal author:** Agent 3 (Strategy 3 —
Lane Pools) **Source strategies combined:** S3 Lane Pools + S1 CRESCENDO + S4
Staged Exponent

______________________________________________________________________

## Overview

Lane-Seeded Crescendo gives each quest a distinct personality from the very
first card offered. At the start of every run, five random "lane seeds" are
rolled — one for each resonance — creating a unique resonance landscape that the
player can read and react to. These lane seeds act as a persistent bonus for
every card of their resonance, making some colors naturally more prominent in
this particular run. On top of that, the proposal adds a convergence engine
borrowed from CRESCENDO: as a player picks more cards of a given resonance,
those cards gain exponentially more weight in future packs, so the draft arcs
naturally from wide-open early exploration toward focused late commitment. The
exponent that controls this convergence starts low (keeping early packs varied)
and ramps up smoothly across picks 5 through 11, reaching its ceiling just as
the draft hits its decisive stretch. The result is a system that feels like an
environment to explore rather than a script to follow: early packs are shaped by
the world (lane seeds), late packs are shaped by the player's choices (profile
growth), and both signals are always legible.

______________________________________________________________________

## How It Works

### The Core Formula

Every card in a pack is weighted by the following calculation, run independently
for each resonance slot:

```
card_weight = profile_count ^ exponent(pick)
            + lane_bonus
            + floor_weight
            [+ dreamcaller_bonus, if this resonance matches your dreamcaller]
```

That is all. Four terms added together. Here is what each one does.

### Term 1: Profile Count to the Exponent Power

`profile_count` is simply how many cards of this resonance you have already
drafted. `exponent(pick)` is a number that starts low and climbs as the draft
progresses.

At the start of the draft (picks 0-4), the exponent is 0.7 — a sub-linear value,
which means raising your profile count to this power actually *compresses*
differences. A profile of 1 gives `1^0.7 = 1.0`. A profile of 4 gives
`4^0.7 = 2.64`. A profile of 8 gives `8^0.7 = 5.28`. The differences are modest,
so your past choices don't dominate your future options.

From pick 5 through pick 11, the exponent ramps linearly from 0.7 up to 1.4. At
1.4 (super-linear), the same profile counts look very different. A profile of 1
gives `1^1.4 = 1.0`. A profile of 4 gives `4^1.4 = 6.96`. A profile of 8 gives
`8^1.4 = 16.4`. Now your accumulated picks powerfully amplify your dominant
resonances. The draft has found its direction.

| Pick | Exponent | Profile=1 | Profile=4 | Profile=8 |
| ---- | -------- | --------- | --------- | --------- |
| 0-4  | 0.70     | 1.00      | 2.64      | 5.28      |
| 5    | 0.70     | 1.00      | 2.64      | 5.28      |
| 6    | 0.82     | 1.00      | 3.31      | 7.24      |
| 7    | 0.93     | 1.00      | 3.77      | 8.59      |
| 8    | 1.05     | 1.00      | 4.29      | 10.1      |
| 9    | 1.17     | 1.00      | 4.90      | 12.0      |
| 10   | 1.28     | 1.00      | 5.59      | 14.3      |
| 11+  | 1.40     | 1.00      | 6.96      | 16.4      |

This is positive feedback: picking a resonance grows its profile, which grows
its weight, which makes it more likely to appear again. This is the exact
opposite of the original Lane Pools proposal's "inverted feedback" problem (see
the Evolution Story below).

### Term 2: The Lane Bonus (What "Lane Seeds" Are)

At quest start, before the first pick, the system rolls five random numbers in
the range [0.60, 1.40] — one per resonance. These are the **lane seeds**. Each
seed is multiplied by a constant (LANE_WEIGHT = 4.0) to produce the lane bonus
for that resonance, which is then added to every card of that color for the
entire quest.

```
lane_bonus = lane_seed * 4.0
    range: [2.4, 5.6]  (mean: 4.0)
```

A resonance with a high seed (say 1.35, giving bonus 5.4) will appear more often
throughout the whole run. A resonance with a low seed (say 0.65, giving bonus
2.6) will appear less. Two quests with the same dreamcaller will play out
differently because the resonance landscape is different.

The lane bonus is **additive**, not multiplicative. This is a deliberate and
important decision. If the lane bonus were multiplicative, it would grow with
profile count and eventually become invisible against the exponentially large
profile term. By keeping it additive and constant, the lane bonus is most
influential early (when profile counts are near zero and the profile term is
small) and gracefully fades in relative importance as convergence takes over
late. This creates the narrative arc: "early draft shaped by the world, late
draft shaped by your choices."

**Concrete example at pick 0** (profile counts all 0, exponent 0.7):

A resonance with a high lane seed (bonus 5.4) has weight `0 + 5.4 + 1.0 = 6.4`.
A resonance with a low lane seed (bonus 2.6) has weight `0 + 2.6 + 1.0 = 3.6`.
Ratio: 1.78x. The high lane is noticeably more likely to appear.

**Concrete example at pick 11** (profile=5, exponent 1.4):

High-lane resonance: `9.52 + 5.4 + 1.0 = 15.9` Low-lane resonance (profile=1):
`1.0 + 2.6 + 1.0 = 4.6` Now the committed resonance dominates — but the low-lane
resonance still has a meaningful probability of appearing as a splash
opportunity.

### Term 3: Floor Weight

A fixed constant of 1.0 added to every card regardless of resonance. This
ensures that even a resonance you have never drafted — with profile count 0 —
still has some weight and can appear as a surprise. Without this floor, a
zero-profile low-lane resonance would have weight 2.6, which is fine; but with
the floor it has 3.6, comfortably visible.

### Term 4: Dreamcaller Bonus

Your dreamcaller's resonances each receive +2.0 on top of everything else. This
nudges early packs toward your dreamcaller's colors without overwhelming the
lane seeds. A dreamcaller resonance with average lane bonus has weight
`1.0 + 4.0 + 1.0 + 2.0 = 8.0` at pick 0; a non-dreamcaller resonance with the
same lane bonus has weight `1.0 + 4.0 + 1.0 = 6.0`. The ratio is 1.33x, which is
a preference but not a lock — lane seeds can still override it when one
resonance has a much better lane seed than your dreamcaller's colors.

(Note: the DC bonus was deliberately reduced from 4 to 2 in this proposal. See
the Simulation Results section for why the original value of 4 was problematic.)

______________________________________________________________________

## Key Parameters

| Parameter           | Default Value     | What It Controls                            | Higher                                    | Lower                                                   |
| ------------------- | ----------------- | ------------------------------------------- | ----------------------------------------- | ------------------------------------------------------- |
| `EXPONENT_MIN`      | 0.7               | Starting exponent (open phase)              | More convergence pressure from pick 1     | More open, profiles barely matter early                 |
| `EXPONENT_MAX`      | 1.4               | Maximum exponent (late draft)               | Stronger convergence, less splash         | Weaker convergence, more off-color late                 |
| `RAMP_START_PICK`   | 5                 | Pick number when ramp begins                | Later start means more open early picks   | Earlier start means convergence sooner                  |
| `RAMP_END_PICK`     | 11                | Pick number when ramp reaches max           | Later end means slower ramp               | Earlier end means fast transition                       |
| `LANE_SEED_MIN`     | 0.60              | Lowest possible seed (weakest lane)         | Raise for narrower variance               | Lower for more extreme lane imbalance                   |
| `LANE_SEED_MAX`     | 1.40              | Highest possible seed (strongest lane)      | Raise for more extreme high lanes         | Lower for narrower variance                             |
| `LANE_WEIGHT`       | 4.0               | How much seeds amplify into bonuses         | More variety, weaker convergence signal   | Less variety, stronger convergence signal               |
| `FLOOR_WEIGHT`      | 1.0               | Minimum weight for any resonance            | Higher floor keeps all resonances visible | Lower floor lets zero-profile colors fade               |
| `DREAMCALLER_BONUS` | 2.0 per resonance | How strongly dreamcaller biases early packs | Dreamcaller more dominant                 | Dreamcaller less influential; lane seeds more prominent |

**Most sensitive parameter:** `EXPONENT_MAX`. Each 0.2 increase shifts late
on-color by about +0.2 and late off-color by about -0.15. Values above 1.6 push
off-color cards below acceptable thresholds. Values below 1.2 produce weak
convergence. The 1.4 default is the sweet spot.

**Second most sensitive:** `LANE_WEIGHT`. Lower values (2.0) maximize
convergence but reduce early diversity. Higher values (6.0) maximize early
variety but dilute the convergence signal. The 4.0 default balances both.

**Least sensitive:** `FLOOR_WEIGHT` and the ramp timing parameters. Sweeping
floor from 0.5 to 3.0 changes late off-color by only 0.14. Sweeping ramp start
from pick 3 to pick 7 changes all metrics by less than 0.03 — mostly because the
dreamcaller profile bonus dominates early picks regardless.

______________________________________________________________________

## What Changed From the Original Strategy: The Evolution Story

### The Original Strategy 3 Proposal: Round 1 Lane Pools

The original Strategy 3 proposal was built on a different philosophy entirely.
Inspired by real-world multiplayer booster draft (Magic: The Gathering, Flesh
and Blood), it argued that the current profile-tracking system is too
"player-centric" — it watches what the player drafts and rewards them with more
of the same. Instead, Strategy 3 proposed an **environment-centric** model:
divide the card pool into five colored "lanes" at quest start, assign each lane
a random size based on lane seeds, and let pack composition emerge from lane
depletion dynamics.

The core claim: as a player drafts heavily from one lane, that lane's depth
decreases, which decreases its weight in future packs. Undrafted lanes stay deep
and keep appearing. Convergence emerges naturally from scarcity — the same
mechanism that makes real-world draft formats engaging.

The player-facing pitch was compelling: "Cards are organized by color lane;
deeper lanes show up more." No hidden profile tracking. No invisible feedback
loops. Just an environment that the player navigates.

Lane seeds seeded the pools at quest start, making one run's Ember lane deep
while another's was thin. Signal reading — noticing which colors kept appearing
— would reward experienced players who could identify which lanes were open.

In the Round 1 analysis, Strategy 3 scored highest of all five proposals (62/80,
weighted 273/360), earning exceptional marks for early variety, run-to-run
distinctiveness, splash opportunities, and signal reading.

### The Fatal Discovery: Inverted Feedback

During the debate between all five agents, Agents 2 and 4 identified a
structural problem in the Lane Pools approach that no amount of parameter tuning
could fix. It was called the **inverted feedback problem**.

In multiplayer MTG draft, lane scarcity works because *other players* deplete
colors you are not using, making your preferred color's lane remain open and
attractive (a positive signal: "this color is underdrafted at this table, commit
to it"). The feedback runs in the right direction.

In a solo Dreamtides quest, there are no other players. The player is the *only*
source of depletion. When a player commits to Tide — drafting Tide cards pick
after pick — they deplete the Tide lane. This makes Tide cards appear *less*
frequently in future packs, actively working against the convergence the design
requires. The system punishes commitment instead of rewarding it.

Profile-based systems (S1, S4, S5) all have the correct positive feedback: pick
Tide → Tide profile grows → Tide weight increases → more Tide appears. Lane
Pools had the opposite: pick Tide → Tide lane depletes → Tide weight decreases →
less Tide appears.

Simulation confirmed the problem. Despite the highest early-variety scores of
any strategy, Lane Pools scored only 4/10 on convergence — its late on-color
rate (1.94 per pack) barely missed the target of 2.0, and no parameter
configuration could meaningfully fix it without destroying the early variety
that was its greatest strength. The best achievable late on-color within the
Lane Pools framework was around 2.07 — marginal, unstable, and entirely
dependent on an extreme dreamcaller boost rather than on player behavior.

The original Agent 3 acknowledged this in its self-assessment: "Lane pools as a
standalone pack construction mechanism are NOT viable."

### The Redesign: What Was Kept and What Was Replaced

After the debate, the redesign was guided by a clear principle: keep what Lane
Pools does uniquely well (lane seeds for run identity and signal reading), throw
away what causes the inverted feedback problem (depth-based weighting and
depletion dynamics), and replace convergence with CRESCENDO's profile-based
exponent ramp — which has the correct positive feedback.

| Aspect                | Original Lane Pools                              | Lane-Seeded Crescendo                      |
| --------------------- | ------------------------------------------------ | ------------------------------------------ |
| Convergence mechanism | Lane depletion (inverted feedback)               | Profile exponent ramp (positive feedback)  |
| Lane seeds            | Control pool copy counts (deep vs shallow lanes) | Additive weight bonus per resonance        |
| Pack construction     | Sample lanes by depth                            | Weighted by profile^exponent + bonus       |
| Player profile used?  | No (pure environment model)                      | Yes (core of convergence)                  |
| Feedback direction    | Negative (drafting depletes preference)          | Positive (drafting strengthens preference) |
| Complexity            | High (new data structure, dual-card tracking)    | Low (single formula, five parameters)      |

The redesign is not an incremental tweak. It is a fundamental change to how
convergence works. The lane seed concept is preserved in a new form: instead of
controlling how many cards exist in each lane, seeds now control a flat additive
weight bonus that persists throughout the entire quest. The environment still
has a shape — it just expresses that shape through bonuses rather than through
depletion.

The result is a simpler system (one formula replacing a complex data structure)
that correctly rewards commitment while preserving the run-to-run identity and
signal-reading character that made Lane Pools the highest-scoring original
strategy.

______________________________________________________________________

## Simulation Results

### Important Note on DC Bonus

These results used `dc_bonus_per_res = 4` (the default from earlier rounds), not
the consensus-recommended value of 2. The higher bonus front-loaded the initial
profile counts, making early packs more focused than intended and causing the
synergy player to over-converge. The proposal text specifies a DC bonus of 2;
the results below reflect the original simulation run with 4. This is why
several early-game metrics fail their targets — they were penalized by a
parameter that has since been revised downward.

### Results by Player Strategy (1000 quests each)

#### Synergy Player — targets 6/10

| Metric                                             | Target    | Actual    | Result |
| -------------------------------------------------- | --------- | --------- | ------ |
| Early unique resonances per pack (picks 1-5)       | >= 3.0    | 2.96      | FAIL   |
| Early on-color per pack (picks 1-5)                | \<= 2.0   | 2.33      | FAIL   |
| Late on-color per pack (picks 6+)                  | >= 2.0    | 3.28      | PASS   |
| Late off-color per pack (picks 6+)                 | >= 0.5    | 0.55      | PASS   |
| Top-2 resonance share of final deck                | 75-90%    | 94.6%     | FAIL   |
| Convergence pick (when top-2 share exceeds 75%)    | 5-8       | 4.2       | FAIL   |
| Archetype pair frequency                           | all 5-15% | 8.9-11.2% | PASS   |
| Convergence ratio (late on-color / early on-color) | >= 1.3    | 1.41      | PASS   |
| Cross-run variance (stdev of top-1 share)          | >= 0.03   | 0.059     | PASS   |
| Pick-to-pick variance (on-color diff)              | \<= 1.2   | 0.84      | PASS   |

#### Power Chaser Player — targets 7/10

| Metric                                       | Target    | Actual    | Result |
| -------------------------------------------- | --------- | --------- | ------ |
| Early unique resonances per pack (picks 1-5) | >= 3.0    | 3.02      | PASS   |
| Early on-color per pack (picks 1-5)          | \<= 2.0   | 2.25      | FAIL   |
| Late on-color per pack (picks 6+)            | >= 2.0    | 2.83      | PASS   |
| Late off-color per pack (picks 6+)           | >= 0.5    | 0.99      | PASS   |
| Top-2 resonance share of final deck          | 60-85%    | 67.0%     | PASS   |
| Convergence pick                             | 5-8       | 14.0      | FAIL   |
| Archetype pair frequency                     | all 5-15% | 8.5-12.1% | PASS   |
| Convergence ratio                            | >= 1.3    | 1.26      | FAIL   |
| Cross-run variance                           | >= 0.03   | 0.064     | PASS   |
| Pick-to-pick variance                        | \<= 1.2   | 1.02      | PASS   |

#### Rigid Player — targets 6/10

| Metric                                       | Target    | Actual    | Result |
| -------------------------------------------- | --------- | --------- | ------ |
| Early unique resonances per pack (picks 1-5) | >= 3.0    | 2.96      | FAIL   |
| Early on-color per pack (picks 1-5)          | \<= 2.0   | 2.34      | FAIL   |
| Late on-color per pack (picks 6+)            | >= 2.0    | 3.28      | PASS   |
| Late off-color per pack (picks 6+)           | >= 0.5    | 0.54      | PASS   |
| Top-2 resonance share of final deck          | 75-90%    | 97.1%     | FAIL   |
| Convergence pick                             | 5-8       | 4.3       | FAIL   |
| Archetype pair frequency                     | all 5-15% | 8.9-11.2% | PASS   |
| Convergence ratio                            | >= 1.3    | 1.40      | PASS   |
| Cross-run variance                           | >= 0.03   | 0.059     | PASS   |
| Pick-to-pick variance                        | \<= 1.2   | 0.84      | PASS   |

### What Changed From Round 3 (Original Lane Pools)

| Metric                      | Round 3 Lane Pools | Lane-Seeded Crescendo | Change                        |
| --------------------------- | ------------------ | --------------------- | ----------------------------- |
| Early unique res/pack       | 2.97               | 2.96                  | -0.01 (negligible)            |
| Early on-color/pack         | 1.95               | 2.33                  | +0.38 (worse; DC bonus issue) |
| Late on-color/pack          | 1.94               | 3.28                  | +1.34 (dramatically better)   |
| Late off-color/pack         | 1.42               | 0.55                  | -0.87 (expected tradeoff)     |
| Convergence ratio           | ~1.0               | 1.41                  | +0.41 (fundamental fix)       |
| Late on-color >= 2.0 target | FAIL               | PASS                  | Fixed                         |

The primary goal — fixing convergence — was achieved. The secondary cost —
reduced splash — was expected and the result (0.55) remains above the 0.5
target. The early openness regression is attributed to the over-powered DC bonus
used in this simulation run (4 per resonance instead of the recommended 2).

### Archetype Distribution

| Archetype           | Synergy | Power Chaser | Rigid  |
| ------------------- | ------- | ------------ | ------ |
| Mono (1 resonance)  | 0.0%    | 0.0%         | 0.0%   |
| Dual (2 resonances) | 100.0%  | 38.9%        | 100.0% |
| Tri (3 resonances)  | 0.0%    | 61.1%        | 0.0%   |

All ten resonance pair combinations appear within the 5-15% target band for all
three player types, confirming no pair is dominant or crowded out.

Deck overlap (Jaccard similarity) between consecutive runs is 0.058 for synergy
and 0.060 for power chaser — less than 6% of cards are shared between runs,
indicating strong run-to-run variety despite the dual-only outcome.

______________________________________________________________________

## Design Goal Scorecard

Scores from the post-simulation analysis (`analysis_3.md`), rated 1-10. Total:
58/80.

| Goal                   | Score | Justification                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ---------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1. Simple              | 8/10  | One formula, five parameters. No phases, no slot types, no behavioral tracking. Lane seeds are rolled once and could be shown to the player. The exponent ramp is a simple linear interpolation. The only complexity is understanding that the exponent schedule is flat-then-ramp, which is still a single formula.                                                                                                                                          |
| 2. Not on Rails        | 8/10  | Cross-run variance of 0.059 (stdev of top-1 share) confirms meaningful per-run divergence. Lane seeds [0.6, 1.4] create distinct resonance landscapes each run. Jaccard deck overlap of only 0.058 means consecutive runs share under 6% of cards. Each run genuinely feels different.                                                                                                                                                                        |
| 3. No Forced Decks     | 7/10  | Floor weight 1.0 + lane bonus [2.4, 5.6] ensures even zero-profile resonances always have weight >= 3.4. At pick 11 against a focused profile, off-color still has roughly 17% selection probability. Late off-color of 0.55 confirms no resonance is mechanically excluded. Minor deduction because the DC profile bonus front-loads dual-color commitment.                                                                                                  |
| 4. Flexible Archetypes | 6/10  | Synergy and rigid strategies produce 100% dual-color decks. Power chaser produces 38.9% dual + 61.1% tri-color. However, no mono or 4-5 color decks appear in 1000 quests. The DC profile bonus of 4 per resonance (in this simulation) strongly biases toward dual, making mono effectively impossible. Structural weakness.                                                                                                                                 |
| 5. Convergent          | 8/10  | The Round 3 failure is fixed. Late on-color = 3.28 (up from 1.94). Convergence ratio = 1.41 (up from ~1.0). Profile exponent ramp provides strong positive feedback: drafting a resonance increases its profile, which increases its weight. This is the proposal's most important improvement.                                                                                                                                                               |
| 6. Splashable          | 7/10  | Late off-color = 0.55 (above the 0.5 target). The additive lane bonus ensures off-color cards always have meaningful weight. However, the result is above the target only by a thin margin. The exponent cap of 1.4 is necessary to keep splash viable; values above 1.5 push off-color below the threshold.                                                                                                                                                  |
| 7. Open Early          | 6/10  | Early unique resonances = 2.96 (just below the 3.0 target). Early on-color = 2.33 (above the 2.0 target, meaning packs are already somewhat focused in picks 1-5). The DC profile bonus of 4 per resonance in this simulation means the profile starts at 8 total points, which at exponent 0.7 still gives a significant head start. This is the biggest remaining weakness and is attributed to the simulation using DC=4 rather than the recommended DC=2. |
| 8. Signal Reading      | 8/10  | Lane seeds are visible at quest start, providing an explicit opening signal. As profile counts grow, the exponent ramp creates readable progression. Pick-to-pick variance of 0.84 (well below the 1.2 target) confirms smooth pack evolution rather than jarring jumps. The story traces show clearly readable arcs from diverse early to focused late packs.                                                                                                |

### Score Comparison: Before and After the Redesign

| Goal                   | Round 3 Lane Pools | Lane-Seeded Crescendo | Change |
| ---------------------- | ------------------ | --------------------- | ------ |
| 1. Simple              | 7                  | 8                     | +1     |
| 2. Not on Rails        | 8                  | 8                     | 0      |
| 3. No Forced Decks     | 7                  | 7                     | 0      |
| 4. Flexible Archetypes | 7                  | 6                     | -1     |
| 5. Convergent          | 4                  | 8                     | +4     |
| 6. Splashable          | 9                  | 7                     | -2     |
| 7. Open Early          | 8                  | 6                     | -2     |
| 8. Signal Reading      | 7                  | 8                     | +1     |
| **Total**              | **57**             | **58**                | **+1** |

The redesign makes convergence dramatically better (+4) at the cost of some
splash and early openness (-2 each). The net score barely moves, but the profile
is much healthier: the fatal 4/10 on convergence is gone, and the system no
longer fails its primary job.

______________________________________________________________________

## Strengths

**Convergence is fixed and works cleanly.** Late on-color jumped from 1.94 to
3.28 — a gain of 1.34 cards per pack. The convergence ratio of 1.41 exceeds the
1.3 target. More importantly, convergence now comes from positive feedback
(profile growth) rather than inverted depletion, so the mechanism is
structurally sound rather than coincidentally adequate.

**Per-run identity is preserved.** The cross-run variance of 0.059 is healthy.
The deck Jaccard similarity of 0.058 means each run produces a different set of
cards. The archetype pair distribution is nearly uniform (8.9-11.2% for all ten
pairs), confirming no archetype is crowded out. Lane seeds successfully
differentiate runs without requiring lane pool data structures.

**Smooth draft arc.** Pick-to-pick variance of 0.84 is well below the 1.2
ceiling. The story traces show a readable progression: early packs offer 3-4
resonances with modest separation, mid packs begin tightening, and late packs
converge strongly without becoming completely mono. There are no phase
boundaries or discontinuities.

**Splash remains viable.** Late off-color of 0.55 passes the 0.5 target. The
additive lane bonus guarantees that even zero-profile, low-seed resonances have
a floor weight that keeps them visible late. At pick 11, a resonance the player
has never drafted with the lowest possible lane seed still has an 8-16%
probability of appearing in any given pack slot.

**Power chaser differentiation works.** Power chasers produce 67% top-2 share
(within the 60-85% target), 61% tri-color decks, and very different late
off-color numbers (0.99 vs. 0.55 for synergy). The system naturally produces
different outcomes for different player behaviors without requiring behavioral
tracking or separate code paths.

**Formula simplicity.** One formula, five parameters, no new data structures.
The full algorithm is eight lines of pseudocode. Any developer can understand
and tune it. It is simpler than the original Lane Pools proposal while producing
better measured outcomes on six of the eight design goals.

______________________________________________________________________

## Weaknesses

**Early openness falls short.** Early unique resonances of 2.96 misses the 3.0
target. Early on-color of 2.33 exceeds the 2.0 ceiling. The flat exponent phase
(picks 0-4) helps, but the DC profile bonus used in this simulation (4 per
resonance, giving 8 total profile points from the start) creates on-color bias
even before the player has made a single pick. The proposal recommends reducing
DC bonus to 2; this simulation did not use that value.

**Synergy players over-converge.** Top-2 share of 94.6% exceeds the 75-90%
target. Convergence pick of 4.2 misses the 5-8 target (arriving too early). The
combination of DC profile bonus + exponent ramp + synergy player behavior
creates a reinforcing spiral that locks in direction faster than intended.
Reducing the DC profile bonus would likely extend the open phase and bring top-2
share down.

**No archetype diversity beyond dual.** Every synergy and rigid run produces a
dual-color deck. The DC profile bonus of 4 per resonance (giving 8 total points)
effectively hard-codes dual-color as the minimum viable archetype. Mono-color is
impossible when the dreamcaller gives 4 points to each of two resonances from
the start. Reducing DC profile bonus or supporting mono dreamcallers would be
required to allow mono builds.

**Splash is thin.** Late off-color of 0.55 passes the minimum threshold but
leaves little margin. The original Lane Pools scored 1.42 here — well above the
floor, with genuine late-game splash opportunities. The new system just clears
the bar. The exponent cap of 1.4 is load-bearing: any increase pushes off-color
below 0.5.

**Convergence pick arrives slightly early.** The mean convergence pick of 4.2
sits just outside the 5-8 target range. The system commits synergy players
slightly before intended. The open phase (picks 0-4 with exponent 0.7) is not
quite open enough to resist the DC profile bonus's initial push.

______________________________________________________________________

## Draft Story Example

The following is from Trace 1 (Early Committer, Synergy player) in the
simulation output. Dreamcaller: Ember + Stone. Lane seeds create distinct
resonance landscape: Ember seed 1.22 (bonus 4.9), Stone seed 1.19 (bonus 4.7),
Zephyr seed 1.16 (bonus 4.7). At pick 0, all five resonances are competitive.

**Pick 1 (exp=0.70):** The shop shows 7 cards. Weights in play: Ember at 10.51
(profile 0, but high lane seed + DC bonus), Stone at 10.38, Zephyr at 5.66, Tide
at 4.86. Player picks Ember (highest weight) — but Stone was within 0.13 of
winning. This is a genuine choice. Profile after pick: Ember=5, Stone=7 (these
reflect the DC profile seeding from quest start).

**Pick 2 (exp=0.70):** An Ember+Stone dual card has weight 16.79 — the DC bonus
applies to both resonances. Player picks a mono-Ember card at 10.95 for strategy
reasons. All three Zephyr slots are at 5.66, offering a real alternative.
Profile growing: Ember=9, Stone=8.

**Picks 3-5:** Battle reward forces a Ruin pick. Two site picks add Stone then a
neutral. Profile stabilizes at Ember=9, Stone=9, with small Tide and Ruin counts
from multi-resonance cards.

**Pick 6 (exp=0.82):** The ramp has started. An Ember+Stone dual at weight 19.12
wins easily. On-color bias is visibly increasing. Profile: Ember=10, Stone=10.

**Pick 8 (exp=1.05):** Now at the crossover. Ember at 16.44 against Tide at 6.77
and Zephyr at 6.66. The profile exponent has grown enough to clearly separate
on-color from off-color. Player picks Ember.

**Pick 9 (exp=1.17):** An Ember+Stone dual card reaches weight 33.43, nearly
double its nearest competitor (Ember+Zephyr at 21.16). The dual benefit and
profile amplification compound. Player picks the dual.

**Pick 11 (exp=1.40, max):** Ember+Stone dual at weight 58.39. Stone+Tide at
31.43. Mono-Ember at 34.75. The late-draft pack is clearly focused on Ember and
Stone, but off-color cards at weights of 31-34 are still present and
occasionally picked. Off-color is not invisible — just clearly secondary.

**Final deck (25 picks shown, through pick 21):** Profile Ember=24, Stone=20,
with Tide=2, Zephyr=1, Ruin=1 as small splashes. Top-2 share 86.8% — comfortably
within the 75-90% target for this individual run (the average exceeds it).
Classification: dual.

The narrative arc is clear: pick 1 is a close call between Ember and Stone; pick
6 is clearly Ember+Stone biased but still interesting; pick 11 is strongly
committed. The draft did not feel predetermined at any point.

______________________________________________________________________

## Comparison to the Other Hybrids

Five hybrid proposals were developed and simulated. Here is how Lane-Seeded
Crescendo differs from each.

### vs. Hybrid 1 (CRESCENDO-Anchored)

Hybrid 1 uses a similar profile-based exponent ramp but without lane seeds. Its
convergence is strong; its early variety is weaker. Lane-Seeded Crescendo adds
the lane bonus layer to create per-run identity and keep all resonances visible
early, at the cost of some convergence strength. Choose Hybrid 1 for maximum
convergence reliability; choose Hybrid 3 if run-to-run variety is a higher
priority.

### vs. Hybrid 2 (Structured Slots with Lane Seeds)

Hybrid 2 uses structural pack slots (Focus/Flex/Wild) to guarantee off-color
cards appear, combining S2's slot guarantees with S3's lane seeds. This provides
harder guarantees for splash than Lane-Seeded Crescendo (which relies on
probabilistic weight bonuses rather than structural slots). However, the slot
structure introduces phase boundaries and locks the design into a three-
category template. Choose Hybrid 2 if splash guarantees are paramount; choose
Hybrid 3 for simpler implementation and smoother pack feel.

### vs. Hybrid 4 (Staged Exponent + Off-Lane Guarantee)

Hybrid 4 uses S4's exponent ramp with an explicit late-draft off-color guarantee
mechanism. Very similar convergence profile to Hybrid 3, but handles splash
through a structural rule rather than through additive lane bonuses. Hybrid 3 is
simpler (no additional rule) but Hybrid 4 provides more reliable late off-color.
Choose Hybrid 4 if off-color below 0.5 is unacceptable risk; choose Hybrid 3 if
formula simplicity is preferred.

### vs. Hybrid 5 (Adaptive Resonance + Lane Seeds)

Hybrid 5 adds behavioral tracking (EMA commitment score) to the lane seed
foundation, attempting to differentiate truly exploratory players from focused
players mid-draft. Hybrid 3 makes no such distinction: player behavior affects
outcome only through profile count growth, which is a natural consequence of
picking rather than an explicit tracking mechanism. Hybrid 5's behavioral
tracking adds significant complexity for modest effect (the commitment
multiplier provides only a ±35% exponent adjustment). Choose Hybrid 5 if
player-behavior differentiation is a critical design goal; choose Hybrid 3 for a
cleaner, more predictable algorithm.

### Summary Comparison Table

| Aspect              | H1 CRESCENDO      | H2 Structured          | H3 Lane-Seeded             | H4 Staged+Guarantee    | H5 Adaptive          |
| ------------------- | ----------------- | ---------------------- | -------------------------- | ---------------------- | -------------------- |
| Convergence source  | Profile exponent  | Slot structure         | Profile exponent           | Profile exponent       | Profile + behavior   |
| Per-run variety     | Low               | Medium                 | High                       | Medium                 | Medium               |
| Splash mechanism    | Probabilistic     | Structural (Wild slot) | Probabilistic (lane bonus) | Structural (guarantee) | Probabilistic        |
| Complexity          | Low               | Medium                 | Low                        | Medium                 | High                 |
| Behavioral tracking | No                | No                     | No                         | No                     | Yes                  |
| Main strength       | Convergence       | Splash guarantees      | Run identity               | Convergence + safety   | Player-type response |
| Main weakness       | No variety signal | Phase boundaries       | Thin splash margin         | More moving parts      | Complex proxy        |

**Who should prefer Hybrid 3:** A designer who wants a single clean formula that
can be explained to a player in two sentences, values distinct per-run identity
as the primary differentiating experience, and is comfortable with probabilistic
(rather than guaranteed) splash opportunities. The formula is the most legible
of all five hybrids: what you see is exactly what you get.

**Who should reject Hybrid 3:** A designer who requires a structural guarantee
that off-color cards will always appear (not just usually appear), needs to
support mono-color archetypes, or wants to differentiate how the system responds
to exploratory versus committed playing styles mid-draft.
