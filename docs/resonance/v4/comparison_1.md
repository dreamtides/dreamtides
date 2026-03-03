# Comparison 1: Cross-Strategy Analysis (Agent 1 — Exile Pressure)

## Scorecard Table (1-10, with justification)

| Goal | Exile Pressure | Sqrt Affinity | Pack Widening v2 | Phantoms | Deck Echo |
|------|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 5 | 6 | 9 | 9 | 7 |
| 2. Not on rails | 8 | 9 | 6 | 9 | 9 |
| 3. No forced decks | 8 | 8 | 5 | 9 | 9 |
| 4. Flexible archetypes | 7 | 8 | 7 | 7 | 8 |
| 5. Convergent | 4 | 4 | 9 | 2 | 3 |
| 6. Splashable | 8 | 9 | 8 | 8 | 9 |
| 7. Open early | 9 | 9 | 5 | 9 | 9 |
| 8. Signal reading | 6 | 3 | 3 | 10 | 3 |
| **Total** | **55** | **56** | **52** | **63** | **57** |

**Justifications for key scores:**

- **Pack Widening convergence (9):** Only strategy hitting 2.0+ S/A (3.35 at cost 2, 2.70 at cost 3). Docked 1 point for over-convergence (98.6% concentration).
- **Phantoms convergence (2):** 1.26 S/A is worst of all five; parameter sweeps show no path to 2.0.
- **Phantoms signal reading (10):** Only strategy where signal reading is a first-class mechanic — phantom consumption is directly observable.
- **Pack Widening simplicity (9):** One-sentence description is fully implementable by a programmer without ambiguity.
- **Exile Pressure simplicity (5):** Three-clause run-on sentence; technically complete but not truly simple.
- **Pack Widening open early (5):** 2.48 early S/A fails the <=2 target; spending from pick 1 narrows too fast.

## Biggest Strength and Weakness per Strategy

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|------------------|
| Exile Pressure | Archetype balance (6.8-7.7 conv. band) | Resonance≠archetype dilution caps S/A at ~1.6 |
| Sqrt Affinity | Best splash/flexibility (1.11 off-arch, concave scaling) | Convergence too slow (pick 10.5) and too weak (1.74) |
| Pack Widening v2 | Only strategy hitting convergence target (3.35 S/A) | Over-converges (98.6% conc.) with trivial spending decisions |
| Phantoms | First-class signal reading (10/10) | Fundamentally insufficient pool suppression (1.26 S/A) |
| Deck Echo | Best natural variance profile (0.97 stddev, 87% conc.) | Same structural ceiling as Exile (~1.55 S/A) |

## The V3 Comparison

**No V4 algorithm clearly dominates Lane Locking.** Lane Locking passes convergence (2.08-2.39 S/A) where four of five V4 strategies fail. However, Lane Locking consistently fails deck concentration (96-98% vs 60-90% target) and borderline-fails variance (stddev 0.70-0.95).

The tradeoff is structural: V4 algorithms universally win on variance, splash, flexibility, and deck diversity. Lane Locking universally wins on convergence speed and strength. Pack Widening v2 is the only V4 algorithm that matches Lane Locking's convergence power, but it shares Lane Locking's over-convergence problem.

**The meta-lesson:** Probabilistic approaches (filtering, weighting, scarcity) cannot overcome ~11% archetype density in a 360-card pool. Only mechanisms that ADD targeted cards (Pack Widening) or deterministically PLACE cards (Lane Locking) can reach 2.0+ S/A. This is the fundamental finding of V4 — natural variance and strong convergence require card injection, not card selection.

## Proposed Best Algorithm: Modified Pack Widening v3

**One-sentence description:** "Each symbol you draft earns 1 matching token (primary earns 2); you may spend 3 tokens of one resonance to add 1 extra card with that primary resonance to the pack."

This is a tuned Pack Widening v2 with two key changes, informed by cross-team discussion:

1. **Cost 3 instead of 2:** Creates real save/spend decisions. At cost 3, players earn ~3 tokens per pick (primary=2 + secondary=1), so spending is possible roughly every other pick — creating a genuine save/spend rhythm rather than cost 2's trivial always-spend.

2. **Bonus 1 instead of 2:** Reduces over-concentration. Agent 3's parameter sweep showed bonus=1 achieves 2.34 S/A (passes the 2.0 target) while being a gentler nudge (5-card pack vs 6-card). This should bring deck concentration down from 98.6% toward the 60-90% target range.

**No spending gate needed.** Initially proposed a pick-5 gate, but Agent 3 and Agent 5 correctly noted that cost 3 is naturally self-limiting — players rarely accumulate 3 tokens before pick 3-4, and even then, early spending on one resonance barely narrows the draft. The early S/A failure (2.48) was specific to cost 2's trivial always-spend. Dropping the gate keeps the one-sentence description clean with no exceptions.

**Why not a cross-domain hybrid?** Agent 2 proposed Phantoms + Pack Widening for signal reading. While conceptually appealing, adding a second mechanism makes the one-sentence description significantly harder and violates the simplicity principle (goal #1). Pack Widening's token system already provides implicit signal reading. Signal reading is goal #8 (lowest priority); simplicity is goal #1.

**Why not Lane Locking?** Modified Pack Widening preserves convergence power while adding: (a) natural pack-to-pack variance via stochastic bonus cards, (b) player agency via spend/save decisions, (c) better deck diversity via non-deterministic composition. It uniquely gives the player an active choice — "do I spend tokens now or save?" — which no other algorithm offers.

**Why not sub-2.0 probabilistic approaches?** At 1.55-1.74 S/A, committed players see 0 S/A cards in 10-14% of packs — roughly 3-4 completely dead packs over picks 6-30. Dead packs aren't "interesting variance"; they're frustrating. The variance target should prevent mechanical delivery, not justify frustrating delivery.

**Predicted metrics (needs Round 5 simulation):** Late S/A ~2.3, stddev ~1.2, convergence pick ~6-7, deck concentration ~80-90%. The exact numbers require simulation, but the mechanism has a clear path to passing all targets.
