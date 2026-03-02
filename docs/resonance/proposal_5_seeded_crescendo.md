# Proposal 5: Seeded CRESCENDO (Recommended Strategy)

**Status:** Recommended for implementation
**Design score:** 60/80 (ADDITIVE) or 59/80 (MAX)
**Targets passed:** 15/21 (ADDITIVE) or 14/21 (MAX) — highest of all hybrids tested
**Proposal author:** Agent 5 (originally Strategy 5: Adaptive Resonance)

---

## 1. Overview

Seeded CRESCENDO is the simplest resonance algorithm that addresses all eight design goals,
and it is the final recommendation from the Dreamtides resonance redesign project. The core
idea is straightforward: every quest begins by randomly determining which colors have deeper
card pools and which have shallower ones, giving each run a different set of archetypes that
reward drafting. From there, a single weight formula governs every pack, every pick, the
entire draft. Early packs are nearly open — the formula's floor weight ensures all five
colors are always competitive — and late packs gradually focus on whatever colors the player
has been accumulating, through a simple exponent that grows over time. No slots, no phases,
no behavioral surveillance, no hidden tracking. Just one formula, two mechanisms (pool seeds
and a ramp), and two parameter changes (a high floor weight and a reduced dreamcaller bonus)
that together solve the problems that plagued every individual strategy tested before this
synthesis.

This is the recommended strategy from the final synthesis report because it is the only
design among six hybrids tested to pass the convergence timing target — the single most
distinguishing metric — and it does so while also being the simplest design, achieving the
highest design-goal score, and passing the most measurable targets outright.

---

## 2. How It Works

### The Two Core Mechanisms

**Mechanism 1: Lane Seeds (per quest)**

At the start of each quest, the game rolls a random "lane seed" for each of the five
resonances — Ember, Ruin, Stone, Tide, and Zephyr. The seed is a multiplier between 0.60
and 1.40, drawn independently for each color.

```
seed[r] ~ Uniform(0.60, 1.40)   for each resonance r
```

This multiplier is applied to the copy count of every card of that color in the draft pool.
If the base pool contains 8 copies of a common Ember card:

- With seed 1.35: the pool contains `round(8 × 1.35) = 11` copies
- With seed 0.72: the pool contains `round(8 × 0.72) = 6` copies

A "deep" lane (high seed) means more copies of that color are available, making those cards
appear more frequently — even before accounting for any profile weight. A "shallow" lane
(low seed) means fewer copies, making them rarer throughout the draft.

Lane seeds are set once at quest start and do not change. There is no depletion. Cards you
draft do not remove copies from the pool. The pool is static; only its composition varies.

**Why seeds matter:** Two runs with the same dreamcaller can feel genuinely different. One
run might have a deep Ruin lane and shallow Tide; the next reverses this. The optimal
archetype — the one that is both plentiful in the pool and synergistic with the dreamcaller
— varies per run. A player who notices "I keep seeing Ruin cards" can pivot toward Ruin and
be rewarded with more of them throughout. A player who ignores this and drafts purely to
plan will find the pool fighting them.

**Mechanism 2: Exponent Ramp (per card, per pick)**

Each card's weight is calculated from the player's profile — the running count of how many
cards of each resonance they have drafted. The formula has two variants; both share the same
exponent ramp.

The exponent starts low (sub-linear) and increases linearly to its maximum over the first 12
picks:

```
exponent = base_exp + (max_exp - base_exp) * clamp((pick_number - 1) / (ramp_picks - 1), 0, 1)
```

With the recommended parameters (base_exp=0.5, max_exp=1.1, ramp_picks=12):

| Pick | Exponent |
|------|----------|
| 1    | 0.50     |
| 4    | 0.64     |
| 8    | 0.86     |
| 12   | 1.10     |
| 13+  | 1.10 (capped) |

### The Two Formula Variants

Both variants use the exponent ramp and lane seeds. They differ in how the floor weight is
combined with the profile component.

---

#### ADDITIVE Variant (Recommended)

```
weight[card] = floor_weight + sum(profile[r]^exponent  for r in card.resonances)
```

For a neutral card: `weight = neutral_base` (a fixed constant, not subject to the formula).

For a dual-resonance card (e.g., Ember+Tide): both profile values contribute additively.

The floor weight is *always added* to the profile component. Even if a card's color is at
zero in the profile, it still has weight of exactly `floor_weight`.

**Concrete examples (ADDITIVE, dreamcaller gives Ember:2 and Tide:2 to start):**

| Situation | Pick 1 (exp=0.50) | Pick 5 (exp=0.72) | Pick 10 (exp=0.99) | Pick 20 (exp=1.10) |
|-----------|-------------------|-------------------|--------------------|---------------------|
| Ember card (profile=2, DC start) | 3.5 + 2^0.50 = **4.91** | 3.5 + 4^0.72 = **6.51** | — | — |
| Ember card (profile=8, committed) | — | — | 3.5 + 8^0.99 = **11.06** | 3.5 + 12^1.10 = **17.59** |
| Ruin card (profile=0, no picks) | 3.5 + 0 = **3.50** | 3.5 + 0 = **3.50** | 3.5 + 0 = **3.50** | 3.5 + 0 = **3.50** |
| Neutral card | **4.00** | **4.00** | **4.00** | **4.00** |

The key observation: at pick 1, an Ember card (starter color, count=2) has weight 4.91 and
a Ruin card (never picked, count=0) has weight 3.50. The ratio is only 1.4:1. The colors
are nearly equal. The player faces a real choice.

By pick 20, an Ember card at count=12 has weight 17.59, while an unseen Ruin card still has
weight 3.50. The ratio is 5:1. Strong convergence, but off-color cards remain visible — in a
4-card pack, roughly 1 card is expected to be off-color.

---

#### MAX Variant (Alternative)

```
weight[card] = max(floor_weight, sum(profile[r]^exponent  for r in card.resonances))
```

The floor weight is used as a *minimum* — it applies only when the profile component is
below it. Once a resonance is drafted enough that `profile^exp` exceeds 3.5, the floor
becomes irrelevant and the raw profile weight takes over.

**Concrete examples (MAX, same dreamcaller):**

| Situation | Pick 1 (exp=0.50) | Pick 5 (exp=0.72) | Pick 10 (exp=0.99) | Pick 20 (exp=1.10) |
|-----------|-------------------|-------------------|--------------------|---------------------|
| Ember card (profile=2, DC start) | max(3.5, 2^0.50=1.41) = **3.50** | max(3.5, 4^0.72=2.67) = **3.50** | — | — |
| Ember card (profile=8, committed) | — | — | max(3.5, 8^0.99=7.65) = **7.65** | max(3.5, 12^1.10=16.10) = **16.10** |
| Ruin card (profile=0, never picked) | max(3.5, 0) = **3.50** | max(3.5, 0) = **3.50** | max(3.5, 0) = **3.50** | max(3.5, 0) = **3.50** |
| Neutral card | **4.00** | **4.00** | **4.00** | **4.00** |

In the MAX variant, the dreamcaller bonus (count=2 at pick 1) does not lift cards above the
floor at all for the first several picks. Every color is exactly at the floor weight —
weights of 3.50 all around — until a resonance is drafted enough to break through. This
makes the very earliest picks even more open than the ADDITIVE variant, but reduces early
on-color bias slightly too much, requiring slightly more picks to feel the profile focusing.

---

### How the Floor Weight Creates Implicit Phases

The high floor weight (3.5) is the key innovation of this design. It creates three implicit
behavioral phases without any explicit phase-switching logic:

**Phase 1: Exploration (picks 1–3)**
The exponent is low (0.50–0.61). Even the dreamcaller colors at count=2 produce a profile
component of only 1.41 to 1.59, which is below the floor of 3.50. Every resonance — whether
drafted 0 times or 2 times — sits at or near the floor. Packs feel genuinely open.
Early on-color per pack: **1.33–1.77** (simulation result).

**Phase 2: Gradual Focus (picks 4–8)**
The exponent has risen enough (0.64–0.86) that heavily-drafted resonances begin to exceed
the floor. A player who has drafted Stone 6 times has Stone at weight 3.50 + 4.39 = 7.89
(ADDITIVE) or 4.39 (MAX), while a never-seen color remains at 3.50. The draft begins
focusing, but off-color options are still plentiful.
Convergence (point where top-2 colors dominate) happens here: **pick 5–6** for synergy
players.

**Phase 3: Convergence (picks 9–12 and beyond)**
The exponent has reached its maximum (1.10). A player with 10+ cards in their primary color
has weights of 13–18, compared to the floor of 3.50 for off-color cards. On-color cards
are strongly preferred, but the floor ensures roughly 1 off-color card still appears per
4-card pack. Late on-color per pack: **2.55** (ADDITIVE synergy simulation result).

This is one formula, three behaviors, zero explicit transition points.

---

## 3. Key Parameters

| Parameter | Value | What It Controls | Up → | Down → | Sensitivity |
|-----------|-------|-----------------|------|--------|-------------|
| `dc_bonus` | **2** | Starting profile count for each dreamcaller color | More initial bias; earlier convergence; more on-color from pick 1 | Less initial guidance; draft feels more random early | **HIGH** (game design decision) |
| `floor_weight` | **3.5** | Minimum weight for any resonance card | More off-color cards in every pack; later convergence | Fewer off-color cards; earlier and stronger convergence | **HIGH** |
| `max_exp` | **1.1** | How strongly the profile dominates in late picks | Stronger late-game focus; less splash | Weaker late focus; more tri-color outcomes | **HIGH** |
| `base_exp` | **0.5** | Exponent at pick 1 | More on-color early; less open start | More even early packs (currently at a good point) | MODERATE |
| `ramp_picks` | **12** | Over how many picks the exponent ramps to maximum | Slower ramp; exponent stays low longer | Faster ramp; sharper early focus transition | LOW |
| `neutral_base` | **4.0** | Weight for neutral (color-free) cards | Neutrals appear more often; less color identity | Neutrals appear less; more resonance in every pack | LOW |
| `seed_min` / `seed_max` | **0.60 / 1.40** | Range of per-quest pool multipliers | Wider variance between runs; deeper/shallower lanes | Narrower variance; more similar runs | LOW |

**The high floor weight (3.5) is the single most important parameter.** In the original
pre-redesign system, the floor was 0.5. At that level, off-color cards had weight 0.5 while
on-color cards at pick 10 had weight 15–50, a ratio of 30:1 to 100:1. Off-color cards were
effectively invisible. At floor=3.5, the worst-case ratio is about 5:1, and off-color cards
appear in almost every pack throughout the draft.

**Tuning guidance by sensitivity level:**
- If convergence feels too slow or decks are too scattered: increase `max_exp` (try 1.2–1.3)
- If off-color splash feels insufficient: increase `floor_weight` (try 4.0–4.5)
- If the draft feels too guided from pick 1: decrease `dc_bonus` to 1
- If runs feel too similar: widen the seed range (try 0.50–1.50)

---

## 4. What Changed From the Original Strategy: The Abandonment of Adaptive Resonance

This proposal has the most dramatic evolution of any hybrid in the redesign project. The
original S5 strategy — Adaptive Resonance — was not refined into the final hybrid. It was
abandoned entirely, and the final design was built from scratch.

### What the Original S5 Proposed

Adaptive Resonance (original Strategy 5) tracked player *behavior*, not just deck
composition. The idea was: the system already knows what you have drafted (the resonance
profile). But does it know *how* you drafted it? A player who takes the highest-weight card
from every pack is committing decisively. A player who consistently takes low-weight cards is
exploring or power-chasing. These should produce different pack experiences.

S5 implemented this with a "commitment score" — a rolling exponential moving average over
the last 5 picks measuring how close each pick was to the maximum-weight option in that
pack. A decisive player (high commitment score) got an accelerated convergence multiplier,
making the exponent ramp faster. An exploring player (low commitment score) got a slowed
multiplier, keeping the draft open longer. Additionally, a "freshness bonus" of 0.8 was
added to cards representing resonances not seen in the last 3 packs, to keep variety visible.

This required: a rolling pick history, an EMA computation, a threshold-to-multiplier mapping
function, an early gate for picks 1–5, and 8 new tunable constants — all on top of the base
algorithm.

### Why Simulation Killed It

The parameter sweep of the original S5 measured the impact of every behavioral tracking
constant across its full range:

- `slow_min` (exploratory multiplier): < 0.1 change on every metric across full range
- `fast_max` (decisive multiplier): < 0.1 change on every metric across full range
- `freshness_bonus`: +0.14 early unique resonances, +0.04 late off-color (negligible)

The commitment-pace EMA was tracking a signal that was largely redundant with the profile
itself. A player who picks on-color cards accumulates a concentrated profile; a player who
picks off-color cards has a scattered profile. The profile *already encodes* the behavioral
difference — the commitment score added a second, noisier encoding of the same information.

The freshness bonus suffered from a design contradiction: it was meant to compensate for
high convergence, but the decisive-player path *accelerated* the exponent — making on-color
weights reach 40–50, against which a +0.8 bonus is less than 2% of on-color weight. The
mechanism to counteract convergence was undermined by the convergence it was supposed to
counteract.

Additionally, the original S5's draft traces showed the same problem as every other Round 3
strategy: 100% on-color packs from pick 1, essentially no off-color cards, and synergy
players converging by pick 4. The behavioral tracking improved none of this because the
underlying issue was the dreamcaller bonus of 4, not the algorithm.

### What Agent 5 Did Next

Faced with simulation evidence that the original proposal was effectively zero-impact, Agent
5 wrote in the hybrid proposal:

> "My original strategy (Adaptive Resonance) is empirically dead. Behavioral tracking via
> commitment-pace EMA has negligible impact on any metric... Starting from scratch, I
> designed this hybrid by asking: what is the simplest system that addresses all 8 design
> goals?"

The answer was to take the two mechanisms that analysis showed actually mattered and combine
them cleanly:

1. **S1's temporal exponent ramp** — the core convergence mechanism, proven to work, 9/10
   on simplicity
2. **S3's lane seeds** — the only mechanism providing run-to-run variety and signal reading

Then apply two parameter changes that address all remaining failures:

3. **Floor weight of 3.5** — transforms off-color visibility from 0.05 to 0.82–0.93 per
   late pack
4. **DC bonus of 2 instead of 4** — halves the initial weight asymmetry, fixing early
   openness and convergence timing

Every behavioral tracking mechanism, every special parameter from the original S5, was
removed. The result is a system with 6 parameters and one formula — and it outperforms
every other design by the measurable targets.

---

## 5. Simulation Results

All results from 1000-quest simulations with seed 12345.

### ADDITIVE Variant — Full Target Table (15/21 targets passed)

| Metric | Target | Synergy | Power Chaser | Rigid | Result |
|--------|--------|---------|--------------|-------|--------|
| Early unique res/pack | >= 3.0 | 2.76 | 2.79 | 2.75 | **FAIL** (all 3) |
| Early on-color/pack | <= 2.0 | 1.77 | 1.69 | 1.77 | **PASS** (all 3) |
| Late on-color/pack | >= 2.0 | 2.55 | 2.04 | 2.57 | **PASS** (all 3) |
| Late off-color/pack | >= 0.5 | 0.82 | 1.34 | 0.79 | **PASS** (all 3) |
| Final top-2 share | Syn: 75–93%, Pwr: 60–85%, Rig: 75–95% | 95.4% | 61.1% | 98.0% | **FAIL** syn+rig, **PASS** pwr |
| Convergence pick | 5.0–8.0 | 5.0 | 15.7 | 5.2 | **PASS** syn+rig, **FAIL** pwr (by design) |
| Pair frequency range | 5–15% each pair | 8.8–10.9% | 8.7–11.1% | 8.8–10.9% | **PASS** (all 3) |

**Archetype distribution (ADDITIVE):**
- Synergy: 99.5% dual, 0.5% mono, 0.0% tri
- Power chaser: 20.2% dual, 79.8% tri
- Rigid: 99.4% dual, 0.6% mono, 0.0% tri

### MAX Variant — Full Target Table (14/21 targets passed)

| Metric | Target | Synergy | Power Chaser | Rigid | Result |
|--------|--------|---------|--------------|-------|--------|
| Early unique res/pack | >= 3.0 | 2.66 | 2.66 | 2.66 | **FAIL** (all 3) |
| Early on-color/pack | <= 2.0 | 1.33 | 1.35 | 1.33 | **PASS** (all 3) |
| Late on-color/pack | >= 2.0 | 2.34 | 1.97 | 2.32 | **PASS** syn+rig, **NEAR** pwr |
| Late off-color/pack | >= 0.5 | 0.93 | 1.24 | 0.95 | **PASS** (all 3) |
| Final top-2 share | Syn: 75–93%, Pwr: 60–85%, Rig: 75–95% | 94.5% | 61.1% | 97.0% | **FAIL** syn+rig, **PASS** pwr |
| Convergence pick | 5.0–8.0 | 5.8 | 17.7 | 6.1 | **PASS** syn+rig, **FAIL** pwr (by design) |
| Pair frequency range | 5–15% each pair | 8.8–11.0% | 8.2–12.0% | 8.9–10.9% | **PASS** (all 3) |

**Archetype distribution (MAX):**
- Synergy: 99.2% dual, 0.8% mono, 0.0% tri
- Power chaser: 22.7% dual, 77.3% tri
- Rigid: 98.4% dual, 1.6% mono, 0.0% tri

### Variant Comparison Summary

| Metric | MAX | ADDITIVE | Preferred |
|--------|-----|----------|-----------|
| Targets passed | 14/21 | **15/21** | ADDITIVE |
| Early on-color (synergy) | **1.33** | 1.77 | MAX (more open) |
| Early unique res (synergy) | 2.66 | **2.76** | ADDITIVE (more variety) |
| Late on-color (synergy) | 2.34 | **2.55** | ADDITIVE (stronger convergence) |
| Late off-color (synergy) | **0.93** | 0.82 | MAX (more splash) |
| Convergence pick (synergy) | 5.8 | **5.0** | ADDITIVE (hits target exactly) |
| Top-2 share (synergy) | 94.5% | 95.4% | Neither (both fail) |
| Formula complexity | Slightly simpler | Slightly more complex | MAX |
| Lane signal in late draft | Pool composition only | **Weight-level** | ADDITIVE |
| Design goal score | 59/80 | **60/80** | ADDITIVE |

### Improvement Over Round 3 (Original S5 — Adaptive Resonance)

| Metric | Original S5 (Round 3) | H5-ADDITIVE | Target |
|--------|----------------------|-------------|--------|
| Early unique res | 2.51 | 2.76 | >= 3.0 |
| Early on-color | 2.85 | **1.77** | <= 2.0 |
| Late on-color | 3.57 | **2.55** | >= 2.0 |
| Late off-color | 0.08 | **0.82** | >= 0.5 |
| Convergence pick | 4.3 | **5.0** | 5.0–8.0 |
| Top-2 share (synergy) | 97.6% | 95.4% | 75–93% |
| Targets passed | 8/21 | **15/21** | — |

The additive variant nearly doubles the target pass rate versus the original proposal.

---

## 6. Design Goal Scorecard

Scored 1–10, where 1–2 = fundamentally broken, 5–6 = acceptable, 9–10 = excellent.

### ADDITIVE Variant (60/80 total)

| Goal | Score | Evidence and Justification |
|------|-------|---------------------------|
| **1. Simple** — explainable in one sentence | **8/10** | One formula: `sum(profile[r]^exp) + floor`. One ramp: exponent grows linearly from 0.5 to 1.1 over 12 picks. One pool mod: lane seeds at quest start. No slots, no phases, no behavioral tracking. The only complexity versus pure CRESCENDO is lane seeds, which are conceptually simple ("some colors have more cards this run"). |
| **2. Not on Rails** — genuine choices per pack | **8/10** | Early on-color: 1.77/pack (passes <= 2.0). Floor weight ensures all 5 resonances are competitive early. Players face 3–4 way choices in early packs, not "pick from among 4 Tide cards." Reduced DC bonus (2, not 4) is what enables this. |
| **3. No Forced Decks** — can't force the same deck every run | **8/10** | Deck overlap: 0.388 cosine similarity (low). All 10 resonance pairs appear at 8.8–10.9% frequency (well within 5–15% target). Lane seeds ensure different optimal archetypes each run. |
| **4. Flexible Archetypes** — can build outside the core pairs | **8/10** | Power chasers achieve 79.8% tri-color decks. Floor of 3.5 ensures off-color cards are always visible and viable. Late off-color: 1.34/pack for power chasers. Tri-color and splash builds are genuinely supported. |
| **5. Convergent** — committed players see 2+ on-color cards per pack | **8/10** | Late on-color: 2.55/pack (synergy), 2.57/pack (rigid). Convergence pick: 5.0 for synergy (only variant to hit the 5–8 target). The additive formula's exponent arc feels like natural increasing focus. |
| **6. Splashable** — ~1 off-color card per late pack | **8/10** | Late off-color: 0.82/pack (synergy), 1.34/pack (power chaser). Before redesign this was 0.08. Floor of 3.5 guarantees off-color cards always have meaningful weight even at maximum convergence. |
| **7. Open Early** — early picks show variety | **5/10** | Early unique res: 2.76/pack (fails >= 3.0 target). However, this target is likely structurally unreachable: with a 4-card pack, 5 resonances plus neutral, and any profile-based weighting at all, achieving 3.0 unique resonances requires an extremely flat distribution. Revised target of >= 2.7 would pass. |
| **8. Signal Reading** — pool composition is readable and rewarding | **7/10** | Lane seeds create genuine pool asymmetries detectable by attentive players. The additive formula preserves lane influence throughout the draft (floor weight always added, so lane-seeded copy counts affect selection all the way to pick 20+). Stronger than any profile-only approach but weaker than pure lane-pool strategies (S3). |

### MAX Variant (59/80 total)

| Goal | Score | Evidence and Justification |
|------|-------|---------------------------|
| **1. Simple** | **9/10** | One formula: `max(floor, sum)`. Marginally simpler than ADDITIVE — the floor is a threshold, not always added. |
| **2. Not on Rails** | **8/10** | Early on-color: 1.33/pack (even more open than ADDITIVE). Floor completely dominates DC colors in picks 1–3. |
| **3. No Forced Decks** | **8/10** | Overlap: 0.380 cosine. Pair range: 8.8–11.0%. Equivalent to ADDITIVE. |
| **4. Flexible Archetypes** | **8/10** | Power chaser tri-color: 77.3%. Off-color: 0.93/pack. Slightly less flexible than ADDITIVE (floor only active when profile^exp < 3.5). |
| **5. Convergent** | **7/10** | Late on-color: 2.34/pack (passes >= 2.0). Convergence pick: 5.8 (inside target but slower). Max formula's threshold means off-color cards drop below the floor once profile^exp > 3.5, reducing the effective off-color weight at maximum convergence. |
| **6. Splashable** | **8/10** | Late off-color: 0.93/pack — actually better than ADDITIVE because the floor acts as a hard floor regardless of profile. |
| **7. Open Early** | **5/10** | Early unique res: 2.66/pack. DC colors at count=2 don't exceed the floor until exp ~0.8, so picks 1–4 are maximally open (all at 3.50). However, this also means slightly less early color variety than ADDITIVE because DC colors don't even get a small advantage until mid-draft. |
| **8. Signal Reading** | **6/10** | Lane seeds still provide pool asymmetries, but the MAX formula means off-color cards (once they exceed the floor) are weighted purely by profile, not by lane depth. Lane influence is most visible in picks 1–5 (floor phase) and then fades as profile^exp takes over. |

---

## 7. Strengths

**Simplest successful design among all hybrids tested.**
Six parameters, one formula, no slot types, no phase logic, no behavioral tracking. Every
card in every pack is weighted by the same calculation. Implementation is 30–50 lines of
straightforward code.

**Most targets passed: 15/21 (ADDITIVE) or 14/21 (MAX).**
The next best hybrid scored 14 (H1, by a different count) or 13 (H4). Seeded CRESCENDO wins
on the combined measure of simplicity and target compliance.

**Only strategy to pass convergence timing for synergy players.**
Mean convergence pick of 5.0 (ADDITIVE) or 5.8 (MAX), against a target of 5–8. All other
hybrids converge at 4.2–4.8, meaning they focus the draft *before* the player has had
enough picks to make meaningful choices. This is the single most distinguishing metric in
the final comparison, and Seeded CRESCENDO is the only design that gets it right.

**Off-color splash transformed.**
Pre-redesign late off-color: 0.08 cards/pack (essentially zero). Seeded CRESCENDO late
off-color: 0.82–0.93 cards/pack (synergy). A player who commits fully to two colors in late
draft still sees roughly one off-color card per pack — a real splash option in every choice.

**No design-goal score below 5/10.**
Every other strategy tested (both individual strategies and other hybrids) has at least one
design goal where it scores 1–4: S1 scores 1/10 on splashability, S5 scores 2/10, S3 scores
3/10 on convergence. Seeded CRESCENDO is the only design with a balanced profile — no score
lower than 5, most scores at 7–8.

**Pair frequency perfectly distributed.**
All 10 possible resonance pairs appear at 8.8–11.0% frequency (target: 5–15%). No pair is
over-represented, no pair is starved. Decks genuinely feel different from run to run.

**Eliminates dead parameters.**
The redesign identified several parameters in the existing codebase that have negligible
impact on any metric (< 0.1 change across full range): `staleness_factor`, `slow_min`,
`fast_max`, `freshness_bonus`. These can all be removed, reducing the parameter space the
team needs to reason about.

---

## 8. Weaknesses

**Early unique resonances falls short: 2.76/pack (target >= 3.0).**
The final report recommends revising this target to >= 2.7, which H5-ADDITIVE passes. The
structural argument: with 4 cards per pack drawn from 5 resonances plus neutral, and any
profile-based weighting that concentrates some resonances, achieving 3.0 unique resonances
is effectively impossible. Every hybrid tested falls between 2.66 and 2.96. The best
possible result would require essentially uniform weighting — which defeats the purpose of
having a convergence algorithm.

**Synergy top-2 share: 95.4% (target 75–93%).**
No algorithm can solve this within the current pack-of-4 structure. A simulated synergy
player who always picks the highest fit-times-power card will naturally build a 90–96%
dual-color deck, because the dreamcaller seeds the profile toward two colors, and the player
rewards that seeding by consistently picking from those colors. Real human players — who
sometimes take powerful off-color cards — will naturally see lower top-2 share. The final
report recommends revising the target ceiling to 96% to reflect structural reality.

**Power chaser convergence: 15.7 picks (target 5–8).**
Power chasers never converge by design — they pick purely by card power regardless of
resonance fit, so their profile never concentrates. The 5–8 target was intended for players
who are actively building a resonance identity. The final report recommends applying the
convergence target only to synergy and rigid player strategies, treating power chaser
non-convergence as correct behavior.

**Power chaser late on-color: 2.04/pack (ADDITIVE), 1.97/pack (MAX).**
Just barely passes or just barely fails the >= 2.0 target, depending on variant. Power
chasers see approximately 2 on-color and 2 off-color cards per late pack — which is actually
the intended experience for a player who deliberately ignores resonance — but it means one
target is very sensitive at the boundary. Increasing max_exp to 1.2 would push this to
approximately 2.2/pack for power chasers without significantly affecting synergy or rigid
player outcomes.

**No structural guarantee of off-color visibility.**
The floor weight provides a probabilistic guarantee: off-color cards always have meaningful
weight, and they will appear in roughly 0.8–0.9 out of every 4 cards on average. But unlike
a "wild card slot" approach (proposed by H2), this is not a hard guarantee. A run of bad
luck could produce several consecutive packs without an off-color card. If this is a
concern, adding a soft diversity check — "if no card outside the top-2 resonances is offered
in a pack, resample one card" — would address it with minimal complexity, and the existing
codebase already has a diversity check that could be adapted.

---

## 9. Draft Story Examples

All examples below use Dreamcaller Ruin+Stone (dc_bonus=2 each, so starting profile
Ruin:2, Stone:2), with lane seeds Ember=1.39, Ruin=1.26, Stone=1.16, Tide=0.68,
Zephyr=1.11. This run has deep Ember and shallow Tide — if the player notices and has
freedom to pivot, Ember is rewarding. However, the dreamcaller is pointing toward Stone and
Ruin.

Notation: `pow` is the card's base power rating (1–10); `w` is the selection weight. `<--`
marks the picked card.

### Example A: Early Committer (Rigid Player)

This player picks the highest-fit card available, reflecting a player who commits to their
dreamcaller colors immediately.

**MAX variant, picks 3–12:**

```
Pick  3 (exp=0.61, profile Stone:8 + Ruin:4):
  Stone  pow=10  w= 3.50 <--   [floor dominates — same weight as off-color]
  Stone  pow= 7  w= 3.50
  Neutral pow=10 w= 4.00

Pick  5 (exp=0.72, profile Stone:10 + Ruin:4):
  Neutral pow=10  w= 4.00
  Ruin+Stone pow=1 w= 7.55 <-- [dual card — sums both resonances, breaks floor]
  Stone   pow= 5  w= 4.85

Pick  8 (exp=0.88, profile Stone:12 + Ruin:5):
  Stone  pow=10  w= 8.29 <--   [profile^exp now dominates the floor]
  Tide   pow= 1  w= 3.50       [off-color still shows up at floor weight]
  Neutral pow=5  w= 4.00
  Zephyr pow= 3  w= 3.50

Pick 12 (exp=1.10, profile Stone:15 + Ruin:5):
  Stone  pow= 7  w=18.23 <--   [5x the floor — strongly focused]
  Stone+Tide pow=3 w=18.23     [splash option visible but losing]
  Ember+Zephyr pow=10 w= 3.50  [off-color, powerful card, but floor weight only]
```

**Final deck:** Stone 70%, Ruin 25%, Ember 5%. Top-2 share: 95.0%. Classification: dual.

Notice: at pick 12, a power-10 Ember+Zephyr card has weight 3.50, while a Stone card at
weight 18.23 is in the pack. A rigid player ignores it; a power chaser would weigh whether
18/3.5 = 5:1 odds justify the sacrifice. The off-color card is visible and the decision is
real — it's just not winning.

### Example B: Late Committer (Synergy Player, Balanced Weights)

This player picks cards based on a balance of resonance fit and power, occasionally taking a
neutral or off-color card when the power differential is large enough.

**MAX variant, picks 5–12:**

```
Pick  5 (exp=0.72, profile Stone:9 + Ruin:4):
  Neutral pow=10  w= 4.00 <--  [power chasing: neutral beats off-color at this weight]
  Ruin+Stone pow=1 w= 7.55
  Stone  pow= 5   w= 4.85

Pick 11 (exp=1.05, profile Stone:14 + Ruin:4):
  Ember  pow=10   w= 3.50 <--  [power chasing: all off-color at floor, picks highest power]
  Zephyr pow= 1   w= 3.50
  Zephyr pow=10   w= 3.50
  Ember  pow= 5   w= 3.50

Pick 12 (exp=1.10, profile Stone:15 + Ruin:4):
  Stone  pow= 7   w=18.23 <--  [back to Stone — high enough weight to pull synergy player]
  Stone+Tide pow=5 w=19.23
  Ember+Zephyr pow=10 w= 3.50
```

**Final deck:** Stone 69%, Ruin 26%, Ember 3%, Tide 3%. Top-2 share: 94.9%.

Pick 11 demonstrates the floor working as intended: when the profile component is below
3.50 (as all off-color resonances are at this point), all off-color cards have the same
weight, and the player has a genuine choice among them based on power alone. They're not
locked into their color — they're just choosing which splash to take.

### Example C: Power Chaser (Always Picks Highest Power)

This player picks the card with the highest power rating, regardless of resonance fit. The
profile accumulates wherever the power happens to be.

**ADDITIVE variant, picks 3–20:**

```
Pick  3 (exp=0.61, profile Stone:9 + Ruin:5):
  Stone  pow=10  w= 7.05 <--   [picks power-10 Stone — happens to be on-color]
  Stone  pow= 7  w= 7.05
  Neutral pow=10 w= 4.00

Pick  7 (exp=0.83, profile Stone:12 + Ruin:5):
  Tide  pow= 8   w= 3.50 <--   [picks power-8 Tide — off-color! floor weight wins]
  Stone pow= 8   w=11.31        [Stone is better-weighted, but same power rating]
  Neutral pow=6  w= 4.00

Pick 11 (exp=1.05, profile Stone:12 + Ruin:5):
  Ember pow= 3   w= 6.65
  Ember pow= 2   w= 6.65
  Zephyr pow= 8  w= 4.50 <--   [picks power-8 Zephyr — off-color at floor+lane bonus]
  Tide  pow= 2   w= 5.56

Pick 20 (exp=1.10, profile Stone:17 + Ruin:11):
  Stone pow= 7   w=24.61
  Ember+Stone pow=9 w=29.21 <-- [picks power-9 dual card — highest power AND high weight]
  Stone+Tide pow=3 w=27.96
```

**Final deck:** Stone 37%, Ruin 29%, Ember 17%, Tide 10%, Zephyr 6%. Top-2 share: 66.7%.
Classification: tri.

The power chaser ends up with a tri-color deck driven by power picks. Note that by pick 20,
even this scattered player's profile has enough Stone concentration (count=17) that Stone
cards are weighted 24+ — but Ember+Stone dual cards with high power beat them. The floor
ensures every resonance the power chaser occasionally drafts remains visible.

---

## 10. Comparison to Alternative Hybrids

Six hybrid strategies were simulated in the final round. Here is how Seeded CRESCENDO
(H5) compares to each.

### Comparison Table

| Hybrid | Key Formula | DC Bonus | Targets | Design Score | Convergence Pick | Late Off-Color |
|--------|------------|----------|---------|-------------|-----------------|----------------|
| **H5-ADD (Recommended)** | sum(profile^exp) + floor | 2 | **15/21** | **60/80** | **5.0** | 0.82 |
| H5-MAX | max(floor, sum(profile^exp)) | 2 | 14/21 | 59/80 | 5.8 | **0.93** |
| H1: CRESCENDO-LANES | sum(profile^exp) + floor + lane_weight | 2 | ~12/21 | 58/80 | 4.5 | 0.61 |
| H3: Lane-Seeded Crescendo | sum(profile^exp) + floor (staged ramp) | 4 per-res | ~14/21 | 58/80 | 4.2 | 0.55 |
| H4: Lane-Seeded Staged Exp | sum(lane_base * profile^exp) | 2 + boost | 13/21 | 58/80 | 4.6 | 0.78 |
| H2: Seeded Ramp 3+1 | 3 cards convergence + 1 wild card slot | 2 | 18/24* | 52/80 | 4.8 | 0.70 |

*H2 used additional custom metrics not shared with other hybrids, making its raw count not
directly comparable.

### Why H5 Beats the Tied Tier (H1, H3, H4)

H1, H3, and H4 all score 58/80 on design goals but all fail the convergence timing target
(picks 4.2–4.5). They use variations of the same mechanisms as H5 but with configurations
that front-load the profile weight too aggressively:

- **H1** adds a separate `lane_weight` term on top of the formula (additive lane influence,
  good) but its continuous lane term also increases the effective early weight, pushing
  convergence earlier.
- **H3** correctly uses the additive formula but retained dc_bonus=4 per resonance (instead
  of reducing to 2), which causes the same early on-color dominance problem that plagued
  all Round 3 strategies. This is why H3 fails early on-color (2.33 vs target <= 2.0).
- **H4** uses a "lane-base exponent" approach that incorporates lane depth into the
  exponent calculation, adding complexity for marginal improvement.

H5's distinctive contribution is the recognition that **the floor weight and the DC bonus
are the two levers that control convergence timing**, not the formula structure. All three
tied designs have formulas that are nearly equivalent to H5 but miss the parameter insight.

### Why H5 Beats H2 (Higher Raw Targets But Lower Design Score)

H2 (Seeded Ramp + Structural Splash) claims 18 targets passed using a custom expanded
metric set. But its design-goal score is the lowest of all hybrids (52/80), and on the
shared standard 21-target evaluation, it passes 14 — one fewer than H5-ADDITIVE.

H2's structural approach — reserving 1 of 4 cards per pack as a "wild card" slot with
flattened weighting — was theoretically appealing but empirically inferior to a high floor
weight. A single floor parameter of 3.5 achieves 0.82 off-color per late pack; the wild
card slot achieves 0.70. The simpler approach wins.

H2 also fails convergence timing (4.8 picks) and achieves the highest synergy top-2 share
(96.7% — the worst result on this metric). The structural partitioning adds complexity that
the parametric approach matches or beats without the overhead.

The final report conclusion: "a single floor_weight parameter achieves comparable splash" to
the slot structure, "The simpler approach wins."

---

## 11. ADDITIVE vs MAX: A Dedicated Comparison

The two variants share all parameters and the lane seed mechanism. The difference is in one
line of the weight formula.

### When Does the Difference Matter?

**Early draft (picks 1–3):**
- MAX: DC colors (count=2) compute `2^0.50 = 1.41`, which is below the floor of 3.50.
  Every resonance sits exactly at 3.50. Maximum openness.
- ADDITIVE: DC colors compute `3.50 + 1.41 = 4.91` while off-color sits at `3.50 + 0 =
  3.50`. Slight early bias toward DC colors (ratio 1.4:1).

**Mid draft (picks 5–8, profile count 4–8):**
- MAX: The threshold question — at what count does profile^exp exceed 3.5? At exp=0.72
  (pick 5): `4^0.72 = 2.67 < 3.50`, still at floor. At exp=0.86 (pick 8): `6^0.86 = 4.81
  > 3.50`, floor no longer applies. The floor acts as a gate.
- ADDITIVE: The floor is always added. At pick 5, count=6: weight = `3.50 + 4.39 = 7.89`.
  At pick 5, count=0 (off-color): weight = `3.50 + 0 = 3.50`. Ratio is 2.3:1, which is
  meaningful but not crushing.

**Late draft (picks 12+, profile count 10+):**
- MAX: `10^1.10 = 12.59`. Floor of 3.50 is far below. Ratio on:off = 3.6:1.
- ADDITIVE: `3.50 + 10^1.10 = 3.50 + 12.59 = 16.09` for on-color vs `3.50 + 0 = 3.50`
  for off-color. Ratio = 4.6:1.

The ADDITIVE variant has a higher late-game on:off ratio because the floor is *added* to a
high profile component, increasing the gap. This produces stronger convergence (2.55 vs 2.34
late on-color) but also slightly less splash (0.82 vs 0.93 off-color/pack).

### Signal Reading Advantage of ADDITIVE

In the ADDITIVE formula, the floor weight acts as a baseline that is always present. Since
lane-seeded cards have more copies in the pool, they appear more frequently even when their
weight is at the floor minimum. This means the lane seed's influence on card selection
probability operates through both channels — pool composition AND weight — throughout the
entire draft.

In the MAX formula, once the profile^exp component exceeds the floor for the player's main
colors, the floor becomes irrelevant to those cards. Off-color cards at the floor are still
affected by lane seeds (more copies = more likely to appear), but the player's on-color
cards are weighted purely by profile, not by lane depth. The lane seed's influence on
on-color cards diminishes as the draft progresses.

This is why the ADDITIVE variant scores 7/10 on signal reading vs 6/10 for MAX.

### Recommendation: ADDITIVE

The ADDITIVE variant is recommended for implementation because:

1. It passes one more target than MAX (15 vs 14), specifically the power chaser late on-color
   target (2.04 vs 1.97)
2. It has a better design goal score (60 vs 59)
3. It achieves better convergence timing (pick 5.0 vs 5.8 — both inside target, but ADDITIVE
   hits the center while MAX is near the edge)
4. It preserves lane signal reading throughout the draft, supporting Goal 8

The only metric where MAX is clearly better is early on-color (1.33 vs 1.77). If early
openness is the highest design priority and the 1.77 value feels like too much early
on-color in playtesting, MAX would be the correct choice. Otherwise, ADDITIVE is preferred.

---

## 12. Implementation Notes

### What Changes From the Current System

The current resonance system uses:
- `dc_bonus = 4` per dreamcaller resonance (change to **2**)
- `affinity_exponent` = static constant (change to **linear ramp** from 0.5 to 1.1 over 12
  picks)
- `floor_weight` = 0.5 (change to **3.5**)
- Staleness factor (remove)

The current system does not have:
- Lane seeds (add)

### Parameters to Remove

These parameters have been confirmed negligible by simulation (< 0.1 change across full
range) and should be removed from the codebase:

- `staleness_factor` and the staleness tracking mechanism (zero impact across all strategies)
- `commitment_score` / `convergence_multiplier` (S5-specific, empirically zero impact)
- `freshness_bonus` (S5-specific, negligible at high exponents)
- `slow_min` / `fast_max` (S5-specific, < 0.1 impact)

### Core Algorithm (Pseudocode)

```python
# Quest initialization (run once per quest)
def initialize_quest(dreamcaller, rng):
    lane_seeds = {}
    for resonance in ALL_RESONANCES:
        lane_seeds[resonance] = rng.uniform(0.60, 1.40)

    # Apply seeds to pool copy counts
    for card in pool:
        if card.resonances:
            avg_seed = mean(lane_seeds[r] for r in card.resonances)
            card.effective_copies = round(card.base_copies * avg_seed)
        # Neutral cards: effective_copies = base_copies (unaffected)

    # Initialize profile from dreamcaller
    profile = defaultdict(int)
    profile[dreamcaller.resonance_1] = 2   # dc_bonus
    profile[dreamcaller.resonance_2] = 2   # dc_bonus

    return lane_seeds, profile


# Per-card weight calculation
def compute_weight(card, profile, pick_number):
    if card.is_neutral:
        return 4.0  # neutral_base

    t = clamp((pick_number - 1) / (12 - 1), 0.0, 1.0)
    exp = 0.5 + (1.1 - 0.5) * t  # ramp from base_exp=0.5 to max_exp=1.1

    # ADDITIVE variant:
    return 3.5 + sum(profile[r] ** exp for r in card.resonances)

    # MAX variant (alternative):
    # return max(3.5, sum(profile[r] ** exp for r in card.resonances))


# Profile update after each pick
def on_draft(card, profile):
    for r in card.resonances:
        profile[r] += 1
```

### Notes on Dual-Resonance Cards

Dual-resonance cards (e.g., Ember+Stone) sum both profile contributions:

```
weight(Ember+Stone) = floor + profile[Ember]^exp + profile[Stone]^exp
```

This naturally makes dual cards more attractive to players who have drafted both resonances
— if you have Ember:6 and Stone:6, an Ember+Stone dual has weight `3.5 + 4.39 + 4.39 = 12.28`
compared to a single-color Ember card at `3.5 + 4.39 = 7.89`. Dual cards serve as
bridge rewards for committed dual-color players.

### Prerequisite: Dual Dreamcallers

All strategies in the redesign produce degenerate results with mono-color dreamcallers. The
original S5 simulation showed that a mono dreamcaller with a synergy player produces 98.6%
mono decks with essentially zero off-color cards — the algorithm cannot compensate for a
starting profile of one resonance at count=4 versus all others at count=0. This is a game
design constraint, not an algorithm constraint: all dreamcallers should have exactly two
resonances.

If three-color dreamcallers are introduced, dc_bonus should be reduced to 1 per resonance
(total DC profile = 3) to avoid the same asymmetry problem at a higher count.

### Open Questions for Playtesting

1. **Is floor_weight=3.5 the right value?** The sweep shows [3.0, 4.0] produces good
   results. The exact value controls the convergence-splash balance and should be validated
   with human drafters. If late packs feel too scattered, try 3.0; if too focused, try 4.0.

2. **Should lane seeds be visible?** Explicitly showing players which colors are deep ("Ruin
   is abundant this quest") would strengthen Goal 8 (signal reading) but might reduce the
   discovery feeling. Consider revealing lane depths after the first 5 picks, or in a
   post-quest summary.

3. **Does real player behavior match the synergy simulation?** Human players sometimes take
   powerful off-color cards that the simulated synergy player ignores. This may naturally
   reduce top-2 share below 95% without any algorithm change.

4. **How do shops interact?** Shops offer 6 cards and allow multiple purchases. The
   simulation uses a pack-of-4 model. Shop behavior may change convergence timing,
   especially for power chasers who buy across resonances.
