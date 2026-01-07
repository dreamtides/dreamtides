# Predicate Parsing

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

