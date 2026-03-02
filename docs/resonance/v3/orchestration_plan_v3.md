# Resonance Draft System V3 — Orchestration Plan

## Lessons from V1 and V2

V1 assumed a "5 colors / 10 color-pair archetypes" structure (like Magic: The
Gathering) and spent all effort tuning weighting formulas. V2 corrected this by
starting from archetype fitness scores and exploring the design space more
broadly. V2 produced useful algorithms and converged on N=8 archetypes with
tiered weighted sampling.

**However, both V1 and V2 share a fatal flaw:** the algorithms are too
"magical" and opaque to players. The recommended V2 algorithm uses hidden
weight ramps (7x/8x/9x), soft floor guarantees, commitment detection
heuristics, and splash slots — none of which are visible or understandable to
the player. The player-facing explanation from V2 was: *"Each quest draws from
a shifting pool of strategies — the system nudges you toward your chosen
archetype after you commit."* This is meaningless. A player cannot predict or
reason about what will happen based on their choices.

**V3's core requirement:** The draft algorithm must be explainable *in complete
technical detail* in one or two sentences of normal English. Not a vague
summary — the actual algorithm. A player should be able to read the description
and write the code. They should be able to think "I took a Tide card, and that
means X will happen to my future packs."

---

## The Resonance System

### Four Resonance Types

Dreamtides has four resonance types: **Ember**, **Stone**, **Tide**, **Zephyr**.

Each card has between 0 and 3 **resonance symbols** printed on it. The symbols
are **ordered** — the leftmost symbol is the card's primary resonance. A card
with symbols [Tide, Zephyr] is a Tide-primary card with Zephyr secondary. This
is mechanically different from [Zephyr, Tide].

- ~10% of cards (~36 of 360) have 0 symbols (generic/neutral cards)
- The remaining ~90% have 1-3 symbols

### Eight Archetypes on a Circle

The 8 archetypes are arranged in a circle. Each resonance type sits between
its two core archetypes. The circle, listed clockwise as a numbered sequence:

1. **Flash/Tempo/Prison** — Zephyr primary, Ember secondary
2. **Blink/Flicker** — Ember primary, Zephyr secondary
   *(Ember sits between positions 2 and 3)*
3. **Storm/Spellslinger** — Ember primary, Stone secondary
4. **Self-Discard** — Stone primary, Ember secondary
   *(Stone sits between positions 4 and 5)*
5. **Self-Mill/Reanimator** — Stone primary, Tide secondary
6. **Sacrifice/Abandon** — Tide primary, Stone secondary
   *(Tide sits between positions 6 and 7)*
7. **Warriors/Midrange** — Tide primary, Zephyr secondary
8. **Ramp/Spirit Animals** — Zephyr primary, Tide secondary
   *(Zephyr sits between positions 8 and 1, wrapping the circle)*

**Adjacency rules:**
- Positions 1 and 2 are adjacent (share Ember/Zephyr)
- Positions 2 and 3 are adjacent (share Ember)
- Positions 7 and 8 are adjacent (share Zephyr/Tide)
- Positions 8 and 1 are adjacent (share Zephyr) — the circle wraps
- Positions 1 and 5 are **opposite** (no shared resonance)

**Key property:** Each resonance is the primary for exactly 2 archetypes and
the secondary for exactly 2 archetypes. Adjacent archetypes on the circle
share a resonance (one as primary, one as secondary).

### What Cards Look Like

Cards belonging to an archetype typically carry symbols from that archetype's
resonances:

- A Warriors card might have symbols [Tide] or [Tide, Zephyr] or [Tide, Tide, Zephyr]
- A Storm card might have [Ember] or [Ember, Stone]
- A card bridging Warriors and Ramp might have [Tide, Zephyr] or [Zephyr, Tide]

Most decks should be firmly centered in one resonance. A Warriors deck will
have a majority of Tide cards and some supporting Zephyr cards. A Storm deck
will have mostly Ember cards and some Stone.

---

## The Problem

Design a draft algorithm that uses resonance symbols to construct packs in a
way that is:

1. **Actually simple** — explainable to players in one sentence of complete
   technical detail, not a vague summary
2. **Transparent** — the player can predict what will happen based on their
   choices
3. **Convergent** — rewards commitment to a resonance/archetype
4. **Not on rails** — doesn't force one path
5. **Varied** — different runs feel different
6. **Splashable** — off-resonance cards appear regularly

The algorithm should operate primarily on **visible card properties**, not
hidden fitness scores. Resonance symbols are the strongest candidate for the
primary input, but agents are free to propose algorithms that also incorporate
other visible properties (rarity, card type, keywords, power level) or even
algorithms where resonance symbols play a secondary role — as long as the
result passes the Simplicity Test. If an algorithm works better using
something other than resonance symbols as its main lever, that's worth knowing.

### Fixed Parameters

- **360 unique cards** in the draft pool
- **4 cards per pack**, pick 1
- **30 picks** per quest
- **4 resonance types**: Ember, Stone, Tide, Zephyr
- **8 archetypes** arranged on a circle
- **0-3 ordered resonance symbols** per card
- **~10% of cards** have 0 symbols (generic)
- **Symbol order matters**: primary symbol has more weight than secondary/tertiary

---

## Design Goals

Ranked by priority:

1. **Simple.** Explainable to players *in complete technical detail* in one
   sentence. The player should be able to write the code from the description.
   NOT "the system nudges you toward your archetype" — that's meaningless.
   YES "each symbol you draft adds a token to a bag, and pack cards are drawn
   by pulling tokens from that bag" — that's a real algorithm.
2. **Not on rails.** The player should not be forced into one archetype or have
   only 1 real choice per pack.
3. **No forced decks.** The player should not be able to force the same deck
   every time they play.
4. **Flexible archetypes.** It should be possible to build decks outside the
   core archetypes, or combine 2 archetypes.
5. **Convergent.** If you HAVE committed to an archetype (around pick 6 on
   average), you should see a minimum of 2 cards from that archetype in most of
   your draft picks.
6. **Splashable.** You should see around 1 card from outside your archetype in
   most draft picks.
7. **Open-ended early.** In the first ~5 picks, you should see a variety of
   cards from different archetypes.
8. **Signal reading.** There should be a moderate benefit to figuring out which
   archetype is over-represented in the starting pool.

### Measurable Targets

| Metric | Target |
|--------|--------|
| Picks 1-5: unique resonances represented per pack | >= 3 of 4 on average |
| Picks 1-5: cards fitting player's emerging archetype per pack | <= 2 of 4 |
| Picks 6+: cards fitting committed archetype per pack | >= 2 of 4 on average |
| Picks 6+: strong off-archetype cards per pack | >= 0.5 of 4 on average |
| Convergence pick (player regularly sees 2+ archetype cards) | Pick 5-8 |
| Deck archetype concentration (committed player) | 60-80% S/A-tier cards |
| Run-to-run variety (same starting conditions) | < 40% card overlap |
| Archetype frequency across runs | No archetype > 20% or < 5% |

"Fitting" a committed archetype = the card has resonance symbols that match the
archetype's primary resonance (and ideally secondary).

### Simplicity Test

Every proposed algorithm must pass this test: **Can you write the complete
algorithm as a one-sentence instruction to a programmer?** If the sentence
includes words like "nudges," "adapts," "responds to," or "considers" — it
fails. The sentence must describe concrete operations on concrete data.

Examples of FAILING descriptions:
- "The system nudges you toward your chosen archetype after you commit"
- "Packs adapt to your emerging strategy"
- "The draft responds to your resonance preferences"

Examples of PASSING descriptions:
- "Each symbol you draft adds a matching token to a bag; to make a pack, draw
  4 tokens and show a random card of each token's resonance"
- "Count your drafted symbols — your top resonance fills 2 of 4 pack slots,
  second fills 1, last slot is random"
- "Your last pick's symbols each lock one slot in your next pack to that
  resonance; remaining slots are random"

---

## Simulation Card Model

Each simulated card has:

```python
class SimCard:
    id: int
    symbols: list[Resonance]  # ordered, 0-3 elements, [] = generic
    archetype: str            # primary archetype this card belongs to
    archetype_fitness: dict   # archetype_id -> tier (S/A/B/C/F) — for EVALUATION only
    rarity: Rarity            # common/uncommon/rare/legendary
    power: float              # raw card strength (0-10)
```

**Critical separation:** The draft algorithm should use **visible card
properties** — `symbols` are the primary candidate, but `rarity`, `power`,
and other visible attributes are fair game too. The `archetype_fitness` scores
are used ONLY to evaluate how well the algorithm serves players — they are
invisible to the algorithm itself. Agents are encouraged to explore whether
incorporating factors beyond resonance symbols (e.g., rarity-based mechanics,
card type, keywords) produces better or simpler algorithms than pure
symbol-based approaches.

### Card Pool Construction

For the simulation, generate 360 cards as follows:

**Per archetype (~40 cards each, 320 total across 8 archetypes):**

Each archetype's cards carry symbols from that archetype's primary and
secondary resonances. For example, Warriors (Tide/Zephyr) cards might have:
[Tide], [Tide, Zephyr], [Tide, Tide], [Tide, Tide, Zephyr], [Zephyr, Tide],
etc.

**The distribution of symbol counts (how many cards have 1 vs 2 vs 3 symbols)
is an open design question that agents should explore.** This directly affects
how strongly the draft algorithm can distinguish committed vs. uncommitted
players. For instance:
- If most cards have only [Primary], algorithms based on counting symbols
  accumulate state slowly
- If most cards have [Primary, Primary, Secondary], state accumulates fast
  and multi-resonance cards create more cross-archetype signal
- The "right" distribution likely depends on which algorithm is chosen

Agents should propose and test their own symbol distributions as part of their
simulation design.

**Generic cards (36 total, ~10%):** No symbols. Spread across rarities. These
are "good stuff" cards playable in any deck.

**Fitness assignment (for evaluation only — invisible to the draft algorithm):**
- Each card is S-tier in its home archetype
- A-tier in the adjacent archetype sharing its primary resonance
- B-tier in archetypes sharing its secondary resonance
- C/F in distant archetypes
- Generic cards are B-tier in all archetypes

### Symbol Counting Rules

When counting a player's resonance symbols (for algorithms that use counts):
- **Primary symbol** (first/leftmost on card): counts as **2**
- **Secondary symbol** (second position): counts as **1**
- **Tertiary symbol** (third position): counts as **1**

This makes the ordering of symbols matter mechanically.

### Simulated Player Strategies

- **Archetype-committed:** Picks cards with highest fitness in their strongest
  archetype. Commits around pick 5-6.
- **Power-chaser:** Picks the highest raw power card regardless of archetype.
- **Signal-reader:** Evaluates which resonance/archetype seems most available
  and drafts toward the open archetype.

---

## The Five Investigation Areas

Each agent explores a fundamentally different **domain of mechanism design**.
These domains represent structurally distinct philosophies for how a draft
algorithm can work. Within their domain, each agent must **brainstorm 5
concrete algorithms**, analyze their tradeoffs, and **champion one** to
develop further.

The domains are deliberately broad. The example algorithm for each is just ONE
possibility — agents should NOT treat the example as the answer. The point is
to explore the space and find the best algorithm within their domain.

### Domain 1: Accumulation-Based Mechanisms

**Core idea:** Drafting builds up some kind of persistent state over time.
Future packs are shaped by the accumulated state of all previous picks.

**Example (one of many possibilities):** "Each card you draft adds its
resonance symbols as tokens to a bag, and your next pack's 4 card slots are
each filled by drawing a random token from that bag." — This is a bag-building
approach, but there are many other accumulation mechanisms: running counters,
weighted pools, resonance "meters" that fill up, etc.

**What makes this domain distinct:** State grows monotonically. Early state is
small/volatile, late state is large/stable. The player's full history matters.

### Domain 2: Structural/Guaranteed Mechanisms

**Core idea:** The pack structure itself has explicit rules about what goes in
each slot. The resonance composition of packs is determined by structural
guarantees, not probability.

**Example (one of many possibilities):** "Every pack shows exactly one card
from each resonance type, but cards from your most-drafted resonance are drawn
from better pools as your count grows." — But there are many other structural
approaches: guaranteed minimums, rotating slot assignments, fixed ratios that
shift, etc.

**What makes this domain distinct:** The player can predict pack structure
before seeing it. Convergence comes through slot allocation or quality
variation, not random chance.

### Domain 3: Threshold/Progression Mechanisms

**Core idea:** The draft system has discrete states that change when the
player crosses specific thresholds. Instead of gradual scaling, there are
clear "level up" moments.

**Example (one of many possibilities):** "Drafting 3 symbols of a resonance
unlocks that resonance's uncommon cards in your packs; 6 unlocks rares; 10
unlocks legendaries." — But there are many other threshold ideas: unlocking
new card pools, changing pack size, gaining "bonus picks," opening/closing
resonance lanes, etc.

**What makes this domain distinct:** Non-linear progression with discrete
jumps. The player has clear milestones to aim for and can feel specific
moments where the system changes behavior.

### Domain 4: Reactive/Immediate Mechanisms

**Core idea:** Only recent picks matter. The algorithm responds to what you
just did, not your full history. The player can pivot at any time without
accumulated penalty.

**Example (one of many possibilities):** "Each resonance symbol on the card
you just drafted reserves one slot in your next pack for that resonance;
remaining slots are random." — But there are many other reactive ideas:
streak bonuses for consecutive picks of the same resonance, "echo chains"
where effects ripple forward a few picks, cooldown systems, etc.

**What makes this domain distinct:** Little or no long-term memory. Maximum
responsiveness. The player's most recent decision has the strongest effect.
May sacrifice convergence for flexibility.

### Domain 5: Pool Manipulation Mechanisms

**Core idea:** Instead of changing how packs are assembled FROM the pool,
change the pool itself. The available cards shift based on the player's
choices.

**Example (one of many possibilities):** "When you draft a card, 3 random
cards of the same primary resonance are added to the pool from a reserve."
— But there are many other pool manipulation ideas: cards you pass on get
removed, resonance-based filtering narrows the pool, sub-pools that grow/
shrink, a "shop" that restocks based on your resonance, etc.

**What makes this domain distinct:** The mechanism is invisible at the pack
level but creates emergent shifts in what's available. Can create natural
signal-reading opportunities (observing what appears/disappears).

---

## Rounds

### Round 1: Algorithm Exploration and Design (5 parallel agents)

Each agent explores their assigned domain of mechanism design. This round is
pure reasoning and analysis — **no simulation code**.

**All agents read:** This orchestration plan.

**Each agent must produce (max 2000 words):**

1. **Five algorithm proposals:** Brainstorm 5 concrete, distinct algorithms
   within your domain. **All 5 must appear in the output document** — later
   rounds and the final algorithm_overview.md depend on having a complete
   catalog. For each of the 5, provide:
   - A name
   - A one-sentence player-facing description (must pass the Simplicity Test)
   - A 2-3 sentence technical description of how it works
   - A quick assessment: which design goals does it serve well? Which does it
     fail?
   - What symbol distribution (1 vs 2 vs 3 symbols per card) works best for
     this algorithm?

2. **Champion selection:** Pick the most promising algorithm from your 5 and
   explain why. This is the one you'll develop further in Round 3.

3. **Champion deep-dive:** For your championed algorithm:
   - Walk through example draft sequences (early-committer, flexible player,
     pivot attempt)
   - Predict failure modes honestly
   - Propose 2-3 parameter variants worth testing
   - Propose a symbol distribution to use in simulation

4. **Key Takeaways** section at the top: 5-7 bullet points summarizing the
   most important findings across all 5 proposals.

**Concrete agent assignments:**

| Agent | Domain | Output File |
|-------|--------|-------------|
| Agent 1 | Accumulation-Based | `docs/resonance/v3/design_1_accumulation.md` |
| Agent 2 | Structural/Guaranteed | `docs/resonance/v3/design_2_structural.md` |
| Agent 3 | Threshold/Progression | `docs/resonance/v3/design_3_threshold.md` |
| Agent 4 | Reactive/Immediate | `docs/resonance/v3/design_4_reactive.md` |
| Agent 5 | Pool Manipulation | `docs/resonance/v3/design_5_pool.md` |

---

### Round 2: Cross-Strategy Discussion (5-agent team, interactive)

All 5 agents reconvene as a team. Each reads all 5 Round 1 design documents
(all 25 proposed algorithms across 5 domains) and engages in structured debate.

**All agents read:** All 5 Round 1 design documents.

**Discussion structure (minimum 40 total messages):**

1. **Best-of-breed review (messages 1-10):** Each agent reviews ALL 25
   proposed algorithms (5 from each domain), not just the 5 champions. Are
   there unchampioned proposals from other domains that are actually stronger
   than the champion? Call them out.

2. **Simplicity audit (messages 11-20):** Evaluate the 5 championed algorithms
   against the Simplicity Test. Can you actually write each algorithm from its
   one-sentence description? Which sentences are secretly hiding complexity?
   Which are genuinely implementable from the description alone?

3. **Goal tradeoff analysis (messages 21-30):** For each design goal, which
   championed algorithm is best? Which is worst? Are there goals that are
   fundamentally in tension for certain approaches?

4. **Refinement proposals (messages 31-40):** Each agent proposes specific
   modifications to their championed algorithm based on the discussion. May
   switch champions if convinced a different proposal from their domain is
   stronger. These changes will be incorporated into Round 3 simulations.

**Key discussion questions:**
- Which algorithm is *actually* the simplest to explain? Not "sounds simple"
  but "a 12-year-old could predict what their next pack looks like."
- Which algorithms degenerate into "random noise" early and "on rails" late
  with no interesting middle ground?
- Are there unchampioned proposals that deserve simulation? Should any agent
  switch which algorithm they develop?
- Which algorithms make signal reading (goal 8) possible vs. impossible?
- What symbol distribution (1 vs 2 vs 3 symbols per card) works best for
  each championed algorithm?

**Output per agent (max 800 words each):**
- `docs/resonance/v3/discussion_{1|2|3|4|5}.md`

Each discussion output should include:
- A **simplicity ranking** of all 5 championed algorithms (most to least simple)
- A **scorecard table** (algorithm x goal matrix, 1-10 scores)
- Their final championed algorithm (may have changed from Round 1)
- Specific modifications they plan for Round 3
- Their proposed symbol distribution for simulation

---

### Round 3: Simulation Implementation (5 parallel agents)

Each agent implements their championed algorithm (potentially refined during
Round 2 discussion) in Python and runs simulations.

**All agents read:** All 5 discussion outputs from Round 2, plus this
orchestration plan (for the card model and measurable targets).

**Requirements:**

1. **Implement the card pool:** Generate 360 cards following the Card Pool
   Construction specification above, using the symbol distribution they
   proposed in Round 2 discussion.
2. **Implement the draft algorithm:** The EXACT championed algorithm,
   operating only on resonance symbols (not archetype fitness).
3. **Simulate 1000 drafts** of 30 picks each with all 3 player strategies.
4. **Measure all 8 metrics** from the measurable targets table.
5. **Run parameter sensitivity sweeps** on 2-3 key parameters identified in
   Round 1, PLUS a sweep on symbol distribution (what happens with mostly
   1-symbol cards vs. mostly 2-symbol vs. mostly 3-symbol?).
6. **Produce 3 detailed draft traces** (pick-by-pick):
   - A player who commits early (by pick 5)
   - A player who stays flexible for 8+ picks
   - A signal-reader who identifies the open resonance
7. **Test the one-sentence claim:** Can you reconstruct the algorithm from just
   the one-sentence description, without reading the code? If the algorithm has
   drifted from the one-sentence description during refinement, UPDATE the
   one-sentence description or simplify the algorithm.

**Each agent must produce:**

- `docs/resonance/v3/sim_{1|2|3|4|5}.py` — simulation code
- `docs/resonance/v3/results_{1|2|3|4|5}.md` (**max 800 words**) structured as:
  - **One-sentence algorithm** (final version)
  - **Target scorecard table** (metric | target | actual | pass/fail)
  - **Symbol distribution used** and sensitivity results
  - **Parameter sensitivity** section: how results change with key parameters
  - **Draft traces** section: 3 annotated pick-by-pick examples
  - **Self-assessment:** score each design goal 1-10 with 1-sentence
    justification

---

### Round 4: Cross-Comparison and Refinement (5-agent team, interactive)

All 5 agents reconvene as a team. Each reads all simulation results and
engages in structured comparison.

**All agents read:** All 5 results documents and simulation code from Round 3.

**Task for each agent:**

1. Score each strategy on each of the 8 design goals (1-10), with 1-sentence
   justification.
2. Identify the single biggest strength and biggest weakness of each strategy.
3. Propose specific improvements — what would you change about each strategy
   to fix its weaknesses?
4. **The hybrid question:** Given the simulation results, what is the best
   possible algorithm you can construct, drawing from any strategy? Each agent
   should propose their ideal hybrid.

**Discussion topics (minimum 30 total messages):**
- Which strategies actually hit the measurable targets? Which consistently
  miss, and why?
- Rank the 5 one-sentence descriptions from most honest/complete to most
  hand-wavy. Does the one-sentence description actually capture what the
  algorithm does?
- For the strategies that failed targets: is the mechanism fixable with
  parameter tuning, or is the fundamental approach wrong?
- Is there a hybrid that clearly dominates? Or are there genuine tradeoffs
  between the top strategies?

**Output per agent (max 800 words each):**
- `docs/resonance/v3/comparison_{1|2|3|4|5}.md`

Each comparison output should include:
- A **scorecard table** (strategy x goal matrix, 1-10 scores)
- Their **proposed best algorithm** (can be their original, a modification, or
  a hybrid)
- The one-sentence description of their proposed best algorithm

---

### Round 5: Final Synthesis (1 agent)

A single agent produces the definitive comparison and recommendation.

**Reads (in priority order):**
1. This orchestration plan (goals, targets, card model)
2. All 5 comparison outputs from Round 4
3. All 5 results documents from Round 3
4. Round 2 discussion documents (for context on design evolution)
5. Only if needed: simulation code from Round 3

**Task:**

1. **Run all 5 simulations** with identical parameters and produce a unified
   comparison table.
2. For each algorithm, compute all measurable targets and flag pass/fail.
3. **Rank the 5 algorithms** by overall design goal satisfaction.
4. **Apply the simplicity test** independently: for each one-sentence
   description, can the synthesis agent write the algorithm from scratch? Flag
   any descriptions that are misleading or incomplete.
5. Write the **recommended algorithm** with:
   - Complete specification (step-by-step, unambiguous)
   - One-sentence player description
   - One-paragraph player description
   - Implementation notes (edge cases, parameter values)
   - Recommended symbol distribution (how many 1/2/3-symbol cards)
6. Identify remaining open questions for playtesting.

**Output:**

- `docs/resonance/v3/final_report.md` (**max 2500 words**) — The definitive
  comparison, recommendation, and implementation specification.

- `docs/resonance/v3/algorithm_overview.md` (**max 3000 words**) — A
  comprehensive catalog of ALL algorithms considered during V3 (all 25
  proposals from Round 1, plus any hybrids from later rounds). For each
  algorithm:
  - One-sentence description
  - How it works (2-3 sentences)
  - Which domain it belongs to
  - Whether it was championed and simulated
  - If simulated: scorecard results (pass/fail on each target)
  - If not simulated: why it was not championed (1 sentence)
  - Final ranking/recommendation status
  This document should be a readable reference for anyone who wants to
  understand the full design space that was explored, similar to
  `docs/resonance/v2/algorithm_overview.md`.

---

## Agent Summary

| Round | Agents | Type | Description |
|-------|--------|------|-------------|
| 1 | 5 | Parallel background | Algorithm design per strategy |
| 2 | 5 | Team (interactive) | Cross-strategy discussion and critique |
| 3 | 5 | Parallel background | Simulation implementation + results |
| 4 | 5 | Team (interactive) | Cross-comparison and hybrid proposals |
| 5 | 1 | Background | Final synthesis report |
| **Total** | **21** | | |

## Output Files

| File | Round | Description |
|------|-------|-------------|
| `design_{1..5}_*.md` (x5) | 1 | 5 proposals per domain + champion selection |
| `discussion_{1..5}.md` (x5) | 2 | Cross-domain discussion outputs |
| `sim_{1..5}.py` (x5) | 3 | Simulation implementations |
| `results_{1..5}.md` (x5) | 3 | Simulation results + scorecards |
| `comparison_{1..5}.md` (x5) | 4 | Cross-comparison + hybrid proposals |
| `final_report.md` | 5 | Definitive comparison and recommendation |
| `algorithm_overview.md` | 5 | Catalog of all 25+ algorithms considered |

All output files are in `docs/resonance/v3/`.

## Key Principles for Agents

1. **Simplicity is non-negotiable.** If you cannot explain your algorithm in
   one sentence of concrete operations, simplify it until you can. Hidden
   complexity (weight ramps, commitment detection heuristics, soft floors) is
   a failure, not a feature.

2. **The algorithm uses visible properties, not fitness scores.** Players can
   see resonance symbols, rarity, card type, keywords, etc. The algorithm
   must operate on what players can see. Archetype fitness scores exist only
   for evaluation. Resonance symbols are the strongest candidate for the
   primary input, but if an algorithm works better using other visible
   properties (or a combination), that's a valid and interesting finding.

3. **The one-sentence description IS the algorithm.** If your implementation
   does something your one-sentence description doesn't mention, either
   simplify the implementation or admit the description is incomplete. Do not
   hide complexity behind vague language.

4. **Genuinely different domains.** The 5 domains represent structurally
   distinct design philosophies. Do NOT converge them into minor variations of
   the same approach. If your refinement makes your algorithm identical to
   another agent's, you've failed. Explore YOUR domain deeply.

5. **Symbol order matters.** Primary symbols (first position) should carry
   more weight than secondary/tertiary. The exact weighting (2x vs 1.5x vs
   other) is a parameter for agents to explore.

6. **Symbol distribution is an open question.** How many cards should have 1
   vs 2 vs 3 resonance symbols? This directly affects how fast state
   accumulates, how strong convergence signals are, and how the algorithm
   feels. Each agent should propose and justify their distribution.

7. **Test honestly.** Measure your algorithm against the targets and report
   failures clearly. An algorithm that fails 3 targets but is genuinely simple
   may be better than one that passes all targets with hidden complexity.

## Recovery

Check which `docs/resonance/v3/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained — later rounds depend only on
earlier outputs.
