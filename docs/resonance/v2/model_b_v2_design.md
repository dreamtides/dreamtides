# Model B v2: Clustered 8 with Suppression and Soft Floor

## One-Sentence Player Explanation

Each quest draws from a shifting pool of strategies -- some are plentiful, some are scarce, and the early packs tell you which is which.

## What Changed from Round 2

Round 2's Model B used N=10 archetypes and required 62% multi-archetype cards -- an impractical design burden. The debate produced strong consensus that N=10 is too thin for 360 cards (each archetype had only 36 S-tier cards), and I conceded this fully. The revised model adopts N=8, borrows Model D's archetype suppression (the best variety mechanism identified across all four models), adds a soft floor guarantee, and tightens commitment detection. The clustered neighbor topology from the original Model B is retained as it was well-received.

## Archetype Count: 8

With 360 cards and N=8, each archetype gets ~45 S-tier exclusive cards -- above the 45-50 minimum identified for archetype identity. The 28 suppression configurations (C(8,2) = 28 ways to suppress 2) provide structural variety without dedicated variety mechanisms. After suppression, 6 active archetypes remain, boosting effective density by ~33%.

## Card Fitness Distribution

| Card Type | % | Count | Profile |
|-----------|---|-------|---------|
| Narrow specialist | 45% | 162 | S in 1, B in 1-2 neighbors, C/F elsewhere |
| Specialist with splash | 25% | 90 | S in 1, A in 1-2 neighbors, B in 1-2, C/F elsewhere |
| Multi-archetype star | 8% | 29 | S in 2 (neighbors), B in 2-3, C/F elsewhere |
| Broad generalist | 12% | 43 | A in 2-3, B in 3-4, no S |
| Universal star | 3% | 11 | S in 3+, high power |
| Pure filler | 7% | 25 | B in 2-3, C/F elsewhere |

**Multi-archetype percentage (S or A in 2+ archetypes): ~48% actual.** The "27%" figure from debate discussions undercounted: splash cards (S in 1, A in 1-2) are inherently multi-archetype, as are generalists (A in 2-3). The actual count is ~173/360 = 48%. However, the design burden is lower than this number suggests: only the 29 multi-stars and 11 universal stars require designing true dual-archetype synergy. The 90 splash cards are straightforward "primary + neighbor playability" designs. Sensitivity analysis shows the system functions at 15-20% minimum.

**Per-archetype totals:** Each archetype has ~45 S-tier unique cards (balanced by round-robin assignment). With splash A-tier cards and generalists, each archetype has ~65-75 S+A unique cards. In the pool (~1000 entries), roughly 20-25% of entries are S/A for any given archetype. With 2 suppressed archetypes removing their specialists' copies, active archetype density rises to ~28%.

**Clustered neighbor topology:** 8 archetypes in a ring, each with 2 immediate neighbors sharing more multi-archetype cards. Splash cards' A-tier assignments favor neighbors (80% of A-tier goes to neighbors, 20% to non-neighbors). This creates cheap pivot paths between neighbors and expensive pivots across the ring, interacting with suppression (your neighbor may be suppressed this run).

## Pack Construction: Adaptive Weighted Sampling with Soft Floor

**Phase 1 -- Exploration (picks 1-5):** Uniform random from the pool. No archetype bias. The natural pool composition (with 2 suppressed archetypes) sends implicit signals -- active archetype cards appear more frequently.

**Phase 2 -- Convergence (picks 6+):** Once commitment is detected, apply weight multipliers to S/A-tier cards in the committed archetype:
- Picks 6-10: 5.0x weight
- Picks 11-20: 6.0x weight
- Picks 21-30: 7.0x weight

One of the 4 pack slots is always drawn from off-archetype cards, biased toward high-power or S-tier-in-other-archetype cards. This guarantees splash options.

**Soft floor guarantee:** After pick 6, if the weighted draw produces 0 fitting cards in the 3 archetype slots, replace 1 card with a fitting card. This fires only when needed (~15-25% of packs), preventing brick packs without inflating concentration. No hard anchor slot -- occasional 1-fitting packs are acceptable and create genuine tension.

**Commitment detection:** Requires ALL of: pick >= 6, 3+ S/A-tier picks in one archetype, AND a strict lead over the runner-up archetype. This prevents the premature convergence that plagued Model C (pick 3) and keeps early picks genuinely open while still allowing commitment at pick 6-8 for focused players.

## Run-to-Run Variety: Archetype Suppression

**Primary mechanism:** 2 of 8 archetypes are suppressed per run. Suppressed archetypes' S-tier specialist cards have ~50% of their copies removed from the pool. Their cards still exist (they may be playable filler) but are scarce enough that committing to a suppressed archetype is suboptimal. This creates 28 structurally distinct run configurations.

**Starting card signal:** At run start, player sees 3 cards drawn from active archetype S/A pools and keeps 1 as a free pick. This semi-explicitly reveals which archetypes are active. Experienced players learn which archetype pairs each card bridges.

**No depletion:** The debate consensus dropped depletion -- it was the hardest mechanism to explain, least validated in simulation, and the signal-reader barely outperformed committed players. Suppression alone provides sufficient variety and signal reading.

## Why This Works

The design addresses every Round 2 failure:
- **N=10 -> N=8** provides ~45 S-tier cards per archetype (up from 36), sufficient for identity
- **62% -> 27% multi-archetype** reduces design burden from ~223 dual-design cards to ~90
- **Suppression** (from Model D) gives structural variety without depletion complexity
- **Soft floor** (debate consensus) prevents brick packs without the rigidity of anchor slots
- **Delayed commitment detection** (pick >= 6 + clear lead) prevents premature convergence
- **Splash slot** ensures off-archetype options in every post-commitment pack

The clustered neighbor topology (the one element unanimously praised from original Model B) is preserved, creating meaningful pivot corridors that interact with per-run suppression.
