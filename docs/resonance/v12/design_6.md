# V12 Design 6: Hybrid Approaches + Novel Mechanisms

## Key Findings

- **Biased refills are the essential pairing with avoidance.** Balanced refills
  restore all archetypes equally at each boundary, canceling the density
  gradient avoidance builds. Open-lane-biased refills compound avoidance: AIs
  drain non-player archetypes while refills preferentially restore the player's
  lane. This is the single most important combinatorial insight from the
  research.

- **S/A exhaustion is the late-draft binding constraint, not pool size alone.**
  The concentration math shows that by pick 25-30, the player's own picks have
  consumed most of their archetype's S/A (starting ~5, ending ~1-3 without
  replenishment). Any design that fails to maintain S/A supply hits M3 = 0 in
  the final picks regardless of pool density.

- **Progressive N (4 → 12) solves the exploration/exploitation tension
  naturally.** Early packs at N = 4 show variety from the large starting pool;
  late packs at N = 12 show concentration from the contracted pool. The ramp
  aligns oversampling intensity with both pool contraction rate and player
  commitment timing, producing M3 gains without reducing M1 (early diversity).

- **Transparent oversampling is achievable.** The "best 4 of N" ranking can use
  only visible resonance symbols (no hidden pair-affinity), making the curation
  mechanism fully observable and honest. Research Agent A confirmed the
  face-up pool sets a natural honesty standard: AIs use the same public
  information the player sees.

- **The avoidance cascade is a design amplifier, not a separate mechanism.**
  When all AIs avoid the player's archetype and concentrate on 7 others, they
  push into adjacent archetypes sharing resonance symbols with the player's
  lane. This secondary depletion slightly erodes off-archetype options but
  compounds concentration. The cascade is real but mild and requires only
  awareness, not mitigation.

- **Variable AI count by round amplifies early exploration and late
  concentration in the same design.** 3 AIs in Round 1 (slow contraction,
  diverse pool) + 5 AIs in Rounds 2-3 (fast contraction, concentrated pool)
  produces a natural escalation arc. Fewer removal events early preserve pool
  diversity (M1); more removal events late accelerate density (M3).

- **Split oversampling (2 archetype-best + 2 power-best) preserves a residual
  exploration tension that pure archetype-biased oversampling eliminates.**
  Late-game packs still present off-archetype power cards, supporting pivot
  opportunities and maintaining M4 (off-archetype filler) without sacrificing
  M3.

---

## Three Algorithm Proposals

### Proposal A: Progressive-N with Biased Refills

**Name:** Progressive Commitment

**Description:** N ramps from 4 to 12 as the player commits, combined with
open-lane-biased refills and gradual AI avoidance; the entire mechanism aligns
with the natural arc of a draft.

**Technical Spec:**

- Starting pool: 120 cards (15 per archetype)
- Round structure: 3 rounds of 10 picks each
- Refill schedule: 50 open-lane-biased after R1, 30 open-lane-biased after R2,
  0 after R3. Total supply = 200 cards; 180 consumed = ~20-card final pool.
  Open-lane bias factor: 2.0x (each of the 3 open lanes gets 2x cards vs each
  of the 5 AI lanes per refill unit). Pool trajectory: ~120 → ~110 → ~60 → ~20.
- AI count: 5 throughout. Avoidance ramps from pick 3 (20% weight reduction)
  to pick 12 (80% weight reduction). Infers from aggregate resonance-symbol
  depletion rates over prior 3 pick cycles.
- Oversample N: pick 1-5 N=4, pick 6-10 N=6, pick 11-15 N=8, pick 16-20
  N=10, pick 21-30 N=12. Show best 4 by visible resonance symbol match to
  player's drafted-card signature.
- "Best 4" ranking: cosine similarity of card's visible symbols to player's
  accumulated resonance signature. No pair-affinity used. Fully transparent.
- AI pick logic: each AI drafts highest-fitness card in their archetype from
  the pool, reducing weight on the player's inferred archetype proportional to
  avoidance strength. Saturation threshold: ease off after 12 on-arch cards.
- Player information: face-up pool browseable at all times.

**Predicted metrics:**

| Metric | Prediction |
|--------|-----------|
| M3 | 1.8-2.2 |
| M10 | 2-3 |
| M11' | 2.2-2.7 |
| M6 | 70-80% |
| M12 | 0.3-0.4 |
| M13 | pick 8-10 |
| M14 | pick 5-7 |

---

### Proposal B: Split Oversampling with Avoidance

**Name:** Dual-Track Pack Construction

**Description:** Draw N = 10 cards from the contracted pool, show 2 best for
the player's archetype plus 2 highest power regardless of archetype, at every
pick from 6 onward.

**Technical Spec:**

- Starting pool: 120 cards
- Round structure: 3 rounds of 10 picks
- Refill schedule: 50 open-lane-biased after R1, 25 open-lane-biased after R2,
  0 after R3. Total supply = 195; 180 consumed = ~15 final pool.
- AI count: 5 throughout. Gradual avoidance from pick 4, ramping to 75%
  reduction by pick 14.
- Oversample N = 10 from pick 6 onward; N = 4 uniform for picks 1-5 (exploration).
- "Best 4" construction: draw 10, select the 2 cards with highest resonance
  signature match (archetype slots), then select 2 cards with highest raw power
  score among remaining 8 (power slots). Show all 4.
- Power-slot cards create exploration tension in late draft: the player must
  decide between a synergy card and a high-power off-arch card at every pick.
- Player information: face-up pool. Pack labels indicate which 2 slots are
  archetype-recommended vs power-recommended.

**Predicted metrics:**

| Metric | Prediction |
|--------|-----------|
| M3 | 1.5-2.0 (power slots reduce pure S/A yield) |
| M10 | 2-4 |
| M11' | 1.8-2.3 |
| M6 | 65-75% (power cards diversify deck) |
| M12 | 0.35-0.5 (power tension rewards signal readers who commit early) |
| M13 | pick 9-12 |
| M14 | pick 5-8 |

---

### Proposal C: Variable AI Count by Round

**Name:** Escalating Table

**Description:** 3 AIs in Round 1 (slow contraction), 5 AIs in Round 2, 7 AIs
in Round 3, with gradual avoidance and moderate oversampling (N = 10).

**Technical Spec:**

- Starting pool: 120 cards
- Round structure: 3 rounds of 10 picks
- AI count: Round 1 = 3 AIs (4 total removals/pick cycle), Round 2 = 5 AIs
  (6 total), Round 3 = 7 AIs (8 total)
- Refill schedule: 55 balanced after R1 (no bias yet — table is still small,
  open lanes unclear), 35 open-lane-biased after R2, 0 after R3.
  Pool trajectory: ~120 → ~75 → ~60 → ~18. Total supply = 210; 180 consumed
  = ~20-30 final pool.
- AI avoidance: begins pick 5 (3 pick cycles of data), ramps to 80% by pick 15.
  Round 3 AIs (the 2 added in pick 21+) begin with full avoidance immediately
  (the pool state at Round 3 start is sufficient for confident inference).
- Oversample N = 4 picks 1-5, N = 8 picks 6-15, N = 10 picks 16-30.
- "Best 4" ranking: resonance symbol match. Fully transparent.
- Player information: face-up pool. Player can observe that the table size
  changes between rounds — the "number of AI opponents" is visible.

**Predicted metrics:**

| Metric | Prediction |
|--------|-----------|
| M3 | 1.9-2.3 |
| M10 | 2-3 |
| M11' | 2.3-2.8 |
| M6 | 72-82% |
| M12 | 0.3-0.45 |
| M13 | pick 7-10 |
| M14 | pick 5-7 |

---

## Champion Selection

**Champion: Proposal A (Progressive Commitment)**

**Justification:** The three proposals attack the same problem from different
angles. Proposal B's split oversampling reduces pure M3 yield — the power slots
consume 2 of 4 pack slots, cutting archetype-density contribution by half. This
trades M3 for design texture, which is a reasonable tradeoff only if M3 baseline
is already robust. Given the math research's finding that even N=12 from a
20-card pool barely reaches M3 = 2.0 with 4-5 S/A, using only 2 archetype slots
rather than 4 risks landing at M3 = 1.0-1.5. Proposal B is the right design for
a game where M3 = 2.0 is already achieved; it is not the right design when M3 is
the primary constraint to solve.

Proposal C's variable AI count is appealing for narrative reasons and is a
genuine amplifier of late concentration. However, it introduces UI complexity
(explaining why the table changes size) and the Round 1 balanced refill
(necessary because the table is small and archetype signals are weak) undoes
some of the early avoidance benefit. The 3-AI Round 1 also means slower
contraction precisely when the player needs variety — this is good for M1 but
requires the R2-R3 contraction to be steep enough to compensate, which requires
aggressive late-stage pool shrinkage that risks M10 degradation (too-rapid swing
from few S/A to many).

Proposal A aligns all three mechanisms — N ramp, biased refills, avoidance
ramp — into a single coherent trajectory. Early picks see a large pool with
low N: natural variety. Late picks see a contracted pool with high N: natural
concentration. The open-lane-biased refills compound avoidance rather than
canceling it. The resonance-symbol-only ranking is fully transparent. All
components reinforce the same arc.

The champion can absorb Proposal C's variable-AI idea as a variant if simulation
shows early-round pool depletion is too aggressive (simply reduce AI count in
Round 1 from 5 to 4 or 3 without restructuring the design).

---

## Champion Deep-Dive: Progressive Commitment

### Architecture Summary

| Parameter | Value |
|-----------|-------|
| Starting pool | 120 cards (15 per archetype) |
| Round structure | 3 rounds × 10 picks |
| Refill schedule | 50 open-lane-biased (R1→R2), 30 open-lane-biased (R2→R3), 0 (R3) |
| Refill bias | 2.0x open-lane multiplier (3 open lanes, 5 AI lanes) |
| AI count | 5 throughout |
| AI avoidance onset | Pick 3 (20% weight reduction), ramps to 80% by pick 12 |
| AI inference | Rolling 3-pick-cycle depletion rate per archetype vs. expected |
| Oversample N | Pick 1-5: N=4, Pick 6-10: N=6, Pick 11-15: N=8, Pick 16-20: N=10, Pick 21-30: N=12 |
| "Best 4" criterion | Cosine similarity to player's resonance signature (visible symbols only) |
| AI pick logic | Best on-arch card, with avoidance weight applied to player's inferred arch |
| AI saturation | Ease off after 12 on-arch cards drafted |
| Player information | Face-up pool, browseable by archetype/symbol |

### Pick-by-Pick Walkthrough

**Picks 1-5: Exploration Phase (N=4, Pool ~120, No Avoidance)**

The player sees 4 random cards from a 120-card pool. All 8 archetypes are
represented; pack contents are diverse. AIs have not yet inferred the player's
archetype — the pool shows 3 pick cycles of roughly uniform depletion.

The player picks 2 Storm/Ember cards at picks 3 and 4. After pick 5, the pool
has lost ~5 player picks + ~25 AI picks = 30 cards total. Ember cards are
depleting slightly faster than expected (4 gone vs ~3.75 expected from random
draw), but this is within noise at 3 pick cycles.

AIs at this point have 0-20% avoidance weight on any archetype. Their behavior
is effectively Level 0.

**Picks 6-10: Early Commitment Phase (N=6, Avoidance Ramps, Pool ~90→~60→refill→~110)**

The player has drafted 3 Storm S/A cards. Their resonance signature is
Ember-heavy with some Stone. Pack construction switches to N=6, showing best 4
by resonance match. The player sees slightly more Ember/Storm options than
picks 1-5 — not dramatically different, but the curation provides mild
guidance.

After 5 more pick cycles, Storm/Ember cards have depleted at ~1.8x expected
rate (2 sources: the player + incidental AI picks). AIs observe this and begin
applying 30-50% weight reduction to Storm and Blink (the two primary Ember
archetypes). They shift toward Tide-heavy and Stone-heavy archetypes.

End of Round 1 (pick 10): Pool has ~60 cards remaining. Before the refill,
the player's archetype (Storm) has ~8-9 cards remaining. Open-lane-biased
refill adds 50 cards: open lanes get ~21 cards total across 3 lanes (~7 per
lane), AI lanes get ~29 cards total across 5 lanes (~5.8 per lane). The
player's Storm lane receives ~7 new cards → pool rises to ~110 with Storm
at ~15-16 cards.

**Picks 11-15: Committed Phase (N=8, Full Avoidance Building, Pool ~110→~60)**

The player is committed to Storm. Their resonance signature is clearly
Ember-dominant. AIs' 3-cycle depletion analysis shows Storm at 2.2x expected
rate — confident inference. Avoidance weight on Storm reaches 65-70%. Five
AIs collectively take fewer than 1 Storm card per 5-pick window.

N=8 now draws 8 cards from a pool of ~90 cards. Storm is ~14% of the pool
(~13 cards), so a random draw of 8 hits ~1.1 Storm cards. But the best 4 of
these 8 (by resonance match) show ~1.5-2.0 Storm S/A cards per pack. This is
the first meaningful M3 gain: the combination of slight pool Storm-enrichment
from avoidance + N=8 curation raises the per-pack experience from 0.3-0.5
(V11-level) to 1.5-2.0.

**Picks 16-20: Concentration Phase (N=10, Strong Avoidance, Pool ~60→refill→~90→~50)**

End of Round 2 (pick 20): Pool has ~50 cards. Second biased refill adds 30
cards: open lanes get ~13 cards (3 lanes × ~4.3 each), AI lanes get ~17 cards.
Storm receives ~4-5 cards from the refill → Storm is now ~10-12 cards in a
pool of ~80.

N=10 from a pool of ~75 cards. Storm is ~13% density (~10 cards). Drawing 10:
expected Storm draws = ~1.33. Best 4 of 10 with resonance-match ranking:
~2.0-2.5 Storm cards shown. S/A rate among Storm cards drawn = 36% sibling
rate. Expected S/A per pack: ~0.72-0.9. This still underperforms M3 target.

**Picks 21-30: Endgame (N=12, Pool ~80→20)**

N=12 from a pool that contracts from ~80 to ~20 over 10 picks. The critical
late-draft scenario: by pick 26, pool is ~32 cards. Storm is ~15% density with
avoidance fully active (~5 Storm cards), S/A count ~3-4.

Drawing N=12 from 32 cards: E[Storm draws] = 12 × (5/32) = 1.875. E[S/A
in Storm draws] = 1.875 × 0.36 ≈ 0.67. Best 4 of 12 selects all drawn Storm
cards → M3 at pick 26 ≈ 1.5-2.0 if 3-4 S/A remain.

By pick 28-30, pool ~22 cards, Storm ~5 cards, S/A ~2-3: N=12 draws 12 from
22. E[Storm draws] = 12 × (5/22) = 2.73. E[S/A] = 2.73 × 0.36 ≈ 0.98.
Best 4 of 12 shows ~0.98-2.5 S/A depending on S/A count.

**Pool Composition Evolution**

| Pick | Pool Size | Storm Cards | Storm % | S/A in Storm | N | E[S/A per pack] |
|:----:|:---------:|:-----------:|:-------:|:------------:|:-:|:---------------:|
| 1 | 120 | 15 | 12.5% | 5 | 4 | 0.17 |
| 6 | 90 | 12 | 13.3% | 4 | 6 | 0.27 |
| 11 (post-refill) | 110 | 15 | 13.6% | 5 | 8 | 0.36 |
| 16 | 75 | 13 | 17.3% | 4 | 10 | 0.53 |
| 21 (post-refill) | 85 | 14 | 16.5% | 4.5 | 12 | 0.64 |
| 26 | 38 | 7 | 18.4% | 3 | 12 | 0.95 |
| 30 | 20 | 5 | 25.0% | 2 | 12 | 1.20 |

These raw E[S/A] values appear below M3 = 2.0. The discrepancy versus
prediction arises because the table above uses conservative estimates. The
research concentration math notes that when N approaches P (12/20 = 60%
of pool drawn), the formula `min(N × S/P, 4)` overstates because of sampling
without replacement — but in this range, the hypergeometric actually produces
*higher* expected hits than sampling with replacement would suggest. More
critically, the "best 4 of N" step selects cards with the highest resonance
match, not uniformly — if Storm S/A cards have higher resonance similarity
scores than Storm C/F cards (they do, since S/A cards typically have the
cleaner symbol alignment), the selection bias toward S/A within the Storm
cards drawn is significant.

**Failure Modes**

1. **S/A exhaustion by pick 25-30.** If the player picks all their S/A early
   (picks 1-15), the late pool has only C/F Storm cards. Mitigation: biased
   refills add S/A proportionally to all refilled cards. Players who take off-
   archetype cards early (signal readers) preserve more late-draft S/A.

2. **Avoidance inference failure.** With 5 AIs producing confounding depletion
   signals, AIs may infer the wrong archetype through pick 8-10. This produces
   a 3-5 pick window where AIs may take some of the player's S/A. Mitigation:
   the gradual ramp keeps avoidance weight low (20-30%) during the uncertain
   inference window; false-negatives cost less than they would under full
   avoidance.

3. **Biased refill revealing AI assignments.** If the player notices that
   their archetype's cards are always well-stocked after refills, they may
   deduce that their archetype is an "open lane." This is technically public
   information (the bias is level 0) but may feel like the system is playing
   favorites. Mitigation: the bias is 2x, not 5x — the difference is
   noticeable only over multiple rounds.

4. **Pool exhaustion edge case.** Total supply = 120 + 50 + 30 = 200 cards.
   With 6 drafters × 30 picks = 180 removals, the final pool should have ~20
   cards. If AI saturation kicks in early (fewer than 30 AI picks), the pool
   may not contract enough for N=12 to work. Mitigation: AI saturation
   threshold of 12 cards means each AI drafts ~12 on-arch + ~3 incidental =
   ~75 total AI picks, close to the designed 150. Track minimum pool size; if
   pool > 25 at pick 25, raise N to 14 (within the extended range).

---

## Complete Specification

### Starting Pool

- 120 cards total: 15 per archetype × 8 archetypes
- ~11% generic, ~79% single-symbol, ~10% dual-symbol (per fixed assumptions)
- Each archetype: ~5 S/A cards (36% sibling A-tier), ~10 C/F cards

### Refill Schedule

| After Round | Volume | Bias | Open-Lane Allocation |
|-------------|:------:|------|---------------------|
| Round 1 → Round 2 | 50 cards | 2.0x open-lane | 3 open lanes × ~9.1 each; 5 AI lanes × ~4.5 each |
| Round 2 → Round 3 | 30 cards | 2.0x open-lane | 3 open lanes × ~5.5 each; 5 AI lanes × ~2.7 each |
| After Round 3 | 0 | — | — |

Total supply: 200 cards. Total removals: 180 (6 drafters × 30 picks).
Expected final pool: ~20 cards.

### Oversample N Schedule

| Picks | N |
|:-----:|:-:|
| 1-5 | 4 (uniform random) |
| 6-10 | 6 |
| 11-15 | 8 |
| 16-20 | 10 |
| 21-30 | 12 |

### "Best 4" Ranking Criterion

Rank all N drawn cards by cosine similarity to the player's accumulated
resonance signature vector (4 elements: Tide, Stone, Ember, Zephyr). Signature
updates after each pick: +2 to primary symbol, +1 to secondary symbol of the
picked card. For picks 1-5 (N=4 uniform), no ranking is applied — all 4 shown.

No pair-affinity metadata used. Mechanism is fully transparent.

### AI Count

5 AIs per draft. Archetype assignment: random 5 of 8, no duplicates. Open lanes:
3 per game. C(8,5) = 56 compositions for run-to-run variety.

### AI Avoidance Model

Graduated avoidance. Avoidance weight on inferred player archetype:

| Pick Range | Avoidance Weight (reduction) |
|:----------:|:----------------------------:|
| 1-2 | 0% |
| 3-5 | 20% |
| 6-8 | 40% |
| 9-11 | 60% |
| 12+ | 80% |

Avoidance is archetype-specific: an AI avoiding Storm still freely drafts non-
Storm Ember cards.

### AI Inference Mechanism

After each pick cycle, compute observed vs. expected depletion rate per
archetype over the prior 3 cycles:

```
observed_rate[arch] = cards_removed_from_arch / arch_cards_at_window_start
expected_rate[arch] = total_cards_removed / pool_size_at_window_start
depletion_signal[arch] = observed_rate[arch] / expected_rate[arch]
```

The archetype with the highest sustained `depletion_signal` (above 1.5x
expected for 2+ consecutive windows) is the inferred player archetype.
Inference resets if signal drops below 1.2x for 2 windows (pivot detection).

Information used: pool state snapshots only (publicly observable). No per-pick
attribution to specific drafters.

### AI Pick Logic

For each AI pick:
1. Find all pool cards with fitness >= 0.7 for the AI's assigned archetype.
2. Apply avoidance weight: if any candidate card also scores >= 0.5 fitness for
   the inferred player archetype, reduce that card's selection probability by
   the current avoidance weight.
3. Pick highest-weighted candidate. If archetype is saturated (12+ on-arch
   cards drafted), pick from any remaining pool card by power score.

### Player Information

- Face-up pool: all pool cards visible at all times, browseable by archetype
  and resonance symbol.
- Pack construction: pack label indicates current N value and that cards were
  selected by resonance match. No archetype label on individual cards beyond
  visible symbols.
- AI drafters: visible as named opponents. Cards disappearing from pool are
  attributable to AI or player picks (who took what is secret). AI archetype
  assignments are not displayed.
- Round boundaries: displayed with refill narrative. Pool card count before
  and after refill is visible.

---

## Post-Critique Revision

The critic's walkthrough table confirms the core problem: the champion as
specified produces M3 ≈ 1.2 at pick 30, not 2.0. The table shows 2 S/A in a
20-card pool at pick 30 with N=12, yielding E[S/A] = 12 × (2/20) = 1.2 — well
below target. The critic is correct that this discrepancy invalidates the
1.8-2.2 prediction. The original design relied on selection-bias arguments
(S/A cards having higher resonance similarity scores than C/F cards) to bridge
the gap from 1.2 to 2.0. This is insufficiently rigorous; the walkthrough math
is the honest constraint.

### Revised Parameters

**Open-lane bias: 2.0x → 2.5x**

The 2.0x bias adds ~9.1 cards per open lane per refill. At 2.5x, the R1 refill
of 50 cards allocates ~10.7 per open lane vs ~4.3 per AI lane. Over two biased
refills, the player's Storm lane receives ~16 additional cards vs ~7 under 2.0x.
This is the primary lever for raising late-draft S/A: more S/A arrive in
refills, partially offsetting player consumption.

**Floor slot: added**

Guarantee 1 S/A in the shown 4 whenever any S/A cards appear in the N drawn.
If the N draw contains at least 1 S/A, the lowest-ranked non-S/A card is
replaced by the highest-resonance-match S/A card. This fires approximately
60-75% of packs in late draft and is the most direct mechanism to raise M3
when S/A count is low. When S/A = 2 in a 20-card pool at pick 30, without a
floor slot E[S/A shown] = 12 × (2/20) × (best-4 filter) ≈ 1.2; with the floor
slot, any draw containing ≥1 S/A guarantees 1 shown, raising effective M3 to
≈ 1.6-1.8 under realistic S/A conditions.

**Simplified N ramp: 4→8→12 (picks 1-5, 6-15, 16-30)**

The five-step ramp (4→6→8→10→12) adds tuning complexity without meaningful
differentiation. The N=6 step during picks 6-10 provides marginally more
curation than N=4 but the pool is still ~90 cards — the oversampling gain is
small. The N=10 step during picks 16-20 delays N=12 onset into the critical
contraction window (picks 16-25) when pool shrinks fastest and oversampling
matters most. Collapsing to 4→8→12 moves full N=12 to pick 16, gaining 5
additional picks at maximum oversampling during peak contraction.

### Revised N Schedule

| Picks | N |
|:-----:|:-:|
| 1-5   | 4 (uniform random) |
| 6-15  | 8 |
| 16-30 | 12 |

### S/A Tracking Requirement

Simulation must track S/A count at picks 20, 25, and 30 as explicit checkpoints.
If S/A < 3 at pick 25, the design cannot achieve M3 = 2.0 regardless of N or
pool size — this is a hard mathematical constraint (12 × 3/20 = 1.8, below
target even with floor slot). The 2.5x bias and floor slot are designed to
keep S/A ≥ 3 at pick 25 under realistic player behavior; simulation will
determine whether they succeed.

### Revised M3 Prediction

Under these revisions: 1.6-2.0. The floor slot converts the pick-30 scenario
(2 S/A, 20-card pool, N=12) from E[S/A] = 1.2 to a guaranteed minimum of 1
shown S/A, with probability ~1.0 when any S/A remain. The 2.5x bias raises
expected S/A at pick 25 from ~2 to ~3, improving the pick-25 E[S/A] from 1.2
to ~1.8 without floor slot, or ~2.0 with floor slot. M3 = 2.0 remains
achievable but now requires the bias and floor slot to both perform as designed.
The original 1.8-2.2 range was optimistic; 1.6-2.0 is the honest revised range.
