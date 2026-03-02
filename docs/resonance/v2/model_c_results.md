# Model C Results: Sub-Pool Carousel with Guaranteed Floors

## Target Scorecard (Committed Strategy, Default Distribution)

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Picks 1-5: unique archetypes per pack | >= 3 | 5.66 | PASS |
| Picks 1-5: cards fitting emerging archetype | <= 2 | 1.59 | PASS |
| Picks 6+: cards fitting committed archetype | >= 2 | 2.29 | PASS |
| Picks 6+: strong off-archetype per pack | >= 0.5 | 1.34 | PASS |
| Convergence pick | 5-8 | 3.0 | FAIL |
| Deck archetype concentration | 60-80% | 95.6% | FAIL |
| Run-to-run card overlap | < 40% | 7.2% | PASS |
| Archetype freq (no arch > 20%) | <= 20% | 20.1% | BORDERLINE |
| Archetype freq (no arch < 5%) | >= 5% | 9.4% | PASS |

**Overall: 6/9 passed, 1 borderline, 2 failed.**

The two failures (convergence too early, deck concentration too high) are related: the sub-pool system is *too effective* at feeding archetype cards. The anchor slot guarantees at least 1 fitting card per pack, and with 65% multi-archetype cards at the default distribution, secondary slots frequently also contain fitting cards. This causes convergence before pick 5 and pushes deck concentration above 80%.

## Multi-Archetype Card Sensitivity

The sensitivity sweep reveals how the system behaves across different multi-archetype percentages (MA% = fraction of cards that are S/A in 2+ archetypes):

| MA% | Early Uniq Archs | Early Fitting | Late Fitting | Late Off-Arch | Conv. Pick | Deck Conc. | Overlap | Arch Max |
|-----|-------------------|---------------|--------------|---------------|------------|------------|---------|----------|
| 10% | 4.0 | 0.90 | 1.78 | 2.11 | 5.4 | 88% | 6% | 23% |
| 20% | 4.5 | 1.09 | 1.93 | 1.91 | 3.5 | 91% | 6% | 20% |
| 30% | 4.9 | 1.23 | 2.04 | 1.71 | 3.1 | 93% | 7% | 18% |
| 40% | 5.1 | 1.33 | 2.09 | 1.60 | 3.1 | 94% | 6% | 18% |
| 50% | 5.2 | 1.42 | 2.15 | 1.53 | 3.0 | 94% | 6% | 21% |
| 60% | 5.5 | 1.50 | 2.23 | 1.43 | 3.0 | 95% | 6% | 18% |

Key findings:

**Convergence pick hits the 5-8 target only at 10% MA** (5.4). At 20%+, commitment detection fires too early because multi-archetype cards accidentally trigger commitment in multiple archetypes simultaneously.

**Late fitting crosses >= 2 around 30% MA.** Below that, the anchor slot provides ~1 guaranteed card but other slots do not reliably add a second. The sub-pool mechanism needs ~25-30% multi-archetype cards for the convergence target.

**Late fitting and off-archetype trade off directly.** At 10% MA, off-archetype is abundant (2.11) but fitting is scarce (1.78). At 60%, this reverses (2.23 fitting, 1.43 off-archetype). High overlap means splash/wild slots accidentally fit the committed archetype.

**Deck concentration is too high at all MA% levels** (88-95%). The committed strategy always picks fitting cards; the power-chaser shows more realistic 50-65% concentration.

**Run-to-run variety is excellent (6-7% overlap)** across all MA% -- the system's standout strength.

## Analysis: What Works and What Does Not

### What Works

**Variety and early exploration are excellent.** 5.66 unique archetypes per pack (target >= 3) and 7.2% run-to-run overlap (target < 40%) show the carousel and pool restriction create genuinely different experiences. Off-archetype temptation is robust at 1.34 per pack (target >= 0.5). Archetype frequency is balanced (max 20.1%, min 9.4%).

### What Does Not Work

**The system converges too fast** because commitment detection fires at pick 3. Fix: delay anchor-mode to pick 6 regardless, or require a clear lead (3+ S/A cards AND 2+ more than runner-up).

**Deck concentration is too high** (95.6%). Partly a measurement artifact -- the committed strategy always picks the best fitting card. Power-chaser shows 64.9%, within target. Fix: make the anchor slot probabilistic (80% committed sub-pool, 20% full pool).

### Tuning Recommendations

The optimal operating point is **20-30% multi-archetype cards** with adjusted commitment detection. The core sub-pool carousel mechanism is sound; the failures are in commitment detection sensitivity and deck concentration parameters, not the structural approach.
