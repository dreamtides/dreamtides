# Agent 4 Results: Symbol Pattern Composition

## Token Profiles Per Pattern

Each pattern generates a distinct token profile (for archetype P=primary, S=secondary):

| Pattern | Pri Tokens | Sec Tokens | Other Tokens | Total |
|---------|-----------|-----------|-------------|-------|
| [P] | 2 | 0 | 0 | 2 |
| [S] | 0 | 2 | 0 | 2 |
| [P,S] | 2 | 1 | 0 | 3 |
| [P,P] | 3 | 0 | 0 | 3 |
| [S,P] | 1 | 2 | 0 | 3 |
| [P,O] | 2 | 0 | 1 | 3 |
| [P,P,S] | 3 | 1 | 0 | 4 |
| [P,S,S] | 2 | 2 | 0 | 4 |
| [P,S,O] | 2 | 1 | 1 | 4 |
| [P,P,O] | 3 | 0 | 1 | 4 |
| [P,O,O] | 2 | 0 | 2 | 4 |

There are 11 mechanically distinct token profiles. The critical axis is how much each pattern concentrates tokens in the primary resonance versus spreading them.

## Seven Configurations Tested (1000 drafts each)

| Config | Description | Late S/A | Genuine Choice | Token Scatter | Spend Rate | Accidental Commit |
|--------|------------|---------|---------------|---------------|-----------|------------------|
| A: Uniform [P,S] | All 2-sym cards use [P,S] | 1.95 | 59.2% | 17.9% | 59.9% | 38.5% |
| B: Primary Heavy | All [P], [P,P], [P,P,P] | **2.08** | 58.7% | **6.7%** | **73.3%** | **17.5%** |
| C: Balanced Variety | Mix of [P,S], [P,P], [P,O] | 1.92 | 60.8% | 18.5% | 57.1% | 39.5% |
| D: Max Variety | Every pattern equally weighted | 1.81 | 57.2% | 22.8% | 54.8% | 44.5% |
| E: Bridge Optimized | Heavy [P,O] and [P,S,O] | 1.79 | 56.1% | 28.1% | 48.0% | 50.0% |
| F: Secondary Spread | Heavy [S,P], [S,S], [P,S,S] | 1.73 | 54.0% | 25.9% | 52.4% | 47.6% |
| G: Concentrated+Bridge | Mix of [P,P] and [P,O] | 1.97 | **62.0%** | 15.7% | 61.1% | 35.1% |

All configs pass 7/8 targets. Only B (Primary Heavy) passes all 8 by reaching the 2.0 S/A late-game threshold.

## Key Findings

### 1. Token Concentration Drives Convergence

Config B (all primary-heavy) achieves the highest spend rate (73.3%) and the only passing late S/A score (2.08). Every token goes toward primary resonance, enabling near-constant spending after commitment. Configs that scatter tokens (D, E, F) see spend rates drop to 48-55% and late S/A falls to 1.73-1.81 as tokens leak into unused resonances.

### 2. Pattern Variety and Genuine Choice Are in Tension with Convergence

Config G (Concentrated+Bridge) achieves the highest genuine choice rate (62.0%) -- packs frequently contain 2+ S/A cards with different token profiles, forcing the player to decide between faster primary accumulation and broader token spread. But its late S/A is 1.97, just below the 2.0 target.

Config B has the lowest genuine choice rate among top performers (58.7%) because all its cards produce similar token profiles (all primary-concentrated). The "choice" between [P,P] and [P,P,P] cards has no strategic dimension for the token economy.

### 3. Accidental Token Scatter Is the Primary Enemy

Accidental commitment (tokens in non-archetype resonances) ranges from 17.5% (Config B) to 50.0% (Config E). High scatter not only reduces spend frequency but creates false bridge signals -- a player accumulates 3+ tokens in an irrelevant resonance, producing misleading spend options.

### 4. Bridge Spending Is Viable But Costly

Config E generates the most off-primary tokens (48.9 avg) but its late S/A drops to 1.79 because splitting tokens means neither resonance reaches spend threshold quickly. Bridge spending works mechanically but trades convergence for flexibility.

The viable bridge path requires deliberate [P,O] card drafting, not accidental scatter. If the pool forces scattered tokens on everyone, it weakens primary spending without creating a real alternative.

### 5. Secondary-Primary ([S,P]) Patterns Hurt Archetype Balance

Configs with many [S,P] patterns (F) show worse archetype frequency balance, with Flash spiking to 23.2%. When many cards have secondary-as-primary symbols, the fitness assignment becomes skewed -- cards that should be B-tier for adjacent archetypes get treated as if they have that resonance as primary, distorting which archetypes the commitment heuristic selects.

## Recommendation

**Config G (Concentrated+Bridge)** is the best overall design, with one parameter adjustment needed:

- **Core pattern mix for 2-symbol cards:** 45% [P,P], 35% [P,O], 20% [P,S]
- **Core pattern mix for 3-symbol cards:** 20% [P,P,P], 30% [P,P,O], 25% [P,P,S], 25% [P,S,O]
- **1-symbol cards:** 100% [P]
- **Avoid [S,P] and [S,S] patterns** -- they weaken convergence and distort archetype balance

This composition achieves:
- Highest genuine choice rate (62.0%) because [P,P] and [P,O] cards create distinct token profiles
- Moderate scatter (15.7%) -- enough for bridge viability, not so much it wastes tokens
- Strong spend rate (61.1%) and late S/A just below target (1.97)
- Accidental commitment contained at 35.1%

The late S/A gap (1.97 vs 2.0 target) is likely closable by tuning the spend cost or symbol count distribution (Agent 1/5's territory) rather than changing the pattern composition. Pattern composition's job is to maximize genuine choice; convergence power is primarily a function of overall token earn rate and spend cost.

**The critical design principle:** every archetype's card pool should contain cards with at least 3 distinct token profiles. A Warriors player choosing between [Tide,Tide] (3 Tide tokens, fast spend) and [Tide,Ember] (2 Tide + 1 Ember, bridge toward Flash/Blink) is making a genuinely strategic decision about their token economy.
