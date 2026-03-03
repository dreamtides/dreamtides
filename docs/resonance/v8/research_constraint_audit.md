# V8 Research: Constraint Audit & Prior Work Review

## Purpose

This document audits every design constraint carried forward from V3-V7,
quantifies what each relaxation buys, and reviews the strengths of
Pair-Escalation Slots (V5) and Lane Locking (V3) that later versions lost.

______________________________________________________________________

## 1. The Six Inherited Constraints

### 1.1 Dual-Resonance Cap (15%)

**What it is.** Only 54 of 360 cards carry symbols from two different resonance
types. The remaining 270 non-generic cards are single-resonance.

**What raising it costs in card design.** Each dual-resonance card must make
thematic sense in two resonance families. A Tide/Zephyr card needs to feel
coherent in the water-and-wind flavor space. At 15%, the designer creates ~7
such cards per archetype pair -- manageable. At 40% (144 cards), dual-resonance
becomes the default card type. Every archetype pair needs ~18 dual-resonance
cards. This is a substantial increase in cross-resonance design work but notably
different from cross-archetype fitness. A card can carry Tide and Zephyr symbols
while still being mechanically narrow (e.g., "When a creature with Tide enters
play, gain 2 energy" works only in Sacrifice/Warriors archetypes despite having
dual-resonance symbols). The cost is flavor complexity, not necessarily
mechanical breadth.

**What raising it buys.** At 15%, the pair-matched subpool per archetype
contains roughly 25-30 cards (V5 data). Each ordered pair (e.g., Tide-Zephyr for
Warriors) draws from cards whose first two symbols match, which is a narrow
filter. Raising to 30-40% roughly doubles the pair-matched subpool to 50-60
cards, reducing depletion risk and enabling pair-based algorithms like V5's
Pair-Escalation to function at higher cap probabilities without exhausting
candidates. More critically, a richer dual-resonance pool gives algorithms a
second axis of filtering. Single-resonance filtering (V7 Surge) targets by
primary symbol: 50% home archetype, 50% sibling. Dual-resonance filtering
targets by ordered pair: ~80% home archetype (V5 finding). Raising the cap makes
dual-resonance filtering viable for more algorithm classes by ensuring the
filtered subpool is large enough to draw from reliably.

**Value assessment: HIGH.** This is the single highest-value relaxation. The
jump from 50% to 80% archetype precision per targeted slot is the difference
between M3=1.85 and M3=2.6 under Optimistic fitness. The cost is flavor design
work, which is inherently less constrained than mechanical fitness.

### 1.2 Four-Card Fixed Packs

**What it is.** Every pack contains exactly 4 cards. The player picks 1.

**What 5 or 6 cards buys in S/A.** Under Surge+Floor with 3 targeted slots, a
4-card pack has 3 targeted + 1 random. With 5-card packs, the algorithm could
target 3 and leave 2 random (same S/A per pick, more splash) or target 4 and
leave 1 random (higher S/A). At 75% precision per targeted slot:

- 4-card, 3 targeted: 3 x 0.75 + 1 x 0.25 = 2.50 expected S/A (Optimistic)
- 5-card, 4 targeted: 4 x 0.75 + 1 x 0.25 = 3.25 expected S/A (Optimistic)
- 5-card, 3 targeted: 3 x 0.75 + 2 x 0.25 = 2.75 expected S/A (Optimistic)

Under Moderate fitness (75% precision), an extra targeted slot adds ~0.56 M3.
Under Pessimistic (62.5%), it adds ~0.38.

**What it costs.** Decision complexity scales with pack size. Choosing from 5
cards is ~25% more cognitively demanding than from 4, and the draft takes 25%
fewer rounds (30 picks from 5-card packs = 150 cards shown vs. 120 from 4-card).
Larger packs also increase the probability that the "best" card is obvious,
reducing meaningful choice. In a roguelike deckbuilder where draft fatigue is a
concern, adding a card per pack is a real player-experience cost. Draft length
is unaffected (still 30 picks) but each pick decision is heavier.

**What variable pack sizes buy.** Surge packs could expand to 5-6 cards while
non-surge packs stay at 4. This preserves cognitive simplicity on most packs
while amplifying surge quality. However, variable pack sizes add implementation
complexity and may feel arbitrary ("why did this pack have more cards?"). The V6
Double Enhancement algorithm (T=1/B=2) tested variable packs (4 base + 2 bonus =
6 cards on trigger) and achieved 2.13 S/A, passing 9/9. Variable sizes work
mechanically but add a "why?" question the player must understand.

**Value assessment: MEDIUM.** A meaningful S/A boost but with real
player-experience cost. Best used selectively (variable surge packs) rather than
as a blanket increase.

### 1.3 One-Sentence Explainability

**What it is.** Every algorithm must be describable in one sentence of concrete
operations that a programmer could implement.

**What algorithms become available if relaxed.** Three families open up:

1. **Multi-layered hybrids** like Surge+Floor+Bias (~1.97 projected M3 under
   Moderate, never tested). This requires two sentences minimum: one for the
   surge/floor structure, one for the bias weighting. V7 projected this at +0.12
   M3 over Surge+Floor -- potentially the cheapest path to crossing 1.9.

2. **Adaptive algorithms** that change behavior based on draft phase. Example:
   pair-matching in picks 1-10, then surge-based in picks 11-30. These require
   paragraphs, not sentences, but could combine V5's early precision with V7's
   late robustness.

3. **Hidden-state algorithms** where the player never learns the mechanism.
   Machine-learned pack construction, adaptive targeting that adjusts based on
   the player's behavior patterns, or multi-variable optimization of pack
   composition. The player observes "packs get better as I commit" without
   understanding why. These have no inherent S/A ceiling -- they can use any
   amount of information.

**What is lost.** Transparency, predictability, and the player's ability to form
mental models. V3's Lane Locking scored highest on simplicity precisely because
the player can see and predict lock state. Opaque algorithms that produce better
outcomes may feel less satisfying than transparent ones that produce slightly
worse outcomes. The designer (user) noted in V8 that algorithms where the
experience is intuitive ("my packs keep getting better") are acceptable even if
the mechanism is not. This is a significant softening of the constraint.

**Value assessment: MEDIUM-HIGH.** Relaxing to "two-sentence" descriptions
unlocks Surge+Floor+Bias and simple phase-based hybrids at minimal transparency
cost. Full relaxation to hidden-state algorithms opens a much larger design
space but with real player-experience risks. The highest value is in the partial
relaxation zone.

### 1.4 Zero to Three Symbols Per Card

**What it is.** Cards carry 0-3 resonance symbols. V7's pool had most cards at
1-2 symbols with 25% at 3 symbols (V3/V5 recommended distribution: 15/60/25 for
1/2/3 symbols among non-generics).

**What mandating more symbols costs.** If every card had exactly 3 symbols, the
designer must assign a meaningful third resonance to every card. For a card like
"Warriors Shield Bash" (mechanically pure Tide/Zephyr), what is the third
symbol? Forcing a third resonance creates arbitrary assignments that weaken the
resonance system's semantic clarity. At 2 mandatory symbols, the cost is lower:
single-resonance cards (currently 15% of non-generics) would need a secondary
symbol, which is usually justifiable by adjacent-archetype flavor.

**What it buys.** More symbols per card means faster token accumulation (for
Surge-type algorithms) and more pair data (for Pair-Escalation). Under V5,
pair-matching required 60% of cards to have 2+ symbols; moving to 85% (only
generic cards at 0-1) would increase the pair-data generation rate by ~40%,
allowing Pair-Escalation to reach its 50% cap 2-3 picks earlier. For Surge
algorithms, more symbols means faster counter growth and more frequent surges.
At T=3 with an average of 3.5 weighted symbols per pick (up from ~3.0), surges
fire roughly every 1.0-1.2 picks instead of 1.5-2.0.

**Value assessment: LOW-MEDIUM.** The benefit is incremental (faster
accumulation, earlier convergence) rather than structural. The V7 finding that
T=3 dominates T=4 already compensates for slower accumulation. Mandating 2
symbols minimum is low-cost and modestly helpful; mandating 3 is higher cost
with diminishing returns.

### 1.5 360-Card Pool Size

**What it is.** The pool contains exactly 360 cards: ~40 per archetype (320)
plus 36 generic.

**Is this the right size?** The key metric is per-resonance subpool size. Each
resonance is primary for ~90 cards (360/4). A Surge-filtered draw pulls from ~90
candidates, which is large enough to avoid depletion over 30 picks. At 240 cards
(~60 per resonance), depletion becomes possible for aggressive algorithms: 3
surge slots per pack x 15 surge packs = 45 cards drawn from a 60-card subpool.
At 480 cards (120 per resonance), there is no depletion risk but the card
designer must create 120 more cards.

For pair-matched subpools, pool size matters more. Each ordered pair has ~25-30
cards at 360/15% dual-resonance. A larger pool of 480 cards at 30%
dual-resonance would give ~50 cards per pair, making pair-based algorithms
substantially more robust.

**Value assessment: LOW.** 360 is well-calibrated for single-resonance
algorithms. For pair-based algorithms, the binding constraint is dual-resonance
percentage, not total pool size. Increasing the dual-resonance cap at 360 cards
achieves the same effect as expanding to 480 cards at 15% dual-resonance, with
far less card design work.

### 1.6 Zero Player Decisions (Beyond Card Selection)

**What it is.** The player's only action is picking 1 card from the pack. No
token spending, no "choose a row," no meta-decisions.

**What a middle ground looks like.** V4's Pack Widening required spending tokens
each turn (complex: 1 decision per pack, resource management). Simpler middle
grounds:

- **"Choose a row" (binary choice per pack).** Two rows of 2 cards each; one row
  is resonance-targeted, the other is random. The player picks a row, then picks
  a card from it. This adds exactly 1 binary decision per pack, with perfect
  information. Total cognitive load: low. S/A benefit: the player can opt out of
  targeted packs when the random row has a powerful card, preserving
  flexibility. Estimated value: +0.2-0.3 M3 from optimal row timing. But this
  fundamentally changes the draft UX from "pick a card" to "pick a row then pick
  a card" -- a 2-step process.

- **"Accept or reject the surge."** When a surge fires, the player can decline
  it (keeping a random pack) and bank the tokens. This is simpler than Pack
  Widening's spending decision (binary rather than resource management) and lets
  players time their surges. Estimated value: +0.3-0.5 M3 from optimal surge
  timing. Cost: adds a "do you want this?" prompt every 1.5-2 picks.

- **One-time archetype declaration (pick 5-6).** Player explicitly selects an
  archetype, and subsequent packs are targeted at both the primary and secondary
  resonance of that archetype. This is not a recurring decision but a single
  commitment point. Estimated value: potentially large (enables dual-resonance
  targeting from a known archetype, not inferred). Cost: reduces the organic
  feel of gradual commitment.

**What V6 found.** Auto-Spend Pack Widening (zero-decision version) scored 1.76
S/A vs. Pack Widening v2's 3.35 with spending decisions. The spending decision
was worth 1.6 S/A. However, V6's Auto-Spend was poorly optimized. The gap
between "zero decisions" and "one simple binary decision" is likely much smaller
than 1.6 S/A -- perhaps 0.2-0.5.

**Value assessment: MEDIUM.** A single binary decision per surge (accept/reject)
is the highest-value minimal-decision option. It costs minimal cognitive load
while recovering some of the 1.6 S/A gap V6 identified. However, the V8
orchestration plan explicitly marks "no extra actions" as non-negotiable. This
constraint should only be relaxed if zero-decision algorithms definitively
cannot reach 2.0 under realistic fitness even with pool changes.

______________________________________________________________________

## 2. V5 Pair-Escalation Slots: Lost Strengths

### What V5 Achieved

Pair-Escalation Slots reached 2.61 S/A with zero decisions, natural variance
(stddev 0.98), and convergence at pick 6.3 -- superior to V7's Surge+Floor on
the primary metric by 41%. The mechanism's core insight was profound: ordered
resonance pairs (primary, secondary) achieve ~80% S-tier archetype precision vs.
~50% for single-resonance matching. This is the only approach that structurally
bypasses the sibling A-tier problem: if you can filter by (Tide, Zephyr) instead
of just Tide, you get Warriors cards ~80% of the time instead of ~50%.

### What V7 Lost

V7 never tested Pair-Escalation under realistic fitness. The 2.61 figure assumes
Optimistic fitness (100% sibling A-tier). V7's structural finding that "R2 slots
are worthless" was about secondary-resonance SLOTS (filling a slot with
R2-primary cards), not about using R2 as a FILTER on R1-primary cards. These are
fundamentally different operations:

- **R2 slot (worthless):** Fill a slot with a card whose primary resonance is
  R2. This delivers sibling-archetype cards from R2's pool -- wrong archetypes.
- **R2 filter on R1 (V5 approach):** Fill a slot with a card whose ordered pair
  is (R1, R2). This narrows the R1 pool to cards tagged with both resonances,
  dramatically increasing archetype precision.

V7 correctly killed R2 slots but may have incorrectly extrapolated this to
dismiss all R2 involvement, losing V5's filtering advantage.

### Untested Failure Modes Under Realistic Fitness

1. **Pair-matched subpool shrinkage.** Under V5's pool (60% 2+ symbol cards),
   each ordered pair had ~25-30 cards. Under Moderate fitness, ~50% of sibling
   cards in that subpool are B/C-tier. The effective A-tier subpool drops to
   ~12-18 cards. With pair-matching at 50% cap and 4 slots, the algorithm draws
   ~2 pair-matched cards per pack. Over 25 post-commitment picks, that is ~50
   draws from a 12-18 card effective pool -- depletion or repetition becomes a
   real risk.

2. **Pair accumulation rate under fitness degradation.** Pair-Escalation
   requires drafting 2+ symbol cards to build pair counts. If the best available
   card is a 1-symbol or generic card (because pair-matched options are B-tier),
   the player's pair count stalls. Under Pessimistic fitness, the player may
   frequently choose power over pair contribution, slowing convergence below the
   pick 6.3 target.

3. **Deck concentration.** V5 reported 96.2% concentration (fails 60-90%). Under
   degraded fitness, this should improve (more B/C-tier pair-matched cards means
   more off-archetype drafting), but the algorithm's fundamental tendency toward
   high concentration may persist.

4. **The 80% precision assumption.** V5's 80% S-tier precision per pair-matched
   slot assumed perfect pair identification. Under Moderate fitness, some
   pair-matched cards are B/C-tier for the target archetype despite having the
   correct symbol pair. The effective precision would be closer to 80% x 75%
   (pair precision x fitness) = 60% -- still better than single-resonance's 50%
   but less transformative than the Optimistic figure suggests.

### Conditions for Viability

Pair-Escalation becomes viable under realistic fitness if: (a) the
dual-resonance cap is raised to 30%+, ensuring pair-matched subpools of 40-50
cards per archetype pair; (b) the fitness rate for pair-matched cards
specifically (not all sibling cards) is above 50%, which is plausible since
cards tagged with both resonances of an archetype pair are more likely to be
mechanically relevant to that archetype; and (c) minimum symbols per card is
raised to 2 for non-generics, ensuring pair data generation on most picks.

______________________________________________________________________

## 3. V3 Lane Locking: Lost Strengths

### What V3 Achieved

Lane Locking reached 2.72 S/A at the archetype level (V3's corrected data), the
highest raw convergence of any algorithm in V3-V7. It converged at pick 6.1, had
0.84 off-archetype splash per pack, and balanced archetype frequency at 8-19%.
Its simplicity was unmatched: binary lock state, perfect predictability. V6
retested it at 2.22 S/A (different pool/measurement), still the highest raw S/A
in V6.

### What Later Versions Lost

V6 and V7 rejected Lane Locking for three failures: convergence timing (too fast
at pick 3.3 in V6), deck concentration (96-99%), and variance (stddev 0.50). But
these failures have nuance:

- **Too-fast convergence** was measured with thresholds (3, 8). Higher
  thresholds (5, 12) would slow convergence into the 5-8 window while
  maintaining the lock mechanism. V3-V7 never tested Lane Locking with
  pair-based locking (lock at ordered pair thresholds rather than
  single-resonance thresholds), which would be slower to trigger and more
  precise when it does.

- **Excessive concentration** is partly a fitness model artifact (V3 noted
  this). If the fitness gap between S/A and B/C tiers is narrowed, committed
  players draft more B-tier cards, reducing concentration naturally.

- **Low variance** (stddev 0.50) is the most structural problem. Locked slots
  produce identical pack structures every pick. V7's Surge mechanism solved this
  through state alternation. But a hybrid -- locked slots plus probabilistic
  unlocked slots -- was never tested. A "soft lane lock" where locked slots show
  resonance-matched cards 80% of the time (random 20%) would increase variance
  while preserving most of the convergence benefit.

### Conditions for Viability

Lane Locking becomes viable if: (a) thresholds are tuned higher to fix M5; (b)
locks are softened to 80-85% probability per locked slot (fixing M9 variance);
(c) pair-based locking is used instead of single-resonance locking (increasing
archetype precision from ~75% to ~80%); and (d) concentration targets are
relaxed to 60-95% (acknowledging the fitness model artifact). Under these
modifications, Lane Locking becomes "Soft Pair Locking" -- a hybrid of V3's
determinism and V5's pair precision. V6 Agent 3 tested Soft Locks (75%
probability) at 1.75 S/A, but used single-resonance matching. Pair-based soft
locking is untested and potentially significantly stronger.

______________________________________________________________________

## 4. Constraint Relaxation Priority Matrix

| Constraint                          | Relaxation Cost               | S/A Gain                           | Priority        |
| ----------------------------------- | ----------------------------- | ---------------------------------- | --------------- |
| Dual-resonance cap (15% to 30-40%)  | Moderate (flavor design)      | +0.5 to +0.8 M3 via pair-filtering | **HIGHEST**     |
| Explainability (1 sentence to 2-3)  | Low (transparency)            | +0.1 to +0.3 M3 via hybrids        | **HIGH**        |
| Pack size (4 to variable)           | Moderate (UX complexity)      | +0.3 to +0.5 M3 per extra slot     | **MEDIUM**      |
| Min symbols (0-3 to 2-3 mandatory)  | Low (minor design constraint) | +0.1 M3 via faster accumulation    | **MEDIUM-LOW**  |
| Zero decisions to minimal decisions | High (core design change)     | +0.2 to +0.5 M3                    | **CONDITIONAL** |
| Pool size (360)                     | High (card creation work)     | Negligible if dual-res raised      | **LOW**         |

### The Highest-Value Combination

Raising the dual-resonance cap to 30-40% combined with relaxing explainability
to two sentences enables algorithms that use pair-based filtering on a
sufficiently large subpool. This combination addresses the fundamental
mathematical constraint: single-resonance targeting at 75% precision cannot
reach M3=2.0 with 3 targeted slots per 4-card pack (needs 3.5 slots). Pair-
based targeting at ~80% precision needs only 2.9 targeted slots -- achievable
with 3 slots at a slightly higher pair-precision or with the pair-escalation
probability model.

The key insight across all versions is that **the binding constraint is
archetype precision per targeted slot, not slot count or algorithm cleverness.**
V3 through V7 optimized slot count and targeting frequency while holding
precision at 50-75%. V5 showed that pair-matching raises precision to 80% but
required a pool composition that V6-V7 did not adopt. V8 should treat precision
improvement (via pool composition enabling pair-based algorithms) as the primary
lever, with algorithm refinement as secondary.

______________________________________________________________________

## 5. Summary for Round 2 Agents

1. **Raise the dual-resonance cap.** This is the single most impactful change.
   It enables pair-based filtering, which is the only demonstrated path to 80%+
   archetype precision per targeted slot.

2. **Test V5 Pair-Escalation under Moderate and Pessimistic fitness** with the
   raised dual-resonance pool. The 2.61 Optimistic figure will degrade, but
   starting from 2.61 instead of 1.85 gives substantially more headroom.

3. **Test soft pair-locked Lane Locking** as a deterministic alternative to
   probabilistic Pair-Escalation. This hybrid is the untested intersection of
   V3's strongest feature (structural slot guarantees) and V5's strongest
   feature (pair precision).

4. **Relax explainability to two sentences** for the top candidates. The
   Surge+Floor+Bias hybrid (~1.97 projected) and phase-based approaches become
   testable.

5. **Keep 4-card packs as default** but test variable packs (5-card surge packs)
   as a secondary lever if pair-based approaches fall short.

6. **Do not add player decisions** unless all zero-decision approaches fail to
   reach 2.0 under Pessimistic fitness even with pool changes.
