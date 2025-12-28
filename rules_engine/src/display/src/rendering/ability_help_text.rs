use battle_queries::battle_card_queries::card;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use fluent::fluent_args;
use tabula_ids::string_id;

use crate::core::response_builder::ResponseBuilder;

pub fn help_texts(builder: &ResponseBuilder, battle: &BattleState, card_id: CardId) -> Vec<String> {
    let rules = &card::get_definition(battle, card_id).displayed_rules_text.to_ascii_lowercase();

    let mut out = Vec::new();

    if rules.contains("{dissolve}") {
        out.push(builder.string(string_id::HELP_TEXT_DISSOLVE));
    }

    if rules.contains("{prevent}") {
        out.push(builder.string(string_id::HELP_TEXT_PREVENT));
    }

    if let Some(n) = capture_number(rules, "{-foresee(n:") {
        if n == 1 {
            out.push(builder.string(string_id::HELP_TEXT_FORESEE_1));
        } else {
            out.push(
                builder.string_with_args(string_id::HELP_TEXT_FORESEE_N, fluent_args!["n" => n]),
            );
        }
    }

    if rules.contains("{anchored}") {
        out.push(builder.string(string_id::HELP_TEXT_ANCHORED));
    }

    if rules.contains("{reclaim}") {
        out.push(builder.string(string_id::HELP_TEXT_RECLAIM_WITHOUT_COST));
    }

    if let Some(e) = capture_number(rules, "{-reclaim-cost(e:") {
        out.push(
            builder
                .string_with_args(string_id::HELP_TEXT_RECLAIM_WITH_COST, fluent_args!["e" => e]),
        );
    }

    out
}

fn capture_number(text: &str, prefix: &str) -> Option<u32> {
    if let Some(start) = text.find(prefix) {
        let rest = &text[start + prefix.len()..];
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if digits.is_empty() { None } else { digits.parse().ok() }
    } else {
        None
    }
}
