# Design 5 AWNG — Simulation Results
**Affinity-Weighted Narrative Gravity, V9 Round 4**

---

## Scorecard

### Graduated Realistic (Primary), Committed Strategy

| Metric | Value | Target | Pass? |
|--------|------:|--------|-------|
| M1 (early variety, unique archs/pack) | 0.68 | >= 3 of 8 | FAIL* |
| M2 (early S/A, picks 1-5) | 0.80 | <= 2.0 | PASS |
| M3 (post-commit S/A, picks 6+) | **2.32** | >= 2.0 | PASS |
| M4 (off-arch cards, picks 6+) | 1.68 | >= 0.5 | PASS |
| M5 (convergence pick) | 8.9 | pick 5-8 | FAIL |
| M6 (deck concentration) | **0.89** | 60-90% | PASS |
| M7 (run variety, overlap) | 7.2% | < 40% | PASS |
| M8 (archetype freq 11-14%) | 11-14% | no arch >20% or <5% | PASS |
| M9 (S/A stddev) | 1.04 | >= 0.8 | PASS |
| M10 (max consec bad packs) | **2.9** | <= 2 | FAIL |
| M11 (late S/A, picks 15+) | **2.71** | >= 3.0 | FAIL |

*M1 measures unique archs with S/A cards per pack; low value reflects low
sibling crossover in the low-dual-res pool, not a true diversity failure.

### Pessimistic (Secondary), Committed Strategy

| M3 | M10 | M11 | M6 |
|---:|----:|----:|---:|
| 2.07 | 3.7 | 2.42 | 0.85 |

### Per-Archetype M3, Graduated Realistic

| Archetype | M3 | M5 | M6 | M9 | M10 | M11 |
|-----------|---:|---:|---:|---:|----:|----:|
| Flash | 2.26 | 9.3 | 0.87 | 1.09 | 3.3 | 2.69 |
| **Blink** | **1.80** | 9.1 | 0.85 | 0.93 | 3.5 | 1.94 |
| Storm | 2.60 | 9.6 | 0.87 | 1.15 | 3.3 | 3.20 |
| Self-Discard | 2.03 | 8.3 | 0.88 | 0.94 | 2.8 | 2.21 |
| Self-Mill | 2.68 | 8.9 | 0.90 | 1.13 | 2.9 | 3.25 |
| Sacrifice | 2.68 | 7.6 | 0.92 | 1.07 | 2.1 | 3.13 |
| Warriors | 2.58 | 7.4 | 0.92 | 1.02 | 2.1 | 2.99 |
| Ramp | 2.05 | 9.4 | 0.87 | 0.98 | 3.5 | 2.36 |

**Blink fails M3 >= 2.0 (1.80). M3 spread = 0.88 (largest archetype gap).**
Archetypes above M3 >= 2.0: 7/8 (Blink fails).
Archetypes with M11 >= 3.0: 3/8 (Storm, Self-Mill, Sacrifice).

### Pack Quality Distribution (picks 6+, Graduated, committed)

| P10 | P25 | P50 | P75 | P90 |
|----:|----:|----:|----:|----:|
| 1 | 1 | 2 | 3 | 4 |

Consecutive bad packs (S/A < 1.5): avg=2.9, worst=19.

---

## V1-V4 Measurements

### V1: Visible Symbol Influence — **99.3%**

| Condition | M3 |
|-----------|---:|
| Full AWNG (with affinity vectors) | 2.309 |
| Visible-symbol-only (keywords stripped) | 2.300 |
| No contraction baseline | 1.077 |

**V1 = (2.300 - 1.077) / (2.309 - 1.077) = 0.993 (99.3%)**

This result contradicts the design prediction of V1 = 40-45%. The mechanical
keywords (graveyard, flash_speed, ramp, creature_warrior) add only +0.009 M3
over visible-symbol-only mode. The visible primary resonance symbol contribution
(+0.60) dominates the affinity vector so completely that stripping keywords
changes nothing measurable. **The 8-float affinity vector is functionally
identical to a 1-float visible-resonance score for contraction purposes.**

**V1 PASSES the 40% threshold but reveals the hidden metadata is not doing what
the design claimed.** The system is honest (V3 = 9/10), but the "affinity
refinement" is illusory — visible symbols are doing 99% of the work, not 40-45%.

### V2: Hidden Info Quantity — **11,520 bits (~1.4 KB)**

8 floats per card × 360 cards. At 4-bit precision: 11,520 bits. This is ~11x
more hidden information than a 3-bit archetype tag (1,080 bits), but the
additional information (mechanical keywords) produces no measurable M3 gain.
The information overhead is entirely unjustified by the performance results.

### V3: Reverse-Engineering Defensibility — **9/10**

Affinity derivation rules are published and algorithmic. A player who looked up
any card's affinity vector would find it derivable in 30 seconds from the card's
visible text. No arbitrary assignments. Score unchanged from design prediction.

### V4: Visible Pick Alignment — **48.0%**

In 48% of packs, the best visible pick (by primary resonance symbol matching)
agrees with the best affinity-relevance pick. In 52% of packs, hidden affinity
diverges from visible symbol signal. This divergence is primarily driven by the
floor slot selecting a high-affinity card that may not have the target resonance
symbol — a card with partial-symbol contribution from a different resonance.

---

## Draft Traces

### Trace 1: Warriors (Committed, Graduated)

Pool contracts from 360 to 20 cards. Profile commits to Warriors by pick 3
(dual-res signpost at pick 2 triggered the profile update). By pick 17-18,
pack S/A = 4/4 consistently. Final: 28/30 S/A (93%).

Key pattern: contraction is fast once Warriors profile is established. Pool
reaches ~20-card floor by pick 26. Late-draft packs (picks 17-30) are
consistently 3-4 S/A. This trace is a best-case scenario for Warriors.

### Trace 2: Self-Mill (Signal Reader, Graduated)

Profile oscillates between Self-Mill and Self-Discard in picks 1-5 (both are
Stone-primary archetypes). Profile locks onto Self-Mill by pick 3 due to
floor slot selection. Final: 29/30 S/A (97%). Signal reader performs well
when the dominant resonance matches a strong archetype.

---

## V8 Comparison

| Algorithm | Pool | M3 (Grad) | M3 (Pess) | M10 | M11 | M6 |
|-----------|------|----------:|----------:|----:|----:|---:|
| V8 NG (Narrative Gravity) | 40% dual-res | 2.75 | 2.59 | 3.3 | N/A | 0.85 |
| V8 SF+Bias R1 | V7 15% pool | 2.24 | N/A | 8.0 | N/A | N/A |
| V8 CSCT | V7 15% pool | 2.92 | N/A | 2.0 | N/A | 0.99 |
| **Design 5 AWNG** | **10% dual-res** | **2.32** | **2.07** | **2.9** | **2.71** | **0.89** |

AWNG on 10% pool achieves M3=2.32 — 0.43 below V8 Narrative Gravity on 40%
pool, and only 0.08 above V8 SF+Bias R1 on the 15% pool. The pool contraction
mechanism is working (otherwise M3 would be ~1.1), but the affinity precision
gain over raw visible symbols is near zero. AWNG is functionally equivalent to
V8 Narrative Gravity run on a 10% visible dual-res pool — exactly what the
mathematical analysis predicted as the visible-symbol-only ceiling.

---

## Key Findings

**1. The affinity vector is dominated by the visible resonance symbol.**
The +0.60 primary resonance contribution so heavily outweighs mechanical
keywords (+0.20 max, ~30-50% of cards) that stripping keywords changes M3
by only 0.009. The 8-float design adds nothing to targeting precision over
a 1-float resonance match. This is the simulation's principal finding.

**2. M3 = 2.32 misses the predicted range (2.65-2.80) by 0.33-0.48 points.**
The prediction assumed affinities would distinguish same-symbol archetypes
(Warriors vs. Sacrifice within Tide), but in simulation this distinction
contributes negligibly because mechanical keywords are sparse and their
affinity contribution is overwhelmed by the visible symbol.

**3. Blink fails M3 >= 2.0 (1.80).** Ember has four archetypes (Blink, Storm,
Flash secondary, Self-Discard secondary), diluting Ember-symbol cards across
more archetypes. The sibling fitness rate for Blink/Storm is only 30%, and
contraction struggles to distinguish Blink from Storm cards when both show
primarily the Ember symbol. Flash also underperforms (2.26) vs. Tide-primary
archetypes (Warriors=2.58, Sacrifice=2.68) for the same reason.

**4. M11 = 2.71 misses 3.0.** Late-draft quality improves with contraction
(2.71 vs 2.32 average), but the 3.0 target requires stronger archetype
precision in the contracted pool than AWNG achieves. Warriors is closest (2.99)
but still misses.

**5. V1 = 99.3% is not a positive finding for AWNG.** It means the complex
8-float affinity machinery is unnecessary — a simple visible-symbol dot product
would achieve the same result. The design's claimed advantage (mechanical
keyword refinement) is empirically absent.

---

## Self-Assessment

AWNG **passes M3 >= 2.0 (barely)** but fails on multiple key targets. The
fundamental problem is not the algorithm mechanics but the affinity design:
mechanical keywords contribute too little to distinguish same-resonance siblings
effectively. The design achieves excellent V3 (9/10, maximum honesty) and
reasonable V1 (99.3%, visible symbols dominant), but at the cost of M11 and
per-archetype equity.

**What would fix the failures:**
- **M3 and M11:** The affinity vector needs to use the hidden tag pattern (a
  true pair-level distinction) rather than mechanical keywords that are too
  sparse. If Warriors cards had a Warriors affinity of 0.85 and Sacrifice cards
  had Warriors affinity of 0.15, contraction would distinguish them precisely.
  This is Hybrid B's insight: two floats per resonance-pair is the right level.
- **Blink equity:** Ember-primary archetypes need stronger mechanical-keyword
  differentiation (Blink/Flicker keywords vs. Storm/Spellslinger keywords).

**Design verdict:** AWNG as specified does not outperform a simple visible-symbol
contraction algorithm. The V3 = 9/10 advantage is real but costs nothing — it
just means the system is honest about the limited information it actually uses.
The Hybrid B design (two-float pair affinity per resonance pair, explicitly
encoding Warriors vs. Sacrifice distinction) is the correct next step. AWNG's
full 8-float vector over-parameterizes without proportionate benefit.
