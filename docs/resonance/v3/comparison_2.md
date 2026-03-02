# Cross-Comparison — Agent 2 (Structural Domain)

## Scorecard (1-10, based on simulation data)

| Goal | S1 Weighted Lottery | S2 Balanced Pack | S3 Lane Locking | S4 Echo Window | S5 Resonance Swap |
|------|:---:|:---:|:---:|:---:|:---:|
| 1. Simple | 8 | 9 | 8 | 9 | 5 |
| 2. Not on rails | 7 | 7 | 4 | 8 | 8 |
| 3. No forced decks | 7 | 8 | 7 | 7 | 9 |
| 4. Flexible archetypes | 7 | 6 | 5 | 7 | 7 |
| 5. Convergent | 8 | 7 | 5 | 8 | 3 |
| 6. Splashable | 8 | 9 | 7 | 4 | 8 |
| 7. Open early | 6 | 9 | 8 | 4 | 8 |
| 8. Signal reading | 4 | 3 | 3 | 2 | 9 |
| **Total** | **55** | **58** | **47** | **49** | **57** |

### Scoring Justifications

**S1 Weighted Lottery:** Simple (8) — clear formula but tracking 4 floating weights is nontrivial. Not on rails (7) — wildcard helps but late weights snowball. No forced (7) — 7.5% overlap excellent. Flexible (7) — dual-resonance archetypes work via weight accumulation in two channels. Convergent (8) — 2.31 arch-fit at pick 6.4, passes cleanly. Splashable (8) — 1.69 off-arch via wildcard guarantee. Open early (6) — early-fit 2.02 marginally fails <=2 cap. Signal (4) — amplifies player signals but no pool-awareness.

**S2 Balanced Pack:** Simple (9) — binary state (1/1/1/1 or 2/1/1/0), trivially predictable. Not on rails (7) — guaranteed non-majority slots prevent full lock-in, but majority snowballs. No forced (8) — 5.8% overlap, excellent variety. Flexible (6) — only supports one resonance majority, not dual-resonance simultaneously. Convergent (7) — 2.08 arch-fit passes but barely. Splashable (9) — 1.92 off-arch, best of all strategies. Open early (9) — 3.72 unique resonances, pre-majority packs are perfect 1/1/1/1. Signal (3) — no pool mechanism whatsoever.

**S3 Lane Locking:** Simple (8) — binary lock state with clear thresholds, very transparent. Not on rails (4) — permanent locks are fundamentally anti-flexibility. No forced (7) — 5.6% overlap. Flexible (5) — 4-lock cap pressures single resonance pairs. Convergent (5) — 1.83 arch-fit **fails** the 2.0 target across all parameter sweeps. Splashable (7) — 0.84 off-arch passes. Open early (8) — 3.32 unique resonances, no locks before first pick. Signal (3) — locks respond only to player's own picks.

**S4 Echo Window:** Simple (9) — clear formula, predictable from last 3 cards. Not on rails (8) — 3-pick memory allows cheap pivots; best flexibility. No forced (7) — 8% overlap, even distribution. Flexible (7) — second-resonance slot supports dual archetypes. Convergent (8) — 2.83 arch-fit exceeds target, possibly over-convergent at 84% S/A. Splashable (4) — 0.43 off-arch **fails** 0.5 target for committed players. Open early (4) — 2.58 early arch-fit **fails** <=2 cap; biases from pick 1. Signal (2) — uses only player's own picks, no pool awareness.

**S5 Resonance Swap:** Simple (5) — reserve is hidden infrastructure the player can't see or reason about. Not on rails (8) — gentle pool shifts keep all paths open. No forced (9) — 6.5% overlap, lowest of all. Flexible (7) — pool always contains all resonances. Convergent (3) — 1.61 fitting **fails** badly; parameter sweep proves structurally unfixable. Splashable (8) — 1.17 off-arch well above target. Open early (8) — 3.48 unique resonances. Signal (9) — 44.8% detection rate, strongest by far.

## Biggest Strength / Weakness per Strategy

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|------------------|
| S1 Weighted Lottery | Best convergence/splash balance (2.31 fit + 1.69 splash) | Deck concentration 92.7% — committed players exploit adjacency |
| S2 Balanced Pack | Best early openness + splash (3.72 unique res, 1.92 off-arch) | Single-resonance majority limits dual-archetype support |
| S3 Lane Locking | Most transparent — players always know exact lock state | Convergence structurally capped below 2.0 (resonance ≠ archetype) |
| S4 Echo Window | Best pivot flexibility — instant direction changes in 3 picks | Early bias (2.58 arch-fit from pick 1) violates open-early goal |
| S5 Resonance Swap | Best signal reading (44.8%) and variety (6.5% overlap) | Convergence structurally unfixable — 360-card pool absorbs swaps |

## Proposed Improvements

**S1:** Adopt starting weight 3 (passes the early-fit target at 1.73 vs baseline 2.02) and keep wildcard slot. This fixes 2 of 3 failures with zero complexity cost.

**S2:** Introduce a threshold gate (majority must lead by 3+ weighted symbols) to delay activation and prevent accidental majority from a single pick. This was already tested at threshold=3 with good results (convergence at 6.7, 3.96 early unique resonances).

**S3:** Make locks temporary (decay after 5 picks without reinforcement) to address the permanent-lock flexibility problem. The convergence gap (1.83 vs 2.0) requires a different approach — perhaps locking to the player's top *two* resonances for a slot rather than one.

**S4:** Adopt the 2/1/0+1 slot allocation (fixes splash to 0.56, passes target). Start the window empty for the first 3 picks to address early bias. These are parameter changes, not mechanism changes.

**S5:** Drop the swap mechanism and keep only the asymmetric starting pool for signal reading. Pair with a primary convergence mechanism (Balanced Pack or Weighted Lottery) that handles Goals 2-7.

## Discussion Insights

**Emerging consensus (3 of 5 agents):** Agents 2, 3, and 5 independently converged on the same hybrid: Balanced Pack + asymmetric starting pool. This suggests the combination is a natural Pareto optimum — orthogonal mechanisms that compose without interference. Pack structure handles convergence/splash/early-openness; pool asymmetry handles signal reading/variety.

**Deck concentration analysis:** All 5 strategies exceed the 60-80% S/A concentration target (83.8-95.0%). Agent 5 identified the root cause: the fitness model assigns S/A-tier to 3/8 archetypes (37.5% of cards). A committed player selecting from 4-card packs will almost always find an S/A card. The target may need adjustment to 75-90%, though 92%+ still indicates overly smooth deckbuilding for a roguelike.

**One-sentence transparency ranking (confirmed by Agents 2, 3, 5):** Balanced Pack > Lane Locking > Echo Window > Weighted Lottery > Resonance Swap. Key insight from Agent 3: strategies 1-4 describe pack CONSTRUCTION (visible output), while strategy 5 describes pool MANIPULATION (hidden infrastructure). This is a fundamentally different kind of complexity.

## Proposed Best Algorithm

**Balanced Pack with Majority Threshold + Asymmetric Pool** (hybrid of S2 + S5):

> "Each pack shows one card per resonance type; once your most-drafted resonance leads the next-highest by 3+ weighted symbols (primary=2, others=1), it replaces one non-majority slot, giving you 2 of that resonance and 1 each of two others. Each run starts with one random resonance having extra cards in the pool."

The core mechanism is the Balanced Pack with threshold=3 gate, which simulation confirmed as the sweet spot: perfect early diversity (1/1/1/1 until majority established), clean convergence (2.07 fitting/pack at pick 6.7), and guaranteed splash (2 of 4 slots always non-majority). The asymmetric starting pool adds signal reading (~35% detection rate estimated from Agent 5's data) without changing pack construction.

This hybrid inherits the Balanced Pack's 7/8 target passes while adding meaningful signal reading (Goal 8). The only remaining failure — deck concentration — is a fitness model artifact shared by all strategies and should be addressed through card design, not the draft algorithm.
