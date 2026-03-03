# Resonance Draft System V12 — Orchestration Plan

## Background: The Problem We're Solving

### What Is Dreamtides?

Dreamtides is a roguelike deckbuilding game. Before each battle run, the player
drafts a 30-card deck from a shared card pool. The draft is the core
progression system — a good draft produces a powerful, synergistic deck; a bad
draft produces an incoherent pile. The draft must feel like the player *earned*
their deck through skillful card evaluation and strategic commitment.

### How the Draft Works

The player drafts at a table with AI opponents, all sharing a single **face-up
card pool**. The player can browse the full pool at any time — every card is
visible. When it's time to pick, the system presents 4 cards from the pool and
the player picks 1 (show 4, pick 1). Between the player's picks, AI opponents
also take cards from the pool. The player can see that cards are disappearing
from the pool, but **which cards were taken by which drafter is secret** — the
player must infer who is drafting what by observing patterns in what remains.

Over 30 picks, the player builds a 30-card deck. The key strategic decision is
**archetype commitment**: the game has 8 archetypes, each built around
synergistic card combinations. A deck focused on one archetype is stronger than
a deck scattered across many. The player must identify which archetype is
"open" (available with good cards) and commit to it early enough to draft
enough synergy pieces.

### Archetypes and Resonance

Each archetype has a primary and secondary **resonance symbol** (Tide, Stone,
Ember, Zephyr). Cards display 0-2 visible resonance symbols. The 8 archetypes
form a circle where adjacent archetypes share a resonance symbol:

| Archetype | Primary | Secondary |
|-----------|---------|-----------|
| Flash/Tempo | Zephyr | Ember |
| Blink/Flicker | Ember | Zephyr |
| Storm/Spellslinger | Ember | Stone |
| Self-Discard | Stone | Ember |
| Self-Mill/Reanimator | Stone | Tide |
| Sacrifice/Abandon | Tide | Stone |
| Warriors/Midrange | Tide | Zephyr |
| Ramp/Spirit Animals | Zephyr | Tide |

Each card has a hidden **pair-affinity score** — how well it fits each
archetype pair. A card might display an Ember symbol but actually be much
better for Blink than for Storm. The player discovers this by reading the
card's text; the system knows it from the pair-affinity metadata (8 bits per
card).

### Card Quality Tiers

Cards are rated by **fitness** for each archetype:
- **S-tier / A-tier (S/A):** High-synergy cards that are strong in a specific
  archetype. These are what the player wants.
- **C-tier / F-tier (C/F):** Generic or low-synergy cards. Filler.

The **sibling A-tier rate** varies by archetype pair:

| Pair | Sibling A-Tier Rate |
|------|:---:|
| Warriors / Sacrifice (Tide) | 50% |
| Self-Discard / Self-Mill (Stone) | 40% |
| Blink / Storm (Ember) | 30% |
| Flash / Ramp (Zephyr) | 25% |
| **Weighted Average** | **~36%** |

This means roughly 36% of cards in a given archetype are S/A tier — the cards
the player wants to see in their packs.

### The Core Design Challenge

The draft must achieve two goals simultaneously:

1. **Concentration:** After the player commits to an archetype (around pick
   5-6), their packs should contain good cards for that archetype. The key
   metric is **M3: average S/A cards for the committed archetype per pack,
   picks 6+.** Target: >= 2.0. This means the player should see at least 2
   cards worth taking for their archetype in a typical pack of 4.

2. **Signal reading:** The player should be able to *figure out* which
   archetype is open by reading signals from the draft. This creates a skill
   axis — better players read signals faster, commit earlier, and get better
   decks. The metric is **M12: signal-reader M3 minus committed-player M3.**
   Target: >= 0.3.

The fundamental tension: concentration requires that packs are biased toward
the player's archetype. But signal reading requires that the bias comes from
the *state of the draft*, not from the system cheating on the player's behalf.
The player should earn concentration through correct reading, not receive it
automatically.

### What We've Tried (V9-V11)

**V9: Virtual Pool Contraction (M3 = 2.70 — the gold standard)**

V9's approach: maintain a pool of 360 cards. Each pick, silently remove the
bottom 12% of cards by relevance to the player's emerging archetype. The pool
shrinks from 360 to ~17 cards by pick 30. By late draft, the surviving cards
are 60%+ the player's archetype — producing excellent packs.

V9 works mathematically but has a narrative problem: the player never sees
*why* the pool is concentrating. Cards just silently disappear. There are no
opponents, no competition, no table to read. The contraction is invisible and
the draft feels like a slot machine that gradually gets nicer.

V9 uses 4-card packs: 3 random from the surviving pool + 1 guaranteed
top-quartile "floor slot" from pick 3 onward.

**V10: AI Drafters (M3 = 0.84 — structural failure)**

V10's approach: introduce 5 AI opponents who physically take cards from the
shared pool. Each AI is assigned one of 8 archetypes and takes good cards for
its archetype. The 3 archetypes not assigned to any AI are "open lanes" — the
player should draft one of these.

V10 proved the **AI drafter narrative** is valuable: "other players took those
cards" explains why certain archetypes are scarce. Signal reading becomes
natural — the player observes which archetypes are being depleted by AIs and
picks the one nobody is contesting.

But V10 failed structurally. Three root causes:
1. **Pool exhaustion:** AIs remove 20-35 cards per round, exhausting the
   360-card pool by pick 12-15.
2. **S/A preferential depletion:** AIs take the best cards first, draining
   quality from the pool. V9 removed low-relevance cards (enriching quality);
   V10's AIs remove high-relevance cards (depleting quality).
3. **Targeting dilution:** The player's archetype is 1 of 3 open lanes. AIs
   concentrate the pool toward open lanes generally, but the player's specific
   archetype only gets ~1.7x concentration vs V9's 5-7x.

**V11: Multi-Round Refills (best M3 = 0.89 — structural failure)**

V11's approach: keep the AI drafters from V10, but replenish the pool between
rounds (like opening new packs in a booster draft). This fixes pool exhaustion
(root cause 1) and partially addresses S/A depletion (root cause 2).

V11 explored the complete parameter space: balanced refills, biased refills,
declining refills, asymmetric replacement, 3/4/5-round structures. Six
simulations spanning the plausible design space. All failed M3 by wide margins.

V11 identified the **pack-sampling bottleneck** as the binding constraint: with
a pool of 100-130 cards and 8 archetypes, the player's committed archetype is
12-21% of the pool. Drawing 4 cards uniformly from this pool yields ~0.5-1.0
on-archetype cards per pack. With 36% sibling A-tier rate, expected S/A per
pack is ~0.18-0.36. M3 >= 2.0 requires 50% archetype density in the pack,
which is impossible without pool contraction or pack-level manipulation.

V11's positive contributions:
- **Design 5 information system:** Archetype availability bars, round-start
  snapshots, and depletion trend arrows. V12's face-up pool may supersede this
  (the player can browse the pool directly), but Design 5's UI concepts may
  still be valuable as overlays on the face-up pool.
- **Pack-sampling bottleneck identification:** The definitive finding that
  pool-level composition does not translate to pack-level quality with uniform
  sampling. This is the constraint V12 must address.

### Where V12 Picks Up

V12 starts from V11's conclusion: uniform pack sampling cannot achieve M3 >=
2.0 from a large pool. But V11 only tested Level 0 (static) AIs. V12
introduces a **face-up shared pool** and three mechanisms that work together:

1. **AI avoidance:** V10 and V11 used Level 0 AIs that ignore the player's
   behavior. But with a face-up pool, all drafters can observe depletion
   patterns and rationally avoid contested archetypes. If AIs avoid the
   player's archetype, the player's S/A cards stay in the pool. This was
   raised during V11 but not explored — V12 makes it the central thesis.

2. **Physical pool contraction:** V11 tested declining refills with Level 0
   AIs → M3 = 0.83. But with AI avoidance, declining refills produce a
   fundamentally different result: AIs physically take non-player cards, the
   pool shrinks (refills don't fully replenish), and what remains is
   concentrated toward the player's archetype. This is V9's contraction
   achieved transparently through actual drafting behavior.

3. **Modest oversampling:** With a contracted pool (20-30 cards late-draft,
   concentrated toward the player's archetype), the system draws 8-12 cards
   and shows the best 4. This is a modest supplement, not the primary
   mechanism — pool contraction does the heavy lifting.

---

## The Central Idea

V12 introduces a **face-up shared pool** — the player can browse all cards in
the pool at any time. AI opponents draft from the same pool. The pool is fully
visible; who took what is secret. Three mechanisms produce concentration:

1. **Public-information-reactive AI avoidance:** AIs observe the same face-up
   pool the player sees. They infer what the player is drafting from depletion
   patterns and rationally avoid competing for those cards.
2. **Physical pool contraction:** AIs physically take non-player-archetype
   cards. Declining refills mean the pool shrinks over the draft. What remains
   is concentrated toward the player's archetype — the same effect as V9's
   contraction, but achieved transparently through actual drafting.
3. **Modest oversampling:** When it's time to pick, the system draws 8-12
   cards from the (now-concentrated) pool and presents the best 4. This is a
   light supplement, not the primary driver.

**Player-facing explanation:** "You're drafting at a table with AI opponents.
The card pool is face-up — browse it anytime to see what's available. When you
pick, the system shows you 4 strong options from the pool. Watch what
disappears to figure out what your opponents are drafting, and find the lane
nobody is contesting."

V12 attacks the problem differently from V9-V11: AI avoidance preserves the
player's S/A supply (demand side), declining refills shrink the pool so the
player's archetype dominates what remains (supply concentration), and modest
oversampling ensures the player sees those cards in their packs (supply
curation). The pool is face-up, honest, and shared. Concentration emerges
from rational drafting behavior by all participants, amplified by natural
pool shrinkage.

### The Three Design Levers

**Lever 1: AI Avoidance Behavior.** AIs see the same face-up pool as the
player. They observe which archetypes are depleting fastest and infer that
another drafter is contesting those cards. They then rationally avoid
competing for those archetypes — the same behavior a skilled human opponent
would exhibit at a real draft table.

The key insight: the face-up pool makes avoidance natural. No one needs to
observe individual picks (those are secret). AIs just watch the pool change
and draw the same inferences a player would: "Blink cards are disappearing —
someone is drafting Blink, I'll focus elsewhere." The information is fully
symmetric — the player reads the pool the same way.

**Lever 2: Physical Pool Contraction (Declining Refills).** This is the
primary concentration mechanism. AIs physically take cards from non-player
archetypes. Between rounds, the pool is partially replenished — but each
refill is smaller than the last (declining refills). The pool shrinks over
the draft, and because AIs avoid the player's archetype, the player's cards
accumulate as a larger fraction of what remains.

V11 tested declining refills with Level 0 AIs (no avoidance) → M3 = 0.83.
The refills reset the concentration gradient because balanced refills
restored all archetypes equally. With AI avoidance, the dynamic changes
fundamentally: AIs deplete non-player archetypes faster than refills restore
them, while the player's archetype is only depleted by the player's own
picks. The pool contracts AND concentrates.

**The math:** Starting pool 120 cards. Each pick cycle removes 6 cards (5 AIs
+ 1 player). With declining refills (e.g., 60/36/0 across 3 rounds), the
pool shrinks from 120 to ~20-30 by late draft. If AIs avoid the player's
archetype, the player's ~15 starting cards minus ~10 player picks = ~5
remaining, but refills add more. The key question for simulation: at what
pool size and archetype density does M3 reach 2.0 with modest oversampling?

Example late-draft scenario (pool = 25 cards, player's archetype = 50% of
pool = ~12 cards, ~5 S/A among them):
- Draw 8: expected S/A = 8 × 5/25 = 1.6
- Draw 12: expected S/A = 12 × 5/25 = 2.4
- Show best 4 of those → M3 ≈ 1.6-2.4

This is V9's contraction achieved through transparent physical drafting. V9
silently removed low-relevance cards; V12's AIs physically take cards from
other archetypes. The pool shrinks the same way, but the player can see it
happening and understand why.

**Lever 3: Modest Oversampling (Draw 8-12, Show Best 4).** With pool
contraction doing the heavy lifting, oversampling is a light supplement. The
system draws 8-12 cards from the face-up pool and shows the best 4 by fitness
for the player's emerging archetype. This is modest curation — not the
dramatic "draw 48 from 120" that would feel like hidden manipulation.

The narrative framing is natural: the player can see the pool, the system
picks a handful and recommends the best options. At N = 8-12 from a pool of
25-30 cards, the player is seeing roughly a third to half of what's
available — comparable to a real draft where you see a pack of cards, not the
entire card pool.

---

## The Face-Up Pool and Information Model

### The Pool Is Face-Up

The card pool is fully visible to all drafters (player and AIs) at all times.
The player can browse every card in the pool — scrolling, filtering by
archetype or resonance, reading card text. This is the foundation of V12's
information model: the pool itself is the information.

This replaces V11's Design 5 information system (bars, trends, snapshots) with
something simpler and more direct: the player just *looks at the pool*. They
can see that Blink has 14 cards remaining while Storm has only 6. They can see
which specific cards are still available. The information is complete,
unmediated, and always current.

### What Is Public (Visible to All Drafters)

- **The full pool:** Every card currently in the pool, face-up. Both the player
  and AIs can see all cards, their resonance symbols, and their text.
- **Pool changes over time:** Cards disappear from the pool as drafters take
  them. Everyone can see what's gone — the pool is a shared, shrinking
  resource.
- **Aggregate depletion patterns:** "Tide cards are disappearing faster than
  Ember cards" is visible to anyone watching the pool.

### What Is Secret

- **Who took what:** Individual draft picks are secret. The player sees cards
  leave the pool but does not see which AI (or whether an AI at all) took a
  specific card. This is the signal-reading skill axis: the player must infer
  "someone is aggressively drafting Sacrifice" from the pattern of Tide-symbol
  cards disappearing, without knowing which opponent is responsible.
- **The player's picks (from AIs' perspective):** AIs can see that cards leave
  the pool, but they don't directly observe the player's picks either. They
  infer the player's archetype from the same depletion patterns — exactly as
  the player infers AI behavior. The information is symmetric.
- **Internal strategy:** No drafter's commitment level, future plans, or
  fitness evaluations are visible. Only the pool state and its changes over
  time.

### How AIs Read the Table

AIs infer what the player is drafting using only pool-observable information:
- After each pick cycle, compare the current pool to the previous state
- Identify which archetypes lost cards (depletion patterns)
- Attribute depletion to "some drafter is taking those" without knowing who
- Build a probabilistic model: "Ember cards are depleting disproportionately
  fast — likely one drafter is focused on an Ember archetype"
- Reduce priority for archetypes that appear contested

This is symmetric: the player uses the same process. "Storm cards keep
disappearing — someone is drafting Storm, I should avoid it." Both player and
AIs are reading the same face-up pool and drawing the same inferences.

Note that AIs cannot observe the player's picks directly. They infer the
player's archetype from depletion patterns, which are confounded by other AIs'
picks. This imprecision is realistic and intentional — it mirrors the
uncertainty a real drafter faces when reading the table.

---

## Fixed Assumptions (Not Variables in V12)

The following are constant across all V12 designs. See the Background section
for detailed explanations.

- **Fitness model:** Graduated Realistic (~36% weighted-average sibling A-tier)
- **Total draft:** 30 picks
- **Pack size:** 4 cards shown (show 4, pick 1). How those 4 are selected from
  the pool IS a design variable (see Variable 2)
- **Archetypes:** 8 on a circle, each with primary/secondary resonance symbol
- **Visible symbol distribution:** ~11% generic, ~79% single-symbol, ~10%
  dual-symbol
- **Player strategies for simulation:** Archetype-committed (commits pick 5-6),
  Power-chaser (ignores archetype), Signal-reader (reads pool state)

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
- How many picks before depletion patterns are strong enough for AIs to infer
  the player's archetype? (Remember: AIs don't see individual picks — they
  only see the pool shrinking. With 6 drafters, each pick cycle removes 6
  cards. AIs must disentangle the player's signal from other AIs' picks.)
- Should avoidance be binary (avoid/don't avoid) or graduated (reduce
  priority)?
- Should AIs avoid only the player's specific archetype, or also the player's
  resonance symbol pair?
- What happens when the player pivots? How quickly do AIs re-read the pool?
- How does inference accuracy degrade when multiple AIs draft the same
  archetype? (Confounding: AIs can't distinguish player depletion from
  other-AI depletion.)

### Variable 2: Oversample Size (N)

How many cards does the system draw from the pool before showing the best 4?
Oversampling is a modest supplement to pool contraction, not the primary
mechanism. N is limited to the 4-12 range.

| Config | N Drawn | Show | Notes |
|--------|:-------:|:----:|-------|
| A: No Oversample | 4 | 4 | Uniform random baseline — isolates pool contraction |
| B: Light Oversample | 8 | Best 4 | Draw 8, show best 4 — light curation |
| C: Moderate Oversample | 10 | Best 4 | Draw 10, show best 4 |
| D: Standard Oversample | 12 | Best 4 | Draw 12, show best 4 — upper bound |

At N = 4 (uniform), M3 depends entirely on pool contraction. At N = 8-12
from a contracted pool of 25 cards with 5 S/A, expected S/A = N × 5/25 =
1.0-2.4. The oversampling provides a meaningful but modest boost — the
difference between "usually 1 S/A" and "usually 2 S/A" in a pack.

**"Best 4" ranking:** Cards drawn are ranked by fitness for the player's
inferred archetype. S/A cards for the player's archetype rank highest (~0.9
fitness), followed by sibling-archetype S/A (~0.5-0.7), on-archetype C/F
(~0.3-0.5), and off-archetype cards (~0.0-0.2). The top 4 by this ranking
are shown.

**The narrative framing:** At N = 8-12, oversampling feels natural — the
system draws a handful of cards and shows the best options. The player can
browse the face-up pool and verify these cards exist. At this modest scale,
it doesn't feel like hidden manipulation — it feels like the system picking
a few cards from the market and recommending the best ones.

**The face-up pool replaces Design 5 information.** With the pool visible, the
player doesn't need archetype bars or trend arrows — they can browse the pool
directly and see exactly what's available. Design 5's UI concepts (grouping
by archetype, visual indicators of quantity) may still be useful as overlays
on the pool browser, but the underlying information is complete and
unmediated. The exploration phase (picks 1-5) is served by the pool browser
rather than by the pack contents — the player browses the pool to decide
which archetype to target, then receives modestly oversampled packs once
committed.

### Variable 3: Pool Structure and Refill Schedule

The pool is always face-up. Pool contraction via declining refills is the
primary concentration mechanism. The key design question is the refill
schedule — how much is replenished between rounds, and how fast does the pool
shrink?

| Structure | Description | Late-Draft Pool |
|-----------|-------------|:---------:|
| A: Static Pool | No refills, pool shrinks from drafting alone | Very small (~0-20) |
| B: Steep Decline (60/30/0) | 3 rounds, aggressively declining refills | Small (~20-30) |
| C: Moderate Decline (60/36/0) | 3 rounds, moderately declining refills | Medium (~25-40) |
| D: Gradual Decline (48/36/21/0) | 4 rounds, gently declining refills | Medium (~30-50) |
| E: Continuous Market | Drafted cards partially replaced face-up | Stable (~60-80) |

V11 tested declining refills with Level 0 AIs: best M3 = 0.83 (SIM-4).
The refills reset the concentration gradient because AIs didn't avoid the
player's archetype. With AI avoidance, the same refill schedules should
produce fundamentally different results — AIs deplete non-player archetypes
while refills partially restore all archetypes, but the net effect is
concentration toward the player's lane.

The critical interaction: pool contraction rate determines how much
oversampling is needed. A pool contracted to 25 cards with 50% player
archetype density needs only N = 8-12 for M3 ≈ 2.0. A pool that stays at 60
cards with 25% density would need much higher N. Agents should find the
refill schedule that contracts the pool enough for modest oversampling to
work.

**Pool sizing:** With 5 AIs + 1 player, each pick cycle removes 6 cards.
Over 30 picks = 180 total removals. Starting pool of 120 would exhaust
without refills by pick 20. The refill schedule must balance: enough cards
to sustain 30 picks, but declining enough that the pool contracts to 20-30
cards by late draft. Agents should determine the right starting pool size
and refill schedule.

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

### M3 Depends on Pool Contraction + Modest Oversampling

With pool contraction as the primary mechanism and oversampling at N = 8-12,
M3 depends on the late-draft pool size, archetype density, and N:

| Pool Size | Archetype % | S/A in Pool | N = 4 (uniform) | N = 8 | N = 12 |
|:---------:|:-----------:|:-----------:|:---:|:---:|:---:|
| 120 (no contraction) | 12% | 5 | 0.17 | 0.33 | 0.50 |
| 60 | 25% | 5 | 0.33 | 0.67 | 1.00 |
| 40 | 35% | 5 | 0.50 | 1.00 | 1.50 |
| 30 | 45% | 5 | 0.67 | 1.33 | 2.00 |
| 25 | 50% | 5 | 0.80 | 1.60 | 2.40 |
| 20 | 55% | 5 | 1.00 | 2.00 | 3.00 |

**The critical insight:** Pool contraction and oversampling are substitutes.
A large pool (120) needs high N (48+) for M3 = 2.0 — impractical. A
contracted pool (20-30) needs only N = 8-12 — natural and honest. V12's
strategy: use AI avoidance + declining refills to contract the pool, then
use modest oversampling as a light supplement.

**Target pool trajectory:** For M3 ≈ 2.0 at N = 8, the pool needs to
contract to ~20 cards with ~5 S/A remaining. For M3 ≈ 2.0 at N = 12, the
pool needs ~30 cards with ~5 S/A. AI avoidance is critical for maintaining
the S/A count — without it, AIs take S/A cards and late-draft S/A drops to
1-2, collapsing M3 regardless of pool size.

**Comparison to V9:** V9 contracted the pool from 360 to ~17 cards with 60%+
archetype density and a floor slot, achieving M3 = 2.70. V12 aims to contract
from 120 to ~20-30 cards with 45-55% archetype density and N = 8-12
oversampling. If the contraction trajectory is right, V12 should approach V9's
M3 through transparent physical mechanisms rather than invisible removal.

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
    oversample_n: int                  # N cards drawn from pool (4 = uniform, 8-12 = standard)
    show_count: int                    # always 4 (show best 4 of N drawn)
    ranking_method: str                # "archetype_fitness", "symbol_match", "power"
    player_signature: list[float]      # current resonance signature for ranking
    inferred_archetype: str            # player's inferred archetype for fitness ranking

class RefillSchedule:
    rounds: int                        # number of rounds (3-4)
    picks_per_round: list[int]         # picks in each round (e.g., [10, 10, 10])
    refill_amounts: list[int]          # cards added between rounds (e.g., [60, 36, 0])
    refill_bias: str                   # "balanced", "open_lane", "underrepresented"

class DraftState:
    pool: list[SimCard]                # face-up pool (visible to all drafters)
    pick_number: int
    player_picks: list[SimCard]        # player's drafted cards (secret from AIs)
    ai_picks: dict[int, list[SimCard]] # each AI's drafted cards (secret from player)
    pool_history: list[list[SimCard]]  # pool state at each pick (for depletion inference)
```

---

## Round 1: Research (3 parallel agents)

Pure research — no algorithm design. Map the AI avoidance + pool contraction +
pack construction design space.

### Research Agent A: AI Avoidance in Competitive Drafting

**Question:** How do human drafters read opponents and adjust strategy in
competitive draft formats, and how can AI drafters replicate this behavior?

**Context:** In V12, the card pool is face-up (all cards visible to all
drafters), but individual picks are secret. Drafters infer opponents' strategies
by watching which cards disappear from the pool, not by seeing picks directly.

Explore:
- In shared-pool games (7 Wonders, Sushi Go, Ascension-style markets), how do
  players observe and respond to opponents' picks? Is active avoidance of
  contested archetypes a documented strategy?
- In MTG drafts, how do skilled players read signals from passed packs? How
  does this translate to a face-up pool where you see the full supply but not
  who's buying?
- With a face-up pool and secret picks, how many pick cycles are needed to
  reliably infer another drafter's archetype from depletion patterns? How does
  the number of drafters (5-7 AIs) affect inference confidence? (More drafters
  = more confounding depletion.)
- How does avoidance timing affect draft dynamics? Early avoidance (pick 3-5)
  vs late avoidance (pick 8-10) — which produces better outcomes for all
  drafters?
- What is the boundary between "reading the table" (acceptable) and
  "surveillance" (unacceptable) in terms of player perception?

**Reads:** This plan, V10 final report (`docs/resonance/v10/final_report.md`),
V11 final report (`docs/resonance/v11/final_report.md`).

**Output:** `docs/resonance/v12/research_ai_avoidance.md` (max 2000 words)

### Research Agent B: Pool Contraction via Physical AI Drafting

**Question:** How does AI avoidance + declining refills produce pool
contraction, and what contraction trajectories achieve M3 >= 2.0 with modest
oversampling (N = 8-12)?

**Context:** V12's primary concentration mechanism is physical pool
contraction. AIs take non-player cards, declining refills don't fully
replenish, and the pool shrinks. With AI avoidance, the player's archetype
accumulates as a larger fraction of the shrinking pool. V11 tested declining
refills with Level 0 AIs (M3 = 0.83, SIM-4). V12 adds AI avoidance to the
same mechanism. Oversampling is limited to N = 8-12 (draw 8-12, show best 4).

Explore:
- **V11 declining refill results:** V11 SIM-4 used 4 rounds (8/8/7/7 picks)
  with declining balanced refills (48/36/21/0) and Level 0 AIs → M3 = 0.83.
  What specific pool composition trajectory did SIM-4 produce? Where did
  concentration fail — at the pool level or the pack level?
- **Adding avoidance to V11's refill schedules:** If SIM-4's AIs had avoided
  the player's archetype, how would the pool trajectory change? Model the
  archetype density at each round boundary.
- **Contraction targets:** For M3 ≈ 2.0 at N = 8, the pool needs ~20 cards
  with ~5 S/A. For N = 12, ~30 cards with ~5 S/A. What refill schedule
  achieves these targets?
- **Pool exhaustion risk:** With 6 removals per pick cycle and declining
  refills, at what point does the pool run out of cards? What is the minimum
  safe pool size?
- **Refill bias:** Should refills be balanced (equal per archetype) or biased
  (more open-lane cards)? V11 found biased refills are Level 0 (determined by
  pre-draft AI configuration). Does bias help or hurt with AI avoidance?
- **Starting pool size:** Is 120 right, or should V12 use a different starting
  pool? Larger pools (180, 240) take longer to contract but provide more card
  variety. Smaller pools (80, 100) contract faster but may feel thin.
- **Round structure:** 3 rounds vs 4 rounds. More rounds = more refill events
  (each partially resets gradient). Fewer rounds = fewer resets but steeper
  per-boundary decline.
- **How do existing draft formats handle pool shrinkage?** MTG Rochester draft
  (face-up), Ascension market rows, 7 Wonders card passing. Which create
  natural late-game concentration?

**Reads:** This plan, V11 final report (`docs/resonance/v11/final_report.md`),
V11 algorithm overview (`docs/resonance/v11/algorithm_overview.md`), V11
Design 3 (`docs/resonance/v11/design_3.md`).

**Output:** `docs/resonance/v12/research_pool_contraction.md` (max 2000 words)

### Research Agent C: Concentration Math for Pool Contraction + Oversampling

**Question:** What pool contraction trajectories produce M3 >= 2.0 with modest
oversampling (N = 8-12)?

Analyze:
- **Baseline (no contraction, no avoidance, N = 4):** 120 cards, 8 archetypes,
  15 per archetype, 36% sibling A-tier. M3 ≈ 0.17. This is the floor.
- **Avoidance only (no contraction, N = 4):** Static 120-card pool, 5 AIs
  avoid player's archetype. Player's archetype accumulates but pool stays
  large. Model archetype density and M3 over 30 picks. Expected: modest
  improvement, far below 2.0.
- **Contraction only (declining refills, no avoidance, N = 4):** V11 SIM-4
  result (M3 = 0.83). What does the pool trajectory look like?
- **Avoidance + contraction (N = 4):** The key combination. Model: 5 AIs with
  avoidance from pick 6, declining refills (e.g., 60/36/0). Track pool size,
  archetype density, and S/A count at each pick. What M3 does this achieve
  with uniform 4-card packs (no oversampling)?
- **Avoidance + contraction + oversampling (N = 8 and N = 12):** Add modest
  oversampling to the above. What M3 does each N achieve? Is the pool
  contraction sufficient that N = 8 reaches M3 ≈ 2.0?
- **S/A trajectory:** With avoidance, only the player takes their own S/A.
  Starting S/A: ~5 per archetype. Refills add more S/A. By pick 20, how many
  S/A remain? Is the player at risk of exhausting their own S/A supply?
- **Pool size sensitivity:** What if the pool contracts to 40 instead of 25?
  To 15? Plot M3 vs final pool size at N = 8 and N = 12.
- **Comparison to V9:** V9 achieved M3 = 2.70 with pool contraction from 360
  to 17 + floor slot. V12 aims for pool contraction from 120 to 20-30 +
  N = 8-12. Are these structurally equivalent?
- **Exploration phase (picks 1-5):** Before commitment, should N = 4 (uniform)
  or should the pool browser serve exploration entirely?

**Reads:** This plan, V11 final report (`docs/resonance/v11/final_report.md`),
V11 algorithm overview (`docs/resonance/v11/algorithm_overview.md`), V9
algorithm overview (`docs/resonance/v9/algorithm_overview.md`), V11 Design 3
(`docs/resonance/v11/design_3.md`).

**Output:** `docs/resonance/v12/research_concentration_math.md` (max 2000 words)

---

## Round 2: Algorithm Design (6 parallel agents)

Each agent reads all Round 1 research (`research_ai_avoidance.md`,
`research_pool_contraction.md`, `research_concentration_math.md`) plus this
plan, V11 final report (`docs/resonance/v11/final_report.md`), and V11
algorithm overview (`docs/resonance/v11/algorithm_overview.md`). Each explores
a different region of the V12 design space.

**Fixed for all agents:**
- Fitness: Graduated Realistic
- Total draft: 30 picks
- Pack size: 4 cards (show 4, pick 1)
- Visible symbols: ~10% dual-res
- AI drafter framing required
- All V10 and V11 structural findings available
- AIs must use public-information-based avoidance of the player's archetype
  somewhere in the design (the strength, timing, and mechanism vary by agent)
- Pool contraction via declining refills is the primary concentration
  mechanism. Agents must specify a refill schedule.
- Pack construction uses "draw N, show best 4" oversampling with N in the
  range 4-12 (N = 4 means uniform random baseline)

**Output format (all agents):**

1. Key findings (5-7 bullets)
2. Three algorithm proposals: name, one-sentence description, technical spec,
   predicted M3/M10/M11'/M6/M12/M13/M14
3. Champion selection with justification
4. Champion deep-dive: pick-by-pick walkthrough showing when AI avoidance kicks
   in, how pack construction changes, what the player sees, pool composition
   evolution, failure modes
5. Complete specification (starting pool size, refill schedule, oversample N
   (4-12), "best 4" ranking criterion, AI count, AI avoidance model, AI
   inference mechanism, AI pick logic, player information)

Max 1500 words per agent.

### Agent 1: Avoidance + Contraction, No Oversampling (Isolation Test)

**Starting point:** Test AI avoidance + declining refills with N = 4 (no
oversampling). Uniform random 4-card packs. This isolates the contribution
of pool contraction + avoidance without any pack curation.

**Question:** How much M3 does AI avoidance + pool contraction produce with
uniform 4-card packs? This is the baseline — if contraction alone gets close
to M3 = 2.0, only light oversampling is needed.

Explore:
- V11 SIM-4 (declining refills, Level 0 AIs) → M3 = 0.83. What happens when
  you add AI avoidance to the same refill schedule (48/36/21/0, 4 rounds)?
- Track pool composition at each round boundary: how many of the player's
  archetype cards remain? What is archetype density?
- How does avoidance timing affect the contraction trajectory? (Avoidance
  from pick 5 vs pick 8 — earlier avoidance preserves more S/A.)
- What pool size does the draft reach by picks 20, 25, 30? Is the
  contraction sufficient for M3 ≈ 1.5+ even without oversampling?
- What is the sensitivity to refill schedule? Test 60/36/0 (3 rounds) vs
  48/36/21/0 (4 rounds) vs 60/0/0 (aggressive 3-round).

### Agent 2: Steep Contraction + Light Oversampling (N = 8)

**Starting point:** Aggressive declining refills (steep contraction) with
gradual AI avoidance and light oversampling (N = 8).

**Question:** Can aggressive pool contraction with N = 8 achieve M3 >= 2.0?

Explore:
- Test steep refill schedules: 60/30/0 (3 rounds), 60/20/0 (3 rounds),
  48/24/0/0 (4 rounds). Which contracts the pool to ~20-25 cards by pick 25?
- With gradual avoidance (ramp from pick 3 to pick 15), how does archetype
  density evolve? At what pick does the player's archetype become 40%+ of
  the pool?
- At N = 8 from a contracted pool of 20-25 cards, what M3 is achieved?
- How does the "best 4" ranking work during exploration (picks 1-5) before
  archetype inference is confident? Options: rank by power, rank by symbol
  diversity, or use N = 4 during exploration.
- Pool exhaustion risk: with steep decline, does the pool run out of cards
  before pick 30? What AI saturation threshold prevents this?
- How does N = 8 interact with off-archetype variety (M4)? From a 25-card
  pool, drawing 8 is a third of the pool — does this leave enough variety?

### Agent 3: Moderate Contraction + Standard Oversampling (N = 12)

**Starting point:** Moderate declining refills with delayed AI avoidance
(pick 8+) and standard oversampling (N = 12). The "best 4" ranking uses
pair-affinity scores (hidden 8-bit metadata from V9) for targeting precision.

**Question:** Can moderate pool contraction with N = 12 achieve M3 >= 2.0
even with delayed avoidance?

Explore:
- Test moderate refill schedules: 60/36/0 (3 rounds), 48/36/21/0 (4 rounds).
  Which contracts the pool to ~30-40 cards by pick 25?
- Delayed avoidance (pick 8+) means 7 picks of AIs potentially taking the
  player's S/A. How much S/A is lost in picks 1-7? Does moderate contraction
  compensate?
- At N = 12 from a pool of 30-40 cards, what M3 is achieved? Compare ranking
  by archetype fitness (pair-affinity) vs visible symbol match only.
- Can a floor slot be added within the oversampling framework? E.g., "draw 12,
  guarantee 1 S/A in the top 4, fill remaining 3 from best of 12." How does
  this interact with the contraction math?
- How does the contraction trajectory compare to V9? V9 contracted from 360
  to 17 with invisible removal. V12 Agent 3 contracts from 120 to 30-40 with
  physical drafting. Is the density at 30-40 cards sufficient, or does the
  pool need to contract further?

### Agent 4: V9 Engine + AI Avoidance Narrative (Non-Face-Up Fallback)

**Starting point:** V9 Hybrid B's contraction engine runs unchanged. This is
the **non-face-up fallback** — if V12's face-up pool mechanisms don't achieve
M3 >= 2.0, V9's invisible contraction remains the proven engine. AI avoidance
is layered on top as a narrative enhancement — AIs appear to avoid the
player's archetype because the contraction engine removes non-relevant cards
and attributes removals to AIs.

**Important:** V9's invisible contraction is incompatible with a face-up pool
(you can't silently remove cards from a visible pool). This agent evaluates
the V9 engine in its native non-face-up context, enhanced with AI avoidance
narrative. It serves as the performance ceiling and fallback comparison.

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
- How does the non-face-up V9 experience compare to V12's face-up designs in
  terms of player agency and information access? What does the player lose
  by not being able to browse the pool?

### Agent 5: High-AI-Count + Avoidance (7 AIs, 1 Open Lane)

**Starting point:** 7 AIs, only 1 open lane per game. All 7 AIs avoid the
player's archetype once detected. Combined with declining refills and light
oversampling (N = 8).

**Question:** Does maximizing AI count accelerate pool contraction enough to
achieve M3 >= 2.0 with light oversampling?

Explore:
- With 7 AIs (8 cards removed per pick cycle instead of 6), the pool
  contracts faster. What refill schedule keeps the pool viable for 30 picks?
- 7 AIs avoiding the player's archetype = zero competition + faster
  contraction of non-player cards. Does this reach 50%+ archetype density
  earlier than 5-AI designs?
- But with only 1 open lane, there's no "choosing the right lane" skill — the
  open lane is whatever the player picks. Is this acceptable?
- What happens to game-to-game variety? C(8,7) = 8 compositions vs
  C(8,5) = 56. Is 8 enough variety?
- The 1-open-lane structure eliminates M12 (signal reading) as a skill axis.
  Is this acceptable? What does it replace it with?
- Does 7-AI contraction achieve M3 >= 2.0 at N = 8? At N = 4 (uniform)?

### Agent 6: Hybrid Approaches + Novel Mechanisms

**Starting point:** Free exploration. Combine AI avoidance, pool contraction,
and modest oversampling in novel ways, or propose entirely new mechanisms.

Explore freely. Some starting ideas:
- **Progressive N:** N starts at 4 (picks 1-5, exploration) and ramps to
  8-12 as the player commits. Combined with pool contraction, early packs are
  diverse (large pool, low N), late packs are focused (small pool, higher N).
- **Biased refills + avoidance:** V11's open-lane biased refills combined
  with AI avoidance. Biased refills add more cards for the player's archetype
  while avoidance prevents AIs from taking them. Does this compound the
  contraction effect enough to reduce N further (to 4-8)?
- **Oversampling without hidden metadata:** Can "best 4 of N" ranking work
  using only visible resonance symbols (no pair-affinity encoding), making
  the entire mechanism transparent?
- **Avoidance cascade:** When the player commits to archetype X, AIs avoid X.
  This pushes AIs toward the remaining archetypes, creating secondary
  concentration effects in the pool.
- **Split oversampling:** Draw N cards, but show 2 "best for your archetype" +
  2 "highest power regardless of archetype." Maintains exploration tension
  even in late draft.
- **Continuous market with avoidance:** Instead of declining refills, use a
  continuous market (drafted cards partially replaced). AIs cycle their
  archetypes through the market while the player's archetype accumulates.
  Does this achieve contraction through a different path?
- **Variable AI count by round:** Start with 3 AIs (slow contraction) and
  add 2 more in later rounds (fast contraction). Fewer AIs early = more
  exploration; more AIs late = faster concentration.

---

## Round 3: Critic Review (1 agent, sequential)

A single critic reads all 6 design proposals, all research, and this plan.

**Task:**

1. Rank all proposals on: M3/M11' potential, player experience, simplicity,
   signal reading quality, AI avoidance narrative quality, contraction
   trajectory, pool exhaustion risk.
2. Evaluate whether AI avoidance is genuinely "public information" behavior or
   a dressed-up Level 2+ mechanism. Where is the line?
3. Assess pool contraction trajectories: which refill schedules produce the
   right late-draft pool size? Is pool exhaustion a risk?
4. Assess oversampling at N = 8-12: does it feel natural? Is it necessary if
   contraction is aggressive enough? Could N = 4 (uniform) suffice?
5. Evaluate the interaction between AI avoidance, pool contraction, and
   oversampling: which combination achieves M3 >= 2.0 most naturally?
6. Evaluate whether the pool contraction approach is structurally equivalent
   to V9's invisible contraction. Is it more honest? Does it preserve the
   same concentration quality?
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
- Must implement declining refills per the specified schedule
- Must implement oversampled pack construction (draw N, rank by fitness, show
  best 4) with specified N value (4-12)
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
- **Pool contraction trajectory:** Track pool size, archetype density, and
  S/A count at each pick. Show the contraction curve. At what pick does the
  player's archetype become 40%+ of the pool?
- **Oversampling analysis:** What is the actual per-pack archetype density
  achieved at the specified N (4-12)? How does it compare to N = 4 (uniform)?
  Track S/A count remaining in pool to verify avoidance maintains supply.
- Pack quality distribution (p10/p25/p50/p75/p90 for picks 6+)
- Consecutive bad pack analysis
- **Pool composition trajectory:** Show how archetype distribution evolves
  as AIs avoid the player's lane
- 2 draft traces (committed player, signal reader) — including AI avoidance
  moments
- Comparison to V9 baseline and V11 results
- Self-assessment: Is AI avoidance + physical pool contraction + modest
  oversampling a viable replacement for V9's invisible contraction?

---

## Round 5: Final Synthesis (1 agent)

**Produces two files:**

### File 1: `docs/resonance/v12/final_report.md` (max 4000 words)

1. Unified comparison table (all V12 algorithms + V9/V10/V11 baselines)
2. The key question: **Can AI avoidance + physical pool contraction + modest
   oversampling replace V9's virtual contraction?**
3. AI avoidance analysis: which models work and which are surveillance?
4. Pool contraction analysis: which refill schedules produce the right
   trajectory? How does physical contraction compare to V9's invisible version?
5. Oversampling analysis: is N = 8 sufficient, or is N = 12 needed? Can
   N = 4 (uniform) work if contraction is aggressive enough?
6. Per-archetype convergence for top 3 algorithms
7. V12 vs V9 vs V10 vs V11 comparison
8. Recommendation tiers:
   - **Contraction Only:** Best design using AI avoidance + pool contraction
     with N = 4 (uniform packs). Establishes the physical contraction baseline.
   - **Standard:** Best overall design combining AI avoidance + pool contraction
     + modest oversampling (N = 8-12).
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
4. Structural findings about AI avoidance, pool contraction, and oversampling

---

## Agent Summary

| Round | Agents | Type | Description |
|-------|--------|------|-------------|
| 1 | 3 | Parallel | Research: avoidance, pool contraction, math |
| 2 | 6 | Parallel | Algorithm design |
| 3 | 1 + 6 responses | Sequential | Critic review + designer responses |
| 4 | 6 | Parallel | Simulation |
| 5 | 1 | Single | Final synthesis |
| **Total** | **~19** | | |

## Output Files

| File | Round | Description |
|------|-------|-------------|
| `research_ai_avoidance.md` | 1 | AI avoidance in competitive drafting |
| `research_pool_contraction.md` | 1 | Pool contraction via physical AI drafting |
| `research_concentration_math.md` | 1 | Math for contraction + avoidance + oversampling |
| `design_{1..6}.md` (x6) | 2 | Algorithm proposals |
| `critic_review.md` | 3 | Cross-proposal analysis |
| `sim_{1..6}.py` (x6) | 4 | Simulation code |
| `results_{1..6}.md` (x6) | 4 | Results |
| `final_report.md` | 5 | Recommendation + specification |
| `algorithm_overview.md` | 5 | Catalog of all algorithms |

All files in `docs/resonance/v12/`.

## Key Principles

1. **AI avoidance is the demand-side mechanism.** Every design must include
   AIs that detect and avoid the player's draft archetype using publicly
   available information. The strength, timing, and inference method vary, but
   the avoidance behavior is a central thesis. Avoidance preserves the
   player's S/A supply in the pool.
2. **Physical pool contraction is the primary concentration mechanism.** AIs
   physically take non-player cards. Declining refills mean the pool shrinks.
   With avoidance, the player's archetype accumulates as a larger fraction of
   the shrinking pool. This is V9's contraction achieved transparently through
   actual drafting. The refill schedule is the primary tuning parameter.
3. **Oversampling is a modest supplement (N = 8-12).** With pool contraction
   doing the heavy lifting, oversampling provides a light boost — drawing
   8-12 cards from a contracted pool and showing the best 4. This feels
   natural (a third to half of a 25-card pool) and honest (the player can
   browse the pool). N is NOT the primary mechanism.
4. **Public information is the honesty criterion.** AI avoidance must use only
   information available to all players. The player's visible resonance
   signature (computed from their drafted cards' symbols) is public. The
   player's internal strategy, commitment level, and card-by-card evaluation
   are private. AIs must not use private information.
5. **V9 is the fallback, not the enemy.** If no V12 face-up pool mechanism
   achieves M3 >= 2.0 independently, the recommendation should be V9's engine
   (non-face-up) enhanced with AI avoidance narrative. V12's contribution
   would then be the improved AI narrative (avoidance behavior makes AIs feel
   smarter and more realistic). Agent 4 explicitly tests this fallback.
6. **4-card packs are fixed.** The game uses "show 4, pick 1." Pack size is not
   a variable. Pack construction method is.
7. **AI avoidance must feel natural.** The player should perceive AI avoidance
   as rational opponent behavior, not as the game manipulating outcomes. "They
   noticed I'm drafting Storm and backed off" should feel like a competitive
   dynamic, not a designed safety net.
8. **Isolate the three mechanisms.** Agent 1 tests avoidance + contraction
   with N = 4 (no oversampling) to establish the pool-contraction baseline.
   Comparing Agent 1 (N = 4) to agents with N = 8 or N = 12 isolates the
   oversampling contribution. This calibration is essential.
9. **Transparency over stealth.** The face-up pool makes AI avoidance and pool
   contraction naturally visible — the player can browse the pool and see
   their archetype's cards persisting while other archetypes deplete and the
   pool shrinks. The player should be able to observe and exploit this.

## Recovery

Check which `docs/resonance/v12/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained.
