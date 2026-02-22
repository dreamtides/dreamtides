# Dreamtides Battle Rules

Dreamtides is a two-player card game in the tradition of collectible card games
like Magic: The Gathering. Players build decks of character and event cards,
then compete to score victory points by accumulating spark on the battlefield.
Two key differences from traditional card games: the shared Dreamwell system
replaces lands for energy production, and there is no combat phase — instead,
spark totals are compared each turn during the Judgment phase to score points.

## Table of Contents

- [Objective](#objective)
- [Card Types](#card-types)
- [Zones](#zones)
- [The Dreamwell and Energy](#the-dreamwell-and-energy)
- [Turn Structure](#turn-structure)
- [Playing Cards and the Stack](#playing-cards-and-the-stack)
- [Spark and Scoring](#spark-and-scoring)
- [Keywords and Effects](#keywords-and-effects)
- [Ability Types](#ability-types)
- [Targeting](#targeting)
- [Figments](#figments)

## Objective

The first player to reach the victory point threshold wins the game. The default
threshold is 12 points, but this is configurable per battle. Points are scored
during the Judgment phase at the start of each turn by having more total spark
than your opponent. If 50 turns pass without a winner, the game ends in a draw.

## Card Types

**Character** — Permanent cards that enter the battlefield when they resolve.
Each character has a spark value, which contributes to scoring during the
Judgment phase. Characters remain on the battlefield until removed by an effect
(Dissolve, Banish, or Abandon). They can have triggered, activated, and static
abilities. Characters have subtypes (Mage, Warrior, Robot, etc.) that other
cards can reference.

**Event** — One-shot cards that produce an effect when they resolve, then move
to the void. Events can be marked as "fast," allowing them to be played during
the opponent's turn or in response to other cards on the stack.

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

**Hand** — Cards held by a player, hidden from the opponent. A player's hand can
hold at most 10 cards. If a draw effect would exceed this limit, the player
gains 1 energy instead of drawing.

**Stack** — A temporary zone for cards that have been played but not yet
resolved. While a card is on the stack, the opponent may respond with fast
cards. Characters move to the battlefield when they resolve; events move to the
void.

**Battlefield** — Where characters reside and generate spark. Each player can
have up to 8 characters on the battlefield at once (see Character Limit under
Spark and Scoring).

**Void** — The discard pile. Events go here after resolving. Characters go here
when dissolved or abandoned. Some cards can interact with cards in the void
(notably via Reclaim).

**Banished** — A permanent exile zone. Cards sent here cannot return to play
under normal circumstances.

## The Dreamwell and Energy

Energy is the resource used to play cards. Unlike traditional card games that
use land cards, Dreamtides uses the Dreamwell — a shared deck of special cards
that both players draw from, one per turn.

**How the Dreamwell works:**

- The Dreamwell is a shared deck of cards (the size varies by configuration).
  During each player's Dreamwell phase, the next card is drawn automatically (no
  player choice involved).
- Each Dreamwell card has an energy production value that permanently increases
  the player's total energy production.
- At the start of each turn, your current energy is reset to equal your total
  production. Unspent energy does not carry over between turns.
- Many Dreamwell cards also have bonus effects such as drawing a card, using
  Foresee, gaining a point, gaining extra energy, or milling cards to the void.

**Phases and cycling:**

- Dreamwell cards have a phase number. Phase 0 cards only appear during the
  first cycle through the deck, typically providing a larger early energy boost.
  Higher-phase cards appear in every subsequent cycle, producing less energy per
  card but with bonus effects attached.
- When the deck cycles, it is reshuffled, with cards sorted by phase so that
  lower-phase cards always come first within a cycle.

For example, a phase 0 Dreamwell card might produce 2 energy with no bonus
effect, while a phase 1 card might produce 1 energy and also let you Foresee 1.

## Turn Structure

Each turn progresses through these phases in order:

1. **Judgment** — Spark totals are compared and points are scored (see Spark and
   Scoring).
2. **Dreamwell** — The active player draws the next Dreamwell card, permanently
   increasing their energy production and resetting their current energy. Any
   bonus effect on the card is applied.
3. **Draw** — The active player draws one card from their deck. (Skipped on the
   very first turn of the game.)
4. **Main** — The active player can play cards from hand, activate abilities,
   and take other actions. This is the primary action phase.
5. **Ending** — The active player passes. The opponent may play fast cards
   during this window. Once the opponent also passes, end-of-turn triggers fire
   and the turn passes to the opponent.

**Game start:** Each player draws 5 cards as their opening hand.

## Playing Cards and the Stack

To play a card, the active player must have enough current energy to pay its
energy cost. Playing a card deducts the cost from current energy, moves the card
to the stack, fires "played card" triggers, and gives the opponent priority to
respond.

Cards can be played from hand during the Main phase. Fast cards can additionally
be played during the Ending phase, during the opponent's Main phase, or in
response to a card on the stack.

**Stack resolution:** Unlike Magic: The Gathering, only one pass is needed to
resolve a card (not two). Cards on the stack resolve last-in, first-out (LIFO).
Events resolve by applying their effects and moving to the void. Characters
resolve by entering the battlefield. After a card resolves, if the stack is not
empty, the card's controller receives priority.

## Spark and Scoring

Spark is the primary stat on characters. Each character on the battlefield
contributes its spark value to the player's total. Characters have no health or
toughness — spark is their only stat. Characters are removed from the
battlefield by specific effects (Dissolve, Banish, Abandon), not by damage.

During the Judgment phase at the start of each turn, the active player's total
spark (sum of all their characters' spark values plus any spark bonus) is
compared to the opponent's total spark. If the active player's total is higher,
they gain victory points equal to the difference. Otherwise, nothing happens.

**Spark bonus:** A permanent addition to a player's spark total that is not tied
to any character. This is gained primarily from the character limit mechanic.

**Character limit:** Each player can have at most 8 characters on the
battlefield. When a 9th character would enter, the character with the lowest
spark value is automatically abandoned to the void. The abandoned character's
current spark value is permanently added to the player's spark bonus.

## Keywords and Effects

**Dissolve** — Destroy a target character, moving it from the battlefield to the
void. Fires the "Dissolved" trigger. Can target any character (yours or the
opponent's).

**Banish** — Permanently remove a card from the game by sending it to the
Banished zone. Several variants exist: banish from the battlefield, banish from
the void, banish until the banishing card leaves play, and banish until the next
main phase.

**Materialize** — Put a character onto the battlefield. This is the term for a
character entering play, whether from hand (played normally), from the void (via
Reclaim or effects), from the deck (via effects), or as a token (Figments).

**Prevent** — Counter a card on the stack, sending it to the void without
resolving. Prevent effects are always fast (they must be played in response to a
card on the stack).

**Abandon** — Move one of your own characters from the battlefield to the void.
Cannot be prevented and only targets your own characters. Fires the "Dissolved"
trigger. Often used as a cost for abilities.

**Kindle N** — Add N spark to your leftmost character on the battlefield.

**Foresee N** — Look at the top N cards of your deck. You may reorder them in
any order and optionally send any of them to the void.

**Reclaim** — A named ability that allows you to play a card from your void
instead of from your hand. The card is played at its normal cost (or at a
specified alternate cost: Reclaim N means it costs N energy when played from the
void). When a reclaimed card later leaves play, it is banished instead of
returning to the void.

**Fast** — A property on cards and abilities indicating they can be used outside
normal main phase timing: during the opponent's main phase, during the ending
phase, or in response to cards on the stack.

**Discover** — Look at 3 cards from your deck that match a specified criteria,
then choose one to add to your hand.

**Copy** — Create a duplicate of a card or effect. Variants include copying a
character on the battlefield or copying the next card played.

**Gain Control** — Take control of an opponent's character, moving it to your
side of the battlefield.

**Other effect categories:** Effects also exist for drawing cards, gaining or
losing energy, gaining or losing points, modifying spark values on characters,
granting temporary abilities until end of turn, taking extra turns, triggering
additional Judgment phases, and shuffling hands and voids back into decks.

## Ability Types

**Event abilities** — Effects printed on event cards. They resolve when the
event resolves from the stack, then the event moves to the void.

**Triggered abilities** — Abilities that fire automatically when a specific game
event occurs. Three keyword triggers can appear on characters: **Materialized**
(fires when the character enters the battlefield), **Judgment** (fires during
the Judgment phase each turn), and **Dissolved** (fires when the character is
destroyed). Triggered abilities can also use descriptive conditions like "When
you play a card" or "At end of turn." Characters can have combined triggers such
as "Materialized, Judgment" (fires both on entry and each Judgment phase).

**Activated abilities** — Abilities with a cost that a player chooses to use,
written as "Cost: Effect" (e.g., "2 energy: Draw a card"). Can be once per turn
or unlimited use. Can be Fast for off-turn activation.

**Static abilities** — Always-on rule modifications that apply as long as the
source is on the battlefield. Examples include cost reductions, spark bonuses
for matching characters, or modifications to game rules.

**Modal abilities** — Abilities that present multiple options to choose from,
written as "Choose one:" followed by the available effects and their costs.

## Targeting

Effects target cards using ownership and type predicates. Ownership predicates
include your cards, enemy cards, any card, or another card (not the source).
Type predicates include character, event, specific subtypes, characters with a
minimum spark value, or cards with a specific energy cost.

Targeting is specified when a card is played (for stack targets) or when an
effect resolves (for pending effect targets). Players are prompted to select
valid targets from the available options.

## Figments

Figments are token characters created by card effects rather than played from a
deck. There are four types: Celestial, Radiant, Halcyon, and Shadow. Figments
enter the battlefield through "Materialize Figments" effects and behave like
regular characters — they have spark values, count toward the character limit,
and can be targeted by effects.
