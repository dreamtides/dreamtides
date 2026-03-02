# Model D: Variety-First Draft System

## Core Idea

N=8 archetypes with **per-run pool restriction** (2 archetypes suppressed per
run, leaving 6 "active"), a **semi-explicit starting signal** (player sees a
starting card hinting at pool composition), and **adaptive depletion** (unpicked
cards from packs are partially removed, creating emergent mid-draft signals).
The system is optimized so that every run pushes the player toward different
archetypes and rewards observation.

**Player-facing explanation:** "Each quest draws from a shifting pool of
strategies -- pay attention to what appears early, because it tells you what's
plentiful."

## Why N=8

Q1 analysis shows N=6-8 is the operational sweet spot. N=8 specifically gives:
- 28 archetype pairs for combinatorial replayability
- High early-draft diversity (90% of random packs show 3+ archetypes)
- Enough S/A cards per archetype (~45 S-tier, ~135 S+A) for internal variety
- When restricted to 6 active archetypes per run, each active archetype gets a
  ~33% boost in effective density (360 cards concentrated into 6 instead of 8)

The restriction mechanism is the key variety lever. With 8 total and 6 active
per run, there are C(8,2) = 28 possible suppression pairs, meaning 28
structurally distinct run configurations before any card-level randomness.

## Card Fitness Distribution

360 unique cards, each with fitness scores in all 8 archetypes:

- **40% Narrow Specialists (144 cards):** S in 1, B in 1-2, C/F elsewhere.
  These are archetype-defining cards. 18 per archetype at S-tier.
- **30% Specialists with Splash (108 cards):** S in 1, A in 1-2, B in 1-2.
  The workhorse convergence cards. ~13-14 per archetype at S-tier, with each
  also being A-tier in 1-2 neighbors.
- **12% Multi-Archetype Stars (43 cards):** S in 2 archetypes, B in 2-3.
  Concentrated at archetype-pair intersections. Clustered overlap topology:
  each archetype has 2-3 "neighbor" archetypes sharing more stars.
- **13% Broad Generalists (47 cards):** A in 2-3, B in 3-4, S in none.
  Flexible filler that prevents brick packs.
- **5% Universal Stars (18 cards):** S in 3+ or raw power 9+. Rare/legendary.
  Everyone wants these; they create contested picks.

**Multi-archetype percentage:** ~42% of cards are S or A in 2+ archetypes
(108 splash + 43 stars). This is at the high end of Q2's recommended range but
justified because pool restriction effectively reduces the active multi-archetype
pool per run.

**Per-archetype totals:** Each archetype has ~45 S-tier cards (18 narrow + 14
splash + ~5 multi-star + ~8 universal-attributed). With A-tier from splash cards
aimed at it and generalists, each archetype has roughly 70-80 S+A unique cards.
In the pool (~1000 entries with copies), roughly 200-240 entries per archetype
at S/A tier = ~22% of pool. With 6 active archetypes, suppressed archetype
cards effectively become filler, boosting active archetype density to ~28%.

## Pack Construction: Adaptive Weighted Sampling with Depletion

**Phase 1 (Picks 1-5): Exploration mode.**
Draw 4 cards with uniform weights from the pool. No archetype bias. The pool's
natural composition (with 2 suppressed archetypes) creates implicit signals --
cards from suppressed archetypes appear less often. The starting card provides a
semi-explicit hint.

**Phase 2 (Picks 6+): Convergence mode with adaptive ramp.**
Once the player has 3+ S/A-tier cards in one archetype (commitment detected),
apply a weight multiplier to cards fitting that archetype. The multiplier ramps:
- Pick 6-10: 5x weight for S/A cards in committed archetype
- Pick 11-20: 6x weight
- Pick 21-30: 7x weight

One pack slot (of 4) is always drawn from outside the committed archetype with
a bias toward high-power or S-tier-in-other-archetype cards. This guarantees
the splashable target.

**Depletion mechanism:** When a 4-card pack is shown and the player picks 1,
each of the 3 unpicked cards has a 40% chance of being removed from the pool
entirely. This creates emergent signal reading: archetypes whose cards keep
appearing but not being picked gradually thin out, while the player's committed
archetype thins naturally from being drafted. Over 30 picks, roughly 36
additional cards are removed (30 picks x 3 unpicked x 0.4), thinning the pool
by ~3.6% -- enough to create detectable signals without catastrophic depletion.

## Variety Mechanisms (Distinguishing Feature)

**Layer 1 -- Archetype Restriction (structural variety):** With 2 of 8
archetypes suppressed per run, the player confronts a fundamentally different
strategic landscape each time. Cards that are S-tier in a suppressed archetype
still appear but function as B/C-tier filler, changing their effective value.
This means the same card can be a first-pick bomb one run and a 20th-pick filler
the next.

**Layer 2 -- Starting Signal (explicit signal):** At run start, the player sees
3 cards and keeps 1. These cards are drawn from the active archetype pool with
a bias toward multi-archetype stars. The kept card reveals which archetypes are
active (experienced players will learn which archetype pairs each card bridges).
The starting card is free -- it does not count as a draft pick.

**Layer 3 -- Depletion Signals (implicit signal):** As the draft progresses,
frequency shifts become detectable. If Tokens cards appeared in 4 of your first
5 packs but only 1 of your next 5, an expert player infers the Tokens pool is
thinning (possibly because Tokens shares cards with a suppressed archetype).
This rewards pattern recognition across multiple runs.

**Layer 4 -- Clustered Overlap (pivot variety):** Archetype neighbor topology
means pivoting is cheap to neighbors and expensive to distant archetypes. Since
different archetypes are suppressed each run, the "easy pivot" options change.
A player who starts Reanimator can pivot to Sacrifice (neighbor) cheaply, but
only if Sacrifice is active this run.

**Signal reliability:** The starting card is ~80% reliable (it always indicates
active archetypes, but the "best" archetype may still be something else based on
pool composition). Depletion signals are ~60-70% reliable and improve with
experience. This matches Q4's insight that 70-80% reliable signals create the
best decision-making tension.

## Archetype Frequency Control

With 2-of-8 suppressed uniformly at random, each archetype is active in 75% of
runs. Combined with pool randomization and depletion dynamics, no archetype
should dominate. The starting card mechanism prevents the player from always
choosing the same archetype even when it's active, because the starting card
options vary and nudge toward different archetypes each run.
