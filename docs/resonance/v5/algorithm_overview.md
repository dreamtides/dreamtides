# Resonance V5 -- Algorithm Overview

Comprehensive catalog of all algorithms considered during V5, organized by
domain. Each entry includes the one-sentence description, mechanism summary,
simulation status, and final recommendation.

---

## Domain 1: Passive Resonance Bonus (Auto-Widening)

**Core idea:** Take Pack Widening's bonus-card injection and make spending
automatic. The player's only action is picking a card; bonuses happen whenever
token thresholds are reached.

### 1.1 Simple Threshold Auto-Spend

**One-sentence:** "Each symbol you draft adds a matching token (primary=2);
when any resonance reaches 4, your next pack gets a bonus card of that
resonance and 4 tokens are deducted."

Accumulates resonance tokens and auto-spends on the highest resonance. Simple
threshold-reset cycle. Fires roughly every 1.3 picks for committed players.

Domain: Passive Bonus. Not championed. Single-resonance matching inherits V4's
~50% dilution ceiling, structurally capping S/A below 2.0.

### 1.2 Pair-Based Threshold Auto-Spend (CHAMPIONED, SIMULATED)

**One-sentence:** "Each 2+ symbol card adds 1 to its ordered pair count; at 3
your next pack gets a bonus card of that pair and the count resets."

Tracks ordered pairs instead of single resonances. Bonus cards drawn from the
pair-matched subset achieve ~80% S-tier precision. Threshold-reset creates
bursty delivery with natural variance.

Domain: Passive Bonus. **Championed and simulated as D1.**

| Target | Result | P/F |
|--------|--------|-----|
| Late S/A >=2 | 1.54 | FAIL |
| Conv 5-8 | 14.7 | FAIL |
| StdDev >=0.8 | 1.02 | PASS |

**Verdict:** Structural ceiling at ~1.5 S/A. Bonus injection cannot match
slot replacement for convergence. Best role: variance layer atop D2/D4.

### 1.3 Fractional Probability Bonus

**One-sentence:** "Each symbol adds to your resonance total; each pack has a
chance equal to your highest resonance divided by 12 of including a bonus card."

Monotonically increasing probability, never resets. Good early variance but
late-game packs are nearly always 5 cards, eliminating variance.

Domain: Passive Bonus. Not championed. Non-resetting counters produce monotonic
convergence that eliminates late-game variance.

### 1.4 Dual-Track Auto-Spend

**One-sentence:** "When your two highest resonances each have 3+ tokens, your
next pack gets two bonus cards -- one from each -- and 3 tokens are deducted
from each."

Targets both primary and secondary resonances simultaneously, producing highly
archetype-targeted dual bonuses. Pack sizes of 4/5/6.

Domain: Passive Bonus. Not championed. Too complex for one-sentence test --
dual-trigger logic with variable pack sizes is confusing.

### 1.5 Cooldown Auto-Spend

**One-sentence:** "After every 3 picks, your next pack gets a bonus card from
your highest resonance and that resonance loses 3 tokens."

Fixed temporal cadence: bonus every 3rd pack. Simplest of all proposals but
only 33% of packs get bonuses, capping late S/A at ~1.5-1.8.

Domain: Passive Bonus. Not championed. Fixed cadence feels mechanical and
convergence is insufficient.

---

## Domain 2: Probabilistic Slot Targeting (Soft Lane Locking)

**Core idea:** Replace Lane Locking's binary slot locks with per-slot
probabilities that scale with the player's commitment. No permanent state
transitions.

### 2.1 Top-Resonance Probability Slots

**One-sentence:** "Each pack slot independently shows a card matching your top
resonance with probability min(count/15, 0.75), otherwise random."

Single-resonance probability per slot. Simple but inherits V4's dilution
ceiling (~50% archetype precision). Projected ~1.5 S/A.

Domain: Soft Locking. Not championed. Single-resonance matching caps below 2.0.

### 2.2 Pair-Escalation Slots (CHAMPIONED, SIMULATED, RECOMMENDED)

**One-sentence:** "Track the ordered pair of each 2+ symbol card you draft;
each slot shows a card matching your top pair with probability min(count/6,
cap), otherwise random."

The central V5 innovation: pair-based probabilistic slot targeting. Each slot
independently rolls against a probability driven by the player's top pair
count. Pair precision (~80% S-tier) breaks through the dilution ceiling.

Domain: Soft Locking. **Championed, simulated as D2. V5 RECOMMENDED
ALGORITHM.**

| Target | cap=0.65 | cap=0.50 | P/F (.50) |
|--------|:-:|:-:|-----|
| Late S/A >=2 | 2.97 | 2.61 | PASS |
| Conv 5-8 | 5.9 | 6.3 | PASS |
| StdDev >=0.8 | 0.92 | 0.98 | PASS |
| Deck conc 60-90% | 96.9% | 96.2% | FAIL |

Cap=0.50 recommended: better splash (0.70 vs 0.52 off-arch), better variance,
still well above 2.0 S/A.

### 2.3 Dual-Track Probability (Primary + Pair)

**One-sentence:** "Each slot rolls twice: first against your top pair count/6
for a pair-matched card, then against your top resonance count/20 for a
resonance-matched card, otherwise random."

Two probability layers per slot. Better early convergence (single resonance
kicks in first) but too complex for simplicity test.

Domain: Soft Locking. Not championed. Two interacting probability layers with
different caps fail the one-sentence test.

### 2.4 Threshold-Gated Soft Slots

**One-sentence:** "When your weighted symbol count reaches 3, each slot gains
50% chance of showing a resonance-matched card; at 8, the chance rises to 75%."

Lane Locking but probabilistic instead of deterministic. Mirrors Lane Locking's
sentence structure. Single-resonance matching caps projected S/A at ~1.5.

Domain: Soft Locking. Not championed. Inherits single-resonance dilution.

### 2.5 Proportional Pair Slots with Spillover

**One-sentence:** "Each slot is assigned to your top pair with probability
equal to that pair's share of your total drafted pairs."

Self-normalizing probability based on pair proportion. Elegant but sluggish
convergence -- the denominator grows with every 2+ symbol card, requiring
extreme commitment to reach high probabilities.

Domain: Soft Locking. Not championed. Self-normalizing denominator makes
convergence too slow for moderately committed players.

---

## Domain 3: Pool Evolution (Seeding)

**Core idea:** Instead of manipulating packs, change the pool itself. Add
archetype-matched cards to the pool after each pick, increasing natural density.

### 3.1 Primary Resonance Flood

**One-sentence:** "When you draft a card, 3 cards matching its primary
resonance are added to the pool from a reserve."

Single-resonance injection. Simple and invisible. Pool grows by 3 per pick.
Single-resonance matching means ~50% of injected cards are wrong-archetype.

Domain: Pool Evolution. Not championed. V4 dilution ceiling applies.

### 3.2 Pair-Matched Injection

**One-sentence:** "When you draft a card with 2+ symbols, 3 cards matching its
ordered pair are added to the pool from a reserve."

Pair-matched injection with higher precision. 1-symbol picks inject nothing,
creating dead turns.

Domain: Pool Evolution. Not championed. Dead turns on 1-symbol picks slow
convergence.

### 3.3 Proportional Pair Seeding

**One-sentence:** "After each pick, 4 cards are added to the pool from a
reserve, distributed across your drafted pairs proportionally."

Continuous injection spread proportionally across all drafted pairs. Most
balanced pool evolution approach. Self-adjusting allocation.

Domain: Pool Evolution. Not championed in final form. Champion selection went
to Top-Pair Pool Seeding (3.4 variant) for simplicity.

### 3.4 Top-Pair Pool Seeding (CHAMPIONED, SIMULATED)

**One-sentence:** "After each pick, if you have 2+ cards of the same ordered
pair, 4 cards matching your top pair are added to the pool from a reserve."

Simplified version of proportional seeding -- all injection goes to the top
pair once threshold met. Pool grows from 360 to ~420+ over 30 picks.

Domain: Pool Evolution. **Championed and simulated as D3.**

| Target | Result | P/F |
|--------|--------|-----|
| Late S/A >=2 | 1.50 | FAIL |
| Conv 5-8 | 16.3 | FAIL |
| StdDev >=0.8 | 0.97 | PASS |

**Verdict:** Pool bloat creates structural ceiling at ~1.5 S/A. Adding cards
increases both numerator and denominator. Value is as invisible complement to
D2/D4 for signal reading.

### 3.5 Flood and Drain

**One-sentence:** "When you draft a card, 3 cards matching its primary
resonance join the pool and 2 cards of unrelated resonances leave."

Addition plus removal to sharpen density shift. Net +1 per pick. Removal
narrows early variety, conflicting with open-early goal.

Domain: Pool Evolution. Not championed. Premature pool narrowing hurts
flexibility.

### 3.6 Escalating Pair Injection

**One-sentence:** "After each pick, cards equal to your top pair's count
(capped at 5) matching that pair are added to the pool."

Accelerating injection: slow early, heavy late. Best temporal profile but
extreme pool bloat late (500+ cards) and harder to explain.

Domain: Pool Evolution. Not championed. Late-game pool bloat and complex
escalation rule.

---

## Domain 4: Pair-Based Pack Construction

**Core idea:** Use ordered pairs as the primary matching unit for deterministic
pack slot construction. Exploit the ~80% S-tier precision of pair matching.

### 4.1 Pair-Weighted Sampling

**One-sentence:** "Each card is drawn with weight 4 if its ordered pair matches
your top pair, weight 1 otherwise."

Pure probabilistic weighting by pair match. Even with pair precision, weighting
alone is too weak -- projected ~1.6 S/A.

Domain: Pair-Based. Not championed. Weighting-only mechanisms cap below 2.0
even with pair precision.

### 4.2 Pair Slot Guarantee

**One-sentence:** "Once you have 3+ cards of the same ordered pair, one slot
always shows a card with that pair, rest random."

Single guaranteed pair-matched slot. Projected ~1.75-2.1 S/A. On the boundary
of crossing 2.0. Evolved into D4 Dual-Threshold with a second slot.

Domain: Pair-Based. Evolved into champion (4.3). Single slot may not cross 2.0
reliably.

### 4.3 Dual-Threshold Pair Guarantee (CHAMPIONED, SIMULATED)

**One-sentence:** "Track each 2+ symbol card's ordered pair; at 3 matching
picks one slot is pair-matched, at 7 a second, rest random."

Two guaranteed pair-matched slots at separate thresholds. Simplest algorithm
crossing 2.0. Binary thresholds are maximally transparent.

Domain: Pair-Based. **Championed and simulated as D4.**

| Target | (3,7) | (2,5) | P/F (2,5) |
|--------|:-:|:-:|-----|
| Late S/A >=2 | 2.32 | 2.52 | PASS |
| Conv 5-8 | 9.6 | 6.9 | PASS |
| StdDev >=0.8 | 0.90 | 0.78 | FAIL |
| Deck conc 60-90% | 93.4% | 95.9% | FAIL |

**Verdict:** Simplest algorithm crossing 2.0. D4(2,5) converges fastest but
narrowly fails variance. D4(3,7) passes variance but fails convergence speed.
Alternative recommendation if simplicity is paramount.

### 4.4 Pair Echo Replacement

**One-sentence:** "After you pick a 2+ symbol card, one random card in the
next pack is replaced by a card whose ordered pair matches the card you just
picked."

Reactive: only the last pick matters, no accumulation. Instant pivots. Only
~0.75 pair-matched slots per pack on average, capping at ~1.5 S/A.

Domain: Pair-Based. Not championed. No accumulation means no commitment reward.

### 4.5 Pair Bonus Injection

**One-sentence:** "Each 2+ symbol card adds 1 to its pair counter; at 3 a
bonus card of that pair joins your next pack and the counter resets."

Pair-based analog of Pack Widening with automatic threshold. Very similar to
D1 (1.2). Projected ~1.8 S/A -- close but likely short of 2.0.

Domain: Pair-Based. Not championed. Functionally identical to D1's champion;
bonus injection structurally capped.

### 4.6 Pair Cascade

**One-sentence:** "At pair counts 2/5/9, one/two/three slots are permanently
pair-matched."

Three-tier cascade to 3 guaranteed slots. Strongest convergence (~3.0 S/A) but
essentially "Lane Locking with pairs" -- same mechanical feel and on-rails
problem.

Domain: Pair-Based. Not championed. Too mechanical; only 1 random slot at
threshold 3.

---

## Domain 5: Conditional Pack Enhancement

**Core idea:** Add bonus cards only when the random base pack naturally clusters
with the player's profile. Pack composition triggers enhancement.

### 5.1 Resonance Cluster Bonus

**One-sentence:** "Draw 4 random; if 2+ share a primary resonance with your
top resonance, add 1 bonus card of that resonance."

Single-resonance cluster trigger. ~40-50% fire rate but single-resonance
matching limits bonus precision to ~50%.

Domain: Conditional. Not championed. V4 dilution ceiling applies to both
trigger and bonus.

### 5.2 Pair Cluster Bonus

**One-sentence:** "Draw 4 random; if 2+ share their ordered pair with your top
pair, add 1 bonus card matching that pair."

Pair-based cluster trigger with pair-matched bonus. Higher precision but lower
fire rate (~15-25% for 2-of-4 pair match). Projected ~1.6-1.8 S/A.

Domain: Conditional. Initially championed but evolved into Hybrid Trigger (5.6)
after analysis showed 2-of-4 pair cluster fires too rarely.

### 5.3 Resonance Echo Pack

**One-sentence:** "Draw 4 random; for each card sharing primary resonance with
your last pick, replace it with a card matching your top pair."

Replacement-based (pack stays 4 cards). Two interacting conditions make this
hard to explain. Variable 0-3 replacements per pack.

Domain: Conditional. Not championed. Too complex; replacement caps pack size.

### 5.4 Density-Scaled Bonus

**One-sentence:** "Draw 4 random; if 3+ overlap with your top 2 resonances,
add 1 bonus; if 2 overlap, 50% chance of bonus."

Graduated trigger with two probability tiers. More complex than binary
triggers. Single-resonance overlap check inherits dilution.

Domain: Conditional. Not championed. Graduated probability is hard to explain
and single-resonance dilution applies.

### 5.5 Cascade Bonus

**One-sentence:** "Draw 4 random; if any matches your last pick's pair, add 1
bonus of that pair; if the bonus also matches, add another."

Recursive cascade producing guaranteed +2 when triggered. Binary 4-or-6 packs.
Too swingy and complex.

Domain: Conditional. Not championed. Recursive cascade is confusing and
variance is too extreme.

### 5.6 Hybrid Resonance-Triggered Pair Bonus (CHAMPIONED, SIMULATED)

**One-sentence:** "Draw 4 random; if any card's primary resonance matches your
top resonance, add 1 bonus card matching your top pair."

Uses single-resonance trigger (higher fire rate ~64%) with pair-matched bonus
(higher precision ~80%). Best of both worlds for conditional enhancement.

Domain: Conditional. **Championed and simulated as D5.**

| Target | Result | P/F |
|--------|--------|-----|
| Late S/A >=2 | 1.96 | FAIL |
| Conv 5-8 | 9.6 | FAIL |
| StdDev >=0.8 | 1.25 | PASS |

**Verdict:** Best organic variance (1.25 stddev) but fails 2.0 S/A standalone.
Value is as variance layer. D4+D5 hybrid scored 2.10 in Round 3 simulations.

---

## Hybrid Proposals (from Rounds 2 and 4)

### H1: D4+D5 Hybrid (Guaranteed Slots + Conditional Bonus)

**One-sentence:** "At 3 matching pairs one slot is pair-matched, at 6 a second;
if any random card shares your top resonance, add 1 bonus pair-matched card."

Combines D4's convergence floor with D5's variance. Round 3 tested at 2.10 S/A,
1.21 stddev. Competitive but two-mechanism complexity.

### H2: D2 + D3 (Pair Slots + Pool Seeding)

**One-sentence:** "Each slot has up to 50% pair-matching probability; after
each pick, 4 pair-matched cards are added to the pool."

Adds signal reading to D2. Two independent mechanisms. Not simulated as
unified system.

### H3: Modified D2 with Discrete Steps

**One-sentence:** "At pair counts 2/4/6+, each slot has 25%/50%/65% chance of
being pair-targeted."

Replaces continuous formula with discrete thresholds for transparency.
Functionally equivalent to D2 but easier to communicate. Proposed by Agent 1
in Round 4.

---

## Final Rankings

| Rank | Algorithm | Status |
|------|-----------|--------|
| 1 | D2 Pair-Escalation Slots (cap=0.50) | **V5 RECOMMENDED** |
| 2 | D4 Dual-Threshold (2,5) | Alternative if simplicity paramount |
| 3 | D2 Pair-Escalation Slots (cap=0.65) | Higher convergence, less splash |
| 4 | D4 Dual-Threshold (3,7) | Better splash, slower convergence |
| 5 | D5 Hybrid Trigger | Best variance, needs D4 to reach 2.0 |
| 6 | D1 Pair Threshold Auto-Spend | Structurally capped, variance layer only |
| 7 | D3 Top-Pair Pool Seeding | Signal reading layer only |
| -- | H1 D4+D5 Hybrid | Viable if two-mechanism cost accepted |
| -- | H3 Discrete Steps D2 | Communication improvement, functionally same |

**Cross-generation recommendation:**
V5 D2 Pair-Escalation Slots supersedes both V3 Lane Locking and V4 Pack
Widening. It achieves the highest convergence (2.61 S/A) with zero player
decisions, natural variance, and no permanent commitment.
