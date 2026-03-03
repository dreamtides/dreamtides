# Resonance Draft System V8 — Orchestration Plan

## The Central Problem V8 Must Solve

V7 established that **no zero-decision algorithm reaches M3 >= 2.0 under
Moderate fitness** (50% sibling A-tier). The best result was Surge+Floor (T=3)
at 1.85 S/A. V7 also concluded that the gap to 2.0 is "a card design problem,
not an algorithm problem."

**V8 challenges that conclusion.** V7 held the card pool fixed (15%
dual-resonance cap, 0-3 symbols per card, 360 cards in 4-card packs) and only
varied the algorithm. But the pool composition, symbol system, pack structure,
and explainability constraints are all design choices that can be revisited. V8
asks: **what combination of pool design, symbol system, algorithm, and
constraint relaxation can achieve 2.0+ S/A under realistic — or even pessimistic
— fitness assumptions?**

Additionally, V7 ignored player experience. Surge+Floor creates an alternating
"good pack / bad pack" rhythm that may feel unpleasant to play despite healthy
aggregate numbers. V8 treats player experience as a first-class design
constraint alongside the metrics.

Finally, the designer (the user) believes that even V7's Moderate fitness model
(50% sibling A-tier) may be optimistic. Cross-archetype card design is hard. V8
should calibrate its default fitness assumption downward and find solutions that
work even when cross-archetype playability is low.

______________________________________________________________________

## What Changed Since V7

### V7 Key Results (for agent reference)

- **Recommended algorithm:** Surge+Floor (T=3, S=3, floor_start=3)
- **Best M3 under Moderate fitness:** 1.85 (Surge+Floor), 1.88 (plain Surge T=3)
- **Algorithms eliminated:** Aspiration Packs (R2 slot 3-17% S/A), Compass Packs
  (neighbor rotation worthless), Dual-Counter Surge (cost filter +0.05), all
  dual-resonance targeting approaches
- **Structural findings:**
  1. Slot count dominates slot precision (3 R1 slots at 75% > 1 R1 + 1 R2)
  2. R2 (secondary resonance) slots are structurally worthless for S/A
  3. The degradation from Optimistic to Moderate is universal (~30-50% loss)
  4. T=3 dominates T=4 under realistic fitness
  5. Only ADD/PLACE mechanisms cross 2.0 under Optimistic fitness
- **Untested hybrid:** Surge+Floor+Bias projected ~1.97 M3, never simulated

### V5 Key Results (Pair-Escalation Slots)

V5's Pair-Escalation Slots achieved 2.61 S/A under Optimistic fitness by
matching ordered resonance pairs (primary, secondary) instead of single
resonances. This achieved ~80% S-tier precision per targeted slot. However:

- V5 was never tested under realistic fitness models
- Required 60% of cards to have 2+ symbols (only 15% dual-resonance-type)
- Deck concentration was 96.2% (fails 60-90% target)
- The algorithm feels "on rails" — probabilistic but strongly convergent

V8 should determine: **does Pair-Escalation survive realistic fitness? Can pool
changes make it viable?**

### New Degrees of Freedom in V8

V3-V7 held these parameters fixed. V8 treats them as variables:

1. **Dual-resonance cap:** Previously 15% (54/360 cards with 2 different
   resonance types). What if 25%? 40%? Higher?
2. **Symbols per card:** Previously 0-3, most cards having 1-2. What if every
   card had exactly 3 symbols? What if symbols could repeat (Tide Tide Ember)?
3. **Archetype-resonance binding:** Previously implicit. What if each
   archetype's resonance pair is explicit and enforced in the pool?
4. **Pack size:** Previously fixed at 4. What if 5? 6? Variable?
5. **Explainability:** Previously required one-sentence description. What if
   some mechanics are hidden from the player?
6. **Fitness baseline:** Previously Moderate (50% sibling A-tier). What if the
   default is harder?

______________________________________________________________________

## Lessons from V3–V7

### Mechanism Class Ceilings (V7 finding)

| Class                    | Best M3(A) | Best M3(B) | Binding Constraint           |
| ------------------------ | :--------: | :--------: | ---------------------------- |
| Slot-Filling Surge       |    2.70    |    1.85    | Sibling A-tier rate          |
| Pair-Escalation (V5)     |    2.61    |  untested  | Requires 60% 2+ symbol cards |
| Deterministic Placement  |    2.22    |    ~1.6    | M5/M9/M6 failures            |
| Additive Injection       |    2.13    |    ~1.5    | Trigger rate                 |
| Probabilistic Weighting  |    1.87    |    1.39    | ~2.0 ceiling                 |
| Dual-Resonance Targeting |    1.02    |    0.84    | R2 structurally worthless    |

### The Core Mathematical Constraint

When the algorithm draws from the R1 pool under Moderate fitness:

- 50% of cards are home-archetype (always S-tier)
- 50% are sibling-archetype (50% A-tier under Moderate = 25% of pool)
- Net S/A precision per R1 slot = **75%**

To reach M3 = 2.0 with 3 targeted slots per 4-card pack: 3 × P + 1 × 0.25 = 2.0
→ P = 0.583 → need **58.3% S/A precision minimum**

To reach M3 = 2.0 with 75% precision, need more than 3 targeted slots: N × 0.75
\+ (4-N) × 0.25 = 2.0 → N = 3.5 → need **3.5 targeted slots per pack** on average
(impossible with 4-card packs at 75% precision)

The paths to M3 = 2.0 are:

- **Increase precision above 75%** (better pool composition, dual-res filtering)
- **Increase targeted slot count** (larger packs, more frequent surges)
- **Increase fitness** (better card design — but user says this is hard)
- **Some combination of the above**

______________________________________________________________________

## The Resonance System (V8 Baseline — Subject to Modification)

### Four Resonance Types

Dreamtides has four resonance types: **Ember**, **Stone**, **Tide**, **Zephyr**.

### Eight Archetypes on a Circle

1. **Flash/Tempo** — Zephyr primary, Ember secondary
2. **Blink/Flicker** — Ember primary, Zephyr secondary
3. **Storm/Spellslinger** — Ember primary, Stone secondary
4. **Self-Discard** — Stone primary, Ember secondary
5. **Self-Mill/Reanimator** — Stone primary, Tide secondary
6. **Sacrifice/Abandon** — Tide primary, Stone secondary
7. **Warriors/Midrange** — Tide primary, Zephyr secondary
8. **Ramp/Spirit Animals** — Zephyr primary, Tide secondary

Each resonance is primary for exactly 2 archetypes, secondary for exactly 2.

### Symbol Counting (V7 baseline)

Primary symbol (leftmost): **+2 weight**. Secondary/tertiary: **+1 each**.

**V8 agents may propose changes to the symbol system** — different weighting,
different symbol counts per card, different pool compositions. The system above
is a starting point, not a constraint.

______________________________________________________________________

## Fitness Models

### V8 Default: Pessimistic-Moderate

The designer believes V7's Moderate model (50% sibling A-tier) may be
optimistic. V8 uses a **harsher default** and agents should propose their own
calibrations.

**Suggested fitness range for V8 (agents should explore broadly):**

| Model       | Sibling A-tier | S/A Precision | Purpose                     |
| ----------- | :------------: | :-----------: | --------------------------- |
| Optimistic  |      100%      |     100%      | V6 backward compatibility   |
| Moderate    |      50%       |      75%      | V7's primary target         |
| Pessimistic |      25%       |     62.5%     | V7's stress test            |
| Harsh       |      15%       |     57.5%     | Low cross-archetype overlap |
| Hostile     |       0%       |      50%      | Archetypes are fully siloed |

Under **Hostile** fitness, drawing from the R1 pool gives: 50% home (S-tier) +
50% sibling (all B/C-tier) = 50% S/A precision. This is the absolute floor —
resonance targeting provides zero cross-archetype benefit.

**Agents should calibrate their fitness models to the question they're
testing.** Pool composition changes that increase home-archetype precision
effectively bypass the sibling A-tier problem entirely.

______________________________________________________________________

## Design Goals

Ranked by priority. **Items marked NEW or MODIFIED reflect V8 changes.**

01. **No extra actions.** Player picks 1 card from pack. Non-negotiable.
02. **Not on rails.** Player retains genuine choice. Non-negotiable.
03. **No forced decks.** Can't force the same deck every run.
04. **NEW: Feels good to play.** Pack quality should not have jarring
    alternation. The draft should feel like a smooth progression, not a slot
    machine.
05. **Convergent.** After committing (~pick 6), 2+ S/A cards per pack.
06. **Flexible archetypes.** Can build outside core archetypes.
07. **MODIFIED: Simple — but explainability is negotiable.** One-sentence
    description is preferred. But algorithms with hidden mechanics that produce
    better player experience are acceptable if the experience itself is
    intuitive ("my packs keep getting better" doesn't need a formula).
08. **Splashable.** ~1 off-archetype card in most packs.
09. **Open-ended early.** Picks 1-5 show variety.
10. **Signal reading.** Moderate benefit to identifying open archetype.

### Measurable Targets

**ALL metrics at ARCHETYPE level, not resonance level.**

| ID  | Metric                                               | Target                     |
| --- | ---------------------------------------------------- | -------------------------- |
| M1  | Picks 1-5: unique archetypes with S/A cards per pack | >= 3 of 8                  |
| M2  | Picks 1-5: S/A cards for emerging archetype per pack | \<= 2 of 4                 |
| M3  | Picks 6+: S/A cards for committed archetype per pack | >= 2.0 of 4 avg            |
| M4  | Picks 6+: off-archetype (C/F) cards per pack         | >= 0.5 of 4                |
| M5  | Convergence pick                                     | Pick 5-8                   |
| M6  | Deck archetype concentration                         | 60-90% S/A-tier cards      |
| M7  | Run-to-run variety                                   | < 40% card overlap         |
| M8  | Archetype frequency across runs                      | No archetype > 20% or < 5% |
| M9  | StdDev of S/A cards per pack (picks 6+)              | >= 0.8                     |

**NEW metric:**

| ID  | Metric                                                       | Target |
| --- | ------------------------------------------------------------ | ------ |
| M10 | Pack quality smoothness: max consecutive packs below 1.5 S/A | \<= 2  |

M10 captures the "bad pack streak" problem. Surge+Floor scores poorly here
because non-surge packs can deliver 3-4 consecutive low-S/A packs before a surge
fires.

### Fitness Robustness Target

Every algorithm must report metrics under at least 3 fitness models including
one at or below Pessimistic (25% sibling A-tier). An algorithm that only works
under Moderate or better is not sufficient for V8.

______________________________________________________________________

## Simulation Card Model

```python
class SimCard:
    id: int
    symbols: list[Resonance]  # ordered, length depends on pool design
    archetype: str            # primary archetype (for EVALUATION only)
    archetype_fitness: dict   # archetype_id -> tier — EVALUATION only
    rarity: Rarity
    power: float              # raw card strength (0-10)
```

### Card Pool Construction

**V8 pool construction is agent-configurable.** The V7 baseline was:

- 360 cards: ~40 per archetype (320) + 36 generic
- 15% dual-resonance cap (54 cards with 2 different resonance types)
- 0-3 symbols per card

V8 agents may propose different pool compositions. When doing so, they must
provide a **complete Set Design Specification** (see below) and also:

1. Justify why the pool change is realistic for card design
2. Report metrics under the standard V7 pool AND their proposed pool for
   comparison
3. Keep total pool size at 360 cards (for comparability)

### Set Design Specification (Required Format)

**Every algorithm proposal in V8 must include a concrete set design
specification.** This tells the card designer exactly how to build the pool.
Previous investigations left pool composition vague; V8 requires precision.

A complete set design specification contains:

**1. Pool Breakdown by Archetype:**

| Archetype | Total Cards | Home-Only | Cross-Archetype | Generic |
| --------- | :---------: | :-------: | :-------------: | :-----: |
| Flash     |      ?      |     ?     |        ?        |    —    |
| Blink     |      ?      |     ?     |        ?        |    —    |
| ...       |     ...     |    ...    |       ...       |    —    |
| Generic   |      ?      |     —     |        —        |    ?    |
| **Total** |   **360**   |           |                 |         |

"Home-Only" = cards designed for this archetype specifically (S-tier here, B/C
elsewhere). "Cross-Archetype" = cards designed to be at least A-tier in both
this archetype and its sibling.

**2. Symbol Distribution:**

|     Symbol Count      | Cards | % of Pool | Example             |
| :-------------------: | :---: | :-------: | ------------------- |
|      0 (generic)      |   ?   |     ?     | No resonance        |
|       1 symbol        |   ?   |     ?     | (Tide)              |
|   2 symbols (same)    |   ?   |     ?     | (Tide, Tide)        |
| 2 symbols (different) |   ?   |     ?     | (Tide, Zephyr)      |
|       3 symbols       |   ?   |     ?     | (Tide, Tide, Ember) |

**3. Dual-Resonance Breakdown:**

| Type                            | Cards | % of Pool | Filtering Implications              |
| ------------------------------- | :---: | :-------: | ----------------------------------- |
| Single-resonance                |   ?   |     ?     | Matches 2 archetypes on R1 filter   |
| Dual-resonance (same pair)      |   ?   |     ?     | Matches 1 archetype on R1∩R2 filter |
| Dual-resonance (different pair) |   ?   |     ?     | Matches ? archetypes                |
| Tri-resonance                   |   ?   |     ?     | If applicable                       |

**4. Per-Resonance Pool Sizes:**

For each resonance (Ember, Stone, Tide, Zephyr): how many cards have this as
primary? How many have it as any symbol? When the algorithm filters by "primary
= Tide," how large is the candidate pool and what fraction is home-archetype vs.
sibling?

**5. Cross-Archetype Requirements (the card designer's task):**

"Of the N cards in archetype X, M cards (P%) must be at least A-tier in sibling
archetype Y. This means designing cards with [specific guidance]."

**6. What the Card Designer Must Do Differently:**

Concrete guidance: "Compared to V7's assumptions, you need to \[add N more
dual-resonance cards / ensure every card has 3 symbols / design M cards per
archetype as cross-archetype playable / etc.\]."

______________________________________________________________________

**Agents must fill in this specification for their champion algorithm.** If an
agent proposes no pool changes from V7, they should still provide the V7
baseline specification and note the cross-archetype fitness requirements their
algorithm assumes.

### Simulated Player Strategies

- **Archetype-committed:** Picks highest fitness for strongest archetype.
  Commits around pick 5-6.
- **Power-chaser:** Picks highest raw power regardless of archetype.
- **Signal-reader:** Evaluates which resonance/archetype seems most available
  and drafts toward it.

______________________________________________________________________

## Round 1: Foundational Research (4 parallel agents)

Pure research — no algorithm design. These agents map the new parameter space
that V8 opens up. Their outputs inform all Round 2 agents.

### Research Agent A: Pool Composition Space

**Question:** What pool compositions are viable, and how do they affect the
mathematical ceiling for draft algorithms?

Explore the design space of card pool construction:

- What happens when the dual-resonance cap is raised or removed?
- What happens when cards carry more resonance symbols?
- What happens when symbol repetition is allowed (Tide Tide Ember)?
- How does each change affect the size and composition of resonance-filtered
  subpools?
- What are the archetype-identification properties of each pool design? (Given a
  filter query like "show me cards with Tide," how many archetypes are
  represented? How does this change with richer symbol information?)
- What pool compositions make it mathematically possible to reach 2.0+ S/A under
  Pessimistic fitness?

**Do not design algorithms.** Map the pool design space with mathematical
analysis. Identify which pool compositions create the best conditions for
algorithms to succeed.

**Reads:** This plan, V7 final report, V7 algorithm overview, V5 final report.

**Output:** `docs/resonance/v8/research_pool_composition.md` (max 2500 words)

### Research Agent B: Fitness Calibration

**Question:** What is the right fitness model for a game where cross-archetype
card design is genuinely difficult?

Analyze the archetype circle and its mechanical implications:

- Which archetype pairs share natural mechanical overlap? (e.g., Warriors and
  Sacrifice both care about creatures, but in different ways)
- Which pairs are mechanically distant despite sharing a resonance?
- What is a realistic per-pair sibling A-tier estimate?
- Should fitness be uniform across all sibling pairs, or should it vary by pair?
- What does "15% sibling A-tier" actually mean for card design? Is it achievable
  without trying? What does the designer get "for free"?
- Model at least 4 fitness levels from Optimistic to Hostile, with per-pair
  granularity where appropriate.

**Do not design algorithms.** Establish the fitness landscape that algorithms
must navigate.

**Reads:** This plan, V7 final report, V7 algorithm overview.

**Output:** `docs/resonance/v8/research_fitness_calibration.md` (max 2500 words)

### Research Agent C: Player Experience Analysis

**Question:** What makes a draft feel good or bad, independent of aggregate S/A
numbers?

Research the player experience dimension that V7 ignored:

- The "good pack / bad pack" alternation problem: why does it feel bad? What
  draft rhythms feel natural vs. mechanical?
- What level of pack quality variance is desirable vs. frustrating?
- How much does transparency matter? Can a "magical" algorithm that the player
  can't explain feel better than a transparent one?
- What is the minimum per-pack S/A floor that avoids "dead pack" frustration?
- How do different pack structures (fixed size vs. variable, structured slots
  vs. pure random) affect perceived quality?
- What can we learn from existing digital card game drafts (MTG Arena,
  Hearthstone Arena, etc.) about player satisfaction with draft quality curves?
- Is "on rails" always bad? When does strong convergence feel like support
  rather than constraint?

**Do not design algorithms.** Establish player experience criteria that
algorithms should satisfy.

**Reads:** This plan, V7 final report.

**Output:** `docs/resonance/v8/research_player_experience.md` (max 2500 words)

### Research Agent D: Constraint Audit & Prior Work Review

**Question:** Which V3-V7 constraints should we relax, and what does each
relaxation buy us?

Audit every design constraint carried forward from V3-V7:

- 15% dual-resonance cap → what does raising it cost in card design effort?
- 4-card fixed packs → what does 5 or 6 cards buy in S/A? What does it cost in
  decision complexity and draft length?
- One-sentence explainability → what algorithms become available if we relax
  this? What is lost?
- 0-3 symbols per card → what does mandating more symbols cost in card design?
- 360-card pool → is this the right size?
- Zero player decisions → is there a middle ground (e.g., "choose a row" that's
  simpler than Pack Widening's token spending)?

Also review V5 Pair-Escalation Slots and V3 Lane Locking specifically:

- What were their strengths that V7 lost?
- Under what assumptions do they become viable again?
- What are their untested failure modes under realistic fitness?

**Do not design algorithms.** Map the constraint space and identify the
highest-value relaxations.

**Reads:** This plan, V7 final report, V7 algorithm overview, V5 final report,
V6 final report, V6 algorithm overview.

**Output:** `docs/resonance/v8/research_constraint_audit.md` (max 2500 words)

______________________________________________________________________

## Round 2: Algorithm Design (9 parallel agents)

Each agent reads all Round 1 research outputs plus this plan and the V5-V7
reports. Each explores a broad design space. **Agents should invent freely
within their domain — the descriptions below frame the investigation area, not
the specific algorithms to propose.**

### Agent 1: Baselines Under New Assumptions

Implement and test the top algorithms from V5-V7 under the new conditions
established by Round 1 research:

- Run V7's Surge+Floor (T=3), V5's Pair-Escalation Slots, and V3's Lane Locking
  under at least 4 fitness models (including Pessimistic and below)
- Run each algorithm under at least 2 pool compositions (the V7 standard pool
  AND the most promising pool from Research Agent A)
- Report the untested Surge+Floor+Bias hybrid
- Establish the performance floor and ceiling for V8

This agent's results are the reference point for all other agents.

**Output:**

- `docs/resonance/v8/design_1_baselines.md` (max 2000 words)

### Agent 2: Surge Framework Evolution

The Surge framework (token accumulation → periodic pack transformation) has been
the strongest algorithm family since V6. But it has a fundamental feel problem:
surge packs are exciting, non-surge packs are dull.

Explore the design space of Surge variants that address the rhythm and feel
problem while maintaining or improving S/A numbers. Consider how new pool
compositions or symbol systems might change what Surge can achieve.

**Output:**

- `docs/resonance/v8/design_2_surge_evolution.md` (max 2000 words)

### Agent 3: Slot-Locking Approaches

Lane Locking (V3) achieved strong convergence (2.22 S/A) but was rejected for
poor variance, excessive concentration, and deterministic rigidity. However,
V3-V7 only tested Lane Locking with single-resonance matching on a limited
symbol pool.

Explore whether richer symbol information, different pool compositions, or
modified locking mechanics can address Lane Locking's historical weaknesses
while preserving its convergence strength. The core question: can deterministic
or semi-deterministic slot assignment work if the targeting is more precise?

**Output:**

- `docs/resonance/v8/design_3_slot_locking.md` (max 2000 words)

### Agent 4: Pair-Matching Under Realistic Fitness

V5's Pair-Escalation Slots achieved 2.61 S/A by matching ordered resonance
pairs, achieving ~80% S-tier precision per targeted slot. But V5 was never
tested under realistic fitness models, and it required 60% of cards to have 2+
symbols.

Explore what happens to pair-matching approaches under realistic and pessimistic
fitness. Can pool composition changes (more multi-symbol cards, higher
dual-resonance cap) make pair-matching viable? What is the realistic performance
ceiling for pair-based algorithms?

**Output:**

- `docs/resonance/v8/design_4_pair_matching.md` (max 2000 words)

### Agent 5: Symbol-Rich Architectures

V3-V7 assumed most cards have 1-2 resonance symbols. What if the symbol system
itself is redesigned? What if cards carry more resonance information — more
symbols, repeated symbols, weighted symbols?

Explore the design space of richer symbol systems and the algorithms they
enable. A card with symbols (Tide, Tide, Ember) carries more archetype
information than one with just (Tide). What algorithms can exploit this
additional information? What does this require of card designers?

**Output:**

- `docs/resonance/v8/design_5_symbol_rich.md` (max 2000 words)

### Agent 6: Smooth Delivery Architectures

Surge+Floor creates a "spike and valley" S/A pattern: surge packs deliver ~2.5
S/A, floor packs deliver ~1.2. This alternation may feel bad to play even though
the average is healthy.

Explore algorithms that deliver pack quality more smoothly — where the
difference between the best and worst packs is smaller, but the average remains
high. Can we achieve 2.0+ S/A with a flatter quality distribution? What are the
tradeoffs?

**Output:**

- `docs/resonance/v8/design_6_smooth_delivery.md` (max 2000 words)

### Agent 7: Beyond Explainability

V3-V7 required every algorithm to be describable in one sentence of concrete
operations. This is a strong constraint that eliminates many approaches.

Explore what becomes possible if we relax explainability. "Magical" algorithms
where the player simply observes that their packs get better over time, without
understanding the mechanism. Machine-learned pack construction. Adaptive
algorithms that adjust to the player's behavior. The player experience should
remain intuitive ("my packs are improving") even if the mechanism is not.

What is the performance ceiling when explainability is not a constraint? Is it
significantly higher? Is the player experience better or worse?

**Output:**

- `docs/resonance/v8/design_7_beyond_explainability.md` (max 2000 words)

### Agent 8: Pool-Algorithm Co-Design

V3-V7 treated the card pool as fixed input and optimized the algorithm. But the
pool and algorithm are designed together — changing the pool changes what
algorithms can achieve.

This agent designs the pool composition AND algorithm as a joint system. What
pool is optimal for draft algorithm performance? What does that pool require of
card designers? Is there a "sweet spot" where modest pool changes yield large
algorithm improvements?

**Output:**

- `docs/resonance/v8/design_8_codesign.md` (max 2000 words)

### Agent 9: Open Exploration

Propose any approach not covered by Agents 2-8. This could be:

- A mechanism class never explored in V3-V7
- An approach that reframes what "draft quality" means
- A design that embraces the low cross-archetype fitness reality instead of
  fighting it
- An approach borrowed from a completely different game genre
- Something that changes what the player is choosing between (not just "which
  card" but "which direction")

The only hard constraints: no extra actions beyond card selection, and tested
under at least 3 fitness models.

**Output:**

- `docs/resonance/v8/design_9_open.md` (max 2000 words)

______________________________________________________________________

### Round 2 Output Format (All Agents 1-9)

Each agent produces a single markdown file containing:

1. **Key Takeaways** (5-7 bullets)
2. **Five algorithm proposals:** Name, one-sentence description, 2-3 sentence
   technical description, predicted behavior under at least 3 fitness levels.
   Each proposal that involves pool changes must include a brief pool summary
   (full spec is only required for the champion).
3. **Champion selection** with justification.
4. **Champion deep-dive:** Example draft sequences, failure modes, 2-3 parameter
   variants, proposed fitness model(s) for testing.
5. **REQUIRED: Complete Set Design Specification for the champion** (using the
   format defined in the Simulation Card Model section above). This must
   include: pool breakdown by archetype, symbol distribution, dual-resonance
   breakdown, per-resonance pool sizes, cross-archetype requirements, and
   concrete guidance for the card designer.

**Agent 1 (Baselines) has a different format:** Report results for existing
algorithms under new conditions. No need to propose 5 new algorithms — instead,
report performance tables and analysis. Must include a Set Design Specification
for the V7 standard pool AND for any alternative pool tested.

______________________________________________________________________

## Round 3: Cross-Strategy Discussion (9-agent team, interactive)

All 9 agents read all Round 1 research and all Round 2 design documents and
engage in structured debate.

**Discussion structure (minimum 60 total messages):**

1. **Fitness model alignment (messages 1-15):** Converge on 3-4 shared fitness
   models for simulation. Must include at least one below V7's Pessimistic (25%
   sibling A-tier).
2. **Pool composition alignment (messages 16-30):** Do agents agree on which
   pool compositions to test? Converge on 2-3 shared pool configurations.
3. **Player experience audit (messages 31-45):** Score each champion on M10
   (smoothness) and subjective feel. Identify which algorithms would feel best
   to play, independent of S/A numbers.
4. **Best-of-breed review (messages 46-55):** Cross-pollinate ideas. Can
   components from one agent's design improve another's champion?
5. **Refinement proposals (messages 56-60+):** Modify champions based on
   discussion. May switch champions. May propose hybrids.

**Output per agent (max 1000 words):**

- `docs/resonance/v8/discussion_{1..9}.md`

Each output: agreed fitness models, agreed pool compositions, simplicity
ranking, player experience ranking, scorecard table (algorithm x goal), final
champion, planned modifications for simulation.

______________________________________________________________________

## Round 4: Simulation (9 parallel agents)

Each agent implements and simulates their champion.

**All agents read:** All 9 discussion outputs, all 4 research outputs, plus this
plan.

**Requirements:**

01. Implement the card pool(s) agreed in Round 3 with configurable fitness
    model.
02. Implement the exact championed algorithm.
03. Simulate 1000 drafts x 30 picks x 3 player strategies.
04. **Run under ALL agreed fitness models** (at least 3, including one at or
    below Pessimistic).
05. **Run under ALL agreed pool compositions** (at least 2).
06. Measure all 10 metrics (M1-M10) — ALL AT ARCHETYPE LEVEL.
07. Parameter sensitivity sweeps on 2-3 key parameters.
08. 3 detailed draft traces (early committer, flexible player, signal reader).
09. Per-archetype convergence table (8 rows).
10. Fitness degradation curve across all tested fitness levels.
11. **NEW: Pack quality distribution.** Histogram of per-pack S/A for picks 6+.
    Report the 10th, 25th, 50th, 75th, 90th percentiles. This captures the
    smoothness of delivery.
12. **NEW: Consecutive bad pack analysis.** Report the average and worst-case
    number of consecutive packs with S/A < 1.5 (for committed player, picks 6+).
13. Agent 1: run all baselines. All other agents compare to Agent 1's results at
    each fitness level and pool composition.
14. Verify no player decisions in the implementation (unless the algorithm
    explicitly relaxes this constraint, in which case note it clearly).

**Output per agent:**

- `docs/resonance/v8/sim_{1..9}.py`
- `docs/resonance/v8/results_{1..9}.md` (max 1500 words): one-sentence
  algorithm, **complete Set Design Specification for the pool used** (using the
  format from the Simulation Card Model section), scorecard at each fitness
  level and pool composition, pack quality distribution, consecutive bad pack
  analysis, fitness degradation curve, per-archetype convergence, baseline
  comparison, parameter sensitivity, draft traces, self-assessment. The Set
  Design Specification in the results must reflect the actual pool implemented
  in the simulation code, not just a theoretical proposal.

______________________________________________________________________

## Round 5: Cross-Comparison (9-agent team, interactive)

All 9 agents read all simulation results and engage in structured comparison.

**Minimum 50 total messages. Each agent must:**

1. Score each strategy on each design goal (1-10) at each fitness level.
2. **Player experience scoring:** Rate each algorithm on "how would this feel to
   play?" (1-10) with justification.
3. Identify the single biggest strength and weakness of each strategy.
4. **KEY QUESTION: Which algorithm achieves M3 >= 2.0 under the harshest fitness
   model?** If none do, which comes closest and what would push it over?
5. **KEY QUESTION: Which algorithm feels best to play?** M10 score, pack quality
   distribution, absence of "dead pack" streaks.
6. Compare to all baselines at each fitness level and pool composition.
7. Assess: what combination of pool composition + algorithm + fitness rate is
   needed for 9/9 (or 10/10) metric pass? Give the card designer a concrete,
   achievable target.
8. **If pool composition changes are required:** What is the minimum pool change
   needed? What does it cost the card designer?

**Output per agent (max 1000 words):**

- `docs/resonance/v8/comparison_{1..9}.md`

Each output: scorecard at each fitness level, player experience rating, proposed
best algorithm + pool composition with **complete Set Design Specification**,
minimum fitness requirement, minimum pool change requirement, recommendations to
the card designer. The Set Design Specification must answer: "If I'm the card
designer, exactly how many cards of each type do I need to create?"

______________________________________________________________________

## Round 6: Final Synthesis (1 agent)

A single agent produces the definitive comparison and recommendation.

**Reads:** This plan, all comparison outputs, all results, all discussion
documents, all research documents, V7 final report, V7 algorithm overview, V5
final report.

**Task:**

01. Unified comparison table of all algorithms at each fitness level and pool
    composition.
02. Rank algorithms by **robustness** (performance under harsh fitness) AND
    **player experience** (smoothness, feel).
03. Per-archetype convergence table for the top 3 algorithms.
04. **The key question:** What is the best draft system (algorithm + pool
    composition) that achieves M3 >= 2.0 under realistic fitness? If this
    requires pool changes, specify them precisely.
05. **The feel question:** Which algorithm feels best to play? Is there a
    tradeoff between M3 performance and player experience?
06. Apply the simplicity test independently to each champion.
07. **Card designer's brief:** For the recommended system:
    - What pool composition is required? (dual-resonance %, symbols per card)
    - What fitness rate is assumed? Is it achievable?
    - What does the card designer need to do differently from V7's assumptions?
    - What is the minimum viable card design effort?
08. Write the recommended algorithm with complete specification: one-sentence
    and one-paragraph descriptions, implementation notes, parameter values,
    per-archetype convergence table, AND a **complete Set Design Specification**
    that tells the card designer exactly how to build the 360-card pool. This
    specification must include:
    - Exact card counts per archetype
    - How many cards per archetype must be cross-archetype playable (A-tier in
      sibling), and in which sibling specifically
    - Symbol distribution (how many 1-symbol, 2-symbol, 3-symbol cards)
    - Dual-resonance card count and distribution across archetype pairs
    - Per-resonance pool sizes (how many cards are available when filtering by
      each resonance)
    - A worked example: "For the Warriors archetype (Tide/Zephyr), you need X
      total cards. Y of them must also work in Sacrifice. Z must carry both Tide
      and Zephyr symbols. Here is a concrete breakdown..."
09. **V8 vs V7 vs V5 comparison:** What did V8 find that V7 missed? Was the
    "card design problem" actually a "pool design problem"?
10. **Honest assessment:** Is M3 >= 2.0 achievable without heroic card design?
    What is the realistic S/A target? What player-experience mitigations exist?
11. **Recommendation tiers:** Provide 2-3 recommendations at different
    complexity/effort levels, **each with its own complete Set Design
    Specification:**
    - **Minimal change:** Best algorithm with V7's standard pool (for designers
      who can't change the pool). Include the V7 pool spec and fitness
      requirements.
    - **Moderate change:** Best algorithm with modest pool modifications.
      Specify exactly what changes: "add N dual-resonance cards, ensure M cards
      per archetype have 3 symbols, etc."
    - **Full redesign:** Best algorithm if pool composition is fully flexible.
      Specify the complete pool from scratch.
12. Open questions for playtesting.

**Output:**

- `docs/resonance/v8/final_report.md` (max 4500 words)
- `docs/resonance/v8/algorithm_overview.md` (max 3500 words) — catalog of all
  algorithms **ordered by preference**, not by mechanism class. Structure:
  1. **Gold / Silver / Bronze:** The top 3 recommended systems (algorithm + pool
     composition), each with complete Set Design Specification, full metric
     results, and a clear statement of what the card designer must do. Present
     these as the primary output — the designer should be able to pick one and
     start building.
  2. **Honorable mentions:** Algorithms that showed promise but fell short, with
     brief explanation of why and what would make them viable.
  3. **Eliminated:** Algorithms conclusively ruled out, organized by failure
     mode, with brief rationale. Include algorithms eliminated in V3-V7 that
     were re-tested in V8.
  4. **Structural findings:** Cross-cutting lessons learned.

______________________________________________________________________

## Agent Summary

| Round     | Agents | Type               | Description                          |
| --------- | ------ | ------------------ | ------------------------------------ |
| 1         | 4      | Parallel           | Foundational research                |
| 2         | 9      | Parallel           | Algorithm design per area            |
| 3         | 9      | Team (interactive) | Cross-strategy discussion            |
| 4         | 9      | Parallel           | Simulation under multiple conditions |
| 5         | 9      | Team (interactive) | Cross-comparison                     |
| 6         | 1      | Single             | Final synthesis                      |
| **Total** | **41** |                    |                                      |

## Output Files

| File                              | Round | Description                          |
| --------------------------------- | ----- | ------------------------------------ |
| `research_pool_composition.md`    | 1     | Pool design space analysis           |
| `research_fitness_calibration.md` | 1     | Fitness model calibration            |
| `research_player_experience.md`   | 1     | Player experience criteria           |
| `research_constraint_audit.md`    | 1     | Constraint relaxation analysis       |
| `design_{1..9}_*.md` (x9)         | 2     | Algorithm proposals + champion       |
| `discussion_{1..9}.md` (x9)       | 2     | Cross-domain discussion              |
| `sim_{1..9}.py` (x9)              | 4     | Simulation code                      |
| `results_{1..9}.md` (x9)          | 4     | Results at multiple fitness levels   |
| `comparison_{1..9}.md` (x9)       | 5     | Cross-comparison + experience rating |
| `final_report.md`                 | 6     | Recommendation + specification       |
| `algorithm_overview.md`           | 6     | Catalog of all algorithms            |

All files in `docs/resonance/v8/`.

## Key Principles

01. **No player decisions beyond card selection.** Non-negotiable (unless an
    agent explicitly proposes relaxing this with justification — the proposal
    must be clearly marked and compared to zero-decision alternatives).
02. **ALL MEASUREMENT AT ARCHETYPE LEVEL.** A "Tide card" is NOT a "Warriors
    card."
03. **Pool composition is a design variable.** Agents may propose pool changes
    alongside algorithm changes. Pool + algorithm = system.
04. **Every proposal needs a Set Design Specification.** No algorithm is
    complete without telling the card designer exactly how to build the pool:
    how many cards per archetype, how many must be cross-archetype playable, how
    many symbols per card, how many dual-resonance cards, per-resonance pool
    sizes. Use the required format defined in the Simulation Card Model section.
    An algorithm without a pool spec is like a recipe without ingredient
    quantities.
05. **Test under harsh fitness.** Every algorithm must report metrics under at
    least 3 fitness models including one at or below Pessimistic (25% sibling
    A-tier). Moderate-only results are insufficient.
06. **Player experience matters.** Report M10 (smoothness). Discuss how the
    algorithm feels, not just how it scores. An algorithm with M3=1.9 that feels
    great may beat one with M3=2.1 that feels bad.
07. **Robustness over peak performance.** An algorithm scoring 1.8 S/A under
    Pessimistic fitness beats one scoring 2.5 under Optimistic but 1.2 under
    Pessimistic.
08. **Give the card designer achievable targets.** If the algorithm requires
    pool changes, specify exactly what. If it requires a certain fitness rate,
    say whether that's realistic.
09. **Compare to baselines at every fitness level and pool composition.** V7
    Surge+Floor, V5 Pair-Escalation, and V3 Lane Locking are the references.
10. **Natural variance is a goal.** Consistent delivery is a failure mode. But
    jarring alternation is also a failure mode.
11. **The one-sentence description is preferred but not mandatory.** Mark any
    algorithm that requires more than one sentence as "complex" and justify the
    complexity.
12. **Explore broadly.** Agents should invent freely within their domain.
    Propose mechanisms V3-V7 never considered.
13. **Test honestly.** Report failures clearly. If your champion doesn't reach
    2.0, say so and explain what would be needed.
14. **Tiered recommendations.** The final synthesis should provide options at
    different effort levels, not just a single "best" algorithm.

## Recovery

Check which `docs/resonance/v8/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained.
