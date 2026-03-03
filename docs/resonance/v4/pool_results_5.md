# Pool Results 5: Algorithm Parameter Tuning

## Key Finding: Bonus Cards Are the Master Lever

The single most important finding is that **bonus card count matters more than spend cost or primary weight**. Bonus=2 produces avg late S/A of 2.14 vs 1.52 for bonus=1. With bonus=1, no configuration reaches the 2.0 S/A target at the archetype level. With bonus=2, many configurations hit or exceed it. This makes sense: bonus=1 adds one filtered card to a 4-card random pack, giving the filtered card only ~20% influence. Bonus=2 means 2 of 6 cards are filtered (~33%), which is the threshold needed for reliable archetype convergence.

**Recommendation: Bonus cards = 2 is mandatory for Pack Widening v3 to work.**

## Spend Cost x Distribution: The Critical Interaction

Crossing 4 spend costs with 3 distributions reveals a clear pattern:

| Cost | Heavy 1-sym (50/35/15) | Default (20/55/25) | Heavy 3-sym (10/30/60) |
|------|----------------------|-------------------|----------------------|
| 2 | Spend every pick (0.93), degenerate | Spend every pick (0.98) | Always spend (1.00) |
| 3 | Spend 78% of picks | Spend 87% of picks | Spend 93%, degenerate |
| 4 | Spend 62% of picks | Spend 72% of picks | Spend 80% of picks |
| 5 | Spend 49% of picks | Spend 57% of picks | Spend 66% of picks |

The ideal spend frequency is 50-70%: frequent enough to matter, rare enough that the decision is non-trivial. Cost 2 produces universal always-spend across all distributions. Cost 5 + heavy 1-symbol drops spend rate below 50%, making the mechanism too infrequent.

**Best combinations for spend rhythm:**
- Cost 5, heavy 1-sym: 49% spend rate (interesting decisions, but slow convergence)
- Cost 4, default: 72% spend rate (slightly too frequent, but good convergence)
- Cost 3, heavy 1-sym: 78% spend rate (approaching auto-spend)

## Always-Spend Degeneracy

Across all 72 configurations, "always spend" is rarely strictly dominant -- the committed strategy (which prefers the archetype's primary resonance) usually matches or beats always-spend by 0.2-1.5 percentage points in deck S/A%. The save-then-spend strategy consistently performs worst (0.5-2.0% lower deck S/A%). This means **the system rewards spending but not mindless spending** -- choosing which resonance to spend on matters, but there is little benefit to hoarding tokens.

The degeneracy risk is not "always spend vs save" but rather "always spend on primary resonance." With high-weight, low-cost parameters (W3/C2), the player accumulates tokens so fast that they spend on primary every single pick, eliminating the decision entirely.

## Three-Act Draft Arc

The two EXCELLENT-rated configurations both achieve a clear three-act structure:

**C5 B2 W3 D50/35/15** (recommended):
- Act 1 (picks 1-5): Pure exploration, first spend at pick 3.7, S/A avg 1.5
- Act 2 (picks 6-15): Commitment, S/A rises to 2.0, 65% meaningful choices
- Act 3 (picks 16-30): Steady refinement, S/A holds at 2.0, 65% meaningful choices

**C2 B2 W1 D50/35/15** (alternative):
- Act 1 (picks 1-5): Exploration, first spend at pick 3.1, S/A avg 1.5
- Act 2 (picks 6-15): Commitment, S/A rises to 2.0, 63% meaningful choices
- Act 3 (picks 16-30): Refinement, S/A holds at 2.0, 64% meaningful choices

Both achieve nearly identical outcomes through different paths: C5/W3 uses high cost offset by fast token earning; C2/W1 uses low cost but slow token earning. The C5/W3 path is preferable because it creates more decision tension per spend (spending 5 tokens is a bigger commitment than spending 2).

## Measurable Targets

The top two configs pass 8-9 of 10 targets. The most common failure is convergence pick at 4 instead of 5-8 (with bonus=2, convergence happens early). The C2/B2/W1/D50 config uniquely achieves convergence at exactly pick 8, the upper ideal bound.

| Metric | C5/B2/W3/D50 | C2/B2/W1/D50 | Target |
|--------|-------------|-------------|--------|
| Early unique archs | 5.34 | 5.41 | >= 3 |
| Early S/A emerging | 1.47 | 1.54 | <= 2 |
| Late S/A committed | 2.00 | 1.97 | >= 2 |
| Late C/F off-arch | 1.39 | 1.41 | >= 0.5 |
| Convergence pick | 4 | 8 | 5-8 |
| Deck S/A% | 82% | 82% | 60-90% |
| Card overlap | 4.3% | 4.0% | < 40% |
| S/A stddev late | 1.30 | 1.30 | >= 0.8 |

## Primary Weight Impact

Weight controls how fast tokens accumulate and thus when spending begins:

| Weight | First Spend | Spend Freq | Late S/A | Degeneracy Rate |
|--------|------------|-----------|---------|----------------|
| W1 | pick 4.4 | 60% | 1.46 | 42% |
| W2 | pick 3.2 | 81% | 1.87 | 58% |
| W3 | pick 2.7 | 93% | 2.16 | 58% |

W1 produces the most interesting spend decisions but lowest convergence. W3 converges faster but risks auto-spend. The choice of weight must be paired with cost to hit the target spend frequency.

## Recommended Parameter Sets

**Primary recommendation: Cost 5, Bonus 2, Weight 3, Heavy 1-symbol (50/35/15)**
- Spend frequency 69%, first spend pick 3.7, late S/A 2.00
- Not always-spend-dominant, S/A stddev 1.30
- EXCELLENT arc rating, 8/10 targets passed

**Alternative: Cost 2, Bonus 2, Weight 1, Heavy 1-symbol (50/35/15)**
- Identical outcome through inverse parameter path
- Convergence at pick 8 (within ideal range)
- 9/10 targets passed

**Higher convergence option: Cost 3, Bonus 2, Weight 2, Heavy 1-symbol (50/35/15)**
- Late S/A 2.20, spend frequency 84%
- Slightly less decision tension but stronger archetype signal

## Synthesis for Other Agents

- Symbol distribution matters primarily through its effect on token accumulation rate. Heavy 1-symbol (50/35/15) consistently produces the best spend rhythm by slowing token earning.
- Bonus=2 is non-negotiable for archetype-level convergence.
- The ratio spend_cost / (avg_tokens_per_pick) controls the draft arc. Target a ratio that produces 50-70% spend frequency at picks 6+.
- With bonus=2, all configurations produce excellent run-to-run variety (~4% overlap) and perfect archetype balance.
