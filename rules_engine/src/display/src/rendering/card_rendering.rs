use std::collections::HashMap;

use action_data::game_action_data::GameAction;
use battle_queries::battle_card_queries::{card, card_properties, target_queries};
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::{ForPlayer, LegalActions};
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{
    CardId, CardIdType, CharacterId, HandCardId, StackCardId, VoidCardId,
};
use battle_state::battle_cards::stack_card_state::{
    EffectTargets, StackCardAdditionalCostsPaid, StandardEffectTarget,
};
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::card_types::CardType;
use core_data::display_color::{self, DisplayColor};
use core_data::display_types::SpriteAddress;
use core_data::identifiers::CardName;
use core_data::types::CardFacing;
use display_data::card_view::{
    CardActions, CardEffects, CardPrefab, CardView, DisplayImage, InfoZoomData, InfoZoomIcon,
    RevealedCardView,
};
use ui_components::component::Component;
use ui_components::icon;

use crate::core::adapter;
use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::outcome_simulation;
use crate::rendering::positions::ControllerAndZone;
use crate::rendering::supplemental_card_info::SupplementalCardInfo;
use crate::rendering::{card_display_state, positions};

pub fn card_view(builder: &ResponseBuilder, context: &CardViewContext) -> CardView {
    let battle = context.battle();
    CardView {
        id: adapter::client_card_id(context.card_id()),
        position: positions::calculate(builder, context.battle(), context.card_id()),
        revealed: card_display_state::is_revealed_to(
            context.battle(),
            context.card_id(),
            builder.display_for_player(),
        )
        .then(|| revealed_card_view(builder, context)),
        revealed_to_opponents: card_display_state::is_revealed_to(
            context.battle(),
            context.card_id(),
            builder.display_for_player().opponent(),
        ),
        card_facing: CardFacing::FaceUp,
        backless: false,
        create_position: None,
        create_sound: None,
        destroy_position: None,
        prefab: match card_properties::card_type(battle, context.card_id()) {
            CardType::Character(_) => CardPrefab::Character,
            CardType::Event => CardPrefab::Event,
            CardType::Dreamsign => CardPrefab::Dreamsign,
            CardType::Enemy => CardPrefab::Enemy,
            CardType::Dreamwell => CardPrefab::Dreamwell,
        },
    }
}

fn revealed_card_view(builder: &ResponseBuilder, context: &CardViewContext) -> RevealedCardView {
    let battle = context.battle();
    let card_id = context.card_id();
    let legal_actions = legal_actions::compute(battle, builder.act_for_player());

    let play_from_hand = BattleAction::PlayCardFromHand(HandCardId(card_id));
    let can_play_from_hand = legal_actions.contains(play_from_hand, ForPlayer::Human);
    let play_action = can_play_from_hand.then_some(play_from_hand);

    let can_play = play_action.is_some();
    let (selection_color, selection_action) =
        outline_and_selection_action(battle, &legal_actions, card_id);
    let ControllerAndZone { controller, .. } = positions::controller_and_zone(battle, card_id);

    RevealedCardView {
        image: DisplayImage::Sprite(card_image(battle, card_id)),
        name: card_name(battle, card_id),
        cost: card_properties::energy_cost(battle, card_id),
        produced: None,
        spark: card_properties::spark(battle, controller, CharacterId(card_id))
            .or_else(|| card_properties::base_spark(battle, card_id)),
        card_type: card_type(battle, card_id),
        rules_text: rules_text(battle, card_id),
        outline_color: match selection_color {
            Some(color) => Some(color),
            None if can_play => Some(display_color::GREEN),
            _ => None,
        },
        info_zoom_data: build_info_zoom_data(battle, card_id),
        is_fast: false,
        actions: CardActions {
            can_play: play_action.map(GameAction::BattleAction),
            can_select_order: can_select_order_action(&legal_actions, card_id),
            on_click: selection_action,
            play_effect_preview: play_action.map(|play_action| {
                outcome_simulation::action_effect_preview(
                    battle,
                    builder.act_for_player(),
                    play_action,
                )
            }),
            ..Default::default()
        },
        effects: CardEffects::default(),
    }
}

fn outline_and_selection_action(
    battle: &BattleState,
    legal_actions: &LegalActions,
    card_id: CardId,
) -> (Option<DisplayColor>, Option<GameAction>) {
    if legal_actions
        .contains(BattleAction::SelectCharacterTarget(CharacterId(card_id)), ForPlayer::Human)
    {
        return (
            Some(display_color::RED_500),
            Some(GameAction::BattleAction(BattleAction::SelectCharacterTarget(CharacterId(
                card_id,
            )))),
        );
    }

    if legal_actions
        .contains(BattleAction::SelectStackCardTarget(StackCardId(card_id)), ForPlayer::Human)
    {
        return (
            Some(display_color::RED_500),
            Some(GameAction::BattleAction(BattleAction::SelectStackCardTarget(StackCardId(
                card_id,
            )))),
        );
    }

    if let Some(prompt) = battle.prompts.front()
        && let PromptType::ChooseVoidCard(choose_void_prompt) = &prompt.prompt_type
    {
        let void_card_id = VoidCardId(card_id);
        let select = BattleAction::SelectVoidCardTarget(void_card_id);
        let selection_action = legal_actions
            .contains(select, ForPlayer::Human)
            .then_some(GameAction::BattleAction(select));

        if choose_void_prompt.selected.contains(void_card_id) {
            return (Some(display_color::YELLOW_500), selection_action);
        } else if choose_void_prompt.valid.contains(void_card_id) {
            return (Some(display_color::WHITE), selection_action);
        }
    }

    (None, None)
}

fn can_select_order_action(legal_actions: &LegalActions, card_id: CardId) -> Option<CardId> {
    if let LegalActions::SelectDeckCardOrder { .. } = legal_actions { Some(card_id) } else { None }
}

pub fn card_image(battle: &BattleState, card_id: CardId) -> SpriteAddress {
    match card::get(battle, card_id).name {
        CardName::TestVanillaCharacter => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        CardName::TestDissolve => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1907487244.png",
        ),
        CardName::TestCounterspellUnlessPays => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_2123360837.png",
        ),
        CardName::TestCounterspell => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1282908322.png",
        ),
        CardName::TestVariableEnergyDraw => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        CardName::TestDrawOne => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        CardName::TestActivatedAbilityDrawCardCharacter => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        CardName::TestMultiActivatedAbilityDrawCardCharacter => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        CardName::TestFastActivatedAbilityDrawCardCharacter => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        CardName::TestFastMultiActivatedAbilityDrawCardCharacter => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        CardName::TestActivatedAbilityDissolveCharacter => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        CardName::TestDualActivatedAbilityCharacter => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        CardName::TestForeseeOne => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        CardName::TestForeseeTwo => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        CardName::TestForeseeOneDrawACard => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        CardName::TestDrawOneReclaim => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        CardName::TestReturnVoidCardToHand => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        CardName::TestReturnOneOrTwoVoidEventCardsToHand => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
    }
}

pub fn card_name(battle: &BattleState, card_id: CardId) -> String {
    card_properties::display_name(card::get(battle, card_id).name)
}

fn card_type(battle: &BattleState, card_id: CardId) -> String {
    let result = match card_properties::card_type(battle, card_id) {
        CardType::Character(t) => t.to_string(),
        CardType::Event => "Event".to_string(),
        CardType::Dreamsign => "Dreamsign".to_string(),
        CardType::Enemy => "Enemy".to_string(),
        CardType::Dreamwell => "Dreamwell".to_string(),
    };

    if card_properties::is_fast(battle, card_id) { format!("\u{f0e7} {result}") } else { result }
}

pub fn rules_text(battle: &BattleState, card_id: CardId) -> String {
    let base_text = match card::get(battle, card_id).name {
        CardName::TestVanillaCharacter => "<i>As the stars wept fire across the sky, he strummed the chords that once taught the heavens to sing.</i>".to_string(),
        CardName::TestDissolve => "<b>Dissolve</b> an enemy character.".to_string(),
        CardName::TestCounterspellUnlessPays => {
            "<b>Prevent</b> an enemy event unless the enemy pays 2\u{f7e4}.".to_string()
        }
        CardName::TestCounterspell => "<b>Prevent</b> an enemy card.".to_string(),
        CardName::TestVariableEnergyDraw => {
            "Pay one or more \u{f7e4}: Draw a card for each \u{f7e4} spent.".to_string()
        }
        CardName::TestDrawOne => "Draw a card.".to_string(),
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter => {
            "Whenever you materialize another character, this character gains +1 spark.".to_string()
        }
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => {
            "Whenever you play a card during the enemy's turn, this character gains +2 spark.".to_string()
        }
        CardName::TestActivatedAbilityDrawCardCharacter => {
            "1\u{f7e4} -> Draw a card.".to_string()
        }
        CardName::TestMultiActivatedAbilityDrawCardCharacter => {
            "[multi] 1\u{f7e4} -> Draw a card.".to_string()
        }
        CardName::TestFastActivatedAbilityDrawCardCharacter => {
            "[fast] 1\u{f7e4} -> Draw a card.".to_string()
        }
        CardName::TestFastMultiActivatedAbilityDrawCardCharacter => {
            "[fast][multi] 1\u{f7e4} -> Draw a card.".to_string()
        }
        CardName::TestActivatedAbilityDissolveCharacter => {
            "2\u{f7e4} -> <b>Dissolve</b> an enemy character.".to_string()
        }
        CardName::TestDualActivatedAbilityCharacter => {
            "1\u{f7e4} -> Draw a card.\n2\u{f7e4} -> Draw 2 cards.".to_string()
        }
        CardName::TestForeseeOne => {
            "<b>Foresee</b> 1.".to_string()
        }
        CardName::TestForeseeTwo => {
            "<b>Foresee</b> 2.".to_string()
        }
        CardName::TestForeseeOneDrawACard => {
            "<b>Foresee</b> 1. Draw a card.".to_string()
        }
        CardName::TestDrawOneReclaim => {
            "Draw a card. Reclaim 1\u{f7e4}.".to_string()
        }
        CardName::TestReturnVoidCardToHand => {
            "Return a card from your void to your hand.".to_string()
        }
        CardName::TestReturnOneOrTwoVoidEventCardsToHand => {
            "Return one or two events from your void to your hand.".to_string()
        }
    };

    if card::get(battle, card_id).name == CardName::TestVariableEnergyDraw
        && let Some(stack_item) = battle.cards.stack_item(StackCardId(card_id))
        && let StackCardAdditionalCostsPaid::Energy(energy) = &stack_item.additional_costs_paid
    {
        return format!("{} <b><color=\"blue\">({}\u{f7e4} paid)</color></b>", base_text, energy.0);
    }

    base_text
}

fn supplemental_card_info(battle: &BattleState, card_id: CardId) -> Option<String> {
    match card::get(battle, card_id).name {
        CardName::TestDissolve => Some("<b>Dissolve:</b> Send a character to the void".to_string()),
        CardName::TestCounterspellUnlessPays => Some(
            "<b>Prevent:</b> Send a card to the void in response to it being played".to_string(),
        ),
        CardName::TestCounterspell => Some(
            "<b>Prevent:</b> Send a card to the void in response to it being played".to_string(),
        ),
        CardName::TestForeseeOne => Some(
            "<b>Foresee 1:</b> Look at the top card of your deck. You may put it into your void."
                .to_string(),
        ),
        _ => None,
    }
}

fn build_info_zoom_data(battle: &BattleState, card_id: CardId) -> Option<InfoZoomData> {
    let targeting_icons = get_targeting_icons(battle, card_id);
    let supplemental_info = supplemental_card_info(battle, card_id)
        .and_then(|text| SupplementalCardInfo::builder().text(text).build().render()?.flex_node());

    if targeting_icons.is_empty() && supplemental_info.is_none() {
        None
    } else {
        Some(InfoZoomData { supplemental_card_info: supplemental_info, icons: targeting_icons })
    }
}

fn get_targeting_icons(battle: &BattleState, card_id: CardId) -> Vec<InfoZoomIcon> {
    let mut icons = HashMap::new();

    if let Some(prompt) = battle.prompts.front()
        && prompt.source.card_id() == Some(card_id)
        && let PromptType::Choose { choices } = &prompt.prompt_type
    {
        for choice in choices {
            if let Some(targets) = &choice.targets {
                let target_card_ids = match targets {
                    EffectTargets::Standard(StandardEffectTarget::Character(
                        target_character_id,
                        _,
                    )) => {
                        vec![target_character_id.card_id()]
                    }
                    EffectTargets::Standard(StandardEffectTarget::StackCard(
                        target_stack_card_id,
                        _,
                    )) => {
                        vec![target_stack_card_id.card_id()]
                    }
                    EffectTargets::Standard(StandardEffectTarget::VoidCards(void_card_targets)) => {
                        void_card_targets.iter().map(|target| target.id.card_id()).collect()
                    }
                    EffectTargets::EffectList(target_list) => target_list
                        .iter()
                        .flat_map(|target_option| match target_option.as_ref() {
                            Some(StandardEffectTarget::Character(character_id, _)) => {
                                vec![character_id.card_id()]
                            }
                            Some(StandardEffectTarget::StackCard(stack_card_id, _)) => {
                                vec![stack_card_id.card_id()]
                            }
                            Some(StandardEffectTarget::VoidCards(void_card_targets)) => {
                                void_card_targets.iter().map(|target| target.id.card_id()).collect()
                            }
                            None => vec![],
                        })
                        .collect(),
                };

                for target_card_id in target_card_ids {
                    icons.insert(target_card_id, InfoZoomIcon {
                        card_id: adapter::client_card_id(target_card_id),
                        icon: icon::CHEVRON_UP.to_string(),
                        color: display_color::RED_500,
                    });
                }
            }
        }
    }

    if let Some(targets) = target_queries::displayed_targets(battle, StackCardId(card_id)) {
        let target_card_ids = match targets {
            EffectTargets::Standard(StandardEffectTarget::Character(target_character_id, _)) => {
                vec![target_character_id.card_id()]
            }
            EffectTargets::Standard(StandardEffectTarget::StackCard(target_stack_card_id, _)) => {
                vec![target_stack_card_id.card_id()]
            }
            EffectTargets::Standard(StandardEffectTarget::VoidCards(void_card_targets)) => {
                void_card_targets.iter().map(|target| target.id.card_id()).collect()
            }
            EffectTargets::EffectList(target_list) => target_list
                .iter()
                .flat_map(|target_option| match target_option.as_ref() {
                    Some(StandardEffectTarget::Character(character_id, _)) => {
                        vec![character_id.card_id()]
                    }
                    Some(StandardEffectTarget::StackCard(stack_card_id, _)) => {
                        vec![stack_card_id.card_id()]
                    }
                    Some(StandardEffectTarget::VoidCards(void_card_targets)) => {
                        void_card_targets.iter().map(|target| target.id.card_id()).collect()
                    }
                    None => vec![],
                })
                .collect(),
        };

        for target_card_id in target_card_ids {
            icons.insert(target_card_id, InfoZoomIcon {
                card_id: adapter::client_card_id(target_card_id),
                icon: icon::CHEVRON_UP.to_string(),
                color: display_color::RED_500,
            });
        }
    } else if let Some(stack_card) = battle.cards.stack_item(StackCardId(card_id))
        && stack_card.targets.is_some()
        && target_queries::valid_targets(battle, stack_card.targets.as_ref()).is_none()
    {
        icons.insert(card_id, InfoZoomIcon {
            card_id: adapter::client_card_id(card_id),
            icon: icon::XMARK.to_string(),
            color: display_color::RED_500,
        });
    }

    icons.into_values().collect()
}
