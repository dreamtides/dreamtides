# Draft Resonance Algorithm Design — Orchestration Plan

## Game Context

**Dreamtides** is a roguelike deckbuilding card game. During a quest, players
draft a deck of ~30 cards from a shared pool of **360 unique cards**. Cards
belong to **5 resonances** (Tide, Ember, Zephyr, Stone, Ruin), forming **10
two-resonance archetypes** (one per pair). Players see 4 cards per draft pick
and select 1.

For full game rules, see [battle_rules.md](../battle_rules/battle_rules.md). For
quest structure, see [quests.md](../plans/quests/quests.md). For the existing
resonance system (to be redesigned), see
[resonance_and_tags.md](../plans/quests/resonance_and_tags.md). For the existing
draft simulation code, see
[draft_simulation.md](../draft_simulation/draft_simulation.md).

### Current System Summary

The existing design uses an **exponential affinity** formula:
`weight = floor + sum(profile[r]^exponent)` where `profile[r]` is how many cards
of resonance `r` the player has drafted. This produces weighted random sampling
that converges toward 2 dominant resonances. The system is implemented as a
Python simulation at `scripts/draft_sim/`.

### Card Distribution

- ~70% of cards carry exactly 1 resonance (mono)
- ~10% carry exactly 2 resonances (dual)
- ~20% carry 0 resonances (neutral)
- Each resonance appears on roughly equal numbers of cards
- Rarity distribution: 55% common, 25% uncommon, 15% rare, 5% legendary
- Copy counts by rarity: common 4, uncommon 3, rare 2, legendary 1
- Total pool entries: ~1000 (360 unique cards x rarity-based copies)

### Draft Structure

- 7 dreamscapes per quest
- Dreamscapes 0-1: 2 draft sites each (5 picks of 4 cards per site)
- Dreamscapes 2-3: 1 draft site each
- Dreamscapes 4-6: 0 draft sites (battle rewards only)
- Each dreamscape also has 1 battle reward (rare+ pick from 3)
- ~15% chance each draft site becomes a shop (6 cards offered, buy multiple)
- Total picks per quest: ~30

### What Is Being Redesigned

This is a **greenfield redesign** of the draft resonance system. Open questions
include: how many resonance types there should be, how cards should be
distributed across them, and how draft packs should be generated. The current
5-resonance / 10-archetype structure is a starting assumption, not a constraint
— agents are free to propose alternatives if the data supports it.

The core question: given a pool of cards and a player who has drafted N cards so
far, how should we select the 4 cards to show them, and what underlying
resonance structure makes this work best?

## Design Goals

Ranked by priority:

1. **Simple.** Explainable to players in one sentence.
2. **Not on rails.** The player should not be forced into one archetype or have
   only 1 real choice per pack.
3. **No forced decks.** The player should not be able to force the same deck
   every time they play.
4. **Flexible archetypes.** It should be possible to build decks outside the
   core archetypes, or combine 2 archetypes.
5. **Convergent.** If you HAVE committed to an archetype (around pick 6), you
   should see a minimum of 2 cards from that archetype most of the time.
6. **Splashable.** You should see around 1 card from outside your archetype in
   most draft picks.
7. **Open-ended early.** In the first ~5 picks, you should see a variety of
   cards from different archetypes.
8. **Signal reading.** There should be a moderate benefit to figuring out which
   archetype is over-represented in the starting pool.

### Measurable Targets

| Metric                                               | Target                 |
| ---------------------------------------------------- | ---------------------- |
| Picks 1-5: unique resonances seen per pack           | >= 3 of 5 on average   |
| Picks 1-5: on-color cards per pack                   | \<= 2 of 4 on average  |
| Picks 6+: on-color cards per pack (committed player) | >= 2 of 4 on average   |
| Picks 6+: off-color cards per pack                   | >= 0.5 of 4 on average |
| Final deck top-2 share (synergy player)              | 75-90%                 |
| Final deck top-2 share (power chaser)                | 60-85%                 |
| Deck variety across runs (same dreamcaller)          | < 40% card overlap     |
| Convergence pick (top-2 > 75%)                       | Pick 5-8               |
| Archetype pair frequency                             | No pair > 15% or < 5%  |

## Investigation Areas

Each agent is assigned a broad area to investigate. They are expected to develop
their OWN specific algorithm within this area — the area provides direction, not
a solution. Agents should reason about what concrete approach best serves the
design goals, not simply implement the first idea that fits their area.

### Area 1: Continuous Weighting Functions

Explore the design space of mathematical functions that map player state to card
selection probabilities. Any formula is fair game — exponential, logarithmic,
sigmoid, piecewise, polynomial, etc. The agent should reason about what
mathematical properties the ideal weighting function should have (convexity,
convergence rate, sensitivity to early vs. late picks) and propose a specific
function backed by that reasoning. Consider how the function shape affects each
design goal and whether different goals require different curve characteristics.

### Area 2: Structured Pack Composition

Explore systems where each draft pack has explicit composition rules rather than
emerging purely from probability. This could mean guaranteed slots, fixed
ratios, or pack "types" that rotate. The agent should reason about how much
structure is helpful vs. constraining, how the structure should evolve as the
draft progresses, and how to avoid the system feeling mechanical or predictable.
The key tension: structure provides guarantees (convergence, splash) but can
reduce surprise and discovery.

### Area 3: Pool and Environment Manipulation

Explore systems where the draft experience is shaped by modifying the pool
itself rather than weighting selection from it. This could mean pre-seeding the
pool with variance, dynamically adding/removing cards, depletion effects that
create natural concentration, or separate sub-pools. The agent should
investigate whether convergence can emerge from environmental properties rather
than player modeling, and how this interacts with the signal-reading goal.
Consider how real-world card game drafts (where players read signals from what's
available, not from personal preferences) can inform the design.

### Area 4: Game Design Precedent Research

Research how existing roguelike deckbuilders, TCG draft formats, and auto-
battlers handle card selection and draft convergence. Study at least 5-6 real
games (e.g., Slay the Spire, Monster Train, Inscryption, MTG Booster Draft,
Hearthstone Arena, TFT/auto-battlers, Balatro). Identify common patterns, known
failure modes, and innovative solutions. Then propose a specific algorithm for
Dreamtides adapted from the most relevant precedents. The proposal should
explain what was borrowed, what was adapted, and why.

### Area 5: Adaptive and Responsive Systems

Explore systems that change their behavior based on the player's observed
choices, not just their accumulated deck state. This could mean tracking what
the player picks vs. passes, detecting commitment pace, adjusting convergence
speed based on player behavior, or introducing feedback loops. The agent should
reason about the difference between "responding to deck composition" (what the
current system does) and "responding to player behavior" (choices, hesitation
patterns, pick-pass ratios). Consider how the system can feel responsive and
intelligent without being manipulative or opaque.

## Inputs

| File                                        | Description                                        |
| ------------------------------------------- | -------------------------------------------------- |
| `docs/plans/quests/resonance_and_tags.md`   | Current resonance system design (being redesigned) |
| `docs/plans/quests/quests.md`               | Quest structure, draft sites, dreamscape layout    |
| `docs/draft_simulation/draft_simulation.md` | Existing Python simulation documentation           |
| `scripts/draft_sim/ds_algorithm.py`         | Current algorithm implementation                   |
| `scripts/draft_sim/ds_models.py`            | Data models (Resonance, SimCard, etc.)             |
| `scripts/draft_sim/simulation.py`           | Quest simulation loop                              |
| `scripts/draft_sim/output.py`               | Output formatting and metrics                      |
| `scripts/draft_sim/draft_sim.py`            | CLI entry point                                    |
| `docs/battle_rules/battle_rules.md`         | Game rules reference                               |

## Rounds

______________________________________________________________________

### Round 1: Strategy Development (5 parallel agents)

Each agent reads the existing resonance docs, draft sim code, and game context,
then develops a concrete algorithm proposal within their assigned investigation
area. No simulation code yet — this round is pure reasoning and design.

**All agents read:** `docs/plans/quests/resonance_and_tags.md`,
`docs/plans/quests/quests.md`, `docs/draft_simulation/draft_simulation.md`,
`scripts/draft_sim/ds_models.py`, `scripts/draft_sim/ds_algorithm.py`,
`scripts/draft_sim/simulation.py`

**Each agent must produce:**

- A specific, implementable algorithm (not just a direction)
- A one-sentence player-facing explanation
- Analysis of how the proposal performs against each of the 8 design goals
- Identification of the proposal's biggest strengths and risks
- Key parameters and predicted sensitivity
- At least 2 worked examples showing pick-by-pick behavior

**Agent 1** — Investigates Area 1 (Continuous Weighting Functions)

- Output: `/tmp/resonance_redesign/strategy_1_proposal.md`

**Agent 2** — Investigates Area 2 (Structured Pack Composition)

- Output: `/tmp/resonance_redesign/strategy_2_proposal.md`

**Agent 3** — Investigates Area 3 (Pool and Environment Manipulation)

- Output: `/tmp/resonance_redesign/strategy_3_proposal.md`

**Agent 4** — Investigates Area 4 (Game Design Precedent Research)

- Output: `/tmp/resonance_redesign/strategy_4_proposal.md`

**Agent 5** — Investigates Area 5 (Adaptive and Responsive Systems)

- Output: `/tmp/resonance_redesign/strategy_5_proposal.md`

______________________________________________________________________

### Round 2: Cross-Strategy Critique (5 agents as team)

All 5 agents reconvene as an interactive team. Each reads all 5 proposals and
engages in structured debate before any code is written.

**All agents read:** All 5 strategy proposals from Round 1
(`/tmp/resonance_redesign/strategy_*_proposal.md`)

**Task for each agent:**

1. Write a position paper identifying:
   - The 2 strongest design goals their strategy serves
   - The 2 weakest design goals and how they might be addressed
   - The biggest risk their strategy poses to draft quality
2. Send at least 3 messages to EACH other agent (minimum 60 total messages)
   discussing:
   - Which strategies are surprisingly similar under the hood?
   - Which strategies are genuinely incompatible and WHY?
   - Can any two strategies be combined? What would a hybrid look like?
   - Which strategy best serves each individual design goal?
   - Concrete failure modes: describe a specific draft scenario where each
     strategy would produce a bad experience
3. After discussion, each agent writes a revised position paper incorporating
   insights from the debate. The agent may substantially revise their proposal
   based on what they learned.

**Output per agent:**

- `/tmp/resonance_redesign/critique_{N}.md` (position paper + revised thinking)

**Minimum messages:** 60 total (each agent sends 3+ to each other agent)

______________________________________________________________________

### Round 3: Simulation Implementation (5 parallel agents)

Each agent writes a standalone Python simulation of their strategy. The
simulation must use the same card pool generation, quest structure, and player
strategies as the existing `scripts/draft_sim/` codebase but replace the core
pack selection algorithm.

**All agents read:** `scripts/draft_sim/ds_models.py`,
`scripts/draft_sim/ds_algorithm.py`, `scripts/draft_sim/simulation.py`,
`scripts/draft_sim/output.py`, `scripts/draft_sim/draft_sim.py`, plus their own
critique from Round 2 (`/tmp/resonance_redesign/critique_{N}.md`)

**Requirements for each simulation:**

1. Reuse `ds_models.py` types (Resonance, SimCard, PoolEntry, etc.)
2. Implement a
   `select_pack(pool, profile, params, rng, pick_number) -> list[card]` function
   that returns 4 cards
3. Support all 3 player strategies (synergy, power_chaser, rigid)
4. Run 1000 quests and collect metrics:
   - Per-pick: unique resonances seen, on-color count, off-color count
   - Per-quest: convergence pick, top-2 share, HHI, deck classification
   - Across quests: archetype pair frequency, deck overlap between runs
5. Produce output comparing results to the measurable targets from the Design
   Goals section
6. Include a parameter sweep over 3-5 key parameters

**Output per agent:**

- `/tmp/resonance_redesign/sim_{N}.py` (simulation code)
- `/tmp/resonance_redesign/results_{N}.md` (empirical results + analysis)

______________________________________________________________________

### Round 4: Empirical Analysis & Goal Scoring (5 parallel agents)

Each agent runs their simulation from Round 3 with additional analysis focused
specifically on the 8 design goals. They also run each other's simulations for
comparison.

**All agents read:** All 5 result files
(`/tmp/resonance_redesign/results_*.md`), all 5 simulation files
(`/tmp/resonance_redesign/sim_*.py`), plus the design goals section of this plan

**Task for each agent:**

1. Run their OWN simulation with:

   - Parameter sweeps focused on hitting the measurable targets
   - Edge case analysis: mono dreamcaller, power-chaser player, very first/last
     picks
   - "Draft story" traces: 3 example drafts showing pick-by-pick experience

2. Run ALL OTHER simulations with default parameters to produce an apples-to-
   apples comparison

3. Score each strategy on each design goal (1-10) with justification

4. Produce a comparison table:

   | Goal         | S1  | S2  | S3  | S4  | S5  |
   | ------------ | --- | --- | --- | --- | --- |
   | Simple       | ?   | ?   | ?   | ?   | ?   |
   | Not on rails | ?   | ?   | ?   | ?   | ?   |
   | ...          |     |     |     |     |     |

5. Identify which parameters in their own strategy are most sensitive and which
   are robust

**Output per agent:**

- `/tmp/resonance_redesign/analysis_{N}.md` (goal scoring + comparison)
- `/tmp/resonance_redesign/sweep_{N}.md` (parameter sensitivity results)

______________________________________________________________________

### Round 5: Cross-Pollination & Hybrid Design (5 agents as team)

Another interactive team round. Armed with empirical data, agents discuss which
approaches work best for which goals and propose hybrid strategies.

**All agents read:** All 5 analysis files
(`/tmp/resonance_redesign/analysis_*.md`), all 5 sweep files
(`/tmp/resonance_redesign/sweep_*.md`)

**Task:**

1. Each agent reviews the empirical evidence and identifies:
   - Which of the 8 goals their strategy is best at (by the numbers)
   - Which goals require a different approach entirely
   - Surprising results — where did their predictions from Round 1 differ from
     the simulation?
2. Team discussion (minimum 60 messages total):
   - Propose hybrid strategies that combine the best elements
   - Debate trade-offs: can you serve all 8 goals simultaneously?
   - Identify the "Pareto frontier" — which goals are in genuine tension?
   - Converge on 2-3 promising hybrid designs to implement in Round 6
3. Each agent writes a revised proposal incorporating hybrid ideas

**Output per agent:**

- `/tmp/resonance_redesign/hybrid_proposal_{N}.md` (revised strategy with hybrid
  elements)

**Minimum messages:** 60 total

______________________________________________________________________

### Round 6: Revised Implementations (5 parallel agents)

Each agent revises their simulation based on Round 5 feedback. The revised
simulation should incorporate the best hybrid ideas while maintaining each
strategy's core identity.

**All agents read:** All 5 hybrid proposals
(`/tmp/resonance_redesign/hybrid_proposal_*.md`), their own simulation
(`/tmp/resonance_redesign/sim_{N}.py`), and any simulation code from other
agents they want to borrow from

**Requirements:**

1. Implement the revised algorithm
2. Run 1000 quests with all 3 player strategies
3. Run targeted parameter sweeps on new parameters introduced by hybrid elements
4. Produce "draft story" traces for 3 carefully chosen scenarios:
   - A player who commits early to a strong archetype
   - A player who stays open for 8+ picks then commits
   - A power-chaser who ignores resonance entirely
5. Compare revised results to Round 3 results — what improved? What regressed?
6. Final self-assessment: score each design goal 1-10 with the revised system

**Output per agent:**

- `/tmp/resonance_redesign/sim_{N}_v2.py` (revised simulation code)
- `/tmp/resonance_redesign/results_{N}_v2.md` (revised results + comparison)

______________________________________________________________________

### Round 7: Final Synthesis (1 agent)

A single agent reads all outputs and produces the definitive comparison report.

**Reads:** All Round 6 outputs (`/tmp/resonance_redesign/results_*_v2.md`,
`/tmp/resonance_redesign/sim_*_v2.py`), all analysis files from Round 4
(`/tmp/resonance_redesign/analysis_*.md`), all hybrid proposals from Round 5
(`/tmp/resonance_redesign/hybrid_proposal_*.md`), and the design goals from this
plan

**Task:**

1. Run all 5 revised simulations with identical parameters and produce a unified
   comparison table
2. For each strategy, compute all measurable targets and flag pass/fail
3. Rank the 5 strategies by overall goal satisfaction
4. For each design goal, identify which strategy is best and why
5. Identify the "recommended strategy" — the one that best balances all goals
6. Produce a "player-facing explanation" for each strategy (the one-sentence
   version and a one-paragraph version)
7. Identify remaining open questions and suggested follow-up experiments
8. Write a clear recommendation with confidence level

**Output:**

- `/tmp/resonance_redesign/final_report.md`

## Agent Summary

| Round     | Agents | Type                | Description                                     |
| --------- | ------ | ------------------- | ----------------------------------------------- |
| 1         | 5      | Parallel background | Strategy development within investigation areas |
| 2         | 5      | Team (interactive)  | Cross-strategy critique and debate              |
| 3         | 5      | Parallel background | Simulation implementation                       |
| 4         | 5      | Parallel background | Empirical analysis and goal scoring             |
| 5         | 5      | Team (interactive)  | Cross-pollination and hybrid design             |
| 6         | 5      | Parallel background | Revised implementations                         |
| 7         | 1      | Background          | Final synthesis report                          |
| **Total** | **31** |                     |                                                 |

## Output Files

| File                            | Round | Description                               |
| ------------------------------- | ----- | ----------------------------------------- |
| `strategy_{N}_proposal.md` (x5) | 1     | Strategy proposals per investigation area |
| `critique_{N}.md` (x5)          | 2     | Position papers + revised thinking        |
| `sim_{N}.py` (x5)               | 3     | Simulation implementations                |
| `results_{N}.md` (x5)           | 3     | Initial empirical results                 |
| `analysis_{N}.md` (x5)          | 4     | Goal scoring + comparison tables          |
| `sweep_{N}.md` (x5)             | 4     | Parameter sensitivity results             |
| `hybrid_proposal_{N}.md` (x5)   | 5     | Hybrid strategy proposals                 |
| `sim_{N}_v2.py` (x5)            | 6     | Revised simulation code                   |
| `results_{N}_v2.md` (x5)        | 6     | Revised results + comparison              |
| `final_report.md`               | 7     | Definitive comparison and recommendation  |

All output files are in `/tmp/resonance_redesign/`.

## Recovery

Check which `/tmp/resonance_redesign/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained — later rounds depend only on
earlier outputs.
