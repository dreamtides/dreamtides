# Parser Crate Technical Documentation

This document describes the card rules text parser for the Dreamtides card game.

## Overview

The parser converts human-readable card ability text into structured `Ability` data types using the **Chumsky 0.10** parser combinator library. Error reporting uses **Ariadne** for visual span-based error messages.

**Entry point:** `ability_parser::parse(input: &str) -> Result<Vec<Ability>, Vec<InitializationError>>`

**Key behavior:**
- Input text is normalized to lowercase before parsing
- Multiple abilities are separated by double newlines (`\n\n`)
- Each block is parsed via `single_ability_parser()` which tries parsers in priority order

## Ability Types

The parser produces five ability variants defined in [ability.rs](../ability_data/src/ability.rs):

| Type | Module | Description | Text Pattern |
|------|--------|-------------|--------------|
| `Triggered` | [triggered_ability_parser.rs](src/triggered_ability_parser.rs) | Effect fires on event | `when/whenever/at X, effect` or `{keyword}: effect` |
| `Activated` | [activated_ability_parser.rs](src/activated_ability_parser.rs) | Player pays cost for effect | `{modifier} cost: effect` |
| `Named` | [named_ability_parser.rs](src/named_ability_parser.rs) | Keyword expanding to abilities | `{-reclaim}`, `{-reclaim-cost(e:N)}` |
| `Event` | [effect_parser.rs](src/effect_parser.rs) | Immediate effect on play | `[cost:] effect` |
| `Static` | [static_ability_parser.rs](src/static_ability_parser.rs) | Passive rule modification | Declarative statements ending in `.` |

Parser priority order: Triggered → Activated → Named → Event → Static

## Module Reference

### Core Entry Points

**[ability_parser.rs](src/ability_parser.rs)** (78 lines)
- `parse(input)` - Main API, returns `Result<Vec<Ability>, Vec<InitializationError>>`
- `parse_string(text)` - Returns `Vec<ParseResult>` for each ability block
- `single_ability_parser()` - Combines all ability parsers via `choice()`

### Ability Parsers

**[triggered_ability_parser.rs](src/triggered_ability_parser.rs)** (32 lines)
- `parser()` - Main entry, tries keyword and standard formats
- `keyword_trigger_parser()` - Parses `{Materialized}, {Judgment}: effect`
- `standard_trigger_parser()` - Parses `[once per turn,] when/whenever/at event, effect`

**[activated_ability_parser.rs](src/activated_ability_parser.rs)** (23 lines)
- Parses modifier flags: `{a}`, `{fa}` (fast), `{ma}` (multi), `{fma}` (fast+multi)
- Format: `modifiers cost1, cost2: effect`
- Produces `ActivatedAbility { costs, effect, options }`

**[static_ability_parser.rs](src/static_ability_parser.rs)** (221 lines)
- `parser()` - Handles conditional statics: `if condition, ability`
- `standard()` - 18 static ability variants via `choice()`
- Key variants: `OncePerTurnPlayFromVoid`, `SparkBonusYourCharacters`, `CostReductionForEach`, `PlayFromVoid`, `PlayForAlternateCost`, `DisableEnemyMaterializedAbilities`

**[named_ability_parser.rs](src/named_ability_parser.rs)** (17 lines)
- Currently only parses Reclaim: `{-reclaim}` or `{-reclaim-cost(e:N)}`

### Effect Parsing

**[effect_parser.rs](src/effect_parser.rs)** (67 lines)
- `event()` - Parses event abilities with optional additional costs
- `effect()` - Recursive parser for effect sequences
- `single_effect()` - Handles separators: `.`, `, then`, `and then`
- `optional_effect()` - `you may effect` or `you may cost to effect`
- `conditional_effect()` - `if condition, effect`

**[standard_effect_parser.rs](src/standard_effect_parser.rs)** (693 lines) - **Largest module**
- `parser()` - Entry point with recursive effect support
- `non_recursive_effects()` - Groups 90+ effects into categories:
  - `card_effects()` - Materialize, dissolve, draw, discard, return, copy
  - `spark_effects()` - Gain spark, kindle, spark becomes
  - `gain_effects()` - Gain energy, points, control, aegis, reclaim
  - `enemy_effects()` - Enemy gains/loses points
  - `game_state_effects()` - Discover, prevent, banish, abandon, foresee
  - `pay_cost()` - Pay cost effects
- `create_trigger_until_end_of_turn()` - Creates temporary triggered abilities

**[modal_effect_parser.rs](src/modal_effect_parser.rs)** (26 lines)
- Parses "Choose One" effects: `{choose-one} {bullet} $N: effect {bullet} $N: effect`
- Returns `Effect::Modal(Vec<ModalEffectChoice>)`

### Supporting Parsers

**[trigger_event_parser.rs](src/trigger_event_parser.rs)** (106 lines)
- `event_parser()` - Parses trigger conditions (materialize, play, discard, end of turn, etc.)
- `keyword_parser()` - Parses `{Materialized}`, `{Judgment}`, `{Dissolved}` keywords

**[cost_parser.rs](src/cost_parser.rs)** (109 lines)
- `parser()` - Energy costs and standard costs
- `standard_cost()` - Verbal costs: pay, banish, abandon, discard, spend energy
- `present_participle_additional_cost()` - For static abilities ("by abandoning")
- `third_person_singular_present_tense_cost()` - For conditions ("pays $N")

**[card_predicate_parser.rs](src/card_predicate_parser.rs)** (175 lines)
- Parses card targeting: character types, card types, cost/spark predicates
- Handles recursive predicates with cost comparisons

**[condition_parser.rs](src/condition_parser.rs)** (48 lines)
- Parses conditions: control count, void count, dissolved this turn, cards discarded/drawn

**[determiner_parser.rs](src/determiner_parser.rs)** (95 lines)
- `target_parser()` - Explicit targets: "an enemy character", "that card"
- `for_each_parser()` - "for each X" expressions
- `counted_parser()` - "N X" matching expressions
- Predicates: `This`, `That`, `It`, `Them`, `Your`, `Enemy`, `Another`, `Any`

**[collection_expression_parser.rs](src/collection_expression_parser.rs)** (19 lines)
- Parses quantities: "any number of", "all but one", "up to N", "all", "each"

**[quantity_expression_parser.rs](src/quantity_expression_parser.rs)** (32 lines)
- Parses "for each X" calculations: cards discarded/drawn/played/abandoned this turn

**[displayed_ability_parser.rs](src/displayed_ability_parser.rs)** (163 lines)
- Reconstructs human-readable text from parsed abilities
- Preserves original formatting for display

**[parser_utils.rs](src/parser_utils.rs)** (93 lines)
- `phrase(text)` - Padded literal parser
- `numeric(before, mapper, after)` - "before N after" pattern
- `number(mapper)` - Integer parser
- `text_number()` - "one", "two", "3", etc.
- `ordinal_number()` - "first", "second", "3rd"
- `a_or_count()` - "a" or number

## Data Structures

Located in the [ability_data](../ability_data/src/) crate.

### Effect Structure ([effect.rs](../ability_data/src/effect.rs))

```
Effect
├── Effect(StandardEffect)           # Simple effect
├── WithOptions(EffectWithOptions)   # Optional/conditional/costly
├── List(Vec<EffectWithOptions>)     # Sequence
└── Modal(Vec<ModalEffectChoice>)    # Choose-one

EffectWithOptions { effect, optional, trigger_cost, condition }
```

### StandardEffect ([standard_effect.rs](../ability_data/src/standard_effect.rs))

91 variants covering:
- Card movement: Materialize, Dissolve, Banish, Return, Copy
- Spark/Power: GainsSpark, Kindle, SparkBecomes
- Resources: GainEnergy, GainPoints, DrawCards, DiscardCards
- Control: GainControl, DisableActivatedAbilities
- Special: Foresee, Counterspell, CreateTriggerUntilEndOfTurn
- Terminal: TakeExtraTurn, YouWinTheGame, NoEffect

### Cost ([cost.rs](../ability_data/src/cost.rs))

```
Cost
├── Energy(Energy)
├── AbandonCharacters(Predicate, u32)
├── BanishCardsFromYourVoid(u32)
├── DiscardCards(CardPredicate, u32)
├── DiscardHand
└── ... (8 more variants)
```

### TriggerEvent ([trigger_event.rs](../ability_data/src/trigger_event.rs))

```
TriggerEvent
├── Materialize(Predicate)
├── MaterializeNthThisTurn(Predicate, u32)
├── Play(Predicate) / PlayFromHand / PlayDuringTurn
├── Discard / Dissolved / Abandon / Banished
├── GainEnergy / EndOfYourTurn
└── Keywords(Vec<TriggerKeyword>)

TriggerKeyword: Materialized | Judgment | Dissolved
```

### Predicate ([predicate.rs](../ability_data/src/predicate.rs))

```
Predicate
├── This | That | It | Them          # References
├── Your(CardPredicate)
├── Enemy(CardPredicate)
├── Another(CardPredicate)
├── Any(CardPredicate)
└── YourVoid / EnemyVoid(CardPredicate)

CardPredicate
├── Card | Character | Event
├── CharacterType(CardSubtype)
├── CharacterWithSpark(Spark, Operator)
├── CardWithCost { target, cost_operator, cost }
└── ... (8 more variants)
```

### StaticAbility ([static_ability.rs](../ability_data/src/static_ability.rs))

18 variants including:
- `OncePerTurnPlayFromVoid { matching }`
- `PlayFromVoid(PlayFromVoid)` / `PlayForAlternateCost(AlternateCost)`
- `SparkBonusYourCharacters { matching, added_spark }`
- `CostReductionForEach { reduction, quantity }`
- `EnemyCardsCostIncrease { matching, increase }`
- `DisableEnemyMaterializedAbilities`
- `CardsInYourVoidHaveReclaim { matching }`

## Text Syntax Conventions

### Placeholder Syntax

| Pattern | Meaning |
|---------|---------|
| `{-energy-cost(e:N)}` | Energy cost (N energy) |
| `{-gained-spark(n:N)}` | Spark gain amount |
| `{-reclaim}` / `{-reclaim-cost(e:N)}` | Reclaim keyword |
| `{cardtype: warrior}` | Character subtype |
| `{Materialized}` / `{Judgment}` / `{Dissolved}` | Trigger keywords |
| `{choose-one}` | Modal choice marker |
| `{bullet}` | Modal choice bullet |
| `{kw: aegis}` | Keyword ability |
| `{a}` / `{fa}` / `{ma}` / `{fma}` | Activated ability modifiers |

### Activated Ability Modifiers

| Modifier | Meaning |
|----------|---------|
| `{a}` | Standard activation |
| `{fa}` | Fast Activate (can respond) |
| `{ma}` | Multi Activate (multiple times per turn) |
| `{fma}` | Fast + Multi |

## Parser Composition Patterns

Key Chumsky combinators used:

| Combinator | Usage |
|------------|-------|
| `choice((a, b, c))` | Try parsers in order |
| `a.then(b)` | Sequence, return both |
| `a.then_ignore(b)` | Sequence, return left |
| `a.ignore_then(b)` | Sequence, return right |
| `a.or_not()` | Optional, returns `Option` |
| `a.separated_by(sep)` | Delimited list |
| `a.repeated().at_least(n)` | N or more occurrences |
| `a.map(f)` | Transform result |
| `a.boxed()` | Type erasure |
| `recursive(\|r\| ...)` | Recursive grammar |

## CLI Tool

Located at [parser_cli/src/main.rs](../parser_cli/src/main.rs) (62 lines).

```bash
parser_cli "ability text"           # Parse single ability
parser_cli --file abilities.txt     # Parse file (one ability per line)
```

Output: RON (Rusty Object Notation) representation of parsed abilities.

## Testing

Test files in [tests/parser_tests/tests/parser_tests/](../../tests/parser_tests/tests/parser_tests/):

- `triggered_ability_parsing_tests.rs` - Triggered ability tests
- `activated_ability_parsing_tests.rs` - Activated ability tests
- `static_ability_parsing_tests.rs` - Static ability tests
- `event_ability_parsing_tests.rs` - Event ability tests
- `ability_formatting_parsing_tests.rs` - Display/formatting tests

Uses **Insta** snapshot testing with `assert_ron_snapshot!()`.

Test utility: `parser_tests::parser_test_utils::parse()`

Example test pattern:
```rust
#[test]
fn test_materialize_warrior_gain_spark() {
    let result = parse("Whenever you materialize another {cardtype: warrior}, this character gains {-gained-spark(n:1)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Materialize(Another(CharacterType(Warrior))),
        effect: Effect(GainsSpark(target: This, gains: Spark(1))),
      )),
    ]
    "###);
}
```

Run tests: `just battle-test <TEST_NAME>`

## Files to Read for More Information

| Topic | Files |
|-------|-------|
| Adding new effects | [standard_effect_parser.rs](src/standard_effect_parser.rs), [standard_effect.rs](../ability_data/src/standard_effect.rs), [docs/adding_new_effects.md](../../docs/adding_new_effects.md) |
| Adding new triggers | [trigger_event_parser.rs](src/trigger_event_parser.rs), [trigger_event.rs](../ability_data/src/trigger_event.rs), [docs/adding_new_triggers.md](../../docs/adding_new_triggers.md) |
| Adding new costs | [cost_parser.rs](src/cost_parser.rs), [cost.rs](../ability_data/src/cost.rs) |
| Adding new static abilities | [static_ability_parser.rs](src/static_ability_parser.rs), [static_ability.rs](../ability_data/src/static_ability.rs) |
| Card predicates | [card_predicate_parser.rs](src/card_predicate_parser.rs), [predicate.rs](../ability_data/src/predicate.rs) |
| Conditions | [condition_parser.rs](src/condition_parser.rs), [condition.rs](../ability_data/src/condition.rs) |
| Parser utilities | [parser_utils.rs](src/parser_utils.rs) |
| Data structures | [ability_data/src/](../ability_data/src/) directory |
