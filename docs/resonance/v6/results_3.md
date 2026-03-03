# Results: Agent 3 -- Threshold-Triggered Soft Locks

## One-Sentence Algorithm

"Each pack slot starts random; when your top resonance count crosses 3, 6, and 9, one more slot begins showing a resonance-matched card 75% of the time (first two target top resonance, third targets second-highest), while slot 4 stays fully random."

## Scorecard (archetype_committed / power_chaser / signal_reader)

| Metric | Target | Committed | Power | Signal |
|--------|--------|-----------|-------|--------|
| M1: Early diversity | >= 3 | 4.91 | 5.04 | 5.03 |
| M2: Early emerging S/A | <= 2 | 1.67 | 1.43 | 1.44 |
| M3: Late committed S/A | >= 2 | **2.20** | 1.93 | 1.86 |
| M4: Late C/F splash | >= 0.5 | 0.49 | 0.61 | 0.64 |
| M5: Convergence pick | 5-8 | **5.36** | 5.67 | 5.61 |
| M6: Deck concentration | 60-90% | 97.1%* | 53.5%* | 92.2%* |
| M7: Run overlap | < 40% | 4.6% | 9.1% | 4.8% |
| M8: Archetype balance | 5-20% | PASS | 2 fail | PASS |
| M9: StdDev S/A (6+) | >= 0.8 | 0.88 | 0.82 | 0.84 |

*M6 fails: committed too high (97%), power too low (54%).

## Variance Report

Picks 6+ S/A distribution (committed): 0 S/A: 2.3% | 1: 18.1% | 2: 42.8% | 3: 31.3% | 4: 5.5%. StdDev=0.88. The 75% probability creates genuine pack-to-pack unpredictability even at full lock activation.

## Per-Archetype Convergence (archetype_committed)

| Archetype | Conv Pick | Count | Conc |
|-----------|:-:|:-:|:-:|
| Flash | 5.37 | 162 | 97.2% |
| Blink | 5.29 | 120 | 96.7% |
| Storm | 5.27 | 74 | 97.3% |
| Self-Discard | 5.38 | 137 | 97.1% |
| Self-Mill | 5.19 | 105 | 96.6% |
| Sacrifice | 5.48 | 129 | 97.2% |
| Warriors | 5.37 | 131 | 97.3% |
| Ramp | 5.41 | 142 | 97.3% |

All 8 archetypes converge between pick 5.2-5.5. Storm least common (7.4%) but above 5%.

## Parameter Sensitivity

**Lock probability** (thresholds 3/6/9):

| Prob | S/A | StdDev | Conv |
|------|-----|--------|------|
| 0.50 | 1.70 | 0.94 | 5.87 |
| 0.75 | 2.23 | 0.87 | 5.44 |
| 0.85 | 2.45 | 0.81 | 5.12 |
| 1.00 | 2.74 | 0.66 | 5.03 |

75% is the sweet spot: crosses 2.0 S/A while maintaining variance >= 0.8. Binary locks (1.00) reach 2.74 but variance drops to 0.66, failing the target. The Round 2 discussion recommendation to use binary locks would sacrifice variance.

**Thresholds** (prob 0.75): (2,4,7) and (3,6,9) perform identically at S/A=2.28. Slower thresholds (5,10,15) drop to 2.03. Baseline 3/6/9 optimal.

**Split-resonance**: All-top=2.60 S/A, 2top_1sec=2.18, 1top_2sec=1.94. All-top is highest S/A but loses archetype disambiguation. The 2top_1sec split trades ~0.4 S/A for secondary-resonance targeting; still clears 2.0.

## Baseline Comparison

Compare to Agent 1 baselines when available. Expected: Soft Locks matches Lane Locking convergence (~pick 5-6) with better variance (0.88 vs LL's lower StdDev from deterministic locks). S/A of 2.20 likely trails LL's ~2.7 but with healthier pack-to-pack variation.

## Symbol Distribution

54 dual-type (15% cap), ~7 per archetype. Mono distribution: 20% 1-sym, 50% 2-sym, 30% 3-sym. Average ~2.7 weighted symbols per pick, reaching threshold 3 on pick 1-2 and threshold 9 by pick 3-4.

## Draft Traces

**Early committer:** Picks Ember from pick 1, all locks active by pick 5. 83% S/A at pick 12.
**Power chaser:** Picks by power, still converges to Flash (67% S/A) via Zephyr-locked slots.
**Signal reader:** Reads Ember as open, converges to Blink (83% S/A) through Ember locks.

## Self-Assessment (1-10)

| Goal | Score | Note |
|------|:-:|------|
| Simple | 6 | Three mechanisms stacked (thresholds + probability + split) |
| No actions | 10 | Fully automatic |
| Not on rails | 7 | 25% miss rate + random slot 4 |
| No forced decks | 8 | 4.6% overlap |
| Flexible | 6 | Split resonance allows some cross-archetype |
| Convergent | 8 | 2.20 S/A at pick 5.4 |
| Splashable | 6 | C/F=0.49, just misses 0.5 |
| Open early | 8 | 4.9 unique archetypes early |
| Signal reading | 5 | Auto-converges; moderate signal value |

**Strength:** Crosses 2.0 S/A (2.20) with genuine variance (0.88 StdDev). The 75% probability differentiates from Lane Locking's determinism.

**Weakness:** Deck concentration too high (97%) for committed players. Splash at 0.49 borderline. The algorithm drives so many on-resonance cards that committed players rarely pick off-archetype.
