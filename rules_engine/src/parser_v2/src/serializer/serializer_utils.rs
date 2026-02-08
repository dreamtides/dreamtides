use ability_data::predicate::Operator;
use core_data::card_types::CardSubtype;
use rlf::Phrase;
use strings::strings;

/// Capitalizes the first letter of a string, or the first letter of a
/// leading action keyword in braces (e.g., "{kindle}" -> "{Kindle}").
///
/// Only capitalizes known action keywords (kindle, foresee, prevent,
/// dissolve, banish, materialize, reclaim, discover), not other directives
/// like {energy(e)}.
pub fn capitalize_first_letter(s: &str) -> String {
    if s.starts_with('{') {
        if let Some(end) = s.find('}') {
            let keyword = &s[1..end];
            if is_capitalizable_keyword(keyword) {
                let capitalized = title_case_keyword(keyword);
                return format!("{{{capitalized}}}{}", &s[end + 1..]);
            }
        }
    }
    capitalize_string(s)
}

/// Serializes an operator to its string representation.
///
/// Returns a string with a leading space for non-empty operators. This allows
/// format strings like `"cost {energy(e)}{}"` to work correctly for all
/// operators, producing `"cost {energy(e)}"` for Exactly and
/// `"cost {energy(e)} or less"` for OrLess.
pub fn serialize_operator<T>(operator: &Operator<T>) -> String {
    match operator {
        Operator::OrLess => strings::operator_or_less().to_string(),
        Operator::OrMore => strings::operator_or_more().to_string(),
        Operator::Exactly => String::new(),
        Operator::LowerBy(_) => strings::operator_lower().to_string(),
        Operator::HigherBy(_) => strings::operator_higher().to_string(),
    }
}

/// Converts a [CardSubtype] to its corresponding RLF phrase.
pub fn subtype_to_phrase(subtype: CardSubtype) -> Phrase {
    match subtype {
        CardSubtype::Agent => strings::agent(),
        CardSubtype::Ancient => strings::ancient(),
        CardSubtype::Avatar => strings::avatar(),
        CardSubtype::Child => strings::child(),
        CardSubtype::Detective => strings::detective(),
        CardSubtype::Enigma => strings::enigma(),
        CardSubtype::Explorer => strings::explorer(),
        CardSubtype::Guide => strings::guide(),
        CardSubtype::Hacker => strings::hacker(),
        CardSubtype::Mage => strings::mage(),
        CardSubtype::Monster => strings::monster(),
        CardSubtype::Musician => strings::musician(),
        CardSubtype::Outsider => strings::outsider(),
        CardSubtype::Renegade => strings::renegade(),
        CardSubtype::Robot => strings::robot(),
        CardSubtype::SpiritAnimal => strings::spirit_animal(),
        CardSubtype::Super => strings::super_(),
        CardSubtype::Survivor => strings::survivor(),
        CardSubtype::Synth => strings::synth(),
        CardSubtype::Tinkerer => strings::tinkerer(),
        CardSubtype::Trooper => strings::trooper(),
        CardSubtype::Visionary => strings::visionary(),
        CardSubtype::Visitor => strings::visitor(),
        CardSubtype::Warrior => strings::warrior(),
    }
}

fn capitalize_string(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Title-cases each underscore-separated word in a keyword, preserving
/// arguments after `(`.
fn title_case_keyword(s: &str) -> String {
    let (name, args) = match s.find('(') {
        Some(pos) => (&s[..pos], &s[pos..]),
        None => (s, ""),
    };
    let title_cased = name.split('_').map(capitalize_string).collect::<Vec<_>>().join("_");
    format!("{title_cased}{args}")
}

fn is_capitalizable_keyword(keyword: &str) -> bool {
    // Extract just the phrase name before any parenthesis for RLF function call
    // syntax (e.g., "kindle(k)" -> "kindle")
    let name = keyword.split('(').next().unwrap_or(keyword);
    matches!(
        name.to_lowercase().as_str(),
        "kindle"
            | "foresee"
            | "prevent"
            | "dissolve"
            | "banish"
            | "materialize"
            | "reclaim"
            | "reclaim_for_cost"
            | "reclaimforcost"
            | "discover"
    )
}
