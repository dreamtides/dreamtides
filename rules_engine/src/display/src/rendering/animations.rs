use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, StackCardId};
use core_data::display_types::Milliseconds;
use display_data::command::{
    Command, DisplayDreamwellActivationCommand, DisplayEnemyMessageCommand, DisplayJudgmentCommand,
    DrawUserCardsCommand, GameMessageType,
};

use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::rendering::{apply_card_fx, card_rendering, labels};

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
        BattleAnimation::DrawCards { player, cards } => {
            if *player == builder.display_for_player() && !cards.is_empty() {
                let card_views = cards
                    .iter()
                    .map(|&card_id| {
                        card_rendering::card_view(
                            builder,
                            &CardViewContext::Battle(
                                final_state,
                                final_state.cards.card(card_id.card_id()).name,
                                card_id.card_id(),
                            ),
                        )
                    })
                    .collect();

                builder.push(Command::DrawUserCards(DrawUserCardsCommand {
                    cards: card_views,
                    stagger_interval: Milliseconds::new(500),
                    pause_duration: Milliseconds::new(300),
                }));
            }
        }
        BattleAnimation::SelectStackCardTargets { .. } => {}
        BattleAnimation::ApplyEffect { controller, source, targets, effect } => {
            apply_card_fx::apply(builder, snapshot, *controller, *source, effect, targets);
        }
        BattleAnimation::MakeChoice { player, choice } => {
            if *player != builder.display_for_player() {
                builder.push(Command::DisplayEnemyMessage(DisplayEnemyMessageCommand {
                    message: labels::choice_label(*choice),
                    show_duration: Milliseconds::new(2000),
                }));
            }
        }
    }
}
