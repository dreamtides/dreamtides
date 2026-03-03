# Results 4: Hybrid Y -- Escalating Open Table (D1 + D4)

## Full Scorecard (Graduated Realistic, Committed Player)

| Metric | Value | Target | Status |
|--------|------:|--------|--------|
| M1 | 3.11 | >= 3 | PASS |
| M2 | 0.86 | <= 2 | PASS |
| M3 | 0.48 | >= 2.0 | **FAIL** |
| M4 | 3.52 | >= 0.5 | PASS |
| M5 | 14.3 | 5-8 | **FAIL** |
| M6 | 0.36 | 0.60-0.90 | **FAIL** |
| M7 | 0.055 | < 0.40 | PASS |
| M9 | 0.68 | >= 0.8 | **FAIL** |
| M10 | 18.50 | <= 2 | **FAIL** |
| M11 | 0.19 | >= 3.0 | **FAIL** |

**Result: 4/10 pass. Catastrophic failure on M3, M10, M11.**

### M3/M10/M11 Pessimistic

| Metric | Graduated | Pessimistic |
|--------|----------:|------------:|
| M3 | 0.48 | 0.44 |
| M10 | 18.50 | 19.06 |
| M11 | 0.19 | 0.19 |

Both fitness models fail identically. The bottleneck is pool exhaustion, not fitness rates.

## Per-Archetype M3

| Archetype | M3 | M10 | M11 | M6 |
|-----------|---:|----:|----:|---:|
| Flash | 0.44 | 18.81 | 0.17 | 0.34 |
| Blink | 0.42 | 19.39 | 0.15 | 0.32 |
| Storm | 0.45 | 18.90 | 0.19 | 0.35 |
| Self-Discard | 0.55 | 17.42 | 0.29 | 0.39 |
| Self-Mill | 0.54 | 18.06 | 0.24 | 0.39 |
| Sacrifice | 0.51 | 18.17 | 0.20 | 0.38 |
| Warriors | 0.53 | 17.89 | 0.22 | 0.38 |
| Ramp | 0.40 | 19.75 | 0.15 | 0.32 |

Worst: Ramp (M3=0.40). All archetypes fail catastrophically.

## Pack Quality Distribution (Picks 6+)

| P10 | P25 | P50 | P75 | P90 |
|----:|----:|----:|----:|----:|
| 0 | 0 | 0 | 1 | 2 |

Median pack has zero S/A cards. Average max consecutive bad packs: **18.5** (worst: 25).

## AI Behavior Summary

- AI removed per draft: **212** (59% of pool)
- Market cull removed: **104** (29% of pool)
- Total removed: **316** (88% of pool)
- Pool at pick 15: ~37 cards. Pool at pick 30: ~14 cards.

The player's archetype concentration *decreases* from 12% to near 0% over the draft -- the opposite of V9's behavior. AIs' 15% power picks and market culling erode open-lane card pools alongside contested ones.

## Open Lane vs Contested Lane

| Lane | Count | M3 | M11 | M10 |
|------|------:|---:|----:|----:|
| Open | 511 | 0.71 | 0.37 | 15.0 |
| Contested | 489 | 0.23 | 0.01 | 22.1 |

The signal-reading incentive exists (3x M3 difference), but both outcomes are far below targets.

## Draft Traces

**Trace 1: Committed Warriors (Open Lane).** AIs: Flash, Blink, Storm, Self-Mill, Sacrifice. Picks 6-15 produce 1.5-2.0 S/A per pack as Warriors concentration rises to 47%. After pick 15, pool hits floor (29 cards), AI removal stops. Picks 16-24 drain remaining Warriors cards. Picks 25-30: 0 Warriors left, all generics. Final: 21/30 S/A (70%).

**Trace 2: Signal Reader Warriors (Open Lane).** AIs: Flash, Storm, Self-Discard, Sacrifice, Ramp. Player mis-committed to Stone resonance early; system inferred Self-Mill instead of Warriors. After pick 19, Tide signal overtook Stone, but pool had only 1-2 Warriors cards. Final: 11/30 S/A (37%).

## V9 Comparison

| Metric | V9 Hybrid B | Hybrid Y | Delta |
|--------|------------:|---------:|------:|
| M3 | 2.70 | 0.48 | -2.22 |
| M10 | 3.8 | 18.50 | +14.70 |
| M11 | 3.25 | 0.19 | -3.06 |

## Self-Assessment

**Hybrid Y fails structurally, not from tuning.** V9 keeps the full 360-card pool and uses *virtual* contraction (biased sampling). Hybrid Y physically removes 88% of cards, leaving a pool too small for meaningful pack construction. Even with affinity-weighted packs, 30 surviving cards with 5-8 matching archetype cards cannot produce 2+ S/A per pack.

**Key question answered:** "Is 5-AI escalation with 3 open lanes better than 7-AI escalation with 1 open lane?" Both are far worse than V9. The open-lane vs contraction-intensity tradeoff is irrelevant when physical pool depletion prevents either from achieving V9-level convergence. Signal clarity matters (3x M3 open vs contested), but the best open-lane result (M3=0.71) is still 73% below V9.

**What would fix it:** Replace physical depletion with virtual contraction -- keep AI drafters as narrative but don't remove cards from the pool. Track what AIs "would take" and bias pack construction to avoid those cards. This preserves the AI drafter narrative while using V9's proven mathematical engine.
