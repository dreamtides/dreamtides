# Agent 6 Results: Ratcheting Slot Commitment

## One-Sentence Algorithm

"When your top resonance count reaches 3, 6, and 10, lock one more pack slot:
the first two lock to your top resonance, the third locks to your
second-highest; the fourth slot stays random."

## Scorecard (baseline 3/6/10 split)

| Metric                           | Target     | Result    | Pass     |
| -------------------------------- | ---------- | --------- | -------- |
| Picks 1-5: unique archs with S/A | >= 3       | 4.96      | YES      |
| Picks 1-5: S/A for emerging arch | \<= 2      | 1.77      | YES      |
| Picks 6+: S/A per pack           | >= 2.0     | 2.20      | YES      |
| Picks 6+: C/F per pack           | >= 0.5     | 1.31      | YES      |
| Convergence pick                 | 5-8        | 6.7       | YES      |
| Deck concentration               | 60-90%     | 83.2%     | YES      |
| Card overlap                     | < 40%      | 18.4%     | YES      |
| Arch frequency                   | 5-20% each | 5.9-17.0% | MARGINAL |
| S/A stddev                       | >= 0.8     | 0.69      | NO       |

## Variance Report

Pack quality (picks 6+): 0 S/A 0.3%, 1 S/A 12.3%, 2 S/A 58.1%, 3 S/A 26.4%, 4
S/A 3.0%. StdDev 0.69 misses the 0.8 target. The aggressive 2/5/9 split variant
reaches 0.79, nearly meeting it. More locks inherently reduce variance.

## Per-Archetype Convergence

| Archetype    | Mean Conv | Median | Count |
| ------------ | --------- | ------ | ----- |
| Flash        | 6.5       | 3.0    | 321   |
| Blink        | 6.7       | 4.0    | 450   |
| Storm        | 6.4       | 3.0    | 510   |
| Self-Discard | 6.9       | 4.0    | 425   |
| Self-Mill    | 7.3       | 3.0    | 375   |
| Sacrifice    | 6.2       | 3.0    | 361   |
| Warriors     | 6.9       | 4.0    | 382   |
| Ramp         | 6.1       | 3.0    | 176   |

All archetypes converge within pick 5-8. Ramp underrepresented at 5.9%.

## Baseline Comparison (compare to Agent 1)

| Metric   | Lane Locking (3,8) | Ratchet Split (3,6,10) | Ratchet Unsplit |
| -------- | ------------------ | ---------------------- | --------------- |
| Late S/A | 2.25               | 2.20                   | 3.00            |
| Conv     | 6.0                | 6.7                    | 4.2             |
| Deck %   | 82.9%              | 83.2%                  | 88.2%           |
| StdDev   | 0.73               | 0.69                   | 0.64            |
| C/F      | 1.27               | 1.31                   | 0.73            |

Split performs comparably to Lane Locking with slightly lower S/A but better
splash. Unsplit dramatically outperforms (3.00 S/A) but locks 75% of the pack,
failing "not on rails."

## Symbol Distribution

10% dual-type (36 cards). Dual-count sensitivity: S/A ranges 2.19-2.33 across
0-54 dual cards, confirming the algorithm does not rely on dual-type signals.

## Parameter Sensitivity

| Thresholds | Late S/A | Conv | StdDev |
| ---------- | -------- | ---- | ------ |
| 1/3/6      | 2.54     | 4.5  | 0.81   |
| 2/4/7      | 2.34     | 6.9  | 0.81   |
| 3/6/10     | 2.20     | 6.4  | 0.69   |
| 4/8/14     | 2.19     | 6.0  | 0.60   |
| 6/12/20    | 2.14     | 7.2  | 0.62   |

Lower thresholds produce higher S/A and variance but faster commitment. 2/4/7
achieves 2.34 S/A with stddev 0.81 (meeting variance target).

## Draft Traces

**Early committer:** Triple-Ember pick 1 (4 weighted) locked slot 1 immediately.
All 3 locks by pick 3. Saw 3-4 S/A from pick 4 onward.

**Power chaser:** 2 locks by pick 2. Deck concentration only 60% -- algorithm
provides convergence floor even for non-optimizing players.

**Signal reader:** Locked Ember pick 1, Stone pick 3 (secondary). Third lock
went to Stone at pick 4. Storm archetype emerged naturally.

## Self-Assessment

| Goal            | Score | Notes                                          |
| --------------- | ----- | ---------------------------------------------- |
| Simple          | 8     | Three thresholds + split adds minor complexity |
| No actions      | 10    | Fully automatic                                |
| Not on rails    | 4     | 3/4 slots locked mid-draft                     |
| No forced decks | 6     | Good overlap; marginal balance                 |
| Flexible        | 4     | Hard locks resist pivoting                     |
| Convergent      | 8     | 2.20 S/A exceeds 2.0                           |
| Splashable      | 7     | 1.31 C/F; split lock helps                     |
| Open early      | 7     | 4.96 unique archs early                        |
| Signal reading  | 3     | No signal-reading benefit                      |

## Key Finding

Ratcheting crosses 2.0 S/A (2.20) and is fully reconstructible from one
sentence. The split third lock trades ~0.8 S/A versus unsplit (2.20 vs 3.00) for
better splash. Core weakness: identical to Lane Locking -- permanent locks
create a deterministic feel. The 3-threshold structure is smoother than Lane
Locking's 2-threshold but does not fundamentally solve the railroading problem.
If variance is prioritized, use 2/4/7 thresholds (stddev 0.81).
