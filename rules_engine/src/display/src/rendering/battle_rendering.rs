use battle_queries::battle_card_queries::{card, card_properties, valid_target_queries};
use battle_queries::battle_player_queries::player_properties;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::card_id::CharacterId;
use battle_state::battle_cards::stack_card_state::{
    EffectTargets, StackItemId, StandardEffectTarget,
};
use battle_state::battle_player::battle_player_state::BattlePlayerState;
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::types::PlayerName;
use display_data::battle_view::{
    BattlePreviewState, BattleView, DisplayedTurnIndicator, PlayerView,
};
use display_data::command::{ArrowStyle, Command, DisplayArrow, GameMessageType};

use crate::core::adapter;
use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::{display_state, outcome_simulation};
use crate::rendering::{
    card_rendering, identity_card_rendering, interface_rendering, modal_effect_prompt_rendering,
    token_rendering,
};

pub fn run(builder: &mut ResponseBuilder, battle: &BattleState) {
    builder.push_battle_view(battle_view(builder, battle));
    update_display_state(builder, battle);

    if let BattleStatus::GameOver { winner } = battle.status {
        builder.push(Command::DisplayGameMessage(
            if winner == Some(builder.display_for_player()) {
                GameMessageType::Victory
            } else {
                GameMessageType::Defeat
            },
        ));
    }
}

pub fn battle_view(builder: &ResponseBuilder, battle: &BattleState) -> BattleView {
    let mut cards = battle
        .cards
        .all_cards()
        .map(|id| {
            card_rendering::card_view(
                builder,
                &CardViewContext::Battle(
                    battle,
                    card::get(battle, id).identity.tmp_to_card_name(),
                    id,
                ),
            )
        })
        .collect::<Vec<_>>();

    cards.extend(modal_effect_prompt_rendering::cards(builder, battle));

    // Add activated abilities that are currently on the stack
    for stack_item in battle.cards.all_items_on_stack().iter() {
        if let StackItemId::ActivatedAbility(ability_id) = stack_item.id {
            cards.push(token_rendering::activated_ability_card_view_on_stack(
                builder, battle, ability_id,
            ));
        }
    }

    cards.extend(builder.active_triggers().iter().enumerate().map(|(index, trigger)| {
        token_rendering::trigger_card_view(builder, battle, index, trigger)
    }));

    let mut token_offset = 100;

    cards.extend(
        battle.activated_abilities.player(builder.display_for_player()).characters.iter().flat_map(
            |character_id| {
                token_rendering::all_user_character_activated_abilities(
                    builder,
                    battle,
                    character_id,
                    &mut token_offset,
                )
            },
        ),
    );

    cards.extend(token_rendering::all_user_void_card_tokens_with_offset(
        builder,
        battle,
        &mut token_offset,
    ));

    cards.extend(modal_effect_prompt_rendering::cards(builder, battle));

    cards.push(identity_card_rendering::identity_card_view(
        builder,
        battle,
        builder.display_for_player(),
    ));
    cards.push(identity_card_rendering::identity_card_view(
        builder,
        battle,
        builder.display_for_player().opponent(),
    ));

    BattleView {
        id: battle.id,
        user: player_view(
            battle,
            builder.display_for_player(),
            battle.players.player(builder.display_for_player()),
        ),
        enemy: player_view(
            battle,
            builder.display_for_player().opponent(),
            battle.players.player(builder.display_for_player().opponent()),
        ),
        cards,
        interface: interface_rendering::interface_view(builder, battle),
        arrows: current_arrows(builder, battle),
        preview: if builder.is_for_animation() {
            BattlePreviewState::Pending
        } else {
            outcome_simulation::current_prompt_battle_preview(
                builder,
                battle,
                builder.display_for_player(),
            )
            .map(|preview| BattlePreviewState::Active(Box::new(preview)))
            .unwrap_or(BattlePreviewState::None)
        },
    }
}

fn update_display_state(builder: &ResponseBuilder, battle: &BattleState) {
    if battle
        .prompts
        .front()
        .map(|p| &p.prompt_type)
        .is_none_or(|pt| !matches!(pt, PromptType::ChooseEnergyValue { .. }))
    {
        display_state::clear_selected_energy_additional_cost(builder);
    }
}

fn player_view(battle: &BattleState, name: PlayerName, player: &BattlePlayerState) -> PlayerView {
    PlayerView {
        score: player.points,
        can_act: legal_actions::next_to_act(battle) == Some(name),
        energy: player.current_energy,
        produced_energy: player.produced_energy,
        total_spark: player_properties::spark_total(battle, name),
        turn_indicator: if battle.turn.active_player == name {
            if battle.phase == BattleTurnPhase::Ending {
                Some(DisplayedTurnIndicator::Right)
            } else {
                Some(DisplayedTurnIndicator::Left)
            }
        } else {
            None
        },
        is_victory_imminent: outcome_simulation::is_victory_imminent_for_player(battle, name),
    }
}

fn current_arrows(builder: &ResponseBuilder, battle: &BattleState) -> Vec<DisplayArrow> {
    if builder.is_for_animation() {
        return vec![];
    }

    let mut arrows = Vec::new();

    for stack_item in battle.cards.all_items_on_stack().iter() {
        if let Some(targets) = valid_target_queries::displayed_targets(battle, stack_item.id) {
            let source = adapter::stack_item_game_object_id(stack_item.id);
            match targets {
                EffectTargets::Standard(StandardEffectTarget::Character(card_object_id)) => {
                    let target = adapter::card_game_object_id(card_object_id.card_id);
                    let color = character_arrow_color(
                        builder,
                        battle,
                        card_object_id.card_id,
                        stack_item.controller,
                    );
                    arrows.push(DisplayArrow { source, target, color });
                }
                EffectTargets::Standard(StandardEffectTarget::StackCard(card_object_id)) => {
                    let target = adapter::card_game_object_id(card_object_id.card_id);
                    arrows.push(DisplayArrow { source, target, color: ArrowStyle::Blue });
                }
                EffectTargets::Standard(StandardEffectTarget::VoidCardSet(void_card_set)) => {
                    for void_card_target in void_card_set.iter() {
                        let target = adapter::card_game_object_id(void_card_target.card_id);
                        arrows.push(DisplayArrow {
                            source: source.clone(),
                            target,
                            color: ArrowStyle::Green,
                        });
                    }
                }
                EffectTargets::EffectList(target_list) => {
                    for target in target_list.iter().flatten() {
                        match target {
                            StandardEffectTarget::Character(card_object_id) => {
                                let target_id =
                                    adapter::card_game_object_id(card_object_id.card_id);
                                let color = character_arrow_color(
                                    builder,
                                    battle,
                                    card_object_id.card_id,
                                    stack_item.controller,
                                );
                                arrows.push(DisplayArrow {
                                    source: source.clone(),
                                    target: target_id,
                                    color,
                                });
                            }
                            StandardEffectTarget::StackCard(card_object_id) => {
                                let target_id =
                                    adapter::card_game_object_id(card_object_id.card_id);
                                arrows.push(DisplayArrow {
                                    source: source.clone(),
                                    target: target_id,
                                    color: ArrowStyle::Blue,
                                });
                            }
                            StandardEffectTarget::VoidCardSet(void_card_set) => {
                                for void_card_target in void_card_set.iter() {
                                    let target_id =
                                        adapter::card_game_object_id(void_card_target.card_id);
                                    arrows.push(DisplayArrow {
                                        source: source.clone(),
                                        target: target_id,
                                        color: ArrowStyle::Green,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    arrows
}

fn character_arrow_color(
    _builder: &ResponseBuilder,
    battle: &BattleState,
    character_id: CharacterId,
    stack_item_controller: PlayerName,
) -> ArrowStyle {
    let character_controller = card_properties::controller(battle, character_id);
    if character_controller == stack_item_controller { ArrowStyle::Green } else { ArrowStyle::Red }
}
