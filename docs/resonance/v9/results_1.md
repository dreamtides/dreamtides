# Simulation Results: Design 4 — Layered Salience (Two-Stage Filter)

**Simulation Agent 1 — Round 4**

---

## Algorithm Summary

**Visible:** R1 filtering gates 3 of 4 slots to committed primary resonance pool from pick 6 onward. One slot always random.
**Hidden:** Within R1-filtered slots, home-archetype-tagged cards draw at 4x weight relative to sibling-tagged cards.
**Late-draft:** Pool contraction at 8%/pick from pick 12 using archetype tag relevance scores.

Hidden metadata: 3-bit archetype tag per card (1,080 bits total for 360 cards).

---

## Scorecard

### Primary: Graduated Realistic, Committed Strategy

| Metric | Target | Result | Status |
|--------|--------|--------|--------|
| M1: Unique archetypes per pack (picks 1-5) | >= 3.0 | 3.026 | PASS |
| M2: S/A per pack for emerging arch (picks 1-5) | <= 2.0 | 0.588 | PASS |
| M3: S/A per pack for committed arch (picks 6+) | >= 2.0 | **2.364** | PASS |
| M4: Off-archetype cards per pack (picks 6+) | >= 0.5 | 1.636 | PASS |
| M5: Convergence pick | 5-8 | 7.8 | PASS |
| M6: Deck archetype concentration | 60-90% | 87.2% | PASS |
| M7: Run-to-run card overlap | < 40% | 8.6% | PASS |
| M9: StdDev of S/A per pack (picks 6+) | >= 0.8 | 0.802 | PASS |
| M10: Max consecutive bad packs (picks 6+) | <= 2 | **2.13** | MARGINAL FAIL |
| M11: S/A per pack (picks 15+) | >= 3.0 | **2.397** | FAIL |

### Secondary: Pessimistic Fitness

| Metric | Target | Graduated | Pessimistic |
|--------|--------|-----------|-------------|
| M3 | >= 2.0 | 2.364 | **2.221** |
| M10 | <= 2 | 2.13 | 2.65 |
| M11 | >= 3.0 | 2.397 | 2.250 |

All 8 archetypes remain above M3 >= 2.0 at Pessimistic (worst: Flash 2.176, Ramp 2.096).

---

## V1-V4 Measurements

| Metric | Value | Assessment |
|--------|-------|------------|
| V1: Visible symbol influence | **76.3%** | Stage 1 (R1 filtering alone) yields M3 = 1.943 vs full M3 = 2.408. Visible symbols deliver 76% of the gain above random. Exceeds 60% target. |
| V2: Hidden info quantity | **3 bits/card** (1,080 bits total) | Minimum viable hidden metadata: 1 of 8 archetype labels per card. |
| V3: Reverse-engineering defensibility | **8/10** | Tags reflect real card mechanics (card's best-fit archetype). A player who discovered the system would agree each tag is a fair assessment. |
| V4: Visible resonance salience | **62.8% alignment** | In 62.8% of picks 6+, the best visible pick and best hidden pick agree. They diverge in 37.2% of picks — mainly when a sibling-tagged card has higher power than a home-tagged card, creating genuine player decisions. |

**Power-chaser gap (V4 empirical):**
- Committed M3: 2.364 | Power M3: 0.561 | Signal M3: 0.919
- Committed-vs-power gap: **+1.804** (exceeds >= 0.4 target by 4.5x)

---

## Per-Archetype M3 Table (Graduated Realistic, Committed)

| Archetype | Primary | Sibling Fitness | M3 | M5 | M6 | M9 | M10 | M11 |
|-----------|---------|-----------------|----|----|----|----|-----|-----|
| Flash | Zephyr/Ember | 25% | 2.258 | 8.8 | 85.2% | 0.837 | 2.70 | 2.335 |
| Blink | Ember/Zephyr | 30% | 2.324 | 8.0 | 87.1% | 0.813 | 1.90 | 2.384 |
| Storm | Ember/Stone | 30% | 2.302 | 7.6 | 87.0% | 0.826 | 2.34 | 2.336 |
| Self-Discard | Stone/Ember | 40% | 2.350 | 7.7 | 86.8% | 0.790 | 2.33 | 2.372 |
| Self-Mill | Stone/Tide | 40% | 2.438 | 7.5 | 87.8% | 0.786 | 1.80 | 2.472 |
| Sacrifice | Tide/Stone | 50% | 2.535 | 7.0 | 89.3% | 0.783 | 1.46 | 2.562 |
| Warriors | Tide/Zephyr | 50% | 2.477 | 7.5 | 87.1% | 0.762 | 2.10 | 2.494 |
| Ramp | Zephyr/Tide | 25% | 2.164 | 9.2 | 82.4% | 0.822 | 3.03 | 2.192 |

**Worst archetype:** Ramp at 2.164 (M3), within 8.2% of the target floor.
**Best archetype:** Sacrifice at 2.535 (M3), reflecting high sibling fitness (50%).
**M3 spread:** 0.371 across all 8 archetypes.

---

## Pack Quality Distribution (Picks 6+, Committed, Graduated)

| Percentile | SA Count |
|-----------|----------|
| P10 | 1 |
| P25 | 2 |
| P50 | 3 |
| P75 | 3 |
| P90 | 3 |

Median pack delivers 3 S/A cards. The P10 at 1 S/A represents genuine variance — a byproduct of the 1 random splash slot and pick-timing variance in archetype inference.

---

## Consecutive Bad Pack Analysis

Avg consecutive bad packs: **2.13** (target: <= 2.0)

| Consecutive Bad Packs | Drafts | % |
|----------------------|--------|---|
| 0 | 128 | 12.8% |
| 1 | 482 | 48.2% |
| 2 | 162 | 16.2% |
| 3 | 85 | 8.5% |
| 4 | 51 | 5.1% |
| 5 | 34 | 3.4% |
| 6-9 | 40 | 4.0% |
| 10+ | 3 | 0.3% |
| 25 (degenerate) | 15 | 1.5% |

The 1.5% of drafts at "25 consecutive" represents edge cases where signal readers or power chasers failed to commit to any resonance in picks 1-5, resulting in no R1 filtering ever activating. These are not committed-player drafts — they represent the intended penalty for ignoring visible resonance.

---

## Draft Traces

### Trace 1: Warriors (Committed Player)

```
Pick  1: pool=360  SA=1  --/--  res=Tide     inf=?               chose [Warriors:Tide] (S/A)
Pick  2: pool=360  SA=0  --/--  res=Tide     inf=Warriors        chose [Ramp:Zephyr] (C/F)
Pick  3: pool=360  SA=1  --/--  res=Tide     inf=Warriors        chose [Warriors:Tide] (S/A)
Pick  4: pool=360  SA=1  --/--  res=Tide     inf=Warriors        chose [Warriors:Tide] (S/A)
Pick  5: pool=360  SA=2  --/--  res=Tide     inf=Warriors        chose [Warriors:Tide] (S/A)
Pick  6: pool=360  SA=3  R1/T4  res=Tide     inf=Warriors        chose [Sacrifice:Tide] (S/A)
...
Pick 12: pool=360  SA=4  R1/T4  res=Tide     inf=Warriors        chose [Warriors:Tide] (S/A)
Pick 13: pool=332  SA=3  R1/T4  res=Tide     inf=Warriors        chose [Warriors:Tide] (S/A)
...
Pick 20: pool=188  SA=4  R1/T4  res=Tide     inf=Warriors        chose [Warriors:Tide] (S/A)
...
Pick 30: pool= 85  SA=4  R1/T4  res=Tide     inf=Warriors        chose [Warriors:Tide] (S/A)
Final: 29/30 S/A = 97%
```

**Key events:** Stage 1 (R1) activates at pick 6 after first Tide pick commits resonance. Stage 2 (T4 tag weighting) immediately active since Warriors is inferred from picks 1-5. Pack quality holds 3-4 S/A consistently from pick 6. Pool contracts from pick 12 — visible as pool size dropping from 360 to 85 by pick 30. Contraction reinforces Warriors cards while Sacrifice-tagged Tide cards gradually deplete.

### Trace 2: Sacrifice (Signal Reader — committed to wrong resonance)

```
Pick  1-4: Full pool, player reads Zephyr/Stone signals from early packs
Pick  5: Commits to Stone (2x Stone picks from signal reading)
Pick  6+: R1 filters to Stone pool (Self-Mill + Self-Discard)
  SA = 0-1 per pack (Sacrifice cards rarely appear in Stone R1 pool)
Final: 5/30 S/A = 17%
```

**Key events:** Signal reader follows visible resonance signals and commits to Stone after picking 2 Stone cards early. This is the correct player-visible behavior — the algorithm cannot rescue a player who commits to the wrong resonance. This trace illustrates V1 integrity: **visible resonance is causal**. A signal reader who ignores Tide cards and commits to Stone gets a Stone draft, not a Sacrifice draft.

---

## Comparison to V8 Baselines

| Algorithm | Pool | M3 | M10 | M11 | M6 | M9 | V1 | V2 |
|-----------|------|----|----|-----|----|----|----|----|
| V8: Narrative Gravity | 40% dual-res | 2.75 | 3.3 | n/a | 72% | 1.00 | ~35% | ~0 bits |
| V8: SF+Bias R1 | 15% dual-res | 2.24 | 8.0 | n/a | 65% | 1.05 | ~100% | ~0 bits |
| V8: CSCT (detuned) | 15% dual-res | 2.92 | 2.0 | n/a | 99% | 0.85 | ~60% | ~0 bits |
| V9 D4: Layered Salience | **10% dual-res** | **2.364** | **2.13** | **2.397** | **87.2%** | **0.802** | **76.3%** | **3 bits** |

**vs. V8 SF+Bias R1:** +0.124 M3 on a lower dual-res pool, M10 improved from 8.0 to 2.13 (dramatic improvement), at cost of 3 bits/card hidden metadata. V1 stays high (76% vs ~100%) because hidden layer is refinement-only.

**vs. V8 Narrative Gravity:** -0.386 M3 but on a pool with 4x fewer visible dual-res cards (10% vs 40%). M10 improved (2.13 vs 3.3). V1 dramatically higher: 76% vs ~35% (Narrative Gravity's contraction was driven almost entirely by visible symbols in a 40%-dual pool).

**vs. V8 CSCT:** -0.556 M3, but M6 fixed (87.2% vs 99%), V2 minimal (3 bits vs 0 bits but much less pair-matching depth).

---

## Self-Assessment: Does This Algorithm Pass?

**Partial pass. Three metrics pass; two fail.**

**PASS:**
- M3 = 2.364 (target 2.0): Passes with 18.2% margin. All 8 archetypes above 2.0 at both Graduated and Pessimistic fitness.
- M6 = 87.2% (target 60-90%): Passes. Deck concentration is high but not pathological.
- M9 = 0.802 (target 0.8): Marginal pass. Pack-to-pack variance is present.
- M1, M2, M4, M7: All pass comfortably.
- V1 = 76.3%: Strong visible symbol influence. Hidden tags provide refinement, not primary targeting.

**FAIL:**
- M10 = 2.126 (target <= 2.0): Marginal fail. The transition zone (picks 6-10) produces occasional 3-4 consecutive bad packs even with 4x tag weighting. The R1 pool at ~80 cards (40 home + 40 sibling) means ~20% of R1 slots still draw sibling-tagged cards; bad luck clusters produce M10 > 2.
- M11 = 2.397 (target 3.0): Clear fail. Pool contraction starting at pick 12 at 8%/pick does not concentrate the pool fast enough. By pick 15 (3 contractions), the pool is still ~280 cards — far too large to guarantee 3+ S/A per pack. The design spec's prediction of M11 ≈ 3.0-3.1 was optimistic.

**What would fix the failures:**

1. **M11:** Move contraction start earlier (pick 5-8) with the anchor-scaled rates proposed in Hybrid A (6%/10%/18%). Simulation would test whether this improves M11 to 3.0+ without degrading V1 below 70%. Alternatively, increase the contraction rate for picks 15+ to 15%/pick (a late-draft acceleration).

2. **M10:** Add a quality-floor slot mechanism: in each pack, reserve one slot for a "top-quartile R1" card draw (as proposed in the convergent mechanism across Round 2). This provides a guaranteed high-quality card even when two R1 slots draw sibling-tagged cards by chance. The design spec did not include this mechanism — adding it would be a spec modification.

**The core architecture works.** V1 = 76.3% confirms the layered design: visible R1 filtering delivers the majority of targeting power, and the hidden tag layer provides meaningful but secondary refinement. The algorithm passes the V9 integrity benchmark (visible resonance is genuinely causal). The failures are in late-draft density (M11) and streak prevention (M10), both addressable through contraction timing adjustments without changing the fundamental two-stage architecture.

---

## Notes on Algorithm Behavior

The signal-reader trace reveals an important design property: the algorithm does not rescue miscommitted players. A Sacrifice player who commits to Stone gets a Stone draft. This is V1 integrity in action — visible resonance is causal enough to misdirect a player who misreads it. Whether this is desirable depends on design philosophy: it punishes inattentive drafting, but also means signal readers who happen to see misleading early packs are structurally penalized.

Under Optimistic fitness (all sibling cards S/A), M3 rises to 3.309 and M11 to 3.407 — both exceeding their targets. This confirms the algorithm's mechanics are sound; the targets are sensitive to fitness model assumptions. The Graduated Realistic model (25-50% sibling rates) is the binding constraint, particularly for Flash/Ramp at 25%.
