# Comparison Agent 9: Narrative Gravity Perspective

## Defending My Algorithm: Why Pool Contraction Is the Right Paradigm

Every other V8 algorithm is a slot-filling mechanism: it fills pack slots with
cards drawn from targeted subpools. This approach has a fundamental ceiling: the
quality of each slot is bounded by the precision of the subpool filter. Under
realistic fitness, R1-filtered slots deliver 62-75% S/A, pair-matched slots
deliver 80-85% S/A. These precisions create hard M3 ceilings that can only be
exceeded by filling more slots (pushing toward M6=99%) or by increasing fitness
(which the card designer cannot easily do).

Narrative Gravity breaks this paradigm. Instead of targeting individual slots,
it contracts the entire pool. As picks accumulate, cards with low relevance to
the player's resonance signature are permanently removed. This means *every*
slot -- including "random" slots -- draws from an increasingly concentrated
pool. The precision is not fixed at 80% or 85%; it rises to 95-100% as the pool
contracts below 50 cards.

This explains why Narrative Gravity achieves the highest M3 among algorithms
with acceptable M6: at 2.75 under Graduated Realistic, it exceeds Continuous
Surge (2.48), Symbol-Weighted (2.50 on symbol-rich), GPE-45 (2.25), and
Pair-Esc. baseline (2.16). Only CSCT (2.92) is higher, and it does so by
saturating all slots with pair-matched cards (M6=99%).

## Comprehensive Scorecard (Graduated Realistic, 40% Enriched)

| Algorithm            |    M3    |  M4  |  M5  | M6  |  M7  |    M9    |   M10   | All-Arch >= 2.0 | Total Pass |
| -------------------- | :------: | :--: | :--: | :-: | :--: | :------: | :-----: | :-------------: | :--------: |
| 9. Narr. Gravity     | **2.75** | 1.25 | 10.2 | 85% | 5.3% | **1.21** |   3.3   |     **Yes**     |    7/10    |
| 7. CSCT              | **2.92** | 0.47 | 5.0  | 99% | 7.8% |   0.68   | **2.0** |      Yes\*      |    5/10    |
| 5. Sym-Weighted (SR) | **2.50** | 1.50 | 9.2  | 83% | 21%  | **1.18** |   4.3   |   No (Blink)    |    6/10    |
| 2. Cont. Surge       | **2.48** | 1.23 | 3.1  | 85% | 3.2% |   0.78   |   3.8   |    No (Ramp)    |    5/10    |
| 4. GPE-45            |   2.25   | 1.23 | 12.5 | 67% | 17%  |   0.51   |   8.2   |   No (Flash)    |    4/10    |
| 1. Pair-Esc.         |   2.16   | 1.84 | 5.8  | 89% |  --  | **1.00** |   8.0   |       Yes       |    6/10    |
| 6. GF+PE             |   1.72   | 1.78 | 7.5  | 76% |  8%  |   0.74   |   6.3   |    No (Ramp)    |    4/10    |
| 3. Esc. Pair Lock    |   1.50   | 2.00 | 16.8 | 64% | 20%  | **1.23** |   8.8   |       No        |    3/10    |
| 8. Comp. Pair        |   1.45   | 2.55 | 11.4 | 62% | 13%  | **0.83** |   6.9   |       No        |    4/10    |

\*CSCT disqualified by M6=99% despite passing M3.

**Narrative Gravity passes 7/10 metrics -- the most of any algorithm.** Its two
failures (M5=10.2, M10=3.3) and one marginal (M6=85%) are the mildest failure
set.

## Player Experience Rating (1-10)

| Algorithm         | Rating | Key Experience                                                              |
| ----------------- | :----: | --------------------------------------------------------------------------- |
| 9. Narr. Gravity  |   9    | Monotonic ramp, "funnel" feels roguelike, high variety, all archetypes work |
| 7. CSCT           |   6    | Smooth but sterile; every pack identical after pick 5                       |
| 5. Sym-Weighted   |   7    | Good graduated ramp; requires demanding pool                                |
| 2. Cont. Surge    |   5    | Unimodal but unpredictable droughts                                         |
| 1. Pair-Esc.      |   5    | Reliable baseline                                                           |
| 4. GPE-45         |   4    | Dead zone                                                                   |
| 6. GF+PE          |   3    | Floor concept fails                                                         |
| 8. Comp. Pair     |   2    | Misalignment                                                                |
| 3. Esc. Pair Lock |   1    | Broken                                                                      |

I rate my own algorithm 9/10 -- the highest any agent gives any algorithm --
because I believe the monotonic quality ramp is the single most important player
experience property. Research Agent C established that floors matter more than
ceilings, and Narrative Gravity provides the most reliable floor: quality never
decreases once the pool begins contracting. The M10 = 3.3 weakness is real but
concentrated in picks 6-10, which is psychologically the "building" phase where
players tolerate variance.

## KEY QUESTION 1: M3 >= 2.0 under harshest fitness?

Under Hostile fitness, Narrative Gravity achieves M3 = 2.49 with worst archetype
(Flash) at 2.16. Under Pessimistic, M3 = 2.59 with worst archetype at 2.13.
Under Graduated Realistic, M3 = 2.75 with worst archetype at 2.40.

**All 8 archetypes exceed 2.0 under all four fitness models on the 40% Enriched
pool.** No other algorithm (except CSCT, disqualified by M6) achieves this. This
is Narrative Gravity's defining contribution.

The per-archetype results under Graduated Realistic:

| Archetype    |  M3  | Status |
| ------------ | :--: | :----: |
| Warriors     | 3.13 |  Pass  |
| Self-Mill    | 3.03 |  Pass  |
| Storm        | 2.94 |  Pass  |
| Ramp         | 2.92 |  Pass  |
| Sacrifice    | 2.69 |  Pass  |
| Self-Discard | 2.53 |  Pass  |
| Blink        | 2.51 |  Pass  |
| Flash        | 2.40 |  Pass  |

## KEY QUESTION 2: Best feel?

Narrative Gravity. The mechanism is the most intuitive of any algorithm: "the
game learns what you want and removes what you do not want." The player observes
packs getting progressively better without needing to understand counters,
thresholds, ratios, or surges. This is the "magical algorithm" that Research
Agent C described as optimal: complex internally, simple experientially.

The late-draft pool contraction to 10-15 cards is a concern but also a feature:
by pick 25, the player is seeing their "best" cards repeatedly, reinforcing the
feeling that the draft has converged on their strategy. This is similar to
late-draft MtG formats where the pack has been heavily drafted and only a few
playables remain.

## Addressing My Weaknesses

**M5 = 10.2 (target 5-8):** Convergence is slow because pool contraction takes
~10 picks to concentrate the pool. This could be improved by starting
contraction at pick 2 instead of pick 4, or by increasing the initial
contraction rate to 18% per pick for picks 4-6 then dropping to 12% for picks
7+.

**M10 = 3.3 (target \<= 2):** The picks 6-10 transition window produces
occasional bad streaks. The proposed fix (add a pair-matched floor slot from
pick 3+) would address this directly. With 1 guaranteed pair-matched slot
providing ~80% S/A precision, the probability of a zero-quality pack drops from
~10% to ~2%.

**M6 = 85% (target ceiling 90%):** Marginal. The 12% contraction rate keeps M6
at 85-86%, well within the 60-90% target. Higher rates push above 90%.

## Proposed Best Algorithm

**Narrative Gravity + Floor (Hybrid)**

- Pool contracts at 12% per pick from pick 4 using dot-product relevance scoring
- From pick 3+: 1 slot guaranteed pair-matched from contracted pool's
  top-quartile relevance
- Remaining 3 slots drawn randomly from full contracted pool
- Generics protected at 0.5 baseline relevance

**Pool: 40% Enriched, Compensated (per Agent 8's insight)**

**Complete Set Design Specification:**

| Archetype            |  Total  | Home-Only | Dual-Res | Generic |
| -------------------- | :-----: | :-------: | :------: | :-----: |
| Flash (Ze/Em)        |   40    |    22     |    18    |   --    |
| Blink (Em/Ze)        |   40    |    22     |    18    |   --    |
| Storm (Em/St)        |   40    |    22     |    18    |   --    |
| Self-Discard (St/Em) |   40    |    24     |    16    |   --    |
| Self-Mill (St/Ti)    |   40    |    24     |    16    |   --    |
| Sacrifice (Ti/St)    |   40    |    26     |    14    |   --    |
| Warriors (Ti/Ze)     |   40    |    26     |    14    |   --    |
| Ramp (Ze/Ti)         |   40    |    22     |    18    |   --    |
| Generic              |   40    |    --     |    --    |   40    |
| **Total**            | **360** |  **188**  | **132**  | **40**  |

**Symbol Distribution:** 40 generic (11%), 188 single-res (52%), 132 dual-res
(37%).

**Per-Resonance R1:** 80 cards per resonance. Pair subpool: 14-18 per archetype
(compensated).

**Minimum fitness: Graduated Realistic (36% avg).** But the algorithm works down
to Hostile (M3 = 2.49).

**Minimum pool change:** 132 dual-res cards (up from 54). Cost: moderate flavor
design work. No changes to card count, pool size, or pack size.

## Recommendations to the Card Designer

1. Raise dual-res to ~37% (132 cards). Compensate low-overlap pairs with 2 extra
   cards each.
2. Symbol assignment is the primary design task. Every non-generic card needs
   correct primary resonance; 37% need secondary. This is a tagging exercise,
   not a mechanical design constraint.
3. Create 4-5 bridge cards per low-overlap pair (Flash/Ramp, Blink/Storm). These
   should be generic utility effects (unconditional draw, removal, efficient
   bodies) carrying the correct pair symbols.
4. The algorithm explanation for players: "As you draft, packs will show more
   cards matching your style. The more consistently you draft, the faster your
   packs improve." One sentence, fully accurate.
5. The mechanism is hidden. Players never see pool sizes, contraction rates, or
   relevance scores. They observe the result: "my packs keep getting better."
   This is the "magical algorithm" that V8 research identified as optimal.
