# Predicate Parsing

23. **Generality - Limited operators in comparisons**: All comparison suffix
    parsers only support "less than" (`Operator::OrLess`). Examples:
    `with_cost_compared_to_controlled_suffix()`
    (predicate_suffix_parser.rs:54-59),
    `with_cost_compared_to_void_count_suffix()` (line 61-65),
    `with_spark_compared_to_abandoned_suffix()` (line 67-69), and
    `with_spark_compared_to_energy_spent_suffix()` (line 72-78) all hardcode
    "less than" in their word sequences. This prevents parsing "greater than",
    "equal to", or other comparison operators that might appear in future card
    text. Please implement this very carefully to avoid performance regressions.

# Effect Parsing

30. **Code Quality - Duplicate cost parsers in effect_parser**:
    `pay_energy_cost()` (effect_parser.rs:277), `discard_cost()` (line 281), and
    `abandon_cost()` (line 290) duplicate similar parsers already defined in
    cost_parser.rs. When these trigger costs are used in
    effect_with_trigger_cost_parser(), the parsing logic is maintained in two
    places, creating inconsistency risk if cost parsing rules change.


33. **Code Quality - Near-duplicate gains spark branches**:
    `gains_spark_for_each()` (spark_effect_parsers.rs:28-56) contains two nearly
    identical branches (lines 31-40 and 42-53) that differ only in whether the
    target defaults to `Predicate::This` or is parsed. Both branches parse
    identical "gains +{s} spark for each <quantity>" patterns with redundant
    mapping logic. The first branch could simply check if no predicate was
    parsed and default to `This`.

34. **Generality - Hardcoded "each ally" in trigger judgment**:
    `trigger_judgment_ability()` (game_effects_parsers.rs:308-317) hardcodes
    both the word sequence "trigger the {judgment} ability of each ally" and the
    result fields `matching: Predicate::Another(CardPredicate::Character)` and
    `collection: CollectionExpression::All`. The "each ally" portion should be
    parsed as a general predicate to support variations like "trigger the
    {judgment} ability of each {subtype}" or "trigger the {judgment} ability of
    each allied character with cost {e} or less".

