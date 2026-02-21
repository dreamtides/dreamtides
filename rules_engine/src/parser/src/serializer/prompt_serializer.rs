use ability_data::standard_effect::StandardEffect;
use strings::strings;

use crate::serializer::effect_serializer;

/// Returns auto-generated prompt text for a standard effect.
///
/// Reuses the existing rules-text effect serializer and formats it as a
/// capitalized sentence with a trailing period.
pub fn serialize_prompt(effect: &StandardEffect) -> String {
    strings::capitalized_sentence_with_period(effect_serializer::serialize_standard_effect(effect))
        .to_string()
}
