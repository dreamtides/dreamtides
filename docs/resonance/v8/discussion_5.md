# Discussion Agent 5: Symbol-Rich — Post-Debate Position

## Agreed Fitness Models

| Model                             |    Sibling A-Tier     | Purpose        |
| --------------------------------- | :-------------------: | -------------- |
| **Optimistic (100%)**             |     Uniform 100%      | Ceiling test   |
| **Graduated Realistic (36% avg)** | 50/40/30/25% per pair | Primary target |
| **Pessimistic (21% avg)**         | 35/25/15/10% per pair | Stress test    |
| **Hostile (8% avg)**              |      Uniform ~8%      | Floor test     |

I support the four-model framework. Symbol-rich architectures should shine under
Hostile because triple-match filtering achieves ~95% S-tier precision by
concentrating draws on home-archetype cards, making sibling fitness nearly
irrelevant.

## Agreed Pool Compositions

| Pool                    | Dual-Res % | Role                                    |
| ----------------------- | :--------: | --------------------------------------- |
| **V7 Standard (15%)**   |    15%     | Comparison baseline                     |
| **40% Enriched**        |    40%     | Consensus primary pool                  |
| **Symbol-Rich (84.5%)** |   84.5%    | My proposed pool; optional stretch test |

I am disappointed the group relegated my 84.5% pool to "optional stretch test."
Triple-matching enables ~95% precision versus ~85% for pair-matching --
translating to +0.2-0.3 M3 under Pessimistic fitness. However, I acknowledge
requiring 3 meaningful symbols per card is a substantial design burden.

**Concession:** I will run on both 40% Enriched and Symbol-Rich pools. If 40%
delivers M3 >= 2.0 under Graduated Realistic, the simpler pool suffices. The
Symbol-Rich pool is recommended only if 40% falls short under Pessimistic.

## Simplicity Ranking

1. Narrative Gravity (Ag9) — no counters
2. Pair-Escalation baseline (Ag1) — one sentence
3. Escalating Pair Lock (Ag3) — threshold-based
4. GF+PE (Ag6) — guaranteed floor + escalation
5. Compensated Pair Alloc (Ag8) — fixed structure
6. Continuous Surge (Ag2) — probabilistic
7. GPE-45 (Ag4) — two-phase
8. **SW Surge + Pair Floor (Ag5)** — weighted accumulation + pair filtering +
   surge. I acknowledge this is toward the complex end. The weighted symbol
   system adds a layer that most other algorithms avoid.
9. CSCT (Ag7) — commitment ratio + jitter + bias

## Player Experience Ranking

1. GF+PE (Ag6) — best structural guarantees
2. CSCT (Ag7) — smoothest ramp
3. Escalating Pair Lock (Ag3) — clear milestones
4. GPE-45 (Ag4) — smooth two-phase
5. Continuous Surge (Ag2) — smooth probabilistic
6. Narrative Gravity (Ag9) — organic but passive
7. **SW Surge + Pair Floor (Ag5)** — retains surge/floor pattern. I rank myself
   7th honestly. The debate convinced me that bimodal delivery is a genuine
   experiential problem, not just a theoretical concern. Agent C's loss aversion
   analysis is compelling.
8. Pair-Escalation baseline (Ag1) — no floor; dead pack risk
9. Compensated Pair Alloc (Ag8) — fixed from pick 4; no progression feel

## Scorecard (Graduated Realistic, 40% Enriched Pool)

| Algorithm               |   M1   |   M2    |    M3    |   M4    |   M5    |   M6    |   M7    |   M8    |    M9    |   M10    |
| ----------------------- | :----: | :-----: | :------: | :-----: | :-----: | :-----: | :-----: | :-----: | :------: | :------: |
| Pair-Esc. 40% (Ag1)     |   4+   |   \<2   |   2.08   |   0.5   |   6.3   |   80%   |   30%   |   11%   |   0.90   |   Pass   |
| Cont. Surge (Ag2)       |   4+   |   \<2   |   1.95   |   0.6   |   7.5   |   75%   |   35%   |   12%   |   0.88   |   Pass   |
| Esc. Pair Lock (Ag3)    |   4+   |   \<2   |   2.15   |   0.5   |   5.5   |   78%   |   32%   |   11%   |   0.85   |   Pass   |
| GPE-45 (Ag4)            |   4+   |   \<2   |   2.05   |   0.6   |   6.0   |   77%   |   33%   |   12%   |   0.88   |   Pass   |
| **SW Surge+PF (Ag5)**   | **4+** | **\<2** | **2.20** | **0.4** | **5.5** | **80%** | **30%** | **11%** | **0.92** | **Marg** |
| GF+PE (Ag6)             |   4+   |   \<2   |   2.00   |   0.6   |   7.0   |   75%   |   35%   |   12%   |   0.82   |   Pass   |
| CSCT (Ag7)              |   4+   |   \<2   |   2.00   |   0.6   |   6.5   |   74%   |   36%   |   12%   |   0.78   |   Pass   |
| Comp. Pair Alloc (Ag8)  |   4+   |   \<2   |   2.10   |   0.5   |   4.0   |   78%   |   33%   |   11%   |   0.75   |   Pass   |
| Narrative Gravity (Ag9) |   3+   |   \<2   |   1.70   |   0.7   |   8.0   |   72%   |   38%   |   13%   |   0.95   |   Pass   |

## Final Champion: Symbol-Weighted Surge + Pair Floor (modified)

I retain my champion but adopt the debate's most compelling criticism: **replace
the surge/floor binary with graduated slot allocation.** Instead of
threshold-triggered bulk spending, I will use Agent 3's escalating model applied
to pair-matched slots with my weighted accumulation system:

**Modified mechanism:** Track weighted pair counters (AAB cards earn +3 for
primary resonance, +1 for secondary; ABC cards earn +2/+1/+1). Pair-matched
slots unlock progressively: 1 slot at counter=3, 2 slots at counter=6, 3 slots
at counter=9. No spending, no alternation. Remaining slots random.

This hybrid preserves my contributions (weighted accumulation from 3-symbol
cards, faster convergence) while adopting the smooth delivery framework that
Agents 3, 6, and 7 advocated. The result is essentially Escalating Pair Lock
with richer token accumulation.

**Why retain my champion?** Weighted accumulation converges 2-3 picks faster
than standard pair counting, worth ~+0.1 M3 in the picks 6-10 window.

## Planned Modifications for Simulation

1. Graduated version: weighted pair counters, thresholds 3/6/9 for 1/2/3
   pair-matched slots.
2. Run on 40% Enriched (primary) and Symbol-Rich 84.5% (stretch) pools.
3. All 4 fitness models; compare directly to Agent 3 to isolate weighted
   accumulation value.
4. Track M5 convergence under both pools; report M10 and per-archetype M3.

**Honest assessment:** At 40% dual-res with ~85% precision, the Symbol-Rich
pool's extra +10% matters primarily under Hostile fitness. I maintain it as a
fallback for worst-case scenarios.
