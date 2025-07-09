use battle_queries::battle_card_queries::card_abilities;
use battle_queries::{battle_trace, panic_with};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CharacterId;
use battle_state::core::effect_source::EffectSource;
use core_data::identifiers::AbilityNumber;
use core_data::types::PlayerName;

use crate::effects::{apply_effect, pay_cost};

pub fn execute(
    battle: &mut BattleState,
    player: PlayerName,
    character_id: CharacterId,
    ability_number: AbilityNumber,
) {
    battle_trace!("Activating ability", battle, player, character_id, ability_number);

    let abilities = card_abilities::query(battle, character_id);
    let Some(ability_data) =
        abilities.activated_abilities.iter().find(|data| data.ability_number == ability_number)
    else {
        panic_with!("Activated ability not found", battle, character_id, ability_number);
    };

    let source = EffectSource::Activated { controller: player, character_id, ability_number };

    for cost in &ability_data.ability.costs {
        pay_cost::execute(battle, source, player, cost);
    }

    apply_effect::execute(battle, source, &ability_data.ability.effect, None);
}
