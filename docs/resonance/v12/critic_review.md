# V12 Critic Review: Round 3

## Overview

Six designs have been evaluated against the core thesis: that AI avoidance +
physical pool contraction + modest oversampling (N = 8-12) can achieve M3 >=
2.0 through transparent mechanisms. This review ranks the designs, identifies
the binding constraints each fails to address, evaluates the honesty of
proposed AI avoidance mechanisms, and recommends which combinations to simulate.

The honest summary: **no single design, as specified, definitively achieves
M3 >= 2.0 in a way the math fully supports.** Several come close under optimistic
assumptions. The designs that approach the target share two properties: biased
refills and oversampling at N = 12. Designs relying on N = 8 from a ~20-card
pool are structurally on the knife's edge, and S/A exhaustion by late draft is
the binding constraint that multiple designers identified but none fully solved.

---

## Section 1: Ranking All Six Designs

Rankings across the seven criteria requested, with commentary:

### 1. M3 / M11' Potential

| Rank | Design | M3 Prediction | Assessment |
|:----:|--------|:-------------:|-----------|
| 1 | Design 3 (Moderate + N=12 + Floor Slot) | 2.0-2.2 | Most credible path to M3 >= 2.0 |
| 2 | Design 6 (Progressive Commitment) | 1.8-2.2 | Plausible if S/A holds; optimistic |
| 3 | Design 2 (Steep + N=8) | 1.5-1.8 | Honest self-assessment; biased refills needed |
| 4 | Design 4 (V9 Fallback) | 2.70 (proven) | Separate category — different engine |
| 5 | Design 5 (7 AIs) | 1.1-1.4 | Self-reported failure; best late-draft surge |
| 6 | Design 1 (Isolation, N=4) | 0.4-0.7 | Intentionally calibration-only |

Design 3 is the strongest face-up candidate, with the floor slot providing the
key mechanism that converts a near-miss into a likely pass. Design 6's
progressive N is elegant but the S/A trajectories at pick 26-30 show only ~1.2
expected S/A per pack, not 2.0+, in the walkthrough table — the design's
optimistic prediction relies on selection bias from resonance-symbol ranking that
may not materialize uniformly.

Design 4 exists in a separate category: it achieves M3 = 2.70 but through V9's
invisible engine, not through face-up physical mechanisms.

### 2. Player Experience

| Rank | Design | Notes |
|:----:|--------|-------|
| 1 | Design 6 | Progressive N, transparent resonance ranking, no hidden metadata; most honest |
| 2 | Design 3 | Strong late-draft quality; floor slot narrows pack variety slightly |
| 3 | Design 2 | Clear mechanics; biased refills feel natural as "market restocking" |
| 4 | Design 5 | 1 open lane eliminates lane-selection tension; weaker overall experience |
| 5 | Design 1 | N=4 produces poor early-to-mid experience by design |
| 6 | Design 4 | No face-up pool forfeits browsability and honest AI behavior |

### 3. Simplicity

| Rank | Design | Notes |
|:----:|--------|-------|
| 1 | Design 1 | Cleanest; pure isolation test |
| 2 | Design 2 | Three parameters (60/20/0, 1.7x bias, N=8); clear specification |
| 3 | Design 4 | V9 engine unchanged; simplest path to proven metrics |
| 4 | Design 3 | Floor slot + pair-affinity ranking adds implementation layers |
| 5 | Design 6 | Progressive N schedule adds tuning; 2.0x bias factor distinct from D3's 1.7x |
| 6 | Design 5 | 7 AIs + saturation management + inference delay creates most complexity |

### 4. Signal Reading Quality (M12)

| Rank | Design | M12 Prediction | Notes |
|:----:|--------|:--------------:|-------|
| 1 | Design 6 | 0.3-0.4 | Progressive N rewards early commitment |
| 2 | Design 3 | 0.30-0.40 | Delayed avoidance penalizes slow readers slightly |
| 3 | Design 2 | 0.3-0.4 | Gradual avoidance creates readable mid-draft gradient |
| 4 | Design 4 | 0.20-0.35 | Partial signal reading via avoidance log |
| 5 | Design 1 | 0.15-0.30 | Signal exists but N=4 limits translation to quality |
| 6 | Design 5 | 0.05-0.15 | 1 open lane structurally eliminates M12 |

### 5. AI Avoidance Narrative Quality

| Rank | Design | Notes |
|:----:|--------|-------|
| 1 | Design 6 | Public-information only; resonance depletion visible to all |
| 2 | Design 2 | Gradual ramp from pick 3 feels realistic; rolling window transparent |
| 3 | Design 3 | Delayed avoidance (pick 8) is later than ideal but statistically defensible |
| 4 | Design 5 | Inference delay is realistic for 7-AI table; single lane reduces narrative richness |
| 5 | Design 1 | Avoidance exists but N=4 makes it invisible to player experience |
| 6 | Design 4 | Avoidance is post-hoc narrative over V9 contraction; partially dishonest |

### 6. Contraction Trajectory Quality

| Rank | Design | Final Pool Estimate | Notes |
|:----:|--------|:-------------------:|-------|
| 1 | Design 2 | ~20 cards | Hits the N=8 threshold if S/A holds |
| 2 | Design 3 | ~20-25 cards | Hits N=12 threshold with 50/30/0 |
| 3 | Design 6 | ~20 cards | Progressive N masks gradual contraction |
| 4 | Design 5 | ~10-18 cards | Aggressive but inference delay delays benefit |
| 5 | Design 1 | ~15-25 cards | Good contraction; wasted without oversampling |
| 6 | Design 4 | 17 cards (360 → 17) | Best contraction but invisible |

### 7. Pool Exhaustion Risk

| Rank | Design | Risk Level | Notes |
|:----:|--------|:----------:|-------|
| 1 | Design 3 | Low | 50/30/0 = 200 supply vs 180 removals; 20-card cushion |
| 2 | Design 6 | Low | 200 supply vs 180 removals; AI saturation at 12 cards |
| 3 | Design 2 | Moderate | 60/20/0 = 200 supply; tight in R3 with thin AI lanes |
| 4 | Design 1 | Moderate | 60/0/0 = 180 supply; Pool reaches near-zero in final picks |
| 5 | Design 4 | None | V9's pool minimum (17 cards) managed automatically |
| 6 | Design 5 | High | 220 supply vs 240 removals; relies entirely on saturation |

---

## Section 2: Is AI Avoidance Genuinely Public Information?

This is the most important qualitative question in V12. The designs handle it
with varying degrees of rigor.

**The clear line:** AIs may observe pool state changes (card counts per
archetype over time) and infer player archetype from aggregate depletion
patterns. This is symmetric — the player does the same thing. AIs may NOT
directly access which specific cards the player took, the player's internal
commitment level, or card-by-card attribution.

**Designs that stay clearly within the line:** Designs 1, 2, 5, and 6 all use
depletion rate ratios over rolling windows (3-4 pick cycles). The inference is
noisy, proportional to evidence, and uses only pool-state snapshots. This is
public information properly used.

**Design 3's borderline element:** Design 3 uses pair-affinity scores (hidden
8-bit metadata) for the "best 4" ranking step. This is not AI inference — it is
the system curating the pack. The AI avoidance itself uses visible symbol
depletion, which is public. The pair-affinity ranking in pack construction is a
different information layer. However, using hidden metadata for pack curation is
arguably a Level 2 mechanism — the system is reasoning about cards using
information the player cannot verify. This should be flagged for simulation:
test with and without pair-affinity to isolate its M3 contribution.

**Design 4's core honesty problem:** Design 4 is partially dishonest by the
design's own admission. V9's engine removes cards by relevance scoring; the AI
"avoidance" is retroactively attributed to this removal. An AI does not decide
to avoid the player's archetype — V9's algorithm removes their archetype's
low-relevance cards and the avoidance narrative is constructed post-hoc. This
is the clearest case where "avoidance" is a dressed-up Level 2 mechanism: the
system is using pair-affinity metadata to remove cards and calling it AI
behavior. For V12's honesty criterion, Design 4 is the fallback specifically
because it cannot satisfy that criterion.

**Design 5's inverse signal insight:** Design 5 correctly identifies that with
a single open lane, the player's archetype depletes *slower* than expected (only
1 drafter takes it), not faster. The inference should look for below-expected
depletion, not above-expected. This is a real insight that Designs 2, 3, and 6
should incorporate as an explicit check alongside their above-expected depletion
detection.

---

## Section 3: Pool Contraction Trajectories

The research math is unambiguous: to reach M3 >= 2.0, the late-draft pool must
contract to ~20 cards (for N=8 with 5 S/A) or ~30 cards (for N=12 with 5 S/A).

**The S/A exhaustion problem is under-solved.** Every design acknowledges it;
none fully resolves it. The math from Research Agent C shows that the player
consuming ~0.33 S/A per on-arch pick over 30 picks depletes ~10 S/A total. With
a starting count of 5 and refills adding ~4-6 S/A (depending on schedule and
bias), the late-draft S/A count is 1-3, not 5. This makes M3 = 2.0 at N=8
(which requires S/A = 5 in a 20-card pool) mathematically unreachable under
realistic player behavior.

**Designs 2 and 3 partially address this** via biased refills that target higher
S/A rates in open lanes. Design 3 explicitly notes that the R1 biased refill can
restore S/A above the starting level (player arch gets 8-10 cards from the 50-
card refill, including ~3-4 S/A). This is the most concrete S/A recovery
mechanism, but it relies on the 1.7x bias being sufficient. A higher bias (2x)
or an S/A-targeted refill mechanism (refills hit open-lane S/A at 2x the normal
S/A rate) would be more robust.

**The 60/20/0 vs 50/30/0 question:** Design 2's 60/20/0 reaches a ~20-card final
pool. Design 3's 50/30/0 reaches ~20-25 cards. The difference is whether N=8 or
N=12 is the oversampling tool. Given the S/A constraint, N=12 is more forgiving:
M3 = 2.0 at N=12 requires only S/A = 3.3 in a 20-card pool, versus S/A = 5 for
N=8. Design 3's approach is more robust to realistic S/A depletion.

**Pool exhaustion assessment:** With 180 total removals over 30 picks and 200
cards supplied (120 + 80 refills), all designs with ~80 total refills have a
cushion. The risk is not global exhaustion but local exhaustion of AI archetypes
mid-draft, which triggers saturation and changes the removal dynamics. AI
saturation thresholds at 10-12 cards are appropriate; designs that specify this
explicitly (Designs 2, 3, 6) are better specified than those that do not.

---

## Section 4: Oversampling Assessment

**Can N=4 suffice?** No. Research Agent C demonstrates this definitively: with
a 20-card pool and 5 S/A, N=4 yields M3 = 1.0. The best achievable contraction
(~15-card pool, ~5 S/A) gives M3 = 1.33. M3 = 2.0 with N=4 requires a ~10-card
pool, which is incompatible with a 30-pick draft. Design 1's isolation test will
confirm this finding empirically.

**N=8 is a knife's edge.** M3 = 2.0 at N=8 requires exactly S/A = 5 in a 20-
card pool. The S/A exhaustion analysis shows this is optimistic. Under realistic
player behavior, S/A at pick 25 is 2-3, giving M3 = 8×3/20 = 1.2. Design 2's
honest revised prediction (1.4-1.8) reflects this. N=8 is viable only if biased
refills aggressively maintain S/A at 4-5 through pick 25.

**N=12 is the robust choice.** M3 = 2.0 at N=12 requires S/A = 3.3 in a 20-
card pool, or S/A = 5 in a 30-card pool. Both are achievable under realistic S/A
depletion with biased refills. Design 3's champion reaches the target at N=12
with a floor slot. Design 6's progressive N (reaching 12 by pick 21) provides
a natural arc.

**Does oversampling feel natural?** At N=12 from a 20-30 card pool, the system
draws 40-60% of the pool to show 4 cards. This is significant curation. The
face-up pool mitigates the transparency concern — the player can browse the
full pool and see the cards the system drew from. However, if the player browses
and sees 6-8 S/A cards remaining and the system shows them 2-3, the curation is
visible. This is probably fine; the player experiences it as "the system found
the best options," which is accurate. At N=8 from a 20-card pool, drawing 40%
of the pool for a pack of 4 may feel more like a lottery than curation.

---

## Section 5: The Three-Mechanism Interaction

The mechanisms interact in a specific order:

1. **AI avoidance preserves S/A** (demand side): Without avoidance, AIs consume
   the player's S/A at ~0.6 cards per pick cycle. With full avoidance, only the
   player consumes their own S/A. The swing is ~0.5 S/A preserved per pick cycle
   after avoidance locks in (pick 6-10).

2. **Declining refills contract the pool** (supply side): Without declining
   refills, avoidance alone produces 12-30% archetype density in a still-large
   pool — insufficient. The contraction is what creates the density.

3. **Oversampling converts density to quality** (sampling side): With a
   contracted pool of 20-30 cards and 3-5 S/A, N=8-12 converts archetype
   density into pack quality. Without oversampling, even a contracted pool at
   25% density gives M3 = 1.0 at N=4.

**The critical dependency:** All three must work together. Avoidance without
contraction = density stays at 12-25%, M3 = 0.3-0.5. Contraction without
avoidance = V11 SIM-4 (M3 = 0.83, balanced refills reset everything).
Oversampling without contraction = V11's pack-sampling bottleneck problem
(N=12 from a 120-card pool gives M3 = 0.5). The combination is what V12
hypothesizes can reach M3 = 2.0.

**Which combination is most promising?** Design 3's champion: delayed avoidance
(pick 8) + biased contraction (50/30/0, 1.7x) + N=12 + floor slot. This is the
design with the most redundancy against failure modes. The floor slot covers the
gap when S/A depletion leaves the late pool with only 2-3 S/A; the biased
refills cover the gap from delayed avoidance; N=12 covers the gap when the pool
doesn't contract quite to 20 cards.

---

## Section 6: Physical vs V9 Contraction — Is It Honest?

**Structural equivalence:** V9 contracts from 360 to 17 (21:1 ratio, invisible).
V12 targets 120 to 20-30 (4-6:1 ratio, physical). The contraction ratios differ
significantly. V9's much higher ratio is why it achieves M3 = 2.70; V12 can
only target M3 = 2.0-2.2 with its lower ratio + oversampling supplement.

**Is V12 more honest?** Yes, materially. The player can browse the pool and
observe it contracting. They can see that their archetype's cards persist while
others disappear. They can infer that AI opponents are avoiding their lane. The
concentration mechanism is physically real, not mathematically simulated. A
player who browses the pool at pick 25 and sees 6 Storm cards in a 24-card pool
is seeing genuine physical concentration — not an artifact of a hidden scoring
function.

**The oversampling caveat:** The "draw N, show best 4" step is not fully
transparent. The player cannot see which 8-12 cards were drawn before the system
selected the best 4. This is the one hidden step in V12's otherwise transparent
mechanism. Designs using pair-affinity for the "best 4" ranking (Design 3) are
less transparent than designs using visible resonance symbol matching (Designs 2,
6). This is a real design tradeoff: pair-affinity produces M3 gains of 0.2-0.3
(per Design 3's research) but uses hidden information.

**Does it preserve concentration quality?** Partially. V9 achieves M3 = 2.70;
the best V12 face-up designs target 2.0-2.2. The 0.5-0.7 gap is real. It comes
from the lower contraction ratio and the S/A exhaustion constraint. V12's
physical mechanism cannot replicate V9's 21:1 contraction — a 30-pick draft
with 6 drafters physically removes at most 180 cards from a starting pool, which
sets a ceiling on physical contraction. V9's virtual removal has no such ceiling.

---

## Section 7: Proposed Hybrid Designs

### Hybrid 1: "Robustly Biased Pressure + N=12 + Floor" (The Conservative Path)

Combine Design 3's core structure with Design 6's transparent ranking:

- Starting pool: 120 cards
- Refills: 50/30/0 with **2.0x** open-lane bias (Design 6's ratio, not 1.7x)
- AI count: 5, gradual avoidance ramp pick 5 → 80% by pick 12 (not delayed)
- N: 4 (picks 1-5), 12 (picks 6-30)
- "Best 4" ranking: visible resonance symbol match only (no pair-affinity)
- Floor slot: guarantee 1 S/A in shown 4 if any S/A drawn
- S/A targeting: refills add S/A at 40% rate (vs 36% baseline) for open lanes

**Why this is better than either parent:** Design 3's delayed avoidance (pick 8)
loses 2-4 S/A cards in the pre-avoidance window; this hybrid moves avoidance
onset to pick 5 to recover those cards. Design 3's 1.7x bias may be too timid —
2.0x provides stronger S/A recovery. Design 6's transparent ranking trades some
M3 precision for honesty. The floor slot from Design 3 is retained as the
key redundancy mechanism.

**Predicted M3:** 2.0-2.3 (avoidance from pick 5 + 2.0x bias + floor slot is
the most S/A-protective combination tried).

### Hybrid 2: "Progressive N + Steep Biased Contraction" (The Aggressive Path)

Combine Design 2's steep contraction with Design 6's progressive N:

- Starting pool: 120 cards
- Refills: 60/20/0 with 2.0x open-lane bias
- AI count: 5, gradual avoidance ramp pick 3 → 90% by pick 12
- N: 4 (picks 1-5), 8 (picks 6-15), 12 (picks 16-30)
- "Best 4" ranking: visible resonance symbol match
- S/A targeting: refills add S/A at 40% rate for open lanes
- AI saturation: 10 cards per archetype; fallback to adjacent open lane

**Rationale:** The 60/20/0 schedule contracts the pool aggressively to ~20
cards. Progressive N (8 → 12) means during the mid-draft when the pool is still
~60-80 cards, N=8 provides light curation while keeping the draw modest. By
the time the pool reaches ~20 cards in late draft, N=12 draws 60% of the pool
— maximum concentration. The early avoidance (pick 3) with 90% final weight
preserves more S/A than Design 2's pick-5 onset. The 2.0x bias compounds this.

**Predicted M3:** 1.8-2.2 (late-draft N=12 from a deeply contracted pool + high
avoidance weight + 2.0x bias).

---

## Section 8: Algorithms Recommended for Simulation

Six simulation slots. Prioritization:

1. **Design 3 Champion (Moderate Pressure + Floor, N=12):** The strongest face-
   up candidate as specified. Must simulate to determine whether 50/30/0 with
   1.7x bias and delayed avoidance (pick 8) actually achieves M3 >= 2.0 or just
   misses.

2. **Hybrid 1 (Conservative: Robustly Biased Pressure + N=12 + Floor):** The
   most defensible path to M3 >= 2.0, combining earlier avoidance onset,
   stronger bias (2.0x), and transparent ranking. This is the primary test of
   whether the three mechanisms can work together cleanly.

3. **Hybrid 2 (Aggressive: Progressive N + Steep Biased Contraction):** Tests
   whether the 60/20/0 schedule with progressive N=8→12 and earlier/stronger
   avoidance can match or exceed Hybrid 1's M3 at lower N.

4. **Design 1 Champion (N=4 Isolation, 60/0/0):** Required for calibration.
   Establishes the pool-contraction-only baseline so simulations 1-3 can
   measure oversampling's incremental contribution.

5. **Design 2 Champion (Steep N=8, 60/20/0 biased):** Tests the N=8 knife's
   edge. If S/A is maintained at 4-5 by the biased refills, this may reach M3 =
   2.0. If S/A falls to 2-3, it confirms N=12 is needed. Critical calibration
   point between the two N strategies.

6. **Design 4 Champion (V9 Fallback):** The performance ceiling. Simulation
   verifies V9's M3 = 2.70 holds with the avoidance log layer added. Sets the
   target that face-up designs must approach to justify the complexity premium.

Designs 5 (7 AIs) and 6 (Progressive Commitment) are dropped from the
recommended simulation set: Design 5 fails M12 structurally (1 open lane) and
its honest self-assessment predicts M3 = 1.1-1.4; Design 6's champion is
superseded by Hybrid 2, which takes its progressive N idea and pairs it with
a steeper contraction schedule and the 2.0x bias that makes the math work.

---

## Summary Verdict

No V12 face-up design achieves M3 >= 2.0 with certainty as currently specified.
The most likely path is **Hybrid 1 or Design 3**, both targeting N=12 with
biased refills, a floor slot, and a pool contracted to 20-30 cards. The S/A
exhaustion constraint is the single most dangerous failure mode and must be
tracked explicitly in simulation.

If Hybrids 1-2 and Design 3 all fail M3 in simulation, Design 4 (V9 Fallback)
is the correct recommendation. V9's M3 = 2.70 is proven; adding the avoidance
log is a presentation enhancement that does not degrade the engine. The fallback
is real and should be reported honestly if simulation shows the face-up
mechanisms cannot reach M3 = 2.0 independently.

---

## Post-Critique Revision Guidance

### Design 1 Agent

Your isolation test is the most valuable calibration tool in V12, but the
specification needs one change: replace the balanced refill with an open-lane-
biased refill (1.7x or 2.0x). As currently specified (balanced 60/0/0), the
balanced refill will reset your archetype density gradient exactly as Research
Agent B warned. The result will look like V11 SIM-4 at best, not like a genuine
test of avoidance + contraction. You need to establish what avoidance +
contraction WITH biased refills produces at N=4, so that N=8 and N=12 designs
can cleanly attribute their M3 gains to oversampling. Keep N=4 as the isolation
variable; change the refill bias.

### Design 2 Agent

Your honest revised prediction (M3 = 1.4-1.8) is more credible than your
initial prediction (M3 = 1.8-2.0). The walkthrough math at picks 21-30 shows
M3 = 0.83-1.0, not 2.0. The problem is S/A: you need 5 S/A in a 20-card pool,
but the player consumes ~10 S/A over 30 picks while refills add ~5-7 S/A.
Two required changes: (1) increase the bias factor from 1.7x to 2.0x, and
(2) target S/A rates in open-lane refills at 40% instead of 36% baseline.
Additionally, move avoidance onset from pick 3 (20% weight) to "inference weight
proportional to evidence starting from pick 2" with a minimum 3-cycle window
before 50%+ weight. This preserves more S/A without false-positive risk. If
S/A remains the binding constraint after these changes, consider adding a floor
slot and moving to N=10 as a fallback parameter.

### Design 3 Agent

Your champion is the most credible face-up design. Two concerns: First, the
floor slot fires ~60-80% of packs, which is strong but means ~20-40% of picks
6-30 have no guaranteed S/A. Track this distribution carefully in simulation —
if S/A exhaustion leaves the drawn 12 cards with zero S/A in more than 30% of
late picks, M3 will miss the target despite the floor slot mechanism. Second,
test pair-affinity ranking vs visible-symbol-only ranking as a sensitivity
variable. Research notes pair-affinity adds ~0.2-0.3 M3; this is significant
but uses hidden information. If the design achieves M3 >= 2.0 without pair-
affinity, the transparent version is strongly preferred. If it requires pair-
affinity, document this clearly as a honesty tradeoff.

### Design 4 Agent

Your design is the fallback, and you have characterized it honestly. The
simulation here should focus on M12: does the visible avoidance log (Proposal B)
produce M12 = 0.20-0.35 as predicted, or do signal-readers not benefit enough
from the log to differentiate? The key simulation question is whether the
avoidance log accelerates player commitment (M5 improvement). V9's M5 = 9.6 is
one of its two metric failures (alongside M10); if the avoidance log shortens
commitment by even 1-2 picks (to M5 = 7-8), that is a meaningful improvement.
Track M5 explicitly for all three player strategy types (committed, power-chaser,
signal-reader). Also verify: does adding the log layer introduce any meaningful
performance overhead or UI confusion that creates new failure modes?

### Design 5 Agent

Your champion (7 AIs, N=12, 70/30/0 balanced) fails M12 by design and reaches
M3 = 1.1-1.4, well below target. You have already given an honest structural
verdict. For simulation, test one specific additional hypothesis: the inverse
depletion signal (player's archetype depletes *slower* than expected with 7 AIs).
Your walkthrough identifies this correctly — Storm depletes at 0.8x expected
because only 1 drafter takes it. The design should explicitly use this inverse
signal as the primary inference mechanism (not above-expected depletion). This
produces faster, more accurate inference than monitoring for unusual depletion in
the player's lane. The simulation should track: does the inverse signal produce
M14 = 4-6 (infer player archetype at pick 4-6) rather than your current pick
7-9 prediction? If so, earlier avoidance lock-in may recover enough S/A to move
M3 from 1.1-1.4 toward 1.6-1.8. This does not solve M12 but improves M3.

### Design 6 Agent

Your progressive N design is the most elegant V12 concept, but the walkthrough
table (picks 1-30) shows M3 estimations of 0.17-1.20, not 2.0, through pick 30.
The discrepancy between your predicted M3 (1.8-2.2) and the walkthrough (which
shows ~1.2 at pick 30 from a 20-card pool with 2 S/A) is significant. Revise
the specification as follows: increase the open-lane bias factor from 2.0x to
2.5x, add a floor slot to guarantee 1 S/A in the shown 4 when any S/A are drawn,
and move to a fixed N=12 from pick 16 onward (collapsing the N=10 step). The
progressive N ramp (4→6→8→10→12) has elegance but may be diluting the
oversampling benefit during the critical contraction window (picks 16-25) when
the pool is contracting fastest. Simplify to 4→8→12 (picks 1-5, 6-15, 16-30).
Track S/A count explicitly at picks 20, 25, 30 in simulation — if S/A < 3 at
pick 25, the design cannot achieve M3 = 2.0 regardless of pool size.
