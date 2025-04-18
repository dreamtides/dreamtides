use action_data::battle_action::BattleAction;
use battle_data::battle::battle_data::BattleData;
use battle_mutations::play_cards::{play_card, resolve_cards, select_card};
use battle_mutations::turn_step_mutations::end_turn;
use core_data::effect_source::EffectSource;
use core_data::types::PlayerName;
use tracing::instrument;

#[instrument(name = "actions_execute", level = "debug", skip(battle))]
pub fn execute(battle: &mut BattleData, player: PlayerName, action: BattleAction) {
    match action {
        BattleAction::PlayCardFromHand(card_id) => {
            play_card::execute(battle, player, EffectSource::Game, card_id);
        }
        BattleAction::ResolveStack => {
            resolve_cards::resolve_stack(battle, EffectSource::Game);
        }
        BattleAction::EndTurn => {
            end_turn::run(battle, EffectSource::Game);
        }
        BattleAction::SelectCharacter(character_id) => {
            select_card::select_character_for_prompt(battle, EffectSource::Game, character_id);
        }
        BattleAction::SelectStackCard(stack_card_id) => {
            select_card::select_stack_card_for_prompt(battle, EffectSource::Game, stack_card_id);
        }
        _ => {
            todo!("Implement {:?}", action);
        }
    }
}
