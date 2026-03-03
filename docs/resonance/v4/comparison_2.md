# Comparison 2: Cross-Strategy Analysis (Agent 2 — Square-Root Affinity)

## Scorecard Table (1-10, with justification)

| Goal | Exile Pressure | Sqrt Affinity | Pack Widening v2 | Phantoms | Deck Echo |
|------|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 5 | 6 | 9 | 9 | 7 |
| 2. Not on rails | 8 | 9 | 6 | 9 | 9 |
| 3. No forced decks | 8 | 8 | 5 | 9 | 9 |
| 4. Flexible archetypes | 7 | 8 | 7 | 7 | 8 |
| 5. Convergent | 4 | 4 | 8 | 2 | 3 |
| 6. Splashable | 8 | 9 | 8 | 8 | 9 |
| 7. Open early | 9 | 9 | 5 | 9 | 9 |
| 8. Signal reading | 6 | 3 | 3 | 10 | 3 |
| **Total** | **55** | **56** | **51** | **63** | **57** |

**Justifications:**

- **Exile simple=5:** Three interacting clauses (pass counting, decay, skip probability) in one run-on sentence. Technically complete but dense.
- **Sqrt simple=6:** "Square root of symbol overlap" is mathematically precise but assumes mathematical literacy beyond most players.
- **Pack Widening convergent=8:** 3.35 S/A massively exceeds 2.0 target, but 98.6% deck concentration over-converges past the 90% cap. Docked for over-convergence.
- **Phantoms convergent=2:** 1.26 S/A is the worst convergence across all strategies. Parameter sweeps confirm this is structurally unfixable — pool suppression is too weak.
- **Phantoms signal=10:** Only algorithm where signal reading is a first-class observable mechanic. Phantom consumption patterns are directly strategic.
- **Pack Widening open-early=5:** 2.48 early S/A exceeds the <=2.0 cap — spending from pick 3 narrows the draft too fast.

## Biggest Strength and Weakness Per Strategy

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|------------------|
| Exile Pressure | Tightest archetype balance (6.8-7.7 convergence spread) | Structural S/A ceiling at ~1.57; no parameter reaches 2.0 |
| Sqrt Affinity | Concave scaling naturally supports hybrid/flexible archetypes | Convergence pick 10.5 — 3-5 picks too late |
| Pack Widening v2 | Only V4 strategy passing convergence target (3.35 S/A) | Trivial always-spend decision at cost 2; over-concentration (98.6%) |
| Multiple Phantoms | First-class signal reading (10/10); simplest conceptual model | Worst convergence of all (1.26 S/A); fundamentally insufficient pool suppression |
| Deck Echo Filter | Most natural-feeling variance (two-phase randomness, 14% zero-S/A packs) | Same structural S/A ceiling as Exile (~1.55); candidate pool too small |

## Proposed Improvements

**Exile Pressure:** Combine with Pack Widening's bonus cards. When exile counters exceed a threshold, earn bonus pack cards instead of probabilistic skipping. This converts the decay/exile mechanism into a convergence engine.

**Sqrt Affinity:** Replace total-overlap weighting with per-resonance weighting: weight = 1 + sqrt(count_of_card's_primary_resonance_in_deck). Concentrates bias on the player's strongest resonance rather than spreading across all symbols.

**Pack Widening v2:** Cost 3 / bonus 1 / spending gate at pick 5. Fixes all three problems: cost 3 creates real decisions (can't always afford to spend), bonus 1 reduces over-concentration, and the gate preserves early openness.

**Multiple Phantoms:** Cannot fix convergence through tuning — use as a signal-reading layer on top of another mechanism. Phantoms + Pack Widening is the natural hybrid.

**Deck Echo Filter:** Increase candidate pool to 20+ with steeper acceptance curve. But this risks pseudo-deterministic filtering that defeats the natural-variance premise.

## V3 Comparison

**No V4 algorithm clearly dominates Lane Locking.** The tradeoff is structural:

Lane Locking wins on convergence (2.08-2.39 S/A, pick 5.7-6.5) — the two most critical gameplay metrics. It fails on deck concentration (96-98% vs 60-90% target), variance (stddev 0.70-0.94, borderline), and mechanical feel.

V4 algorithms collectively win on variance (stddev 0.92-1.00), splash (1.11-2.43 off-archetype), deck diversity (72-89% concentration), and run variety (4.9-12.1% overlap). They fail on convergence except Pack Widening.

**The fundamental V4 finding:** Probabilistic resonance-based mechanisms (filtering, weighting, scarcity) cannot overcome the ~50% archetype dilution inherent in the resonance→archetype mapping. Only mechanisms that ADD targeted cards (Pack Widening) or deterministically PLACE cards (Lane Locking) cross the 2.0 S/A threshold. This is a mathematical limit, not a tuning problem.

**Fixability assessment:** Exile, Sqrt Affinity, and Deck Echo are fundamentally limited (not tunable to 2.0). Phantoms are fundamentally wrong for convergence (but right for signal reading). Pack Widening is tunable — cost 3 already produces 2.70 S/A with genuine decisions.

## Proposed Best Algorithm

**Pack Widening v3** (modified from Agent 3's original)

**One-sentence:** "Each symbol you draft earns 1 matching token (primary earns 2); starting at pick 5, you may spend 3 tokens of one resonance to add 1 extra card with that primary resonance to the pack."

**Why this algorithm:**
- **Convergence:** Cost 3 / bonus 2 produced 2.70 S/A in simulation. Cost 3 / bonus 1 should reach ~2.0-2.3 (needs simulation verification).
- **Natural variance:** When not spending, packs are fully random (4 cards). When spending, one bonus card nudges without guaranteeing. StdDev projected ~1.0-1.3.
- **Real decisions:** At cost 3, players accumulate ~3-4 tokens per pick and spend every 2-3 picks. The save/spend rhythm creates genuine strategic choices.
- **Early openness:** Spending gate at pick 5 means first 4 picks are pure random, preserving exploration.
- **Simplicity:** One sentence, one mechanism, fully implementable.

**Why not Lane Locking:** Lane Locking's 96-98% deck concentration and deterministic slot assignment make drafts feel mechanical. Pack Widening v3 preserves convergence power while adding player agency and natural variance.

**Why not a hybrid with Phantoms:** Despite my initial advocacy for phantoms, the 3:2 consensus against them is correct — adding a second mechanism violates the simplicity principle (goal #1). Signal reading can be addressed through pool asymmetry (V3's +20/-20) rather than active phantoms.

**Critical caveat:** Cost 3 / bonus 1 has not been simulated. If it cannot reach 2.0 S/A, Lane Locking remains the best option. The Round 5 synthesis agent should run this configuration.
