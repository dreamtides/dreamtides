# Design 6: Hybrid and Novel Multi-Round Approaches

**Agent 6, Round 2 — V11 Algorithm Design**

---

## Key Findings

- **The refill reset is the binding constraint, not round structure.** Balanced
  full refills recover ~half the concentration gradient each round, capping any
  standard multi-round design at ~1.3-1.6x concentration. No amount of round count
  tuning escapes this ceiling without a structural change.

- **V9's 5-7x concentration came from selective removal, not AI behavior.** V9
  removed cards the player does NOT want; multi-round AI depletion removes cards
  the player DOES want. These are opposite mechanisms. To approach V9 levels, V11
  needs selective removal of irrelevant cards OR a structural alternative.

- **Per-pick replacement eliminates the refill reset problem entirely.** If
  depleted cards are replaced one-at-a-time (after each AI pick), there is no
  batch refill event to reset concentration. AI lanes cycle continuously; open
  lanes are never replenished. Concentration accumulates without interruption.

- **Asymmetric replacement is the novel concentration mechanism.** AI picks are
  replaced (maintaining AI-lane supply); player picks are NOT replaced (permanently
  depleting open-lane supply). This is selective removal by another name: instead
  of culling low-relevance cards, we simply do not restock the player's lane.

- **The market draft narrative is the strongest available.** Players see which
  archetypes disappear pick-by-pick. The signal is immediate, continuous, and
  grounded in visible AI behavior. M12 (signal-reader advantage) is structurally
  maximized.

- **One late-draft refill is required for M11'.** With only 5 S/A cards per open
  lane, an efficient player exhausts them by pick 18-20. A single underrepresented-
  biased refill at pick 20 draws from the replacement reserve to restock depleted
  archetypes, providing fresh S/A for picks 21-30.

- **Formal snapshots preserve the exploration-commitment-execution arc.** Review
  moments at picks 10 and 20 create structural phases without interrupting the
  market or resetting concentration.

---

## Three Algorithm Proposals

### Proposal A: Asymmetric Market Replacement (Champion)

**Description:** Continuous visible market where AI picks are individually replaced
from a reserve deck, but player picks permanently deplete the pool. Open lanes
concentrate monotonically; AI lanes cycle without accumulating scarcity.

**Technical Specification:**

- Starting pool: 120 cards (8 archetypes x 15). Replacement reserve: 240 cards
  (8 archetypes x 30).
- Each pick cycle: 5 AIs draft in order (each pick triggers 1 replacement card
  drawn for that AI's archetype from reserve). Player drafts last; no replacement.
  Pool shrinks by 1 per cycle. After 30 picks: pool = 90 cards.
- Round boundaries at picks 10 and 20: game freezes, shows full archetype
  availability snapshot. No structural refill; informational only.
- Round 3 boundary (pick 20): +20 cards drawn from reserve, biased toward
  archetypes currently below 10 cards in market. This restocks open lanes
  depleted by player picks without targeting specific archetypes.
- AI behavior: Level 0. Fixed archetype. Takes highest-fitness card in their lane.
  Saturation at 16 cards drafted (switches to highest-power generic).
- Player information: Full visible pool always. Archetype bars updated per pick.
  AI pick notifications by resonance category (not AI identity). Round snapshots
  at picks 10 and 20.

**Pool Composition Trajectory:**

| Moment | Player Lane (Blink) | AI Lanes | Pool |
|--------|:-------------------:|:--------:|:----:|
| Pick 1 | 15 (5 S/A) | 15 each | 120 |
| Pick 10 | 12 (4 S/A) | ~15 each | 110 |
| Pick 20 | 7 (0-1 S/A) | ~15 each | 100 |
| Pick 20 refill | 12 (1-2 new S/A) | ~15 each | 120 |
| Pick 30 | 2 (0 S/A) | ~15 each | 90 |

**Concentration mechanism:** Player takes all 5 original Blink S/A with zero AI
competition. Remaining S/A comes from the pick-20 refill (reserve has ~10 Blink
S/A in 30 cards; a 5-card refill draw yields 1-2 S/A).

**Predicted Metrics:**

| Metric | Predicted | Target | Status |
|--------|:---------:|:------:|--------|
| M3 | 2.2-2.5 | >= 2.0 | Pass |
| M10 | 1.5-2.5 | <= 2 | Pass (uncertain) |
| M11' | 2.5-3.0 | >= 2.5 | Pass |
| M6 | 72-82% | 60-90% | Pass |
| M12 | 0.4-0.6 | >= 0.3 | Pass |

---

### Proposal B: Three-Round Draft with Mid-Round Culling

**Description:** Standard 3-round structure (10 picks, full refill to 120), but
at pick 5 of each round the bottom 25% of pool cards by visible-resonance relevance
(derived from player's resonance majority so far) are removed. Level 1 status: cull
uses player picks but only visible resonance (public information).

| Metric | Predicted | Target | Status |
|--------|:---------:|:------:|--------|
| M3 | 1.8-2.2 | >= 2.0 | Marginal |
| M10 | 2-3 | <= 2 | Marginal |
| M11' | 2.2-2.8 | >= 2.5 | Marginal |
| M6 | 70-80% | 60-90% | Pass |
| M12 | 0.2-0.4 | >= 0.3 | Marginal |

### Proposal C: Two-Tier Pool (Reserve + Visible Market)

**Description:** Visible 30-card market refills from a 270-card hidden reserve
(never restocked) when market drops below 15. As AIs drain AI-lane cards from
reserve through market cycles, later refills draw progressively from open-lane
remainder. Concentration builds through reserve depletion asymmetry.

| Metric | Predicted | Target | Status |
|--------|:---------:|:------:|--------|
| M3 | 1.5-2.0 | >= 2.0 | Marginal |
| M10 | 2-4 | <= 2 | Fail |
| M11' | 2.0-2.5 | >= 2.5 | Marginal |
| M6 | 65-75% | 60-90% | Pass |
| M12 | 0.3-0.4 | >= 0.3 | Pass |

---

## Champion Selection

**Champion: Proposal A — Asymmetric Market Replacement**

Proposal B requires Level 1 reactivity and achieves only marginal metrics.
Proposal C's reserve-draw RNG creates M10 streak risk. Proposal A's concentration
is purely structural: player picks are irreplaceable, AI picks cycle. No hidden
player state, no weighted contraction. Players watch AI lanes cycle while their
open lane depletes from their own picks. M12 is structurally maximized by
full pool visibility throughout.

---

## Champion Deep-Dive: Asymmetric Market Replacement

### Round-by-Round Walkthrough

**Setup:** 5 AIs on Storm, Self-Mill, Sacrifice, Warriors, Flash. Open lanes:
Blink, Self-Discard, Ramp. Player targets Blink (Ember primary).

**Round 1 (Picks 1-10): Exploration**

AI lanes cycle (replacement drawn immediately after each AI pick). AI bars hold
at ~15. Blink, Self-Discard, Ramp bars hold at 15 — untouched by AIs. Signal is
immediate: "Open lanes are full; AI lanes are churning."

Player commits to Blink by pick 5-6. Takes 3 Blink cards by pick 10 (1 S/A, 2 C/F).
Pick 10 snapshot: Blink at 12 (4 S/A remaining), all AI lanes at ~15.

**Round 2 (Picks 11-20): Commitment**

Player takes 5 more Blink cards. By pick 18-20, all 5 original Blink S/A are gone.
Pick 20 snapshot: Blink at 7 (all C/F). Pick 20 refill: +20 cards from reserve,
biased toward archetypes below 10 cards. ~5 Blink cards added (1-2 S/A). "Market
restocked — depleted archetypes refreshed" shown.

**Round 3 (Picks 21-30): Execution**

Blink at ~12. 1-2 S/A available with zero AI competition. Player takes them all.
~2-3 S/A in final 10 picks satisfies M11' = 2.5.

### What the Player Sees

Every pick: full visible market, filterable by archetype. Per-pick AI notifications
by resonance category. Round snapshots at picks 10 and 20 with explicit counts.
The market feels like a shared resource the player is competing for — except the
competition is localized to AI lanes. Open lanes belong to whoever claims them.

### Failure Modes

**S/A exhaustion before pick 20 (High Risk).** An efficient player takes all 5
Blink S/A by pick 15-18, leaving picks 16-20 as pure C/F. Moving the refill to
pick 15 rather than pick 20 would address this at the cost of providing a long
Round 3 with fresh S/A competing with a picked-over market. Calibration required.

**Market paralysis (Medium Risk).** 120 visible cards is overwhelming compared to
a 15-card pack. Strong UI filtering by archetype and per-pick AI notifications are
essential; this design requires implementation discipline others do not.

**Refill legibility (Low Risk).** The pick-20 refill applies to all archetypes
below the threshold equally. Blink, Self-Discard, and Ramp all restock. The bias
is archetype-count-agnostic, not player-targeted.

---

## Complete Specification

| Parameter | Value |
|-----------|-------|
| Starting pool | 120 cards (8 archetypes x 15) |
| Replacement reserve | 240 cards (8 archetypes x 30) |
| Total picks | 30 |
| Round snapshots | Picks 10, 20 (informational; no batch refill) |
| Scheduled refill | Pick 20: +20 from reserve, biased toward archetypes < 10 cards |
| AI count | 5 (5 of 8 archetypes covered) |
| Open lanes | 3 |
| AI replacement | Yes: 1 card per AI pick, from reserve, same archetype |
| Player replacement | None: player picks permanently deplete the market |
| AI behavior | Level 0: fixed archetype, highest-fitness card, saturation at 16 |
| Player information | Full visible pool; archetype bars per pick; AI pick notifications by resonance; round snapshots at 10, 20 |
| Fitness model | Graduated Realistic |
| Expected M3 | 2.2-2.5 |
| Expected M11' | 2.5-3.0 |
| Expected M10 | 1.5-2.5 |
| Expected M6 | 72-82% |
| Expected M12 | 0.4-0.6 |

---

## Post-Critique Revision

### Refill Timing: Accept Pick 16, Not Pick 20

The critic's adjustment is correct and I should have caught it myself. My own
failure mode analysis identified S/A exhaustion by pick 15-18 as the highest
risk. Setting the refill at pick 20 means the window where S/A is gone (picks
16-20) is exactly the span I flagged as problematic. Moving the refill to pick
16 closes that window directly.

The tradeoff is a longer Round 3 with fresh S/A competing against a partially
exhausted market. That is the right tradeoff: a 14-pick execution phase with
some S/A is better than a 5-pick execution phase with all of it. Revised
specification: scheduled refill at pick 16, +20 cards from reserve biased
toward archetypes below 10 cards. Pick 20 becomes a snapshot boundary only.

### Market Paralysis: Filtering Is Load-Bearing, Not Optional

120 visible cards is genuinely a different UX category from a 15-card pack. I
called this Medium Risk in the failure modes section, which understates how
much implementation discipline it requires. The design functions only if
archetype filtering reduces the effective decision surface to 12-15 cards
before the player evaluates anything. The filter must be on by default and
must persist across picks. If implementation delivers a full 120-card grid
with opt-in filtering, the design fails on usability regardless of metric
performance. This is a hard constraint, not a nice-to-have.

### V9 Comparison: Structural Distinction Is Real, Resemblance Is Worth Naming

The critic is right that these are mechanically different. V9 removed cards the
player did NOT pick; Design 6 removes cards the player DOES pick. The direction
of depletion is opposite. V9 culled irrelevant supply; Design 6 depletes
relevant supply through use. Calling them the same mechanism would be wrong.

That said, both produce monotonic concentration in the player's target archetype
and both avoid weighted random processes. The family resemblance is real and
should be acknowledged when comparing simulation results. If SIM-5 underperforms,
the first diagnostic question is whether the depletion rate is calibrated
differently from V9's removal rate — they are not interchangeable.

### Hybrid B's Rolling Trend Indicators: Endorse

Design 5's rolling 5-pick trend windows adapted for a continuous market are a
direct improvement. In a pack-based draft, archetype signal accumulates at
round boundaries. In a continuous market, a player can read the full 30-pick
depletion arc at any moment, but a trend window shows velocity rather than
total. "Blink dropped 3 cards in the last 5 picks" is more actionable than
"Blink is at 9." Adding trend indicators to the archetype bars is low
implementation cost and directly improves M12 by surfacing the signal the
design is built around. I endorse this addition for SIM-5.
