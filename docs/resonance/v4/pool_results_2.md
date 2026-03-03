# Agent 2 Results: Rarity x Pack Widening v3

## Setup

Tested 5 rarity models across 1200 drafts each, 30 picks per draft, using Pack
Widening v3 (spend cost 3, bonus 1, primary weight 2). Default symbol
distribution (25/55/20 for 1/2/3-symbol cards, ~10% generic).

- **A: Flat** -- 180C/100U/60R/20L, uniform power (~5.0)
- **B: Standard TCG** -- 180C/100U/60R/20L, power scales (C=3.5, U=5.4, R=7.7, L=9.2)
- **C: Roguelike-Skewed** -- 120C/120U/80R/40L, power scales
- **D: Rarity-Symbol Correlation** -- Commons 1-sym, rares 2-3 sym, power scales
- **E: Inverse Correlation** -- Commons 2-3 sym, rares 1-sym, power scales

## Core Finding: Rarity Is Orthogonal to Convergence

All five models produce nearly identical convergence metrics:

| Metric | A:Flat | B:TCG | C:Rogue | D:SymCor | E:InvCor | Target |
|--------|--------|-------|---------|----------|----------|--------|
| Late S/A per pack | 2.05 | 2.07 | 2.09 | 2.03 | 2.00 | >= 2 |
| Convergence pick | 6.5 | 6.4 | 6.4 | 6.5 | 6.5 | 5-8 |
| Early unique archs | 6.47 | 6.49 | 6.48 | 6.49 | 6.49 | >= 3 |
| Deck concentration | 94.1% | 94.2% | 94.6% | 93.7% | 93.2% | 60-90% |
| Late S/A stddev | 1.05 | 1.05 | 1.04 | 1.05 | 1.06 | >= 0.8 |

The token system cares about symbol count, not card quality. Changing rarity
distributions and power curves has no measurable effect on whether a committed
player sees S/A-tier cards. This confirms the hypothesis: rarity and the Pack
Widening convergence mechanism operate on independent axes.

All models exceed the 90% deck concentration cap. This is a structural issue
with the pool/algorithm, not a rarity issue.

## Draft Tension: Rarity's Real Contribution

Where rarity matters is **draft tension** -- how often a player faces "strong
off-archetype rare vs weak on-archetype common" dilemmas.

| Model | Tension (spend) | Tension (non-spend) | Tension (overall) |
|-------|-----------------|---------------------|-------------------|
| A: Flat | 0.0% | 0.0% | 0.0% |
| B: Standard TCG | 28.8% | 27.3% | 28.7% |
| C: Roguelike | 27.8% | 26.9% | 27.7% |
| D: Rarity-Symbol | 29.8% | 29.6% | 29.8% |
| E: Inverse | 30.1% | 26.8% | 29.6% |

Flat rarity produces zero tension -- every card is equally powerful, so picking
on-archetype is always correct. Standard TCG rarity creates ~28% tension rate,
meaning roughly 1 in 4 late-draft packs presents a genuine power-vs-synergy
dilemma. This is a desirable design property.

Tension rates are nearly identical between spend and non-spend packs across all
models, confirming that rarity does not meaningfully interact with the spending
decision.

## Rarity-Symbol Correlation: The Token Feedback Loop

Model D (rares have more symbols) creates a measurable token economy shift:

| Model | Tokens/pick | Tokens from R/L | Tokens from C/U | Late spend freq |
|-------|-------------|-----------------|-----------------|-----------------|
| B: Standard | 2.91 | 2.89 | 2.93 | 93.8% |
| D: Rarity-Sym | 2.81 | 3.47 | 2.50 | 91.0% |
| E: Inverse | 2.64 | 2.10 | 2.89 | 85.7% |

In Model D, rare/legendary picks earn 3.47 tokens (vs 2.50 from commons) -- a
39% premium. This creates a "rare = faster convergence" feedback loop where
drafting a high-power rare also accelerates spending. However, because the
archetype-committed player already picks mostly on-archetype cards regardless of
rarity, the actual impact on spend frequency is modest (91.0% vs 93.8%). The
feedback loop exists but is not degenerate.

Model E shows the opposite: rares earn only 2.10 tokens, slowing the overall
economy to 2.64 tokens/pick and reducing spend frequency to 85.7%. This creates
more non-spend packs, which slightly increases variance but does not improve
convergence.

## Power Variance and Replayability

| Model | Avg Power | Power StdDev | Power Gap (chaser - committed) |
|-------|-----------|-------------|-------------------------------|
| A: Flat | 5.10 | 0.061 | 0.24 |
| B: Standard | 5.71 | 0.402 | 1.83 |
| C: Roguelike | 6.39 | 0.409 | 1.78 |
| D: Rarity-Sym | 5.56 | 0.392 | 1.92 |
| E: Inverse | 5.62 | 0.377 | 1.78 |

Flat rarity produces almost zero power variance between runs (stdev 0.061) --
every draft feels equally powerful. Standard TCG rarity produces 6-7x higher
variance (stdev ~0.4), meaning some runs feel noticeably stronger than others.
This is good for roguelike replayability.

The power gap between committed and power-chaser strategies is ~1.8-1.9 for all
scaled models, indicating a genuine strategic cost to ignoring archetype synergy.
Model D has the largest gap (1.92), suggesting rarity-symbol correlation slightly
punishes power-chasers (they pick strong rares that scatter tokens).

## Recommendations

1. **Use standard TCG rarity with power scaling** (Model B). It produces
   meaningful draft tension (~28%), healthy power variance for replayability, and
   does not interfere with the convergence mechanism.

2. **Avoid flat rarity.** Zero draft tension makes every pick trivial once
   committed.

3. **Rarity-symbol correlation (Model D) is a viable option** if the design
   wants a subtle "rare = token bonus" feel. The feedback loop is present but not
   degenerate at current parameters. It does shift commons toward 1-symbol
   (slower tokens from weak cards) which other agents should evaluate for token
   economy impact.

4. **Avoid inverse correlation (Model E).** Slowing the token economy by putting
   fewer symbols on rares makes the spend mechanic less available without
   improving any other metric.

5. **Avoid roguelike-skewed (Model C).** More rares do not improve convergence
   and push archetype frequency slightly outside bounds.

6. **Rarity is orthogonal to spending.** The spend/save decision should be tuned
   via symbol distribution and spend cost (Agents 1 and 5), not rarity.
