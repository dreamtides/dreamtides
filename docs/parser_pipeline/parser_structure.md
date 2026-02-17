# Chumsky Parser Structure

This document describes the Chumsky combinator parser that consumes resolved
token streams and produces Ability ASTs. It covers the five ability types,
effect composition, effect sub-parser modules, trigger and cost parsing,
condition patterns, and the directive-vs-word keyword distinction.

## 1. Parser Architecture

The parser operates on slices of (ResolvedToken, SimpleSpan) tuples, defined as
the type alias ParserInput in parser_helpers.rs. By this stage, directive tokens
have been transformed into semantically typed ResolvedToken values. The parser
never sees raw text or unresolved directives.

The top-level entry point is the ability_parser function in ability_parser.rs,
which uses choice() over five alternatives tried in priority order. Each
alternative delegates to a specialized parser module and maps the result into
the corresponding Ability enum variant.

A single parser instance is constructed once and reused across all card entries
during batch processing. The test infrastructure caches the parser in a
thread-local to avoid expensive repeated construction. The stacker crate
provides 4MB of additional stack space to accommodate the deep combinator
hierarchy.

## 2. The Five Ability Types

The ability_parser tries five ability parsers in this priority order. Order
matters because Chumsky's choice() commits to the first matching alternative.

**Triggered abilities** are tried first. A triggered ability begins either with
"once per turn, " followed by a trigger event and an effect, or with a trigger
event directly followed by an effect. Trigger events start with keywords like
"when" or "at", or with keyword directives such as materialized, judgment, or
dissolved. The triggered_parser module combines the trigger event with the
effect and produces a TriggeredAbility struct carrying optional once_per_turn
and until_end_of_turn flags.

**Activated abilities** are tried second. Their grammar is: an optional fast
directive, an optional "--" separator, one or more costs separated by commas
(where "once per turn" can appear as an option modifier in place of a cost), a
colon, and an effect. The result is an ActivatedAbility with a costs vector, an
effect, and optional ActivatedAbilityOptions carrying is_fast and is_multi
flags.

**Named abilities** are tried third. The smallest parser module, recognizing
only two patterns: reclaim_for_cost (a reclaim directive, "--", then a cost) and
plain reclaim (a ReclaimCost resolved token).

**Static abilities** are tried fourth. The parser recognizes many patterns
covering continuous rule modifications in three structural forms: a condition
prefix then a standard static ability, a standard static ability then "if" with
a condition suffix, or a standalone standard static ability. Patterns include
play-only-from-void, cards-in-void-have-reclaim, additional-cost-to- play,
once-per-turn-play-from-void, alternate cost patterns, characters-in-
hand-have-fast, disable-enemy-materialized-abilities, has-all-character-types,
allied-spark-bonus, spark-equal-to-predicate-count, cost modifications,
reveal-top-card-of-deck, play-from-top-of-deck, and judgment-triggers-when-
materialized.

**Event abilities** are the fallback, tried last. Any token sequence that does
not match the preceding four types is treated as an event effect. The parser
delegates to effect_or_compound_parser and wraps the result in EventAbility.

## 3. Effect Composition

The effect_parser module provides a two-level dispatch system separating atomic
effect recognition from compound effect wrapping.

**Level one: single_effect_parser.** Recognizes one atomic effect. It first
tries create_trigger_until_end_of_turn ("until end of turn, TRIGGER EFFECT"),
then falls through to base_single_effect_parser, a choice() over the five effect
sub-parser modules in this order: card_effect_parsers, control_effects_parsers,
game_effects_parsers, resource_effect_parsers, spark_effect_parsers.

**Level two: effect_or_compound_parser.** Wraps single effects with compound
structure, trying six alternatives in order:

- **Modal effect.** Recognizes choose_one, a newline, then two bulleted mode
  lines each with a mode-specific energy cost, a colon, effects, and a period.
  Produces Effect::Modal.

- **Optional effect with trigger cost.** "you may COST to EFFECTS." Wraps with
  optional true and the trigger cost attached.

- **Mandatory effect with trigger cost.** "COST to EFFECTS." Same as above but
  with optional false. Tried after the optional variant because both start with
  a trigger cost.

- **Optional effect.** "you may EFFECTS." Wraps with optional true, no trigger
  cost.

- **Conditional effect.** "CONDITION EFFECTS." Attaches the condition from
  condition_parser to the effect.

- **Standard (plain) effect.** The fallback: one or more effects separated by
  effect separators, terminated by a period. Produces Effect::Effect or
  Effect::List.

**Effect separators.** Individual effects within a list are separated by a
period, a comma followed by the word "then", or the word "and".

**Trigger costs.** The trigger_cost_parser recognizes five cost types: abandon,
banish from your void, banish from opponent's void, pay energy, and discard.

## 4. The Five Effect Sub-Parser Modules

Each module lives under parser/effect/ and exports a parser() function returning
a choice() over its recognized patterns. All return StandardEffect variants.

| Module                  | Domain                                                                            | Key ResolvedTokens Used                                                            |
| ----------------------- | --------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| card_effect_parsers     | Drawing, discarding, energy/point gain, void manipulation, reclaim                | CardCount, DiscardCount, Energy, PointCount, ReclaimCost, UpToNEvents, Subtype     |
| spark_effect_parsers    | Spark: kindle, spark gain, spark-for-each, spark-becomes                          | KindleAmount, SparkAmount                                                          |
| control_effects_parsers | Ownership transfer, deck manipulation, ability disabling                          | None (words and predicates only)                                                   |
| resource_effect_parsers | Multiplier effects, point loss, opponent point gain                               | Number, PointCount                                                                 |
| game_effects_parsers    | Foresee, discover, dissolve, banish, materialize, counterspell, copy, extra turns | ForeseeCount, CardCount, Number, Figment, FigmentCount, UpToNAllies, ThisTurnTimes |

**card_effect_parsers.** Handles card movement and resource acquisition: "draw"
plus CardCount for drawing, "draw" plus card predicate and "from your deck" for
matching draws, "discard" plus DiscardCount, "gain" plus Energy or PointCount,
"put the CARDS of your deck into your void" for milling, the reclaim directive
plus predicate for void-to-play, and symmetric "each player" effects. The "for
each" variants of draw, energy gain, and point gain delegate to the quantity
expression parser. Reclaim-granting parsers handle patterns like "it gains
reclaim equal to its cost this turn" and "all cards in your void gain reclaim".

**spark_effect_parsers.** Kindle is a single KindleAmount token with no
surrounding keywords (the directive itself carries the keyword text). Single-
target spark gain matches a predicate, "gains +", SparkAmount, "spark". The
for-each variant extends with a quantity expression. Each-allied uses "have each
allied" as prefix. Each-gains-for-each uses "each CARD_PREDICATE gains spark
equal to the number of CARD_PREDICATE". Spark-becomes matches "the spark of each
allied CARD_PREDICATE becomes N".

**control_effects_parsers.** The smallest module, using no typed ResolvedToken
variants. Gain-control: "gain control of" plus predicate. Put-on-top: "put" plus
predicate plus "on top of the opponent's deck". Disable-activated-abilities:
"disable the activated abilities of" plus predicate plus "while this character
is in play".

**resource_effect_parsers.** Two multiplier parsers start with a Number token
and produce CreateStaticAbilityUntilEndOfTurn wrapping temporary rule
modifications. Multiply-your-energy produces an immediate effect instead. Lose-
points matches "you lose" plus PointCount. Enemy-gains-points matches "the
opponent gains" plus PointCount.

**game_effects_parsers.** The largest module. Foresee is a single ForeseeCount
token. Discover uses the discover directive plus card predicate, with optional
"and materialize it" continuation. Counterspell has three variants (prevent
played, prevent that card, prevent-unless-pays-cost). Dissolve has three
variants by specificity (all-with-predicate, all-characters, single). Banish has
ten parsers covering combinations of from-void, up-to-N, any-number,
then-materialize, until-leaves-play, until-next-main, and simple banish.
Materialize has seven: at-end-of-turn, random-from-deck, collection, copy-of,
figments-quantity, figments, and simple. Also includes copy-next- played,
copy-it, shuffle-and-draw, trigger-judgment, extra turn, and you-win-the-game.

## 5. Ordering Sensitivity

Chumsky's choice() tries alternatives left-to-right and commits to the first
match. A less specific parser tried first will consume tokens that a more
specific parser needed, producing incorrect ASTs or parse failures.

The principle is specific-before-general. Concrete examples:

- dissolve_each_character before dissolve_all_characters before
  dissolve_character. Without this, the simplest variant would match the prefix
  of the more constrained patterns.

- The ten banish parsers place compound variants (banish-then-materialize,
  banish-until) before simple banish_character. Otherwise the simple parser
  would consume the banish-plus-predicate prefix and leave ", then materialize
  it" unparseable.

- materialize_character_at_end_of_turn before materialize_character, since the
  latter would match the prefix and stop before "at end of turn".

- gains_spark_for_each before gains_spark, and draw_cards_for_each before
  draw_cards, because the for-each variants extend the shorter patterns.

- optional_effect_with_trigger_cost before effect_with_trigger_cost, since the
  optional variant has the longer "you may" prefix.

When choice() exceeds its maximum tuple arity, alternatives split into nested
choice() groups with .boxed(). Ordering applies across groups: all alternatives
in an earlier group are tried before any in a later group.

## 6. Trigger Parser

The trigger parser (trigger_parser.rs) recognizes events that cause triggered
abilities to fire. Its entry point tries keyword triggers first, then standard
triggers terminated by a comma.

**Keyword triggers.** Three single-keyword directives: judgment, materialized,
dissolved. Two combined directives: materialized_judgment and
materialized_dissolved (firing on multiple events). These produce
TriggerEvent::Keywords.

**Standard triggers** are organized into four groups:

- **Play triggers** (six variants): play-cards-in-turn, play-from-hand,
  play-during-opponent-turn, play-during-turn, opponent-plays, and basic play.
  More specific suffixed variants come before the generic play trigger.

- **Action triggers** (four variants): abandon-cards-in-turn, discard,
  materialize, and abandon. All begin with "when you".

- **State change triggers** (four variants): dissolved ("when a PREDICATE is
  dissolved"), banished, leaves-play, and put-into-void.

- **Timing triggers** (four variants): draw-cards-in-turn, draw-all-cards (no
  cards in deck), end-of-turn, and gain-energy.

**The dual-parse pattern for Predicate::Your.** The predicate parser maps bare
card types ("character") to Predicate::Any, but trigger contexts like "when you
play a character" require Predicate::Your. The trigger parser solves this with a
choice that tries card_predicate_parser().map(Predicate::Your) before
predicate_parser::predicate_parser(). A bare card type is captured as
Predicate::Your first, falling through to the full predicate parser only for
other ownership scopes like Enemy or Another.

## 7. Cost Parser

The cost parser (cost_parser.rs) recognizes costs for activated abilities and
trigger costs for compound effects. Its entry point tries a cost choice (two or
more costs separated by "or", producing Cost::Choice) before falling back to a
single cost.

**Single cost types** in the single_cost_parser, tried in order:

- Spend-one-or-more-energy: "pay 1 or more" plus energy_symbol directive
- Energy cost: a bare Energy resolved token
- Abandon this character: fixed phrase, targeting Predicate::This
- Abandon cost: three sub-variants (any-number, with-count, single)
- Return to hand: with collection expression or article plus predicate
- Discard hand: fixed phrase "discard your hand"
- Discard cost: DiscardCount token or article plus predicate
- Banish void with min count: banish directive, "your void with" Count "or more
  cards"
- Banish from your void: banish directive plus predicate "in your void"

**Once-per-turn modifier.** In activated abilities, "once per turn" can appear
in the cost list. It is parsed as None and sets the once_per_turn option flag
rather than adding to the costs vector.

**Exported cost parsers.** Several functions are exported for use by other
modules: abandon_cost_for_trigger, abandon_cost_single, banish_from_hand_cost,
lose_maximum_energy_cost, banish_cards_from_your_void_cost,
banish_cards_from_enemy_void_cost, return_to_hand_cost, pay_energy_cost, and
discard_cost.

## 8. Condition Parser

The condition parser (condition_parser.rs) recognizes conditions that gate
effects or modify static abilities, trying seven alternatives:

- **While you have count or more cards in your void.** "while you have" Count
  "or more cards in your void" comma. Produces Condition::CardsInVoidCount.

- **This card is in your void.** "if this card is in your void" comma. Produces
  Condition::ThisCardIsInYourVoid.

- **Dissolved this turn.** "a" card_predicate "dissolved this turn". Produces
  Condition::DissolvedThisTurn.

- **Discarded this turn.** "you have discarded [a]" card_predicate "this turn".
  Produces Condition::CardsDiscardedThisTurn.

- **With count allies sharing a type.** "with" CountAllies "that share a
  character type" comma. Produces Condition::AlliesThatShareACharacterType.

- **With count allied subtype.** "with" SubtypeCount comma. Produces
  Condition::PredicateCount.

- **With an allied subtype.** "with an allied" Subtype comma. Produces
  Condition::PredicateCount with count 1.

All prefix conditions (those appearing before the effect they gate) are
terminated by a comma, which acts as the syntactic boundary between condition
and effect.

## 9. Directive vs. Word Keywords

The parser uses two keyword-matching strategies reflecting how game concepts are
represented in TOML card data.

**Directives** are game-vocabulary keywords with special UI rendering (bold,
color, icons). In TOML they are written in curly braces, preserved as
Token::Directive values, and matched with the directive(name) helper.
Directives: discover, prevent, dissolve, banish, materialize, reclaim, fast,
judgment, materialized, dissolved, banished, energy_symbol, judgment_phase_name,
choose_one, bullet.

**Words** are plain English vocabulary without special rendering. In TOML they
appear as ordinary text, lowercased by the lexer into Token::Word values, and
matched with word(text) or words(sequence). Words include: draw, gain, discard,
return, put, copy, each, player, until, when, you, may, control, disable,
abandon, shuffle, trigger, take, win, and all structural vocabulary (articles,
prepositions, pronouns).

Mixing the two causes parse failures. If a card writes plain "dissolve" instead
of the directive form, directive("dissolve") will not match. Conversely, if a
card wraps "draw" in curly braces, word("draw") will fail.

Game keywords use directives because the serializer reconstructs them with
special formatting: purple for dissolve/banish/reclaim/materialize/prevent/
kindle/foresee/discover, bold for fast and trigger names, teal for energy
symbols. Plain words are rendered as ordinary unformatted text.

Some borderline cases are firmly in one category. The word "abandon" is always a
plain word (no colored keyword rendering). The word "spark" in effect text is
plain, while the KindleAmount resolved token carries kindle as a typed value.
The energy_symbol and judgment_phase_name directives are bare phrases carrying
no variable data but needing special rendering (teal icon and bold
respectively).
