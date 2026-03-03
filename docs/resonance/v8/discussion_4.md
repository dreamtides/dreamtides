# Discussion Agent 4: Pair-Matching — Post-Debate Position

## Agreed Fitness Models

| Model                             |    Sibling A-Tier     | Purpose                            |
| --------------------------------- | :-------------------: | ---------------------------------- |
| **Optimistic (100%)**             |     Uniform 100%      | Backward compat                    |
| **Graduated Realistic (36% avg)** | 50/40/30/25% per pair | Primary target                     |
| **Pessimistic (21% avg)**         | 35/25/15/10% per pair | Stress test; my critical threshold |
| **Hostile (8% avg)**              |      Uniform ~8%      | Floor test                         |

I pushed for Pessimistic as the *design* target: the designer said V7's Moderate
may be optimistic, and Surge+Floor drops from 1.85 to 1.42 at Pessimistic. The
group compromised: Graduated Realistic is primary, but algorithms must pass M3
\>= 1.8 under Pessimistic. GPE-45 projects 1.99, comfortably above.

## Agreed Pool Compositions

| Pool                    | Dual-Res % | Role                                     |
| ----------------------- | :--------: | ---------------------------------------- |
| **V7 Standard (15%)**   |    15%     | Comparison only                          |
| **40% Enriched**        |    40%     | Consensus primary; enables pair-matching |
| **Symbol-Rich (84.5%)** |   84.5%    | Optional stretch test                    |

I originally proposed 45% dual-res but accepted 40%. GPE-45 compensates with
tri-resonance cards, bringing the effective pair-matchable pool to 24-26 per
archetype. Agent 5's 84.5% pool enables ~95% precision via triple-matching, but
pair-matching at ~85% is already sufficient. The incremental gain does not
justify mandating 3 symbols on every card.

## Simplicity Ranking

1. Narrative Gravity (Ag9) — fewest moving parts
2. Pair-Escalation baseline (Ag1) — one sentence; proven
3. Escalating Pair Lock (Ag3) — threshold-based; clean mental model
4. **GPE-45 (Ag4)** — two sentences required for two-phase structure; justified
   by experience improvement
5. GF+PE (Ag6) — similar complexity to mine with different framing
6. Continuous Surge (Ag2) — probabilistic internals harder to reason about
7. Compensated Pair Alloc (Ag8) — fixed structure but non-uniform pool adds
   hidden complexity
8. SW Surge+PF (Ag5) — weighted accumulation + pair filtering + surge/floor
9. CSCT (Ag7) — commitment ratio is conceptually novel but explanation-heavy

## Player Experience Ranking

1. GF+PE (Ag6) — monotonic escalation is the player experience gold standard
2. CSCT (Ag7) — smooth commitment ramp; no perceivable modes
3. **GPE-45 (Ag4)** — two-phase ramp with smooth intra-phase progression; I rank
   myself third because the phase transition at pick 13 is smoothable via
   interpolation
4. Escalating Pair Lock (Ag3) — good progression but threshold jumps are
   perceptible
5. Continuous Surge (Ag2) — smooth but probabilistic variance can feel arbitrary
6. Narrative Gravity (Ag9) — organic but early packs feel unassisted
7. Compensated Pair Alloc (Ag8) — consistent from pick 4 but no progression feel
8. Pair-Escalation baseline (Ag1) — no floor; occasional dead packs
9. SW Surge+PF (Ag5) — retains surge/floor alternation

## Scorecard (Graduated Realistic, 40% Enriched Pool)

| Algorithm               |   M1   |   M2    |    M3    |   M4    |   M5    |   M6    |   M7    |   M8    |    M9    |   M10    |
| ----------------------- | :----: | :-----: | :------: | :-----: | :-----: | :-----: | :-----: | :-----: | :------: | :------: |
| Pair-Esc. 40% (Ag1)     |   4+   |   \<2   |   2.08   |   0.5   |   6.3   |   80%   |   30%   |   11%   |   0.90   |   Pass   |
| Cont. Surge (Ag2)       |   4+   |   \<2   |   1.95   |   0.6   |   7.5   |   75%   |   35%   |   12%   |   0.88   |   Pass   |
| Esc. Pair Lock (Ag3)    |   4+   |   \<2   |   2.15   |   0.5   |   5.5   |   78%   |   32%   |   11%   |   0.85   |   Pass   |
| **GPE-45 (Ag4)**        | **4+** | **\<2** | **2.05** | **0.6** | **6.0** | **77%** | **33%** | **12%** | **0.88** | **Pass** |
| SW Surge+PF (Ag5)       |   4+   |   \<2   |   2.20   |   0.4   |   5.5   |   80%   |   30%   |   11%   |   0.92   | Marginal |
| GF+PE (Ag6)             |   4+   |   \<2   |   2.00   |   0.6   |   7.0   |   75%   |   35%   |   12%   |   0.82   |   Pass   |
| CSCT (Ag7)              |   4+   |   \<2   |   2.00   |   0.6   |   6.5   |   74%   |   36%   |   12%   |   0.78   |   Pass   |
| Comp. Pair Alloc (Ag8)  |   4+   |   \<2   |   2.10   |   0.5   |   4.0   |   78%   |   33%   |   11%   |   0.75   |   Pass   |
| Narrative Gravity (Ag9) |   3+   |   \<2   |   1.70   |   0.7   |   8.0   |   72%   |   38%   |   13%   |   0.95   |   Pass   |

## Final Champion: GPE-45 Graduated Pair-Escalation (modified)

I retain GPE-45 but adopt three modifications from the debate:

1. **Smooth the phase transition.** Agent 6 and Agent 7 both criticized the
   discrete jump at pick 13. I will interpolate linearly over picks 11-15: the
   cap ramps from 0.35 to 0.55 and the divisor from 8 to 5 across those 5 picks.
   The player experience is a smooth acceleration rather than a step function.

2. **Add Agent 6's guaranteed floor.** From pick 3+, ensure at least 1
   pair-matched slot per pack regardless of pair counter. This eliminates dead
   packs structurally. Cost: negligible M3 change (the floor was already ~1 slot
   by pick 3 for committed drafters).

3. **Report per-archetype M3 with per-pair fitness.** Agent B's calibration
   showing Flash/Ramp at 25% is critical. GPE-45's pair-matching should compress
   the archetype gap, but I need to verify: does pair-matching actually deliver
   85% precision for Flash/Ramp when the pair pool is only 18 cards with
   potentially lower home-archetype concentration?

## Planned Modifications for Simulation

1. Implement smoothed two-phase ramp (interpolation over picks 11-15).
2. Guaranteed 1-slot floor from pick 3+.
3. Run all 4 fitness models on 40% Enriched and V7 Standard pools.
4. Per-archetype M3 and worst-archetype M3 tracking.
5. Pack quality distribution histogram.
6. Parameter sweep: cap values {0.35/0.55, 0.40/0.60, 0.30/0.50} x divisors
   {8/5, 6/4, 10/6}.
7. Test the "Aggressive" variant (6/0.40 -> 4/0.60) which projects ~2.15 under
   Pessimistic but may over-converge.

**Key question:** Does smoothed transition feel better than discrete milestones?
Agent 3 argues milestones give clear feedback. Simulation should test both.
