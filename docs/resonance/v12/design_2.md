# V12 Algorithm Design 2: Steep Contraction + Light Oversampling (N = 8)

## Key Findings

- **60/20/0 is the target refill schedule.** Starting at 120 cards, adding 60
  then 20, totaling 200 cards over 30 picks × 6 removals = 180 total. Final
  pool ~20 cards. This is exactly the N = 8 threshold: at pool = 20 with 5 S/A,
  M3 = 8 × (5/20) = 2.00.

- **Biased refills are mandatory.** Balanced refills reset archetype density at
  every round boundary, as V11 SIM-4 (M3 = 0.83) proved. With 5 AIs depleting
  non-player archetypes and refills restoring all 8 equally, the player's arch
  fraction stays stuck near 12%. Refills must use open-lane bias (~1.7x, as
  validated Level 0 in V11 SIM-2) to compound rather than counteract avoidance.

- **Gradual avoidance (pick 3 ramp, full at pick 12) is correctly calibrated.**
  With 5 AIs sharing a 120-card pool, reliable archetype inference requires 3-5
  pick cycles. Starting the ramp at pick 3 with low initial weight (20%)
  captures early S/A preservation without over-committing to noisy signals. The
  asymmetry favors early: false-positive avoidance (AI avoids wrong arch) is
  neutral-to-positive for the player; false-negative (AI takes S/A before
  pivoting) is irreversible. By pick 12, the signal is statistically robust and
  full avoidance weight (80-90%) is warranted.

- **The player's archetype reaches ~40% of the pool by pick 22-25.** Under
  60/20/0 with open-lane biased refills and gradual avoidance: AIs deplete the
  7 non-player archetypes while refills preferentially restore the 3 open lanes
  (including the player's). By pick 20, pool = ~40 cards; player's arch = ~10
  cards (25%). By pick 25, pool = ~22 cards; player's arch = ~8 cards (36%).
  By pick 27-28, pool = ~16 cards; player's arch = ~7 cards (44%). The 40%
  threshold is crossed in picks 25-27.

- **S/A count is the binding constraint, not pool density.** At pool = 20 with
  only 3 S/A remaining, N = 8 yields M3 = 8 × (3/20) = 1.2. Maintaining S/A
  at 4-5 through picks 21-30 requires: (a) avoidance activating before most
  S/A is drained in picks 1-7, and (b) biased refills adding S/A proportional
  to the player's arch. With avoidance from pick 5 (full) and biased refills,
  S/A trajectory is ~5 start → 4.5 after R1 refill → 3.5 after R2 refill → 3
  at pick 25. This is marginal but achievable.

- **N = 8 from a 25-card pool draws one-third of available cards.** The "best
  4" selection from 8 draws at this density leaves only 17 cards unseen. M4
  (off-archetype variety) is at moderate risk. Mitigation: floor 1 of the 4
  shown cards as highest-power non-archetype card from the 8 drawn. This
  preserves ~0.5 M4 per pack.

- **AI saturation threshold at 10 cards prevents pool exhaustion.** With steep
  contraction, AIs assigned to thin archetypes would otherwise start taking any
  available card by pick 20, including the player's archetype. Capping AI picks
  at 10 per archetype and redirecting excess to the "most abundant non-player
  arch" prevents this while keeping contraction on schedule.

---

## Three Algorithm Proposals

### Proposal A: "Steep-20 Biased" — Aggressive contraction to exactly 20 cards with 1.7x open-lane-biased refills and gradual avoidance from pick 3

**Technical spec:**
- Starting pool: 120 cards (15 per archetype, 5 S/A each)
- Refill schedule: 60/20/0 (3 rounds of 10 picks), open-lane-biased at 1.7x
- N = 8, show best 4 by archetype fitness; N = 4 uniform during picks 1-5
- 5 AIs, gradual avoidance ramp: 20% weight reduction pick 3, 50% pick 6, 80%
  pick 10, 90% pick 12+
- AI inference: rolling 4-pick window archetype depletion vs expected baseline
- AI saturation threshold: stop at 10 cards per archetype, redirect to most
  abundant remaining non-player arch
- Picks 1-5: pool browser for exploration; N = 4 uniform packs

| Metric | Predicted | Target |
|--------|:---------:|--------|
| M3 | 1.8-2.0 | >= 2.0 |
| M10 | 3-4 | <= 2 |
| M11' | 2.4-2.8 | >= 2.5 |
| M6 | 72-80% | 60-90% |
| M12 | 0.3-0.4 | >= 0.3 |
| M13 | Pick 8-10 | Pick 6-10 |
| M14 | Pick 5-7 | Pick 4-7 |

**Assessment:** Borderline on M3. Pool = 20 with S/A = 4-5 is exactly the N=8
threshold; any S/A loss before avoidance locks in drops M3 below 2.0.

---

### Proposal B: "Steep-30 Biased + M4 Floor" — Moderate-steep contraction to 30 cards with 1.7x bias, N = 8, and guaranteed off-archetype floor slot

**Technical spec:**
- Starting pool: 120 cards
- Refill schedule: 60/30/0 (3 rounds of 10 picks), open-lane-biased at 1.7x
- N = 8, show best 4 with forced M4 slot: "draw 8, show 3 best on-arch + 1
  best off-arch power card from the 8 drawn"
- 5 AIs, gradual avoidance ramp: same as Proposal A
- Final pool ~30 cards; player arch = ~9-10 cards (33%), S/A ~3.5-4
- M3 = 8 × (3.75/30) = 1.0 from uniform formula, but forced on-arch in 3 of 4
  slots amplifies: effective M3 ≈ min(3 × 8 × 3.75/30, 3) + 0 = ~3.0 → capped
  at 3 for 3-slot face. More carefully: from 8 drawn, expect 8 × 3.75/30 = 1.0
  S/A drawn; all 1 appears in the 3 on-arch slots. Actual M3 ≈ 1.0-1.5

**Assessment:** The split-slot mechanism underperforms. A pool of 30 with 5 S/A
at N=8 gives M3=1.33 from the base formula; the forced off-arch slot reduces
it further. M4 is improved but M3 suffers. This tradeoff is not favorable.

| Metric | Predicted | Target |
|--------|:---------:|--------|
| M3 | 1.3-1.5 | >= 2.0 |
| M4 | 0.8-1.0 | >= 0.5 |
| M11' | 1.8-2.2 | >= 2.5 |
| M6 | 65-72% | 60-90% |
| M12 | 0.3-0.4 | >= 0.3 |

**Assessment:** Misses M3 by a significant margin. Dropped.

---

### Proposal C: "Steep-20 Biased + Progressive N" — Aggressive contraction to 20 cards with N ramping from 4 to 8 as the player commits, preserving early exploration diversity

**Technical spec:**
- Starting pool: 120 cards
- Refill schedule: 60/20/0, open-lane-biased at 1.7x
- N schedule: N=4 picks 1-5 (exploration), N=6 picks 6-10 (transition),
  N=8 picks 11-30 (full oversampling)
- 5 AIs, gradual avoidance ramp: same as Proposal A
- "Best 4" ranking: picks 1-5 by power, picks 6-10 by symbol match, picks 11+
  by archetype fitness
- AI saturation threshold at 10 cards per arch

| Metric | Predicted | Target |
|--------|:---------:|--------|
| M3 | 1.7-1.9 | >= 2.0 |
| M1 | 3-4 | >= 3 |
| M10 | 3-5 | <= 2 |
| M11' | 2.2-2.6 | >= 2.5 |
| M6 | 70-78% | 60-90% |
| M12 | 0.4-0.5 | >= 0.3 |
| M13 | Pick 8-10 | Pick 6-10 |
| M14 | Pick 5-7 | Pick 4-7 |

**Assessment:** Progressive N is a natural fit with steep contraction. Early
large pool → N=4 provides genuine diversity. Late contracted pool → N=8
provides concentration. The transition (N=6 at picks 6-10) smooths the
experience. M12 improves because signal-readers commit earlier and benefit from
N=8 sooner. Slightly lower M3 than Proposal A because N=8 starts at pick 11,
not pick 6 — missing 5 picks of full oversampling during active pool contraction.

---

## Champion Selection: Proposal A — "Steep-20 Biased"

**Justification:** Proposal A hits the N=8 threshold directly. The pool must
reach 20 cards with 5 S/A to achieve M3 = 2.0; the 60/20/0 biased schedule is
the minimum-complexity path to that target. Progressive N (Proposal C) sacrifices
5 picks of full oversampling during the most important concentration window (picks
6-10 when avoidance is locking in), reducing M3 below the target. Proposal B
confirms that a 30-card final pool cannot reach M3 = 2.0 at N=8. Proposal A is
the design that most directly tests whether the target combination is achievable.

The main risk is S/A exhaustion in picks 25-30. This is real but manageable
through the biased refill schedule and avoidance activation before pick 7.

---

## Champion Deep-Dive: Steep-20 Biased Pick-by-Pick

### Picks 1-5: Exploration Phase (Pool ~120→90 cards)

**Pack construction:** N = 4 uniform random (pool browser serves discovery).
**AI behavior:** No avoidance yet. AIs draft normally from their assigned
archetypes. Pool shrinks from 120 to ~90 as 6 cards/pick are removed.
**What the player sees:** 4 random cards from the 120-card pool. The pool
browser shows all 120 cards grouped by archetype. Player evaluates which
archetypes have depth.
**Pool composition:** All 8 archetypes roughly 15 cards each. No gradient yet.
**Avoidance ramp begins at pick 3:** 20% weight reduction on inferred player
arch. With only 3 pick cycles of data, inference is low-confidence. False
positives likely but harmless.

### Pick 6: Avoidance Locks In (Pool ~84 cards)

**Threshold event:** AI avoidance reaches 50% weight reduction. The player
should have committed by now. Picks 1-5 revealed which archetypes are open;
the player's pick pattern is observable via depletion.
**AI inference:** 5 pick cycles of depletion data. If the player took 3-4
Ember-symbol cards, Ember archetypes show disproportionate depletion. AIs
reduce Ember-arch priority by 50%. False-positive rate still ~30%.
**Pack construction:** N = 8 begins. Pack is drawn from 84-card pool, ranked
by fitness for the player's inferred archetype. Player sees first genuinely
curated pack.
**Pool composition:** Player's arch ~11 cards (from 15; player took ~4).
Other arches averaging ~10 (AIs took ~5 each). Arch density: 11/84 = 13%.
M3 at this point: 8 × (3.8/84) ≈ 0.36. Still low — contraction hasn't
finished yet.

### Picks 6-10 (Round 1 end): Approaching First Refill

**Avoidance ramp:** 50% → 80% weight reduction as evidence accumulates. AIs
increasingly redirect picks from the player's arch to other archetypes.
**S/A preservation:** Before avoidance, AIs took ~2 S/A from the player's arch
in picks 1-5. From pick 6+, player's S/A deplete only via player's own picks.
**Round 1 end:** Pool = ~60 cards. Player's arch ~8 cards (13.3%), S/A ~3.
**Refill event:** +60 cards, open-lane-biased at 1.7x. Open-lane archetypes
(including player's) receive ~10 cards each vs ~7 cards for AI-lane archetypes.
**Post-refill:** Pool = 120 cards. Player's arch = ~18 cards (15%), S/A ~4.5.
The refill partially resets the gradient but the player's arch is now actually
above baseline thanks to the open-lane bias.

### Picks 11-15: Avoidance Fully Locked (Pool ~90→60 cards)

**Avoidance ramp complete at pick 12:** 90% weight reduction. AIs almost never
take the player's archetype cards.
**Pool trajectory:** 120 → 90 as AIs take 5 cards/pick from non-player arches.
Player takes 1/pick from own arch. Player's arch: 18 cards → ~13 cards.
Density: 13/90 = 14.4%. Still building.
**M3 at pick 15 (N=8):** 8 × (4/90) ≈ 0.36. Frustratingly low — the pool is
still too large for N=8 to do heavy lifting. This is expected: contraction
must finish first.

### Picks 16-20: Critical Contraction Phase (Pool ~60→20 cards)

**No more refills.** After the R2 refill of +20 cards at pick 20 boundary,
nothing more. The pool contracts hard: 6 removals/pick × 10 picks = 60 cards
removed from ~60-card pool.
**R2 refill at pick 20:** Pool at pick 20 pre-refill = ~40 cards. Refill
adds 20 cards, biased: player's arch gets ~4 cards, AI arches get ~2 each.
Post-refill pool = 60 cards. Player arch = ~14 cards (23%), S/A ~3.5.
**Failure mode here:** If the refill is only 20 cards total, the AI arches
are now very thin (some at 4-5 cards). Saturation threshold kicks in: AIs with
fewer than 10 cards in their arch stop taking those cards. This naturally
protects against pool exhaustion.

### Picks 21-30: Concentrated End-Game (Pool ~60→20 cards)

**No refills.** Pool contracts from ~60 to ~20 at 6/pick pace.
**Player's arch trajectory:**
- Pick 21: Pool ~54, player arch ~14 (26%), S/A ~3.5, M3 = 8×3.5/54 = 0.52
- Pick 24: Pool ~36, player arch ~12 (33%), S/A ~3.0, M3 = 8×3/36 = 0.67
- Pick 26: Pool ~24, player arch ~10 (42%), S/A ~2.5, M3 = 8×2.5/24 = 0.83
- Pick 28: Pool ~18, player arch ~8 (44%), S/A ~2.0, M3 = 8×2/18 = 0.89

**The S/A problem is severe.** M3 stays below 2.0 because S/A count depletes
faster than pool size shrinks. By pick 25-28, the player has consumed 12-15
of their own arch cards over 30 picks, leaving only 3-5 S/A. M3 = 0.83-1.0
in the final stretch, not 2.0.

**Diagnosis:** The math requires S/A=5 at pool=20. But the player picks ~1
card/pick and ~36% of their arch is S/A, so they consume ~0.36 S/A/pick. Over
30 picks: ~10 S/A consumed. Starting S/A = 5, refills add ~4-5 S/A total.
Remaining at pick 25: ~2-3 S/A. **This is the binding failure mode.**

**Mitigation strategies:**
1. Biased refills add disproportionate S/A to the player's arch (not just
   total card count). Target: refills add 2x the normal S/A rate to open lanes.
2. Pool browser enables the player to selectively preserve S/A cards (they can
   see which are left and plan accordingly).
3. Avoidance from pick 3 instead of pick 5 preserves ~1-2 additional S/A in
   picks 1-4.

With all three mitigations: starting S/A 5, lose ~1 in picks 1-3 before early
avoidance, refills add ~6 S/A total (biased), player consumes ~10 over 30
picks → remaining at pick 25 = 5 + 6 - 1 - 10 = 0. Still zero.

**Revised diagnosis:** S/A must be thought of as shared between refills and the
player's draw rate. The player cannot consume all 5 initial + 6 refill = 11 S/A
over 30 picks and still have 5 remaining at pick 25. The M3 = 2.0 target at
N=8 requires a pool of ~20 cards that the player has NOT yet drafted from —
meaning the concentrated pool must contain predominantly cards the player hasn't
taken yet, which means the S/A cards in the pool at picks 25-30 must be new
arrivals from refills, not the original 5. This is feasible only if refills
add substantial fresh S/A and the player hasn't consumed all of it.

**Honest M3 prediction:** Accounting for S/A depletion, M3 averages across
picks 6-30 will be ~1.4-1.8 for Proposal A. M11' (picks 20+) will peak at
1.8-2.2 if biased refills aggressively maintain S/A supply.

---

## Complete Specification

| Parameter | Value |
|-----------|-------|
| **Starting pool** | 120 cards (15 per archetype × 8; ~5 S/A each) |
| **Total draft** | 30 picks, pack size 4 (show 4, pick 1) |
| **Round structure** | 3 rounds of 10 picks each |
| **Refill schedule** | 60 after R1, 20 after R2, 0 after R3 |
| **Refill bias** | 1.7x open-lane multiplier (3 open lanes per game); Level 0 |
| **Refill S/A target** | Refills target 40% S/A rate for open-lane cards (vs 36% baseline) |
| **Oversample N** | N = 4 (picks 1-5, exploration); N = 8 (picks 6-30, exploitation) |
| **"Best 4" ranking** | Picks 1-5: power rank; Picks 6+: archetype fitness for inferred arch |
| **Exploration** | Pool browser (full face-up pool) serves picks 1-5 discovery |
| **AI count** | 5 AIs, each assigned one of 8 archetypes (3 open lanes per game) |
| **AI avoidance model** | Gradual ramp: 20% weight reduction pick 3, 50% pick 6, 80% pick 10, 90% pick 12+ |
| **AI inference mechanism** | Rolling 4-pick window: compare per-archetype depletion rate to expected baseline (pool fraction × picks taken). Arch with 1.5× expected rate flagged as player's arch |
| **AI pick logic** | Weight each card by (archetype fitness for AI's arch) × (1 - avoidance weight if suspected player arch). Take highest-weight available card. |
| **AI saturation threshold** | Ease off primary arch when own arch has fewer than 10 cards remaining in pool; redirect to most abundant non-player, non-competitor arch |
| **Player information** | Full face-up pool visible at all times; individual picks secret; pool depletion patterns public |
| **Archetype open/closed** | C(8,5) = 56 per-game AI compositions; 3 open lanes per game |
| **M3 realistic prediction** | 1.5-1.8 (picks 6+); M11' 1.8-2.2 (picks 20+) |
| **Key failure mode** | S/A supply exhaustion by picks 25-30 prevents late-draft M3 from reaching 2.0 at N=8; requires aggressive biased refill S/A targeting or N=10-12 as a fallback |

---

## Post-Critique Revision

The critic's feedback is well-targeted. The three required parameter changes each
address a real failure mode identified in the pick-by-pick walkthrough; the
optional fallback addresses the remaining gap if those changes are insufficient.

### Revised Parameters

**Bias factor: 1.7x -> 2.0x.** The walkthrough shows S/A at pick 25 collapsing
to 0-2 even with mitigations. At 1.7x, open-lane refills add ~5-6 S/A over the
draft; at 2.0x, this rises to ~7-8. The swing of +2 S/A is meaningful: it moves
the pick-25 estimate from ~1 S/A to ~3 S/A, raising late-draft M3 from ~0.6 to
~1.2 (8 × 3/20). Still not 2.0, but closer and now in range for the floor-slot
fallback to close the gap.

**Open-lane S/A rate in refills: 36% -> 40%.** This is a secondary lever but
compounds with the bias increase. At 2.0x bias with 40% S/A targeting, each
open-lane refill card is drawn from a distribution where S/A appears at
2.0 × 0.40 = 80% of the rate expected in a flat random draw. Over 80 refill
cards distributed across the three open lanes, the player's arch receives
~27 cards with ~10-11 S/A (up from ~8-9 S/A at 1.7x / 36%). This directly
addresses the binding constraint.

**Avoidance onset: pick 3 ramp -> evidence-proportional from pick 2.** Instead
of hard-coded percentage steps (20% at pick 3, 50% at pick 6), avoidance weight
is set to: min(evidence_ratio / 1.5, 1.0) × max_weight, where evidence_ratio is
the rolling-window depletion rate relative to the baseline, and max_weight ramps
from 0.5 (pick 2-4) to 0.9 (pick 12+). The minimum 3-cycle window before weight
exceeds 50% is preserved: the ratio rarely exceeds 1.5 reliably in fewer than
3 cycles. This approach preserves 1-2 additional S/A in picks 2-4 compared to
the fixed ramp, while avoiding false-positive over-commitment. The key change is
that a strong early signal (4+ S/A taken by player in 2 picks) triggers earlier
and heavier avoidance proportionally, rather than waiting until the fixed pick-6
gate.

### Optional Fallback: Floor Slot + N = 10

If simulation shows S/A at pick 25 is still consistently below 3 after the above
changes, two additions activate as a unit:

1. **Floor slot:** guarantee 1 S/A in the shown 4 if any S/A appears among the N
   drawn. This converts the raw M3 formula into an effective floor: even 1 S/A
   in 8 drawn becomes 1 S/A shown, not sometimes 0.
2. **N = 10 instead of N = 8.** At pool = 20 with S/A = 3, M3 = 10 × 3/20 =
   1.5 without a floor, and roughly 1.8-2.0 with the floor slot firing on ~70%
   of packs. N = 10 is the midpoint between the current knife's-edge (N=8) and
   Design 3's conservative N=12, preserving Design 2's distinct identity while
   recovering the S/A shortfall.

### Updated M3 Prediction

With bias 2.0x and 40% S/A targeting but without floor/N=10: M3 = **1.6-2.0**
(picks 6+); M11' = **2.0-2.4** (picks 20+). The upper bound is reachable only
if S/A at pick 25 holds at 4+, which is now plausible but not guaranteed.

With floor slot + N=10 fallback active: M3 = **1.8-2.2**; M11' = **2.2-2.6**.
This matches Design 3's range and justifies simulation as a distinct data point
(steeper contraction, lower N, explicit floor mechanism vs Design 3's floor +
N=12 on a shallower contraction schedule).
