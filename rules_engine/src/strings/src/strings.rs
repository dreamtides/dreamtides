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
    // Reclaim keyword.
    reclaim = "<color=#AA00FF>reclaim</color>";
    // Materialize keyword.
    materialize = "<color=#AA00FF>materialize</color>";
    // Prevent keyword.
    prevent = "<color=#AA00FF>prevent</color>";
    // Kindle keyword with spark amount.
    kindle($k) = "<color=#AA00FF>kindle</color> {$k}";
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
    ally = :an { one: "ally", other: "allies" };
    // Ally count with article (e.g., "an ally" or "2 allies").
    count_allies($n) = :match($n) {
        1: "an ally",
        *other: "{$n} allies",
    };
    // Allied character count with subtype (e.g., "an allied warrior").
    count_allied_subtype($n, $s) = :match($n) {
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
    // Figment tokens (plural) with gold formatting.
    figments_plural($f) = "<color=#F57F17><b><u>{$f} Figments</u></color></b>";
    // N figments with article for singular.
    n_figments($n, $f) = :match($n) {
        1: "a {figment($f)}",
        *other: "{text_number($n)} {figments_plural($f)}",
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
}
