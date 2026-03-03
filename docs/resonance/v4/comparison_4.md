# Comparison 4: Cross-Comparison from the Phantom Drafter Perspective

## Scorecard Table (1-10, with justification)

| Goal | Exile Pressure | Sqrt Affinity | Pack Widening | Phantoms | Deck Echo |
|------|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 5 | 5 | 9 | 9 | 6 |
| 2. Not on rails | 8 | 9 | 5 | 9 | 9 |
| 3. No forced decks | 8 | 8 | 5 | 9 | 9 |
| 4. Flexible archetypes | 7 | 8 | 7 | 7 | 8 |
| 5. Convergent | 4 | 4 | 9 | 3 | 3 |
| 6. Splashable | 8 | 9 | 8 | 8 | 9 |
| 7. Open early | 9 | 9 | 4 | 9 | 9 |
| 8. Signal reading | 6 | 3 | 3 | 10 | 3 |
| **Total** | **55** | **55** | **50** | **64** | **56** |

**Justifications (1 sentence each):**

- **Exile Pressure:** Simple(5) long one-sentence with 3 mechanics; Rails(8) decay enables pivots; Forced(8) 8% overlap; Flex(7) diffuse exile supports bridges; Conv(4) 1.57 S/A misses 2.0; Splash(8) 2.43 off-arch; Open(9) near-zero exile early; Signal(6) rewards early reads indirectly.
- **Sqrt Affinity:** Simple(5) "square root" is mathematically precise but opaque; Rails(9) never locked; Forced(8) 12.1% overlap; Flex(8) concave scaling supports mixed investment; Conv(4) 1.74 S/A misses target; Splash(9) best splash at 1.11; Open(9) near-random early; Signal(3) no pool-state information.
- **Pack Widening:** Simple(9) fully implementable from one sentence; Rails(5) always-spend at cost 2 commits player early; Forced(5) 39.7% overlap is borderline; Flex(7) multi-resonance tokens; Conv(9) 3.35 massively exceeds target; Splash(8) 1.35 off-arch; Open(4) 2.48 early S/A fails target; Signal(3) no pool depletion signal.
- **Phantoms:** Simple(9) fully specified in one sentence; Rails(9) no locks, pivot freely; Forced(9) 4.9% overlap is best; Flex(7) open resonance favors 2-4 archetypes per run; Conv(3) 1.26 worst convergence; Splash(8) 1.37 off-arch; Open(9) near-random early; Signal(10) first-class observable signal.
- **Deck Echo:** Simple(6) three-phase process with probability formula; Rails(9) never locks; Forced(9) 11% overlap; Flex(8) overlap scoring supports bridges; Conv(3) 1.55 fails target; Splash(9) 1.27 off-arch; Open(9) near-random early; Signal(3) no pool-state signal.

## Biggest Strength and Weakness Per Strategy

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|------------------|
| Exile Pressure | Best archetype balance (6.8-7.7 convergence band) | Resonance-level exile creates unfixable ~50% dilution (1.57 S/A) |
| Sqrt Affinity | Best splash/variety balance (1.11 off-arch, 12.1% overlap) | Converges too late (pick 16) and too weakly (1.74 S/A) |
| Pack Widening | Only V4 algorithm passing convergence (3.35 S/A) | Trivial economics at cost 2 + 98.6% over-concentration |
| Phantoms | Best signal reading (10/10) with simplest design | Worst convergence (1.26 S/A), pool suppression fundamentally too weak |
| Deck Echo | Most natural-feeling variance (two-phase randomness) | Candidate pool too small to reliably contain 2+ S/A cards |

## Proposed Improvements

**Exile Pressure:** Operate on card-level affinity rather than resonance counters — exile cards that share fewer symbols with the player's deck, not just cards of specific resonances. This doesn't fix the fundamental resonance≠archetype gap but narrows it.

**Sqrt Affinity:** Combine with pool depletion (phantoms or exile). Affinity biases sampling from a shrinking pool where competing resonances are already scarcer. Two convergence sources might cross 2.0.

**Pack Widening:** Increase cost to 3, reduce bonus to 1, and delay spending eligibility until pick 6. This creates real economic decisions, prevents early-draft railroading, and reduces over-concentration.

**Phantoms:** Add a soft convergence layer. Phantoms alone shift the pool by only ~5%, insufficient for archetype convergence. A lightweight affinity bias on pack sampling (even just +0.5 weight per matching symbol) combined with phantom scarcity could cross 2.0.

**Deck Echo:** Increase candidate pool to 20+ cards and make the filter much more aggressive. But this risks defeating natural variance — the filter would need to be so strong that it becomes pseudo-deterministic.

## V3 Comparison: Does Any V4 Algorithm Beat Lane Locking?

**No V4 algorithm cleanly dominates Lane Locking.** The pattern is consistent across all 5 agents' simulations: Lane Locking outperforms its paired V4 algorithm on convergence. Only Pack Widening exceeds Lane Locking's S/A (3.35 vs 2.78 in Agent 3's simulation), but both over-converge past the 90% concentration cap.

**Key tradeoff:** V4 algorithms collectively win on variance (stddev 0.92-1.00 vs Lane Locking's 0.70-0.94), splash (1.11-2.43 vs 0.23-1.02 off-arch), and variety (4.9-12.1% vs 6.1-19.1% overlap). Lane Locking wins on convergence strength and speed. The question is whether natural variance is worth ~0.5 less S/A per pack. I believe the answer depends on game feel — Lane Locking delivers reliable but mechanical output, while V4 stochastic approaches deliver natural but insufficient convergence.

**Lane Locking baseline discrepancy:** Each agent's Lane Locking implementation produces different numbers (Late S/A ranges 1.81-2.78), suggesting card pool and fitness model differences. This weakens cross-agent comparisons but within-agent comparisons are consistent.

## Proposed Best Algorithm

**My recommendation: Pack Widening v2 at cost 3/bonus 1 with delayed spending.**

One-sentence: "Each symbol you draft earns 1 matching token (primary earns 2); starting at pick 6, you may spend 3 tokens of one resonance before seeing a pack to add 1 extra card with that primary resonance."

**Rationale:** This is the only V4 mechanism that provably crosses the 2.0 S/A threshold. At cost 3/bonus 1 with delayed spending: convergence should reach ~2.0-2.3 S/A (extrapolating from Agent 3's parameter sweeps), early packs remain open (no spending before pick 6), economic decisions are real (cost 3 means ~60% spending rate, not 100%), and the one-sentence description is genuinely implementable.

**Why not Lane Locking:** Lane Locking's 98% deck concentration and zero-variance pack composition make drafts feel mechanical. If Pack Widening at cost 3 can hit 2.0 S/A while preserving natural variance (stddev ~1.0+) and real player decisions, it's a meaningful improvement even if the average convergence is slightly lower.

**Why not a hybrid:** Any hybrid combining two mechanisms (e.g., Phantoms + Pack Widening) fails the simplicity test. One-sentence descriptions with two independent mechanisms are really two sentences. Simplicity is the #1 design goal.

**Team consensus:** All 5 agents independently converged on Pack Widening at cost 3 as the foundation of any best-V4 algorithm. The remaining debate is bonus 1 vs bonus 2, and whether to add phantoms for signal reading. Agent 1 (Gated Pack Widening, cost 3/bonus 2/pick-5 gate), Agent 3 (standalone cost 3/bonus 1), and I align on keeping it simple — no phantoms. Agent 2 and Agent 5 favor adding phantoms for signal reading. The simplicity argument (goal #1 > goal #8) tips the balance toward standalone.

**Fixability insight from discussion:** Three of five V4 strategies (Exile, Phantoms, Deck Echo) hit STRUCTURAL ceilings — no parameter tuning fixes the resonance-to-archetype dilution. Sqrt Affinity is marginally fixable but loses its identity. Only Pack Widening is genuinely tunable to the target zone. Agent 1's math proved that rejection mechanisms max out at ~14.7% archetype density even with perfect exile, far below the ~50% needed for 2.0 S/A in 4-card packs.

**Caveat:** This recommendation needs simulation validation. Agent 3's cost-3 sweep showed 2.70 S/A with bonus 2; bonus 1 showed 2.34 S/A — both pass. If bonus 1 lands in the 60-90% concentration range (which bonus 2 likely doesn't), it's the better choice despite lower raw convergence.
