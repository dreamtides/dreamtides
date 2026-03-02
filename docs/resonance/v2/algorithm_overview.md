# Archetype Draft System: All Algorithms Considered

This document describes every draft algorithm explored during the V2 design
process, in plain language. It covers what each system does, how it performed
in simulation, and how the designs evolved across rounds.

## Table of Contents

- [Background: What Problem Are We Solving?](#background)
- [Key Concepts](#key-concepts)
- [Round 1: Six Theoretical Approaches](#round-1-six-theoretical-approaches)
- [Round 2: Four Simulated Systems](#round-2-four-simulated-systems)
- [Round 4: Four Refined Systems](#round-4-four-refined-systems)
- [How This Relates to Resonance](#how-this-relates-to-resonance)
- [Visual Communication to Players](#visual-communication-to-players)
- [Universal Findings](#universal-findings)

---

## Background

During a Dreamtides quest, the player builds a deck of ~30 cards by drafting.
Each pick, they see 4 cards and choose 1. The full card pool contains ~360
unique cards (with rarity-based copy counts creating ~1000 total pool entries).

The design question: **how should the system choose which 4 cards to show the
player on each pick?** The answer needs to balance competing goals:

1. **Early exploration** (picks 1-5): Show the player a variety of strategies
   so they can figure out what's available
2. **Late convergence** (picks 6-30): Once the player has chosen a strategy,
   show them enough cards that fit it to build a coherent deck
3. **Splashability**: Always include at least one tempting card from *outside*
   the player's chosen strategy
4. **Run-to-run variety**: Two quests should feel different
5. **Signal reading**: Observant players should be able to detect which
   strategies are well-supported in this particular run

## Key Concepts

**Archetype**: A strategic deck theme, like "Reanimator" (plays cards from the
discard pile), "Tokens" (floods the board with small characters), or "Control"
(counters the opponent's plays). The number of archetypes is a design
parameter -- the studies explored everything from 3 to 12.

**Fitness score**: Each card has a rating in each archetype:
- **S-tier**: Core card for this archetype (designed for it)
- **A-tier**: Strong in this archetype (good synergy)
- **B-tier**: Playable filler
- **C-tier**: Weak (would rather not play it)
- **F-tier**: Unplayable in this archetype

A card might be S-tier in Reanimator, A-tier in Sacrifice (a related strategy),
and F-tier in Tokens (completely irrelevant).

**"Fitting" card**: A card rated S or A in the player's chosen archetype. When
this document says "2 fitting cards per pack," it means 2 of the 4 cards shown
are S or A tier in whatever strategy the player is pursuing.

**Multi-archetype card**: A card that is S or A tier in more than one
archetype. These are valuable because they serve multiple strategies, but
they're hard to design -- you need to find natural mechanical overlap between
two archetypes. The percentage of multi-archetype cards in the pool turned out
to be one of the most important design parameters.

**Weight multiplier**: When the system wants to show the player more cards from
their archetype, it increases those cards' probability of appearing. A "5x
weight" means fitting cards are 5 times as likely to be drawn. Higher
multipliers make fitting cards appear more reliably, but if set too high, the
player stops seeing interesting off-archetype options.

**Commitment detection**: The system's guess about which archetype the player
is pursuing, based on their picks so far. Typically triggers when the player
has drafted 3+ S/A-tier cards in one archetype. Getting this wrong (or
triggering it too early) was a major issue in several designs.

**Archetype suppression**: A variety mechanism where 2 of the 8 archetypes are
randomly weakened at the start of each quest. Their specialist cards have fewer
copies in the pool, making those strategies harder to draft. This forces the
player into different strategies each run, and observant players can detect
which archetypes are scarce.

---

## Round 1: Six Theoretical Approaches

Round 1 was pure analysis -- no simulations. Agents mapped out the design
space by reasoning about six fundamentally different approaches to pack
construction. These weren't full system designs; they were building blocks
that later rounds combined.

### 1. Pure Weighted Random Sampling

**How it works**: Every card has a weight. Draw 4 cards randomly, with
probability proportional to weight. Once the player commits to an archetype,
increase the weight of fitting cards.

**The problem**: The math doesn't work. With ~20% of the pool fitting any
given archetype, a random 4-card draw gives only a ~20-35% chance of seeing
2+ fitting cards. Even tripling the weights only gets you to ~40%. You'd need
8-10x weights to reliably hit 2+, but at that point you've nearly eliminated
off-archetype cards from packs.

**Verdict**: Too simple on its own. Needs supplementary mechanisms.

### 2. Deterministic Slot Guarantees

**How it works**: Each pack has fixed roles: "2 fitting cards + 1 splash card
+ 1 random card." The system fills each slot from the appropriate pool.

**The problem**: Guarantees convergence perfectly, but the player can see the
structure. Every pack feels the same -- two good cards for your archetype, one
random thing, one weird splash. It kills surprise and makes the system feel
like it's playing the game for you.

**Verdict**: Hits the convergence numbers but fails the "not on rails" goal.
Signal reading is impossible because the system's hand is visible.

### 3. Sub-Pool Systems

**How it works**: Instead of one big card pool, maintain separate pools per
archetype. Draw from the player's archetype pool plus a secondary pool for
variety.

**The problem**: Cards that fit multiple archetypes need to exist in multiple
sub-pools simultaneously. Managing shared cards, depletion, and bookkeeping
gets complex. Also requires confident archetype detection early.

**Verdict**: Clean conceptual model but implementation complexity is high.
Round 2's Model C explored a refined version of this.

### 4. Cube-Style Pre-Constructed Packs

**How it works**: Before the draft begins, a designer (or algorithm)
pre-builds all 30 packs with intentional tension -- each pack has interesting
choices across multiple archetypes.

**The problem**: After 5-10 runs, players memorize the packs and the draft
becomes solvable. Directly conflicts with run-to-run variety.

**Verdict**: Best individual pack quality, worst replayability. Unsuitable
for a roguelike.

### 5. Depletion-Based Systems

**How it works**: When the player sees cards but doesn't pick them, those
cards get removed from the pool (or have a chance of being removed). Over
time, the pool shifts based on what's been drafted vs. ignored.

**The unique insight**: This is the only approach that creates *emergent*
signal reading. If Reanimator cards keep disappearing from packs, an observant
player can infer someone (or the system) is depleting that strategy.

**The problem**: Depletion and convergence are natural enemies. The more you
draft Reanimator cards, the fewer remain in the pool, making future Reanimator
packs worse. You'd need weight increases to compensate, which fights the
natural dynamics.

**Verdict**: Interesting for variety and signaling, but needs to be combined
with other mechanisms. Round 2's Model D tried this.

### 6. Adaptive Ramp Systems

**How it works**: Start with no bias (pure random early packs), then gradually
increase the weight multiplier for the player's archetype as they commit. The
bias function is continuous -- zero at pick 1, moderate by pick 10, strong by
pick 20.

**The insight**: This directly addresses the "open early, convergent late"
tension. Early packs are exploratory by nature; late packs are focused by
nature. No mode switch needed.

**The challenge**: Requires a continuous estimate of how committed the player
is (fuzzy). Tuning the ramp curve is critical -- too gentle and convergence
fails; too aggressive and late packs feel predetermined.

**Verdict**: The most promising single mechanism. Every Round 2 system used
some form of this.

---

## Round 2: Four Simulated Systems

Each agent designed a complete system and ran 1000 simulated drafts. The
systems differ in how many archetypes they use, how they construct packs, and
how they create variety.

### Model A: "Big Archetypes" (4 archetypes)

**Core idea**: With only 4 archetypes, each one contains ~90 S-tier cards (a
quarter of the 360-card pool). The math is extremely forgiving -- almost any
random 4-card draw contains multiple fitting cards.

**How packs work**:
- Picks 1-5: Pure random (no bias)
- Picks 6+: Fitting cards get a gentle 1.5x-3.0x weight, ramping up over time
- No safety net needed -- the pool is so rich that fitting cards appear
  naturally

**Variety**: Per-run copy-count adjustments and random archetype weighting
(one archetype slightly boosted, one slightly weakened each run).

**What happened in simulation**:

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Early diversity (archetypes per pack) | >= 3 | 2.65 | FAIL |
| Early fitting (cards fitting emerging arch) | <= 2 | 1.67 | Pass |
| Late fitting (fitting cards per pack) | >= 2 | 2.47 | Pass |
| Off-archetype options per pack | >= 0.5 | 1.34 | Pass |
| Convergence timing | Pick 5-8 | Pick 7 | Pass |
| Deck concentration | 60-80% | 94.3% | FAIL |
| Run-to-run card overlap | < 40% | 5.5% | Pass |
| Archetype balance | 5-20% each | 22-27% | Pass |
| **Total** | | | **7/9** |

**Why it failed**: With only 4 archetypes, a third of early packs show cards
from just 2 archetypes -- the player doesn't get to explore. And because
fitting cards are so abundant, committed players build 94% on-archetype decks
with no tension. The system is too easy; there are no interesting decisions.

**Key finding**: 4 archetypes is too few. Convergence is trivially solved
but at the cost of everything that makes drafting interesting.

---

### Model B: "Many Archetypes" (10 archetypes)

**Core idea**: 10 archetypes means each one has only ~36 S-tier cards. To
compensate for thin pools, 62% of cards are rated S or A in multiple
archetypes (heavy multi-archetype overlap). Uses a ring topology where each
archetype has 2-3 "neighbors" sharing more cards.

**How packs work**:
- Picks 1-4: Pure random
- Picks 5-7: 2x weight for fitting cards
- Picks 8-12: 3.5x weight
- Picks 13+: 5x weight
- **Soft floor**: After pick 6, if a pack has 0 fitting cards, one random
  card is replaced with a fitting card (fires ~15-25% of the time)

**Variety**: 3 archetypes boosted (1.5x copies), 3 normal, 4 suppressed
(0.6x copies) per run. 120 possible configurations.

**What happened in simulation**:

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Early diversity | >= 3 | 3.11 | Pass |
| Early fitting | <= 2 | 1.34 | Pass |
| Late fitting | >= 2 | 2.34 | Pass |
| Off-archetype options | >= 0.5 | 1.41 | Pass |
| Convergence timing | Pick 5-8 | 8.4 | FAIL |
| Deck concentration | 60-80% | 94.6% | FAIL |
| Run-to-run overlap | < 40% | 6.6% | Pass |
| Archetype balance | 5-20% each | 5.4-14% | Pass |
| **Total** | | | **6/8** |

**Why it struggled**: Convergence comes too late (pick 8.4) because 10
archetypes spread picks thinly -- it takes longer for any single archetype to
emerge as the leader. And reaching that point requires 62% of cards to be
multi-archetype, which is a massive design burden. The sensitivity analysis
showed that reducing multi-archetype cards below 40% causes convergence to
collapse entirely.

**Key finding**: 10 archetypes is too many for a 360-card pool. You'd need
~500+ cards for this to work. The design cost (62% multi-archetype cards) is
prohibitive.

---

### Model C: "Sub-Pool Carousel" (7 archetypes)

**Core idea**: Instead of drawing from one pool, organize cards into 7
overlapping sub-pools (one per archetype). Each card appears in every sub-pool
where it's rated S or A. Packs have structured slot roles.

**How packs work (pre-commitment, picks 1-5)**:
- Slot 1: Random card from a randomly spotlighted archetype's sub-pool
- Slot 2: Random card from a *different* spotlighted sub-pool
- Slot 3-4: Random from the full pool
- A "carousel" cycles through archetypes so the player sees different
  strategies highlighted each pick

**How packs work (post-commitment, picks 6+)**:
- Slot 1 ("anchor"): Always drawn from the committed archetype's sub-pool
  (guaranteed fitting card)
- Slot 2: 60% from committed sub-pool, 40% from a neighboring archetype
- Slot 3 ("splash"): Random from a non-committed sub-pool
- Slot 4 ("wild"): Random from full pool

**Variety**: 1-2 archetypes suppressed per run, carousel ordering randomized.

**What happened in simulation**:

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Early diversity | >= 3 | 5.66 | Pass |
| Early fitting | <= 2 | 1.59 | Pass |
| Late fitting | >= 2 | 2.29 | Pass |
| Off-archetype options | >= 0.5 | 1.34 | Pass |
| Convergence timing | Pick 5-8 | **3.0** | FAIL |
| Deck concentration | 60-80% | 95.6% | FAIL |
| Run-to-run overlap | < 40% | 7.2% | Pass |
| Archetype balance | 5-20% each | 9.4-20.1% | Borderline |
| **Total** | | | **6/9** |

**Why it failed**: Convergence fires at pick 3 -- before the player has even
finished exploring! The guaranteed anchor slot is so powerful that commitment
is detected almost immediately when multi-archetype cards satisfy multiple
archetypes simultaneously. The carousel itself over-exposes the archetype
space (5.66 of 7 archetypes visible per pack) leaving no mystery.

**Key finding**: Hard guarantees (anchor slots) are too strong. The commitment
detection threshold matters more than the pool composition -- this was the
most surprising finding of Round 2.

---

### Model D: "Variety-First" (8 archetypes)

**Core idea**: Prioritize run-to-run variety and signal reading. 8 archetypes
with 2 randomly suppressed per run (28 possible configurations). Uses a
starting card signal and depletion to create readable per-run asymmetries.

**How packs work**:
- Before the draft: Player sees 3 cards from active archetypes, keeps 1 as a
  free pick (semi-explicit signal about what's available)
- Picks 1-5: Pure random, no bias
- Picks 6+: Fitting cards get 5x weight (ramping to 7x by pick 21+). One slot
  is always drawn from off-archetype cards, biased toward high power.

**Depletion**: Each time the player sees 4 cards and picks 1, the 3 unpicked
cards each have a 40% chance of being removed from the pool entirely. Over 30
picks, ~36 additional cards disappear. Observant players can detect frequency
shifts ("I was seeing lots of Tokens cards early but now they're gone").

**Variety**: 2-of-8 suppression (strongest mechanism), starting card signal,
depletion dynamics, clustered neighbor topology.

**What happened in simulation**:

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Early diversity | >= 3 | 4.24 | Pass |
| Early fitting | <= 2 | 1.06 | Pass |
| Late fitting | >= 2 | 1.94 | FAIL (marginal) |
| Off-archetype options | >= 0.5 | 1.68 | Pass |
| Convergence timing | Pick 5-8 | 5.69 | Pass |
| Deck concentration | 60-80% | 90.7% | FAIL |
| Run-to-run overlap | < 40% | 7.0% | Pass |
| Archetype balance | 5-20% each | 9.3-16.7% | Pass |
| **Total** | | | **7/9** |

**Why it almost worked**: Model D had the best overall balance. Variety and
signal reading were excellent. The only metric failure was late fitting at
1.94 (target is 2.0) -- agonizingly close. The depletion mechanism turned out
to be less useful than hoped; signal-reader players barely outperformed
committed players, suggesting the signals were too subtle for even a
simulated "observant" player to exploit.

**Key finding**: 8 archetypes with 2 suppressed per run is the strongest
variety mechanism. Depletion adds complexity without proven benefit.

---

## Round 3: Debate

All four agents debated their designs. Key consensus:

1. **4 archetypes is too few** (fails early diversity structurally)
2. **10 archetypes is too many** for a 360-card pool (requires impractical
   62% multi-archetype cards)
3. **7-8 archetypes is the sweet spot**
4. **The 60-80% deck concentration target is impossible** when the system
   delivers 2+ fitting cards per pack and the player always picks fitting
   cards. All models relaxed this to 85-95%.
5. **Depletion is over-engineering** -- hard to explain, hard to detect,
   minimal measurable benefit
6. **Soft floors beat hard guarantees** -- occasional 0-fitting packs are OK;
   guaranteed anchor slots cause premature convergence
7. **2-of-8 archetype suppression** is the best variety mechanism (from
   Model D)
8. **Run-to-run variety is trivially easy** -- all models achieved 5-9%
   card overlap against a 40% target. No special mechanism needed.

---

## Round 4: Four Refined Systems

After the debate, all four agents converged toward similar architectures.
The structural consensus was:

- 8 archetypes, 2 suppressed per run
- Adaptive weight ramp (no bias early, increasing bias late)
- Soft floor (replace 1 card only when pack has 0 fitting cards)
- Dedicated splash slot (1 of 4 cards always off-archetype)
- Clustered neighbor topology (related archetypes share more cards)
- Starting card signal (see 3 cards before draft, keep 1 free)
- No depletion

The models differed primarily in **weight multiplier strength** and **multi-
archetype card percentage**. Here's how they compared:

### Model A v2 (gentle weights, low multi-archetype)

Changed from 4 archetypes to 8. Uses 6x/7x/8x weight ramp with 28%
multi-archetype cards.

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Early diversity | >= 3 | 3.56 | Pass |
| Early fitting | <= 2 | 0.82 | Pass |
| Late fitting | >= 2 | **1.83** | FAIL |
| Off-archetype | >= 0.5 | 1.86 | Pass |
| Convergence | Pick 5-8 | 7.5 | Pass |
| Deck concentration | 85-95% | 90.8% | Pass |
| Run overlap | < 40% | 8.0% | Pass |
| Arch balance | 5-20% | 8-18% | Pass |
| **Total** | | | **8/9** |

**Failed because**: Gentle 6-8x weights + only 28% multi-archetype cards
can't reliably produce 2+ fitting cards per pack.

---

### Model B v2 (moderate weights, minimal multi-archetype)

Changed from 10 archetypes to 8. Uses 5x/6x/7x weight ramp. Claimed the
lowest multi-archetype card requirement (15-20% minimum).

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Early diversity | >= 3 | 3.77 | Pass |
| Early fitting | <= 2 | 1.10 | Pass |
| Late fitting | >= 2 | **1.91** | FAIL |
| Off-archetype | >= 0.5 | 1.60 | Pass |
| Convergence | Pick 5-8 | 7.6 | Pass |
| Deck concentration | 85-95% | 90.1% | Pass |
| Run overlap | < 40% | 7.0% | Pass |
| Arch balance | 5-20% | 8-16% | Pass |
| **Total** | | | **7/8** |

**Failed because**: Even more conservative weights (5-7x) miss the late
fitting target. The trade-off is clear: lower weights preserve surprise
but can't guarantee enough fitting cards.

---

### Model C v2: "Tiered Weighted Sampling" (RECOMMENDED)

Completely redesigned. Dropped the carousel. Uses 7x/8x/9x weight ramp --
the strongest of any model -- with 40% multi-archetype cards.

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Early diversity | >= 3 | 3.92 | Pass |
| Early fitting | <= 2 | 0.92 | Pass |
| Late fitting | >= 2 | **2.02** | Pass |
| Off-archetype | >= 0.5 | 1.65 | Pass |
| Convergence | Pick 5-8 | 7.32 | Pass |
| Deck concentration | 85-95% | 89.6% | Pass |
| Run overlap | < 40% | 9.9% | Pass |
| Arch balance | 5-20% | 7.6-15.5% | Pass |
| **Total** | | | **9/9** |

**Why it won**: The only model to pass all 9 targets. The strong 7-9x weight
ramp is aggressive but works: fitting cards make up ~60% of the weighted
distribution, enough to reliably produce 2+ per pack while the dedicated
splash slot ensures off-archetype temptation. The commitment detection fix
(require pick >= 5, clear archetype lead) solved the premature convergence
problem from Round 2.

---

### Model D v2 (strong weights, reduced multi-archetype)

Dropped depletion. Uses the strongest weight ramp of any model (8x/10x)
but with only 28% multi-archetype cards.

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Early diversity | >= 3 | 3.57 | Pass |
| Early fitting | <= 2 | 0.81 | Pass |
| Late fitting | >= 2 | 2.02 | Pass |
| Off-archetype | >= 0.5 | 1.67 | Pass |
| Convergence | Pick 5-8 | **8.31** | FAIL |
| Deck concentration | 85-95% | 88.7% | Pass |
| Run overlap | < 40% | 9.0% | Pass |
| Arch balance | 5-20% | 6.6-17.4% | Pass |
| **Total** | | | **8/9** |

**Failed because**: Commitment detection requires pick >= 6 AND a clear
archetype lead, which delays convergence to pick 8.31 (just over the
target cap of 8). The system is cautious about declaring commitment,
which is good for avoiding premature convergence but pushes it slightly
too late.

---

## How This Relates to Resonance

The V1 system assumed "resonance colors" as the organizing principle -- each
card had 1-2 colors, and archetypes were color pairs (like Magic: The
Gathering's guild system). V2 deliberately dropped that framing to explore
whether something better exists.

**The V2 conclusion**: The recommended system (Model C v2) does NOT require a
visible color/resonance system. The algorithm works entirely on hidden
archetype fitness scores. However, a player-visible tagging system would
provide several benefits:

1. **Learnability**: Players need some way to recognize "this card goes with
   my strategy." Without any visible signal, they'd have to memorize 360
   cards' archetypes.

2. **Signal reading**: The suppression mechanism (2 of 8 archetypes weakened
   per run) is more readable if archetypes have visible markers. "I'm not
   seeing many blue cards" is much easier than "I haven't seen Dissolve-
   synergy cards."

3. **Commitment feedback**: Players should understand that the system is
   helping them once they've committed.

**So resonance could be the visible tagging system** -- but the design space
is wider than "each card has a color icon." See the next section.

---

## Visual Communication to Players

The user correctly identifies that slapping an archetype icon on each card
is the obvious/lame approach. Here are the options explored, from simplest
to most interesting:

### Option 1: Explicit Archetype Icons (the obvious one)

Each card has a symbol or border color indicating its primary archetype(s).
Multi-archetype cards show multiple symbols.

**Pros**: Clear, simple, impossible to misread.
**Cons**: Looks like a CCG color system. Feels mechanical and "solved" --
players see the icon, not the card. Defeats the "signal reading" goal because
signals are shouted, not whispered.

### Option 2: Resonance as Thematic Flavor

Instead of literal archetype tags, use the "resonance" concept as aesthetic
flavor. Cards in the same archetype share visual motifs, art direction, or
thematic elements (moonlight imagery, thorny borders, spectral effects) that
experienced players learn to associate with strategies. New players don't
need to understand the system -- they naturally draft cards that "look like
they go together."

**Pros**: Elegant, rewards pattern recognition, feels organic rather than
mechanical. Supports signal reading through subtle frequency shifts ("lots
of thorny-border cards this run").
**Cons**: Slower to learn. Some players may never notice. Hard to get
right in art direction.

### Option 3: Mechanical Keywords as Implicit Tags

If archetypes are built around mechanical themes (e.g., "cards that dissolve
things" or "cards that kindle"), the keywords themselves serve as archetype
markers. Players don't need a separate tagging system -- they learn that
Dissolve cards tend to synergize with each other.

**Pros**: Most organic. No artificial system visible at all. Aligns with
how real deckbuilders work.
**Cons**: Requires very tight mechanical design. Multi-archetype cards (that
bridge two keyword families) are harder to parse. Doesn't help with the
"2 archetypes suppressed per run" signal.

### Option 4: Hybrid -- Subtle Tags + Emergent Signals

Use a minimal visual system (small resonance symbol, card frame tint, or
energy cost color) that indicates general archetype family, combined with
mechanical keywords that indicate specific synergy. The resonance tag says
"I'm in the aggressive family" while the keywords say "specifically, I
support the Tokens strategy within that family."

This works well with the clustered neighbor topology from the simulation:
neighboring archetypes could share a resonance color, with keywords
distinguishing them. A "red" card might be Tokens or Aggro (neighbors),
and the player learns which red cards serve which strategy.

**Pros**: Fast initial learning ("draft red"), deeper mastery ("draft red
Tokens, not red Aggro"), signal reading works at both levels.
**Cons**: Adds a layer of complexity. The resonance system needs to map
cleanly to the archetype topology.

### What the Algorithm Needs from the Visual System

Regardless of visual approach, the algorithm only needs:
- A way for players to **recognize fitting cards** (so they can draft
  coherently)
- A way for players to **detect suppression** (so signal reading works)

Both of these can be achieved through any of the above approaches. The
algorithm itself is agnostic to how archetypes are visually communicated --
it operates on hidden fitness scores regardless.

The key question is whether the visual system should **reveal** the archetype
structure (Options 1, 4) or **obscure** it (Options 2, 3). Revealing it makes
drafting more accessible but potentially less interesting; obscuring it
rewards mastery but risks confusing new players.

---

## Universal Findings

These findings held across all 8 simulated systems:

1. **Run-to-run variety is free.** Every system achieved 5-10% card overlap
   across runs, far below the 40% target. With 360 unique cards and 30 picks,
   natural randomness provides enormous variety. No special mechanism needed.

2. **Deck concentration and convergence are mathematically opposed.** If the
   system reliably shows 2+ fitting cards per pack and the player always picks
   the best fitting card, the resulting deck is 85-95% on-archetype. The
   original 60-80% target assumed a player who sometimes picks power over fit.

3. **Commitment detection matters more than pool composition.** Several
   systems failed not because their card pools were wrong, but because they
   detected commitment too early (Model C at pick 3) or too late (Model B at
   pick 8.4). The threshold (when to start biasing packs) is the most
   sensitive parameter.

4. **Weight multiplier strength is the critical tuning lever.** The difference
   between 5x weights (misses late fitting at 1.91) and 7x weights (hits at
   2.02) is the difference between passing and failing the convergence target.

5. **The multi-archetype card percentage determines design burden.** 4
   archetypes needs only 20% multi-archetype cards. 8 archetypes needs 25-40%.
   10 archetypes needs 62%. This is the most important practical constraint
   for the card designer.
