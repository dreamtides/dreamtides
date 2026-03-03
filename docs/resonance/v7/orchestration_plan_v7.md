# Resonance Draft System V7 — Orchestration Plan

## The Central Problem V7 Must Solve

V6 found **Surge Packs** (T=4/S=3) as the best zero-decision algorithm,
passing 9/9 metrics at 2.05 S/A. But V6's results rest on a critical
assumption that may not hold in practice:

**Every card whose home archetype shares a primary resonance with the player's
archetype is rated A-tier.** In the simulation, a Warriors card (Tide primary)
is automatically A-tier for Sacrifice (also Tide primary), and vice versa.
This makes resonance-level targeting equivalent to archetype-level targeting
— every Tide card drawn for a Warriors player is either S (Warriors-home) or
A (Sacrifice-home), yielding ~100% S/A precision per resonance-matched slot.

**In reality, this is very hard to achieve in card design.** A Warriors card
built around tribal combat synergies may be C-tier in Sacrifice, which cares
about abandon triggers and death effects. Designing 40 cards per archetype
where each one is genuinely playable in the adjacent archetype sharing its
primary resonance is an enormous design constraint. If even half of
cross-archetype cards are actually B or C tier instead of A, the real S/A
precision per resonance-matched slot drops from ~100% to ~75%, and Surge
Packs' 2.05 S/A could fall to ~1.5-1.7.

V7 investigates: **What draft algorithms perform well under realistic card
fitness assumptions? Can we beat Surge Packs, or must we accept that 2.0+ S/A
requires either unrealistic card design or player decisions?**

---

## Lessons from V3–V6

**V3 (Lane Locking):** Lock pack slots at resonance thresholds 3/8. High
convergence (2.22 S/A) but too fast (pick 3.3), too deterministic (0.50
stddev), 96% deck concentration. Established that slot-locking crosses 2.0.

**V4 (Pack Widening):** Player spends earned resonance tokens to add bonus
cards. 3.35 S/A but requires spending decisions. Established that the
probabilistic ceiling is ~1.7 S/A — only ADD or PLACE mechanisms cross 2.0.

**V5 (Pair Matching):** Abandoned. Required 70%+ dual-resonance cards.

**V6 (Surge Packs):** Token accumulation triggers 3-of-4 resonance-filled
surge packs. 2.05 S/A, 9/9 metrics, zero decisions. Non-permanent state
allows pivoting. Rhythmic surge/normal cycle provides variance. The runner-up,
Double Enhancement (T=1/B=2), scored 2.13 S/A but with later convergence.

**Key structural findings (confirmed across V3–V6):**
1. Probabilistic mechanisms (weighting, filtering, pool manipulation) cap at
   ~1.7-2.0 S/A
2. Only ADD or PLACE mechanisms reliably cross 2.0
3. Zero decisions cost ~1.0-1.3 S/A vs. player-driven Pack Widening
4. Permanent locks destroy variance; non-permanent state preserves it
5. Resonance-level targeting achieves ~100% S/A precision — **but only under
   the assumption that cross-archetype cards are A-tier**

Finding #5 is what V7 must stress-test.

---

## The Resonance System

*(Unchanged from V6 — included for agent reference.)*

### Four Resonance Types

Dreamtides has four resonance types: **Ember**, **Stone**, **Tide**, **Zephyr**.
Each card has 0–3 resonance symbols. The leftmost symbol is the card's primary
resonance. No more than 15% of cards may have 2 different resonance types.

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
Adjacent archetypes on the circle share a resonance.

### Symbol Counting

Primary symbol (leftmost): **+2 weight**. Secondary/tertiary: **+1 each**.

---

## The Fitness Model Problem

### V6's Optimistic Model

V6 assumed:
- **S-tier** in home archetype
- **A-tier** in the adjacent archetype sharing primary resonance
- **B-tier** in archetypes sharing secondary resonance
- **C/F** in distant archetypes

Under this model, when an algorithm draws a Tide card for a Warriors player:
50% chance it's a Warriors-home card (S-tier), 50% chance it's a
Sacrifice-home card (A-tier). S/A precision = 100%.

### Why This Is Unrealistic

Designing cards that are A-tier in two mechanically distinct archetypes is
extremely difficult. Warriors wants tribal synergies, combat tricks, midrange
stats. Sacrifice wants abandon triggers, death payoffs, recursive value.
A card that's S-tier in Warriors (e.g., "When this attacks with 3+ Warriors,
draw a card") is likely C or F tier in Sacrifice. Only "good stuff" cards with
high raw power and no archetype-specific synergies naturally serve both.

### What V7 Must Do

Agents must simulate under **multiple fitness models** and report how their
algorithms perform under each. The goal is to find algorithms that are
**robust across fitness assumptions**, not just optimized for the best case.

**Agents should propose and justify their own fitness models.** The models
below are starting points, not mandates.

**Example models (agents should explore variations):**

1. **Optimistic (V6 baseline):** Adjacent-resonance cards are A-tier.
   Cross-archetype S/A precision ≈ 100%.

2. **Moderate:** 50% of adjacent-resonance cards are A-tier, 30% are B-tier,
   20% are C-tier. Cross-archetype S/A precision ≈ 75%.

3. **Pessimistic:** 25% of adjacent-resonance cards are A-tier, 40% are
   B-tier, 35% are C-tier. Cross-archetype S/A precision ≈ 62%.

The moderate model is probably closest to reality for a game with 8 distinct
archetypes. The pessimistic model represents a game where archetypes are highly
specialized with little mechanical overlap.

**Agents are free to propose alternative fitness models or redefine the
cross-archetype relationship structure entirely.** What matters is that
algorithms are tested honestly against realistic card pool assumptions.

---

## Design Goals

Ranked by priority (unchanged from V6):

1. **Simple.** One-sentence concrete operations.
2. **No extra actions.** Player picks 1 card from pack. Non-negotiable.
3. **Not on rails.** Player retains genuine choice.
4. **No forced decks.** Can't force the same deck every run.
5. **Flexible archetypes.** Can build outside core archetypes.
6. **Convergent.** After committing (~pick 6), 2+ S/A cards per pack.
7. **Splashable.** ~1 off-archetype card in most packs.
8. **Open-ended early.** Picks 1–5 show variety.
9. **Signal reading.** Moderate benefit to identifying open archetype.

### Measurable Targets

**ALL metrics at ARCHETYPE level, not resonance level.**

| Metric | Target |
|--------|--------|
| Picks 1–5: unique archetypes with S/A cards per pack | >= 3 of 8 |
| Picks 1–5: S/A cards for emerging archetype per pack | <= 2 of 4 |
| Picks 6+: S/A cards for committed archetype per pack | >= 2 of 4 average |
| Picks 6+: off-archetype (C/F) cards per pack | >= 0.5 of 4 |
| Convergence pick | Pick 5–8 |
| Deck archetype concentration | 60–90% S/A-tier cards |
| Run-to-run variety | < 40% card overlap |
| Archetype frequency across runs | No archetype > 20% or < 5% |
| StdDev of S/A cards per pack (picks 6+) | >= 0.8 |

### NEW: Fitness Robustness Target

Every algorithm must report metrics under at least 2 fitness models
(optimistic + one realistic model of the agent's choosing). An algorithm that
passes 9/9 under the optimistic model but fails under realistic assumptions is
NOT a satisfactory solution.

---

## Simulation Card Model

```python
class SimCard:
    id: int
    symbols: list[Resonance]  # ordered, 0–3 elements, [] = generic
    archetype: str            # primary archetype (for EVALUATION only)
    archetype_fitness: dict   # archetype_id -> tier — EVALUATION only
    rarity: Rarity
    power: float              # raw card strength (0–10)
```

### Card Pool Construction

360 cards: ~40 per archetype (320 total) + 36 generic. The 15% dual-resonance
cap (max 54 cards with 2 different resonance types) remains.

### NEW: Fitness Model Implementation

Each agent implements a configurable fitness assignment. The **optimistic**
model is required for V6 comparison. At least one realistic model is required.

**Example realistic fitness assignment:**

For a card with home archetype H:
- H: always S-tier
- Adjacent archetype sharing primary resonance: **roll per card** —
  50% A, 30% B, 20% C (or agent's proposed distribution)
- Archetypes sharing secondary resonance: B-tier
- Distant archetypes: C/F-tier

The roll is assigned once during pool construction and stays fixed. This
creates a mixed pool where some cross-resonance cards happen to be playable
in the sibling archetype and others aren't, which matches real card design.

### Simulated Player Strategies

- **Archetype-committed:** Picks highest fitness for strongest archetype.
  Commits around pick 5–6.
- **Power-chaser:** Picks highest raw power regardless of archetype.
- **Signal-reader:** Evaluates which resonance/archetype seems most available
  and drafts toward it.

---

## The Seven Investigation Areas

Each agent explores one area. Agents 1–2 anchor the investigation with
baselines and refinement. Agents 3–7 explore new ground broadly. **Do not
overspecify within your domain — explore freely and let the results guide you.**

### Agent 1: Previous Winners (Baseline Reference)

Implement the three best algorithms from V3–V6 as baselines:
- **Surge Packs** (V6 winner): T=4, S=3
- **Lane Locking** (V3 winner): thresholds 3/8
- **Double Enhancement** (V6 runner-up): T=1, B=2

Run all three under **every fitness model** (optimistic + at least 2
realistic models). Report how each algorithm's metrics degrade as the fitness
model becomes more realistic. This establishes the performance floor for V7.

Key questions to answer:
- At what fitness model does Surge Packs drop below 2.0 S/A?
- Does Lane Locking degrade faster or slower than Surge Packs?
- Is there a fitness model where Double Enhancement beats Surge Packs?
- What is the realistic S/A range we should target?

### Agent 2: Surge Packs Refinement

The V6 winner, Surge Packs, is the algorithm to beat. This agent's job is to
find the best possible version of it. Explore variations along any axis:

- Parameter tuning (threshold, surge slot count, token earning rates)
- Hybrid approaches (e.g., mild bias on non-surge packs, pool sculpting
  between surges, guaranteed minimum slots)
- Structural changes (e.g., partial surges, split-resonance surges, surge
  packs that target both primary and secondary resonance)
- Simplification (can Surge Packs be made simpler while retaining performance?)

Test all variations under multiple fitness models. The goal is either to
**simplify** Surge Packs without losing performance, or **strengthen** it
enough to remain above 2.0 S/A under realistic fitness assumptions.

### Agent 3: Archetype Disambiguation

V6 established that resonance-level targeting treats all 4 archetypes sharing
a resonance identically. Under optimistic fitness, this doesn't matter.
Under realistic fitness, it matters a lot — roughly half the cards drawn from
a resonance pool may not be playable in the player's specific archetype.

This agent explores mechanisms that go beyond resonance to target specific
archetypes more precisely, **without relying on dual-resonance pair matching**
(which is capped at 15% of cards).

Ideas to explore (or invent your own):
- Using the player's secondary resonance signal (top 2 resonances together)
  to narrow from 4 candidate archetypes to 1–2
- Card properties beyond resonance (cost curve, card type distribution) as
  disambiguation signals
- Pattern-based inference (the combination of cards drafted implies an
  archetype even from single-resonance signals)
- Any mechanism that improves archetype precision under realistic fitness

### Agent 4: Layered Mechanisms

V6 found that many individual mechanisms (pool sculpting, soft locks,
probabilistic weighting) are individually too weak to cross 2.0 S/A. But
V6 only tested them in isolation.

This agent explores **combining two or more sub-threshold mechanisms** into
a layered system. The hypothesis: two mechanisms that each contribute 0.3–0.5
S/A above baseline might together cross 2.0, especially if they compensate
for each other's weaknesses.

Explore freely — the only constraint is that the combined system must still
pass the one-sentence simplicity test and require zero player decisions.

### Agent 5: Novel Pack Structures

V3–V6 all assumed fixed 4-card packs. This agent questions that assumption.

Explore mechanisms that change the pack presentation itself:
- Variable pack sizes (sometimes 3, sometimes 5, triggered by draft state)
- Pack composition rules (e.g., packs always contain at least one card of
  the player's top resonance — a guaranteed floor rather than a surge)
- Split packs (show 2+2 grouped by resonance, player picks one from each
  group)
- Structured randomness (each pack has one slot reserved for the player's
  top resonance, one for secondary, two random)
- Any novel approach to how cards are presented to the player

The core question: is there a pack structure that naturally produces better
archetype convergence without needing token counters, surge mechanics, or
other state tracking?

### Agent 6: Alternative Signal Systems

Every algorithm so far uses resonance symbols as the primary signal for pack
construction. This agent asks: **are there other visible card properties that
could supplement or replace resonance as a draft signal?**

Explore ideas like:
- Energy cost as a signal (aggressive decks cluster at low cost, control at
  high cost — cost distribution in drafted cards narrows archetype identity)
- Card type distribution (creature-heavy vs spell-heavy drafting as a signal)
- Rarity as a lever (rare cards provide stronger signals or trigger different
  effects)
- Keyword/mechanic presence as implicit signals
- Hybrid systems that use resonance + one additional property for better
  precision

The question is whether additional signals can break through the
single-resonance ambiguity problem more effectively than pure resonance
mechanisms.

### Agent 7: Open Exploration

Propose any mechanism not covered by Agents 2–6. This could be:
- A genuinely novel mechanism nobody has considered
- A creative riff on a V3–V6 idea adapted for realistic fitness
- A mechanism that trades some S/A convergence for dramatically better
  player experience
- An approach that reframes the problem entirely (e.g., what if the
  algorithm doesn't try to maximize S/A but instead maximizes meaningful
  choices?)

The only requirements: zero player decisions beyond card selection,
one-sentence describable, tested under multiple fitness models.

---

## Rounds

### Round 1: Algorithm Design (7 parallel agents)

Each agent explores their assigned area. Pure reasoning — no simulation code.

**All agents read:** This orchestration plan, the V6 final report
(`docs/resonance/v6/final_report.md`), and the V6 algorithm overview
(`docs/resonance/v6/algorithm_overview.md`).

**Agent 1 produces (max 1500 words):**
- Specification of the three baseline algorithms
- Proposed fitness models (at least 3: optimistic + 2 realistic)
- Predicted impact of realistic fitness on each baseline

**Agents 2–7 each produce (max 2000 words):**

1. **Key Takeaways** (5–7 bullets)
2. **Five algorithm proposals:** Name, one-sentence description, 2–3 sentence
   technical description, predicted behavior under optimistic vs realistic
   fitness.
3. **Champion selection** with justification.
4. **Champion deep-dive:** Example draft sequences, failure modes, 2–3
   parameter variants, proposed fitness model(s) for testing.

**Output files:**

| Agent | Output |
|-------|--------|
| 1 | `docs/resonance/v7/design_1_baselines.md` |
| 2 | `docs/resonance/v7/design_2_surge_refinement.md` |
| 3 | `docs/resonance/v7/design_3_disambiguation.md` |
| 4 | `docs/resonance/v7/design_4_layered.md` |
| 5 | `docs/resonance/v7/design_5_pack_structure.md` |
| 6 | `docs/resonance/v7/design_6_alt_signals.md` |
| 7 | `docs/resonance/v7/design_7_open.md` |

---

### Round 2: Cross-Strategy Discussion (7-agent team, interactive)

All 7 agents read all Round 1 design documents and engage in structured debate.

**Discussion structure (minimum 50 total messages):**

1. **Fitness model alignment (messages 1–15):** Do all agents agree on what
   "realistic" means? Converge on 2–3 shared fitness models for simulation.
2. **Best-of-breed review (messages 16–30):** Review all proposals across all
   domains. Are there unchampioned proposals stronger than some champions?
3. **Simplicity and no-actions audit (messages 31–40):** Stress-test each
   champion's one-sentence description under realistic fitness.
4. **Refinement proposals (messages 41–50+):** Modify champions based on
   discussion. May switch champions. May propose hybrid combinations across
   agent domains.

**Output per agent (max 800 words):**
- `docs/resonance/v7/discussion_{1..7}.md`

Each output: agreed fitness models, simplicity ranking, scorecard table
(algorithm x goal), final champion, planned modifications.

---

### Round 3: Simulation (7 parallel agents)

Each agent implements and simulates their champion.

**All agents read:** All 7 discussion outputs plus this plan.

**Requirements:**

1. Implement the 360-card pool with configurable fitness model.
2. Implement the exact championed algorithm.
3. Simulate 1000 drafts x 30 picks x 3 player strategies.
4. **Run under ALL agreed fitness models** (at least optimistic + 2 realistic).
5. Measure all 9 metrics — ALL AT ARCHETYPE LEVEL.
6. Parameter sensitivity sweeps on 2–3 key parameters.
7. 3 detailed draft traces (early committer, flexible player, signal reader).
8. Per-archetype convergence table (8 rows, average pick for 2+ S/A).
9. **NEW: Fitness degradation curve.** Report how each metric changes as
   fitness model moves from optimistic to realistic.
10. Agent 1: run all three baselines. All other agents compare to Agent 1's
    results at each fitness level.
11. Verify no player decisions in the implementation.
12. Test the one-sentence claim.

**Output per agent:**
- `docs/resonance/v7/sim_{1..7}.py`
- `docs/resonance/v7/results_{1..7}.md` (max 1000 words): one-sentence
  algorithm, scorecard at each fitness level, fitness degradation curve,
  per-archetype convergence, baseline comparison, parameter sensitivity,
  draft traces, self-assessment.

---

### Round 4: Cross-Comparison (7-agent team, interactive)

All 7 agents read all simulation results and engage in structured comparison.

**Minimum 40 total messages. Each agent must:**

1. Score each strategy on each design goal (1–10) at each fitness level.
2. Identify the single biggest strength and weakness of each strategy.
3. **KEY QUESTION: Which algorithm degrades most gracefully under realistic
   fitness?** Raw S/A under optimistic is less important than robustness.
4. Compare to all baselines at each fitness level.
5. Propose the best possible algorithm drawing from any V7 strategy or V3–V6.
6. Assess: what cross-archetype fitness rate is required for each algorithm
   to pass all 9 metrics? This gives the card designer a concrete target.

**Output per agent (max 800 words):**
- `docs/resonance/v7/comparison_{1..7}.md`

Each output: scorecard at each fitness level, proposed best algorithm,
minimum fitness requirement for card design, recommendations to the card
designer.

---

### Round 5: Final Synthesis (1 agent)

A single agent produces the definitive comparison and recommendation.

**Reads:** This plan, all comparison outputs, all results, discussion
documents, V6 final report, V6 algorithm overview.

**Task:**

1. Unified comparison table of all algorithms at each fitness level.
2. Rank algorithms by **robustness** (performance under realistic fitness),
   not just by peak optimistic performance.
3. Per-archetype convergence table for the top 3 algorithms under realistic
   fitness.
4. **The key question:** What is the best zero-decision draft algorithm
   when we honestly account for realistic card design constraints? What S/A
   level should we target?
5. Apply the simplicity test independently to each one-sentence description.
6. **Card designer's brief:** For the recommended algorithm, what are the
   minimum card design requirements? How many cards per archetype need to be
   cross-archetype playable? Is this achievable?
7. Write the recommended algorithm with complete specification, one-sentence
   and one-paragraph player descriptions, implementation notes, parameter
   values, and per-archetype convergence table.
8. **V7 vs. V6 comparison:** Did V7 find anything better than Surge Packs?
   If yes, what? If no, what did we learn about the limits of zero-decision
   draft algorithms?
9. **Honest assessment:** What S/A target is realistic given real-world card
   design? Should the target be lowered? What player-experience mitigations
   exist if S/A is lower than 2.0?
10. Open questions for playtesting.

**Output:**
- `docs/resonance/v7/final_report.md` (max 3500 words)
- `docs/resonance/v7/algorithm_overview.md` (max 3000 words) — catalog of all
  algorithms, with fitness-model-adjusted results.

---

## Agent Summary

| Round | Agents | Type | Description |
|-------|--------|------|-------------|
| 1 | 7 | Parallel | Algorithm design per area |
| 2 | 7 | Team (interactive) | Cross-strategy discussion + fitness alignment |
| 3 | 7 | Parallel | Simulation under multiple fitness models |
| 4 | 7 | Team (interactive) | Cross-comparison with robustness focus |
| 5 | 1 | Single | Final synthesis |
| **Total** | **29** | | |

## Output Files

| File | Round | Description |
|------|-------|-------------|
| `design_{1..7}_*.md` (x7) | 1 | Algorithm proposals + champion |
| `discussion_{1..7}.md` (x7) | 2 | Cross-domain discussion |
| `sim_{1..7}.py` (x7) | 3 | Simulation code |
| `results_{1..7}.md` (x7) | 3 | Results at multiple fitness levels |
| `comparison_{1..7}.md` (x7) | 4 | Cross-comparison + robustness analysis |
| `final_report.md` | 5 | Recommendation + specification |
| `algorithm_overview.md` | 5 | Catalog of all algorithms |

All files in `docs/resonance/v7/`.

## Key Principles

1. **No player decisions beyond card selection.** Non-negotiable.
2. **Simplicity is non-negotiable.** One sentence of concrete operations.
3. **ALL MEASUREMENT AT ARCHETYPE LEVEL.** A "Tide card" is NOT a "Warriors
   card."
4. **Respect the 15% constraint.** Max 54 of 360 cards with 2 resonance types.
5. **Test under realistic fitness.** Optimistic-only results are insufficient.
   Every algorithm must report metrics under at least 2 fitness models.
6. **Robustness over peak performance.** An algorithm scoring 1.8 S/A under
   realistic fitness beats one scoring 2.2 under optimistic but 1.3 under
   realistic.
7. **Give the card designer achievable targets.** If the algorithm requires
   80% of cross-resonance cards to be A-tier, say so. If it works with 30%,
   that's a major advantage.
8. **Compare to baselines at every fitness level.** Surge Packs, Lane
   Locking, and Double Enhancement are the references.
9. **Natural variance is a goal.** Consistent delivery is a failure mode.
10. **Prefer visible card properties.** Note explicitly if using archetype
    fitness directly.
11. **The one-sentence description IS the algorithm.**
12. **Explore broadly.** Agents 3–7 have wide latitude. Propose mechanisms
    V3–V6 never considered. Don't just refine — invent.
13. **Test honestly.** Report failures clearly.

## Recovery

Check which `docs/resonance/v7/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained.
