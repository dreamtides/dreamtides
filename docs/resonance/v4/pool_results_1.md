# Agent 1 Results: Symbol Count Distribution

## Investigation

What ratio of 1/2/3-symbol cards (among the ~90% non-generic pool) produces the
best draft experience with Pack Widening v3? All metrics are at the archetype
level across 2000 drafts per configuration, 30 picks each. Generic cards are
fixed at 10% (36 of 360) for all configurations.

## Token Economy

| Distribution | Tok/Pick | PrimTok/Pick | SpendFreq (6+) | MaxNoSpendStreak |
|---|---|---|---|---|
| All 1-sym (100/0/0) | 1.94 | 1.56 | 53.9% | 3.6 |
| Heavy 1-sym (70/20/10) | 2.36 | 1.64 | 57.3% | 3.0 |
| Moderate 1-sym (50/35/15) | 2.62 | 1.70 | 59.5% | 2.6 |
| Balanced (33/34/33) | 2.97 | 1.76 | 62.1% | 2.3 |
| V4 Default (20/55/25) | 3.02 | 1.77 | 62.6% | 2.1 |
| Heavy 2-sym (10/80/10) | 2.97 | 1.76 | 62.3% | 2.0 |
| Heavy 3-sym (10/30/60) | 3.47 | 1.84 | 65.8% | 2.0 |
| All 3-sym (0/0/100) | 3.96 | 1.91 | 69.2% | 1.8 |

The primary token rate is surprisingly stable (1.56-1.91) across all
distributions because only the primary symbol earns 2 tokens, and all cards
have the same primary. The *total* token rate varies more (1.94-3.96) as
secondary/tertiary symbols feed off-resonances. This means symbol count mostly
controls how many tokens go to *non-primary* resonances, with modest impact on
primary spend frequency.

Spend frequency ranges from 54% (all 1-sym) to 69% (all 3-sym). The save/spend
rhythm in pick counts:

| Distribution | Spend Picks | Save Picks | Ratio |
|---|---|---|---|
| All 1-sym | 13.5 | 11.5 | 13:12 |
| Heavy 1-sym | 14.3 | 10.7 | 14:11 |
| V4 Default | 15.6 | 9.4 | 16:9 |
| All 3-sym | 17.3 | 7.7 | 17:8 |

No configuration reaches "always-spend" degeneracy (0% of drafts spend every
single post-commitment pick). The save/spend decision exists everywhere, but its
frequency varies. All 1-sym creates the most balanced rhythm with nearly equal
spend and save picks.

## Convergence Curve

| Distribution | P6 | P8 | P10 | P15 | P20 | P25 | P30 |
|---|---|---|---|---|---|---|---|
| All 1-sym | 2.10 | 1.82 | 1.88 | 1.85 | 1.86 | 1.86 | 1.85 |
| Heavy 1-sym | 2.20 | 1.93 | 1.92 | 1.91 | 1.91 | 1.89 | 1.92 |
| V4 Default | 2.35 | 2.20 | 1.95 | 1.96 | 1.94 | 1.94 | 1.97 |
| Heavy 3-sym | 2.35 | 2.23 | 1.94 | 1.89 | 1.93 | 1.91 | 1.98 |
| All 3-sym | 2.36 | 2.27 | 1.90 | 1.96 | 2.03 | 1.94 | 2.02 |

All configurations show a spike at pick 6 (first post-commitment pack, often a
spend pack with accumulated tokens) that settles to 1.85-2.04 by pick 10. The
long-run average (SA 6+ mean) ranges from 1.90 to 2.04 -- a narrow band. The
trend slope is slightly negative for all configs (-0.004 to -0.011), meaning
pack quality does not improve over time but holds roughly steady with a gentle
decline from the initial spike.

## Spend vs Non-Spend Pack Quality

| Distribution | SA (Spend) | SA (No Spend) | Gap |
|---|---|---|---|
| All 1-sym | 2.36 | 1.35 | 1.01 |
| V4 Default | 2.36 | 1.36 | 1.00 |
| All 3-sym | 2.35 | 1.35 | 1.00 |

The gap is remarkably stable at ~1.0 S/A cards regardless of symbol
distribution. Spend packs average 2.36 S/A cards (good convergence); non-spend
packs average 1.35 (adequate but random). This gap is driven entirely by the
bonus card mechanic and is independent of symbol count.

## Variance

All distributions produce SA 6+ standard deviation of 1.05-1.07, comfortably
above the 0.8 target. Pack quality has genuine natural variance regardless of
symbol distribution.

## Deck Composition

Deck S/A fraction ranges from 86.8% (all 1-sym) to 89.3% (all 3-sym). All
exceed the 60-90% target range, with higher-symbol distributions slightly more
concentrated because more frequent spending delivers more on-archetype cards.

## Key Finding

**Symbol count distribution has moderate impact on save/spend rhythm but
minimal impact on convergence quality.** The bonus card mechanic (which drives
convergence) depends only on whether the player spends, and spend frequency
varies modestly (54-69%) across the full range of distributions.

The most important differentiator is the save/spend rhythm:
- **All 1-sym** creates the richest economic decisions: spend 54% of the time,
  with maximum non-spend streaks averaging 3.6 picks. The player must genuinely
  plan when to spend.
- **All 3-sym** is the most automatic: spend 69% of the time, with shorter
  forced-save windows (1.8 picks). The decision is less interesting but still
  present.

However, symbol count also affects **token scatter**. More symbols mean more
tokens flowing to non-primary resonances. With all 3-sym cards, a committed
player earns ~2 extra off-resonance tokens per pick. This does not help
convergence (since the player only spends on primary) but could enable
multi-resonance strategies that Agent 4 (symbol patterns) and Agent 5
(parameter tuning) should investigate.

## Recommendation

**Heavy 1-symbol (70/20/10) or Moderate 1-symbol (50/35/15)** are the best
starting points. They provide:
- Genuine save/spend rhythm (57-60% spend frequency, 2.6-3.0 max non-spend streak)
- Good convergence (1.92-1.95 SA mean, exceeding 1.8 threshold)
- Healthy variance (1.06-1.07 stddev)
- Room for pattern variety (the 20-35% 2-sym and 10-15% 3-sym cards create
  different token profiles without flooding the economy)

The V4 Default (20/55/25) and heavier distributions work fine but push toward
more automatic spending. If Agent 5 finds that cost 4 or cost 5 is viable,
heavier symbol distributions become more attractive because the higher cost
would throttle the spend frequency back into the interesting range.

**The symbol distribution should be chosen in coordination with the spend cost
parameter.** Lower symbol counts pair well with cost 3 (the current default).
Higher symbol counts need higher spend costs to preserve decision quality.
