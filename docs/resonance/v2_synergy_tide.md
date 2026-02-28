# Tide Synergy Discovery Report -- Hidden Vectors and Cross-Resonance Bridges

## Premise

This document does not describe what Tide IS. That work is done. Instead, it
maps the game-state conditions that Tide creates as side effects of its normal
operation, identifies which of those conditions are currently unexploited, and
proposes the mechanical templates of connector cards that would exploit them.
Every synergy vector identified here must bridge Tide to at least one non-Tide
resonance.

---

## 1. Side Effect Inventory

Tide's core mechanics (draw, Foresee, Prevent, Materialized-draw bodies, event
recovery) produce the following game-state conditions as unintentional
byproducts:

### 1a. Void Growth from Foresee

Foresee sends unwanted cards to the void. A Tide player running Oracle of
Shifting Skies (Foresee 1 per event played) and Astral Navigators (Materialized:
Foresee 2) can send 4-8 cards to the void per game without ever deliberately
self-milling. This is recognized by the Undertow archetype but is NOT exploited
by any current Tide card in isolation -- Tide has no void-size-matters payoff of
its own.

**Current exploitation:** Undertow (Tide+Ruin) uses this. Nobody else does.

### 1b. Large Hand Size

Drawing cards without discarding is Tide's defining trait. A mid-game Tide
player routinely holds 6-8 cards where the opponent holds 2-4. This large hand
is used for optionality (having answers), but the hand SIZE itself is never
mechanically referenced by any Tide card. No card in the 222-card pool says "do
X for each card in your hand" or "if you have 6+ cards in hand."

**Current exploitation:** Zero. This is the single most obvious untapped
resource in Tide.

### 1c. Opponent Tempo Loss from Prevent

When Tide prevents a card, the opponent has spent their energy and their card
but gotten nothing. They are now behind on both cards AND energy. Tide benefits
from this implicitly (the threat is gone), but no card explicitly rewards the
act of preventing -- no "when you prevent a card" trigger exists in the pool.

**Current exploitation:** Zero. Prevent is purely defensive.

### 1d. Board Presence from Materialized-Draw Bodies

Tide plays characters like Frost Visionary, Keeper of Forgotten Light, and
Looming Oracle primarily for their entry effects. But those bodies remain on the
battlefield afterward with 1-2 spark each. A Tide player with 3-4 low-spark
draw bodies in play has a surprisingly wide board, just not a tall one. These
bodies are currently dead weight after their entry -- no Tide card cares about
board WIDTH.

**Current exploitation:** Minimal. Mirage flickers them (but that re-exploits
the Materialized trigger, not the board presence). Judgment triggers on those
bodies are mostly irrelevant because Tide characters rarely have Judgment
abilities.

### 1e. High Event Density in Void

Tide plays many events (Foresee spells, Prevents, draw spells). These events
go to the void after resolution. By mid-game, a Tide player's void is
disproportionately event-heavy compared to other resonances. Spirit of Smoldering
Echoes (+1 spark per event entering void) cares about this, but it is coded as
Ember/Tempest.

**Current exploitation:** Tempest uses event recovery (Whisper of the Past,
Archive of the Forgotten), but these retrieve events, they do not count them.

### 1f. Deck Thinning from Draw

Heavy draw reduces deck size. A Tide player who has drawn 10+ extra cards has
a significantly thinner deck, making future draws more consistently powerful.
Lumineth ("no cards in deck: you win") exists but is a niche alt-win, not a
systematic exploitation of deck thinning.

**Current exploitation:** Minimal. Lumineth exists but is Zephyr-coded and
situational.

### 1g. Information Advantage from Foresee

After Foresee, the Tide player knows what is on top of their deck. This
information is used implicitly (keeping the best card on top), but no card in
the pool rewards KNOWING what your top card is. Dreamborne Leviathan ("play
characters from top of deck") is the closest, but it is Zephyr-coded Spirit
Animal infrastructure.

**Current exploitation:** Dreamborne Leviathan partially, but it is not a
Tide card.

### 1h. Energy Unspent from Holding Up Prevent

Tide players routinely pass their main phase with energy open, threatening
Prevent. If the opponent does not play into the Prevent, that energy goes
unused. No card currently rewards "energy you did not spend during your main
phase" or converts unspent Prevent energy into value at end of turn.

**Current exploitation:** Zero.

### 1i. Opponent's Void is Empty (from Prevent)

Prevented cards go to the opponent's void (unless Ripple Through Reality puts
them on top of deck). But the opponent's void grows slowly because their cards
keep getting countered before they resolve. This means the opponent cannot
leverage their OWN void for Ruin strategies. This is a negative side effect
for opponent Ruin decks -- but no Tide card currently leverages the
"opponent's void is small" condition.

**Current exploitation:** Soulflame Predator (banish opponent's void) works
in the opposite direction. Ruin Scavenger profits from opponent void cards. No
card punishes a SMALL opponent void.

---

## 2. Cross-Resonance Bridge Opportunities

Existing cards in non-Tide resonances that would become surprisingly powerful in
Tide archetypes with the right connector card:

### 2a. Spirit of Smoldering Echoes (Ember, 4 cost, spark = events in void)

Currently Tempest-coded. But a pure Tide deck that plays 6-8 events and
Foresees aggressively can have 10+ events in void without ever trying to
"storm." This Ancient gains spark passively in Tide. The bridge: if a card
existed that let you COUNT events in void for a non-damage effect (e.g., draw
cards equal to events in void / 3), Spirit of Smoldering Echoes would become
a Depths finisher -- a control body that grows while you Prevent and draw.

**Surprising interaction:** Depths (Tide+Stone) currently lacks a finisher
whose spark scales with the control game plan. Abomination of Memory
(Ruin-coded, spark = total void count) does this for Undertow. Spirit of
Smoldering Echoes does it for event-heavy control but is Ember-coded, making
it a Tempest/Depths bridge that nobody has identified.

### 2b. Starcatcher (Ember, 4 cost, "when you play an event, gain 1 energy")

Depths plays many cheap Prevent events (9 Prevents, most at 1-2 cost). With
Starcatcher in play, each Prevent is energy-neutral or energy-positive. This
transforms Depths from "choose: develop or hold Prevent" to "hold Prevent AND
develop." Currently classified as Tempest Core, but the mechanical interaction
with Depths' Prevent density is arguably stronger than with Tempest's storm
chains.

**Surprising interaction:** A Depths drafter who picks up Starcatcher can
completely eliminate the core tension of the archetype (reactive vs. proactive).
This is a sleeper bridge card.

### 2c. Conduit of Resonance (Zephyr, 5 cost, "when you materialize a character,
trigger the Judgment ability of each ally")

Currently Basalt core. But Tide has characters with Judgment: Draw triggers (The
Calling Night, The Waking Titan, Hope's Vanguard). If Tide assembled a board of
draw-on-Judgment characters and then flickered one ally with a Mirage enabler,
Conduit of Resonance would trigger ALL of their Judgment abilities. This turns
Mirage from "flicker one target for its Materialized" to "flicker one target
to trigger every Judgment ability on the board."

**Surprising interaction:** Mirage + Conduit creates a mass-Judgment engine
that no current Mirage build exploits. The Calling Night becomes "Materialize
any ally: Draw 1, opponent gains 2 points" -- which is absurd value when
flickered repeatedly, especially since the point cost is offset by the card
avalanche.

### 2d. Volcanic Channeler (Ember, 4 cost, "when an ally is dissolved, gain 1
energy")

Classified as Cinder. But Tide's Prevent suite includes Together Against the
Tide ("Prevent a played event which could dissolve an ally") and Voidshield
Guardian ("Prevent opponent's events that could dissolve an ally"). If the
opponent is pressuring the board with removal, Tide already runs these cards.
The non-obvious angle: when Tide CANNOT prevent the removal and an ally IS
dissolved, Volcanic Channeler converts that loss into energy for the next
Prevent cycle. It turns failed defense into fuel for future defense.

**Surprising interaction:** Volcanic Channeler in Depths as a resilience tool
against decks that overwhelm the Prevent suite. Not a sacrifice engine, but an
insurance policy.

### 2e. Bloomweaver (Warrior, 1 cost, "once per turn, when you materialize a
character, gain 1 energy")

Currently Basalt/Crucible. But Mirage flickers characters repeatedly, meaning
Bloomweaver produces 1 energy per flicker cycle. At 1 cost, Bloomweaver
competes with Angel of the Eclipse (4 cost, same effect) for the "energy per
materialize" slot in Mirage. Bloomweaver is dramatically cheaper and comes
online faster. The Warrior subtype is irrelevant in Mirage -- it is the
cheapest materialize-to-energy converter in the pool.

**Surprising interaction:** Mirage drafters should be picking Bloomweaver over
Angel of the Eclipse in most builds. The Warrior subtype creates a false draft
signal (looks like Crucible, plays like Mirage).

### 2f. Moonlit Dancer (Visitor, 3 cost, "characters in hand have fast; when you
play a fast character, gain 1 energy")

Classified as Gale core. But Tide holds the most characters in hand (large hand
size from draw). Moonlit Dancer converts ALL of those held characters to fast,
meaning Tide can deploy its entire hand at instant speed. This transforms Tide
from "reactive control" to "reactive deployment" -- hold up Prevent mana, and
if the opponent plays nothing worth preventing, flash in a Keeper of Forgotten
Light for Materialized: Draw 2 at end of opponent's turn.

**Surprising interaction:** Depths (Tide+Stone) with Moonlit Dancer becomes a
flash-control deck. Every character in hand becomes an instant-speed threat.
This bridges Tide to Zephyr's fast-deployment theme without committing to the
Gale aggression package.

### 2g. Surge of Fury (Event, 1 cost, "trigger additional Judgment phase")

Classified as Crucible/Basalt. But the Depths control deck with persistent
ramp bodies (Virtuoso of Harmony: gain 2 energy EOT, Dawnblade Wanderer:
Judgment gain 2 energy) gets doubled value from an extra Judgment phase. An
extra Judgment on a board with The Calling Night, Flickerveil Adept, and
Virtuoso of Harmony means: Draw 1, flicker a target, gain 2 energy. That is
enormous value for 1 energy.

**Surprising interaction:** Surge of Fury in Depths as a value amplifier rather
than an aggression tool. The card reads as "attack again" but plays as
"compound your engine."

---

## 3. Hidden Synergy Vectors (12 vectors)

### Vector 1: Hand-Size-Matters

**Condition:** Tide consistently maintains 6-8 cards in hand.

**Bridges:** Tide to Stone (Depths), Tide to Ember (Tempest)

**Mechanical template:** A card that provides scaling benefit based on hand
size. Examples: "Characters you play cost 1 less if you have 5+ cards in hand"
(Depths ramp amplifier), or "At end of turn, gain 1 energy for every 2 cards
in your hand" (Stone-coded card that rewards Tide's hoarding), or "+1 spark
for each card in your hand beyond 4" (a finisher body).

**Why non-obvious:** Other card games have hand-size-matters, but Dreamtides
currently has zero such effects. A drafter would not intuit that Tide's draw
is building toward a hand-size threshold because no card in the pool rewards
it.

**Non-obviousness rating: 8/10** -- requires awareness that no current card
uses this axis.

---

### Vector 2: Prevent-Trigger Payoffs

**Condition:** Tide prevents 2-4 cards per game.

**Bridges:** Tide to Ember (Tempest), Tide to Stone (Depths)

**Mechanical template:** A card that triggers when you prevent a card. Examples:
"When you prevent a card, gain 1 energy" (converts defense into mana, Stone-
coded), or "When you prevent a card, draw 1" (pure Tide but doubles the
Prevent's value), or "When you prevent a card, kindle 1" (bridges to Stone's
incremental growth).

**Why non-obvious:** Prevent is treated as a self-contained effect -- you pay
the cost, you stop the card, done. No current card treats Prevent as a trigger
condition. The Prevent player gets nothing except denial. A trigger payoff would
make Prevent proactive rather than purely reactive, fundamentally changing
Depths' play pattern.

**Non-obviousness rating: 7/10** -- other card games have counterspell payoffs,
so the concept is known, but Dreamtides' lack of any such card makes it
invisible in-system.

---

### Vector 3: Board-Width-from-Low-Spark-Bodies

**Condition:** Tide accumulates 3-5 characters with 1-2 spark each from
Materialized-draw plays.

**Bridges:** Tide to Zephyr (Mirage), Tide to Stone (Depths)

**Mechanical template:** A card that rewards having many allies regardless of
their individual spark. Examples: "Gain 1 point for each ally" (going-wide
finisher), "Prevent a played card if you have 4+ allies" (free Prevent from
board width), "Draw 1 for each ally, then discard 2" (mass refuel proportional
to board width).

**Why non-obvious:** Tide's characters look weak individually (1-2 spark). A
drafter sees them as "play for the Materialized, ignore the body." But 4
bodies at 1 spark each = 4 Judgment spark, which is NOT nothing. A card that
converts width into value would retroactively make every Materialized-draw body
a two-for-one: the draw AND the persistent board contribution.

**Non-obviousness rating: 6/10** -- "go wide" strategies exist but are
associated with tribal (Crucible Warriors, Basalt Spirit Animals), not with
Tide's generic low-spark bodies.

---

### Vector 4: Foresee-to-Top-of-Deck Pipeline

**Condition:** After Foresee, Tide knows the top card of the deck and has
curated it to be the best available option.

**Bridges:** Tide to Zephyr (Basalt via Dreamborne Leviathan), Tide to
Stone (Depths)

**Mechanical template:** A card that does something with the top card of your
deck. Examples: "Reveal the top card of your deck; if it's an event, you may
play it for free" (Tempest chain extender), "The top card of your deck has
Reclaim 0" (bridges to Ruin without Tide touching the void directly), "Once per
turn, you may play the top card of your deck" (generic top-deck advantage).

**Why non-obvious:** Foresee's value is framed as "filter out bad draws." But
the secondary effect -- you know EXACTLY what's on top -- is never leveraged.
Dreamborne Leviathan (play characters from top of deck) already exists in
Zephyr, but there is no Tide-coded card that exploits the curated-top-card
condition. A Tide+Zephyr card that says "Foresee 2, then play the top card if
its cost is 3 or less" would create a Foresee-into-deploy pipeline.

**Non-obviousness rating: 9/10** -- the information is created and immediately
consumed by the draw, making the intermediate "known top card" state invisible
to most players.

---

### Vector 5: Unspent-Energy-at-End-of-Turn

**Condition:** Tide holds up 2-3 energy for Prevent, then the opponent plays
around it. That energy evaporates.

**Bridges:** Tide to Stone (Depths)

**Mechanical template:** A card that converts unspent energy into value at end
of turn. Examples: "At end of turn, if you have 2+ energy, draw 1" (rewards
passing with Prevent up), "At end of turn, gain 1 energy next turn for each
unspent energy" (energy banking -- Stone-coded), or Minstrel of Falling Light's
activated ability (3 energy: draw 1) already partially fills this role as a
mana sink.

**Why non-obvious:** The "Prevent or develop" tension is treated as a
fundamental cost of the Depths archetype. Eliminating it feels like it would
make Depths too good. But a CONDITIONAL end-of-turn payoff (only if you did
NOT spend the energy on Prevent) creates a new decision: opponent must decide
whether to play into the Prevent or let you draw the card. This is a genuine
mind game, not a free bonus.

**Non-obviousness rating: 7/10** -- the condition is obvious to control players
from other games, but the absence of any such card in Dreamtides makes it a
gap rather than a known axis.

---

### Vector 6: Event-Count-in-Void

**Condition:** Tide's void becomes disproportionately event-heavy (70-80%
events vs. 40-50% for other resonances).

**Bridges:** Tide to Ember (Tempest), Tide to Ruin (Undertow)

**Mechanical template:** A card that counts events specifically in the void.
Examples: "Draw 1 for every 3 events in your void" (Tide-coded void payoff
that does not step on Ruin's generic void-size-matters), "Gain energy equal to
events in your void" (the proposed Leyline Detonation from Tempest new cards),
"Spark equal to events in void" (a parallel to Abomination of Memory but
event-specific and Tide-coded).

**Why non-obvious:** Void-size-matters is Ruin territory. But EVENT-count-in-
void is mechanically distinct -- it rewards playing events (Tide) rather than
milling generically (Ruin). A Tide player reaches 6+ events in void through
normal play. A Ruin player reaches a large void through mill but may have only
2-3 events. This creates a payoff axis that Tide reaches naturally but Ruin
does not, avoiding the "Ruin does void stuff" objection.

**Non-obviousness rating: 9/10** -- even the design documents conflate "void
count" with Ruin. Differentiating event-void-count as a Tide axis requires
challenging a core assumption.

---

### Vector 7: Opponent's Hand Depletion

**Condition:** Tide's hand disruption (Break the Veil, Lurking Dread, Wraith
of Twisting Shadows) plus Prevent can leave the opponent with very few cards
in hand.

**Bridges:** Tide to Stone (Depths), Tide to Ember (Tempest)

**Mechanical template:** A card that rewards having more cards in hand than
the opponent. Examples: "If you have more cards in hand than the opponent,
your events cost 1 less" (asymmetric advantage from disruption), "Draw cards
equal to the difference between your hand size and the opponent's" (a one-shot
refuel that scales with successful disruption).

**Why non-obvious:** Hand disruption is framed as "remove their best card."
The secondary effect -- the opponent's hand size shrinks while yours grows --
creates a SIZE differential that no card currently exploits. Other card games
have "hand advantage matters" effects; Dreamtides does not.

**Non-obviousness rating: 8/10** -- requires noticing the hand-size-differential
rather than treating disruption as purely card-quality-based.

---

### Vector 8: Prevent as Void Fuel

**Condition:** Prevented cards go to the OPPONENT's void, not yours. But your
own Prevent events go to YOUR void after resolution.

**Bridges:** Tide to Ruin (Undertow)

**Mechanical template:** A card that cares about events entering YOUR void (not
total void size). Examples: Spirit of Smoldering Echoes already does this
(+1 spark per event entering void), but a Tide-coded version would be: "When
an event you own enters the void, Foresee 1" (cycles through your deck faster
as you play events), or "When you play a Prevent event, put the top card of
your deck into your void" (Prevent as self-mill trigger for Undertow).

**Why non-obvious:** Prevent is the most reactive, "anti-void" mechanic in
Tide. Treating Prevent events as void fuel inverts their identity. A player
would never think "my Abolish is also a mill card" unless a connector
explicitly linked the two. This creates a bizarre but functional
Depths-Undertow hybrid where you are simultaneously countering the opponent
and filling your own void.

**Non-obviousness rating: 10/10** -- conceptually paradoxical.

---

### Vector 9: Draw Triggers for Non-Tide Payoffs

**Condition:** Tide draws 3-6 extra cards per turn during big draw turns (The
Power Within doubling, Echoes of the Journey, Keeper of Forgotten Light
flicker).

**Bridges:** Tide to Ruin (Undertow via Eternal Sentry)

**Mechanical template:** Cards that trigger on draw events. Eternal Sentry
("draw 2+ in a turn: if in void, gains Reclaim") is the only current example.
The template extends to: "When you draw your second card each turn, kindle 1"
(Stone-coded incremental growth triggered by Tide's draw), "When you draw 3+
cards in a turn, gain 1 energy" (Stone-coded ramp triggered by draw volume).

**Why non-obvious:** Draw is treated as its own reward (you drew the card,
that IS the value). But making draw a TRIGGER condition opens a second axis
of value. Eternal Sentry is the proof of concept, but it is the only card in
the pool with a draw-trigger. Expanding this axis would make every Tide draw
spell secretly a trigger enabler for cross-resonance payoffs.

**Non-obviousness rating: 7/10** -- Eternal Sentry shows the pattern, but
only one card uses it, making it look like an anomaly rather than an axis.

---

### Vector 10: Materialized Bodies as Sacrifice Fodder

**Condition:** Tide plays low-spark bodies for their Materialized triggers,
then those bodies sit on the battlefield contributing little.

**Bridges:** Tide to Ember (Cinder via sacrifice), Tide to Ruin (Cinder/
Eclipse)

**Mechanical template:** A card that specifically rewards sacrificing characters
that have already used their Materialized ability. Examples: "Abandon an ally
that entered play this turn: Draw 1" (you deployed Frost Visionary, drew 1 from
Materialized, then sacrifice it to draw again -- net +2 cards from a 2-cost
character), or "Abandon an ally: If that character had a Materialized ability,
gain 2 energy instead of 1" (premium sacrifice value for Tide bodies).

**Why non-obvious:** Tide's identity document explicitly says "Tide is NOT
sacrifice" and "Tide characters are worth more alive than dead." This is true
for Tide IN ISOLATION. But in a Tide+Ember deck (Tempest), the tension between
"draw from Materialized" and "sacrifice the spent body" creates a new play
pattern. The Materialized-draw body is not worth keeping alive AFTER it has
drawn -- it is a 2-cost 2-spark body with no further text. Sacrificing it is
objectively correct once the draw has resolved.

**Non-obviousness rating: 8/10** -- violates Tide's stated identity, which
makes drafters instinctively avoid the line.

---

### Vector 11: Information Asymmetry from Opponent Hand Knowledge

**Condition:** Wraith of Twisting Shadows (Materialized: look at opponent's hand
and discard a chosen card) gives Tide information about the opponent's hand.

**Bridges:** Tide to Stone (Depths), Tide to Zephyr (Mirage via flicker)

**Mechanical template:** A card that rewards knowing what is in the opponent's
hand. Examples: "If you have seen the opponent's hand this turn, your Prevents
cost 1 less" (surgical information makes defense cheaper), "Reveal the
opponent's hand: You may play a card from their hand this turn" (information
converted to theft -- aggressive Tide), "Name a card: If the opponent has it,
they discard it; if not, you draw 1" (information test).

**Why non-obvious:** Hand-peeking is treated as a one-time info dump, not as a
game-state condition that persists. But in Dreamtides' digital format, the game
could track "you have seen the opponent's hand this turn" as a state flag. A
card that conditionally checks this flag would make Wraith of Twisting Shadows
(and its flicker repetition) into a setup card for a powerful conditional effect.

**Non-obviousness rating: 9/10** -- information as a game-state condition is
unusual in deckbuilders.

---

### Vector 12: Deck Size Differential

**Condition:** Tide draws aggressively, so Tide's deck is significantly smaller
than the opponent's. This means Tide is closer to decking out BUT also closer
to seeing every card they drafted.

**Bridges:** Tide to Zephyr (Mirage/Basalt via Lumineth), Tide to Ruin
(Undertow)

**Mechanical template:** A card that rewards a small remaining deck. Examples:
"Draw until you have as many cards in hand as cards in your deck" (refuel that
gets better as deck shrinks), "Gain 1 energy for each card fewer than 10 in
your deck" (scaling ramp from deck thinning), "If your deck has 5 or fewer
cards, characters you play cost 2 less" (endgame cost reduction).

**Why non-obvious:** Deck depletion is framed as a risk (you lose when you
cannot draw). Lumineth is the only card that treats it as a win condition.
But there is a spectrum between "full deck" and "empty deck" that no card
currently occupies. A deck-size-matters axis would make Tide's aggressive draw
a resource investment rather than just card accumulation.

**Non-obviousness rating: 7/10** -- alternate win conditions based on library
size exist in other games, but continuous deck-size-scaling effects are rarer.

---

## 4. Anti-Synergy Awareness -- What Tide MUST NOT Do

### 4a. Tide Must Not Generate Energy

Tide's resource is cards, not energy. A bridge card that gives Tide energy
generation violates the fundamental tension of the Depths archetype (reactive
vs. proactive) and the Tempest archetype (patience vs. explosion). Starcatcher
should remain Ember-coded; Tide should not get its own "play event: gain energy"
card. The closest Tide should come is efficiency -- events that cost less
(Keeper of the Lightpath), not events that produce energy.

**Violation example:** "When you draw a card, gain 1 energy" would be a Tide
card by trigger condition but an energy-generation card by output. This
collapses the Tide/Stone/Ember energy boundaries and makes Tide self-sufficient,
destroying the need for partner resonances.

### 4b. Tide Must Not Have Aggressive Spark Scaling

Hand-size-matters and board-width-from-bodies can provide finishing power, but
they must not resemble Ember's aggressive spark scaling. A Tide finisher should
reach 5-7 spark through accumulated game-state conditions, not through
"whenever you attack, +2 spark" or abandon-for-spark. The scaling should be
slow and proportional to game length, not explosive.

**Violation example:** A 2-cost body that gains +1 spark every time you draw a
card would grow to 10+ spark by mid-game, faster than any Ember aggro card.
This turns Tide's draw engine into the best aggro strategy, which contradicts
Tide's identity as patient and reactive.

### 4c. Tide Must Not Do Mass Recursion

Tide's event recovery (Whisper of the Past, Archive of the Forgotten) is
SURGICAL -- it retrieves specific events. A Tide card that says "return all
cards from your void to your hand" or "all cards in your void gain Reclaim 0"
would be Ruin's territory (Path to Redemption is mono-Ruin for exactly this
reason). Bridge cards should exploit void state without performing mass
recursion.

**Violation example:** "Events in your void gain Reclaim 0" would be a Tide
card by event focus but a Ruin card by mass-recursion output.

### 4d. Tide Must Not Replace Tribal Infrastructure

Tide has no subtype-matters payoffs. A bridge card that says "for each Warrior
in play, draw 1" or "Survivors you control have Prevent" would be mechanically
insane and identity-incoherent. Tide may CONTAIN Warrior bodies (Frost
Visionary) and Survivor bodies (Ashlight Caller), but it must never provide
the tribal payoff layer. Those belong to Stone (Warriors), Ruin (Survivors),
and Zephyr (Spirit Animals).

### 4e. Tide Must Not Own Sacrifice Outlets

Even though Vector 10 identifies that Materialized bodies are good sacrifice
fodder, the SACRIFICE OUTLET itself must never be Tide-coded. The connector
card should be Ember-coded or dual Tide+Ember, with the sacrifice being the
Ember half of the bridge. Tide supplies the fodder; Ember supplies the knife.

### 4f. Tide Must Not Bypass the Draw-or-Deploy Tension

Tide's play pattern requires choosing between developing the board and holding
up Prevent mana. Connector cards should create new ways to resolve this tension
(Vector 5: end-of-turn energy conversion) but should not ELIMINATE it. A card
that says "you may play characters as though they were fast events" would
destroy the tension entirely and collapse Tide into Zephyr's fast-deployment
space.

---

## 5. Priority Recommendations

If the design team creates connector cards, the highest-value vectors are:

1. **Vector 1 (Hand-Size-Matters)** -- Completely untapped in the pool,
   immediately bridges Tide to Stone/Ember, and creates a new draft axis that
   rewards Tide's most natural output.

2. **Vector 6 (Event-Count-in-Void)** -- Differentiates Tide's void from Ruin's
   void, creates a Tide-specific payoff axis, and has partial proof-of-concept
   in Spirit of Smoldering Echoes.

3. **Vector 4 (Foresee-to-Top-of-Deck Pipeline)** -- Exploits the most
   invisible side effect (the curated top card) and has an existing connector
   in Dreamborne Leviathan.

4. **Vector 2 (Prevent-Trigger Payoffs)** -- Would transform Depths from
   "reactive by necessity" to "reactive by choice," making the archetype more
   proactive without violating its identity.

5. **Vector 5 (Unspent Energy at End of Turn)** -- Addresses Depths' core
   tension with a conditional payoff that creates new opponent decisions.

Vectors 8 (Prevent as void fuel) and 11 (information asymmetry) are the most
creatively unusual but require the most design care to avoid feeling artificial.
Vector 10 (sacrifice fodder) is powerful but risks blurring Tide's identity
boundary with Ember.
