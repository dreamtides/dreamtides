# Resonance Draft System V6 — Orchestration Plan

## Lessons from V3–V5

V3 explored accumulation, structural, threshold, reactive, and pool-manipulation
domains. Its winner, **Lane Locking**, locks pack slots to a resonance at
thresholds of 3 and 8 drafted symbols (primary=2 weight). Lane Locking achieved
2.72 S/A cards per pack at archetype level with convergence at pick 6.1 and
perfect archetype balance. Weaknesses: too mechanical (deterministic slot
assignment), permanently on rails after commitment, 99% deck concentration.

V4 explored rejection, probabilistic, economic, phantom drafter, and filtered
sampling domains. Its winner, **Pack Widening v2**, lets the player spend
earned resonance tokens to add bonus cards to packs. Pack Widening achieved
3.35 S/A at convergence pick 6.0 with better variance (0.94 stddev) and deck
diversity than Lane Locking. Weakness: requires player spending decisions.

**V4's structural finding:** Probabilistic resonance-based mechanisms
(weighting, filtering, exile, phantoms) are structurally capped at 1.26–1.74
S/A because each resonance is shared by 4 archetypes — roughly 50% of
resonance-matched cards belong to the wrong archetype. Only mechanisms that ADD
targeted cards to packs or deterministically PLACE them in specific slots cross
the 2.0 threshold. This is a mathematical limit, not a tuning problem.

V5 explored pair-based matching (using ordered resonance pairs like
[Tide, Zephyr] to identify archetypes with ~100% precision) as a way to break
through the dilution ceiling. **V5 was abandoned** because it relied on 70%+ of
cards having dual-resonance symbols, which is effectively archetype labeling —
it defeats the purpose of resonance as a draft signal.

**What V6 must solve:** Find zero-decision algorithms that cross the 2.0 S/A
threshold under a strict constraint: no more than 15% of cards may carry two
different resonance types. Most cards are mono-resonance, so the 50% archetype
dilution from V4 is the central challenge. Pair-based matching cannot be a
primary strategy.

---

## The Resonance System

### Four Resonance Types

Dreamtides has four resonance types: **Ember**, **Stone**, **Tide**, **Zephyr**.

Each card has between 0 and 3 **resonance symbols** printed on it. The symbols
are **ordered** — the leftmost symbol is the card's primary resonance. A card
with symbols [Tide, Tide, Tide] has 3 symbols but only 1 resonance type.

### The 15% Constraint (New for V6)

**No more than 15% of cards may have 2 different resonance types.** This means:

- [Tide] — 1 symbol, 1 type → allowed (no limit)
- [Tide, Tide] — 2 symbols, 1 type → allowed (no limit)
- [Tide, Tide, Tide] — 3 symbols, 1 type → allowed (no limit)
- [Tide, Zephyr] — 2 symbols, 2 types → **counts toward the 15% cap**
- [Tide, Tide, Zephyr] — 3 symbols, 2 types → **counts toward the 15% cap**

With 360 cards, at most **54 cards** may have dual-resonance types. These rare
dual-type cards are valuable archetype signals (an ordered pair like
[Tide, Zephyr] maps directly to Warriors), but algorithms cannot rely on them
as a primary strategy.

**Consequence:** Most cards (~270 of 324 non-generic) carry only one resonance
type. A player drafting Tide cards could be building any of 4 archetypes:
Warriors, Sacrifice, Self-Mill, or Ramp. This ambiguity is the core design
challenge. The algorithm must converge toward the player's specific archetype
despite only having single-resonance signals for ~85% of non-generic cards.

### Eight Archetypes on a Circle

The 8 archetypes are arranged in a circle. Each resonance type sits between
its two core archetypes:

1. **Flash/Tempo/Prison** — Zephyr primary, Ember secondary
2. **Blink/Flicker** — Ember primary, Zephyr secondary
   *(Ember sits between 2 and 3)*
3. **Storm/Spellslinger** — Ember primary, Stone secondary
4. **Self-Discard** — Stone primary, Ember secondary
   *(Stone sits between 4 and 5)*
5. **Self-Mill/Reanimator** — Stone primary, Tide secondary
6. **Sacrifice/Abandon** — Tide primary, Stone secondary
   *(Tide sits between 6 and 7)*
7. **Warriors/Midrange** — Tide primary, Zephyr secondary
8. **Ramp/Spirit Animals** — Zephyr primary, Tide secondary
   *(Zephyr sits between 8 and 1, wrapping the circle)*

**Key property:** Each resonance is the primary for exactly 2 archetypes and
the secondary for exactly 2 archetypes. Adjacent archetypes share a resonance.

### CRITICAL: Resonance ≠ Archetype

**Each resonance is shared by 4 archetypes.** Tide is the primary for Warriors
and Sacrifice, and the secondary for Self-Mill and Ramp. A "Tide card" could
belong to any of those 4 archetypes. If a player commits to Warriors, seeing
a Tide card is NOT the same as seeing a Warriors card — roughly half of Tide
cards belong to wrong-archetype decks.

For the ~15% of cards with dual resonance types, the ordered pair (primary,
secondary) does uniquely identify the home archetype. A [Tide, Zephyr] card is
almost certainly Warriors. But these cards are rare by design — algorithms must
primarily work with the ambiguous single-resonance signals.

### Symbol Counting Rules

When counting a player's resonance symbols:
- **Primary symbol** (first/leftmost): counts as **2**
- **Secondary symbol** (second position): counts as **1**
- **Tertiary symbol** (third position): counts as **1**

This makes the ordering of symbols matter mechanically.

---

## The Problem

Design a draft algorithm that requires NO player action beyond picking 1 card
from a pack of 4. The algorithm must use resonance symbols and/or other visible
card properties to construct packs that naturally converge toward the player's
archetype while maintaining variance.

**The hard constraint:** The player's only action each pick is choosing 1 card
from the presented pack. No spending, no mode selection, no rerolling, no
resource management, no "before pack" decisions. Everything the algorithm does
must be automatic and passive, triggered solely by the player's card selections.

**What V3 and V4 established:**
- Pure probabilistic approaches (weighting, filtering) cap at ~1.7 S/A due to
  resonance-archetype dilution
- You must either ADD targeted cards to packs or deterministically PLACE them in
  slots to cross 2.0 S/A
- Lane Locking crosses 2.0 but feels mechanical
- Pack Widening crosses 2.0 but requires player decisions (banned for V6)
- Pair-based matching (V5) breaks the dilution ceiling but requires most cards
  to have dual resonance, which is now capped at 15%

**V6's design space:** Algorithms that automatically achieve 2.0+ S/A
convergence while feeling natural, under the 15% dual-resonance constraint.

---

## Design Goals

Ranked by priority:

1. **Simple.** Explainable to players in one sentence of concrete operations.
   NOT "the system nudges you toward your archetype." YES "each symbol you
   draft adds a matching token; when any type reaches 3, your next pack gets a
   bonus card of that type and 3 tokens are deducted."
2. **No extra actions.** The player's only action is picking 1 card from the
   pack. Everything else is automatic. THIS IS NON-NEGOTIABLE.
3. **Not on rails.** The player should not be forced into one archetype or have
   only 1 real choice per pack.
4. **No forced decks.** The player should not be able to force the same deck
   every time.
5. **Flexible archetypes.** Possible to build outside core archetypes or combine 2.
6. **Convergent.** After committing (~pick 6), see a minimum of 2 S/A cards for
   the specific archetype in most packs.
7. **Splashable.** ~1 off-archetype card in most packs.
8. **Open-ended early.** Picks 1–5 show variety from different archetypes.
9. **Signal reading.** Moderate benefit to identifying the open archetype.

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

### Variance Target

| Metric | Target |
|--------|--------|
| StdDev of S/A cards per pack (picks 6+) | >= 0.8 |

### Simplicity Test

Every proposed algorithm must pass: **Can you write the complete algorithm as
a one-sentence instruction to a programmer?** The sentence must describe
concrete operations on concrete data. It must NOT include any player decisions.

---

## Simulation Card Model

```python
class SimCard:
    id: int
    symbols: list[Resonance]  # ordered, 0–3 elements, [] = generic
    archetype: str            # primary archetype (for EVALUATION only)
    archetype_fitness: dict   # archetype_id -> tier (S/A/B/C/F) — EVALUATION only
    rarity: Rarity
    power: float              # raw card strength (0–10)
```

**Preferred separation:** The draft algorithm should use **visible card
properties** — `symbols`, `rarity`, `power`. The `archetype_fitness` scores are
for evaluation. If an algorithm that uses archetype fitness directly produces
dramatically better results, note this explicitly.

### Card Pool Construction

Generate 360 cards:

**Per archetype (~40 cards each, 320 total across 8 archetypes):**

Each archetype's cards carry symbols from that archetype's resonances. The
distribution of symbol counts (1 vs 2 vs 3 symbols per card) and the split
between mono-resonance and dual-resonance cards are open design questions for
agents to explore, subject to the constraint below.

**The 15% dual-resonance constraint:** Of the 360 total cards, no more than 54
may have 2 different resonance types. Agents should propose how to distribute
these dual-type cards across archetypes.

**Example distributions (agents should propose and justify their own):**
- Conservative: 5% dual-type (18 cards), rest mono-type
- Moderate: 10% dual-type (36 cards)
- Maximum: 15% dual-type (54 cards)

**Mono-resonance cards** use only the archetype's primary resonance:
- Warriors cards: [Tide], [Tide, Tide], [Tide, Tide, Tide]

**Dual-resonance cards** use primary + secondary resonance:
- Warriors cards: [Tide, Zephyr], [Tide, Tide, Zephyr]

**Generic cards (36 total, ~10%):** No symbols. Playable in any deck.

**Fitness assignment (for evaluation only):**
- S-tier in home archetype
- A-tier in the adjacent archetype sharing primary resonance
- B-tier in archetypes sharing secondary resonance
- C/F in distant archetypes
- Generic cards: B-tier in all archetypes

### Simulated Player Strategies

- **Archetype-committed:** Picks highest fitness for strongest archetype.
  Commits around pick 5–6.
- **Power-chaser:** Picks highest raw power regardless of archetype.
- **Signal-reader:** Evaluates which resonance/archetype seems most available
  and drafts toward it.

---

## The Seven Investigation Areas

Each agent explores one area, brainstorms 5 concrete algorithms within it, and
champions one for simulation. Agent 1 is the Lane Locking reference — its
champion is predetermined.

### Agent 1: Lane Locking (Reference Baseline)

Implement V3's Lane Locking algorithm as a reference baseline. When you draft
3 weighted symbols of a resonance, one pack slot locks to that resonance.
A second slot locks at 8. Locked slots always show a random card of the locked
resonance. Run the same simulations and report the same metrics as all other
agents, using V6's card pool (with the 15% dual-resonance constraint).

This agent does NOT need to brainstorm 5 alternatives — its purpose is to
provide an apples-to-apples comparison point. However, it SHOULD report on how
the 15% constraint affects Lane Locking's performance compared to V3's results
(which used a different card pool).

### Agent 2: Auto-Widening

Automatically add bonus resonance-matched cards to packs when accumulated
resonance tokens hit a threshold. The player earns tokens passively from
drafted symbols; spending is automatic. Explore different auto-spend policies
(highest resonance, round-robin, most recent, etc.), threshold values, and
bonus card selection methods.

### Agent 3: Soft Slot Targeting

Give each pack slot a probability of showing a resonance-matched card that
scales with the player's symbol count. Unlike Lane Locking's binary locks,
this is a continuous probability curve — never 0%, never 100%. Explore
different probability functions, whether to target one resonance or multiple,
and diminishing returns.

### Agent 4: Pool Evolution

Change the card pool itself as the draft progresses. When a player drafts a
card, add similar cards to the pool from a reserve. Pack generation stays
simple (draw random from pool) but the pool drifts toward the player's
resonance. Explore seeding rates, whether to also remove off-resonance cards,
and net pool size changes.

### Agent 5: Conditional Pack Enhancement

Generate a random base pack, then inspect it — if the pack naturally clusters
with the player's resonance profile, add a bonus card or replace the
worst-fitting card. The pack's own random composition triggers the enhancement.
Explore different trigger conditions and enhancement methods.

### Agent 6: Escalating Influence

Design mechanisms whose strength increases over the draft timeline. Early picks
are fully random; later picks are increasingly biased toward the player's
resonance. Explore different escalation curves, what "influence" means
(weighted sampling, slot targeting, pool changes, etc.), and how to avoid
feeling too mechanical by pick 25.

### Agent 7: Open Exploration

Propose any mechanism not covered by Agents 2–6. This could be a hybrid of
multiple approaches, a genuinely novel mechanism, or a creative riff on a
V3/V4 idea adapted for V6's constraints. The only requirement is the same as
all agents: zero player decisions, one-sentence describable, targeting 2.0+ S/A.

---

## Rounds

### Round 1: Algorithm Design (7 parallel agents)

Each agent explores their assigned area. Pure reasoning — no simulation code.

**All agents read:** This orchestration plan, plus the V3 final report
(`docs/resonance/v3/final_report.md`) and V4 final report
(`docs/resonance/v4/final_report.md`).

**Agent 1 (Lane Locking) produces (max 1000 words):**
- Lane Locking specification adapted for V6's card pool
- Analysis of how the 15% constraint affects its performance
- Predicted metric values

**Agents 2–7 each produce (max 2000 words):**

1. **Key Takeaways** (5–7 bullets)
2. **Five algorithm proposals:** Name, one-sentence description, 2–3 sentence
   technical description, quick design goal assessment, preferred symbol
   distribution.
3. **Champion selection** with justification.
4. **Champion deep-dive:** Example draft sequences, failure modes, 2–3
   parameter variants, proposed symbol distribution.

**Output files:**

| Agent | Output |
|-------|--------|
| 1 | `docs/resonance/v6/design_1_lane_locking.md` |
| 2 | `docs/resonance/v6/design_2_auto_widening.md` |
| 3 | `docs/resonance/v6/design_3_soft_slot.md` |
| 4 | `docs/resonance/v6/design_4_pool_evolution.md` |
| 5 | `docs/resonance/v6/design_5_conditional.md` |
| 6 | `docs/resonance/v6/design_6_escalating.md` |
| 7 | `docs/resonance/v6/design_7_open.md` |

---

### Round 2: Cross-Strategy Discussion (7-agent team, interactive)

All 7 agents read all Round 1 design documents and engage in structured debate.

**Discussion structure (minimum 50 total messages):**

1. **Best-of-breed review (messages 1–15):** Each agent reviews all proposals
   across all domains. Are there unchampioned proposals stronger than some
   champions?
2. **Simplicity and no-actions audit (messages 16–25):** Stress-test each
   champion's one-sentence description. Can you implement it from the
   description alone? Any hidden player decisions?
3. **Goal tradeoff analysis (messages 26–35):** For each design goal, which
   champion is best? Worst? Focus on convergence — does it cross 2.0?
4. **Refinement proposals (messages 36–50):** Each agent proposes modifications
   to their champion based on discussion. May switch champions.

**Output per agent (max 800 words):**
- `docs/resonance/v6/discussion_{1..7}.md`

Each output: simplicity ranking, scorecard table (algorithm × goal), final
champion, planned modifications, proposed symbol distribution.

---

### Round 3: Simulation (7 parallel agents)

Each agent implements and simulates their champion.

**All agents read:** All 7 discussion outputs plus this plan.

**Requirements:**

1. Implement the 360-card pool with the 15% dual-resonance constraint.
2. Implement the exact championed algorithm.
3. Simulate 1000 drafts × 30 picks × 3 player strategies.
4. Measure all 8 metrics + variance target — ALL AT ARCHETYPE LEVEL.
5. Parameter sensitivity sweeps on 2–3 key parameters + symbol distribution.
6. 3 detailed draft traces (early committer, flexible player, signal reader).
7. Pack-quality variance report (distribution of S/A per pack, stddev).
8. Per-archetype convergence table (8 rows, average pick for 2+ S/A).
9. **Agent 1 only:** Also run auto-spend Pack Widening (cost 3, bonus 1,
   auto-spend on highest resonance) as a second baseline. All other agents
   compare to Agent 1's Lane Locking and Pack Widening results.
10. Test the one-sentence claim: can you reconstruct the algorithm from just
    the description?
11. Verify no player decisions in the implementation.

**Output per agent:**
- `docs/resonance/v6/sim_{1..7}.py`
- `docs/resonance/v6/results_{1..7}.md` (max 800 words): one-sentence
  algorithm, scorecard, variance report, per-archetype convergence, baseline
  comparison, symbol distribution, parameter sensitivity, draft traces,
  self-assessment (each goal 1–10).

---

### Round 4: Cross-Comparison (7-agent team, interactive)

All 7 agents read all simulation results and engage in structured comparison.

**Minimum 40 total messages. Each agent must:**

1. Score each strategy on each design goal (1–10) with 1-sentence justification.
2. Identify the single biggest strength and weakness of each strategy.
3. Propose specific improvements.
4. Compare to Lane Locking and Pack Widening baselines: does any V6 algorithm
   clearly beat both?
5. Propose the best possible algorithm drawing from any V6 strategy or V3/V4.
6. Assess: did the 15% dual-resonance constraint make the problem harder or
   just different? What would change at 10% or 20%?

**Output per agent (max 800 words):**
- `docs/resonance/v6/comparison_{1..7}.md`

Each output: scorecard, proposed best algorithm with one-sentence description,
assessment of 15% constraint impact.

---

### Round 5: Final Synthesis (1 agent)

A single agent produces the definitive comparison and recommendation.

**Reads:** This plan, all comparison outputs, all results, discussion
documents, V3 and V4 final reports.

**Task:**

1. Run all 7 simulations with identical parameters. Unified comparison table
   including Lane Locking and Pack Widening baselines — all at archetype level.
2. Rank the 7 algorithms by overall design goal satisfaction.
3. Per-archetype convergence table for all algorithms (8 archetypes × 7+
   algorithms). Flag convergence outside pick 4–8 range.
4. **The key question:** Does any V6 algorithm beat both Lane Locking and Pack
   Widening? Zero decisions + natural variance + 2.0+ S/A?
5. Apply the simplicity test independently to each one-sentence description.
6. Write the recommended algorithm with:
   - Complete specification
   - One-sentence and one-paragraph player descriptions
   - Implementation notes and parameter values
   - Recommended symbol distribution
   - Per-archetype convergence table
7. **V6 vs V3 vs V4 deep comparison** — quantitative (side-by-side metrics)
   and qualitative (player experience, cognitive load, transparency,
   flexibility, skill expression, simplicity, degeneracy resistance, archetype
   balance). Clear verdict.
8. Open questions for playtesting.

**Output:**
- `docs/resonance/v6/final_report.md` (max 3500 words)
- `docs/resonance/v6/algorithm_overview.md` (max 3000 words) — catalog of all
  algorithms considered (all 30+ proposals), with one-sentence descriptions,
  simulation results if applicable, and why each was or wasn't championed.

---

## Agent Summary

| Round | Agents | Type | Description |
|-------|--------|------|-------------|
| 1 | 7 | Parallel | Algorithm design per area |
| 2 | 7 | Team (interactive) | Cross-strategy discussion |
| 3 | 7 | Parallel | Simulation + results |
| 4 | 7 | Team (interactive) | Cross-comparison |
| 5 | 1 | Single | Final synthesis |
| **Total** | **29** | | |

## Output Files

| File | Round | Description |
|------|-------|-------------|
| `design_{1..7}_*.md` (×7) | 1 | Algorithm proposals + champion |
| `discussion_{1..7}.md` (×7) | 2 | Cross-domain discussion |
| `sim_{1..7}.py` (×7) | 3 | Simulation code |
| `results_{1..7}.md` (×7) | 3 | Results + scorecards |
| `comparison_{1..7}.md` (×7) | 4 | Cross-comparison + proposals |
| `final_report.md` | 5 | Recommendation + specification |
| `algorithm_overview.md` | 5 | Catalog of all algorithms |

All files in `docs/resonance/v6/`.

## Key Principles

1. **No player decisions beyond card selection.** Non-negotiable.
2. **Simplicity is non-negotiable.** One sentence of concrete operations.
3. **ALL MEASUREMENT AT ARCHETYPE LEVEL.** A "Tide card" is NOT a "Warriors
   card." If your simulation counts resonance matches, your numbers are ~2×
   too high.
4. **Respect the 15% constraint.** No more than 54 of 360 cards may have 2
   different resonance types. Do not design around pair-based matching as a
   primary mechanism.
5. **Cross 2.0 S/A or explain why not.** V3 and V4 both did it.
6. **Natural variance is a goal.** Sometimes great packs, sometimes bad packs.
   Consistent mechanical delivery is a failure mode.
7. **Compare to both baselines.** Lane Locking (convergence reference) and
   auto-spend Pack Widening (zero-decision reference). Every simulation
   includes both.
8. **Prefer visible properties.** Algorithms using only symbols, rarity, and
   power are preferred. Note explicitly if using archetype fitness.
9. **The one-sentence description IS the algorithm.** If your code does
   something the sentence doesn't mention, simplify or fix the sentence.
10. **Symbol distribution is an open question.** How many 1 vs 2 vs 3 symbol
    cards, and how many mono vs dual resonance type? Each agent proposes and
    justifies their distribution within the 15% cap.
11. **Test honestly.** Report failures clearly.

## Recovery

Check which `docs/resonance/v6/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained.
