# Pool Design Agent 1: Symbol Count Distribution for Pair-Escalation Slots

## Question

What ratio of 1/2/3-symbol cards (among non-generic cards) produces the best
draft experience with Pair-Escalation Slots (K=6, C=0.50)?

## Method

Tested 10 distributions across 1200 drafts each, 30 picks per draft. Pool: 360
cards (36 generic, 324 split across 8 archetypes). Committed player strategy
with archetype commitment around pick 5-6.

## Key Finding: Distribution Matters Less Than Expected

Unlike Pack Widening (V4) where symbol distribution heavily influenced the
save/spend rhythm, Pair-Escalation's probability cap (C=0.50) creates a ceiling
that most distributions reach by pick 9-12. Once the top pair count exceeds 3
(triggering P = 3/6 = 50% = cap), all further pair accumulation is irrelevant.
The critical window is picks 1-9, where pair accumulation speed determines how
fast the algorithm "turns on."

**Probability ramp comparison (avg P at pick):**

| Distribution | P@3 | P@5 | P@7 | P@9 | P@12 |
|---|---|---|---|---|---|
| All 2+3-sym (0/70/30) | 19.3% | 29.5% | 42.3% | 48.2% | 49.8% |
| V5 Recommended (15/60/25) | 17.6% | 26.9% | 39.2% | 46.5% | 49.3% |
| Heavy 1-sym (60/30/10) | 10.1% | 18.9% | 29.8% | 39.6% | 45.7% |
| All 1-sym (100/0/0) | 0% | 0% | 0% | 0% | 0% |

All non-degenerate distributions reach near-cap by pick 12-15. The difference
is 2-3 picks of earlier activation, not a fundamental quality shift.

## Scorecard Results (7 targets passed = best)

| Distribution | Late S/A | Conv | StdDev | Deck Conc | Passes |
|---|---|---|---|---|---|
| **Heavy 3-sym (10/30/60)** | **2.06** | 10.1 | 1.12 | 82.8% | **7/8** |
| **V5 Recommended (15/60/25)** | **2.00** | 10.3 | 1.08 | 82.8% | **7/8** |
| Balanced (33/34/33) | 1.98 | 10.5 | 1.09 | 82.6% | 6/8 |
| All 2+3-sym (0/70/30) | 1.96 | 10.0 | 1.06 | 83.1% | 6/8 |
| Quarter 1-sym (25/50/25) | 1.91 | 11.4 | 1.10 | 80.5% | 6/8 |
| Moderate 1-sym (40/40/20) | 1.86 | 11.6 | 1.08 | 80.2% | 6/8 |
| Minimal 1-sym (5/70/25) | 1.87 | 11.2 | 1.08 | 81.4% | 6/8 |
| Heavy 1-sym (60/30/10) | 1.81 | 12.2 | 1.07 | 79.5% | 6/8 |
| Heavy 2-sym (10/80/10) | 1.76 | 12.4 | 1.07 | 79.2% | 6/8 |
| All 1-sym (100/0/0) | 0.90 | 27.3 | 0.83 | 57.1% | 5/8 |

The universal failure is convergence pick: no distribution achieves the 5-8
target. All land at 10-12, inherent to K=6/C=0.50.

## Why Heavy 3-Symbol Wins

Heavy 3-sym (10/30/60) achieves the highest Late S/A at 2.06 because 3-symbol
cards contribute to pair counts (via their first two symbols) AND carry more
resonance symbols total, increasing the base S/A density in the pool. Each
3-symbol card like [Tide, Zephyr, Tide] counts as a (Tide, Zephyr) pair for
accumulation while also being broadly useful for any archetype touching Tide or
Zephyr.

## The 1-Symbol Trap

| Distribution | 1-Sym Pick % | Avg Max Stall |
|---|---|---|
| All 1-sym | 100% | 25.0 picks |
| Heavy 1-sym (60/30/10) | 21.5% | 1.9 picks |
| V5 Recommended (15/60/25) | 5.0% | 0.8 picks |
| Heavy 2-sym (10/80/10) | 3.5% | 0.7 picks |
| All 2+3-sym (0/70/30) | 0.0% | 0.3 picks |

At 60% 1-symbol cards, players experience stalls of ~2 consecutive picks
without pair contribution roughly once per draft. At 15% or below, stalls are
negligible (< 1 pick). The 1-symbol trap is a real concern only above 40%.

## Surprising Result: Heavy 2-Symbol Underperforms

Heavy 2-sym (10/80/10) achieves only 1.76 Late S/A despite having the most
pair-contributing cards in the pool. The reason: with 80% 2-symbol cards, the
pool is saturated with pair-matched cards but the *base density* of S/A cards
per random draw is lower than distributions with 3-symbol cards. Three-symbol
cards like [Ember, Stone, Ember] are S-tier for Storm AND provide pair data,
making them double-useful. Two-symbol cards provide only pair data with no
extra resonance breadth.

## Recommendation

**V5 Recommended (15/60/25) is validated as the best practical distribution.**
It ties for the most targets passed (7/8) and sits in a robust plateau where
small changes produce minimal effect. Heavy 3-sym (10/30/60) edges it on raw
S/A (+0.06) but creates a pool where nearly all non-generic cards carry 3
symbols, which may feel monotonous in actual card design.

The practical sweet spot is 10-20% 1-symbol, 50-65% 2-symbol, 20-30% 3-symbol.
Within this range, all metrics are essentially equivalent (Late S/A 1.91-2.06,
convergence 10-11, StdDev 1.06-1.12). Card designers have flexibility to
allocate symbol counts based on gameplay needs without worrying about breaking
the draft algorithm.

**Critical constraint:** Keep 1-symbol cards below 40% of non-generic pool.
Above that threshold, pair accumulation slows measurably (first P>=0.25 delayed
from pick 5-6 to pick 7-8) and stall streaks become noticeable.
