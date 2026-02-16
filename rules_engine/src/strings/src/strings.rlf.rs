rlf::rlf! {
    // =========================================================================
    // Core game symbols
    // =========================================================================

    // Colored energy symbol.
    energy_symbol = "<color=#00838F>\u{25CF}</color>";
    // Colored points symbol.
    points_symbol = "<color=#F57F17>\u{234F}</color>";
    // Fast symbol (lightning bolt).
    fast_symbol = "\u{21AF}";

    // =========================================================================
    // Parameterized energy and points formatters
    // =========================================================================

    // Energy amount with colored symbol (e.g., "2[energy]").
    energy($e) = "<color=#00838F>{$e}\u{25CF}</color>";
    // Points amount with colored symbol (e.g., "3[points]").
    points($p) = "<color=#F57F17>{$p}\u{234F}</color>";
    // Maximum energy display (e.g., "5 maximum [energy]").
    maximum_energy($max) = "{$max} maximum {energy_symbol}";

    // =========================================================================
    // Trigger ability prefixes
    // =========================================================================

    // Parameterized trigger prefix with dynamic text.
    trigger($t) = "\u{25B8} <b>{$t}:</b>";
    // Materialized trigger prefix.
    materialized = "\u{25B8} <b>Materialized:</b>";
    // Judgment trigger prefix.
    judgment = "\u{25B8} <b>Judgment:</b>";
    // Dissolved trigger prefix.
    dissolved = "\u{25B8} <b>Dissolved:</b>";
    // Combined materialized and judgment trigger prefix.
    materialized_judgment = "\u{25B8} <b>Materialized, Judgment:</b>";
    // Combined materialized and dissolved trigger prefix.
    materialized_dissolved = "\u{25B8} <b>Materialized, Dissolved:</b>";
    // Bold judgment phase name for card text references.
    judgment_phase_name = "<b>Judgment</b>";
    // Bare keyword name for Judgment (used in multi-keyword fallback).
    judgment_keyword_name = "Judgment";
    // Bare keyword name for Materialized (used in multi-keyword fallback).
    materialized_keyword_name = "Materialized";
    // Bare keyword name for Dissolved (used in multi-keyword fallback).
    dissolved_keyword_name = "Dissolved";

    // =========================================================================
    // Keywords
    // =========================================================================

    // Parameterized keyword formatting in purple.
    keyword($k) = "<color=#AA00FF>{$k}</color>";
    // Dissolve keyword.
    dissolve = "<color=#AA00FF>dissolve</color>";
    // Banish keyword.
    banish = "<color=#AA00FF>banish</color>";
    // Banished keyword.
    banished = "<color=#AA00FF>banished</color>";
    // Discover keyword.
    discover = "<color=#AA00FF>discover</color>";
    // Discovered keyword (participial form).
    discovered = "<color=#AA00FF>discovered</color>";
    // Reclaim keyword.
    reclaim = "<color=#AA00FF>reclaim</color>";
    // Reclaimed keyword (participial form).
    reclaimed = "<color=#AA00FF>reclaimed</color>";
    // Materialize keyword.
    materialize = "<color=#AA00FF>materialize</color>";
    // Prevent keyword.
    prevent = "<color=#AA00FF>prevent</color>";
    // Prevented keyword (participial form).
    prevented = "<color=#AA00FF>prevented</color>";
    // Kindle keyword with spark amount.
    kindle($k) = "<color=#AA00FF>kindle</color> {$k}";
    // Kindled keyword (participial form).
    kindled = "<color=#AA00FF>kindled</color>";
    // Foresee keyword with card count.
    foresee($n) = "<color=#AA00FF>foresee</color> {$n}";
    // Fast keyword with lightning bolt.
    fast = "<b>\u{21AF}fast</b>";
    // Reclaim with energy cost.
    reclaim_for_cost($r) = "<color=#AA00FF>reclaim</color> <color=#00838F>{$r}\u{25CF}</color>";

    // =========================================================================
    // Modal card formatting
    // =========================================================================

    // Bold "Choose One:" header for modal cards.
    choose_one = "<b>Choose One:</b>";
    // Bullet point for modal options.
    bullet = "\u{2022}";

    // =========================================================================
    // Plural-aware card counts
    // =========================================================================

    // Card noun with article metadata.
    card = :a { one: "card", other: "cards" };
    // Card count with article (e.g., "a card" or "2 cards").
    cards($n) = :match($n) { 1: "a card", *other: "{$n} cards" };
    // Top N cards of deck (e.g., "top card" or "top 3 cards").
    top_n_cards($n) = :match($n) { 1: "top card", *other: "top {$n} {card:$n}" };

    // =========================================================================
    // Spark and generic count
    // =========================================================================

    // Spark value passthrough.
    spark_value($s) = "{$s}";
    // Spark keyword for ability text.
    spark = "spark";
    // Generic count passthrough for numeric conditions.
    count($n) = "{$n}";

    // =========================================================================
    // Ally counts
    // =========================================================================

    // Ally noun with article metadata.
    ally = :an { one: "ally", other: "allies" };
    // Ally count with article (e.g., "an ally" or "2 allies").
    count_allies($n) = :match($n) { 1: "an ally", *other: "{$n} allies" };
    // Allied character count with subtype (e.g., "an allied warrior").
    count_allied_subtype($n, $s) = :from($s) :match($n) {
        1: "an allied {subtype($s)}",
        *other: "{$n} allied {subtype($s):other}",
    };

    // =========================================================================
    // Figment types
    // =========================================================================

    // Celestial figment type, variant-aware for figment composition.
    celestial = :a { one: "Celestial", other: "Celestial" };
    // Halcyon figment type, variant-aware for figment composition.
    halcyon = :a { one: "Halcyon", other: "Halcyon" };
    // Radiant figment type, variant-aware for figment composition.
    radiant = :a { one: "Radiant", other: "Radiant" };
    // Shadow figment type, variant-aware for figment composition.
    shadow = :a { one: "Shadow", other: "Shadow" };

    // =========================================================================
    // Figment tokens
    // =========================================================================

    // Figment token with gold formatting, variant-aware singular/plural.
    figment($f) = :from($f) {
        *one: "<color=#F57F17><b><u>{$f} Figment</u></color></b>",
        other: "<color=#F57F17><b><u>{$f} Figments</u></color></b>",
    };
    // N figments with article for singular.
    n_figments($n, $f) = :from($f) :match($n) {
        1: "a {figment($f)}",
        *other: "{text_number($n)} {figment($f):other}",
    };

    // =========================================================================
    // Character subtypes
    // =========================================================================

    // Agent subtype.
    agent = :an { one: "Agent", other: "Agents" };
    // Ancient subtype.
    ancient = :an { one: "Ancient", other: "Ancients" };
    // Avatar subtype.
    avatar = :an { one: "Avatar", other: "Avatars" };
    // Child subtype.
    child = :a { one: "Child", other: "Children" };
    // Detective subtype.
    detective = :a { one: "Detective", other: "Detectives" };
    // Enigma subtype.
    enigma = :an { one: "Enigma", other: "Enigmas" };
    // Explorer subtype.
    explorer = :an { one: "Explorer", other: "Explorers" };
    // Guide subtype.
    guide = :a { one: "Guide", other: "Guides" };
    // Hacker subtype.
    hacker = :a { one: "Hacker", other: "Hackers" };
    // Mage subtype.
    mage = :a { one: "Mage", other: "Mages" };
    // Monster subtype.
    monster = :a { one: "Monster", other: "Monsters" };
    // Musician subtype.
    musician = :a { one: "Musician", other: "Musicians" };
    // Outsider subtype.
    outsider = :an { one: "Outsider", other: "Outsiders" };
    // Renegade subtype.
    renegade = :a { one: "Renegade", other: "Renegades" };
    // Robot subtype.
    robot = :a { one: "Robot", other: "Robots" };
    // Spirit Animal subtype.
    spirit_animal = :a { one: "Spirit Animal", other: "Spirit Animals" };
    // Super subtype.
    super_ = :a { one: "Super", other: "Supers" };
    // Survivor subtype.
    survivor = :a { one: "Survivor", other: "Survivors" };
    // Synth subtype.
    synth = :a { one: "Synth", other: "Synths" };
    // Tinkerer subtype.
    tinkerer = :a { one: "Tinkerer", other: "Tinkerers" };
    // Trooper subtype.
    trooper = :a { one: "Trooper", other: "Troopers" };
    // Visionary subtype.
    visionary = :a { one: "Visionary", other: "Visionaries" };
    // Visitor subtype.
    visitor = :a { one: "Visitor", other: "Visitors" };
    // Warrior subtype.
    warrior = :a { one: "Warrior", other: "Warriors" };

    // Subtype display with green bold formatting, variant-aware.
    subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";

    // =========================================================================
    // Text number conversion
    // =========================================================================

    // Convert number to word (1-5) or fall back to numeral.
    text_number($n) = :match($n) {
        1: "one",
        2: "two",
        3: "three",
        4: "four",
        5: "five",
        *other: "{$n}",
    };

    // =========================================================================
    // Turn duration and multipliers
    // =========================================================================

    // Turn duration with repetition count.
    this_turn_times($n) = :match($n) {
        1: "this turn",
        2: "twice this turn",
        *other: "this turn {text_number($n)} times",
    };

    // Multiplier effect (Double, Triple, etc.).
    multiply_by($n) = :match($n) { 2: "Double", 3: "Triple", *other: "Multiply by {$n}" };

    // =========================================================================
    // Copy counts
    // =========================================================================

    // Copy count with article (e.g., "a copy" or "two copies").
    copies($n) = :match($n) { 1: "a copy", *other: "{text_number($n)} copies" };

    // =========================================================================
    // Random character targeting
    // =========================================================================

    // Random character count (e.g., "a random character" or "two random characters").
    n_random_characters($n) = :match($n) {
        1: "a random character",
        *other: "{text_number($n)} random characters",
    };

    // =========================================================================
    // Optional event targeting
    // =========================================================================

    // Up to N events (e.g., "an event" or "up to 3 events").
    up_to_n_events($n) = :match($n) { 1: "an event", *other: "up to {$n} events" };

    // =========================================================================
    // Optional ally targeting
    // =========================================================================

    // Up to N allies (e.g., "an ally" or "up to 3 allies").
    up_to_n_allies($n) = :match($n) { 1: "an {ally}", *other: "up to {$n} {ally:other}" };

    // =========================================================================
    // Pronoun agreement
    // =========================================================================

    // Object pronoun with singular/plural agreement.
    pronoun = { one: "it", other: "them" };

    // =========================================================================
    // Icons
    // =========================================================================

    // Dev menu icon (bug).
    bug_icon = "\u{f88d}";
    // Undo button icon.
    undo_icon = "\u{fd88}";
    // Eye icon.
    eye_icon = "\u{f9f9}";
    // Eye icon with slash.
    eye_slash_icon = "\u{f9f8}";
    // Asterisk icon for non-numeric costs.
    asterisk_icon = "\u{f810}";

    // =========================================================================
    // Prompt messages
    // =========================================================================

    // Prompt to target a character.
    prompt_choose_character = "Choose a character";
    // Prompt to pick a card on the stack.
    prompt_select_stack_card = "Select a card";
    // Prompt to pick a card from your void.
    prompt_select_from_void = "Select from your void";
    // Prompt to pick a card from your hand.
    prompt_select_from_hand = "Select from your hand";
    // Prompt to pick a choice among several options.
    prompt_select_option = "Select an option";
    // Prompt to pick an amount of energy.
    prompt_choose_energy_amount = "Choose energy amount";
    // Prompt to pick card ordering within the deck.
    prompt_select_card_order = "Select card position";
    // Prompt to pick a mode of a modal card to play.
    prompt_pick_mode = "Choose a mode";

    // =========================================================================
    // Buttons
    // =========================================================================

    // Dev menu button label.
    dev_menu_button = "{bug_icon} Dev";
    // Decline to take the action associated with a prompt.
    decline_prompt_button = "Decline";
    // Choose to pay energy to take a prompt action.
    pay_energy_prompt_button($e) = "Spend {energy($e)}";
    // Confirm the amount of energy to pay as an additional cost.
    pay_energy_additional_cost_button($e) = "Spend {energy($e)}";
    // Confirm selection of target cards in the void.
    primary_button_submit_void_card_targets = "Submit";
    // Confirm selection of target cards in the hand.
    primary_button_submit_hand_card_targets = "Submit";
    // Confirm selection of ordering of cards in deck.
    primary_button_submit_deck_card_order = "Submit";
    // Resolve the top card of the stack.
    primary_button_resolve_stack = "Resolve";
    // End your turn.
    primary_button_end_turn = "End Turn";
    // End the opponent's turn and begin your turn.
    primary_button_start_next_turn = "Next Turn";
    // Increment the energy amount in a prompt.
    increment_energy_prompt_button = "+1{energy_symbol}";
    // Decrement the energy amount in a prompt.
    decrement_energy_prompt_button = "-1{energy_symbol}";
    // Hide the stack and view the battlefield.
    hide_stack_button = "{eye_icon}";
    // Show the stack after hiding it.
    show_stack_button = "{eye_slash_icon}";
    // Show the battlefield.
    show_battlefield_button = "{eye_icon}";
    // Hide the battlefield.
    hide_battlefield_button = "{eye_slash_icon}";

    // =========================================================================
    // Card rules text annotations
    // =========================================================================

    // Energy paid annotation for variable-cost cards.
    card_rules_text_energy_paid($e) = "({energy($e)} paid)";
    // Reclaimed annotation.
    card_rules_text_reclaimed = "(Reclaimed)";
    // Anchored annotation.
    card_rules_text_anchored = "(Anchored)";

    // =========================================================================
    // Card naming
    // =========================================================================

    // Card name for a numbered modal effect choice.
    modal_effect_choice_card_name($number) = "Choice {$number}";
    // Card name for a character ability.
    character_ability_card_name($character_name) = "{$character_name} Ability";

    // =========================================================================
    // Limit warning messages
    // =========================================================================

    // Warning about exceeding the hand size limit.
    hand_size_limit_exceeded_warning_message =
        "Note: Cards drawn in excess of 10 become {energy_symbol} instead.";
    // Warning about exceeding the character limit.
    character_limit_exceeded_warning_message = "Character limit exceeded: A character will be abandoned, with its spark permanently added to your total.";
    // Warning about exceeding both limits.
    combined_limit_warning_message = "Character limit exceeded: A character will be abandoned. Cards drawn in excess of 10 become {energy_symbol} instead.";

    // =========================================================================
    // Error panel
    // =========================================================================

    // Title for a panel displaying an error message.
    error_message_panel_title = "Error";

    // =========================================================================
    // Card types
    // =========================================================================

    // Character card type.
    card_type_character = "Character";
    // Event card type.
    card_type_event = "Event";
    // Dreamsign card type.
    card_type_dreamsign = "Dreamsign";
    // Dreamcaller card type.
    card_type_dreamcaller = "Dreamcaller";
    // Dreamwell card type.
    card_type_dreamwell = "Dreamwell";

    // =========================================================================
    // Help text
    // =========================================================================

    // Help text for dissolve ability.
    help_text_dissolve = "{@cap dissolve}: Send a character to the void";
    // Help text for prevent ability.
    help_text_prevent = "{@cap prevent}: Send a card to the void in response to it being played";
    // Help text for foresee 1 ability.
    help_text_foresee_1 = "<color=#AA00FF>Foresee</color> 1: Look at the top card of your deck. You may put it into your void.";
    // Help text for foresee N ability.
    help_text_foresee_n($n) = "<color=#AA00FF>Foresee</color> {$n}: Look at the top {$n} cards of your deck. You may put them into your void or put them back in any order.";
    // Help text for anchored status.
    help_text_anchored = "<color=#AA00FF><b>Anchored</b></color>: Cannot be dissolved.";
    // Help text for reclaim without cost.
    help_text_reclaim_without_cost =
        "{@cap reclaim}: You may play a card from your void, then banish it when it leaves play.";
    // Help text for reclaim with energy cost.
    help_text_reclaim_with_cost($e) = "{@cap reclaim} {energy($e)}: You may play this card from your void for {energy($e)}, then banish it.";

    // =========================================================================
    // Token types
    // =========================================================================

    // Activated ability token type.
    token_type_activated_ability = "Activated Ability";
    // Triggered ability token type.
    token_type_triggered_ability = "Triggered Ability";
    // Reclaim ability token type.
    token_type_reclaim_ability = "Reclaim Ability";

    // =========================================================================
    // Cost serializer phrases
    // =========================================================================

    // Cost for discarding your entire hand.
    discard_your_hand_cost = "discard your hand";
    // Cost for paying one or more energy.
    pay_one_or_more_energy_cost = "pay 1 or more {energy_symbol}";
    // Connector for alternative costs.
    cost_or_connector = " or ";
    // Connector for combined costs.
    cost_and_connector = " and ";
    // Pay prefix for trigger costs.
    pay_prefix($cost) = "pay {$cost}";
    // Abandon any number of a target type (uses plural variant).
    abandon_any_number_of($target) = :from($target) "abandon any number of {$target:other}";
    // Abandon a specific target.
    abandon_target($target) = :from($target) "abandon {$target}";
    // Return a target to hand.
    return_target_to_hand($target) = :from($target) "return {$target} to hand";
    // Return a count of targets to hand (uses plural variant).
    return_count_to_hand($n, $target) = :from($target) "return {$n} {$target:other} to hand";
    // Return all but one target to hand.
    return_all_but_one_to_hand($target) = :from($target) "return all but one {$target} to hand";
    // Return all targets to hand.
    return_all_to_hand($target) = :from($target) "return all {$target} to hand";
    // Return any number of targets to hand.
    return_any_number_to_hand($target) = :from($target) "return any number of {$target} to hand";
    // Return up to N targets to hand (uses plural variant).
    return_up_to_to_hand($n, $target) = :from($target) "return up to {$n} {$target:other} to hand";
    // Return each other target to hand.
    return_each_other_to_hand($target) = :from($target) "return each other {$target} to hand";
    // Return N or more targets to hand (uses plural variant).
    return_or_more_to_hand($n, $target) = :from($target)
        "return {$n} or more {$target:other} to hand";

    // =========================================================================
    // Cost serializer phrases — parameterized
    // =========================================================================

    // Abandon a count of allies.
    abandon_count_allies($a) = "abandon {count_allies($a)}";
    // Discard a count of cards.
    discard_cards_cost($d) = "discard {cards($d)}";
    // Energy cost value.
    energy_cost_value($e) = "{energy($e)}";
    // Lose maximum energy cost.
    lose_max_energy_cost($m) = "lose {maximum_energy($m)}";
    // Banish your entire void.
    banish_your_void_cost = "{banish} your void";
    // Banish another card in your void.
    banish_another_in_void = "{banish} another card in your void";
    // Banish a count of cards from your void.
    banish_cards_from_void($c) = "{banish} {cards($c)} from your void";
    // Banish a count of cards from the opponent's void.
    banish_cards_from_enemy_void($c) = "{banish} {cards($c)} from the opponent's void";
    // Banish your void with a minimum card count.
    banish_void_min_count($n) = "{banish} your void with {count($n)} or more cards";
    // Banish a target from hand.
    banish_from_hand_cost($target) = :from($target) "{banish} {$target} from hand";

    // =========================================================================
    // Trigger serializer phrases
    // =========================================================================

    // End of your turn trigger.
    at_end_of_your_turn_trigger = "at the end of your turn, ";
    // Empty deck trigger.
    when_deck_empty_trigger = "when you have no cards in your deck, ";
    // Gain energy trigger.
    when_you_gain_energy_trigger = "when you gain energy, ";
    // Play a target trigger.
    when_you_play_trigger($target) = :from($target) "when you play {$target}, ";
    // Opponent plays a target trigger.
    when_opponent_plays_trigger($target) = :from($target) "when the opponent plays {$target}, ";
    // Play a target from hand trigger.
    when_you_play_from_hand_trigger($target) = :from($target)
        "when you play {$target} from your hand, ";
    // Play a target in a turn trigger.
    when_you_play_in_turn_trigger($target) = :from($target) "when you play {$target} in a turn, ";
    // Play a target during enemy turn trigger.
    when_you_play_during_enemy_turn_trigger($target) = :from($target)
        "when you play {$target} during the opponent's turn, ";
    // Discard a target trigger.
    when_you_discard_trigger($target) = :from($target) "when you discard {$target}, ";
    // Target leaves play trigger.
    when_leaves_play_trigger($target) = :from($target) "when {$target} leaves play, ";
    // Abandon a target trigger.
    when_you_abandon_trigger($target) = :from($target) "when you abandon {$target}, ";
    // Target put into void trigger.
    when_put_into_void_trigger($target) = :from($target) "when {$target} is put into your void, ";

    // =========================================================================
    // Trigger serializer phrases — parameterized
    // =========================================================================

    // Materialize a target trigger.
    when_you_materialize_trigger($target) = :from($target) "when you {materialize} {$target}, ";
    // Target dissolved trigger.
    when_dissolved_trigger($target) = :from($target) "when {$target} is {dissolved}, ";
    // Target banished trigger.
    when_banished_trigger($target) = :from($target) "when {$target} is {banished}, ";
    // Play N cards in a turn trigger.
    when_you_play_cards_in_turn_trigger($c) = "when you play {$c} {card:$c} in a turn, ";
    // Abandon N allies in a turn trigger.
    when_you_abandon_count_in_turn_trigger($a) = "when you abandon {count_allies($a)} in a turn, ";
    // Draw N cards in a turn trigger.
    when_you_draw_in_turn_trigger($c) = "when you draw {$c} {card:$c} in a turn, ";
    // Materialize Nth target in a turn trigger (uses plural variant).
    when_you_materialize_nth_in_turn_trigger($n, $target) = :from($target)
        "when you {materialize} {text_number($n)} {$target:other} in a turn, ";

    // =========================================================================
    // Condition serializer phrases
    // =========================================================================

    // Condition for a character dissolving this turn.
    if_character_dissolved_this_turn = "if a character dissolved this turn";
    // Condition for this card being in your void.
    if_card_in_your_void = "if this card is in your void,";
    // Condition for having discarded a target this turn.
    if_discarded_this_turn($target) = :from($target) "if you have discarded {$target} this turn";
    // Condition wrapper for a predicate count (uses plural variant).
    with_predicate_condition($pred) = :from($pred) "with {$pred:other},";

    // =========================================================================
    // Condition serializer phrases — parameterized
    // =========================================================================

    // Condition for allies sharing a character type.
    with_allies_sharing_type($a) = "with {count_allies($a)} that share a character type,";
    // Condition for drawing a count of cards this turn.
    if_drawn_count_this_turn($n) = "if you have drawn {count($n)} or more cards this turn";
    // Condition for having a count of cards in your void.
    while_void_count($n) = "while you have {count($n)} or more cards in your void,";
    // Condition for having an allied subtype.
    with_allied_subtype($t) = :from($t) "with an allied {subtype($t)},";
    // Condition for a count of allied characters with a subtype.
    with_count_allied_subtype($a, $t) = "{count_allied_subtype($a, $t)}";
    // Condition for a count of allies.
    with_count_allies($a) = "{count_allies($a)}";

    // =========================================================================
    // Operator phrases
    // =========================================================================

    // Operator suffix for "or less" comparisons.
    operator_or_less = " or less";
    // Operator suffix for "or more" comparisons.
    operator_or_more = " or more";
    // Operator suffix for "lower" comparisons.
    operator_lower = " lower";
    // Operator suffix for "higher" comparisons.
    operator_higher = " higher";

    // =========================================================================
    // Effect serializer phrases — parameterized
    // =========================================================================

    // Draw cards effect fragment (no trailing period).
    draw_cards_effect($c) = "draw {cards($c)}";
    // Discard cards effect fragment (no trailing period).
    discard_cards_effect($d) = "discard {cards($d)}";
    // Gain energy effect fragment (no trailing period).
    gain_energy_effect($e) = "gain {energy($e)}";
    // Gain points effect fragment (no trailing period).
    gain_points_effect($p) = "gain {points($p)}";
    // Lose points effect fragment (no trailing period).
    lose_points_effect($p) = "you lose {points($p)}";
    // Opponent gains points effect fragment (no trailing period).
    opponent_gains_points_effect($p) = "the opponent gains {points($p)}";
    // Opponent loses points effect fragment (no trailing period).
    opponent_loses_points_effect($p) = "the opponent loses {points($p)}";
    // Foresee effect fragment (no trailing period).
    foresee_effect($f) = "{foresee($f)}";
    // Kindle effect fragment (no trailing period).
    kindle_effect($k) = "{kindle($k)}";
    // Each player discards effect fragment (no trailing period).
    each_player_discards_effect($d) = "each player discards {cards($d)}";
    // Prevent that card effect fragment (no trailing period).
    prevent_that_card_effect = "{prevent} that card";
    // Then materialize it effect fragment (no trailing period).
    // Accepts antecedent $target for gendered pronoun agreement in translations.
    then_materialize_it_effect($target) = "then {materialize} it";
    // Gain twice energy instead effect fragment (no trailing period).
    gain_twice_energy_instead_effect = "gain twice that much {energy_symbol} instead";
    // Gain energy equal to that character's cost effect fragment (no trailing period).
    gain_energy_equal_to_that_cost_effect = "gain {energy_symbol} equal to that character's cost";
    // Gain energy equal to this character's cost effect fragment (no trailing period).
    gain_energy_equal_to_this_cost_effect = "gain {energy_symbol} equal to this character's cost";
    // Put top cards of deck into void effect fragment (no trailing period).
    put_deck_into_void_effect($v) = "put the {top_n_cards($v)} of your deck into your void";
    // Banish cards from enemy void effect fragment (no trailing period).
    banish_cards_from_enemy_void_effect($c) = "{banish} {cards($c)} from the opponent's void";
    // Banish enemy void effect fragment (no trailing period).
    banish_enemy_void_effect = "{banish} the opponent's void";
    // Judgment phase at end of turn effect fragment (no trailing period).
    judgment_phase_at_end_of_turn_effect =
        "at the end of this turn, trigger an additional {judgment_phase_name} phase";
    // Multiply energy effect fragment (no trailing period).
    multiply_energy_effect($n) = "{multiply_by($n)} the amount of {energy_symbol} you have";
    // Spend all energy dissolve effect fragment (no trailing period).
    spend_all_energy_dissolve_effect = "spend all your {energy_symbol}. {dissolve} an enemy with cost less than or equal to the amount spent";
    // Spend all energy draw discard effect fragment (no trailing period).
    spend_all_energy_draw_discard_effect = "spend all your {energy_symbol}. Draw cards equal to the amount spent, then discard that many cards";
    // Each player shuffles and draws effect fragment (no trailing period).
    each_player_shuffles_and_draws_effect($c) =
        "each player shuffles their hand and void into their deck and then draws {cards($c)}";
    // Return up to events from void effect fragment (no trailing period).
    return_up_to_events_from_void_effect($n) =
        "return {up_to_n_events($n)} from your void to your hand";
    // Fast prefix for activated abilities.
    fast_prefix = "{Fast} -- ";

    // =========================================================================
    // Effect serializer — predicate-consuming phrases
    // =========================================================================

    // Discard a chosen card from the opponent's hand.
    discard_chosen_from_enemy_hand($target) = :from($target)
        "discard a chosen {$target} from the opponent's hand";
    // Discard a chosen card from the opponent's hand, then they draw.
    discard_chosen_from_enemy_hand_then_draw($target) = :from($target)
        "discard a chosen {$target} from the opponent's hand. They draw {cards(1)}";
    // Put a card from your void on top of your deck.
    put_from_void_on_top_of_deck($target) = :from($target)
        "put {$target} from your void on top of your deck";
    // Put up to N cards from your void on top of your deck.
    put_up_to_from_void_on_top_of_deck($n, $target) =
        "put up to {cards($n)} {$target} from your void on top of your deck";
    // Materialize random characters from your deck.
    materialize_random_from_deck($n, $constraint) =
        "{materialize} {n_random_characters($n)} {$constraint} from your deck";
    // Copy the next card you play this turn.
    copy_next_played($target, $times) = :from($target)
        "copy the next {$target} you play {this_turn_times($times)}";
    // Create a trigger until end of turn with keyword trigger.
    create_trigger_until_end_of_turn_keyword($trig, $eff) =
        "until end of turn, {$trig} {@cap $eff}";
    // Create a trigger until end of turn.
    create_trigger_until_end_of_turn($trig, $eff) = "until end of turn, {$trig}{$eff}";
    // Dissolve a target.
    dissolve_target($target) = :from($target) "{dissolve} {$target}";
    // Banish a target.
    banish_target($target) = :from($target) "{banish} {$target}";
    // Banish a target until another leaves play.
    banish_until_leaves($target, $until) = :from($target)
        "{banish} {$target} until {$until} leaves play";
    // Banish a target until your next main phase.
    banish_until_next_main($target) = :from($target)
        "{banish} {$target} until your next main phase";
    // Banish a target when it leaves play.
    banish_when_leaves_play($target) = :from($target) "{banish} {$target} when it leaves play";
    // Gain control of a target.
    gain_control_of($target) = :from($target) "gain control of {$target}";
    // Discover a card predicate.
    discover_target($target) = :from($target) "{discover} {$target}";
    // Discover a card and materialize it.
    discover_and_materialize($target) = :from($target) "{discover} {$target} and {materialize} it";
    // Materialize a target.
    materialize_target($target) = :from($target) "{materialize} {$target}";
    // Materialize a target at end of turn.
    materialize_at_end_of_turn($target) = :from($target) "{materialize} {$target} at end of turn";
    // Materialize a target from your void.
    materialize_from_void($target) = :from($target) "{materialize} {$target} from your void";
    // Return a target to hand.
    return_to_hand($target) = :from($target) "return {$target} to hand";
    // Return this character to your hand.
    return_this_to_hand = "return this character to your hand";
    // Return an enemy or ally to hand.
    return_any_character_to_hand = "return an enemy or ally to hand";
    // Return an ally to hand.
    return_ally_to_hand = "return an ally to hand";
    // Return a target from your void to your hand.
    return_from_void_to_hand($target) = :from($target)
        "return {$target} from your void to your hand";
    // Reclaim a target from your void.
    reclaim_target($target) = :from($target) "{reclaim} {$target}";
    // Reclaim a random card type.
    reclaim_random($target) = :from($target) "{reclaim} a random {$target}";
    // Put a target on top of the opponent's deck.
    put_on_top_of_enemy_deck($target) = :from($target)
        "put {$target} on top of the opponent's deck";
    // Copy a target.
    copy_target($target) = :from($target) "copy {$target}";
    // Disable activated abilities of a target while this is in play.
    disable_activated_abilities($target) = :from($target)
        "disable the activated abilities of {$target} while this character is in play";
    // Draw a matching card from your deck.
    draw_matching_from_deck($target) = :from($target) "draw {$target} from your deck";
    // Abandon a target and gain energy for its spark.
    abandon_and_gain_energy_for_spark($target) = :from($target)
        "abandon {$target} and gain {energy_symbol} for each point of spark that character had";
    // Abandon a target at end of turn.
    abandon_at_end_of_turn($target) = :from($target) "abandon {$target} at end of turn";
    // Each player abandons a matching card.
    each_player_abandons($target) = :from($target) "each player abandons {$target}";
    // Target cannot be dissolved this turn.
    prevent_dissolve_this_turn($target) = :from($target)
        "{$target} cannot be {dissolved} this turn";
    // Prevent a played target.
    prevent_played_target($target) = :from($target) "{prevent} a played {$target}";
    // Prevent a played target unless opponent pays cost.
    prevent_unless_pays($target, $cost) = :from($target)
        "{prevent} a played {$target} unless the opponent pays {$cost}";
    // Gain energy equal to a target's cost.
    gain_energy_equal_to_cost($target) = :from($target)
        "gain {energy_symbol} equal to {$target}'s cost";
    // Target gains spark.
    gains_spark($target, $s) = :from($target) "{$target} gains +{$s} spark";
    // Opponent pays a cost.
    opponent_pays_cost($cost) = "the opponent pays {$cost}";
    // Pay a cost.
    pay_cost_effect($cost) = "pay {$cost}";

    // =========================================================================
    // Effect serializer phrases — standalone
    // =========================================================================

    // Opponent gains points equal to spark effect fragment (no trailing period).
    // Accepts antecedent $target for gendered pronoun agreement in translations.
    opponent_gains_points_equal_spark($target) = "the opponent gains points equal to its spark";
    // Take extra turn effect fragment (no trailing period).
    take_extra_turn_effect = "take an extra turn after this one";
    // You win the game effect fragment (no trailing period).
    you_win_the_game_effect = "you win the game";
    // No effect.
    no_effect = "";

    // =========================================================================
    // Structural phrases
    // =========================================================================

    // Optional action prefix for abilities the player may choose to activate.
    you_may_prefix = "you may ";
    // Connects a cost to its effect in activated abilities (e.g., "{cost} to {effect}").
    cost_to_connector($cost) = "{$cost} to ";
    // Prefix for effects that last until the end of the current turn.
    until_end_of_turn_prefix = "Until end of turn, ";
    // Prefix for abilities that can only be used once per turn.
    once_per_turn_prefix = "Once per turn, ";
    // Suffix for abilities that can only be used once per turn.
    once_per_turn_suffix = ", once per turn";
    // Joins cost and effect in activated abilities (e.g., "cost: effect").
    cost_effect_separator = ": ";
    // Joins sequential effects in ability text (e.g., "draw a card, then dissolve").
    then_joiner = ", then ";
    // Joins parallel effects in ability text (e.g., "draw a card and gain 1 energy").
    and_joiner = " and ";
    // Joins separate effect sentences in event context (e.g., "Draw a card. Gain 1 energy.").
    sentence_joiner = ". ";
    // Joins pre-punctuated effect sentences (e.g., "Draw a card." + " " + "Gain 1 energy.").
    sentence_separator = " ";
    // Terminal punctuation for ability text.
    period_suffix = ".";

    // =========================================================================
    // Predicate base entity nouns
    // =========================================================================

    // Character noun with article metadata.
    character = :a { one: "character", other: "characters" };
    // Event noun with article metadata.
    event = :an { one: "event", other: "events" };
    // Enemy noun with article metadata.
    enemy = :an { one: "enemy", other: "enemies" };

    // This card noun with article metadata.
    this_card = :a { one: "this card", other: "these cards" };
    // This character noun with article metadata.
    this_character = :a { one: "this character", other: "these characters" };
    // This event noun with article metadata.
    this_event = :an { one: "this event", other: "these events" };
    // That character noun with article metadata.
    that_character = :a { one: "that character", other: "those characters" };
    // Predicate pronoun, variant-aware.
    pronoun_it = { *one: "it", other: "them" };
    // Plural predicate pronoun.
    pronoun_them = "them";
    // Selects the :other (plural) variant of a phrase as its default text.
    as_plural($p) = :from($p) "{$p:other}";
    // Helper for packaging asymmetric singular/plural as variants.
    // The $other parameter should be an already-pluralized phrase.
    with_plural($one, $other) = :from($one) { *one: "{$one}", other: "{$other}" };
    // Asymmetric Your(Character): singular="character", plural="allies".
    your_generic_character = :a { *one: "character", other: "allies" };
    // Asymmetric Your(Card): singular="card", plural="your cards".
    your_generic_card = :a { *one: "card", other: "your cards" };
    // Asymmetric Your(Event): singular="event", plural="your events".
    your_generic_event = :an { *one: "event", other: "your events" };
    // Asymmetric Your(CharacterType): singular=subtype, plural=allied subtypes.
    your_generic_subtype($t) = :from($t) {
        *one: "{subtype($t)}",
        other: "allied {subtype($t):other}",
    };
    // Applies an English indefinite article to a predicate noun, variant-aware.
    predicate_with_indefinite_article($p) = :from($p) { *one: "{@a $p}", other: "{$p}" };
    // Another/other qualifying prefix for a predicate noun, variant-aware.
    another_pred($p) = :from($p) { *one: "another {$p}", other: "other {$p}" };

    // =========================================================================
    // Ownership-qualified predicate nouns
    // =========================================================================

    // Your card noun with article metadata.
    your_card = :a { one: "your card", other: "your cards" };
    // Your character noun with article metadata.
    your_character = :a { one: "your character", other: "your characters" };
    // Your event noun with article metadata.
    your_event = :a { one: "your event", other: "your events" };
    // Enemy card noun with article metadata.
    enemy_card = :an { one: "enemy card", other: "enemy cards" };
    // Enemy character noun with article metadata.
    enemy_character = :an { one: "enemy character", other: "enemy characters" };
    // Enemy event noun with article metadata.
    enemy_event = :an { one: "enemy event", other: "enemy events" };
    // Allied character noun with article metadata.
    allied_character = :an { one: "allied character", other: "allied characters" };
    // Allied event noun with article metadata.
    allied_event = :an { one: "allied event", other: "allied events" };
    // Other character noun with article metadata.
    other_character = :an { one: "other character", other: "other characters" };
    // Allied predicate noun, variant-aware.
    allied_pred($base) = :from($base) "allied {$base}";
    // Enemy predicate noun, variant-aware.
    enemy_pred($base) = :from($base) "enemy {$base}";
    // Predicate noun in your void location, variant-aware.
    in_your_void($target) = :from($target) "{$target} in your void";
    // Predicate noun in the opponent's void location, variant-aware.
    in_opponent_void($target) = :from($target) "{$target} in the opponent's void";
    // Predicate noun in your hand location.
    in_your_hand($target) = :from($target) "{$target} in your hand";

    // Cost constraint phrase.
    with_cost_constraint($op, $val) = "with cost {energy($val)}{$op}";
    // Spark constraint phrase.
    with_spark_constraint($op, $val) = "with spark {$val}{$op}";
    // Predicate with composed constraint, variant-aware.
    pred_with_constraint($base, $constraint) = :from($base) "{$base} {$constraint}";
    // Non-subtype qualifier, variant-aware.
    non_subtype($s) = :from($s) "non-{$s}";
    // Relative clause for events that could dissolve a target.
    could_dissolve_target($target) = :an "event which could {dissolve} {$target}";
    // Plural clause for events that could dissolve a target.
    could_dissolve_target_plural($target) = :from($target)
        "events which could {dissolve} {$target}";
    // Owned event that could dissolve a target.
    your_event_could_dissolve($target) = :from($target)
        "your event which could {dissolve} {$target}";
    // Plural owned events that could dissolve a target.
    your_event_could_dissolve_plural($target) = :from($target)
        "your events which could {dissolve} {$target}";
    // Fast predicate prefix.
    fast_predicate($target) = :a "{fast} {$target}";
    // Fast predicate prefix plural.
    fast_predicate_plural($target) = :from($target) "{fast} {$target}";
    // Constraint for materialized abilities (singular).
    with_materialized_ability_constraint = "with a {materialized} ability";
    // Constraint for materialized abilities (plural).
    with_materialized_abilities_constraint = "with {materialized} abilities";
    // Constraint for activated abilities (singular).
    with_activated_ability_constraint = "with an activated ability";
    // Constraint for activated abilities (plural).
    with_activated_abilities_constraint = "with activated abilities";
    // Constraint for spark compared to energy spent.
    with_spark_less_than_energy_paid_constraint =
        "with spark less than the amount of {energy_symbol} paid";
    // Constraint for cost compared to allied count (uses plural variant).
    with_cost_less_than_allied_count($target) = :from($target)
        "with cost less than the number of allied {$target:other}";
    // Constraint for cost compared to abandoned ally.
    with_cost_less_than_abandoned_ally_constraint = "with cost less than the abandoned ally's cost";
    // Constraint for spark compared to abandoned ally.
    with_spark_less_than_abandoned_ally_constraint =
        "with spark less than the abandoned ally's spark";
    // Constraint for spark compared to abandoned ally count.
    with_spark_less_than_abandoned_count_this_turn_constraint =
        "with spark less than the number of allies abandoned this turn";
    // Constraint for cost compared to void count.
    with_cost_less_than_void_count_constraint =
        "with cost less than the number of cards in your void";
    // Constraint for spark compared to that ally's spark.
    with_spark_less_than_that_ally_constraint = "with spark less than that ally's spark";

    // =========================================================================
    // Compound predicate nouns with subtype propagation
    // =========================================================================

    // Allied subtype noun.
    allied_subtype($t) = :an "allied {subtype($t)}";
    // Allied subtype plural noun.
    allied_subtype_plural($t) = :from($t) "allied {subtype($t):other}";
    // Enemy subtype noun.
    enemy_subtype($t) = :an "enemy {subtype($t)}";
    // Enemy subtype plural noun.
    enemy_subtype_plural($t) = :from($t) "enemy {subtype($t):other}";
    // Your subtype noun inheriting article metadata from subtype.
    your_subtype($t) = :from($t) "your {$t}";
    // Other subtype noun inheriting article metadata from subtype.
    other_subtype($t) = :from($t) "other {$t}";
    // Subtype in your void inheriting article metadata from subtype.
    subtype_in_your_void($t) = :from($t) "{$t} in your void";
    // Character that is not a subtype.
    character_not_subtype($s) = :a "character that is not {@a subtype($s)}";
    // Characters that are not a subtype (plural).
    character_not_subtype_plural($s) = :from($s) "characters that are not {subtype($s):other}";
    // Ally that is not a subtype.
    ally_not_subtype($s) = :an "ally that is not {@a subtype($s)}";
    // Allies that are not a subtype (plural).
    ally_not_subtype_plural($s) = :from($s) "allies that are not {subtype($s):other}";
    // Enemy that is not a subtype.
    non_subtype_enemy($s) = :a "non-{subtype($s)} enemy";
    // Enemies that are not a subtype (plural).
    non_subtype_enemy_plural($s) = :from($s) "non-{subtype($s):other} enemies";

    // =========================================================================
    // For-each predicate phrases
    // =========================================================================

    // For-each ally phrase (without article).
    for_each_ally = "ally";
    // For-each allied character phrase.
    for_each_allied_character = "allied character";
    // For-each enemy phrase.
    for_each_enemy = "enemy";
    // For-each character phrase.
    for_each_character = "character";
    // For-each card phrase.
    for_each_card = "card";
    // For-each card in your void phrase.
    for_each_card_in_your_void = "card in your void";
    // For-each this character phrase.
    for_each_this_character = "this character";
    // For-each that character phrase.
    for_each_that_character = "that character";
    // For-each other character phrase.
    for_each_other_character = "other character";
    // For-each allied event phrase.
    for_each_allied_event = "allied event";
    // For-each event phrase.
    for_each_event = "event";
    // For-each character in your void phrase.
    for_each_character_in_your_void = "character in your void";
    // For-each event in your void phrase.
    for_each_event_in_your_void = "event in your void";
    // For-each card in the opponent's void phrase.
    for_each_card_in_enemy_void = "card in the opponent's void";
    // For-each character in the opponent's void phrase.
    for_each_character_in_enemy_void = "character in the opponent's void";
    // For-each event in the opponent's void phrase.
    for_each_event_in_enemy_void = "event in the opponent's void";
    // For-each allied subtype phrase inheriting article metadata.
    for_each_allied_subtype($t) = :from($t) "allied {subtype($t)}";
    // For-each enemy subtype phrase inheriting article metadata.
    for_each_enemy_subtype($t) = :from($t) "enemy {subtype($t)}";
    // For-each subtype phrase inheriting article metadata.
    for_each_subtype($t) = :from($t) "{subtype($t)}";
    // For-each other subtype phrase inheriting article metadata.
    for_each_other_subtype($t) = :from($t) "other {subtype($t)}";
    // For-each subtype in your void phrase inheriting article metadata.
    for_each_subtype_in_your_void($t) = :from($t) "{subtype($t)} in your void";
    // For-each ally with spark condition phrase.
    for_each_ally_with_spark($s, $op) = "ally with spark {$s}{$op}";
    // Generic for-each predicate phrase.
    for_each_predicate($target) = :from($target) "each {$target}";

    // =========================================================================
    // Count expression phrases
    // =========================================================================

    // Ally abandoned this turn for count expressions.
    ally_abandoned_this_turn = "ally abandoned this turn";
    // Allied subtype abandoned this turn for count expressions.
    allied_subtype_abandoned_this_turn($t) = :from($t) "allied {subtype($t)} abandoned this turn";
    // Ally abandoned (this way) for count expressions.
    ally_abandoned = "ally abandoned";
    // Allied subtype abandoned (this way) for count expressions.
    allied_subtype_abandoned($t) = :from($t) "allied {subtype($t)} abandoned";
    // Ally returned to hand for count expressions.
    ally_returned = "ally returned";
    // Allied subtype returned to hand for count expressions.
    allied_subtype_returned($t) = :from($t) "allied {subtype($t)} returned";
    // Generic card predicate returned for count expressions.
    card_predicate_returned($base) = :from($base) "{$base} returned";
    // Energy spent for count expressions.
    energy_spent = "{energy_symbol} spent";
    // Card predicate played this turn for count expressions.
    card_predicate_played_this_turn($base) = :from($base) "{$base} you have played this turn";
    // Card predicate drawn this turn for count expressions.
    card_predicate_drawn_this_turn($base) = :from($base) "{$base} you have drawn this turn";
    // Card predicate discarded this turn for count expressions.
    card_predicate_discarded_this_turn($base) = :from($base) "{$base} you have discarded this turn";
    // Card predicate dissolved this turn for count expressions.
    card_predicate_dissolved_this_turn($base) = :from($base) "{$base} which dissolved this turn";
    // Generic card predicate abandoned this turn for count expressions.
    card_predicate_abandoned_this_turn($base) = :from($base) "{$base} abandoned this turn";
    // Generic card predicate abandoned for count expressions.
    card_predicate_abandoned($base) = :from($base) "{$base} abandoned";

    // =========================================================================
    // For-each effect phrases
    // =========================================================================

    // Draw cards for each matching predicate.
    draw_cards_for_each($c, $target) = "draw {cards($c)} for each {$target}";
    // Gain energy for each matching predicate.
    gain_energy_for_each($e, $target) = "gain {energy($e)} for each {$target}";
    // Gain points for each matching quantity.
    gain_points_for_each($p, $target) = "gain {points($p)} for each {$target}";
    // Target gains spark for each matching quantity.
    gains_spark_for_each($target, $s, $quantity) = :from($target)
        "{$target} gains +{$s} spark for each {$quantity}";
    // Target gains spark until next main phase for each matching predicate.
    gains_spark_until_next_main_for_each($target, $s, $for_each) = :from($target)
        "{$target} gains +{$s} spark until your next main phase for each {$for_each}";
    // Each matching gains spark equal to count of another group.
    each_gains_spark_equal_to($each, $count_of) = :from($each)
        "each {$each} gains spark equal to the number of {$count_of:other}";
    // Have each matching gain spark.
    have_each_gain_spark($each, $s) = :from($each) "have each {$each} gain +{$s} spark";
    // Spark of each matching becomes a value.
    spark_of_each_becomes($each, $s) = :from($each) "the spark of each {$each} becomes {$s}";
    // Dissolve all with cost less than or equal to quantity count (uses plural variant).
    dissolve_all_with_cost_lte_quantity($target, $quantity) = :from($target)
        "{dissolve} all {$target:other} with cost less than or equal to the number of {$quantity}";

    // =========================================================================
    // Collection expression target phrases
    // =========================================================================

    // All of a target (uses plural variant).
    collection_all($target) = :from($target) "all {$target:other}";
    // Exactly N of a target (uses plural variant).
    collection_exactly($n, $target) = :from($target) "{$n} {$target:other}";
    // Up to N of a target (uses plural variant).
    collection_up_to($n, $target) = :from($target) "up to {$n} {$target:other}";
    // Any number of a target (uses plural variant).
    collection_any_number_of($target) = :from($target) "any number of {$target:other}";

    // =========================================================================
    // Collection effect phrases
    // =========================================================================

    // Dissolve a collection target.
    dissolve_collection($target) = :from($target) "{dissolve} {$target}";
    // Banish a collection target.
    banish_collection_target($target) = :from($target) "{banish} {$target}";

    // Materialize them.
    // Accepts antecedent $target for gendered pronoun agreement in translations.
    materialize_them($target) = "{materialize} them";
    // Materialize a collection target.
    materialize_collection_target($target) = :from($target) "{materialize} {$target}";

    // Materialize a copy of a target.
    materialize_copy_of($target) = :from($target) "{materialize} a copy of {$target}";
    // Materialize N copies of a target.
    materialize_n_copies_of($n, $target) = "{materialize} {$n} copies of {$target}";
    // Materialize copies of target equal to count of matching (uses plural variant).
    materialize_copies_equal_to_matching($target, $matching) = :from($target)
        "{materialize} a number of copies of {$target} equal to the number of {$matching:other}";
    // Materialize copies of target equal to energy spent.
    materialize_copies_equal_to_energy($target) = :from($target) "{materialize} a number of copies of {$target} equal to the amount of {energy_symbol} spent";
    // Materialize copies of target equal to quantity.
    materialize_copies_equal_to_quantity($target, $quantity) = :from($target)
        "{materialize} a number of copies of {$target} equal to the number of {$quantity}";

    // Trigger judgment ability of a collection target.
    trigger_judgment_of_collection($target) = :from($target)
        "trigger the {Judgment} ability of {$target}";
    // Trigger judgment ability of each matching target.
    trigger_judgment_of_each($target) = :from($target)
        "trigger the {Judgment} ability of each {$target}";

    // =========================================================================
    // Materialize figment quantity phrases
    // =========================================================================

    // Materialize figments for each quantity.
    materialize_figments_for_each_quantity($fig, $quantity) = :from($fig)
        "{materialize} {$fig} for each {$quantity}";

    // =========================================================================
    // Banish then materialize phrases
    // =========================================================================

    // Banish a single target then materialize it.
    banish_then_materialize_it($target) = :from($target)
        "{banish} {$target}, then {materialize} it";
    // Banish any number of targets then materialize them (uses plural variant).
    banish_then_materialize_any_number($target) = :from($target)
        "{banish} any number of {$target:other}, then {materialize} them";
    // Banish up to N allies then materialize them (uses plural variant).
    banish_then_materialize_up_to($n, $target) = :from($target)
        "{banish} up to {$n} {$target:other}, then {materialize} {pronoun:$n}";
    // Banish targets then materialize them (default plural).
    banish_then_materialize_them($target) = :from($target)
        "{banish} {$target}, then {materialize} them";

    // =========================================================================
    // Allied card predicate phrases
    // =========================================================================

    // Allied card predicate with subtype, variant-aware.
    allied_card_with_subtype($t) = :from($t) {
        *one: "allied {subtype($t)}",
        other: "allied {subtype($t):other}",
    };
    // Allied card predicate with base text, variant-aware.
    allied_card_with_base($base) = :from($base) "allied {$base}";

    // =========================================================================
    // Gains reclaim effect phrases
    // =========================================================================

    // It gains reclaim with cost.
    // Accepts antecedent $target for gendered pronoun agreement in translations.
    it_gains_reclaim_for_cost($target, $r) = "it gains {reclaim_for_cost($r)}";
    // It gains reclaim equal to its cost.
    // Accepts antecedent $target for gendered pronoun agreement in translations.
    it_gains_reclaim_equal_cost($target) = "it gains {reclaim} equal to its cost";
    // This card gains reclaim with cost.
    this_card_gains_reclaim_for_cost($r) = "this card gains {reclaim_for_cost($r)}";
    // This card gains reclaim equal to its cost.
    this_card_gains_reclaim_equal_cost = "this card gains {reclaim} equal to its cost";
    // Target gains reclaim with cost.
    target_gains_reclaim_for_cost($target, $r) = :from($target)
        "{$target} gains {reclaim_for_cost($r)}";
    // Target gains reclaim equal to its cost.
    target_gains_reclaim_equal_cost($target) = :from($target)
        "{$target} gains {reclaim} equal to its cost";

    // It gains reclaim with cost this turn.
    // Accepts antecedent $target for gendered pronoun agreement in translations.
    it_gains_reclaim_for_cost_this_turn($target, $r) = "it gains {reclaim_for_cost($r)} this turn";
    // It gains reclaim equal to its cost this turn.
    // Accepts antecedent $target for gendered pronoun agreement in translations.
    it_gains_reclaim_equal_cost_this_turn($target) =
        "it gains {reclaim} equal to its cost this turn";
    // This card gains reclaim with cost this turn.
    this_card_gains_reclaim_for_cost_this_turn($r) =
        "this card gains {reclaim_for_cost($r)} this turn";
    // This card gains reclaim equal to its cost this turn.
    this_card_gains_reclaim_equal_cost_this_turn =
        "this card gains {reclaim} equal to its cost this turn";
    // Target gains reclaim with cost this turn.
    target_gains_reclaim_for_cost_this_turn($target, $r) = :from($target)
        "{$target} gains {reclaim_for_cost($r)} this turn";
    // Target gains reclaim equal to its cost this turn.
    target_gains_reclaim_equal_cost_this_turn($target) = :from($target)
        "{$target} gains {reclaim} equal to its cost this turn";

    // =========================================================================
    // Void collection subject phrases
    // =========================================================================

    // A single card in your void (subject), capitalizing the predicate.
    void_subject_single($pred) = :from($pred) "{@cap $pred} in your void";
    // Exactly N cards in your void (subject, uses plural variant).
    void_subject_exactly($n, $pred) = :from($pred) "{$n} {$pred:other} in your void";
    // All cards currently in your void (subject).
    void_subject_all = "all cards currently in your void";
    // All but one cards in your void (subject, uses plural variant).
    void_subject_all_but_one($pred) = :from($pred) "all but one {$pred:other} in your void";
    // Up to N cards in your void (subject, uses plural variant).
    void_subject_up_to($n, $pred) = :from($pred) "up to {$n} {$pred:other} in your void";
    // Any number of cards in your void (subject, uses plural variant).
    void_subject_any_number($pred) = :from($pred) "any number of {$pred:other} in your void";
    // N or more cards in your void (subject, uses plural variant).
    void_subject_or_more($n, $pred) = :from($pred) "{$n} or more {$pred:other} in your void";
    // Each other card in your void (subject).
    void_subject_each_other = "Each other card in your void";

    // =========================================================================
    // Void gains reclaim assembly phrases
    // =========================================================================

    // Singular subject gains reclaim for a specific cost.
    void_gains_reclaim_for_cost_singular($subject, $r) = :from($subject)
        "{$subject} gains {reclaim_for_cost($r)}";
    // Singular subject gains reclaim equal to its cost.
    void_gains_reclaim_equal_cost_singular($subject) = :from($subject)
        "{$subject} gains {reclaim} equal to its cost";
    // Plural subject gains reclaim for a specific cost.
    void_gains_reclaim_for_cost_plural($subject, $r) = :from($subject)
        "{$subject} gain {reclaim_for_cost($r)}";
    // Plural subject gains reclaim equal to their cost.
    void_gains_reclaim_equal_cost_plural($subject) = :from($subject)
        "{$subject} gain {reclaim} equal to their cost";
    // Singular subject gains reclaim for a specific cost this turn.
    void_gains_reclaim_for_cost_singular_this_turn($subject, $r) = :from($subject)
        "{$subject} gains {reclaim_for_cost($r)} this turn";
    // Singular subject gains reclaim equal to its cost this turn.
    void_gains_reclaim_equal_cost_singular_this_turn($subject) = :from($subject)
        "{$subject} gains {reclaim} equal to its cost this turn";
    // Plural subject gains reclaim for a specific cost this turn.
    void_gains_reclaim_for_cost_plural_this_turn($subject, $r) = :from($subject)
        "{$subject} gain {reclaim_for_cost($r)} this turn";
    // Plural subject gains reclaim equal to their cost this turn.
    void_gains_reclaim_equal_cost_plural_this_turn($subject) = :from($subject)
        "{$subject} gain {reclaim} equal to their cost this turn";

    // =========================================================================
    // Static ability serializer phrases
    // =========================================================================

    // Your matching cards cost more energy (uses plural variant).
    your_cards_cost_increase($matching, $e) = :from($matching)
        "{$matching:other} cost you {energy($e)} more";
    // Your matching cards cost less energy (uses plural variant).
    your_cards_cost_reduction($matching, $e) = :from($matching)
        "{$matching:other} cost you {energy($e)} less";
    // The opponent's matching cards cost more energy (uses plural variant).
    enemy_cards_cost_increase($matching, $e) = :from($matching)
        "the opponent's {$matching:other} cost {energy($e)} more";
    // Allied matching characters have bonus spark (uses plural variant).
    spark_bonus_other_characters($matching, $s) = :from($matching)
        "allied {$matching:other} have +{$s} spark";
    // To play this card, pay an additional cost.
    additional_cost_to_play($cost) = "To play {this_card}, {$cost}";
    // This card type costs alternate energy.
    play_for_alternate_cost_simple($card_type, $e) = "{$card_type} costs {energy($e)}";
    // Additional cost prefix: Play this card type for alternate energy.
    play_for_alternate_cost_with_additional($cost, $card_type, $e) =
        "{$cost}: Play {$card_type} for {energy($e)}";
    // Additional cost with if-you-do abandon effect.
    play_for_alternate_cost_abandon($cost, $card_type, $e) =
        "{$cost}: Play {$card_type} for {energy($e)}, then abandon it";
    // Characters in your hand have fast.
    characters_in_hand_have_fast = "characters in your hand have {fast}";
    // Disable the materialized abilities of enemies.
    disable_enemy_materialized_abilities = "disable the {Materialized} abilities of enemies";
    // Has all character types.
    has_all_character_types = "has all character types";
    // Multiply the amount of energy gained from card effects.
    multiply_energy_gain($n) =
        "{multiply_by($n)} the amount of {energy_symbol} you gain from card effects this turn";
    // Multiply the number of cards drawn from card effects.
    multiply_card_draw($n) =
        "{multiply_by($n)} the number of cards you draw from card effects this turn";
    // Once per turn, play a matching card from your void.
    once_per_turn_play_from_void($matching) = :from($matching)
        "once per turn, you may play {$matching} from your void";
    // Reveal the top card of your deck.
    reveal_top_card = "reveal the top card of your deck";
    // You may look at the top card of your deck.
    you_may_look_at_top_card = "you may look at the top card of your deck";
    // You may play matching cards from the top of your deck (uses plural variant).
    you_may_play_from_top_of_deck($matching) = :from($matching)
        "you may play {$matching:other} from the top of your deck";
    // Judgment ability of matching characters triggers when materialized (uses plural variant).
    judgment_triggers_when_materialized($matching) = :from($matching)
        "the '{Judgment}' ability of {$matching:other} triggers when you {materialize} them";
    // This character's spark equals predicate count (uses plural variant).
    spark_equal_to_predicate_count($matching) = :from($matching)
        "{this_character}'s spark is equal to the number of {$matching:other}";
    // You may only play this character from your void.
    play_only_from_void = "you may only play {this_character} from your void";
    // You may play this card from your hand or void for a cost.
    play_from_hand_or_void_for_cost($e) =
        "you may play {this_card} from your hand or void for {energy($e)}";
    // Cards in your void have reclaim equal to their cost.
    cards_in_void_have_reclaim = "they have {reclaim} equal to their cost";
    // This card costs less for each matching quantity.
    cost_reduction_for_each($e, $quantity) =
        "{this_card} costs {energy($e)} less for each {$quantity}";
    // Your matching characters have bonus spark (uses plural variant).
    spark_bonus_your_characters($matching, $s) = :from($matching)
        "{$matching:other} have +{$s} spark";
    // Play this card from your void for a cost.
    play_from_void_for_cost($e) = "play {this_card} from your void for {energy($e)}";
    // Play this card from your void with additional cost prefix.
    play_from_void_with_additional_cost($cost, $e) =
        "{$cost}: play {this_card} from your void for {energy($e)}";
    // Play this card from your void with additional cost and if-you-do effect.
    play_from_void_with_effect($cost, $e, $effect) =
        "{$cost}: play {this_card} from your void for {energy($e)}, then {$effect}";
    // Play this card from your void for cost with if-you-do effect.
    play_from_void_for_cost_with_effect($e, $effect) =
        "play {this_card} from your void for {energy($e)}, then {$effect}";

    // =========================================================================
    // Static ability condition phrases
    // =========================================================================

    // Condition prefix: if this card is in your void.
    if_this_card_in_void_prefix($base) = "if {this_card} is in your void, {$base}";
    // Condition prepended to base ability text.
    condition_prepended($condition, $base) = "{$condition} {$base}";
    // Condition appended to base ability text.
    condition_appended($base, $condition) = "{$base} {$condition}";

    // =========================================================================
    // Ability serializer structural phrases
    // =========================================================================

    // Capitalizes the first visible character of a string.
    capitalized_sentence($s) = "{@cap $s}";
    // Reclaim with dash-separated cost text.
    reclaim_with_cost($cost) = "{Reclaim} -- {$cost}";

    // =========================================================================
    // Ability assembly phrases
    // =========================================================================

    // Standard triggered ability: trigger text followed by effect text.
    triggered_ability($trig, $eff) = "{@cap $trig}{$eff}";
    // Triggered ability with prefix modifiers (once per turn, until end of turn).
    prefixed_triggered_ability($pfx, $trig, $eff) = "{$pfx}{$trig}{$eff}";
    // Keyword-triggered ability: trigger followed by capitalized effect.
    keyword_triggered_ability($trig, $eff) = "{$trig} {@cap $eff}";
    // Keyword-triggered ability with prefix modifiers.
    prefixed_keyword_triggered_ability($pfx, $trig, $eff) = "{$pfx}{$trig} {@cap $eff}";
    // Activated ability: capitalized costs, separator, capitalized effect.
    activated_ability($c, $eff) = "{$c}{cost_effect_separator}{@cap $eff}";
    // Activated ability with once-per-turn suffix on costs.
    activated_ability_once_per_turn($c, $eff) =
        "{$c}{once_per_turn_suffix}{cost_effect_separator}{@cap $eff}";
    // Fast activated ability: fast prefix, costs, separator, effect.
    fast_activated_ability($c, $eff) = "{fast_prefix}{$c}{cost_effect_separator}{@cap $eff}";
    // Fast activated ability with once-per-turn suffix.
    fast_activated_ability_once_per_turn($c, $eff) =
        "{fast_prefix}{$c}{once_per_turn_suffix}{cost_effect_separator}{@cap $eff}";
    // Activated ability cost separator for joining multiple costs.
    activated_cost_separator = ", ";

    // =========================================================================
    // Compound effect assembly phrases
    // =========================================================================

    // Single effect fragment with trailing period.
    effect_with_period($e) = "{$e}{period_suffix}";
    // Condition prefix joined with effect body by a space.
    condition_with_effect($cond, $body) = "{$cond} {$body}";
    // Optional prefix with effect body: "you may {body}".
    optional_effect_body($body) = "{you_may_prefix}{$body}";
    // Optional prefix with cost and effect body: "you may {cost} to {body}".
    optional_cost_effect_body($cost, $body) = "{you_may_prefix}{cost_to_connector($cost)}{$body}";
    // Mandatory cost with effect body: "{cost} to {body}".
    cost_effect_body($cost, $body) = "{cost_to_connector($cost)}{$body}";
    // Per-effect optional prefix: "you may {body}".
    per_effect_optional($body) = "{you_may_prefix}{$body}";
    // Per-effect cost prefix: "{cost} to {body}".
    per_effect_cost($cost, $body) = "{cost_to_connector($cost)}{$body}";
    // Per-effect condition prefix: "{cond} {body}".
    per_effect_condition($cond, $body) = "{$cond} {$body}";
    // Capitalized sentence with trailing period.
    capitalized_sentence_with_period($s) = "{@cap $s}{period_suffix}";

    // =========================================================================
    // Modal effect assembly phrases
    // =========================================================================

    // Single modal choice line with energy cost and effect text.
    modal_choice_line($energy_cost, $effect) =
        "{bullet} {$energy_cost}{cost_effect_separator}{@cap $effect}";
}
