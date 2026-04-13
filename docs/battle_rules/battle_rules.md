# Dreamtides Battle Rules

Dreamtides is a two-player card game in the tradition of collectible card games
like Magic: The Gathering. Players build decks of character and event cards,
then compete to score victory points through positional combat on a staggered
battlefield. Two key differences from traditional card games: the shared
Dreamwell system replaces lands for energy production, and combat is resolved
positionally on a staggered battlefield — during the Judgment phase at the start
of each turn, the active player's deployed characters attack across the four
deployed lanes while the non-active player's deployed characters block.
Unblocked attackers score points, while paired attackers and blockers compare
spark and the weaker is dissolved.

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
- [Supplement: Reading Dreamtides Through an MTG Lens](#supplement-reading-dreamtides-through-an-mtg-lens)
- [Appendix: Translating MTG Mechanics into Dreamtides](#appendix-translating-mtg-mechanics-into-dreamtides)

## Objective

The first player to reach the victory point threshold wins the game. The default
threshold is 12 points, but this is configurable per battle. Points are scored
during the Judgment phase at the start of each turn when unblocked attacking
characters score victory points equal to their spark. If 50 turns pass without a
winner, the game ends in a draw.

## Card Types

**Character** — Permanent cards that enter the battlefield when they resolve.
Each character has a spark value used in combat during the Judgment phase.
Characters enter your reserves as reserved characters and can be deployed on
subsequent turns. Characters remain on the battlefield until removed by an
effect (Dissolve or Banish) or defeated in combat. Surviving deployed characters
remain where they are after Judgment; they do not automatically return to your
reserves. They can have triggered, activated, and static abilities. Characters
have subtypes (Mage, Warrior, Robot, etc.) that other cards can reference.

**Event** — One-shot cards that produce an effect when they resolve, then move
to the void. Events can be marked as "fast," allowing them to be played during
the opponent's turn or in response to other cards on the stack.

**Dreamcaller** — A player's identity card, an animated 3D character that starts
each battle already in play. Dreamcallers provide powerful ongoing abilities
(static, triggered, or activated) that define a player's playstyle. Each
Dreamcaller also has an Awakening number, which is the turn on which that
Dreamcaller's effects become active. For example, a Dreamcaller with Awakening 4
and "Judgment: Draw a card" would begin applying that ability starting on turn
4\. Primarily chosen during quest mode.

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

**Battlefield** — Where characters reside. Each player has a staggered
battlefield with 4 deployed lanes (`D0-D3`) in front and 5 reserve slots
(`R0-R4`) behind, for 9 total positions. Dreamtides does **not** use columns.
Because the grid is staggered, a deployed lane sits in front of one or two
reserve slots: `D0` is in front of `R0` and `R1`, `D1` is in front of `R1` and
`R2`, `D2` is in front of `R2` and `R3`, and `D3` is in front of `R3` and `R4`.
`D0` and `R0` are not a column, and `D0` and `R1` are not a column either.

- `R0` supports `D0`
- `R1` supports `D0` and `D1`
- `R2` supports `D1` and `D2`
- `R3` supports `D2` and `D3`
- `R4` supports `D3`

Only deployed characters participate directly in Judgment phase combat. A player
can have at most 9 total characters on the battlefield, and new characters
always enter the reserves as reserved characters.

**Void** — The discard pile. Events go here after resolving. Characters go here
when dissolved. Some cards can interact with cards in the void (notably via
Reclaim).

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

1. **Judgment** — Start-of-turn trigger window and combat resolution. Judgment
   abilities trigger first. Then the active player's deployed characters attack
   and the non-active player's deployed characters block. Each deployed lane
   (`D0-D3`) is resolved independently (see Spark and Scoring).
2. **Dreamwell** — The active player draws the next Dreamwell card, permanently
   increasing their energy production and resetting their current energy. Any
   bonus effect on the card is applied.
3. **Draw** — The active player draws one card from their deck. (Skipped on the
   very first turn of the game.)
4. **Main** — The active player can play cards from hand, activate abilities,
   deploy or reserve characters by moving them between the battlefield's 9
   positions, and take other actions. This is the primary action phase.
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

Spark is the primary stat on characters. Characters have no health or toughness
— spark is their only stat. When an effect modifies a character's spark,
including support-based effects from other characters, that effective spark is
what Judgment, scoring, and other game rules use.

**Attackers and blockers:** The active player's deployed characters are the
attacking side during Judgment. The non-active player's deployed characters are
the blocking side. Combat happens only in the four deployed lanes.

- If both players have a deployed character in the same lane, they are paired
  for combat in that lane.
- If only the active player has a deployed character in a lane, that attacker is
  unblocked and can score points.
- If only the non-active player has a deployed character in a lane, nothing
  happens in that lane.

**Judgment phase resolution:** During the Judgment phase at the start of each
turn, the active player's deployed characters are the attackers and the
non-active player's deployed characters are the blockers. Each deployed lane
(`D0-D3`) is resolved independently:

- **Attacker with a blocker (paired judgment):** Compare their spark values. The
  character with lower spark is dissolved. If both have the same spark, both are
  dissolved. A paired attacker does **not** score points. Dissolved triggers
  fire after each lane is resolved.
- **Attacker with no blocker (unblocked):** The attacker scores victory points
  equal to its spark value for the attacking player.
- **Only the non-active player has a character in the lane:** Nothing happens —
  the non-active player's deployed characters are blockers, not attackers.
- **Neither player has a character in the lane:** Nothing happens.

**After Judgment:** Surviving characters stay where they are. There is no
automatic return to the reserves after combat, so a surviving deployed character
remains in that lane until it is moved or removed.

Your reserves are safe during Judgment — reserved characters do not directly
fight and do not score points, though their abilities can still affect deployed
characters they support.

**Entering reserved:** When a character enters the battlefield, it is placed in
your reserves and enters reserved. A reserved character cannot be deployed. This
temporary reserved status wears off at the start of the controlling player's
next turn.

**Repositioning:** During the Main phase, a player can freely reposition their
characters between deployed lanes and reserve slots, and between positions on
the same row of the battlefield, subject to the reserved condition. Moving a
character onto an occupied position swaps the two characters. Characters cannot
be moved outside the Main phase, and no cards can be played during Judgment.

**Materializing new characters:** Characters always enter the battlefield in the
reserves. If all 5 reserve slots are occupied, no additional characters can be
played or materialized until a reserve slot is freed, even if the player has
open deployed lanes.

**Spark modification:** Spark may be modified by card effects before Judgment,
but once Judgment begins, no new cards can be played in response.

**Character limit:** Each player can have at most 9 characters on the
battlefield at once. If the battlefield is full, additional characters cannot be
played.

## Keywords and Effects

**Dissolve** — Destroy a target character, moving it from the battlefield to the
void. Fires the "Dissolved" trigger. Can target any character (yours or the
opponent's).

**Banish** — Permanently remove a card from the game by sending it to the
Banished zone. Several variants exist: banish from the battlefield, banish from
the void, banish until the banishing card leaves play, and banish until the next
main phase.

**Materialize** — Put a character into your reserves. This is the term for a
character entering play, whether from hand (played normally), from the void (via
Reclaim or effects), from the deck (via effects), or as a token (Figments).
Characters enter reserved and cannot be deployed on the turn they are
materialized. Materialize requires an empty reserve slot.

**Supported / Supporting** — These terms describe the staggered adjacency
between the 5 reserve slots and 4 deployed lanes. A reserved character's
**supported** characters are the deployed characters in the lanes its slot
supports. A deployed character's **supporting** characters are the reserved
characters behind it. On the standard battlefield, `R0` supports `D0`, `R1`
supports `D0/D1`, `R2` supports `D1/D2`, `R3` supports `D2/D3`, and `R4`
supports `D3`; equivalently, `D0` is supported by `R0/R1`, `D1` by `R1/R2`, `D2`
by `R2/R3`, and `D3` by `R3/R4`. Support has no built-in effect by itself, but
abilities can reference these relationships.

**Prevent** — Counter a card on the stack, sending it to the void without
resolving. Prevent effects are always fast (they must be played in response to a
card on the stack).

**Abandon** — Move one of your own characters from the battlefield to the void.
Cannot be prevented and only targets your own characters. Fires the "Dissolved"
trigger. Often used as a cost for abilities.

**Kindle N** — Add N spark to your character with the highest spark value. If
there is a tie, the oldest character (earliest materialized) is chosen.

**Foresee N** — Look at the top N cards of your deck. You may reorder them in
any order and optionally send any of them to the void.

**Reclaim** — A named ability that allows you to play a card from your void
instead of from your hand. The card is played at its normal cost (or at a
specified alternate cost: Reclaim N means it costs N energy when played from the
void). When a reclaimed card later leaves play, it is banished instead of
returning to the void.

**Fast** — A property on cards and abilities indicating they can be used outside
normal main phase timing: during the opponent's Main phase, during the Ending
phase, or in response to cards on the stack.

**Discover** — Look at 3 cards from your deck that match a specified criteria,
then choose one to add to your hand.

**Copy** — Create a duplicate of a card or effect. Variants include copying a
character on the battlefield or copying the next card played.

**Gain Control** — Take control of an opponent's character, moving it to your
side of the battlefield.

**Test** — Initiate a one-on-one judgment between your character and a target
character. The two characters compare spark as in a normal paired judgment — the
character with lower spark is dissolved, and if both have the same spark, both
are dissolved. No points are scored from a test.

**Dread N** — During judgment, this character dissolves opposing characters as
though its spark were N higher. The bonus applies only to the paired spark
comparison, not to points scored if unblocked.

**Preeminence** — This character wins spark ties in judgment. If both characters
in a paired judgment have preeminence, both are dissolved as normal.

**Unbound** — This character enters deployed instead of entering your reserves,
and it does not enter reserved — it can attack or block on the turn it is
materialized.

**Unstoppable** — This character scores victory points equal to its spark even
when blocked. The paired spark comparison still occurs as normal.

**Veil X** — This character costs X additional energy for the opponent to target
with cards or abilities.

**Reserve** — Keep a character in your reserves. A reserved character cannot be
deployed, does not attack or block during the next Judgment phase, and is
treated as absent during that phase. New characters enter reserved, and if an
effect or game rule says a character becomes reserved, that restriction lasts
until the start of its controller's next turn.

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
the Judgment phase at the start of each turn), and **Dissolved** (fires when the
character is destroyed). Triggered abilities can also use descriptive conditions
like "When you play a card" or "At end of turn." Characters can have combined
triggers such as "Materialized, Judgment" (fires both on entry and each Judgment
phase).

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
deck. Figments enter the battlefield through "Materialize Figments" effects and
behave like regular characters — they have spark values, count toward the
character limit, and can be targeted by effects.

## Supplement: Reading Dreamtides Through an MTG Lens

Dreamtides is often easiest for Magic: The Gathering players to learn by
thinking of it as an MTG-like rules engine whose biggest changes are a shared
mana system and a positional combat board.

- The Dreamwell plays the same strategic role that lands and mana development
  play in MTG, but it does so automatically and symmetrically. You still care
  about curve, tempo, and resource scaling, but not about mana screw or land
  count.
- Materialize, void, banish, prevent, fast, and reclaim are close cousins of
  "enters the battlefield," graveyard, exile, counterspell, flash, and casting
  from the graveyard.
- The largest shift is combat. Dreamtides moves much of what MTG handles during
  combat declaration into battlefield positioning during the previous Main
  phase.

Judgment is the closest Dreamtides analog to MTG combat, but it compresses
several MTG steps into one automatic resolution:

- The active player's deployed characters are the attackers.
- The non-active player's deployed characters are the blockers.
- Each deployed lane is effectively a predeclared combat pairing. Repositioning
  during Main is where you decide which creatures will attack, which ones will
  block, and which lanes will be contested.
- An unblocked attacker scoring points equal to its spark plays the role that
  combat damage to the defending player plays in MTG.
- A blocked lane behaves like a simplified one-attacker/one-blocker combat:
  compare spark, dissolve the weaker character, and dissolve both on a tie.
- Because no cards can be played during Judgment, Dreamtides has no direct
  analog to MTG's combat-trick window inside combat damage. The relevant choices
  happen before Judgment starts.

The reserve row is also important to the analogy. A reserved character is still
on the battlefield, but it is not currently participating in combat. In MTG
terms, it is closer to a creature you have committed to board presence and
synergy but not to the present combat exchange. Many Dreamtides cards are best
understood as asking, "Should this body be deployed for immediate Judgment
pressure, or left in reserve as support for later turns?"

## Appendix: Translating MTG Mechanics into Dreamtides

This appendix is about design translation, not strict rules identity. Some MTG
mechanics map directly onto existing Dreamtides rules text; others map only at
the level of play pattern. Unless noted otherwise, "evergreen" here refers to
the evergreen MTG keyword abilities that players most commonly use as shorthand
when comparing games.

### Evergreen MTG keyword abilities

- **Deathtouch** — In Dreamtides, the clean translation is "wins any paired
  judgment or Test it touches" or "when this pairs in Judgment, dissolve the
  opposing character." Because Dreamtides has no damage persistence,
  deathtouch-style design is about guaranteed kills, not 1-damage lethality.
- **Defender** — Best translated as a reserve-focused character that cannot be
  deployed or cannot score. Dreamtides makes "wall" gameplay by asking a
  character to support lanes without entering Judgment.
- **Double strike** — Usually becomes "wins its lane twice" rather than "deals
  damage twice": for example, an extra Judgment trigger, an extra Test, or "if
  this survives paired judgment, it also scores its spark."
- **First strike** — Dreamtides has no first-damage step, so the closest
  translation is "dissolve the opposing character before it can trade" or, in a
  lighter form, a built-in combat edge such as `Preeminence`.
- **Flash** — Very close to `Fast`. A flash creature in MTG is usually a `Fast`
  character or a materialize effect that can be used off-turn.
- **Flying** — Dreamtides has no separate air layer, so the right translation is
  evasive blocking restrictions, not literal altitude: "this can be blocked only
  by special defenders" or "this is usually unblocked unless answered by a
  specific kind of character."
- **Haste** — Usually `Unbound`, or any text that lets a new character enter
  deployed and skip `Reserve`.
- **Hexproof** — Usually softens into `Veil X` rather than total
  untargetability, because Dreamtides generally wants answers to remain
  possible.
- **Indestructible** — Usually "cannot be dissolved" or a recurring
  dissolve-prevention shield.
- **Lifelink** — Life gain becomes race swing. In Dreamtides, lifelink usually
  maps to gaining points or energy when this scores, wins Judgment, or dissolves
  an opposing character.
- **Menace** — Because one blocker per lane is already the norm, menace becomes
  "blocking this is unusually costly or restricted," not "needs two blockers." A
  Dreamtides version usually taxes blockers or makes the lane hard to contest.
- **Reach** — Only matters if a card set introduces flying-like evasion. Use it
  as the permission to block those special evasive attackers.
- **Trample** — `Unstoppable` is the clean analog: the character still gets its
  spark through even when paired.
- **Vigilance** — This is close to baseline Dreamtides behavior. A deployed
  character already attacks on your turns and blocks on the opponent's turns
  without tapping, so many vigilance play patterns require no extra rules text.
- **Ward** — `Veil X` is the direct analog.

### Non-evergreen MTG mechanics that translate well

- **Prowess** — "Whenever you play an event, this gets +1 spark this turn" or a
  Judgment-only spark boost. Prowess was once evergreen in MTG, but it is now
  better treated as a recurring non-evergreen translation tool.
- **Flashback** — `Reclaim` is the closest existing translation.
- **Kicker** — Extra-energy rider text or modal abilities on play translate
  cleanly.
- **Cycling** — "Pay N, void this from hand: draw a card" is a natural fit for
  Dreamtides hand smoothing.
- **Convoke** — Using supporting or reserved characters to reduce a card's
  energy cost is the most natural Dreamtides version.
- **Delve** — Banish cards from your void to reduce energy cost.
- **Unearth** — Materialize from the void with `Unbound` or temporary
  deployment, then banish the character when it leaves play. This is especially
  close to `Reclaim` on characters.
- **Escape** — Repeatable `Reclaim` with an additional void-banish payment.
- **Surveil** — A more void-oriented `Foresee`: look at cards, keep some on top,
  send some to the void.
- **Exert** — Attack now, then become `Reserved` or skip your next Judgment.
- **Cascade** — `Discover`, then immediately play or materialize the found card.
- **Exploit** — "Materialized: you may `Abandon` an ally. If you do, ..." is
  almost a literal Dreamtides translation.
- **Riot** — Choice between extra spark and `Unbound` / immediate deployment.
- **Ninjutsu** — Replace an unblocked attacker with a character from hand or
  void after lane commitments are known. Dreamtides positioning makes this
  especially intuitive.
- **Afterlife** — `Dissolved: Materialize` figment(s).
- **Dash** — Materialize deployed and `Unbound` for immediate pressure, then
  return the character to hand or reserve instead of leaving it fully committed.
- **Suspend** — Exile a card into a delayed state, then materialize or play it
  after a fixed number of turns or at a future Judgment.

Mechanics tied tightly to MTG lands, tapping, or multi-blocker combat usually
need the most reinterpretation. In general, Dreamtides translations work best
when they preserve the play pattern the MTG mechanic creates: burst tempo,
evasion, recursion, lane pressure, support synergy, or race compression.
