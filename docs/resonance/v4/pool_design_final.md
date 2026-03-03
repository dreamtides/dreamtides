# V4 Pool Design: Final Specification

## Complete Pool Specification

**360 cards total.** Pack Widening v3 parameters: **Spend Cost 5, Bonus Cards 2, Primary Weight 3.**

### Card Distribution

| Category | Count | Notes |
|----------|-------|-------|
| Generic (0 symbols) | 36 | 10% of pool, no resonance, B-tier for all archetypes |
| Bridge cards | 48 | 6 per adjacent archetype pair (8 pairs), S-tier for both home archetypes |
| Archetype cards | 276 | 34-35 per archetype, evenly distributed across all 8 archetypes |

### Symbol Count Distribution (per archetype's 34 cards)

| Symbols | Fraction | Cards per Archetype | Pool Total |
|---------|----------|---------------------|------------|
| 1-symbol | 50% | 17 | 136 |
| 2-symbol | 35% | 12 | 96 (+48 bridge = 144 2-sym total) |
| 3-symbol | 15% | 5 | 40 |

### Pattern Composition

**1-symbol cards:** 100% [Primary]. Clean identity signal, slow token accumulation.

**2-symbol cards (non-bridge):**
- 45% [P,P] -- 3 primary tokens, fast spend accumulation
- 35% [P,O] -- 2 primary + 1 off-resonance, bridge viability
- 20% [P,S] -- 2 primary + 1 secondary, standard dual-resonance

**3-symbol cards:**
- 20% [P,P,P] -- deep primary commitment (5 tokens, 3 primary)
- 30% [P,P,O] -- concentrated primary + bridge token
- 25% [P,P,S] -- primary concentration + secondary support
- 25% [P,S,O] -- spread across three resonances

**Bridge cards:** 2-symbol [A-primary, B-primary], alternating which archetype's primary leads. S-tier for both adjacent archetypes.

**Avoid:** [S,P] and [S,S] patterns entirely. These distort archetype balance and weaken convergence (Agent 4 finding).

### Rarity Distribution

| Rarity | Count | Power Range |
|--------|-------|-------------|
| Common | 180 | 2.0-5.0 |
| Uncommon | 100 | 4.0-7.0 |
| Rare | 60 | 6.0-9.0 |
| Legendary | 20 | 8.0-10.0 |

Rarity is distributed uniformly across archetypes and symbol counts. No rarity-symbol correlation.

---

## How Each Agent's Findings Were Incorporated

**Agent 1 (Symbol Distribution):** Adopted 50/35/15 (heavy 1-symbol). This creates the best save/spend rhythm at cost 5, with a 2.1-pick max non-spend streak and 65% spend frequency. Agent 1's finding that convergence is surprisingly stable across distributions (1.85-2.04 S/A) confirmed that symbol distribution is primarily about decision quality, not convergence power.

**Agent 2 (Rarity):** Adopted standard TCG rarity (180/100/60/20) with no rarity-symbol correlation. Agent 2 conclusively showed rarity is orthogonal to Pack Widening's convergence mechanism. TCG rarity contributes ~28% draft tension rate (power-vs-synergy dilemmas), which is the right design function for rarity.

**Agent 3 (Archetypes):** Adopted equal archetype sizes with 10% generic and 48 bridge cards (6 per adjacent pair). Agent 3 showed that bridge cards produce the highest late S/A (1.71 in their bonus-1 tests) and strongest bridge strategy viability (78.9%). Large generic pools and asymmetric sizes were rejected as strictly inferior.

**Agent 4 (Patterns):** Adopted Config G (Concentrated+Bridge) pattern mix. This achieves the highest genuine choice rate (62% in Agent 4's tests, 58.3% in the synthesis simulation) by ensuring each archetype's card pool contains at least 3 distinct token profiles. The mix of [P,P] (fast primary) and [P,O] (bridge option) creates meaningful decisions between concentration and flexibility.

**Agent 5 (Parameters):** Adopted Cost 5, Bonus 2, Weight 3 as the primary recommendation. Agent 5's key finding -- bonus cards = 2 is mandatory for archetype-level convergence -- was the single most impactful change. No configuration with bonus=1 reached the 2.0 S/A target. Cost 5 with weight 3 produces the ideal spend frequency (65%) with genuine decision tension.

---

## Tensions and Resolutions

### 1. Token Rate vs Decision Quality

Agent 1 wanted fewer symbols (slower tokens, more save/spend decisions). Agent 4 wanted pattern variety (requiring multi-symbol cards). Resolution: the 50/35/15 distribution satisfies both. The heavy 1-symbol allocation (50%) keeps average tokens/pick moderate (3.60), while the 35% 2-symbol and 15% 3-symbol cards provide the pattern variety Agent 4 needs for genuine choice. The high spend cost (5) compensates for weight-3 primary tokens, keeping spend frequency at 65% rather than the 84% auto-spend seen in the V4 default.

### 2. Simplicity vs Richness

The Concentrated+Bridge pattern mix uses only 7 pattern types total (1 for 1-sym, 3 for 2-sym, 4 for 3-sym) plus bridge cards. This is manageable for card designers while creating 3+ distinct token profiles per archetype. Full pattern variety (Agent 4's Config D with 11+ types) was rejected because scatter above 20% kills convergence.

### 3. Bonus Hit Rate vs Pool Diversity

Bridge cards improve the bonus card S/A hit rate by adding S-tier overlap between adjacent archetypes. With 48 bridge cards in the pool, each resonance's primary-pool contains more cards that are S-tier for adjacent archetype players, improving bonus card quality without concentrating the entire pool.

---

## Before/After Comparison

All metrics below are for the archetype-committed strategy, 1200 drafts.

| Metric | V4 Default | Reconciled | Target | Delta |
|--------|-----------|------------|--------|-------|
| Late S/A per pack | 1.56 | **2.09** | >= 2.0 | **+0.53** |
| Early diversity | 5.35 | 5.52 | >= 3.0 | +0.17 |
| Early S/A (not-on-rails) | 1.15 | 1.33 | <= 2.0 | +0.18 |
| Late off-archetype C/F | 1.34 | 1.22 | >= 0.5 | -0.12 |
| Deck concentration | 82.3% | 80.9% | 60-90% | -1.4pp |
| Card overlap | 4.7% | 5.3% | < 40% | +0.6pp |
| SA StdDev (variance) | 0.96 | **1.30** | >= 0.8 | **+0.34** |
| Genuine choice rate | 44.3% | **58.3%** | -- | **+14pp** |
| Spend frequency (6+) | 84.4% | **65.4%** | 50-70% | **-19pp** |
| Max no-spend streak | 1.4 | **2.1** | -- | **+0.7** |
| Targets passed | 7/9 | **8/9** | 9/9 | **+1** |

The reconciled design passes Late S/A (the primary convergence target) that the default fails. The only remaining failure is convergence pick (3.1 vs target 5-8), meaning convergence happens slightly too early. This is a consequence of bonus=2 providing strong early filtering. The early convergence is acceptable because early packs still show 5.5 unique archetypes (well above the 3.0 openness target) and only 1.33 S/A for the emerging archetype (well below the 2.0 on-rails threshold).

---

## Token Economy Analysis

**Tokens earned per pick:** 3.60 average (primary: 3.08). With cost 5, this means a committed player accumulates enough to spend roughly every 1.5 picks after commitment.

**Spend frequency:** 65.4% of picks 6+ involve spending. This creates a genuine save/spend rhythm: roughly 2 spend packs for every 1 non-spend pack, with occasional streaks of 2-3 non-spend picks that create planning tension.

**Save/spend decision quality:** The max no-spend streak of 2.1 picks means players regularly face "spend now on a mediocre bonus, or save for a better opportunity next pack" decisions. This contrasts with the V4 default's 1.4-pick streak and 84% spend frequency, where spending is nearly automatic.

**Three-act draft arc:**
- **Act 1 -- Exploration (picks 1-5):** S/A 1.33/pack, first spend at pick 4.6, 5.5 unique archetypes per pack. The player sees a wide variety of options with no pressure to commit. Token accumulation begins but spending is not yet available.
- **Act 2 -- Commitment (picks 6-15):** S/A rises to 2.11/pack. Spending begins in earnest at 65% frequency. The player locks into an archetype and begins seeing 2+ good cards per pack regularly.
- **Act 3 -- Refinement (picks 16-30):** S/A holds steady at 2.08/pack. Genuine choice rate of 58.3% means most packs present meaningful decisions between cards with different token profiles. The draft does not decline -- the convergence curve is flat from pick 6 onward (trend: -0.02).

---

## Open Questions for Playtesting

1. **Convergence pick too early?** The simulation shows convergence at pick 3.1 (target is 5-8). In practice, human players take longer to identify their archetype than the simulation's pick-5 commitment model. Playtesting should verify whether the early statistical convergence translates to felt-too-early commitment pressure.

2. **Bonus 2 pack size feel.** Spend packs contain 6 cards (4 random + 2 bonus). Does picking 1 from 6 feel meaningfully different from picking 1 from 4? If 6-card packs feel overwhelming, consider reducing base pack to 3 (giving 5-card spend packs).

3. **Cost 5 feel.** Spending 5 tokens is a bigger commitment than spending 3. Does this create satisfying "big moment" tension, or does it feel punishing when the bonus cards whiff?

4. **Bridge card density.** 48 bridge cards (13.3% of pool) is significant. Do bridge strategies emerge naturally, or do bridge cards just act as better generic filler?

5. **Pattern variety vs complexity.** The 7 pattern types create 3+ distinct token profiles per archetype. In practice, can players distinguish between [P,P] and [P,O] cards quickly enough for the token economy decision to feel deliberate rather than accidental?

6. **Signal reader benefit.** The simulation shows signal readers achieve 2.11 S/A (slightly above committed's 2.09), suggesting modest benefit to reading the pool. Is this gap large enough to be perceptible and rewarding?
