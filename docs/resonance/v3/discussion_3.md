# Discussion Output — Agent 3 (Threshold/Progression)

## Simplicity Ranking (Most to Least Simple)

1. **Lane Locking** — Binary slot state (locked or open), threshold check against a counter. A 12-year-old can say: "I have 1 Tide slot, 1 Zephyr slot, 2 random slots." Perfect prediction of pack structure. No sorting, no probability, no re-computation each pack.

2. **Weighted Lottery** — One sentence, genuine algorithm. But "probability proportional to weight" requires understanding weighted random selection. A 12-year-old can grasp the concept ("more tokens = more likely") but cannot predict their exact next pack — only tendencies. Prediction is fuzzy, not concrete.

3. **Echo Window** — Requires counting weighted symbols across 3 cards, sorting 4 resonances by total, and applying a 2/1/1 allocation. Doable but more mental bookkeeping per pack than Lane Locking. Tie-breaking (common with a 3-card window) is unspecified in the one-sentence description. Also: this is structurally identical to Domain 1's Running Tally Slots and Domain 2's Majority Rules — the same 2/1/1 formula with different memory lengths.

4. **Resonance Swap** — The one-sentence description hides the reserve concept. "Replace 2 non-matching with 2 matching reserve cards" — but what IS the reserve? How big? What happens when it runs out? These are not cosmetic details; they're load-bearing infrastructure invisible to the player. Pool state is also invisible — the player can't predict pack composition, only observe vague frequency trends.

5. **Rotating Wheel** — Two interacting systems: a rotation cycle (which resonance fills which slot) AND a majority calculation (which resonance gets duplication). The player must track both the wheel position and their symbol counts, then reason about how they interact. The duplication-into-opposite-slot rule adds a third concept. Too many moving parts.

## Scorecard Table

| Goal | Lane Locking | Weighted Lottery | Echo Window | Resonance Swap | Rotating Wheel |
|------|:-----------:|:---------------:|:----------:|:-------------:|:-------------:|
| 1. Simple | **9** | 8 | 7 | 6 | 5 |
| 2. Not on rails | 7 | 6 | **9** | 6 | 7 |
| 3. No forced decks | 6 | 7 | **8** | 7 | 7 |
| 4. Flexible archetypes | **8** | 6 | 8 | 5 | 6 |
| 5. Convergent | 7 | **8** | 7 | 7 | 4 |
| 6. Splashable | **8** | 7 | 7 | 5 | 7 |
| 7. Open early | **9** | 9 | 9 | 7 | 8 |
| 8. Signal reading | 4 | 3 | 3 | **8** | 7 |
| **Total** | **58** | **54** | **58** | **51** | **51** |

**Key observations:**
- Lane Locking and Echo Window tie at 58, but Lane Locking wins the top-priority goal (Simplicity) by 2 points while Echo Window wins the lower-priority goals (Not on rails, No forced decks).
- Weighted Lottery's reliable convergence (8) is its standout, but it pays with weak signal reading (3) and moderate railroading risk (6) in the late game.
- Rotating Wheel's convergence score (4) is its fatal flaw — averaging only 1.0 committed cards per pack across rotation cycles, well below the 2+ target.
- Resonance Swap is the only algorithm with strong signal reading (8), but its hidden reserve and invisible pool state undercut simplicity and transparency.

## Final Championed Algorithm: Refined Lane Locking

**One-sentence description:** "Your pack has 4 slots; when your symbol count in a resonance first reaches 3, one open slot locks to that resonance; when it first reaches 8, a second slot locks."

**Changes from Round 1:**
- Two thresholds per resonance (3 and 8) instead of one (3). This addresses the convergence weakness (was 1.75 expected committed cards, now 2+ after second lock).
- Up to 2 locks per resonance. A committed player gets 2 guaranteed slots of their primary resonance by pick 6-7.
- Max 4 total locked slots (unchanged). Once full, no further structural changes.

**Why I'm not switching:** Lane Locking's core strength — discrete, visible, permanent structural milestones — is unique among the 5 champions. No other algorithm gives the player a concrete "level up" moment where their pack structure visibly changes. The refinement fixes convergence without sacrificing this identity.

## Modifications for Round 3 Simulation

1. **Dual thresholds (3 and 8)** as the primary variant. Also test single threshold at 3 (original) and single threshold at 5 (slower) for comparison.
2. **Threshold sensitivity sweep:** Test thresholds (2,6), (3,8), (4,10), (5,12) to find the convergence sweet spot.
3. **Lock cap variants:** Test max 4 total locks vs max 6 (allowing up to 2 per resonance with no total cap — all 4 slots could be double-locked if a player invests in all resonances).

## Proposed Symbol Distribution

| Symbols | Count | % of non-generic |
|---------|-------|-----------------|
| 0 (generic) | 36 | — |
| 1 | 81 | 25% |
| 2 | 178 | 55% |
| 3 | 65 | 20% |

With 2-symbol cards dominant, each pick contributes ~3 weighted symbols (2 primary + 1 secondary). Threshold 3 is reachable by pick 2 for a committed player (two [Primary] cards = 4 symbols). Threshold 8 is reachable by pick 4-5 with focused drafting. This creates the first "level up" early enough to feel impactful and the second around the convergence target window (picks 5-8).
