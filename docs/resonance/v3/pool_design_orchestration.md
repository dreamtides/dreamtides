# Pool Design Orchestration Plan

A multi-step simulation analysis and refinement pipeline for figuring out how
to break up a 360-card draft pool, given a chosen draft algorithm.

## Prerequisites

Before starting this pipeline, you need:

1. **A chosen draft algorithm** with a one-sentence description and complete
   specification (pack construction rules, player-facing state, parameters).
2. **The archetype structure** — how many archetypes, their resonance
   assignments, and the adjacency/circle relationships.
3. **Fitness tier definitions** — how to evaluate whether a card is S/A/B/C/F
   for each archetype (used for measuring outcomes, not by the algorithm).
4. **Measurable targets** — the design goals translated into numbers (e.g.,
   "2+ archetype-fitting cards per pack after pick 6").

## Important: Archetype-Level Evaluation

All metrics must be measured at the **archetype level**, not the resonance
level. A card matching your resonance is NOT the same as a card fitting your
archetype — a resonance is shared by multiple archetypes. Specifically:

- "Cards fitting your archetype" = cards with S-tier or A-tier fitness for the
  player's specific target archetype.
- "Off-archetype cards" = cards with C-tier or F-tier fitness.
- "Unique archetypes per pack" = how many distinct archetypes have at least one
  S/A-tier card represented.

Getting this wrong will inflate convergence numbers and produce misleading
results.

---

## Round 1: Five Parallel Investigations (5 agents, background)

Launch 5 agents in parallel. Each investigates one axis of pool design
independently. Each agent builds its own simulation, tests multiple
configurations, and produces a results document.

All agents share the same draft algorithm implementation — only the pool
construction and evaluation vary.

### Agent 1: Symbol Count Distribution

**Question:** What ratio of 0/1/2/3-symbol cards produces the best draft
experience?

**Method:**
- Test at least 8 distributions from extreme to balanced:
  - All 1-symbol (100/0/0)
  - Heavy 1-symbol (70/20/10)
  - Moderate 1-symbol (50/35/15)
  - Balanced (33/34/33)
  - The current default
  - Heavy 2-symbol (10/80/10)
  - Heavy 3-symbol (10/30/60)
  - All 3-symbol (0/0/100)
- For each, measure:
  - Algorithm-specific state progression (e.g., lock timing, weight growth,
    window fill rate — depends on the chosen algorithm)
  - S/A cards per pack at picks 5, 10, 15, 20, 25, 30 (the convergence curve)
  - How fast the algorithm's convergence mechanism activates
  - Whether state accumulates too fast (instant commitment) or too slow
    (no feedback)
  - "SA Trend" — whether pack quality improves, plateaus, or declines over
    the draft

**Key insight to look for:** More symbols per card means faster state
accumulation, but this can cause accidental/premature commitment and reduce
the quality of late-game packs by spreading state across multiple resonances.

**Output:** `pool_results_1.md` (max 1000 words) + `pool_sim_1.py`

### Agent 2: Rarity System

**Question:** How should rarity interact with the draft algorithm and
resonance symbols?

**Method:**
- Test at least 5 rarity models:
  - Flat rarity (no power differences)
  - Standard TCG (180C/100U/60R/20L, power scales)
  - Roguelike-skewed (more rares/legendaries)
  - Rarity-symbol correlation (rares have more symbols)
  - Inverse correlation (rares have fewer symbols)
- For each, measure:
  - All standard archetype-level targets
  - Draft tension rate — how often the player faces "strong off-archetype
    rare vs weak on-archetype common" decisions
  - Power variance across runs (replayability)
  - Whether rarity interacts with the draft algorithm or is orthogonal

**Key insight to look for:** For most draft algorithms, rarity is orthogonal
to the convergence mechanism. Its value is in creating draft tension and
run-to-run variance, not in shaping pack structure.

**Output:** `pool_results_2.md` (max 1000 words) + `pool_sim_2.py`

### Agent 3: Archetype Breakdown

**Question:** How should cards be distributed across archetypes, and how many
generic/bridge cards should exist?

**Method:**
- Test at least 5 breakdown models:
  - Equal cards per archetype + small generic pool (~10%)
  - Equal + large generic pool (~33%)
  - Equal + explicit bridge card category (cards assigned to two archetypes)
  - Asymmetric archetype sizes (some deeper than others)
  - Mono-symbol only (all cards have just [Primary])
- For each, measure:
  - All standard archetype-level targets
  - S-tier vs A-tier ratio in drafted decks (home archetype vs adjacent)
  - Bridge strategy viability (committing to 2 adjacent archetypes)
  - Whether generics dilute convergence or improve flexibility
  - Per-archetype frequency balance

**Key insight to look for:** Most draft algorithms are surprisingly robust to
pool composition changes. The algorithm's pack construction mechanism tends to
dominate over internal pool structure. But extreme configurations (too many
generics, asymmetric sizes) can break archetype frequency balance.

**Output:** `pool_results_3.md` (max 1000 words) + `pool_sim_3.py`

### Agent 4: Symbol Pattern Composition

**Question:** What specific symbol patterns should cards have, and how do
different patterns affect draft decisions?

**Method:**
- Enumerate all mechanically distinct symbol patterns for a card in a given
  archetype (e.g., for Warriors = Tide/Zephyr):
  - 1-sym: [P], [S]
  - 2-sym: [P,S], [P,P], [S,P], [P,Other]
  - 3-sym: [P,P,S], [P,S,S], [P,S,Other], etc.
- Test at least 5 pattern compositions from uniform to maximally varied
- For each, measure:
  - **Genuine choice rate:** How often a pack contains 2+ S/A cards with
    different symbol patterns, forcing the player to evaluate which pattern
    best serves their strategy. This is the critical "decision quality" metric.
  - Unwanted/accidental commitment rate
  - "Wasted" state (symbols accumulating in already-maxed categories)
  - Lock timing or equivalent algorithm-state progression

**Key insight to look for:** Pattern variety is mandatory for meaningful
decisions. If all cards have the same pattern (e.g., all [P,S]), picking
between them is purely about card power — resonance becomes irrelevant to the
decision. The draft becomes autopilot. Different patterns create "depth vs
breadth" and "commit vs bridge" tensions.

**Output:** `pool_results_4.md` (max 1000 words) + `pool_sim_4.py`

### Agent 5: Algorithm Parameter Tuning

**Question:** How should the draft algorithm's parameters be tuned relative to
the pool design, and what is the ideal progression curve?

**Method:**
- Identify the 2-3 key parameters of the chosen algorithm (e.g., threshold
  values, weight multipliers, window sizes, slot counts).
- Test a matrix of parameter values crossed with 2-3 symbol distributions.
- For each, measure:
  - Algorithm-specific progression timing (e.g., "pick at which first
    threshold fires," "pick at which weights cross 50%")
  - What % of drafts have the algorithm activate on pick 1 (too fast)
  - The full convergence curve
  - S/A per pack at each pick number
  - Decision quality / meaningful choice frequency

**Key insight to look for:** The algorithm's parameters and the pool's symbol
distribution interact — you can tune EITHER to control progression speed, but
the right combination eliminates degenerate cases (instant activation,
never-activating) while creating a satisfying three-act draft arc:
exploration → commitment → refinement.

**Output:** `pool_results_5.md` (max 1000 words) + `pool_sim_5.py`

---

## Round 2: Synthesis (1 agent, background)

A single agent reads all 5 results documents and reconciles their findings.

**Tensions to expect and resolve:**

1. **Speed vs quality:** One agent will want slow accumulation (more 1-symbol
   cards, higher parameters) for better late-game quality. Another will want
   pattern variety (which requires multi-symbol cards) for decision quality.
   The parameter tuning agent's findings usually break the tie — higher
   parameters can compensate for faster accumulation from multi-symbol cards.

2. **Simplicity vs richness:** Fewer pattern types are simpler to design but
   create autopilot drafts. More pattern types create richer decisions but
   require more card design effort.

3. **Agent-specific vs universal findings:** Some findings will be specific to
   the chosen algorithm (e.g., threshold values). Others will be universal
   (e.g., pattern variety is always needed, rarity is usually orthogonal).

**Method:**
- Build a simulation implementing the reconciled pool design.
- Test it with the algorithm under 2-3 parameter configurations.
- Compare against the original default to quantify improvements.
- Simulate all player strategies (committed, power-chaser, signal-reader).

**Output:** `pool_design_final.md` (max 1500 words) + `pool_synthesis_sim.py`

The final document should include:
1. **Complete pool specification** — exact card counts, pattern breakdown,
   rarity distribution, algorithm parameters. Specific enough that a card
   designer could build the set from it.
2. **How each agent's finding was incorporated** (or why it was overridden).
3. **Tensions and how they were resolved.**
4. **Before/after comparison** on key metrics.
5. **Open questions for playtesting.**

---

## Agent Summary

| Round | Agents | Type | Focus |
|-------|--------|------|-------|
| 1 | 5 | Parallel background | Symbol ratios, rarity, archetype breakdown, symbol patterns, parameter tuning |
| 2 | 1 | Background | Synthesis and reconciliation |
| **Total** | **6** | | |

## Output Files

| File | Round | Description |
|------|-------|-------------|
| `pool_sim_{1..5}.py` (x5) | 1 | Simulation code per investigation |
| `pool_results_{1..5}.md` (x5) | 1 | Results per investigation |
| `pool_synthesis_sim.py` | 2 | Reconciled simulation |
| `pool_design_final.md` | 2 | Complete pool specification |

## Key Principles

1. **Archetype-level evaluation is non-negotiable.** Never measure resonance
   matching as a proxy for archetype fitness. A resonance is shared by
   multiple archetypes; only S/A-tier fitness tells you whether a card
   actually serves the player's strategy.

2. **Decision quality matters more than metric optimization.** A pool that
   produces perfect convergence numbers but autopilot drafting (0% genuine
   choice) is worse than one with slightly lower convergence but meaningful
   pick-by-pick decisions.

3. **Parameters and pool design interact.** You cannot design the pool without
   knowing the algorithm's parameters, and you cannot tune parameters without
   knowing the pool composition. Agent 5 exists specifically to map this
   interaction.

4. **Universal findings vs algorithm-specific findings.** Some results
   generalize across all draft algorithms:
   - Rarity is almost always orthogonal to the convergence mechanism.
   - Pattern variety is almost always necessary for decision quality.
   - Equal archetype sizes are almost always better than asymmetric.
   - ~10% generic cards is a robust default.

   Other results are algorithm-specific:
   - Optimal symbol count distribution depends on how the algorithm
     accumulates state.
   - Parameter values must be tuned to the chosen algorithm.

5. **The convergence curve is the most informative metric.** A single "late
   S/A" number hides whether packs improve steadily, peak and decline, or
   plateau early. Always plot S/A per pack at each pick number. The ideal
   curve rises through the first third of the draft and holds steady or
   continues rising — never declining.

6. **Simulate honestly.** Run 1000+ drafts per configuration. Use actual
   simulation output in results documents. Do not fabricate or estimate
   numbers.

## Recovery

Check which `pool_results_*.md` and `pool_sim_*.py` files exist to determine
progress. Round 2 depends only on Round 1 outputs being complete.
