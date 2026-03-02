# Comparison 4: Echo Window Agent (Reactive Domain)

## Scorecard

| Goal | Weighted Lottery (1) | Balanced Pack (2) | Lane Locking (3) | Echo Window (4) | Resonance Swap (5) |
|------|---------------------|-------------------|-------------------|-----------------|---------------------|
| 1. Simple | 9 | 9 | 8 | 9 | 6 |
| 2. Not on rails | 7 | 7 | 4 | 8 | 8 |
| 3. No forced decks | 7 | 7 | 7 | 7 | 9 |
| 4. Flexible archetypes | 7 | 6 | 5 | 8 | 7 |
| 5. Convergent | 8 | 8 | 6 | 7 | 3 |
| 6. Splashable | 8 | 9 | 7 | 5 | 9 |
| 7. Open early | 6 | 9 | 8 | 5 | 8 |
| 8. Signal reading | 3 | 2 | 3 | 2 | 9 |
| **Total** | **55** | **57** | **48** | **51** | **59** |

Justifications (1-sentence each):

**Weighted Lottery:** Simple (genuine one-sentence algorithm), not-on-rails (wildcard prevents lock-in but late weights dominate), no-forced (7.5% overlap), flexible (dual weights bridge archetypes), convergent (2.31 late fit passes cleanly), splashable (1.69 off-arch via wildcard), early (2.02 early fit marginal fail), signal (no pool awareness).

**Balanced Pack:** Simple (two-state system a child could predict), not-on-rails (1/1/1/1 base guarantees diversity), no-forced (5.8% overlap), flexible (majority supports only one resonance), convergent (2.08 just passes), splashable (1.92 off-arch is the gold standard), early (0.00 early fit is perfect), signal (zero pool mechanism).

**Lane Locking:** Simple (clear thresholds but permanent lock adds conceptual weight), not-on-rails (permanent locks from pick 2 is the worst flexibility), no-forced (5.6% overlap), flexible (4-lock cap pressures single pairs), convergent (1.83 misses 2.0 — locked slot guarantees resonance not archetype fit), splashable (0.84 passes), early (3.32 unique resonances), signal (no pool awareness).

**Echo Window:** Simple (one sentence is the complete algorithm), not-on-rails (3-pick memory allows pivots anytime), no-forced (8% overlap, even distribution), flexible (second-resonance slot supports dual archetypes naturally), convergent (2.83 late fit is strong but 84% SA overshoots), splashable (0.43 misses 0.5 target), early (2.58 early fit well exceeds 2.0 cap — biases after 1 pick), signal (no pool mechanism).

**Resonance Swap:** Simple (two sentences + hidden reserve infrastructure), not-on-rails (gentle pool shifts keep all paths open), no-forced (6.5% overlap is lowest), flexible (pool retains all resonances), convergent (1.61 far below 2.0 — structurally unfixable with 360-card pool), splashable (1.17 off-arch far exceeds target), early (3.48 unique resonances), signal (44.8% detection rate is transformative).

## Biggest Strength & Weakness Per Strategy

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|------------------|
| Weighted Lottery | Balanced convergence (2.31) with strong splash (1.69) — best all-rounder | No signal reading; weights grow monotonically making late pivots progressively harder |
| Balanced Pack | Early openness (3.72 unique res, 0.00 early fit) — structurally perfect exploration phase | Majority bonus only supports one resonance; dual-resonance archetypes get half the benefit |
| Lane Locking | Transparency — player always knows exact pack structure from lock state | Permanent locks from pick 2 destroy flexibility; 1.83 late fit misses convergence target |
| Echo Window | Maximum pivotability — 3-pick memory means any player can change direction instantly | Over-biases immediately (2.58 early fit); splash at 0.43 narrowly misses target |
| Resonance Swap | Signal reading (44.8% detection) is unique and genuinely valuable for skilled players | Convergence structurally broken at 1.61; 360-card pool too large for meaningful shifts |

## Proposed Improvements

**Weighted Lottery:** Increase starting weight to 3 (fixes early fit from 2.02 → 1.73). Consider a decay mechanism where oldest symbols contribute less, enabling late pivots.

**Balanced Pack:** Add a secondary resonance consideration — if a player's top two resonances are from the same archetype pair, grant a quality bonus on the secondary slot. Alternatively, use windowed counting (last 5 picks) instead of full history for majority detection, enabling pivots.

**Lane Locking:** Replace permanent locks with decaying locks — threshold 3 locks a slot for 5 picks, threshold 8 locks permanently. This preserves the milestone feel while allowing early-game exploration without permanent commitment.

**Echo Window:** Switch to 2/1/0+1 slot allocation (top resonance 2, second resonance 1, one true random). Simulation data shows this improves splash from 0.43 to 0.56 while keeping late fit at 2.75. For early bias, expand the window to 5 picks for the first 8 picks, then shrink to 3.

**Resonance Swap:** Either operate on a smaller sub-pool (~40 cards) for meaningful convergence shifts, or accept the role as a complementary layer rather than a standalone algorithm.

## Proposed Best Algorithm

**Balanced Pack with Windowed Majority + Pool Seeding**

One sentence: "Each pack shows one card per resonance, but if your last 5 picks have a majority resonance (primary symbols count double), that resonance takes a second slot; each run starts with a randomly seeded pool favoring one resonance over another."

This hybrid takes:
- **Pack structure from Balanced Pack** — 1/1/1/1 base guarantees early diversity (3.72 unique resonances) and structural splash (1.92 off-arch). The majority bonus provides convergence (2.08 late fit).
- **Windowed counting from Echo Window** — Using last 5 picks instead of full history adds pivotability. A committed player always maintains majority; a pivoting player can shift within 5 picks. This addresses Balanced Pack's weakness of permanent majority lock-in from accumulated history.
- **Initial pool seeding from Resonance Swap** — Each run starts with +10 cards of one random resonance and -10 of another. No runtime pool manipulation (preserving simplicity), but players who notice the seeded resonance appearing more often can draft toward it. Estimated ~35% signal detection vs 44.8% for full Swap, but zero ongoing complexity.

The algorithm passes the Simplicity Test: a player can predict their next pack's structure from (a) their last 5 picks and (b) which resonance is their majority. The pool seeding is invisible but discoverable — the kind of metagame signal that rewards experienced players without burdening new ones.

Expected target performance: early diversity PASS (inherited from Balanced Pack), convergence PASS (2.0+ from majority bonus), splash PASS (structural guarantee), overlap PASS, signal reading PARTIAL (weaker than full Swap but present), concentration still likely over 80% (all strategies share this issue with simulated committed players).
