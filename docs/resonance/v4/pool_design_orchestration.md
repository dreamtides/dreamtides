# V4 Pool Design Orchestration Plan

A multi-step simulation analysis and refinement pipeline for figuring out how
to break up a 360-card draft pool for the Pack Widening v3 draft algorithm.

---

## The Resonance System

### Four Resonance Types

Dreamtides has four resonance types: **Ember**, **Stone**, **Tide**,
**Zephyr**.

Each card has between 0 and 3 **resonance symbols** printed on it. The symbols
are **ordered** — the leftmost symbol is the card's primary resonance. A card
with symbols [Tide, Zephyr] is a Tide-primary card with Zephyr secondary. This
is mechanically different from [Zephyr, Tide].

- ~10% of cards (~36 of 360) have 0 symbols (generic/neutral cards)
- The remaining ~90% have 1-3 symbols

### Eight Archetypes on a Circle

The 8 archetypes are arranged in a circle. Each resonance type sits between
its two core archetypes. Listed clockwise:

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

### CRITICAL: Resonance ≠ Archetype

**This distinction is the single most important concept in this document.**
Resonance types (Ember, Stone, Tide, Zephyr) are *visible card properties* —
the symbols printed on each card. Archetypes (Warriors, Storm, Blink, etc.)
are the 8 *strategic identities* that players build decks around.

**Each resonance is shared by 4 archetypes.** Tide is the primary resonance
for Warriors and Sacrifice, and the secondary resonance for Self-Mill and
Ramp. This means that a "Tide card" could belong to any of 4 different
archetypes. If a player commits to Warriors, seeing a Tide card in their pack
is NOT the same as seeing a Warriors card — roughly half of Tide cards belong
to archetypes that are bad for a Warriors deck.

**Consequence for evaluation:** All metrics must be evaluated by asking "does
this card have S/A-tier fitness for the player's specific target archetype?"
NOT "does this card share a resonance with the player's archetype?" Resonance-
level measurement inflates convergence numbers by ~2x because it counts cards
from the wrong archetypes as hits.

### What Cards Look Like

Cards belonging to an archetype typically carry symbols from that archetype's
resonances:

- A Warriors card might have symbols [Tide] or [Tide, Zephyr] or [Tide, Tide, Zephyr]
- A Storm card might have [Ember] or [Ember, Stone]
- A card bridging Warriors and Ramp might have [Tide, Zephyr] or [Zephyr, Tide]

---

## The Draft Algorithm: Pack Widening v3

**One-sentence player description:**

> "Each symbol you draft earns 1 matching token (primary earns 2); you may
> spend 3 tokens of one resonance to add 1 extra card with that primary
> resonance to the pack."

**Complete specification:**

1. Initialize 4 resonance token counters at 0.
2. To generate a pack:
   a. If the player has >= 3 tokens of any resonance, they may choose one
      resonance and spend 3 tokens.
   b. Draw 4 cards uniformly at random from the full pool.
   c. If the player spent tokens on resonance R, draw 1 additional card
      randomly from cards whose primary resonance is R. Pack is now 5 cards.
   d. Player picks 1 card from the pack.
3. After picking: add 2 tokens for the card's primary resonance, 1 for each
   secondary/tertiary resonance. Generic cards (0 symbols) earn no tokens.
4. Repeat from step 2.

**Parameters:**
- Spend cost: 3 tokens
- Bonus cards per spend: 1
- Primary symbol weight: 2 tokens
- Secondary/tertiary weight: 1 token each
- Base pack size: 4 cards (5 when spending)

**Key algorithm properties that affect pool design:**

- **Token earn rate:** A committed player earns ~3 tokens/pick in their
  primary resonance (from mostly 2-symbol cards with primary=2, secondary=1).
  This means they can spend (cost 3) roughly every pick once committed.
- **Bonus card pool:** The bonus card is drawn from all cards whose primary
  resonance matches the spent resonance. This pool spans 2 primary archetypes
  and 2 secondary archetypes, so ~50% of bonus cards are S/A for the player's
  specific archetype.
- **Base pack is always random:** The 4 base cards are drawn uniformly from
  the full 360-card pool. Only the 5th bonus card is resonance-filtered.
- **Spending is optional:** Non-spend packs are fully random (4 cards). This
  creates natural variance — the algorithm's convergence power comes entirely
  from the bonus card.

### Symbol Counting Rules

When counting tokens earned from a drafted card:
- **Primary symbol** (first/leftmost on card): earns **2** tokens
- **Secondary symbol** (second position): earns **1** token
- **Tertiary symbol** (third position): earns **1** token
- **Generic cards** (0 symbols): earn **0** tokens

This makes the ordering of symbols matter mechanically.

---

## Fixed Parameters

- **360 unique cards** in the draft pool
- **4 cards per pack** (5 when spending), pick 1
- **30 picks** per quest
- **4 resonance types**: Ember, Stone, Tide, Zephyr
- **8 archetypes** arranged on a circle
- **0-3 ordered resonance symbols** per card
- **~10% of cards** have 0 symbols (generic)
- **Symbol order matters**: primary symbol earns more tokens

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

The draft algorithm uses only **visible card properties** — `symbols`,
`rarity`, `power`. The `archetype_fitness` scores are for evaluation only.

### Fitness Tier Definitions

For each card, fitness is assigned per archetype:

- **S-tier:** The card's home archetype (the archetype it was designed for)
- **A-tier:** The adjacent archetype sharing the card's primary resonance
- **B-tier:** Archetypes sharing the card's secondary resonance; also generic
  cards in all archetypes
- **C-tier / F-tier:** Distant archetypes with no shared resonance

Example: A Warriors card [Tide, Zephyr] is S-tier for Warriors (home), A-tier
for Sacrifice (adjacent, shares Tide primary), B-tier for Ramp (shares Zephyr
secondary), and C/F for distant archetypes like Storm or Self-Discard.

### Simulated Player Strategies

- **Archetype-committed:** Picks cards with highest fitness in their strongest
  archetype. Commits around pick 5-6. Spends tokens on primary resonance when
  able.
- **Power-chaser:** Picks the highest raw power card regardless of archetype.
  Accumulates scattered tokens, spends opportunistically.
- **Signal-reader:** Evaluates which resonance/archetype seems most available
  and drafts toward the open archetype. Spends tokens based on observed pack
  composition.

---

## Design Goals

Ranked by priority:

1. **Simple.** Explainable to players in complete technical detail in one
   sentence.
2. **Not on rails.** The player should not be forced into one archetype or
   have only 1 real choice per pack.
3. **No forced decks.** The player should not be able to force the same deck
   every time they play.
4. **Flexible archetypes.** It should be possible to build decks outside the
   core archetypes, or combine 2 archetypes.
5. **Convergent.** A committed player should see a minimum of 2 cards that are
   actually good for their specific archetype (S/A-tier) in most packs after
   pick 6.
6. **Splashable.** You should see around 1 card from outside your archetype in
   most draft picks.
7. **Open-ended early.** In the first ~5 picks, you should see a variety of
   cards from different archetypes.
8. **Signal reading.** There should be a moderate benefit to figuring out which
   archetype is over-represented in the starting pool.

### Measurable Targets

**ALL metrics must be measured at the ARCHETYPE level, not the resonance
level.**

| Metric | Target |
|--------|--------|
| Picks 1-5: unique archetypes with S/A cards per pack | >= 3 of 8 on average |
| Picks 1-5: S/A cards for player's emerging archetype per pack | <= 2 of 4 |
| Picks 6+: S/A cards for committed archetype per pack | >= 2 of 4 on average |
| Picks 6+: off-archetype (C/F) cards per pack | >= 0.5 of 4 on average |
| Convergence pick (player regularly sees 2+ archetype S/A cards) | Pick 5-8 |
| Deck archetype concentration (committed player) | 60-90% S/A-tier cards |
| Run-to-run variety (same starting conditions) | < 40% card overlap |
| Archetype frequency across runs | No archetype > 20% or < 5% |

### Variance Target

| Metric | Target |
|--------|--------|
| StdDev of S/A cards per pack (picks 6+) | >= 0.8 |

An algorithm that always delivers exactly 2.0 S/A cards (stddev ~0) fails
this. An algorithm that averages 2.0 but ranges from 0-4 (stddev ~1.0) passes.
The goal is *natural variance around a good average*.

---

## Important: Archetype-Level Evaluation

All metrics must be measured at the **archetype level**, not the resonance
level. A card matching your resonance is NOT the same as a card fitting your
archetype — a resonance is shared by 4 archetypes. Specifically:

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

All agents share the same Pack Widening v3 algorithm implementation — only the
pool construction and evaluation vary.

### Agent 1: Symbol Count Distribution

**Question:** What ratio of 0/1/2/3-symbol cards produces the best draft
experience with Pack Widening v3?

**Method:**
- Test at least 8 distributions from extreme to balanced:
  - All 1-symbol (100/0/0)
  - Heavy 1-symbol (70/20/10)
  - Moderate 1-symbol (50/35/15)
  - Balanced (33/34/33)
  - The V4-recommended default (20/55/25)
  - Heavy 2-symbol (10/80/10)
  - Heavy 3-symbol (10/30/60)
  - All 3-symbol (0/0/100)
- For each, measure:
  - **Token accumulation rate:** Average tokens earned per pick, tokens in
    primary resonance per pick for committed player
  - **First spend timing:** What pick can a committed player first afford to
    spend 3 tokens?
  - **Spend frequency:** What fraction of picks 6+ involve spending?
  - S/A cards per pack at picks 5, 10, 15, 20, 25, 30 (the convergence curve)
  - **Spend vs non-spend pack quality:** Average S/A on spend packs vs
    non-spend packs. The gap should be meaningful but not extreme.
  - **SA Trend** — whether pack quality improves, plateaus, or declines over
    the draft

**Key insight to look for:** More symbols per card means faster token
accumulation. With cost 3, a committed player drafting mostly 3-symbol cards
(earning 4 tokens/pick) can spend every pick from pick 2 onward — this
eliminates the save/spend decision entirely (always-spend is dominant). A
committed player drafting mostly 1-symbol cards (earning 2 tokens/pick) can
only spend every other pick — creating genuine save/spend rhythm. The symbol
distribution directly controls whether the economic decision is interesting
or trivial.

**Output:** `pool_results_1.md` (max 1000 words) + `pool_sim_1.py`

### Agent 2: Rarity System

**Question:** How should rarity interact with Pack Widening v3 and resonance
symbols?

**Method:**
- Test at least 5 rarity models:
  - Flat rarity (no power differences)
  - Standard TCG (180C/100U/60R/20L, power scales)
  - Roguelike-skewed (more rares/legendaries)
  - Rarity-symbol correlation (rares have more symbols)
  - Inverse correlation (rares have fewer symbols)
- For each, measure:
  - All standard archetype-level targets
  - **Draft tension rate:** How often the player faces "strong off-archetype
    rare vs weak on-archetype common" decisions, measured separately for
    spend packs (where the bonus card adds an on-archetype option) and
    non-spend packs
  - Power variance across runs (replayability)
  - **Rarity-token interaction:** Do rares with more symbols cause token
    flooding? Do rares with fewer symbols slow convergence?
  - Whether rarity interacts with the spend decision or is orthogonal

**Key insight to look for:** With Pack Widening, rarity is likely orthogonal
to the convergence mechanism because the token system cares only about symbol
count, not card quality. But rarity-symbol correlation could create a
meaningful interaction: if rares have 3 symbols (earning 4 tokens), drafting a
rare accelerates spending — creating a "rare = faster convergence" feedback
loop that may be desirable or degenerate depending on magnitude.

**Output:** `pool_results_2.md` (max 1000 words) + `pool_sim_2.py`

### Agent 3: Archetype Breakdown

**Question:** How should cards be distributed across archetypes, and how many
generic/bridge cards should exist?

**Method:**
- Test at least 5 breakdown models:
  - Equal cards per archetype + small generic pool (~10%)
  - Equal + large generic pool (~25%)
  - Equal + explicit bridge card category (cards assigned to two archetypes,
    carrying symbols from both)
  - Asymmetric archetype sizes (some deeper than others)
  - Mono-symbol only (all archetype cards have just [Primary])
- For each, measure:
  - All standard archetype-level targets
  - S-tier vs A-tier ratio in drafted decks (home archetype vs adjacent)
  - **Bridge strategy viability:** A player committing to 2 adjacent
    archetypes — how often can they find S/A cards for BOTH archetypes? With
    Pack Widening, this means spending tokens on a resonance shared by both
    archetypes.
  - Whether generics dilute convergence or improve flexibility. With Pack
    Widening, generics earn no tokens — too many generics slow token
    accumulation and delay spending.
  - Per-archetype frequency balance
  - **Bonus card hit rate:** When spending on resonance R, what fraction of
    bonus cards are S/A for the player's specific archetype? This varies with
    how many cards of each archetype have R as primary.

**Key insight to look for:** Pack Widening's bonus card is drawn from the
primary-resonance pool, which contains cards from 2 archetypes (the two that
have that resonance as primary) plus cards from 2 more (that have it as
secondary). The archetype breakdown determines what fraction of this pool is
S/A for the player. With equal distribution, ~50% of bonus cards are S/A. With
more bridge cards or skewed distributions, this fraction shifts. The
archetype breakdown also affects whether generics (which earn no tokens) slow
convergence enough to matter.

**Output:** `pool_results_3.md` (max 1000 words) + `pool_sim_3.py`

### Agent 4: Symbol Pattern Composition

**Question:** What specific symbol patterns should cards have, and how do
different patterns affect the token economy and draft decisions?

**Method:**
- Enumerate all mechanically distinct symbol patterns for a card in a given
  archetype (e.g., for Warriors = Tide/Zephyr):
  - 1-sym: [P], [S]
  - 2-sym: [P,S], [P,P], [S,P], [P,Other]
  - 3-sym: [P,P,S], [P,S,S], [P,S,Other], [P,Other,Other], etc.
- Test at least 5 pattern compositions from uniform to maximally varied
- For each, measure:
  - **Token profile:** How many tokens flow to which resonances from each
    pattern. E.g., [P,S] gives 2 primary + 1 secondary tokens. [P,P] gives
    2 primary + 1 primary = 3 primary tokens. [P,Other] gives 2 primary +
    1 off-resonance tokens.
  - **Genuine choice rate:** How often a pack contains 2+ S/A cards with
    different token profiles, forcing the player to evaluate which card
    best serves their token economy. With Pack Widening, this matters more
    than with other algorithms because token allocation directly affects
    future spending options.
  - **Accidental token scatter:** How often does a committed player earn
    tokens in resonances they don't want to spend? High scatter means more
    tokens are "wasted" — relevant for cost 3 where every token matters.
  - **Bridge-spend viability:** Cards with [P,Other] patterns let a player
    earn tokens in two resonances simultaneously, enabling multi-resonance
    spending strategies. How viable is this path?
  - Unwanted/accidental commitment rate

**Key insight to look for:** Pattern variety is critical for Pack Widening
because different patterns produce different token profiles. A player choosing
between [Tide, Zephyr] and [Tide, Tide] is making a real decision: the first
spreads tokens across two resonances (supporting pivot or bridge strategies),
while the second concentrates tokens in Tide (faster spending but less
flexibility). If all cards have the same pattern (e.g., all [P,S]), every pick
generates the same token distribution and the economic decision becomes
autopilot.

**Output:** `pool_results_4.md` (max 1000 words) + `pool_sim_4.py`

### Agent 5: Algorithm Parameter Tuning

**Question:** How should Pack Widening v3's parameters be tuned relative to
the pool design, and what is the ideal progression curve?

**Method:**
- Test a matrix of parameter values:
  - **Spend cost:** 2, 3, 4, 5
  - **Bonus card count:** 1, 2
  - **Primary token weight:** 1, 2, 3
- Crossed with 3 symbol distributions:
  - Heavy 1-symbol (50/35/15)
  - Default (20/55/25)
  - Heavy 3-symbol (10/30/60)
- For each, measure:
  - **First spend pick:** What pick can a committed player first afford to
    spend?
  - **Spend frequency at picks 6+:** What fraction of picks involve spending?
  - **Always-spend degeneracy:** Is "always spend whenever possible" a
    dominant strategy, or does saving sometimes produce better outcomes?
  - S/A per pack at each pick number (the convergence curve)
  - The full measurable targets table
  - **Three-act draft arc:** Does the draft have clear exploration (picks 1-5),
    commitment (picks 6-15), and refinement (picks 16-30) phases? Or does the
    algorithm create a monotone ramp / flat plateau / decline?
  - Decision quality / meaningful choice frequency

**Key insight to look for:** The interaction between spend cost and symbol
distribution is the critical finding. Cost 3 + heavy 3-symbol cards = always-
spend (degenerate). Cost 3 + heavy 1-symbol cards = spend every other pick
(interesting). Cost 4 + default distribution = spend every ~1.5 picks (slower
but more tension). The right combination creates a satisfying three-act arc:
exploration (no spending, random packs), commitment (start spending, enhanced
packs), refinement (optimized spending, targeted packs).

**Output:** `pool_results_5.md` (max 1000 words) + `pool_sim_5.py`

---

## Round 2: Synthesis (1 agent, background)

A single agent reads all 5 results documents and reconciles their findings.

**Tensions to expect and resolve:**

1. **Token rate vs decision quality:** Agent 1 will want fewer symbols per
   card (slower token accumulation, more save/spend decisions). Agent 4 will
   want pattern variety (which requires multi-symbol cards). Agent 5's
   parameter tuning findings can resolve this — higher spend cost can
   compensate for faster token accumulation from multi-symbol cards, preserving
   genuine decisions while allowing pattern variety.

2. **Simplicity vs richness:** Fewer pattern types are simpler to design but
   create autopilot token allocation. More pattern types create richer
   spend/save decisions but require more card design effort.

3. **Bonus card hit rate vs pool diversity:** Agent 3 can increase the bonus
   card S/A hit rate by concentrating more cards in the player's primary
   resonance — but this reduces pool diversity and may affect early-draft
   openness.

4. **Agent-specific vs universal findings:** Some findings will be specific to
   Pack Widening (e.g., optimal spend cost relative to token rate). Others
   will be universal (e.g., rarity is orthogonal, equal archetype sizes are
   better).

**Method:**
- Build a simulation implementing the reconciled pool design.
- Test it with Pack Widening v3 under 2-3 parameter configurations.
- Compare against the V4 default pool to quantify improvements.
- Simulate all 3 player strategies (committed, power-chaser, signal-reader).

**Output:** `pool_design_final.md` (max 1500 words) + `pool_synthesis_sim.py`

The final document should include:
1. **Complete pool specification** — exact card counts per archetype, pattern
   breakdown within each archetype, rarity distribution, generic card count,
   and algorithm parameters. Specific enough that a card designer could build
   the set from it.
2. **How each agent's finding was incorporated** (or why it was overridden).
3. **Tensions and how they were resolved.**
4. **Before/after comparison** on key metrics (against the V4 default pool).
5. **Token economy analysis** — average tokens earned per pick, spend
   frequency, save/spend decision quality, and the three-act draft arc.
6. **Open questions for playtesting.**

---

## Agent Summary

| Round | Agents | Type | Focus |
|-------|--------|------|-------|
| 1 | 5 | Parallel background | Symbol ratios, rarity, archetype breakdown, symbol patterns, parameter tuning |
| 2 | 1 | Background | Synthesis and reconciliation |
| **Total** | **6** | | |

## Output Files

All output files are in `docs/resonance/v4/`.

| File | Round | Description |
|------|-------|-------------|
| `pool_sim_{1..5}.py` (x5) | 1 | Simulation code per investigation |
| `pool_results_{1..5}.md` (x5) | 1 | Results per investigation |
| `pool_synthesis_sim.py` | 2 | Reconciled simulation |
| `pool_design_final.md` | 2 | Complete pool specification |

## Key Principles

1. **Archetype-level evaluation is non-negotiable.** Never measure resonance
   matching as a proxy for archetype fitness. A resonance is shared by 4
   archetypes; only S/A-tier fitness tells you whether a card actually serves
   the player's strategy.

2. **Decision quality matters more than metric optimization.** A pool that
   produces perfect convergence numbers but trivial spend/save decisions (0%
   genuine choice) is worse than one with slightly lower convergence but
   meaningful economic tension. With Pack Widening, the key decision quality
   metric is whether "always spend" is dominant or whether saving creates
   better outcomes in identifiable situations.

3. **Parameters and pool design interact.** You cannot design the pool without
   knowing the spend cost, and you cannot tune the spend cost without knowing
   the token earn rate (which depends on symbol distribution). Agent 5 exists
   specifically to map this interaction.

4. **Universal findings vs algorithm-specific findings.** Some results
   generalize across all draft algorithms:
   - Rarity is almost always orthogonal to the convergence mechanism.
   - Pattern variety is almost always necessary for decision quality.
   - Equal archetype sizes are almost always better than asymmetric.
   - ~10% generic cards is a robust default.

   Other results are specific to Pack Widening:
   - Symbol distribution controls token earn rate, which controls spend
     frequency, which controls whether the economic decision is interesting.
   - The spend cost / token rate ratio is the master parameter: it determines
     how often the player can spend and whether saving is ever optimal.
   - Generics earn no tokens — too many generics slow token accumulation.

5. **The convergence curve is the most informative metric.** A single "late
   S/A" number hides whether packs improve steadily, peak and decline, or
   plateau early. Always plot S/A per pack at each pick number. The ideal
   curve rises through the first third of the draft and holds steady or
   continues rising — never declining.

6. **The token economy curve is equally important.** Plot token balance over
   time. The ideal pattern: tokens accumulate during exploration (picks 1-5),
   spending begins during commitment (picks 6-10), and a steady spend rhythm
   develops during refinement (picks 11-30). If tokens accumulate faster than
   they can be spent, the cost is too low. If tokens never accumulate enough
   to spend, the cost is too high.

7. **Simulate honestly.** Run 1000+ drafts per configuration. Use actual
   simulation output in results documents. Do not fabricate or estimate
   numbers.

## Recovery

Check which `pool_results_*.md` and `pool_sim_*.py` files exist to determine
progress. Round 2 depends only on Round 1 outputs being complete.
