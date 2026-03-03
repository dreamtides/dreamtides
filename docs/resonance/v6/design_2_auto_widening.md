# Agent 2: Auto-Widening -- Algorithm Design

## Key Takeaways from V3/V4/V5

- **The 2.0 S/A ceiling is structural, not tunable.** V4 proved that
  probabilistic approaches (weighting, filtering, exile, phantoms) cap at
  1.26--1.74 S/A because each resonance is shared by 4 archetypes. Only
  mechanisms that ADD targeted cards to packs or deterministically PLACE them in
  specific slots cross the threshold.

- **Pack Widening v2 crossed 2.0 decisively (3.35 S/A) but required spending
  decisions.** The player chose when and which resonance to spend on. V6 bans
  this -- the system must auto-spend. The core challenge is designing an
  auto-spend policy that preserves Pack Widening's power without player input.

- **Lane Locking crossed 2.0 (2.72 S/A) with zero decisions but felt
  mechanical.** Permanent binary locks create deterministic pack structures. The
  goal is to find a middle ground: automatic like Lane Locking, but with the
  natural variance of Pack Widening's stochastic bonus cards.

- **Token accumulation rate matters.** With primary=2, secondary/tertiary=1, a
  committed player earning ~3 tokens per pick reaches cost-3 thresholds roughly
  every pick after commitment. The auto-spend policy must match this cadence
  without requiring player timing decisions.

- **The 15% dual-resonance constraint means ~85% of non-generic cards are
  mono-resonance.** A bonus card drawn from a resonance pool has only ~50%
  chance of being S/A for the specific archetype. To hit 2.0+ S/A, the system
  needs to add bonus cards frequently enough that even at 50% archetype
  precision, the expected S/A count crosses the threshold.

- **V5 pair-matching is off limits as a primary strategy.** With only 54
  dual-type cards, pair-based filtering cannot drive convergence. However,
  dual-type cards can serve as high-value bonus draws when available.

- **Variance is a design goal, not a side effect.** Lane Locking's weakness was
  its 0.84 stddev (barely passing). Auto-Widening should preserve the natural
  variance of random base packs while adding targeted bonus cards on a cadence
  that is not perfectly predictable.

______________________________________________________________________

## Five Algorithm Proposals

### 1. Threshold Auto-Spend (Highest Resonance)

**One sentence:** Each drafted symbol earns 1 matching token (primary earns 2);
when any resonance reaches 3 tokens, the system automatically spends 3 tokens
from that resonance and adds 1 card of that resonance to the next pack.

**Technical description:** Tokens accumulate identically to Pack Widening v2.
After each pick, the system checks all four resonance counters. If any counter
is at or above 3, the system spends 3 tokens from the highest counter (ties
broken randomly) and flags the next pack for a bonus card drawn from cards with
that primary resonance. The pack becomes 5 cards (4 random + 1 bonus). Only one
auto-spend fires per pick. Unspent residual tokens persist.

**Design goal assessment:** Passes simplicity (one sentence, fully concrete).
Zero player decisions. Convergence depends on how often the threshold fires -- a
committed player earning ~3 tokens/pick triggers nearly every pick after pick
5-6, projecting ~2.0-2.5 S/A. Variance comes from random base packs plus the
~50% archetype hit rate on bonus cards. Risk: the "always spend highest" policy
may lock onto the primary resonance too rigidly, reducing splash.

**Preferred symbol distribution:** 15% dual-type (54 cards). Heavy 2-symbol
cards (55%) to ensure ~3 tokens per pick.

### 2. Round-Robin Auto-Spend

**One sentence:** Each drafted symbol earns 1 matching token (primary earns 2);
every 3 picks, the system cycles through resonances in order of your token count
(highest first) and adds 1 card of that resonance to the next pack, deducting 1
token.

**Technical description:** Instead of a threshold, the system fires on a fixed
cadence: every 3rd pick. It selects the resonance with the most accumulated
tokens, deducts 1 token, and adds a bonus card of that resonance. The
round-robin aspect emerges because a committed player's primary resonance
dominates early, but secondary resonance accumulates and eventually gets its
turn. This produces a natural primary-primary-secondary rhythm.

**Design goal assessment:** Zero decisions. Simpler cadence than threshold
(every 3 picks, predictable). But the low deduction cost (1 token) means tokens
grow unboundedly, and the signal is weaker -- only 1 bonus card per 3 picks
gives roughly 0.33 bonus cards per pick. With 50% S/A hit rate, this adds only
~0.17 S/A per pick on average. Unlikely to cross 2.0 alone. Would need to add 2+
bonus cards per trigger to compensate.

**Preferred symbol distribution:** 15% dual-type, standard 2-symbol-heavy
distribution.

### 3. Overflow Auto-Spend (Multiple Fires)

**One sentence:** Each drafted symbol earns 1 matching token (primary earns 2);
after each pick, for every resonance with 3+ tokens, the system spends 3 and
adds 1 bonus card of that resonance to the next pack.

**Technical description:** Unlike Proposal 1 which fires at most once, this
fires for EVERY resonance that crosses the threshold simultaneously. A player
drafting a [Tide, Zephyr] card earns 2 Tide and 1 Zephyr. If Tide was already at
2 and Zephyr at 3, the next pack gets 1 Tide bonus + 1 Zephyr bonus = 6 cards
total. In practice, a committed player usually fires 1 resonance consistently,
occasionally 2 when secondary resonance overflows. This creates natural variance
in pack size (4, 5, or rarely 6).

**Design goal assessment:** Zero decisions. Strong convergence -- fires as often
as tokens allow. The variable pack size (4-6) adds variance without player
input. Projected S/A: if primary resonance fires ~80% of picks and secondary
~20% after commitment, average bonus cards per pack is ~1.0, yielding ~0.5 S/A
bonus per pack. Combined with baseline ~1.0-1.2 S/A from random packs, total
reaches ~1.5-1.7. Still might not cross 2.0. Needs lower threshold (cost 2) or
higher token rate.

**Preferred symbol distribution:** 15% dual-type, slightly more 3-symbol cards
(30%) to increase token generation rate.

### 4. Momentum Auto-Spend

**One sentence:** Each drafted symbol earns 1 matching token (primary earns 2);
when any resonance hits 3 tokens, the system spends 3 and adds 1 bonus card of
that resonance to the next pack, but each consecutive trigger of the same
resonance adds 1 additional bonus card (resetting when a different resonance
triggers).

**Technical description:** This layers a streak mechanic on top of Proposal 1.
The first time Tide triggers, 1 Tide bonus card. If Tide triggers again next
pick, 2 Tide bonus cards. A third consecutive Tide trigger gives 3 bonus cards.
If Zephyr triggers instead, the streak resets to 1. This rewards commitment: a
player consistently drafting one resonance sees escalating pack sizes (5, 6,
7...) while a player who splits resonances resets frequently and stays at
smaller bonus counts. The escalation is bounded by the practical token
generation rate (~3/pick means the streak caps around 3-4 before the player runs
out of primary tokens).

**Design goal assessment:** Zero decisions. Strong convergence for committed
players -- the escalating bonus count means S/A density climbs steeply after
commitment. Estimated 2.5-3.0 S/A at full commitment. Good variance (streak
length varies based on card draws). Risk: may over-reward single-resonance
commitment, creating an "always go mono" meta. The momentum mechanic adds a
visible, exciting progression. However, the one-sentence description is
borderline complex (two clauses).

**Preferred symbol distribution:** 10% dual-type (36 cards). Fewer dual-type
cards to make the streak mechanic more meaningful -- mono-resonance drafting
sustains streaks.

### 5. Drip Auto-Spend

**One sentence:** Each drafted symbol earns 1 matching token (primary earns 2);
when any resonance reaches 2 tokens, the system spends 2 and adds 1 bonus card
of that resonance to the next pack.

**Technical description:** This is the simplest possible auto-widening: a low
threshold (2 tokens) that fires frequently. A committed player earning ~3 tokens
per pick in their primary resonance triggers every single pick after pick 2-3 (2
tokens earned, 2 spent, 1 residual carries over and combines with next pick's
earnings). The low threshold also means secondary resonances trigger
occasionally (every 2-3 picks), providing natural splash. The pack is always 4
random + 1 bonus = 5 cards, with the bonus cycling between primary and secondary
resonance based on which crosses 2 first.

**Design goal assessment:** Zero decisions. Maximum simplicity -- the lowest
possible non-trivial threshold. Fires very frequently, projecting ~0.9-1.0 bonus
cards per pick. With 50% S/A hit rate on bonus cards, adds ~0.45-0.5 S/A per
pick. Combined with baseline random S/A of ~1.0-1.2, total is ~1.5-1.7. Still
structurally limited by the 50% archetype dilution on bonus cards. To cross 2.0,
would need cost 2 with 2 bonus cards, but then over-concentration returns.

**Preferred symbol distribution:** 15% dual-type, standard distribution. The low
threshold makes symbol count less critical.

______________________________________________________________________

## Champion Selection: Proposal 1 -- Threshold Auto-Spend (Highest Resonance)

**Justification:** Proposal 1 strikes the best balance across all design goals.
It is the most direct automation of V4's Pack Widening -- replacing the player's
spend decision with the simplest possible auto-policy (spend the highest counter
when it hits threshold). It inherits Pack Widening's proven power of adding
targeted cards to packs while eliminating the decision entirely.

Why not the others:

- **Round-Robin (#2)** fires too infrequently at 1 bonus per 3 picks. The math
  does not reach 2.0.
- **Overflow (#3)** fires multiple resonances but still faces the same S/A
  ceiling unless thresholds drop very low, at which point it becomes equivalent
  to #5.
- **Momentum (#4)** has the best raw convergence potential but introduces
  streak-tracking complexity and risks creating a dominant mono-resonance
  strategy. The escalation mechanic, while exciting, adds a second rule on top
  of the core auto-spend.
- **Drip (#5)** is the simplest but preliminary math suggests it caps at ~1.7
  S/A due to the 50% archetype dilution on every bonus card. The low threshold
  fires frequently but each fire only adds ~0.5 expected S/A.

Proposal 1 can be tuned to cross 2.0 by adjusting threshold and bonus count
together. At cost 3 / bonus 2, a committed player who triggers every pick gets 2
bonus cards per pack, adding ~1.0 expected S/A on top of the ~1.0-1.2 baseline,
reaching ~2.0-2.2. This is the configuration I will deep-dive.

______________________________________________________________________

## Champion Deep-Dive: Threshold Auto-Spend

### One-Sentence Description

Each drafted symbol earns 1 matching token (primary earns 2); when any resonance
reaches 3 tokens, the system automatically spends 3 from the highest counter and
adds 2 bonus cards of that resonance to the next pack.

### Example Draft Sequence

**Scenario: Player commits to Warriors (Tide primary, Zephyr secondary)**

| Pick | Card Drafted       | Symbols              | Tokens After (T/Z/E/S) | Auto-Spend?        | Next Pack Size |
| ---- | ------------------ | -------------------- | ---------------------- | ------------------ | -------------- |
| 1    | Generic power card | []                   | 0/0/0/0                | No                 | 4              |
| 2    | Tide character     | [Tide, Tide]         | 3/0/0/0                | Yes: Tide (3->0)   | 4+2=6          |
| 3    | Warriors card      | [Tide, Zephyr]       | 2/1/0/0                | No                 | 4              |
| 4    | Tide spell         | [Tide]               | 4/1/0/0                | Yes: Tide (4->1)   | 4+2=6          |
| 5    | Warriors 3-sym     | [Tide, Tide, Zephyr] | 3/2/0/0                | Yes: Tide (3->0)   | 4+2=6          |
| 6    | Tide character     | [Tide, Tide]         | 3/2/0/0                | Yes: Tide (3->0)   | 4+2=6          |
| 7    | Zephyr card        | [Zephyr]             | 0/4/0/0                | Yes: Zephyr (4->1) | 4+2=6          |
| 8    | Tide spell         | [Tide, Tide]         | 3/1/0/0                | Yes: Tide (3->0)   | 4+2=6          |
| 9    | Warriors dual      | [Tide, Zephyr]       | 2/2/0/0                | No                 | 4              |
| 10   | Tide card          | [Tide]               | 4/2/0/0                | Yes: Tide (4->1)   | 4+2=6          |

**Observations:** The committed player triggers auto-spend on roughly 7 of 9
non-generic picks. Most triggers fire on Tide (primary resonance). Pack 7 fires
Zephyr instead -- providing secondary-resonance splash naturally. Packs
alternate between 4 and 6 cards in a somewhat unpredictable rhythm, creating
variance. Of the ~14 bonus Tide cards seen across picks 2-10, roughly 7 are
Warriors S/A-tier (50% archetype precision). Combined with baseline random S/A,
this projects to ~2.0-2.5 S/A per pack from pick 5 onward.

**Scenario: Slower committer exploring 2 resonances**

| Pick | Card Drafted | Symbols        | Tokens After (T/Z/E/S) | Auto-Spend?       | Next Pack Size |
| ---- | ------------ | -------------- | ---------------------- | ----------------- | -------------- |
| 1    | Ember card   | [Ember]        | 0/0/2/0                | No                | 4              |
| 2    | Tide card    | [Tide]         | 2/0/2/0                | No                | 4              |
| 3    | Ember spell  | [Ember, Stone] | 2/0/4/1                | Yes: Ember (4->1) | 4+2=6          |
| 4    | Tide card    | [Tide, Tide]   | 5/0/1/1                | Yes: Tide (5->2)  | 4+2=6          |
| 5    | Tide card    | [Tide]         | 4/0/1/1                | Yes: Tide (4->1)  | 4+2=6          |

The explorer triggers both Ember and Tide, seeing bonus cards from both
resonances. The system naturally tracks their exploration without requiring a
decision about which resonance to invest in. When they settle on Tide, Tide
triggers dominate. Ember tokens sit at 1, occasionally accumulating to 3 and
firing a splash bonus.

### Failure Modes

1. **Archetype dilution ceiling.** The fundamental problem: each bonus card
   drawn from a resonance pool has only ~50% chance of being S/A for the
   player's specific archetype. With 2 bonus cards per trigger, expected S/A
   from bonuses is ~1.0 per widened pack. If the baseline random pack delivers
   ~1.0-1.2 S/A, the total of ~2.0-2.2 barely clears the threshold. Poor luck
   streaks could drop individual packs below 2.0 regularly. Mitigation: higher
   bonus count (3 per trigger) at cost of over-concentration.

2. **Generic card dead picks.** Picking a generic card earns 0 tokens, delaying
   the next trigger. A player who picks 3+ generics in a row gets 3+ packs
   without bonus cards. This is acceptable variance if generics are strong
   enough to justify picking, but could feel punishing. Mitigation: generics
   could grant 1 token to each resonance equally, but this adds complexity.

3. **Split-resonance stalling.** A player who drafts evenly across 2 resonances
   may never hit threshold 3 in either. E.g., alternating Tide and Ember cards:
   each resonance accumulates 2 tokens per pick but spends 3, staying near 0-2.
   In practice this is unlikely because primary=2 weighting means even one pick
   pushes a counter to 2, and a second pick in the same resonance reaches 4. But
   a truly even split could reduce trigger frequency to every other pick instead
   of every pick.

4. **Auto-spend on wrong resonance.** The "highest counter" policy could spend
   secondary resonance tokens when the player would prefer primary. E.g., a
   Warriors player who picks a Zephyr-heavy pack could see Zephyr accumulate to
   3 and trigger a Zephyr bonus instead of waiting for Tide. This is by design
   (it provides splash), but could feel suboptimal. The player has no way to
   override.

### Parameter Variants for Simulation

**Variant A (Baseline): Cost 3 / Bonus 2**

- Threshold: 3 tokens to trigger
- Bonus cards per trigger: 2
- Projected: ~2.0-2.2 S/A, moderate variance, ~every-pick triggers for committed
  players

**Variant B (Conservative): Cost 4 / Bonus 2**

- Higher threshold delays first trigger to pick 3-4 instead of pick 2
- Slower trigger cadence (~every 1.3 picks) reduces over-concentration
- Projected: ~1.8-2.0 S/A, higher variance (more 4-card packs), better early
  openness

**Variant C (Aggressive): Cost 3 / Bonus 3**

- Same threshold but 3 bonus cards per trigger
- Pack size is 4+3=7 when triggered -- lots of choice
- Projected: ~2.5-3.0 S/A, risk of over-concentration (>90%), high bonus card
  feel

### Proposed Symbol Distribution

| Symbol Count          | % of Non-Generic | Cards | Rationale                                           |
| --------------------- | ---------------- | ----- | --------------------------------------------------- |
| 0 (generic)           | --               | 36    | Standard 10% generic pool                           |
| 1 symbol              | 20%              | 65    | Low-commitment cards for early exploration          |
| 2 symbols (mono-type) | 45%              | 146   | Core drafting: [Tide, Tide] gives 3 weighted tokens |
| 2 symbols (dual-type) | 10%              | 32    | [Tide, Zephyr] archetype signals, within 15% cap    |
| 3 symbols (mono-type) | 18%              | 59    | High-commitment: [Tide, Tide, Tide] gives 4 tokens  |
| 3 symbols (dual-type) | 7%               | 22    | [Tide, Tide, Zephyr] strong signal + tokens         |

**Totals:** 36 generic + 324 non-generic = 360 cards. Dual-type cards: 32 + 22 =
54 (exactly 15% of 360).

**Justification:** The 45% mono-type 2-symbol cards are the workhorse -- they
generate 3 weighted tokens per pick, which means a committed player hits cost-3
threshold every pick. The 10% dual-type 2-symbol cards provide the
rare-but-valuable archetype pair signals. The 18% mono-type 3-symbol cards are
high-commitment payoffs (4 tokens per pick). The 7% dual-type 3-symbol cards are
the strongest signals in the pool -- [Tide, Tide, Zephyr] is unambiguously
Warriors and generates 4 tokens.

This distribution gives each archetype approximately: 8 mono-1-symbol, 18
mono-2-symbol, 4 dual-2-symbol, 7 mono-3-symbol, 3 dual-3-symbol = 40 cards. The
token generation rate for a committed player averages ~3.1 tokens per pick in
their primary resonance, sustaining near-every-pick auto-spend at cost 3.
