use ability_data::predicate::Operator;
use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;
use rlf::Phrase;
use strings::strings;

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

/// Returns the RLF phrase name for a [CardSubtype].
///
/// This is the name used in template strings to reference the subtype
/// phrase (e.g., "agent", "survivor", "spirit_animal").
pub fn subtype_phrase_name(subtype: CardSubtype) -> &'static str {
    match subtype {
        CardSubtype::Agent => "agent",
        CardSubtype::Ancient => "ancient",
        CardSubtype::Avatar => "avatar",
        CardSubtype::Child => "child",
        CardSubtype::Detective => "detective",
        CardSubtype::Enigma => "enigma",
        CardSubtype::Explorer => "explorer",
        CardSubtype::Guide => "guide",
        CardSubtype::Hacker => "hacker",
        CardSubtype::Mage => "mage",
        CardSubtype::Monster => "monster",
        CardSubtype::Musician => "musician",
        CardSubtype::Outsider => "outsider",
        CardSubtype::Renegade => "renegade",
        CardSubtype::Robot => "robot",
        CardSubtype::SpiritAnimal => "spirit_animal",
        CardSubtype::Super => "super_",
        CardSubtype::Survivor => "survivor",
        CardSubtype::Synth => "synth",
        CardSubtype::Tinkerer => "tinkerer",
        CardSubtype::Trooper => "trooper",
        CardSubtype::Visionary => "visionary",
        CardSubtype::Visitor => "visitor",
        CardSubtype::Warrior => "warrior",
    }
}

/// Returns the RLF phrase name for a [FigmentType].
pub fn figment_phrase_name(figment: FigmentType) -> &'static str {
    match figment {
        FigmentType::Celestial => "celestial",
        FigmentType::Halcyon => "halcyon",
        FigmentType::Radiant => "radiant",
        FigmentType::Shadow => "shadow",
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
