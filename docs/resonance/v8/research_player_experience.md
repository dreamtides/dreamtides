# Research Agent C: Player Experience Analysis

## The Question

What makes a draft feel good or bad, independent of aggregate S/A numbers? V7
optimized for average pack quality (M3) and variance (M9) but never asked
whether the *pattern* of delivery creates a satisfying player experience. V8
elevates player experience to a first-class design constraint alongside the
metrics. This analysis establishes the experiential criteria that any V8
algorithm must satisfy.

______________________________________________________________________

## 1. The Alternation Problem: Why Surge+Floor Feels Like a Slot Machine

V7's recommended algorithm (Surge+Floor T=3) produces a characteristic rhythm:
surge packs deliver ~2.5 S/A, floor packs deliver ~1.2 S/A, and the two
alternate roughly every 1.5--2 picks for committed players. The aggregate
average (1.85) is healthy, but the *felt experience* is one of dramatic
oscillation.

This matters because of loss aversion -- one of the most robust findings in
behavioral psychology. Losses are experienced roughly twice as intensely as
equivalent gains. A floor pack that delivers 1.2 S/A (0.65 below average)
produces a negative reaction approximately twice as strong as the positive
reaction to a surge pack at 2.5 S/A (0.65 above average). The net emotional
valence of the alternation is *negative* even though the arithmetic average is
positive.

The alternation also creates a **predictability trap**. Once committed players
understand the surge cycle (which V7 recommends making visible via token
counters), they can predict which packs will be good and which will be bad.
Predictable bad experiences are worse than unpredictable ones: the player
*knows* the next floor pack will be disappointing and dreads it. This is the
opposite of the anticipation that makes surge packs rewarding. Predictable
rewards create anticipation; predictable disappointments create dread.

Contrast this with natural variance -- the randomness inherent in drawing from a
card pool. When pack quality varies because of random draw, the player
attributes both good and bad packs to luck. When it varies because of an
algorithm's explicit cycle, the player attributes bad packs to the *system*.
"The game gave me a bad pack on purpose" feels worse than "I got unlucky."

**Key finding:** The surge/floor alternation creates a bimodal quality
distribution with a valley in the middle. Algorithms should aim for a unimodal
distribution -- a central tendency with natural spread -- rather than two
distinct pack types.

______________________________________________________________________

## 2. How Much Variance Is Desirable?

Some variance in pack quality is not just acceptable but *necessary*. M9 (stddev
\>= 0.8) exists because perfectly uniform packs feel sterile and remove the
excitement of discovery. The question is what *kind* of variance feels good.

**Desirable variance properties:**

- **Organic-feeling:** Arises from the random draw process, not from explicit
  algorithmic switching between modes.
- **Positively skewed:** Occasional high-quality packs feel like windfalls.
  Occasional low-quality packs feel like bad luck. But consistent low-quality
  packs feel like systemic failure.
- **Non-periodic:** Variance that follows a predictable cycle (surge every N
  picks) becomes mechanical. Variance with irregular timing feels natural.
- **Bounded below:** The worst packs should still contain at least one card the
  player actively wants. A pack of four C/F-tier cards is a "dead pack" and
  should be structurally impossible for committed players.

**The 7 Wonders principle:** The board game 7 Wonders structures its draft into
three ages of ascending card power. Players experience progression *within* each
age and a power jump *between* ages. This works because the quality ramp is
monotonic at a macro level even as individual packs vary at a micro level.
Dreamtides' 30-pick draft is long enough to create a similar macro arc: picks
1--5 should feel exploratory, 6--15 should feel like building momentum, and
16--30 should feel like refinement with occasional exciting finds.

**Recommended variance targets:**

- StdDev >= 0.8 (preserving M9 from V7) to prevent sterility
- No more than 2 consecutive packs below 1.5 S/A (the M10 target) to prevent
  dread streaks
- The 10th percentile pack (worst 1 in 10) should deliver >= 1.0 S/A -- at least
  one clearly desirable card
- The 90th percentile pack should not exceed 3.5 S/A, to prevent the "rest of
  the draft doesn't matter" feeling

______________________________________________________________________

## 3. The Minimum Per-Pack Floor: Eliminating Dead Packs

A "dead pack" -- one where the committed player sees zero S/A-tier cards -- is
the single most damaging experience in a draft. It triggers loss aversion, it
feels like punishment for the player's archetype commitment, and it undermines
trust in the system.

Under V7's Surge+Floor algorithm, floor packs deliver 1 resonance-targeted slot
(75% S/A precision under Moderate fitness) and 3 random slots (25% S/A each).
The probability of a floor pack delivering 0 S/A cards is:

P(0 S/A) = 0.25 * (0.75)^3 = 0.105 (about 10.5%)

One in ten floor packs is completely dead. Over a 24-pack post-commitment draft
(~12 floor packs), the player expects to see approximately 1.3 dead packs. This
is too many.

**Recommendation:** The minimum viable floor is 1.5 S/A per pack for committed
players (picks 6+). This means the algorithm must structurally guarantee at
least one S/A card in every post-commitment pack with high probability (>= 95%).
The most direct way to achieve this is ensuring at least 1.5 targeted slots per
pack on average (not just during surges), where "targeted" means drawn from a
pool with >= 75% S/A precision.

Algorithms that achieve a high average (e.g., 2.0 M3) but with a long tail of
dead packs will feel worse than algorithms that achieve a lower average (e.g.,
1.8 M3) with a guaranteed floor. **Consistency of the floor matters more than
height of the ceiling** for player satisfaction.

______________________________________________________________________

## 4. Transparency vs. Magic: The Explainability Question

V3--V7 required every algorithm to be describable in one sentence. V8 relaxes
this, asking whether a "magical" algorithm that the player cannot explain can
feel better than a transparent one. The answer depends on what the player needs
to understand.

**What the player must perceive (non-negotiable):**

1. "My packs are getting better as I commit to an archetype." The quality ramp
   must be felt, even if the mechanism is not understood.
2. "Drafting cards with the same resonance type improves my future packs." The
   cause-and-effect between drafting resonance-consistent cards and pack quality
   must be apparent.
3. "I have meaningful choices." Every pack must contain at least one card the
   player genuinely wants.

**What the player does NOT need to understand:**

1. The exact token counting mechanism.
2. How many slots are "targeted" vs. "random."
3. The precise threshold for triggering quality improvements.
4. Whether the algorithm uses surge cycles, continuous weighting, or some other
   mechanism.

**Hearthstone Arena's lesson:** Blizzard's bucket system grouped cards into
quality tiers so that each draft pick presented cards of similar power level.
The goal was to make every choice interesting rather than obvious. But the
system made decks feel too similar across runs, and the community rejected it.
When Blizzard removed buckets in 2019, they acknowledged that while the system
made individual picks more interesting, it sacrificed deck variety. More
recently, Hearthstone's 2025 Arena revamp introduced synergy-based starting
packages -- a form of guided convergence where the player's first four cards
synergize and guide future picks. This is an "on rails" start that players
welcomed because it felt like support rather than constraint.

**The key insight:** Transparency of *mechanism* is less important than
transparency of *feedback*. The player does not need to understand the
algorithm. They need to observe the result: "I drafted Tide cards, and now I'm
seeing more Tide cards." The feedback loop must be legible even if the mechanism
is not.

This opens design space for algorithms that would score poorly on
one-sentence-description tests but produce superior felt experiences: smooth
probability weighting, adaptive slot allocation, quality floors that scale with
commitment level. The algorithm can be complex internally as long as the
player's *experience* of it is simple: "my packs improve as I commit."

______________________________________________________________________

## 5. Pack Structure: Fixed vs. Variable, Slots vs. Pure Random

**Fixed vs. variable pack size:** Variable pack sizes (e.g., Expanding Echo
Packs from V7 Agent 5) provide a visible signal of progression -- "my packs are
literally getting bigger." This is intuitive and rewarding. However, it also
changes the cognitive load per pick (evaluating 5 cards is harder than 4) and
extends draft duration. For Dreamtides' 30-pick draft, a fixed pack size of 4 is
well-calibrated: small enough for quick evaluation, large enough for meaningful
choice. Variable pack sizes should only be considered if the quality ceiling of
fixed-size packs is insufficient.

**Structured slots vs. pure random:** Structured slots (e.g., "slot 1 is always
your top resonance") create a mental model the player can learn: "I always check
the first card because it's usually on-archetype." This is subtly satisfying --
it gives the player a shortcut through the evaluation process. However, if the
structured slot is always in the same position, it reduces the surprise of
discovery ("I already know the top-left card will be Tide"). A better approach
may be structured *probability* -- guaranteeing that at least one card matches
the player's resonance, without fixing which slot it occupies. The player
discovers the on-archetype card each time rather than knowing where to look.

**The roguelike deckbuilder standard:** Slay the Spire presents 3 card choices
per reward, all drawn randomly from the character's card pool. There is no
hidden bias toward the player's current strategy -- every offer is a uniformly
random draw. Players accept this because the pool itself is curated (only cards
from the chosen character class). The perceived quality of offers comes from
pool design, not offer manipulation. This suggests that a well- designed card
pool with strong resonance identity may reduce the need for aggressive
algorithmic intervention. If the Tide pool naturally contains cards that a
Warriors player wants, then even "random" draws from that pool feel targeted.

______________________________________________________________________

## 6. Lessons from Digital Card Game Drafts

**MTG Arena draft:** The 2024 shift to Play Boosters reduced commons and
increased rare/mythic variance per pack. Ranked drafts feel good when the
*format* supports clear archetypes with cross-archetype overlap -- the best
draft formats (like Murders at Karlov Manor) have strong color-pair identities
where many cards serve multiple archetypes. Weak formats (like early Bloomburrow
reception) have narrow archetypes where off-archetype cards feel dead. This
directly parallels Dreamtides' fitness problem: the feeling of a draft is
determined more by how many cards *could* be good for the player than by how
many the algorithm *presents*.

**Hearthstone Arena (post-bucket):** The 2025 Arena revamp starts players with
four synergistic cards, then presents normal draft picks. This "guided start"
functions like an archetype commitment mechanism -- the starter package biases
the entire draft without the player needing to understand the bias. Dreamtides
could learn from this: the first few picks matter disproportionately for
perception because they establish the player's mental model of what the draft is
"about."

**Monster Train's dual-clan system:** Players choose two monster clans before
drafting, then see cards from both clans throughout the run. The player *knows*
they will see both clans' cards and plans accordingly. The dual-clan choice is
the equivalent of Dreamtides' resonance pair -- it establishes what the player
will see. Monster Train does not hide or manipulate the card pool; it simply
constrains it through the initial choice. The felt experience is one of
consistent relevance: every card reward is from one of your two clans, so every
offer has potential. Dead offers are rare because the pool is pre- filtered to
relevance.

**Synthesized lesson:** The most satisfying digital card game drafts share a
common property: **a high base rate of relevance.** The player encounters cards
that could fit their strategy most of the time, not because of a hidden
algorithm, but because the pool or format is structured for it. Dreamtides'
resonance system should aim for this: a high enough natural S/A base rate that
the algorithm's job is *refinement*, not *rescue*.

______________________________________________________________________

## 7. When Does Convergence Feel Like Support vs. Constraint?

The V5 Pair-Escalation algorithm achieved 2.61 S/A but with 96.2% deck
concentration -- nearly every card in the final deck was S/A-tier for the
player's archetype. This was rejected for feeling "on rails." But Hearthstone's
2025 Arena revamp introduces an even more directive starting mechanism
(synergy-based starter packages) that players welcome. What distinguishes
supportive convergence from constraining convergence?

**Convergence feels like support when:**

- The player chose the direction. The system amplifies a decision the player
  already made rather than making the decision for them.
- Off-archetype options remain available. Even in a converged draft, the player
  sees 0.5--1.0 C/F-tier cards per pack and can choose to take them for
  strategic reasons (splash, hate-drafting, power level).
- The convergence is gradual. A slow ramp from 25% S/A (pick 1) to 50% S/A (pick
  10\) to 60% S/A (pick 20) feels like building momentum. An abrupt jump from 25%
  to 75% at pick 6 feels like the algorithm took over.
- Run-to-run variety persists. If convergence produces different decks on
  different runs (M7 < 40% overlap), it does not feel constraining.

**Convergence feels like constraint when:**

- The player has no meaningful off-ramp. If the algorithm locks onto an
  archetype and the player wants to pivot, resistance should be moderate (slower
  convergence) not absolute (locked slots).
- Every pack looks the same. If S/A cards dominate every post-commitment pack,
  the player stops discovering interesting cards and starts auto-piloting. This
  is the "sterile draft" problem.
- Deck composition is predetermined. If 90%+ of the deck is S/A-tier with no
  room for creative choices, the draft loses its roguelike character.

**The sweet spot for Dreamtides:** M6 (60--90% deck concentration) is the right
window. Within this range, the player drafts a coherent archetype deck (60%+)
with room for splash picks, power picks, and speculative picks (up to 40%). The
algorithm should make convergence *available* without making divergence
*impossible*.

______________________________________________________________________

## 8. Player Experience Criteria for V8 Algorithms

Based on this analysis, V8 algorithms should be evaluated against these
experiential criteria in addition to the quantitative metrics:

### Hard Requirements

1. **No dead packs.** For committed players (picks 6+), every pack must contain
   \>= 1 S/A card with >= 95% probability. Algorithms with > 5% dead pack rate
   fail this criterion regardless of M3 score.

2. **No dread streaks.** Maximum 2 consecutive packs below 1.5 S/A (the M10
   target). This prevents the "when will my next good pack come?" anxiety.

3. **Legible feedback loop.** The player must perceive that drafting
   on-resonance cards improves future packs. The mechanism need not be
   explained, but the result must be observable.

### Strong Preferences

4. **Unimodal quality distribution.** Pack quality should cluster around a
   central value with natural spread, not alternate between two distinct modes
   (surge/floor). A smooth Gaussian-like distribution is preferred over a
   bimodal one.

5. **Non-periodic variance.** Quality variation should feel random, not
   cyclical. If the player can predict which packs will be good and which bad
   based on pick number alone, the algorithm is too mechanical.

6. **Gradual quality ramp.** Average pack quality should increase smoothly from
   picks 1--5 (exploratory, ~1.0 S/A) through picks 6--15 (building, ~1.5--2.0
   S/A) to picks 16--30 (converged, ~2.0+ S/A). Sharp transitions undermine the
   progression feel.

7. **Floor consistency over ceiling height.** An algorithm that guarantees 1.5+
   S/A in every post-commitment pack but averages 1.8 is preferable to one that
   averages 2.0 but occasionally delivers 0 S/A.

### Acceptable Tradeoffs

08. **Explainability can be sacrificed for feel.** A complex algorithm that
    produces smooth, natural-feeling pack quality is preferable to a simple
    algorithm that produces jarring alternation.

09. **Moderate convergence is welcome.** 70--80% deck concentration in the M6
    range is the experiential sweet spot: enough coherence to feel like the
    draft is "working," enough openness to feel like the player is making
    genuine choices.

10. **Slightly lower M3 is acceptable for better distribution.** An algorithm
    with M3=1.8 and no dead packs is experientially superior to one with M3=2.0
    and a 10% dead pack rate.

______________________________________________________________________

## Summary

V7 treated pack quality as an aggregate statistic: maximize M3, maintain M9
variance. V8 must treat it as a distribution with shape properties that affect
player experience. The critical insights:

- **Loss aversion makes bad packs disproportionately impactful.** Bimodal
  distributions (surge/floor) create net-negative emotional experiences even
  with positive averages.
- **Floors matter more than ceilings.** Eliminating dead packs and dread streaks
  contributes more to satisfaction than maximizing peak pack quality.
- **Natural-feeling variance is a goal; periodic algorithmic variance is a
  failure mode.** The player should attribute quality variation to luck, not to
  the system's explicit cycling.
- **Transparency of feedback, not mechanism, is what matters.** The player needs
  to see "my packs are getting better" without needing to understand why. This
  opens space for internally complex algorithms.
- **The best drafts in existing games achieve high base relevance through pool
  design, not offer manipulation.** Dreamtides should invest in pool composition
  (ensuring the resonance-filtered pools naturally contain cards the player
  wants) alongside algorithmic improvements.

These criteria should inform every V8 algorithm proposal. An algorithm that
scores M3=2.0 but violates the experiential criteria described here is worse
than one that scores M3=1.8 and satisfies all of them.
