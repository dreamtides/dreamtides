# Dreamtides Battle Rules

Dreamtides is a two-player card game in the tradition of collectible card games
like Magic: The Gathering. Players build decks of character and event cards, then
compete to score victory points by accumulating spark on the battlefield. Two key
differences from traditional card games: the shared Dreamwell system replaces
lands for energy production, and there is no combat phase — instead, spark totals
are compared each turn during the Judgment phase to score points.

## Table of Contents

- [Objective](#objective)
- [Card Types](#card-types)
- [Zones](#zones)
- [The Dreamwell and Energy](#the-dreamwell-and-energy)
- [Turn Structure](#turn-structure)
- [Playing Cards](#playing-cards)
- [The Stack and Priority](#the-stack-and-priority)
- [Spark and Scoring](#spark-and-scoring)
- [Character Limit](#character-limit)
- [Hand Size Limit](#hand-size-limit)
- [Keywords](#keywords)
- [Trigger Keywords](#trigger-keywords)
- [Ability Types](#ability-types)
- [Effects Reference](#effects-reference)
- [Targeting](#targeting)
- [Character Subtypes](#character-subtypes)
- [Figments](#figments)
- [Card Rarity](#card-rarity)
- [Glossary](#glossary)

## Objective

The first player to reach 12 victory points wins the game. Points are scored
during the Judgment phase at the start of each turn by having more total spark
than your opponent. If 50 turns pass without a winner, the game ends in a draw.

## Card Types

**Character** — Permanent cards that enter the battlefield when they resolve.
Each character has a spark value, which contributes to scoring during the
Judgment phase. Characters remain on the battlefield until removed by an effect
(Dissolve, Banish, or Abandon). They can have triggered, activated, and static
abilities. Characters have subtypes (Mage, Warrior, Robot, etc.) that other
cards can reference.

**Event** — One-shot cards that produce an effect when they resolve, then move to
the void. Events can be marked as "fast," allowing them to be played during the
opponent's turn or in response to other cards on the stack.

**Dreamcaller** — A player's identity card, an animated 3D character that starts
each battle already in play. Dreamcallers provide powerful ongoing abilities
(static, triggered, or activated) that define a player's playstyle. Primarily
chosen during quest mode.

**Dreamsign** — A quest-layer card representing a 2D illustrated object that
provides ongoing effects. Selected during quest mode and active throughout
battles.

**Dreamwell** — Special shared cards drawn during the Dreamwell phase. Not part
of either player's deck. They produce energy and can have bonus effects.

In constructed decks, the main card types are Characters and Events.

## Zones

**Deck** — A player's shuffled draw pile. Cards are drawn from the top during
the Draw phase and by card effects.

**Hand** — Cards held by a player, hidden from the opponent. Cards are played
from here (or from the void via Reclaim).

**Stack** — A temporary zone for cards that have been played but not yet
resolved. While a card is on the stack, the opponent may respond with fast cards.
Characters move to the battlefield when they resolve; events move to the void.

**Battlefield** — Where characters reside and generate spark. Each player can
have up to 8 characters on the battlefield at once.

**Void** — The discard pile. Events go here after resolving. Characters go here
when dissolved or abandoned. Some cards can interact with cards in the void
(notably via Reclaim).

**Banished** — A permanent exile zone. Cards sent here cannot return to play
under normal circumstances.

## The Dreamwell and Energy

Energy is the resource used to play cards. Unlike traditional card games that use
land cards, Dreamtides uses the Dreamwell — a shared deck of special cards that
both players draw from, one per turn.

**How the Dreamwell works:**

- The Dreamwell is a deck of 7 named cards, shared between both players.
- During each player's Dreamwell phase, the next card is drawn from the
  Dreamwell automatically (no player choice involved).
- Each Dreamwell card has an `energy produced` value that permanently increases
  the player's energy production. Your total production grows by this amount
  every time you draw a Dreamwell card.
- At the start of each turn, your current energy is reset to equal your total
  production. Unspent energy does not carry over between turns.
- Many Dreamwell cards also have bonus effects (draw a card, foresee, gain a
  point, gain extra energy, or mill cards to the void).

**Phases and cycling:**

- Dreamwell cards have a phase number. Phase 0 cards (producing 2 energy each)
  only appear during the first cycle through the deck, providing an early energy
  boost. Phase 1 cards (producing 1 energy each, with bonus effects) appear in
  every subsequent cycle.
- When the deck cycles, it is reshuffled, but cards are sorted by phase so that
  phase 0 cards always come first within a cycle.

**Current Dreamwell cards:**

- Dawning Horizon — 2 energy (phase 0, first cycle only)
- Sunrise Cavern — 2 energy (phase 0, first cycle only)
- Skypath — 1 energy, Foresee 1 (phase 1)
- Autumn Glade — 1 energy, gain 1 point (phase 1)
- Twilight Radiance — 1 energy, gain 1 extra energy this turn (phase 1)
- Astral Interface — 1 energy, draw 1 card then discard 1 card (phase 1)
- Auroral Passage — 1 energy, put top 3 cards of your deck into your void
  (phase 1)

## Turn Structure

Each turn progresses through these phases in order:

1. **Starting** — Internal bookkeeping; transitions immediately to Judgment.
2. **Judgment** — Spark totals are compared and points are scored (see Spark and
   Scoring below).
3. **Dreamwell** — The active player draws the next Dreamwell card,
   permanently increasing their energy production and resetting their current
   energy. Any bonus effect on the card is applied.
4. **Draw** — The active player draws one card from their deck. (Skipped on the
   very first turn of the game.)
5. **Main** — The active player can play cards from hand, activate abilities,
   and take other actions. This is the primary action phase.
6. **Ending** — The active player passes. The opponent may play fast cards
   during this window. Once the opponent also passes, end-of-turn triggers fire
   and the turn passes to the opponent.

**Game start:** Each player draws 5 cards as their opening hand.

## Playing Cards

To play a card, the active player must have enough current energy to pay its
energy cost. Playing a card:

1. Deducts the card's energy cost from the player's current energy.
2. Moves the card from hand (or void, if using Reclaim) to the stack.
3. Fires "played card" triggers.
4. Gives the opponent priority to respond.

Cards can be played from hand during the Main phase. Fast cards can additionally
be played during the Ending phase, during the opponent's Main phase, or in
response to a card on the stack.

## The Stack and Priority

When a card is played, it goes on the stack. The opponent then receives priority
and may play fast cards in response. Unlike Magic: The Gathering, only one pass
is needed to resolve a card (not two). Cards on the stack resolve last-in,
first-out (LIFO):

- **Events** resolve by applying their effects, then move to the void.
- **Characters** resolve by entering the battlefield.

After a card resolves, if the stack is not empty, the card's controller receives
priority. When both players pass with an empty stack, the game continues to the
next phase.

## Spark and Scoring

Spark is the primary stat on characters. Each character on the battlefield
contributes its spark value to the player's total. During the Judgment phase at
the start of each turn:

- The active player's total spark (sum of all their characters' spark values
  plus any spark bonus) is compared to the opponent's total spark.
- If the active player's total spark is higher, they gain victory points equal
  to the difference.
- If the active player's total spark is equal to or lower than the opponent's,
  nothing happens.

The first player to reach 12 points wins.

Characters have no health or toughness — spark is their only stat. Characters
are removed from the battlefield by specific effects (Dissolve, Banish, Abandon),
not by damage.

## Character Limit

Each player can have at most 8 characters on the battlefield. When a 9th
character would enter, the character with the lowest spark value is automatically
abandoned to the void. (Ties are broken by lowest energy cost, then by card ID.)
The abandoned character's current spark value is permanently added to the
player's spark bonus, which counts toward their total spark for Judgment scoring
going forward.

## Hand Size Limit

A player's hand can hold at most 10 cards. If a draw effect would cause a
player to exceed this limit, the player gains 1 energy instead of drawing.

## Keywords

**Dissolve** — Destroy a target character, moving it from the battlefield to the
void. This fires the "Dissolved" trigger on the affected character. Can be
prevented if the character is Anchored. Unlike Abandon, Dissolve can target any
character (yours or the opponent's).

**Banish** — Permanently remove a card from the game by moving it to the
Banished zone. Banished cards cannot be returned to play. Several variants
exist: banish from the battlefield, banish from the void, banish until the
banishing card leaves play, and banish until the next main phase.

**Materialize** — Put a character onto the battlefield. This is the term for a
character entering play, whether from hand (played normally), from the void
(via Reclaim or effects), from the deck (via effects), or as a token (Figments).

**Prevent** — Counter a card on the stack, sending it to the void without
resolving. This is the "counterspell" mechanic. Prevent effects are always fast
(they must be played in response to a card on the stack).

**Abandon** — Move one of your own characters from the battlefield to the void.
Unlike Dissolve, Abandon cannot be prevented and only targets your own
characters. It fires the "Dissolved" trigger. Often used as a cost for
abilities.

**Kindle N** — Add N spark to your leftmost character on the battlefield. If you
have no characters, Kindle does nothing.

**Foresee N** — Look at the top N cards of your deck. You may reorder them in
any order and optionally send any of them to the void. This gives control over
future draws and can fuel void-based strategies.

**Reclaim** — A named ability that allows you to play a card from your void
instead of from your hand. The card is played at its normal cost (or at a
specified alternate cost: Reclaim N means it costs N energy when played from the
void). When a reclaimed card later leaves play, it is banished instead of
returning to the void.

**Fast** — A property on cards and abilities indicating they can be used outside
the normal main phase timing. Fast cards can be played during the opponent's
main phase, during the ending phase, or in response to cards on the stack. Fast
activated abilities follow the same timing rules.

**Discover** — Look at 3 cards from your deck that match a specified criteria,
then choose one to add to your hand. The other cards return to the deck.

**Anchored** — A temporary status that prevents a character from being dissolved
this turn. Visually represented as a shield effect. Does not prevent Banish or
Abandon.

**Copy** — Create a duplicate of a card or effect. Variants include copying a
character on the battlefield or copying the next card played.

**Gain Control** — Take control of an opponent's character, moving it to your
side of the battlefield.

## Trigger Keywords

Trigger keywords appear on characters and fire automatically when their
condition is met:

**Materialized** — Triggers when this character enters the battlefield. This is
the "enters play" trigger, similar to ETB (enter the battlefield) effects in
other card games.

**Judgment** — Triggers during the Judgment phase at the start of each turn
while this character is on the battlefield.

**Dissolved** — Triggers when this character is destroyed (dissolved or
abandoned). This is the "leaves play via destruction" trigger.

Characters can have combined triggers like "Materialized, Judgment" (fires both
on entry and each Judgment phase) or "Materialized, Dissolved" (fires on both
entry and destruction).

## Ability Types

**Event abilities** — Effects printed on event cards. They resolve when the
event resolves from the stack, and then the event moves to the void.

**Triggered abilities** — Abilities that fire automatically when a specific game
event occurs. Triggered abilities use keywords (Materialized, Judgment,
Dissolved) or descriptive conditions ("When you play a card," "Whenever a
character is dissolved," "At end of turn"). They go on the stack when triggered
and can be responded to.

**Activated abilities** — Abilities with a cost that a player chooses to use.
Written as "Cost: Effect" (e.g., "2 energy: Draw a card"). Can be once per turn
or unlimited use. Can be Fast for off-turn activation.

**Static abilities** — Always-on rule modifications that apply as long as the
character is on the battlefield. Examples include cost reductions for certain
card types, spark bonuses for matching characters, or modifications to how
game rules work.

**Modal abilities** — Abilities that present multiple options to choose from.
Written as "Choose one:" followed by bullet points listing the available effects
and their costs.

## Effects Reference

The following categories of effects exist in the game:

**Character removal:** Dissolve (to void, preventable), Banish (permanent
exile), Abandon (self-only, not preventable)

**Materialization:** Materialize characters from hand, void, or deck; create
Figment tokens

**Card draw and deck manipulation:** Draw cards, Foresee (reorder top of deck
and optionally void), put cards from deck into void, put cards from void on top
of deck

**Zone movement:** Return characters to hand, return cards from void to hand or
to play

**Energy:** Gain energy, gain energy for each matching condition, multiply your
energy, gain twice that much energy instead

**Points:** Gain points, lose points, opponent gains points, "you win the game"

**Spark modification:** Kindle (add spark to leftmost), characters gain spark,
each matching character gains spark, spark becomes a fixed value

**Counterspell:** Prevent a card from resolving, prevent unless the opponent
pays a cost

**Protection:** Prevent dissolve this turn (Anchored), banish when leaves play

**Ability grants:** Grant Reclaim to a card, create temporary static abilities
or triggers until end of turn

**Copy and control:** Copy a character, copy the next card played, gain control
of an opponent's character

**Special:** Take an extra turn, trigger additional Judgment phase at end of
turn, each player shuffles hand and void into deck and draws cards

## Targeting

Effects target cards using ownership and type predicates:

**Ownership:** Your cards, enemy cards, any card, another card (not the source)

**Card type:** Character, event, specific subtypes, characters with a minimum
spark value, cards with a specific energy cost

Targeting is specified when a card is played (for stack targets) or when an
effect resolves (for pending effect targets). Players are prompted to select
valid targets from the available options.

## Character Subtypes

Characters belong to one or more subtypes. Some cards reference subtypes for
bonuses or targeting:

Agent, Ancient, Avatar, Child, Detective, Enigma, Explorer, Guide, Hacker,
Mage, Monster, Musician, Outsider, Renegade, Robot, Spirit Animal, Super,
Survivor, Synth, Tinkerer, Trooper, Visionary, Visitor, Warrior

## Figments

Figments are token characters created by card effects rather than played from a
deck. There are four types: Celestial, Radiant, Halcyon, and Shadow. Figments
enter the battlefield through "Materialize Figments" effects and behave like
regular characters (they have spark values, count toward the character limit,
and can be targeted by effects).

## Card Rarity

Cards have one of five rarities: Common, Uncommon, Rare, Legendary, and Special.
Rarity affects how often cards appear during quest mode drafting and
deckbuilding.

## Glossary

| Term | Definition |
|------|------------|
| Abandon | Move your own character to the void (cannot be prevented) |
| Anchored | Cannot be dissolved this turn |
| Banish | Permanently remove a card from the game |
| Battlefield | Zone where characters reside and generate spark |
| Copy | Create a duplicate of a card or effect |
| Deck | A player's shuffled draw pile |
| Discover | Look at 3 matching cards from deck, choose one for hand |
| Dissolve | Destroy a character, moving it to the void |
| Dreamcaller | A player's identity card, starts in play |
| Dreamsign | A quest-layer card providing ongoing effects |
| Dreamwell | Shared energy-producing card deck |
| Energy | Resource spent to play cards |
| Event | One-shot card that resolves and goes to void |
| Fast | Can be played outside normal main phase timing |
| Figment | Token character created by effects |
| Foresee N | Look at top N of deck, reorder and optionally void |
| Gain Control | Take control of an opponent's character |
| Hand | Cards held by a player, hidden from opponent |
| Judgment | Turn phase where spark is compared and points scored |
| Kindle N | Add N spark to your leftmost character |
| Materialize | Put a character onto the battlefield |
| Points | Victory points; first to 12 wins |
| Prevent | Counter a card on the stack |
| Reclaim | Play a card from your void; banished when it later leaves play |
| Spark | Character stat used to score points during Judgment |
| Spark Bonus | Permanent spark added from abandoned characters (character limit) |
| Stack | Temporary zone for played but unresolved cards |
| Void | Discard pile |
