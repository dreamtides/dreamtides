# Resonance Draft System V4 — Orchestration Plan

## Lessons from V3

V3 explored five mechanistic domains (accumulation, structural, threshold,
reactive, pool manipulation) and recommended **Lane Locking** — a threshold
system where drafting 3 symbols of a resonance permanently locks a pack slot to
that resonance, with a second lock at 8. This algorithm scored 7/8 on
measurable targets and is genuinely simple to explain.

**However, Lane Locking has problems that motivate V4:**

1. **Too mechanical.** Locking a pack slot to a resonance is a deterministic
   intervention in pack generation. Once you have 2 locked Tide slots, you
   *always* see exactly 2 Tide-resonance cards — but Tide is shared by 4
   archetypes (Warriors, Sacrifice, Self-Mill, Ramp), so those 2 Tide cards
   might not even be good for your specific archetype. There is no variance —
   no lucky packs where you find 3 perfect archetype cards, no unlucky packs
   where you have to make do. The draft feels like a machine dispensing
   predictable but imprecise output.

2. **On rails after commitment.** Permanent locks mean the player cannot pivot
   after picks 6-8. A misread or an interesting late-draft opportunity is
   wasted because the algorithm has already decided your pack structure.

3. **99% deck concentration.** Committed players ended up with almost no
   off-archetype cards, suggesting the algorithm over-converges.

4. **All V3 algorithms shared a philosophy:** the system *assigns* cards to
   pack slots based on the player's resonance state. Whether through locked
   slots (Lane Locking), guaranteed-one-per-resonance (Balanced Pack), or
   weighted slot filling (Weighted Lottery), the system is always *placing*
   specific cards in specific slots. This makes every algorithm feel like a
   vending machine with different settings.

**V4's core question:** Can we find algorithms that feel like *natural
variation* rather than *mechanical intervention*? Algorithms where the player's
choices shift probabilities and create tendencies without ever guaranteeing
specific pack compositions? Algorithms where a Warriors player sometimes gets a
fantastic pack with 3 cards that are genuinely good for Warriors, and sometimes
gets 0, but on average they're converging toward their archetype — and that
variance is part of the fun?

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

**Consequence for algorithm design:** An algorithm that operates on resonance
symbols (which is fine — resonance is visible) must be EVALUATED by asking
"does this card have S/A-tier fitness for the player's specific target
archetype?" NOT "does this card share a resonance with the player's
archetype?" Resonance-level measurement inflates convergence numbers by ~2x
because it counts cards from the wrong archetypes as hits.

**Consequence for how we talk about the player's experience:** When describing
what a committed player should see in their packs, always use archetype
language, not resonance language:

- WRONG: "A player committed to Tide sees 2+ Tide cards per pack"
- RIGHT: "A player committed to Warriors sees 2+ Warriors cards per pack"

The algorithm may internally track Tide symbol counts, but what matters to
the player is whether those Tide cards are actually good for Warriors
specifically — and many of them won't be.

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

Design a draft algorithm that uses resonance symbols to construct packs in a
way that creates natural variance while converging on average. The algorithm
should feel like the draft environment is responding to your choices through
shifting currents, not through mechanical slot assignment.

**The anti-pattern to avoid:** Any algorithm that deterministically assigns
specific pack slots to specific resonances. No "slot X always shows a Tide
card." No "2 of your 4 pack slots are filled by your top resonance." The
algorithm should influence what cards are *likely* to appear without ever
*guaranteeing* a specific pack composition.

**The ideal:** A player who has committed to the Warriors archetype by pick 6
should *usually* see 2+ cards that are actually good for Warriors in their
packs — but sometimes they see 3, sometimes 1, occasionally 0. The average is
right, but individual packs are unpredictable. A great pack feels like a gift;
a bad pack forces interesting decisions. Note: "good for Warriors" means cards
with S/A-tier archetype fitness for Warriors specifically, NOT merely "cards
with Tide symbols" (since Tide is shared by 4 archetypes, many Tide cards are
bad for Warriors).

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
   YES "each symbol you draft adds a matching token to a bag, and pack cards
   are drawn by pulling tokens from that bag" — that's a real algorithm.
2. **Not on rails.** The player should not be forced into one archetype or have
   only 1 real choice per pack.
3. **No forced decks.** The player should not be able to force the same deck
   every time they play.
4. **Flexible archetypes.** It should be possible to build decks outside the
   core archetypes, or combine 2 archetypes.
5. **Convergent.** If you HAVE committed to an archetype (around pick 6 on
   average), you should see a minimum of 2 cards that are actually good for
   that specific archetype (S/A-tier) in most of your draft picks. "Good for
   Warriors" means the card is S/A-tier for Warriors — not merely "has Tide
   symbols."
6. **Splashable.** You should see around 1 card from outside your archetype in
   most draft picks.
7. **Open-ended early.** In the first ~5 picks, you should see a variety of
   cards from different archetypes.
8. **Signal reading.** There should be a moderate benefit to figuring out which
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

### Variance Target (New for V4)

In addition to the above averages, V4 algorithms should report the
**standard deviation** of archetype S/A cards per pack for picks 6+. The
target is:

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

Examples of FAILING descriptions:
- "The system nudges you toward your chosen archetype after you commit"
- "Packs adapt to your emerging strategy"
- "The draft responds to your resonance preferences"

Examples of PASSING descriptions:
- "Each symbol you draft adds a matching token to a bag; to make a pack, draw
  4 tokens and show a random card of each token's resonance"
- "When you draft a card, 3 random cards sharing its primary resonance are
  shuffled into the pool from a reserve"
- "Cards you pass on are removed from the pool for 5 picks, then return"

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
algorithm can work — and they are deliberately chosen to be outside the design
space explored in V3. Within their domain, each agent must **brainstorm 5
concrete algorithms**, analyze their tradeoffs, and **champion one** to
develop further.

The domains are deliberately broad. The example algorithm for each is just ONE
possibility — agents should NOT treat the example as the answer. The point is
to explore the space and find the best algorithm within their domain.

**V3 explored these domains (DO NOT repeat them):**
- Accumulation-based (bag building, running counters, resonance meters)
- Structural/guaranteed (fixed pack composition rules, one-per-resonance)
- Threshold/progression (discrete state changes at milestones, lane locking)
- Reactive/immediate (only recent picks matter, sliding windows)
- Pool manipulation (adding/removing cards from the available pool)

V4 agents should be aware of V3's results but must explore genuinely different
design territory. If an agent's proposal is equivalent to a V3 algorithm with
different parameters, it has failed.

### Anti-Pattern: Mechanical Slot Assignment

**No V4 algorithm should deterministically assign pack slots to resonances.**
The following patterns are banned:
- "Slot X always shows a card of resonance R"
- "N of your 4 pack slots are filled by your top resonance"
- "Each pack guarantees exactly one card per resonance"
- Any mechanism that makes a specific pack slot always show a specific resonance

The algorithm should influence the *probability distribution* of what appears,
not *mechanically place* cards. Individual packs should be unpredictable even
if the average behavior is convergent.

Note: even if slot assignment weren't banned, assigning a slot to a *resonance*
(e.g., Tide) does not guarantee a card for the player's *archetype* (e.g.,
Warriors). Tide is shared by 4 archetypes. This is why resonance-level
mechanisms felt mechanical in V3 without actually delivering reliable archetype
convergence.

### Domain 1: Rejection & Passing Mechanisms

**Core idea:** What you *don't* pick shapes future packs. Every V3 algorithm
only cared about the card the player chose. This domain explores the 3 cards
the player passes on each pick — the information and mechanical consequences
of rejection.

**Example (one of many possibilities):** "Cards you pass on are removed from
the pool for the next 5 picks, then return." — But there are many other
rejection mechanisms: passed cards boost the likelihood of similar cards
appearing, passed resonances become temporarily scarce, accumulated rejections
shift probabilities, "hate-drafting" has mechanical consequences, etc.

**What makes this domain distinct:** The algorithm's primary input is what the
player *didn't* choose, not what they did. Convergence emerges because
repeatedly passing on off-archetype cards removes them temporarily, naturally
increasing the density of cards from the player's target archetype in the
pool — but never guaranteeing specific pack compositions. This creates natural
variance because the effect depends on *which specific cards* were passed, not
just aggregate counts.

**Why it might produce natural variance:** The player passes 3 cards per pick
(vs. taking 1), so there's a large, noisy input signal. The effects of passing
cascade through the pool in hard-to-predict ways. Sometimes you pass on a
bunch of Tide cards and they all come back at once; sometimes they stay away.

### Domain 2: Soft Probabilistic Influence

**Core idea:** The player's draft history creates a soft bias in how cards are
*sampled* for packs, without ever guaranteeing specific resonances in specific
slots. All 4 pack slots are drawn from the same biased distribution. The
bias grows with commitment but is always stochastic.

**Example (one of many possibilities):** "Each drafted symbol slightly
increases the chance of seeing cards with that resonance — cards are sampled
with weight (1 + 0.3 * your_symbol_count_for_that_resonance), so a player
committed to Warriors who has accumulated Tide symbols sees more Tide-resonance
cards (many of which will be Warriors cards) but could see any card in any
slot." — But there are many
other soft influence mechanisms: multiplicative vs. additive weighting,
diminishing returns on heavy investment, per-card weighting based on symbol
overlap with drafted deck, influence that operates on card-level affinity
rather than resonance-level counts, etc.

**What makes this domain distinct:** Every pack slot is sampled from the same
pool with the same (biased) probability distribution. There are no "assigned"
or "locked" slots. The bias makes on-archetype cards more likely but never
certain. This produces natural pack-to-pack variance because each of the 4
draws is an independent random event from the same skewed distribution.

**Why it might produce natural variance:** With probabilistic sampling, a
committed player might see 0, 1, 2, 3, or even 4 on-archetype cards in a
single pack. The average converges but individual packs fluctuate. The key
challenge is making the bias strong enough for convergence without making it
so strong that variance disappears.

### Domain 3: Economic & Resource Mechanisms

**Core idea:** The player accumulates a resource (automatically or through
choices) and can *spend* it to influence what appears in their packs. The
draft algorithm is simple, but the player has agency over *when* to exert
influence — creating deliberate peaks and valleys in pack quality.

**Example (one of many possibilities):** "Each pick earns you 1 resonance
token matching the card's primary symbol; before seeing a pack, you may
spend any number of tokens to add that many extra cards of that resonance to
a 4+N card pack (still pick 1)." — But there are many other economic
mechanisms: tokens spent to reroll individual slots, tokens spent to filter
the pool before a pack is generated, tokens that accumulate automatically vs.
tokens earned by specific actions, dual-currency systems, etc.

**What makes this domain distinct:** The player has *active agency* over the
draft algorithm, choosing when and how to spend influence. This creates natural
variance because the player alternates between "spending" turns (great packs)
and "saving" turns (random packs). It also creates interesting strategic
decisions: do you spend early to find your archetype, or save for powerful
late-draft packs?

**Why it might produce natural variance:** When the player isn't spending, packs
are fully random. When they spend, packs are enhanced. This creates a natural
rhythm of good and bad packs driven by player choice rather than mechanical
assignment. The simplicity challenge is keeping the spending mechanism to one
sentence.

### Domain 4: Phantom Drafter / Competitive Scarcity

**Core idea:** Simulate one or more phantom drafters who also take cards from
the shared pool. The player is drafting against invisible competition for a
finite shared resource. Scarcity emerges naturally from competition, and signal
reading comes from observing what the phantom drafters leave behind.

**Example (one of many possibilities):** "A phantom drafter picks 1 card per
round from the same pool, always choosing the card with the most symbols
matching its randomly-assigned resonance; the pool shrinks naturally over the
draft." — But there are many other competitive mechanisms: multiple phantoms
with different strategies, phantoms whose preferences are revealed gradually,
phantoms that react to the player's choices, card "demand" systems where
popular resonances become naturally scarcer, etc.

**What makes this domain distinct:** Convergence and variance emerge from
*competition for shared resources*, not from the pack generation algorithm
itself. The algorithm for generating packs can be trivially simple (draw 4
random cards from the pool). All the interesting behavior comes from how the
pool is depleted by competition. Signal reading is natural: if you notice
Storm and Blink cards disappearing (both Ember-primary archetypes), a phantom
is likely competing for Ember-resonance cards, making those archetypes harder
to draft.

**Why it might produce natural variance:** Phantom drafters create
unpredictable scarcity. Sometimes the phantom takes a card you wanted;
sometimes it takes cards you don't care about. The pool state is never quite
the same between picks, creating natural variance in what's available. Pack
quality depends on what the phantom left behind, which is inherently
unpredictable.

### Domain 5: Curated Randomness / Filtered Sampling

**Core idea:** Instead of mechanically assigning slots, generate a larger
candidate set and then filter or sample from it using criteria influenced by
the player's history. The indirection between "what could appear" and "what
does appear" creates natural variance while allowing soft convergence.

**Example (one of many possibilities):** "To make a pack, deal 8 random cards
face-down, then reveal 4 chosen by weighted coin flips that favor cards
sharing resonance with your drafted cards." — But there are many other
filtering mechanisms: generate N candidates and keep the 4 with highest
affinity score, oversample then randomly cull, apply a stochastic acceptance/
rejection filter to candidates, use the player's history as a "lens" that
makes some cards more likely to pass through, etc.

**What makes this domain distinct:** Pack generation has two phases: broad
random generation, then selective filtering. The randomness of the first phase
ensures variance; the filtering of the second phase ensures convergence. The
player never sees the rejected candidates, so the algorithm feels like natural
variation rather than mechanical assignment.

**Why it might produce natural variance:** The initial random generation
ensures that sometimes the candidate pool has many on-archetype cards (great
pack after filtering) and sometimes few (mediocre pack after filtering). The
filter amplifies tendencies but can't create cards that weren't in the
candidate pool. This produces natural variance where pack quality depends on
the luck of the initial draw plus the strength of the filter.

---

## Rounds

### Round 1: Algorithm Exploration and Design (5 parallel agents)

Each agent explores their assigned domain of mechanism design. This round is
pure reasoning and analysis — **no simulation code**.

**All agents read:** This orchestration plan, plus the V3 final report
(`docs/resonance/v3/final_report.md`) for context on V3's results.

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
| Agent 1 | Rejection & Passing | `docs/resonance/v4/design_1_rejection.md` |
| Agent 2 | Soft Probabilistic Influence | `docs/resonance/v4/design_2_soft_prob.md` |
| Agent 3 | Economic & Resource | `docs/resonance/v4/design_3_economic.md` |
| Agent 4 | Phantom Drafter / Competitive Scarcity | `docs/resonance/v4/design_4_phantom.md` |
| Agent 5 | Curated Randomness / Filtered Sampling | `docs/resonance/v4/design_5_filtered.md` |

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
- Which algorithm produces the most *natural* variance? Not "high variance"
  for its own sake, but variance that feels like a natural consequence of the
  draft environment rather than mechanical slot assignment?
- Which algorithms degenerate into "random noise" early and "on rails" late
  with no interesting middle ground?
- Are there unchampioned proposals that deserve simulation? Should any agent
  switch which algorithm they develop?
- Which algorithms make signal reading (goal 8) possible vs. impossible?
- How do these algorithms compare to V3's Lane Locking? Is there a clear
  winner, or do the best V4 proposals trade off differently?
- What symbol distribution (1 vs 2 vs 3 symbols per card) works best for
  each championed algorithm?

**Output per agent (max 800 words each):**
- `docs/resonance/v4/discussion_{1|2|3|4|5}.md`

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
   operating only on visible card properties (not archetype fitness).
3. **Simulate 1000 drafts** of 30 picks each with all 3 player strategies.
4. **Measure all 8 metrics** from the measurable targets table, PLUS the
   variance target (stddev of S/A cards per pack for picks 6+).
5. **ALL METRICS MUST BE AT ARCHETYPE LEVEL.** "S/A cards for archetype" means
   the card has S-tier or A-tier archetype_fitness for the player's specific
   target archetype. Do NOT measure at the resonance level. Do NOT count a
   card as "fitting" merely because it shares a resonance with the target
   archetype. V3 made this mistake initially and had to issue corrections.
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
10. **Compare to V3 Lane Locking baseline:** Run the Lane Locking algorithm
   (threshold 3/8, primary=2) on the same card pool and report the same
   metrics for comparison. This gives a direct apples-to-apples comparison.
11. **Test the one-sentence claim:** Can you reconstruct the algorithm from
    just the one-sentence description, without reading the code? If the
    algorithm has drifted from the one-sentence description during refinement,
    UPDATE the one-sentence description or simplify the algorithm.

**Each agent must produce:**

- `docs/resonance/v4/sim_{1|2|3|4|5}.py` — simulation code
- `docs/resonance/v4/results_{1|2|3|4|5}.md` (**max 800 words**) structured as:
  - **One-sentence algorithm** (final version)
  - **Target scorecard table** (metric | target | actual | pass/fail) — ALL AT
    ARCHETYPE LEVEL
  - **Variance report** (stddev, distribution of S/A cards per pack)
  - **Per-archetype convergence table** (8 rows, one per archetype, showing
    average pick at which 2+ S/A cards per pack become regular)
  - **V3 Lane Locking comparison** (same metrics, side by side)
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
4. **The V3 comparison:** Based on simulation results, does any V4 algorithm
   clearly beat V3's Lane Locking? What are the tradeoffs? Is the natural
   variance worth any loss in average convergence?
5. **The hybrid question:** Given the simulation results, what is the best
   possible algorithm you can construct, drawing from any V4 strategy (and
   potentially incorporating ideas from V3)?

**Discussion topics (minimum 30 total messages):**
- Which strategies actually hit the measurable targets while also hitting the
  variance target? Which sacrifice convergence for variance or vice versa?
- Rank the 5 one-sentence descriptions from most honest/complete to most
  hand-wavy.
- For the strategies that failed targets: is the mechanism fixable with
  parameter tuning, or is the fundamental approach wrong?
- Is there a V4 algorithm that clearly dominates V3's Lane Locking? Or does
  Lane Locking remain the best option despite feeling mechanical?
- Is there a hybrid that clearly dominates?

**Output per agent (max 800 words each):**
- `docs/resonance/v4/comparison_{1|2|3|4|5}.md`

Each comparison output should include:
- A **scorecard table** (strategy x goal matrix, 1-10 scores)
- Their **proposed best algorithm** (can be their original, a modification, a
  hybrid, or even "Lane Locking is still better")
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
6. Only if needed: simulation code from Round 3

**Task:**

1. **Run all 5 simulations** with identical parameters and produce a unified
   comparison table — ALL METRICS AT ARCHETYPE LEVEL.
2. For each algorithm, compute all measurable targets (including variance
   target) and flag pass/fail.
3. **Include V3 Lane Locking** in the comparison table as a baseline.
4. **Rank the 5 V4 algorithms** by overall design goal satisfaction.
5. **The key question:** Does any V4 algorithm beat Lane Locking? Is the
   tradeoff of natural variance worth any loss in average convergence? Make a
   clear recommendation with justification.
6. **Apply the simplicity test** independently: for each one-sentence
   description, can the synthesis agent write the algorithm from scratch? Flag
   any descriptions that are misleading or incomplete.
7. **Per-archetype convergence table** for each algorithm (including Lane
   Locking). For each of the 8 archetypes, report the average pick number at
   which a committed player begins regularly seeing 2+ S/A cards per pack.
   Present as a unified table with all algorithms as columns and all 8
   archetypes as rows. Flag any archetype where convergence is slower than
   pick 8 or faster than pick 4.
8. Write the **recommended algorithm** with:
   - Complete specification (step-by-step, unambiguous)
   - One-sentence player description
   - One-paragraph player description
   - Implementation notes (edge cases, parameter values)
   - Recommended symbol distribution
   - Per-archetype convergence table
9. **V4 vs V3 deep comparison.** Write a dedicated section comparing the V4
   recommended algorithm against V3's Lane Locking (threshold 3/8 + pool
   asymmetry). This must include both quantitative and qualitative analysis:

   **Quantitative:** A side-by-side table of all measurable targets (including
   per-archetype convergence picks and the variance target), showing exact
   numbers for both algorithms on the same card pool. Flag where each
   algorithm wins and by how much.

   **Qualitative:** Analyze the pros and cons of each algorithm across these
   dimensions:
   - **Player experience:** Does the draft feel like natural variation or
     mechanical delivery? How does pack-to-pack variance affect the emotional
     arc of a draft?
   - **Transparency:** Can a player predict/reason about their next pack? Is
     the algorithm's influence visible or hidden?
   - **Flexibility:** Can a player pivot mid-draft? What happens if you
     misread signals early?
   - **Skill expression:** Does the algorithm reward good decision-making?
     Are there interesting strategic choices beyond "pick the best card"?
   - **Simplicity:** Which algorithm is easier to explain to a new player?
     Which one-sentence description is more honest?
   - **Degeneracy resistance:** Can a player exploit the algorithm to force
     the same deck every run? How robust is each algorithm to min-maxing?
   - **Archetype balance:** Does each algorithm treat all 8 archetypes
     fairly, or do some archetypes converge faster/slower?

   End with a clear verdict: is the V4 algorithm an improvement over V3, a
   sidegrade with different tradeoffs, or worse? If the V4 algorithm is not
   clearly better, say so honestly and explain what V3 does better.
10. Identify remaining open questions for playtesting.

**Output:**

- `docs/resonance/v4/final_report.md` (**max 3500 words**) — The definitive
  comparison, recommendation, and implementation specification. Must include
  the V4 vs V3 deep comparison section with quantitative and qualitative
  analysis.

- `docs/resonance/v4/algorithm_overview.md` (**max 3000 words**) — A
  comprehensive catalog of ALL algorithms considered during V4 (all 25
  proposals from Round 1, plus any hybrids from later rounds). For each
  algorithm:
  - One-sentence description
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

All output files are in `docs/resonance/v4/`.

## Key Principles for Agents

1. **Simplicity is non-negotiable.** If you cannot explain your algorithm in
   one sentence of concrete operations, simplify it until you can.

2. **No mechanical slot assignment.** Do not deterministically assign pack
   slots to resonances. Influence probabilities, don't place cards.

3. **Natural variance is a goal, not a bug.** Your algorithm should sometimes
   produce great packs and sometimes produce bad packs. Consistent mechanical
   delivery is a failure mode in V4, not a success.

4. **ALL MEASUREMENT AT ARCHETYPE LEVEL.** Evaluate S/A fitness for the
   player's specific target archetype. Not resonance match. Not primary
   resonance match. Archetype-specific fitness. A "Tide card" is NOT a
   "Warriors card" — Tide is shared by Warriors, Sacrifice, Self-Mill, and
   Ramp, so roughly half of Tide cards are bad for Warriors. If your
   simulation counts resonance matches instead of archetype fitness, your
   convergence numbers will be ~2x too high. This is the single most
   important measurement requirement and V3's biggest mistake.

5. **Prefer visible properties, but don't rule out fitness-aware algorithms.**
   Algorithms that operate on visible card properties (symbols, rarity, etc.)
   are preferred because they are more transparent to players. But if an
   algorithm that uses archetype fitness directly produces dramatically better
   results, that's a valid finding. Always note whether your algorithm uses
   only visible properties or also hidden fitness data.

6. **The one-sentence description IS the algorithm.** If your implementation
   does something your one-sentence description doesn't mention, either
   simplify the implementation or admit the description is incomplete.

7. **Compare to V3.** Lane Locking is the baseline to beat. Every simulation
   must include Lane Locking results on the same card pool for direct
   comparison. It's acceptable to recommend Lane Locking if no V4 algorithm
   is clearly better — the goal is to explore the design space honestly, not
   to force a new winner.

8. **Genuinely different domains.** The 5 domains represent structurally
   distinct design philosophies. Do NOT converge them into minor variations.

9. **Symbol distribution is an open question.** How many cards should have 1
   vs 2 vs 3 resonance symbols? Each agent should propose and justify their
   distribution.

10. **Test honestly.** Report failures clearly. An algorithm that fails 3
    targets but produces beautiful natural variance may be more interesting
    than one that passes all targets mechanically.

## Recovery

Check which `docs/resonance/v4/*.md` and `*.py` files exist to determine
progress. Each round's output is self-contained — later rounds depend only on
earlier outputs.
