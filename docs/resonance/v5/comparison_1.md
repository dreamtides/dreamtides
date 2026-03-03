# Comparison 1 — Agent 1 (Passive Resonance Bonus)

## Scorecard Table (Strategy x Goal, 1-10)

| Goal | D1: Pair Thresh | D2: Pair-Esc Slots | D3: Pool Seeding | D4: Dual-Thresh | D5: Hybrid Trigger |
|------|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 8 | 5 | 7 | 9 | 7 |
| 2. No extra actions | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 8 | 6 | 9 | 4 | 9 |
| 4. No forced decks | 7 | 5 | 9 | 6 | 8 |
| 5. Flexible archetypes | 7 | 5 | 7 | 5 | 8 |
| 6. Convergent (>=2.0) | 2 | 10 | 2 | 7 | 3 |
| 7. Splashable | 8 | 5 | 8 | 7 | 8 |
| 8. Open early | 9 | 7 | 9 | 8 | 9 |
| 9. Signal reading | 2 | 3 | 8 | 3 | 3 |
| **Total** | **61** | **56** | **69** | **59** | **65** |

**Scoring notes:** Totals are misleading. D2 scores lowest overall but is
the ONLY algorithm that comfortably crosses 2.0 S/A — the single hardest
constraint. D3 scores highest but fails the convergence gate entirely.
Convergence is pass/fail, not a gradient.

**Key 1-sentence justifications for divergent scores:**

- D1 convergence=2: 1.10 S/A is structurally capped; even T=2/B=2 only
  reaches 1.36.
- D2 convergence=10: 3.00 S/A at cap=0.65, 2.61 at cap=0.50 — dominates
  all other algorithms by 0.4-1.9 S/A.
- D2 simple=5: `min(count/6, 0.65)` formula is honest but inaccessible to
  players; discrete steps (25/50/65%) would improve this.
- D4 not-on-rails=4: After both thresholds, 50% of every pack is
  deterministic — more rigid than Lane Locking's 2/4 slots at threshold 8.
- D3 signal=8: Only algorithm interacting with pool composition; signal
  readers are genuinely rewarded.

## Biggest Strength and Weakness Per Strategy

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|------------------|
| D1 Pair Threshold | Excellent variance (0.99 stddev) with natural feel | Structural S/A ceiling at ~1.3 — cannot cross 2.0 |
| D2 Pair-Escalation | Dominant convergence (3.00 S/A) with probabilistic variance | 97% deck concentration crushes splash and flexibility |
| D3 Pool Seeding | Most natural feel + only algorithm enabling signal reading | Pool bloat caps S/A at 1.18 — cannot cross 2.0 alone |
| D4 Dual-Threshold | Most balanced profile (8/9 targets pass) with simple rules | Convergence pick 11.9 fails 5-8 target; feels on-rails |
| D5 Hybrid Trigger | Best organic variance (1.21 stddev in D4 hybrid) | 1.52 standalone S/A requires hybridization to be viable |

## Proposed Improvements

- **D1:** Retire as standalone. Use as variance layer atop D2/D4.
- **D2:** Reduce cap to 0.50 (2.61 S/A, 0.71 splash). Use discrete
  probability steps (25%/50%/65% at pair counts 2/4/6+) for simplicity.
- **D3:** Use as enhancement layer only. Add initial pool asymmetry for
  signal reading without ongoing injection complexity.
- **D4:** Lower thresholds to (2,5) for convergence pick ~9.2. Consider
  3-step variant (2/4/6 → 1/2/2 slots) for smoother progression.
- **D5:** D4 hybrid (2.10 S/A, 1.21 stddev) is the only viable path.
  Standalone is retired.

## V3/V4 Comparison

D2 at cap=0.50 beats both baselines: 2.61 S/A vs Lane Locking's ~2.3 and
Pack Widening auto-spend's ~1.4-1.96. The pair-matching breakthrough
(100% archetype precision for 2+ symbol cards) eliminates the ~50% dilution
that limited V3/V4 single-resonance approaches. Pack Widening auto-spend
is conclusively outclassed — single-resonance targeting cannot compete with
pair targeting at archetype level. Lane Locking remains competitive on
convergence speed but loses on variance (0.74-0.96 vs D2's 0.97) and feel
(deterministic slot locking vs probabilistic escalation).

## Pair-Matching Verdict

**Pair matching decisively breaks the dilution ceiling.** V4's ~1.7 S/A cap
applied to single-resonance matching only. Ordered pairs achieve ~100%
archetype precision for 75% of cards (those with 2+ symbols), enabling D2
to reach 3.00 S/A via pure probabilistic slot targeting — a result V4
declared structurally impossible. This is V5's most important finding.

## Proposed Best Algorithm

**Pair-Escalation Slots with Discrete Steps (modified D2)**

One-sentence: "Track the ordered symbol pair of each 2+ symbol card you
draft; at pair counts 2/4/6+, each pack slot independently has a
25%/50%/65% chance of showing a card matching your top pair, otherwise
random."

This preserves D2's dominant convergence (~2.5-2.6 S/A) and natural
probabilistic variance (~0.97 stddev) while improving simplicity from a
formula to three discrete thresholds. The 65% cap (vs D2's tested 0.65)
maintains 1.4 expected random slots per pack for splash. Adding initial
pool asymmetry (20% bias toward 2 random resonances per run) would restore
signal reading without mechanism complexity.
