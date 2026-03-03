# Archetype Draft System Design — Orchestration Plan

## Lessons from V1

This is V2 of the draft system design project. V1 produced useful simulation
infrastructure and algorithm insights, but the agents misframed the problem:
they assumed a fixed "5 resonance colors / 10 color-pair archetypes" structure
(like Magic: The Gathering) and spent all their effort optimizing the weighting
formula within that structure. They measured "on-color cards per pack" when the
real question was about **archetype-fitting cards per pack**.

For V2, the key shift is:

- **Start from archetypes, not colors.** An archetype is a strategic deck theme
  (like "Reanimator" or "Tokens"), not a color pair. Cards have explicit fitness
  scores per archetype (S/A/B/C/F). The number of archetypes, how cards relate
  to them, and whether any color/resonance system is needed are all open
  questions.

- **Explore the structure, not just the formula.** V1 converged quickly on
  "exponential weighting + lane seeds" and spent 5 rounds tuning parameters. V2
  should spend more time questioning whether the right structural foundation is
  in place before optimizing within it.

- **Measure archetype fitness, not color matching.** "2+ archetype cards per
  pack" means 2+ cards with S/A-tier fitness in the player's specific archetype
  — not cards that happen to share a tag or color.

You're welcome to draw on V1's simulation techniques or conclude that a
resonance-like system is part of the answer, but don't let that be your starting
assumption. Start from the archetype fitness model and see where the analysis
leads.

______________________________________________________________________

## The Problem

Dreamtides is a roguelike deckbuilding game. During a quest, players draft a
deck of ~30 cards from a shared pool of **360 unique cards**. Players see 4
cards per pick and select 1.

There are somewhere between **7 and 10 draft archetypes**. Each card has a
fitness score in each archetype:

- Most cards are strong (S-tier) in exactly 1 archetype
- Most cards are also playable (A/B-tier) in a few other archetypes
- A few rare cards are strong in multiple archetypes or universally strong
- Some cards are narrow specialists (S-tier in 1, unplayable elsewhere)

The core design question: **How should the draft system be structured so that
players have a good drafting experience?** This encompasses everything:

- How many archetypes should there be?
- What card-to-archetype fitness distribution works best?
- How should the system detect and respond to a player's emerging archetype?
- How should draft packs be constructed?
- What makes runs feel different from each other?

There is NO existing system to constrain this exploration. Agents should not
reference "resonance" or "colors" — those are possible solutions, not givens.
The only way to think about card identity for this exercise is through
**archetype fitness scores**.

## Draft Structure

**Fixed parameters:**

- ~360 unique cards in the pool
- 4 cards per draft pack, pick 1
- ~30 total picks per quest

**Flexible parameters (current defaults, open to revision):**

- Rarity distribution: 55% common, 25% uncommon, 15% rare, 5% legendary
- Copy counts by rarity: common 4, uncommon 3, rare 2, legendary 1
- Total pool entries: ~1000 (360 unique cards x rarity-based copies)

Agents may propose changes to the rarity system, copy counts, or pool structure
if their analysis suggests a different approach works better.

## Design Goals

Ranked by priority:

1. **Simple.** Explainable to players in one sentence.
2. **Not on rails.** The player should not be forced into one archetype or have
   only 1 real choice per pack.
3. **No forced decks.** The player should not be able to force the same deck
   every time they play.
4. **Flexible archetypes.** It should be possible to build decks outside the
   core archetypes, or combine 2 archetypes.
5. **Convergent.** If you HAVE committed to an archetype (around pick 6 on
   average), you should see a minimum of 2 cards from that archetype most of the
   time.
6. **Splashable.** You should see around 1 card from outside your archetype in
   most draft picks.
7. **Open-ended early.** In the first ~5 picks, you should see a variety of
   cards from different archetypes.
8. **Signal reading.** There should be a moderate benefit to figuring out which
   archetype is over-represented in the starting pool.

### What "From Your Archetype" Means

A card is "from your archetype" if it has a high fitness score (S or A tier) in
the archetype you're pursuing. This is NOT the same as sharing a color or
resonance tag. An archetype is a strategic theme — a coherent deck strategy that
a subset of cards supports. Cards can be S-tier in one archetype and B-tier in
another; the system needs to understand this graded relationship.

### Measurable Targets

These targets are framed in terms of archetype fitness, NOT colors:

| Metric                                                        | Target                     |
| ------------------------------------------------------------- | -------------------------- |
| Picks 1-5: unique archetypes represented per pack             | >= 3 of 4 on average       |
| Picks 1-5: cards fitting player's emerging archetype per pack | \<= 2 of 4                 |
| Picks 6+: cards fitting committed archetype per pack          | >= 2 of 4 on average       |
| Picks 6+: strong off-archetype cards per pack                 | >= 0.5 of 4 on average     |
| Convergence pick (player regularly sees 2+ archetype cards)   | Pick 5-8                   |
| Deck archetype concentration (committed player)               | 60-80% S/A-tier cards      |
| Run-to-run variety (same starting conditions)                 | < 40% card overlap         |
| Archetype frequency across runs                               | No archetype > 20% or < 5% |

"Fitting" a committed archetype = S-tier or A-tier fitness in that archetype.
"Strong off-archetype" = high raw power or S-tier in a different archetype.

## Simulation Card Model

Simulated cards have:

```python
class SimCard:
    id: int
    rarity: Rarity          # common/uncommon/rare/legendary
    power: float             # raw card strength (0-10)
    archetype_fitness: dict  # archetype_id -> fitness_tier (S/A/B/C/F)
```

**Fitness tiers:**

- **S** = core card for this archetype (designed for it)
- **A** = strong in this archetype (good synergy)
- **B** = playable (acceptable filler)
- **C** = weak (would rather not play it)
- **F** = unplayable in this archetype

The DISTRIBUTION of fitness scores across cards is a design parameter that
agents should explore. Example distributions for a single card with N=8
archetypes: "S in 1, A in 2, B in 2, C in 2, F in 1" vs. "S in 1, B in 4, F in
3" vs. "A in 3, B in 5" (generalist). The mix of specialist vs. generalist cards
is critical.

### Multi-Archetype Card Design Cost

A key real-world constraint: designing cards that are S or A tier in MORE than
one archetype is very difficult. It requires finding mechanical overlap between
archetypes and creating cards that serve both strategies naturally without
feeling forced. It's great game design when it works, but it's a significant
design burden per card.

Agents should treat the NUMBER of multi-archetype cards (S or A in 2+
archetypes) as a critical output of their analysis. Specifically:

- What is the MINIMUM number of multi-archetype cards needed for the system to
  work? Can it function with zero? With 10%? Does it need 30%?
- How sensitive are the design goals to this percentage? Does convergence fall
  apart without enough multi-archetype cards, or is it mostly about flexibility
  and splashing?
- Is there a difference between "S in 2 archetypes" (true dual-archetype star)
  and "S in 1, A in 2" (specialist with splash value)?

This is one of the most important practical questions for the game designer: how
much multi-archetype card design work is required to make the draft system
function well?

## Simulated Player Strategies

- **Archetype-committed:** Picks cards with highest fitness in their strongest
  archetype. Commits to an archetype once they've picked 3+ S/A cards in it.
- **Power-chaser:** Picks the highest raw power card regardless of archetype.
- **Signal-reader:** Evaluates which archetype seems most available (most cards
  offered) and drafts toward the open archetype.

## Investigation Areas

Each agent in Round 1 is assigned a broad **question** to investigate. These are
different dimensions of the design space, NOT different solutions. Agents are
expected to reason deeply about their question, map the possibility space,
identify key tradeoffs, and surface non-obvious insights.

### Question 1: The Cardinality Problem

How does the number of archetypes (N) interact with the fixed draft structure
(360 cards, 4 per pack, 30 picks)?

Explore this mathematically and intuitively. Consider:

- With N archetypes and 360 cards, how many cards per archetype (at S/A tier)?
  What's the probability of seeing 2+ in a random 4-card pack?
- Is there a "sweet spot" for N where convergence is achievable but variety is
  preserved?
- Does N affect how *different* archetypes feel from each other? (Fewer
  archetypes → more cards per archetype → more internal variety within each?)
- Are there non-obvious N values worth considering? What about N=3 where each
  archetype is huge? N=15 where archetypes are tiny?
- How does N interact with the other design goals (simplicity, flexibility,
  signal reading)?

Do NOT propose a specific algorithm. Map the design space and identify the key
tradeoffs at different N values.

### Question 2: The Fitness Distribution Problem

How should cards be distributed across archetypes, and how does this
distribution affect draft dynamics?

This is about the card FITNESS MATRIX — the big spreadsheet of "card X has
fitness Y in archetype Z." Explore:

- What percentage of cards should be narrow specialists (S in 1, F in most)?
  What about broad generalists (B+ in many)? Multi-archetype stars (S in 2+)?
- How does the specialist-generalist spectrum affect convergence speed,
  flexibility, and the "on rails" feeling?
- Should all archetypes have the same number of S-tier cards, or should some be
  deeper than others?
- How does card overlap between archetypes affect the experience? If archetypes
  share too many playable cards, do they feel distinct?
- What's the relationship between the fitness distribution and the minimum
  viable archetype — how few S-tier cards can an archetype have before it stops
  working?

Do NOT propose a specific algorithm. Characterize the design space of fitness
distributions and their consequences.

### Question 3: The Pack Construction Problem

Given that a player has committed to an archetype, how should 4-card packs be
assembled? This is the ALGORITHM question — but explored broadly, not as a
single proposal.

Explore FUNDAMENTALLY DIFFERENT approaches to pack construction:

- Weighted random sampling (the obvious approach). What are its limits?
- Deterministic guarantees (e.g., "always 2 fitting + 1 splash + 1 random")
- Sub-pool systems (maintaining separate pools per archetype)
- Cube-style pre-constructed packs
- Depletion-based systems (archetype pools shrink as you draft them)
- Any other structural approach you can think of

For EACH approach, reason about how it serves or undermines each design goal.
Identify which approaches are fundamentally incompatible with which goals.
Consider approaches that make the system transparent vs. opaque to the player.

Do NOT recommend one approach. Map the landscape of possible mechanisms.

### Question 4: The Variety and Signaling Problem

How do you ensure that two runs feel different from each other, and that
observant players can read signals about what's available?

This is about the META-STRUCTURE around drafting. Explore:

- What makes runs feel same-y vs. varied? (Same cards? Same archetype? Same
  decision points? Same deck structure?)
- How can the system create per-run asymmetries without making some runs unfair?
- What signals can the system send to observant players? How explicit should
  they be?
- Is there a tension between variety and fairness? Between variety and
  learnability?
- How do existing roguelikes create replayability? What's specific to card
  drafting vs. other roguelike systems?
- Should the player be able to influence which archetypes are available, or
  should it be entirely system-driven?

Do NOT propose a specific algorithm. Explore the design space of variety and
information mechanics.

## Rounds

______________________________________________________________________

### Round 1: Design Space Mapping (4 parallel agents)

Each agent investigates their assigned question in depth. This round is pure
reasoning and analysis — no simulation code. Agents should think broadly,
consider extreme positions, and surface non-obvious tradeoffs.

**All agents read:** This orchestration plan (for context and design goals).

The V1 files in `/tmp/resonance_redesign/` and `scripts/draft_sim/` describe a
color-based system. You may glance at them for simulation technique inspiration,
but do not adopt their framing. Think from first principles about archetypes and
fitness scores. See "Lessons from V1" above.

**Each agent must produce (max 1500 words):**

- A thorough exploration of their question's design space
- Key tradeoffs and tensions identified
- At least 3 "surprising insights" — things that aren't obvious at first
- Specific dimensions or parameters that Round 2 simulations should test
- Concrete predictions: "I predict that X will produce Y effect"
- A **"Key Takeaways" section** at the top: 5-7 bullet points summarizing the
  most important findings (this is what later rounds will rely on most)

**Agent 1** — Investigates Question 1 (The Cardinality Problem)

- Output: `docs/resonance/v2/q1_cardinality.md`

**Agent 2** — Investigates Question 2 (The Fitness Distribution Problem)

- Output: `docs/resonance/v2/q2_fitness_distribution.md`

**Agent 3** — Investigates Question 3 (The Pack Construction Problem)

- Output: `docs/resonance/v2/q3_pack_construction.md`

**Agent 4** — Investigates Question 4 (The Variety and Signaling Problem)

- Output: `docs/resonance/v2/q4_variety_signaling.md`

______________________________________________________________________

### Round 2: Model Design and Simulation (4 parallel agents)

Each agent reads ALL Round 1 outputs, then designs and implements a COMPLETE
draft system. Each agent's system should represent a genuinely different
approach — not minor parameter variations.

The intent is that each agent synthesizes the Round 1 findings into a coherent
system. They should pick a specific N (number of archetypes), a specific fitness
distribution, a specific pack construction mechanism, and a specific variety
mechanism. These should be meaningfully different from each other — the agents
should deliberately span the design space.

**All agents read:** All 4 Round 1 outputs
(`docs/resonance/v2/q1_cardinality.md` through `q4_variety_signaling.md`)

Your card model must use explicit per-archetype fitness scores (S/A/B/C/F). See
"Lessons from V1" — make sure you're measuring archetype fitness, not color
matching.

**Each agent must produce:**

1. A complete system design document (**max 1000 words**) explaining:
   - How many archetypes and why
   - The card fitness distribution (including % of multi-archetype cards)
   - How packs are constructed
   - How variety is created across runs
   - One-sentence player-facing explanation
2. A Python simulation implementing the system that:
   - Generates a pool of 360 cards with the specified fitness distribution
   - Simulates 1000 drafts of 30 picks each
   - Runs all 3 player strategies
   - Measures all 8 metrics from the measurable targets table
   - Includes 3 detailed draft traces (pick-by-pick for illustrative runs)
3. Results document (**max 800 words**) structured as:
   - A **target scorecard table** (metric | target | actual | pass/fail)
   - A **multi-archetype card sensitivity** section: how do results change when
     you vary the % of cards that are S/A in 2+ archetypes?
   - Brief analysis of what works and what doesn't

**Concrete agent assignments:**

**Agent A** — Design and simulate a system with a SMALL number of archetypes
(3-5). Lean into the "big archetypes, lots of internal variety" direction.

- Output: `docs/resonance/v2/model_a_design.md`,
  `docs/resonance/v2/model_a_sim.py`, `docs/resonance/v2/model_a_results.md`

**Agent B** — Design and simulate a system with a LARGE number of archetypes
(8-12). Lean into the "many distinct archetypes, tighter card pools" direction.

- Output: `docs/resonance/v2/model_b_design.md`,
  `docs/resonance/v2/model_b_sim.py`, `docs/resonance/v2/model_b_results.md`

**Agent C** — Design and simulate a system that uses a NON-STANDARD pack
construction mechanism (not weighted random sampling). Try something
structurally different — guaranteed archetype slots, curated packs, sub-pools,
depletion, or whatever else seems promising based on Round 1 findings.

- Output: `docs/resonance/v2/model_c_design.md`,
  `docs/resonance/v2/model_c_sim.py`, `docs/resonance/v2/model_c_results.md`

**Agent D** — Design and simulate a system that prioritizes VARIETY AND SIGNAL
READING above other goals. Whatever N and mechanism you choose should be
optimized for making runs feel different and rewarding observation.

- Output: `docs/resonance/v2/model_d_design.md`,
  `docs/resonance/v2/model_d_sim.py`, `docs/resonance/v2/model_d_results.md`

______________________________________________________________________

### Round 3: Cross-Comparison and Debate (4 agents as team)

All 4 agents reconvene as an interactive team. Each reads all 4 designs and
simulation results, then engages in structured discussion.

**All agents read:** All model designs, simulations, and results
(`docs/resonance/v2/model_*_design.md`, `docs/resonance/v2/model_*_results.md`)

**Task for each agent:**

1. Score each model on each of the 8 design goals (1-10), with 1-sentence
   justification for each score.
2. Identify the single biggest strength and biggest weakness of each model.
3. Propose specific modifications or hybrid ideas — what elements from different
   models could be combined?
4. Address specific questions to other agents about their design choices.

**Discussion topics (minimum 40 total messages):**

- Which models actually hit the "2+ archetype cards per pack" convergence
  target? If none do, why not? What would need to change?
- Is there a fundamental tension between any design goals that means you CAN'T
  satisfy all 8 simultaneously? Which goals are in real conflict?
- What did the simulations reveal that was surprising or different from Round 1
  predictions?
- For each goal, which model is best? Can you take "the best element for each
  goal" and combine them?

**Output per agent (max 1000 words each):**

- `docs/resonance/v2/debate_{A|B|C|D}.md` (analysis + hybrid proposals)

Each debate output should include a structured **scorecard table** (model x goal
matrix with 1-10 scores) at the top, followed by prose analysis. Keep it concise
— the Round 4 and Round 5 agents need to read all 4 of these.

______________________________________________________________________

### Round 4: Refined Simulations (4 parallel agents)

Each agent revises their model based on Round 3 discussion, incorporating the
best hybrid ideas. Agents may substantially change their approach.

**All agents read:** All 4 debate outputs (`docs/resonance/v2/debate_*.md`),
plus any model designs/sims they want to borrow from

**Requirements:**

1. Implement the revised system
2. Run 1000 drafts with all 3 player strategies
3. Run parameter sensitivity sweeps on 2-3 key parameters, MUST include varying
   the % of multi-archetype cards (S/A in 2+ archetypes)
4. Produce 3 draft story traces for:
   - A player who commits to an archetype early (by pick 4-5)
   - A player who stays flexible for 8+ picks
   - A signal-reader who identifies the "open" archetype
5. Compare to Round 2 results — what improved? What regressed?
6. Final self-assessment: score each design goal 1-10

**Output per agent:**

- `docs/resonance/v2/model_{A|B|C|D}_v2_design.md` (**max 1000 words**)
- `docs/resonance/v2/model_{A|B|C|D}_v2_sim.py`
- `docs/resonance/v2/model_{A|B|C|D}_v2_results.md` (**max 800 words**,
  structured with target scorecard table at top)

Each results doc MUST begin with a **200-word executive summary** covering: the
model's core idea in 1-2 sentences, the target scorecard (pass/fail counts), the
biggest strength, the biggest weakness, and the minimum % of multi-archetype
cards needed. This summary is what the Round 5 synthesizer will primarily rely
on.

______________________________________________________________________

### Round 5: Final Synthesis (1 agent)

A single agent produces the definitive comparison.

**Reads:** To stay within context limits, read in this priority order:

1. This orchestration plan (design goals and measurable targets)
2. The **executive summaries** at the top of each Round 4 results doc
   (`docs/resonance/v2/model_*_v2_results.md` — read the first 200 words)
3. The Round 4 design docs (`docs/resonance/v2/model_*_v2_design.md`)
4. The Round 3 debate scorecards (`docs/resonance/v2/debate_*.md` — read the
   scorecard tables at the top)
5. Only if needed for specific details: the full results docs or sims

**Task:**

1. Run all 4 revised simulations with identical parameters and produce a unified
   comparison table
2. For each model, compute all measurable targets and flag pass/fail
3. Rank the 4 models by overall design goal satisfaction
4. Identify the "recommended approach" with confidence level
5. Write player-facing explanations (1-sentence and 1-paragraph)
6. For each model, report the minimum % of multi-archetype cards needed
7. Identify remaining open questions and what playtesting should focus on
8. Produce a clear implementation specification for the recommended approach

**Output (max 2000 words):**

- `docs/resonance/v2/final_report.md`

## Agent Summary

| Round     | Agents | Type                | Description                       |
| --------- | ------ | ------------------- | --------------------------------- |
| 1         | 4      | Parallel background | Design space mapping per question |
| 2         | 4      | Parallel background | Model design + simulation         |
| 3         | 4      | Team (interactive)  | Cross-comparison and debate       |
| 4         | 4      | Parallel background | Refined simulations               |
| 5         | 1      | Background          | Final synthesis report            |
| **Total** | **17** |                     |                                   |

## Output Files

| File                                 | Round | Description                              |
| ------------------------------------ | ----- | ---------------------------------------- |
| `q1_cardinality.md`                  | 1     | Cardinality design space analysis        |
| `q2_fitness_distribution.md`         | 1     | Fitness distribution analysis            |
| `q3_pack_construction.md`            | 1     | Pack construction mechanism analysis     |
| `q4_variety_signaling.md`            | 1     | Variety and signaling analysis           |
| `model_{A,B,C,D}_design.md` (x4)     | 2     | System design documents                  |
| `model_{A,B,C,D}_sim.py` (x4)        | 2     | Simulation implementations               |
| `model_{A,B,C,D}_results.md` (x4)    | 2     | Initial empirical results                |
| `debate_{A,B,C,D}.md` (x4)           | 3     | Analysis + hybrid proposals              |
| `model_{A,B,C,D}_v2_design.md` (x4)  | 4     | Revised designs                          |
| `model_{A,B,C,D}_v2_sim.py` (x4)     | 4     | Revised simulations                      |
| `model_{A,B,C,D}_v2_results.md` (x4) | 4     | Revised results                          |
| `final_report.md`                    | 5     | Definitive comparison and recommendation |

All output files are in `docs/resonance/v2/`.

## Key Principles for Agents

1. **Start from archetypes, not colors.** An archetype is a strategic deck
   theme. Cards have explicit per-archetype fitness scores (S/A/B/C/F). You may
   conclude that a color/resonance system helps implement archetypes, but don't
   start there — start from the fitness model and see what structure emerges.
   See "Lessons from V1."

2. **Explore the structure, not just the formula.** The big questions are how
   many archetypes, how cards relate to them, and how packs are built. Don't
   assume a structure and tune parameters within it.

3. **The fitness distribution matters as much as the algorithm.** Even a perfect
   pack construction algorithm can't produce good drafts if the card-archetype
   matrix is wrong.

4. **"From your archetype" is the key metric.** The convergence target is about
   seeing cards that are specifically good in your archetype (S or A tier
   fitness), not cards that vaguely share a trait with your deck.

5. **Simulate to discover, not to confirm.** Use simulations to find surprising
   behaviors and edge cases, not just to check boxes on targets.

## Recovery

Check which `docs/resonance/v2/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained — later rounds depend only on
earlier outputs.
