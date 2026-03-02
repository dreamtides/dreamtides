# Resonance Draft V3 -- Algorithm Overview (Corrected)

A catalog of all algorithms explored during V3. Five domains were investigated,
each producing five proposals (25 total), plus hybrid proposals from later
rounds. All simulation results use **corrected archetype-level metrics** (S/A
fitness for the specific target archetype, not just resonance matching).

---

## Domain 1: Accumulation-Based Mechanisms

Drafting builds persistent state over time. Future packs shaped by full history.

### 1A. Token Bag

**Description:** "Each symbol you draft adds a token to a bag; pack slots draw
tokens to determine resonance."

**How it works:** Empty bag grows with each pick. Primary symbol adds 2 tokens,
secondary/tertiary add 1 each. Pack slots draw tokens (with replacement) and
show a random card of the drawn resonance.

**Domain:** Accumulation. **Championed:** No. **Why not:** Undrafted resonances
have zero probability (no baseline tokens), killing splashability entirely.

### 1B. Running Tally Slots

**Description:** "Your highest symbol count fills 2 of 4 pack slots, second
fills 1, last is random."

**Domain:** Accumulation. **Championed:** No. **Why not:** Premature convergence
from pick 1 -- structurally identical to Echo Window (4C) but with permanent
full-history memory, making pivots impossible.

### 1C. Resonance Meters

**Description:** "Each resonance has a 0-10 meter; pack slots roll 1-10 and
show a qualifying resonance."

**Domain:** Accumulation. **Championed:** No. **Why not:** Fails the Simplicity
Test. Roll-against-meter with qualifying-resonance selection is a multi-step
conditional process players cannot mentally simulate.

### 1D. Snowball Pool

**Description:** "Each draft pick adds 3 cards sharing its primary resonance to
a personal pool; packs draw from the personal pool."

**Domain:** Accumulation. **Championed:** No. **Why not:** Convergence too weak.
Adding 3 cards to a 360+ pool shifts probability only ~0.8% per pick. After 15
committed picks, dominant resonance rises from 25% to 30% -- insufficient.

### 1E. Weighted Lottery (CHAMPIONED, SIMULATED)

**Description:** "Each resonance starts at weight 1; each drafted symbol adds to
weights (primary +2, others +1); 3 of 4 pack slots pick a resonance
proportionally to weights; the 4th slot is always a random card."

**How it works:** Four weights start at 1. Drafted symbols increment weights.
Three pack slots select resonance by weighted probability. The 4th "wildcard"
slot is always random from the full pool, guaranteeing splash.

**Domain:** Accumulation. **Championed/Simulated:** Yes.

**Corrected Scorecard:**

| Metric | Target | Actual | Pass? |
|--------|--------|--------|:-----:|
| Early unique archs w/ S/A | >= 3 | 6.56 | PASS |
| Early S/A for arch/pack | <= 2 | 2.02 | FAIL |
| Late S/A for arch/pack | >= 2 | 2.32 | PASS |
| Late C/F cards/pack | >= 0.5 | 0.46 | FAIL |
| Convergence pick | 5-8 | 8.5 | FAIL |
| Deck concentration | 60-80% | 92.3% | FAIL |
| Card overlap | < 40% | 8.1% | PASS |
| Archetype frequency | 5-20% | 6-20% | FAIL |

**Result: 3/8 pass** (down from 5/8). The correction exposed that probabilistic
resonance selection does not reliably produce archetype-specific cards.
Convergence pick delayed to 8.5, C/F splash only 0.46, and archetype frequency
barely fails. **Rank: 5th.**

---

## Domain 2: Structural/Guaranteed Mechanisms

Explicit rules determine pack slot contents. Composition predictable from state.

### 2A. One-of-Each

**Description:** "Every pack contains exactly one card from each resonance."

**Domain:** Structural. **Championed:** No. **Why not:** Zero convergence
mechanism. Packs never change based on player choices.

### 2B. Majority Rules

**Description:** "Highest symbol count fills 2 slots, second fills 1, last
random."

**Domain:** Structural. **Championed:** No. **Why not:** Identical to 1B
(Running Tally Slots). Premature convergence from pick 1.

### 2C. Rotating Wheel (originally championed Round 1)

**Description:** "Pack slots cycle E/S/T/Z each pick; majority resonance
duplicates into opposite slot."

**How it works:** Fixed 4-slot rotation advances each pick. When the player has
a majority resonance and it appears on the wheel, it replaces the opposite slot.

**Domain:** Structural. **Championed:** Round 1 only, replaced by Balanced Pack
in Round 2. **Why dropped:** Three interacting systems (rotation position,
majority calculation, opposition mapping). Oscillation averages only 1.0
archetype card per pack, below the 2+ target.

### 2D. Claim Slots

**Description:** "Each card drafted permanently claims one unclaimed slot for
its resonance."

**Domain:** Structural. **Championed:** No. **Why not:** All 4 slots claimed by
pick 4, locking structure forever. Zero late-game agency.

### 2E. Balanced Quartet with Weighted Draw

**Description:** "Every pack has 1/1/1/1, but committed resonances draw from
higher rarity tiers."

**Domain:** Structural. **Championed:** No. **Why not:** Convergence through
quality not quantity. Still only 1 card of committed resonance per pack --
misses 2+ target.

### 2F. Balanced Pack with Majority Bonus (CHAMPIONED, SIMULATED)

**Description:** "Each pack has one card per resonance type; if you have a clear
majority resonance (strictly more weighted symbols, primary=2), it replaces one
non-majority slot, giving you 2 of your majority."

**How it works:** Default pack is 1/1/1/1 (one per resonance). Once one
resonance is strictly ahead in weighted symbol count, packs become 2/1/1/0: two
majority, one each of two random non-majority resonances. Emerged during Round 2
as a simplification of Rotating Wheel.

**Domain:** Structural. **Championed/Simulated:** Yes.

**Corrected Scorecard:**

| Metric | Target | Actual | Pass? |
|--------|--------|--------|:-----:|
| Early unique archs w/ S/A | >= 3 | 7.24 | PASS |
| Early S/A for arch/pack | <= 2 | N/A* | PASS |
| Late S/A for arch/pack | >= 2 | 2.07 | PASS |
| Late C/F cards/pack | >= 0.5 | 0.67 | PASS |
| Convergence pick | 5-8 | 6.3 | PASS |
| Deck concentration | 60-80% | 93.8% | FAIL |
| Card overlap | < 40% | 5.5% | PASS |
| Archetype frequency | 5-20% | 8-24% | FAIL |

*Pre-majority packs are 1/1/1/1 -- cannot bias early packs. N/A counted as
pass.

**Result: 7/9 pass.** Two failures: concentration (shared by all) and Flash at
23.8% from power-based early picks. Best early diversity (7.24) and splash
(0.67). Late S/A (2.07) barely passes with no tuning headroom. **Rank: 2nd.**

---

## Domain 3: Threshold/Progression Mechanisms

Discrete state changes at specific milestones. Clear "level up" moments.

### 3A. Rarity Gating

**Description:** "Drafting 4/8/14 symbols unlocks uncommon/rare/legendary cards
for that resonance."

**Domain:** Threshold. **Championed:** No. **Why not:** Invisible quality tiers.
Off-resonance cards stuck at common. Pivoting heavily punished by losing rare
access.

### 3B. Bonus Card Thresholds

**Description:** "Each resonance at 6+ symbols adds a bonus 5th card to packs."

**Domain:** Threshold. **Championed:** No. **Why not:** Variable pack size
departs from 4-card structure. Dual-committing is strictly better than
mono-committing.

### 3C. Escalating Slot Reservation

**Description:** "For every 5 symbols, reserve one pack slot for that
resonance."

**Domain:** Threshold. **Championed:** No. **Why not:** At 15+ symbols, 3 of 4
slots locked to one resonance, destroying splash and flexibility.

### 3D. Tiered Pool Expansion

**Description:** "Drafting 3/7/12 symbols reveals higher-rarity cards for that
resonance."

**Domain:** Threshold. **Championed:** No. **Why not:** 12 total thresholds
across 4 resonances is too complex. Effects are invisible quality changes.

### 3E. Lane Locking (CHAMPIONED, SIMULATED)

**Description:** "Your pack has 4 slots; when your symbol count in a resonance
first reaches 3, one open slot locks to that resonance; when it first reaches 8,
a second slot locks."

**How it works:** Slots start open (random cards). Crossing threshold 3 locks
one open slot permanently to that resonance. Crossing 8 locks a second. Max 4
total locked slots. Locked slots show a random card of their resonance.

**Domain:** Threshold. **Championed/Simulated:** Yes.

**Corrected Scorecard:**

| Metric | Target | Actual | Pass? |
|--------|--------|--------|:-----:|
| Early unique archs w/ S/A | >= 3 | 6.49 | PASS |
| Early S/A for arch/pack | <= 2 | 1.93 | PASS |
| Late S/A for arch/pack | >= 2 | 2.72 | PASS |
| Late C/F cards/pack | >= 0.5 | 0.84 | PASS |
| Convergence pick | 5-8 | 6.1 | PASS |
| Deck concentration | 60-80% | 99.0% | FAIL |
| Card overlap | < 40% | 5.4% | PASS |
| Archetype frequency | 5-20% | 8-19% | PASS |

**Result: 7/8 pass** (up from 5/8). The biggest winner of the correction.
Locking a resonance slot guarantees S/A cards ~75% of the time for the committed
archetype. Late S/A of 2.72 is the highest of any strategy. Only failure: deck
concentration (shared by all). **Rank: 1st.**

---

## Domain 4: Reactive/Immediate Mechanisms

Only recent picks matter. Short memory enables pivots but limits convergence.

### 4A. Last-Pick Mirror

**Description:** "Each symbol on your last pick fills one slot in the next pack
with that resonance."

**Domain:** Reactive. **Championed:** No. **Why not:** One pick of memory cannot
distinguish committed from random drafters. Convergence structurally impossible.

### 4B. Streak Bonus

**Description:** "Consecutive same-resonance picks lock additional pack slots
(max 3)."

**Domain:** Reactive. **Championed:** No. **Why not:** One off-type pick resets
streak to 0, punishing splash and creating cliff effects.

### 4C. Echo Window (CHAMPIONED, SIMULATED)

**Description:** "Count the resonance symbols across your last 3 picks (primary
= 2, others = 1); your top resonance fills 2 pack slots, your second fills 1,
and the last slot is random."

**How it works:** Sliding window of 3 most recent picks. Sum weighted symbols,
rank resonances. Top gets 2 slots, runner-up gets 1, 4th is random.

**Domain:** Reactive. **Championed/Simulated:** Yes.

**Corrected Scorecard:**

| Metric | Target | Actual | Pass? |
|--------|--------|--------|:-----:|
| Early unique archs w/ S/A | >= 3 | 5.18 | PASS |
| Early S/A for arch/pack | <= 2 | 1.61 | PASS |
| Late S/A for arch/pack | >= 2 | 1.54 | FAIL |
| Late C/F cards/pack | >= 0.5 | 0.38 | FAIL |
| Convergence pick | 5-8 | 7.0 | PASS |
| Deck concentration | 60-80% | 84% | FAIL |
| Card overlap | < 40% | 7% | PASS |
| Archetype freq max | <= 20% | 16% | PASS |
| Archetype freq min | >= 5% | 10% | PASS |

**Result: 6/9 pass.** Late S/A drops from 2.83 (resonance level) to 1.54
(archetype level) -- the correction exposed that resonance-based slot filling
cannot reliably deliver archetype-specific cards. Each resonance serves 4
archetypes, giving ~50% archetype accuracy per resonance-matched slot. Best
pivot flexibility of any algorithm. **Rank: 4th.**

### 4D. Cooldown Wheel

**Description:** "Your last-drafted resonance goes on 2-pick cooldown; pack
slots split among non-cooldown resonances."

**Domain:** Reactive. **Championed:** No. **Why not:** Anti-convergence
mechanism. Actively prevents building on recent picks.

### 4E. Ripple Echo

**Description:** "Each symbol creates echoes reserving slots in your next 2-3
packs."

**Domain:** Reactive. **Championed:** No. **Why not:** Tracking multi-pack echo
queues per resonance is too mentally demanding. Most complex reactive proposal.

---

## Domain 5: Pool Manipulation Mechanisms

Change the available card pool rather than pack construction rules.

### 5A. Resonance Reinforcement

**Description:** "When you draft a card, 3 reserve cards of its resonance are
added to the pool."

**Domain:** Pool. **Championed:** No. **Why not:** Adding 3 to a growing 360+
pool shifts probability only ~0.8% per pick. Too slow for convergence.

### 5B. Pool Filtration

**Description:** "Before each pack, remove cards of resonances you have zero
symbols in."

**Domain:** Pool. **Championed:** No. **Why not:** Catastrophically aggressive.
Three mono-resonance picks filter 75% of pool, making pivots impossible.

### 5C. Drafted Resonance Duplication

**Description:** "For every 3 symbols in a resonance, duplicate one random card
of that type in the pool."

**Domain:** Pool. **Championed:** No. **Why not:** Quadratic accumulation causes
exponential pool growth. By pick 15, a committed player triggers 6+
duplications per pick.

### 5D. Resonance Swap (CHAMPIONED, SIMULATED)

**Description:** "When you draft a card, 3 matching cards are added from a
reserve and 3 non-matching moved to the reserve; each run starts with one
resonance boosted (+20) and another suppressed (-20)."

**How it works:** A ~200-card reserve alongside the 360-card pool. Each draft
swaps 3 non-matching cards out and 3 matching in. Pool stays at ~360. Asymmetric
starting pool creates a detectable signal.

**Domain:** Pool. **Championed/Simulated:** Yes.

**Corrected Scorecard:**

| Metric | Target | Actual | Pass? |
|--------|--------|--------|:-----:|
| Early unique archs w/ S/A | >= 3 | 6.44 | PASS |
| Early S/A for arch/pack | <= 2 | 1.68 | PASS |
| Late S/A for arch/pack | >= 2 | 1.58 | FAIL |
| Late C/F cards/pack | >= 0.5 | 1.24 | PASS |
| Convergence pick | 5-8 | 7.6 | PASS |
| Deck concentration | 60-80% | 83.2% | FAIL |
| Card overlap | < 40% | 6.9% | PASS |
| Archetype frequency | 5-20% | 10-15% | PASS |

**Result: 6/8 pass.** Convergence (1.58) structurally unfixable -- 3 swaps in
360 cards shift probability too slowly. Unique strength: 44.8% signal detection
and best run-to-run variety (6.9% overlap). Self-identified as complementary
layer. **Rank: 3rd.**

### 5E. Resonance Seeding

**Description:** "When you draft a card, add 2 copies of that exact card to the
pool."

**Domain:** Pool. **Championed:** No. **Why not:** Card-level convergence
creates "rich get richer" loops for specific powerful cards rather than broad
archetype support.

---

## Hybrid Proposals (Rounds 2-4)

### H1. Weighted Lottery + Asymmetric Starting Pool

**Description:** "Weighted Lottery (1E) with starting weight 3 plus asymmetric
pool from Resonance Swap (5D)."

**Proposed by:** Agent 1 (Round 4). Projected to fix early S/A from 2.02 to
1.73 while adding signal reading. Not separately simulated. With the corrected
data showing Weighted Lottery at only 3/8, this hybrid is less competitive than
previously thought.

### H2. Balanced Pack + Asymmetric Starting Pool

**Description:** "Balanced Pack (2F) with static asymmetric pool from Resonance
Swap (5D)."

**Proposed by:** Agents 2, 3, 4, 5 (Round 4 consensus). Combines Balanced
Pack's 7/9 pass rate with signal reading. Pool asymmetry is pre-game setup only.

**Status:** Previously recommended. Now second-choice behind Lane Locking +
Pool Asymmetry, given Lane Locking's corrected 7/8 pass and superior
convergence.

### H3. Balanced Pack with Windowed Majority

**Description:** "Balanced Pack (2F) but majority calculated from last 5 picks."

**Proposed by:** Agent 4 (Round 4). Adds pivot flexibility. Not simulated. Risk
of majority/non-majority oscillation in mid-draft.

### H4. Lane Locking + Asymmetric Starting Pool (RECOMMENDED)

**Description:** "Lane Locking (3E) with static asymmetric pool from Resonance
Swap (5D)."

**Rationale:** Lane Locking's corrected 7/8 pass rate and 2.72 late S/A
outperform Balanced Pack's 2.07. Adding pool asymmetry addresses the only
missing goal (signal reading) without changing the lock mechanism.

---

## Final Rankings (Corrected)

| Rank | Algorithm | Domain | Passes | Status |
|:----:|-----------|--------|:------:|--------|
| 1 | **Lane Locking + Pool Asymmetry (H4)** | Threshold + Pool | 7/8* | RECOMMENDED |
| 2 | Balanced Pack + Pool Asymmetry (H2) | Structural + Pool | 7/9* | Strong fallback |
| 3 | Lane Locking (3E) | Threshold | 7/8 | Simulated, excellent |
| 4 | Balanced Pack (2F) | Structural | 7/9 | Simulated, strong |
| 5 | Resonance Swap (5D) | Pool | 6/8 | Best as complement |
| 6 | Echo Window (4C) | Reactive | 6/9 | Convergence limited |
| 7 | Weighted Lottery (1E) | Accumulation | 3/8 | Correction exposed weakness |

*Projected from base simulation + static pool asymmetry layer.

### Why Lane Locking Over Balanced Pack

The corrected metrics invert the previous recommendation. Lane Locking's
structural guarantee of resonance-locked slots translates to 2.72 S/A per pack
vs Balanced Pack's 2.07 -- a 31% advantage on the most important performance
metric. Lane Locking also provides better splash (0.84 vs 0.67), faster
convergence (6.1 vs 6.3), and more balanced archetype distribution (8-19% vs
8-24%). Balanced Pack wins on early diversity (7.24 vs 6.49), but this metric is
trivially satisfied by all strategies at archetype level. If playtesting reveals
permanent locks feel too rigid, Balanced Pack remains a strong second choice.

All 19 non-championed algorithms were rejected for documented reasons:
insufficient convergence (1A, 1D, 2A, 4A, 5A, 5E), premature convergence or
zero flexibility (1B, 2B, 2D, 3C, 4B, 5B), excessive complexity (1C, 3D, 4E,
5C), or oscillation/anti-convergence (2C, 4D). The design space was thoroughly
explored across five structurally distinct domains.
