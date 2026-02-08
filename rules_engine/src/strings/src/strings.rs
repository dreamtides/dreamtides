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

    // Trigger prefix formatting (for runtime eval_str use).
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

    // =========================================================================
    // Keywords
    // =========================================================================

    // Keyword formatting in purple (for runtime eval_str use).
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
    card = :a{ one: "card", other: "cards" };
    // Card count with article (e.g., "a card" or "2 cards").
    cards($n) = :match($n) {
        1: "a card",
        *other: "{$n} cards",
    };
    // Top N cards of deck (e.g., "top card" or "top 3 cards").
    top_n_cards($n) = :match($n) {
        1: "top card",
        *other: "top {$n} {card:$n}",
    };

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
    ally = :an{ one: "ally", other: "allies" };
    // Ally count with article (e.g., "an ally" or "2 allies").
    count_allies($n) = :match($n) {
        1: "an ally",
        *other: "{$n} allies",
    };
    // Allied character count with subtype (e.g., "an allied warrior").
    count_allied_subtype($n, $s) = :from($s) :match($n) {
        1: "an allied {subtype($s)}",
        *other: "{$n} allied {subtype($s):other}",
    };

    // =========================================================================
    // Figment types
    // =========================================================================

    // Celestial figment type.
    celestial = :a "Celestial";
    // Halcyon figment type.
    halcyon = :a "Halcyon";
    // Radiant figment type.
    radiant = :a "Radiant";
    // Shadow figment type.
    shadow = :a "Shadow";

    // =========================================================================
    // Figment tokens
    // =========================================================================

    // Figment token (singular) with gold formatting, inheriting article metadata.
    figment($f) = :from($f) "<color=#F57F17><b><u>{$f} Figment</u></color></b>";
    // Figment tokens (plural) with gold formatting, inheriting article metadata.
    figments_plural($f) = :from($f) "<color=#F57F17><b><u>{$f} Figments</u></color></b>";
    // N figments with article for singular.
    n_figments($n, $f) = :match($n) {
        1: "a {figment($f)}",
        *other: "{text_number($n)} {figments_plural($f)}",
    };

    // =========================================================================
    // Character subtypes
    // =========================================================================

    // Agent subtype.
    agent = :an{ one: "Agent", other: "Agents" };
    // Ancient subtype.
    ancient = :an{ one: "Ancient", other: "Ancients" };
    // Avatar subtype.
    avatar = :an{ one: "Avatar", other: "Avatars" };
    // Child subtype.
    child = :a{ one: "Child", other: "Children" };
    // Detective subtype.
    detective = :a{ one: "Detective", other: "Detectives" };
    // Enigma subtype.
    enigma = :an{ one: "Enigma", other: "Enigmas" };
    // Explorer subtype.
    explorer = :an{ one: "Explorer", other: "Explorers" };
    // Guide subtype.
    guide = :a{ one: "Guide", other: "Guides" };
    // Hacker subtype.
    hacker = :a{ one: "Hacker", other: "Hackers" };
    // Mage subtype.
    mage = :a{ one: "Mage", other: "Mages" };
    // Monster subtype.
    monster = :a{ one: "Monster", other: "Monsters" };
    // Musician subtype.
    musician = :a{ one: "Musician", other: "Musicians" };
    // Outsider subtype.
    outsider = :an{ one: "Outsider", other: "Outsiders" };
    // Renegade subtype.
    renegade = :a{ one: "Renegade", other: "Renegades" };
    // Robot subtype.
    robot = :a{ one: "Robot", other: "Robots" };
    // Spirit Animal subtype.
    spirit_animal = :a{ one: "Spirit Animal", other: "Spirit Animals" };
    // Super subtype.
    super_ = :a{ one: "Super", other: "Supers" };
    // Survivor subtype.
    survivor = :a{ one: "Survivor", other: "Survivors" };
    // Synth subtype.
    synth = :a{ one: "Synth", other: "Synths" };
    // Tinkerer subtype.
    tinkerer = :a{ one: "Tinkerer", other: "Tinkerers" };
    // Trooper subtype.
    trooper = :a{ one: "Trooper", other: "Troopers" };
    // Visionary subtype.
    visionary = :a{ one: "Visionary", other: "Visionaries" };
    // Visitor subtype.
    visitor = :a{ one: "Visitor", other: "Visitors" };
    // Warrior subtype.
    warrior = :a{ one: "Warrior", other: "Warriors" };

    // Subtype display with green bold formatting, inheriting article metadata.
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
    multiply_by($n) = :match($n) {
        2: "Double",
        3: "Triple",
        *other: "Multiply by {$n}",
    };

    // =========================================================================
    // Copy counts
    // =========================================================================

    // Copy count with article (e.g., "a copy" or "two copies").
    copies($n) = :match($n) {
        1: "a copy",
        *other: "{text_number($n)} copies",
    };

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
    up_to_n_events($n) = :match($n) {
        1: "an event",
        *other: "up to {$n} events",
    };

    // =========================================================================
    // Optional ally targeting
    // =========================================================================

    // Up to N allies (e.g., "an ally" or "up to 3 allies").
    up_to_n_allies($n) = :match($n) {
        1: "an {ally}",
        *other: "up to {$n} {ally:other}",
    };

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
    character_limit_exceeded_warning_message =
        "Character limit exceeded: A character will be abandoned, with its spark permanently added to your total.";
    // Warning about exceeding both limits.
    combined_limit_warning_message =
        "Character limit exceeded: A character will be abandoned. Cards drawn in excess of 10 become {energy_symbol} instead.";

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
    help_text_dissolve =
        "{@cap dissolve}: Send a character to the void";
    // Help text for prevent ability.
    help_text_prevent =
        "{@cap prevent}: Send a card to the void in response to it being played";
    // Help text for foresee 1 ability.
    help_text_foresee_1 =
        "<color=#AA00FF>Foresee</color> 1: Look at the top card of your deck. You may put it into your void.";
    // Help text for foresee N ability.
    help_text_foresee_n($n) =
        "<color=#AA00FF>Foresee</color> {$n}: Look at the top {$n} cards of your deck. You may put them into your void or put them back in any order.";
    // Help text for anchored status.
    help_text_anchored =
        "<color=#AA00FF><b>Anchored</b></color>: Cannot be dissolved.";
    // Help text for reclaim without cost.
    help_text_reclaim_without_cost =
        "{@cap reclaim}: You may play a card from your void, then banish it when it leaves play.";
    // Help text for reclaim with energy cost.
    help_text_reclaim_with_cost($e) =
        "{@cap reclaim} {energy($e)}: You may play this card from your void for {energy($e)}, then banish it.";

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
    // Cost serializer phrases (Category A)
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
    // Abandon any number of a target type.
    abandon_any_number_of($target) = :from($target) "abandon any number of {$target}";
    // Abandon a specific target.
    abandon_target($target) = :from($target) "abandon {$target}";
    // Return a target to hand.
    return_target_to_hand($target) = :from($target) "return {$target} to hand";
    // Return a count of targets to hand.
    return_count_to_hand($n, $target) = :from($target) "return {$n} {$target} to hand";
    // Return all but one target to hand.
    return_all_but_one_to_hand($target) = :from($target) "return all but one {$target} to hand";
    // Return all targets to hand.
    return_all_to_hand($target) = :from($target) "return all {$target} to hand";
    // Return any number of targets to hand.
    return_any_number_to_hand($target) = :from($target) "return any number of {$target} to hand";
    // Return up to N targets to hand.
    return_up_to_to_hand($n, $target) = :from($target) "return up to {$n} {$target} to hand";
    // Return each other target to hand.
    return_each_other_to_hand($target) = :from($target) "return each other {$target} to hand";
    // Return N or more targets to hand.
    return_or_more_to_hand($n, $target) = :from($target) "return {$n} or more {$target} to hand";

    // =========================================================================
    // Cost serializer phrases (Category B) - Phase 2: requires Phrase composition
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
    banish_your_void_cost = "{Banish} your void";
    // Banish another card in your void.
    banish_another_in_void = "{Banish} another card in your void";
    // Banish a count of cards from your void.
    banish_cards_from_void($c) = "{Banish} {cards($c)} from your void";
    // Banish a count of cards from the opponent's void.
    banish_cards_from_enemy_void($c) = "{Banish} {cards($c)} from the opponent's void";
    // Banish your void with a minimum card count.
    banish_void_min_count($n) = "{Banish} your void with {count($n)} or more cards";
    // Banish a target from hand.
    banish_from_hand_cost($target) = :from($target) "{Banish} {$target} from hand";

    // =========================================================================
    // Trigger serializer phrases (Category A)
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
    when_you_play_from_hand_trigger($target) = :from($target) "when you play {$target} from your hand, ";
    // Play a target in a turn trigger.
    when_you_play_in_turn_trigger($target) = :from($target) "when you play {$target} in a turn, ";
    // Play a target during enemy turn trigger.
    when_you_play_during_enemy_turn_trigger($target) = :from($target) "when you play {$target} during the opponent's turn, ";
    // Discard a target trigger.
    when_you_discard_trigger($target) = :from($target) "when you discard {$target}, ";
    // Target leaves play trigger.
    when_leaves_play_trigger($target) = :from($target) "when {$target} leaves play, ";
    // Abandon a target trigger.
    when_you_abandon_trigger($target) = :from($target) "when you abandon {$target}, ";
    // Target put into void trigger.
    when_put_into_void_trigger($target) = :from($target) "when {$target} is put into your void, ";

    // =========================================================================
    // Trigger serializer phrases (Category B)
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
    // Materialize Nth target in a turn trigger.
    when_you_materialize_nth_in_turn_trigger($n, $target) = :from($target) "when you {materialize} {text_number($n)} {$target} in a turn, ";

    // =========================================================================
    // Condition serializer phrases (Category A)
    // =========================================================================

    // Condition for a character dissolving this turn.
    if_character_dissolved_this_turn = "if a character dissolved this turn";
    // Condition for this card being in your void.
    if_card_in_your_void = "if this card is in your void,";
    // Condition for having discarded a target this turn.
    if_discarded_this_turn($target) = :from($target) "if you have discarded {$target} this turn";
    // Condition wrapper for a predicate count.
    with_predicate_condition($pred) = "with {$pred},";

    // =========================================================================
    // Condition serializer phrases (Category B)
    // =========================================================================

    // Condition for allies sharing a character type.
    with_allies_sharing_type($a) = "with {count_allies($a)} that share a character type,";
    // Condition for drawing a count of cards this turn.
    if_drawn_count_this_turn($n) = "if you have drawn {count($n)} or more cards this turn";
    // Condition for having a count of cards in your void.
    while_void_count($n) = "while you have {count($n)} or more cards in your void,";
    // Condition for having an allied subtype.
    with_allied_subtype($t) = "with an allied {subtype($t)},";
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
    // Effect serializer phrases (Category B)
    // =========================================================================

    // Draw cards effect.
    draw_cards_effect($c) = "draw {cards($c)}.";
    // Discard cards effect.
    discard_cards_effect($d) = "discard {cards($d)}.";
    // Gain energy effect.
    gain_energy_effect($e) = "gain {energy($e)}.";
    // Gain points effect.
    gain_points_effect($p) = "gain {points($p)}.";
    // Lose points effect.
    lose_points_effect($p) = "you lose {points($p)}.";
    // Opponent gains points effect.
    opponent_gains_points_effect($p) = "the opponent gains {points($p)}.";
    // Opponent loses points effect.
    opponent_loses_points_effect($p) = "the opponent loses {points($p)}.";
    // Foresee effect.
    foresee_effect($f) = "{foresee($f)}.";
    // Kindle effect.
    kindle_effect($k) = "{kindle($k)}.";
    // Each player discards effect.
    each_player_discards_effect($d) = "each player discards {cards($d)}.";
    // Prevent that card effect.
    prevent_that_card_effect = "{prevent} that card.";
    // Then materialize it effect.
    then_materialize_it_effect = "then {materialize} it.";
    // Gain twice energy instead effect.
    gain_twice_energy_instead_effect = "gain twice that much {energy_symbol} instead.";
    // Gain energy equal to that character's cost effect.
    gain_energy_equal_to_that_cost_effect = "gain {energy_symbol} equal to that character's cost.";
    // Gain energy equal to this character's cost effect.
    gain_energy_equal_to_this_cost_effect = "gain {energy_symbol} equal to this character's cost.";
    // Put top cards of deck into void effect.
    put_deck_into_void_effect($v) = "put the {top_n_cards($v)} of your deck into your void.";
    // Banish cards from enemy void effect.
    banish_cards_from_enemy_void_effect($c) = "{banish} {cards($c)} from the opponent's void.";
    // Banish enemy void effect.
    banish_enemy_void_effect = "{banish} the opponent's void.";
    // Judgment phase at end of turn effect.
    judgment_phase_at_end_of_turn_effect = "at the end of this turn, trigger an additional {judgment_phase_name} phase.";
    // Multiply energy effect.
    multiply_energy_effect($n) = "{multiply_by($n)} the amount of {energy_symbol} you have.";
    // Spend all energy dissolve effect.
    spend_all_energy_dissolve_effect = "spend all your {energy_symbol}. {dissolve} an enemy with cost less than or equal to the amount spent.";
    // Spend all energy draw discard effect.
    spend_all_energy_draw_discard_effect = "spend all your {energy_symbol}. Draw cards equal to the amount spent, then discard that many cards.";
    // Each player shuffles and draws effect.
    each_player_shuffles_and_draws_effect($c) = "each player shuffles their hand and void into their deck and then draws {cards($c)}.";
    // Return up to events from void effect.
    return_up_to_events_from_void_effect($n) = "return {up_to_n_events($n)} from your void to your hand.";
    // Fast prefix for activated abilities.
    fast_prefix = "{Fast} -- ";

    // =========================================================================
    // Effect serializer phrases (Category A)
    // =========================================================================

    // Opponent gains points equal to spark effect.
    opponent_gains_points_equal_spark = "the opponent gains points equal to its spark.";
    // Take extra turn effect.
    take_extra_turn_effect = "take an extra turn after this one.";
    // You win the game effect.
    you_win_the_game_effect = "you win the game.";
    // No effect.
    no_effect = "";

    // =========================================================================
    // Structural Phrases — Category A (Final)
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
    // Terminal punctuation for ability text.
    period_suffix = ".";

    // =========================================================================
    // Predicate base entity nouns
    // =========================================================================

    // Character noun with article metadata.
    character = :a{ one: "character", other: "characters" };
    // Event noun with article metadata.
    event = :an{ one: "event", other: "events" };
    // Enemy noun with article metadata.
    enemy = :an{ one: "enemy", other: "enemies" };

    // This card noun with article metadata.
    this_card = :a{ one: "this card", other: "these cards" };
    // This character noun with article metadata.
    this_character = :a{ one: "this character", other: "these characters" };

    // =========================================================================
    // Ownership-qualified predicate nouns
    // =========================================================================

    // Your card noun with article metadata.
    your_card = :a{ one: "your card", other: "your cards" };
    // Your character noun with article metadata.
    your_character = :a{ one: "your character", other: "your characters" };
    // Your event noun with article metadata.
    your_event = :a{ one: "your event", other: "your events" };
    // Enemy card noun with article metadata.
    enemy_card = :an{ one: "enemy card", other: "enemy cards" };
    // Enemy character noun with article metadata.
    enemy_character = :an{ one: "enemy character", other: "enemy characters" };
    // Enemy event noun with article metadata.
    enemy_event = :an{ one: "enemy event", other: "enemy events" };
    // Allied character noun with article metadata.
    allied_character = :an{ one: "allied character", other: "allied characters" };
    // Allied event noun with article metadata.
    allied_event = :an{ one: "allied event", other: "allied events" };
    // Other character noun with article metadata.
    other_character = :an{ one: "other character", other: "other characters" };

    // =========================================================================
    // Compound predicate nouns with subtype propagation
    // =========================================================================

    // Allied subtype noun inheriting article metadata from subtype.
    allied_subtype($t) = :from($t) "allied {$t}";
    // Enemy subtype noun inheriting article metadata from subtype.
    enemy_subtype($t) = :from($t) "enemy {$t}";
    // Your subtype noun inheriting article metadata from subtype.
    your_subtype($t) = :from($t) "your {$t}";
    // Other subtype noun inheriting article metadata from subtype.
    other_subtype($t) = :from($t) "other {$t}";
    // Subtype in your void inheriting article metadata from subtype.
    subtype_in_your_void($t) = :from($t) "{$t} in your void";

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
}
