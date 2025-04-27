use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::battle_animation::BattleAnimation;
use core_data::display_types::Milliseconds;
use display_data::command::{
    Command, DisplayDreamwellActivationCommand, DisplayJudgmentCommand, GameMessageType,
};

use crate::core::response_builder::ResponseBuilder;

pub fn render(builder: &mut ResponseBuilder, animation: &BattleAnimation, _snapshot: &BattleData) {
    match animation {
        BattleAnimation::StartTurn { player } => {
            builder.push(Command::DisplayGameMessage(if *player == builder.display_for_player() {
                GameMessageType::YourTurn
            } else {
                GameMessageType::EnemyTurn
            }));
        }
        BattleAnimation::Judgment { player, new_score } => {
            builder.push(Command::DisplayJudgment(DisplayJudgmentCommand {
                player: builder.to_display_player(*player),
                new_score: *new_score,
            }));
        }
        BattleAnimation::DreamwellActivation {
            player,
            dreamwell_card_id,
            new_energy,
            new_produced_energy,
        } => {
            builder.push(Command::DisplayDreamwellActivation(DisplayDreamwellActivationCommand {
                player: builder.to_display_player(*player),
                card_id: *dreamwell_card_id,
                new_energy: Some(*new_energy),
                new_produced_energy: Some(*new_produced_energy),
            }));
        }
        BattleAnimation::PlayCardFromHand { player, .. } => {
            if *player != builder.display_for_player() {
                builder.push(Command::Wait(Milliseconds::new(1500)));
            }
        }
    }
}
