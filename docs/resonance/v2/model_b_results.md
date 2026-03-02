# Model B Results: 10 Archetypes with Tight Card Pools

## Target Scorecard

| Metric | Target | Actual | Pass/Fail |
|--------|--------|--------|-----------|
| Picks 1-5: unique archetypes per pack | >= 3 of 4 | 3.11 | PASS |
| Picks 1-5: cards fitting emerging archetype per pack | <= 2 of 4 | 1.34 | PASS |
| Picks 6+: cards fitting committed archetype per pack | >= 2 of 4 | 2.34 | PASS |
| Picks 6+: strong off-archetype cards per pack | >= 0.5 of 4 | 1.41 | PASS |
| Convergence pick | Pick 5-8 | 8.4 | FAIL (marginal) |
| Deck archetype concentration (committed player) | 60-80% | 94.6% | FAIL |
| Run-to-run variety (card overlap) | < 40% | 6.6% | PASS |
| Archetype frequency across runs | No arch > 20% or < 5% | 5.4-14.0% | PASS |

**Score: 6/8 targets met.** Both failures are structurally linked to the high-N, high-overlap design.

## Multi-Archetype Card Sensitivity

| Multi-Arch % | Late Fit/Pack | Early Fit/Pack | Deck Conc % | Convergence Pick | Overlap % |
|--------------|---------------|----------------|-------------|------------------|-----------|
| 10% | 1.65 | 0.76 | 91.3 | 13.8 | 6.5 |
| 20% | 1.79 | 0.85 | 90.2 | 12.9 | 4.6 |
| 30% | 1.87 | 0.91 | 89.8 | 12.0 | 4.6 |
| 40% | 2.02 | 1.05 | 91.7 | 11.0 | 4.1 |
| 50% | 2.12 | 1.09 | 93.5 | 10.3 | 6.2 |
| 62% (baseline) | 2.28 | 1.25 | 94.0 | 8.6 | 5.6 |
| 75% | 2.43 | 1.31 | 94.6 | 8.2 | 6.7 |

**Key findings:**

1. **The late-fitting target (>= 2.0) requires ~40% multi-archetype cards at N=10.** Below 30%, convergence clearly fails. This confirms Q1's prediction that high-N systems demand extensive multi-archetype card design.

2. **Convergence pick is highly sensitive to multi-archetype %.** At 10%, convergence occurs at pick ~14. At 62%, around pick 8-9. Roughly linear in the 20-60% range with diminishing returns above.

3. **Convergence and concentration are inversely coupled.** The same overlap enabling convergence inflates concentration. No multi-archetype % simultaneously hits both targets at N=10. At 30% multi-arch, concentration drops to ~90% but convergence degrades to pick ~12.

4. **Run-to-run variety is excellent everywhere.** Overlap stays below 10% regardless of multi-archetype %, far exceeding the <40% target.

## Analysis

### What Works

**Early-draft diversity is excellent.** With 10 archetypes, packs consistently show 3+ archetypes (3.11 average), creating genuinely open-ended exploration.

**Run-to-run variety is outstanding.** At 6.6% overlap, far below the 40% target. The boosted/normal/suppressed archetype weighting creates C(10,3) = 120 possible configurations, so players rarely see the same landscape twice.

**Archetype balance is solid.** All 10 archetypes fall within 5.4-14.0% frequency. The per-run weighting prevents dominance.

**Off-archetype options are plentiful.** At 1.41 strong off-archetype cards per pack (target >= 0.5), players always have tempting alternatives.

### What Does Not Work

**Deck concentration is too high.** At 94.6%, committed players build near-mono-archetype decks. The high multi-archetype overlap needed for convergence means fitting cards appear in almost every pack. However, the power-chaser strategy lands at ~70% concentration (within target), suggesting real players who balance power against synergy would land closer to the target range.

**Convergence arrives slightly late.** At pick 8.4, just outside the 5-8 target. With 10 archetypes, early picks are informationally sparse -- each pack samples from 10 possibilities, making archetype identification slower. The signal-reader converges earlier (~7.9) by actively reading the landscape.

### The Fundamental Trade-off

Model B reveals a hard structural tension at N=10: convergence requires high card overlap between archetypes, but high overlap makes concentration exceed the target. The only escapes are (a) a slot-based pack construction that forces off-archetype cards, or (b) fewer archetypes so convergence does not require as much overlap. The model's strengths -- variety, balance, early diversity, splashability -- are genuine advantages of high N, but the convergence-concentration coupling is a real constraint that may require N=8 or lower to resolve cleanly.
