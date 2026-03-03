# Research: Hidden Information in Procedural Content Generation

## How Successful Games Use Hidden Weighting — and What Makes It Feel Fair

______________________________________________________________________

## 1. The Central Tension

Hidden algorithmic manipulation in games exists on a spectrum from "design
magic" to "the game is lying to me." The line between them is not about how much
the algorithm does in secret — it is about whether the visible signals given to
the player are honest representations of real forces in the game.

The key finding from game design research: **hidden systems feel fair when the
visible layer accurately predicts the hidden layer's behavior.** They feel
deceptive when the visible layer is decorative — present to create an impression
that the algorithm then ignores.

______________________________________________________________________

## 2. Roguelike Deckbuilders: How Hidden Weighting Works

### Slay the Spire: Hidden Rarity Weighting

Slay the Spire uses a well-documented hidden weighting system for card rewards.
The base rarity chances are offset by a tracker that starts at -5% and increases
by 1% every time a common card appears, resetting when a rare appears (maximum
offset: +40%). The player never sees this counter. They see only the card
rewards.

**Why this feels fair:** The visible layer (rarity) is an accurate signal. When
players receive three commons in a row, they are genuinely more likely to see a
rare next. The hidden counter is a faithful implementation of the visible
promise: "rares are uncommon, but the game won't starve you of them." A player
who learned the exact formula would not feel deceived — they would feel the game
was working as advertised. The system also uses correlated RNG across game
elements, though this was unintentional (an accidental correlation between card
rarity and other random events), and when discovered by the community, players
felt it was an exploit rather than a feature — a signal that unintended hidden
correlations damage trust even when the effect is neutral.

**Elite encounters** offer different reward tables (more uncommons and rares).
Boss rewards are exclusively rares. These are visible signals (elite = harder =
better rewards) backed by the algorithm, so players calibrate accurately.

### Monster Train: Clan Synergy as Implicit Weighting

Monster Train's reward system is designed around dual-clan combinations, with 15
possible pairings creating different strategic identities. The game does not use
explicit visible weighting symbols for most cards. Instead, card rewards are
tilted toward cards relevant to the player's chosen clans through a
pool-construction approach: each run's card pool is populated primarily from the
two chosen clans. The "hidden" work is done at pool construction time, not at
moment-of-offer.

**Why this feels fair:** Players understand they chose two clans and expect to
see cards from those clans. The mechanism (pool restriction) matches the visible
signal (clan identity). There is no gap between what the player thinks is
happening and what the algorithm is doing.

### Inscryption: Hidden Manipulation as Narrative Device

Inscryption is a special case: it deliberately violates player expectations as a
narrative mechanic. The game's antagonist (Leshy) manipulates card rewards and
encounters in ways players cannot fully model. This hidden manipulation feels
intentional and diegetically justified — the opponent is *supposed* to be
cheating.

**The lesson for V9 is the inverse:** Inscryption can get away with opaque
hidden manipulation because it is thematically justified. A draft system that
cannot offer this justification must be held to a stricter honesty standard. The
player must be able to construct a mental model of the system that is
approximately correct.

______________________________________________________________________

## 3. MTG Arena Bot Drafting: A Case Study in Hidden Preferences Gone Wrong

MTG Arena's Quick Draft format pairs one human player against seven algorithmic
bots, all drafting from the same packs simultaneously. The bots have hidden
preference orderings for cards that players cannot inspect. Several dynamics
emerge:

**The "universally rare-drafts" bias:** Bots pick rares above their playable
value, as a deliberate WotC decision to limit rare circulation and support pack
sales. Players discovered this and felt it was exploitable rather than fair —
because the hidden preference (pick all rares) is disconnected from the visible
signal (card strength/archetype fit). The visible signal says "pick what fits
your deck"; the hidden algorithm says "always take the rare." This creates an
honest/hidden gap.

**Color undervaluation errors:** In some formats, bots systematically undervalue
certain colors (e.g., black and green in Throne of Eldraine). Expert players
exploited this by drafting those colors reliably, producing strong decks by
picking up quality cards as late picks. When this became widely documented, the
community's reaction was not "the game is lying to me" but rather "the bots are
bad at their job" — a different kind of trust failure, but equally damaging to
the draft experience.

**Signal reliability:** Research by 17lands showed that bot signals are actually
stronger (more consistent) than human signals in some formats. The bots reliably
pass certain cards late, which provides legible information. But this
consistency also means the drafting metagame stabilizes into a known exploit
state — the exact opposite of the variety and discovery V9 values.

**The core MTG Arena lesson:** Hidden preferences only feel acceptable when they
are aligned with the visible signals the game is sending. A bot that secretly
overvalues rares while pretending to draft for deck quality creates a felt
dishonesty even when players cannot articulate exactly what is wrong. The gap
between "what the visible system implies the algorithm values" and "what the
algorithm actually values" is the source of the trust failure.

______________________________________________________________________

## 4. Dynamic Difficulty Adjustment: The Closest Analog to V9

Dynamic Difficulty Adjustment (DDA) in games like Resident Evil 4 and Left 4
Dead is the closest parallel to V9's hidden pack-construction algorithm. These
systems secretly adjust difficulty parameters in response to player performance
— directly analogous to V9's algorithm secretly adjusting pack composition in
response to the player's draft choices.

**Resident Evil 4** (original) was one of the first games to popularize hidden
adaptive difficulty. If the player was performing well (accurate shots, few hits
taken), enemies became more aggressive. If the player was struggling, enemies
eased off. Capcom never announced this feature. Players discovered it years
later.

Community reaction to this discovery was split:

- Players who felt the game was well-designed responded with "that explains why
  it always felt well-balanced — the game was reading me." Satisfaction.
- Players who felt ownership over their demonstrated skill responded with "the
  difficulty I experienced wasn't real." Mild betrayal.

**The factor that determines which reaction dominates:** Whether the player's
visible signals were honest. In RE4, the player's performance genuinely
determined difficulty — the visible signal (enemies respond to how well you're
doing) was accurate. The hidden system was a mechanical implementation of a
correct model. Players who felt satisfied had constructed an accurate mental
model; the hidden layer only surprised them in its precision, not its direction.

**Left 4 Dead's AI Director** took a different approach: Valve publicized the
Director openly, describing it as a "film director" that paces tension and
relief. By making the system's existence and purpose visible (even if the
moment-to-moment decisions were hidden), Valve created a situation where players
experienced the algorithm as a collaborator rather than a manipulator. When a
player says "the Director is really throwing waves at us tonight," they are
attributing agency to a system they understand — and feel good about it.

**V9 implication:** There is a design choice between "keep the algorithm fully
hidden" and "describe the algorithm's purpose openly, even if its decisions are
hidden." Left 4 Dead's approach — announcing what the system does while hiding
how it does it in the moment — produced better player satisfaction than RE4's
fully silent approach.

______________________________________________________________________

## 5. What Makes Hidden Systems Feel Honest: Design Patterns

Synthesizing across these examples, five patterns consistently produce "honest
hidden system" reactions rather than "the game is lying to me" reactions:

### Pattern 1: The Hidden Layer Is a Faithful Implementation of the Visible Signal

The most reliable pattern. When the algorithm does exactly what the visible
signals imply it should do — just more precisely or efficiently than the player
could implement manually — discovery triggers satisfaction rather than betrayal.

- **Slay the Spire rarity counter:** Visible signal is "rares are uncommon but
  not starved." Hidden counter implements this precisely. Satisfying on
  discovery.
- **RE4 adaptive difficulty:** Visible signal is "enemies respond to how well
  you play." Hidden system implements this literally. Satisfying on discovery.
- **Bad version (MTG Arena rare drafting):** Visible signal is "draft for deck
  quality." Hidden preference is "always take the rare." Discovery produces
  sense of manipulation.

**For V9:** If the hidden metadata is genuinely derived from each card's
mechanical properties — not assigned arbitrarily to optimize algorithm
performance — then a player who discovered the metadata would likely respond:
"Yes, that card really does fit Warriors better than Ramp. The algorithm got it
right." This is the satisfying pattern.

### Pattern 2: The Outcome Is What the Visible System Promised

Players care more about whether the experience delivered on its visible promises
than about whether the mechanism was complex. If the player commits to Tide and
their packs improve, the visible promise ("commit to a resonance and your packs
improve") was kept — regardless of whether the improvement came from visible
symbols, hidden metadata, or pool contraction.

Narrative Gravity in V8 was rated 7.9/10 player experience precisely because it
delivered on its promise. Players never needed to know about pool contraction.

**For V9:** The visible promise of the resonance system must be kept: "drafting
Tide cards should produce better packs for Tide strategies." If the algorithm
achieves this via hidden metadata while the player attributes the improvement to
their visible-symbol choices, the promise is kept. No trust violation.

### Pattern 3: Announcing Mechanism Type, Not Mechanism Parameters

Left 4 Dead's explicit Director announcement shows that describing *what kind of
thing* the algorithm is — even without revealing its parameters — satisfies
players' need for a coherent mental model without reducing the system's
effectiveness.

A V9 equivalent: "As you draft, the game tracks what you're building and tries
to send you more of it" is a true, complete, non-deceptive description of pack
construction with hidden metadata. No parameters need be disclosed. The player
understands the system type and can form an accurate model of what to expect.

This is categorically different from a system where the description would be
misleading if given. If the algorithm were secretly assigning arbitrary
archetype tags to optimize performance regardless of card mechanics, the honest
description would have to say something false or evasive.

### Pattern 4: Hidden Information Derived from Genuine Card Properties

The "design integrity spectrum" in the V9 plan maps directly onto player trust.
At the honest end: hidden metadata reflects real mechanical properties of the
card (affinity scores that any player would agree are fair assessments). At the
dishonest end: tags assigned purely for algorithmic performance, disconnected
from card identity.

Research on game transparency consistently finds that hidden systems based on
real game-world properties are tolerated or celebrated, while hidden systems
based on arbitrary designer fiat are resented when discovered. The test is:
"Would a player who discovered this metadata agree that it's accurate?" If yes,
the system is defensible.

### Pattern 5: Visible Signals Must Predict Outcomes Reliably

This is the most critical pattern for V9. If the player commits to Tide and sees
a high ratio of Tide-resonance cards, they form a causal model: "I'm drafting
Tide, so I'm getting Tide cards." This model must be approximately correct. If
the algorithm is actually doing all the targeting work through hidden metadata
and the visible Tide symbols are decorative, the player's causal model is wrong
— and when they try to apply it (e.g., splash off-color), it will fail in ways
that feel random and arbitrary.

The V9 V1 metric (visible symbol influence) is precisely the right measure of
this. If V1 drops below ~40-50%, the visible resonance system is no longer the
primary driver of outcomes, and the player's causal model becomes systematically
wrong. This is the failure mode to avoid.

______________________________________________________________________

## 6. The Threshold Problem: How Much Hidden Manipulation Is Detectable?

Research on DDA suggests players are often unaware of active hidden manipulation
during play. Studies show that players don't notice DDA implementation in real
time in most cases. However, there are two separate thresholds:

**Detection threshold during play:** High. Players rarely notice pack-by-pack
targeting unless it creates jarring patterns (e.g., every pack has exactly 2
on-archetype cards, visible as too perfect).

**Detection threshold via analysis:** Low. Any player who examines their draft
data across multiple runs will notice patterns. MTG Arena's bot preferences were
reverse- engineered within weeks of Quick Draft launching. Slay the Spire's
rarity counter was documented in community wikis within months.

**The practical implication:** V9 should assume the hidden system *will be
discovered* by some players. The design question is not "can we keep this
hidden?" but "when discovered, will players feel it is fair?" Design for
post-discovery satisfaction, not for pre-discovery concealment.

______________________________________________________________________

## 7. Specific Recommendations for V9

### On the Visible-Hidden Split

The sweet spot, based on the patterns above:

1. **Visible resonance as genuine signal** (not decoration): The ~10% visible
   dual- resonance cards should behave as reliable anchors — when a player picks
   a (Tide, Zephyr) card, their subsequent packs should noticeably improve for
   Warriors. This visible cause-and-effect is what makes the system feel real.

2. **Hidden metadata derived from card mechanics**: Archetype affinity scores or
   archetype tags that any game-literate player would endorse are the most
   defensible. "This card has (Tide) symbol and cares about creatures dying — of
   course it's tagged as Warriors/Sacrifice." Arbitrary tags are the least
   defensible.

3. **Announce the system type, not the parameters**: The one-sentence
   player-facing description should accurately describe what the algorithm does
   ("your picks shape future packs"). The hidden layer is the precise mechanism,
   not the purpose.

### On Minimum Viable Hidden Information

The MTG Arena case shows that even minimal hidden preferences (a simple
valuation ordering) can feel manipulative when they misalign with visible
signals. More hidden information that accurately reflects card properties is
more defensible than less hidden information that is arbitrary.

This suggests: a simple 3-bit archetype tag that genuinely reflects card
mechanics (a card's best-fit archetype) is more honest than an 8-float affinity
matrix assigned to maximize algorithm performance metrics. The form matters less
than the derivation.

### On Visible Resonance Salience

The V1 metric must stay above 40-50% for the visible system to feel real to
players. Below this threshold, players applying the visible signal as their
primary guide will encounter outcomes that feel unpredictable — even though the
algorithm is working correctly. Their model will be wrong too often to reinforce
the right behavior.

The practical test: a player who drafts Tide cards but ignores hidden metadata
should still see their packs improving toward Tide/Warriors/Sacrifice. The
visible symbols should do most of the causal work. Hidden metadata should be the
refinement that distinguishes Warriors from Sacrifice within the Tide-committed
player's pool.

______________________________________________________________________

## 8. Summary: The Honest Hidden System Checklist

For V9 algorithm designers, a hidden system is defensible if:

- [ ] The visible layer accurately predicts the direction of outcomes
  (committing to Tide means more Tide-relevant cards — visible symbols cause
  this, not just correlate with it).
- [ ] Hidden metadata reflects genuine card properties, not arbitrary
  optimization labels. A player who examined the metadata would endorse it as
  fair.
- [ ] The visible description of the system type is accurate, even if parameters
  are unstated. "The game tracks what you draft and sends you more of it" is
  true.
- [ ] V1 (visible symbol influence) >= 40-50%. The visible system must be doing
  a majority of the targeting work.
- [ ] Discovery produces "I see, it's more precise than I thought" rather than
  "the symbols are fake and the algorithm does whatever it wants."
- [ ] The number of hidden bits per card is minimized. More hidden information
  creates more surface area for player resentment.

The ultimate test: describe the hidden system to a skeptical player who has been
drafting well using the visible system. If their reaction is "yes, that makes
sense and doesn't change how I play," the system is honest. If their reaction is
"so my choices didn't matter," the system has failed.
