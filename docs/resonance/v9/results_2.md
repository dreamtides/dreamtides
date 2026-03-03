# Simulation Results: Design 6 — Anchor-Scaled Contraction

**Agent 2, Round 4 — V9 Simulation**

______________________________________________________________________

## Scorecard: Graduated Realistic, Committed Strategy (1000 drafts)

| Metric | Value | Target  | Status          |
| ------ | ----: | ------- | --------------- |
| M1     |  2.84 | >= 3.0  | **FAIL**        |
| M2     |  0.85 | \<= 2.0 | PASS            |
| M3     |  2.00 | >= 2.0  | PASS (marginal) |
| M4     |  2.00 | >= 0.5  | PASS            |
| M5     |  11.5 | 5–8     | **FAIL**        |
| M6     | 72.3% | 60–90%  | PASS            |
| M7     |  6.9% | < 40%   | PASS            |
| M9     |  0.84 | >= 0.8  | PASS            |
| M10    |  7.44 | \<= 2   | **FAIL**        |
| M11    |  2.21 | >= 3.0  | **FAIL**        |

**Pessimistic (secondary):** M3=1.87 (FAIL), M10=8.33, M11=2.08

______________________________________________________________________

## V1–V4 Measurements

| Criterion              |           Value | Notes                                              |
| ---------------------- | --------------: | -------------------------------------------------- |
| V1 (visible influence) | ~116% (paradox) | Visible-only M3=2.25 > full M3=2.00 — see analysis |
| V2 (hidden info)       |     3 bits/card | 1,080 bits total, as specified                     |
| V3 (defensibility)     |            8/10 | Tags reflect genuine mechanical best-fit           |
| V4 (power-chaser gap)  |        +1.32 M3 | Committed 2.00 vs. power 0.69 — PASS               |

______________________________________________________________________

## Per-Archetype M3 (Graduated Realistic, committed, 125 drafts each)

| Archetype    |   M3 |  M11 |   M5 |    M6 |   M10 |
| ------------ | ---: | ---: | ---: | ----: | ----: |
| Flash        | 1.87 | 2.07 | 13.4 | 66.2% |  8.82 |
| Blink        | 1.60 | 1.71 | 13.5 | 66.0% |  8.86 |
| Storm        | 2.60 | 3.01 | 10.0 | 80.8% |  5.09 |
| Self-Discard | 1.97 | 2.11 | 11.8 | 69.9% |  8.06 |
| Self-Mill    | 2.29 | 2.56 |  9.0 | 80.8% |  5.15 |
| Sacrifice    | 2.11 | 2.24 |  9.9 | 75.9% |  6.54 |
| Warriors     | 2.70 | 3.06 |  8.1 | 85.1% |  3.95 |
| Ramp         | 1.28 | 1.34 | 14.9 | 58.6% | 11.06 |

**Worst M3 = 1.28 (Ramp). Spread = 1.42.** 5 of 8 archetypes pass M3 >= 2.0.
Archetypes that fail: Flash (1.87), Blink (1.60), Self-Discard (1.97), Ramp
(1.28).

______________________________________________________________________

## Pack Quality Distribution (picks 6+, Graduated, committed)

| P10 | P25 | P50 | P75 | P90 |
| --- | --- | --- | --- | --- |
| 0   | 0   | 2   | 4   | 4   |

The bimodal 0/4 distribution confirms structural problems: many packs contain
zero S/A cards (M10 failures), and many contain 4 (late convergence for
archetypes like Warriors that do resolve correctly). This is the V8 M3–M10
triangle in full effect.

______________________________________________________________________

## Strategy Comparison (Graduated Realistic)

| Strategy  |   M3 |  M11 |   M5 |    M6 |   M10 |
| --------- | ---: | ---: | ---: | ----: | ----: |
| committed | 2.00 | 2.21 | 11.5 | 72.3% |  7.44 |
| power     | 0.69 | 0.73 | 22.6 | 17.7% | 18.62 |
| signal    | 0.99 | 1.09 | 20.5 | 38.2% | 15.78 |

The power-chaser and signal-reader show catastrophic M3 because the contraction
algorithm locks onto the wrong archetype when the player's picks don't reinforce
the resonance signature cleanly. Signal-reader especially suffers: it follows
power for picks 1–3 then locks onto whatever resonance happened to be highest,
which often does not match the player's actual archetype.

______________________________________________________________________

## Draft Traces

**Trace 1: Warriors, committed (good case)**

The committed Warriors player sends clean signals: majority Tide picks from the
start. Archetype resolves to Warriors at pick 2 due to the early Tide
accumulation. Pool contracts from 360 → 18 by pick 30. Picks 15–30 show
pack_SA=3–4 consistently. Final 28/30 S/A (93%). This is the algorithm working
as designed for a high-overlap archetype with strong visible signal.

**Trace 2: Flash, signal reader (failure case)**

The signal reader picks power for picks 1–3, landing on Ember cards randomly.
Archetype resolves to Storm (Ember-primary, incorrect for Flash/Zephyr).
Contraction culls Zephyr cards and concentrates Ember/Stone cards. The player
gets 1/30 S/A (3%). This represents the core failure mode: the algorithm's
archetype inference is not robust to noisy early picks.

______________________________________________________________________

## V1 Paradox: The Key Finding

**Visible-only M3 (2.25) > Full M3 (2.00).** The hidden archetype tag is
*hurting* performance, not helping it. This is diagnostic, not a bug.

The mechanism: at 10% visible dual-resonance, the archetype inference often
resolves to the wrong same-primary-resonance sibling. For example, a Warriors
(Tide/Zephyr) player whose first 3 picks happen to be Tide-primary cards that
are tagged Sacrifice (equally S/A for a Warriors player) resolves as Sacrifice.
The contraction then culls Zephyr cards, which includes many Warriors-specific
cards. The player now has a Tide-only pool biased toward Sacrifice cards — M3
recovers because both Tide archetypes are S/A, but the specific archetype
precision that the hidden tag was supposed to add is misdirected.

When the hidden tags are stripped (visible-only mode), the contraction works on
pure visible signal alignment without disambiguation errors. Since both
same-primary-resonance archetypes are S/A by definition for the player, the
undirected Tide contraction produces slightly better M3 on average.

This means V1 cannot be meaningfully computed as defined. The formula
(visible_gain / full_gain) produces values > 100% because the hidden component
is net-negative. The correct interpretation: visible-only achieves M3=2.25;
adding hidden tags degrades M3 to 2.00.

______________________________________________________________________

## V8 Comparison

| Algorithm              | Pool | M3   | M11  | M10  | M6    |
| ---------------------- | ---- | ---- | ---- | ---- | ----- |
| Design 6 (Graduated)   | 10%  | 2.00 | 2.21 | 7.44 | 72.3% |
| Design 6 (Pessimistic) | 10%  | 1.87 | 2.08 | 8.33 | —     |
| V8 Narrative Gravity   | 40%  | 2.75 | n/a  | 3.3  | ~75%  |
| V8 SF+Bias R1          | 15%  | 2.24 | n/a  | 8.0  | —     |
| V8 CSCT                | 15%  | 2.92 | n/a  | 2.0  | 99%   |

Design 6 achieves M3=2.00 — matching the target floor at 10% visible
dual-resonance. This is comparable to V8 SF+Bias R1 (M3=2.24) but with worse M10
(7.44 vs 8.0) and many per-archetype failures. The algorithm does not improve on
V8's best algorithms at any metric.

______________________________________________________________________

## Self-Assessment

**Does this algorithm pass?** No. 4 critical failures.

**M10 = 7.44 (target ≤ 2):** The floor slot provides minimal streak mitigation.
The P25 = 0 S/A per pack confirms frequent zero-quality packs throughout the
draft. The anchor-scaled contraction was predicted to improve M10 by delivering
quality bursts after dual-resonance picks, but at 10% dual-res, players average
only 3 dual-res picks across 30 picks — insufficient to drive aggregate M10.

**M11 = 2.21 (target ≥ 3.0):** Late-draft pool is ~18–25 cards, but when
archetype inference fails, those cards are the wrong archetype. Warriors reaches
M11=3.06; Ramp reaches only 1.34. The aggregate is dragged down by archetypes
where the inference fails persistently.

**M5 = 11.5 (target 5–8):** Convergence is slow because the algorithm does not
begin concentrating the pool until the archetype is inferred (often picks 5–8),
and the contraction rates of 6–10% per pick for generic/single-symbol picks are
modest. By pick 11–12, the pool is concentrated enough for consistent S/A
delivery, but this is 3–4 picks too late.

**Per-archetype equity:** Spread of 1.42 M3 (Warriors 2.70 vs. Ramp 1.28) is the
largest equity failure. The algorithm systematically underperforms for
archetypes that share primary resonance with a strong sibling (Ramp shares
Zephyr with Flash; Flash is actually the better-represented visible pair because
Blink also uses Ember, not Zephyr, making disambiguation harder).

**What would fix the failures:**

1. **M11/M5:** Contraction must start earlier (pick 1–2) and at higher rates.
   The 6%/10%/18% rates are tuned for a pool that stays large long enough for
   the full visible feedback loop; at 10% dual-res the pool needs to contract
   60–70% in the first 15 picks.

2. **M10:** Floor slot alone is insufficient. 2 floor slots, or a guaranteed
   home-primary slot from pick 5, would substantially reduce zero-quality packs.

3. **Archetype inference:** The hidden-tag majority-vote approach is unreliable
   for same-primary siblings. An alternative is to use the secondary resonance
   of picked cards (when available) directly for disambiguation rather than
   inferring from tag counts.

4. **Per-archetype equity:** The Ramp/Flash pair needs compensated
   representation (more Zephyr-tagged cards or specific floor-slot weighting) to
   reach M3 >= 2.0.

The core design insight — differentiated contraction rates by pick type — is
sound and produces the right shape of player experience. The anchor mechanic
creates visible feedback. The measured failure is in the archetype inference
precision and aggregate contraction speed, both of which could be addressed with
parameter tuning outside the current spec.
