# V12 Design 3: Moderate Contraction + Standard Oversampling (N = 12)

## Key Findings

- **Delayed avoidance (pick 8+) costs 2-4 S/A in picks 1-7.** With 5 AIs
  taking randomly, ~0.6 player-arch cards exit per cycle = ~3-4 player-arch
  cards gone before avoidance preserves anything. Open-lane biased refills
  (1.7x) at pick 10 are essential to recover this S/A loss.

- **Balanced refills reset concentration; biased refills compound it.** A 60-
  card balanced refill restores ~7.5 cards per archetype uniformly. At 1.7x
  open-lane bias, the player's lane gets ~8-10 refill cards vs ~5 for AI lanes.

- **N = 12 reaches M3 = 2.0 at pool = 30 cards with 5 S/A.** M3 = 12*5/30 =
  2.0. The 60/36/0 schedule (96 refills) leaves ~36 cards — above the threshold.
  The 50/30/0 schedule (80 refills) reaches ~20-25 cards, inside the target.

- **Pair-affinity ranking outperforms symbol-only by ~0.2-0.3 M3.** Symbol-only
  misidentifies ~20-25% of high-affinity cards with wrong or no symbol. The 8-bit
  metadata corrects these, lifting M3 without visible complexity.

- **Floor slot within N = 12 is the key delayed-avoidance compensator.** "Draw
  12, guarantee 1 S/A if any drawn" fires ~60-80% of packs in picks 8-20,
  adding ~0.3-0.5 to M3 and reducing M10 by eliminating zero-S/A packs.

- **60/36/0 falls short; 50/30/0 + floor slot reaches M3 = 2.0.** Steepening
  to 50/30/0 contracts the pool to ~20-25 cards where N = 12 alone achieves
  M3 = 2.0-2.4. Combining both is the robust design.

- **V9 contracted 21:1 (360 to 17); Design 3 contracts ~5:1 (120 to 20-25).**
  Lower ratio requires N = 12 + floor slot to compensate. V9 achieves M3 = 2.70;
  Design 3 targets M3 = 2.0-2.2 via transparent physical mechanisms.

---

## Three Algorithm Proposals

### Proposal A: "Delayed Guardian"

**Description:** Moderate 60/36/0 declining refills with 1.7x open-lane bias,
delayed avoidance (pick 8), N = 12 ranked by pair-affinity, no floor slot.

**Technical spec:** Pool 120 cards. Refills 60/36/0 after picks 10/20/30 with
1.7x open-lane bias. 5 AIs (3 open lanes). Avoidance starts pick 8: 50% weight
reduction ramping to 70% by pick 12. Inference: archetype depleting at >= 1.5x
expected rate over 3 pick cycles flagged as player archetype. N = 4 picks 1-5,
N = 12 picks 6-30. Pair-affinity ranking. No floor slot. Final pool: ~36-40
cards.

**Predicted:** M3 = 1.6-1.8, M10 = 4-5, M11' = 2.0-2.2, M6 = 65-75%,
M12 = 0.25-0.35, M13 = 8-10, M14 = 5-7.

*Shortfall: Final pool too large; no floor slot. Does not reach M3 = 2.0.*

---

### Proposal B: "Moderate Pressure + Floor" (Champion)

**Description:** Steepened 50/30/0 declining refills with 1.7x open-lane bias,
delayed avoidance (pick 8), N = 12 with floor slot.

**Technical spec:** Pool 120 cards. Refills 50/30/0 after picks 10/20/30 with
1.7x open-lane bias. 5 AIs (3 open lanes). Avoidance sigmoid ramp: 30% pick 8,
60% pick 10, 80% pick 14+. Inference: symbol-weighted depletion count, flag at
>= 1.8x expected rate over 3 cycles. N = 4 picks 1-5, N = 12 picks 6-30.
Pair-affinity ranking. Floor slot: guarantee 1 S/A in shown 4 if any S/A drawn
in the 12. Final pool: ~20-25 cards.

**Predicted:** M3 = 2.0-2.2, M10 = 3-4, M11' = 2.4-2.8, M6 = 72-82%,
M12 = 0.30-0.40, M13 = 8-10, M14 = 5-7.

*Assessment: Steepened contraction (20-25 card final pool) + floor slot crosses
M3 = 2.0. Open-lane bias compensates for S/A lost pre-avoidance. Champion.*

---

### Proposal C: "Pair-Affinity Precision"

**Description:** 60/36/0 balanced refills, delayed avoidance, N = 12 with
double floor slot to compensate for larger pool.

**Technical spec:** Pool 120 cards. Refills 60/36/0 balanced (no open-lane
bias). 5 AIs. Avoidance pick 8, 70% weight reduction. N = 12. Enhanced double
floor: "if 2+ S/A drawn, guarantee 2 appear in shown 4." Pair-affinity ranking.
Final pool: ~36 cards.

**Predicted:** M3 = 1.7-1.9, M10 = 5-6, M11' = 2.0-2.3, M6 = 60-70%,
M12 = 0.15-0.25, M13 = 8-10, M14 = 5-7.

*Shortfall: Balanced refills suppress M12 and reduce density gradient. Double
floor compensates partially but cannot fix the 36-card pool without bias.*

---

## Champion: Proposal B — "Moderate Pressure + Floor"

Proposal B is the only design in this space that credibly achieves M3 >= 2.0.
The 50/30/0 schedule contracts to ~20-25 cards by pick 30 — exactly at the N = 12
threshold. Open-lane bias (1.7x) compounds the delayed avoidance: the R1 refill
replenishes the player's lane disproportionately, recovering S/A lost in picks
1-7. The floor slot eliminates zero-S/A packs in the mid-draft when the pool is
still large and contraction alone is insufficient. Pair-affinity ensures the
guaranteed S/A is the *right* one for the committed archetype. Proposal A lacks
the floor slot and stays above the contraction threshold. Proposal C suppresses
M12 via balanced refills.

---

## Champion Deep-Dive: Pick-by-Pick Walkthrough

**Setup:** 120 cards, 8 archetypes (15 each, ~5 S/A each). 5 AIs on 5
archetypes; player's archetype is one of 3 open lanes. Refills 50/30/0.
N = 12 from pick 6. Pair-affinity ranking. Floor slot active picks 6+.

**Picks 1-5 (Exploration).** N = 4 uniform packs. Player browses face-up pool.
AIs take from assigned lanes; ~0.6 player-arch cards taken per cycle by random
AI picks. Pool end: ~90 cards, player arch ~12, ~4 S/A.

**Picks 6-7 (Inference window).** N = 12 engages. Player-arch signal ~1.25x
expected — below the 1.8x threshold. No avoidance yet; ranking defaults to
power-score fallback. No floor slot fires reliably (pool ~80 cards, few S/A
in any 12-card draw).

**Pick 8 (Avoidance activates).** 3 pick cycles of depletion data. Player arch
at ~1.8-2.0x expected depletion rate. 30% weight reduction begins (sigmoid
ramp). Floor slot fires ~49% of packs. Pool: ~78 cards, player arch ~11, ~3.5
S/A.

**Picks 8-10 (Pre-refill).** Avoidance ramps 30% to 60%. Pool at pick 10
pre-refill: ~60 cards, player arch ~10, ~3 S/A, density 16.7%.

**Pick 10 Refill: 50 cards, 1.7x open-lane bias.** Player arch: 10 + 8.4 = 18
cards, ~6 S/A. Pool: ~110 cards. Key insight: biased refill restores the S/A
lost in picks 1-7 — player now has *more* S/A than at draft start.

**Picks 11-20 (Full avoidance, 80% by pick 14).** Pool trajectory: pick 15 ~80
cards, player arch ~15, S/A ~5, M3 ~1.3-1.5. Pick 20 pre-refill: ~56 cards,
player arch ~10, S/A ~3.5.

**Pick 20 Refill: 30 cards, 1.7x open-lane bias.** Player arch: 15 cards, ~5.3
S/A. Pool: ~86 cards. Visible in face-up pool: player's lane replenishes more
than AI lanes.

**Picks 21-30 (Endgame contraction, no refills).** Pool shrinks 6/cycle.
Pick 25: ~62 cards, player arch ~10, S/A ~3.5; floor fires ~82%. Pick 30: ~32
cards, player arch ~6, S/A ~2.5; N=12 expected 0.94; floor fires ~88%; M3
reaches ~2.0-2.1. Late contraction drives the density needed for M3 target.

**Failure modes:**
1. *Inference failure:* Wrong archetype flagged; cost 1-2 S/A. AIs re-read on
   any 3 cycles showing new depletion patterns.
2. *S/A exhaustion:* Floor slot ensures last S/A cards always appear when
   available; risk only if player exhausts all S/A before pick 20.
3. *Post-refill M3 dip:* 2-3 packs after each refill see density reset. M10 =
   3-4, with bad packs concentrated at refill events rather than random.

---

## Complete Specification

| Parameter | Value |
|-----------|-------|
| Starting pool | 120 cards (15 per archetype, ~5 S/A per archetype) |
| Total picks | 30 (player); 5 AIs pick each cycle |
| Pack size shown | 4 cards |
| Oversample N | 4 (picks 1-5); 12 (picks 6-30) |
| "Best 4" ranking | Pair-affinity scores (8-bit hidden metadata); S/A of inferred archetype ranked highest |
| Floor slot | Picks 6+: if any S/A cards drawn in N=12 sample, guarantee 1 appears in shown 4. Zero S/A drawn: show best 4 by pair-affinity, no guarantee. |
| Refill schedule | 50 cards after pick 10; 30 after pick 20; 0 after pick 30 |
| Refill bias | 1.7x open-lane: each of 3 open-lane archetypes receives 1.7x cards vs each of 5 AI-lane archetypes. Bias determined at draft start by AI assignment; player-independent. |
| AI count | 5 |
| AI lane structure | 5 AIs on 5 archetypes; 3 archetypes open (uncontested) |
| AI avoidance model | Delayed: inference window picks 5-7; sigmoid ramp 30% (pick 8), 60% (pick 10), 80% (pick 14+). |
| AI inference mechanism | Symbol-weighted aggregate depletion over 3 pick cycles. Flag archetype when >= 1.8x expected rate. Expected = archetype fraction of pool * picks per cycle. |
| AI pick logic | Score cards: pair-affinity for assigned archetype * (1 - avoidance weight for inferred player archetype). Pick highest scorer. Saturation at 12 primary-arch cards: reduce primary weight 50%, expand to sibling. |
| Player information | Full face-up pool browsable at all times. Cards disappear as drafters take; who took what is secret. System draws 12, shows best 4 by pair-affinity. Pool browser supports archetype and resonance filtering. |
| Exploration support | Picks 1-5: N=4 uniform packs; pool browser serves exploration. N=12 + pair-affinity ranking engage at pick 6. |
| Archetype inference (ranking) | Weighted resonance signature (+2 primary, +1 secondary per pick). Pair-affinity identifies best-matching archetype pair. Starts pick 3, confidence-weighted until pick 8. |

---

## Post-Critique Revision

### Acknowledgment

The critic's two concerns are valid. First, the floor slot fires only when S/A
cards are present in the N=12 draw — if S/A exhaustion leaves the late pool
with zero S/A, the floor slot provides no guarantee and M3 collapses to whatever
pair-affinity ranking delivers from non-S/A cards. Second, pair-affinity ranking
uses hidden 8-bit metadata for pack curation; if the design can reach M3 >= 2.0
without it, the transparent version is preferable and the honesty tradeoff should
not be accepted without evidence that it is necessary.

### Simulation Variants

Two variants of Proposal B (Champion) must be simulated:

**Variant B1 — Pair-Affinity Ranking (as specified):** "Best 4" selected using
8-bit pair-affinity scores. This is the champion as written. Expected M3: 2.0-2.2.

**Variant B2 — Visible-Symbol-Only Ranking:** "Best 4" selected by visible
resonance symbol match to inferred archetype. No pair-affinity metadata. All
other parameters identical: 50/30/0, 1.7x bias, N=12, delayed avoidance (pick
8), floor slot active. Expected M3: 1.7-2.0 (pair-affinity estimated to
contribute 0.2-0.3 M3).

If Variant B2 achieves M3 >= 2.0, the pair-affinity layer is dropped and B2
becomes the recommended design. If B2 falls to 1.7-1.9, pair-affinity is
retained as a necessary mechanism and documented explicitly as a honesty
tradeoff: the system curates packs using information the player cannot verify.

### Updated Tracking Requirements

The simulation must record the following beyond standard metrics:

1. **Floor slot firing rate by pick band:** Track separately for picks 6-10,
   11-20, and 21-30 the fraction of packs where the N=12 draw contains zero S/A
   (floor slot fails to fire). If zero-S/A draws exceed 30% in the 21-30 band,
   the S/A supply at late draft is insufficient and M3 will miss target
   regardless of pool size. If this threshold is exceeded, the corrective is to
   raise the R1/R2 refill bias from 1.7x to 2.0x or add an S/A-targeted refill
   component (open-lane refills set S/A at 40% vs 36% baseline).

2. **S/A count at picks 20, 25, 30:** Confirm the walkthrough's predicted values
   (~5.3, ~3.5, ~2.5). A shortfall at pick 20 signals that delayed avoidance
   (picks 1-7) depleted more S/A than the R1 biased refill recovered.

3. **Pair-affinity M3 delta:** B1 M3 minus B2 M3. If delta < 0.1, pair-affinity
   is not earning its honesty cost and should be dropped regardless of whether
   B2 reaches 2.0.

### Honesty Tradeoff Statement

If pair-affinity is required (B2 < 2.0): the design uses hidden metadata for
pack construction, not for AI inference. AI avoidance remains public-information
only (visible symbol depletion). The hidden step is the system's internal ranking
of the N=12 draw, which the player cannot verify from the pool browser. This is
a Level 2 mechanism in the pack-curation layer only. If this tradeoff is
accepted, it must be documented in the player-facing design rationale as "the
system prioritizes cards with strong archetype synergy" without specifying the
scoring mechanism.

### No Other Adjustments

The champion specification (Proposal B) requires no other changes pending
simulation results. The 50/30/0 schedule, 1.7x bias, delayed avoidance sigmoid,
and floor slot are unchanged. If the floor slot firing rate tracking reveals S/A
exhaustion at the 30% threshold, increase refill bias to 2.0x as the first
corrective before any structural changes.
