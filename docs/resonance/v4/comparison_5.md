# Comparison 5: Agent 5 (Deck Echo Filter)

## Scorecard Table

| Goal | Exile | SqrtAff | PackWide | Phantom | DeckEcho |
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

**Key justifications:** Exile's verbose description hurts simplicity (5). PackWide's always-spend at cost=2 means no real choice (rails=6, forced=5). PackWide is the only strategy passing 2.0 S/A but over-converges at 98.6% (convergent=8, not 10). Phantom has worst convergence (1.26, score=2) but best signal reading (10). SqrtAff's concave scaling supports hybrids (flex=8) but converges too slowly at pick 10.5. DeckEcho produces genuinely organic variance but is capped at 1.55 S/A by candidate pool density.

## Strength and Weakness

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|------------------|
| Exile | Tightest archetype balance (6.8-7.7 spread) | S/A ceiling ~1.57; unfixable |
| SqrtAff | Most natural soft bias with diminishing returns | Convergence pick 10.5, far too slow |
| PackWide | Only V4 strategy passing 2.0 S/A target | Trivial decisions at cost=2; early narrowing (2.48) |
| Phantom | First-class signal reading; most elegant model | Worst convergence (1.26); subtraction too weak |
| DeckEcho | Genuine natural variance (14% zero-S/A, 14% 3+) | 1.55 S/A structurally capped |

## Proposed Improvements

**Exile/DeckEcho/Phantom:** All structurally capped by resonance-archetype dilution. No parameter tuning reaches 2.0 S/A. Only fix: hybridize with a card-injection mechanism (Pack Widening-style bonus cards).

**SqrtAff:** Aggressive tuning (base=0.5, no cap) reaches 2.11 S/A but over-concentrates at 94.2%. Fixable in theory but loses its signature splash advantage.

**PackWide:** Cost 3 restores real decisions (S/A=2.70, stddev=1.34). Bonus 1 instead of 2 reduces over-concentration. Cost 3 is self-limiting early, fixing the 2.48 early S/A problem without needing an explicit gate.

## V3 Comparison

**No individual V4 algorithm clearly dominates Lane Locking.** Lane Locking passes convergence (2.08-2.39 S/A) and timing (pick 5.7-6.5). All stochastic V4 approaches fail convergence due to the ~50% archetype dilution ceiling. PackWide passes convergence but over-concentrates worse (98.6% vs 96-98%).

**V4 wins collectively on:** variance (stddev 0.92-1.00 vs 0.70-0.84), splash (1.11-2.43 vs 0.23-1.02), deck diversity (72-89% vs 96-98%), and run variety (4.9-12.1% vs 6-19% overlap). Lane Locking over-converges past the 90% cap.

## Proposed Best Algorithm: Modified Pack Widening v3

**One sentence:** "Each symbol you draft earns 1 matching token (primary earns 2); you may spend 3 tokens of one resonance to add 1 extra card with that primary resonance to the pack."

**Why this wins:** Pack Widening is the only V4 mechanism where convergence is fixable by tuning. The four stochastic approaches hit a structural ceiling (1.26-1.74 S/A) because resonance-based filtering cannot overcome ~50% archetype dilution. Pack Widening sidesteps this by ADDING cards.

**Changes from v2:** Cost 3 (was 2) creates genuine save/spend rhythm — spending roughly every other pick. Bonus 1 (was 2) gives 5-card packs on spend turns instead of 6, reducing concentration toward 60-90%. No gate needed: cost 3 self-limits early spending.

**Expected metrics:** Late S/A ~2.3-2.5 (PASS). StdDev ~1.2-1.4 (PASS). Deck concentration ~80-88% (needs verification). Convergence pick ~6-7 (PASS).

**Why not Phantoms?** I initially proposed a phantom hybrid for signal reading. The group split 3:2 against phantoms — simplicity (goal 1) outweighs signal reading (goal 8). The single-mechanism version is genuinely one sentence.

**Why this beats Lane Locking:** Same convergence mechanism (both ADD resonance-matched cards) but Pack Widening gives visible player agency instead of invisible slot assignment. Lane Locking's 96-98% concentration fails 60-90%; intermittent voluntary spending should land in range.

## Key Structural Insight

V4's design space divides into **convergence engines** (Pack Widening, Lane Locking) that ADD cards to overcome dilution, and **variance layers** (Exile, Sqrt, Phantom, Deck Echo) that filter existing pools but cap at 1.26-1.74 S/A. No tuning converts a variance layer into a convergence engine. The ~11% archetype density in a 360-card pool means resonance-level biasing can only push to ~15-18% — never enough for 2+ S/A in 4-card packs.

**Group consensus:** All 5 agents converged on cost-3 Pack Widening. Minor disagreements (bonus 1 vs 2, phantoms yes/no) should be resolved by Round 5 simulation.
