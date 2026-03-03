# V9 Critic Review: Round 2 Algorithm Analysis

**Critic Agent — Round 3**

______________________________________________________________________

## 1. Overview

Six Round 2 agents each proposed three algorithms and selected a champion. All
six champions converge on the same hidden metadata primitive: a 3-bit archetype
tag per card (1,080 bits total for 360 cards). This convergence is not a sign
that agents copied each other — they arrived independently at the same minimum
viable unit of hidden information, exactly as Research Agent B predicted. The
real differentiation is in *how* the tag is used and *how much* additional
mechanism is layered on top of it.

The central V9 question — minimum hidden information to reach M3 >= 2.0 and M11
\>= 3.0 while keeping visible resonance primary — has a clear answer across these
proposals: 3-bit archetype tag + pool contraction. Every design that achieves
M11 >= 3.0 includes pool contraction; none achieves M11 >= 3.0 without it. This
is the structural finding of Round 2.

______________________________________________________________________

## 2. Strongest Proposal: Design 4 (Layered Salience)

**Champion:** Layered Salience (Two-Stage Filter)

**Why it leads:**

Design 4 achieves the best V1 score of any proposal — an estimated 79% visible
symbol influence — by making architectural choices that force visible symbols to
do primary work. The two-stage structure is mechanically principled:

- **Stage 1:** R1 filtering gates three of four slots to the committed primary
  resonance pool (pure visible).
- **Stage 2:** Within the R1-filtered pool, hidden archetype tags provide a 4x
  weighting toward home-tagged cards.
- **Phase 3:** Late-draft contraction (pick 12+) at 8% per pick concentrates the
  pool for M11.

This architecture is uniquely honest: the hidden layer cannot substitute for
visible commitment because Stage 1 never activates without visible symbol
alignment. A power-chaser who ignores resonance gets M3 ≈ 1.8-2.0 — visible
resonance produces a 0.4-0.6 M3 gap over the power-chaser, satisfying Research
Agent C's gap test (>= 0.4 required). The V4 metric (80-85% pick alignment) is
the highest among all proposals, confirming visible choices are driving most
decisions.

**Predicted metrics:** M3 ≈ 2.45, M10 ≈ 2-3, M11 ≈ 3.0-3.1, M6 ≈ 80-85%, V1 ≈
79%, V2 = 3 bits/card, V3 = 8/10.

**Genuine weakness:** The M3 prediction of 2.45 is the lowest among proposals
that achieve M11 >= 3.0. There is meaningful headroom to the nearest competitor
(Design 2 at 2.55-2.70). Whether 2.45 leaves enough margin given Pessimistic
fitness and Flash/Ramp worst-case is uncertain — agents should confirm that
Flash/Ramp stay comfortably above 2.0 under the 4x weighting scheme given the
small R1 pool size (80 cards, split 40/40 home/sibling). A second weakness: the
late-contraction-only mechanism (picks 12+) means M10 improvement relies
entirely on the R1 slot plus the 4x weighting. M10 = 2-3 is borderline.

______________________________________________________________________

## 3. Weakest Proposal: Design 3 (CSCT-2+C)

**Champion:** CSCT-2+C (Commitment-Scaled Continuous Targeting, Detuned +
Contraction)

**Why it is weakest:**

Design 3's champion is not a bad algorithm — it is a defensible one — but it is
the weakest among the six for specific reasons:

**V1 is the lowest at ~78%, but this number is misleading.** The Design 3 agent
calculates V1 as (2.10 - 0.125) / (2.65 - 0.125) ≈ 78%, claiming the visible
commitment ratio provides ~78% of the gain. This calculation assumes a
visible-only version of CSCT-2 (using R1 filtering for the targeted slots)
achieves M3 ≈ 2.10. However, this visible-only baseline is actually close to the
maximum visible-only ceiling established by Research Agent B. When hidden tags
are added to the targeted slots, they lift slot precision from P ≈ 0.58 to P ≈
1.0 — nearly doubling the targeted slot quality. The claimed V1 = 78% implies
the visible mechanism does most of the work, but the doubling of slot precision
from P=0.58 to P=1.0 is the most consequential improvement in the system. This
deserves skepticism.

**The mechanism is the most complex of all proposals.** CSCT-2+C has three
independent sub-systems running simultaneously: (1) a commitment ratio engine
tracking a rolling window of picks, (2) 2 hard-capped archetype-tagged slots per
pack, (3) a pool contraction layer with a 60/40 blended relevance score. This is
more moving parts than any other proposal. The V9 principle of minimal hidden
systems explicitly penalizes complexity.

**The commitment ratio mechanism adds fragility.** The rolling-window reset
(last 8 picks) creates a pivot recovery mechanism — sensible — but the early
disambiguation of Warriors vs. Sacrifice within Tide (necessary for the
tagged-slot system) must happen at picks 5-6. This is the pair alignment problem
that V8's discrete pair counters failed on. CSCT-2+C does not use discrete
counters, but inferring the exact archetype at pick 6 still requires enough
signal. The R1 slot fallback partially mitigates this, but the two-system
approach (CSCT targeting + contraction) creates potential inconsistencies when
the commitment ratio engine and contraction relevance engine disagree.

**M10 is not clearly fixed.** Predicted M10 = 2-4 is a wide range. The most
honest value is the upper end of that range, given V8's finding that the
transition zone (picks 6-10) is structurally resistant to M10 improvement.

CSCT-2+C is not eliminated — its predicted M3 range (2.55-2.75) is promising —
but its complexity cost and V1 accounting concerns make it the weakest candidate
for clean simulation interpretation.

______________________________________________________________________

## 4. Convergent Mechanisms Across All Proposals

All six champions share these mechanisms, confirming they are structural
requirements, not design preferences:

**Convergent 1: 3-bit archetype tag as the hidden metadata primitive.** Every
champion uses exactly a 3-bit archetype tag per card (0 bits from Design 6 for
the champion, but Design 6 also uses 3 bits). Research Agent B's analysis is
confirmed: this is the minimum unit that lifts M3 meaningfully and enables M11

> = 3.0 via contraction.

**Convergent 2: Pool contraction for M11 >= 3.0.** Every champion that predicts
M11 >= 3.0 uses pool contraction. Design 1 (Proposal B, Tagged R1 Bias) and
Design 5 (Proposal B, ALSF) explicitly fail M11 without contraction. This is the
definitive finding: M11 >= 3.0 is not achievable through slot-filling alone at
10% visible dual-res.

**Convergent 3: A quality floor slot from pick 3 (top-quartile draw).** Designs
1, 2, 3, 5, and 6 all include this mechanism. It is the most reliable M10
mitigation available within the Narrative Gravity family.

**Convergent 4: Archetype inference from drafted cards, not from a lock
mechanism.** All six proposals use continuous inference (mode of hidden tags,
dominant resonance, commitment ratio) rather than discrete pair locking. V8's
structural finding that discrete counters fail catastrophically is unanimously
adopted.

**Convergent 5: 60/40 or similar visible/hidden blend in contraction
relevance.** Designs 1, 2, 3, and 6 all propose a weighted blend of visible
dot-product and hidden tag matching for contraction relevance scores, typically
40-60% visible. This ratio appears to be the practical range where V1 stays
above 40% while hidden tags provide enough precision for contraction to
distinguish same-primary- resonance siblings (Warriors vs. Sacrifice within Tide
cards).

______________________________________________________________________

## 5. Best Hidden/Visible Split: Designs 4 and 6

**Design 4 (Layered Salience)** has the best V1 score (79%) and the most honest
architectural guarantee that visible symbols do primary work. The layered
structure — R1 filtering first, tag weighting second — ensures the hidden layer
cannot substitute for visible commitment. V1, V3, and V4 scores are all strong.
V2 is minimal (3 bits/card).

**Design 6 (Anchor-Scaled Contraction)** has the most innovative approach to
visible salience: differentiated contraction rates based on pick type (6%
generic, 10% single-symbol, 18% dual-resonance). This creates a visible
cause-and-effect that no other proposal achieves: "taking a (Tide, Zephyr) card
made my next 2-3 packs noticeably better." This is Research Agent C's "quality
felt as earned" pattern — the strongest driver of visible salience according to
the psychological research. The anchor mechanic directly addresses V4 in a way
that mere tag-weighting cannot: the player has a specific visible action (taking
a dual-resonance card) that they can observe producing a visible effect.

Design 6's V4 is likely the highest of all proposals in terms of player-felt
salience (though the quantitative estimate, ~55% V1, is moderate). Design 4 has
the best quantitative V1 score; Design 6 has the best qualitative V4 experience.

Both satisfy V1-V4 constraints. For the hidden/visible split question, these two
proposals represent the honest frontier: visible resonance is genuinely causal,
hidden metadata is genuinely refinement, and a player who discovered the system
would endorse it.

______________________________________________________________________

## 6. Flag Check: Cheating (V3 < 5) and Decorative Visible Resonance (V1 < 40%)

**No proposal is flagged for V3 < 5 (cheating).** All six champions propose
archetype tags derived from genuine card mechanics — the gold standard of
defensible hidden information. Tags are assigned by asking "which archetype does
this card play best in?", a question any game-literate player would endorse. V3
scores of 8/10 across five proposals and 9/10 for Design 5 reflect this
consistently honest approach.

**Design 5 (AWNG) warrants a V1 caution flag.** Reported V1 ≈ 40-45% is at or
below the threshold where visible resonance starts feeling decorative. The
affinity contraction is driven primarily by hidden affinity scores (8 floats),
which contain substantially more information than the visible primary resonance
symbol provides. From pick 6 onward, the algorithm's decisions are mostly
determined by the hidden affinity vector, not by the visible symbol.

However, Design 5 does not fall below V1 = 40% in its own estimate (40-45%), and
Research Agent C's threshold is ~40-50% for the deception risk. Design 5 is a
borderline case, not a clear failure. The flag is: **simulate Design 5 with V1
measured precisely**. If V1 < 40%, the visible resonance symbols are
insufficiently causal and the system should be modified or eliminated.

**Design 2's Proposal B (Tag-Gravity-Pure)** was explicitly noted by the agent
as having V1 ≈ 25-35%, below threshold. The agent correctly chose Tag-Gravity
(the 60/40 blend) over this. This is an internal proposal, not the champion — no
flag needed on the champion selection.

______________________________________________________________________

## 7. Rankings

All 6 champions ranked across five dimensions. Scores are 1-6 (1 = best).

| Rank | Design                               | M3 Potential | Player Experience | V3 (Integrity) | V1/V4 (Salience) | V2 (Minimality) |
| :--: | ------------------------------------ | :----------: | :---------------: | :------------: | :--------------: | :-------------: |
|  1   | Design 4 (Layered Salience)          |      5       |         3         |    2 (tie)     |      **1**       |     1 (tie)     |
|  2   | Design 6 (Anchor-Scaled Contraction) |      4       |       **1**       |    2 (tie)     |        2         |     1 (tie)     |
|  3   | Design 2 (Tag-Gravity)               |      2       |         2         |    2 (tie)     |        4         |     1 (tie)     |
|  4   | Design 1 (Tagged Narrative Gravity)  |      3       |         3         |    2 (tie)     |        3         |     1 (tie)     |
|  5   | Design 3 (CSCT-2+C)                  |   2 (tie)    |         4         |    2 (tie)     |        5         |        6        |
|  6   | Design 5 (AWNG)                      |      1       |         3         |     **1**      |        6         |        5        |

**Notes on scoring:**

- **M3 Potential:** Design 5 (2.65-2.80) and Design 2 (2.55-2.70) lead; Design 4
  (2.45) trails but still comfortably above target.
- **Player Experience:** Design 6's anchor mechanic creates the most legible
  visible feedback loop. Design 2 inherits V8 Narrative Gravity's 7.9/10
  experience. Designs 1 and 4 have the same mechanism family.
- **V3 (Integrity):** All proposals use mechanically-derived tags; Design 5
  earns 9/10 for published derivation rules, all others 8/10.
- **V1/V4 (Salience):** Design 4's layered architecture structurally guarantees
  V1 >= 60%; Design 6's anchor mechanic creates the strongest player-felt V4.
  Design 5 is weakest by V1 estimate.
- **V2 (Minimality):** Designs 1, 2, 3, 4, and 6 all use 3 bits/card (tied at
  minimum). Design 3's additional mechanism complexity is the tiebreaker
  penalty. Design 5 uses 16-32 bits/card.

______________________________________________________________________

## 8. Proposed Hybrid Designs

### Hybrid A: "Visible-First Anchor Gravity" (Designs 4 + 6)

**The gap this fills:** Design 4 has the best V1 architecture but lacks Design
6's visible feedback mechanism (the anchor mechanic). Design 6 has the best V4
experience but no layered visible-first guarantee on slot construction.

**How it works:**

- Stage 1 (visible): R1 filtering gates 3 slots to the committed primary
  resonance pool, exactly as in Design 4. One slot is always random (splash
  window).
- Stage 2 (hidden): Within R1-filtered slots, archetype tag provides 4x
  weighting toward home-tagged cards, as in Design 4.
- Anchor-scaled contraction: pool contraction rate is 6%/10%/18% based on pick
  type (generic/single-symbol/dual-resonance), as in Design 6.
- Contraction relevance: 60% visible dot-product + 40% hidden tag match. Applies
  from pick 5, not pick 12 — earlier than Design 4's late-phase-only
  contraction.

**What this achieves:** V1 remains structurally high (layered architecture from
Design 4) while the anchor mechanic creates legible visible feedback (Design 6).
Earlier contraction start (pick 5 vs. pick 12) should strengthen M11 vs. Design
4 while maintaining the V4 visible feedback loop. Predicted: M3 ≈ 2.50-2.60, M11
≈ 3.1-3.2, V1 ≈ 70-75%, V2 = 3 bits/card, V3 = 8/10.

**Why simulate this:** It directly tests whether the anchor mechanic is
compatible with the layered visible-first architecture. If it is, this hybrid
may outperform both parents on the V1/V4 balance.

### Hybrid B: "Affinity-Tagged Gravity" (Designs 2 + 5, minimized)

**The gap this fills:** Design 5 (AWNG) has the best V3 (9/10, published
derivation rules) but V1 < 50% and substantially more hidden information than
needed (16-32 bits/card). Design 2 (Tag-Gravity) has the minimum hidden
information and solid M3/M11 but V3 = 8/10 with the acknowledged
one-archetype-per-card simplification.

**How it works:**

- Use Design 2's Tag-Gravity mechanism (pool contraction, 60/40 blend, floor
  slot).
- Replace the binary archetype tag with a two-float pair: one affinity score for
  each of the two archetypes sharing the card's primary resonance symbol. A Tide
  card has (warriors_affinity, sacrifice_affinity) instead of a single tag.
- Two floats at 4-bit resolution = 8 bits per card (vs. 3 bits for a tag, vs.
  16+ bits for full affinity vector). This is the minimum information needed to
  handle multi-archetype bridge cards without forcing a single tag assignment.
- The contraction relevance uses the two-float affinity for the committed
  archetype: relevance = 0.4 * visible_dot + 0.6 \*
  affinity_score[committed_archetype].
- Cards genuinely strong in both archetypes (bridge cards) survive contraction
  longer — they have high affinity for both, so they remain relevant regardless
  of which Tide archetype the player is committed to.

**What this achieves:** V3 improves to 9/10 (no forced single-archetype
simplification; each card's real cross-archetype value is encoded). V2 rises
modestly from 3 to 8 bits/card. M3 and M11 are essentially unchanged from
Tag-Gravity (the performance ceiling is similar across Level 1 and Level 3 per
Research Agent B). The design integrity gain is real and the information cost is
minimal: 8 vs 3 bits per card, vs. 64+ bits for full affinity.

**Why simulate this:** It directly tests whether the V3 improvement from
encoding genuine cross-archetype affinity is worth the 2.7x information cost
over a binary tag. If not, the binary tag is confirmed as the right abstraction.

______________________________________________________________________

## 9. Recommendation: Algorithms to Advance to Simulation

**6 slots: the following should be simulated.**

1. **Design 4 (Layered Salience)** — Advances unchanged. Strongest V1
   architecture. Essential for understanding how much M3 headroom is sacrificed
   by visible-first design. This is the V9 integrity benchmark.

2. **Design 6 (Anchor-Scaled Contraction)** — Advances unchanged. Unique player
   experience contribution from anchor mechanic. Essential for understanding
   whether differentiated contraction rates are worth implementing vs. uniform
   rates.

3. **Design 2 (Tag-Gravity, 60/40 blend)** — Advances unchanged. This is the
   clearest direct successor to V8's Narrative Gravity with minimum hidden
   information. Baseline for all contraction-with-tag comparisons.

4. **Hybrid A: Visible-First Anchor Gravity** (Designs 4 + 6) — New proposal.
   Displaces Design 1, which is closely redundant with Design 2 (same mechanism
   family, similar predicted numbers, no new differentiation). Design 1's Tagged
   Narrative Gravity adds a 0.5 tag bonus on top of V8's Narrative Gravity but
   does not fundamentally differ from Tag-Gravity (Design 2's champion). Hybrid
   A is the more valuable test: does the anchor mechanic work within a layered
   visible-first architecture?

5. **Design 5 (AWNG, Affinity-Weighted Narrative Gravity)** — Advances with
   mandatory V1 monitoring. This is the only proposal with substantially more
   hidden information (16-32 bits/card vs. 3 bits). Its V3 = 9/10 justifies
   simulation to answer: is 5x more information worth 0.15-0.25 more M3 and
   genuinely better design integrity? If V1 < 40% in simulation, the proposal is
   eliminated. If V1 >= 40% and M3 >= 2.5, it is a viable "full hidden support"
   recommendation for the final synthesis.

6. **Hybrid B: Affinity-Tagged Gravity** (Designs 2 + 5 minimized) — New
   proposal. Displaces Design 3 (CSCT-2+C). The complexity of CSCT-2+C (three
   independent sub-systems) makes simulation interpretation difficult and the V1
   accounting concern is unresolved. Hybrid B is a cleaner test of whether
   encoding genuine cross-archetype affinity in two floats (vs. a single tag)
   improves V3 without sacrificing V1. It also directly answers the V9 minimum
   information question: is 8 bits/card (two-float pair affinity) the right
   stopping point between 3 bits/card (tag) and 16-32 bits/card (full affinity)?

______________________________________________________________________

## 10. Key Open Questions for Simulation

1. **Flash/Ramp M3 floor under Design 4's 4x weighting.** The R1 pool for Zephyr
   has 80 cards split between Flash and Ramp. At 4x home-tag weighting,
   effective precision ≈ 80% per slot. With 3 slots at 80% precision and 1
   random: M3 = 3*0.80 + 1*0.125 = 2.525. But Graduated Realistic fitness for
   Flash/Ramp (F=25%) may reduce this. The per-archetype Flash/Ramp numbers must
   be confirmed.

2. **Design 6 anchor mechanic vs. uniform contraction.** Does the 18%
   contraction rate on dual-resonance picks meaningfully improve M10 and V4, or
   does the 10%/18% split create irregular pool dynamics that hurt M9?

3. **M10 under all proposals.** Every proposal predicts M10 ≈ 2-3, which is
   either at or marginally above the \<= 2 target. The floor slot is the primary
   mitigation. Simulation will determine whether M10 \<= 2 is achievable without
   sacrificing M6, or whether the M3-M10-M6 triangle from V8 persists.

4. **AWNG V1 measurement.** Run Design 5 with hidden affinities stripped. If
   M3_visible < 2.0, V1 is below threshold and AWNG is eliminated regardless of
   its M3 and V3 scores.

5. **Power-chaser M3 gap.** All proposals claim the visible resonance strategy
   produces 0.4-0.8 M3 advantage over power-chasers. This is the V4 empirical
   test. Simulation must confirm the gap is >= 0.4 for each proposal.
