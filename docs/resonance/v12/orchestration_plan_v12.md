# Resonance Draft System V12 — Orchestration Plan

## The Central Idea

V12 introduces **public-information-reactive AI avoidance** — AIs that
rationally avoid the player's draft archetype by reading the same pool
information available to the player. Combined with **weighted pack
construction** as a new design variable, this creates concentration through
reduced competition rather than pool contraction.

**Player-facing explanation:** "You're drafting at a table with AI opponents.
Everyone can see what's popular — and smart drafters avoid competing for the
same cards. Find the open lane and you'll have it to yourself."

### Why This Is Different From V9-V11

V9 achieved M3 = 2.70 through invisible virtual contraction — the system
silently removed low-relevance cards from the pool. The player never saw this
happen. V10 tried physical removal by AI drafters but failed because physical
depletion exhausts the pool (M3 = 0.84). V11 tried multi-round refills but
failed because the pack-sampling bottleneck caps per-pack archetype density at
12-21% regardless of pool-level composition (best M3 = 0.89).

V12 attacks the problem from a different direction: **demand-side
concentration**. Instead of manipulating the supply of cards (contraction,
depletion, refills), V12 manipulates the demand. When AIs actively avoid the
player's archetype, the player faces zero competition for their lane's cards.
This doesn't change pool composition — it changes who takes what from the pool.

The mechanism is honest and visible: all drafters (player and AI) have access
to the same public pool state information. AIs use this information to avoid
contested archetypes, just as a skilled human drafter would. The player sees AI
avoidance behavior and can reason about it. This is not surveillance of the
player's private strategy — it is table-reading using shared information.

### The Two Design Levers

**Lever 1: AI Avoidance Behavior.** AIs observe the same pool information the
player sees (archetype counts, depletion trends) and rationally infer which
archetype the player is drafting. Once they identify the player's lane, they
avoid it — taking cards from other archetypes instead. This creates a de facto
"open lane" for the player without requiring the system to know or encode the
player's preference.

The key insight: avoidance using public information is realistic drafting
behavior. In a real draft, all players observe what others are taking and
adjust. An AI that sees "Blink cards are disappearing fast" and concludes
"someone is drafting Blink, I should avoid it" is behaving exactly as a skilled
human opponent would. The information is symmetric — the player can also observe
that "Storm cards are disappearing fast" and conclude an AI is drafting Storm.

**Lever 2: Weighted Pack Construction.** V11 proved that uniform random
sampling from a 100-130 card pool cannot achieve M3 >= 2.0 with 4-card packs.
But pack construction is a design variable. If packs are constructed with
weights favoring the player's emerging archetype (or, equivalently, if the
pool's effective sampling distribution is shaped), per-pack archetype density
can exceed the uniform baseline.

Pack construction weighting is the mechanism that converts pool-level AI
avoidance into pack-level card quality. Without it, even perfect AI avoidance
only produces a modest pool-level gradient (player's archetype accumulates in
the pool while AI archetypes deplete). With it, the pack preferentially samples
from the player's accumulated archetype, amplifying the gradient into M3-
relevant per-pack density.

### What Previous Versions Established (Carried Forward)

- **V9:** Virtual contraction achieves M3 = 2.70. Pair-affinity encoding (8
  bits/card) is minimum sufficient hidden metadata. Visible symbols do 85% of
  targeting work. Floor slot (1 guaranteed top-quartile card) prevents
  consecutive bad packs.
- **V10:** AI drafter narrative is a genuine contribution to player experience.
  Level 0 (static) AIs provide signal-reading skill. 5-AI / 3-open-lane
  structure works for game-to-game variety. Physical AI removal cannot replace
  virtual contraction.
- **V11:** Pack-sampling bottleneck is the binding constraint for uniform
  sampling. Design 5 information system (bars + trends + snapshots) is the
  strongest signal-reading architecture. Open-lane-biased refills are genuinely
  Level 0. Multi-round refills solve pool exhaustion but cannot produce M3 >=
  2.0. 3-round structure is optimal among multi-round designs.

---

## The Public Information Framework

### What Counts as "Public Information"

In a real draft, certain information is public — visible to all players at the
table:
- What cards are currently available in the pool
- Which cards were taken from the pool (by observation)
- Aggregate patterns (which archetypes are depleting)
- Who is taking what type of card (table-reading)

V12 treats all pool-state information as public. Both the player and AIs have
access to:
- Current archetype card counts in the pool
- Depletion rates (which archetypes are shrinking fastest)
- Their own draft history (what they've taken)
- General awareness of what other drafters are taking (via pool observation)

### What Remains Private

- The specific fitness scores of cards (AIs know these; the player evaluates
  card quality by reading card text)
- The player's internal strategy and commitment level
- The AI's internal archetype assignment (prior to observable behavior)
- Individual card identities in the pool (the player sees their pack, not the
  full pool)

### The Avoidance Mechanism

AIs observe the pool state after each pick cycle. When the player's picks
establish a visible pattern (e.g., consistently taking Ember-symbol cards), AIs
infer the player's likely archetype. AIs then actively avoid that archetype in
their own picks — not because they're told to, but because rational drafters
avoid competition.

The inference uses only public information:
- After pick N, look at the player's drafted cards' visible resonance symbols
- Compute the player's apparent resonance signature (same method the player
  would use to read an AI's preferences)
- Identify the 1-2 archetypes most consistent with the player's picks
- Reduce priority for those archetypes in AI pick selection

This is Level 1.5 reactivity: AIs react to publicly observable player behavior
(not private player state), using the same information framework available to
all participants. It is more reactive than Level 0 (static) but less than
Level 2+ (private surveillance). The key distinction: a human opponent watching
the draft would make the same inference.

---

## Fixed Assumptions (Not Variables in V12)

### Fitness Model: Graduated Realistic (Fixed)

| Pair | Sibling A-Tier Rate |
|------|:---:|
| Warriors / Sacrifice (Tide) | 50% |
| Self-Discard / Self-Mill (Stone) | 40% |
| Blink / Storm (Ember) | 30% |
| Flash / Ramp (Zephyr) | 25% |
| **Weighted Average** | **~36%** |

### Total Draft: 30 Picks (Fixed)

### Pack Size: 4 Cards (Show 4, Pick 1) (Fixed)

The player sees 4 cards and picks 1 each turn. How those 4 cards are selected
from the pool is a design variable (see Variable 2 below).

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

### Player Strategies (Fixed)

- **Archetype-committed:** Picks highest fitness for strongest archetype.
  Commits around pick 5-6.
- **Power-chaser:** Picks highest raw power regardless of archetype.
- **Signal-reader:** Evaluates which archetype seems most available and drafts
  toward it.

---

## The V12 Design Space

Four interconnected design variables:

### Variable 1: AI Avoidance Model

How do AIs infer the player's archetype and how aggressively do they avoid it?

| Model | Inference Method | Avoidance Strength | Reactivity |
|-------|-----------------|-------------------|:---:|
| A: No Avoidance (Baseline) | None | None | Level 0 |
| B: Delayed Avoidance | Infer from picks 5+, avoid from pick 8+ | Moderate (50% weight reduction) | Level 1 |
| C: Gradual Avoidance | Infer continuously, ramp avoidance | Graduated (20% pick 3 → 80% pick 15) | Level 1.5 |
| D: Immediate Avoidance | Infer from pick 2+, full avoidance | Strong (90% weight reduction) | Level 1.5 |
| E: Symmetric Avoidance | All AIs avoid each other AND the player | Moderate (50% mutual) | Level 1 |

Key design questions:
- How many picks does the AI need to confidently infer the player's archetype?
  (The player commits around pick 5-6; can the AI detect this by pick 4-5 using
  public symbol patterns?)
- Should avoidance be binary (avoid/don't avoid) or graduated (reduce
  priority)?
- Should AIs avoid only the player's specific archetype, or also the player's
  resonance symbol pair?
- What happens when the player pivots? How quickly do AIs re-read the table?

### Variable 2: Pack Construction Method

How are the 4 cards in each pack selected from the pool?

| Method | Description | Archetype Density | Level |
|--------|-------------|:-----------------:|:---:|
| A: Uniform Random | 4 cards drawn uniformly from pool | ~6% (4/120 × 15) | 0 |
| B: Symbol-Weighted | Weight toward player's visible resonance signature | 15-30% | 1 |
| C: Affinity-Weighted | Weight toward player's inferred archetype pair-affinity | 25-45% | 2 |
| D: Floor + Random | 1 guaranteed on-archetype card + 3 random | 25%+ base | 2 |
| E: Hybrid Floor + Weighted | 1 floor slot + 3 symbol-weighted random | 30-50% | 2 |

This is the critical variable. V11 proved that uniform random (Method A) from a
100-130 card pool cannot achieve M3 >= 2.0 with 4-card packs. Methods B-E
introduce weighting that increases per-pack archetype density.

**The narrative question:** Can pack construction weighting be explained
honestly? V9's virtual contraction was invisible. Weighted pack construction
could be framed as: "The cards you see are influenced by your interests — the
market shows you what's relevant." Or it could be hidden entirely (the player
just sees 4 cards and doesn't know how they were selected). Neither framing
requires revealing the mechanic.

**Math baseline:** With 4-card packs, M3 = 2.0 requires that on average 2 of 4
cards are S/A for the player's committed archetype. With 36% weighted-average
sibling A-tier rate, this requires ~5.6 on-archetype cards per 4-card pack
(before fitness filtering), or ~56% of each pack being the player's archetype.
This is a high bar. Even Method D (floor + random) only guarantees 25% base
density before the random slots contribute.

### Variable 3: Pool Structure

How is the card pool organized and does it change during the draft?

| Structure | Description | Pool Size |
|-----------|-------------|:---------:|
| A: Static Pool | Fixed pool, no refills | 120-360 |
| B: Multi-Round Refills | 3 rounds with refill between (V11 style) | 120 + refills |
| C: Continuous Market | Cards replaced as drafted (Design 6 style) | 120 + reserve |
| D: V9-Style Virtual | 360 cards, virtual contraction | 360→17 |

V11 conclusively showed that multi-round refills (B) and continuous markets (C)
cannot achieve M3 >= 2.0 alone. However, when combined with AI avoidance
(Variable 1) and weighted pack construction (Variable 2), the pool structure
becomes a secondary variable — the primary concentration mechanism is pack
construction, not pool manipulation.

The interesting question is whether AI avoidance + weighted packs can work with
a simple static pool (A), eliminating the need for V9-style virtual contraction
entirely.

### Variable 4: AI Count and Lane Structure

How many AIs, and how does the avoidance behavior interact with the open-lane
structure?

| Config | AIs | Open Lanes | Avoidance Effect |
|--------|:---:|:----------:|-----------------|
| A: Standard | 5 | 3 | AIs avoid player's 1 lane, leaving 4 AI lanes + 2 other open |
| B: Dense | 7 | 1 | AIs avoid player's lane, creating a single clear open lane |
| C: Adaptive | 5 start, some pivot | Variable | AIs that avoid player's lane may pivot to open lanes |
| D: Symmetric | 5, all avoid each other | 3 open | Mutual avoidance creates natural lane separation |

Config B (7 AIs, 1 open lane) is particularly interesting with avoidance: if
all 7 AIs avoid the player's archetype, the player has zero competition for
their lane while 7 AIs compete across the other 7 archetypes. This is the
strongest possible demand-side concentration.

---

## Metrics

### Recalibrated M3 for 4-Card Packs

With 4-card packs (show 4, pick 1), the expected S/A cards per pack for a
committed archetype depends on pack construction method:

| Pack Method | Expected On-Archetype per Pack | Expected S/A per Pack | M3 Target Feasibility |
|-------------|:---:|:---:|:---:|
| Uniform (120-card pool) | 0.50 | 0.18 | Impossible |
| Symbol-weighted (2x) | 1.00 | 0.36 | Impossible |
| Symbol-weighted (4x) | 2.00 | 0.72 | Insufficient |
| Affinity-weighted (8x) | 3.00 | 1.08 | Marginal |
| Floor(1) + 3 random | 1.38 | 0.61 | Insufficient |
| Floor(1) + 3 weighted(4x) | 2.50 | 0.97 | Marginal |

**Important recalibration note:** M3 = 2.0 with 4-card packs is extremely
aggressive. V9 achieved M3 = 2.70 with 4-card packs (3 random + 1 floor), but
only because virtual contraction reduced the effective pool to ~17 cards by pick
30, achieving 60%+ archetype density in the surviving pool.

V12 should explore whether M3 targets need adjustment for the AI-avoidance
paradigm. The relevant question is: does the player's draft *feel* good? A
lower M3 might be acceptable if the player consistently faces meaningful
archetype choices (2-3 S/A cards from their archetype available somewhere in
every few packs, rather than 2+ in every single pack).

### Full Metric Table

| ID | Metric | Target | Notes |
|----|--------|--------|-------|
| M1 | Picks 1-5: unique archetypes with S/A cards per pack | >= 3 of 8 | |
| M2 | Picks 1-5: S/A cards for emerging archetype per pack | <= 2 | |
| M3 | Picks 6+: S/A cards for committed archetype per pack | >= 2.0 avg | May need recalibration |
| M4 | Picks 6+: off-archetype (C/F) cards per pack | >= 0.5 | |
| M5 | Convergence pick | Pick 5-8 | |
| M6 | Deck archetype concentration | 60-90% S/A-tier cards | |
| M7 | Run-to-run variety | < 40% card overlap | |
| M8 | Archetype frequency across runs | No archetype > 20% or < 5% | |
| M9 | StdDev of S/A cards per pack (picks 6+) | >= 0.8 | |
| M10 | Max consecutive packs below 1.5 S/A (picks 6+) | <= 2 | |
| M11' | Picks 20+: S/A cards for committed archetype per pack | >= 2.5 | |
| M12 | Signal-reader M3 minus Committed M3 | >= 0.3 | |
| M13 | AI avoidance detection pick | Pick at which AIs detectably change behavior | New: target 6-10 |
| M14 | Player archetype visibility pick | Pick at which AI correctly infers player's archetype | New: target 4-7 |

### V9 Baselines

Hybrid B: M3 = 2.70, M11(old) = 3.25, M10 = 3.8, M5 = 9.6, M6 = 86%

---

## Simulation Card Model

```python
class SimCard:
    id: int
    visible_symbols: list[Resonance]   # what the player sees (0-2 symbols)
    archetype: str                     # primary archetype (for evaluation)
    archetype_fitness: dict            # archetype_id -> fitness score (0.0-1.0)
    power: float                       # raw card strength (0-10)
    pair_affinity: dict                # archetype_pair -> affinity score (hidden)

class AIDrafter:
    archetype_preference: str          # primary lane this AI drafts
    cards_drafted: list[SimCard]       # tracking for saturation
    saturation_threshold: int          # when to ease off primary
    inferred_player_archetype: str     # what AI thinks player is drafting
    avoidance_strength: float          # 0.0 = no avoidance, 1.0 = full avoidance
    avoidance_start_pick: int          # when avoidance kicks in

class PackConstructor:
    method: str                        # "uniform", "symbol_weighted", "affinity_weighted", "floor_random", "hybrid"
    weight_multiplier: float           # how much to weight toward player's archetype
    floor_count: int                   # guaranteed on-archetype slots (0-1)
    player_signature: list[float]      # current resonance signature for weighting

class DraftState:
    pool: list[SimCard]                # current available pool
    pick_number: int
    player_picks: list[SimCard]        # player's drafted cards
    ai_picks: dict[int, list[SimCard]] # each AI's drafted cards
    public_pool_info: dict             # archetype counts, trends (visible to all)
```

---

## Round 1: Research (3 parallel agents)

Pure research — no algorithm design. Map the AI avoidance + pack construction
design space.

### Research Agent A: AI Avoidance in Competitive Drafting

**Question:** How do human drafters read opponents and adjust strategy in
competitive draft formats, and how can AI drafters replicate this behavior?

Explore:
- In MTG drafts, how do skilled players read signals from passed packs? What
  information do they use to detect what neighbors are drafting?
- In shared-pool drafts (7 Wonders, Sushi Go), how do players observe and
  respond to opponents' picks? Is active avoidance of contested archetypes a
  documented strategy?
- What is the minimum number of picks required to reliably identify an
  opponent's draft direction? How confident can inference be at pick 3 vs
  pick 5 vs pick 8?
- How does avoidance timing affect draft dynamics? Early avoidance (pick 3-5)
  vs late avoidance (pick 8-10) — which produces better outcomes for all
  drafters?
- What is the boundary between "reading the table" (acceptable) and
  "surveillance" (unacceptable) in terms of player perception?

**Output:** `docs/resonance/v12/research_ai_avoidance.md` (max 2000 words)

### Research Agent B: Pack Construction Methods

**Question:** How do existing card games construct the cards presented to
players, and what methods can convert pool-level composition into pack-level
archetype density?

Explore:
- How does MTG construct booster packs? (Collation, print runs, rarity
  distribution.) How do digital CCGs (Hearthstone arena, Legends of Runeterra
  expedition) construct draft picks?
- What is "weighted sampling" in the context of card games? How do roguelike
  deckbuilders (Slay the Spire, Monster Train) weight card offerings toward
  player synergies?
- Given a pool of 120 cards with 15 per archetype and 4-card packs: what
  weighting schemes achieve 25%, 40%, 50% per-pack archetype density? What are
  the mathematical tradeoffs?
- How does pack construction interact with player perception of fairness? When
  do players notice that offerings are "suspiciously good" vs "naturally lucky"?
- Can pack construction weighting be framed as a natural feature of the draft
  format rather than a hidden manipulation?

**Output:** `docs/resonance/v12/research_pack_construction.md` (max 2000 words)

### Research Agent C: Concentration Math for AI Avoidance + Weighted Packs

**Question:** What combinations of AI avoidance strength and pack construction
weighting produce M3 >= 2.0 with 4-card packs from a pool?

Analyze:
- **Baseline:** With 120 cards, 8 archetypes, 15 per archetype, 36% sibling
  A-tier, uniform 4-card packs: what is M3? (Expected: ~0.18)
- **AI avoidance only (uniform packs):** If 5 AIs avoid the player's archetype,
  the player's archetype accumulates in the pool. After 30 picks, how many
  cards remain per archetype? What is the resulting M3 with uniform sampling?
- **Pack weighting only (no avoidance):** What weight multiplier on the player's
  primary resonance symbol achieves M3 >= 2.0? What multiplier on the player's
  specific archetype (using pair-affinity)?
- **Combined:** AI avoidance + pack weighting. Model the interaction: avoidance
  enriches the pool, weighting amplifies the enrichment into packs. What
  combination achieves M3 >= 2.0?
- **4-card pack constraint:** V9 used 3 random + 1 floor = 4 cards. Can V12
  use the same floor mechanism? What floor definition works with AI avoidance?
- **Comparison to V9:** V9 contracted the pool from 360 to 17 cards.
  V12 maintains a 120-card pool but weights pack construction. At what
  weighting does V12 match V9's per-pack archetype density?

**Reads:** This plan, V11 final report, V11 algorithm overview, V9 algorithm
overview.

**Output:** `docs/resonance/v12/research_concentration_math.md` (max 2000 words)

---

## Round 2: Algorithm Design (6 parallel agents)

Each agent reads all Round 1 research plus this plan, V11 final report, and V11
algorithm overview. Each explores a different region of the V12 design space.

**Fixed for all agents:**
- Fitness: Graduated Realistic
- Total draft: 30 picks
- Pack size: 4 cards (show 4, pick 1)
- Visible symbols: ~10% dual-res
- AI drafter framing required
- All V10 and V11 structural findings available
- AIs must use public-information-based avoidance of the player's archetype
  somewhere in the design (the strength, timing, and mechanism vary by agent)
- Pack construction method is a design variable (not fixed at uniform random)

**Output format (all agents):**

1. Key findings (5-7 bullets)
2. Three algorithm proposals: name, one-sentence description, technical spec,
   predicted M3/M10/M11'/M6/M12/M13/M14
3. Champion selection with justification
4. Champion deep-dive: pick-by-pick walkthrough showing when AI avoidance kicks
   in, how pack construction changes, what the player sees, pool composition
   evolution, failure modes
5. Complete specification (pool size, pack method, AI count, AI avoidance
   model, AI inference mechanism, AI pick logic, player information)

Max 1500 words per agent.

### Agent 1: Minimal Avoidance + Uniform Packs (Isolation Test)

**Starting point:** Test AI avoidance alone, with NO pack weighting. Uniform
random 4-card packs. This isolates the contribution of avoidance behavior.

**Question:** How much M3 improvement does AI avoidance alone produce over a
Level 0 baseline, when packs are constructed uniformly?

Explore:
- With 5 Level 0 AIs (no avoidance), M3 should be ~0.18-0.25 (V11 SIM-1
  baseline). What does M3 become when AIs avoid the player's detected
  archetype?
- How much pool-level archetype accumulation does avoidance create? If 5 AIs
  stop taking Blink cards after pick 6, how does Blink's count in the pool
  grow?
- Is the avoidance effect large enough to be meaningful without pack weighting?
- What is the sensitivity to avoidance timing (pick 5 vs pick 8 vs pick 12)?
- How does the player's archetype inference accuracy affect the mechanism?

### Agent 2: Symbol-Weighted Packs + Gradual Avoidance

**Starting point:** Combine graduated AI avoidance with symbol-weighted pack
construction. Packs favor the player's visible resonance signature.

**Question:** Can symbol-level pack weighting (using visible resonance symbols
only, no hidden pair-affinity) achieve M3 >= 2.0 when combined with gradual AI
avoidance?

Explore:
- What symbol weight multiplier produces packs with 2+ on-archetype cards?
- Symbol weighting helps both archetypes sharing a resonance (e.g., Blink and
  Storm both have Ember). How does this affect M3 vs archetype-specific
  weighting?
- Is symbol weighting honest? The player sees their own symbol preferences;
  weighting toward those symbols is consistent with "the market shows you
  relevant cards."
- How does symbol weighting interact with dual-symbol cards?

### Agent 3: Affinity-Weighted Packs + Delayed Avoidance

**Starting point:** Use pair-affinity scores (hidden 8-bit metadata from V9) to
weight pack construction toward the player's inferred archetype. AIs begin
avoidance after pick 8 (delayed).

**Question:** Can affinity-based pack weighting — the most targeted mechanism —
achieve M3 >= 2.0 with moderate AI avoidance?

Explore:
- Affinity weighting targets the specific archetype, not just the resonance
  symbol. What weight multiplier is needed?
- Is affinity weighting V9 contraction by another name? V9 used pair-affinity
  to remove low-relevance cards; affinity-weighted packs use pair-affinity to
  boost high-relevance cards in packs. The mechanism direction is opposite
  (inclusion vs exclusion) but the effect is similar. Evaluate honestly.
- Delayed avoidance (pick 8+) means the first 7 picks have zero avoidance
  benefit. Is this too late? Or does it prevent premature AI commitment?
- What is the floor needed? V9's floor slot guaranteed 1 top-quartile card.
  Can this be adapted?

### Agent 4: V9 Engine + AI Avoidance Narrative

**Starting point:** V9 Hybrid B's contraction engine runs unchanged. AI
avoidance is layered on top as a narrative enhancement — AIs appear to avoid
the player's archetype because the contraction engine removes non-relevant
cards and attributes removals to AIs.

**Question:** Does adding visible AI avoidance behavior to V9's engine improve
player experience (M12, M5) without degrading M3?

Explore:
- V9's engine already produces M3 = 2.70. AI avoidance is purely
  presentational — the engine doesn't change, but the AI narrative is enriched
  by visible avoidance behavior.
- Does this make the AI narrative more convincing? ("AIs noticed I'm drafting
  Storm and backed off" vs "AIs just happen to be drafting other things")
- Can AI avoidance behavior be derived from V9's contraction pattern? (Cards
  removed by contraction are attributed to AI picks; AIs whose archetypes
  overlap with the player's will naturally appear to "avoid" as their cards
  are not being removed by contraction.)
- Is this design dishonest? The avoidance is presentational, not mechanical.
  The AI isn't really avoiding — the engine is just removing irrelevant cards.
  Evaluate the narrative integrity.

### Agent 5: High-AI-Count + Avoidance (7 AIs, 1 Open Lane)

**Starting point:** 7 AIs, only 1 open lane per game. All 7 AIs avoid the
player's archetype once detected. Combined with moderate pack weighting.

**Question:** Does maximizing AI count and minimizing open lanes, combined with
universal avoidance, create the strongest demand-side concentration?

Explore:
- With 7 AIs avoiding the player's archetype, the player faces zero
  competition. But with only 1 open lane, there's no "choosing the right lane"
  skill — the open lane is whatever the player picks.
- Does 7-AI avoidance create enough pool-level concentration that moderate pack
  weighting (2-3x) suffices for M3 >= 2.0?
- What happens to game-to-game variety? C(8,7) = 8 compositions vs
  C(8,5) = 56. Is 8 enough variety?
- The 1-open-lane structure eliminates M12 (signal reading) as a skill axis.
  Is this acceptable? What does it replace it with?

### Agent 6: Hybrid Approaches + Novel Mechanisms

**Starting point:** Free exploration. Combine AI avoidance with pack
construction in novel ways, or propose entirely new mechanisms.

Explore freely. Some starting ideas:
- **Progressive revelation packs:** Pack construction starts uniform, becomes
  increasingly weighted as the draft progresses. Early packs are exploratory;
  late packs are focused. Mirrors V9's contraction trajectory.
- **AI avoidance with multi-round refills:** V11's 3-round structure with
  open-lane biased refills, plus AI avoidance within each round. Does the
  combination of refill bias + avoidance + pack weighting finally cross M3 >=
  2.0?
- **Symmetric information with asymmetric packs:** All drafters see the same
  pool state (Design 5 information), but the player's packs are weighted while
  AI "packs" (their selections from the pool) are not. Is this fair?
- **Avoidance cascade:** When the player commits to archetype X, AIs avoid X.
  This pushes AIs toward the remaining 7 archetypes, creating secondary
  avoidance effects. Some AIs may compete with each other more intensely,
  potentially creating interesting dynamics.
- **Pack construction without hidden metadata:** Can pack weighting work using
  only visible resonance symbols (no pair-affinity encoding), making the entire
  mechanism transparent?

---

## Round 3: Critic Review (1 agent, sequential)

A single critic reads all 6 design proposals, all research, and this plan.

**Task:**

1. Rank all proposals on: M3/M11' potential, player experience, simplicity,
   signal reading quality, AI avoidance narrative quality, pack construction
   honesty.
2. Evaluate whether AI avoidance is genuinely "public information" behavior or
   a dressed-up Level 2+ mechanism. Where is the line?
3. Assess pack construction methods: which are honest, which are V9 contraction
   in disguise, which are somewhere in between?
4. Evaluate the M3 target: is M3 >= 2.0 with 4-card packs a reasonable target
   for the AI-avoidance paradigm, or should it be recalibrated?
5. Evaluate the interaction between AI avoidance and pack weighting: are they
   complementary, redundant, or in tension?
6. Propose 1-2 hybrid designs combining the best elements.
7. Recommend 4-6 algorithms for simulation.

**Output:** `docs/resonance/v12/critic_review.md` (max 2500 words)

After the critic review, each of the 6 design agents gets a brief response turn
(max 500 words each) appended as "## Post-Critique Revision".

---

## Round 4: Simulation (6 parallel agents)

Each agent implements and simulates their champion as modified by the critic.

**Fixed simulation parameters:**
- 1000 drafts x 30 picks x 3 player strategies
- Fitness: Graduated Realistic (primary), Pessimistic (secondary)
- All 14 metrics (M1-M11', M12, M13, M14)
- Must implement AI avoidance logic (inference + behavior change)
- Must implement pack construction method as specified
- Must track AI inference accuracy (how often does AI correctly identify the
  player's archetype, and at what pick?)

**Required outputs per agent:**

1. Simulation code: `docs/resonance/v12/sim_{1..6}.py`
2. Results: `docs/resonance/v12/results_{1..6}.md` (max 1000 words)

Results must include:
- Full scorecard (all metrics at Graduated Realistic; M3/M10/M11' at
  Pessimistic)
- Per-archetype M3 table (8 rows)
- **AI avoidance timeline:** At what pick does each AI begin avoiding the
  player's archetype? How accurate is the inference?
- **Pack construction analysis:** What is the actual per-pack archetype density
  achieved? How does it compare to the uniform baseline?
- Pack quality distribution (p10/p25/p50/p75/p90 for picks 6+)
- Consecutive bad pack analysis
- **Pool composition trajectory:** Show how archetype distribution evolves
  as AIs avoid the player's lane
- 2 draft traces (committed player, signal reader) — including AI avoidance
  moments
- Comparison to V9 baseline and V11 results
- Self-assessment: Is AI avoidance + pack weighting a viable replacement for
  V9 contraction, or is it a complement?

---

## Round 5: Final Synthesis (1 agent)

**Produces two files:**

### File 1: `docs/resonance/v12/final_report.md` (max 4000 words)

1. Unified comparison table (all V12 algorithms + V9/V10/V11 baselines)
2. The key question: **Can AI avoidance + weighted pack construction replace
   V9's virtual contraction?**
3. AI avoidance analysis: which models work and which are surveillance?
4. Pack construction analysis: which methods achieve M3 targets honestly?
5. The interaction: how much does avoidance contribute vs pack weighting?
6. Per-archetype convergence for top 3 algorithms
7. V12 vs V9 vs V10 vs V11 comparison
8. Recommendation tiers:
   - **Pure AI Avoidance:** Best design using only AI avoidance behavior (no
     pack weighting). Establishes the demand-side contribution.
   - **Standard:** Best overall design combining AI avoidance + pack
     construction. May include moderate weighting.
   - **V9 Enhanced:** V9's engine with AI avoidance as a narrative layer. The
     fallback if V12's mechanisms don't achieve M3 >= 2.0 independently.
9. Complete specification for the recommended algorithm
10. Implementation guide
11. Open questions and V13 directions

### File 2: `docs/resonance/v12/algorithm_overview.md` (max 2500 words)

Catalog of all algorithms ordered by preference:
1. Recommended (1-2 algorithms)
2. Viable alternatives
3. Eliminated algorithms organized by failure mode
4. Structural findings about AI avoidance and pack construction

---

## Agent Summary

| Round | Agents | Type | Description |
|-------|--------|------|-------------|
| 1 | 3 | Parallel | Research: avoidance, pack construction, math |
| 2 | 6 | Parallel | Algorithm design |
| 3 | 1 + 6 responses | Sequential | Critic review + designer responses |
| 4 | 6 | Parallel | Simulation |
| 5 | 1 | Single | Final synthesis |
| **Total** | **~19** | | |

## Output Files

| File | Round | Description |
|------|-------|-------------|
| `research_ai_avoidance.md` | 1 | AI avoidance in competitive drafting |
| `research_pack_construction.md` | 1 | Pack construction methods |
| `research_concentration_math.md` | 1 | Math for avoidance + packs |
| `design_{1..6}.md` (x6) | 2 | Algorithm proposals |
| `critic_review.md` | 3 | Cross-proposal analysis |
| `sim_{1..6}.py` (x6) | 4 | Simulation code |
| `results_{1..6}.md` (x6) | 4 | Results |
| `final_report.md` | 5 | Recommendation + specification |
| `algorithm_overview.md` | 5 | Catalog of all algorithms |

All files in `docs/resonance/v12/`.

## Key Principles

1. **AI avoidance is the core demand-side mechanism.** Every design must include
   AIs that detect and avoid the player's draft archetype using publicly
   available information. The strength, timing, and inference method vary, but
   the avoidance behavior is the central thesis.
2. **Pack construction is the supply-side amplifier.** V11 proved that uniform
   4-card packs from a 120-card pool cannot achieve M3 >= 2.0. Pack
   construction weighting is how V12 bridges the pack-sampling bottleneck.
   Whether this uses visible symbols only or hidden pair-affinity is a design
   variable.
3. **Public information is the honesty criterion.** AI avoidance must use only
   information available to all players. The player's visible resonance
   signature (computed from their drafted cards' symbols) is public. The
   player's internal strategy, commitment level, and card-by-card evaluation
   are private. AIs must not use private information.
4. **V9 is the fallback, not the enemy.** If no V12 mechanism achieves M3 >=
   2.0 independently, the recommendation should be V9's engine enhanced with
   AI avoidance narrative and Design 5 information. V12's contribution would
   then be the improved AI narrative (avoidance behavior makes AIs feel smarter
   and more realistic).
5. **4-card packs are fixed.** The game uses "show 4, pick 1." Pack size is not
   a variable. Pack construction method is.
6. **AI avoidance must feel natural.** The player should perceive AI avoidance
   as rational opponent behavior, not as the game manipulating outcomes. "They
   noticed I'm drafting Storm and backed off" should feel like a competitive
   dynamic, not a designed safety net.
7. **M3 target may need recalibration.** M3 = 2.0 with 4-card packs requires
   50% archetype density per pack. This is extremely aggressive. Agents should
   evaluate whether a lower M3 target (e.g., 1.5) produces acceptable player
   experience, and what the minimum M3 is for the draft to "feel good."
8. **Separate avoidance from weighting.** Agent 1 explicitly tests avoidance
   alone (uniform packs) to isolate its contribution. This is essential
   calibration. If avoidance alone produces M3 = 0.5, and weighting alone
   produces M3 = 1.5, but the combination produces M3 = 2.5, the interaction
   effect is clear.
9. **Transparency over stealth.** The AI avoidance mechanism should be visible
   to the player through Design 5's information system. When AIs start avoiding
   the player's archetype, the depletion trend arrows should show it. The
   player should be able to observe and exploit this behavior.

## Recovery

Check which `docs/resonance/v12/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained.
