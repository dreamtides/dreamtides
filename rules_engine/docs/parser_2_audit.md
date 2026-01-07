# Predicate Parsing

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

