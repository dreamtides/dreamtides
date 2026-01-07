# Quantity Expression Parsing

17. **Generality - Hardcoded "ally" predicates**: Three parsers in
    quantity_expression_parser.rs hardcode `CardPredicate::Character` for "ally"
    text: "ally abandoned this turn" (line 18), "ally abandoned" (line 21), and
    "ally returned" (line 24). These are followed by more general parsers at
    lines 25-30 that parse card predicates for "<card-predicate> abandoned" and
    "<card-predicate> returned". The specific "ally" cases prevent parsing
    variations like "enemy abandoned" or "event returned", and duplicate logic
    that the general parsers already handle.

18. **Code Quality - Redundant parser combinator pattern**: Lines 16-24 use
    `.to(()).map(|_| ...)` to create unit values then map them. This is
    redundant - they could use `.to(QuantityExpression::...)` directly instead
    of the two-step `.to(()).map(|_| QuantityExpression::...)` pattern.

# Predicate Parsing

19. **Code Quality - Massive duplication in card_predicate_parser**: The
    `parser()` function (card_predicate_parser.rs:8-109) has extensive
    duplication where lines 20-61 parse base predicate + suffix, while lines
    70-105 parse just the suffix defaulting to `CardPredicate::Character`. This
    pattern repeats for 6 suffixes: `with_cost_compared_to_controlled` (lines
    20-26 vs 70-77), `with_cost_compared_to_void_count` (lines 28-34 vs 79-83),
    `with_spark_compared_to_abandoned` (lines 36-42 vs 85-89),
    `with_spark_compared_to_energy_spent` (lines 44-50 vs 91-95), `with_cost`
    (lines 52-57 vs 97-102), and `with_spark` (lines 59-61 vs 104-105). Each
    appears twice with nearly identical mapping logic, differing only in whether
    `target` is the parsed base or hardcoded `Character`.

20. **Code Quality - Unused helper function**: The `ally()` function
    (predicate_parser.rs:12-14) returns
    `Predicate::Another(CardPredicate::Character)` but is never referenced
    anywhere in the codebase, making it dead code that should be removed.

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

26. **Code Quality - Duplicate "gains reclaim" parsers**:
    `gains_reclaim_for_cost()` (control_effects_parsers.rs:45-55) and
    `gains_reclaim_for_cost_this_turn()` (card_effect_parsers.rs:192-210) are
    nearly identical parsers split across two files. The control_effects version
    parses "it gains {reclaim-for-cost} this turn" while the card_effect version
    has a choice between "it gains {reclaim} equal to its cost this turn" and
    "<predicate> gains {reclaim-for-cost} this turn". These should be unified
    into a single parser that handles all cases.

27. **Generality - Hardcoded card types in return from void**:
    `return_from_void_to_hand()` (card_effect_parsers.rs:128-133) hardcodes
    `Predicate::Any(CardPredicate::Event)` when parsing "return {up-to-n-events}
    from your void to your hand" instead of parsing the card type as a general
    predicate. This prevents patterns like "return up to {n} {subtype} from your
    void to your hand" or "return up to {n} characters from your void to your
    hand".

28. **Generality - Hardcoded allies in banish**: `banish_up_to_n()`
    (game_effects_parsers.rs:181-186) hardcodes `target:
    Predicate::Another(CardPredicate::Character)` instead of parsing which cards
    to banish. The parser uses `up_to_n_allies()` helper which implies allies,
    but the parser itself doesn't verify this matches the parsed text, limiting
    extensibility to other card types.

29. **Code Quality - Massive duplication in void reclaim parser**:
    `cards_in_void_gain_reclaim()` (card_effect_parsers.rs:212-267) has three
    branches parsing "all cards", "a card/character/event", and "{subtype}"
    separately. Lines 217-223, 244-250, and 255-260 all repeat nearly identical
    logic for parsing "in your void gains {reclaim} this turn". The card type
    parsing logic (lines 228-243) also duplicates basic card type matching. This
    55-line function could be significantly simplified with a unified predicate
    parser.

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

