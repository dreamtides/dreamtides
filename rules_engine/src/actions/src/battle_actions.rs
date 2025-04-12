use action_data::battle_action::BattleAction;
use battle_data::battle::battle_data::BattleData;
use battle_mutations::play_cards::play_card;
use core_data::source::Source;
use core_data::types::PlayerName;
use tracing::instrument;

#[instrument(name = "actions_execute", level = "debug", skip(battle))]
pub fn execute(battle: &mut BattleData, player: PlayerName, action: BattleAction) {
    match action {
        BattleAction::PlayCard(card_id) => {
            play_card::execute(battle, player, Source::Game, card_id);
        }
        _ => {}
    }
}
