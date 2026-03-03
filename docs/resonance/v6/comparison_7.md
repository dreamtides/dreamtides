# Agent 7 Comparison: Surge Packs

## Scorecard (1-10, all 7 algorithms x 9 design goals)

| Goal               | LL (1) | TAS (2) | SL (3) | PS (4) | DE (5) | RS (6) | SP (7) |
| ------------------ | :----: | :-----: | :----: | :----: | :----: | :----: | :----: |
| 1. Simple          |   9    |    7    |   5    |   4    |   7    |   8    |   9    |
| 2. No actions      |   10   |   10    |   10   |   10   |   10   |   10   |   10   |
| 3. Not on rails    |   2    |    7    |   6    |   9    |   7    |   3    |   8    |
| 4. No forced decks |   7    |    8    |   7    |   8    |   7    |   6    |   7    |
| 5. Flexible        |   2    |    6    |   5    |   8    |   5    |   3    |   8    |
| 6. Convergent      |   7    |    6    |   8    |   3    |   4    |   8    |   7    |
| 7. Splashable      |   4    |    8    |   5    |   3    |   7    |   7    |   8    |
| 8. Open early      |   6    |    8    |   7    |   9    |   7    |   6    |   9    |
| 9. Signal reading  |   3    |    6    |   5    |   5    |   5    |   3    |   5    |
| **Total**          | **50** | **66**  | **58** | **59** | **59** | **54** | **71** |

**My scoring philosophy emphasizes player experience over raw metrics:**

- LL rails 2, flexibility 2: A player who changes their mind at pick 8 has 2
  permanently wrong lock slots. In Surge, they just accumulate a different
  counter.
- SP convergent 7: 2.08 crosses 2.0. Bimodal distribution is a feature for
  roguelike players.
- SP open-early 9: 75% of early packs are fully random. Most open early game in
  V6.
- RS rails 3: Three permanent locks mean 75% deterministic by mid-draft. Worse
  than LL.

**Key disagreement with Agent 6:** RS at 2.20 S/A with 0.69 stddev is
mechanically strong but experientially hollow -- the player sees the same pack
structure every time. Convergence should measure the player's ability to build a
deck, not the algorithm's ability to produce identical packs.

## Biggest Strength and Weakness Per Strategy

| Algo    | Biggest Strength                                                        | Biggest Weakness                                                                     |
| ------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| LL (1)  | The mathematical foundation: 100% S/A per locked slot                   | The experience: drafting on autopilot for 25+ picks                                  |
| TAS (2) | No catastrophic failures anywhere; most balanced algorithm              | 2.01 S/A: too fragile to recommend with confidence                                   |
| SL (3)  | Only algorithm achieving both convergence and variance targets          | Splash (0.49) and concentration (97%) -- two near-misses that compound               |
| PS (4)  | Invisible mechanism creates the most "normal draft" experience          | Structural ceiling confirmed: V4's finding holds under V6 constraints                |
| DE (5)  | Pack-inspection trigger is genuinely novel design space                 | Fundamentally unfixable: no threshold produces both sufficient fire rate and 2.0 S/A |
| RS (6)  | Clean escalation arc with strong raw numbers                            | Worse railroading than LL: 3 locked slots vs 2, with no compensating advantage       |
| SP (7)  | Only algorithm combining: non-permanence + 2.0+ S/A + rhythm + pivoting | 25.5% zero-S/A packs post-commitment create painful valleys                          |

## Proposed Improvements

- **LL (1):** Add voluntary unlock (player can sacrifice a pick to unlock a
  slot). But this adds a decision, violating Goal 2. LL is stuck.
- **TAS (2):** C4B3 would push S/A to ~2.5 with acceptable concentration. This
  is the obvious fix.
- **SL (3):** Forced-splash slot 4 fixes the 0.49 C/F problem. This is Agent 3's
  best path.
- **PS (4):** Merge into Surge Packs as a background layer. Between surges, pool
  sculpting provides mild bias instead of pure random.
- **DE (5):** Reframe thresh=1/bonus=2 as "Resonance Echo" and accept the 63%
  fire rate. Stop treating conditionality as a goal.
- **RS (6):** The variance problem is fundamental to hard locks. No threshold
  tuning fixes it below 3 locks. Use 2 hard locks + 1 soft lock (75%) for the
  third.
- **SP (7) -- self-improvement:** The zero-S/A valley is my biggest weakness.
  Two solutions:
  - **Mild fix:** When not surging, guarantee 1 of 4 slots shows top-resonance
    card. Non-surge packs go from ~1.0 to ~1.75 S/A. Blended S/A rises to ~2.2.
  - **Aggressive fix:** Lower threshold to 3 (surge fires 80%+ of the time). But
    this loses the surge/normal rhythm entirely.
  - I prefer the mild fix: it fills valleys without destroying the surge
    identity.

## Baseline Comparison

**Does any V6 algorithm clearly beat both baselines?**

Against Lane Locking: Surge beats LL on 6 of 9 metrics. LL beats SP only on raw
S/A (2.11 vs 2.08 -- tied) and simplicity (marginal).

Against Pack Widening: No zero-decision algorithm matches PW's 3.35 S/A. For a
roguelike where decision fatigue matters, zero-decision + 2.08 is preferable.

**My ranking of V6 algorithms:**

1. Surge Packs (T=4/S=3) -- best experience, sufficient convergence
2. Threshold Auto-Spend (C4B2) -- most robust metrics
3. Soft Locks (75%, 3/6/9) -- best convergence-variance balance
4. Ratcheting (3/6/10 split) -- strongest raw convergence, poor variance
5. Lane Locking (3/8) -- reference point, outclassed by RS and SL
6. Double Enhancement (thresh=1) -- viable but unfocused identity
7. Pool Sculpting (18/pick) -- confirmed structural ceiling

## Proposed Best Algorithm

**Surge Packs with guaranteed-slot valley fill.**

"Each drafted symbol adds tokens (+2 primary, +1 others); when any counter
reaches 4, spend 4 and fill 3 of the next pack's 4 slots with that resonance's
cards, fourth slot random; when NOT surging, one slot always shows a card of
your top resonance."

This hybrid adds a single guaranteed slot on non-surge picks. Projected: surge
packs 2.5 S/A (unchanged), non-surge packs 1.75 S/A (up from 1.0), blended ~2.2
S/A. Stddev drops from 1.39 to ~1.0 (still above 0.8 target). The guaranteed
slot is non-permanent: it tracks the current top counter, allowing pivots.

The one-sentence is longer but remains concrete and implementable. The two
states (surge vs non-surge) create a clean mental model: "sometimes your pack
surges with 3 matched cards, other times you get 1 matched card plus 3 random."

## 15% Dual-Resonance Constraint Impact

The constraint made the problem **just different**. The expected challenge was
archetype disambiguation under single-resonance ambiguity. The actual finding:
disambiguation is unnecessary because the S/A tier structure means
resonance-level targeting IS archetype-level targeting (at the S/A tier). A
Tide-committed player benefits equally from Warriors-home and Sacrifice-home
cards because both are S/A.

This is the most important structural finding of V6: **the 50% dilution ceiling
from V3/V4 was measured at the wrong tier level.** At S/A-tier, each resonance's
primary pool serves exactly 2 archetypes that are both S/A for each other (100%
precision).

At 10%: Zero impact on algorithms. Fewer player-facing archetype signals. At
20%: Marginal algorithm benefit (+0.05-0.1 S/A). More significant player
experience benefit from dual-type archetype clarity.

My recommendation: keep 15% as the right balance.
