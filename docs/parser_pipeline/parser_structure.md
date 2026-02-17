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

| Module                  | Domain                                                          |
| ----------------------- | --------------------------------------------------------------- |
| card_effect_parsers     | Card movement, draw, discard, energy, points, reclaim           |
| spark_effect_parsers    | Kindle, spark gain, spark manipulation                          |
| control_effects_parsers | Gain control, deck manipulation, disable abilities              |
| resource_effect_parsers | Resource multipliers, point manipulation                        |
| game_effects_parsers    | Foresee, discover, dissolve, banish, materialize, prevent, copy |

Each module contains multiple parser functions for its domain's variants (e.g.,
game_effects_parsers has separate parsers for dissolve, banish, materialize, and
others, each with sub-variants for specificity). The "for each" variants of
quantity effects delegate to a shared quantity expression parser. See each
module's source for the complete list of recognized patterns.

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
abilities to fire. Its entry point tries keyword triggers first (judgment,
materialized, dissolved, and compound combinations), then standard triggers
terminated by a comma. Standard triggers are organized into four groups: play
triggers, action triggers (abandon, discard, materialize), state change triggers
(dissolved, banished, leaves-play), and timing triggers (draw, end-of-turn,
gain-energy). Within each group, more specific variants precede less specific
ones.

**The dual-parse pattern for Predicate::Your.** The predicate parser maps bare
card types to Predicate::Any, but trigger contexts like "when you play a
character" require Predicate::Your. The trigger parser solves this by trying
card_predicate_parser mapped to Predicate::Your before falling through to the
full predicate_parser. This pattern appears in play, discard, and materialize
triggers.

## 7. Cost Parser

The cost parser (cost_parser.rs) recognizes costs for activated abilities and
trigger costs for compound effects. Its entry point tries a cost choice (two or
more costs separated by "or", producing Cost::Choice) before falling back to a
single cost. Single cost types include energy, abandon (with sub-variants),
return to hand, discard, and banish from void. In activated abilities, "once per
turn" can appear in the cost list as an option modifier rather than a true cost.
Several cost parsers are exported for reuse by other modules.

## 8. Condition Parser

The condition parser (condition_parser.rs) recognizes conditions that gate
effects or modify static abilities. Conditions cover board-state checks like
cards in void, subtypes controlled, allies sharing a type, and cards dissolved
or discarded this turn. All prefix conditions are terminated by a comma, which
acts as the syntactic boundary between condition and effect.

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
