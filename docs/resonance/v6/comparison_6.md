# Agent 6 Comparison: Ratcheting Slots

## Scorecard (1-10, all 7 algorithms x 9 design goals)

| Goal               | LL (1) | TAS (2) | SL (3) | PS (4) | DE (5) | RS (6) | SP (7) |
| ------------------ | :----: | :-----: | :----: | :----: | :----: | :----: | :----: |
| 1. Simple          |   9    |    7    |   5    |   4    |   7    |   8    |   8    |
| 2. No actions      |   10   |   10    |   10   |   10   |   10   |   10   |   10   |
| 3. Not on rails    |   3    |    7    |   6    |   8    |   7    |   4    |   8    |
| 4. No forced decks |   7    |    8    |   7    |   8    |   7    |   7    |   7    |
| 5. Flexible        |   3    |    6    |   5    |   7    |   5    |   4    |   7    |
| 6. Convergent      |   7    |    6    |   7    |   3    |   4    |   9    |   6    |
| 7. Splashable      |   4    |    8    |   5    |   3    |   7    |   7    |   8    |
| 8. Open early      |   6    |    8    |   7    |   8    |   7    |   7    |   8    |
| 9. Signal reading  |   3    |    5    |   5    |   5    |   5    |   3    |   5    |
| **Total**          | **52** | **65**  | **57** | **56** | **59** | **59** | **67** |

**Where I differ from the field:**

- RS convergent 9: Strongest reliable convergence in V6 at 2.20 S/A, pick 6.7,
  uniform per-archetype.
- LL convergent 7: 2.11 S/A at pick 3.3 is outside the 5-8 target. Converging
  too fast is a failure.
- TAS convergent 6: 2.01 S/A is statistically fragile. ~40% of drafts may see
  sub-2.0 packs.
- SP convergent 6: 2.08 mean is inflated; median post-commitment S/A is ~1.5.

## Biggest Strength and Weakness Per Strategy

| Algo    | Biggest Strength                                                                | Biggest Weakness                                                          |
| ------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| LL (1)  | 100% S/A locked slots: the mathematical foundation all lock algorithms build on | Pick 3.3 convergence and 0.49 stddev: too fast, too smooth                |
| TAS (2) | Perfect 9/9 metric passage; best generalist algorithm                           | 2.01 S/A with no headroom; one config change drops below 2.0              |
| SL (3)  | Only dual-target achievement: 2.0+ S/A and 0.8+ stddev simultaneously           | 97% concentration and 0.49 splash: committed players get no variety       |
| PS (4)  | Best deck concentration (77.9%) and most organic variance                       | Two hard failures (splash 0.36, convergence pick 8.6); structural ceiling |
| DE (5)  | Highest raw variance (1.55 stddev); most creative trigger mechanic              | No viable parameter point: the thresh=1/thresh=2 dilemma is unsolvable    |
| RS (6)  | Strongest S/A (2.20) in the correct convergence window (pick 6.7)               | Variance 0.69 fails the 0.8 target; 75% locked pack feels deterministic   |
| SP (7)  | Only non-permanent mechanism; best player experience and flexibility            | 25% zero-S/A packs create frustrating dry spells between surges           |

## Proposed Improvements

- **LL (1):** Raise both thresholds by 2 (threshold 5/10). This delays
  convergence to pick 5-6 and slightly improves variance. Still insufficient for
  the 0.8 stddev target.
- **TAS (2):** Switch to C3B2 gated at pick 3. Higher S/A (2.6+) provides
  genuine headroom. Accept concentration increase.
- **SL (3):** The 75% probability is the right choice for variance but splash is
  the weak link. Agent 3's forced-splash slot 4 idea would fix this cleanly.
- **PS (4):** Cannot stand alone. Best future: supplementary layer under another
  algorithm.
- **DE (5):** Switch permanently to thresh=1/bonus=2 and rebrand as "Resonance
  Echo" -- the pack echoes your top resonance when it naturally contains a
  match. At 63% fire rate, this IS the algorithm. Stop calling it "conditional."
- **RS (6) -- self-improvement:** The variance problem is my Achilles heel. Two
  paths:
  - **Path A:** Use 2/4/7 thresholds (stddev 0.81, S/A 2.34) -- nearly meets
    variance target.
  - **Path B:** Make the third lock probabilistic (75%) while keeping first two
    hard. Projected: 2.10 S/A, ~0.80 stddev. A Ratcheting-Soft Lock hybrid.
- **SP (7):** The zero-S/A valley problem is the dealbreaker. Hybrid fix: when
  NOT surging, guarantee 1 slot shows top-resonance card (like a permanent soft
  lock at 100%). This converts non-surge packs from ~1.0 to ~1.75 S/A, smoothing
  the distribution.

## Baseline Comparison

**Does any V6 algorithm clearly beat both baselines?**

Against Lane Locking: Ratcheting Split matches LL's convergence power (2.20 vs
2.11) with significantly better splash (1.31 vs 0.64) and convergence timing
(pick 6.7 vs 3.3). I believe RS is a strict improvement over LL.

Against Pack Widening: No zero-decision algorithm matches 3.35 S/A. The best V6
algorithms cluster at 2.0-2.2. This confirms V4's finding: player spending
decisions provide ~1.0-1.3 additional S/A.

**My claim:** Ratcheting Split is the best convergence-focused algorithm in V6.
TAS and SP are better on secondary metrics but weaker on the primary convergence
target.

## Proposed Best Algorithm

**Ratcheting Slots at 2/5/9 with split-resonance third lock.**

"When your top resonance count reaches 2, 5, and 9, lock one more pack slot: the
first two lock to your top resonance, the third locks to your second-highest;
the fourth slot stays random."

Projected metrics: S/A ~2.30, stddev ~0.80, convergence pick ~5.5. The 2/5/9
spacing creates a three-act structure: first lock (immediate feedback), second
(commitment confirmed), third (specialization). Deterministic placement sits
atop the convergence hierarchy.

## 15% Dual-Resonance Constraint Impact

The constraint made the problem **neutral to slightly easier** for slot-locking
approaches. The critical discovery: mono-resonance primary pools deliver 100%
S/A because adjacent archetypes sharing a primary resonance are mutually S/A.
This means slot-locking algorithms work BETTER than predicted because the 50%
dilution fear was wrong.

Dual-type cards serve player-facing archetype signals (a [Tide, Zephyr] card
communicates "Warriors"), but the algorithm does not need them mechanically. My
simulation tested dual counts from 0 to 54 and found S/A ranging only 2.19-2.33
-- confirming the algorithm is robust to this parameter.

At 10%: No algorithm impact. Fewer player-readable archetype signals. At 20%:
Marginal algorithm improvement (+0.05 S/A from better archetype targeting in the
third lock's secondary-resonance pool). The real benefit is player-facing: more
dual-type cards help players understand which archetype they are drafting.

The constraint's primary impact is on player experience, not algorithm
performance. I recommend 15% as a good balance between archetype signals and
preventing pair-matching dominance.
