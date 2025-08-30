use asset_paths::hovl;
use battle_queries::battle_card_queries::card;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, StackCardId};
use battle_state::core::effect_source::EffectSource;
use core_data::display_types::{AudioClipAddress, Milliseconds, ProjectileAddress};
use display_data::battle_view::DisplayPlayer;
use display_data::command::{
    Command, DisplayDreamwellActivationCommand, DisplayEffectCommand, DisplayEnemyMessageCommand,
    DisplayJudgmentCommand, DrawUserCardsCommand, FireProjectileCommand, GameMessageType,
    GameObjectId, PlayAudioClipCommand,
};
use display_data::object_position::Position;
use masonry::flex_style::FlexVector3;

use crate::core::adapter;
use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::rendering::{
    apply_card_fx, battle_rendering, card_rendering, labels, modal_effect_prompt_rendering,
};

pub fn render(
    builder: &mut ResponseBuilder,
    source: EffectSource,
    animation: &BattleAnimation,
    snapshot: &BattleState,
    final_state: &BattleState,
) {
    apply_card_fx::apply_effect(builder, source, animation, snapshot);

    match animation {
        BattleAnimation::ActivatedAbility { player, .. } => {
            if *player != builder.display_for_player() {
                // Pause so the opponent can see the ability being activated.
                push_snapshot(builder, snapshot);
                builder.push(Command::Wait(Milliseconds::new(1000)));
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
                                card::get_base_card_id(snapshot, card_id),
                                card_id.card_id(),
                            ),
                        )
                    })
                    .collect();

                builder.push(Command::DrawUserCards(DrawUserCardsCommand {
                    cards: card_views,
                    stagger_interval: Milliseconds::new(500),
                    pause_duration: Milliseconds::new(300),
                    destination: Position::InHand(DisplayPlayer::User),
                }));
            }
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
                card_id: adapter::client_card_id(*dreamwell_card_id),
                new_energy: Some(*new_energy),
                new_produced_energy: Some(*new_produced_energy),
            }));
        }

        BattleAnimation::GainSpark { character_id, .. } => {
            push_snapshot(builder, snapshot);
            builder.push(Command::DisplayEffect(DisplayEffectCommand {
                target: GameObjectId::CardId(adapter::client_card_id(character_id.card_id())),
                effect: hovl::gain_spark(),
                duration: Milliseconds::new(500),
                scale: FlexVector3::new(5.0, 5.0, 5.0),
                sound: Some(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Light Magic/RPG3_LightMagic_Buff01.wav")),
            }));
        }

        BattleAnimation::Judgment { player, new_score } => {
            push_snapshot(builder, snapshot);
            builder.push(Command::DisplayJudgment(DisplayJudgmentCommand {
                player: builder.to_display_player(*player),
                new_score: *new_score,
            }));
        }

        BattleAnimation::MakeChoice { player, choice } => {
            if *player != builder.display_for_player() {
                push_snapshot(builder, snapshot);
                builder.push(Command::PlayAudioClip(PlayAudioClipCommand {
                    sound: AudioClipAddress::new("Assets/ThirdParty/Cafofo/Magic Spells Sound Effects V2.0/General Spell/Cast 12.wav"),
                    pause_duration: Milliseconds::new(0),
                }));
                builder.push(Command::DisplayEnemyMessage(DisplayEnemyMessageCommand {
                    message: labels::choice_label(builder, *choice),
                    show_duration: Milliseconds::new(2000),
                }));
                builder.push(Command::Wait(Milliseconds::new(1000)));
            }
        }

        BattleAnimation::PlayCard { player, .. } => {
            if *player != builder.display_for_player() {
                builder.push(Command::PlayAudioClip(PlayAudioClipCommand {
                    sound: AudioClipAddress::new("Assets/ThirdParty/Cafofo/Magic Spells Sound Effects V2.0/General Spell/Magic Whoosh 4.wav"),
                    pause_duration: Milliseconds::new(0),
                }));
            }
        }

        BattleAnimation::PlayedCard { player, card_id, .. } => {
            if *player != builder.display_for_player()
                && final_state.cards.stack_item(StackCardId(card_id.card_id())).is_none()
            {
                // If the played card is no longer on the stack, insert a pause
                // so it can be seen.
                push_snapshot(builder, snapshot);
                builder.push(Command::Wait(Milliseconds::new(1000)));
            }
        }

        BattleAnimation::PutCardsFromDeckIntoVoid { player, cards } => {
            if !cards.is_empty() {
                push_snapshot(builder, snapshot);
                let card_views = cards
                    .iter()
                    .map(|&card_id| {
                        card_rendering::card_view(
                            builder,
                            &CardViewContext::Battle(
                                final_state,
                                card::get_base_card_id(snapshot, card_id),
                                card_id.card_id(),
                            ),
                        )
                    })
                    .collect();

                builder.push(Command::DrawUserCards(DrawUserCardsCommand {
                    cards: card_views,
                    stagger_interval: Milliseconds::new(500),
                    pause_duration: Milliseconds::new(300),
                    destination: Position::InVoid(builder.to_display_player(*player)),
                }));
            }
        }

        BattleAnimation::ResolveCharacter { .. } => {
            push_snapshot(builder, snapshot);
            builder.push(Command::PlayAudioClip(PlayAudioClipCommand {
                sound: AudioClipAddress::new("Assets/ThirdParty/Cafofo/Magic Spells Sound Effects V2.0/General Spell/Positive Effect 10.wav"),
                pause_duration: Milliseconds::new(0),
            }));
        }

        BattleAnimation::ScorePoints { player, source } => {
            push_snapshot(builder, snapshot);
            if let Some(card_id) = source.card_id() {
                builder.push(Command::FireProjectile(
                    FireProjectileCommand::builder()
                        .source_id(adapter::card_game_object_id(card_id))
                        .target_id(GameObjectId::Avatar(builder.to_display_player(*player)))
                        .projectile(ProjectileAddress::new("Assets/ThirdParty/Hovl Studio/AAA Projectiles Vol 1/Prefabs/Dreamtides/Projectile 4 yellow arrow.prefab"))
                        .travel_duration(Milliseconds::new(300))
                        .fire_sound(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Generic Magic and Impacts/RPG3_Generic_SubtleWhoosh03.wav"))
                        .impact_sound(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Fire Magic/RPG3_FireMagicCannon_Impact01.wav"))
                        .build()
                ));
            }
        }

        BattleAnimation::SelectModalEffectChoice { player, item_id, choice_index } => {
            if *player != builder.display_for_player() {
                push_snapshot(builder, snapshot);
                let definition = card::get_definition(snapshot, item_id.underlying_card_id());
                let descriptions = modal_effect_prompt_rendering::modal_effect_descriptions(
                    &definition.displayed_abilities,
                );
                builder.push(Command::DisplayEnemyMessage(DisplayEnemyMessageCommand {
                    message: descriptions[choice_index.value()].clone(),
                    show_duration: Milliseconds::new(2000),
                }));
                builder.push(Command::Wait(Milliseconds::new(1000)));
            }
        }

        BattleAnimation::SelectedTargetsForCard { .. } => {
            push_snapshot(builder, snapshot);
        }

        BattleAnimation::SetActiveTriggers { triggers } => {
            builder.set_active_triggers(triggers.clone());
            push_snapshot(builder, snapshot);
            if !triggers.is_empty() {
                builder.push(Command::Wait(Milliseconds::new(300)));
            }
        }

        BattleAnimation::StartTurn { player } => {
            push_snapshot(builder, snapshot);
            builder.push(Command::DisplayGameMessage(if *player == builder.display_for_player() {
                GameMessageType::YourTurn
            } else {
                GameMessageType::EnemyTurn
            }));
        }

        _ => {}
    }
}

/// Appends a command to update the battle view to the current state.
pub fn push_snapshot(builder: &mut ResponseBuilder, snapshot: &BattleState) {
    builder.push_battle_view(battle_rendering::battle_view(builder, snapshot));
}
