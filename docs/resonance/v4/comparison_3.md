# Comparison 3: Agent 3 (Pack Widening v2 Domain)

## Scorecard (1-10, with justification)

| Goal | Exile (1) | SqrtAff (2) | PackWide (3) | Phantom (4) | DeckEcho (5) |
|------|:---------:|:-----------:|:------------:|:----------:|:-----------:|
| 1. Simple | 5 | 6 | 9 | 9 | 6 |
| 2. Not on rails | 8 | 9 | 6 | 9 | 9 |
| 3. No forced decks | 8 | 8 | 5 | 9 | 9 |
| 4. Flexible archetypes | 7 | 8 | 7 | 7 | 8 |
| 5. Convergent | 4 | 4 | 8 | 2 | 3 |
| 6. Splashable | 8 | 9 | 8 | 8 | 9 |
| 7. Open early | 9 | 9 | 5 | 9 | 9 |
| 8. Signal reading | 6 | 3 | 3 | 10 | 3 |
| **Total** | **55** | **56** | **51** | **63** | **56** |

**Justifications for key scores:**

- **Exile simple=5:** One-sentence is really three clauses (pass counting, decay, skip probability). Complete but complex.
- **PackWide convergent=8 (not 10):** 3.35 S/A exceeds 2.0 target, but 98.6% deck concentration *over*-converges past the 90% cap.
- **Phantom convergent=2:** 1.26 S/A is worst across all strategies. Parameter sweeps confirm it's structurally unfixable.
- **PackWide open-early=5:** 2.48 early S/A exceeds the <=2.0 target. Spending from pick 3 narrows the draft too fast.
- **Phantom signal=10:** Only algorithm where signal reading is a first-class mechanic. Observing phantom consumption patterns is genuinely strategic.

## Biggest Strength and Weakness Per Strategy

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|------------------|
| Exile Pressure | Tightest archetype balance (6.8-7.7 convergence band) | Structural convergence cap at ~1.6 S/A; unfixable via tuning |
| Sqrt Affinity | Most natural-feeling soft influence; best splash (1.11 off-arch) | Convergence pick 10.5 — too late by 3-5 picks |
| Pack Widening | Only V4 algorithm that passes convergence (3.35 S/A) | Trivial always-spend decision eliminates player agency |
| Multiple Phantoms | Signal reading is built into the core experience | Convergence fundamentally broken (1.26 S/A) |
| Deck Echo | Variance feels genuinely organic (14% zero-S/A packs) | Same resonance-archetype dilution ceiling as Exile (~1.55 S/A) |

## Why Most Strategies Fail Convergence

Four of five V4 algorithms fail the 2.0 S/A target because they operate on resonance-level properties. Each archetype is ~11% of the 360-card pool, and each resonance is shared by 4 archetypes. Probabilistic biasing at the resonance level can push archetype density from ~11% to ~15-18% — not enough for 2+ S/A in a 4-card pack. Only algorithms that *add* resonance-matched cards to packs (Lane Locking via slot locking, Pack Widening via bonus cards) generate enough volume to cross the threshold despite the ~50% archetype dilution.

## V3 Lane Locking Comparison

No V4 algorithm as-simulated clearly dominates Lane Locking. Lane Locking passes convergence (2.08-2.39 S/A) where 4/5 V4 algorithms fail, and converges by pick 5.7-6.0. Its failures: over-concentration (96-99%, failing 60-90% target), borderline variance (stddev 0.70-0.94), mechanical feel, and no pivot flexibility.

Pack Widening v2 at cost 2 matches Lane Locking's convergence but shares its over-concentration problem. At cost 3, Pack Widening projects to ~2.3-2.7 S/A with stddev ~1.3 and concentration ~80-90% — potentially beating Lane Locking on every metric while adding genuine player decisions. This variant needs simulation to confirm.

The V4 investigation reveals a structural truth: soft probabilistic mechanisms (SqrtAff, DeckEcho, Exile) produce beautiful variance but cannot overcome the resonance-archetype dilution ceiling. Convergence requires directly adding resonance-matched cards — which both Lane Locking and Pack Widening do through different UX patterns.

## Proposed Best Algorithm

**Modified Pack Widening v3**

"Each symbol you draft earns 1 matching token (primary earns 2); starting at pick 5, you may spend 3 tokens of one resonance to add 1 extra card with that primary resonance to the pack."

**Key parameters:** Cost 3 (creates genuine save/spend decisions), bonus 1 (reduces over-concentration), tokens from all drafted symbols (primary=2, secondary/tertiary=1), spending gate at pick 5 (protects early openness).

**Why this beats Lane Locking:**
- **Player agency:** "Do I spend 3 Tide tokens now or save for next turn?" is more interesting than Lane Locking's invisible slot assignment.
- **Natural variance:** When not spending, packs are fully random. When spending, one bonus card nudges without guaranteeing. StdDev projected ~1.2-1.4 vs Lane Locking's 0.70-0.94.
- **Flexible commitment:** Tokens accumulate across resonances. A player can pivot by spending a different resonance's tokens. Lane Locking permanently locks slots.
- **Convergence:** Projected ~2.3-2.7 S/A (competitive with Lane Locking's 2.08-2.39).
- **Simplicity:** One sentence, one mechanism, no hidden state beyond token counts.
- **Early openness:** Pick 5 gate ensures first 4 packs are fully random, protecting exploration.

**Discussion consensus (all 5 agents):** Pack Widening cost 3 is the only V4 mechanism that crosses 2.0 S/A. The pick 5 gate (adopted from Agents 1 and 2) prevents early narrowing. Bonus 1 (vs bonus 2) reduces over-concentration while still passing convergence. Phantoms were considered for signal reading but rejected 3-2 in favor of simplicity (goal #1). Round 5 should simulate this exact configuration to confirm projected metrics.
