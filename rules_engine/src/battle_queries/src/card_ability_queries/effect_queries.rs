use ability_data::standard_effect::StandardEffect;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CharacterId;
use battle_state::battle_cards::card_set::CardSet;

use crate::battle_card_queries::card;
use crate::card_ability_queries::effect_predicates::CharacterTargetingFlags;

/// Returns the [CharacterTargetingFlags] for targeting a character with the
/// given effect.
pub fn character_targeting_flags(effect: &StandardEffect) -> CharacterTargetingFlags {
    if is_dissolve_effect(effect) {
        CharacterTargetingFlags { for_dissolve: true }
    } else {
        CharacterTargetingFlags::default()
    }
}

/// Returns the set of [CharacterId]s for characters which cannot currently be
/// dissolved.
pub fn prevent_dissolved_set(battle: &BattleState) -> CardSet<CharacterId> {
    let mut result = CardSet::default();
    if battle.ability_state.until_end_of_turn.prevent_dissolved.is_empty() {
        return result;
    }

    for character_id in battle.ability_state.until_end_of_turn.prevent_dissolved.iter() {
        result.insert(character_id.card_id);
    }

    result
}

/// Returns true if the given character cannot currently be dissolved.
pub fn should_prevent_dissolve(battle: &BattleState, id: CharacterId) -> bool {
    let object_id = card::get(battle, id).object_id;
    battle
        .ability_state
        .until_end_of_turn
        .prevent_dissolved
        .iter()
        .any(|card_object_id| card_object_id.object_id == object_id)
}

/// Returns true if the given effect is a dissolve effect.
pub fn is_dissolve_effect(effect: &StandardEffect) -> bool {
    matches!(
        effect,
        StandardEffect::DissolveCharacter { .. }
            | StandardEffect::DissolveCharactersCount { .. }
            | StandardEffect::DissolveCharactersQuantity { .. }
    )
}
