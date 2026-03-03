# Multi-Round Draft Formats: Research Findings

## Overview

This document examines how existing games structure multi-round / multi-pack
drafts, with emphasis on pacing, decision points, signal reading, and pool
sizing. Findings are organized by design question relevant to V11's multi-round
refill structure.

---

## MTG's 3-Pack Structure: What Changes Between Rounds

MTG booster draft uses 3 packs of 15 cards. Each pack passes in alternating
directions (left, right, left), creating distinct strategic phases:

**Pack 1 — Open Exploration.**
Early picks (1-7) are the primary window for identifying which archetypes are
available. Drafters take the best cards and signals emerge from what is passed.
Pack 1 is when signals are most meaningful because players have not yet locked
in. Expert advice consistently places the optimal commitment window at picks
2-7 of pack 1. Critically, Pack 1 pick 9 (the first "wheel" — the pack
returning to its opener) is considered the single most information-dense pick
in the entire draft: it reveals, with certainty, what 7 other drafters
declined to take.

**Pack 2 — Commitment and Refinement.**
The passing direction reverses, so signals now arrive from different neighbors.
This creates a noticeable experience shift: cards feel fresher because they
come from different directions. Players who did not commit during pack 1 face
significant risk — switching during pack 2 leaves too little time in pack 3 to
build a coherent deck. Strong players treat pack 2 as a refinement phase:
filling gaps in the committed archetype, picking up fixing and late-round
enablers. Wheeling good cards in an archetype during pack 2 — even from a
different direction — remains a reliable signal that the archetype is open.

**Pack 3 — Consolidation and Urgency.**
With roughly 14 picks remaining, there is no room to pivot. Players lock in
their strategy and compete for the best available cards in their lane. The
experience shifts from exploratory to competitive and urgent. Opening a strong
rare in pack 3 can reward a player who stayed flexible through pack 2, but
this is high-variance; expert advice is to be committed by pack 3 pick 1 or 2.

**The "new pack" moment.** Opening a fresh pack creates a psychological reset.
The player sees 15 new cards at once, offering choices that the depleted pack 2
could not. This freshness is a design asset: it breaks up monotony, resets
decision energy, and creates a clear structural boundary. It also carries
information — cards in the new pack have not been touched by any drafter, so
they reflect the format's underlying distribution, not the table's preferences.

**Key structural insight:** The 3-pack structure works because each pack has a
distinct emotional texture — exploration, commitment, consolidation — and the
directional alternation ensures players receive signals from all neighbors over
the course of the draft. Three rounds is not arbitrary; it matches the natural
arc of exploration → commitment → execution.

---

## Other Games: Multi-Round Draft Structures

### 7 Wonders — 3 Ages, Qualitatively Distinct Decks

7 Wonders uses 3 Ages of 7 cards each (6 picked, 1 discarded). Each Age uses
an entirely different deck with different card types:

- **Age I**: Resource foundation cards (brown and grey). Players build the
  economic base for later ages.
- **Age II**: More resource cards and economic cards (yellow). Players complete
  their resource engine.
- **Age III**: No resource cards. Exclusively scoring-oriented guilds,
  science, military, and civic cards. This is when resources convert to points.

After each Age, military conflicts resolve between neighbors, creating a
mid-draft evaluation moment.

The Age transition does two things simultaneously: it invalidates the prior
strategy's card type (no brown cards in Age III) and escalates the strategic
stakes (guilds are high-variance high-reward). This forces players to mentally
shift modes each round, keeping engagement high throughout.

**Passing direction also alternates:** Age I passes left, Age II passes right,
Age III passes left. This ensures every player interacts with every neighbor
over the course of the game.

The 7 Wonders structure demonstrates that qualitatively different card pools
per round can sustain distinct strategic phases. Players do not merely draft
the same cards faster — each round has a different purpose.

### Blood Rage — 3 Ages, Escalating Card Power

Blood Rage uses 3 Ages, each with its own draft pool. Players draft 8 cards
per age (selecting 6, discarding the last 2). Packs pass between players.

Card types span combat upgrades (red), quests (green), and clan enhancements
(black). Crucially, each Age's card pool is more powerful than the previous
one, creating an escalating stakes arc. Early ages set up the strategy; later
ages deliver the payoff. Drafting a combat-heavy strategy in Age 1 that cannot
be sustained in Ages 2-3 is a strategic failure.

Blood Rage also illustrates the "commit early or pivot" tension clearly: your
clan's fixed starting stats push you toward particular strategies, but the
draft can reward flexibility if a high-value card type is flowing.

### Sushi Go — 3 Rounds, Same Card Pool, Pure Hand-Reading

Sushi Go uses 3 rounds with 10 cards per round (2-5 players). All card types
are present in every round. There is no qualitative shift between rounds — the
tension comes from hand memory (tracking which cards you have already seen)
and the fundamental scarcity of certain card types (e.g., puddings, wasabi).

Sushi Go shows the lower bound on round complexity: with the same pool each
round and no directional alternation, the main skill axis is pure memorization
of what cards are circulating. This works for a light party game but provides
limited strategic depth for a deckbuilder.

**Relevant finding:** Sushi Go's 10-card rounds give each player roughly 8-9
choices before the pack is exhausted, with 1-2 cards discarded. At 2-5
players, this means 10-45 visible cards per player per round — a wide variance
that affects how much information is actually available.

### Flesh and Blood — 3 Packs, 14-16 Picks Per Pack

FaB draft uses 3 booster packs of 16 cards (14 playable), passing
alternating left-right-left. Between packs, a 60-second review period allows
players to examine their full drafted pool before the next pack opens.

The review pause is a structural decision point not present in MTG, where
packs open in sequence without formal breaks. This review moment functions as
a forced strategy recalibration — players look at what they have, decide what
they need, and approach the next pack with updated priorities. This is directly
relevant to multi-round refill design: the refill boundary itself can serve as
a natural review and recalibration moment.

---

## Pool Size and Scarcity/Variety Balance

### The Standard: 15 Cards × N Players

MTG's pack of 15 cards for 8 players means each drafter sees approximately
5-7 cards from any given pack (depending on where in the pass order they sit).
This creates meaningful scarcity — not everything that interests you will still
be available — while still providing enough choices that most picks feel active.

### Fewer Players: Smaller Packs

When drafting with fewer players, standard advice is to reduce pack size:
- 6+ players: 3 packs × 15 cards
- 5 players: 4 packs × 12 cards
- 4 players or fewer: 5 packs × 9 cards

The rationale: fewer players means more cards wheel (return to their opener),
reducing variety and making the draft feel stale. Smaller packs compensate by
cycling faster, so each player sees more distinct packs total. The total card
pool seen stays roughly constant (~200 cards), but the experience is more
varied.

**For Dreamtides (6 participants total: 1 player + 5 AIs):** This maps closely
to the 6-player bracket, suggesting a natural pack size of 12-15 cards visible
at any given decision point. The total pool size (120-150 for Config B) is
within the correct range for the player count.

### Wheeling as an Information Event

In a pool that all drafters share, the return of cards to the pool after
others have picked from it is the equivalent of "wheeling" in a pass-style
draft. Cards that remain after each pick cycle represent what competitors did
not want — a concrete signal about which archetypes are crowded.

With 5 AIs and 1 player each taking 1 card per turn, a starting pool of 120
cards loses 6 cards per pick cycle. After 10 picks (one round), 60 cards
remain (50% depletion). This is aggressive depletion — more than in an 8-player
pack-pass draft, where 8 picks deplete a 15-card pack to 7 cards (53%).
The per-pick depletion rate is similar, but the pool is much larger, so each
individual pick decision has lower apparent scarcity ("there are still many
cards left"). This can reduce the urgency that drives good decisions.

**Design implication:** Smaller starting pools (Config A: 80-100 cards) may
create a more viscerally scarce experience within each round, making the
player feel the competition more acutely. Larger pools (Config C: 180-240)
feel more like browsing a market with competitors.

---

## Number of Rounds: Is 3 the Right Number?

Based on the survey of existing games, three rounds is the dominant structure
across games with distinct round phases (MTG, 7 Wonders, Blood Rage, Sushi Go,
FaB). This is not accidental — three rounds maps to a natural dramatic arc:

1. **Act 1 (Exploration):** Wide possibilities, read what's available, stay
   flexible, identify your direction.
2. **Act 2 (Commitment):** Narrow focus, commit to a lane, fill gaps, respond
   to mid-game information.
3. **Act 3 (Execution):** No pivoting, compete for best cards in your lane,
   consolidate and finish.

Two rounds (Config C) eliminates the middle act. Players must commit during or
immediately after round 1, with no exploration phase and a very short window
for mid-draft learning. This approximates the V10 single-pool design and likely
shares its failure modes: not enough time to build a signal-reading advantage.

Five or six rounds (Config A) creates more decision boundaries but may dilute
the strategic arc. Each round is too short (5-6 picks) to establish a clear
pattern before the next refill. The frequent fresh pools may feel more like
"market restocking" than a meaningful phase transition. More rounds also
increase cognitive overhead — tracking pool state across 6 rounds is harder
than across 3.

**Assessment:** Three rounds (Config B) has the strongest structural precedent
across games. It provides a complete exploration → commitment → execution arc,
creates two distinct information events (round 2 start and round 3 start), and
is simple enough to explain immediately.

---

## Reading the Table When New Cards Arrive

### The Signal Coherence Problem

In a traditional pass-draft, signals accumulate within a pack and persist into
the next. If an archetype is open in pack 1, it is typically still open in
pack 2 (the same 7 drafters are at the table; their preferences did not change).
New packs provide new card opportunities, not new table information.

In a shared-pool draft with refills, each refill partially resets the pool
state. This creates a "signal coherence" question: does reading the pool in
round 1 tell you anything useful about round 2?

The answer depends on refill composition:
- **Balanced refills (equal per archetype):** Round 2's pool looks like a
  freshly shuffled version of the starting pool, restocked uniformly. The
  signal from round 1 remains valid because round 1 depletion reflects AI
  preferences that have not changed. An archetype that was depleted in round 1
  will be depleted again in round 2 — the refill temporarily restocks it, but
  the same AIs will drain it again. The player's read from round 1 transfers
  cleanly.
- **Random-draw refills:** Round 2's pool composition is less predictable.
  The player must re-evaluate at the start of each round, which increases
  cognitive load and reduces the long-term value of early signal reading.
- **Underrepresented-bias refills:** Round 2 preferentially restocks depleted
  archetypes, partially counteracting the concentration built in round 1. This
  could feel like the game is fighting the player's correct read.

### The Round-Start Snapshot as a Decision Point

7 Wonders' Age transitions and FaB's between-pack review both demonstrate
that formal pauses between rounds create valuable decision moments. When the
new pool arrives (or the new pack is opened), players have a concentrated
moment to recalibrate — to take stock of what they have, what the new pool
offers, and whether their strategy is still correct.

This round-start decision point is one of the primary sources of "reading the
table" experience in multi-round play. In a shared-pool design, the round-start
snapshot — seeing what cards the AIs depleted and what the refill added — is
the functional equivalent of reading signals from passed packs in MTG.

**Design implication:** Making the pool state legible at round start (via UI
or structural design) is likely more important in a shared-pool draft than in
a pass-style draft, because the player cannot infer table state from the order
of cards they receive. They need explicit or semi-explicit information about
what was taken.

---

## Key Synthesis for V11

**On round count:** Three rounds is the right number. The exploration-commitment-
execution arc is natural, well-precedented across games, and simple to explain.
Two rounds collapses the middle phase; five-plus rounds dilutes it.

**On pool size:** For 6 total drafters, a starting pool of 120-150 cards (Config
B) is consistent with precedent for 6-player groups. The key variable is not
total pool size but picks-per-round: 10 picks from 120 cards creates meaningful
depletion without exhaustion.

**On round transitions:** The new-pool moment should be treated as a design
asset. It is a natural pause, a decision point, and a moment of fresh
information. The 7 Wonders and FaB examples show this pause is worth
formalizing rather than eliding.

**On signal coherence:** Balanced refills preserve round-1 signal validity
better than random or biased refills. Because the same AIs draft the same
archetypes in every round, the player's early read transfers directly: an
archetype that was open in round 1 will remain open in round 2 unless the
player's situation changed. This is a structural advantage of level-0 AI
design — AI preferences are stable, so the signal never misleads.

**On the commit window:** Existing games consistently place the optimal commit
point in the first third to first half of the draft (picks 2-7 of pack 1 in
MTG; early Age 1 in 7 Wonders). For Dreamtides at 30 picks over 3 rounds,
this maps to picks 2-10 (rounds 1 and early round 2). The current V11 target
of M5 = convergence at pick 5-8 is consistent with this literature.

**On pack size for small groups:** When the draft table is smaller than 8,
reducing pack size (and adding more packs) is the standard recommendation to
preserve signal quality and reduce stale wheeling. Config A (small/frequent)
applies this principle — frequent small pools cycle fast and reduce the chance
of the same low-quality cards appearing at every decision point.

---

*Sources:*
- [Signals in Booster Draft — Magic: The Gathering](https://magic.wizards.com/en/news/feature/signals-booster-draft-2015-01-19)
- [Booster Draft, Part 3 — Magic: The Gathering](https://magic.wizards.com/en/news/feature/booster-draft-part-3-2015-02-02)
- [Drafting 101: Understanding Signals — Magic: The Gathering](https://magic.wizards.com/en/news/feature/drafting-101-understanding-signals-2016-04-12)
- [How to Stay Open in MTG Draft — Bolt the Bird](https://boltthebirdmtg.com/how-to-stay-open-in-mtg-draft-a-limited-guide/)
- [How to Wheel in Drafts — MTG Arena Zone](https://mtgazone.com/how-to-wheel-in-drafts/)
- [Understanding The Wheel While Drafting — Star City Games](https://articles.starcitygames.com/articles/understanding-the-wheel-while-drafting/)
- [7 Wonders Strategy Guide — Cardboard Mountain](https://cardboardmountain.com/7-wonders-strategy/)
- [7 Wonders Strategic Analysis — Board Game Business](https://boardgame.business/7-wonders-strategic-analysis/)
- [Blood Rage (board game) — Wikipedia](https://en.wikipedia.org/wiki/Blood_Rage_(board_game))
- [Flesh and Blood: Booster Draft](https://fabtcg.com/en/resources/gameplay-formats/booster-draft/)
- [Booster Draft — Lucky Paper](https://luckypaper.co/resources/formats/booster-draft/)
- [Optimal Draft Variants for Less than 8 Players — Riptide Lab](https://riptidelab.com/forum/threads/optimal-draft-variants-for-less-than-8-players.1881/)
- [The Game Mechanics: Card Drafting — Bert.games](https://www.bert.games/post/card-drafting)
- [A Step by Step Guide to Reading Signals — Cardsphere](https://blog.cardsphere.com/a-step-by-step-guide-to-reading-signals/)
