# Resonance Draft System V5 — Orchestration Plan

## Lessons from V3 and V4

V3 explored five mechanistic domains (accumulation, structural, threshold,
reactive, pool manipulation) and recommended **Lane Locking** — a threshold
system where drafting 3 symbols of a resonance permanently locks a pack slot to
that resonance, with a second lock at 8. Lane Locking achieved 2.72 S/A cards
per pack at archetype level, converging at pick 6.1 with perfect archetype
balance. Its weaknesses: too mechanical (deterministic slot assignment),
permanently on rails after commitment, and 99% deck concentration.

V4 explored five new domains (rejection, soft probabilistic, economic, phantom
drafter, filtered sampling) and recommended **Pack Widening v3** — a system
where drafted symbols earn resonance tokens that the player can spend to add
bonus resonance-matched cards to their pack. Pack Widening crossed the 2.0 S/A
threshold (projected 2.3-2.5 at cost 3/bonus 1) with better variance, splash,
and deck diversity than Lane Locking.

**V4's key structural finding:** Probabilistic resonance-based mechanisms
(weighting, filtering, exile, phantoms) are structurally capped at 1.26-1.74
S/A cards per pack because each resonance is shared by 4 archetypes — roughly
50% of resonance-matched cards belong to the wrong archetype. Only mechanisms
that ADD targeted cards to packs (Pack Widening) or deterministically PLACE them
in specific slots (Lane Locking) cross the 2.0 threshold. This is a
mathematical limit, not a tuning problem.

**What V5 must solve:** Pack Widening requires the player to make spending
decisions before each pack — choosing when and which resonance to spend tokens
on. This is too fiddly. Players just want to pick cards. Lane Locking requires
no decisions beyond drafting but feels like a vending machine. V5 must find
algorithms that:

1. Require NO player action beyond picking 1 card from a pack
2. Cross the 2.0 S/A threshold at archetype level
3. Feel like natural variation, not mechanical delivery
4. Maintain pack-to-pack variance

**V5's core question:** Can we achieve Pack Widening's convergence with Lane
Locking's zero-decision interface — while avoiding Lane Locking's mechanical
feel?

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

**CRITICAL: Ordered resonance pairs ARE archetype identifiers.** The ordered
pair (primary, secondary) maps much more precisely to archetypes than a single
resonance:
- [Tide, Zephyr] → Warriors (the archetype with Tide primary, Zephyr secondary)
- [Tide, Stone] → Sacrifice (Tide primary, Stone secondary)
- [Zephyr, Tide] → Ramp (Zephyr primary, Tide secondary)

For cards with 2+ symbols, the ordered pair uniquely identifies the home
archetype. For 1-symbol cards, the primary resonance identifies 2 candidate
archetypes (the two archetypes sharing that primary). For 0-symbol cards
(generics), no archetype is identified.

This means **pair-aware algorithms can achieve ~100% archetype precision for
the 75% of cards that have 2+ symbols**, compared to ~50% precision for
single-resonance algorithms. This is a potential breakthrough for V5: it may
allow probabilistic approaches to cross the 2.0 S/A threshold that V4 proved
was unreachable with single-resonance matching.

**Consequence for algorithm design:** V5 agents should seriously consider
whether their algorithms can use ordered resonance pairs as their matching
unit rather than single resonances. This is not required — single-resonance
algorithms are fine if they cross 2.0 through other means (adding cards,
etc.). But pair-based matching is a new tool available to V5 that V3 and V4
did not explore.

### What Cards Look Like

Cards belonging to an archetype typically carry symbols from that archetype's
resonances:

- A Warriors card might have symbols [Tide] or [Tide, Zephyr] or [Tide, Tide, Zephyr]
- A Storm card might have [Ember] or [Ember, Stone]
- A card bridging Warriors and Ramp might have [Tide, Zephyr] or [Zephyr, Tide]

Most decks should be firmly centered in one archetype. A Warriors deck will
contain mostly Warriors cards (which happen to carry Tide and Zephyr symbols),
plus some cards from adjacent archetypes like Sacrifice or Ramp. A Storm deck
will contain mostly Storm cards (which carry Ember and Stone symbols), plus
some adjacent archetype cards.

---

## The Problem

Design a draft algorithm that requires NO player action beyond picking 1 card
from a pack of 4. The algorithm must use resonance symbols and/or other visible
card properties to construct packs that naturally converge toward the player's
archetype while maintaining variance.

**The hard constraint:** The player's only action each pick is choosing 1 card
from the presented pack. There is no spending, no mode selection, no rerolling,
no resource management, no "before pack" decisions. Everything the algorithm
does must be automatic and passive, triggered solely by the player's card
selections.

**What V3 and V4 established:**
- Pure probabilistic approaches (weighting, filtering) cap at ~1.7 S/A due to
  resonance-archetype dilution (V4 finding)
- You must either ADD targeted cards to packs or deterministically PLACE them in
  slots to cross 2.0 S/A (V4 finding)
- Deterministic slot assignment (Lane Locking) crosses 2.0 but feels mechanical
  (V3/V4 finding)
- Active resource spending (Pack Widening) crosses 2.0 but requires player
  decisions (V4 finding, ruled out for V5)
- Ordered resonance pairs map to archetypes with ~100% precision for 2+ symbol
  cards, compared to ~50% for single resonances (V5 insight, untested)

**V5's design space:** Algorithms that automatically (without player
decisions) achieve 2.0+ S/A convergence while feeling natural. The available
mechanisms include:
- Automatically adding bonus cards to packs (passive widening)
- Probabilistically targeting pack slots (soft locking)
- Evolving the pool composition (seeding)
- Using pair-based matching for higher archetype precision
- Conditioning pack generation on the current pack's random composition

### Fixed Parameters

- **360 unique cards** in the draft pool
- **4 cards per pack** baseline, pick 1 (algorithms may increase pack size
  automatically)
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
   YES "each symbol you draft adds a matching token to a bag; when a token
   type reaches 3, your next pack gets a bonus card of that type and the count
   resets" — that's a real algorithm.
2. **No extra actions.** The player's only action is picking 1 card from the
   pack. Everything else is automatic. No spending, no mode selection, no
   rerolling. THIS IS NON-NEGOTIABLE for V5.
3. **Not on rails.** The player should not be forced into one archetype or have
   only 1 real choice per pack.
4. **No forced decks.** The player should not be able to force the same deck
   every time they play.
5. **Flexible archetypes.** It should be possible to build decks outside the
   core archetypes, or combine 2 archetypes.
6. **Convergent.** If you HAVE committed to an archetype (around pick 6 on
   average), you should see a minimum of 2 cards that are actually good for
   that specific archetype (S/A-tier) in most of your draft picks. "Good for
   Warriors" means the card is S/A-tier for Warriors — not merely "has Tide
   symbols."
7. **Splashable.** You should see around 1 card from outside your archetype in
   most draft picks.
8. **Open-ended early.** In the first ~5 picks, you should see a variety of
   cards from different archetypes.
9. **Signal reading.** There should be a moderate benefit to figuring out which
   archetype is over-represented in the starting pool.

### Measurable Targets

**CRITICAL: All metrics must be measured at the ARCHETYPE level, not the
resonance level.** V3 initially measured at resonance level and had to issue
corrections because resonance-level metrics inflate convergence numbers — a
resonance like Tide is shared by 4 archetypes, so roughly half of
resonance-matched cards are S/A for the *wrong* archetype. Every simulation
must evaluate whether cards are good for the player's *specific target
archetype*, not merely whether they share a resonance.

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

"S/A cards for committed archetype" means the card has S-tier or A-tier fitness
for the player's specific target archetype, not merely that it shares a
resonance with that archetype. **Example:** A player committed to Warriors
sees a pack with [Tide/Zephyr Warriors card, Tide/Stone Sacrifice card, Ember
Storm card, generic card]. The Warriors card and possibly the generic card are
S/A for Warriors. The Sacrifice card has Tide symbols but is C/F-tier for
Warriors — it does NOT count as an archetype hit despite sharing a resonance.

### Variance Target

| Metric | Target |
|--------|--------|
| StdDev of S/A cards per pack (picks 6+) | >= 0.8 |

An algorithm that always delivers exactly 2.0 S/A cards (stddev ~0) fails
this. An algorithm that averages 2.0 but ranges from 0-4 (stddev ~1.0) passes.
The goal is *natural variance around a good average*, not *consistent
mechanical delivery*.

### Simplicity Test

Every proposed algorithm must pass this test: **Can you write the complete
algorithm as a one-sentence instruction to a programmer?** If the sentence
includes words like "nudges," "adapts," "responds to," or "considers" — it
fails. The sentence must describe concrete operations on concrete data.

Additionally, the one-sentence description must NOT include any player
decisions. If it says "you may spend," "you can choose to," or "before seeing
a pack, decide whether to" — it violates the V5 no-extra-actions constraint.

Examples of FAILING descriptions:
- "The system nudges you toward your chosen archetype after you commit"
- "You may spend 3 tokens of one resonance to add a bonus card" (player
  decision)
- "Before each pack, choose whether to activate your resonance bonus" (player
  decision)

Examples of PASSING descriptions:
- "Each symbol you draft adds a matching token; when a token type reaches 3,
  your next pack includes a bonus card of that type and the count resets"
- "When you draft a card, 2 random cards sharing its primary resonance are
  shuffled into the pool from a reserve"
- "Each pack slot has a chance equal to your top resonance's symbols divided
  by 15 of showing a card from that resonance"

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

**Preferred separation:** Ideally, the draft algorithm should use **visible
card properties** — `symbols` are the primary candidate, but `rarity`,
`power`, and other visible attributes are fair game too. The
`archetype_fitness` scores are primarily for evaluation. However, this is a
preference, not a hard constraint. If an algorithm that uses archetype fitness
directly (e.g., the system knows a card is "a Warriors card" and uses that
information) produces dramatically better results, that's worth exploring.
Agents should note when their algorithm relies on hidden fitness data vs.
visible properties only.

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
players.

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

### Pair Counting Rules (New for V5)

When counting a player's resonance PAIRS (for algorithms that use pair
matching):
- A 2+ symbol card contributes one pair: (primary, secondary). E.g., a card
  with [Tide, Zephyr, Tide] contributes the pair (Tide, Zephyr).
- A 1-symbol card contributes NO pairs (no secondary symbol to form a pair).
- A 0-symbol card contributes NO pairs.

The player's pair profile is a dictionary mapping each of the 12 possible
ordered pairs (4 x 3 combinations, since primary ≠ secondary) to a count.
E.g., {(Tide, Zephyr): 5, (Zephyr, Tide): 2, (Ember, Stone): 1, ...}.

### Simulated Player Strategies

- **Archetype-committed:** Picks cards with highest fitness in their strongest
  archetype. Commits around pick 5-6.
- **Power-chaser:** Picks the highest raw power card regardless of archetype.
- **Signal-reader:** Evaluates which resonance/archetype seems most available
  and drafts toward the open archetype.

---

## The Five Investigation Areas

Each agent explores a fundamentally different **approach to automatic
convergence** — ways to cross the 2.0 S/A threshold without requiring player
decisions beyond picking cards. These approaches build on V3/V4 findings and
are chosen to cover the design space of "passive algorithms that add or target
cards effectively."

Within their domain, each agent must **brainstorm 5 concrete algorithms**,
analyze their tradeoffs, and **champion one** to develop further.

**V3 explored these domains (see V3 final report for results):**
- Accumulation-based (bag building, running counters, resonance meters)
- Structural/guaranteed (fixed pack composition rules, one-per-resonance)
- Threshold/progression (discrete state changes at milestones, lane locking)
- Reactive/immediate (only recent picks matter, sliding windows)
- Pool manipulation (adding/removing cards from the available pool)

**V4 explored these domains (see V4 final report for results):**
- Rejection & passing (passed cards affect future packs)
- Soft probabilistic influence (weighted sampling, sqrt affinity)
- Economic & resource (token earning/spending, pack widening)
- Phantom drafter / competitive scarcity (phantom AI drafters deplete pool)
- Curated randomness / filtered sampling (oversample then filter)

**V5 agents should draw on V3 and V4 results but must satisfy the V5
constraint (no player decisions).** An agent may propose a modified version of
a V3/V4 algorithm (e.g., "Pack Widening but with automatic spending") as long
as the modification is substantive and the one-sentence description requires
no player action. Simply removing player choice from an existing algorithm
without thought does NOT count — the agent must analyze how the removal
changes behavior and propose appropriate parameter adjustments.

### Anti-Pattern: Player Decisions

**No V5 algorithm may require player decisions beyond picking 1 card from the
pack.** The following patterns are banned:
- "You may spend tokens to..."
- "Before seeing a pack, choose whether to..."
- "The player decides which resonance to invest in"
- Any mechanism that asks the player to do something other than pick a card

The algorithm must be fully automatic. The player's only input is which card
they select from each pack.

### Domain 1: Passive Resonance Bonus (Auto-Widening)

**Core idea:** Take Pack Widening's proven bonus-card mechanism (V4's
recommended algorithm) and make spending automatic. Resonance tokens accumulate
passively from drafted symbols. When a threshold is reached, a bonus card is
automatically added to the next pack and tokens are deducted. No player choice
about when or what to spend — it happens whenever possible.

**What this inherits from V4:** Pack Widening's core insight — adding
resonance-matched bonus cards to packs is the only probabilistic-compatible
mechanism that crosses 2.0 S/A. The bonus card pool (all cards with matching
primary resonance) provides natural variance because only ~50% of those cards
are S/A for the specific archetype.

**What changes from V4:** The player has no spending decision. The algorithm
auto-spends whenever tokens are available. The key design questions become:
which resonance to auto-spend on (always highest? round-robin? most recent?),
how to prevent over-concentration without the player's strategic save/spend
timing, and whether the removal of player agency hurts or helps the feel.

**Example (one of many possibilities):** "Each symbol you draft adds a matching
token (primary=2); when any resonance reaches 3 tokens, your next pack gets a
bonus card of that resonance and 3 tokens are deducted." — But there are many
other auto-widening designs: spending based on highest resonance, spending a
fixed fraction of all tokens, per-pack probability of bonus based on token
count, etc.

**Why this domain is promising:** Pack Widening already crosses 2.0 S/A. The
question is whether removing player agency degrades the experience or just
simplifies it. If auto-spending on the highest resonance mimics what a smart
player would do anyway, this might be strictly better (same convergence, less
cognitive load).

### Domain 2: Probabilistic Slot Targeting (Soft Lane Locking)

**Core idea:** Take Lane Locking's proven slot-targeting mechanism (V3's
recommended algorithm) and make it probabilistic instead of deterministic.
Instead of permanently locking a slot to a resonance at threshold 3, each slot
independently has a *probability* of showing a resonance-matched card that
scales with the player's symbol count. No binary state transitions — just
escalating probabilities.

**What this inherits from V3:** Lane Locking's insight — targeting specific
pack slots to a resonance is effective because it guarantees resonance-matched
cards, ~75% of which are S/A for the committed archetype. The slot-targeting
mechanism provides strong convergence.

**What changes from V3:** No permanent locks. No threshold discontinuities.
Instead, a smooth probability curve. At 0 symbols of Tide, each slot has 0%
chance of being Tide-targeted. At 3 symbols, maybe 30%. At 10 symbols, maybe
70%. Never 100%. This creates natural variance (sometimes you get 3 targeted
slots, sometimes 0) while converging on average.

**Example (one of many possibilities):** "Each pack slot independently becomes
a resonance slot with probability min(your top resonance's weighted symbols /
15, 0.75); resonance slots show a random card of that resonance." — But there
are many other probabilistic targeting designs: per-resonance probabilities
instead of top-resonance-only, diminishing returns curves, probability based on
ratio of symbols rather than absolute count, etc.

**Why this domain is promising:** Lane Locking's convergence is proven (2.72
S/A). A probabilistic version should converge somewhat lower (because not every
slot fires) but potentially still above 2.0 — while gaining the natural
variance that Lane Locking lacked (stddev 0.84, barely passing).

### Domain 3: Pool Evolution (Seeding)

**Core idea:** Instead of manipulating pack construction, change the pool
itself. When the player drafts a card, similar cards are automatically added
to the pool from a reserve, increasing the natural density of the player's
preferred resonance. Pack generation remains trivially simple (draw 4 random
cards from the pool), but the pool's composition drifts toward the player's
archetype over time.

**What this inherits from V3/V4:** V3's Resonance Swap explored pool
manipulation (adding/removing cards based on drafting) but only achieved 1.58
S/A — below the 2.0 threshold. V4's key finding was that probabilistic
approaches cap at ~1.7 because of resonance-archetype dilution. Pool seeding
may break through this ceiling by changing the pool MORE aggressively than V3
attempted — e.g., adding 3-5 cards per draft rather than swapping 3-for-3.

**What this brings new to V5:** V3's Resonance Swap was conservative (swap
3 in / 3 out, net zero pool size change). V5 should explore more aggressive
seeding that nets *adds* cards to the pool, growing the density of the
player's resonance. The pool grows from 360 to 400+ over 30 picks, but since
the additions are resonance-concentrated, the effective density of on-archetype
cards increases much faster than dilution from off-archetype cards.

**Example (one of many possibilities):** "When you draft a card, 3 random
cards sharing its primary resonance are added to the pool from a reserve." —
But there are many other seeding designs: pair-based seeding (add cards matching
the drafted card's ordered pair), variable seeding rate based on commitment
strength, seeding with removal of off-resonance cards, etc.

**Why this domain is promising:** Pool seeding is invisible and natural. The
player just notices more good cards appearing over time. No slot assignment, no
UI state to track, no visible mechanism. If seeding rates are high enough to
cross 2.0, this could be the most elegant V5 solution.

### Domain 4: Pair-Based Pack Construction

**Core idea:** Use ordered resonance pairs (primary, secondary) rather than
individual resonances as the matching unit for pack construction. Since ordered
pairs uniquely identify archetypes for 2+ symbol cards (~75% of the pool),
pair-based algorithms can achieve ~100% archetype precision — compared to ~50%
for single-resonance algorithms. This may allow approaches that were
structurally capped at 1.7 S/A in V4 to cross 2.0.

**What this brings new:** V3 and V4 always matched on single resonances (Tide,
Ember, etc.). V5 is the first time ordered pairs are considered as a matching
unit. The insight: a card with [Tide, Zephyr] is almost certainly a Warriors
card (S/A for Warriors). A card with just [Tide] might be Warriors, Sacrifice,
Self-Mill, or Ramp. Pair matching is dramatically more precise.

**Example (one of many possibilities):** "Track your drafted resonance pairs;
each pack, 1 slot shows a random card whose ordered pair matches your most
common pair, the rest are random." — But there are many other pair-based
designs: weighted sampling by pair overlap with drafted deck, pair-based slot
probabilities, pair-based pool seeding, etc.

**Why this domain is promising:** V4 proved that the ~50% archetype dilution
from single-resonance matching is the structural bottleneck capping
probabilistic approaches at ~1.7 S/A. Pair matching eliminates this
bottleneck for 75% of cards. This means a probabilistic approach using pairs
(weighted sampling, filtering, etc.) might achieve 2.0+ S/A WITHOUT needing to
add bonus cards or lock slots — a genuinely new result.

**The simplicity challenge:** Pair matching is slightly more complex to explain
than single-resonance matching. The one-sentence description must clearly
convey "match on the combination of first and second symbols, not just the
first." Agents must ensure their pair-based algorithm is genuinely
explainable.

### Domain 5: Conditional Pack Enhancement

**Core idea:** Instead of always adding bonus cards (Domain 1) or always
targeting slots (Domain 2), add bonus cards only when the random base pack
naturally clusters with the player's archetype. The pack's own random
composition triggers (or doesn't trigger) an enhancement. This creates natural
variance because sometimes the random 4 cards happen to include on-archetype
cards (triggering a bonus), and sometimes they don't (no bonus).

**What this brings new:** All previous algorithms (V3, V4, and Domains 1-4
above) generate packs without considering what was already drawn in the same
pack. This domain makes pack generation *self-referential* — the pack itself
helps determine whether it gets enhanced. This creates natural clustering
where good packs get better and bad packs stay random, mimicking the feel of
lucky vs. unlucky draws.

**Example (one of many possibilities):** "Draw 4 random cards; if 2 or more
share a resonance with your most-drafted resonance, add 1 bonus card of that
resonance." — But there are many other conditional designs: bonus triggered by
matching the drafted card's pair, enhancement probability based on how many
pack cards match the player's profile, replacing the worst-matching card
instead of adding a bonus, etc.

**Why this domain is promising:** Conditional enhancement creates organic
variance. A committed player gets bonus cards roughly 50-60% of the time (when
the random draw clusters favorably), creating a natural mix of great packs
(5 cards with bonus) and normal packs (4 random cards). The variance feels
like genuine luck rather than mechanical intervention because it depends on
the current pack's random composition — something neither the player nor the
algorithm can fully control.

---

## Rounds

### Round 1: Algorithm Exploration and Design (5 parallel agents)

Each agent explores their assigned domain of mechanism design. This round is
pure reasoning and analysis — **no simulation code**.

**All agents read:** This orchestration plan, plus the V3 final report
(`docs/resonance/v3/final_report.md`) and V4 final report
(`docs/resonance/v4/final_report.md`) for context.

**Each agent must produce (max 2000 words):**

1. **Five algorithm proposals:** Brainstorm 5 concrete, distinct algorithms
   within your domain. **All 5 must appear in the output document** — later
   rounds and the final algorithm_overview.md depend on having a complete
   catalog. For each of the 5, provide:
   - A name
   - A one-sentence player-facing description (must pass the Simplicity Test
     AND the no-extra-actions constraint)
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
| Agent 1 | Passive Resonance Bonus | `docs/resonance/v5/design_1_passive_bonus.md` |
| Agent 2 | Probabilistic Slot Targeting | `docs/resonance/v5/design_2_soft_locking.md` |
| Agent 3 | Pool Evolution / Seeding | `docs/resonance/v5/design_3_pool_evolution.md` |
| Agent 4 | Pair-Based Pack Construction | `docs/resonance/v5/design_4_pair_based.md` |
| Agent 5 | Conditional Pack Enhancement | `docs/resonance/v5/design_5_conditional.md` |

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

2. **Simplicity and no-actions audit (messages 11-20):** Evaluate the 5
   championed algorithms against both the Simplicity Test and the V5
   no-extra-actions constraint. Can you write each algorithm from its
   one-sentence description? Is the description hiding any player decisions?
   Which are genuinely passive and implementable from the description alone?

3. **Goal tradeoff analysis (messages 21-30):** For each design goal, which
   championed algorithm is best? Which is worst? Special focus on convergence
   (Goal 6) — does the algorithm cross 2.0 S/A? If not, can it be modified to
   do so without adding player decisions?

4. **Refinement proposals (messages 31-40):** Each agent proposes specific
   modifications to their championed algorithm based on the discussion. May
   switch champions if convinced a different proposal from their domain is
   stronger. These changes will be incorporated into Round 3 simulations.

**Key discussion questions:**
- Which algorithms produce the most *natural* variance?
- Which algorithms are most likely to cross the 2.0 S/A threshold?
- Does pair-based matching (Domain 4) really break through the dilution
  ceiling? What's the expected improvement over single-resonance matching?
- How do passive auto-spending algorithms compare to intelligent manual
  spending? Does the loss of player agency matter for convergence?
- Which algorithms make signal reading (Goal 9) possible vs. impossible?
- How do these algorithms compare to V3's Lane Locking and V4's Pack
  Widening? Is there a clear winner?
- What symbol distribution (1 vs 2 vs 3 symbols per card) works best for
  each championed algorithm?

**Output per agent (max 800 words each):**
- `docs/resonance/v5/discussion_{1|2|3|4|5}.md`

Each discussion output should include:
- A **simplicity ranking** of all 5 championed algorithms (most to least
  simple)
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
   operating only on visible card properties (not archetype fitness) unless
   explicitly noted.
3. **Simulate 1000 drafts** of 30 picks each with all 3 player strategies.
4. **Measure all 8 metrics** from the measurable targets table, PLUS the
   variance target (stddev of S/A cards per pack for picks 6+).
5. **ALL METRICS MUST BE AT ARCHETYPE LEVEL.** "S/A cards for archetype" means
   the card has S-tier or A-tier archetype_fitness for the player's specific
   target archetype. Do NOT measure at the resonance level. Do NOT count a
   card as "fitting" merely because it shares a resonance with the target
   archetype.
6. **Run parameter sensitivity sweeps** on 2-3 key parameters identified in
   Round 1, PLUS a sweep on symbol distribution (what happens with mostly
   1-symbol cards vs. mostly 2-symbol vs. mostly 3-symbol?).
7. **Produce 3 detailed draft traces** (pick-by-pick):
   - A player who commits early (by pick 5)
   - A player who stays flexible for 8+ picks
   - A signal-reader who identifies the open resonance
8. **Report pack-quality variance:** For the committed player (picks 6+),
   report the distribution of S/A cards per pack (how often 0, 1, 2, 3, 4)
   and the standard deviation.
9. **Per-archetype convergence table:** For each of the 8 archetypes, run
   simulations where a committed player targets that specific archetype.
   Report the average pick number at which the player begins regularly seeing
   2+ S/A cards for that archetype per pack. Present this as a table:

   | Archetype | Avg. Convergence Pick |
   |-----------|----------------------|
   | Flash/Tempo/Prison | ? |
   | Blink/Flicker | ? |
   | Storm/Spellslinger | ? |
   | Self-Discard | ? |
   | Self-Mill/Reanimator | ? |
   | Sacrifice/Abandon | ? |
   | Warriors/Midrange | ? |
   | Ramp/Spirit Animals | ? |

   This table is critical for verifying that the algorithm treats all 8
   archetypes fairly. If some archetypes converge at pick 5 and others at
   pick 12, the algorithm has a balance problem.
10. **Compare to V3 Lane Locking AND V4 Pack Widening baselines:** Run both
    Lane Locking (threshold 3/8, primary=2) and Pack Widening v3 (cost 3,
    bonus 1, auto-spend on highest resonance) on the same card pool and report
    the same metrics for comparison. The auto-spend Pack Widening serves as
    the natural baseline for Domain 1, and Lane Locking serves as the baseline
    for Domain 2.
11. **Test the one-sentence claim:** Can you reconstruct the algorithm from
    just the one-sentence description, without reading the code? If the
    algorithm has drifted from the one-sentence description during refinement,
    UPDATE the one-sentence description or simplify the algorithm.
12. **Verify no player decisions:** Confirm that the algorithm is fully
    automatic. The simulated player never makes a decision beyond which card
    to pick from the pack. If the algorithm requires any other input from the
    player, it fails the V5 constraint.

**Each agent must produce:**

- `docs/resonance/v5/sim_{1|2|3|4|5}.py` — simulation code
- `docs/resonance/v5/results_{1|2|3|4|5}.md` (**max 800 words**) structured
  as:
  - **One-sentence algorithm** (final version, must pass Simplicity Test and
    no-actions constraint)
  - **Target scorecard table** (metric | target | actual | pass/fail) — ALL AT
    ARCHETYPE LEVEL
  - **Variance report** (stddev, distribution of S/A cards per pack)
  - **Per-archetype convergence table** (8 rows, one per archetype, showing
    average pick at which 2+ S/A cards per pack become regular)
  - **V3/V4 comparison** (same metrics, side by side, against Lane Locking AND
    auto-spend Pack Widening)
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

1. Score each strategy on each of the 9 design goals (1-10), with 1-sentence
   justification.
2. Identify the single biggest strength and biggest weakness of each strategy.
3. Propose specific improvements — what would you change about each strategy
   to fix its weaknesses?
4. **The V3/V4 comparison:** Based on simulation results, does any V5
   algorithm clearly beat Lane Locking AND Pack Widening? What are the
   tradeoffs? Is the zero-decision interface worth any loss in convergence
   compared to Pack Widening? Is the natural variance worth any loss compared
   to Lane Locking?
5. **The hybrid question:** Given the simulation results, what is the best
   possible algorithm you can construct, drawing from any V5 strategy (and
   potentially incorporating ideas from V3/V4)?
6. **The pair-matching question:** Did pair-based algorithms (Domain 4) break
   through the archetype dilution ceiling as hypothesized? If so, does this
   open up new hybrid possibilities? If not, why not?

**Discussion topics (minimum 30 total messages):**
- Which strategies actually hit the measurable targets while also hitting the
  variance target? Which sacrifice convergence for variance or vice versa?
- Rank the 5 one-sentence descriptions from most honest/complete to most
  hand-wavy.
- For the strategies that failed targets: is the mechanism fixable with
  parameter tuning, or is the fundamental approach wrong?
- Is there a V5 algorithm that clearly dominates both V3's Lane Locking and
  V4's Pack Widening? Or do the earlier recommendations remain better despite
  their respective flaws?
- Is there a hybrid that clearly dominates?

**Output per agent (max 800 words each):**
- `docs/resonance/v5/comparison_{1|2|3|4|5}.md`

Each comparison output should include:
- A **scorecard table** (strategy x goal matrix, 1-10 scores)
- Their **proposed best algorithm** (can be their original, a modification, a
  hybrid, or even "Lane Locking / Pack Widening is still better")
- The one-sentence description of their proposed best algorithm

---

### Round 5: Final Synthesis (1 agent)

A single agent produces the definitive comparison and recommendation.

**Reads (in priority order):**
1. This orchestration plan (goals, targets, card model)
2. All 5 comparison outputs from Round 4
3. All 5 results documents from Round 3
4. Round 2 discussion documents (for context on design evolution)
5. V3 final report (`docs/resonance/v3/final_report.md`) for Lane Locking
   baseline
6. V4 final report (`docs/resonance/v4/final_report.md`) for Pack Widening
   baseline
7. Only if needed: simulation code from Round 3

**Task:**

1. **Run all 5 simulations** with identical parameters and produce a unified
   comparison table — ALL METRICS AT ARCHETYPE LEVEL.
2. For each algorithm, compute all measurable targets (including variance
   target) and flag pass/fail.
3. **Include V3 Lane Locking AND V4 Pack Widening** in the comparison table
   as baselines.
4. **Rank the 5 V5 algorithms** by overall design goal satisfaction.
5. **The key question:** Does any V5 algorithm beat both Lane Locking and Pack
   Widening? Does it achieve Pack Widening's convergence (2.0+) with zero
   player decisions and natural variance? Make a clear recommendation with
   justification.
6. **Apply the simplicity test** independently: for each one-sentence
   description, can the synthesis agent write the algorithm from scratch? Flag
   any descriptions that are misleading, incomplete, or that hide player
   decisions.
7. **Per-archetype convergence table** for each algorithm (including Lane
   Locking and Pack Widening). For each of the 8 archetypes, report the
   average pick number at which a committed player begins regularly seeing 2+
   S/A cards per pack. Present as a unified table with all algorithms as
   columns and all 8 archetypes as rows. Flag any archetype where convergence
   is slower than pick 8 or faster than pick 4.
8. Write the **recommended algorithm** with:
   - Complete specification (step-by-step, unambiguous)
   - One-sentence player description (no player decisions allowed)
   - One-paragraph player description
   - Implementation notes (edge cases, parameter values)
   - Recommended symbol distribution
   - Per-archetype convergence table
9. **V5 vs V3 vs V4 deep comparison.** Write a dedicated section comparing
   the V5 recommended algorithm against both V3's Lane Locking and V4's Pack
   Widening. This must include both quantitative and qualitative analysis:

   **Quantitative:** A side-by-side table of all measurable targets (including
   per-archetype convergence picks and the variance target), showing exact
   numbers for all three algorithms on the same card pool. Flag where each
   algorithm wins and by how much.

   **Qualitative:** Analyze the pros and cons of each algorithm across these
   dimensions:
   - **Player experience:** Does the draft feel like natural variation,
     mechanical delivery, or resource management? Which feels best for a
     player who just wants to pick cards?
   - **Cognitive load:** How much does the player need to think about the
     draft system vs. just thinking about which card they want? Lane Locking
     requires monitoring lock state. Pack Widening requires spending decisions.
     V5 should require nothing beyond card evaluation.
   - **Transparency:** Can a player predict/reason about their next pack? Is
     the algorithm's influence visible or hidden?
   - **Flexibility:** Can a player pivot mid-draft? What happens if you
     misread signals early?
   - **Skill expression:** Does the algorithm reward good decision-making?
     Are there interesting choices beyond "pick the best card"?
   - **Simplicity:** Which algorithm is easiest to explain to a new player?
     Which one-sentence description is most honest?
   - **Degeneracy resistance:** Can a player exploit the algorithm to force
     the same deck every run? How robust is each algorithm to min-maxing?
   - **Archetype balance:** Does each algorithm treat all 8 archetypes
     fairly, or do some archetypes converge faster/slower?

   End with a clear verdict: is the V5 algorithm an improvement over both V3
   and V4? If V5 achieves comparable convergence with zero player decisions,
   is the reduced cognitive load a meaningful advantage? If not, which of the
   three generations (V3 Lane Locking, V4 Pack Widening, V5 recommended)
   should be implemented?

10. Identify remaining open questions for playtesting.

**Output:**

- `docs/resonance/v5/final_report.md` (**max 3500 words**) — The definitive
  comparison, recommendation, and implementation specification. Must include
  the V5 vs V3 vs V4 deep comparison section with quantitative and qualitative
  analysis.

- `docs/resonance/v5/algorithm_overview.md` (**max 3000 words**) — A
  comprehensive catalog of ALL algorithms considered during V5 (all 25
  proposals from Round 1, plus any hybrids from later rounds). For each
  algorithm:
  - One-sentence description (must satisfy no-player-decisions constraint)
  - How it works (2-3 sentences)
  - Which domain it belongs to
  - Whether it was championed and simulated
  - If simulated: scorecard results (pass/fail on each target)
  - If not simulated: why it was not championed (1 sentence)
  - Final ranking/recommendation status

---

## Agent Summary

| Round | Agents | Type | Description |
|-------|--------|------|-------------|
| 1 | 5 | Parallel background | Algorithm design per domain |
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

All output files are in `docs/resonance/v5/`.

## Key Principles for Agents

1. **No player decisions beyond card selection.** This is the #1 V5 constraint.
   If your algorithm asks the player to do anything other than pick 1 card from
   a pack, it fails. Review your one-sentence description — if it contains "you
   may," "choose whether to," or "spend," rewrite it with automatic behavior.

2. **Simplicity is non-negotiable.** If you cannot explain your algorithm in
   one sentence of concrete operations, simplify it until you can.

3. **ALL MEASUREMENT AT ARCHETYPE LEVEL.** Evaluate S/A fitness for the
   player's specific target archetype. Not resonance match. Not primary
   resonance match. Archetype-specific fitness. A "Tide card" is NOT a
   "Warriors card" — Tide is shared by Warriors, Sacrifice, Self-Mill, and
   Ramp, so roughly half of Tide cards are bad for Warriors. If your
   simulation counts resonance matches instead of archetype fitness, your
   convergence numbers will be ~2x too high.

4. **Cross 2.0 S/A or explain why not.** V3 and V4 both found algorithms above
   2.0. A V5 algorithm below 2.0 needs a compelling justification for why it's
   still worth considering (e.g., dramatically better variance, simplicity, or
   flexibility that compensates for lower convergence).

5. **Natural variance is a goal, not a bug.** Your algorithm should sometimes
   produce great packs and sometimes produce bad packs. Consistent mechanical
   delivery is a failure mode.

6. **Consider pair-based matching.** V5 introduces the insight that ordered
   resonance pairs map to archetypes with ~100% precision for 2+ symbol cards.
   Even if your domain is not Domain 4, consider whether pair-based matching
   would improve your algorithm.

7. **Compare to both V3 and V4 baselines.** Lane Locking is the convergence
   baseline. Pack Widening (auto-spend variant) is the "what if we just removed
   the player decision" baseline. Every simulation must include both for direct
   comparison.

8. **Prefer visible properties, but don't rule out fitness-aware algorithms.**
   Algorithms that operate on visible card properties (symbols, rarity, etc.)
   are preferred. But if an algorithm that uses archetype fitness directly
   produces dramatically better results, that's a valid finding.

9. **The one-sentence description IS the algorithm.** If your implementation
   does something your one-sentence description doesn't mention, either
   simplify the implementation or admit the description is incomplete.

10. **Symbol distribution is an open question.** How many cards should have 1
    vs 2 vs 3 resonance symbols? This matters especially for pair-based
    algorithms (Domain 4), which need 2+ symbol cards to identify pairs. Each
    agent should propose and justify their distribution.

11. **Test honestly.** Report failures clearly. An algorithm that fails 3
    targets but produces beautiful natural variance with zero cognitive load
    may be more interesting than one that passes all targets but feels
    mechanical.

## Recovery

Check which `docs/resonance/v5/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained — later rounds depend only on
earlier outputs.
