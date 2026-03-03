# Discussion Agent 1: Baselines — Post-Debate Position

## Agreed Fitness Models

The group converged on four shared models for simulation:

| Model                             |    Sibling A-Tier     | Rationale                                                |
| --------------------------------- | :-------------------: | -------------------------------------------------------- |
| **Optimistic (100%)**             |     Uniform 100%      | Backward compatibility with V5-V7                        |
| **Graduated Realistic (36% avg)** | 50/40/30/25% per pair | V8 primary target; reflects per-pair mechanical distance |
| **Pessimistic (21% avg)**         | 35/25/15/10% per pair | Stress test; "low design effort" scenario                |
| **Hostile (8% avg)**              |      Uniform ~8%      | Floor test; archetypes designed in isolation             |

I pushed for Hostile inclusion because baseline data at the floor is essential.
Agent B's research showing Flash/Ramp at 10-20% natural overlap means per-pair
Hostile is plausible for the worst pairs.

## Agreed Pool Compositions

| Pool             |           Dual-Res %            | Symbols                   | Primary Use                                  |
| ---------------- | :-----------------------------: | ------------------------- | -------------------------------------------- |
| **V7 Standard**  |               15%               | 0-3 (avg ~1.5)            | Backward comparison; minimum-change scenario |
| **40% Enriched** |               40%               | 2+ mandatory              | Primary test pool; consensus sweet spot      |
| **Symbol-Rich**  | 84.5% (Agent 5's 3-symbol pool) | Exactly 3 per non-generic | Stretch test for symbol-dependent algorithms |

I argued strongly for keeping V7 Standard as a mandatory test pool. Several
agents (3, 5, 6) wanted to drop it since "nothing works on V7 pool." But the
designer needs to see the cost of not changing the pool -- that is the
baseline's entire purpose. The group agreed.

The 40% Enriched pool is consensus. Agent 5's 84.5% pool is included as an
optional stretch test, not a recommendation -- requiring every card to have 3
symbols is a substantial design burden. Agent 8's non-uniform distribution (more
dual-res for low-overlap pairs) is an interesting variant but adds complexity;
it will be tested as a parameter sweep within 40% Enriched.

## Simplicity Ranking (simplest to most complex)

1. **Narrative Gravity (Agent 9)** — one sentence; no counters, no modes
2. **Pair-Escalation on 40% pool (Agent 1)** — one sentence; V5 mechanism
   unchanged
3. **Escalating Pair Lock (Agent 3)** — two sentences; threshold-based,
   deterministic
4. **GF+PE (Agent 6)** — two sentences; guaranteed floor + escalation
5. **Compensated Pair Allocation (Agent 8)** — one sentence player-facing, but
   non-uniform pool adds hidden complexity
6. **Continuous Surge (Agent 2)** — one sentence possible but probabilistic
   internals are complex
7. **GPE-45 (Agent 4)** — two sentences; two-phase ramp
8. **Symbol-Weighted Surge + Pair Floor (Agent 5)** — two sentences; weighted
   accumulation + pair filtering
9. **CSCT (Agent 7)** — not describable in one sentence; commitment ratio +
   jitter + bias

## Player Experience Ranking (best feel to worst)

1. **GF+PE (Agent 6)** — monotonic escalation, no dead packs, unimodal
2. **CSCT (Agent 7)** — smooth ramp, no modes; slight M9 concern
3. **Escalating Pair Lock (Agent 3)** — gradual locks, structural floor; mild
   repetition risk
4. **Narrative Gravity (Agent 9)** — organic feel; early packs may feel aimless
5. **GPE-45 (Agent 4)** — smooth two-phase ramp; phase transition slightly
   perceptible
6. **Continuous Surge (Agent 2)** — eliminates bimodal but probabilistic
   variance can produce occasional dead packs
7. **Compensated Pair Allocation (Agent 8)** — consistent but fixed 2+1+1 may
   feel sterile
8. **Pair-Escalation baseline (Agent 1)** — V5 mechanism is proven but
   M9/concentration risks
9. **Symbol-Weighted Surge + Pair Floor (Agent 5)** — retains surge/floor
   alternation despite pair upgrade

## Scorecard (Graduated Realistic Fitness, 40% Enriched Pool)

| Algorithm               | M1  | M2  |  M3  | M4  | M5  | M6  | M7  |   M8   |  M9  |   M10    |
| ----------------------- | :-: | :-: | :--: | :-: | :-: | :-: | :-: | :----: | :--: | :------: |
| Pair-Esc. 40% (Ag1)     | 4+  | \<2 | 2.08 | 0.5 | 6.3 | 80% | 30% | 10-15% | 0.90 |   Pass   |
| Continuous Surge (Ag2)  | 4+  | \<2 | 1.95 | 0.6 | 7.5 | 75% | 35% |  12%   | 0.85 |   Pass   |
| Esc. Pair Lock (Ag3)    | 4+  | \<2 | 2.15 | 0.5 | 5.5 | 78% | 32% |  11%   | 0.85 |   Pass   |
| GPE-45 (Ag4)            | 4+  | \<2 | 2.05 | 0.6 | 6.0 | 77% | 33% |  12%   | 0.88 |   Pass   |
| SW Surge+PF (Ag5)       | 4+  | \<2 | 2.20 | 0.4 | 5.5 | 80% | 30% |  11%   | 0.92 | Marginal |
| GF+PE (Ag6)             | 4+  | \<2 | 2.00 | 0.6 | 7.0 | 75% | 35% |  12%   | 0.82 |   Pass   |
| CSCT (Ag7)              | 4+  | \<2 | 2.00 | 0.6 | 6.5 | 74% | 36% |  12%   | 0.78 |   Pass   |
| Comp. Pair Alloc (Ag8)  | 4+  | \<2 | 2.10 | 0.5 | 4.0 | 78% | 33% |  11%   | 0.75 |   Pass   |
| Narrative Gravity (Ag9) | 3+  | \<2 | 1.70 | 0.7 | 8.0 | 72% | 38% |  13%   | 0.95 |   Pass   |

## Final Champion: Pair-Escalation on 40% Dual-Res Pool (M3=2.08)

My reference baseline remains the strongest "simple" algorithm.
Pair-Escalation's V5 mechanism is proven, its one-sentence description is clean,
and it is the only algorithm with historical simulation data (V5) to validate
projections against. On the 40% pool, it crosses 2.0 under Graduated Realistic.

## Planned Modifications for Simulation

1. **Add per-pair fitness** instead of uniform -- validate that pair-matching
   actually insulates worst-case archetypes as projected.
2. **Test cap=0.50 and cap=0.60** -- the cap determines the M3/M4/M6 tradeoff.
3. **Run on all 3 pool compositions** to establish the complete baseline matrix.
4. **Report per-archetype M3** (8 rows) to expose the worst-archetype gap.
5. **Incorporate Agent 6's guaranteed floor idea**: add a permanent 1-slot
   pair-matched floor from pick 3+ to eliminate dead packs. This is the single
   best cross-pollination from the debate -- it costs almost nothing in M3 but
   structurally prevents the 10.5% dead-pack rate that pure Pair-Escalation
   inherits from V5.
