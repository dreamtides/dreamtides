# Resonance Algorithm Investigation: Consolidated Report

This document summarizes seven iterations (V1–V7) of investigation into the
resonance draft algorithm for Dreamtides. Each iteration explored multiple
candidate algorithms via simulation, debate, and analysis before converging on a
recommendation. Later iterations built on — and identified weaknesses in —
earlier recommendations.

______________________________________________________________________

## V1: Seeded CRESCENDO (H5-ADDITIVE)

### Algorithm

A single weighted-sampling formula with per-quest lane seeds. At quest start,
each of 5 resonances receives a random multiplier (0.60–1.40) that scales card
copy counts in the pool. During drafting, each card's sampling weight is
computed as:

```
weight = floor_weight + sum(profile[r]^exponent for r in card.resonances)
exponent = base_exp + (max_exp - base_exp) * clamp((pick - 1) / (ramp_picks - 1), 0, 1)
```

The player's resonance profile starts from their dreamcaller's resonances and
grows with each pick. The floor weight (3.5) ensures off-color cards always have
a chance of appearing. The exponent ramps from 0.5 to 1.1 over 12 picks,
creating implicit exploration/focus/convergence phases from a single formula.

### Benefits

- Simplest viable system: one formula, 6–8 parameters, no phase logic
- Best target compliance of any V1 variant (15/21 targets, 71%)
- Only V1 design with correct convergence timing (pick 5.0)
- Strong off-color splash (0.82 cards/pack vs 0.05 pre-redesign)
- Run-to-run variety via lane seeds (only 6% deck overlap between runs)
- Clean parameter orthogonality — each knob controls one thing

### Weaknesses

- Early unique resonances (2.76) falls short of 3.0 target — identified as a
  structural ceiling of 4-card packs with 5 resonances
- Synergy top-2 share stuck at ~95% (target 75–90%) — a player behavior
  artifact, not an algorithm failure
- No hard guarantee of off-color visibility (probabilistic only)
- Power-chaser convergence too slow (pick 15.7), though arguably correct by
  design

### Weaknesses Identified by Later Rounds

- V2 moved from 5 resonances to 4 resonances / 8 archetypes, fundamentally
  restructuring the design space
- The weighted-sampling approach was superseded by more targeted mechanisms
  (slot-based, token-based) that could achieve higher S/A convergence

______________________________________________________________________

## V2: Tiered Weighted Sampling with Soft Floors (Model C v2)

### Algorithm

Operates on 8 archetypes arranged in a ring topology. Two archetypes are
randomly suppressed per run (50% of S-tier specialist copies removed). Draft
proceeds in two phases:

- **Exploration (picks 1–4):** 4 cards drawn uniformly at random
- **Convergence (picks 5+):** Triggered when the player has 3+ S/A-tier picks in
  one archetype with a 1+ pick lead. Three slots draw weighted-random from the
  committed archetype (7x/8x/9x multiplier ramping over picks 5–30), one
  dedicated splash slot draws off-archetype cards. A soft floor replaces the
  lowest-power card with an on-archetype S/A card if the weighted draw produces
  zero fitting cards (~15–25% of post-commitment packs).

### Benefits

- Only V2 model to pass all 9 simulation targets
- Late fitting of 2.02 S/A (target >= 2.0)
- Convergence at pick 7.32 (target 5–8)
- Deck concentration 89.6% (target 85–95%)
- Run-to-run overlap only 9.9% (target < 40%)
- Invisible ramp (hidden multiplier, not a visible UI element)
- Splash slot creates genuine in-pack tension

### Weaknesses

- Requires ~40% multi-archetype cards (~144 of 360 unique) — highest design
  burden of any model
- The 40% threshold is a hard floor; dropping below causes late fitting to fall
  under 2.0
- ~29 "multi-archetype star" cards (S in 2 neighboring archetypes) are genuinely
  hard to design
- No visible resonance/color system — players need some external way to
  recognize fitting cards
- 89.6% concentration risks feeling "on rails"

### Weaknesses Identified by Later Rounds

- V3 showed that commitment detection (tracking S/A picks per archetype) adds
  complexity compared to symbol-counting approaches
- V4 demonstrated that the weighted-sampling ceiling (~2.0 S/A) could be broken
  by adding cards to packs rather than just reweighting

______________________________________________________________________

## V3: Lane Locking + Pool Asymmetry

### Algorithm

Draft packs have 4 slots, all starting open (random). Players accumulate
resonance symbols as they pick cards (+2 for primary symbol, +1 for
secondary/tertiary). When weighted symbol count reaches specific thresholds,
slots permanently lock:

- **Threshold 4:** One slot locks to the leading resonance (always shows a card
  of that type)
- **Threshold 10:** A second slot locks

Maximum 4 locked slots total. Unlocked slots remain random. A pool asymmetry
layer adds +20 cards of one resonance and removes 20 of another per run for
variety.

### Benefits

- Best late-draft convergence: 2.72 S/A (31% higher than next-best)
- Fastest convergence speed: pick 6.1
- Maximum transparency — lock state is binary and fully visible
- Best archetype frequency balance (8–19%)
- Pool asymmetry adds run-to-run signal reading for experienced players
- Simple enough to describe in one sentence

### Weaknesses

- Deck concentration 99% — worst of all algorithms (target 60–80%)
- Permanent locks prevent pivoting — a player who accidentally commits is stuck
- No signal reading without the pool asymmetry layer
- Late C/F (off-archetype) metric fails for committed players (0.21 vs 0.5
  target)
- Convergence pick fires too early (2.3) due to natural pool composition

### Weaknesses Identified by Later Rounds

- V4 explicitly targeted the permanent lock problem, showing token-spending
  achieves comparable convergence without irreversible commitment
- V5 showed pair-based matching achieves higher S/A precision (~80% vs ~50%) by
  targeting archetype pairs rather than single resonances
- V6 demonstrated that similar convergence could be achieved without any
  permanent state via surge packs

______________________________________________________________________

## V4: Pack Widening

### Algorithm

A token economy with active player decisions. Each drafted card earns resonance
tokens (+2 for primary symbol, +1 for secondary/tertiary). Before each pack, the
player may spend tokens of one resonance to add bonus cards to the pack. The
base 4-card pack is always fully random; bonus cards have primary resonance
matching the spent type.

- **Recommended parameters (v3):** Cost 3 tokens, +1 bonus card
- **Simulated parameters (v2):** Cost 2 tokens, +2 bonus cards

### Benefits

- Breaks the probabilistic S/A ceiling: 3.35 S/A (v2) or ~2.34 S/A (v3
  projected)
- Player agency: spend/save decisions add genuine strategic depth
- Pivot flexibility: tokens accumulate across resonances, player chooses which
  to spend
- No permanent state — fully reversible
- All 8 archetypes converge at the same rate (pick 6–7)
- Projected 80–88% deck concentration (within 60–90% target)

### Weaknesses

- v2 over-commits early (2.48 early S/A, failing \<= 2.0 target)
- v2 over-concentrates (98.6% deck concentration)
- Bonus card dilution: ~50% of bonus cards are wrong-archetype (right resonance,
  wrong archetype)
- Decision fatigue risk: spending decision before every pack (30 picks)
- Token visibility adds UI complexity (4 counters)
- Low signal-reading score (3/10)
- v3 parameters not fully simulated

### Weaknesses Identified by Later Rounds

- V5 showed pair-matching solves the 50% dilution problem (achieving ~80%
  archetype precision)
- V6 demonstrated that zero-decision algorithms can reach 2.05 S/A, making the
  decision fatigue tradeoff questionable — Pack Widening's 1.3 S/A advantage may
  not justify per-turn spending decisions in a roguelike context
- V7 confirmed that the spending decision layer is unnecessary if card design
  achieves adequate sibling A-tier rates

______________________________________________________________________

## V5: Pair-Escalation Slots

### Algorithm

Tracks **ordered resonance pairs** (first symbol, second symbol) instead of
individual symbols. Each 2+ symbol card drafted increments its ordered pair
counter. Before each pack, the probability of each slot being pair-targeted is:

```
P = min(top_pair_count / 6.0, 0.50)
```

Each of 4 slots independently rolls against P. On success, the slot draws from
the pair-matched subset; on failure, from the full pool. Zero player decisions —
convergence is automatic.

### Benefits

- Breaks the dilution ceiling: ~80% S-tier precision for target archetype (vs
  ~50% for single-resonance matching)
- 2.61 S/A with zero player decisions
- Natural pack-to-pack variance from independent slot rolls (stddev 0.98)
- Fully pivot-friendly — no permanent state
- Zero cognitive overhead for players
- Excellent archetype balance (all 8 within 1.2-pick range)

### Weaknesses

- Deck concentration too high (96.2%, target 60–90%)
- Off-archetype splash limited (0.70/pack, lower than Lane Locking's 0.89)
- Slightly slower convergence than Lane Locking (pick 6.3 vs 5.6)
- Probability formula is invisible to players (less transparent than Lane
  Locking)
- 1-symbol cards contribute nothing to pair counts, creating convergence stalls
- No signal reading (pool-independent)

### Weaknesses Identified by Later Rounds

- V6 moved away from pair-based matching back to single-resonance primary
  matching, finding that the complexity of pair tracking wasn't necessary when
  surge mechanics could deliver comparable results
- V7 confirmed that the R2 (secondary resonance) slot concept is "structurally
  worthless" — secondary resonance pools contain cards from unrelated
  archetypes, undermining the pair approach's precision advantage

______________________________________________________________________

## V6: Surge Packs (T=4, S=3)

### Algorithm

Maintains 4 resonance token counters (Ember, Stone, Tide, Zephyr), all starting
at 0. After each pick, tokens are added (+2 primary, +1 secondary/tertiary).
Before each pack:

- If any counter >= 4: subtract 4, generate a **surge pack** (3 of 4 slots
  filled with cards whose primary resonance matches; 4th slot random)
- Otherwise: generate a **normal pack** (all 4 slots random)

Zero player decisions. Ties broken randomly.

### Benefits

- Passes all 9 metrics in unified simulation
- Correct convergence timing (pick 5.9, target 5–8)
- Healthy deck concentration (76.5%, target 60–90%)
- Strong variance (stddev 1.42, well above 0.8 target)
- Non-permanent state — surges follow current leading resonance, allowing
  genuine pivots
- Extreme simplicity: describable in one sentence
- Uniform convergence across all archetypes
- Constant pack size of 4
- Early packs (first 2–3) are almost always fully random, preserving exploration

### Weaknesses

- Barely passes S/A target: 2.05 vs 2.0 threshold (only 0.05 margin)
- Significantly lower S/A than Pack Widening: 2.05 vs 3.35 — the spending
  decision is worth ~1.30 S/A
- Threshold sensitivity: T=3 risks mechanical feel (too many surges), T=5 drops
  convergence
- Non-surge packs are fully random, creating a "dead zone" with near-zero
  targeting value
- Surge visibility question unresolved (whether to show counters to players)

### Weaknesses Identified by Later Rounds

- V7 identified the "dead zone" problem as the primary weakness and solved it by
  adding a floor slot to non-surge packs
- V7 also found T=3 (not T=4) was optimal when combined with the floor mechanism

______________________________________________________________________

## V7: Surge Packs + Floor (T=3, S=3, floor_start=3)

### Algorithm

Builds on V6's surge mechanism with a lower threshold and a new floor mode.
Maintains 4 resonance token counters (+2 primary, +1 secondary/tertiary per
pick).

- **Picks 1–2:** Fully random (4 random slots)
- **Non-surge packs (pick 3+):** **Floor mode** — 1 of 4 slots is filled with a
  card whose primary resonance matches the player's highest counter; other 3
  slots random
- **Surge packs (any counter >= 3):** Subtract 3, fill 3 of 4 slots with
  matching primary resonance cards; 4th slot random

Zero player decisions. Non-permanent state allows pivoting.

### Benefits

- Highest M3 (1.85 S/A under Moderate fitness) among all algorithms with healthy
  convergence — approaches 2.0+ with good card design (65%+ sibling A-tier rate)
- Uniform convergence: all 8 archetypes converge within picks 7.4–8.0 under
  Moderate fitness
- Convergence pick stability: holds constant at 5.0 across all fitness models
- Eliminates V6's dead zone: floor slot raises non-surge pack minimum S/A from
  ~0.5 to ~0.75
- Rhythmic, legible experience: alternating surge/floor modes create visible
  draft rhythm
- Non-permanent state allows genuine pivoting
- Passes 8/9 non-M3 metrics under Moderate fitness

### Weaknesses

- Does not reach 2.0 S/A under Moderate fitness (best: 1.85) — the gap is a card
  design problem, not an algorithm problem
- Slightly lower raw M3 than plain Surge T=3 (1.85 vs 1.88) — floor adds safety
  net but reduces average
- Degrades under Pessimistic fitness (25% sibling A-tier rate → M3 drops to
  1.42)
- T=3 surge frequency may feel routine (~every 1.5–2 picks for committed
  players)
- Floor packs may be perceptually indistinguishable from random packs (1
  targeted slot out of 4)
- Power-chaser support is poor across all tested algorithms
- Surge+Floor+Bias hybrid (projected ~1.97 M3) was proposed but never simulated

______________________________________________________________________

## Cross-Iteration Themes

### The S/A Ceiling Problem

A persistent theme across all iterations is the difficulty of consistently
delivering 2+ on-archetype (S/A-tier) cards per 4-card pack. This target proved
to be constrained by two factors:

1. **Resonance-to-archetype dilution:** With 4 resonances mapping to 8
   archetypes, each resonance is shared by ~2 archetypes. Single-resonance
   matching delivers only ~50% archetype precision. V5's pair matching raised
   this to ~80%, but V7 found the secondary resonance slot "structurally
   worthless."

2. **Card design quality:** V7 conclusively showed that the binding constraint
   is sibling A-tier rate — what fraction of cards from one archetype are also
   good in its sibling archetype. At 50% sibling A-tier, the best algorithm
   achieves 1.85; at 65%+, algorithms automatically cross 2.0.

### Decision vs. Zero-Decision Tradeoff

V4 (Pack Widening) demonstrated that player spending decisions are worth ~1.3
S/A over zero-decision approaches. However, V6 and V7 argued that in a roguelike
context where decision fatigue during drafting is a concern, zero-decision
algorithms delivering 1.85–2.05 S/A are preferable to 3.35 S/A with per-turn
spending.

### Permanent vs. Non-Permanent State

V3's Lane Locking used permanent slot locks — transparent but unforgiving. Every
subsequent iteration moved toward non-permanent mechanisms (tokens that can be
spent on different resonances, probabilistic targeting that follows current
draft direction). V7 confirmed: "Permanent locks are strictly inferior when
variance is valued."

### Convergence vs. Concentration

High convergence (delivering fitting cards) tends to produce high deck
concentration (mono-archetype decks). V3 hit 99% concentration; V5 hit 96.2%.
V6's surge/normal alternation was the first mechanism to achieve strong
convergence (2.05 S/A) with healthy concentration (76.5%).

### Algorithm Complexity Trend

The investigation followed an arc from complex (V2's multi-phase commitment
detection, archetype ring topology, tiered multipliers) toward simple (V7's
one-sentence description). The final recommendation is the simplest algorithm
with the best performance, vindicating the principle that mechanism complexity
does not correlate with metric performance.

______________________________________________________________________

## Evolution of Recommendations

| Version | Recommended Algorithm    | S/A      | Conv. Pick | Decisions | Key Innovation                               |
| ------- | ------------------------ | -------- | ---------- | --------- | -------------------------------------------- |
| V1      | Seeded CRESCENDO         | N/A\*    | 5.0        | Zero      | Floor weight + exponent ramp                 |
| V2      | Tiered Weighted Sampling | 2.02     | 7.32       | Zero      | Commitment detection + splash slot           |
| V3      | Lane Locking             | 2.72     | 6.1        | Zero      | Permanent slot locks                         |
| V4      | Pack Widening            | 3.35     | 6–7        | Per-pack  | Token spending adds cards to packs           |
| V5      | Pair-Escalation Slots    | 2.61     | 6.3        | Zero      | Ordered pair matching                        |
| V6      | Surge Packs              | 2.05     | 5.9        | Zero      | Threshold-triggered surge/normal alternation |
| V7      | Surge + Floor            | 1.85\*\* | 5.0        | Zero      | Floor slot eliminates dead zone              |

\*V1 used different metrics not directly comparable to V2–V7's S/A measurement.

\*\*V7's 1.85 is measured under "Moderate" fitness assumptions (50% sibling
A-tier rate). Under "Optimistic" fitness (75%), the same algorithm delivers
2.0+. V7 recommends revising the M3 target to 1.8 under Moderate fitness,
reframing 2.0 as a card design quality gate.
