# Resonance Draft System V10 — Orchestration Plan

## The Central Idea

V10 reframes the draft algorithm around a single player-facing narrative:

**"You are drafting against AI opponents who are also building decks from the
same card pool."**

Every game, a set of AI drafters sits at the table alongside the player. Each
AI has its own archetype preference and drafts cards from the shared pool to
build its own deck. The cards the player sees in each pack are whatever remains
after the AIs have made their picks. This is the only explanation the player
needs.

This mental model is immediately intuitive to anyone who has played a draft
format: you want to find the "open lane" — the archetype that fewer AIs are
competing for. When you commit to the open lane, you see more and better cards
because nobody else is taking them. When you fight an AI for its archetype, the
picks are thinner.

### Why This Is Different from V9

V9's pool contraction was an abstract mechanism: "the game removes cards that
don't match your style." It worked mathematically (Hybrid B: M3 = 2.70, M11 =
3.25) but the player-facing explanation was vague. Why is the game removing
cards? By what right?

V10 gives the same mathematical behavior a concrete justification. Cards
disappear from the pool because other drafters took them — the same reason cards
disappear in any draft. The hidden manipulation isn't "the game is secretly
helping you" but rather "other players are pursuing their own strategies, and
you benefit from reading the table."

This framing also opens new design axes that pure contraction couldn't access:

- **Lane signaling.** The player can observe which archetypes have more cards
  available and infer which AIs are (and aren't) at the table. This creates a
  genuine skill axis: reading the draft.
- **Competition dynamics.** If the player drafts into an AI's lane, both compete
  for the same cards. This creates natural tension and meaningful tradeoffs.
- **Game-to-game variety.** Varying which AIs are present (and how aggressive
  they are) creates natural replayability without explicit randomization knobs.
- **Fairness narrative.** "You're competing against other drafters" is a
  universally understood and accepted framing. No player feels cheated by this.

---

## What Changed Since V9

### V9 Key Results (for agent reference)

**Recommended: Hybrid B (Affinity-Tagged Gravity)**

Pool contraction at 12% per pick using blended relevance (40% visible
dot-product + 60% pair-affinity score). 8 bits of hidden metadata per card
(two 4-bit pair-affinity floats).

- M3 = 2.70, M11 = 3.25 (only algorithm to pass both targets)
- V1 = 84.8% (visible symbols do 85% of targeting work)
- V3 = 9/10 (honestly derived metadata)
- Per-archetype spread = 0.25 (best equity)
- Fails M5 = 9.6 (target 5-8) and M10 = 3.8 (target <= 2)

**Key V9 Structural Findings:**

1. Pool contraction is mandatory for M11 >= 3.0.
2. A 3-bit archetype tag is necessary but not sufficient for M11 >= 3.0.
3. Two-float pair affinity (8 bits) is the right abstraction level.
4. Visible symbols do 77-99% of targeting work at 10% visible dual-res.
5. The M3-M10-M6 triangle persists: no algorithm achieves all three.
6. Archetype inference from early picks is the critical implementation challenge.
7. Design integrity and performance are not in tension.

### V10 Starting Point

V10 takes V9's mathematical foundations and asks: **can we achieve comparable or
better metrics by replacing abstract pool contraction with concrete AI drafters,
and does the AI drafter framing unlock solutions to V9's persistent failures
(M5 convergence delay, M10 transition zone streaks)?**

The hypothesis: AI drafters are a superset of pool contraction. Every
contraction algorithm can be expressed as "N AIs each took some cards." But AI
drafters can also do things contraction cannot — they can create observable
patterns (lane signals), they can interact with each other (avoiding the same
archetype), and their behavior can be tuned per-AI rather than as a single
contraction rate. This additional design surface may solve V9's open problems.

---

## Fixed Assumptions (Not Variables in V10)

### Fitness Model: Graduated Realistic (Fixed)

| Pair | Sibling A-Tier Rate |
|------|:---:|
| Warriors / Sacrifice (Tide) | 50% |
| Self-Discard / Self-Mill (Stone) | 40% |
| Blink / Storm (Ember) | 30% |
| Flash / Ramp (Zephyr) | 25% |
| **Weighted Average** | **~36%** |

### Pool Size: 360 Cards (Fixed)

40 per archetype (320) + 40 generic. Not a variable.

### Pack Size: 4 Cards (Fixed)

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

### Visible Symbol Distribution (Fixed from V9)

| Symbol Count | Cards | % |
|:---:|:---:|:---:|
| 0 (generic) | 40 | 11% |
| 1 visible symbol | 284 | 79% |
| 2 visible symbols | 36 | 10% |

Visible dual-resonance stays at ~10%. Not a variable.

---

## The AI Drafter Design Space

This is V10's novel contribution. Instead of an abstract contraction algorithm,
the system models N AI opponents who draft from the same pool.

### Core Questions for V10

1. **How many AIs?** Could be 7 (one per non-player archetype), 3-4 (a subset),
   or variable per game. More AIs = more cards removed = faster convergence but
   less player choice. Fewer AIs = more open pool but weaker lane signals.

2. **How are AI archetypes assigned?** Randomly each game? Fixed set with
   random activations? Weighted by some seed? This controls game-to-game
   variety (M7, M8) and signal reading.

3. **How do AIs evaluate cards?** This is a rich design axis:
   - **Visible resonance only:** AIs look at the same symbols the player sees.
     Most transparent, but may lack precision for same-resonance siblings.
   - **Archetype tags:** AIs know each card's archetype. Simple, effective, but
     the AIs have information the player doesn't.
   - **Internal power level:** AIs assign their own "how good is this card for
     my deck" score. Could be derived from card properties or assigned.
   - **Deckbuilding logic:** AIs track what they've drafted and make synergy-
     aware picks. More realistic but more complex.
   - **Some combination:** e.g., visible resonance for early picks, archetype
     tags for refinement, power level for tiebreaking.

4. **When do AIs pick?** Before each player pack? All at once before the draft?
   In a rotating order? This affects the pacing of pool depletion and when
   lane signals become visible.

5. **How aggressive are AIs?** Do they always take the best available card for
   their archetype? Do they sometimes take generics or off-archetype cards?
   Do they have a "power threshold" below which they pass? More aggressive
   AIs create stronger lane signals but leave fewer options.

6. **Do AIs react to the player?** This is the most important design axis:
   - **Fully predetermined:** AIs decide all picks before the draft starts.
     Player actions have zero influence on AI behavior. Maximum transparency
     but no dynamic interaction.
   - **Partially reactive:** AIs adjust their strategy based on what the player
     drafts. E.g., if the player takes Warriors cards, the Warriors AI might
     pivot or become more aggressive.
   - **Lane-avoidant:** AIs actively avoid the player's lane, creating the
     "open lane" dynamic naturally. But this means the player always finds an
     open lane — is that too easy?
   - **Fully reactive:** AIs respond to every player pick. Maximum dynamic
     interaction but potentially feels like the game is conspiring.

7. **What does the player observe?** Do they see which AIs are present? Do they
   see what the AIs drafted? Or do they only see the resulting card pool and
   infer the rest? More information = more signal reading skill axis, but also
   more UI complexity.

### The Reactivity Spectrum

The most important design question in V10. From least to most reactive:

**Level 0 — Static:** All AI picks are determined before the draft starts (seed-
based). The player's picks don't change AI behavior at all. The player is
solving a fixed puzzle: "which lane is open in this particular game?" This is
the simplest model and the most transparent. It's also how real booster drafts
work — the other players made their picks independently.

**Level 1 — Delayed reaction:** AIs determine their first N picks in advance
(establishing lanes), then begin reacting to the player's picks from pick N+1
onward. The player reads pre-set signals early, then the AIs adapt. This
creates a "the draft gets personal" feeling in the mid-game.

**Level 2 — Soft reaction:** AIs have predetermined archetype preferences but
adjust pick order based on what the player takes. If the player takes a Warriors
card, the Warriors AI might prioritize its remaining Warriors cards more
urgently. The AIs don't change their lane, but they draft more aggressively
within it in response to competition.

**Level 3 — Lane-avoidant:** AIs actively avoid the player's emerging
archetype. As the player signals Warriors commitment, AIs that were considering
Warriors cards deprioritize them. This creates the "rewarded for commitment"
feeling but may feel engineered.

**Level 4 — Fully dynamic:** AIs run full deckbuilding logic that responds to
the entire draft state, including the player's picks, other AIs' picks, and
remaining pool composition. Maximum realism but maximum complexity and hardest
to make feel fair.

V10 should test at least Levels 0, 1, and 3 to determine which level of
reactivity produces the best balance of signal reading, convergence, and
fairness perception.

---

## Design Goals (Ranked by Priority)

1. **Simple.** "You're drafting against AI opponents." One sentence. The player
   understands immediately why some cards are available and others aren't.
2. **Not on rails.** Multiple archetypes should be viable in any given game.
   The player should have real choice among at least 2-3 reasonable lanes.
3. **Can't force the same deck.** Game-to-game variety in which AIs are present
   and what they draft should prevent repetitive strategies.
4. **Cross-archetype decks possible.** The player should be able to combine
   elements of two archetypes, or draft outside the core eight.
5. **Convergent.** After committing (~pick 6), the player should see 2+ S/A
   cards from their archetype per pack on average, scaling to ~3 late in the
   draft.
6. **Splashable.** ~1 off-archetype card should appear in most packs.
7. **Open-ended early.** Picks 1-5 should show a variety of archetypes,
   allowing the player to survey the table before committing.
8. **Signal reading.** There should be a moderate but meaningful benefit to
   figuring out which archetype is over-represented in the available pool
   (i.e., which lane is open).
9. **Not punitive.** The player should not be forced into a single archetype
   because AIs took everything else. Even "fighting" an AI for its lane should
   produce a playable (if suboptimal) deck.

---

## Simulation Card Model

```python
class SimCard:
    id: int
    visible_symbols: list[Resonance]   # what the player sees (0-2 symbols)
    archetype: str                     # primary archetype (for evaluation)
    archetype_fitness: dict            # archetype_id -> tier — evaluation only
    rarity: Rarity
    power: float                       # raw card strength (0-10)
    # hidden metadata is agent-defined — agents may add any fields they need
    # for their AI drafter logic (archetype tags, affinity scores, synergy
    # data, etc.)

class AIDrafter:
    archetype_preference: str          # the lane this AI wants to draft
    # everything else is agent-defined: pick logic, reactivity, aggression,
    # internal state, etc.
```

---

## Round 1: Research (3 parallel agents)

Pure research — no algorithm design. Map the AI drafter design space.

### Research Agent A: AI Drafting in Games

**Question:** How do existing games implement AI drafters / draft bots, and
what makes them feel like real opponents vs. transparent algorithms?

Explore:
- How does MTG Arena implement draft bots? What do bots consider when picking?
  How have bot behaviors changed over time? What do players think of them?
- How do other digital card games (Legends of Runeterra Expeditions, Hearthstone
  Arena, Eternal Draft) handle AI opponents in draft formats?
- What makes an AI drafter feel like a "real player" vs. an obvious algorithm?
  What level of sophistication is needed?
- How do real draft formats (MTG, sports fantasy drafts) create the "open lane"
  dynamic? What makes lane reading feel skillful vs. random?
- What are the failure modes of AI drafters? (Too predictable? Too random?
  Too aggressive? Too passive?)

**Output:** `docs/resonance/v10/research_ai_drafting.md` (max 2000 words)

### Research Agent B: Reactivity and Fairness

**Question:** How should AI drafters respond to the player's actions, and at
what point does reactivity feel unfair?

Explore:
- In real drafts, other players don't know what you're picking (sealed packs).
  Should AI drafters simulate this independence, or should they react to
  visible signals?
- What happens when AI drafters are fully predetermined (Level 0)? Does the
  draft feel static? Does the player feel like their picks don't matter because
  the AIs already decided?
- What happens when AI drafters are fully reactive (Level 3-4)? Does the draft
  feel rigged? Does the player feel like the game is conspiring to help/hinder
  them?
- Is there a sweet spot of reactivity that feels like "other players at the
  table" without feeling like "the game is reading my mind"?
- How does reactivity interact with the signal reading goal? Can the player
  read signals from a predetermined AI, or does signal reading require
  reactive AIs?

**Output:** `docs/resonance/v10/research_reactivity.md` (max 2000 words)

### Research Agent C: V9 Mechanism Translation

**Question:** How do V9's proven mechanisms (pool contraction, pair-affinity,
floor slots) map onto AI drafter behavior?

Analyze:
- V9's Hybrid B contracts the pool by 12% per pick. What does this look like
  as AI drafters? How many AIs picking how many cards per round produces
  equivalent contraction?
- V9's pair-affinity scores let the contraction distinguish Warriors from
  Sacrifice within Tide cards. How would an AI drafter replicate this? Does
  the AI need hidden affinity data, or can it use a simpler preference model?
- V9's floor slot (1 top-quartile card from pick 3) guarantees a minimum
  quality. What's the AI drafter equivalent? (A weaker AI that doesn't take
  the best cards? A "coach" mechanic? Simply not modeled?)
- V9 fails M5 (convergence at pick 9.6) and M10 (3.8 consecutive bad packs).
  Could AI drafters solve these by creating stronger early signals and smoother
  mid-game transitions?
- What are the mathematical constraints on AI drafter designs that must be
  respected to maintain V9-level M3 and M11?

**Reads:** This plan, V9 final report, V9 algorithm overview.

**Output:** `docs/resonance/v10/research_v9_translation.md` (max 2000 words)

---

## Round 2: Algorithm Design (6 parallel agents)

Each agent reads all Round 1 research outputs plus this plan and V9 reports.
Each explores a different region of the AI drafter design space.

**Fixed parameters for all agents:**
- Fitness: Graduated Realistic (36% avg, per-pair)
- Pool: 360 cards, ~10% visible dual-res, ~79% single-symbol, ~11% generic
- All V9 reference results available for comparison
- AI drafters must be explainable as "other players building decks"

**Output format (all agents):**

1. Key findings (5-7 bullets)
2. Three algorithm proposals: name, one-sentence player description, technical
   description, AI drafter behavior, predicted M3/M10/M11/M6
3. Champion selection with justification
4. Champion deep-dive: how it works, what the player sees vs. what the AIs do,
   example draft showing AI + player picks, failure modes
5. AI drafter specification (number, archetype assignment, pick logic,
   reactivity level, aggression)

Max 1500 words per agent.

### Agent 1: Static AI Drafters (Level 0 Reactivity)

**Starting point:** All AI behavior is determined before the draft starts. The
player is solving a fixed puzzle.

**Question:** Can fully predetermined AI drafters achieve M3 >= 2.0 and
M11 >= 3.0 while creating a meaningful signal reading experience?

Explore:
- How many AIs and how many picks per AI per round create good pool depletion
  pacing?
- How should AIs select cards? Visible resonance only? With archetype knowledge?
  With power-level evaluation?
- How does varying the number of active AIs per game affect M7 (variety) and
  M8 (archetype frequency)?
- Can a static model create the convergence ramp (more archetype cards as the
  draft progresses) that V9's contraction achieved?
- What does signal reading look like when AIs are predetermined?

### Agent 2: Delayed-Reaction AI Drafters (Level 1 Reactivity)

**Starting point:** AIs establish their lanes early (first ~5 picks
predetermined), then begin reacting to the player's choices.

**Question:** Does a two-phase AI (predetermined early, reactive late) produce
better M5 convergence and M10 smoothness than a fully static model?

Explore:
- What transition mechanism feels natural? How do AIs "notice" what the player
  is doing?
- Does late-game reactivity help or hurt the signal reading experience? (If AIs
  react, do the early signals become unreliable?)
- How should AIs react? Avoid the player's lane? Compete harder? Pivot to a
  new lane?
- Can the two-phase structure solve V9's M5 failure (convergence at pick 9.6)?

### Agent 3: Lane-Avoidant AI Drafters (Level 3 Reactivity)

**Starting point:** AIs actively avoid the player's committed archetype,
creating a natural "open lane rewards commitment" dynamic.

**Question:** Does lane avoidance produce convergent drafts that feel earned
rather than gifted?

Explore:
- If AIs avoid the player's lane, the player always finds an open lane. Is
  that too easy? Does it remove the signal reading skill?
- How much lane avoidance is right? Should AIs partially avoid (reduce pick
  rate by 50% for player's archetype) or fully avoid (never pick from
  player's lane)?
- Does lane avoidance feel like "the game is helping me" or "the other players
  moved on to other strategies"?
- Can lane avoidance be combined with static early behavior to preserve signal
  reading in picks 1-5?

### Agent 4: Aggressive vs. Passive AI Spectrum

**Starting point:** Instead of varying reactivity, vary how aggressively the
AIs draft.

**Question:** What is the right aggression level for AI drafters to produce the
target metrics while keeping multiple lanes viable?

Explore:
- What happens when AIs are very aggressive (take top card every time)?
  Moderate (take top card 70% of the time, generic/off-archetype 30%)?
  Passive (only take cards above a power threshold)?
- Can you create natural lane width by having some AIs be more aggressive
  than others? (An aggressive Warriors AI makes Warriors thin; a passive Storm
  AI leaves Storm wide open.)
- How does aggression interact with the "not punitive" goal? If an AI is
  aggressive in the player's lane, is the result "challenging but viable" or
  "unplayable"?
- Can variable aggression replace reactivity as the mechanism for convergence?

### Agent 5: Deckbuilding-Aware AI Drafters

**Starting point:** AIs run simplified deckbuilding logic — they track what
they've drafted and make synergy-aware picks.

**Question:** Does giving AIs more sophisticated pick logic produce better
drafts, or does the added complexity not translate to better player experience?

Explore:
- What if AIs track their drafted cards and prefer cards that synergize with
  their existing picks? (e.g., the Warriors AI has drafted 3 creatures, so it
  now values combat tricks more highly.)
- Does sophisticated AI logic produce more realistic draft patterns (e.g.,
  AIs that naturally stop taking a card type they have enough of)?
- Is there a sweet spot of AI sophistication — complex enough to create
  natural-feeling drafts, simple enough to be deterministic and predictable?
- Does AI sophistication affect any metrics measurably, or is a simple
  preference function equivalent in practice?

### Agent 6: Hybrid and Novel Approaches

**Starting point:** Combine elements from the other agents or propose something
entirely new.

**Question:** What is the best overall AI drafter design, potentially combining
ideas from multiple approaches?

Explore freely. Some starting ideas (but feel free to ignore these and propose
your own):
- AIs with different personalities (aggressive/passive, focused/flexible) that
  create natural variety.
- "Rivalry" mechanics where some AIs compete with each other, leaving certain
  lanes open by mutual avoidance.
- A "draft table" simulation where AIs and the player pass packs in a circle
  (more like a real draft structure).
- AIs that occasionally make "bad picks" (off-archetype power picks) to
  simulate human drafting imperfection.
- Meta-game AIs that shift preferences across games based on which archetypes
  the player has been drafting in previous runs.

---

## Round 3: Critic Review (1 agent, sequential)

A single critic agent reads all 6 Round 2 proposals plus all Round 1 research
and this plan.

**Task:**

1. Rank all proposals on: M3/M11 potential, player experience (does it feel
   like drafting against real opponents?), simplicity, signal reading quality,
   "not on rails" score.
2. Identify the best reactivity level. Is static (Level 0) sufficient? Does
   reactivity help or hurt the player experience?
3. Identify the best AI pick logic. Do AIs need hidden archetype knowledge, or
   can they draft on visible information alone?
4. Propose 1-2 hybrid designs combining strengths from multiple proposals.
5. Flag any proposal that is too punitive (player forced into one lane), too
   generous (player always finds perfect lane with no effort), or too complex
   (cannot be explained as "AI opponents drafting alongside you").
6. Recommend which 4-6 algorithms should advance to simulation (may include
   hybrids).

**Output:** `docs/resonance/v10/critic_review.md` (max 2500 words)

After the critic review is written, each of the 6 design agents gets a brief
response turn (max 500 words each) to modify their champion based on the
critique. These responses are appended to their original design documents as
a "## Post-Critique Revision" section.

---

## Round 4: Simulation (6 parallel agents)

Each agent implements and simulates their champion (as modified by the critic
response). Agent assignments may change based on the critic's recommendations.

**All agents read:** Critic review, all design documents (with post-critique
revisions), all research documents, this plan.

**Fixed simulation parameters:**
- 1000 drafts x 30 picks x 3 player strategies
- Fitness: Graduated Realistic (primary), Pessimistic (secondary)
- Pool: 360 cards, ~10% visible dual-res
- All 11 metrics (M1-M11) at archetype level

**Required outputs per agent:**

1. Simulation code: `docs/resonance/v10/sim_{1..6}.py`
2. Results: `docs/resonance/v10/results_{1..6}.md` (max 1000 words)

Results must include:
- Scorecard (all metrics at Graduated Realistic; M3/M10/M11 at Pessimistic)
- Per-archetype M3 table (8 rows)
- Pack quality distribution (p10/p25/p50/p75/p90 for picks 6+)
- Consecutive bad pack analysis
- 2 draft traces (committed player, signal reader) — showing AI picks alongside
  player picks to illustrate the competitive drafting narrative
- AI drafter behavior summary: how many cards did AIs take? From which
  archetypes? How did the pool evolve?
- Comparison to V9 baseline: Hybrid B (M3 = 2.70, M11 = 3.25)
- Self-assessment: does this algorithm pass? What would fix the failures?

Run the simulation and report actual numbers. No projections.

---

## Round 5: Final Synthesis (1 agent)

A single agent produces the definitive comparison and recommendation.

**Reads:** This plan, all research, all design documents (with revisions),
critic review, all simulation results, V9 final report, V9 algorithm overview.

**Produces two files:**

### File 1: `docs/resonance/v10/final_report.md` (max 4000 words)

1. Unified comparison table of all algorithms (all metrics)
2. The key question: **Does the AI drafter framing produce better or equivalent
   metrics to V9's abstract contraction, and does the narrative justification
   improve the player experience?**
3. The reactivity question: what level of AI reactivity to the player produces
   the best results?
4. The aggression question: how aggressive should AIs be, and should different
   AIs have different aggression levels?
5. Per-archetype convergence table for the top 3 algorithms
6. V10 vs V9 comparison: what did we gain and lose by switching to AI drafters?
7. Recommendation tiers:
   - **Simple (static AIs, visible-only picks):** Best algorithm where AIs
     use only information the player can also see.
   - **Standard (static or mildly reactive AIs, with archetype knowledge):**
     Best overall algorithm balancing simplicity, fairness narrative, and
     metrics.
   - **Advanced (reactive AIs with deckbuilding logic):** Best algorithm if
     complexity is acceptable.
8. Complete specification for the recommended algorithm: number of AIs, how
   they're assigned, pick logic, reactivity rules, aggression levels, all
   parameters.
9. Implementation guide: how to translate the simulation into the real game
   system.
10. Open questions for playtesting.

### File 2: `docs/resonance/v10/algorithm_overview.md` (max 2500 words)

Catalog of all algorithms ordered by preference:

1. **Recommended (1-2 algorithms):** Complete specification, full metrics
2. **Viable alternatives:** Different tradeoff points
3. **Eliminated:** Algorithms that failed, organized by failure mode
4. **Structural findings:** Cross-cutting lessons about AI drafter design

---

## Agent Summary

| Round | Agents | Type | Description |
|-------|--------|------|-------------|
| 1 | 3 | Parallel | AI drafter research |
| 2 | 6 | Parallel | Algorithm design |
| 3 | 1 + 6 responses | Sequential | Critic review + designer responses |
| 4 | 6 | Parallel | Simulation |
| 5 | 1 | Single | Final synthesis |
| **Total** | **~19** | | |

## Output Files

| File | Round | Description |
|------|-------|-------------|
| `research_ai_drafting.md` | 1 | AI drafting in games |
| `research_reactivity.md` | 1 | Reactivity and fairness |
| `research_v9_translation.md` | 1 | V9 mechanisms as AI drafters |
| `design_{1..6}.md` (x6) | 2 | Algorithm proposals |
| `critic_review.md` | 3 | Cross-proposal analysis |
| `sim_{1..6}.py` (x6) | 4 | Simulation code |
| `results_{1..6}.md` (x6) | 4 | Results |
| `final_report.md` | 5 | Recommendation + specification |
| `algorithm_overview.md` | 5 | Catalog of all algorithms |

All files in `docs/resonance/v10/`.

## Key Principles

1. **"AI opponents" is the player-facing explanation.** Every mechanism must be
   expressible as "other drafters took those cards." If it can't be framed
   this way, it doesn't belong in V10.
2. **Not punitive.** The player should never feel locked into exactly one
   archetype. Fighting an AI for its lane should produce a weaker but playable
   deck, not a disaster.
3. **Signal reading is a skill, not a chore.** The player should be rewarded
   for reading which lane is open, but should not be punished severely for
   missing the signal.
4. **Simpler AI logic is better.** A deterministic preference function that
   works is preferable to a full deckbuilding AI that works slightly better.
5. **Build on V9, don't repeat it.** V9 established the mathematical
   foundations. V10 asks whether the AI drafter framing improves the player
   experience and unlocks solutions to V9's open problems (M5, M10).
6. **Compare to V9 baselines at every step.** Hybrid B (M3 = 2.70, M11 = 3.25)
   is the reference. V10 algorithms should match or exceed these numbers.
7. **Game-to-game variety matters.** The AI drafter model naturally creates
   variety through random AI assignment. Lean into this strength.
8. **Less concerned about hidden information quantity.** V9 obsessed over bits
   of hidden metadata. V10 accepts that AI drafters have internal state the
   player can't see — this is expected and fair. The question is whether the
   AI behavior feels reasonable, not whether it's minimal.
9. **Let agents be creative.** This plan sets the framing and goals but does
   not prescribe specific solutions. Agents should explore freely within the
   AI drafter paradigm.

## Recovery

Check which `docs/resonance/v10/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained.
