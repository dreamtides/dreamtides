# V12 Design 4: V9 Engine + AI Avoidance Narrative (Non-Face-Up Fallback)

## 1. Key Findings

- **V9's engine is incompatible with a face-up pool.** Silent removal of cards
  the player can browse would be immediately detectable. This design operates
  in V9's native non-face-up context as the fallback, not a V12 face-up variant.

- **AI avoidance is derivable from V9's contraction pattern.** V9 removes the
  bottom 12% by blended relevance. Cards removed are precisely those least
  relevant to the player's archetype. AIs assigned to non-player archetypes
  appear to "back off" because their archetype's cards are not being removed.
  Attribution is structurally honest: the right cards go to the right AIs.

- **M3 = 2.70 is preserved exactly.** Avoidance as a presentation layer makes
  zero mechanical changes. M3, M11', and M6 are identical to V9 Hybrid B.
  This design's new contribution is to M12, M5, M13, and M14 only.

- **The avoidance narrative is partially dishonest.** AIs display "avoidance"
  not because they decided anything, but because V9's relevance scorer never
  included their cards in the removal set. The "decision" is post-hoc narrative
  over a mathematical outcome. The narrative is a simplification of truth, not
  a contradiction — defensible but not fully transparent.

- **M5 will not improve materially.** V9 Hybrid B's M5 = 9.6 (target 5-8)
  reflects slow convergence because players lack early information about open
  lanes. An avoidance log without a face-up pool adds no actionable early
  information. M5 improvement requires pool browsability; this design has none.

- **The non-face-up context forfeits V12's primary thesis.** V12 claims that
  transparent physical concentration through AI drafting can replicate V9's
  quality. Design 4 sidesteps this: the player cannot browse the pool or
  exploit avoidance as a genuine strategic signal. It is the fallback.

- **Design 4 is the performance ceiling.** Among all V12 designs, Design 4
  provides the highest guaranteed M3 (2.70, proven) at the cost of zero
  transparency. It answers: what is the best we can do if face-up mechanisms
  fail?

---

## 2. Three Algorithm Proposals

### Proposal A: V9 Pure Attribution
**Description:** V9 Hybrid B unchanged; contracted cards attributed to AIs
by nearest-archetype match; no visible avoidance behavior shown.

**Spec:** Engine: V9 Hybrid B (12%/pick, 40/60 blend, floor slot from pick 3).
5 AIs assigned randomly, 3 open lanes. Attribution invisible to player.

| Metric | Prediction |
|--------|:----------:|
| M3 | 2.70 |
| M10 | 3.8 |
| M11' | 3.25 |
| M6 | 86% |
| M12 | 0.00-0.10 |
| M13 | N/A (not visible) |
| M14 | N/A (not visible) |

---

### Proposal B: V9 + Visible Avoidance Log (Champion)
**Description:** Attributions surfaced as a visible draft log; AIs sharing
resonance with the player's archetype display "pass" events when V9's
contraction rate for their cards drops below baseline.

**Spec:** Engine: V9 Hybrid B (unchanged). 5 AIs, archetype symbols shown.
After each player pick, log shows 1-2 AI events (pick or pass). Pass event
triggered when the AI's archetype removal rate in V9 drops below 2-pick
baseline average (indicating V9 is not actively culling their cards = avoidance
display). No card names in log; no face-up pool.

| Metric | Prediction |
|--------|:----------:|
| M3 | 2.70 |
| M10 | 3.8 |
| M11' | 3.25 |
| M6 | 86% |
| M12 | 0.20-0.35 |
| M13 | 6-10 |
| M14 | 5-8 |

---

### Proposal C: V9 + Full Avoidance Theater
**Description:** V9 Hybrid B with Design 5's full information system (bars,
trends, snapshots) plus explicit avoidance narrative text at round boundaries.

**Spec:** Engine: V9 Hybrid B (unchanged). 5 AIs with visible archetype
symbols. Full Design 5 information system (availability bars, depletion trend
arrows, round-start snapshots). At round boundaries, AIs adjacent to player's
archetype display "pivoted away from contested lane" narrative. No face-up
pool.

| Metric | Prediction |
|--------|:----------:|
| M3 | 2.70 |
| M10 | 3.8 |
| M11' | 3.25 |
| M6 | 86% |
| M12 | 0.30-0.45 |
| M13 | 5-8 |
| M14 | 5-7 |

---

## 3. Champion Selection: Proposal B

Proposal A adds no player-facing value — pure accounting overhead.

Proposal C replicates V11's Standard recommendation (V9 + AI Narrative +
Design 5), already established as the best V9-family design. No new knowledge.

Proposal B tests genuinely new ground: whether a lightweight avoidance log
without full Design 5 is sufficient to improve M12 at lower implementation
cost. This is Design 4's incremental contribution.

**Narrative integrity:** Pass events are triggered by V9's contraction rate.
When V9 is not removing Blink cards in a given window, the Blink AI displaying
"held off this cycle" is directionally honest — there genuinely was less Blink
activity in the pool. The narrative is a simplification, not a fabrication.

---

## 4. Champion Deep-Dive: Pick-by-Pick Walkthrough

**Setup:** 360-card pool. Player targets Storm (Ember/Stone). AI lanes: Blink,
Warriors, Sacrifice, Self-Mill, Flash. Open lanes: Storm, Self-Discard, Ramp.

**Picks 1-4 (pre-contraction):** V9 does not contract until pick 4. Floor slot
begins pick 3. Draft log shows all 5 AIs taking cards; archetype symbols
visible. No pass events yet. Player observes no AI is in Storm.

**Picks 5-8 (inference + avoidance onset):** V9 infers Storm from pick 5
(pair-affinity mean from 3 Ember picks). Contraction now targets Storm
affinity. From pick 6, Blink AI (shares Ember primary resonance) begins showing
pass events when V9's Blink-removal rate drops below baseline. Log at pick 7:
"Blink AI: fewer options matching focus — held off this cycle." Player reads:
"Storm lane is open; Blink noticed." M13 target (AI avoidance detectable by
picks 6-10) is met. M14 target (AI infers player's archetype by picks 4-7) is
met via V9's inference at pick 5.

**Picks 9-15 (contraction ramps):** V9 has removed ~120 cards. Storm density
rising in surviving pool. Blink AI pass events more frequent. Pack quality
visibly improving — floor slot reliably landing Storm S/A. Log: "Blink AI:
stepped back from Ember (player's focus area)."

**Picks 16-25 (avoidance dominant):** Pool ~120-150 cards; Storm at 30-40%
density. Adjacent AIs (Blink, Self-Discard) showing predominantly passes.
Non-adjacent AIs (Warriors, Flash) still drafting normally. Player sees 2-3
Storm S/A per pack.

**Picks 26-30 (pool floor):** Pool approaching 17-card minimum. Storm at 60%+
density. M11' target met: 3+ Storm S/A per pack. All AIs showing minimal
activity — pool is nearly exhausted for their archetypes.

**Failure modes:**

1. **M5 stays at ~9.6.** No face-up pool means no early archetype browsing.
   The log does not accelerate commitment because packs are not concentrated
   enough at picks 1-5 to signal a clear winner.

2. **Narrative breaks under scrutiny.** A careful player may notice that AI
   "avoidance" and normal drafting produce similar apparent removal rates. The
   nearest-archetype attribution heuristic creates occasional contradictions.

3. **M12 lower than Proposal C.** Without availability bars, signal readers
   and committed players receive similar information. Early commitment advantage
   is small.

---

## 5. Complete Specification

| Parameter | Value |
|-----------|-------|
| Starting pool | 360 cards |
| Pool composition | 40 generic (11%), 284 single-symbol (79%), 36 dual (10%) |
| Contraction | V9 Hybrid B: 12%/pick from pick 4, 40/60 blend |
| Archetype inference | Pick 5: mode of higher-affinity label among drafted cards |
| Floor slot | 1 top-quartile guaranteed from pick 3 |
| Generic protection | 0.5 baseline relevance floor |
| Pool minimum | 17 cards |
| Pack construction | 1 floor slot + 3 random; N = 4 total (no oversampling) |
| Face-up pool | No |
| AI count | 5, random 5 of 8; 3 open lanes |
| AI mechanical effect | None (presentation only) |
| AI avoidance model | Derived from V9 contraction; not mechanical |
| AI inference | V9 infers at pick 5; log active from pick 6 |
| Pass event trigger | AI's archetype removal rate drops below 2-pick rolling baseline |
| Player information | Draft log: AI archetype symbols, pick/pass events; no card names |
| Per-game variety | C(8,5) = 56 compositions |
| Hidden metadata | 8 bits/card (two 4-bit pair-affinity floats) |

### Design 4 vs V12 Face-Up Designs

| Property | Design 4 (fallback) | V12 Face-Up |
|----------|:-------------------:|:-----------:|
| M3 | 2.70 (proven) | 2.0-2.4 (target) |
| M11' | 3.25 (proven) | 2.5+ (target) |
| Player pool access | None | Full browse |
| AI avoidance honesty | Partial | Full |
| M12 | 0.20-0.35 | 0.30+ |
| M5 | 9.6 (unchanged) | Target 5-8 |
| Player agency | Low | High |
| Implementation risk | Zero (proven engine) | High |

**Use when:** Face-up V12 mechanisms fail M3 >= 2.0 in simulation. Proven M3,
proven M11', zero new implementation risk.

**Do not use when:** The team values pool transparency, honest AI behavior,
or M5 improvement. A working V12 face-up design is strictly superior on those
dimensions and likely comparable on M3.

---

## Post-Critique Revision

The critic's feedback is accepted. Three points required revision.

### 1. M12 Simulation Focus

The original design asserted M12 = 0.20-0.35 for Proposal B without specifying
how to measure it. The critic correctly identifies this as the primary simulation
question: does the visible avoidance log produce differentiated outcomes for
signal-readers versus committed players and power-chasers?

**Required simulation tracking for M12:** At each pick, record whether the
player's pick correlated with an AI pass event shown in the log in the prior 2
picks. At draft end, group players into the three strategy archetypes (committed,
power-chaser, signal-reader) by observed pick patterns, and measure archetype
quality (M3 proxy) per group. M12 is confirmed if signal-readers average 0.5+
more S/A per pack than power-chasers over picks 10-25.

### 2. M5 Tracking for All Three Strategy Types

The original design conceded M5 stays at ~9.6 globally. The critic asks for
explicit per-archetype tracking — this is correct and was under-specified.

**Required M5 tracking:** Record pick at which each simulated player's final
archetype first appears as their committed archetype (defined as 3+ picks in
a single symbol pairing with no more than 1 intervening off-arch pick).

Expected per-type results:
- Committed players: M5 = 6-7 (these players converge early regardless of log)
- Signal-readers: M5 = 7-9 (log may accelerate by 1-2 picks vs Proposal A)
- Power-chasers: M5 = 10-12 (log provides no benefit; power picks delay commitment)

The critic's framing is precise: if the avoidance log shortens signal-reader
commitment by 1-2 picks (M5 dropping from ~9.6 toward 7-8 for that cohort),
that is meaningful improvement even if the global M5 average barely moves.
The simulation must track M5 by cohort, not only globally. If signal-reader M5
remains >= 9.6, Proposal B provides no improvement over Proposal A and Proposal C
should be recommended instead.

### 3. Performance and UI Verification

The critic flags that the log layer may introduce overhead or confusion. Two
concrete checks are added to the simulation protocol:

- **Performance:** The pass-event trigger (AI archetype removal rate vs 2-pick
  rolling baseline) runs once per pick after V9 contraction. This is O(1) per
  AI per pick — no meaningful overhead. Confirm by timing 10,000 simulated
  drafts with and without log generation.

- **UI confusion check:** In playtest, measure rate at which players
  misinterpret a pass event as "AI has no cards available" rather than "AI chose
  to avoid." If misinterpretation rate exceeds 30%, add a tooltip clarifying
  that pass events reflect AI preference, not pool depletion.

### Updated Predictions

M12 prediction (Proposal B) narrows to 0.20-0.30 (revised down from 0.35);
the upper bound assumed signal-readers would change picks based on the log, which
is optimistic without availability bars. M5 global prediction remains 9.6; M5
for signal-reader cohort is revised to 7.5-9.0 conditional on log readability
passing the UI confusion check.
