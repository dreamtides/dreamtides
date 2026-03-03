# Agent 4: Symbol Pattern Composition for Pair-Escalation Slots

## Algorithm

Pair-Escalation Slots (K=6, C=0.50, 4 slots, 30 picks). Each slot independently shows a pair-matched card with probability min(top_pair_count/6, 0.50).

## Core Finding

**The ordered pair extracted from positions [0] and [1] creates a critical filtering layer.** Only `[P,S,...]` patterns produce the home archetype's pair. Patterns like `[P,P]` produce degenerate (same-resonance) pairs that waste accumulation. Patterns like `[S,P]` feed the adjacent archetype's pair counter instead. This makes symbol pattern composition a primary lever for controlling how quickly and reliably pair-escalation activates.

## Five Configurations Tested (1200 drafts each)

| Config | 2-sym Patterns | Home Pair Rate | Degen% | Cross% |
|--------|---------------|----------------|--------|--------|
| 1: All [P,S] | 100% PS | 58.8% | 0.0% | 41.2% |
| 2: Conc+Bridge | 50% PS, 25% PP, 25% PO | 31.8% | 31.7% | 30.5% |
| 3: Home-80% | 80% PS, 10% PP, 10% SP | 51.6% | 9.3% | 39.0% |
| 4: Mixed | 50% PS, 25% SP, 25% PO | 39.5% | 0.0% | 52.0% |
| 5: Degen-Heavy | 50% PS, 30% PP, 20% PO | 30.3% | 36.3% | 26.7% |

Note: "Home Pair Rate" measures what fraction of a committed player's drafted 2+ symbol cards produce the home archetype's pair. Even the "All [P,S]" config only reaches 58.8% because the player also drafts A-tier cards from adjacent archetypes whose pairs differ.

## Standard Target Results

| Metric | Target | All PS | Conc+Br | Home-80 | Mixed | Degen |
|--------|--------|--------|---------|---------|-------|-------|
| Early unique archs | >=3 | 6.23 P | 6.31 P | 6.27 P | 6.37 P | 6.32 P |
| Early S/A emerging | <=2 | 1.86 P | 1.77 P | 1.81 P | 1.72 P | 1.77 P |
| Late S/A committed | >=2 | 2.61 P | 2.46 P | 2.55 P | 2.33 P | 2.48 P |
| Late off-arch C/F | >=0.5 | 1.18 P | 1.34 P | 1.25 P | 1.47 P | 1.31 P |
| Convergence pick | 5-8 | 6.5 P | 7.7 P | 6.9 P | 8.4 F | 7.5 P |
| Deck concentration | 60-90% | 96.3% F | 94.8% F | 95.8% F | 93.9% F | 95.0% F |
| Overlap | <40% | 17.1% P | 12.5% P | 15.7% P | 13.0% P | 12.4% P |
| StdDev S/A late | >=0.8 | 0.99 P | 1.05 P | 1.01 P | 1.06 P | 1.04 P |
| **Targets passed** | | **9/10** | **9/10** | **9/10** | **8/10** | **9/10** |

All configs fail only deck concentration (too high at 94-96%). The Mixed config also fails convergence (8.4, just outside the 5-8 window).

## Key Findings

**1. Home pair rate directly controls convergence speed.** All-PS achieves 58.8% home pair rate and converges at pick 6.5. Home-80% reaches 51.6% and converges at 6.9. Mixed (39.5%) converges at 8.4 (just failing). The relationship is roughly linear: each 10% drop in home pair rate delays convergence by ~1 pick.

**2. Degenerate pairs [P,P] are pure waste.** The Degenerate-Heavy config (36.3% of pair picks wasted) converges at 7.5 despite only 30.3% home pair rate -- because degenerate pairs can still accidentally become the "top pair" and trigger matching on same-resonance cards, which happen to include some S/A cards. However, this is unreliable and reduces late S/A to 2.48 vs 2.61 for All-PS.

**3. Cross-archetype feeding via [S,P] is the most destructive pattern.** The Mixed config has 52% cross-archetype feeding -- half of all pair-producing picks feed the wrong archetype's pair counter. This is worse than degenerate waste because cross-fed pairs actively compete with the home pair for "top pair" status, fragmenting the escalation signal. Mixed uniquely fails the convergence target.

**4. Pair-matched pool composition is always 100% S/A.** Across all configurations, every card whose ordered pair matches an archetype is either S-tier or A-tier for that archetype. This is a structural property: if a card has pair (P,S) matching archetype X, it must have P as position[0] (primary) and S as position[1] (secondary), which means it shares resonances with X. The S vs A split varies: All-PS shows 100% S-tier, while Mixed shows only 54.9% S-tier (rest A-tier from adjacent archetypes).

**5. Genuine choice rate is surprisingly stable (30-35%).** All configurations produce similar genuine choice rates (packs with 2+ S/A cards feeding different pairs). This suggests genuine choice comes primarily from the random slots, not the pair-matched slots.

**6. Pair scatter correlates with pattern variety.** All-PS produces only 2.7 distinct pairs per draft; Concentrated-Bridge and Degenerate-Heavy produce 4.4. More scatter means the pair counter is more fragmented, but the algorithm's "top pair" selection means only the leading pair matters.

## Recommendation

**Use 80-100% [P,S] patterns for 2-symbol cards.** The Home-Dominant-80% and All-PS configurations both pass 9/10 targets and deliver the strongest late-game S/A counts. Home-80% is preferred over pure All-PS because a small fraction of [P,P] and [S,P] patterns adds variety without meaningfully degrading performance (convergence only shifts from 6.5 to 6.9).

Avoid [P,P] patterns above 10% -- they waste pair accumulation. Strictly avoid [S,P] patterns above 10% -- they actively feed the adjacent archetype's pair counter and can fragment convergence.

For 3-symbol cards, any pattern starting with [P,S,...] produces the home pair regardless of the third symbol, so 3-symbol cards are naturally more forgiving. The third symbol position is a free design space for adding flavor resonances without affecting pair matching.

The single failing target across all configs (deck concentration too high at ~95%) is a pool composition issue, not a pattern issue -- it indicates the committed player strategy is too effective at picking on-archetype cards from 4-slot packs with escalating pair matching.
