# Resonance Draft System V12 — Orchestration Plan

## The Central Idea

V12 introduces **public-information-reactive AI avoidance** — AIs that
rationally avoid the player's draft archetype by reading the same pool
information available to the player. Combined with **oversampled pack
construction** (draw N cards from the pool, show the best 4), this creates
concentration through reduced competition and curated presentation rather than
pool contraction.

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

**Lever 2: Oversampled Pack Construction (Draw N, Show Best 4).** V11 proved
that drawing 4 cards uniformly from a 100-130 card pool cannot achieve M3 >=
2.0. But the game doesn't have to show all N cards it draws — it can draw more
and present the best subset.

The mechanism: the system draws N cards from the pool (where N > 4), ranks them
by fitness for the player's emerging archetype, and shows the top 4. The player
sees 4 cards as usual but doesn't know (or need to know) that more were drawn.
This is transparent, tunable, and has a single parameter: N.

The narrative framing is honest and natural: "the market has many cards; you see
the ones most relevant to your interests." Or it can be invisible — the player
just sees 4 cards and the selection feels like the draft naturally offering
good options. Unlike V9's invisible contraction (which removed cards from the
pool), oversampling doesn't change the pool at all — it just curates what the
player sees from what's available.

**The math:** With ~5 S/A cards for the player's archetype in a 120-card pool,
M3 ≈ N × 5/120 = N/24. For M3 = 2.0, N ≈ 48. For M3 = 1.5, N ≈ 36. This
holds when S/A supply is maintained in the pool — which is exactly what AI
avoidance provides (AIs don't take from the player's archetype, so S/A count
stays stable throughout the draft).

Oversampled pack construction is the mechanism that converts pool-level AI
avoidance into pack-level card quality. Without it, even perfect AI avoidance
only produces a modest pool-level gradient. With it, the larger draw naturally
includes more on-archetype cards, and the "show best 4" filter ensures the
player sees them.

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

### Variable 2: Oversample Size (N)

How many cards does the system draw from the pool before showing the best 4?

| Config | N Drawn | Show | Expected S/A (5 S/A in 120 pool) | M3 Estimate |
|--------|:-------:|:----:|:---:|:---:|
| A: No Oversample | 4 | 4 | 0.17 | 0.17 |
| B: Light Oversample | 16 | Best 4 | 0.67 | 0.67 |
| C: Moderate Oversample | 32 | Best 4 | 1.33 | 1.33 |
| D: Standard Oversample | 48 | Best 4 | 2.00 | 2.00 |
| E: Heavy Oversample | 64 | Best 4 | 2.67 | 2.67 |

This is the critical variable. N controls how much the system curates the
player's options. At N = 4, the player sees a uniform random sample (V11
baseline — proven insufficient). At N = 48, the system draws 40% of the pool
and shows the 4 cards with highest fitness for the player's archetype, yielding
M3 ≈ 2.0.

**"Best 4" ranking:** Cards drawn are ranked by fitness for the player's
inferred archetype. S/A cards for the player's archetype rank highest (~0.9
fitness), followed by sibling-archetype S/A (~0.5-0.7), on-archetype C/F
(~0.3-0.5), and off-archetype cards (~0.0-0.2). The top 4 by this ranking
are shown. This means any S/A card for the player's archetype that appears in
the N drawn will be in the shown 4 (as long as fewer than 4 are drawn, which
is almost always true).

**The late-draft problem:** The M3 estimates above assume 5 S/A cards remain
in the pool. As the player drafts S/A cards, fewer remain. After taking 3 S/A
(~pick 15), only 2 remain, and M3 ≈ N × 2/115. For N = 48, late-draft M3 ≈
0.83. This is where AI avoidance is critical: if AIs avoid the player's
archetype, S/A cards stay in the pool because no AI is taking them. The player
is the only one depleting their own S/A supply. With AI avoidance, the S/A
count stays at ~5 until the player takes them, keeping N ≈ 48 sufficient
throughout the draft.

**The narrative framing:** Oversampling is naturally honest. Possible framings:
- Invisible: player just sees 4 cards, doesn't know N exists
- "Your advisor scouts the market and presents the best options"
- "The market has many cards; these are the ones that caught your eye"
- Explicit: "Draw 48, show best 4" as a stated game rule

All framings are defensible because the pool is real, the draw is real, and the
filter criterion (fitness for the player's demonstrated preferences) is
derivable from public information (the player's own picks).

**Interaction with AI avoidance:** Oversampling and avoidance are complementary,
not redundant. Avoidance maintains S/A supply in the pool (demand side);
oversampling ensures S/A cards reach the player's packs (supply side). Without
avoidance, late-draft S/A depletion forces N to grow impractically large
(N > 100). Without oversampling, even perfect avoidance only produces M3 ≈
0.25 with uniform 4-card packs (V11's pack-sampling bottleneck).

**Can oversampling reduce the need for Design 5 information?** Partially. For
committed drafting (picks 6+), oversampled packs self-signal — the player sees
good cards for their archetype and infers "my lane is open." But for
exploration (picks 1-5), the player hasn't committed yet and the "best 4"
can't be targeted to any archetype. Design 5's bars and trends remain valuable
for the initial lane-selection decision. Agents should explore whether
oversampled packs make some Design 5 elements redundant.

### Variable 3: Pool Structure

How is the card pool organized and does it change during the draft?

| Structure | Description | Pool Size |
|-----------|-------------|:---------:|
| A: Static Pool | Fixed pool, no refills | 120-360 |
| B: Multi-Round Refills | 3 rounds with refill between (V11 style) | 120 + refills |
| C: Continuous Market | Cards replaced as drafted (Design 6 style) | 120 + reserve |
| D: V9-Style Virtual | 360 cards, virtual contraction | 360→17 |

V11 conclusively showed that multi-round refills (B) and continuous markets (C)
cannot achieve M3 >= 2.0 with uniform pack sampling. However, when combined
with AI avoidance (Variable 1) and oversampled pack construction (Variable 2),
the pool structure becomes a secondary variable — the primary concentration
mechanism is oversampling, not pool manipulation.

The interesting question is whether AI avoidance + oversampling can work with
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

### Recalibrated M3 for Oversampled 4-Card Packs

With oversampled pack construction (draw N, show best 4), M3 depends on N and
the number of S/A cards for the player's archetype remaining in the pool:

| N Drawn | S/A in Pool = 5 | S/A in Pool = 4 | S/A in Pool = 3 | S/A in Pool = 2 |
|:-------:|:---:|:---:|:---:|:---:|
| 4 (uniform) | 0.17 | 0.13 | 0.10 | 0.07 |
| 16 | 0.67 | 0.53 | 0.40 | 0.27 |
| 24 | 1.00 | 0.80 | 0.60 | 0.40 |
| 36 | 1.50 | 1.20 | 0.90 | 0.60 |
| 48 | 2.00 | 1.60 | 1.20 | 0.80 |
| 64 | 2.67 | 2.13 | 1.60 | 1.07 |

**The critical interaction:** Without AI avoidance, S/A depletes as AIs take
on-archetype cards. By mid-draft, S/A in pool may drop to 2-3, requiring
N = 80-100 for M3 = 2.0. With AI avoidance, S/A stays at ~5 throughout (only
the player depletes their own lane), keeping N ≈ 48 sufficient.

**Important recalibration note:** M3 = 2.0 with 4-card packs requires N ≈ 48
(drawing 40% of the pool each pick). V9 achieved M3 = 2.70 with 4-card packs
(3 random + 1 floor) by contracting the pool to ~17 cards — effectively drawing
from a tiny pool where archetype density was 60%+.

V12 should explore whether M3 targets need adjustment for the oversampling
paradigm. The relevant question is: does the player's draft *feel* good? A
lower M3 (e.g., 1.5, achievable at N ≈ 36) might be acceptable if the player
consistently faces meaningful archetype choices. Agents should test multiple N
values and evaluate the qualitative experience, not just the metric.

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
    oversample_n: int                  # N cards drawn from pool (4 = uniform, 48 = standard)
    show_count: int                    # always 4 (show best 4 of N drawn)
    ranking_method: str                # "archetype_fitness", "symbol_match", "power"
    player_signature: list[float]      # current resonance signature for ranking
    inferred_archetype: str            # player's inferred archetype for fitness ranking

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

### Research Agent B: Oversampled Pack Construction

**Question:** How do existing card games use oversampling or curated presentation
to improve card offerings, and what are the design tradeoffs of "draw N, show
best K"?

Explore:
- How do roguelike deckbuilders (Slay the Spire, Monster Train, Inscryption)
  curate card offerings toward player synergies? Do any use explicit
  oversampling (draw more than shown, filter)?
- How do digital CCGs (Hearthstone arena, Legends of Runeterra expedition)
  construct draft picks? Is there evidence of hidden oversampling or filtering?
- What does "best" mean in "show best 4"? Options: highest archetype fitness,
  highest power, highest resonance symbol match, composite score. How does the
  ranking criterion affect the skill axis?
- How does oversampling interact with player perception? At what N does the
  player notice that offerings are "suspiciously good"? Is there a perceptual
  threshold where packs feel curated rather than random?
- What is the exploration-exploitation tradeoff? High N makes committed-
  archetype packs excellent but reduces off-archetype discovery (the best 4
  are all on-archetype). How does this affect picks 1-5 (exploration phase)?
- Should N be constant throughout the draft, or should it increase as the
  player commits? (Low N early for exploration, high N late for execution.)
- Can oversampling be framed as an explicit game rule ("the market scouts 48
  cards for you") or is it better left invisible?

**Output:** `docs/resonance/v12/research_pack_construction.md` (max 2000 words)

### Research Agent C: Concentration Math for AI Avoidance + Oversampling

**Question:** What combinations of AI avoidance strength and oversample size N
produce M3 >= 2.0 with "draw N, show best 4" packs from a pool?

Analyze:
- **Baseline:** With 120 cards, 8 archetypes, 15 per archetype, 36% sibling
  A-tier, N = 4 (uniform): what is M3? (Expected: ~0.17)
- **AI avoidance only (N = 4):** If 5 AIs avoid the player's archetype, the
  player's archetype accumulates in the pool. Model the S/A trajectory over 30
  picks. What M3 does avoidance alone produce with uniform packs?
- **Oversampling only (no avoidance):** What N achieves M3 >= 2.0 at draft
  start (5 S/A in pool)? How does N need to grow as S/A depletes through the
  draft? Model the N required at picks 1, 10, 20, 30.
- **Combined:** AI avoidance + oversampling. Avoidance maintains S/A count at
  ~5; oversampling at N = 48 yields M3 ≈ 2.0. Verify this interaction
  mathematically. What is the sensitivity — what happens at N = 36? N = 64?
- **Late-draft analysis:** With avoidance, only the player depletes their own
  S/A. After taking 3 S/A by pick 15, 2 remain. What is M3 at N = 48? Does
  the draft need multi-round refills to maintain S/A supply, or does avoidance
  alone suffice?
- **Comparison to V9:** V9 contracted the pool from 360 to 17 cards, achieving
  60%+ archetype density. "Draw 48, show best 4" from a 120-card pool with 5
  S/A achieves M3 ≈ 2.0 without contraction. At what N does V12 match V9's
  M3 = 2.70?
- **Exploration phase (picks 1-5):** Before the player commits, the system
  cannot rank by archetype fitness (no archetype inferred yet). What should
  "best 4" mean during exploration? Options: rank by power, rank by diversity,
  uniform random, or N = 4 (no oversampling during exploration).

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
- Pack construction uses "draw N, show best 4" oversampling (N is a design
  variable; N = 4 means uniform random baseline)

**Output format (all agents):**

1. Key findings (5-7 bullets)
2. Three algorithm proposals: name, one-sentence description, technical spec,
   predicted M3/M10/M11'/M6/M12/M13/M14
3. Champion selection with justification
4. Champion deep-dive: pick-by-pick walkthrough showing when AI avoidance kicks
   in, how pack construction changes, what the player sees, pool composition
   evolution, failure modes
5. Complete specification (pool size, oversample N, "best 4" ranking criterion,
   AI count, AI avoidance model, AI inference mechanism, AI pick logic, player
   information)

Max 1500 words per agent.

### Agent 1: Minimal Avoidance + No Oversampling (Isolation Test)

**Starting point:** Test AI avoidance alone, with N = 4 (no oversampling).
Uniform random 4-card packs. This isolates the contribution of avoidance
behavior.

**Question:** How much M3 improvement does AI avoidance alone produce over a
Level 0 baseline, when packs are not oversampled?

Explore:
- With 5 Level 0 AIs (no avoidance), M3 should be ~0.17-0.25 (V11 SIM-1
  baseline). What does M3 become when AIs avoid the player's detected
  archetype?
- How much pool-level archetype accumulation does avoidance create? If 5 AIs
  stop taking Blink cards after pick 6, how does Blink's count in the pool
  grow?
- Is the avoidance effect large enough to be meaningful without oversampling?
- What is the sensitivity to avoidance timing (pick 5 vs pick 8 vs pick 12)?
- How does the player's archetype inference accuracy affect the mechanism?

### Agent 2: Moderate Oversampling + Gradual Avoidance

**Starting point:** Combine graduated AI avoidance with moderate oversampling
(N = 24-36). Test whether a lower N with avoidance can achieve M3 >= 1.5-2.0.

**Question:** What is the minimum N that produces acceptable M3 when combined
with gradual AI avoidance?

Explore:
- Test N = 24, 32, 36 with gradual avoidance (ramp from pick 3 to pick 15).
  What M3 does each achieve?
- How does the "best 4" ranking work during exploration (picks 1-5) before
  archetype inference is confident? Options: rank by power, rank by symbol
  diversity, or use low N during exploration.
- Should N increase over the draft? (N = 8 for picks 1-5, N = 36 for picks
  6-15, N = 48 for picks 16+.) This mirrors V9's increasing contraction.
- At what N does the player start to notice that packs are "too good" — always
  containing on-archetype cards? Is there a perceptual sweet spot?
- How does moderate N interact with off-archetype variety (M4)? If best-4
  always includes on-archetype cards, are the remaining slots diverse enough?

### Agent 3: Heavy Oversampling + Delayed Avoidance

**Starting point:** Use high N (48-64) with delayed AI avoidance (pick 8+).
The "best 4" ranking uses pair-affinity scores (hidden 8-bit metadata from V9)
for maximum targeting precision.

**Question:** Can heavy oversampling with archetype-specific fitness ranking
achieve M3 >= 2.0 even with delayed avoidance?

Explore:
- Test N = 48 and N = 64 with delayed avoidance (pick 8+). What M3 does each
  achieve? Does high N compensate for delayed avoidance?
- How does the ranking criterion affect results? Compare: ranking by archetype
  fitness (pair-affinity) vs ranking by visible symbol match only. Is hidden
  metadata necessary for effective oversampling, or do visible symbols suffice?
- Delayed avoidance (pick 8+) means the first 7 picks have zero avoidance
  benefit — AIs may take S/A from the player's archetype early. How much S/A
  is lost in picks 1-7, and can high N compensate?
- Is oversampling with pair-affinity ranking V9 contraction by another name?
  V9 removed low-relevance cards; oversampling includes high-relevance cards.
  The direction is opposite (inclusion vs exclusion) but the effect is similar.
  Evaluate honestly.
- Can a floor slot be added within the oversampling framework? E.g., "draw N,
  guarantee 1 S/A in the top 4, fill remaining 3 from best of N." How does
  this interact with the oversampling math?

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
player's archetype once detected. Combined with moderate oversampling (N =
24-36).

**Question:** Does maximizing AI count and minimizing open lanes, combined with
universal avoidance, allow lower N for the same M3?

Explore:
- With 7 AIs avoiding the player's archetype, the player faces zero
  competition and S/A supply is maximally preserved. Does this allow lower N
  (24-32) to achieve M3 >= 2.0?
- But with only 1 open lane, there's no "choosing the right lane" skill — the
  open lane is whatever the player picks. Is this acceptable?
- What happens to game-to-game variety? C(8,7) = 8 compositions vs
  C(8,5) = 56. Is 8 enough variety?
- The 1-open-lane structure eliminates M12 (signal reading) as a skill axis.
  Is this acceptable? What does it replace it with?
- Does 7-AI avoidance create enough pool-level concentration that the
  oversampled "best 4 of N" consistently contains 2+ S/A at moderate N?

### Agent 6: Hybrid Approaches + Novel Mechanisms

**Starting point:** Free exploration. Combine AI avoidance with oversampling
in novel ways, or propose entirely new mechanisms.

Explore freely. Some starting ideas:
- **Progressive N:** N starts low (4-8 for picks 1-5, exploration) and ramps
  up as the player commits (N = 48 by pick 10). This mirrors V9's contraction
  trajectory — early packs are diverse, late packs are focused. The player
  experiences natural concentration without any pool manipulation.
- **AI avoidance with multi-round refills + oversampling:** V11's 3-round
  structure with open-lane biased refills, plus AI avoidance within each round,
  plus moderate oversampling. Does the combination of refill bias + avoidance +
  oversampling finally cross M3 >= 2.0 at lower N?
- **Oversampling without hidden metadata:** Can "best 4 of N" ranking work
  using only visible resonance symbols (no pair-affinity encoding), making the
  entire mechanism transparent? The player's visible resonance signature
  determines "best" — this is derivable from purely public information.
- **Avoidance cascade:** When the player commits to archetype X, AIs avoid X.
  This pushes AIs toward the remaining 7 archetypes, creating secondary
  avoidance effects. Combined with oversampling, the enriched pool makes N
  more efficient.
- **Split oversampling:** Draw N cards, but show 2 "best for your archetype" +
  2 "highest power regardless of archetype." Maintains exploration tension
  even in late draft — the player must choose between synergy and raw power.
- **Explicit N as game rule:** What if the player knows N? "The market scouts
  48 cards and shows you the 4 best matches." Does transparency change the
  player experience? Does it create a different skill axis?

---

## Round 3: Critic Review (1 agent, sequential)

A single critic reads all 6 design proposals, all research, and this plan.

**Task:**

1. Rank all proposals on: M3/M11' potential, player experience, simplicity,
   signal reading quality, AI avoidance narrative quality, oversampling honesty.
2. Evaluate whether AI avoidance is genuinely "public information" behavior or
   a dressed-up Level 2+ mechanism. Where is the line?
3. Assess oversampling configurations: is "draw N, show best 4" honest? At
   what N does it feel curated vs natural? Is it V9 contraction in disguise?
4. Evaluate the M3 target: is M3 >= 2.0 with 4-card packs a reasonable target
   for the oversampling paradigm, or should it be recalibrated?
5. Evaluate the interaction between AI avoidance and oversampling N: are they
   complementary, redundant, or in tension? What is the minimum viable N with
   avoidance vs without?
6. Evaluate whether progressive N (low early, high late) is better than
   constant N for player experience and metric performance.
7. Propose 1-2 hybrid designs combining the best elements.
8. Recommend 4-6 algorithms for simulation.

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
- Must implement oversampled pack construction (draw N, rank by fitness, show
  best 4) with specified N value
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
- **Oversampling analysis:** What is the actual per-pack archetype density
  achieved at the specified N? How does it compare to N = 4 (uniform baseline)?
  Track S/A count remaining in pool over the draft to verify avoidance
  maintains supply.
- Pack quality distribution (p10/p25/p50/p75/p90 for picks 6+)
- Consecutive bad pack analysis
- **Pool composition trajectory:** Show how archetype distribution evolves
  as AIs avoid the player's lane
- 2 draft traces (committed player, signal reader) — including AI avoidance
  moments
- Comparison to V9 baseline and V11 results
- Self-assessment: Is AI avoidance + oversampling a viable replacement for
  V9 contraction, or is it a complement?

---

## Round 5: Final Synthesis (1 agent)

**Produces two files:**

### File 1: `docs/resonance/v12/final_report.md` (max 4000 words)

1. Unified comparison table (all V12 algorithms + V9/V10/V11 baselines)
2. The key question: **Can AI avoidance + oversampled pack construction replace
   V9's virtual contraction?**
3. AI avoidance analysis: which models work and which are surveillance?
4. Oversampling analysis: what N values achieve M3 targets? Is "draw N, show
   best 4" honest? How does the ranking criterion affect results?
5. The interaction: how much does avoidance contribute vs oversampling N?
6. Per-archetype convergence for top 3 algorithms
7. V12 vs V9 vs V10 vs V11 comparison
8. Recommendation tiers:
   - **Pure AI Avoidance:** Best design using only AI avoidance behavior (N = 4,
     no oversampling). Establishes the demand-side contribution.
   - **Standard:** Best overall design combining AI avoidance + oversampling.
     Specifies the optimal N value.
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
4. Structural findings about AI avoidance and oversampling

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
2. **Oversampling is the supply-side amplifier.** V11 proved that uniform
   4-card packs (N = 4) from a 120-card pool cannot achieve M3 >= 2.0.
   Oversampling (draw N > 4, show best 4) is how V12 bridges the pack-sampling
   bottleneck. N is the primary tuning parameter. The "best 4" ranking
   criterion (visible symbols only vs hidden pair-affinity) is a secondary
   design variable.
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
7. **M3 target may need recalibration.** M3 = 2.0 requires N ≈ 48 with AI
   avoidance maintaining S/A supply. Agents should evaluate whether a lower M3
   target (e.g., 1.5, achievable at N ≈ 36) produces acceptable player
   experience, and what the minimum M3 is for the draft to "feel good."
8. **Separate avoidance from oversampling.** Agent 1 explicitly tests avoidance
   alone (N = 4) to isolate its contribution. This is essential calibration.
   If avoidance alone produces M3 = 0.5, and oversampling alone (N = 48, no
   avoidance) produces M3 = 1.5, but the combination produces M3 = 2.5, the
   interaction effect is clear.
9. **Transparency over stealth.** The AI avoidance mechanism should be visible
   to the player through Design 5's information system. When AIs start avoiding
   the player's archetype, the depletion trend arrows should show it. The
   player should be able to observe and exploit this behavior.

## Recovery

Check which `docs/resonance/v12/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained.
