# Resonance Draft System V11 — Orchestration Plan

## The Central Idea

V11 introduces **multi-round drafting with pool refills** — the same structure
that makes real booster drafts work.

Instead of a single 360-card pool that depletes monotonically, the draft
operates in multiple rounds. Each round, the player and AI opponents draft from
a shared pool. Between rounds, the pool is replenished with fresh cards —
analogous to opening a new pack in a booster draft. This periodic replenishment
solves both of V10's structural failures (pool exhaustion and S/A preferential
depletion) while preserving the AI drafter narrative that V10 validated.

**Player-facing explanation:** "You're drafting at a table with AI opponents.
Each round, everyone opens new packs and picks from the shared pool. Read which
archetypes are open and commit before the good cards are gone."

### Why This Fixes V10's Failures

**Pool exhaustion (V10 root cause 1):** V10's 360-card pool was depleted by pick
12-15 because flat per-round removal didn't self-regulate. Multi-round refills
make pool exhaustion impossible — the pool is replenished before it runs dry.
The draft always has fresh cards to offer.

**S/A preferential depletion (V10 root cause 2):** V10's AIs took the best cards
first, draining S/A density from the pool. With refills, fresh S/A cards enter
the pool each round. Crucially, *refills replenish all archetypes equally*, but
AIs only deplete *their* archetypes. Over multiple rounds, the net effect is:
- AI-lane archetypes: S/A cards are drained by AIs, partially replenished by
  refills → declining S/A density
- Open-lane archetypes: S/A cards are NOT drained (no AI is taking them),
  replenished by refills → accumulating S/A density
- This produces natural, emergent concentration toward open lanes through honest
  physical removal — no virtual contraction needed.

**Targeting dilution (V10 root cause 3):** V10's Level 0 AIs could only
concentrate toward open archetypes generally (player's archetype = 1 of 3).
Multi-round drafting improves this because the player is the *only* drafter
taking cards from their specific open-lane archetype. Over multiple rounds, the
player cherry-picks the best cards from their lane while AIs deplete other lanes.
The player's lane may still share the pool with 2 other open archetypes, but
the natural process of AIs depleting 5 lanes while 3 remain untouched creates
progressive concentration.

### What V10 Established (Carried Forward)

- **Level 0 (static) AIs** are optimal for signal reading and fairness narrative
- **5-AI / 3-open-lane structure** was V10's best-performing configuration, but
  the AI count and open-lane ratio are negotiable in V11's multi-round context
- **Pair-affinity pick logic** (8-bit) is needed for same-resonance sibling
  discrimination
- **Deckbuilding-aware saturation** is the best AI pick refinement
- **The AI drafter narrative** is a genuine contribution to player experience

---

## What Changed Since V10

### V10 Key Results (for agent reference)

All six V10 algorithms failed catastrophically on M3, M10, M11. Best results:
Hybrid X (M3 = 0.84, M11 = 0.69) — 69-79% below V9 targets.

**Three structural root causes identified:**
1. Physical pool depletion (flat removal exhausts 360-card pool by pick 12-15)
2. S/A preferential depletion (AIs take best cards first, depleting quality)
3. Level 0 targeting dilution (AIs concentrate generally, not toward player's
   specific archetype)

**V10 also validated:** Level 0 reactivity, 5-AI/3-lane structure, pair-affinity
pick logic, saturation mechanic, AI drafter narrative value.

### V10 → V11 Hypothesis

Multi-round refills should resolve root causes 1 and 2 directly (pool never
exhausts, S/A cards are replenished). Root cause 3 (targeting dilution) may
partially resolve through cumulative multi-round concentration — AIs deplete
their lanes over time while open lanes accumulate. V11 explores whether this
emergent concentration is sufficient for M3 >= 2.0 and the relaxed M11' target.

---

## Fixed Assumptions (Not Variables in V11)

### Fitness Model: Graduated Realistic (Fixed)

| Pair | Sibling A-Tier Rate |
|------|:---:|
| Warriors / Sacrifice (Tide) | 50% |
| Self-Discard / Self-Mill (Stone) | 40% |
| Blink / Storm (Ember) | 30% |
| Flash / Ramp (Zephyr) | 25% |
| **Weighted Average** | **~36%** |

### Total Draft: 30 Picks (Fixed)

The player drafts 30 cards total across all rounds. This is constant regardless
of round structure.

### Archetypes: 8 on a Circle (Fixed)

1. Flash/Tempo — Zephyr primary, Ember secondary
2. Blink/Flicker — Ember primary, Zephyr secondary
3. Storm/Spellslinger — Ember primary, Stone secondary
4. Self-Discard — Stone primary, Ember secondary
5. Self-Mill/Reanimator — Stone primary, Tide secondary
6. Sacrifice/Abandon — Tide primary, Stone secondary
7. Warriors/Midrange — Tide primary, Zephyr secondary
8. Ramp/Spirit Animals — Zephyr primary, Tide secondary

### Visible Symbol Distribution (Fixed from V9)

| Symbol Count | Cards | % |
|:---:|:---:|:---:|
| 0 (generic) | ~11% of pool | 11% |
| 1 visible symbol | ~79% of pool | 79% |
| 2 visible symbols | ~10% of pool | 10% |

Proportions apply to each refill batch as well as the starting pool.

### Player Strategies (Fixed)

- **Archetype-committed:** Picks highest fitness for strongest archetype.
  Commits around pick 5-6.
- **Power-chaser:** Picks highest raw power regardless of archetype.
- **Signal-reader:** Evaluates which archetype seems most available and drafts
  toward it.

---

## Modified Metrics

### Relaxed M11 → M11'

V9-V10 used M11 = picks 15+, S/A >= 3.0 per pack. This was extremely strict
and only V9 Hybrid B ever passed it (3.25). V11 relaxes this:

**M11' = picks 20+, S/A cards for committed archetype per pack >= 2.5**

This acknowledges that late-draft concentration is important but need not be as
extreme as V9 demanded. Picks 20+ (the final third of the draft) is a fairer
window for measuring late-draft quality.

### New Metric: M12 (Pool Information Value)

**M12 = Signal-reader M3 minus Committed M3.** Measures how much benefit a
player gets from reading pool information. Target: >= 0.3 (signal reading
should provide meaningful but not overwhelming advantage).

### Full Metric Table

| ID | Metric | Target |
|----|--------|--------|
| M1 | Picks 1-5: unique archetypes with S/A cards per pack | >= 3 of 8 |
| M2 | Picks 1-5: S/A cards for emerging archetype per pack | <= 2 |
| M3 | Picks 6+: S/A cards for committed archetype per pack | >= 2.0 avg |
| M4 | Picks 6+: off-archetype (C/F) cards per pack | >= 0.5 |
| M5 | Convergence pick | Pick 5-8 |
| M6 | Deck archetype concentration | 60-90% S/A-tier cards |
| M7 | Run-to-run variety | < 40% card overlap |
| M8 | Archetype frequency across runs | No archetype > 20% or < 5% |
| M9 | StdDev of S/A cards per pack (picks 6+) | >= 0.8 |
| M10 | Max consecutive packs below 1.5 S/A (picks 6+) | <= 2 |
| M11' | Picks 20+: S/A cards for committed archetype per pack | >= 2.5 |
| M12 | Signal-reader M3 minus Committed M3 | >= 0.3 |

### V9 Baselines (for comparison)

Hybrid B: M3 = 2.70, M11(old) = 3.25, M10 = 3.8, M5 = 9.6, M6 = 86%

---

## The Multi-Round Design Space

This is V11's core contribution. Four interconnected design variables:

### Variable 1: Pool Size and Round Structure

How many cards start in the pool, and how is the draft divided into rounds?

| Config | Starting Pool | Rounds | Picks/Round | Character |
|--------|:---:|:---:|:---:|-----------|
| A: Small/Frequent | 80-100 | 5-6 | 5-6 | Fast cycling, strong per-round signals |
| B: Medium/3-Pack | 120-150 | 3 | 10 | Classic booster draft feel |
| C: Large/Rare | 180-240 | 2 | 15 | Closer to V10, fewer refill points |

The pool should be large enough for meaningful early picks but small enough that
AI picks create visible scarcity within each round.

### Variable 2: Refill Quantity

How many cards are added at each refill? Options:

- **Full replenishment:** Refill to starting pool size. Pool never shrinks.
  Maximum variety but may prevent concentration.
- **Partial replenishment:** Refill 50-75% of what was consumed. Pool
  gradually shrinks across rounds, creating natural late-draft concentration.
- **Declining refills:** First refill is large, later refills are smaller.
  Early variety → late concentration.
- **Fixed refill:** Same number of cards each time regardless of consumption.

### Variable 3: Refill Bias

What archetype distribution do refill cards have?

- **Balanced:** Equal cards per archetype (~5 per archetype per refill for 40
  total). Fair, predictable, maintains supply for all archetypes.
- **Random from master pool:** Draw from the same 360-card master pool as the
  starting cards. Creates natural variance.
- **Weighted toward underrepresented:** Refill preferentially restocks archetypes
  that have been depleted below a threshold. Justified as "the market restocks
  what's been selling out." Acts as a soft concentration mechanism.
- **Weighted toward player (mild Level 1):** Refill includes slightly more
  cards from the player's emerging archetype. The player's visible picks inform
  the refill — a mild reactivity that could be framed as "the market responds
  to customer demand." Requires careful calibration to avoid feeling rigged.

### Variable 4: AI Behavior Across Rounds

How do AIs behave across multiple rounds?

- **Consistent:** Same archetype preference, same pick logic all game. Most
  predictable; strongest signal reading.
- **Escalating:** AIs become more focused in later rounds (more archetype picks,
  fewer generics). Mirrors real drafter behavior.
- **Saturating:** AIs ease off their archetype after accumulating enough cards
  (V10's D3 mechanic). Creates natural late-round convergence.
- **Round-aware:** AIs adjust pick priority based on what's available in the new
  pool (not player-reactive — pool-reactive). Justified as "they opened their
  new packs and recalibrated."

---

## Pool Information Design Space

V11 introduces the possibility that players receive explicit information about
the pool state. This is a new design axis not explored in V9 or V10.

### What Information Could Be Shown?

1. **Archetype availability bars:** Visual indicator showing relative card counts
   per archetype currently in the pool. "Tide cards: ████░░ (many available)"
   vs "Ember cards: ██░░░░ (scarce)". Shows which lanes are open.

2. **Refill preview:** Before/during refill, show what types of cards are being
   added. "New shipment: mostly Stone and Zephyr cards." Helps the player plan
   for the next round.

3. **Archetype trend indicators:** Show which archetypes are being depleted
   fastest (implying AI activity). "Trending down: Blink, Warriors."

4. **AI pick hints:** Show the most recent card type each AI drafted (but not
   the specific card). "AI 1 took a Tide card. AI 3 took an Ember card."

5. **Pool composition summary:** Show how many cards of each rarity and
   resonance type are in the current pool. Pure statistics — player infers
   the rest.

6. **Round-start snapshot:** At the beginning of each round (after refill),
   show a complete pool composition summary. Player sees what's available before
   drafting begins.

### Design Questions

- How much information is "enough" to create a skill axis without making signal
  reading trivial?
- Should information be free or require a "scouting" action that costs a pick?
- Should all players (real and AI) have the same information access?
- Does explicit pool info reduce the "reading the table" skill by making it
  mechanical rather than intuitive?

---

## Simulation Card Model

```python
class SimCard:
    id: int
    visible_symbols: list[Resonance]   # what the player sees (0-2 symbols)
    archetype: str                     # primary archetype (for evaluation)
    archetype_fitness: dict            # archetype_id -> tier
    power: float                       # raw card strength (0-10)
    pair_affinity: dict                # {archetype_a: float, archetype_b: float}

class AIDrafter:
    archetype_preference: str          # the lane this AI drafts
    cards_drafted: list[SimCard]       # tracking for saturation
    saturation_threshold: int          # when to ease off

class DraftRound:
    pool: list[SimCard]                # current available pool
    round_number: int
    picks_this_round: int
    refill_cards: list[SimCard]        # what gets added next round
```

---

## Round 1: Research (3 parallel agents)

Pure research — no algorithm design. Map the multi-round design space.

### Research Agent A: Multi-Round Draft Formats

**Question:** How do existing games structure multi-round / multi-pack drafts,
and what creates good pacing and decision points between rounds?

Explore:
- How does MTG's 3-pack structure affect draft strategy? What changes between
  Pack 1, Pack 2, and Pack 3? How does the "new pack" moment feel?
- How do other games with multi-phase drafts (7 Wonders, Sushi Go, Blood Rage)
  handle the transition between rounds?
- What pool sizes produce the right balance of scarcity and variety within each
  round?
- How does the number of rounds affect strategic depth? Is 3 the right number,
  or do more/fewer rounds produce better outcomes?
- What does "reading the table" feel like when new cards arrive periodically?

**Output:** `docs/resonance/v11/research_multi_round.md` (max 2000 words)

### Research Agent B: Pool Information in Drafts

**Question:** How do existing games provide players with information about
available cards, and what level of visibility creates the best skill axis?

Explore:
- In MTG draft, all information is inferred (you see what was passed to you).
  In 7 Wonders, you see other players' tableaus. In some deckbuilders, the
  market is visible. What are the tradeoffs?
- Does explicit pool information (seeing what's available) make drafting better
  or worse? More strategic or more mechanical?
- How much information creates "informed decisions" vs "solved puzzle"?
- What information formats work best in games? (counts, categories, specific
  cards, trends)
- How does information interact with AI opponents? Should the player know what
  the AIs are taking, or only see the result?

**Output:** `docs/resonance/v11/research_pool_info.md` (max 2000 words)

### Research Agent C: V10 Remediation Analysis

**Question:** What specifically does the multi-round refill mechanism fix in
V10's failures, and what might it NOT fix?

Analyze:
- Model the math: If the pool starts at 120 cards, 5 AIs take 1 card each per
  pick, the player takes 1, and we refill to 120 every 10 picks — what does
  the archetype composition look like at each refill? Does the open-lane
  archetype naturally concentrate?
- Does refilling counteract S/A preferential depletion? If AIs drain S/A cards
  from their lanes but refills add S/A cards back proportionally, what's the
  net S/A trajectory for the player's lane?
- Does targeting dilution (player = 1 of 3 open lanes) persist, or does the
  multi-round accumulation reduce it?
- What refill quantities and biases produce the best concentration trajectory?
- Are there failure modes unique to multi-round that don't exist in single-pool
  designs? (e.g., "refill reset" where concentration built in round 1 is
  washed out by round 2's fresh cards)

**Reads:** This plan, V10 final report, V10 algorithm overview.

**Output:** `docs/resonance/v11/research_v10_remediation.md` (max 2000 words)

---

## Round 2: Algorithm Design (6 parallel agents)

Each agent reads all Round 1 research plus this plan, V10 final report, and V10
algorithm overview. Each explores a different region of the multi-round design
space.

**Fixed for all agents:**
- Fitness: Graduated Realistic
- Total draft: 30 picks
- Visible symbols: ~10% dual-res
- AI drafter framing required
- All V10 structural findings available
- Level 0 reactivity is the baseline; Level 1 (pool-reactive) is acceptable if
  justified

**Output format (all agents):**

1. Key findings (5-7 bullets)
2. Three algorithm proposals: name, one-sentence description, technical spec,
   predicted M3/M10/M11'/M6/M12
3. Champion selection with justification
4. Champion deep-dive: round-by-round walkthrough, refill mechanics, what the
   player sees, pool composition evolution, failure modes
5. Complete specification (pool size, rounds, picks/round, refill quantity,
   refill bias, AI count, AI pick logic, player information)

Max 1500 words per agent.

### Agent 1: 3-Pack Classic (MTG-Inspired)

**Starting point:** 3 rounds of 10 picks. Medium pool (~120 cards). Full or
near-full replenishment between rounds.

**Question:** Can a classic 3-pack structure with AI drafters produce M3 >= 2.0
and M11' >= 2.5 through emergent concentration alone?

Explore:
- What starting pool size works with 5 AIs taking 1 card each per pick?
- Does full replenishment prevent concentration, or does cumulative AI depletion
  of their lanes naturally concentrate open lanes?
- What does the round-over-round archetype composition trajectory look like?
- How does the "new pack" moment affect player decision-making?

### Agent 2: Small Pool, Fast Cycles

**Starting point:** 5-6 rounds of 5-6 picks. Small pool (~60-80 cards).
Frequent small refills.

**Question:** Do frequent refills with a small pool create stronger per-round
signals and faster convergence than fewer rounds with a larger pool?

Explore:
- With a 60-card pool and 5 AIs taking 1 each, 5 cards are removed per pick
  (6 including player). After 5 picks, 30 cards removed from 60 — is the pool
  too thin?
- Do short rounds create stronger within-round lane signals?
- Does frequent refilling prevent long-term concentration (the "reset" problem)?
- How does the player's commitment decision change when rounds are short?

### Agent 3: Declining Refills (Natural Concentration Ramp)

**Starting point:** 3-4 rounds with decreasing refill sizes. First refill is
large (full replenishment), later refills are smaller (50%, then 25%).

**Question:** Can declining refill quantities produce a natural concentration
ramp — open early, concentrated late — that matches V9's trajectory?

Explore:
- What declining refill schedule produces V9-equivalent concentration by pick 20?
- Does the concentration feel natural ("the supply is running low") or
  artificial ("the game is restricting me")?
- How does the shrinking pool interact with AI saturation mechanics?
- Is there a sweet spot where the final round is concentrated enough for M11'
  but not so thin that it feels repetitive?

### Agent 4: Biased Refills

**Starting point:** Explore different refill bias strategies rather than a
specific round structure. Can apply to any of the structures from Agents 1-3.

**Question:** What refill bias produces the best concentration without feeling
unfair?

Explore:
- **Balanced refills:** Equal per-archetype. Does this achieve M3 >= 2.0
  through emergent concentration alone?
- **Underrepresented bias:** Refill preferentially restocks depleted archetypes.
  How strong can this bias be before it feels like the game is manipulating the
  pool?
- **Player-adjacent bias:** Mild enrichment of the player's primary resonance
  (not specific archetype). "The market responds to regional trends." Is this
  Level 1 reactivity in disguise?
- Can refill bias replace V9's contraction engine, or is it a complement?
- What bias level produces M12 >= 0.3 (signal reading value)?

### Agent 5: Player Information Systems

**Starting point:** Independent of round structure — what information about the
pool should the player see, and how does it affect the skill axis?

**Question:** What pool information creates the best balance of informed
decisions and strategic depth?

Explore:
- **Archetype availability display:** Show approximate counts per archetype.
  Does this make signal reading trivial or strategic?
- **Refill preview:** Show what's coming in the next refill. Does foreknowledge
  improve or reduce decision quality?
- **AI activity hints:** Show which archetypes AIs are drafting (but not
  specific cards). Does this replace or enhance pack-based signal reading?
- **Round-start snapshot:** Full pool composition at round start. Too much info?
- How does information interact with different player strategies (committed vs.
  signal-reader)?
- Can information create M12 >= 0.3 without making the draft feel like a
  spreadsheet?

### Agent 6: Hybrid and Novel Multi-Round Approaches

**Starting point:** Free exploration. Combine ideas from other agents or propose
entirely new multi-round structures.

Explore freely. Some starting ideas:
- **Draft table rotation:** Player position at the table rotates each round.
  Different seat = different AIs to your left/right = different signals.
- **Shared draft pool:** AIs and player all draft from the same displayed pool
  (like 7 Wonders), taking turns. Player sees AI picks in real time.
- **Market draft:** Pool is a visible "market" (like Ascension). Picked cards
  are immediately replaced from a deck. AIs buy from the market too.
- **Pack passing:** Actual pack-passing simulation where packs rotate between
  players. Player sees what each previous drafter passed.
- **Progressive reveal:** Pool starts hidden. Each round reveals more of the
  pool's contents, with AIs drafting from what's visible.
- **Multi-round with AI pivoting:** AIs that perform poorly in early rounds
  pivot to adjacent archetypes in later rounds (pool-aware Level 1).

---

## Round 3: Critic Review (1 agent, sequential)

A single critic reads all 6 design proposals, all research, and this plan.

**Task:**

1. Rank all proposals on: M3/M11' potential, player experience, simplicity,
   signal reading quality, "not on rails" score.
2. Evaluate whether multi-round refills genuinely solve V10's three root causes
   or just defer them.
3. Assess refill bias strategies: which are honest, which are V9 contraction in
   disguise?
4. Evaluate player information proposals: which create skill vs. which trivialize
   signal reading?
5. Propose 1-2 hybrid designs combining the best elements.
6. Recommend 4-6 algorithms for simulation.

**Output:** `docs/resonance/v11/critic_review.md` (max 2500 words)

After the critic review, each of the 6 design agents gets a brief response turn
(max 500 words each) appended as "## Post-Critique Revision".

---

## Round 4: Simulation (6 parallel agents)

Each agent implements and simulates their champion as modified by the critic.

**Fixed simulation parameters:**
- 1000 drafts × 30 picks × 3 player strategies
- Fitness: Graduated Realistic (primary), Pessimistic (secondary)
- All 12 metrics (M1-M11', M12)
- Must implement the full multi-round structure (refills, AI behavior, etc.)

**Required outputs per agent:**

1. Simulation code: `docs/resonance/v11/sim_{1..6}.py`
2. Results: `docs/resonance/v11/results_{1..6}.md` (max 1000 words)

Results must include:
- Full scorecard (all metrics at Graduated Realistic; M3/M10/M11' at Pessimistic)
- Per-archetype M3 table (8 rows)
- **Round-by-round pool composition:** Show how archetype distribution evolves
  across refill cycles
- Pack quality distribution (p10/p25/p50/p75/p90 for picks 6+)
- Consecutive bad pack analysis
- **S/A density trajectory:** Track S/A card density in the pool at each pick,
  showing whether refills successfully counteract depletion
- 2 draft traces (committed player, signal reader) — including refill moments
- AI drafter behavior summary
- Comparison to V9 baseline and V10 results
- Self-assessment

---

## Round 5: Final Synthesis (1 agent)

**Produces two files:**

### File 1: `docs/resonance/v11/final_report.md` (max 4000 words)

1. Unified comparison table
2. The key question: **Does multi-round drafting with refills produce V9-level
   metrics while preserving the AI drafter narrative?**
3. Round structure analysis: 3-pack vs small/frequent vs declining
4. Refill bias analysis: which strategies work and which are contraction in
   disguise?
5. Player information analysis: what level of pool visibility optimizes the
   skill axis?
6. Per-archetype convergence for top 3 algorithms
7. V11 vs V9 vs V10 comparison
8. Recommendation tiers:
   - **Simple:** Best multi-round design with no refill bias (pure emergent
     concentration)
   - **Standard:** Best overall multi-round design (may include mild bias)
   - **Advanced:** Best design with full player pool information
9. Complete specification for the recommended algorithm
10. Implementation guide
11. Open questions

### File 2: `docs/resonance/v11/algorithm_overview.md` (max 2500 words)

Catalog of all algorithms ordered by preference:
1. Recommended (1-2 algorithms)
2. Viable alternatives
3. Eliminated algorithms organized by failure mode
4. Structural findings about multi-round draft design

---

## Agent Summary

| Round | Agents | Type | Description |
|-------|--------|------|-------------|
| 1 | 3 | Parallel | Multi-round research |
| 2 | 6 | Parallel | Algorithm design |
| 3 | 1 + 6 responses | Sequential | Critic review + designer responses |
| 4 | 6 | Parallel | Simulation |
| 5 | 1 | Single | Final synthesis |
| **Total** | **~19** | | |

## Output Files

| File | Round | Description |
|------|-------|-------------|
| `research_multi_round.md` | 1 | Multi-round drafts in games |
| `research_pool_info.md` | 1 | Pool information mechanics |
| `research_v10_remediation.md` | 1 | V10 failure remediation analysis |
| `design_{1..6}.md` (x6) | 2 | Algorithm proposals |
| `critic_review.md` | 3 | Cross-proposal analysis |
| `sim_{1..6}.py` (x6) | 4 | Simulation code |
| `results_{1..6}.md` (x6) | 4 | Results |
| `final_report.md` | 5 | Recommendation + specification |
| `algorithm_overview.md` | 5 | Catalog of all algorithms |

All files in `docs/resonance/v11/`.

## Key Principles

1. **Multi-round refills are the core mechanism.** Every design must use
   periodic pool replenishment. Single-pool designs are V10 — they've been
   proven nonviable.
2. **AI drafters physically remove cards.** This is the narrative foundation.
   AIs take cards from the shared pool. The player sees scarcity because other
   drafters took the cards.
3. **Level 0 is the baseline; Level 1 (pool-reactive) is negotiable.** AIs
   should not read the player's specific picks. AIs MAY observe the overall
   pool state and adjust (like a real drafter opening a new pack and seeing
   what's available). This is a design variable, not a constraint.
4. **Refill bias is the new contraction lever.** Instead of V9's invisible
   contraction or V10's physical depletion, refill composition is how V11 can
   tune concentration. This must be explored honestly — "balanced refills with
   emergent concentration" is the ideal; "biased refills targeting the player's
   archetype" is V9 contraction by another name.
5. **Player pool information is a genuine skill axis.** V11 should explore what
   information about the pool state helps players make better decisions. This
   is a new dimension not available in V9/V10.
6. **M11' is relaxed but M3 is not.** Late-draft quality (picks 20+) can be
   2.5 S/A instead of 3.0. Mid-draft quality (M3, picks 6+) remains at 2.0.
   This reflects the reality that some concentration loss is acceptable for a
   better narrative.
7. **Compare to both V9 and V10.** V9 Hybrid B (M3 = 2.70) is the mathematical
   ceiling. V10 Hybrid X (M3 = 0.84) is the floor. V11 should aim to close the
   gap significantly even if it doesn't match V9 exactly.
8. **The refill reset problem is the key risk.** If each refill washes out the
   concentration built during the previous round, multi-round drafting won't
   work. Agents must address this directly.
9. **Simpler is better.** A 3-round structure the player instantly understands
   is preferable to a 6-round structure with declining refill sizes and
   weighted biases — even if the latter produces slightly better metrics.

## Recovery

Check which `docs/resonance/v11/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained.
