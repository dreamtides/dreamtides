# Design 1: Static AI Drafters (Level 0 Reactivity)

## Key Findings

- **Pool depletion math demands many AIs or multi-card picks.** V9's 12%
  contraction removes ~38 cards at pick 4 (pool=317). With 4-card packs, matching
  this requires 9-10 AIs each taking ~4 cards per round, or fewer AIs taking
  multiple cards. A "realistic" 3-5 AI table produces only 1-3% contraction --
  far below what M11 >= 3.0 requires. The central design challenge is bridging
  this gap without supplemental culling.

- **AI picks and V9 contraction pull in complementary directions.** V9 removes
  low-relevance cards from the player's perspective; AIs remove high-relevance
  cards from their own perspective. The net effect is similar -- off-archetype
  cards disappear from the pool -- but AI picks create directional depletion
  (specific archetypes thin out) rather than uniform concentration. This is
  narratively better ("other players took those cards") and potentially more
  efficient at creating lane signals.

- **Static AIs do create readable signals.** 17Lands data confirms that Arena's
  predetermined bots produce actionable lane signals. The key requirement is
  per-AI archetype consistency within a draft and composition variety across
  drafts. Signal reading with static AIs is "which lane is open this game?" -- a
  genuine puzzle that varies each run.

- **Pair-affinity AI preferences are required for M11.** An AI using only visible
  resonance to pick cannot distinguish Warriors from Sacrifice within Tide cards,
  capping M11 at ~2.1. AIs using pair-affinity scores replicate Hybrid B's
  discrimination and can reach M11 >= 3.0. This constraint is mathematical and
  architecture-independent.

- **Convergence ramp emerges naturally from static AIs.** As AIs deplete their
  home archetypes over 30 picks, the remaining pool increasingly consists of
  cards from non-AI archetypes. If the player's archetype has no competing AI (or
  a weak one), their cards accumulate proportionally -- creating the same
  convergence ramp V9 achieved through explicit contraction.

- **Game-to-game variety requires variable AI composition.** Each game should
  activate a random subset of 5-7 AIs from the 8 possible archetypes. The 1-3
  missing archetypes become naturally open lanes with strong signals. Fixing all
  8 AIs every game eliminates the "which lane is open?" puzzle.

---

## Three Algorithm Proposals

### Proposal A: "Full Table" (7 AIs, 5 cards each per round)

**Player description:** You draft alongside 7 AI opponents, each building a
different deck.

**Technical description:** 7 AIs, each assigned a unique archetype (the player
occupies the 8th slot implicitly). Each round, every AI picks 5 cards from the
pool using pair-affinity-weighted preferences before the player sees their
4-card pack. AIs pick in random order. Total removal: 35 AI cards + 1 player
card per round = 36 cards per round = ~11% contraction at pick 4.

**AI drafter behavior:** Each AI scores every card as 0.7 * pair_affinity[own_archetype] + 0.3 * visible_resonance_match. AIs take their top 5 cards.
85% of picks follow this scoring; 15% are "imperfect" picks (highest raw power
regardless of archetype), preventing perfect predictability.

**Predicted metrics:** M3 ~2.55, M10 ~4.0, M11 ~3.1, M6 ~82%. The high AI
count produces strong contraction but every archetype is contested, narrowing the
player's open-lane advantage. M10 suffers because all lanes have a competing AI.

### Proposal B: "Open Table" (5 random AIs, 4 cards each per round)

**Player description:** Each game, 5 of 8 possible AI opponents sit down to
draft -- figure out which 3 archetypes are uncontested.

**Technical description:** 5 AIs randomly selected from 8 archetypes each game.
Each AI picks 4 cards per round using pair-affinity preferences. Total removal:
20 AI cards + 1 player card = 21 cards per round = ~6.6% contraction at pick 4.
To compensate for lower contraction, add a "market culling" step: after AI picks,
remove the 12 lowest-power remaining cards (framed as "cards nobody wanted").
Total effective removal: ~33 cards per round = ~10.4%.

**AI drafter behavior:** Each AI scores cards as pair_affinity[own_archetype].
Top 3 picks are best-scoring; 4th pick is highest raw power (simulating
power-drafting). No additional noise.

**Predicted metrics:** M3 ~2.65, M10 ~3.2, M11 ~3.2, M6 ~85%. The 3 open
lanes provide strong convergence for signal-reading players. Market culling
supplements AI-driven depletion to reach V9-equivalent contraction. Signal
reading is meaningful: the player must identify which 3 of 8 archetypes are
open.

### Proposal C: "Rivalry Draft" (6 AIs with aggression tiers, 3-6 cards each)

**Player description:** Six AI drafters compete for cards -- some are aggressive
collectors, others are casual pickers.

**Technical description:** 6 AIs selected from 8 archetypes. 2 "aggressive" AIs
take 6 cards per round (deeply depleting their archetype). 2 "moderate" AIs take
4 cards per round. 2 "casual" AIs take 3 cards per round. Total removal: 12 + 8
+ 6 + 1 player = 27 cards per round = ~8.5% contraction. Supplemental culling
of 8 lowest-power cards per round brings total to ~11%.

**AI drafter behavior:** Aggressive AIs use strict pair-affinity scoring.
Moderate AIs use 80% affinity / 20% power blend. Casual AIs use 60% affinity /
40% power blend, often grabbing generics. Aggression tiers are assigned randomly
each game.

**Predicted metrics:** M3 ~2.60, M10 ~3.5, M11 ~3.15, M6 ~84%. Variable
aggression creates natural lane width: aggressive AI lanes are nearly stripped
(strong avoid signal), casual AI lanes remain wide (soft signal). Signal reading
has texture -- not just "is this lane open?" but "how contested is it?"

---

## Champion Selection

**Champion: Proposal B, "Open Table" (5 random AIs, 4 cards each, market
culling).**

Justification: Proposal B best balances the static AI model's strengths and
weaknesses.

1. **Signal reading is most meaningful with 3 open lanes.** With 5 of 8
   archetypes contested, the player has a genuine 3-way choice among uncontested
   archetypes. This is neither trivially easy (one obvious lane) nor
   frustratingly narrow (no open lane).

2. **Market culling is narratively clean.** "Cards nobody wanted get discarded"
   is a natural draft table mechanic. It supplements AI picks to reach V9-level
   contraction without requiring 9-10 AIs (which would feel crowded and leave no
   open lanes).

3. **Variable AI composition drives M7/M8.** With C(8,5) = 56 possible AI
   configurations, game-to-game variety is high. No archetype is always open or
   always contested.

4. **Predicted metrics match or exceed V9 Hybrid B.** M3 ~2.65 and M11 ~3.2 are
   competitive with V9 (M3=2.70, M11=3.25) while offering a superior player
   narrative.

---

## Champion Deep-Dive: Open Table

### How It Works

**Before the draft starts:** The system randomly selects 5 of 8 archetypes and
creates one AI drafter per selected archetype. Each AI receives its archetype's
pair-affinity scoring function (the same 8-bit two-float encoding from V9 Hybrid
B). The AI pre-computes a pick order for every card in the pool, sorted by
pair_affinity[own_archetype] descending, with the 4th pick per round being the
highest raw power card remaining (simulating an occasional power pick).

**Each round (30 rounds total):**
1. Each AI drafts 4 cards from the shared pool using its preference order.
2. Market culling removes the 12 lowest-power cards remaining in the pool.
3. The player is shown a 4-card pack drawn randomly from the surviving pool.
4. The player picks 1 card; the other 3 return to the pool.

**What the player sees:** A 4-card pack each round. Over time, the player
notices patterns: Tide cards appear frequently (no Tide-primary AIs are
present), while Ember cards are scarce (two Ember-primary AIs are aggressively
drafting them). The player infers that Warriors or Sacrifice is open and commits
accordingly.

**What the AIs do:** Each AI mechanically follows its pre-computed pick order.
The Warriors AI takes the highest warriors_affinity cards first, depleting the
Tide pool of Warriors-home cards. The Storm AI takes the highest storm_affinity
Ember cards, thinning Storm. These picks are entirely predetermined -- the
player's actions have zero influence.

### Example Draft (Player as Signal Reader)

**Setup:** AIs assigned to Flash, Blink, Storm, Self-Mill, Sacrifice. Open
lanes: Self-Discard, Warriors, Ramp.

- **Picks 1-4:** Player sees mixed packs. Notes that Tide and Zephyr cards
  appear often; Ember cards are scarce (3 Ember AIs: Flash secondary, Blink
  primary, Storm primary). Player tentatively leans toward Tide (Warriors or
  Sacrifice).
- **Picks 5-7:** Player notices Sacrifice-affinity Tide cards are thinning
  (Sacrifice AI is active) but Warriors-affinity Tide cards flow freely. Player
  commits to Warriors.
- **Picks 8-15:** With Warriors uncontested, the player sees 2-3 S/A Warriors
  cards per pack. The Sacrifice AI has taken many Sacrifice-home Tide cards,
  leaving Warriors cards concentrated. Market culling has removed low-power
  generics, further concentrating quality.
- **Picks 16-25:** Pool is now heavily concentrated in Warriors/Ramp/Self-Discard
  cards (the three open lanes). Player sees 3+ S/A Warriors cards per pack.
  Occasional Ramp splash cards appear as natural off-archetype options.
- **Picks 26-30:** Pool is small (~40-60 cards). Nearly every pack contains
  strong Warriors cards. Player fills remaining deck slots.

### Failure Modes

1. **Player drafts into a contested lane.** If the player commits to Storm (an
   active AI lane), they compete directly with the Storm AI for cards. Packs
   contain fewer Storm cards, and the AI has already taken the best ones. Result:
   a weaker but playable Storm deck (M3 ~1.5-1.8 instead of ~2.6). Not
   unplayable, but noticeably worse. This is the correct incentive -- reading
   signals matters.

2. **Market culling removes desirable cards.** The 12 lowest-power cards culled
   each round are bottom-tier regardless of archetype, so they are unlikely to
   be cards the player wanted. Risk: a low-power card with high archetype
   synergy gets culled. Mitigation: culling uses raw power, not archetype
   fitness, so synergistic low-power cards are rare casualties.

3. **Player cannot distinguish open lanes early.** In picks 1-3, the pool has
   only lost ~60-90 cards (AI picks + culling). All archetypes still have cards
   available. The signal is weak. This is acceptable: V9 also converges late
   (M5=9.6), and the player is expected to stay open for picks 1-5.

4. **Predictability across runs.** After many runs, the player learns that 3
   lanes are always open and drafts into whichever they identify first. The
   puzzle becomes rote. Mitigation: 56 possible AI compositions, variable
   aggression within AIs (4th pick is power-based), and the player still must
   identify *which* 3 are open this game.

---

## AI Drafter Specification

| Parameter | Value |
|-----------|-------|
| Number of AIs | 5 per game (randomly selected from 8) |
| Archetype assignment | Each AI gets one unique archetype |
| Cards per AI per round | 4 |
| Pick logic | Top 3: highest pair_affinity[own_archetype]; 4th: highest raw power |
| Reactivity level | Level 0 (fully predetermined) |
| Aggression | Uniform across all 5 AIs (each takes exactly 4 cards) |
| Market culling | 12 lowest-power cards removed per round after AI picks |
| AI pick order | Random (order among AIs is randomized per round but does not affect outcomes since picks are predetermined) |
| Pool minimum | Stop culling when pool < 40 cards |
| Hidden metadata per card | 8 bits (two 4-bit pair-affinity floats, identical to V9 Hybrid B) |
| Pack construction | 4 cards drawn randomly from post-AI, post-culling pool |
| Player picks returned | 3 unpicked cards return to pool |

### AI Preference Function

```
score(card, ai_archetype) =
    if card is generic: 0.3 * card.power / max_power
    else: card.pair_affinity[ai_archetype]
```

For the power pick (4th card per round):
```
score(card) = card.power / max_power
```

### Archetype Frequency Control (M8)

With 5-of-8 random selection, each archetype appears as an AI in 5/8 = 62.5% of
games and is absent (open lane) in 37.5% of games. Across many games, every
archetype is open with equal probability. The player's archetype frequency (M8)
depends on how often they draft each archetype, which depends on signal reading
skill, but no archetype is structurally advantaged or disadvantaged.

---

## Post-Critique Revision

### 1. Addressing "Too Generous"

The critique's concern is valid: identifying any of three open lanes gives a
strong deck with minimal effort, potentially making signal reading trivially
rewarding rather than meaningfully skillful.

The honest response is that the Open Table does not eliminate the skill axis --
it relocates it. With 3 open lanes, the player still faces a genuine puzzle:
which three archetypes are uncontested this game, and which of those three fits
the cards they've already seen? The lane-identification step is not automatic.
In picks 1-4, all three open lanes look equally promising; the player must read
which lane's best cards are flowing most freely and commit before the pool
concentrates too heavily.

That said, the critique identifies a real floor problem: even a player who
identifies the correct open lane by pick 8 (late) will still draft a strong
deck. The penalty for slow signal reading is low because all three open lanes
converge to S/A quality by pick 15.

A concrete mitigation that preserves the design: reduce market culling from 12
to 8 cards per round, and add modest resonance pressure in open lanes by
ensuring at least one of the five AIs drafts a resonance that overlaps with each
open archetype. This narrows the quality gap between contested and open lanes
without eliminating it, ensuring the player's signal-reading speed still
matters. The simulation baseline should test both culling rates to quantify the
difference.

### 2. Market Culling Is V9 Contraction

The critique is correct, and I should have said it plainly in the original
design: market culling is pool contraction re-labeled. Removing 12 low-power
cards per round is mathematically equivalent to V9 removing the bottom 12% of
the pool by relevance. The AI drafter layer adds two genuinely novel things --
directional depletion (specific archetypes thin differentially) and a player
narrative ("someone else drafted those") -- but the contraction engine
underneath is the same mechanism V9 used.

This reframing has a useful implication: the culling rate should be set to
whatever value, combined with AI picks, closes the contraction gap to V9 levels.
It is not a secondary mechanism or an awkward patch; it is the primary
contraction source, with AI picks layered on top to provide signals. Calling it
"market culling" in internal documentation obscures this. The simulation should
report AI removal and culling removal separately so we can see how much each
contributes to M3 and M11.

### 3. Parameter Adjustments

Based on the critique's flags and the cross-proposal analysis, I recommend two
parameter changes before simulation:

- **Market culling: 10 cards per round (down from 12).** This reduces total
  contraction from ~10.4% to ~9.7% at pick 4, slightly below V9's 12%. The
  residual gap tests whether 3 open lanes produce enough natural concentration
  to compensate for lower mechanical contraction. If M11 falls short, culling
  can be increased incrementally.
- **4th pick power-draft: retain as specified.** The critique did not flag this
  parameter, and it provides verisimilitude without adding tuning complexity.
  One power pick per AI per round (5 total) also introduces mild cross-archetype
  noise, which prevents open-lane pools from becoming monolithic.

No changes to AI count (5), cards per AI per round (4), or archetype assignment
logic. These parameters drove the design's top rankings on signal reading and
"not on rails" -- they should not be adjusted without simulation evidence.

### 4. Hybrid Endorsement

Of the two hybrids, Hybrid X (Open Table + D3 saturation) is the more
interesting extension and I tentatively endorse it over Hybrid Y.

The saturation mechanic -- where AIs in active lanes ease off after accumulating
12 archetype cards -- is exactly the kind of natural convergence ramp the Open
Table lacks. The current design produces convergence through pool depletion
alone; an AI that is "satisfied" and shifts to generic picks provides a second
convergence pathway that is more gradual and realistic. This would give the
player a readable mid-draft signal: "the Storm AI seems to have slowed down on
Ember cards" -- an observation that rewards attentive play without requiring the
player to understand the implementation.

Hybrid Y (Open Table + D4 escalation) is structurally sound but adds complexity
without a clear benefit over the baseline. Escalating pick rates with only 5
AIs means early picks have very low AI activity (10 cards removed per round in
picks 1-5, per the hybrid's specification), which could make early signal
reading genuinely difficult -- the critique already noted that D4's escalation
produces signals that arrive too late. Adding escalation to the Open Table
inherits this weakness.

My recommended simulation priority: D1 baseline first (Slot 1 as specified),
Hybrid X second, with Hybrid Y deferred unless the baseline and Hybrid X both
fail on M10.
