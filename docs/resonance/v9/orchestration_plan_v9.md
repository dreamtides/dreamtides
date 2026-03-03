# Resonance Draft System V9 — Orchestration Plan

## The Central Problem V9 Must Solve

V8 proved that M3 >= 2.0 is achievable under realistic fitness — but at a cost.
The winning approach (Narrative Gravity on a 40% dual-resonance pool) requires
nearly 4 in 10 cards to carry two visible resonance symbols. When 40% of cards
wear their archetype pair on their sleeve, the "correct" pick becomes obvious:
just grab whatever matches your (Tide, Zephyr). The draft loses its decision
texture. The player stops evaluating cards and starts pattern-matching symbols.

**V9 asks: can we keep V8's mathematical gains while hiding most of the
targeting information from the player?**

Specifically, V9 explores the design space of **hidden vs. visible resonance
information**:

- **Visible resonance** (~10% dual-resonance, ~90% single or zero symbols):
  what the player sees on the card. This is the drafting signal. It must feel
  important and meaningful — the primary factor guiding draft decisions.

- **Hidden metadata** (algorithmic tagging the player never sees): additional
  archetype-pair information used by the pack construction algorithm to stay
  on target. A card showing (Tide) to the player might be internally tagged as
  Warriors (Tide/Zephyr) for algorithmic purposes.

The key design constraints:

1. **Visible dual-resonance must be rare (~10%).** Players should encounter
   dual-symbol cards as noteworthy signposts, not as the default.

2. **Visible resonance must feel like it matters.** If the algorithm is doing
   all the work behind the scenes and the visible symbols are decorative, the
   system fails even if the numbers are good.

3. **Hidden manipulation should be minimal.** Every unit of hidden
   manipulation is "cheating" to some degree — it's the algorithm making
   decisions the player thinks are random. The less we need, the better the
   design integrity. V9 should find the **minimum hidden information** needed
   to reach M3 >= 2.0 and M11 >= 3.0.

4. **No player decisions beyond card selection.** Non-negotiable.

5. **The algorithm should feel honest.** If a player reverse-engineered the
   pack construction, they shouldn't feel deceived. "The game tags each card
   with its best archetype and uses that to build better packs" is defensible.
   "The game secretly assigns cards to archetypes they don't belong to" is not.

---

## What Changed Since V8

### V8 Key Results (for agent reference)

**Gold: Narrative Gravity** (pool contraction) on 40% Enriched Compensated Pool.
After each pick, permanently remove the least relevant cards from the pool based
on the player's accumulated resonance signature. Future packs draw from an
increasingly concentrated pool.

- M3 = 2.75 (Graduated Realistic), 2.59 (Pessimistic), 2.49 (Hostile)
- All 8 archetypes above M3 >= 2.0 under Graduated Realistic
- Best player experience rating (7.9/10): monotonic quality ramp
- M10 = 3.3 (fails target <= 2): transition zone picks 6-10 creates bad streak
- M5 = 10.2 (fails target 5-8): slow convergence
- Requires 40% dual-resonance pool (132 dual-res cards of 360)

**Silver: CSCT** (Commitment-Scaled Continuous Targeting). Pair-matched slots
scale continuously with the player's commitment ratio.

- M3 = 2.92 (Graduated Realistic) — highest of any algorithm
- Nearly fitness-immune (0.23 degradation Optimistic to Hostile)
- M10 = 2 (only algorithm to pass smoothness)
- **Disqualified by M6 = 99%** (deck concentration, target 60-90%)
- Works equally well on V7 15% pool (M3 = 2.93) — barely needs dual-res
- Per-archetype equity: 0.08 spread (best tested)

**Bronze: Symbol-Weighted Graduated Escalation** on Symbol-Rich pool (84.5%
dual-res, 3 symbols per card).

- M3 = 2.50 (Graduated Realistic), near-immune to fitness
- Requires demanding 3-symbol-per-card pool design

**Other notable results:**

- **Continuous Surge:** M3 = 2.48 on 40% pool, but 3 archetypes below 2.0
  (Ramp = 1.55). Strong aggregate, poor equity.
- **SF+Bias (R1):** M3 = 2.24 on V7 15% pool with ALL archetypes above 2.0
  (worst = Ramp 2.06). Best low-dual-res algorithm. Fails M10 (streak = 8).
- **Discrete pair-counter mechanisms** (Agents 3, 6, 8): All failed due to
  pair alignment catastrophe (~40-55% of drafts locked onto wrong archetype).

### V8 Structural Findings

1. **Pool composition dominates algorithm choice.** Same algorithm yields
   M3 = 1.99 on V7 standard, 2.29 on 40% enriched — the pool is worth +0.30.
2. **Pair-matching is the key to fitness robustness.** 11% degradation
   Optimistic-to-Hostile vs. 46% for R1-filtering.
3. **Continuous methods beat discrete counters.** Commitment ratios and
   resonance signatures degrade gracefully; discrete pair counters fail
   catastrophically.
4. **The M3-M10-M6 triangle has no perfect solution.** High M3 + good M10
   requires M6 = 99% (CSCT). High M3 + good M6 produces M10 = 3.3
   (Narrative Gravity).
5. **Pool contraction is a new mechanism class.** Removing bad cards from
   the pool raises ALL slots, bypassing per-slot precision ceilings.

### V9 Starting Point

V9 takes the V8 results and asks: **what happens when we split the information
that V8's algorithms used into visible (player-facing) and hidden (algorithm-
only) components?**

The hypothesis: V8's pair-matching worked because it gave the algorithm enough
information to identify archetypes precisely. That information doesn't need to
come from visible symbols — it can come from hidden metadata assigned during
card design. The visible symbols then serve a different purpose: giving the
player a legible, honest signal about what the card does thematically.

---

## Fixed Assumptions (Not Variables in V9)

V8 explored fitness models, pool sizes, and constraint relaxations. V9 fixes
these to enable clean comparison.

### Fitness Model: Graduated Realistic (Fixed)

| Pair | Sibling A-Tier Rate |
|------|:---:|
| Warriors / Sacrifice (Tide) | 50% |
| Self-Discard / Self-Mill (Stone) | 40% |
| Blink / Storm (Ember) | 30% |
| Flash / Ramp (Zephyr) | 25% |
| **Weighted Average** | **~36%** |

All simulations use this model. Agents may report a secondary run at Pessimistic
(21% avg) for robustness checking, but Graduated Realistic is the primary
evaluation target.

### Pool Size: 360 Cards (Fixed)

40 per archetype (320) + 40 generic. Not a variable.

### Pack Size: 4 Cards (Fixed)

Not a variable.

### Archetypes: 8 on a Circle (Fixed)

1. Flash/Tempo — Zephyr primary, Ember secondary
2. Blink/Flicker — Ember primary, Zephyr secondary
3. Storm/Spellslinger — Ember primary, Stone secondary
4. Self-Discard — Stone primary, Ember secondary
5. Self-Mill/Reanimator — Stone primary, Tide secondary
6. Sacrifice/Abandon — Tide primary, Stone secondary
7. Warriors/Midrange — Tide primary, Zephyr secondary
8. Ramp/Spirit Animals — Zephyr primary, Tide secondary

### Metrics: M1-M11 (Fixed, All at Archetype Level)

| ID | Metric | Target |
|----|--------|--------|
| M1 | Picks 1-5: unique archetypes with S/A cards per pack | >= 3 of 8 |
| M2 | Picks 1-5: S/A cards for emerging archetype per pack | <= 2 of 4 |
| M3 | Picks 6+: S/A cards for committed archetype per pack | >= 2.0 of 4 avg |
| M4 | Picks 6+: off-archetype (C/F) cards per pack | >= 0.5 of 4 |
| M5 | Convergence pick | Pick 5-8 |
| M6 | Deck archetype concentration | 60-90% S/A-tier cards |
| M7 | Run-to-run variety | < 40% card overlap |
| M8 | Archetype frequency across runs | No archetype > 20% or < 5% |
| M9 | StdDev of S/A cards per pack (picks 6+) | >= 0.8 |
| M10 | Max consecutive packs below 1.5 S/A (picks 6+) | <= 2 |
| M11 | Picks 15+: S/A cards for committed archetype per pack | >= 3.0 of 4 avg |

### Player Strategies (Fixed)

- **Archetype-committed:** Picks highest fitness for strongest archetype.
  Commits around pick 5-6.
- **Power-chaser:** Picks highest raw power regardless of archetype.
- **Signal-reader:** Evaluates which resonance/archetype seems most available
  and drafts toward it.

---

## The Information Design Space

This is V9's novel contribution. Every card has two layers of information:

### Layer 1: Visible Resonance (Player-Facing)

What the player sees on the card. This is what V3-V7 called "resonance symbols."

**V9 constraint:** ~10% of cards show dual resonance (two symbols). ~50-60%
show single resonance (one symbol). ~30-40% show no resonance (generics or
mechanically-typed cards). The exact distribution is a variable for agents to
explore, but visible dual-resonance must stay near 10%.

The visible symbols must feel meaningful:
- A player who drafts Tide cards should see their packs improve.
- A player who sees a rare (Tide, Zephyr) card should recognize it as a
  strong Warriors signal.
- The visible resonance should be the primary basis for the player's draft
  strategy.

### Layer 2: Hidden Metadata (Algorithm-Only)

What the algorithm knows but the player doesn't see. This is assigned during
card design and used by the pack construction algorithm. **Hidden metadata
is not limited to resonance-style symbols — agents should explore any form
of per-card data that helps the algorithm build better packs.** Examples
include but are not limited to:

- **Archetype tag:** Each card is tagged with its "best" archetype. A card
  showing (Tide) might be tagged "Warriors" or "Sacrifice" based on its
  mechanical design. This is the simplest form of hidden metadata (~3 bits
  for 8 archetypes).

- **Archetype affinity scores:** Each card has a score (0-1) for each
  archetype, representing how well it plays in that archetype. More
  information than a tag, but more honest — the scores reflect real
  mechanical fit.

- **Mechanical keyword tags:** Cards are tagged with gameplay-relevant
  properties (e.g., "removal," "card-draw," "creature," "graveyard-matters")
  that the algorithm uses to ensure pack diversity or archetype coherence
  without referencing resonance at all.

- **Synergy graph edges:** Cards carry hidden links to other cards they
  synergize with, and the algorithm biases packs toward cards that connect
  to the player's existing draft. No resonance information needed.

- **Archetype cluster membership:** Cards are grouped into overlapping
  clusters by mechanical theme (e.g., "creature-matters," "spell-matters,"
  "graveyard-matters") and the algorithm uses cluster overlap to identify
  cross-archetype cards.

- **Hidden secondary resonance:** Each card has a hidden second (or third)
  resonance symbol used for algorithmic pair-matching but not shown to the
  player. Equivalent to V8's dual-resonance pool but invisible. This is
  only one option among many — not the default assumption.

- **Power-curve metadata:** Cards carry hidden indicators of their draft
  timing (early pick vs. late pick value) that the algorithm uses to shape
  pack quality progression.

Agents should feel free to propose hidden metadata schemes not listed here.
The key constraint is design integrity (V3 metric), not the form of the data.

### The Design Integrity Spectrum

From most honest to most "cheating":

1. **No hidden info (V7 baseline):** Algorithm uses only visible symbols.
   Limited to M3 ~ 2.24 (SF+Bias R1) on the low-dual-res pool.

2. **Hidden metadata derived from real card properties:** Archetype affinity
   scores, mechanical keyword tags, synergy relationships — data that
   reflects genuine properties of the card. A player who looked it up would
   agree the metadata is accurate. Defensible because it reflects reality.

3. **Hidden archetype tag (best-fit assignment):** Each card gets one
   archetype label. Simple but loses nuance — a card that's B-tier in two
   archetypes gets assigned to only one. Still honest, but a simplification.

4. **Hidden algorithmic labels disconnected from card mechanics:** Tags
   assigned purely to optimize algorithm performance, regardless of whether
   they reflect the card's actual mechanical fit. Mathematically powerful
   but players who discover the system may feel it's arbitrary.

5. **Arbitrary hidden manipulation:** The algorithm can do anything it wants
   with hidden data. Maximum performance, minimum design integrity.

**V9 should explore levels 1-4 and identify the sweet spot: the minimum
hidden information that reaches M3 >= 2.0 and M11 >= 3.0 while keeping the
visible resonance system feeling important. The hidden information need not
be resonance-based at all — the best solution may use a completely different
kind of metadata.**

---

## Design Goals

Ranked by priority.

1. **No extra actions.** Player picks 1 card from pack. Non-negotiable.
2. **Not on rails.** Player retains genuine choice. Non-negotiable.
3. **Visible resonance feels important.** The player's drafting decisions
   should be guided primarily by the visible symbols, not by hidden forces
   they can't perceive. If the algorithm is doing all the work, the visible
   system is decorative.
4. **Feels good to play.** No jarring alternation. Smooth progression.
5. **Convergent.** After committing (~pick 6), 2+ S/A cards per pack.
6. **Honest.** A player who reverse-engineers the system should feel the
   hidden layer is a reasonable extension of the visible one, not a deception.
7. **Flexible archetypes.** Can build outside core archetypes.
8. **Simple — visible system is one sentence, hidden system is acceptable.**
   The player-facing description must be one sentence. The hidden algorithm
   can be more complex, but simpler is better.
9. **Splashable.** ~1 off-archetype card in most packs.
10. **Open-ended early.** Picks 1-5 show variety.

### V9-Specific Evaluation Criteria (Beyond M1-M11)

In addition to the standard metrics, V9 algorithms must report:

| ID | Criterion | Description |
|----|-----------|-------------|
| V1 | Visible symbol influence | What % of the algorithm's targeting power comes from visible symbols alone? (Run the algorithm with hidden metadata stripped — the delta is the hidden contribution.) |
| V2 | Hidden info quantity | How many bits of hidden metadata per card? (1-bit archetype tag = 3 bits for 8 archetypes. Affinity vector = 8 floats = much more.) |
| V3 | Reverse-engineering defensibility | If a player figured out the hidden system, would they feel the game is fair? (Subjective 1-10 rating with justification.) |
| V4 | Visible resonance salience | Does the visible resonance system create meaningful draft decisions, or are picks determined by hidden forces? (Run 100 drafts and count how often the "best visible pick" differs from the "best hidden pick".) |

**The ideal V9 algorithm scores high on V1 (visible symbols do most of the
work), low on V2 (minimal hidden info), high on V3 (honest), and high on V4
(visible choices matter).**

---

## Simulation Card Model

```python
class SimCard:
    id: int
    visible_symbols: list[Resonance]   # what the player sees (0-2 symbols)
    hidden_metadata: dict              # algorithm-only info (agent-defined)
    archetype: str                     # primary archetype (for EVALUATION only)
    archetype_fitness: dict            # archetype_id -> tier — EVALUATION only
    rarity: Rarity
    power: float                       # raw card strength (0-10)
```

### Visible Symbol Distribution (V9 Baseline)

| Symbol Count | Cards | % | Notes |
|:---:|:---:|:---:|---|
| 0 (generic) | 40 | 11% | No visible resonance |
| 1 visible symbol | 284 | 79% | Shows primary resonance only |
| 2 visible symbols | 36 | 10% | Rare dual-resonance signposts |

Agents may propose variations (e.g., more generics, fewer dual-res) but
visible dual-resonance must stay in the 8-12% range.

### Hidden Metadata (Agent-Defined)

Each agent defines what hidden metadata their algorithm needs. The synthesis
agent will compare algorithms partly on how much hidden information they
require (less is better).

---

## Round 1: Foundational Research (3 parallel agents)

Pure research — no algorithm design. These agents map the information design
space that V9 opens up.

### Research Agent A: Information Design in Games

**Question:** How do successful games use hidden information in procedural
content generation, and what makes it feel fair vs. deceptive?

Explore:
- How do roguelike deckbuilders (Slay the Spire, Monster Train, Inscryption)
  use hidden weighting in card rewards?
- How does MTG Arena's draft bot system use hidden preferences? Do players
  feel cheated by algorithmic pack construction?
- When does hidden algorithmic manipulation feel like "good game design" vs.
  "the game is lying to me"?
- What is the relationship between visible signals and hidden systems in
  successful games? How much visible information do players need to feel
  in control?
- What design patterns make hidden systems feel honest?

**Output:** `docs/resonance/v9/research_information_design.md` (max 2000 words)

### Research Agent B: Mathematical Ceiling at Low Visible Dual-Res

**Question:** What is the maximum M3 achievable with only visible information
(~10% dual-res, no hidden metadata), and how much does each type of hidden
metadata add?

Analyze mathematically (no simulation — pure analysis):
- V8's algorithms on a 10% visible dual-res pool using ONLY visible symbols.
  What's the M3 ceiling?
- If we add a 3-bit archetype tag (hidden) to each card, what precision can
  the algorithm achieve per targeted slot?
- If we add a hidden secondary resonance symbol, how does this compare to
  V8's visible 40% dual-res pool?
- If we add archetype affinity scores (8 floats per card), what's the ceiling?
- What is the minimum hidden information per card needed to reach M3 = 2.0
  and M11 = 3.0 (late-draft density, picks 15+) under Graduated Realistic
  fitness?
- How does the V1 metric (visible symbol influence %) change across these
  levels?

Use V8's simulation data and mathematical models. Reference the per-archetype
convergence tables from V8's results.

**Reads:** This plan, V8 final report, V8 algorithm overview.

**Output:** `docs/resonance/v9/research_math_ceiling.md` (max 2000 words)

### Research Agent C: Visible Resonance Salience

**Question:** Under what conditions does the visible resonance system feel
like the primary drafting signal, even when hidden manipulation is active?

Explore:
- If the algorithm secretly makes 60% of pack quality decisions, but the
  player attributes quality to their visible-resonance strategy, is that
  good design or deception?
- What visible resonance patterns create the strongest sense of "I'm building
  toward Tide"? (Seeing more Tide cards over time? Higher-power Tide cards?
  Tide cards that synergize with each other?)
- How much of the player's draft strategy should be determined by visible
  symbols vs. card mechanics vs. power level?
- Is there a threshold below which hidden manipulation is undetectable by
  the player and above which it becomes obvious?
- How should the visible dual-resonance signpost cards (the ~10%) be
  designed to maximize their signal value despite being rare?

**Reads:** This plan, V8 player experience research, V8 final report.

**Output:** `docs/resonance/v9/research_visible_salience.md` (max 2000 words)

---

## Round 2: Algorithm Design (6 parallel agents)

Each agent reads all Round 1 research outputs plus this plan and V8 reports.
Each explores a different approach to the hidden/visible information split.

**Fixed parameters for all agents:**
- Fitness: Graduated Realistic (36% avg, per-pair)
- Pool: 360 cards, ~10% visible dual-res (~36 cards), ~79% single-symbol
  (~284 cards), ~11% generic (~40 cards)
- All V8 reference results available for comparison

**Output format (all agents):**

1. Key findings (5-7 bullets)
2. Three algorithm proposals: name, one-sentence visible description,
   technical description, hidden metadata requirements, predicted M3/M10/M11/M6
   under Graduated Realistic
3. Champion selection with justification
4. Champion deep-dive: how it works, what the player sees vs. what the
   algorithm does, example draft, failure modes, V1-V4 metrics
5. Pool specification (visible symbol distribution + hidden metadata schema)

Max 1500 words per agent. No Set Design Specification required — the synthesis
agent will standardize the winner.

### Agent 1: Visible-Only Baseline + Minimal Enhancement

**Starting point:** V8's SF+Bias (R1) achieved M3 = 2.24 with ALL archetypes
above 2.0 on V7's 15% dual-res pool, using only visible symbols. This is the
proven visible-only ceiling.

**Question:** What is the smallest hidden enhancement that pushes this family
of algorithms from M3 = 2.24 to M3 >= 2.0 (and M11 >= 3.0 for late-draft
density) with better M10? Can a simple 3-bit archetype tag (the minimum
possible hidden metadata) close the gap on the metrics SF+Bias fails
(M10 = 8)?

Explore the design space between "pure visible" and "minimally hidden." The
goal is the least hidden information that fixes SF+Bias's weaknesses.

### Agent 2: Narrative Gravity with Hidden Metadata

**Starting point:** V8's Narrative Gravity achieved M3 = 2.75 on the 40%
dual-res pool but only M3 = 2.38 on the V7 15% pool (with Flash at 1.47).

**Question:** Can Narrative Gravity's pool contraction mechanism work with
~10% visible dual-res if the contraction algorithm uses hidden archetype
metadata instead of visible pair symbols?

The player sees: "my packs are getting better as I commit to Tide."
The algorithm does: contraction using hidden metadata — which could be
archetype tags, affinity scores, mechanical keyword tags, synergy graph
edges, or any other per-card data — producing the same quality ramp but
without requiring visible dual-res.

What kind of hidden metadata does Narrative Gravity need? Is a simple
archetype tag sufficient? Would mechanical keywords or synergy data work
better? Does the form of hidden data change the player experience?

### Agent 3: CSCT Detuned with Visible Emphasis

**Starting point:** V8's CSCT achieved M3 = 2.92 but was disqualified by
M6 = 99%. However, it barely depended on pool composition (M3 = 2.93 on V7
15% pool).

**Question:** Can CSCT be detuned (capped at 2 pair-matched slots, reduced
multiplier) to fix M6/M9 while using hidden metadata for the pair-matching
and visible resonance for the player experience?

CSCT has massive M3 headroom (2.92 vs. 2.0 target). Trading 0.5+ M3 for
better M6, M9, and M10 should be feasible. The detuned version needs some
form of hidden metadata (archetype tags, mechanical keywords, affinity
scores, or other per-card data) for targeting precision, but the player
experiences the visible resonance as the guiding signal.

### Agent 4: Layered Salience

**Question:** Can we design an algorithm where the visible resonance does
most of the targeting work (V1 >= 60%) and hidden metadata only provides a
modest boost?

Instead of treating visible and hidden as separate systems, design them as
layers where the visible layer provides the foundation and the hidden layer
provides refinement:

- **Visible layer:** R1 filtering by primary resonance (as in V7). This
  delivers ~50-75% S/A precision depending on fitness.
- **Hidden layer:** Within the R1-filtered pool, bias toward cards whose
  hidden archetype tag matches the player's inferred archetype. This
  narrows from 75% to ~85% precision.

The visible symbols are doing the heavy lifting (R1 filtering). The hidden
metadata is a tiebreaker within an already-filtered pool. V1 should be high.

### Agent 5: Honest Affinity System

**Question:** What if the hidden metadata is maximally honest — each card
carries archetype affinity scores that genuinely reflect its mechanical fit,
and the algorithm uses these transparently?

Design a system where:
- Every card has publicly-knowable archetype affinities (even if not printed
  on the card, they're in a database the player could look up)
- The algorithm uses these affinities for targeting
- The affinities are a direct function of the card's visible properties
  (cost, type, keywords, resonance symbols)
- A player who understands the system should agree that the affinities are
  fair assessments of card quality per archetype

This is "hidden" only in the sense of not being printed on the card face.
It's the most defensible form of hidden information. How well does it
perform?

### Agent 6: Adaptive Contraction with Visible Feedback

**Question:** Can we design a pool contraction system (Narrative Gravity
family) that uses hidden metadata for precision but provides visible feedback
that reinforces the player's sense of agency?

Explore:
- What if the contraction is driven 60% by visible symbols and 40% by hidden
  metadata? What does this look like to the player?
- Can the rare visible dual-resonance cards (~10%) serve as high-value
  "anchor" cards that the contraction algorithm weights heavily, making the
  player feel like "finding a (Tide, Zephyr) card really focused my packs"?
- What contraction rate and metadata scheme achieves M3 >= 2.0 and
  M11 >= 3.0 while keeping V1 >= 50%?

---

## Round 3: Critic Review (1 agent, sequential)

A single critic agent reads all 6 Round 2 proposals plus all Round 1 research
and this plan.

**Task:**

1. Identify the strongest and weakest proposal. Justify with specific metrics.
2. Identify common mechanisms across proposals that seem convergent.
3. Identify the best approach to the hidden/visible split — which proposal
   best satisfies V1-V4?
4. Propose 1-2 hybrid designs combining strengths from multiple proposals.
5. Flag any proposal that "cheats" (V3 < 5) or makes visible resonance
   decorative (V1 < 40%).
6. Rank all champions on: M3 potential, player experience, design integrity
   (V3), visible salience (V1/V4), hidden info minimality (V2).
7. Recommend which 4-6 algorithms should advance to simulation (may include
   hybrids).

**Output:** `docs/resonance/v9/critic_review.md` (max 2500 words)

After the critic review is written, each of the 6 design agents gets a brief
response turn (max 500 words each) to modify their champion based on the
critique. These responses are appended to their original design documents as
a "## Post-Critique Revision" section.

---

## Round 4: Simulation (6 parallel agents)

Each agent implements and simulates their champion (as modified by the critic
response). Agent assignments may change based on the critic's recommendations
— if the critic eliminates a proposal and recommends a hybrid, the displaced
agent implements the hybrid instead.

**All agents read:** Critic review, all design documents (with post-critique
revisions), all research documents, this plan.

**Fixed simulation parameters:**
- 1000 drafts x 30 picks x 3 player strategies
- Fitness: Graduated Realistic (primary), Pessimistic (secondary)
- Pool: 360 cards, ~10% visible dual-res, hidden metadata per agent's design
- All 11 metrics (M1-M11) at archetype level
- All 4 V9 criteria (V1-V4)

**Required outputs per agent:**

1. Simulation code: `docs/resonance/v9/sim_{1..6}.py`
2. Results: `docs/resonance/v9/results_{1..6}.md` (max 1000 words)

Results must include:
- Scorecard (all metrics at Graduated Realistic; M3/M10/M11 at Pessimistic)
- V1-V4 measurements
- Per-archetype M3 table (8 rows)
- Pack quality distribution (p10/p25/p50/p75/p90 for picks 6+)
- Consecutive bad pack analysis
- 2 draft traces (committed player, signal reader)
- Comparison to V8 baselines: Narrative Gravity (M3 = 2.75 on 40% pool),
  SF+Bias R1 (M3 = 2.24 on V7 pool), CSCT (M3 = 2.92 on V7 pool)
- Self-assessment: does this algorithm pass? What would fix the failures?

Run the simulation and report actual numbers. No projections.

---

## Round 5: Final Synthesis (1 agent)

A single agent produces the definitive comparison and recommendation.

**Reads:** This plan, all research, all design documents (with revisions),
critic review, all simulation results, V8 final report, V8 algorithm overview.

**Produces two files:**

### File 1: `docs/resonance/v9/final_report.md` (max 4000 words)

1. Unified comparison table of all algorithms (metrics + V1-V4)
2. The key question: **What is the minimum hidden manipulation needed to
   reach M3 >= 2.0 and M11 >= 3.0 while keeping visible resonance as the
   primary drafting signal?**
3. The design integrity question: where on the spectrum (no hidden info →
   full hidden manipulation) is the sweet spot?
4. Per-archetype convergence table for the top 3 algorithms
5. V9 vs V8 comparison: what did we gain and lose by moving to hidden
   metadata?
6. Honest assessment: is V1 >= 50% (visible symbols doing most of the work)
   achievable alongside M3 >= 2.0 and M11 >= 3.0?
7. Recommendation tiers:
   - **Minimal hidden info:** Best algorithm with <= 3 bits of hidden
     metadata per card. What M3 does this achieve?
   - **Moderate hidden info:** Best algorithm with hidden archetype tag +
     limited affinity data. Target M3 >= 2.0, M11 >= 3.0.
   - **Full hidden support:** Best algorithm with whatever hidden metadata
     is needed. V8's equivalent but with rare visible dual-res.
8. Complete Set Design Specification for the recommended algorithm (pool
   breakdown, visible symbol distribution, hidden metadata schema, cross-
   archetype requirements, worked example for Warriors)
9. Card designer's brief: what to do differently from V8
10. Open questions for playtesting

### File 2: `docs/resonance/v9/algorithm_overview.md` (max 2500 words)

Catalog of all algorithms ordered by preference:

1. **Recommended (1-2 algorithms):** Complete specification, full metrics,
   V1-V4 scores, card designer guidance
2. **Viable alternatives:** Algorithms that work but with different tradeoffs
   on the hidden/visible spectrum
3. **Eliminated:** Algorithms that failed, organized by failure mode
4. **Structural findings:** Cross-cutting lessons about hidden vs. visible
   information design

---

## Agent Summary

| Round | Agents | Type | Description |
|-------|--------|------|-------------|
| 1 | 3 | Parallel | Foundational research |
| 2 | 6 | Parallel | Algorithm design |
| 3 | 1 + 6 responses | Sequential | Critic review + designer responses |
| 4 | 6 | Parallel | Simulation |
| 5 | 1 | Single | Final synthesis |
| **Total** | **~19** | | |

## Output Files

| File | Round | Description |
|------|-------|-------------|
| `research_information_design.md` | 1 | Hidden info in games |
| `research_math_ceiling.md` | 1 | Mathematical analysis of hidden info value |
| `research_visible_salience.md` | 1 | When visible resonance feels important |
| `design_{1..6}.md` (x6) | 2 | Algorithm proposals |
| `critic_review.md` | 3 | Cross-proposal analysis |
| `sim_{1..6}.py` (x6) | 4 | Simulation code |
| `results_{1..6}.md` (x6) | 4 | Results with V1-V4 |
| `final_report.md` | 5 | Recommendation + specification |
| `algorithm_overview.md` | 5 | Catalog of all algorithms |

All files in `docs/resonance/v9/`.

## Key Principles

1. **Visible resonance is the primary signal.** If hidden manipulation makes
   visible symbols decorative, the design fails regardless of M3.
2. **Minimum hidden information wins.** Between two algorithms with equal M3,c
   prefer the one with less hidden metadata.
3. **Design integrity matters.** A player who discovers the hidden system
   should feel the game is fair, not deceptive.
4. **Graduated Realistic fitness is fixed.** Do not re-explore fitness models.
   Use the V8-calibrated per-pair rates.
5. **Report V1-V4 alongside M1-M11.** The information design metrics are
   as important as the performance metrics.
6. **Build on V8, don't repeat it.** V8 established that pair-matching +
   pool contraction works. V9 asks how to implement it honestly with
   hidden metadata. Do not re-derive V8's findings.
7. **Compare to V8 baselines at every step.** Narrative Gravity (40% pool),
   SF+Bias R1 (V7 pool), and CSCT (V7 pool) are the references.
8. **Practical card designer output.** The final recommendation must tell the
   card designer exactly what hidden metadata to assign and how.
9. **Simpler hidden systems are better hidden systems.** A 3-bit archetype
   tag that works is preferable to a 64-float affinity matrix that works
   slightly better.

## Recovery

Check which `docs/resonance/v9/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained.
