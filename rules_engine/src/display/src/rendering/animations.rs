use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, StackCardId};
use core_data::display_types::Milliseconds;
use display_data::command::{
    Command, DisplayDreamwellActivationCommand, DisplayJudgmentCommand, GameMessageType,
};

use crate::core::response_builder::ResponseBuilder;
use crate::rendering::target_projectile_effects;

pub fn render(
    builder: &mut ResponseBuilder,
    animation: &BattleAnimation,
    snapshot: &BattleState,
    final_state: &BattleState,
) {
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
        BattleAnimation::PlayCardFromHand { player, card_id } => {
            if *player != builder.display_for_player()
                && final_state.cards.stack_card(StackCardId(card_id.card_id())).is_none()
            {
                // If the played card is no longer on the stack, insert a pause
                // so it can be seen.
                builder.push(Command::Wait(Milliseconds::new(2000)));
            }
        }
        BattleAnimation::SelectStackCardTargets { .. } => {}
        BattleAnimation::ApplyEffectToTargets { source, targets } => {
            eprintln!(">>>>>>> Applying effect to targets: {:?}", targets);
            builder.extend_optional(target_projectile_effects::effect(snapshot, *source, targets));
        }
    }
}
