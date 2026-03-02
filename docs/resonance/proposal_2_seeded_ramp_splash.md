# Proposal 2: Seeded Ramp with Structural Splash

**Resonance Algorithm Redesign — Hybrid Proposal**

---

## Overview

Seeded Ramp with Structural Splash is a drafting algorithm that gives every
pack of four cards a clear job to do. Three of the four cards are drawn using a
profile-based weighting formula that starts open and gradually tightens toward
your chosen resonances — the further into a draft you are, the more the system
emphasizes cards that match what you have already built. The fourth card is
always drawn from a separately flattened formula designed specifically to keep
off-color options visible throughout the entire draft, not just early on. The
whole pool is shaped at the start of each quest by a set of per-resonance
multipliers called lane seeds, which ensure that the optimal resonance mix
varies from run to run. The player never sees any of this machinery; they just
see four cards, shuffled, and make a pick.

---

## How It Works

### The Basic Idea: Two Jobs, Two Formulas

The central insight behind this proposal is that a single weighting formula
cannot simultaneously do two things well: push you toward your committed colors
late in a draft (convergence), and reliably surface cards from colors you
haven't committed to (splash). These goals pull in opposite directions.

The solution is to stop trying to do both with the same formula. Every pack is
assembled in two stages:

- **Three convergence cards** are drawn using a formula that grows stronger as
  your profile deepens. These are the cards that reward your choices.
- **One wild card** is drawn using a separate formula specifically tuned to
  remain nearly flat even when your profile is deep. This is the card that
  keeps surprising you.

The pack is then shuffled before you see it. You do not know which card was the
wild card. You just see four options.

### The Convergence Formula (Three of Four Cards)

Each of the three convergence cards is weighted by this formula:

```
weight = profile_count ^ exponent(pick) + 0.5
```

Where:

- `profile_count` is how many cards of that resonance you have accumulated,
  capped at 10.
- `exponent(pick)` starts at **0.5** on pick 1 and ramps up to **1.4** by pick
  8, then stays there.
- The `+ 0.5` is a floor — even a resonance you have never touched has a small
  chance to appear.

**What the exponent ramp means in practice:** At exponent 0.5, the math is
sub-linear — doubling your profile count does not double a card's weight. A
resonance with profile 10 has weight 10^0.5 + 0.5 = 3.66. A resonance with
profile 0 has weight 0.5. The ratio is about 7:1 — tilted toward your colors
but genuinely open. By pick 8, at exponent 1.4, the same resonance has weight
10^1.4 + 0.5 = 25.6 versus 0.5 for an unchosen color. The ratio is now 51:1.
The system has become strongly focused.

**The profile cap:** Profile counts are capped at 10 for the purposes of weight
calculation, even if you have drafted more. Without this cap, a profile of 25
would produce weight ratios over 134:1 late in a draft, making off-color cards
effectively invisible even in the wild slot. The cap keeps weight ratios in a
meaningful range.

**Concrete example:** You are at pick 10 with Tide profile 8 and Ruin profile
6. A Tide card has weight 8^1.4 + 0.5 = 18.5. A Ruin card has weight 6^1.4 +
0.5 = 12.3. A Zephyr card (profile 0) has weight 0.5. The system strongly
favors Tide and Ruin, but Zephyr is not impossible in this slot.

### The Wild Card Formula (One of Four Cards)

```
wild_weight = profile_count ^ 0.3 + 5.0
```

The two key differences from the convergence formula:

- The exponent is fixed at **0.3** (extremely sub-linear — profile count barely
  matters).
- The floor is **5.0** (ten times higher than the convergence floor).

**What this means in practice:** At profile 10, an on-color card in the wild
slot has weight 10^0.3 + 5.0 = 7.0. A completely unchosen resonance has weight
5.0. The ratio is only 1.4:1 — nearly flat. An off-color card appears in the
wild slot roughly 40-50% of the time, regardless of where you are in the draft.

This is the structural guarantee. No matter how deep your Tide+Ruin commitment
is by pick 15, the wild slot still has close to even odds of showing you
something from Ember, Stone, or Zephyr. The convergence formula cannot do this
— at exponent 1.4, the weight ratio becomes prohibitive.

### The 3+1 Pack Structure

Picks 1 through 5 use a **2+2 split** — two convergence cards and two wild
cards. This wider-open early phase is designed to maximize the variety of
resonances you see while your profile is still forming.

Starting at pick 6, the pack shifts to **3+1** — three convergence cards and
one wild card. This is the main convergence signal: as you move from exploration
into commitment, the packs tilt noticeably toward what you've been building.

The split at pick 6 is a hard boundary. A pack at pick 5 feels genuinely open;
a pack at pick 6 feels more focused. This is intentional — it mirrors the
moment when a drafter should be crystallizing their identity.

### Lane Seeds: Making Each Run Distinct

Before a quest begins, the system rolls a random multiplier for each resonance,
drawn uniformly between 0.60 and 1.40. These are called **lane seeds**.

A resonance with lane seed 1.30 effectively has 30% more cards in the pool.
A resonance with lane seed 0.70 has 30% fewer. Every card in the pool has its
effective copy count scaled by its primary resonance's lane seed.

**What this does for play:** In one run, Stone might be the deepest resonance
in the pool and Ember the shallowest — even before a single card is picked.
This means the optimal archetype genuinely changes from quest to quest. A player
who learns to read which resonances are appearing frequently can exploit this
signal. It also means that even with the same dreamcaller, different runs
produce structurally different decks.

**Draft Story 2** (from the simulation) illustrates this well: a player with an
Ember+Tide dreamcaller found that Ember had the lowest lane seed (0.62) in that
run. Despite the dreamcaller's identity, the pool had fewer Ember cards than
any other resonance. The player's eventual deck was Tide-primary, not Ember,
because the signals pointing toward Tide were stronger.

### The Dreamcaller Bonus

At quest start, your dreamcaller adds **2 profile points** to each of its two
resonances (reduced from 4 in earlier proposals). This provides a starting
direction — your dreamcaller's colors appear roughly twice as often as unchosen
colors in pick 1 — without locking in an archetype before you've seen a single
card.

At the initial profile of 2 with exponent 0.5, an on-color card has weight
0.5 + 2^0.5 = 1.91. An off-color card has weight 0.5. The ratio is 3.8:1 —
tilted but genuinely open.

### Battle Rewards and Shops

Battle rewards do not include a wild card. They use the standard convergence
formula with the current pick's exponent. These are high-stakes, on-color
opportunities and are intentionally more focused.

Shops also do not use the wild card structure. Shop cards use the committed-
phase exponent (1.4) and standard weighting. Shops feel curated, not
exploratory.

---

## Key Parameters

| Parameter | Default | Controls | Higher values | Lower values |
|-----------|---------|----------|---------------|--------------|
| `lane_seed_min` | 0.60 | Minimum pool copy-count multiplier per resonance | Narrower run-to-run variety | Wider variety, more lopsided runs |
| `lane_seed_max` | 1.40 | Maximum pool copy-count multiplier per resonance | Wider run-to-run variety | More uniform pools across runs |
| `dreamcaller_bonus` | 2 | Profile points added per DC resonance at start | Earlier convergence, less variety early | More open early picks, weaker DC identity |
| `base_exp` | 0.5 | Exponent at pick 1 (convergence cards) | More on-color tilt early | More variety early |
| `max_exp` | 1.4 | Exponent at pick 8+ (convergence cards) | Stronger late convergence | Weaker late convergence, more late splash |
| `convergence_picks` | 8 | How many picks to reach `max_exp` | Slower ramp | Faster ramp to full convergence |
| `floor_weight` | 0.5 | Minimum weight for unchosen resonances in convergence cards | Slightly more off-color in convergence slots | Less effect (floor already low) |
| `neutral_base` | 3.0 | Weight of neutral cards in convergence slots | Fewer on-color convergence cards | More on-color convergence cards |
| `profile_cap` | 10 | Maximum profile count used in weight calculations | Stronger convergence, less splash late | Weaker convergence, more variety late |
| `wild_floor` | 5.0 | Minimum weight for any resonance in the wild slot | More off-color in wild slot | Less off-color, wild slot becomes more on-color biased |
| `wild_exp` | 0.3 | Exponent for wild slot (controls how much profile matters) | Wild slot favors on-color more | Wild slot becomes flatter, more off-color |
| `open_phase_max_pick` | 5 | Last pick using the 2+2 (two wild) partition | Longer open phase | Shorter open phase, faster commitment |

### Which Parameters Matter Most

**High sensitivity — tune carefully:**
- `wild_floor` is the primary lever for off-color visibility. The simulation
  sweep showed that moving from 3.0 to 7.0 changes late off-color from 0.64 to
  0.73 cards per pack. Values below 3.0 risk failing the off-color target.
- `max_exp` is the primary lever for late convergence strength. Moving from 1.0
  to 1.8 changes late on-color from 2.66 to 3.21 per pack.
- `dreamcaller_bonus` controls convergence timing, early variety, and top-2
  share simultaneously. It is as much a design question as a tuning question.

**Moderate sensitivity — set thoughtfully:**
- `base_exp`, `profile_cap`, and `wild_exp` each move metrics by a meaningful
  but not dramatic amount. They are good second-order adjustments once the high-
  sensitivity parameters are locked.

**Low sensitivity — set and forget:**
- `convergence_picks`, `floor_weight`, `neutral_base`, `staleness_factor`, and
  the lane seed range are all robust across a wide range of values. The
  simulation sweep showed near-zero sensitivity for staleness in particular;
  it could be eliminated entirely without affecting outcomes.

---

## What Changed from the Original Strategy

The original Strategy 2 (Structured Pack Composition) was a different beast. It
used three named slot types — Focus, Flex, and Wild — with separate weight
functions for each, and a three-phase system (Open, Commit, Refine) that
changed slot counts and exponents at two boundaries.

### What the Original Strategy Did

In the original proposal:

- **Focus slots** drew on-color cards with a high exponent (1.2 in Open, 1.6 in
  Commit, 2.0 in Refine). A Commit-phase pack had two Focus slots.
- **Flex slots** drew neutral cards or bridge duals — cards that share one
  resonance with your top-2 and one with a third color. This was a dedicated
  mechanism for supporting three-color builds.
- **Wild slots** drew from a flattened distribution (exponent 0.7, floor 0.8).
  Wild floor of 0.8 turned out to be far too low: simulation showed late off-
  color of only 0.19 per pack, far below the 0.5 target.

The original had 17 tunable parameters and three explicit phase boundaries.

### What the Debate Revealed

**The Focus death spiral.** When a resonance has a low lane seed and the player
has drafted many cards from it, the Focus slot begins drawing from a small,
stale subset of that resonance's cards. Repeated draws from the same handful
of cards create staleness pile-up, and the Focus slot gradually stops feeling
useful. Single-distribution approaches degrade gracefully; the slot structure
creates isolated sub-pools that can fail individually.

**The Flex slot was expendable.** The bridge formula in the Flex slot was
conceptually interesting but negligible in practice. The parameter sweep showed
bridge_bonus had sensitivity of only 0.01 on late off-color. The mechanism
could be preserved as a weight modifier applied globally, but a dedicated slot
was not needed.

**The Wild floor was the only parameter that mattered for off-color.** After
testing many configurations, the simulation showed that only `wild_floor` and
`wild_exp` had meaningful impact on whether the off-color target was met. The
composite configuration "WF5+0F+1CF" (wild_floor=5.0, exponent_wild=0.3, no
open-phase Focus slots, one committed-phase Focus slot) achieved late off-color
of 0.66 — the only configuration in the entire S2 sweep that passed the target.

**The high dreamcaller bonus was causing most failures.** The dreamcaller bonus
of 4 per resonance (8 total profile points at pick 0) was the single largest
source of metric failures across all five strategies. It created a 7-14x weight
ratio at pick 1, pushing synergy top-2 share above 95% and convergence pick
below 5 — failures that persisted regardless of algorithm choice. Reducing the
bonus to 2 was the most impactful single change.

**The phase system created abrupt transitions.** The jump from 1 Focus slot to
2 Focus slots at pick 6 was noticeable and could not be smoothed within an
integer-valued slot system. The continuous exponent ramp from Strategy 1
(CRESCENDO) achieves the same convergence effect with a smoother experience.

### What Was Kept, Dropped, and Added

| Element | Decision | Reason |
|---------|----------|--------|
| Slot type labels (Focus/Flex/Wild) | Dropped | Invisible wild card replaces visible slot categories |
| Three-phase system | Simplified | One 2+2 to 3+1 transition replaces three phase boundaries |
| Flex slot | Dropped | Bridge formula negligible; dedicated slot not worth complexity |
| Wild slot (high floor) | Kept, retuned | Floor raised from 0.8 to 5.0 — the key insight from the sweep |
| Wild exponent | Kept, lowered | Lowered from 0.7 to 0.3 to complement higher floor |
| Continuous exponent ramp | Added from S1 | Cleaner convergence than per-phase constant exponents |
| Lane seeds | Added from S3 | Per-resonance copy-count multipliers for run-to-run variety |
| Dreamcaller bonus | Halved (4 to 2) | Single largest lever for early variety and convergence timing |
| Profile cap | Added | Prevents astronomical weight ratios at high profile counts |
| Bridge formula | Dropped | Negligible effect confirmed by sensitivity analysis |
| Staleness | Kept | Negligible sensitivity but harmless to keep |

The resulting design has **11 tunable parameters** compared to 17 in the
original and 15+ in the adaptive strategy.

---

## Simulation Results

Results from 1,000 simulated quests per player type.

### Target Checklist

| Metric | Target | Synergy | Power Chaser | Rigid | Pass? |
|--------|--------|---------|--------------|-------|-------|
| Picks 1-5: unique resonances per pack | >= 3.0 | 2.78 | 2.81 | 2.77 | **FAIL** |
| Picks 1-5: on-color cards per pack | <= 2.0 | 1.99 | 1.88 | 1.99 | **PASS** |
| Picks 6+: on-color cards per pack | >= 2.0 | 2.98 | 2.47 | 3.01 | **PASS** |
| Picks 6+: off-color cards per pack | >= 0.5 | 0.70 | 1.23 | 0.66 | **PASS** |
| Convergence pick (mean) | 5-8 | 4.8 | 13.4 | 5.0 | **FAIL** |
| Archetype pair max frequency | <= 15% | 11.3% | 12.0% | 11.2% | **PASS** |
| Archetype pair min frequency | >= 5% | 8.9% | 7.8% | 8.9% | **PASS** |
| Final top-2 share (synergy) | 75-90% | 96.7% | -- | -- | **FAIL** |
| Final top-2 share (power chaser) | 60-85% | -- | 65.4% | -- | **PASS** |

**Overall pass rate: 18/24 (75%)**, up from 50% in the original S2 proposal.

### Results by Player Type

**Synergy player** (always picks the highest-fit on-color option):

| Metric | Value |
|--------|-------|
| Early unique resonances per pack | 2.78 |
| Early on-color per pack | 1.99 |
| Late on-color per pack | 2.98 |
| Late off-color per pack | 0.70 |
| Top-2 share | 96.7% (std 3.2%) |
| Convergence pick (mean) | 4.8 (median 5.0) |
| Deck overlap (Jaccard) | 0.059 |
| Dual-color decks | 99.8% |
| Tri-color decks | 0.0% |

**Power chaser** (always picks highest raw power regardless of resonance):

| Metric | Value |
|--------|-------|
| Early unique resonances per pack | 2.81 |
| Early on-color per pack | 1.88 |
| Late on-color per pack | 2.47 |
| Late off-color per pack | 1.23 |
| Top-2 share | 65.4% (std 8.5%) |
| Convergence pick (mean) | 13.4 (median 6.0) |
| Deck overlap (Jaccard) | 0.055 |
| Dual-color decks | 31.2% |
| Tri-color decks | 68.8% |

**Rigid player** (always picks on-color, even low-power):

| Metric | Value |
|--------|-------|
| Early unique resonances per pack | 2.77 |
| Early on-color per pack | 1.99 |
| Late on-color per pack | 3.01 |
| Late off-color per pack | 0.66 |
| Top-2 share | 99.1% (std 1.7%) |
| Convergence pick (mean) | 5.0 |
| Dual-color decks | 99.2% |
| Tri-color decks | 0.0% |

### Where the Failures Come From

**Early unique resonances (2.78 vs 3.0 target):** This may be a structural
ceiling of a 4-card pack with 5 resonances. With dual-resonance cards
representing about 10% of the pool, reaching 3.0 unique resonances per 4-card
pack requires more consistent color spread than the 2+2 partition can reliably
deliver. The hybrid came closer than any previous configuration (the original
S2 default reached only 2.42), but the last 0.22 may require a design
intervention beyond parameter tuning.

**Synergy convergence pick (4.8 vs target of 5-8):** Even with dreamcaller
bonus reduced to 2, the synergy player picks so aggressively on-color that they
establish top-2 dominance very early. The dc_bonus sweep showed this barely
changes even at dc_bonus=1 (4.8) vs dc_bonus=4 (4.7). The failure is driven by
player behavior, not the algorithm.

**Synergy top-2 share (96.7% vs target of 75-90%):** Same root cause. The
algorithm offers off-color cards (0.70 per late pack), but the synergy player
doesn't take them. The 75-90% range may only be achievable if the synergy player
model is given weaker fit preference, or if the design target is recalibrated to
acknowledge that highly focused drafters will produce highly focused decks.

### Comparison to Original S2 Proposal

| Metric | Original S2 (Synergy) | Hybrid (Synergy) | Change |
|--------|----------------------|------------------|--------|
| Early unique res per pack | 2.42 | 2.78 | +0.36 |
| Early on-color per pack | 2.51 | 1.99 | -0.52 (fixed) |
| Late on-color per pack | 3.07 | 2.98 | -0.09 (minor regression) |
| Late off-color per pack | 0.19 | 0.70 | +0.51 (3.7x improvement) |
| Top-2 share | 97.6% | 96.7% | Marginal improvement |
| Convergence pick | 4.1 | 4.8 | +0.7 (improved) |
| Overall pass rate | 12/24 (50%) | 18/24 (75%) | +6 checks |

---

## Design Goal Scorecard

Scores from the post-simulation analysis, rated 1-10.

| Goal | Score | Justification |
|------|-------|---------------|
| **1. Simple** (explainable in one sentence) | 7/10 | No visible slot types, no phase labels, no player-facing complexity. One sentence: "your deck shapes what you see, and every pack has one wild card." Slightly more complex than pure CRESCENDO because the wild card is an invisible structural mechanism, but the player experience is nearly identical. |
| **2. Not on rails** (real choices per pack) | 7/10 | Early on-color of 1.99 passes the <= 2.0 target. Late off-color of 0.70 means every pack reliably contains off-color options. Wild card structurally guarantees variety in a way that pure weighting cannot. Lane seeds create genuine tension between synergistic and pool-signal picks. |
| **3. No forced decks** | 8/10 | Lane seeds create pool shapes that vary per quest. Archetype pair frequency range of 8.9%-11.3% is well within the 5%-15% target. Jaccard deck overlap of 0.059 is low. Each quest has a genuinely different optimal resonance path. |
| **4. Flexible archetypes** | 5/10 | Power chasers build tri-color 68.8% of the time (massive improvement from 2.3% in original S2). But synergy players produce 0% tri-color and 96.7% top-2 share — the algorithm offers off-color but the player ignores it. Score limited by the synergy player model's behavior. |
| **5. Convergent** | 6/10 | Late on-color of 2.98 easily clears the 2.0 target. But convergence pick of 4.8 falls just outside the 5-8 target, and it barely moved from the 4.1 of the original despite a halved dreamcaller bonus. The metric is more sensitive to player behavior than to algorithm design. |
| **6. Splashable** | 9/10 | Late off-color of 0.70 comfortably exceeds the 0.5 target. The wild card with floor=5.0 is the structural guarantee — the S2 sweep confirmed that wild_floor=5.0 with exp=0.3 reliably delivers 0.51-0.66 off-color per pack. This is the proposal's signature strength. |
| **7. Open early** | 4/10 | Early on-color of 1.99 passes the <= 2.0 target (barely). But early unique resonances of 2.78 falls below the 3.0 target. The 2+2 partition in picks 1-5 helped significantly (from 2.42 in original S2), but closing the last gap may require deeper structural changes. |
| **8. Signal reading** | 6/10 | Lane seeds create visible resonance asymmetries — a player who notices "more Tide cards than usual" in early packs is reading a real signal. But the three convergence cards in each pack still dominate pack composition based on profile, partially drowning the pool signal. |

**Total: 52/80**

For reference, the original S2 strategy scored approximately 34/80 using the
same 8-goal framework across all strategies in Round 3.

---

## Strengths

**Splash is structurally guaranteed.** This is the proposal's defining
contribution. The wild card with floor=5.0 and exponent=0.3 produces off-color
cards roughly 40-50% of the time, consistently across all three player types.
The synergy player sees 0.70 off-color cards per late pack; even the rigid
player (who picks on-color regardless) sees 0.66. No pure-weighting approach
achieved this without a supplementary procedural mechanism.

**Power chaser experience is excellent.** Power chasers build tri-color decks
68.8% of the time (up from 2.3% in the original S2). Their top-2 share of 65.4%
falls within the 60-85% target. The wild card continuously surfaces off-color
high-power options, and power chasers take them. This is the largest single
improvement over any Round 3 strategy for this player type.

**Early game is meaningfully open.** The 2+2 partition in picks 1-5 combined
with the reduced dreamcaller bonus produces early on-color of 1.99 — right at
the target. The original S2 produced 2.51 early on-color, a clear failure.

**Run-to-run variety is real.** The archetype pair frequency range (8.9%-11.3%)
is evenly distributed, and lane seeds create pool shapes that shift which
archetype is optimal. Draft Story 2 from the simulation showed lane seeds
directly causing a player to build Tide-primary despite an Ember+Tide dreamcaller
— the pool was signaling Tide, and the player who noticed that signal was
rewarded.

**Dramatically improved over the original S2 on every metric that failed.** Late
off-color improved 3.7x (0.19 to 0.70). Early on-color moved from fail to pass.
Pass rate went from 50% to 75%.

**Fewer parameters than any other complex strategy.** 11 tunable parameters
compared to 17 in original S2 and 15+ in the adaptive strategy, with most
parameters being low-sensitivity (set and forget).

---

## Weaknesses

**Early variety still falls short.** Early unique resonances of 2.78 falls 0.22
below the 3.0 target. The wild card sweep showed this metric barely moves with
wild_floor changes (sensitivity 0.02-0.04 across the tested range). This may
be a ceiling imposed by the 4-card pack size with 5 resonances, not a parameter
tuning problem.

**Synergy convergence pick remains too fast.** The mean of 4.8 falls just
outside the 5-8 target. Crucially, the dc_bonus sweep showed this metric barely
moves: dc_bonus=1 produces convergence pick 4.8, dc_bonus=4 produces 4.7. The
synergy player establishes top-2 dominance so quickly through aggressive on-color
picking that the algorithm cannot delay it without fundamentally changing the
weighting structure.

**Synergy top-2 share is stuck at 96.7%.** The 75-90% target was not achieved by
any strategy for synergy players. The algorithm offers off-color via the wild
card, but the synergy player never takes it. Whether this represents a design
target calibration problem or a real usability gap is an open question.

**Synergy players still build almost exclusively dual-color.** 99.8% dual, 0%
tri-color for synergy players. The wild card appears but is ignored. The proposal
supports flexible archetypes through availability, not through incentive.

**The 2+2 to 3+1 transition is a hard boundary.** Pick 5 feels open; pick 6
feels committed. This is intentional, but it lacks the smoothness of a
continuous exponent ramp applied to all four cards. A designer who finds the
boundary jarring could fall back to fixed 3+1 throughout with base_exp lowered
to 0.3, at the cost of weaker early variety.

---

## Draft Story Examples

### Story 1: Early Committer (Synergy, Ember-high lane seed)

**Dreamcaller:** Ember + Stone
**Lane seeds:** Ember=1.22, Stone=1.19, Zephyr=1.16, Ruin=0.95, Tide=0.96

This quest had two of the highest lane seeds on the dreamcaller's colors. The
synergy player found strong Ember cards immediately.

- **Picks 1-3:** Three on-color Ember picks. Profile reaches Ember=5, Stone=3
  quickly. The 2+2 open phase offered off-color options in wild slots (Tide,
  Zephyr, and Ruin appeared) but the synergy player preferred on-color value.
- **Pick 4:** A wild slot offered a Stone+Zephyr dual card. The synergy player
  took it for partial fit — a bridge pick enabled by the wild card's consistent
  off-color surfacing.
- **Pick 8:** A late-convergence pack included three on-color Ember/Stone cards
  plus a Tide wild card. Even at pick 8, the wild slot delivered off-color.
- **Pick 10:** Two Ruin cards appeared in the pack (one convergence, one wild).
  The wild slot successfully surfaced off-color at pick 10.

**Final deck:** Ember=24, Stone=19, Zephyr=1. Top-2 share 97.7%.
Convergence pick 4. Classic early committer — the algorithm offered variety
throughout, the player chose convergence.

### Story 2: Lane-Signal Reader (Synergy, Ember-low lane seed)

**Dreamcaller:** Ember + Tide
**Lane seeds:** Ember=0.62 (lowest), Ruin=1.33, Stone=1.24, Tide=1.19, Zephyr=1.09

This run created genuine tension: the dreamcaller pointed toward Ember and Tide,
but the pool had significantly fewer Ember cards than normal (lane seed 0.62).

- **Picks 1-3:** Neutral picks and one Ember. The 2+2 open phase showed Ruin,
  Zephyr, and Stone cards in wild slots — resonances with high lane seeds.
- **Pick 7:** Synergy player took an Ember+Zephyr bridge dual at partial fit —
  a splash pick that expanded the archetype slightly.
- **Picks 8-10:** Tide began dominating convergence slots because Tide had both
  a high lane seed (1.19) and a growing profile. By pick 10 the profile was
  Tide=6, Ember=6, Zephyr=1, Ruin=1.

**Final deck:** Tide=24, Ember=17, Zephyr=1, Ruin=1. Top-2 share 95.3%.
Convergence pick 7. The lane seed effect was visible: Tide (high lane seed)
overtook Ember (low lane seed) as the primary color despite Ember being the
dreamcaller's first resonance. A player reading the pool would have seen this
coming.

### Story 3: Power Chaser (Ruin+Zephyr dreamcaller, mixed lane seeds)

**Dreamcaller:** Ruin + Zephyr
**Lane seeds:** Stone=1.26, Ruin=0.94, Zephyr=0.88, Tide=0.88, Ember=0.60

- **Pick 1:** Took a Ruin+Zephyr card (power=10) from a wild slot — happened
  to be on-color, but power was the reason.
- **Picks 2-3:** Took two Ruin+Tide cards (power=10) — already drifting toward
  Tide despite it being off-color.
- **Pick 4:** Took Stone (power=9) — completely off the dreamcaller archetype.
- **Picks 6-10:** Mixed Ruin, Tide, Stone, and Zephyr picks driven by power.
  The wild card frequently surfaced off-color high-power options, and the power
  chaser took them whenever power was high enough.

**Final deck:** Tide=15, Ruin=15, Stone=9, Zephyr=8, Ember=2. Top-2 share
61.2%. A genuine four-color deck. The combination of wild cards, lane seeds,
and reduced dreamcaller bonus successfully supported a flexible, off-archetype
build without the player ever feeling pushed away from options they wanted.

---

## Comparison to the Other Hybrid Proposals

The five hybrid proposals each take a different position on the
convergence-variety-splash tradeoff. Here is how this proposal (Hybrid 2)
differs from the others:

### Hybrid 1: CRESCENDO Variant

Hybrid 1 stays closest to a pure exponent ramp (Strategy 1's approach). It
achieves strong convergence through a continuous exponent but relies on a
supplementary diversity check — a procedural rule that fires when no off-color
card appears — to guarantee splash. This is less clean than a structural wild
card: the check is a reactive correction rather than a proactive guarantee.

**Choose Hybrid 1 if:** You want maximum simplicity and minimal code change.
The exponent ramp is the most elegant mechanism, and adding a diversity check
is straightforward. Accept that splash comes from a fallback, not from a
designed slot.

**Choose Hybrid 2 over Hybrid 1 if:** You want off-color to appear organically
and reliably throughout the draft, not just when the procedural check fires.
Hybrid 2's late off-color of 0.70 vs a diversity-check approach's floor-level
performance is a meaningful difference.

### Hybrid 3: Lane-Primary Hybrid

Hybrid 3 takes Strategy 3's lane pool architecture as its core, adding
convergence on top. Because it treats the pool environment (not the player
profile) as the primary shaping mechanism, it naturally produces high variety
and strong signal reading. But the fundamental weakness of lane pools — that
drafting from a resonance depletes it, creating negative feedback against
convergence — is hard to fully overcome without essentially replacing the lane
mechanism with profile-based weighting.

**Choose Hybrid 3 if:** Signal reading and run-to-run variety are the top
priorities, and you are comfortable with weaker convergence guarantees.

**Choose Hybrid 2 over Hybrid 3 if:** You want the convergence guarantee to be
robust and independent of pool state. Hybrid 2's convergence cards do not
compete with lane depletion dynamics.

### Hybrid 4: Staged Exponent with Splash Slot

Hybrid 4 is the most directly comparable to Hybrid 2. Both use a continuous
exponent ramp for convergence and add a structural splash mechanism. The key
difference is the exponent range: Hybrid 4 pushes to exponent 2.0 (quadratic),
which produces weight ratios over 150:1 at high profiles. Even with a dedicated
splash slot, the convergence cards become very predictable late. Hybrid 2's
max_exp of 1.4 produces ratios of 51:1 — still strongly convergent but less
deterministic.

**Choose Hybrid 4 if:** You want the strongest possible convergence signal for
synergy players and are comfortable with late-game packs feeling more
deterministic.

**Choose Hybrid 2 over Hybrid 4 if:** You want late packs to still carry some
genuine choice weight among on-color options, and you want the wild card to
remain meaningful rather than being swamped by massive on-color weights.

### Hybrid 5: Adaptive with Structural Splash

Hybrid 5 combines behavioral tracking (watching how decisively the player picks
on-color to modulate the exponent) with a structural splash mechanism. The
behavioral layer is the most sophisticated feature in the entire design space —
the exponent adjusts based on what the player actually does, so explorers
naturally get slower convergence. The cost is implementation complexity and the
"false signal" problem: if a pack happens to offer only one good option and the
player takes it, the system reads that as a commitment signal even though the
player had no real choice.

**Choose Hybrid 5 if:** Differentiating the experience for explorers vs
committed synergy players is a high design priority, and the implementation
complexity is acceptable.

**Choose Hybrid 2 over Hybrid 5 if:** You want a simpler, more neutral system
that does not infer player intent from picks. Hybrid 2 treats all players the
same and relies on structural guarantees rather than behavioral modeling.

### Summary Table

| Property | Hybrid 2 | Hybrid 1 | Hybrid 3 | Hybrid 4 | Hybrid 5 |
|----------|----------|----------|----------|----------|----------|
| Core convergence mechanism | Exponent ramp (0.5-1.4) | Exponent ramp + check | Lane pools + profile | Exponent ramp (0.5-2.0) | Adaptive exponent |
| Splash mechanism | Structural wild card | Diversity check | Lane depth | Splash slot | Structural wild card |
| Run-to-run variety | Lane seeds | Pool variance | Lane seeds + depletion | Pool variance | Pool variance |
| Implementation complexity | Medium | Low | High | Medium | High |
| Synergy late off-color | 0.70 | Lower | Higher | Lower | Lower |
| Power chaser tri-color | 68.8% | Lower | Higher | Moderate | Moderate |
| Open questions | Early variety ceiling | Splash reliability | Convergence ceiling | Late determinism | False signal problem |

---

*Document based on simulation data from 1,000 quests per player type (synergy,
power_chaser, rigid) under hybrid v2 parameters. Parameter sweep data from 200
quests per configuration. Full simulation results and sweep tables in
`results_2_v2.md`; goal scoring methodology in `analysis_2.md`.*
