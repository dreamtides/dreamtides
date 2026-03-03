# Agent 3: Archetype Breakdown Results

## Summary

Tested 5 archetype distribution models with Pack Widening v3 across 1000 drafts
each (plus 1000 bridge-strategy drafts per model). The **Equal + Bridge Cards**
model performs best overall, followed closely by the **Equal + Small Generic
(10%)** baseline. Large generic pools and mono-symbol designs hurt convergence.

## Models Tested

| Model | Generic | Archetype Cards | Special |
|-------|---------|-----------------|---------|
| Equal + Small Generic (10%) | 36 | 40-41 per arch | Baseline |
| Equal + Large Generic (25%) | 90 | 33-34 per arch | More generics |
| Equal + Bridge Cards | 36 | 34 per arch + 48 bridge | 6 bridge per adjacent pair |
| Asymmetric Sizes | 36 | 24-55 per arch | 2 large, 2 small |
| Mono-Symbol Only | 36 | 40-41 per arch, all 1-sym | No multi-symbol cards |

## Key Findings

### 1. Bridge Cards Produce the Best Archetype Concentration

The Bridge model achieves the highest late S/A (1.71 per pack) and deck
concentration (85.5% S/A). Bridge cards are S-tier for two adjacent archetypes,
which creates more S/A density without needing algorithm changes. The S-tier
share rises to 60.7% (vs 56.4% baseline), while A-tier drops slightly to 24.8%.
This is desirable: committed players find more home-archetype cards rather than
merely adjacent ones.

### 2. Large Generic Pools Hurt Convergence

Increasing generics from 10% to 25% drops late S/A from 1.63 to 1.46 and deck
concentration from 84.5% to 81.1%. The mechanism is twofold: fewer archetype
cards in the pool means fewer S/A hits per random draw, and generic cards earn
no tokens, slowing spending. Token earn rate drops from 2.98 to 2.82 per pick,
first spend delays from pick 3.7 to 4.1, and spend frequency drops from 90.0%
to 85.5%. The 25% generic model also produces the lowest early diversity (4.79
unique archetypes per pack, vs 5.37+ for others). Generics dilute the pool
without contributing to any archetype's identity.

**Recommendation:** Keep generics at ~10% (36 of 360). Even this amount is
marginally harmful to convergence; anything higher is clearly worse.

### 3. Mono-Symbol Design Cripples the Token Economy

When all archetype cards have only [Primary], token earn rate drops to 1.92 per
pick (vs ~3.0 for multi-symbol designs). This delays first spend to pick 5.0
and reduces spend frequency to 61.4%. The reduced spending means fewer bonus
cards, which drops late S/A to 1.46. Deck concentration falls to 78.9%.

However, mono-symbol produces the highest bonus card S/A hit rate (90.5% vs
~80% for others). This makes sense: all cards in the primary-resonance pool have
[Primary] as their only symbol, meaning they belong purely to the two archetypes
with that primary resonance. With equal distribution, 50% are home archetype (S)
and 50% are adjacent (A), giving near-100% S/A hit rate. But this advantage is
overwhelmed by the lower spend frequency.

### 4. Asymmetric Sizes Offer No Benefits

The asymmetric model (sizes 24-55 per archetype) performs nearly identically to
the equal baseline on all metrics. Late S/A is 1.63 for both. The only
difference is archetype frequency balance: the equal model keeps all archetypes
within 11.5-14.3%, while asymmetric shows comparable range but the underlying
pool gives players of smaller archetypes fewer options. Since asymmetry adds
design complexity without measurable benefit, equal sizes are preferred.

### 5. Bridge Strategy Viability Is Strong Across All Models

A player committing to two adjacent archetypes finds S/A cards for both
archetypes in 70-79% of late packs. The Bridge model leads at 78.9% with
balanced S/A rates for both archetypes (1.58/1.58). The baseline achieves 75.9%
with slightly asymmetric rates (1.51/1.48). Even the worst performer (Mono-
Symbol at 69.4%) provides viable bridge strategy support.

Explicit bridge cards enhance this further by adding S-tier overlap between
adjacent archetypes.

### 6. Bonus Card Hit Rate Is ~80% Across Standard Models

When spending on resonance R, roughly 80% of bonus cards are S/A for the
player's archetype. The primary-resonance pool contains cards from 2 primary
archetypes and 2 secondary archetypes; for a committed player, one primary
archetype is home (S) and one is adjacent (A), giving ~50% S/A from primary
cards alone, plus additional hits from secondary-archetype cards.

### 7. No Model Achieves the >= 2 S/A Convergence Target

All models converge at pick 30 (never reaching the >= 2.0 S/A threshold). Peak
late S/A ranges from 1.46 (worst) to 1.71 (best). This is an algorithm-
parameter issue, not an archetype-breakdown issue: with only 1 bonus card per
spend, the bonus's contribution to average pack S/A is limited. Parameter tuning
(Agent 5) will need to address this gap.

## Target Scorecard

| Metric | Target | Best Model (Bridge) | Baseline (10%) |
|--------|--------|--------------------:|----------------:|
| Early diversity | >= 3 | 5.62 PASS | 5.37 PASS |
| Early S/A | <= 2 | 1.22 PASS | 1.16 PASS |
| Late S/A | >= 2 | 1.71 FAIL | 1.63 FAIL |
| Late off-arch | >= 0.5 | 1.21 PASS | 1.35 PASS |
| Convergence pick | 5-8 | 30 FAIL | 30 FAIL |
| Deck concentration | 60-90% | 85.5% PASS | 84.5% PASS |
| Run-to-run variety | < 40% | 13.6% PASS | 13.9% PASS |
| SA StdDev (late) | >= 0.8 | 0.97 PASS | 0.95 PASS |

## Recommendation

Use **equal archetype sizes** (~40 cards each) with **~10% generic cards** (36)
and **6 bridge cards per adjacent pair** (48 total). This provides:
- Highest late S/A (1.71) and deck concentration (85.5%)
- Strongest bridge strategy support (78.9% dual-S/A packs)
- Good bonus card hit rate (80.3%)
- Excellent early diversity (5.62 unique archetypes per pack)

The remaining convergence gap (1.71 vs target 2.0) should be addressed through
algorithm parameter tuning rather than further pool restructuring.
