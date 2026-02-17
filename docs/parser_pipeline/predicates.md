# Predicate and Targeting System

The predicate system translates natural-language targeting text on Dreamtides
cards into structured AST nodes. When a card says "Dissolve an enemy character"
or "Banish another character with cost 3 or less," the parser produces a
two-layer AST from the Predicate and CardPredicate enums. The architecture is
split across four files: predicate.rs for type definitions, predicate_parser.rs
for the ownership-scope layer, card_predicate_parser.rs for the type-constraint
layer, and predicate_suffix_parser.rs for numeric and keyword suffixes.

## Two-Layer Architecture

The outer layer, Predicate, answers "whose card?" It captures ownership scope:
whether the target belongs to you, the opponent, either player, or is a
back-reference to a previously mentioned card. The inner layer, CardPredicate,
answers "what kind of card?" It captures type constraints: character, event,
specific subtype, or a card meeting some numeric threshold on cost or spark.

Most Predicate variants wrap a CardPredicate as their payload. The four
reference variants (This, It, Them, That) carry no inner CardPredicate because
they refer to a specific card already identified by context. This two-layer
split means the predicate_parser identifies the ownership keyword ("enemy",
"another", "allied", or nothing) and then delegates to card_predicate_parser for
the type constraint. The card_predicate_parser never concerns itself with
ownership, keeping both parsers small and composable.

## The Predicate Enum

The Predicate enum is defined in ability_data/src/predicate.rs.

- **This** matches the card that owns the ability. Parsed from "this character",
  "this event", or "this card." It carries no inner CardPredicate because the
  target is always the specific card bearing the ability text.
- **It** is a back-reference to a card targeted by a prior effect in the same
  ability. Parsed from the word "it" and used in chains like "banish a
  character, then materialize it." The referenced card is resolved at runtime.
- **Them** is the plural counterpart of It. Parsed from "them" for patterns like
  "banish any number of cards, then materialize them."
- **That** refers to the card that triggered a triggered ability. Parsed from
  "that character", "that event", or "that card" in effect clauses following a
  trigger, such as "when you materialize a spirit animal, that character gains
  +1 spark."
- **Enemy** wraps a CardPredicate and matches cards controlled by the opponent.
  Parsed from "enemy" followed by an optional CardPredicate; bare "enemy"
  defaults to Character. The pattern "non-Warrior enemy" is handled by a
  dedicated parser producing Enemy(NotCharacterType(subtype)).
- **Another** wraps a CardPredicate and matches cards controlled by the ability
  owner, excluding the owning card itself. Parsed from "another" plus a required
  CardPredicate, or from "ally"/"allies" plus an optional CardPredicate
  (defaulting to Character), or from "allied" plus a required CardPredicate. The
  exclusion of self is the key distinction from Your.
- **Your** wraps a CardPredicate and matches any card controlled by the ability
  owner, including the owning card. The main predicate parser never produces
  Your directly. It is only created by the trigger parser's dual-parse pattern
  (described below), reflecting the convention that trigger contexts default to
  "your own card."
- **Any** wraps a CardPredicate and matches any card regardless of controller.
  It is the default ownership scope: bare "character" becomes Any(Character),
  bare "event" becomes Any(Event). The phrase "enemy or ally" also maps to
  Any(Character).
- **AnyOther** wraps a CardPredicate and matches any card from either player
  except the owning card. Used in counting contexts like "for each other
  character" where the source card should not count itself.
- **YourVoid** wraps a CardPredicate and matches cards in the owner's void
  (discard pile). Parsed from a CardPredicate followed by "in your void."
- **EnemyVoid** wraps a CardPredicate and matches cards in the opponent's void.
  Parsed from a CardPredicate followed by "in the opponent's void."

The distinction between Another and Your is one of the most important semantic
boundaries. Another always excludes the owning card, making it appropriate for
"ally" effects. Your includes the owning card and is reserved for trigger
contexts where the triggering card might be the ability's owner. The distinction
between Any and AnyOther follows the same self-exclusion pattern but across both
players' cards.

## The CardPredicate Enum

CardPredicate variants fall into three complexity tiers.

**Simple type predicates** are leaf nodes. Card matches any card type. Character
matches character-type cards and is the most common variant, serving as the
default when no explicit type is specified. Event matches event-type cards.
CharacterType takes a CardSubtype and matches characters of that subtype.
NotCharacterType matches characters not of a given subtype and only appears in
the "non-Subtype enemy" pattern. CharacterWithMaterializedAbility matches
characters with a "materialized" keyword ability.
CharacterWithMultiActivatedAbility matches characters with an activated ability.
Both keyword-ability variants require an explicit base type in the parser (they
do not use the or_not() optional-base pattern).

**Numeric constraint predicates** carry a boxed target CardPredicate (the base
type being filtered) plus an Operator and a comparison value or reference.
CharacterWithSpark compares spark against a fixed value. CardWithCost compares
cost against a fixed energy value and is the most common numeric predicate.
CharacterWithCostComparedToControlled compares cost against the count of allied
characters of a specific subtype. CharacterWithCostComparedToAbandoned compares
cost against an abandoned ally's cost. CharacterWithSparkComparedToAbandoned
compares spark against an abandoned ally's spark.
CharacterWithSparkComparedToAbandonedCountThisTurn compares spark against the
number of allies abandoned this turn. CharacterWithSparkComparedToEnergySpent
compares spark against the energy paid. CharacterWithCostComparedToVoidCount
compares cost against the number of cards in the owner's void.

**Wrapper predicates** create nesting. Fast wraps another CardPredicate and
requires the fast keyword; it is the source of the parser's only recursion.
CouldDissolve wraps a full Predicate (not a CardPredicate) and matches events
that could dissolve the specified target. It is the only CardPredicate variant
that wraps a Predicate, necessary because the dissolution target has its own
ownership scope.

The Operator enum covers: LowerBy (relative decrease), OrLess (absolute upper
bound), Exactly (exact match), OrMore (absolute lower bound), and HigherBy
(relative increase). Two families of operator parsers exist: absolute operators
("or less", "or more", "higher", "lower") for fixed-value suffixes, and
comparison operators ("less than", "greater than", "equal to") for "compared to"
suffixes. Notably, "less than" maps to OrLess and "greater than" maps to OrMore,
so the runtime uses inclusive comparison despite the English wording.

## The card_predicate_parser

The card_predicate_parser is the only recursive parser in the system. It uses
Chumsky's recursive() combinator, with the recursion enabling the fast directive
prefix to wrap any other CardPredicate including another Fast.

The parser works in two steps. First, it defines five base parsers tried in
order: fast (takes the recursive handle, enabling nesting), subtype, character,
event, and card. Second, it tries ten alternatives combining bases with
suffixes. The first six pair base.or_not() with a suffix parser, meaning the
base type is optional and defaults to Character when absent. The next two pair a
required base with the materialized-ability and activated-ability suffixes. The
ninth is the standalone which_could_dissolve_suffix. The tenth is the bare base
with no suffix as the fallback.

The optional-base-plus-suffix pattern is the critical design element. "With cost
3 or less" (no explicit type) parses identically to "character with cost 3 or
less" because the absent base defaults to Character. The priority ordering
follows a longest-match-first strategy: complex "compared to" suffixes are tried
before simpler absolute-value suffixes because both begin with "with cost" or
"with spark," and the simpler parser would consume the prefix tokens if tried
first.

## Predicate Suffix System

The predicate_suffix_parser module provides suffix functions in three
categories.

**Absolute numeric constraints** compare cost or spark against a fixed value.
The with_cost_suffix parses "with cost" followed by an energy value and an
optional operator ("or less", "or more", "higher", "lower", or nothing for
exactly). For "higher" and "lower", the energy value is embedded in the HigherBy
or LowerBy variant. The with_spark_suffix parses "with spark" followed by a
spark value and a required operator ("or less" or "or more").

**Comparative constraints** compare cost or spark against a dynamic game-state
quantity. The with_cost_compared_to_controlled_suffix parses "with cost
less/greater/equal to the number of allied Subtype" and returns an operator plus
a CharacterType predicate representing what to count. The
with_cost_compared_to_void_count_suffix compares cost against the number of
cards in your void. The with_spark_compared_to_abandoned_suffix compares spark
against "that ally's spark." The with_spark_compared_to_energy_spent_suffix
compares spark against "the amount of energy paid."

**Keyword ability constraints** check for specific abilities. The
with_materialized_ability_suffix parses "with a materialized ability" (handling
optional quote marks around the directive). The with_activated_ability_suffix
parses "with an activated ability."

The which_could_dissolve_suffix stands apart: it parses "event which could
dissolve an ally" and returns a fixed Predicate value of Another(Character). It
is the only suffix that produces a full Predicate and the only one not paired
with a base predicate in the choice list.

## The Trigger Parser's Dual-Parse Pattern

In triggers like "when you play a character," the card being played is always
the player's own. However, the main predicate_parser maps bare "character" to
Any (either player). The trigger parser resolves this mismatch with a dual-parse
pattern: it first tries card_predicate_parser mapped to Predicate::Your, then
falls back to predicate_parser.

This pattern appears in six triggers: play_trigger, play_from_hand_trigger,
play_during_turn_trigger, play_during_opponent_turn_trigger, discard_trigger,
and materialize_trigger. It ensures "when you play a character" produces
Your(Character) while an ownership-qualified trigger would correctly fall
through to predicate_parser.

Triggers describing third-party events (dissolved_trigger, banished_trigger,
leaves_play_trigger, put_into_void_trigger) do not use this pattern. They call
predicate_parser directly because the affected card could belong to either
player.

## Where Predicates Appear

Predicates are consumed in six distinct contexts across the parser.

- **Triggers** use predicates to describe what must be played, materialized, or
  dissolved for the trigger to fire, with Your as the default for player
  actions.
- **Effects** use predicates for dissolve, banish, materialize, gain control,
  return to hand, and spark targets, commonly producing Enemy, Another, Any,
  This, It, Them, and That.
- **Costs** use predicates to describe what must be sacrificed, typically
  Another for "abandon an ally."
- **Conditions** use CardPredicate (not full Predicate) to check board state,
  such as whether a specific subtype is controlled.
- **Static abilities** use CardPredicate to describe which cards are affected by
  ongoing rule modifications like cost reductions or spark bonuses.
- **Quantity expressions** use both Predicate and CardPredicate for "for each"
  counting, such as Another(CharacterType(Warrior)) for "each allied Warrior."

## Common Targeting Pitfalls

**Your vs Any confusion.** The predicate_parser maps bare type words to Any. In
trigger contexts the correct semantic is Your. Anyone adding a new trigger must
use the dual-parse pattern (card_predicate_parser mapped to Your tried before
predicate_parser). Omitting it causes triggers to match both players' cards.

**Another vs Your.** "Ally", "allies", "allied", and "another" all map to
Another, which excludes self. Your includes self but is never produced by the
predicate_parser; it only comes from the trigger parser. When adding effects
targeting friendly characters, use Another.

**CardPredicate defaults to Character.** When a suffix uses base.or_not() and no
base is present, the default is Character. "Enemy with cost 3 or less" (no
explicit type after "enemy") parses as a character with that cost constraint.
This is intentional but can surprise when card text omits the type word.

**Suffix ordering matters.** Complex suffixes starting with "with cost" or "with
spark" must precede simpler ones in the choice list. The simpler parser would
consume the shared prefix tokens, leaving comparison tokens unparseable.

**Specific-before-general in predicate_parser.** Ownership keywords (enemy,
another, allied, ally) are tried before general fallbacks (bare type words, void
parsers). New ownership keywords must go in specific_predicates to avoid being
shadowed.

**CouldDissolve is unique.** It wraps a full Predicate, not a CardPredicate, and
its parser currently hardcodes Another(Character) as the dissolution target.

**Reference predicates carry no type info.** This, It, Them, and That discard
any following type word and produce the bare reference variant, since the
referenced card is resolved at runtime.
