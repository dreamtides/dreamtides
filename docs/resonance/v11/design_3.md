# Algorithm Design 3: Declining Refills (Natural Concentration Ramp)

**Agent 3 — Round 2 Design**

---

## Key Findings

- **Balanced full refills reset ~half the concentration gradient each round.**
  The remediation research quantifies this: a balanced refill reduces the
  open-lane/AI-lane ratio from 1.18x back to 1.11x. Over 3 rounds, cumulative
  concentration plateaus at ~1.3-1.6x — no better than V10's 1.7x. Full
  balanced refills are structurally incompatible with V9-level concentration.

- **Declining refills let AI depletion outrun replenishment.** If each refill
  adds fewer cards than AIs removed in the prior round, the net pool shrinks
  and open-lane density compounds across rounds rather than resetting. This is
  the mechanism that avoids the "refill reset" failure mode.

- **The 100/75/50 schedule with 4 rounds is the sweet spot.** Spreading the
  decline across four 7-8 pick rounds produces starting pools of 120, 120,
  108, 87. No round risks exhaustion; each boundary event is smaller than the
  last; the final round's open-lane gradient reaches ~4-6x.

- **Late-round thinning feels like natural scarcity, not restriction.** A
  visibly shrinking shared pool reads as "the supply is running low" — the
  opposite of V9's invisible filter. The player sees the depleted AI lanes,
  understands why their lane is rich, and earns the late-draft concentration
  through correct early-round reading rather than receiving it automatically.

- **AI saturation mechanics are required in the final round.** If AIs pick at
  full rate from an 87-card pool, 5 AIs × 7 picks = 35 removals. Saturation
  (threshold: 7 cards for round 4) reduces this to ~20 removals, keeping the
  pool above 45 cards and preventing late exhaustion.

- **Round-start pool summaries create a genuine skill axis without revealing
  too much.** Showing archetype card counts at each round start gives the
  player the equivalent of "reading passed packs" — but in a shared-pool
  format where no packs are literally passed. The counts narrow the field but
  do not solve the pick: "Tide: 18 cards" still requires choosing Warriors vs.
  Sacrifice.

- **M12 compounds across rounds.** A signal-reader who commits to an open lane
  in Round 1 benefits from three rounds of widening gradient. A player who
  commits incorrectly and pivots in Round 2 faces a smaller pool with fewer
  correction opportunities. Correct early reading is disproportionately
  valuable — producing M12 >= 0.35.

---

## Three Algorithm Proposals

### Proposal A: 3-Round Symmetric Decline (100/60/30)

**Description:** Three 10-pick rounds with refills at 100%, 60%, and 30% of
consumed cards.

**Technical Spec:** Starting pool 120. R1: 60 consumed, refill 100% (60
added), R2 starts at 120. R2: 60 consumed, refill 60% (36 added), R3 starts
at 96. R3: 60 consumed, 36 remain. Balanced refills throughout. 5 Level-0 AIs
with saturation at 8 cards in R3.

**Predicted M3/M10/M11'/M6/M12:** 1.7-2.1 / 2-3 / 2.2-2.7 / 65-80% / 0.3-0.4

**Weakness:** R2 starts with a near-full 120-card pool after the 100% refill,
partially resetting R1's concentration. The refill reset problem occurs at the
R1-to-R2 boundary even though it is mitigated at R2-to-R3.

---

### Proposal B: 3-Round Steep Decline (100/40/0)

**Description:** Three rounds; R2 refill is only 40% of consumed; no R3
refill — final round drafts from the residual.

**Technical Spec:** Starting pool 120. R1: 60 consumed, refill 60 (100%), R2
starts 120. R2: 60 consumed, refill 24 (40%), R3 starts 84. R3: 60 consumed;
AI saturation required or pool exhaustion at ~pick 28. Saturation threshold
lowered to 6 cards. Balanced refills.

**Predicted M3/M10/M11'/M6/M12:** 1.9-2.3 / 2-4 / 2.5-3.0 / 70-85% / 0.4-0.5

**Weakness:** Pool safety risk. 5 AIs × 10 picks from 84 cards is marginal
without reliable saturation. M10 upper bound of 4 reflects the risk of
uneven AI saturation creating thin packs in picks 25-28.

---

### Proposal C: 4-Round Graduated Decline (100/75/50/0) — Champion

**Description:** Four rounds of 7-8 picks with refills declining at each
boundary; underrepresented-archetype bias on the final refill.

**Technical Spec:**
- Round 1: 8 picks, 48 consumed, 72 remain. Refill 100% = 48 added (6.0 per
  archetype). Round 2 starts: 120.
- Round 2: 8 picks, 48 consumed, 72 remain. Refill 75% = 36 added (4.5 per
  archetype). Round 3 starts: 108.
- Round 3: 7 picks, 42 consumed, 66 remain. Refill 50% = 21 added with
  underrepresented bias: archetypes below 8 cards receive proportionally more;
  the surplus is distributed equally among the rest. Round 4 starts: ~87.
- Round 4: 7 picks, 42 consumed. AI saturation threshold: 7 cards. Pool ends
  at ~45-52 cards.
- AI count: 5, Level 0. Each assigned one archetype. Pick logic: highest
  (fitness × power) for archetype until saturation, then highest-power generic.
- Player information: round-start archetype card counts displayed at start of
  Rounds 1-3; no summary for Round 4.

**Predicted M3/M10/M11'/M6/M12:** 2.0-2.4 / 1-2 / 2.5-2.9 / 65-82% / 0.35-0.5

---

## Champion Selection: Proposal C

Proposal A's near-full first refill (60 cards → 120) resets Round 1
concentration and limits the compounding effect to just one boundary (R2-R3).
The gradient at Round 3 start is weaker than what four boundaries can produce.

Proposal B's aggressive decline creates the strongest concentration but
carries real pool exhaustion risk. The 84-card Round 3 pool with uncertain AI
saturation is a simulation prerequisite, not a safe recommendation.

Proposal C spreads the decline across three refill events rather than
compressing into two. Each boundary is smaller (48, 36, 21 cards), so no
single event dominates. The gradient compounds across four rounds without the
exhaustion risk of Proposal B. The Round 3 underrepresented bias prevents
complete AI-lane depletion without diluting the open-lane advantage — it is
pool-reactive, not player-reactive, and stays Level 0.

---

## Champion Deep-Dive: Graduated Four-Round

### Round-by-Round Walkthrough and Pool Composition

| Round Start | AI Lanes (each) | Open Lanes (each) | Gradient | Total |
|-------------|:---------------:|:-----------------:|:--------:|:-----:|
| Round 1 | 15.0 | 15.0 | 1.0x | 120 |
| Round 2 | 13.0 | 18.0 | 1.38x | 120 |
| Round 3 | 9.5 | 19.5 | 2.05x | 108 |
| Round 4 | ~4.5 | ~18.0 | ~4.0x | 87 |
| End | ~2.5 | ~13.5 | ~5.4x | ~50 |

Round 1 is open exploration — the pool is balanced and the round-start summary
shows no meaningful differentiation. The skill is in correctly reading which
lanes will stay open.

Round 2 starts with a visible 1.38x gradient (the pool summary shows AI lanes
with 13 cards, open lanes with 18). Commitment here costs one round of
compounding but retains 8 picks of benefiting from the gradient. Late
commitment (Round 3) is significantly more costly: the 2.05x gradient forgone
is worth roughly 0.4-0.6 M3 per pick.

Round 3 delivers M11'-critical picks (picks 17-23). The concentrated 108-card
pool and underrepresented bias on the final refill means these picks see the
sharpest open-lane density. AIs at 8-card saturation easing off reduces
competition further.

Round 4 is pure execution. No pool summary. Expected yield: 3.0-3.5 S/A per
7 picks for a committed player.

### What the Player Sees

Each round transition shows: "The pool has been restocked. Current counts:
Flash: 12, Blink: 11, Warriors: 19, Ramp: 20, ..." The player directly reads
which archetypes are open (high counts = uncontested) vs. depleted (low
counts = multiple AIs competing). This substitutes for the "reading passed
packs" signal in traditional drafts.

### Failure Modes

**Trivial signal reading.** Round-start counts show resonance totals (Tide,
Zephyr, etc.), not archetype level — Warriors vs. Sacrifice both register as
Tide. The player infers the sub-archetype split from card text during the
round, preserving meaningful decision-making.

**Round 4 thin-pool feel.** By Round 4, the player is in execution mode. The
smaller pool concentrates choices rather than reducing them.

**Late commitment penalty too steep.** A Round 3 pivot faces a concentrated
pool biased against the new archetype — the honest cost of delayed reading,
not a design bug. M12 rewards early readers without catastrophically
punishing correct late pivots.

---

## Complete Specification

| Parameter | Value |
|-----------|-------|
| Pool size | 120 (starting), 8 archetypes × 15 cards |
| Rounds | 4 |
| Picks per round | R1: 8, R2: 8, R3: 7, R4: 7 (total: 30) |
| Refill R1→R2 | 48 cards, balanced (6.0/archetype) |
| Refill R2→R3 | 36 cards, balanced (4.5/archetype) |
| Refill R3→R4 | 21 cards, underrepresented-biased (archetypes below 8 cards receive proportionally more) |
| Refill R4 | None |
| AI count | 5, Level 0, one archetype each |
| AI pick logic | Best available (fitness × power) until saturation |
| AI saturation | R1-R2: 9 cards; R3: 8 cards; R4: 7 cards |
| Post-saturation | Take highest-power generic |
| Player information | Round-start archetype card counts (Rounds 1-3); refill composition shown at each transition |
| Reactivity level | Level 0 (refill bias is pool-state-reactive, not player-pick-reactive) |
| Card distribution | ~11% generic, ~79% single-resonance, ~10% dual-resonance |
| Fitness model | Graduated Realistic (weighted avg ~36% A-tier) |

---

## Post-Critique Revision

**Accepting the underrepresented-bias critique.** The critic is correct. I
built the Round 3 underrepresented bias to prevent AI-lane exhaustion, but the
mechanism is self-defeating: underrepresented archetypes are underrepresented
precisely because AIs are drafting them. Restocking those lanes preferentially
reduces the concentration gradient at the moment it should be sharpest. The
bias is a refill-reset in disguise, smaller in scale than a full balanced
refill but operating on the same logic.

The modified champion drops the bias entirely. The Round 3 refill becomes 21
cards balanced (2.625 per archetype). If a given AI-lane drops below a safe
floor (~4 cards), AI saturation absorbs the risk — that mechanism exists
independently and does not require the bias to function. The spec table above
should be read with "underrepresented-biased" replaced by "balanced" for
R3-to-R4.

**Defending the 4-round structure, with acknowledged cost.** The critic asks
whether the marginal M3 gain from 4 rounds justifies the added complexity. My
honest answer: marginally, not decisively.

The 4-round structure has one concrete advantage: the declining refill events
occur at three boundaries (R1→R2, R2→R3, R3→R4) rather than two. Each
boundary adds a compounding step to the concentration gradient. The Round-by-
Round table shows this — the gradient moves from 1.0x to 1.38x to 2.05x to
~4.0x. A 3-round structure with the same total refill volume (105 cards: 48 +
36 + 21) arrives at the final round one step earlier, which means the 4.0x
gradient appears at start of pick 23 rather than pick 17. That is six fewer
picks at peak concentration.

The cost is real: one extra round boundary to tune, one extra round-start
summary the player reads, and a slightly higher AI saturation tuning surface.
If simulation shows that 3-round declining volume reaches M3 >= 2.2 reliably,
the 4-round structure cannot justify itself on M3 grounds alone.

**On the Hybrid A proposal.** Hybrid A takes my declining-volume mechanism and
combines it with Design 4's open-lane bias. I think this is the right
direction, but the two mechanisms are redundant for M3: declining volume alone
compounds the gradient; open-lane bias compounds it faster but adds
player-reactivity complexity. Whether the combination is better than either
alone depends on whether the bias's reactivity overhead (Level 1) is worth the
predicted M3 gain (2.2-2.6 vs. 2.0-2.4 for unbiased 4-round).

My preference: run SIM-4 (modified Design 3, balanced refills throughout) and
Hybrid A in parallel. If SIM-4 reaches M3 >= 2.2, the simpler structure wins.
If SIM-4 caps at 2.0-2.1, Hybrid A's bias is earning its complexity cost.

**Modified champion summary:** 4 rounds (8/8/7/7), declining balanced refills
(48/36/21/0), no underrepresented bias anywhere, AI saturation as specified.
Predicted M3: 2.0-2.4. The declining-volume insight stands; the bias
correction removes the mechanism that was working against it.
