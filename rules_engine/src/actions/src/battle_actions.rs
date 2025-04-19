use action_data::battle_action::BattleAction;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_mutations::play_cards::{play_card, resolve_cards, select_card};
use battle_mutations::turn_step_mutations::end_turn;
use core_data::types::PlayerName;
use tracing::instrument;

#[instrument(name = "actions_execute", level = "debug", skip(battle))]
pub fn execute(battle: &mut BattleData, player: PlayerName, action: BattleAction) {
    let source = EffectSource::Game { controller: player };
    match action {
        BattleAction::PlayCardFromHand(card_id) => {
            play_card::execute(battle, player, source, card_id);
        }
        BattleAction::ResolveStack => {
            resolve_cards::resolve_stack(battle, source);
        }
        BattleAction::EndTurn => {
            end_turn::run(battle, source);
        }
        BattleAction::SelectCharacter(character_id) => {
            select_card::select_character_for_prompt(battle, source, character_id);
        }
        BattleAction::SelectStackCard(stack_card_id) => {
            select_card::select_stack_card_for_prompt(battle, source, stack_card_id);
        }
        _ => {
            todo!("Implement {:?}", action);
        }
    }
}
