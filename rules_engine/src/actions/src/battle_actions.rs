use action_data::battle_action::BattleAction;
use battle_data::battle::battle_data::BattleData;
use battle_mutations::play_card_mutations::play_card;
use battle_mutations::turn_step_mutations::end_turn;
use core_data::source::Source;
use core_data::types::PlayerName;
use tracing::instrument;

#[instrument(name = "actions_execute", level = "debug", skip(battle))]
pub fn execute(battle: &mut BattleData, player: PlayerName, action: BattleAction) {
    match action {
        BattleAction::PlayCard(card_id) => {
            play_card::execute(battle, player, Source::Game, card_id);
        }
        BattleAction::EndTurn => {
            end_turn::run(battle, Source::Game);
        }
        _ => {
            todo!("Implement {:?}", action);
        }
    }
}
