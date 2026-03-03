# Comparison Agent 7: CSCT Perspective

## Defending CSCT's Position

CSCT is the only algorithm that passes M10 (\<= 2) on the 40% Enriched pool. It
is also the most fitness-immune algorithm tested (0.23 M3 degradation from
Optimistic to Hostile). Its per-archetype equity is unmatched (0.08 M3 spread).
These are not marginal advantages -- they represent structural superiority in
the two dimensions V8 elevated: smoothness and robustness.

The M6 = 99% failure is real. But I contend it is fixable through detuning,
whereas other algorithms' M10 failures are structural.

## Scorecard: The Tradeoff Triangle

| Algorithm         | M3 (GR) |   M10   | M6  |    M9    |      "Triangle Score"      |
| ----------------- | :-----: | :-----: | :-: | :------: | :------------------------: |
| 7. CSCT           |  2.92   | **2.0** | 99% |   0.68   |      M3+M10 dominate       |
| 9. Narr. Gravity  |  2.75   |   3.3   | 85% | **1.21** |     M3+M6+M9 dominate      |
| 5. Sym-Weighted   |  2.50   |   4.3   | 83% | **1.18** |   Balanced but M10 weak    |
| 2. Cont. Surge    |  2.48   |   3.8   | 85% |   0.78   | M3 decent, M10/M9 mediocre |
| 4. GPE-45         |  2.25   |   8.2   | 67% |   0.51   |      M10/M9 both fail      |
| 1. Pair-Esc.      |  2.16   |   8.0   | 89% | **1.00** |  M3+M9 decent, M10 fails   |
| 6. GF+PE          |  1.72   |   6.3   | 76% |   0.74   |    Everything mediocre     |
| 3. Esc. Pair Lock |  1.50   |   8.8   | 64% | **1.23** | M3 fails, alignment broken |
| 8. Comp. Pair     |  1.45   |   6.9   | 62% | **0.83** | M3 fails, alignment broken |

The "triangle" is M3 vs M10 vs M6. No algorithm lives in the center of all
three. CSCT occupies the M3+M10 corner. Narrative Gravity occupies the M3+M6
corner. The question is which corner matters more.

## Player Experience Rating (1-10)

| Algorithm         | Rating | Justification                                                         |
| ----------------- | :----: | --------------------------------------------------------------------- |
| 7. CSCT           |   7    | I rate myself honestly: the smoothness is *real* player value         |
| 9. Narr. Gravity  |   7    | Tied: monotonic ramp is excellent, but M10=3.3 means early dead packs |
| 5. Sym-Weighted   |   6    | Good ramp on symbol-rich pool; Blink weakness is unfortunate          |
| 2. Cont. Surge    |   5    | Ramp archetype at 1.55 is a broken experience for 12.5% of players    |
| 1. Pair-Esc.      |   5    | Reliable but worst streak = 8 means some drafts feel terrible         |
| 4. GPE-45         |   4    | 8-pick dead zone is too long                                          |
| 6. GF+PE          |   3    | Floor concept fails in practice                                       |
| 8. Comp. Pair     |   2    | Misalignment                                                          |
| 3. Esc. Pair Lock |   1    | Broken                                                                |

I rate CSCT and Narrative Gravity equally because they represent two
philosophies:

- **CSCT:** "Every pack is good, every time. You can rely on the system."
- **Narrative Gravity:** "Your packs get better over time. The journey has
  drama."

Both are legitimate player experiences. CSCT's M6 failure means the player is on
rails, but Hearthstone's 2025 Arena revamp showed that guided convergence can be
welcomed if it feels like support. Narrative Gravity's M10 failure means 3-4
consecutive bad packs early in the commitment phase, which undermines trust in
the system.

## KEY QUESTION 1: M3 >= 2.0 under harshest fitness?

CSCT achieves M3 = 2.85 under Hostile fitness with per-archetype spread of only
0.08. This is the strongest result by a wide margin. Narrative Gravity achieves
2.49 under Hostile -- excellent, but 0.36 lower. The gap widens at lower
fitness, demonstrating CSCT's superior fitness immunity.

If we restrict to "M3 >= 2.0 for worst archetype under Hostile": CSCT (worst =
2.80) beats Narrative Gravity (worst = 2.09 for Flash/Blink). CSCT delivers the
most equitable experience across archetypes under every fitness model.

## KEY QUESTION 2: Can CSCT be rescued?

**Yes, and here is how.** The M6 = 99% problem stems from three factors:

1. Commitment ratio reaches 1.0 by pick 5
2. Multiplier 5 converts ratio 0.8 into 4.0 pair-matched slots (all slots)
3. Once all slots are pair-matched, the player never sees off-archetype cards

**Fix 1: Hard cap at 2 pair-matched slots.** Even at ratio 1.0, only 2 of 4
slots are pair-matched. The other 2 are R1-filtered (62-75% S/A precision) or
random. Projected: M3 drops to ~2.4 (still well above 2.0), M6 drops to ~75-80%,
M9 recovers to ~0.8.

**Fix 2: Slow-start ratio.** Use `effective_ratio = max(0, (ratio - 0.4) / 0.6)`
so that targeting does not begin until ratio exceeds 0.4 (roughly pick 3-4).
This delays convergence, improving M5 and M2.

**Fix 3: Random injection.** Force 1 purely random slot in every pack regardless
of ratio. This guarantees splash (M4) and variety (M6/M9) at the cost of 0.25 M3
per pack.

**Detuned CSCT projected performance (Fix 1 + Fix 3):**

| Metric | Original CSCT | Detuned  | Target |
| ------ | :-----------: | :------: | :----: |
| M3     |     2.92      | ~2.3-2.4 | >= 2.0 |
| M6     |      99%      | ~72-78%  | 60-90% |
| M9     |     0.68      |  ~0.82   | >= 0.8 |
| M10    |       2       |   ~3-4   | \<= 2  |
| M4     |     0.47      |   ~1.0   | >= 0.5 |

The detuned variant sacrifices M10 (which was CSCT's unique advantage) to rescue
M6 and M9. This is the fundamental problem: CSCT's M10 performance is a
consequence of its over-concentration, not independent of it.

## Proposed Best Algorithm

I must honestly assess: **CSCT cannot be detuned without losing its unique
advantage (M10).** Once you cap pair-matched slots and inject randomness, you
get something that looks like GPE-45 or Pair-Escalation with a commitment-ratio
trigger instead of a counter trigger. The detuned variant is decent (~2.3 M3,
~75% M6, ~0.82 M9) but is no longer uniquely compelling.

Therefore, I propose a **hybrid: Narrative Gravity with CSCT-style minimum
floor.**

- Narrative Gravity's contraction provides the quality ramp and variety
- CSCT's commitment-ratio determines a minimum floor: if ratio > 0.5, at least 1
  slot is pair-matched
- This addresses Narrative Gravity's M10 weakness without creating CSCT's M6
  problem

**Pool: 40% Enriched (128 dual-res, 192 single-res, 40 generic).**

**Set Design Specification:**

- 360 cards: 320 archetype + 40 generic
- Each archetype: 22 home-only + 18 dual-res (slightly more than uniform 16 to
  support pair-matching)
- Per-resonance R1 pool: 80 cards
- Pair subpool: 18 cards per archetype
- Fitness: Graduated Realistic (36% avg minimum)

## Recommendations to the Card Designer

1. CSCT proves that pair-matching with a 40% enriched pool achieves M3 >= 2.85
   under Hostile fitness. The M3 target is not the problem -- the problem is
   delivering high M3 without over-concentration.
2. The 40% enriched pool is the foundation. Every competitive algorithm requires
   it.
3. CSCT's per-archetype equity (0.08 spread) demonstrates that pair-matching
   inherently solves the archetype-fairness problem. Other algorithms should
   adopt pair-matching as their targeting mechanism.
4. The card designer does not need to worry about cross-archetype fitness. CSCT
   proves that 0% sibling A-tier still yields M3 = 2.84. The algorithm is the
   solution, not the card design.
5. Focus design effort on symbol assignment accuracy (correct primary and
   secondary resonance per archetype) and on making the draft experience feel
   like a choice, not a funnel.
