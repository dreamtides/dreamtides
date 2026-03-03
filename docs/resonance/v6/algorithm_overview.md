# V6 Algorithm Overview: Complete Catalog

This document catalogs every algorithm proposed, simulated, and evaluated across
all 7 agents in the V6 resonance draft investigation. Each entry includes the
one-sentence description, simulation results where available, and the reason it
was championed or rejected.

## Baselines (from V3/V4)

### Lane Locking (Agent 1 baseline)
**One-sentence:** "Your pack has 4 slots; when your weighted symbol count in a
resonance first reaches 3, one open slot locks to that resonance and always
shows a card with that primary resonance; a second slot locks at 8."

**Unified simulation:** 2.22 S/A, convergence pick 3.3, stddev 0.50,
concentration 96.1%. Passes 6/9 metrics (fails M5 convergence timing, M6
concentration, M9 variance).

**Status:** Reference baseline. The algorithm that all V6 proposals must beat.
Strong convergence but too fast (pick 3.3), too deterministic (0.50 stddev), and
too concentrating (96%). Establishes the structural finding that locked
resonance slots deliver approximately 75-100% S/A because adjacent archetypes
sharing a primary resonance are mutually S/A.

### Auto-Spend Pack Widening (Agent 1 baseline)
**One-sentence:** "Each symbol you draft adds tokens (+2 primary, +1 each
secondary/tertiary); when any resonance reaches 3 tokens, 3 are auto-spent and 1
bonus card of that primary resonance is added to the pack."

**Unified simulation:** 1.76 S/A, convergence pick 8.2, stddev 0.91,
concentration 91.3%. Passes 6/9 metrics (fails M3 convergence, M5 timing, M6
concentration).

**Status:** Decision-free version of V4's Pack Widening. Demonstrates that
removing the spend/save decision costs approximately 1.6 S/A compared to V4's
3.35. Too weak as a standalone zero-decision algorithm.

## Agent 1: Lane Locking Domain

Agent 1 served as the baseline reference. No alternative proposals were made;
Lane Locking was implemented exactly as specified for apples-to-apples
comparison.

## Agent 2: Auto-Widening Domain

### Proposal 1: Threshold Auto-Spend Highest (CHAMPION)
**One-sentence:** "Each drafted symbol earns matching tokens (+2 primary, +1
others); when any counter reaches 4, auto-spend 4 tokens from the highest
counter and add 2 bonus resonance-matched cards to the pack."

**Unified simulation (C4/B2):** 1.50 S/A, convergence pick 10.4, stddev 0.97.
Passes 6/9 metrics.

**Status:** Championed by Agent 2 as the "most balanced." Individual agent
simulation showed 2.01 S/A (passing all 9), but the unified simulation with
shared card pool revealed this was fragile -- the algorithm fails M3 convergence
under standardized conditions. The token dilution problem: earning tokens across
4 resonances means the highest counter accumulates slowly, especially for
power-chaser and signal-reader strategies.

### Proposal 2: Round-Robin Auto-Spend
**One-sentence:** "Spend tokens on each resonance in rotating order, adding 1
bonus card of each resonance in sequence."

**Status:** Rejected in Round 1. Spreading bonuses across resonances defeats
convergence. No simulation.

### Proposal 3: Overflow Auto-Spend
**One-sentence:** "When any resonance exceeds a threshold, overflow tokens
convert to bonus cards of the second-highest resonance."

**Status:** Rejected in Round 1. Complexity without convergence benefit.

### Proposal 4: Momentum Auto-Spend
**One-sentence:** "Consecutive same-resonance picks earn escalating bonus cards
(1 for 2 in a row, 2 for 3 in a row)."

**Status:** Rejected in Round 1 but recognized in discussion as the strongest
dismissed idea. The streak mechanic directly addresses the "barely crossing 2.0"
problem. Not simulated as a standalone.

### Proposal 5: Drip Auto-Spend
**One-sentence:** "Fractional tokens (0.5 per symbol) accumulate slowly;
auto-spend at 2 for 1 weak bonus card."

**Status:** Rejected in Round 1. Too slow, insufficient convergence.

## Agent 3: Soft Slot Targeting Domain

### Proposal 1: Sigmoid Saturation
**One-sentence:** "Each slot's probability of showing a resonance-matched card
follows a sigmoid curve based on your weighted symbol count, approaching 90%
asymptotically."

**Status:** Rejected. Purely probabilistic; inherits the 50% dilution ceiling.
Estimated S/A approximately 1.6-1.8.

### Proposal 2: Dual-Resonance Cascade
**One-sentence:** "Track top two resonances; lock slots to each independently
when thresholds are crossed."

**Status:** Rejected. Multi-resonance locks split convergence and produce
scattered decks.

### Proposal 3: Aggressive Multi-Slot Replacement
**One-sentence:** "At threshold 5, replace 3 of 4 slots with resonance-matched
cards at 100% probability."

**Status:** Rejected. Too aggressive; 75% deterministic pack is worse than Lane
Locking.

### Proposal 4: Split-Resonance Slot Pairs
**One-sentence:** "Lock slot pairs: first pair to top resonance, second pair to
secondary resonance."

**Status:** Not championed but recognized as the strongest single insight:
targeting BOTH top resonances narrows archetype ambiguity from 4 to 1-2. The
split-resonance concept was adopted by Agents 3, 6 into their final champions.

### Proposal 5: Threshold-Triggered Soft Locks (CHAMPION)
**One-sentence:** "When your top resonance crosses 3, 6, and 9, lock one more
slot to show a resonance-matched card 75% of the time (first two target top
resonance, third targets second-highest)."

**Unified simulation (3/6/9, 75%):** 1.75 S/A, convergence pick 5.5, stddev
0.81, concentration 90.8%. Passes 6/9 metrics (fails M3, M6, M8).

**Status:** Championed by Agent 3. Individual agent simulation showed 2.20 S/A,
but the 75% probability substantially weakens convergence under unified
conditions. The core tension: softening locks improves variance but trades too
much S/A. Agent 3 acknowledged in discussion that binary locks at 75% S/A
precision would have been stronger.

## Agent 4: Pool Evolution Domain

### Proposal 1: Flood and Drain
**One-sentence:** "Add 20 resonance-matched cards to the pool per pick, then
remove 10 random off-resonance cards."

**Status:** Rejected. Pool size grows unboundedly; implementation complexity.

### Proposal 2: Shrinking Focused Pool
**One-sentence:** "Remove off-resonance cards from the pool each pick until only
target-resonance cards remain."

**Status:** Rejected. Converges too fast; deck concentration exceeds 99%.

### Proposal 3: Resonance Cascade
**One-sentence:** "Each drafted resonance symbol increases the draw weight of
matching cards by 5%, compounding."

**Status:** Rejected. Purely probabilistic; hits the 50% dilution ceiling.

### Proposal 4: Pool Replacement
**One-sentence:** "After each pick, replace 12 off-resonance cards in the pool
with resonance-matched cards from a reserve."

**Status:** Tested as a parameter variant; less effective than the T1-heavy
split.

### Proposal 5: Dual-Resonance Pool Sculpting (CHAMPION)
**One-sentence:** "After each pick (from pick 3+), replace 18 off-resonance
cards in the pool with cards matching your top two resonances (67% top, 33%
secondary) drawn from an enriched reserve."

**Unified simulation:** 1.99 S/A, convergence pick 10.4, stddev 1.09,
concentration 86.0%. Passes 6/9 metrics (fails M3, M4, M5).

**Status:** Championed by Agent 4 as an honest ceiling test. Confirms V4's
structural finding: probabilistic pool manipulation caps at approximately 2.0
S/A. Agent 4 acknowledged this is the "honest failure" of V6 -- pool evolution
should be a supplementary layer, not a standalone algorithm. Best card overlap
(9.3%) and best deck variety of any algorithm.

## Agent 5: Conditional Pack Enhancement Domain

### Proposal 1: Resonance Cluster Add
**One-sentence:** "Draw 4 random cards; if 2+ share a primary resonance with
your top resonance, add 1 random card of that resonance."

**Status:** Rejected. Single bonus card with 50% archetype dilution adds only
approximately 0.23 S/A. Total approximately 1.73, below 2.0.

### Proposal 2: Dual-Type Precision Bonus
**One-sentence:** "Draw 4 random; if 2+ match your top resonance, add 1 card
from the dual-type pool matching your top two resonances."

**Status:** Rejected. Dual-type pool too small (6-7 per archetype); total
approximately 1.98, borderline.

### Proposal 3: Double Enhancement (FINAL CHAMPION)
**One-sentence:** "Draw 4 random cards; if 2 or more share a primary resonance
with your top resonance, add 2 cards of that resonance to the pack."

**Unified simulation (T=2/B=2):** 1.32 S/A, convergence pick 13.4, stddev 1.57.
Passes 7/9 (fails M3, M5). Champion configuration fails because at 22% base rate
for any resonance, requiring 2-of-4 matches fires only approximately 21% of the
time.

**Unified simulation (T=1/B=2):** 2.13 S/A, convergence pick 7.4, stddev 1.71.
**Passes 9/9 metrics.** At threshold 1 (any match fires the trigger), the fire
rate is approximately 63%, delivering sufficient volume to cross 2.0.

**Status:** Switched champion from Cascading Enhancement to Double Enhancement
in discussion round. The T=1/B=2 variant is one of only two algorithms passing
all 9 metrics. The conditional trigger based on pack composition (rather than
accumulated state) creates organic variance: lucky packs get luckier, unlucky
packs stay random. Weakness: 63% fire rate means the "conditional" aspect is
marginal; this is closer to "add 2 bonus cards most of the time."

### Proposal 4: Replace-Worst Enhancement
**One-sentence:** "Draw 4 random; if 2+ match your top resonance, replace the
worst-fitting card with a resonance-matched card."

**Status:** Rejected. Replacement adds approximately 0.25 S/A, insufficient.
Also removes splash candidates.

### Proposal 5: Cascading Resonance Enhancement (ORIGINAL CHAMPION, DROPPED)
**One-sentence:** "Draw 4 random; for each card whose primary resonance matches
your top, roll 40% to add 1 bonus card from your top two resonances' dual-type
pool."

**Status:** Original champion, dropped in discussion. Fails the one-sentence
simplicity test: three independent mechanisms (per-card scanning, probability
roll, dual-type pool) plus hidden rules (activation threshold, bonus cap). Agent
5 self-critiqued and switched to Double Enhancement.

## Agent 6: Escalating Influence Domain

### Proposal 1: Graduated Slot Locking
**One-sentence:** "Each slot independently shows a resonance-matched card with
probability equal to your top resonance count divided by 16, capped at 75%."

**Status:** Rejected. Purely probabilistic; estimated approximately 1.6 S/A at
best.

### Proposal 2: Escalating Additive Injection
**One-sentence:** "Earn resonance tokens; auto-spend 4 of highest to inject 1
bonus card per spend, repeating until under threshold."

**Status:** Rejected. Single bonus card at 50% precision adds approximately 0.38
S/A, total approximately 1.73.

### Proposal 3: Threshold-Gated Escalation
**One-sentence:** "Slots escalate probabilistically, plus one hard lock at
threshold 6."

**Status:** Rejected. Hybrid complexity without sufficient convergence.
Estimated approximately 1.75 S/A.

### Proposal 4: Escalating Dual-Signal Injection
**One-sentence:** "Auto-spend 3 tokens to inject 1 card, preferring dual-type
cards matching your top two resonances."

**Status:** Rejected. Dual-type pool too small for reliable precision; total
approximately 1.97.

### Proposal 5: Ratcheting Slot Commitment (CHAMPION)
**One-sentence:** "When your top resonance count reaches 3, 6, and 10, lock one
more pack slot: first two to top resonance, third to second-highest; fourth slot
stays random."

**Unified simulation (3/6/10):** 2.08 S/A, convergence pick 2.8, stddev 0.60,
concentration 94.9%. Passes 6/9 (fails M5, M6, M9).

**Unified simulation (2/4/7):** 2.11 S/A, convergence pick 2.4, stddev 0.58,
concentration 95.8%. Passes 6/9 (same failures).

**Status:** Championed by Agent 6. Achieves strong convergence (2.08-2.11 S/A)
but inherits all of Lane Locking's structural weaknesses: permanent locks cause
too-fast convergence, excessive concentration, and insufficient variance. The
split-resonance third lock (targeting secondary) is the unique contribution --
it provides archetype disambiguation. But 75% of the pack being deterministic by
mid-draft creates a mechanical experience. Both threshold variants converge too
fast (pick 2.4-2.8, far below the 5-8 target).

## Agent 7: Open Exploration Domain

### Proposal 1: Momentum Injection
**One-sentence:** "If your current pick shares a primary resonance with your
previous pick, one slot in the next pack shows a resonance-matched card."

**Status:** Rejected. Binary on/off provides only 1 guaranteed slot at
approximately 0.5 S/A; total approximately 1.25.

### Proposal 2: Echo Accumulator
**One-sentence:** "Earn resonance points; at 5, reset to 0 and inject one
resonance-matched slot into the next pack."

**Status:** Rejected. Single injection every 2 picks yields approximately 1.25
S/A.

### Proposal 3: Resonance Gravity
**One-sentence:** "Each slot draws from a weighted pool: 3x for top resonance,
2x for second, with anti-monotony rerolls."

**Status:** Rejected. Probabilistic with weight boosting; estimated
approximately 1.6-1.8. The anti-monotony reroll further reduces convergence.

### Proposal 4: Surge Packs (CHAMPION)
**One-sentence:** "Each drafted symbol adds tokens (+2 primary, +1 others); when
any counter reaches 4, spend 4 and fill 3 of the next pack's 4 slots with random
cards of that resonance, fourth slot random."

**Unified simulation (T=4/S=3):** 2.05 S/A, convergence pick 5.9, stddev 1.42,
concentration 76.5%. **Passes 9/9 metrics.**

**Status:** The recommended algorithm. Provides the best combination of
convergence (2.05 S/A), timing (pick 5.9), variance (1.42 stddev), and player
experience (rhythmic surge/normal cycle). Non-permanent state tracking allows
genuine pivoting. Constant pack size of 4 (unlike additive approaches that
create variable-size packs).

### Proposal 5: Draft Imprint
**One-sentence:** "After pick 5, lock 1 slot to your top resonance; after pick
10, lock a second slot to your top pair, both tracking the current leader
(non-permanent)."

**Status:** Rejected. Non-permanent tracking is the key innovation (later
adopted by Surge Packs). Two slots insufficient for 2.0 S/A; estimated
approximately 1.65.

## Mechanism Class Summary

| Class | Algorithms | Best S/A | Ceiling | Why |
|---|---|:-:|---|---|
| Deterministic Placement | Lane Locking, Ratcheting | 2.22 | Unlimited (add more locks) | Guaranteed precision per slot |
| Additive Injection | Auto-Spend, TAS, Double Enhance | 2.13 | ~2.5 (with frequent triggers) | Extra cards bypass pack limit |
| Slot-Filling Surge | Surge Packs | 2.05 | ~2.5 (with more surge slots) | Replaces random slots with targeted |
| Probabilistic Soft Lock | Soft Locks | 1.75 | ~2.2 (at 100% = hard lock) | Probability reduces expected S/A |
| Pool Manipulation | Pool Sculpting | 1.99 | ~2.0 (confirmed ceiling) | Random draws from modified pool |
| Conditional Enhancement | Double Enhance (T=2) | 1.32 | ~2.1 (at T=1) | Fire rate limits expected value |

The investigation confirms V4's hierarchy: deterministic placement delivers the
highest raw S/A, followed by additive injection, then slot-filling surges.
However, when all 9 design goals are weighted equally, Surge Packs achieves the
best overall score because deterministic approaches fail variance and
concentration targets.

## Cross-Agent Consensus Points

1. **All 7 agents agree** that Pool Sculpting cannot standalone and should be a
   supplementary layer.
2. **All 7 agents agree** that the zero-decision constraint costs approximately
   1.0-1.3 S/A vs. Pack Widening.
3. **6 of 7 agents agree** that Lane Locking converges too fast and concentrates
   too heavily.
4. **5 of 7 agents** ranked either Surge Packs or Threshold Auto-Spend as the
   top algorithm.
5. **All 7 agents agree** that the 15% dual-resonance constraint had minimal
   impact on algorithm performance. The constraint's effect was psychological
   (forced designers to build pure single-resonance systems) rather than
   mechanical.

## Final Recommendation

**Surge Packs (T=4/S=3)** is the recommended algorithm for the Dreamtides
resonance draft system. It is the only algorithm that simultaneously achieves
convergence, variance, healthy concentration, correct timing, and zero player
decisions. Its rhythmic surge/normal alternation creates a distinctive player
experience that permanent-lock and steady-state algorithms cannot match.
