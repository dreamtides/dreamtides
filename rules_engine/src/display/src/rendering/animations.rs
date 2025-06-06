use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, StackCardId};
use battle_state::core::effect_source::EffectSource;
use core_data::display_types::{AudioClipAddress, Milliseconds};
use display_data::command::{
    Command, DisplayDreamwellActivationCommand, DisplayEnemyMessageCommand, DisplayJudgmentCommand,
    DrawUserCardsCommand, GameMessageType, PlayAudioClipCommand,
};

use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::rendering::{apply_card_fx, battle_rendering, card_rendering, labels};

pub fn render(
    builder: &mut ResponseBuilder,
    source: EffectSource,
    animation: &BattleAnimation,
    snapshot: &BattleState,
    final_state: &BattleState,
) {
    match animation {
        BattleAnimation::StartTurn { player } => {
            push_snapshot(builder, snapshot);
            builder.push(Command::DisplayGameMessage(if *player == builder.display_for_player() {
                GameMessageType::YourTurn
            } else {
                GameMessageType::EnemyTurn
            }));
        }

        BattleAnimation::Judgment { player, new_score } => {
            push_snapshot(builder, snapshot);
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
            push_snapshot(builder, snapshot);
            builder.push(Command::DisplayDreamwellActivation(DisplayDreamwellActivationCommand {
                player: builder.to_display_player(*player),
                card_id: *dreamwell_card_id,
                new_energy: Some(*new_energy),
                new_produced_energy: Some(*new_produced_energy),
            }));
        }

        BattleAnimation::PlayCardFromHand { player, .. } => {
            if *player != builder.display_for_player() {
                builder.push(Command::PlayAudioClip(PlayAudioClipCommand {
                    sound: AudioClipAddress::new("Assets/ThirdParty/Cafofo/Magic Spells Sound Effects V2.0/General Spell/Magic Whoosh 4.wav"),
                    pause_duration: Milliseconds::new(0),
                }));
            }
        }

        BattleAnimation::PlayedCardFromHand { player, card_id } => {
            if *player != builder.display_for_player()
                && final_state.cards.stack_card(StackCardId(card_id.card_id())).is_none()
            {
                // If the played card is no longer on the stack, insert a pause
                // so it can be seen.
                push_snapshot(builder, snapshot);
                builder.push(Command::Wait(Milliseconds::new(2000)));
            }
        }

        BattleAnimation::DrawCards { player, cards } => {
            if *player == builder.display_for_player() && !cards.is_empty() {
                push_snapshot(builder, snapshot);
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

        BattleAnimation::MakeChoice { player, choice } => {
            if *player != builder.display_for_player() {
                push_snapshot(builder, snapshot);
                builder.push(Command::PlayAudioClip(PlayAudioClipCommand {
                    sound: AudioClipAddress::new("Assets/ThirdParty/Cafofo/Magic Spells Sound Effects V2.0/General Spell/Cast 12.wav"),
                    pause_duration: Milliseconds::new(0),
                }));
                builder.push(Command::DisplayEnemyMessage(DisplayEnemyMessageCommand {
                    message: labels::choice_label(*choice),
                    show_duration: Milliseconds::new(2000),
                }));
                builder.push(Command::Wait(Milliseconds::new(1000)));
            }
        }

        BattleAnimation::SelectStackCardTargets { .. } => {
            push_snapshot(builder, snapshot);
        }

        _ => {}
    }

    apply_card_fx::apply_effect(builder, source, animation, snapshot);
}

/// Appends a command to update the battle view to the current state.
pub fn push_snapshot(builder: &mut ResponseBuilder, snapshot: &BattleState) {
    builder.push_battle_view(battle_rendering::battle_view(builder, snapshot));
}
