# Design Agent 1: 3-Pack Classic (MTG-Inspired)

## Key Findings

- **Full balanced refills produce ~1.3-1.6x concentration** — far below V9's 5-7x and
  barely above V10's 1.7x. The refill reset problem (each refill recovers ~half the
  concentration gradient) is confirmed and structural. Balanced full refills alone cannot
  reach M3 >= 2.0.
- **The 3-round structure is the right shape.** The exploration → commitment → execution
  arc (MTG Pack 1/2/3, 7 Wonders Ages I/II/III, Blood Rage) is naturally expressed in
  3 rounds of 10 picks. The round transition is a design asset, not overhead.
- **Targeting dilution persists.** The player is still 1 of 3 open lanes after refills.
  Balanced refills restore the archetype distribution equally, partially undoing
  whatever gradient formed in the prior round. No variant of balanced full refill
  solves this.
- **A mild asymmetric refill rescues the design.** If AI-lane archetypes receive fewer
  refill cards than open-lane archetypes (justified by "the market restocks what hasn't
  sold"), the refill reset can be weakened enough to accumulate concentration round
  over round. This is not V9-style player-targeting (the player's specific archetype is
  not privileged); it is pool-state-reactive at the archetype-occupancy level — Level 0.5.
- **S/A quality masking is essential.** Round-start archetype availability bars (showing
  card counts, not quality) give signal-readers a meaningful advantage without
  collapsing the puzzle. Quality must remain hidden even when quantity is visible.
- **The "new pack" moment is a strategic recalibration point.** Research confirms that
  opening a fresh pool (like opening Pack 2 in MTG) resets decision energy, provides
  genuine new information, and creates a natural pause for reassessing commitment.
- **M12 is achievable through bars + quality masking.** Signal-readers can see which
  lanes are plentiful but cannot see which of those plentiful cards are S/A. This
  creates a genuine deduction skill layer on top of quantity reading.

---

## Three Algorithm Proposals

### Algorithm A — Pure Emergence (Full Balanced Refills)

**Description:** 3 rounds of 10 picks from a 120-card pool, refilled to 120 after each
round using balanced archetype distribution. No bias, no culling. Test whether emergent
concentration alone is sufficient.

**Technical Spec:**
- Pool: 120 cards, 8 archetypes x 15 cards each. S/A density: 33% per archetype.
- 5 AIs, each assigned one archetype. Level 0 static pick preference.
- AI pick logic: highest archetype-fitness card from preferred archetype; generic if lane
  exhausted. Deckbuilding saturation at 6 on-archetype cards (ease off, pick generics).
- Refill: After round 1 and round 2, add exactly 60 cards (7.5 per archetype) to restore
  pool to 120. Balanced: each archetype receives equal refill regardless of depletion.
- Player information: Round-start availability bars (relative counts per archetype symbol,
  not quality). AI pick hints by archetype category, not AI identity.

**Predicted Metrics:**
- M3: 1.3-1.5 (below target; refill reset prevents concentration accumulation)
- M10: 3-4 consecutive bad packs (misses <= 2 target)
- M11': 1.5-1.8 (below 2.5 target)
- M6: 55-70% (below 60-90% range)
- M12: 0.2-0.3 (borderline; bars help but quality masking limits signal-reader edge)

**Verdict:** Likely fails M3, M10, M11'. Useful as a baseline to confirm the refill
reset problem is structural, not incidental.

---

### Algorithm B — Asymmetric Refill (Level 0.5 Concentration)

**Description:** 3 rounds of 10 picks, refill after rounds 1 and 2, but refill volume
is asymmetric: AI-occupied lanes receive 50% of their balanced share; open lanes receive
100%. Framed as "the market restocks what hasn't sold." This is pool-state-reactive but
not player-reactive — the system observes AI picks by archetype (which archetypes are
occupied) but does not observe the player's specific archetype.

**Technical Spec:**
- Pool: 120 cards, same as Algorithm A.
- 5 AIs, Level 0 static assignment.
- Refill logic: Determine which 5 archetypes are AI-occupied. Distribute 60 refill cards
  as follows: 3.75 cards per AI-lane archetype (50% of 7.5 balanced share), 12.5 cards
  per open-lane archetype (100% base + 50% redistribution from AI lanes). Total refill
  = 5 x 3.75 + 3 x 12.5 = 18.75 + 37.5 = 56.25 ≈ 56 cards (trim to maintain pool
  ceiling at 120).
- Pool does not grow beyond 120; excess allocation rounds down.
- AI pick logic: same as Algorithm A.
- Player information: Same bars + archetype-category AI hints. Bars now show meaningful
  concentration gradient by round 2.

**Predicted Metrics:**
- M3: 1.9-2.3 (near or above target; open lanes accumulate concentration across rounds)
- M10: 1-2 consecutive bad packs (meets target)
- M11': 2.2-2.7 (meets or approaches target)
- M6: 65-80% (within range)
- M12: 0.3-0.5 (signal-readers see larger bars in open lanes and commit accordingly)

**Verdict:** Champion candidate. The asymmetric refill is the minimum additional mechanism
required to escape the refill reset trap. The narrative framing (market restocks slow-
moving inventory) is clean and does not require the player to be targeted specifically.

---

### Algorithm C — Declining Refills with Asymmetric Bias

**Description:** 3 rounds with declining refill volume. Round 1 → 2: full 60-card refill
with 2:1 open-to-AI-lane bias. Round 2 → 3: 30-card refill with 3:1 bias. Round 3: no
refill. Concentration ramps naturally — open in round 1, concentrated by round 3.

**Technical Spec:**
- Pool: 120 cards starting. 5 AIs, Level 0.
- Round 1 → 2 refill: 60 cards total. AI lanes: 4 cards each (20 total). Open lanes:
  13-14 cards each (40 total). Pool returns to ~110 (slight shrinkage because 6 players
  x 10 picks = 60 removed, 60 added; pool stays at 120 before round 2 but with shifted
  archetype distribution).
- Round 2 → 3 refill: 30 cards total. AI lanes: 2 cards each (10 total). Open lanes:
  6-7 cards each (20 total). Round 3 starts with ~90 cards, concentrated toward open
  lanes.
- AI pick logic: Same as B, plus round-aware escalation — AIs prioritize archetype
  cards more aggressively in round 3 (saturation threshold raised from 6 to 8 by round 3).
- Player information: Round-start bars, plus an "incoming supply" preview at round end
  showing whether the next refill leans toward particular resonance types (not specific
  archetypes).

**Predicted Metrics:**
- M3: 2.0-2.5 (meets target; declining refills let concentration accumulate)
- M10: 1-2 (meets target)
- M11': 2.5-3.0 (meets target; round 3 pool is concentrated)
- M6: 70-85% (within range)
- M12: 0.3-0.5 (refill preview gives signal-readers additional edge in round transitions)

**Verdict:** Stronger ceiling than B but more mechanically complex. The declining refill
schedule requires a second parameter set and the "incoming supply preview" adds UI
complexity. If B passes M3/M11', prefer B for simplicity.

---

## Champion: Algorithm B — Asymmetric Refill

**Justification:** Algorithm B is the minimum mechanism that solves the refill reset
problem without adding round-structure complexity. The asymmetric refill is a single
additional parameter (AI-lane refill fraction = 0.5). It does not require declining
volumes, preview UI, or round-aware AI escalation. The narrative framing ("the market
restocks slow-moving inventory") is internally consistent and Level 0 with respect to
player behavior — the system observes archetype occupancy, which is a structural property
of the draft setup, not a reactive read of the player's picks.

Algorithm C is likely stronger on metrics but harder to tune and explain. The critic
and simulation rounds should validate whether B clears M3 = 2.0 comfortably; if not,
C's declining volume can be grafted onto B's bias logic.

---

## Champion Deep-Dive: Algorithm B

### Round-by-Round Walkthrough

**Round 1 (Picks 1-10): Exploration**

Pool opens at 120 cards. Each archetype has 15 cards; 5 S/A and 10 C/F. Five AIs each
assigned one archetype. Three archetypes are open (no AI).

Picks 1-5: The player sees a rich pool. Every archetype has cards. Signal-readers
observe that bars for open-lane archetypes are slightly higher than bars for AI-lane
archetypes (no depletion yet in open lanes, light depletion beginning in AI lanes).
Picks 1-5 produce M1 and M2 behavior: multiple archetypes have S/A cards (M1), no
single archetype is yet dominant (M2). Convergence should begin pick 5-8.

Picks 6-10: AI depletion in AI-lane archetypes is now visible in the bars. AI-lane
S/A cards are being taken; open-lane S/A cards accumulate relative to pool. Player
committing to an open lane begins to see M3-quality packs. Player still sees some
off-archetype options (M4 satisfied by the 3-open-lane structure).

End of round 1: 60 cards removed. Pool at 60 cards, archetype distribution skewed: AI
lanes have ~9-10 cards each (S/A depleted), open lanes have ~11-13 cards each.

**Refill Moment (Between Round 1 and Round 2)**

Pool state displayed as a round-end summary. Player sees final round-1 bars — open lanes
clearly fuller. This is the "new pack" moment: a natural strategic recalibration.

Refill logic: Add 56 cards. AI lanes receive 3.75 cards each (~19 total). Open lanes
receive 12.5 cards each (~37 total). Pool rises to ~116 cards, with open-lane archetypes
now at 23-26 cards each and AI-lane archetypes at 13-14 cards each.

Round 2 starts: Player sees bars where open lanes are markedly richer (23-26 cards vs
13-14). The gradient is now visible and unambiguous. Signal-reader advantage: a player
who has not committed can now see clearly which lanes are open. Committed player
advantage: a player who committed in round 1 is already positioned for the concentrated
open lane.

**Round 2 (Picks 11-20): Commitment**

Pool starts at ~116 with strong open-lane concentration. AI lanes continue to deplete
their (smaller) stock. Open lanes deplete slowly (player is the only drafter). M3
target: 2.0+ S/A cards per pick for committed player in an open lane. With 23-26 cards
in the open lane at round start and 5 of the original 15 already held by the player,
the remaining pool within the player's lane is ~18-21 cards, of which ~8-10 are S/A
(refill added ~3.75 S/A to the open lane). Probability of S/A on any given pick from
the open lane: ~45-50%. Probability of S/A per pack (across all visible cards the player
can choose from, weighted toward their archetype): meets M3 comfortably.

Picks 16-20: Late commitment phase. Pool continues depleting. AI saturation begins
(some AIs have 6+ archetype cards and ease off). Player's open lane remains the richest.

End of round 2: ~56 cards removed. Pool at ~60 cards, heavily skewed toward open lanes.

**Refill Moment (Between Round 2 and Round 3)**

Second round-end summary. Open-lane dominance is now very visible. Refill adds 56 cards
with same 50/100% ratio. Open lanes receive another ~12.5 cards each. Round 3 starts
with open lanes at ~35-40 cards each and AI lanes at ~10-12 cards each.

**Round 3 (Picks 21-30): Execution**

Highly concentrated pool. Player competes primarily against refill-supplied cards in
their own lane. AI lanes are nearly exhausted of S/A cards (3 rounds of depletion with
minimal refill). M11' target (picks 20+): 2.5 S/A per pick. With ~30-35 S/A-eligible
cards in the player's open lane at round 3 start (accumulated across 3 refill cycles),
probability exceeds M11' target.

### What the Player Sees

- **Pick view:** Cards displayed. Resonance symbols visible. Quality (S/A/C/F rating)
  not labeled — player evaluates from card effects and names.
- **Archetype bars:** Always visible, updated after each AI pick. Shows relative card
  count per resonance symbol. Open lanes are visibly richer by pick 5 and markedly
  richer by round 2.
- **AI pick hints:** After each AI takes a card, a brief message appears: "A Tide card
  was drafted." No AI identity revealed. Player accumulates a sense of which archetypes
  are contested without building an exact tracking table.
- **Round transition:** Round-end screen shows pool state, confirms what archetypes were
  depleted, previews incoming refill as "the market restocks slower-moving cards."

### Pool Composition Evolution

| Moment | Pool Size | Open Lane Cards Each | AI Lane Cards Each |
|--------|-----------|---------------------|--------------------|
| Round 1 start | 120 | 15 | 15 |
| Round 1 end (pre-refill) | 60 | 12 | 8 |
| Round 2 start (post-refill) | 116 | 25 | 13 |
| Round 2 end (pre-refill) | 60 | 15 | 7 |
| Round 3 start (post-refill) | 116 | 38 | 12 |
| Round 3 end | 60 | 25 | 5 |

Open lanes accumulate from 15 → 25 → 38 across round starts. AI lanes deplete from
15 → 13 → 12 (partially replenished but never recovering to starting density).

### Failure Modes

**S/A cycling in open lanes:** Refills add S/A cards to all archetypes including open
ones, but the player is also taking those S/A cards. If the player picks efficiently
(always taking the best open-lane card), the refill-added S/A cards are consumed by the
player rather than accumulating. Net effect: player's quality is maintained, not
compounded. This limits M11' ceiling but does not cause failure.

**Two players converging on same open lane:** If a second player (in a multiplayer
scenario) also targets the same open lane, open-lane depletion mirrors an AI lane.
Design note: Dreamtides is singleplayer against AI; this failure mode is not applicable.

**AI saturation misfiring:** If AIs saturate too early (hit their 6-card threshold by
round 2) and stop drafting their archetype, AI-lane depletion slows. Open-lane
concentration becomes less extreme (AIs are no longer enriching the gradient). Mitigation:
raise saturation threshold or make it round-aware (higher threshold in round 3).

---

## Complete Specification

| Parameter | Value |
|-----------|-------|
| Pool size | 120 cards at round start |
| Rounds | 3 |
| Picks per round | 10 |
| Total picks | 30 |
| Refill quantity | 56 cards after rounds 1 and 2 |
| Refill bias | AI-lane archetypes: 3.75 cards each; open-lane archetypes: 12.5 cards each |
| Refill narrative | "The market restocks slow-moving inventory" |
| AI count | 5 |
| AI archetype assignment | Static, Level 0, pre-assigned before draft |
| AI pick logic | Highest archetype-fitness card from preferred archetype; saturation threshold 6 |
| AI saturation behavior | After 6 on-archetype cards, pick generics or secondary-archetype cards |
| AI pick information shown | Archetype category hint per pick ("A Tide card was drafted"), no AI identity |
| Player information | Archetype availability bars (quantity, not quality); round-start summary |
| S/A density | 33% per archetype (5 S/A, 10 C/F per archetype per 15-card starting batch) |
| Dual-symbol cards | ~10% of pool |
| Generic cards | ~11% of pool |
| Fitness model | Graduated Realistic |
| Player strategies | Archetype-committed, power-chaser, signal-reader |
| M3 prediction | 1.9-2.3 |
| M11' prediction | 2.2-2.7 |
| M12 prediction | 0.3-0.5 |

---

## Post-Critique Revision

### On the Convergence with Design 4

The critic is correct. Algorithm B and Design 4's Static Open-Lane Multiplier are the
same design expressed through different parameterizations.

My framing: AI lanes receive 50% of their balanced share (3.75 cards each), open lanes
receive 100% base plus redistribution from AI lanes (~12.5 cards each). The resulting
ratio is approximately 3.3:1 open-to-AI per card.

Design 4's framing: open lanes receive a 1.7x multiplier, AI lanes receive the remainder.
With a 60-card refill and 3 open lanes and 5 AI lanes, this produces roughly 10.7 cards
per open lane and 4.3 per AI lane — a ratio of 2.5:1.

These are not identical values, but they are structurally indistinguishable. Both fix bias
at draft initialization from archetype occupancy. Both predict M3 in the 1.9-2.3 range.
Neither is player-reactive. The difference is a tuning dial, not a design difference.

I accept this. There is no meaningful distinction worth defending. For simulation
purposes, treating Algorithm B as equivalent to SIM-2 (Design 4's specification) is
correct. If both agents submit separate specs, the simulator should run one and apply
results to both.

### On Level 0 Classification

The critic confirmed open-lane-biased refills are honest Level 0. I had described my
design as "Level 0.5" in the findings section and as "Level 0" in the champion
justification — an inconsistency. The correct classification is Level 0. The refill bias
is determined by archetype occupancy set at draft initialization, which is a structural
fact known before the first pick. It never observes the player's pick sequence. I was
being overly cautious in labeling it 0.5. The market-restocks framing holds without
reservation.

### Algorithm B vs. Algorithm C: Updated Recommendation

Given that Algorithm B is SIM-2 and the critic's Hybrid A (combining Design 4 Proposal
C's graduated bias with declining volume and Design 5's information system) captures the
spirit of Algorithm C, my recommendation shifts.

**Algorithm B remains the right champion for the base simulation.** It is the minimal
mechanism, it answers whether asymmetric refill alone is sufficient, and it has a clean
specification that does not depend on Design 5's information layer.

**Algorithm C is better represented by Hybrid A than by my original spec.** My Algorithm
C added declining volume and a refill preview UI but no full information system. Hybrid A
adds Design 5's full information layer, which is a materially stronger change than a
refill preview. If the simulation proceeds to Hybrid A (SIM-3), that is strictly an
improvement over my Algorithm C — and I have no objection to that framing. I would not
advocate for running my Algorithm C as a distinct simulation target.

**Final recommendation: Submit Algorithm B as-specified for SIM-2. Defer to Hybrid A
(SIM-3) for the stronger variant.** If SIM-2 clears M3 = 2.0 comfortably, the additional
complexity of SIM-3 may not be necessary. If SIM-2 lands at the low end of 1.9-2.0, the
graduated bias and declining volume in SIM-3 are the next tuning levers to pull.
