use ability_data::cost::Cost;
use action_data::game_action_data::GameAction;
use battle_queries::battle_card_queries::{card, card_properties};
use battle_queries::legal_action_queries::legal_actions_data::ForPlayer;
use battle_queries::legal_action_queries::{can_play_cards, legal_actions};
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_animation_data::TriggerAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{
    ActivatedAbilityId, CardId, CardIdType, CharacterId, VoidCardId,
};
use battle_state::battle_cards::stack_card_state::StackItemId;
use bon::Builder;
use core_data::display_color::{self, DisplayColor};
use core_data::display_types::{AudioClipAddress, SpriteAddress};
use core_data::types::CardFacing;
use display_data::battle_view::DisplayPlayer;
use display_data::card_view::{
    CardActions, CardEffects, CardPrefab, CardView, ClientCardId, DisplayImage, InfoZoomData,
    RevealedCardView,
};
use display_data::object_position::{ObjectPosition, Position};
use fluent::fluent_args;
use tabula_ids::string_id;
use ui_components::icon;

use crate::core::adapter;
use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::outcome_simulation;
use crate::rendering::{card_rendering, positions};

pub fn trigger_card_view(
    builder: &ResponseBuilder,
    battle: &BattleState,
    index: usize,
    trigger: &TriggerAnimation,
) -> CardView {
    let current_stack = positions::current_stack_type(builder, battle);
    let character_card_id = trigger.character_id.card_id();
    let definition = card::get_definition(battle, character_card_id);
    token_card_view(
        TokenCardView::builder()
            .id(format!("T{:?}/{:?}", character_card_id.0, trigger.ability_number))
            .position(ObjectPosition {
                position: Position::OnStack(current_stack),
                sorting_key: (battle.cards.next_object_id_for_display().0 + index + 5) as u32,
            })
            .image(card_rendering::card_image(battle, character_card_id))
            .name(card_rendering::card_name(battle, character_card_id))
            .rules_text(card_rendering::ability_token_text(
                builder,
                &definition,
                trigger.ability_number
            ))
            .create_position(ObjectPosition {
                position: Position::HiddenWithinCard(adapter::client_card_id(character_card_id)),
                sorting_key: 0,
            })
            .destroy_position(ObjectPosition {
                position: Position::HiddenWithinCard(adapter::client_card_id(character_card_id)),
                sorting_key: 0,
            })
            .create_sound(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/UI, Pads, Enchantments and Misc/RPG3_Enchantment_Subtle01v2.wav"))
            .maybe_info_zoom_data(build_token_info_zoom_data(builder, battle, character_card_id))
            .build(),
    )
}

/// Returns a list of all activated ability views for a character to display in
/// the player's hand.
pub fn all_user_character_activated_abilities(
    builder: &ResponseBuilder,
    battle: &BattleState,
    character_id: CharacterId,
    token_offset: &mut usize,
) -> Vec<CardView> {
    let abilities = card::ability_list(battle, character_id);
    let player = builder.act_for_player();
    let base_sorting_key = battle.cards.next_object_id_for_display().0 + *token_offset;

    let result: Vec<CardView> = abilities
        .activated_abilities
        .iter()
        .enumerate()
        .filter_map(|(index, ability)| {
            let ability_id =
                ActivatedAbilityId { character_id, ability_number: ability.ability_number };

            let options = ability.ability.options.as_ref();
            let is_multi = options.map(|options| options.is_multi).unwrap_or(false);

            // If the ability is not multi-use and has already been activated
            // this turn cycle, don't show it.
            if !is_multi
                && battle
                    .activated_abilities
                    .player(player)
                    .activated_this_turn_cycle
                    .contains(&ability_id)
            {
                return None;
            }

            // If the ability is currently on the stack, don't show it in hand
            // (it will be displayed on the stack instead)
            let is_on_stack = battle
                .cards
                .all_items_on_stack()
                .iter()
                .any(|stack_item| {
                    matches!(
                        stack_item.id,
                        StackItemId::ActivatedAbility(stack_ability_id) if stack_ability_id == ability_id
                    )
                });

            if is_on_stack {
                return None;
            }

            let hand_sorting_key = (base_sorting_key + index) as u32;
            Some(activated_ability_card_view(
                builder,
                battle,
                ability_id,
                None,
                Some(hand_sorting_key),
            ))
        })
        .collect();

    *token_offset += result.len();
    result
}

/// Returns a card view for an activated ability that is currently on the
/// stack.
pub fn activated_ability_card_view_on_stack(
    builder: &ResponseBuilder,
    battle: &BattleState,
    ability: ActivatedAbilityId,
) -> CardView {
    let current_stack = positions::current_stack_type(builder, battle);
    let stack_position = ObjectPosition {
        position: Position::OnStack(current_stack),
        sorting_key: battle.cards.activated_ability_object_id(ability).unwrap_or_default().0 as u32,
    };

    activated_ability_card_view(builder, battle, ability, Some(stack_position), None)
}

fn activated_ability_card_view(
    builder: &ResponseBuilder,
    battle: &BattleState,
    ability: ActivatedAbilityId,
    position_override: Option<ObjectPosition>,
    hand_sorting_key: Option<u32>,
) -> CardView {
    let character_card_id = ability.character_id.card_id();
    let abilities = card::ability_list(battle, character_card_id);
    let definition = card::get_definition(battle, character_card_id);

    let ability_data = abilities
        .activated_abilities
        .iter()
        .find(|data| data.ability_number == ability.ability_number)
        .expect("Ability not found");

    let action = BattleAction::ActivateAbilityForCharacter(ability.character_id);
    let activate_action = Some(GameAction::BattleAction(action));
    let cost = ability_data.ability.costs.iter().find_map(|cost| match cost {
        Cost::Energy(energy) => Some(*energy),
        _ => None,
    });

    let character_name = card_rendering::card_name(battle, character_card_id);
    let ability_name = builder.string_with_args(
        string_id::CHARACTER_ABILITY_CARD_NAME,
        fluent_args!("character-name" => character_name),
    );

    let legal_actions = legal_actions::compute(battle, builder.act_for_player());
    let is_legal_action = legal_actions.contains(action, ForPlayer::Human);

    let position = if let Some(override_position) = position_override {
        override_position
    } else if let Some(sorting_key) = hand_sorting_key {
        ObjectPosition { position: Position::InHand(DisplayPlayer::User), sorting_key }
    } else {
        ObjectPosition {
            position: Position::InHand(DisplayPlayer::User),
            sorting_key: card::get(battle, character_card_id).object_id.0 as u32,
        }
    };

    token_card_view(
        TokenCardView::builder()
            .id(adapter::stack_item_client_card_id(ability))
            .position(position)
            .image(card_rendering::card_image(battle, character_card_id))
            .name(ability_name)
            .maybe_cost(cost.map(|cost| cost.to_string()))
            .maybe_card_type(Some("Activated Ability".to_string()))
            .rules_text(card_rendering::ability_token_text(
                builder,
                &definition,
                ability.ability_number,
            ))
            .create_position(ObjectPosition {
                position: Position::HiddenWithinCard(adapter::client_card_id(character_card_id)),
                sorting_key: 0,
            })
            .destroy_position(ObjectPosition {
                position: Position::HiddenWithinCard(adapter::client_card_id(character_card_id)),
                sorting_key: 0,
            })
            .actions(CardActions {
                can_play: if is_legal_action { activate_action } else { None },
                play_effect_preview: if is_legal_action {
                    Some(outcome_simulation::action_effect_preview(
                        builder,
                        battle,
                        builder.act_for_player(),
                        action,
                    ))
                } else {
                    None
                },
                ..Default::default()
            })
            .maybe_outline_color(if is_legal_action {
                Some(display_color::PURPLE_300)
            } else {
                None
            })
            .is_fast(
                ability_data.ability.options.as_ref().map(|opts| opts.is_fast).unwrap_or(false),
            )
            .maybe_info_zoom_data(build_token_info_zoom_data(builder, battle, character_card_id))
            .build(),
    )
}

/// Returns a list of all void card tokens for a player to display in
/// their hand.
pub fn all_user_void_card_tokens(builder: &ResponseBuilder, battle: &BattleState) -> Vec<CardView> {
    let mut offset = 0;
    all_user_void_card_tokens_with_offset(builder, battle, &mut offset)
}

/// Returns a list of all void card tokens for a player to display in
/// their hand, updating the token offset for coordinated sorting.
pub fn all_user_void_card_tokens_with_offset(
    builder: &ResponseBuilder,
    battle: &BattleState,
    token_offset: &mut usize,
) -> Vec<CardView> {
    let void_ability_cards =
        battle.ability_state.has_play_from_void_ability.player(builder.act_for_player());
    let base_sorting_key = battle.cards.next_object_id_for_display().0 + *token_offset;

    let result: Vec<CardView> = void_ability_cards
        .iter()
        .enumerate()
        .map(|(index, card_id)| {
            let hand_sorting_key = (base_sorting_key + index) as u32;
            void_card_token_view(builder, battle, card_id, hand_sorting_key)
        })
        .collect();

    *token_offset += result.len();
    result
}

fn void_card_token_view(
    builder: &ResponseBuilder,
    battle: &BattleState,
    void_card_id: VoidCardId,
    hand_sorting_key: u32,
) -> CardView {
    let card_id = void_card_id.card_id();
    let from_void_with_cost = can_play_cards::can_play_from_void_energy_cost(battle, void_card_id);
    let legal_actions = legal_actions::compute(battle, builder.act_for_player());
    let play_action = if from_void_with_cost.is_some() {
        let action = BattleAction::PlayCardFromVoid(void_card_id);
        if legal_actions.contains(action, ForPlayer::Human) { Some(action) } else { None }
    } else {
        None
    };

    token_card_view(
        TokenCardView::builder()
            .id(adapter::void_card_token_client_id(void_card_id))
            .position(ObjectPosition {
                position: Position::InHand(DisplayPlayer::User),
                sorting_key: hand_sorting_key,
            })
            .image(card_rendering::card_image(battle, card_id))
            .name(card_rendering::card_name(battle, card_id))
            .cost(
                from_void_with_cost
                    .map(|cost| cost.cost.to_string())
                    .unwrap_or_else(|| format!("<size=50%>{}</size>", icon::NON_NUMERIC)),
            )
            .maybe_spark(
                card_properties::base_spark(battle, card_id).map(|spark| spark.to_string()),
            )
            .rules_text(card_rendering::rules_text(builder, battle, card_id))
            .create_position(ObjectPosition {
                position: Position::InVoid(DisplayPlayer::User),
                sorting_key: 32768,
            })
            .destroy_position(ObjectPosition {
                position: Position::InVoid(DisplayPlayer::User),
                sorting_key: 32768,
            })
            .actions(CardActions {
                can_play: play_action.map(GameAction::BattleAction),
                play_effect_preview: play_action.map(|action| {
                    outcome_simulation::action_effect_preview(
                        builder,
                        battle,
                        builder.act_for_player(),
                        action,
                    )
                }),
                ..Default::default()
            })
            .maybe_outline_color(play_action.map(|_| display_color::PURPLE_300))
            .is_fast(card_properties::is_fast(battle, card_id))
            .maybe_info_zoom_data(build_token_info_zoom_data(builder, battle, card_id))
            .build(),
    )
}

/// A view for a token card.
#[derive(Builder)]
pub struct TokenCardView {
    id: ClientCardId,
    position: ObjectPosition,
    image: SpriteAddress,
    name: String,
    cost: Option<String>,
    spark: Option<String>,
    card_type: Option<String>,
    rules_text: String,
    create_position: Option<ObjectPosition>,
    destroy_position: Option<ObjectPosition>,
    #[builder(default)]
    is_fast: bool,
    create_sound: Option<AudioClipAddress>,
    #[builder(default)]
    actions: CardActions,
    outline_color: Option<DisplayColor>,
    info_zoom_data: Option<InfoZoomData>,
}

/// Converts a [TokenCardView] to a [CardView].
pub fn token_card_view(view: TokenCardView) -> CardView {
    CardView {
        id: view.id,
        position: view.position,
        revealed: Some(RevealedCardView {
            image: DisplayImage::Sprite(view.image),
            name: view.name,
            cost: view.cost,
            produced: None,
            spark: view.spark,
            card_type: view.card_type.unwrap_or_default(),
            rules_text: view.rules_text,
            outline_color: view.outline_color,
            is_fast: view.is_fast,
            actions: view.actions,
            effects: CardEffects::default(),
            info_zoom_data: view.info_zoom_data,
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        backless: true,
        create_position: view.create_position,
        create_sound: view.create_sound,
        destroy_position: view.destroy_position,
        prefab: CardPrefab::Token,
    }
}

fn build_token_info_zoom_data(
    builder: &ResponseBuilder,
    battle: &BattleState,
    parent_card_id: CardId,
) -> Option<InfoZoomData> {
    card_rendering::build_info_zoom_data(builder, battle, parent_card_id)
}
