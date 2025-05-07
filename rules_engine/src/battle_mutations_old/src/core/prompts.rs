use assert_with::assert_that;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle::effect_source::EffectSource;
use battle_data_old::prompt_types::prompt_data::{PromptConfiguration, PromptData};

use crate::zone_mutations::move_card;

/// Sets the prompt for the given battle.
///
/// Panics if there is already an active prompt.
pub fn set(battle: &mut BattleData, prompt: PromptData) {
    assert_that!(battle.prompt.is_none(), battle, || "Cannot set a prompt while another is active");
    battle.prompt = Some(prompt);
}

/// Moves the 'source' card to the destination zone specified in
/// [PromptConfiguration], if one has been requested. Does nothing if 'source'
/// is not associated with a card, if the source card no longer exists, or if
/// the move is prevented.
pub fn move_to_zone_if_requested(
    battle: &mut BattleData,
    source: EffectSource,
    options: PromptConfiguration,
) {
    if let (Some(move_to_zone), Some(card_id)) = (options.move_source_to, source.card_id()) {
        move_card::to_destination_zone(battle, source, card_id, move_to_zone);
    }
}
