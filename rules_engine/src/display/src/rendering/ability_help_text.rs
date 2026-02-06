use battle_queries::battle_card_queries::card;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use strings::strings;

/// Returns help text descriptions for keywords used in the given card's rules
/// text.
pub fn help_texts(battle: &BattleState, card_id: CardId) -> Vec<String> {
    let rules = &card::get_definition(battle, card_id).displayed_rules_text.to_ascii_lowercase();

    let mut out = Vec::new();

    if rules.contains("{dissolve}") {
        out.push(strings::help_text_dissolve().to_string());
    }

    if rules.contains("{prevent}") {
        out.push(strings::help_text_prevent().to_string());
    }

    if let Some(n) = capture_number(rules, "{-foresee(n:") {
        if n == 1 {
            out.push(strings::help_text_foresee_1().to_string());
        } else {
            out.push(strings::help_text_foresee_n(n).to_string());
        }
    }

    if rules.contains("{anchored}") {
        out.push(strings::help_text_anchored().to_string());
    }

    if rules.contains("{reclaim}") {
        out.push(strings::help_text_reclaim_without_cost().to_string());
    }

    if let Some(e) = capture_number(rules, "{-reclaim-cost(e:") {
        out.push(strings::help_text_reclaim_with_cost(e).to_string());
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
