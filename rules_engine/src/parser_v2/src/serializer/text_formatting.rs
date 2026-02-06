use ability_data::predicate::CardPredicate;

use crate::serializer::serializer_utils;

pub struct FormattedText {
    base: String,
    plural: String,
    starts_with_vowel_sound: bool,
}

pub fn card_predicate_base_text(predicate: &CardPredicate) -> FormattedText {
    match predicate {
        CardPredicate::Card => FormattedText::new("card"),
        CardPredicate::Character => FormattedText::new("character"),
        CardPredicate::Event => FormattedText::new("event"),
        CardPredicate::CharacterType(_) => {
            FormattedText::with_plural("{subtype(subtype)}", "{subtype(subtype):other}")
        }
        CardPredicate::NotCharacterType(_) => {
            FormattedText::new("character that is not {@a subtype(subtype)}")
        }
        CardPredicate::Fast { target } => card_predicate_base_text(target),
        CardPredicate::CardWithCost { target, .. } => card_predicate_base_text(target),
        CardPredicate::CharacterWithSpark(..) => FormattedText::new("character"),
        CardPredicate::CharacterWithMaterializedAbility => FormattedText::new("character"),
        CardPredicate::CharacterWithMultiActivatedAbility => FormattedText::new("character"),
        CardPredicate::CouldDissolve { .. } => FormattedText::new("event"),
        _ => FormattedText::new("character"),
    }
}

impl FormattedText {
    pub fn new(base: &str) -> Self {
        let starts_with_vowel_sound = base.starts_with(['a', 'e', 'i', 'o', 'u']);
        Self { base: base.to_string(), plural: format!("{}s", base), starts_with_vowel_sound }
    }

    /// Creates a new FormattedText with explicit non-vowel article treatment.
    ///
    /// Use this when the text doesn't start with a vowel sound
    /// (e.g., "non-warrior enemy" uses "a" not "an").
    pub fn new_non_vowel(base: &str) -> Self {
        Self {
            base: base.to_string(),
            plural: format!("{}s", base),
            starts_with_vowel_sound: false,
        }
    }

    pub fn with_plural(base: &str, plural: &str) -> Self {
        let starts_with_vowel_sound = base.starts_with(['a', 'e', 'i', 'o', 'u']);
        Self { base: base.to_string(), plural: plural.to_string(), starts_with_vowel_sound }
    }

    pub fn with_article(&self) -> String {
        if self.starts_with_vowel_sound {
            format!("an {}", self.base)
        } else {
            format!("a {}", self.base)
        }
    }

    pub fn without_article(&self) -> String {
        self.base.clone()
    }

    pub fn plural(&self) -> String {
        self.plural.clone()
    }

    pub fn capitalized(&self) -> String {
        serializer_utils::capitalize_first_letter(&self.base)
    }

    pub fn capitalized_with_article(&self) -> String {
        serializer_utils::capitalize_first_letter(&self.with_article())
    }
}
