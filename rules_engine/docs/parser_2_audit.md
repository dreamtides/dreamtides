# Predicate Parsing

21. **Consistency - Inconsistent optional predicates**: `allied_parser()`
    (predicate_parser.rs:84-86) requires a card predicate after "allied" via
    `ignore_then(card_predicate_parser::parser())` without `.or_not()`, while
    `enemy_parser()` (line 78-82) and `ally_parser()` (line 88-92) make the
    predicate optional using `.or_not().map(|pred| ...
    pred.unwrap_or(CardPredicate::Character))`. This means "enemy" and "ally
    character" both work, but "allied" alone fails while "allied character"
    succeeds—an inconsistent pattern.

23. **Generality - Limited operators in comparisons**: All comparison suffix
    parsers only support "less than" (`Operator::OrLess`). Examples:
    `with_cost_compared_to_controlled_suffix()`
    (predicate_suffix_parser.rs:54-59),
    `with_cost_compared_to_void_count_suffix()` (line 61-65),
    `with_spark_compared_to_abandoned_suffix()` (line 67-69), and
    `with_spark_compared_to_energy_spent_suffix()` (line 72-78) all hardcode
    "less than" in their word sequences. This prevents parsing "greater than",
    "equal to", or other comparison operators that might appear in future card
    text.

24. **Generality - Hardcoded ownership in cost comparison**:
    `with_cost_compared_to_controlled_suffix()`
    (predicate_suffix_parser.rs:54-59) hardcodes "allied" in the phrase "with
    cost less than the number of allied {subtype}". This prevents parsing
    patterns like "with cost less than the number of enemy {subtype}" or using
    general predicate parsing for the ownership component.

25. **Generality - Hardcoded reference in spark comparison**:
    `with_spark_compared_to_abandoned_suffix()`
    (predicate_suffix_parser.rs:67-69) hardcodes "that ally's spark" instead of
    parsing a general predicate reference. This prevents comparing spark to
    other targets like "this character's spark", "that enemy's spark", or any
    other predicate reference pattern.

# Effect Parsing

30. **Code Quality - Duplicate cost parsers in effect_parser**:
    `pay_energy_cost()` (effect_parser.rs:277), `discard_cost()` (line 281), and
    `abandon_cost()` (line 290) duplicate similar parsers already defined in
    cost_parser.rs. When these trigger costs are used in
    effect_with_trigger_cost_parser(), the parsing logic is maintained in two
    places, creating inconsistency risk if cost parsing rules change.

31. **Code Quality - Near-duplicate multiply effect parsers**:
    `multiply_energy_gain_from_card_effects()`
    (resource_effect_parsers.rs:33-44) and
    `multiply_card_draw_from_card_effects()` (lines 46-58) are structurally
    identical parsers differing only in the effect name and word sequence ("the
    amount of {energy-symbol} you gain" vs "the number of cards you draw").
    These could be unified with a helper that accepts the effect type and word
    pattern as parameters.

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

