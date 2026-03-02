# Proposal 4: Lane-Seeded Staged Exponent

**Status:** Simulation complete (v2). 13/21 targets passed across 3 player strategies.

---

## Overview

The Lane-Seeded Staged Exponent hybrid is a draft algorithm that combines two
ideas from separate earlier proposals: per-quest "lane seeds" that make each
run's card pool feel different (borrowed from Strategy 3), and a time-varying
exponent that starts soft and grows as your deck takes shape (borrowed from
Strategy 4). Together, these break a fundamental tradeoff that all simpler
approaches face: in every other proposal, making late-game packs strongly
on-color requires also making off-color cards nearly invisible, but in this
hybrid, the lane seed term keeps off-color cards meaningfully present no matter
how committed your deck becomes. The result is a draft that feels genuinely open
in the first five picks — with a 1.8x on-color advantage rather than the
current 8x — transitions smoothly into commitment, and still offers roughly one
off-color card per pack in the late game, compared to near-zero in previous
designs. Each quest begins by rolling a unique seed for each of the five
resonance lanes, which means two runs with the same dreamcaller will feel
observably different: one might have a deep Ruin lane you can read and pivot
into, while another might funnel you toward Stone despite your original plan.

---

## How It Works

### The Core Formula

For every card the algorithm considers offering you, it computes a weight:

```
card_weight = floor + sum(lane_score(r)^exponent for each resonance r on the card)
```

The higher the weight, the more likely the card appears in your pack. Two
numbers make this interesting: `lane_score` (which tracks how much of a
resonance the world has available and how committed your deck is to it), and
`exponent` (which controls how aggressively the weight differences between colors
are amplified).

### Lane Score: What the World Knows About a Color

```
lane_score(r) = lane_base * lane_seed[r]
              + dc_boost  (if r matches your dreamcaller)
              + picks_in_r * pick_scale
```

Breaking each piece down:

- **`lane_base * lane_seed[r]`** — The "structural" weight of this color in the
  world. At the start of every quest, each resonance gets a random seed between
  0.60 and 1.40. With `lane_base = 4.0`, a deep lane (seed 1.40) starts at 5.6,
  and a shallow lane (seed 0.60) starts at 2.4. This is what makes two Ruin
  quests feel different: sometimes Ruin is a deep lane (1.30) and sometimes it's
  shallow (0.75). Crucially, even a shallow off-color lane has a nonzero weight —
  it never collapses to zero the way a pure profile-based floor does at high
  exponents.

- **`dc_boost`** (3.0 if your dreamcaller has this resonance) — A fixed bonus to
  the lane score for each of your dreamcaller's colors. This is separate from the
  profile seed below. It persists throughout the run.

- **`picks_in_r * pick_scale`** — Every time you pick a card with resonance r,
  that color's lane score grows. With `pick_scale = 0.5`, picking 10 Tide cards
  adds 5.0 to the Tide lane score. This is the convergence engine: the more you
  draft a color, the more that color appears.

**Concrete example at pick 12 (exponent 1.5), Tide+Ruin dreamcaller, having
drafted 8 on-color picks:**

| Lane | Lane Score Calculation | Lane Score |
|------|----------------------|------------|
| Tide (on-color) | 4.0 * 1.20 + 3.0 + 8 * 0.5 | 11.8 |
| Ruin (on-color) | 4.0 * 1.30 + 3.0 + 7 * 0.5 | 12.7 |
| Ember (off-color) | 4.0 * 0.80 + 0 + 0 * 0.5 | 3.2 |
| Stone (off-color) | 4.0 * 0.70 + 0 + 0 * 0.5 | 2.8 |
| Zephyr (off-color) | 4.0 * 1.00 + 0 + 0 * 0.5 | 4.0 |

At exponent 1.5: Tide weight = 11.8^1.5 = 40.6, Ember weight = 3.2^1.5 = 5.7.
In a 4-card pack with 2 on-color lanes and 3 off-color lanes:

- On-color share: 2 * 40.6 = 81.2
- Off-color share: (5.7 + 4.6 + 8.0) = 18.3
- Total: 99.5
- Expected on-color per pack: 4 * (81.2 / 99.5) = **3.27**
- Expected off-color per pack: 4 * (18.3 / 99.5) = **0.74**

Both the "convergent" target (>= 2.0 on-color) and the "splashable" target
(>= 0.5 off-color) are met simultaneously. This is the key result that simpler
approaches cannot achieve.

### The Exponent: How Time Changes the Weight Amplification

```
Pick 1-5  (open phase):     exponent = 0.7  [flat]
Pick 6-14 (ramp phase):     exponent rises linearly from 0.7 to 1.5
Pick 15+  (committed):      exponent = 1.5  [capped]
```

The exponent controls how much the algorithm amplifies differences between lane
scores. At exponent 0.7 (sub-linear), a lane with twice the score as another
lane gets only 1.6x the weight — barely a nudge. At exponent 1.5 (super-linear),
that same 2x score difference becomes a 2.8x weight difference, and at extreme
ratios the effect is much larger.

**At pick 1, exponent = 0.7:** Tide lane score of 7.8 vs. Ember lane score of
3.2. Weight ratio: 7.8^0.7 / 3.2^0.7 = 4.9 / 2.3 = **2.1x**. You see roughly
2 on-color and 2 off-color/neutral per pack. The draft feels wide open.

**At pick 15, exponent = 1.5:** Same Tide score of 11.8 vs. Ember score of 3.2.
Weight ratio: 11.8^1.5 / 3.2^1.5 = 40.6 / 5.7 = **7.1x**. Now you see roughly
3+ on-color and 0.7 off-color per pack. The draft feels focused, but you still
see splash options regularly.

### The "Split DC Bonus" Concept

The original staged exponent proposal gave the dreamcaller a large single bonus
(4 profile points per color). This immediately created an 8x weight advantage
for on-color cards, making the "open" early phase feel dishonest.

This hybrid splits the dreamcaller's influence across two mechanisms:

| Mechanism | Parameter | Effect |
|-----------|-----------|--------|
| Profile seed (old system) | `dc_bonus = 2` | Dreamcaller adds 2 profile picks per color, giving a gentle early direction |
| Lane weight boost (new) | `dc_boost = 3.0` | Dreamcaller's colors get +3 added to their lane score permanently |

The total influence is slightly larger than before (10 effective units vs. 8),
but it is distributed so that the profile seed is weaker early (less weight
ratio amplification at exponent 0.7) while the lane boost provides a durable
structural advantage that grows in importance as the exponent rises.

**Before (dc_bonus=4, exponent 0.7):** On:off weight ratio at pick 1 = **8.0x**
**After (dc_bonus=2 + dc_boost=3, exponent 0.7):** On:off weight ratio at pick 1 = **1.8x**

That 4.4x improvement in early openness is the main driver of why this hybrid
passes early-game targets that the pure staged exponent could not.

---

## Key Parameters

| Parameter | Default | What It Controls | Turn It Up | Turn It Down |
|-----------|---------|-----------------|-----------|--------------|
| `exp_open` | 0.7 | Exponent during picks 1-5 | More on-color early, less variety | More variety early, weaker direction |
| `exp_committed` | 1.5 | Exponent cap at pick 15+ | Stronger late convergence, less splash | More splash late, weaker convergence |
| `open_end` | 5 | Last pick of flat open phase | Longer open exploration | Convergence ramp starts earlier |
| `committed_start` | 15 | Pick where exponent caps out | Longer ramp, more gradual | Faster ramp, more abrupt commitment |
| `lane_base` | 4.0 | Base weight of each color before seeding | Flatter pack composition, more variety | Stronger profile effect, less variety |
| `seed_min` | 0.60 | Minimum lane seed per quest | Lower floor, more lane asymmetry | Less variation between runs |
| `seed_max` | 1.40 | Maximum lane seed per quest | Higher ceiling, more lane asymmetry | Less variation between runs |
| `dc_bonus` | 2 | Profile points added per dreamcaller color | Stronger early direction from DC | Weaker early direction, more open |
| `dc_boost` | 3.0 | Lane score addition for DC colors | Faster convergence toward DC colors | Weaker DC identity |
| `pick_scale` | 0.5 | Lane score gain per card you pick | Faster convergence as you draft | Slower convergence as you draft |
| `floor_weight` | 1.0 | Minimum weight for any resonance card | More off-color cards always visible | Less baseline splash protection |
| `neutral_base` | 3.0 | Weight of neutral cards | More neutrals in packs | Fewer neutrals, more colored options |
| `staleness_factor` | 0.3 | How quickly seen-but-rejected cards fade | Faster card rotation | Same cards keep reappearing |

**Tuning priority:** Start with `dc_bonus` (2-3) and `exp_committed` (1.3-1.7),
since these control the convergence-vs-variety tradeoff. Tune `exp_open`
(0.6-0.8) and `lane_base` (3-5) second for early feel. Leave the rest at defaults.

---

## What Changed from the Original Strategy 4 Proposal

The original "Staged Exponent" proposal (Round 1) was straightforward: replace
the game's current static exponent of 1.4 with one that starts at 0.9 during
picks 1-5, ramps linearly, and reaches 2.0 at pick 11. The dreamcaller bonus
stayed at 4 per color. The inspiration was explicit MTG draft wisdom: "take the
best card for your first 5 picks, then lock into two colors."

### What the Original Got Right

The core insight — that the same formula should behave differently early versus
late — was confirmed by simulation and endorsed by every other proposal. The
original also contributed the phrase-by-phase parameter independence: tuning
`exp_open` doesn't affect late-game metrics, and tuning `exp_committed` barely
touches early metrics. This clean separation survived into the hybrid.

### What the Original Got Wrong

**The committed exponent of 2.0 was too aggressive.** At exponent 2.0 with a
developed profile, off-color cards have roughly 0.5% probability of appearing
per card draw. In a 4-card pack, that means fewer than 1 off-color card per 50
packs. Another proposal in the debate computed this as "5%", making explicit
what the simulation later confirmed: late off-color was 0.34 per pack (well below
the 0.5 target), and no amount of exponent tuning could fix it.

**The dreamcaller bonus of 4 dominated too early.** With dc_bonus=4 and
exponent=0.9, the on:off weight ratio at pick 1 was 7-8x. This was identified
as "the root cause of most target failures" across all five proposals — it made
early variety, convergence timing, and late splash simultaneously impossible to
achieve through pure weighting.

**The pure profile approach cannot break the convergence-splash tradeoff.** In
any system where off-color weight = `floor / (floor + profile^exp)`, raising
the exponent to get convergence necessarily makes `profile^exp` overwhelm the
`floor`, sending off-color to zero. The hybrid breaks this by replacing the
static `floor` with `lane_base^exp`, which scales up with the exponent instead
of being overwhelmed by it.

**The splash slot was a workaround, not a solution.** The original proposal added
a forced off-color card after pick 8. The hybrid doesn't need this because the
lane_base term naturally maintains ~25% off-color weight in late packs. The
structural solution beats the patch.

### The Evolution in Numbers

| Metric | Original S4 | Hybrid v2 | Target |
|--------|-------------|-----------|--------|
| Early on:off ratio (pick 1) | 8.0x | 1.8x | — |
| Late off-color per pack | 0.34 | 0.78 | >= 0.5 |
| Early unique resonances | 2.12 | 2.93 | >= 3.0 |
| Early on-color per pack | 2.21 | 1.99 | <= 2.0 |
| Targets passed (synergy) | 2/7 | 4/7 | 7/7 |

---

## Simulation Results

Simulations ran 1000 quests per player strategy (synergy, power_chaser, rigid).

**Parameter note:** The simulation used `floor_weight = 1.0` (not the 3.0 cited
in the hybrid proposal spec), `lane_base = 4.0`, `dc_bonus = 2`, `dc_boost =
3.0`, `exp_open = 0.7`, `exp_committed = 1.5`.

### Synergy Player (always picks the highest resonance-fit card)

| Metric | Target | Round 3 Baseline | Hybrid v2 | Result |
|--------|--------|-----------------|-----------|--------|
| Early unique resonances/pack | >= 3.0 | 2.12 | 2.93 | FAIL (near miss) |
| Early on-color/pack | <= 2.0 | 2.21 | 1.99 | PASS |
| Late on-color/pack | >= 2.0 | 2.94 | 2.68 | PASS |
| Late off-color/pack | >= 0.5 | 0.34 | 0.78 | PASS |
| Top-2 resonance share | 75%-90% | 97.0% | 94.3% | FAIL |
| Convergence pick | 5-8 | 4.4 | 4.6 | FAIL |
| Archetype pair frequency | all in [5%, 15%] | [8.9%, 11.2%] | [8.7%, 11.1%] | PASS |

### Power Chaser (always picks the highest raw power card, ignoring color)

| Metric | Target | Round 3 Baseline | Hybrid v2 | Result |
|--------|--------|-----------------|-----------|--------|
| Early unique resonances/pack | >= 3.0 | 2.15 | 2.94 | FAIL (near miss) |
| Early on-color/pack | <= 2.0 | 2.17 | 1.96 | PASS |
| Late on-color/pack | >= 2.0 | 2.71 | 2.33 | PASS |
| Late off-color/pack | >= 0.5 | 0.54 | 1.18 | PASS |
| Top-2 resonance share | 60%-85% | 75.0% | 61.2% | PASS |
| Convergence pick | 5-8 | 8.0 | 16.4 | FAIL |
| Archetype pair frequency | all in [5%, 15%] | [8.5%, 11.3%] | [8.4%, 12.1%] | PASS |

### Rigid Player (committed synergy player, never deviates from on-color)

| Metric | Target | Round 3 Baseline | Hybrid v2 | Result |
|--------|--------|-----------------|-----------|--------|
| Early unique resonances/pack | >= 3.0 | 2.11 | 2.93 | FAIL (near miss) |
| Early on-color/pack | <= 2.0 | 2.21 | 1.99 | PASS |
| Late on-color/pack | >= 2.0 | 2.95 | 2.69 | PASS |
| Late off-color/pack | >= 0.5 | 0.33 | 0.77 | PASS |
| Top-2 resonance share | 75%-95% | 99.7% | 97.2% | FAIL |
| Convergence pick | 5-8 | 4.4 | 4.9 | FAIL (near miss) |
| Archetype pair frequency | all in [5%, 15%] | [8.9%, 11.2%] | [8.8%, 11.2%] | PASS |

### Combined: 13/21 targets passed (up from 8/21 in Round 3)

### About the Remaining Failures

**Top-2 share too high (94.3% vs. 75-90% target):** The algorithm does offer
0.78 off-color cards per pack late, but the simulated synergy player always
ignores them in favor of on-color options. The top-2 share measures the player's
picks, not what the algorithm offers. Real players sometimes take an off-color
card for power reasons, which would lower this number. The parameter sweep shows
this metric is almost entirely driven by player strategy, not algorithm tuning —
no configuration tested could push below 92% for synergy players.

**Convergence pick too fast (4.6 vs. 5-8 target):** The dc_bonus=2 + dc_boost=3
combination still provides enough initial direction that early picks tend on-color
from the start. Setting dc_boost=1.0 in the parameter sweep achieves convergence
pick = 5.0 (inside the target), at the cost of slightly weaker early DC identity.

**Early unique resonances near miss (2.93 vs. >= 3.0):** Setting lane_base=5.0
achieves 3.01 (passes). The tradeoff is marginally weaker late convergence.

**Power chaser convergence (16.4 vs. 5-8):** This may actually be correct design
behavior. A power-chasing player who ignores resonance when picking should
converge slowly, and the algorithm correctly does not force them. Their deck ends
up genuinely diverse (top-2 share 61.2%, within target), which is a better
outcome than artificial convergence.

---

## Design Goal Scorecard

Scores are based on simulation outcomes and structural analysis, not theoretical
predictions. Each score is 1-10.

| # | Goal | Score | Key Evidence |
|---|------|-------|--------------|
| 1 | Simple | 7/10 | Two clean concepts (lane seeds + exponent ramp), 12 parameters, no slot types, no behavioral tracking. Comparable to S1 in simplicity. |
| 2 | Not On Rails | 8/10 | 1.8x early on:off ratio (down from 8.0x). 2.93 unique resonances early. Lane seeds create genuine per-run asymmetry. Loses 2 points for mild dc_boost bias from pick 1. |
| 3 | No Forced Decks | 8/10 | Deck overlap (Jaccard) = 0.056. Pair frequencies [8.7%, 11.1%], all within target. Lane seeds ensure different lane depths per quest even with same dreamcaller. |
| 4 | Flexible Archetypes | 7/10 | Late off-color 0.78/pack. Power chaser tri-color rate: 78.6%. Players who want a third color can find cards for it. Synergy tri-color remains near 0% (player choice, not algorithm force). |
| 5 | Convergent | 6/10 | Late on-color 2.68 (well above 2.0 target). Convergence pick 4.6 (slightly below 5-8 target). Top-2 share 94.3% (above 75-90% ceiling). Convergence is real but arrives a bit early. |
| 6 | Splashable | 8/10 | Late off-color 0.78 (up from 0.34 in Round 3, well above 0.5 target). The lane_base term provides a structural floor that scales with the exponent rather than being overwhelmed by it. |
| 7 | Open-Ended Early | 7/10 | Early unique resonances 2.93 (near 3.0 target). Early on-color 1.99 (passes <= 2.0). About 2 on-color and 2 off-color/neutral per pack in picks 1-5. |
| 8 | Signal Reading | 7/10 | Lane seeds [0.60, 1.40] create a 2.1x seed range. A deep Ruin lane (seed 1.33) visibly produces more Ruin cards than a shallow Ember lane (seed 0.62). Players who notice and respond are rewarded. |
| **Total** | | **58/80** | Highest score in the five-strategy analysis. Nearest competitor scores 54/80 (Strategy 3, but fails convergence). |

---

## Strengths

**1. Breaks the convergence-splash tradeoff.** This is the proposal's defining
achievement. In every other proposal, "late on-color >= 2.0" and "late off-color
>= 0.5" are in direct conflict. This hybrid passes both simultaneously for
synergy and rigid players (2.68 on-color, 0.78 off-color), and passes both by
wider margins for power chasers (2.33, 1.18).

**2. Dramatic improvement in early variety.** Early unique resonances rose from
2.12 to 2.93 (+0.81). Early on-color dropped from 2.21 to 1.99 (now passes
target). The on:off weight ratio at pick 1 fell from 8.0x to 1.8x — a 4.4x
improvement.

**3. Genuine per-run variety from lane seeds.** Deck overlap (Jaccard) = 0.056,
the lowest of any proposal with convergence. Two runs with the same dreamcaller
produce detectably different drafts because the lane depths differ. This is the
mechanism that directly addresses goal #3 (no forced decks).

**4. Naturally differentiates player types.** A synergy player's concentrated
picks amplify specific lane scores, which the exponent then heavily rewards,
creating fast convergence. A power chaser's scattered picks produce flat lane
scores that the exponent barely differentiates, enabling diverse multi-color
decks. No explicit behavioral tracking needed — the math does it.

**5. Off-color floor that scales correctly.** At late-game exponent 1.5, the
off-color lane_base term (4.0 * seed ≈ 3.2) contributes 3.2^1.5 = 5.7 weight
units. An on-color lane score of roughly 11 contributes 11^1.5 = 36.5. The ratio
is roughly 6:1 — strong convergence, but not the 200:1 ratio that pure
profile-based systems produce at high exponents. Off-color cards maintain roughly
18% weight share in late packs.

**6. +5 targets passed vs. Round 3 baseline.** Moving from 8/21 to 13/21 is the
largest single-round improvement in the simulation series.

---

## Weaknesses

**1. Top-2 share too concentrated for synergy players (94.3% vs. 75-90% target).**
The algorithm offers 0.78 off-color cards per pack late, but the simulated synergy
player always picks on-color. This means the algorithm passes its own "offering"
test but fails the "player outcome" test. The failure is partly a simulation
artifact — real players are more flexible — but it is a real concern. No algorithm
configuration tested could push synergy top-2 share below 92%.

**2. Convergence happens slightly too fast (4.6 picks vs. 5-8 target).** The
combination of dc_bonus=2 and dc_boost=3.0 provides enough initial direction that
decks typically cross the convergence threshold by pick 4-5 instead of 5-8.
dc_boost=1.0 fixes this (achieves 5.0), but reduces the dreamcaller's early
identity feel.

**3. Early unique resonances just misses (2.93 vs. >= 3.0 target).** Tantalizing
but consistent near-miss. Requires either lane_base=5.0 (passes at 3.01, slight
convergence cost) or some other structural mechanism to guarantee a third distinct
resonance in early packs.

**4. Power chaser convergence very late (16.4 vs. 5-8 target).** Power chasers
pick by power rather than resonance, producing scattered profiles, which the
lane-based system does not aggressively convergence. Whether 16.4 is a bug or a
feature depends on design intent — a power chaser who ignores resonance arguably
should not converge to a tight dual-color deck. Their final deck is still legal
and internally diverse (top-2 share 61.2%, within target).

**5. Two concepts instead of one.** The simplest proposal (Strategy 1) needs one
sentence to explain: "cards matching your resonances appear more as your
collection grows." This proposal needs two: lane seeds and the exponent ramp.
Both are intuitive, but there is genuine added cognitive load compared to the
purest approaches.

---

## Draft Story Examples

Three concrete runs from the simulation, illustrating how lane seeds and the
exponent ramp feel in practice.

### Story 1: Early Committer (Synergy Player)

**Dreamcaller:** Ember, Stone
**Lane seeds rolled:** Tide=0.96, Ember=1.22, Zephyr=1.16, Stone=1.19, Ruin=0.95

Both of the player's dreamcaller colors happen to be in deep lanes. From pick 1,
packs contain a clear signal: Ember and Stone cards appear consistently alongside
weaker Tide and Zephyr options. The exponent at pick 1 is 0.7, so on:off ratio
is about 1.8x — genuinely open — but Ember and Stone keep appearing most
naturally. The player takes on-color picks throughout.

By pick 5, the exponent is still 0.7, but the player has 5 profile points in
Ember and 4 in Stone. The ramp begins. By pick 8, the exponent is 0.94 and
packs are noticeably more on-color. By pick 15, the exponent caps at 1.5 and
packs are heavily Ember+Stone with regular off-color appearances that the player
doesn't need but could take.

**Final deck:** Ember=20, Stone=19, Tide=1. Top-2 share: 97.5%. Convergence
pick 4.

**Verdict:** A clean, satisfying dual-color draft. The lane seeds reinforced the
dreamcaller direction. The player never felt forced — they just made good picks
and the world cooperated.

---

### Story 2: Late Committer (Same Dreamcaller, Different Lanes)

**Dreamcaller:** Ember, Stone
**Lane seeds rolled:** Tide=0.96, Ember=0.89, Zephyr=0.82, Stone=0.87, Ruin=1.20

Same dreamcaller. Completely different run. Ember and Stone are both shallow
lanes this time. Ruin is the deepest lane at 1.20, despite being off-DC.

Early packs show Ruin cards appearing frequently — not because Ruin matches the
dreamcaller, but because Ruin has the deepest structural weight. The player still
has dc_boost nudging them toward Ember and Stone, but the competition is real. At
pick 5, the player picks up a Ruin+Stone dual card (a bridge between their DC
color and the deep lane). By the shop, they buy a mixed batch including Zephyr+Stone.

**Final deck:** Ember=22, Stone=17, Zephyr=3, Ruin=3, Tide=1. Convergence: pick 5.
Top-2 share: 84.8%.

**Verdict:** The shallow DC lanes created a more exploratory draft. A player who
notices Ruin cards appearing frequently could read the signal and pivot away from
their dreamcaller colors entirely. This is signal reading in action: the "open
lane" this run was Ruin, and the draft rewarded players who noticed.

---

### Story 3: Power Chaser (Different Dreamcaller)

**Dreamcaller:** Ember, Tide
**Lane seeds rolled:** Tide=1.19, Ember=0.62, Zephyr=1.09, Stone=1.24, Ruin=1.33

Ember — one of the dreamcaller colors — has the shallowest lane (0.62). Ruin
and Stone have the deepest lanes despite being off-DC. The power chaser picks
purely by power, ignoring resonance.

By pick 10, the profile is Tide=4, Ember=3, Ruin=2, Stone=1 — a four-color
spread. The deep Ruin and Stone lanes keep feeding high-power off-DC cards that
the power chaser finds attractive. At the shop, every available card from every
color gets picked.

The late game shows a fascinating dynamic: even at exponent 1.5, the power
chaser's scattered profile means no single lane has a dominant lane score. Packs
remain diverse throughout.

**Final deck:** Tide=13, Ember=11, Stone=9, Ruin=7, Zephyr=4. Top-2 share: 54.5%.
Classification: tri-color. Convergence pick: 16+ (never really converges).

**Verdict:** The power chaser gets the diverse deck they want because the lane
seeds spread weight across all colors, and their scattered picks never create
the concentrated profile that the exponent needs to produce convergence. No
behavioral tracking required — the math naturally produces this differentiation.

---

## Comparison to the Other Four Hybrids

This project evaluated five redesign strategies (S1-S5) before combining them
into five hybrid proposals. Here is how the Lane-Seeded Staged Exponent compares
to the alternatives.

### vs. Hybrid 1 (CRESCENDO / Profile-Only Ramp)

**What it shares:** Both use a pick-number-dependent exponent that starts low
and ends higher. Both are philosophically similar in approach.

**Key difference:** CRESCENDO uses a pure profile-based formula where off-color
weight = a static floor / (floor + profile^exp). At any exponent capable of
producing convergence, the floor is overwhelmed and off-color approaches zero.
CRESCENDO's simulation showed late off-color = 0.05 per pack (compared to 0.78
here). CRESCENDO compensates with an aggressive diversity check, but this is
a procedural patch rather than a structural solution.

**Choose CRESCENDO if:** Simplicity is the overriding concern and you are
willing to rely on the diversity check to provide splash visibility. One formula,
no lane seeds.

**Choose this hybrid if:** You want late-game splash to emerge naturally from
the weighting rather than from a forced procedural intervention.

### vs. Hybrid 2 (Structured Pack Slots)

**What it shares:** Both try to guarantee a mix of on-color and off-color cards
per pack.

**Key difference:** Hybrid 2 uses slot types — Focus slots (always on-color),
Flex slots (bridge cards), Wild slots (off-color) — to structurally guarantee
pack composition. This produces stronger guarantees but creates a mechanical feel
where experienced players learn to predict slot outcomes ("there are always
exactly 2 good cards and 2 others"). The parameter sweep also showed that Focus
slot exponents have near-zero sensitivity — Focus slots deliver on-color regardless
of their exponent because profile values dominate.

**Choose Hybrid 2 if:** You want guaranteed pack composition structure and
are comfortable with a more "designed" feel to each pack.

**Choose this hybrid if:** You want probabilistic variety that feels organic —
sometimes 3 on-color, sometimes 2, sometimes a surprising off-color — rather
than a reliable structure that experienced players will parse quickly.

### vs. Hybrid 3 (Lane Pool)

**What it shares:** Both use lane seeds for per-quest pool asymmetry and signal
reading. This hybrid directly incorporates S3's lane seeding concept.

**Key difference:** Pure Lane Pool strategies let players deplete their preferred
lanes by drafting from them — a negative feedback loop that fights convergence.
Late on-color for the Lane Pool strategy was 1.94 per pack, narrowly failing the
2.0 target. This hybrid replaces the depletion mechanic with profile-based
positive feedback (picks * pick_scale) combined with the staged exponent, so the
more you commit to a color, the more of that color appears — the direction of
feedback is correct.

**Choose Hybrid 3 if:** You believe the environment-depletion model is
philosophically more interesting (convergence emerges from scarcity, not
amplification) and are willing to accept weaker convergence.

**Choose this hybrid if:** You want the signal-reading and run-variety benefits
of lane seeds with genuine, reliable convergence.

### vs. Hybrid 5 (Adaptive / Behavioral Tracking)

**What it shares:** Both track some state beyond the basic profile to modulate
the algorithm.

**Key difference:** Hybrid 5 tracks player behavior (commitment score, recent
pick weights) through an exponential moving average and adjusts the exponent
multiplier accordingly. The parameter sweep showed that the behavioral tracking
parameters have near-zero sensitivity: `slow_min` and `fast_max` produce less
than 0.1 change across all metrics. Hybrid 5's own author ranked it last after
seeing these results. This hybrid achieves natural player-type differentiation
(synergy vs. power chaser) through the lane score math without any explicit
behavioral tracking, using 12 parameters instead of Hybrid 5's 18.

**Choose Hybrid 5 if:** You believe behavioral differentiation is important
enough to justify the complexity and want the algorithm to explicitly adapt to
how decisively players pick.

**Choose this hybrid if:** You want player-type differentiation to emerge from
the design naturally, with less implementation complexity and easier debugging.

### Summary Comparison

| | H1 CRESCENDO | H2 Structured | H3 Lane Pool | **H4 Lane+Staged** | H5 Adaptive |
|--|--|--|--|--|--|
| Parameters | ~7 | ~13 | ~10 | **12** | ~18 |
| Targets passed (synergy) | 3/7 | 3/7 | 3/7 | **4/7** | 2/7 |
| Late off-color | 0.05 | 0.19 | 1.42 | **0.78** | 0.08 |
| Late on-color | 3.66 | 3.07 | 1.94 | **2.68** | 3.57 |
| Design goal total | 37/80 | 35/80 | 54/80 | **58/80** | 33/80 |
| Passes both convergence+splash | No | No | No | **Yes** | No |

The defining difference is the final row. This is the only proposal that
simultaneously passes the convergence target (late on-color >= 2.0) and the
splashability target (late off-color >= 0.5) for synergy players — something no
other proposal can achieve without a structural procedural intervention like forced
slot types.

---

## Appendix: Weight Table at Default Parameters

With DC = Tide+Ruin, lane seeds Tide=1.20, Ember=0.80, Zephyr=1.00, Stone=0.70,
Ruin=1.30, after drafting 4 Tide picks and 3 Ruin picks:

| Pick | Exp | Tide Score | Tide Wt | Ruin Score | Ruin Wt | Neutral | Ember Score | Ember Wt | On:Off Ratio |
|------|-----|-----------|---------|-----------|---------|---------|------------|---------|-------------|
| 1 | 0.70 | 7.8 | 5.6 | 8.2 | 5.9 | 3.0 | 3.2 | 2.6 | **1.8x** |
| 5 | 0.70 | 7.8 | 5.6 | 8.2 | 5.9 | 3.0 | 3.2 | 2.6 | **1.8x** |
| 8 | 0.94 | 9.8 | 9.5 | 10.2 | 9.9 | 3.0 | 3.2 | 3.0 | **2.3x** |
| 11 | 1.18 | 12.3 | 15.8 | 12.7 | 16.6 | 3.0 | 3.2 | 3.8 | **3.1x** |
| 15 | 1.50 | 14.8 | 57.1 | 15.2 | 59.2 | 3.0 | 3.2 | 5.7 | **4.5x** |

*Weights shown are lane_score^exponent for the resonance term only; final card weight also adds floor_weight.*

Lane scores grow as the profile accumulates picks. On:off ratio rises from 1.8x
(pick 1) to 4.5x (pick 15). At no point does off-color collapse to zero, because
even a shallow off-color lane with seed=0.80 contributes lane_base * 0.80 = 3.2
to the lane score, and 3.2^1.5 = 5.7 at the committed exponent — a meaningful
weight that does not vanish.
