# Comparison 4: Pair-Based Pack Construction — Round 4 Cross-Comparison

## Scorecard (Strategy x Goal, 1-10)

| Goal | D1 Pair Thresh | D2 Pair-Esc | D3 Pool Seed | D4 Dual-Thresh | D5 Hybrid Trig |
|------|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 8 | 5 | 7 | **9** | 7 |
| 2. No actions | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 8 | 4 | **9** | 5 | 8 |
| 4. No forced decks | 7 | 4 | **8** | 6 | 7 |
| 5. Flexible archetypes | 7 | 4 | 7 | 5 | **8** |
| 6. Convergent | 2 | **10** | 2 | 8 | 3 |
| 7. Splashable | 8 | 3 | **8** | 7 | 8 |
| 8. Open early | **9** | 7 | **9** | 8 | **9** |
| 9. Signal reading | 2 | 3 | **8** | 3 | 3 |
| **Total** | **61** | **50** | **68** | **61** | **63** |

D2 earns 10 on convergence (3.00 S/A, pick 5.8) but 3-4 on splash/rails/forced
because 97% deck concentration means "pick one of three identical cards"
post-commitment. D4 scores 8 on convergence (2.22 S/A passes, conv pick 11.9
misses target). D3 leads totals via qualitative goals despite failing
convergence. Totals are misleading — convergence is a pass/fail gate.

## Biggest Strength / Weakness Per Strategy

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|------------------|
| D1 | Perfect archetype balance (1.3-pick spread) | Capped at 1.36 S/A |
| D2 | Raw convergence (3.00 S/A, pick 5.8) | 97% deck concentration |
| D3 | Only signal-reading support + natural feel | Pool bloat caps at ~1.2 S/A |
| D4 | Simplest algorithm crossing 2.0 | Conv pick 11.9 too slow |
| D5 | Best organic variance (stddev 1.23) | 1.52 S/A standalone |

## Proposed Improvements

**D1:** Retire standalone. Structural ceiling (~1.3 S/A) unfixable.

**D2:** Cap=0.50 gives 2.61 S/A with 0.71 off-arch. Agent 2's discrete steps
(25/50/65% at counts 2/4/6) improve explainability. Still harder to reason
about than thresholds.

**D3:** Value as complement only. Pool seeding adds ~0.12 S/A to D4's random
slots plus signal reading. But two mechanisms in one sentence fails simplicity.

**D4:** Lower to (2,5): 2.48 S/A, conv 9.2, stddev 0.82. Count 1-symbol cards
at 0.5 weight to accelerate convergence ~1-2 picks further.

**D5:** Agent 5 conceded D4 (2,5) standalone (2.48 S/A) outperforms D4+D5
hybrid (2.10 S/A) on convergence. Conditional trigger's value is limited to
variance, which D4 already passes.

## V3/V4 Comparison

**Honest disclosure:** Lane Locking achieves 2.61 S/A in my simulation —
HIGHER than D4 at 2.22. LL converges faster (6.8 vs 11.9). D4 at (3,7) is
strictly worse on convergence metrics alone.

**D4 (2,5) closes the gap:** 2.48 vs LL's 2.61 (~5% reduction). D4's
advantages: better variance (0.82 vs 0.74 — LL fails variance target), better
splash (1.29 vs 1.19), no permanent locks (dynamic pair tracking enables
pivots), and pair precision (100% vs ~60% for LL). The ~5% S/A reduction buys
meaningfully better draft experience.

**V5 decisively beats Pack Widening auto-spend** (1.35-1.96 S/A). Pair matching
eliminates single-resonance dilution. D4 (2,5) at 2.48 matches manual Pack
Widening (projected 2.3-2.5) with zero decisions.

**D2 at cap=0.50 is the strongest V5 candidate against LL:** 2.61 S/A vs LL
2.34 in Agent 2's simulation — a clear win. But D2's probability formula is
harder for players to reason about than D4's thresholds. The choice between D2
and D4 comes down to whether convergence power or transparency matters more.

## Pair-Matching Analysis

**Confirmed:** Ordered pairs break through the archetype dilution ceiling. 100%
S-tier precision for 2+ symbol cards vs ~50% for single-resonance. This is V5's
defining breakthrough, validated by all 5 simulations.

Agent 2 and Agent 5 confirmed pair matching's benefit is primarily in PRECISION
per targeted slot, not fire rate. Slot replacement + pair matching = 3.00 S/A.
Slot guarantee + pair matching = 2.48 S/A. Bonus injection + pair matching =
1.10-1.52 S/A. The mechanism type still matters — pair matching doesn't make
weak mechanisms strong, it makes strong mechanisms precise.

**1-symbol limitation:** 15% of cards can't form pairs, creating a convergence
bottleneck. Fix: count 1-symbol cards at 0.5 weight toward both possible pairs.
Unsimulated but should accelerate convergence ~1-2 picks.

## Proposed Best Algorithm

**Dual-Threshold Pair Guarantee (2,5)**

One-sentence: "Track the ordered symbol pair (first, second) of each 2+ symbol
card you draft; at 2 matching picks one pack slot is pair-matched, at 5 a second
slot is pair-matched, remaining slots are random."

**Why D4 over D2:** D2 at cap=0.50 has stronger raw numbers (2.61 vs 2.48 S/A),
but D4's threshold description is actionable — players can count pairs and
strategize. D2's probability formula is a black box. For a roguelike deckbuilder
where players replay many times and want to understand the system, transparency
beats organic feel. D4's "I need 2 more pairs for my second slot" creates
anticipation. D2's "each slot has some probability" doesn't.

**Acknowledged weakness:** Conv pick 9.2 misses the 5-8 target. Early pair
scattering (a feature, not a bug — prevents premature commitment) delays
convergence. Counting 1-symbol cards at partial weight or playtesting may reveal
9.2 is acceptable.

**Runner-up:** D2 at cap=0.50 (2.61 S/A, conv ~6-7) if convergence speed
matters more than transparency. Both are clear improvements over V3/V4.
