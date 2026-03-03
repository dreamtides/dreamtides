# Lane Locking -- V6 Reference Baseline

## One-Sentence Description

> "Your pack has 4 slots; when your weighted symbol count in a resonance first
> reaches 3, one open slot locks to that resonance and always shows a card with
> that primary resonance; a second slot locks at 8."

## Lane Locking Algorithm Adapted for V6

### Proposed Symbol Distribution

With 360 total cards (36 generic, 324 non-generic across 8 archetypes, ~40 per
archetype), subject to the 15% dual-resonance cap (max 54 dual-type cards):

| Category             | Per Archetype |  Total  | Notes                                                                   |
| -------------------- | :-----------: | :-----: | ----------------------------------------------------------------------- |
| Generic (0 symbols)  |      --       |   36    | Playable in any deck                                                    |
| 1 symbol, mono-type  |      10       |   80    | e.g. [Tide]                                                             |
| 2 symbols, mono-type |      17       |   136   | e.g. [Tide, Tide]                                                       |
| 3 symbols, mono-type |       6       |   48    | e.g. [Tide, Tide, Tide]                                                 |
| 2 symbols, dual-type |       4       |   32    | e.g. [Tide, Zephyr]                                                     |
| 3 symbols, dual-type |       3       |   24    | e.g. [Tide, Tide, Zephyr]                                               |
| **Total**            |    **40**     | **360** | 56 dual-type (rounds to ~15.6%; use 54 by trimming 2 archetypes to 3+2) |

To strictly satisfy the 54-card cap: 6 archetypes get 7 dual-type cards (4+3)
and 2 archetypes get 6 (3+3). This distributes dual-type cards as evenly as
possible.

**Weighted symbols per pick (averages for a committed player):**

- 1-symbol mono: 2.0 weighted (primary counts 2)
- 2-symbol mono: 3.0 weighted (2+1)
- 3-symbol mono: 4.0 weighted (2+1+1)
- 2-symbol dual: 3.0 weighted (2+1)
- 3-symbol dual: 4.0 weighted (2+1+1)

Across the pool, the average non-generic card provides ~3.0 weighted symbols in
its primary resonance and ~0.3 in a secondary resonance. A committed player
drafting on-primary cards reaches threshold 3 by pick 1-2 and threshold 8 by
pick 3-4.

### Step-by-Step Algorithm

1. Initialize 4 pack slots as OPEN. Initialize 4 resonance counters at 0.
2. For each pack, fill each slot:
   - LOCKED to resonance R: draw a random card whose primary resonance is R from
     the full pool.
   - OPEN: draw a random card from the full pool (any resonance including
     generic).
3. Present the pack. Player picks 1 card.
4. Update resonance counters: +2 for the card's primary (leftmost) symbol, +1
   for each secondary/tertiary symbol. Generic cards add nothing.
5. Check thresholds for each resonance:
   - Counter first reaches 3: lock one random OPEN slot to that resonance.
   - Counter first reaches 8: lock a second random OPEN slot to that resonance.
   - Maximum 4 locked slots total.
6. Repeat from step 2.

**Locked slot behavior:** A slot locked to Tide always draws from cards with
Tide as primary resonance. This pool includes cards from 2 primary archetypes
(Warriors, Sacrifice) and 2 secondary archetypes (Self-Mill, Ramp). Roughly 50%
of these cards are S/A-tier for any given one of those archetypes. With 2 locked
slots, a committed player sees ~2 resonance-matched cards, of which ~1.0 are S/A
for their specific archetype from locked slots alone. Open slots contribute
additional random S/A hits.

**No pool asymmetry for V6 baseline.** V3's recommended hybrid included pool
asymmetry for signal reading. For V6, this baseline omits it to isolate Lane
Locking's pure performance under the new card pool constraints.

______________________________________________________________________

## Impact of the 15% Dual-Resonance Constraint

### How V3 Differed

V3 used a card pool where ~55% of cards had 2 symbols and many had dual
resonance types. The V3 report did not enforce a strict dual-type cap. Lane
Locking's V3 performance (2.72 S/A, convergence at pick 6.1) was measured
against a pool where dual-type cards were common.

### What the 15% Cap Changes

**Faster threshold accumulation.** With ~85% of non-generic cards being
mono-resonance, a committed player's picks concentrate all their weighted
symbols into a single resonance counter. A mono-type [Tide, Tide] card gives 3
weighted Tide symbols and 0 to any other resonance. In V3's pool, many cards
split symbols across two resonances, diluting the primary counter. Under V6,
threshold 3 is hit on pick 1-2 (vs V3's pick 2) and threshold 8 on pick 3-4 (vs
V3's pick 4-5). This means **faster lock-in but also faster commitment**.

**Locked slots are slightly less archetype-precise.** When a slot locks to Tide,
it draws from all Tide-primary cards. In V3, many of these cards also carried a
secondary resonance that could help disambiguate archetypes (a [Tide, Zephyr]
card is clearly Warriors). Under V6, most Tide-primary cards are mono-type
([Tide], [Tide, Tide], [Tide, Tide, Tide]) -- they could belong to Warriors OR
Sacrifice equally. Only ~7 of the ~40 Tide-primary cards carry a secondary
resonance. This increases the archetype dilution within locked slots from V3
levels.

**Net effect: marginal negative.** Faster thresholds help convergence speed but
locked-slot archetype precision drops. The two effects partially cancel. The
core structural advantage of Lane Locking -- deterministic slot assignment
guaranteeing resonance-matched cards -- remains intact. The slot still shows a
resonance-correct card every time; it just cannot distinguish between the two
primary archetypes sharing that resonance as well as V3's pool could.

**Quantitative estimate:** In V3, locked slots delivered ~75% S/A-tier cards for
the committed archetype. Under V6, with fewer dual-type cards providing
archetype disambiguation, this drops to ~60-65%. A card drawn from the
Tide-primary pool belongs to Warriors with ~50% base probability (home archetype
S-tier) plus some A-tier from the adjacent archetype sharing Tide as primary
(Sacrifice), yielding ~60% S/A overall. The 2 open slots contribute ~25% S/A
randomly (1/8 chance of exact archetype match, plus partial fitness).

______________________________________________________________________

## Predicted Metric Values

All predictions at **archetype level** for a committed player strategy under
V6's card pool with 54 dual-type cards.

| Metric                                         | Target           | Predicted |   Status   | Reasoning                                                                                                            |
| ---------------------------------------------- | ---------------- | :-------: | :--------: | -------------------------------------------------------------------------------------------------------------------- |
| Picks 1-5: unique archetypes with S/A per pack | >= 3             |  6.0-6.5  |    PASS    | Open slots before locking show broad random diversity; same structural behavior as V3's 6.49                         |
| Picks 1-5: S/A for emerging archetype per pack | \<= 2            |  1.5-1.8  |    PASS    | Slightly lower than V3 (1.93) due to fewer dual-type signals; early packs are mostly random                          |
| Picks 6+: S/A for committed archetype per pack | >= 2             |  2.2-2.5  |    PASS    | Down from V3's 2.72; 2 locked slots at ~60-65% S/A = 1.2-1.3 from locks + ~0.9-1.2 from 2 open slots = 2.1-2.5 total |
| Picks 6+: off-archetype (C/F) cards per pack   | >= 0.5           |  0.7-0.9  |    PASS    | 2 open slots produce ~35-40% C/F cards; locked slots contribute ~35-40% off-archetype cards                          |
| Convergence pick                               | 5-8              |    4-5    | BORDERLINE | Faster than V3 due to mono-type concentration; may converge too early for the 5-8 target window                      |
| Deck concentration                             | 60-90%           |  95-99%   |    FAIL    | Same structural problem as V3; deterministic locks funnel nearly all picks through resonance filter                  |
| Run-to-run variety (card overlap)              | < 40%            |   6-10%   |    PASS    | Large pool per resonance (~80-90 cards) ensures low overlap                                                          |
| Archetype frequency                            | no >20%, no \<5% |   8-19%   |    PASS    | Lane Locking treats all archetypes symmetrically; no structural bias                                                 |

**Variance prediction:**

| Metric                            | Target | Predicted |   Status   |
| --------------------------------- | ------ | :-------: | :--------: |
| StdDev of S/A per pack (picks 6+) | >= 0.8 | 0.75-0.90 | BORDERLINE |

### Summary Assessment

Lane Locking under V6's 15% dual-resonance cap remains a viable algorithm that
crosses the 2.0 S/A threshold. However, performance degrades modestly from V3
levels:

- **Late S/A drops from 2.72 to ~2.2-2.5** due to reduced archetype precision
  within locked resonance slots (fewer dual-type cards to disambiguate).
- **Convergence accelerates to pick 4-5** (potentially too fast) because
  mono-type cards concentrate all symbols into one counter.
- **Deck concentration remains the primary failure** at 95-99%, unchanged from
  V3. This is inherent to deterministic slot locking.
- **Variance is borderline** at the 0.8 threshold, as locked slots produce
  mechanically consistent packs.

The algorithm passes 6/8 metrics cleanly, with convergence pick borderline-early
and deck concentration failing. It remains a strong convergence mechanism but
its weaknesses (mechanical feel, over-concentration, no pivot flexibility) are
unchanged from V3. The 15% constraint makes it marginally worse at archetype
targeting but marginally faster at commitment -- roughly a wash.
