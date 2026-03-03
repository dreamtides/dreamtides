# Research Results: V10 Remediation Analysis

## Question

What specifically does the multi-round refill mechanism fix in V10's failures,
and what might it NOT fix?

---

## 1. Pool Depletion Math

### Full Refill (Replenish to 120 Each Round)

Setup: 120-card pool, 5 AIs + 1 player = 6 cards removed per pick, 10 picks
per round, 30 picks total across 3 rounds.

- Round 1 start: 120 cards. 10 picks x 6 removals = 60 cards removed.
  Round 1 end: 60 cards remain. Refill to 120 adds 60 cards.
- Round 2 start: 120 cards. 60 removed. Round 2 end: 60 remain. Refill to 120.
- Round 3 start: 120 cards. 60 removed. Draft ends. No refill needed.

**Pool exhaustion verdict: Fully solved under full replenishment.** The pool
never drops below 60 cards (50% of starting size), which is far above V10's
exhaustion threshold. V10 exhausted a 360-card pool by pick 12-15 because
20-35 cards were removed per pick with no refill. A 120-card pool with 6
removals per pick and full refills is mathematically sustainable for any
number of rounds.

### Partial Refill (75% Replenishment)

- Round 1: 120 -> 60 remain -> refill 75% of 60 = +45 cards -> 105 start of
  Round 2.
- Round 2: 105 -> 45 remain -> refill 75% of 60 = +45 cards (fixed) -> but
  only 60 consumed, so +45 -> 90 start of Round 3.
- Round 3: 90 -> 30 remain. Draft ends.

Alternatively, if refill is 75% of the 60-card gap (replenish to 110 not 120):
Round 2 starts at 105, Round 3 starts at ~90. Pool shrinks 10-15 cards per
round, which creates natural late-draft concentration pressure.

**Partial refill verdict:** Creates a mild declining-availability effect.
Reduces pool from 120 to ~90 by the final round. This is a controlled scarcity
ramp, not exhaustion. Even at 50% refill, the final-round pool (60-70 cards)
is above V10's pick-12 exhaustion point (30-40 cards remaining).

Pool depletion (V10 root cause 1) is **fixed by any refill strategy** that
adds more than zero cards between rounds. The math is not close.

---

## 2. S/A Preferential Depletion Under Refills

### The Core Model

8 archetypes x 15 cards each = 120 cards. Each archetype: 5 S/A cards (33%),
10 C/F cards (67%). 5 AIs cover 5 archetypes; 3 archetypes are open (no AI).

**AI behavior:** Each AI drafts from its own archetype. Since pair-affinity
score correlates with S/A fitness, AIs take S/A cards first. By end of Round 1:

- Each of 5 AI-lane archetypes: AIs took ~5 S/A picks from their lane (one AI
  takes 10 picks total in the round; not all from their own archetype, but
  preferentially so). Estimate 3-4 S/A cards taken per AI-lane archetype
  per round.
- Each of 3 open-lane archetypes: No AI takes cards. Open lanes lose 0 S/A
  cards to AI depletion. Player takes ~3-4 picks from their open lane per
  round, but player is picking the BEST (S/A) cards by design.

**Refill effect:** Full balanced refill adds 60 cards = 7.5 cards per archetype
= ~2.5 S/A + 5 C/F per archetype.

**Net trajectory after one round:**

- AI lanes: Lost 3-4 S/A, gained 2.5 S/A via refill. Net change: -0.5 to -1.5
  S/A cards per AI-lane archetype per round. S/A density declines slowly.
- Open lanes (non-player): Lost 0 S/A to AIs, gained 2.5 S/A via refill.
  Player took 3-4 total picks from their lane, ~2 of which were S/A. Net
  change: ~+0.5 S/A from refill minus ~2 player picks. Roughly stable.
- Player's specific open lane: Refill adds S/A, player removes S/A. Net neutral
  within the lane. But from the player's perspective, the S/A cards THEY receive
  are drawn from a pool that includes their open lane's S/A-rich cards.

**Critical insight:** The refill partially counteracts S/A depletion in AI
lanes. By Round 3, AI lanes may have lost 1.5-4.5 S/A cards net (over 3 rounds
of depletion minus refill). This means by Round 3, open lanes are meaningfully
more S/A-dense than AI lanes even under full replenishment.

**S/A depletion verdict: Significantly mitigated but not fully solved.** The
depletion still occurs (AIs take S/A cards), but refills slow the degradation
curve. Over 3 rounds, the S/A density gap between open lanes and AI lanes
grows from 0% in Round 1 to a measurable difference in Round 3. V10's
catastrophic S/A drain (best S/A depleted by pick 8-10) becomes a gradual,
recoverable decline.

However, the mechanism still works in the wrong direction: AIs take high-quality
cards and leave C/F cards in AI lanes. Refills add new S/A cards that AIs will
take again next round. This is the "S/A cycling" failure mode (see Section 5).

---

## 3. Targeting Dilution Under Refills

### The 1-of-3 Problem

With 5 AIs covering 5 archetypes and 3 lanes open, the player's specific
archetype is 1 of 3 open lanes. AI depletion enriches all 3 open lanes equally.
The player benefits from concentration in their lane, but so do the 2 other
open lanes.

**Concentration achievable through AI depletion (per round):**

- Pool starts: Each open lane has 15 cards (~12.5% of 120-card pool each).
- After Round 1: Each AI lane has ~11 cards remaining (5 S/A taken, partially
  offset by player picks from their lane). Each open lane has ~13 cards (player
  took 3-4 picks). Total pool: ~55 AI-lane cards + ~39 open-lane cards = 94
  cards (before refill).
- Open-lane share: 39/94 = 41.5% of pool, up from 37.5% (3 open x 12.5%).
  But the player's specific lane = 13/94 = 13.8%, up from 12.5%.
- Concentration: 13.8% / 12.5% = 1.1x improvement in Round 1.

Over 3 rounds (cumulative, with refills adding balanced cards back):

Refills reset the denominator partially. If refill adds 7.5 cards to each
archetype equally, the concentration advantage of open lanes is partially
washed out each round. Conservative estimate: cumulative multi-round
concentration reaches 1.4-1.6x by end of Round 3.

Compare to V9 (5-7x) and V10 at best (1.7x). Multi-round refills alone produce
concentration in the 1.3-1.6x range -- essentially the same as V10's best
result, not a dramatic improvement.

**Targeting dilution verdict: NOT meaningfully fixed by refills alone.** This
is the most important finding. Balanced refills restore the archetype
distribution each round, partially undoing whatever concentration built up. The
player's lane is still 1 of 3 open lanes. The concentration mechanism
(AI depletion) is weakened each round by balanced refills.

---

## 4. The Refill Reset Problem

### Mechanism

Round 1 builds concentration: AI lanes are depleted, open lanes are preserved.
A balanced refill adds equal cards to all 8 archetypes, partially restoring AI
lanes and partially undoing the concentration gradient.

**Quantified reset effect:**

After Round 1 (before refill):
- AI lane archetype share: ~11/94 = 11.7% each (down from 12.5%)
- Open lane archetype share: ~13/94 = 13.8% each (up from 12.5%)
- Gradient: 13.8% vs 11.7% = 1.18x

After refill (balanced, adding 7.5 cards/archetype):
- AI lane: 11 + 7.5 = 18.5 cards
- Open lane: 13 + 7.5 = 20.5 cards
- Total: 5 x 18.5 + 3 x 20.5 = 154 cards (but pool is 120 after exact refill)

If refilled to exactly 120 (adding 60 total, 7.5 per archetype):
- AI lane share: 18.5/120 = 15.4%
- Open lane share: 20.5/120 = 17.1%
- Gradient: 17.1% vs 15.4% = 1.11x (DOWN from 1.18x pre-refill)

**The refill reset is real and measurable.** A full balanced refill recovers
~half the concentration built during the round. The concentration gradient
shrinks from 1.18x to 1.11x due to the refill. Over 3 rounds, concentration
accumulates SLOWER than it would without refills.

### When Does Concentration Accumulate vs Reset?

**Accumulation occurs when:** Refills are partial (AI lanes filled less than
open lanes), refills are biased toward open lanes, or AI behavior is more
aggressive than refill can compensate.

**Reset occurs when:** Refills are balanced and full. The reset proportion
equals (refill cards per archetype) / (total cards per archetype after depletion).
With high-volume balanced refills, the reset can exceed 80% of the round's
concentration gain.

**Key constraint:** Balanced refills structurally prevent concentration
accumulation above a low ceiling (approximately 1.5-2.0x over 3 rounds) unless
AI removal per round significantly exceeds refill volume per archetype in AI
lanes.

---

## 5. Unique Multi-Round Failure Modes

### "S/A Cycling" (High Risk)

AIs take S/A cards. Refills add S/A cards back. AIs take the new S/A cards
next round. Net over 3 rounds: zero progress on S/A enrichment for the player.
This is not a net-zero outcome -- the player sees some S/A cards in each round
(from their open lane), but the S/A DENSITY improvement (the mechanism behind
V9's M3 gains) does not compound. Each round resets to roughly the same S/A
density as the round before.

### "Refill Reset" (High Risk, Confirmed by Math)

Already quantified: balanced full refills recover ~half the concentration
gradient each round. After 3 rounds, concentration reaches 1.3-1.6x instead
of the cumulative 1.7x V10 achieved (which required no refills). Multi-round
with full refills may actually be WORSE at concentration than V10's single-pool
design because refills actively undo AI lane depletion.

### "Open Lane Dilution" (Medium Risk)

Refills add cards to all 8 archetypes, including the 3 open lanes. The player's
open lane gets restocked with cards the player has not seen yet -- which sounds
good but also dilutes any lane-specific concentration the player was accumulating.
If the player has been building toward Flash/Tempo, a refill that adds Warriors
cards to the player's open lane (Flash/Tempo) dilutes the signal.

### "Late-Round Thinning" (Low Risk with Full Refills, Medium Risk with Partial)

With partial refills (75%), the pool shrinks ~10-15 cards per round. By Round 3,
the pool is ~90 cards instead of 120. This is not catastrophic (V10 exhausted
at 30-40 cards), but with aggressive AI picks it could produce meaningful
scarcity in the final round. Declining refills exacerbate this.

---

## 6. What Refill Strategies Work and Why

### Full Balanced Refills

**Fixes:** Pool exhaustion completely. S/A depletion partially (slows drain).
**Does not fix:** Targeting dilution. Actively resets concentration each round.
**Predicted M3:** 1.2-1.5. Better than V10's 0.84 but well below V9's 2.70.

### Partial Balanced Refills (75%)

**Fixes:** Pool exhaustion. Creates natural declining-availability pressure.
**Partially fixes:** Targeting dilution (shrinking pool amplifies AI depletion
gradient). S/A depletion is worse than full refill (fewer replenishments).
**Predicted M3:** 1.3-1.7. Moderate improvement.

### Biased Refills (Underrepresented Archetypes Get More Cards)

**Fixes:** Pool exhaustion. Amplifies AI lane depletion (AI lanes are already
depleted, so they receive more refill cards, which AIs then take again -- net
effect unclear). Does NOT help open lane concentration unless bias explicitly
favors open lanes.

### Biased Refills (Open Lanes Receive More Cards)

**Fixes:** Pool exhaustion. Actively counteracts refill reset by concentrating
refill cards in open lanes.
**Predicted M3:** 1.5-2.0 depending on bias strength. This approaches the V9
target but requires knowing which lanes are open -- approaching Level 1 behavior.

---

## 7. Honest Assessment

### What Multi-Round Refills Fix

1. **Pool exhaustion:** Completely and reliably. Any refill strategy keeps the
   pool above 50% of starting size throughout the draft.
2. **S/A depletion rate:** Slowed significantly. Refills add S/A cards back
   each round. Depletion still occurs (AIs still prefer S/A cards) but the
   curve is shallow enough that late rounds still have S/A cards available.
3. **Draft pacing:** Refills create natural "round" structure with decision
   points. Players experience fresh cards entering the pool, creating genuine
   re-evaluation moments.

### What Multi-Round Refills Do Not Fix

1. **Targeting dilution:** The player is still 1 of 3 open lanes. Balanced
   refills restore the archetype distribution, partially undoing AI lane
   depletion. Concentration gains from Round 1 are partially erased by Round 2's
   refill.
2. **S/A cycling:** AIs take S/A cards, refills add S/A cards back, AIs take
   them again. Net S/A enrichment for the player's lane is minimal.
3. **Concentration ceiling:** The math suggests balanced multi-round refills
   achieve ~1.3-1.6x concentration, comparable to V10's 1.7x, and well below
   V9's 5-7x. Refills reset concentration as fast as AI depletion builds it.

### What Additional Mechanism Is Required

To reach M3 >= 2.0, multi-round refills must be combined with at least one of:

- **Biased refills** that explicitly favor open lanes (requires knowing which
  lanes are open -- approaches Level 1 reactivity).
- **Declining refill volume** that lets AI depletion outpace replenishment in
  later rounds (creates concentration ramp at the cost of late-draft pool size).
- **Supplemental culling** between rounds that removes low-relevance cards from
  the pool before refill (which is V9's contraction by another name).
- **Higher AI pick rates** within each round such that AI lanes are more
  severely depleted per round than refills can compensate.

---

## Connections

These findings directly inform Round 2 algorithm design:

- **Agent 1 (3-Pack Classic):** Full balanced refills will achieve ~1.4-1.6x
  concentration. M3 target (2.0) likely requires additional bias or culling.
- **Agent 2 (Small Pool, Fast Cycles):** Frequent refills make the reset problem
  worse, not better. Short rounds with aggressive AI picks might outrun the
  reset, but only if per-round AI removal significantly exceeds per-round refill
  in AI lanes.
- **Agent 3 (Declining Refills):** The most promising structural fix. By making
  later refills smaller, AI depletion outpaces replenishment in late rounds.
  Concentration accumulates where balanced refills would reset it.
- **Agent 4 (Biased Refills):** Biasing refills toward open lanes directly
  counteracts the refill reset problem. This is the mechanism most likely to
  achieve M3 >= 2.0 without supplemental culling. Needs to be calibrated against
  Level 1 reactivity concerns.
- **Agent 5 (Player Information):** Pool information becomes more valuable in
  multi-round because refills create observable events. Round-start snapshots
  give players a real decision point.
- **Agent 6 (Hybrid/Novel):** The "market draft" or "shared visible pool" formats
  may avoid the refill reset problem by making pool state transparent and
  allowing players to react to it in real time.

---

## Open Questions

1. Does the refill reset produce WORSE concentration than V10's single-pool
   design? The math suggests it might: V10 achieved 1.7x through monotonic AI
   depletion, while balanced multi-round refills interrupt the depletion with
   resets each round, potentially landing at 1.3-1.5x.

2. What declining refill schedule produces a concentration ramp equivalent to
   V9's self-regulating percentage-based contraction? This is the key design
   question for Agent 3.

3. How strong must refill bias toward open lanes be to reach M3 >= 2.0, and
   at what bias level does it become detectable as a rigged mechanism to players?

4. Is S/A cycling the binding constraint on concentration, or is targeting
   dilution? If S/A cycling dominates, bias toward S/A within open-lane refills
   would help more than lane-level bias. If targeting dilution dominates,
   lane-level bias matters more.

5. Can any multi-round design achieve V9's 5-7x concentration without some form
   of V9-style virtual contraction? The math here suggests the answer is no --
   but declining refills + biased composition is the closest approximation.
