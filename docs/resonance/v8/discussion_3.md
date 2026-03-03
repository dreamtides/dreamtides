# Discussion Agent 3: Slot-Locking — Post-Debate Position

## Agreed Fitness Models

| Model                             |    Sibling A-Tier     | Purpose                   |
| --------------------------------- | :-------------------: | ------------------------- |
| **Optimistic (100%)**             |     Uniform 100%      | Ceiling / backward compat |
| **Graduated Realistic (36% avg)** | 50/40/30/25% per pair | Primary target            |
| **Pessimistic (21% avg)**         | 35/25/15/10% per pair | Stress test               |
| **Hostile (8% avg)**              |      Uniform ~8%      | Floor test                |

I advocated for Hostile because Escalating Pair Lock's near-immunity to fitness
degradation is its defining strength. At Hostile (0% sibling A-tier),
pair-matching still delivers 80% precision per locked slot.

## Agreed Pool Compositions

| Pool                    | Dual-Res % | Role                              |
| ----------------------- | :--------: | --------------------------------- |
| **V7 Standard (15%)**   |    15%     | Backward comparison               |
| **40% Enriched**        |    40%     | Primary pool; consensus threshold |
| **Symbol-Rich (84.5%)** |   84.5%    | Agent 5's stretch test            |

The 40% threshold is exactly what slot-locking needs. At 15%, pair-locked slots
cannot sustain. At 40%, each archetype pair has ~18 cards -- the minimum for 2-3
locked slots across 25+ packs.

## Simplicity Ranking

1. **Narrative Gravity (Ag9)** — no counters, no modes, one sentence
2. Pair-Escalation baseline (Ag1) — proven V5 mechanism
3. **Escalating Pair Lock (Ag3)** — deterministic thresholds; two sentences but
   the mental model is intuitive: "draft pairs, unlock slots"
4. GF+PE (Ag6) — similar to mine but with guaranteed floor framing
5. Compensated Pair Alloc (Ag8) — fixed structure hides pool complexity
6. Continuous Surge (Ag2) — smooth but opaque internals
7. GPE-45 (Ag4) — two-phase with parameter switch
8. SW Surge+PF (Ag5) — weighted accumulation adds a layer
9. CSCT (Ag7) — commitment ratio is a novel concept players must grok

**Debate note:** Agent 7 argued simplicity of *mechanism* matters less than
simplicity of *experience*. I maintain that deterministic slot-locking has the
strongest mental model: "I can see my slots improving."

## Player Experience Ranking

1. **GF+PE (Ag6)** — monotonic + guaranteed floor is the gold standard for feel
2. **Escalating Pair Lock (Ag3)** — I rank myself second. Graduated unlocking
   creates clear progression milestones. The 85% lock probability generates
   organic variance within a structural guarantee. Locked slots are visible and
   predictable (in a good way).
3. CSCT (Ag7) — smooth but lacks perceptual anchors
4. Continuous Surge (Ag2) — smooth probability is good but indistinguishable
   from randomness to many players
5. GPE-45 (Ag4) — solid two-phase ramp
6. Narrative Gravity (Ag9) — organic but the player cannot see the mechanism
   working
7. Pair-Escalation baseline (Ag1) — probabilistic; no structural floor
8. Compensated Pair Alloc (Ag8) — fixed structure from pick 4 may feel abrupt
9. SW Surge+PF (Ag5) — surge/floor alternation persists

## Scorecard (Graduated Realistic, 40% Enriched Pool)

| Algorithm                |   M1   |   M2    |    M3    |   M4    |   M5    |   M6    |   M7    |   M8    |    M9    |   M10    |
| ------------------------ | :----: | :-----: | :------: | :-----: | :-----: | :-----: | :-----: | :-----: | :------: | :------: |
| Pair-Esc. 40% (Ag1)      |   4+   |   \<2   |   2.08   |   0.5   |   6.3   |   80%   |   30%   |   11%   |   0.90   |   Pass   |
| Cont. Surge (Ag2)        |   4+   |   \<2   |   1.95   |   0.6   |   7.5   |   75%   |   35%   |   12%   |   0.88   |   Pass   |
| **Esc. Pair Lock (Ag3)** | **4+** | **\<2** | **2.15** | **0.5** | **5.5** | **78%** | **32%** | **11%** | **0.85** | **Pass** |
| GPE-45 (Ag4)             |   4+   |   \<2   |   2.05   |   0.6   |   6.0   |   77%   |   33%   |   12%   |   0.88   |   Pass   |
| SW Surge+PF (Ag5)        |   4+   |   \<2   |   2.20   |   0.4   |   5.5   |   80%   |   30%   |   11%   |   0.92   | Marginal |
| GF+PE (Ag6)              |   4+   |   \<2   |   2.00   |   0.6   |   7.0   |   75%   |   35%   |   12%   |   0.82   |   Pass   |
| CSCT (Ag7)               |   4+   |   \<2   |   2.00   |   0.6   |   6.5   |   74%   |   36%   |   12%   |   0.78   |   Pass   |
| Comp. Pair Alloc (Ag8)   |   4+   |   \<2   |   2.10   |   0.5   |   4.0   |   78%   |   33%   |   11%   |   0.75   |   Pass   |
| Narrative Gravity (Ag9)  |   3+   |   \<2   |   1.70   |   0.7   |   8.0   |   72%   |   38%   |   13%   |   0.95   |   Pass   |

## Final Champion: Escalating Pair Lock (modified)

Escalating Pair Lock remains my champion. During debate, several agents
acknowledged that its graduated unlock mechanism is convergent with their own
designs (Agent 6's GF+PE is structurally similar, Agent 4's GPE-45 uses a
similar ramp). The key differentiator is *determinism*: once a slot locks, it
stays locked. This creates a ratchet effect that is psychologically satisfying
and structurally prevents quality regression.

**Cross-pollination adopted:**

- **Agent 6's guaranteed floor:** Add a permanent 1-slot pair-matched draw from
  pick 3+, even before the first threshold. This means the progression is: 1
  slot (pick 3+) -> 2 slots (threshold 4) -> 3 slots (threshold 7). The
  guaranteed floor eliminates dead packs before the first lock fires.
- **Agent 2's probability jitter on locked slots:** Instead of a fixed 85% lock
  probability, use 80-90% with per-pack jitter. This slightly increases M9
  variance without changing the average.

**Rejected:** Agent 9's pool contraction (incompatible with stable slot draws)
and Agent 7's commitment ratio (replaces clean thresholds with a
harder-to-internalize function).

## Planned Modifications for Simulation

1. Three-stage escalation: 1 slot (pick 3+), 2 slots (pair count >= 4), 3 slots
   (pair count >= 7).
2. Lock probability: 80-90% per pack with uniform jitter.
3. Run all 4 fitness models, all 3 pool compositions.
4. Per-archetype M3 reporting (8 rows).
5. Track consecutive bad packs and pack quality distribution.
6. Parameter sweep on thresholds: {3,5,7}, {4,7,10}, {5,9,13}.
7. Compare M9 stddev under jittered vs. fixed lock probability.

**Confidence:** M3=2.15 is the second-highest projection. Unlike Agent 5, my
algorithm passes M10 structurally and uses a simpler description. The main risk
is M9 variance (0.85) being borderline; jitter should address this.
