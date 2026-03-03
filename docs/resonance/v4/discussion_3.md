# Discussion Output: Agent 3 (Economic & Resource Domain)

## Simplicity Ranking (Most to Least Simple)

1. **Multiple Phantoms (Domain 4)** — "Three phantoms each remove 1 card per
   round" is the simplest concept. One ambiguity: how phantoms select their
   card isn't fully specified. But the core is implementable from one sentence.

2. **Pack Widening v2 (Domain 3, mine)** — "Earn tokens, spend to add cards" is
   fully specifiable. The earn/spend/effect are all concrete, discrete
   operations. No hidden math, no edge cases.

3. **Deck Echo Filter (Domain 5)** — Secretly two mechanisms: the stochastic
   filter AND the fallback ("fill from rejects if fewer than 4 survive"). The
   candidate pool size (10) is an arbitrary parameter baked into the sentence.
   "Symbol overlap" needs clarification.

4. **Square-Root Affinity Sampling (Domain 2)** — The math is precise but
   "total resonance overlap" is ambiguous. A programmer could produce 3-4
   different implementations from the sentence. Best *behavioral* simplicity
   ("matching cards more likely, diminishing returns") but the algorithmic
   sentence needs work.

5. **Exile Pressure (Domain 1)** — Critical simplicity failure: the one-sentence
   description omits the decay rule (counters decay by 1 per pick), which is
   load-bearing for pivoting and preventing over-convergence. Without decay,
   exile counters grow monotonically to 100% skip. Adding decay makes the
   sentence three clauses of complexity.

## Scorecard Table

| Goal | Exile Pressure | Sqrt Affinity | Pack Widening v2 | Multiple Phantoms | Deck Echo Filter |
|------|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 4 | 6 | 8 | 9 | 5 |
| 2. Not on rails | 7 | 7 | 9 | 6 | 7 |
| 3. No forced decks | 5 | 4 | 5 | 9 | 5 |
| 4. Flexible archetypes | 6 | 8 | 7 | 5 | 7 |
| 5. Convergent | 6 | 8 | 5 | 5 | 7 |
| 6. Splashable | 6 | 5 | 8 | 7 | 7 |
| 7. Open-ended early | 8 | 8 | 7 | 6 | 8 |
| 8. Signal reading | 3 | 2 | 3 | 10 | 2 |
| **Weighted Total** | **5.6** | **6.3** | **6.6** | **6.5** | **6.1** |

*Weighted total uses priority weighting: Goal 1 = 1.5x, Goals 2-4 = 1.2x,
Goals 5-8 = 1.0x.*

**Key observations:**
- **Sqrt Affinity and Pack Widening** score highest overall but for different
  reasons: Sqrt has the best convergence, Pack Widening has the best agency and
  splash.
- **Multiple Phantoms** has the most polarized profile (5-10 range). Signal
  reading is a unique, first-class strength. Archetype flexibility improves
  with phantom overlap configurations (2/3 of runs have 4+ viable archetypes).
- **Deck Echo Filter** is the most balanced (5-8 range on everything) but has
  no standout strength.
- **Exile Pressure** is penalized by hidden complexity and invisible state.

## Final Championed Algorithm: Pack Widening v2

I am keeping Pack Widening as my champion but with significant modifications.
The economic domain's unique contribution — player agency over *when* to apply
influence — produces variance through player choice rather than probability,
which is structurally different from every other domain. This agency is worth
preserving even if raw convergence numbers are lower.

**One-sentence description (revised):** "Each symbol you draft earns 1 matching
token (primary earns 2); before seeing a pack, you may spend 2 tokens of one
resonance to add 2 extra cards with that primary resonance to the pack."

## Specific Modifications for Round 3

1. **Spend cost reduced: 3 → 2 tokens.** A committed player earning ~3
   tokens/pick can spend on ~75% of picks instead of ~50%, increasing
   convergence frequency.

2. **Bonus cards increased: 1 → 2.** Spend-packs are 6 cards (4 random + 2
   primary-resonance filtered). This doubles convergence effect per spend.

3. **Primary resonance filter on bonus cards.** Bonus cards must have the spent
   resonance as their PRIMARY symbol. This narrows from 4 archetypes to 2
   (e.g., spending Tide gets Warriors OR Sacrifice), approximately doubling
   archetype-level hit rate from ~25% to ~50%.

4. **Projected convergence (corrected):** Base S/A rate in a random 4-card
   pack is ~0.88 (22% of pool is S/A for any given archetype). On spend picks
   (~75% of picks), 2 primary-resonance bonus cards add ~1.0 S/A (50% hit rate
   each). Spend-pack total: ~1.88 S/A. Non-spend packs: ~0.88 S/A. Weighted
   average: 0.75 × 1.88 + 0.25 × 0.88 = **~1.63 S/A per pack**. Below the 2.0
   target but within parameter-tuning range (spend cost, bonus count, primary
   weight). Note: economic mechanisms may serve best as a complementary layer
   atop a passive convergence algorithm rather than as standalone winners.

5. **Complementary layer thesis (key discussion finding).** The cross-domain
   discussion revealed a convergence ranking: Sqrt Affinity > Deck Echo Filter
   > Pack Widening v2 (~1.63 S/A) > Exile Pressure (~1.0-1.1) > Multiple
   Phantoms. Algorithms using per-card overlap scoring structurally outperform
   resonance-level mechanisms. Pack Widening's unique contribution is *player
   agency* — choosing when to exert influence — which no other domain provides.
   Round 3 simulation should test both standalone Pack Widening v2 AND Pack
   Widening as a complement to a probabilistic base (e.g., mild Sqrt Affinity +
   token spending for pack widening). The economic domain's best outcome may be
   "the agency layer that makes the winner more engaging" rather than "the
   winner itself."

**Parameter variants to test in simulation:**
- Spend cost sweep: 1, 2, 3 tokens
- Bonus card count: 1 vs 2 vs 3
- Primary weight: 2 vs 3 (higher primary weight = faster token accumulation)
- Whether bonus cards are drawn from primary-only or any-matching pool

## Proposed Symbol Distribution

| Symbols | % of non-generic | Cards |
|---------|:---:|:---:|
| 0 (generic) | — | 36 |
| 1 symbol | 20% | 65 |
| 2 symbols | 55% | 178 |
| 3 symbols | 25% | 81 |

**Rationale:** 2-symbol cards dominate to provide ~3 tokens per pick, funding
frequent spending. The 25% at 3 symbols creates occasional high-earn picks (4
tokens) for double-spending or accelerated saving. The 20% at 1 symbol creates
natural income variance. Average ~3.1 tokens per pick for a committed player.
The primary resonance filter on bonus cards means 2-symbol cards in the bonus
pool carry clear archetype signals (e.g., [Tide, Zephyr] = Warriors, [Tide,
Stone] = Sacrifice).
