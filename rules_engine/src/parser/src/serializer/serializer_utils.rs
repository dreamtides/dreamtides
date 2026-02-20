use ability_data::predicate::Operator;
use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;
use rlf::Phrase;
use strings::strings;

/// Serializes an operator to its phrase representation.
///
/// Returns a phrase with a leading space for non-empty operators. This allows
/// format strings like `"cost {energy(e)}{}"` to work correctly for all
/// operators, producing `"cost {energy(e)}"` for Exactly and
/// `"cost {energy(e)} or less"` for OrLess.
pub fn serialize_operator<T>(operator: &Operator<T>) -> Phrase {
    match operator {
        Operator::OrLess => strings::operator_or_less(),
        Operator::OrMore => strings::operator_or_more(),
        Operator::Exactly => Phrase::empty(),
        Operator::LowerBy(_) => strings::operator_lower(),
        Operator::HigherBy(_) => strings::operator_higher(),
    }
}

/// Converts a [FigmentType] to its corresponding RLF phrase.
pub fn figment_to_phrase(figment: FigmentType) -> Phrase {
    match figment {
        FigmentType::Celestial => strings::celestial(),
        FigmentType::Halcyon => strings::halcyon(),
        FigmentType::Radiant => strings::radiant(),
        FigmentType::Shadow => strings::shadow(),
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
