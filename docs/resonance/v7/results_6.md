# V7 Agent 6 Results: Dual-Counter Surge

## One-Sentence Description

Each drafted symbol adds resonance tokens (+2 primary, +1 others); maintain running average cost of drafted cards; when any resonance counter reaches 4, spend 4 and fill 3 surge slots with cards matching that resonance AND within +/-1 of the player's average cost (widening to +/-2, then unfiltered if insufficient), fourth slot random.

## Scorecard

### Fitness Model A (Optimistic): 9/9 PASS

| Metric | Target | Result | Status |
|--------|--------|--------|--------|
| M1: Unique archs early | >= 3 | 4.48 | PASS |
| M2: S/A early | <= 2 | 1.91 | PASS |
| M3: S/A late | >= 2.0 | 2.02 | PASS |
| M4: Off-arch late | >= 0.5 | 0.71 | PASS |
| M5: Convergence | 5-8 | 6.5 | PASS |
| M6: Deck concentration | 60-90% | 75.5% | PASS |
| M7: Card overlap | < 40% | 27.2% | PASS |
| M8: Arch freq | 5-20% | 7.0%-19.9% | PASS |
| M9: S/A stddev | >= 0.8 | 1.43 | PASS |

### Fitness Model B (Moderate: 50%A/30%B/20%C sibling): 7/9 PASS

| Metric | Target | Result | Status |
|--------|--------|--------|--------|
| M1: Unique archs early | >= 3 | 3.63 | PASS |
| M2: S/A early | <= 2 | 1.45 | PASS |
| M3: S/A late | >= 2.0 | **1.41** | **FAIL** |
| M4: Off-arch late | >= 0.5 | 0.92 | PASS |
| M5: Convergence | 5-8 | **11.1** | **FAIL** |
| M6: Deck concentration | 60-90% | 65.7% | PASS |
| M7: Card overlap | < 40% | 25.8% | PASS |
| M8: Arch freq | 5-20% | 8.0%-17.8% | PASS |
| M9: S/A stddev | >= 0.8 | 1.23 | PASS |

### Fitness Model C (Pessimistic: 25%A/40%B/35%C sibling): 6/9 PASS

| Metric | Target | Result | Status |
|--------|--------|--------|--------|
| M1: Unique archs early | >= 3 | 3.22 | PASS |
| M2: S/A early | <= 2 | 1.26 | PASS |
| M3: S/A late | >= 2.0 | **1.13** | **FAIL** |
| M4: Off-arch late | >= 0.5 | 1.07 | PASS |
| M5: Convergence | 5-8 | **16.4** | **FAIL** |
| M6: Deck concentration | 60-90% | **58.9%** | **FAIL** |
| M7: Card overlap | < 40% | 25.1% | PASS |
| M8: Arch freq | 5-20% | 7.6%-19.1% | PASS |
| M9: S/A stddev | >= 0.8 | 1.14 | PASS |

## Cost Filter Effectiveness Analysis

The core hypothesis was that filtering surge slots by the player's average cost would disambiguate between sibling archetypes (same primary resonance, different cost profiles). Results:

| Model | Home Archetype | Sibling Archetype | Home Selection Rate |
|-------|---------------|-------------------|---------------------|
| Moderate | 38.4% | 33.5% | 38.4% |
| Pessimistic | 36.6% | 30.8% | 36.6% |
| Expected (unfiltered) | ~50% | ~50% | ~50% |

The cost filter shifts home-archetype selection from the expected ~50% to 38.4% under moderate fitness. This is a modest improvement -- the filter is working directionally (home archetype is favored over sibling) but the disambiguation effect is weak. The home:sibling ratio is approximately 53:47 within the resonance-matched portion, compared to the expected 50:50 without filtering. This +3% home-archetype precision translates to the +0.05 S/A advantage over Surge Packs seen in the headline numbers.

The secondary-sharing category (24.2% of surge slots) reveals a problem: the cost filter also draws in cards from non-sibling archetypes that happen to share the cost band. These cards are only B-tier, diluting the benefit.

**Verdict: cost filtering provides marginal disambiguation (+3-5% home selection improvement) that does not justify the added complexity over standard Surge Packs.**

## Fitness Degradation Curve

| Metric | Optimistic | Moderate | Pessimistic | Delta |
|--------|-----------|----------|-------------|-------|
| M3 S/A Late (DCS) | 2.02 | 1.41 | 1.13 | -0.89 |
| M3 S/A Late (SP) | 2.03 | 1.36 | 1.10 | -0.93 |
| M5 Convergence (DCS) | 6.5 | 11.1 | 16.4 | +9.9 |
| M5 Convergence (SP) | 6.0 | 11.2 | 17.0 | +11.0 |
| M6 Deck Conc (DCS) | 75.5% | 65.7% | 58.9% | -16.6 |
| M6 Deck Conc (SP) | 75.9% | 65.0% | 59.1% | -16.8 |

DCS degrades slightly less than Surge Packs on M3 (0.89 drop vs 0.93 drop) and M5 (9.9 picks vs 11.0 picks of convergence delay). The cost filter provides a small cushion under realistic fitness, reducing the S/A penalty by approximately 0.03-0.05 per pack. This is consistent across both moderate and pessimistic models but is too small to change pass/fail outcomes on any metric.

## Per-Archetype Convergence (Moderate Fitness)

| Archetype | Mean Pick | Median Pick | Count |
|-----------|----------|-------------|-------|
| Flash | 10.6 | 8.0 | 300 |
| Blink | 10.4 | 7.0 | 172 |
| Storm | 9.5 | 7.0 | 296 |
| Self-Discard | 11.3 | 8.0 | 324 |
| Self-Mill | 15.0 | 13.0 | 194 |
| Sacrifice | 10.0 | 7.0 | 355 |
| Warriors | 10.6 | 8.0 | 159 |
| Ramp | 13.2 | 10.0 | 200 |

Self-Mill and Ramp converge latest (mean 15.0 and 13.2), consistent with their higher cost profiles pushing them toward smaller cost-filtered candidate pools. Flash and Sacrifice converge fastest, reflecting their dominant resonance types and moderate cost bands.

## Parameter Sensitivity (Moderate Fitness)

| Parameters | M3 S/A | M5 Conv | M6 Deck% | M9 StdDev | Pass |
|-----------|--------|---------|----------|-----------|------|
| **T=3, W=1.0** | **1.88** | **5.7** | **77.6%** | 1.21 | **8/9** |
| T=3, W=1.5 | 1.88 | 5.3 | 77.8% | 1.21 | 8/9 |
| T=3, W=2.0 | 1.84 | 6.0 | 77.0% | 1.21 | 7/9 |
| T=4, W=1.0 | 1.41 | 11.1 | 65.7% | 1.23 | 7/9 |
| T=4, W=1.5 | 1.41 | 10.9 | 66.2% | 1.23 | 7/9 |
| T=4, W=2.0 | 1.37 | 11.6 | 65.2% | 1.22 | 6/9 |
| T=5, W=1.0 | 1.20 | 19.3 | 61.2% | 1.18 | 7/9 |
| T=5, W=1.5 | 1.20 | 18.7 | 61.7% | 1.17 | 7/9 |
| T=5, W=2.0 | 1.20 | 19.4 | 61.6% | 1.17 | 7/9 |

**Threshold dominates; cost window is nearly irrelevant.** T=3 is the only threshold that reaches 8/9 under moderate fitness (1.88 S/A, convergence 5.7). The cost window (+/-1 vs +/-1.5 vs +/-2) changes M3 by at most 0.04, confirming that cost filtering provides negligible disambiguation compared to the threshold parameter.

At T=3, the algorithm surges more frequently (every ~2 picks instead of ~3), which is what actually drives the S/A improvement. This is equivalent to lowering the threshold in standard Surge Packs -- the cost filter is not the lever that matters.

## Draft Traces (Moderate Fitness, T=4)

**Trace 1 -- Early Committer (Warriors):** Committed to Tide-primary by pick 3. Drafted heavily in Tide cards at cost 3-4. Surges fired frequently but cost filter at avg 3.2 pulled in both Warriors (cost 3-4) and Sacrifice (cost 2-3) cards. Convergence occurred around pick 5 with 2-3 S/A per surge pack.

**Trace 2 -- Signal Reader (Flash):** Built Zephyr signal early with low-cost cards (avg 2.0-2.8). Surges pulled Zephyr cards filtered to cost 1-4, favoring Flash-home (cost 2-3) over Ramp-home (cost 4-5). This is the best case for cost filtering -- clear cost separation between Flash (2.5 avg) and Ramp (4.8 avg).

**Trace 3 -- Power Chaser:** No committed archetype assignment in trace output. Power chasers select high-power cards regardless of archetype, leading to scattered resonance accumulation. Surges fire but cost filter oscillates with the unstable average cost, providing no disambiguation value.

## Self-Assessment: Does Cost Filtering Justify Added Complexity?

**No.** The simulation provides clear evidence that cost-based disambiguation is insufficient to justify the added complexity over standard Surge Packs:

1. **Marginal S/A gain:** DCS achieves 1.41 vs Surge Packs' 1.36 under moderate fitness (+0.05), and 1.13 vs 1.10 under pessimistic (+0.03). These differences are within noise for practical purposes.

2. **Cost window has near-zero impact:** Varying the cost window from +/-1 to +/-2 changes M3 by <0.04 across all thresholds. The cost dimension carries almost no signal above what resonance already provides.

3. **Home selection rate improvement is weak:** Cost filtering shifts home-archetype selection from ~50% to ~53% within resonance-matched slots. A 3% improvement is not perceptible to players.

4. **Threshold is the real lever:** T=3 with any cost window scores 1.88 S/A under moderate fitness. Standard Surge Packs at T=3 (without cost filtering) would achieve a similar result. The improvement comes from surge frequency, not cost precision.

5. **Complexity cost is real:** Players must mentally track "average cost of my deck" in addition to resonance counters, even though this tracking produces negligible benefit. The one-sentence description is longer and harder to parse than standard Surge Packs.

**Recommendation:** Abandon cost filtering as a disambiguation strategy. The fundamental problem is that archetype cost profiles overlap too much (Sacrifice at 3.0 vs Warriors at 3.7 = only 0.7 energy separation), and the running average is too noisy to exploit even well-separated pairs (Flash at 2.5 vs Ramp at 4.8). Standard Surge Packs with a lower threshold (T=3) would achieve the same or better results with simpler rules. Disambiguation likely requires structural signals (dual-resonance pair matching, secondary resonance tracking) rather than statistical signals like cost.
