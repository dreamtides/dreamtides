# V12 Algorithm Design 5: High-AI-Count + Avoidance (7 AIs, 1 Open Lane)

## Key Findings

- **7 AIs accelerate non-player contraction but inference lag is a real cost.** With 8 total
  drafters removing 8 cards per pick cycle, the pool contracts faster — but research_ai_avoidance
  establishes that with 7 AIs creating confounding depletion, reliable player-archetype inference
  requires 5-7 pick cycles instead of 3-5. This delays avoidance onset and erodes the S/A
  preservation advantage.

- **The single-open-lane structure is strong for contraction, weak for variety.** C(8,7) = 8
  possible AI configurations vs C(8,5) = 56. Game-to-game variety collapses to 8 distinct
  compositions. This is a significant loss of replayability and a design trade-off that 7 AIs can
  only justify if contraction performance substantially exceeds 5-AI designs.

- **M12 (signal reading) is effectively eliminated.** With all 7 AIs avoiding the player's
  archetype once inferred, there is zero competition in the player's lane. The signal-reading
  skill axis (identifying which lane is open) is trivial: whatever the player picks becomes the
  open lane. A committed player and a signal-reading player see identical packs from pick 7
  onward, so M12 → 0.

- **The math favors N = 8 as sufficient if contraction is steep enough.** With 8 removals per
  pick cycle (vs 6 for 5-AI designs), starting pool 120, targeting 20-card late pool requires
  total refills of only ~40 cards (120 + 40 - 8×20 = 0 at pick 20). A refill schedule of
  40/0/0 is viable — but produces a very thin pool in Round 2-3, requiring tight AI saturation
  management.

- **Balanced refills remain poisonous; open-lane bias is essential but structurally
  different here.** With 1 open lane instead of 3, "open-lane-biased refills" now mean refills
  biased heavily toward the player's single lane. This is more precisely targeted than V11's
  1.7x open-lane bias (which spread over 3 lanes), but requires a player-reactive refill
  mechanism — which crosses into Level 1 territory.

- **S/A supply is more precarious at 7 AIs due to pre-avoidance window.** With inference
  delayed to picks 5-7, AIs take ~4-5 on-archetype cards in the pre-avoidance window (7 AIs
  × 5 cycles × ~12.5% archetype fraction ≈ 4.4 expected player-arch cards taken). Starting
  S/A = 5; pre-avoidance losses of 1.5-2.0 S/A leave only ~3 S/A before avoidance locks in.

- **N = 8 reaches M3 ≈ 2.0 only in the optimistic case (pool = 20, S/A = 5).** The S/A
  depletion analysis from research_concentration_math shows that maintaining 5 S/A through
  pick 20 requires biased refills adding back at least 5-8 S/A total. With 1 open lane and
  declining refills, this is achievable but leaves no margin for the inference delay cost.

---

## Three Algorithm Proposals

### Proposal A: Rapid-Contraction Blitz (7 AIs, Steep Decline, N = 8)

**Description:** 7 AIs with gradual avoidance (ramp picks 5-12), aggressive 2-round refill
schedule (50/0), targeting pool = 20 cards by pick 25 for M3 = 2.0 at N = 8.

**Technical Specification:**
- Starting pool: 120 cards (15/archetype, 5 S/A/archetype)
- AI count: 7 (1 open lane: player's archetype)
- AI archetype assignment: random 7 of 8 at draft start
- Refill schedule: 2 rounds (15 picks / 15 picks); refill after pick 15: 50 cards balanced;
  no R2 refill
- Total cards: 120 + 50 = 170; total removals: 8 × 30 = 240 (pool exhausted at pick ~21
  without saturation management — see below)
- AI saturation: each AI holds at most 15 archetype cards; once saturated, shifts to any
  remaining non-player card
- Avoidance model: gradual ramp starting pick 5, sigmoid to 90% weight reduction by pick 12
- Inference method: depletion ratio — track archetype departure rates over rolling 4-cycle
  window; flag archetype with > 1.5× expected depletion rate for 3 consecutive cycles
- Oversample N: 8 (draw 8, show best 4 by archetype fitness score)
- Exploration: picks 1-5 use N = 4 (uniform), pool browser provides archetype discovery
- Pack ranking: archetype_fitness score (S/A ranks ~0.9, sibling ~0.5-0.7, off-arch ~0.1)

**Pool trajectory (estimated):**

| Pick | Pool Size | P-Arch Cards | S/A Remaining | Arch % | N=8 M3 |
|:----:|:---------:|:------------:|:-------------:|:------:|:------:|
| 1 | 120 | 15 | 5 | 12.5% | 0.33 |
| 5 | 80 | 13 | 4.5 | 16.3% | 0.45 |
| 10 | 42 | 11 | 3.5 | 26.2% | 0.67 |
| 15 (pre-refill) | 8 | 9 | 3.0 | 112% — pool too thin | — |

**Problem:** With 8 removals/cycle and no refill until pick 15, the pool reaches ~8 cards
by pick 15 — non-viable. Saturation kicks in by pick 10-12 (AIs reach 15 cards), but even
with saturation, the pool exhausts before pick 15. **Proposal A fails due to pool
exhaustion.** The math is unforgiving: 120 cards / 8 per cycle = 15 picks to exhaustion.
Proposal A is not viable without earlier refills.

**Predicted metrics:** Fails — pool exhaustion by pick 12-15 makes draft impossible.

---

### Proposal B: Front-Loaded Refill (7 AIs, Moderate Decline, N = 8)

**Description:** 3-round structure with a front-loaded refill schedule (70/30/0) designed to
sustain viability while allowing steep late contraction, combined with gradual AI avoidance
(picks 5-12) and N = 8.

**Technical Specification:**
- Starting pool: 120 cards
- AI count: 7
- Refill schedule: 3 rounds (10 picks each); refill after R1: 70 cards; after R2: 30 cards;
  no R3 refill
- Total supply: 120 + 70 + 30 = 220 cards; total removals: 8 × 30 = 240 cards
- Math: 220 - 240 = -20 → pool exhausts in R3 unless AI saturation reduces effective removal
  rate. With saturation (max 15 cards/AI), by R3 each AI is saturated and takes ~0-1 card
  per cycle → effective R3 removal rate ≈ 1-2 per cycle. Pool viability in R3: 30-card refill
  baseline, ~15-20 picks remain at 1-2 removals/cycle → pool ends at ~5-15 cards. Viable.
- AI saturation: 15-card hard cap per AI, then switches to any available non-player card
- Avoidance: gradual ramp, inference picks 5-7 (rolling 4-cycle depletion window), 90%
  weight by pick 12
- Oversample N: 8 (picks 6+); N = 4 picks 1-5
- Refill bias: balanced (adds cards proportionally to all archetypes including player's)
- Pack ranking: archetype_fitness

**Pool trajectory:**

| Event | Pool | P-Arch | S/A | Arch % | N=8 M3 |
|-------|:----:|:------:|:---:|:------:|:------:|
| Start | 120 | 15 | 5.0 | 12.5% | 0.33 |
| Pick 5 | 80 | 13 | 4.3 | 16.3% | 0.43 |
| R1 end (pre-refill) | 42 | 10 | 3.2 | 23.8% | 0.61 |
| R2 start (post-70 balanced refill) | 112 | 18.8 | 5.0 | 16.8% | 0.36 |
| R2 pick 10 (avoidance active) | 32 | 10 | 3.5 | 31.3% | 0.88 |
| R3 start (post-30 balanced refill) | 62 | 14 | 4.1 | 22.6% | 0.53 |
| Pick 25 (saturation phase) | 18 | 7 | 2.5 | 38.9% | 1.11 |
| Pick 30 | 10 | 4 | 1.5 | 40.0% | 1.20 |

**Assessment:** Balanced refill resets the gradient hard at R2 start (23.8% → 16.8%), and
again at R3 start (31.3% → 22.6%). S/A count rebuilds via refills but stays low. Late-draft
M3 reaches ~1.2 at pick 30 — better than V11's best (0.89) but nowhere near 2.0. The 7-AI
configuration accelerates contraction in absolute terms but the balanced refill reset and
inference delay combine to prevent density from building fast enough.

**Predicted metrics:** M3 ≈ 1.0-1.2 (picks 6+), M11' ≈ 1.4-1.6, M10 ≈ 4-6 (consecutive
bad packs), M6 ≈ 65-72%, M12 ≈ 0.05 (M12 nearly zero — no signal-reading advantage with 1
open lane), M13 ≈ pick 7, M14 ≈ pick 6.

---

### Proposal C: Dense AI + Open-Lane Biased Refills (7 AIs, Biased Refills, N = 8)

**Description:** 7 AIs with gradual avoidance, open-lane-biased refills (all refill cards
go to the single open lane = player's archetype), and N = 8. The key departure: since there
is only 1 open lane, open-lane bias is maximally targeted — every refill card replenishes the
player's archetype, replacing what the player consumes while AIs deplete everything else.

**Technical Specification:**
- Starting pool: 120 cards
- AI count: 7 (1 open lane: player's archetype)
- Refill schedule: 3 rounds (10 picks each); after R1: 50 cards, all to player's archetype;
  after R2: 20 cards, all to player's archetype; no R3 refill
- Refill mechanism: classified Level 1 (player-reactive) — refills target the player's
  inferred archetype. This crosses the Level 0 boundary but creates a cohesive mechanic:
  "the market restocks the open lane heavily." With 1 open lane, the mechanism is highly
  visible and explainable.
- Total supply: 120 + 70 = 190 cards. Non-player archetypes: 105 cards consumed by 7 AIs
  over 30 picks (105 = 15 × 7). Player's archetype: starts at 15, +70 via biased refills =
  85 available; player takes 30 cards over 30 picks → 55 remaining at end. Pool = 55 + (0
  non-player remaining) = 55 cards at end — too large.
- Revised: reduce starting pool to 80 cards (10/archetype). Non-player supply: 10 × 7 = 70.
  Removals by AIs: 7 × 30 = 210 (each AI picks 1/cycle), but saturation caps at 15 → 7 × 15
  = 105 total AI picks. After saturation, AIs stop (no non-player cards left). Player supply:
  10 + 70 refills = 80 → player takes ~25-28 → ~52-55 remaining. Pool at end = 52-55 mostly
  player's archetype.
- With 80-card start and biased refills, late pool = 50+ player-arch cards with S/A = 12-18
  (70 refill adds 70 × 36% = 25 S/A; player consumes ~10-12 S/A; remaining S/A = 13-15).
  Late-pool density: 13-15 S/A out of 50-55 = 24-30% density. N=8 M3 = 8 × 13/50 ≈ 2.08.
- AI avoidance: gradual ramp picks 5-12, inference from 4-cycle depletion window
- Oversample N: 8; N = 4 for picks 1-5
- Refill bias: 100% to player's inferred archetype (Level 1 mechanism)

**Pool trajectory (80-card start, biased refills):**

| Event | Pool | P-Arch | S/A | Arch % | N=8 M3 |
|-------|:----:|:------:|:---:|:------:|:------:|
| Start | 80 | 10 | 3.6 | 12.5% | 0.29 |
| Pick 5 (pre-avoidance) | 43 | 8.5 | 3.0 | 19.8% | 0.56 |
| R1 end (pre-refill) | 5 | 6.0 | 2.2 | — | pool danger |

**Problem:** 80 cards / 8 picks per cycle = 10 picks to exhaustion without refills. Pool runs
out mid-R1 before the refill event. A front-loaded R1 partial refill of 30 cards at pick 5
stabilizes things but adds complexity. The single-open-lane biased refill mechanism, while
mathematically viable, is a Level 1 mechanism that undermines V12's honesty criterion. It also
requires the system to infer the player's archetype to determine where to send refills —
exactly the kind of player-reactive behavior V12 rules out in Level 0 designs.

**Adjusted conclusion:** Proposal C's biased refill variant produces M3 ≈ 2.0+ in late picks
but violates Level 0 honesty. The pool math also requires careful management. The mechanism
works but at a design integrity cost.

**Predicted metrics (if Level 1 is accepted):** M3 ≈ 1.8-2.1 (picks 6+), M11' ≈ 2.2-2.6,
M10 ≈ 2-3, M6 ≈ 70-82%, M12 ≈ 0.05, M13 ≈ pick 7, M14 ≈ pick 6-7.

---

## Champion Selection

**Champion: Proposal B (Front-Loaded Refill, 7 AIs, Balanced Refills, N = 8) — with a
critical modification: increase N to 12.**

**Justification:** Proposal A fails on pool exhaustion. Proposal C achieves M3 ≈ 2.0 but
requires a Level 1 player-reactive mechanism that violates V12's core honesty principles.
Proposal B is the only design that stays within Level 0 constraints and sustains viability
through all 30 picks.

However, Proposal B as specified reaches only M3 ≈ 1.0-1.2. Upgrading to N = 12 (draw 12,
show best 4) addresses this within the orchestration plan's allowed range. The late-draft
pool of 10-18 cards at picks 25-30 with N = 12 yields M3 = 12 × 2.5/18 ≈ 1.67 — still
below 2.0.

**Honest assessment:** No configuration of 7 AIs + Level 0 balanced refills + N = 8-12
reaches M3 = 2.0. The 7-AI design's fundamental problem is the inference delay: with 7 AIs
creating confounding depletion, the AI avoidance doesn't fully engage until picks 7-12, which
is too late. By then, the pool has already contracted steeply and S/A has been partially
consumed. The front-loaded refill (70/30/0) partially offsets this but triggers a severe
gradient reset.

**The champion is the best viable 7-AI design, not a design that achieves the target.** Its
purpose is comparative: establishing how much worse 7 AIs perform on M12, variety (M7/M8),
and signal-reading relative to 5-AI designs, to inform the critic review's cross-proposal
ranking.

---

## Champion Deep-Dive: Front-Loaded Refill (Proposal B, N = 12 Modified)

### Parameters
- Starting pool: 120 cards (15/archetype)
- AIs: 7 (assigned to random 7 of 8 archetypes; 1 open lane = player's)
- Refill: 70 cards after pick 10, 30 cards after pick 20, 0 thereafter
- Refills: balanced (1/8 to each archetype)
- Avoidance: gradual ramp, inference from rolling 4-cycle depletion window; 90% weight by
  pick 12
- N = 12 (picks 6+), N = 4 (picks 1-5)
- Pack ranking: archetype_fitness score

### Pick-by-Pick Walkthrough

**Picks 1-5: Exploration Phase (N = 4, uniform packs)**

Pool = 120 cards. Player browses face-up pool — all 8 archetypes visible with ~15 cards each.
Packs are random 4 from pool. Player explores and observes which archetypes look promising.

AIs pick freely (no avoidance yet). Each AI takes ~1 card from its archetype per cycle.
7 AIs × 5 cycles = 35 AI picks + 5 player picks = 40 total removals.
Pool at pick 5: ~80 cards. Player's archetype: ~13 cards (2 taken by AI randomly, picks 1-5
pre-avoidance), S/A ≈ 4.3.

Depletion inference begins. No archetype yet shows 1.5× expected depletion rate sustained
over 3+ cycles. No avoidance kicks in. The player observes that 7 of 8 archetypes are losing
cards fast.

**Picks 6-9: Early Commitment and Inference Window (N = 12 begins)**

Player commits to an archetype (let's say Storm/Ember) around pick 5-6.
Pack construction shifts to N = 12: draw 12 from 80-card pool, show best 4 by Storm fitness.
Expected Storm S/A shown: 12 × 4.3/80 ≈ 0.65 per pack. Low, but better than N = 4 (0.22).

AIs continue taking from their archetypes. By pick 9, the AI inference system has 4 cycles of
data post-pick 5. Storm cards (player's arch) are depleting at 1 card/cycle (player only) vs
expected ~10/80 per cycle per all drafters = 1.25 expected. The depletion ratio for Storm is
0.8 — BELOW expected, because only 1 drafter (player) is taking Storm, while 7 AIs take from
other archetypes. This is the key avoidance signal.

The AI inference correctly flags Storm as the archetype with below-expected depletion (not
above — this is the inverse of detecting an AI-overdriven archetype; the player's lane is
notable for being the ONLY archetype losing cards at the single-drafter rate).

By pick 9: AIs begin mild avoidance of Storm (20-30% weight reduction). AIs that have reached
15 cards for their assigned archetype start taking incidental cards, but specifically avoid
Storm incidentals.

**Picks 10-12: Refill Event and Gradient Reset (M13 = pick 9-10)**

Pick 10 triggers the 70-card balanced refill. Pool state pre-refill: ~40 cards.
Post-refill: ~110 cards. Storm cards jump from ~11 to ~11 + 70/8 ≈ 19.8 cards.

**What the player sees:** The pool visibly refills. Many archetypes the player wasn't tracking
reappear with cards. Storm goes from ~11 cards to ~20 cards (large jump visible in pool
browser). The player notes: "My archetype got more cards, but so did everything else."

AI avoidance: by pick 12, AIs have 7+ cycles of data and Storm's depletion ratio is well
below expected (0.75-0.85). Avoidance ramps to 60-70% weight reduction. AIs will take non-
Storm incidental cards only.

Pool state at pick 12: ~90-95 cards. Storm = ~18-19 cards (player has taken ~3 more since
refill). S/A in Storm: ≈ 4.5-5.0 (refill added ~2.5 S/A, player took ~1.5).

M3 at pick 12: 12 × 5/95 ≈ 0.63. Still below target.

**Picks 13-20: Contraction Phase (avoidance fully active)**

7 AIs fully avoiding Storm. Effective removal rate: 7 AIs × ~0.8 non-Storm picks/cycle +
1 player Storm pick = ~6.6 removals/cycle, mostly non-Storm. Pool contracts from ~95 to
~30-35 cards over picks 13-20. Storm fraction grows.

At pick 15: pool ≈ 75 cards. Storm ≈ 15 cards. S/A ≈ 3.8. M3 = 12 × 3.8/75 ≈ 0.61.
At pick 18: pool ≈ 50 cards. Storm ≈ 12 cards. S/A ≈ 3.0. M3 = 12 × 3.0/50 ≈ 0.72.
At pick 20: pool ≈ 30 cards. Storm ≈ 9 cards. S/A ≈ 2.5. M3 = 12 × 2.5/30 ≈ 1.00.

Pick 20 triggers 30-card balanced refill. Pool jumps to ~60 cards. Storm: ~9 + 30/8 ≈ 12.8
cards. S/A: ~2.5 + ~1.4 = ~3.9. Gradient reset again: Storm density drops from 30% back to
21%. M3 = 12 × 3.9/60 ≈ 0.78. Second reset undoes substantial progress.

**Picks 21-30: Final Contraction (no refill)**

After pick 20 refill, pool = 60 cards. 7 AIs still avoiding Storm. Removal rate ≈ 6.5/cycle.
Pool contracts from 60 to ~5-10 cards by pick 30. Storm accumulates as dominant fraction.

At pick 25: pool ≈ 28 cards. Storm ≈ 9 cards. S/A ≈ 2.8. M3 = 12 × 2.8/28 = 1.20.
At pick 28: pool ≈ 13 cards. Storm ≈ 7 cards. S/A ≈ 2.2. M3 = 12 × 2.2/13 = 2.03.
At pick 30: pool ≈ 7 cards. Storm ≈ 5 cards. S/A ≈ 1.8. M3 = 12 × 1.8/7 = 3.09 (cap 4).

**What the player experiences in picks 21-30:** The pool shrinks visibly. Storm cards remain
while everything else disappears. The player's packs are high-quality — nearly all Storm cards,
mostly S/A. The oversampling at N = 12 ensures the player sees all the S/A in the tiny pool
repeatedly. This is the "earned" feeling: staying in an uncrowded lane produces excellent
late-draft packs.

**Failure modes:**

1. **Double gradient reset.** Two balanced refills (at picks 10 and 20) each reset the
   archetype density gradient. Picks 10-13 and 20-23 are substantially weaker than surrounding
   picks. M10 (consecutive bad packs) will exceed target.

2. **Late-onset avoidance reduces mid-draft quality.** Picks 6-12 see M3 = 0.4-0.6 even at
   N = 12 because the pool is still large and avoidance hasn't fully engaged. This is the
   7-AI inference delay cost.

3. **S/A exhaustion in picks 26-30.** With S/A dropping to ~1.5-2.0 in the tiny final pool,
   and N = 12 drawing most of the pool, the player may see the same 1-2 S/A cards repeated
   across consecutive packs (they appear every pack but already taken). Pool must maintain
   at least 3 S/A through pick 27 to avoid empty-feeling packs.

4. **Narrow variety.** C(8,7) = 8 game compositions. Players who draft frequently will see
   all 8 within a handful of runs. The single open lane means every game feels structurally
   similar: you are always the unchallenged lane. Strategic diversity within a run is also
   limited — there is no decision about which lane to find, only which lane you happen to
   start picking.

### M12 Analysis

With 1 open lane and all 7 AIs eventually avoiding the player's archetype:
- A committed player (commits pick 5-6) receives avoidance benefit starting pick 9-12.
- A signal-reading player (commits pick 2-4 from pool browser signals) receives avoidance
  benefit starting pick 6-9.
- The difference is 3-6 picks of avoidance protection, corresponding to ~1-2 S/A cards
  preserved. Expected M3 advantage for signal-reader: 0.05-0.15.

**M12 ≈ 0.05-0.15 (fails target of >= 0.3).** The 1-open-lane structure essentially
guarantees M12 failure: there is no "wrong lane" to avoid, so reading signals correctly
provides minimal benefit over a committed player.

What replaces M12 as a skill axis: **pack-efficiency optimization.** With all 30 picks on-
archetype, the player's skill expression comes from sequencing S/A vs C/F picks wisely,
managing deck composition within the archetype, and knowing when to take a strong off-arch
card (visible in the pool). This is a narrower skill axis than lane-reading.

---

## Complete Specification

| Parameter | Value |
|-----------|-------|
| Starting pool size | 120 cards (15/archetype, 5 S/A/archetype) |
| AI count | 7 |
| Open lanes | 1 (player's chosen archetype) |
| AI archetype assignment | Random 7 of 8 at draft start; no duplicates |
| Refill schedule | 3 rounds (10 picks each); +70 balanced after pick 10; +30 balanced after pick 20; 0 thereafter |
| Total supply | 220 cards; total removals 240 (viable via AI saturation) |
| AI saturation threshold | 15 archetype cards per AI; post-saturation picks any non-player card |
| Oversample N | 4 (picks 1-5, exploration); 12 (picks 6-30, execution) |
| "Best 4" ranking criterion | archetype_fitness score: S/A for committed archetype ≈ 0.9; sibling S/A ≈ 0.5-0.7; on-arch C/F ≈ 0.3; off-arch ≈ 0.0-0.2 |
| Player archetype inference for ranking | Mode of higher-affinity label among player's drafted cards, from pick 6 |
| AI avoidance model | Gradual ramp: 0% avoidance picks 1-4; 20-40% picks 5-8; 60-80% picks 9-12; 90% picks 13+ |
| AI inference mechanism | Rolling 4-cycle depletion window; flag archetype with < 0.85× expected departure rate for 3+ consecutive cycles as likely player archetype (inverse signal: player's arch depletes slower than expected) |
| AI pick logic | Pick highest-fitness card from assigned archetype, excluding player's inferred archetype by avoidance weight; post-saturation: pick any non-player card by fitness |
| Refill bias | Balanced (1/8 per archetype per refill) — preserves Level 0 honesty |
| Player information | Full face-up pool (browse anytime); pack = 4 cards drawn as best of N=12; who took what is secret; pool browser shows card counts by archetype |
| Game-to-game variety | C(8,7) = 8 possible AI configurations |
| Predicted M3 (picks 6+) | 0.7-0.9 early-mid (picks 6-22), 1.8-2.5 late (picks 26-30); overall average ≈ 1.1-1.4 |
| Predicted M11' (picks 20+) | 1.6-2.0 |
| Predicted M10 | 4-6 (fails target <= 2) |
| Predicted M6 | 70-80% |
| Predicted M12 | 0.05-0.15 (fails target >= 0.3) |
| Predicted M13 | Pick 9-11 |
| Predicted M14 | Pick 7-9 |

### Structural Verdict

The 7-AI, 1-open-lane design achieves faster absolute pool contraction than 5-AI designs but
pays three compounding costs: (1) delayed AI inference (5-7 cycles vs 3-5 cycles), which
erodes the S/A preservation advantage; (2) balanced refill gradient resets, which both 5-AI
and 7-AI designs suffer but are proportionally more damaging here because the steeper
contraction amplifies the reset cost; and (3) M12 collapse, which is structural and
unavoidable with 1 open lane. Unless the biased refill mechanism (Proposal C, Level 1) is
accepted, the 7-AI design underperforms the 5-AI equivalents on M3 while also failing M12
and variety metrics. Its comparative value is as a data point proving that maximizing AI
count does not compensate for loss of lane structure and signal-reading skill.

---

## Post-Critique Revision

The critic's feedback is accepted. The walkthrough already identified the inverse depletion
signal in passing (Storm depletes at 0.8× expected because only 1 drafter takes it), but the
champion specification treated it as a secondary observation rather than the primary inference
mechanism. That framing was wrong. This revision corrects it.

### Inverse Depletion as Primary Inference Mechanism

The original champion specifies the inference trigger as: flag archetype with
`> 1.5× expected` departure rate for 3+ consecutive cycles. This monitors for unusual
*above-expected* depletion — the correct signal for detecting an AI-overdriven archetype. For
the 7-AI table, the player's archetype is the one depleting *below* expected, not above. The
corrected inference mechanism is:

- For each archetype each cycle, compute: `actual_departure / expected_departure`
- Expected departure per cycle: `archetype_cards / pool_size × total_picks_per_cycle`
- Flag archetype as likely player archetype when ratio `< 0.85×` sustained for 3+ consecutive
  cycles — interpreted as "this archetype is being taken by only one drafter, not multiple"
- Begin avoidance ramp immediately on flag, without waiting for additional confirmation cycles

This is faster than monitoring for above-expected depletion because the below-expected signal
is unambiguous at a 7-AI table: any archetype losing cards at a single-drafter rate in a field
of 8 drafters stands out strongly. The three-cycle confirmation window can be shortened to two
cycles because false positives are rare — with 7 AIs each depleting their own archetype
heavily, the player's lane will consistently show suppressed depletion from pick 3 onward.

### Updated M14 Prediction

With a two-cycle confirmation window starting from pick 3, inference completes by pick 5 in
expectation. The revised M14 prediction is **pick 4-6** (revised from pick 7-9). The critic's
target range of 4-6 is achievable under this mechanism.

Earlier lock-in at pick 4-6 means avoidance engages 3-5 picks sooner than the original
specification predicted. This preserves approximately 1.5-2.5 additional S/A cards in the
player's archetype before the balanced refills reset the gradient.

### Updated M3 Prediction

With avoidance locking in at M14 = 4-6 rather than 7-9, the pre-avoidance S/A loss falls
from ~1.5-2.0 to ~0.8-1.2. Combined with N = 12 and the steep contraction trajectory in
picks 21-30, the revised M3 prediction is **1.4-1.8** (revised from 1.1-1.4). The critic's
suggested range of 1.6-1.8 is the optimistic end of this band, achievable if the pool reaches
~18-22 cards at picks 25-28 with 2.5+ S/A remaining.

### Unchanged Assessments

M12 remains 0.05-0.15 and fails the target. This is structural: 1 open lane eliminates lane-
selection skill regardless of inference speed. Earlier avoidance lock-in does not rescue M12.

The champion remains Proposal B with N = 12 modification. This revision improves its M3
ceiling but does not change the fundamental comparative verdict: the 7-AI design is a data
point establishing the cost of collapsing lane structure, not a candidate for the primary
simulation recommendation.
