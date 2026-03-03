# Round 2 Discussion Output — Agent 5 (Conditional Pack Enhancement)

## Simplicity Ranking (Most to Least Simple)

1. **Pair Slot Guarantee (D4)** — "Draft 3 of a pair, 1 slot guarantees that pair." Clearest threshold → deterministic result. (Refined to dual-threshold 3/7 in discussion.)
2. **Pair-Based Threshold Auto-Spend (D1)** — "Pair count hits 3, bonus card, reset." Clean threshold/reset cycle. Independently invented by two agents (D1 champion = D4 Algorithm 5).
3. **Variant C Hybrid Trigger (D5, mine)** — "2+ resonance matches in random pack → pair bonus card." Two interacting concepts (resonance trigger + pair bonus), but each is concrete.
4. **Pair-Escalation Slots (D2)** — "Probability = count/K, capped." Formula-based; players cannot intuit P=0.60 vs P=0.65. (Refined to K=6, cap=0.65.)
5. **Top-Pair Pool Seeding (D3)** — "4 pair-matched cards added to pool per pick." Invisible mechanism + reserve concept. (Simplified from proportional allocation during discussion.)

All five require explaining "ordered pair (first two symbols)." If pairs prove too complex for players, D1's unchampioned Simple Threshold Auto-Spend (single-resonance) is the universal fallback.

## Scorecard Table

Scores updated to reflect discussion math, especially Agent 3's convergence reality check (baseline ~0.7 S/A), Agent 4's pair precision analysis (~100% S-tier, not 90%), and Agent 1's convergence hierarchy.

| Goal | D1: Pair Threshold | D2: Pair-Escalation | D3: Top-Pair Seeding | D4: Dual-Threshold Guarantee | D5: Hybrid Trigger (mine) |
|------|-------------------|--------------------|-----------------------|----------------------|--------------------------|
| 1. Simple | 8 | 6 | 5 | 9 | 7 |
| 2. No Extra Actions | 10 | 10 | 10 | 10 | 10 |
| 3. Not On Rails | 7 | 4 | 8 | 5 | 9 |
| 4. No Forced Decks | 7 | 5 | 8 | 6 | 8 |
| 5. Flexible Archetypes | 7 | 5 | 6 | 5 | 8 |
| 6. Convergent (≥2.0) | 3 | 9 | 3 | 7 | 3 |
| 7. Splashable | 7 | 5 | 7 | 7 | 7 |
| 8. Open Early | 7 | 6 | 7 | 8 | 9 |
| 9. Signal Reading | 3 | 3 | 7 | 3 | 3 |
| **Total** | **59** | **53** | **61** | **60** | **64** |

**Critical convergence finding from discussion:** Agent 1 identified a convergence hierarchy: Tier 1 (per-slot targeting, D2) can target all 4 slots → ceiling ~3.0+ S/A. Tier 2 (guaranteed slots, D4) deterministically fills N slots → ceiling ~2.4 at N=2. Tier 3 (bonus injection, D1/D5) adds cards atop random baseline → ceiling ~1.0-1.7. Tier 4 (pool manipulation, D3) shifts base rates → ceiling ~1.1-1.5. Only Tier 1-2 mechanisms reliably cross 2.0. Agent 3's back-of-envelope confirms: D1 ~1.0, D3 ~1.1, D4(dual) ~2.4, D5 ~1.2, D2 ~3.0. My domain (Tier 3) is structurally limited as a standalone convergence mechanism.

**Totals are misleading.** D5 scores highest overall but is weakest on convergence. D2 scores lowest but dominates the hardest constraint. Convergence is likely a pass/fail gate, not a continuous tradeoff.

## Final Championed Algorithm

**Variant C: Hybrid Resonance-Triggered Pair Bonus.** I am switching from the original Pair Cluster Bonus (2-of-4 pair trigger, ~5% fire rate) to the hybrid variant recommended by Agents 1, 2, and 3 during discussion.

**One-sentence:** "Draw 4 random cards; if 2 or more share a primary resonance with your most-drafted resonance, add 1 bonus card whose ordered pair matches your most-drafted pair."

Fire rate: ~26-32% for committed players (single resonance = 25% of pool, P(2+ of 4) ≈ 26%). Projected S/A: ~0.7 baseline + 0.28 * 1.0 bonus precision = ~1.0. Below 2.0 but honestly reported. This algorithm's value is organic variance and natural feel, not raw convergence. It may be best as a hybrid layer atop D2 or D4.

## Modifications for Round 3 Simulation

1. **Primary:** Variant C hybrid trigger (+1 bonus card, ~26% fire rate)
2. **Aggressive:** +2 bonus cards per trigger (projects ~1.2 S/A — still marginal)
3. **D4 Hybrid:** 1 guaranteed pair slot + conditional resonance-triggered pair bonus on remaining 3 random slots. This is the most promising path to 2.0: guaranteed slot provides ~1.0 S/A floor, conditional adds ~0.2-0.3 on top, random slots add ~0.5. Total ~1.7-2.0.
4. **Baselines:** Lane Locking and auto-spend Pack Widening on same pool
5. **Measure actual pair precision** in simulated pool (Agent 2's recommendation — the single most important validation across all simulations)

## Proposed Symbol Distribution

| Symbols | % of Non-Generic | Cards |
|---------|-----------------|-------|
| 0 (generic) | — | 36 |
| 1 | 15% | 49 |
| 2 | 60% | 194 |
| 3 | 25% | 81 |

All 5 agents converged on 60-65% two-symbol / 15% one-symbol / 20-25% three-symbol. Test 30% one-symbol as robustness check per Agent 3's recommendation. The key number is 85% of cards with 2+ symbols to ensure adequate pair generation for all pair-based champions.
