# Discussion Output: Agent 1 (Rejection & Passing Domain)

## Simplicity Ranking (Most to Least Simple)

1. **Pack Widening (Economic)** — "Earn tokens, spend to add a card." One active
   decision, trivially implementable. A programmer writes this in 20 minutes.
2. **Multiple Phantoms (Phantom)** — "3 phantoms each remove 1 card/round." Dead
   simple concept, but "preferring" a resonance is vague — the phantom's card
   selection heuristic is hand-waved.
3. **Square-Root Affinity Sampling (Soft Prob)** — One formula, fully
   determined. "Square root" is one extra concept but concrete and
   unambiguous. A programmer can implement it from the description alone.
4. **Exile Pressure (Rejection)** — My Round 1 one-sentence was *incomplete*:
   it omitted the decay mechanic (-1/pick) essential for pivoting. The
   revised description is complete but long — closer to a one-paragraph
   description.
5. **Deck Echo Filter (Filtered)** — The *most honest* one-sentence description
   (specifies everything including the fallback mechanism), but also the
   densest: three operations (draw 10, filter independently, fill from
   rejects), a formula with division, and an edge case handler.

## Scorecard Table

| Goal | Exile Pressure | Sqrt Affinity | Pack Widening | Multiple Phantoms | Deck Echo Filter |
|------|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 5 | 7 | 9 | 8 | 4 |
| 2. Not on rails | 7 | 7 | 9 | 6 | 7 |
| 3. No forced decks | 5 | 4 | 6 | 9 | 5 |
| 4. Flexible archetypes | 7 | 8 | 7 | 3 | 7 |
| 5. Convergent | 4 | 8 | 3 | 4 | 6 |
| 6. Splashable | 6 | 5 | 8 | 7 | 7 |
| 7. Open early | 9 | 8 | 7 | 4 | 8 |
| 8. Signal reading | 6 | 2 | 3 | 9 | 3 |
| **Total** | **49** | **49** | **52** | **50** | **47** |

**Scoring rationale for key cells:**

- Exile Pressure scores 4 on convergence because resonance-level suppression
  cannot distinguish archetypes within a resonance. ~50% of "on-resonance"
  cards serve the wrong archetype, capping expected S/A at ~1.2-1.5.
- Sqrt Affinity scores 8 on convergence because per-card symbol overlap
  naturally distinguishes [Tide, Zephyr] Warriors from [Tide, Stone] Sacrifice.
- Pack Widening scores 3 on convergence per its own analysis (0.75 S/A projected).
- Multiple Phantoms scores 3 on flexibility because 3 contested resonances
  narrows viable primary archetypes to ~2-3 of 8.
- Multiple Phantoms scores 9 on signal reading — it's the only algorithm
  where signal reading is a first-class native feature.

## Final Championed Algorithm

**Exile Pressure (revised)** — I am sticking with my Round 1 champion but with
significant modifications to address the two critical weaknesses identified
during discussion.

**Revised one-sentence description:** "When you pass a card, add 2 to its
primary resonance's exile counter and 1 per secondary/tertiary symbol; all
counters decay by 1 each pick; each pack card is independently skipped with
probability (its primary resonance's counter / 20), rerolling on a skip."

## Specific Modifications for Round 3

1. **Multi-symbol exile increment.** Round 1 only incremented the primary
   resonance counter (+1). Now: +2 for primary, +1 for each secondary/tertiary
   symbol. Three passed cards generate ~6-9 exile increments per pick instead
   of 3, roughly doubling convergence pressure.

2. **Decay included in the one-sentence.** Round 1's one-sentence omitted the
   -1/pick decay. This was dishonest — the decay is essential for pivoting
   (Goal 2). The revised one-sentence includes it at the cost of length.

3. **Divisor reduced from 30 to 20.** Higher skip probabilities: counter 10 =
   50% skip instead of 33%. Combined with multi-symbol input, a committed
   player should see ~40-50% skip rates for off-archetype resonances by pick
   10-12.

4. **Exile counter cap at 20.** Prevents runaway suppression in late draft.
   At cap, the skip probability is 100% — effectively removing that resonance.

**Honest limitations I will test in simulation:**
- Archetype-level convergence will likely fall short of the 2.0 S/A target
  because exile pressure operates on resonance, not archetype.
- The one-sentence is now long (one-paragraph). Simplicity has regressed.
- Convergence pick will likely be 8-10, slightly late vs. the 5-8 target.

## Proposed Symbol Distribution for Simulation

| Symbol Count | % of Non-Generic | Cards |
|---|---|---|
| 0 (generic) | — | 36 |
| 1 symbol | 30% | 97 |
| 2 symbols | 50% | 162 |
| 3 symbols | 20% | 65 |

**Rationale:** 2-symbol cards at 50% provide the best signal for multi-symbol
exile (each passed 2-symbol card generates 3 exile increments). The 30%
1-symbol cards create clean single-resonance exile targets. 20% 3-symbol cards
generate high exile input (4 increments each) and provide rich archetype
signal for evaluation.

## Cross-Domain Structural Insight

The most important finding from discussion: **algorithms that use per-card
symbol overlap scoring (Sqrt Affinity, Deck Echo Filter) have a structural
advantage over algorithms that operate at the resonance level (Exile Pressure,
Multiple Phantoms, Pack Widening).** Per-card overlap naturally distinguishes
archetypes within a resonance (e.g., [Tide, Zephyr] vs. [Tide, Stone]),
while resonance-level mechanisms treat all Tide cards equally. This is why
Domains 2 and 5 are likely to produce the strongest archetype-level
convergence.

The rejection domain's contribution may ultimately be philosophical rather
than mechanical: the insight that *what you pass on carries information* could
be layered onto any winning algorithm as a secondary signal (e.g., temporarily
removing passed cards from the candidate pool of a Deck Echo Filter).
