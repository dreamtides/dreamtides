# Design 1: Avoidance + Contraction, No Oversampling (Isolation Test)

## Key Findings

- **Balanced refills neutralize avoidance.** Research Agent C's models confirm
  that with balanced refills, the player's archetype stays at ~11-14% pool
  density regardless of avoidance strength. The 60-card refill after Round 1
  restores all archetypes proportionally, nearly fully undoing the avoidance
  benefit accumulated during that round. Avoidance + balanced declining refills
  ≈ M3 ≈ 0.20-0.25 — marginally better than SIM-4's 0.83 only because S/A is
  preserved, not because density climbs.

- **N = 4 cannot reach M3 = 2.0 from the pool sizes V12 can achieve.** The
  math is strict: M3 = 4 × (S/P). With 5 S/A in a 20-card pool, M3 = 1.0.
  With 5 S/A in a 10-card pool, M3 = 2.0. Contracting to 10 cards with 30
  picks × 6 removals per cycle = 180 total removals requires supplying ~170
  cards (120 start + ~50 refill), which exhausts by pick 28 with a pool
  dangerously thin for card variety. N = 4 isolation confirms oversampling
  is required for M3 >= 2.0.

- **The S/A trajectory is the binding constraint at N = 4, not pool density.**
  Even in the most aggressive contraction scenario (final pool = 20 cards,
  archetype density = 25%), if only 2 S/A remain, M3 = 4 × 2/20 = 0.40.
  S/A depletion from the player's own picks (~0.33 S/A per on-arch pick) means
  late-draft S/A is 1-3 cards without biased refill replenishment.

- **Avoidance onset pick has outsized impact on S/A preservation.** Avoidance
  starting pick 5 vs pick 8 preserves 2-4 additional S/A cards (5 AIs × 3
  extra picks × ~0.13 S/A rate = ~2 S/A). With a starting S/A count of 5,
  this is a 40% difference in the resource the player needs most.

- **The V11 SIM-4 baseline (M3 = 0.83) is not beaten by avoidance + balanced
  refills.** Adding avoidance to SIM-4's refill schedule (48/36/21/0) produces
  only marginal improvement (~M3 = 0.25-0.35) because the balanced refills
  restore the player's archetype cards at the same rate AIs drain them from
  other archetypes — density stays flat. The improvement comes only from S/A
  preservation (AIs don't drain the player's best cards), but N = 4 cannot
  convert preserved S/A into M3 without pool contraction to the ~10-card range.

- **60/0/0 (single-refill aggressive) is the most interesting schedule for
  isolation testing.** With a single 60-card refill after Round 1 and no
  further refills, the pool contracts from 120 to ~60 at pick 10 (post-refill),
  then to ~0 by pick 30 — but 6 removals × 20 picks = 120, which exactly
  exhausts a 120-card pool without the refill. With 60-card refill: 120 + 60
  = 180 supply, 180 removals → final pool ≈ 0. This schedule achieves maximum
  contraction but risks late-draft exhaustion; AI saturation thresholds at 15
  cards remaining are essential.

- **This isolation test's value is calibration, not a viable champion.** The
  N = 4 result definitively quantifies the avoidance + contraction contribution
  before oversampling is added. If avoidance + contraction alone reaches
  M3 ≈ 0.8-1.0 by late draft, then N = 8 oversampling may be sufficient to
  reach 2.0. If it only reaches 0.3-0.5, N = 12 or higher is needed.

---

## Three Algorithm Proposals

### Algorithm A: SIM-4 + Avoidance (Direct Comparison)

**Description:** SIM-4's exact refill schedule (48/36/21/0, 4 rounds) with
AI avoidance added from pick 5, providing the cleanest comparison to V11's
M3 = 0.83 baseline.

**Technical spec:**
- Starting pool: 120 cards (15/archetype)
- Rounds: 4 (8/8/7/7 picks)
- Refills: Balanced 48/36/21/0
- Oversample N: 4 (uniform random)
- AI count: 5
- AI avoidance: Gradual ramp, 20% weight reduction at pick 5, 60% at pick 8,
  85% at pick 12+. Inference from pool depletion patterns, archetype-specific
  (not resonance-symbol-wide)
- AI inference: Compare observed-to-expected depletion rate per archetype over
  sliding 3-pick window. Archetype with 1.5x+ expected depletion rate flagged
  as player's likely lane
- AI pick logic: Score each card by archetype fitness. Apply avoidance weight
  multiplier to player's inferred archetype. Pick highest score
- Saturation threshold: AI stops taking from its archetype when < 5 cards
  remain in that lane

**Predicted metrics:**
- M3: 0.30-0.45 (S/A preserved by avoidance, but balanced refills keep density
  flat at 12-16%)
- M10: 25+ (late draft has near-zero S/A packs; avoidance helps slightly)
- M11': 0.20-0.35
- M6: 45-60%
- M12: 0.05-0.15 (signal-reader advantage is small since density is uniformly low)
- M13: Pick 6-8 (AI detectable avoidance change; gradual so subtle)
- M14: Pick 5-7 (AI correctly infers player archetype)

---

### Algorithm B: 60/36/0 + Avoidance (3-Round Moderate Contraction)

**Description:** 3-round structure with moderate declining refills, avoidance
ramping from pick 3, targeting maximum late-draft contraction with minimal
refill resets.

**Technical spec:**
- Starting pool: 120 cards
- Rounds: 3 (10/10/10 picks)
- Refills: Balanced 60/36/0
- Oversample N: 4 (uniform random)
- AI count: 5
- AI avoidance: Sigmoid ramp beginning pick 3. Confidence score = (observed
  depletion rate / expected baseline rate) for each archetype. Weight reduction
  = min(0.9, confidence × 0.45). Full avoidance (90%) when confidence > 2.0x
  expected for 3+ consecutive pick cycles
- AI inference: Per-archetype depletion counter normalized by initial proportion.
  Confidence accumulates over pick cycles
- AI pick logic: Same scoring as Algorithm A
- Saturation threshold: 4 cards remaining in lane

**Pool trajectory (modeled):**

| Event | Pool | P-Arch Cards | P-Arch S/A | Arch % | M3 (N=4) |
|-------|:----:|:------------:|:----------:|:------:|:---------:|
| Start | 120 | 15 | 5 | 12.5% | 0.17 |
| Pick 5 (pre-avoidance) | 90 | 13 | 4 | 14.4% | 0.18 |
| R1 end (pre-refill) | 60 | 8 | 3.2 | 13.3% | 0.21 |
| R2 start (post-60 refill) | 120 | 15.5 | 4.7 | 12.9% | 0.16 |
| R2 end (pre-refill) | 60 | 9 | 3.5 | 15.0% | 0.23 |
| R3 start (post-36 refill) | 96 | 12.5 | 4.0 | 13.0% | 0.17 |
| R3 end (no refill) | 36 | 6 | 2.5 | 16.7% | 0.28 |
| Picks 25-30 (pool ~20) | 20 | 5 | 2.0 | 25.0% | 0.40 |

**Predicted metrics:**
- M3: 0.25-0.40 (density climbs only in R3; balanced refills keep resetting)
- M10: 20-28
- M11': 0.25-0.40
- M6: 50-65%
- M12: 0.08-0.20 (signal-readers detect open lane slightly earlier)
- M13: Pick 5-7
- M14: Pick 4-6

---

### Algorithm C: 60/0/0 + Early Avoidance (Aggressive Single-Refill Champion)

**Description:** Single aggressive refill after Round 1, zero refills
thereafter. AI avoidance starts pick 3 (earliest viable inference window).
Pool contracts steeply in Rounds 2-3, reaching 15-25 cards by picks 25-30.
This is the maximum contraction achievable without oversampling.

**Technical spec:**
- Starting pool: 120 cards
- Rounds: 3 (10/10/10 picks, or 8/11/11 with earlier R1 boundary)
- Refills: 60 after Round 1, 0 thereafter
- Oversample N: 4 (uniform random)
- AI count: 5
- AI avoidance: Early-onset inference beginning pick 3 (low initial confidence;
  errors acceptable because false positives are neutral). Avoidance weight:
  30% reduction at pick 3, 60% at pick 6, 85% at pick 10+. Resets if
  confidence collapses (archetype depletion returns to expected rate)
- AI inference: Same depletion-rate comparison. Archetype-specific, not
  symbol-wide
- AI pick logic: Same scoring, saturation threshold = 5 cards in lane

**Pool trajectory (modeled):**

| Event | Pool | P-Arch Cards | P-Arch S/A | Arch % | M3 (N=4) |
|-------|:----:|:------------:|:----------:|:------:|:---------:|
| Start | 120 | 15 | 5 | 12.5% | 0.17 |
| Pick 5 (early avoidance) | 90 | 13.5 | 4.5 | 15.0% | 0.20 |
| R1 end (pre-refill) | 60 | 10 | 3.8 | 16.7% | 0.25 |
| R2 start (post-60 refill) | 120 | 17.5 | 5.0 | 14.6% | 0.17 |
| Pick 15 | 60 | 13 | 4.0 | 21.7% | 0.27 |
| Pick 20 | 30 | 10 | 3.0 | 33.3% | 0.40 |
| Pick 25 | 15 | 7 | 2.2 | 46.7% | 0.59 |
| Pick 30 | 3-5 | 4-5 | 1.5-2.0 | 80-100% | 1.20-2.00 |

Note: Pick 25-30 values assume AI saturation prevents full pool exhaustion; AIs
stop taking once their lanes hit 4-5 cards. With proper saturation logic, a
~15-20 card floor is maintained.

**Predicted metrics:**
- M3: 0.40-0.60 (density climbs significantly in R3; late picks reach M3 ~1.0-1.5)
- M10: 12-18 (bad streaks cluster in R1-R2 before contraction kicks in)
- M11': 0.70-1.00 (pick 20+ sees meaningful concentration)
- M6: 58-72%
- M12: 0.15-0.30 (signal-readers benefit more from earlier avoidance onset)
- M13: Pick 4-6
- M14: Pick 3-5

---

## Champion Selection

**Algorithm C** (60/0/0 + Early Avoidance) is the champion for this isolation
test.

**Justification:** Algorithm C maximizes the measurable contribution of
avoidance + contraction before oversampling is introduced. Its single-refill
structure minimizes refill resets (the primary enemy of concentration in
balanced-refill designs). The 60-card refill after Round 1 sustains the pool
through the mid-draft while allowing steep contraction in Rounds 2-3. The late
R3 pool of 15-25 cards with early avoidance produces meaningful archetype
density (~35-50%), and the modeled M3 of 0.40-0.60 average — reaching 1.0-1.5
in picks 20+ — establishes the clearest baseline for calibrating how much
oversampling Agents 2 and 3 need.

Algorithms A and B both suffer from the balanced-refill-reset problem in every
round. Algorithm C avoids two of three resets, making the late-draft contraction
genuine rather than perpetually reset. The cost is early-draft pool thinness in
R3, which saturation thresholds manage.

---

## Champion Deep-Dive: Pick-by-Pick Walkthrough

### Setup

- Pool: 120 cards (15/archetype, 8 archetypes)
- Player commits to Blink/Ember (a mid-tier archetype at 30% sibling A-tier)
- 5 AIs assigned: Flash, Storm, Self-Discard, Sacrifice, Warriors (3 open lanes:
  Blink, Self-Mill, Ramp)
- Refills: 60 after pick 10, 0 after pick 20
- AI avoidance ramp: 30% at pick 3, 60% at pick 6, 85% at pick 10+

### Round 1 (Picks 1-10)

**Picks 1-2:** No avoidance. AIs take their archetype's best cards randomly.
Pool has 120 cards. Player sees 4 random cards, browsing the face-up pool to
identify open lanes. The player notices Blink cards persisting (no AI assigned
to Blink) and takes a high-synergy Blink card at pick 1. AIs take cards from
Flash, Storm, Self-Discard, Sacrifice, Warriors respectively. Pool composition
changes slowly; Blink still has ~14 S/A-bearing cards.

**Pick 3:** Avoidance ramp begins (30% weight reduction on player's emerging
archetype). AIs have now observed ~12-pick-cycles of depletion signals across 2
player picks. Ember-symbol depletion is marginally higher than expected (both
Blink and Storm use Ember). Signal is noisy — 30% weight reduction is
conservative. Player sees 4 random cards; takes 2nd Blink card. AI picking
Storm sees Ember depletion but attributes it to multiple drafters; applies mild
20% weight reduction to Ember-primary archetypes.

**Pick 6:** Avoidance at 60%. By now, 3+ pick cycles show Blink-specific cards
depleting 1.6x expected rate (player has taken 3 Blink cards; AIs with Ember
archetypes have taken incidental Blink-adjacent cards). AI confidence in player's
Blink commitment crosses 50%. AIs assigned to adjacent archetypes (Storm/Ember)
begin steering toward non-Blink picks when possible. Pool: ~84 cards, Blink has
~11 cards remaining (player took 4; 0 AI Blink picks due to partial avoidance).

**Pick 9-10 (pre-refill):** Avoidance approaching 85%. Pool: ~60 cards, Blink
has ~8 cards (player took 7; early partial-avoidance failure cost ~0 S/A to AIs
after pick 3). S/A remaining in Blink: ~3.0. Player's pack shows 4 random cards
from 60-card pool; expected Blink = 60 × 8/60 = 8/60 × 4 = 0.53 per pack.

**Round 1 end (post-60 refill):** Pool jumps to 120. Refill adds ~7.5 Blink
cards to the pool. Blink is now 15.5/120 = 12.9% of pool. Density reset by the
refill — but unlike balanced declines, this is the LAST major reset.

**Player sees:** Pool is large again. The Blink cards that persisted through
Round 1 are visible, plus new refill Blink cards. Player can browse and confirm
Blink supply looks healthy. AIs at full avoidance have not drained Blink — this
is visible: Blink's bar shows high supply compared to Storm, Flash, Warriors
which have been aggressively drained.

### Round 2 (Picks 11-20, No Refill at End)

**Picks 11-15:** Full avoidance (85%). Pool at 120 starts declining at 6
cards/pick; only 1/pick is from Blink (player's picks). Pool at pick 15: 90
cards. Blink: ~12 cards (15.5 - 4 player picks + 0 AI picks). Density: 12/90
= 13.3%. Still low but S/A preserved: ~4.0 S/A remaining.

**Picks 16-20:** Pool: 90 → 60. Blink: 12 → 8 cards. Density: 8/60 = 13.3%.
Pool is contracting but Blink is holding proportion because ONLY the player is
taking Blink cards. Non-Blink archetypes are depleting at 5× player's rate.
S/A: ~2.5 remaining.

**Critical dynamic:** By pick 18, Storm has only 5 cards left (AI assigned
to Storm has taken ~10). Self-Discard has 6. The non-player archetypes are
thin. AIs hitting saturation thresholds begin taking any available card — but
saturation threshold of 5 cards/lane prevents them from dipping into Blink's
supply (Blink is above the threshold). Pick cycles 18-20 see AIs becoming
less active (some lanes exhausted), reducing overall removals per cycle to
~4 instead of 6.

**Player sees at pick 20:** Pool: ~38 cards. Blink: 8 of 38 = 21%. S/A: ~2.5.
Pack of 4: expected S/A = 4 × 2.5/38 = 0.26. M3 at this point is still low, but
the concentration trajectory is visible — Blink cards are the dominant survivors.

### Round 3 (Picks 21-30, No Refill)

**Picks 21-25:** Pool contracts steeply. Non-player archetypes mostly exhausted;
AIs in saturation mode. Net removal per cycle: ~3 cards (player takes 1 Blink;
AIs take ~2 from exhausted non-Blink). Pool: 38 → 23. Blink: 8 → 5 cards.
Density: 5/23 = 21.7%. S/A: ~2.0. M3 = 4 × 2/23 = 0.35. Better, but not
transformative.

**Pick 25-30:** AIs nearly exhausted. Pool: 23 → 8. Blink: 5 → 3 cards.
Density: 3/8 = 37.5%. S/A: ~1.5. M3 = 4 × 1.5/8 = 0.75. Late-draft M3 nearly
1.0 as pool collapses to near-Blink-only contents.

**Player sees:** The pool is small and dominated by Blink cards. The face-up
browser shows this clearly: "There are 8 cards left in the pool. 5 of them are
Blink cards." The player is essentially picking from a Blink-focused pool.

### Failure Modes

**Failure Mode 1: S/A exhaustion before late contraction.** If the player picks
aggressively on-arch in R1 (all 10 picks from Blink), S/A depletes to ~1.5 by
pick 10. The R1 refill restores ~2.0 S/A (60 × 5/120). S/A at R2 start: ~3.5.
After R2 picks: ~1.5. Late R3 pool of 20 cards with 1.5 S/A → M3 = 0.30. Low.

**Failure Mode 2: Early false-positive avoidance.** If Storm depletes quickly
in picks 1-2 (Storm AI being aggressive), AIs may attribute excess Ember
depletion to the player and partially avoid Storm cards the player might want
as incidental picks. Minor effect since Storm and Blink are siblings and the
player benefits from both.

**Failure Mode 3: AI saturation creating an unintended free-for-all.** When
5+ non-player archetypes hit < 5 cards by pick 20, AIs may start taking
from the pool generally, including Blink. Saturation threshold at 5 cards per
lane prevents this for primary lanes, but AIs whose primary lane is exhausted
need fallback behavior: take from adjacent open lane, not from player's lane.
Fallback priority: (1) own lane if > 5 cards, (2) adjacent open lane with > 8
cards, (3) pass (take nothing). This keeps the player's Blink supply intact.

---

## Complete Specification: Algorithm C Champion

| Parameter | Value |
|-----------|-------|
| Starting pool size | 120 cards (15 per archetype) |
| Refill schedule | 60 after pick 10, 0 after pick 20 (2 refills, 1 active) |
| Refill bias | Balanced (equal per archetype) |
| Oversample N | 4 (uniform random — no oversampling; isolation test) |
| "Best 4" ranking | N/A (all 4 shown are random) |
| AI count | 5 |
| AI avoidance model | Gradual ramp: 30% weight reduction at pick 3, 60% at pick 6, 85% at pick 10+ |
| AI inference mechanism | Per-archetype depletion rate vs. expected baseline, sliding 3-pick window. Confidence = (observed / expected) - 1. Avoidance strength scales with confidence |
| AI pick logic | Score each available card by archetype fitness for AI's assigned archetype. Multiply player's inferred archetype fitness by (1 - avoidance_weight). Pick highest score |
| AI saturation threshold | Stop picking from lane when < 5 cards remain in that lane. Fallback: adjacent open lane with > 8 cards; else pass |
| Player information | Face-up pool (all cards visible). Who took what is secret. Pool count visible per archetype via browser filter |
| Player strategy types | Committed (pick 5-6), Power-chaser (ignores archetype), Signal-reader (reads pool state) |
| Archetype inference (system) | Mode of resonance symbol matches from player's last 5 picks. Used for AI avoidance confidence calculation only; no pack-level manipulation (N = 4 is uniform) |
| Round structure | 3 rounds of 10 picks each |
| Round boundaries | Pick 10 (refill), pick 20 (no refill) |

**Calibration purpose:** This specification is designed to establish the maximum
M3 achievable from AI avoidance + physical pool contraction alone, without any
oversampling. The expected result is M3 ≈ 0.40-0.70 average, reaching ~0.80-1.20
in picks 25-30. This baseline quantifies the oversampling contribution needed by
Agents 2 and 3: if avoidance + contraction produces M3 ≈ 0.60, then N = 8
oversampling must contribute an additional ~1.40 to reach M3 = 2.0. If it
produces M3 ≈ 1.0, only ~1.0 additional contribution is needed from N = 8.

---

## Post-Critique Revision

The critic identified one binding error in the current specification: the refill
schedule is listed as "Balanced (equal per archetype)" in the champion's
parameter table. The critic correctly notes this will reset the archetype density
gradient each round, producing a result indistinguishable from V11 SIM-4 + avoidance
rather than a true test of avoidance + contraction as a combined mechanism. The
isolation test must use biased refills to function as a valid calibration
baseline for Designs 2 and 3.

### Revised Parameter: Refill Bias

The "Refill bias" parameter in the complete specification table changes from:

> Balanced (equal per archetype)

To:

> 2.0x open-lane bias (archetypes with no assigned AI receive 2.0x the per-card
> refill rate; archetypes with an assigned AI receive the remaining budget
> proportionally reduced)

The 2.0x factor matches the bias used in Hybrid 1 (the critic's recommended
champion) rather than Design 3's 1.7x, ensuring the calibration baseline is
conservative relative to the designs it is meant to anchor. A 2.0x bias means
that if the pool has 3 open lanes and 5 closed lanes, the open lanes receive
roughly 54% of the 60-card refill (18 cards split among 3 lanes) while closed
lanes receive 46% (split among 5 lanes). The player's archetype, being an open
lane, receives ~6 additional S/A-capable cards from the refill rather than
~3-4 under balanced distribution.

### Revised M3 Prediction

With 2.0x biased refills, the R1 refill now restores the player's Blink lane
to approximately 16-18 cards (vs ~15.5 under balanced). More importantly, the
S/A restoration is stronger: biased refills add ~3-4 S/A to the open lane
(vs ~2.5 under balanced), improving the S/A trajectory through Round 2.

Revised pool trajectory at key waypoints:

| Event | Pool | P-Arch S/A (revised) | M3 (N=4, revised) |
|-------|:----:|:--------------------:|:-----------------:|
| R2 start (post-60 biased refill) | 120 | 5.5 | 0.18 |
| Pick 20 | 38 | 3.2 | 0.34 |
| Pick 25 | 20 | 2.5 | 0.50 |
| Pick 28-30 | 8-12 | 1.8-2.2 | 0.60-1.10 |

Revised overall M3 prediction: **0.50-0.75** (up from 0.40-0.60), with late-draft
picks 25-30 reaching M3 ~ 0.75-1.10. The biased refill does not dramatically
change N=4 performance because oversampling is still absent, but it ensures that
the S/A trajectory is comparable to what Designs 2 and 3 will experience. The
calibration question becomes: given this biased-refill baseline of M3 ≈ 0.60-0.75,
how much does N=8 (Design 2) or N=12 (Design 3) add on top?

### No Other Changes

The avoidance ramp, round structure, AI count, saturation thresholds, and
pool trajectory are unchanged. N=4 remains the fixed isolation variable — the
only change is making the refill bias consistent with the designs this test
is calibrating against.
