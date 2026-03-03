# Critic Review — V11 Round 3

## 1. Ranking Table

| Rank | Design | Champion | M3/M11' Potential | Player Experience | Simplicity | Signal Quality | "Not on Rails" |
|------|--------|----------|:-----------------:|:-----------------:|:----------:|:--------------:|:--------------:|
| 1 | Design 6 | Asymmetric Replacement | 2.2-2.5 / 2.5-3.0 | High (visible market) | Medium | Excellent | High |
| 2 | Design 4 | Static Open-Lane Multiplier | 1.9-2.2 / 2.3-2.7 | High (clean narrative) | High | Good | High |
| 3 | Design 3 | Graduated 4-Round Decline | 2.0-2.4 / 2.5-2.9 | Medium-High | Medium | Good | Medium |
| 4 | Design 1 | Asymmetric Refill (Algo B) | 1.9-2.3 / 2.2-2.7 | High (MTG familiarity) | High | Good | Medium |
| 5 | Design 5 | Bars + Snapshot + Trends | N/A (orthogonal axis) | High | Medium | Excellent | N/A |
| 6 | Design 2 | Small Pool, Fast Cycles | 1.8-2.2 / 2.2-2.6 | Low (cognitive overload) | Low | Poor | Medium |

**Notes:**

Design 1 (Algorithm B) and Design 4 (Proposal B) are mechanically nearly
identical — both use 3 rounds x 10 picks, 120-card starting pool, 60-card
refills with open-lane bias fixed at draft initialization. The key difference
is bias expression: Design 1 gives AI lanes 50% share (~3.75 cards each, open
lanes ~12.5), while Design 4's 1.7x multiplier yields ~10.7 per open lane and
~4.3 per AI lane. These converge to the same structural outcome. Both champions
predict M3 1.9-2.3, M11' 2.2-2.7. They are one design, not two.

Design 5 cannot be ranked on M3/M11' because it is orthogonal — it is an
information layer applied on top of any mechanism design. Its ranking here
reflects its quality as that layer. It is the strongest information proposal
in the field and should be paired with a mechanism design, not treated as
a standalone.

Design 2 is last on player experience and simplicity despite a competitive M3
ceiling. Five rounds with 6 picks each, 4 refill events, declining volumes,
and depletion-rate tracking creates cognitive overhead that 3-round designs
avoid. Its M3 ceiling (2.2) is not meaningfully higher than simpler designs to
justify this cost.

---

## 2. V10 Root Cause Assessment

### Root Cause 1: Pool Exhaustion

**Verdict: Fully resolved by all V11 designs.**

Any positive refill between rounds prevents the pick-12-15 exhaustion V10
suffered. Even Design 2's thin Round 4 (17 cards) is well above V10's actual
exhaustion threshold. This root cause is closed. The question is not whether
refills fix exhaustion but whether the specific refill schedule creates
other problems.

### Root Cause 2: S/A Preferential Depletion

**Verdict: Partially resolved, not structurally solved.**

The remediation math is clear: full balanced refills add 2.5 S/A cards per
archetype per round while AIs drain 3-4 per round from AI lanes. This narrows
the depletion curve but does not reverse it. The S/A cycling failure mode
(AIs drain S/A, refills replenish S/A, AIs drain it again) is real. Over 3
rounds, AI-lane S/A density declines slowly; open-lane S/A density is
approximately stable. This produces a measurable but modest quality advantage
for the player in their open lane.

The critical implication: M11' (picks 20+, S/A >= 2.5) is achievable not
because the system manufactures S/A concentration but because open-lane S/A
cards accumulate through AI non-competition, not through refill enrichment.
This is the correct mechanism. It works.

### Root Cause 3: Level 0 Targeting Dilution

**Verdict: Not resolved; partially deferred by structural accumulation.**

This is V11's binding constraint. The player is still 1 of 3 open lanes.
Balanced refills mathematically reset the concentration gradient: a 1.18x
gradient after Round 1 compresses to 1.11x after a balanced refill. Over 3
rounds, cumulative concentration plateaus at 1.3-1.6x — comparable to V10's
1.7x, not V9's 5-7x.

All designs that reach M3 >= 2.0 do so by partially circumventing root cause
3 through structural bias (Designs 1, 4, 6) or by letting AI depletion outrun
refill replenishment per round (Design 3). None of them solve the 1-of-3
targeting dilution directly; they mitigate it indirectly.

The honest conclusion: multi-round refills fix root cause 1 cleanly, slow root
cause 2 adequately, and fail to fix root cause 3 without supplemental
mechanisms. The supplemental mechanisms are real and effective — but they
should be named clearly.

---

## 3. Refill Bias Honesty Analysis

### Open-Lane-Biased Refills (Designs 1, 4): Honest Level 0

These are genuinely Level 0. The bias is determined entirely by the AI
archetype configuration set at draft initialization — a static property that
does not change during the draft and does not observe the player's picks. The
system is not watching what the player takes; it is restocking the market
based on pre-draft lane assignments.

The framing concern is real but manageable. "The market restocks slow-moving
inventory" is accurate (open lanes are literally slow-moving because no AI
is buying them) and does not require the player to know that "slow-moving =
no AI assigned." The mechanism is honest.

Design 4's Proposal C (graduated bias with partial refills) is also Level 0
and produces better M3 (2.1-2.5) with higher M12 (0.45-0.60) at the cost of
a second tuning parameter. This is the correct trade.

### Design 3's Underrepresented Bias: Concentration Destroyer

Design 3's Round 3 refill bias — giving proportionally more cards to archetypes
below 8 cards — is not V9 contraction, but it is counterproductive. Design 4
makes this explicit: underrepresented archetypes ARE the AI lanes. They are
depleted because 5 AIs are constantly drafting from them. Restocking them
preferentially means the refill is working against the concentration mechanism
rather than with it.

Design 3's declining volume (100/75/50/0) does build genuine concentration
accumulation — AI depletion outpaces declining refill volume. But the Round 3
underrepresented bias partially cancels this. The declining volume is doing the
work; the bias is friction. Design 3 should drop the underrepresented bias from
Round 3 refills and let the declining volume stand alone.

### Design 6's Asymmetric Replacement: Structural, Not Biased

Design 6 does not use refill bias at all. Its concentration mechanism is
different: AI picks are immediately replaced (AI lanes cycle), player picks
are not replaced (open lanes deplete monotonically). This is not bias — it is
an asymmetric replacement rule that produces concentration as a structural
consequence.

The V9 comparison question from the brief is: is this V9 contraction by another
name? The answer is no, but the resemblance is real and worth naming. V9
removed non-player cards from the pool through an invisible filter. Design 6
achieves similar directionality differently: it does not remove non-player
cards, but it does not restock player-accessible cards either. The mechanism
is visible and narratively grounded (the market restocks what was sold, not
what was taken). Players can observe the asymmetry and reason about it.

The important difference from V9: Design 6's concentration builds from
structural replacement rules, not from the system knowing what the player
wants. The player's open lane depletes because the player is taking from it,
and nothing replaces those cards. V9 depleted the pool based on player identity.
Design 6 depletes based on player action. This is a meaningful distinction.

---

## 4. Player Information Evaluation

### Design 5's Three-Layer System: Creates Genuine Skill

Design 5 (Bars + Snapshot + Depletion Trends) is the best information
architecture in the field. Its key structural insight — quality hidden when
quantity is visible — is the mechanism that preserves the skill axis. A player
who sees "Tide: 18 cards available" still faces the Sacrifice vs. Warriors
decision, still cannot see which of those 18 cards are S/A tier, and still
benefits from committing earlier rather than later.

The depletion trend arrows are the strongest single element. They uniquely
reward signal-readers without helping committed players: a player already
committed to Storm does not need to know that Storm is depleting slowly (they
know — they're the one taking from it). Only an uncommitted player comparing
multiple lanes benefits from relative trend rates. This is the mechanism behind
M12 >= 0.35.

The snapshot staleness mechanism (dimming from pick 4 onward) is correct. A
round-start snapshot that remains equally prominent through pick 9 would teach
players to ignore it by mid-round or to rely on it past its usefulness window.
Dimming communicates information age visually without requiring text.

### Refill Preview: Dangerous for M5

Design 1 Algorithm C and Design 5's discussion both touch refill preview. This
should not be included in any champion design. Showing what the next refill
contains gives players a structural reason to delay commitment past round end,
weakening M5 (convergence target: picks 5-8). A player who knows a large Tide
refill is incoming will defer Tide commitment until round 2, getting 8 picks
of Tide instead of 18. The option value of waiting dominates early commitment.
Design 5 correctly excludes refill preview from its champion.

### Design 6's Full Visibility: Trivializes Nothing

Design 6 shows the full 120-card market, filterable by archetype. This sounds
like it trivializes signal reading — but it does not. The skill axis in Design 6
is not "which lane is open" (that is immediately obvious from AI replacement
activity visible in the market) but "which specific cards in my open lane are
worth taking in what order." The choice collapses from "which archetype" to
"which card within my archetype" — a different but valid skill expression.

The market paralysis risk (120 visible cards vs. a 15-card pack) is a real
concern Design 6 correctly identifies. Strong archetype filtering is required.
Without it, the cognitive overhead could make each pick take significantly
longer than in round-based designs.

---

## 5. Proposed Hybrid Designs

### Hybrid A: Three-Round Classic with Graduated Bias + Design 5 Information

**Structure:** 3 rounds x 10 picks, 120-card starting pool.

**Refill mechanism:** Combines Design 4 Proposal C (graduated open-lane bias
with declining volume) with Design 5's full information system.

- After Round 1: 70 cards added, multiplier 1.4x. Open lanes: ~9.2 each. AI lanes: ~7.3 each.
- After Round 2: 48 cards added, multiplier 2.0x. Open lanes: ~9.6 each. AI lanes: ~4.8 each.
- Round 3 pool: ~108 cards, open lanes at ~57 total vs ~51 AI lanes.
- Information: Availability bars (quantity, not quality), round-start snapshot
  with archetype quality descriptor, depletion trend arrows. No refill preview.

**Why this is better than any individual design:**

Design 4 Proposal C's metrics (M3 2.1-2.5, M11' 2.7-3.1, M12 0.45-0.60) are
the highest among all 3-round designs, but Design 4 proposed static bars only.
Adding Design 5's full information layer (snapshot + trends) raises M12 further
while the graduated bias and declining volume keep concentration building across
rounds. The 3-round structure preserves MTG-familiarity and the "new pack"
strategic recalibration moment.

**Predicted metrics:** M3 2.2-2.6, M11' 2.8-3.2, M12 0.45-0.65.

### Hybrid B: Asymmetric Replacement (Design 6) + Design 5 Trends

**Structure:** Design 6 Proposal A with Design 5's depletion trend indicators
applied to the continuous market.

**Key modification:** Instead of round-start snapshots (the market has no batch
refills to create snapshot moments), use rolling depletion trend arrows that
track each archetype's depletion rate over the last 5 picks. This preserves the
trend indicator's skill value without requiring artificial round boundaries.

**Refill timing adjustment:** Move the single scheduled refill from pick 20 to
pick 16-17. Design 6's own failure mode analysis identifies S/A exhaustion in
an efficient player's open lane by picks 15-18. Moving the refill earlier
prevents a 4-7 pick stretch where the player is taking pure C/F from an
exhausted lane while waiting for restocking.

**Why this is better than Design 6 alone:**

The refill timing fix addresses the highest-risk failure mode. Trend arrows
applied to the continuous market give uncommitted players a richer signal
than archetype bars alone — they can observe that AI lanes are cycling at a
constant rate while their target open lane has barely moved. The information
asymmetry between committed and uncommitted players is larger in this format
than in round-based designs because the signal is continuous and real-time.

**Predicted metrics:** M3 2.3-2.6, M11' 2.5-3.0, M12 0.5-0.7.

---

## 6. Recommended Algorithms for Simulation

### Algorithm SIM-1: Baseline — Pure Balanced Refills (Design 1 Algo A)

3 rounds, 120-card pool, full balanced 60-card refills after rounds 1 and 2.
No bias. Design 5 information (bars only, no trends).

This is the null hypothesis: does emergent concentration through AI depletion
alone reach M3 >= 2.0? Expected to fail (M3 1.3-1.5) but essential as the
calibration anchor for all other algorithms. Without this baseline, we cannot
isolate the contribution of bias mechanisms.

### Algorithm SIM-2: Static Open-Lane Bias (Design 4 Proposal B)

3 rounds, 120-card pool, 60-card refills with 1.7x open-lane multiplier.
Design 5 information (bars + snapshot + trends). Level 0.

The simplest design that should reach M3 >= 2.0. Validates whether fixed
open-lane multiplier alone is sufficient without declining volume.

### Algorithm SIM-3: Graduated Bias + Declining Volume (Hybrid A)

3 rounds, 120-card pool. After Round 1: 70 cards, 1.4x bias. After Round 2:
48 cards, 2.0x bias. Full Design 5 information system.

The strongest 3-round design. Tests whether combining graduated bias with
declining volume reaches M3 2.2+ and M12 0.45+. This is the primary candidate
for "Standard" recommendation.

### Algorithm SIM-4: Graduated 4-Round Decline, No Underrepresented Bias (Design 3 Modified)

4 rounds (8/8/7/7 picks), 120-card pool. Refills: 48 (balanced), 36 (balanced),
21 (balanced — remove underrepresented bias). No final refill. Design 5
information (bars + snapshot, no trends since 4-round boundary events provide
sufficient recalibration moments).

Tests whether declining volume alone (without bias) can build adequate
concentration across 4 rounds. Also tests the 4-round structure's strategic
pacing against 3-round designs on the same M3 metric.

### Algorithm SIM-5: Asymmetric Replacement (Design 6, Timing-Adjusted)

Continuous market (120 starting + 240 reserve), AI picks replaced per pick,
player picks permanent. Single scheduled refill at pick 16 (+20 cards, archetypes
< 10 threshold). Full Design 5 information adapted for continuous format
(rolling 5-pick trend windows instead of round-start snapshots). Snapshots
at picks 10 and 20 as informational pauses.

Tests Design 6's novel concentration mechanism. If this reaches M3 2.2+ and
M12 0.5+, it is the "Advanced" recommendation despite higher implementation
complexity.

### Algorithm SIM-6: Small Pool Concentrated Bias (Design 2 Modified)

5 rounds x 6 picks, 66-card pool, declining refills (70%/55%/40%), 70%
open-lane refill bias (Design 2 Algo 2C). Bars only (no trends — 5-round
structure provides sufficient snapshot moments). Level 1 pool-reactive.

Included to test whether the fast-cycle structure can compete with 3-round
designs on M3/M12 when open-lane bias is strong. Primary risk is Round 4
thin pool (17 cards) and cognitive overhead. If M3 is competitive with SIM-2
but M12 is not, the 3-round designs are confirmed superior.

---

## Cross-Cutting Observations

**The bias vs. structure question (Key Question 4):** Designs 1 and 4 use 3-round
structure with bias; Design 3 uses 4-round declining volume. The research math
suggests bias direction matters more than round count for M3. A 3-round design
with 1.7x open-lane bias (Design 4B) predicts M3 1.9-2.2. Design 3 with pure
declining volume predicts M3 2.0-2.4. Design 3 is slightly stronger on M3 but
uses 4 rounds, 4 refill events, and a more complex pick schedule. The marginal
M3 gain does not justify the added complexity unless simulation confirms a
reliable 0.3+ M3 advantage. The hybrid (SIM-3) tests whether combining both
mechanisms decisively separates from the individual approaches.

**Design 5 and M12 (Key Question 5):** Information contributes to M12 only if
the concentration gradient is strong enough for signal-readers to act on.
Design 5's prediction of M12 0.35-0.45 assumes a companion mechanism (partial
or biased refills) is also present. Under SIM-1's balanced full refills, the
same information system likely produces M12 0.15-0.25 because there is less
signal to read — open and AI lanes converge after each refill. The information
system amplifies a concentration mechanism; it does not create one.

**Design 2's fundamental problem:** The confirmed finding that "frequent balanced
refills make the reset problem WORSE" is the fatal issue. 5 rounds with 4 refill
events means 4 concentration resets. Even with 70% open-lane bias, the frequent
small refills produce many partial resets rather than few large ones. The round
boundary overhead is paid 4 times instead of 2, and each payment is a reset.
The 3-round structure's two refill events are already two concentration setbacks;
5-round designs multiply this without proportionally improving M3. Design 2 is
structurally at a disadvantage against 3-round designs with equivalent bias
strength.
