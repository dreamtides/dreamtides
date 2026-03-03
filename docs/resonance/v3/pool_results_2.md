# Rarity x Lane Locking: Simulation Results

## Summary Comparison Table

All models use Lane Locking (thresholds 3/8, primary=2). 1000 drafts per model,
archetype-committed player strategy. Only deck concentration fails across all
models (a known fitness-model artifact from the V3 final report).

| Metric | Target | A: Flat | B: TCG | C: Rogue | D: SymCorr | E: InvCorr |
|--------|--------|:---:|:---:|:---:|:---:|:---:|
| Early unique archs w/ S/A | >= 3 | 6.50 | 6.49 | 6.49 | 6.51 | 6.48 |
| Early S/A for arch | <= 2 | 1.93 | 1.91 | 1.91 | 1.87 | 1.86 |
| Late S/A for arch | >= 2 | 2.70 | 2.67 | 2.69 | 2.65 | 2.68 |
| Late C/F cards | >= 0.5 | 0.83 | 0.84 | 0.81 | 0.82 | 0.82 |
| Convergence pick | 5-8 | 6.1 | 6.1 | 6.1 | 6.2 | 6.1 |
| Deck concentration | 60-80% | 98.9% FAIL | 98.9% FAIL | 98.9% FAIL | 98.7% FAIL | 98.8% FAIL |
| Card overlap | < 40% | 5.3% | 5.0% | 4.8% | 4.0% | 5.2% |
| Archetype freq | 5-20% | 7-18% | 8-20% | 9-20% | 10-18% | 8-19% |
| **Targets passed** | | **7/8** | **7/8** | **6/8** | **7/8** | **7/8** |

| Rarity Metric | A: Flat | B: TCG | C: Rogue | D: SymCorr | E: InvCorr |
|---------------|:---:|:---:|:---:|:---:|:---:|
| Avg power (committed) | 5.11 | 5.65 | 6.35 | 5.62 | 5.59 |
| Power stdev across runs | 0.063 | 0.428 | 0.436 | 0.369 | 0.359 |
| Rare/Leg in locked slot | 81.3% | 82.1% | 82.3% | 80.4% | 79.0% |
| Draft tension rate | 0.0% | 15.4% | 13.8% | 14.8% | 14.4% |
| Avg power (power chaser) | 5.32 | 7.23 | 7.85 | 7.14 | 7.08 |
| Power gap (chaser - committed) | 0.21 | 1.57 | 1.50 | 1.52 | 1.50 |

## Key Finding: Rarity Does Not Affect Lane Locking's Core Metrics

The eight standard targets are virtually identical across all five models. Lane
Locking operates on resonance symbols, and rarity does not change symbol
distribution in models A, B, or C. Even models D and E -- which explicitly
correlate symbols with rarity -- show negligible movement (<3%) on convergence,
splash, or archetype frequency.

**Rarity is orthogonal to Lane Locking.** This is the central result. The
locking mechanism controls pack structure through resonance slots; rarity
controls card *quality* within those slots. They operate on independent axes and
do not interfere.

## Where Rarity Matters: Power Variance and Draft Tension

The models diverge sharply on three dimensions that rarity *does* control:

**1. Power variance between runs.** Model A (flat power) produces near-zero
variance (stdev 0.063); every run feels roughly the same in power level. Models
B-E produce 6-7x higher variance (stdev 0.36-0.44), meaning some runs find
more rares and feel stronger. For a roguelike, this variance is desirable -- it
creates "good run" and "bad run" experiences that drive replayability.

**2. Draft tension.** Model A creates zero tension moments (0.0%) because all
cards have similar power. Models B-E create tension 13-15% of the time: the
player faces a choice between a high-power off-archetype rare and a low-power
on-archetype common. This is the core interesting draft decision that rarity
enables. Without power differentiation, picking on-archetype is always
correct and the draft becomes mechanical.

**3. Power chaser viability.** With flat power, the power chaser gains only
+0.21 average power over the archetype-committed player, making it a pointless
strategy. With scaled power (models B-E), the gap is +1.5, creating a genuine
strategic tradeoff: sacrifice archetype coherence for raw power. The power
chaser's deck concentration drops to ~62% S/A (vs ~99% committed), confirming
that choosing power costs archetype focus.

## Rarity-Symbol Correlation: Minimal Impact

Models D (rares have more symbols) and E (rares have fewer symbols) were
designed to test whether linking symbol count to rarity creates interesting
interactions with Lane Locking. The answer is no. Model D has slightly more
balanced archetype frequency (10-18% vs 8-20%) because multi-symbol rares
accelerate locking, but the effect is tiny. Model E's single-symbol
legendaries are slightly less likely to appear in locked slots (79.0% vs
82.1%) because a locked slot selects on primary resonance, and 1-symbol cards
have a slightly lower chance of matching -- but again, the effect is negligible.

**Recommendation: keep symbols and rarity independent.** Correlating them adds
design complexity without measurable gameplay benefit. Symbol count should serve
the Lane Locking algorithm's needs (the recommended 25/55/20 distribution);
rarity should serve the power/excitement curve independently.

## Model C Loses a Target: Archetype Frequency

The roguelike-skewed model (120C/120U/80R/40L) is the only model to drop to 6/8
targets, failing archetype frequency at 9-20% (with one archetype hitting
20.1%). More legendaries means more high-power temptations to draft off-
archetype, which slightly distorts archetype distribution. The effect is small
but measurable.

## Locked Slots and Rarity

Across all models, ~80% of rare/legendary sightings occur in locked slots. This
is mechanical: by pick 6-8 most players have 2-3 locked slots out of 4 total,
so locked slots account for the majority of card exposure. This means rares tend
to appear on-resonance, which is *good* -- it creates exciting moments where a
locked slot delivers a high-rarity card in your archetype.

## Recommendation

**Distribution: 180 Common / 100 Uncommon / 60 Rare / 20 Legendary** (Model B,
standard TCG ratios). Rationale:

1. **7/8 targets passed**, matching the best results.
2. **15.4% draft tension** -- the highest of any model, creating the most
   interesting pick decisions.
3. **Standard ratios are intuitive.** Players from other card games understand
   that rares are rarer and stronger. No explanation needed.
4. **Adequate variance** (stdev 0.428) for roguelike replayability without the
   archetype-frequency risk of Model C.

**Rarity-symbol interaction: none.** Keep symbol count and rarity as independent
axes. Symbols serve Lane Locking; rarity serves the power/excitement curve.

**Power scaling:** Commons 2-5, Uncommons 4-7, Rares 6-9, Legendaries 8-10.
Overlapping ranges are deliberate -- the best common should compete with a
mediocre uncommon, creating card-evaluation skill expression.

**Generic card rarity:** Spread across all four tiers proportionally (18C/10U/
6R/2L from the 36 generics). Generic legendaries serve as high-power flexible
picks that any archetype wants, creating natural draft tension.

**Pack construction: rarity is independent of locking.** Do not add rarity
guarantees to packs. Lane Locking already controls resonance structure; adding
rarity guarantees (e.g., "one uncommon+ per pack") would layer a second
structural system on top and violate the algorithm's simplicity. Let rarity
emerge naturally from the pool distribution -- with 50% commons, a 4-card pack
has roughly a 94% chance of containing at least one uncommon+ anyway.
