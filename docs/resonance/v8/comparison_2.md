# Comparison Agent 2: Continuous Surge Perspective

## Scorecard at Each Fitness Level (Algorithm x M3, 40% Enriched)

| Algorithm             | Optimistic | Grad. Real. | Pessimistic | Hostile | Degradation |
| --------------------- | :--------: | :---------: | :---------: | :-----: | :---------: |
| 1. Pair-Esc. Baseline |    2.34    |    2.16     |    2.12     |  2.08   |     11%     |
| 2. Continuous Surge   |    3.10    |    2.48     |    2.43     |  2.25   |     27%     |
| 3. Esc. Pair Lock     |    1.98    |    1.50     |    1.46     |  1.31   |     34%     |
| 4. GPE-45             |    2.73    |    2.25     |    2.21     |  2.05   |     25%     |
| 5. Sym-Weighted       |    2.88    |    2.50     |    2.49     |  2.34   |     19%     |
| 6. GF+PE              |    2.57    |    1.72     |    1.58     |  1.34   |     48%     |
| 7. CSCT               |    3.07    |    2.92     |    2.88     |  2.85   |     7%      |
| 8. Comp. Pair Alloc   |    2.16    |    1.45     |    1.40     |  1.29   |     40%     |
| 9. Narr. Gravity      |    3.39    |    2.75     |    2.59     |  2.49   |     27%     |

## The Core Tension: M3 vs M10 vs M6

The simulation results reveal a three-way tradeoff that no algorithm has solved:

- **High M3 + good M10** (CSCT): M6 = 99%, M9 = 0.68. On rails.
- **High M3 + good M6** (Continuous Surge, Narrative Gravity): M10 fails (3.8,
  3.3).
- **Good M10 + good M6**: No algorithm achieves this with M3 >= 2.0.

This is the fundamental finding. The math is unforgiving: smooth delivery
requires aggressive targeting, which kills variety. Variety requires randomness,
which creates drought streaks.

## Player Experience Rating (1-10)

| Algorithm           | Rating | Justification                                                       |
| ------------------- | :----: | ------------------------------------------------------------------- |
| 1. Pair-Esc.        |   5    | Reliable but occasionally punishing; wide spread (p25=1.0)          |
| 2. Continuous Surge |   6    | Unimodal is better than bimodal but probabilistic droughts persist  |
| 3. Esc. Pair Lock   |   1    | Half of all drafts deliver wrong-archetype cards; disqualifying     |
| 4. GPE-45           |   4    | The bootstrapping dead zone (picks 6-12) is genuinely frustrating   |
| 5. Sym-Weighted     |   7    | Graduated ramp on symbol-rich pool feels natural; best overall feel |
| 6. GF+PE            |   3    | Ramp at 1.13 M3 is a broken experience; floor concept fails         |
| 7. CSCT             |   6    | Smooth but monotonous; every pack looks the same after pick 5       |
| 8. Comp. Pair       |   2    | Misalignment disaster; signal-reader trace at 0.40 is devastating   |
| 9. Narr. Gravity    |   8    | The "funnel" metaphor is intuitive; packs get better monotonically  |

I rate Narrative Gravity highest on feel despite its M10 = 3.3 failure, because
the *quality of the failure mode* matters. Narrative Gravity's bad streaks are
early (picks 6-10, before the pool contracts), and quality monotonically
improves. Continuous Surge's bad streaks are random and can strike at any point
post-commitment.

## Biggest Strength and Weakness

| Algorithm           | Strength                                     | Weakness                                               |
| ------------------- | -------------------------------------------- | ------------------------------------------------------ |
| 2. Continuous Surge | Unimodal distribution; works on V7 pool too  | Per-archetype gap: Ramp 1.55 vs Warriors 2.60          |
| 7. CSCT             | Near-perfect smoothness and fitness immunity | M6 = 99% makes it a solved puzzle, not a draft         |
| 9. Narr. Gravity    | All 8 archetypes above 2.0; monotonic ramp   | Late-draft pool shrinks to 10-15 cards; feels limiting |

## KEY QUESTION 1: M3 >= 2.0 under harshest fitness?

Five algorithms clear M3 >= 2.0 under Hostile. But the question should be
stricter: which clears 2.0 for the **worst archetype** under Hostile? Only
Narrative Gravity achieves this (Flash = 2.16 under Hostile). CSCT likely does
too (per-archetype spread of only 0.08), but its M6 failure disqualifies it.
Continuous Surge fails here: Ramp = 1.55 even under Graduated.

The per-archetype gap is the true binding constraint. Algorithms that pass M3 in
aggregate but leave Flash or Ramp below 2.0 are hiding a broken experience for
those archetypes.

## KEY QUESTION 2: Best feel?

Narrative Gravity's monotonic contraction is the most satisfying mechanic to
play with. The player sees the draft world narrow around their choices -- this
is an inherently roguelike experience. CSCT is smoother on paper but feels like
autopilot. Symbol-Weighted Escalation on the symbol-rich pool is the runner-up:
its graduated ramp produces a natural-feeling quality curve.

## Can CSCT Be Rescued?

I am skeptical. CSCT's M6 = 99% is not a tuning problem -- it is structural. The
commitment ratio rises to 1.0 by pick 5, maxing out pair-matched slots
immediately. Capping at 2 slots helps, but the commitment ratio calculation
itself locks in too fast. You would need to fundamentally change how commitment
is measured -- perhaps using a sliding window rather than cumulative ratio --
which makes it a different algorithm entirely.

A more promising direction: **Narrative Gravity with a floor mechanism.** Add a
guaranteed minimum 1 pair-matched slot from pick 3+, independent of pool
contraction. This addresses the M10 failure (early post-commitment bad packs)
while preserving the contraction mechanic's variety and monotonic ramp.

## Proposed Best Algorithm

**Narrative Gravity + Pair-Matched Floor (Hybrid)**

- Pool contracts as in Agent 9 (12% per pick from pick 4)
- Guaranteed 1 pair-matched slot from pick 3+ (addresses M10)
- Pool: 40% Enriched (128 dual-res cards)
- Projected M3: ~2.4-2.6 under Graduated (floor adds stability without
  over-concentrating)
- Projected M10: ~2.0 (floor prevents early dead packs)
- M6: ~78-82% (contraction provides convergence, floor adds consistency)

**Minimum fitness: Graduated Realistic (36% avg).** Under Pessimistic, Narrative
Gravity alone delivers 2.59; the floor should maintain this.

**Minimum pool change: 40% dual-resonance (74 additional cards).** The enriched
pool is non-negotiable for pair-matching viability.

## Recommendations to the Card Designer

1. The 40% dual-resonance pool is the single most important change. Without it,
   no algorithm consistently clears 2.0 under realistic fitness.
2. Consider the symbol-rich pool (84.5% dual-res, 3 symbols per card) if budget
   allows. It provides the best fitness insulation and enables the strongest
   algorithms. But 40% enriched is the minimum viable target.
3. Bridge cards for Flash/Ramp and Blink/Storm remain important. Even with
   Narrative Gravity's fitness-bypassing contraction, bridge cards improve the
   early-draft experience before the pool has contracted.
4. The draft system can be explained simply: "As you draft, the game learns what
   you want and shows you more of it." No counters, thresholds, or visible
   mechanisms required.
