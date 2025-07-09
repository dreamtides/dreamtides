use ability_data::cost::Cost;
use action_data::game_action_data::GameAction;
use battle_queries::battle_card_queries::{card, card_abilities};
use battle_queries::legal_action_queries::legal_actions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_animation::TriggerAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{ActivatedAbilityId, CardIdType};
use bon::Builder;
use core_data::display_color;
use core_data::display_types::{AudioClipAddress, SpriteAddress};
use core_data::numerics::{Energy, Spark};
use core_data::types::CardFacing;
use display_data::battle_view::DisplayPlayer;
use display_data::card_view::{
    CardActions, CardEffects, CardPrefab, CardView, ClientCardId, DisplayImage, RevealedCardView,
};
use display_data::object_position::{ObjectPosition, Position};

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
    token_card_view(
        TokenCardView::builder()
            .id(format!("T{:?}/{:?}", trigger.character_id.card_id().0, trigger.ability_number))
            .position(ObjectPosition {
                position: Position::OnStack(current_stack),
                sorting_key: (battle.cards.next_object_id_for_display().0 + index) as u32,
                sorting_sub_key: 0,
            })
            .image(card_rendering::card_image(battle, trigger.character_id.card_id()))
            .name(card_rendering::card_name(battle, trigger.character_id.card_id()))
            .rules_text(card_rendering::rules_text(battle, trigger.character_id.card_id()))
            .create_position(ObjectPosition {
                position: Position::HiddenWithinCard(adapter::client_card_id(
                    trigger.character_id.card_id(),
                )),
                sorting_key: 0,
                sorting_sub_key: 0,
            })
            .destroy_position(ObjectPosition {
                position: Position::HiddenWithinCard(adapter::client_card_id(
                    trigger.character_id.card_id(),
                )),
                sorting_key: 0,
                sorting_sub_key: 0,
            })
            .create_sound(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/UI, Pads, Enchantments and Misc/RPG3_Enchantment_Subtle01v2.wav"))
            .build(),
    )
}

pub fn activated_ability_card_view(
    builder: &ResponseBuilder,
    battle: &BattleState,
    ability: ActivatedAbilityId,
) -> CardView {
    let character_card_id = ability.character_id.card_id();
    let abilities = card_abilities::query(battle, character_card_id);

    let ability_data = abilities
        .activated_abilities
        .iter()
        .find(|data| data.ability_number == ability.ability_number)
        .expect("Ability not found");

    let action = BattleAction::ActivateAbility(ability);
    let activate_action = Some(GameAction::BattleAction(action));
    let cost = ability_data.ability.costs.iter().find_map(|cost| match cost {
        Cost::Energy(energy) => Some(*energy),
        _ => None,
    });

    let character_name = card_rendering::card_name(battle, character_card_id);
    let ability_name = format!("{character_name} Ability");

    let legal_actions = legal_actions::compute(battle, builder.act_for_player());
    let is_legal_action = legal_actions.contains(action);

    token_card_view(
        TokenCardView::builder()
            .id(format!("A{:?}/{:?}", character_card_id.0, ability.ability_number.0))
            .position(ObjectPosition {
                position: Position::InHand(DisplayPlayer::User),
                sorting_key: card::get(battle, character_card_id).object_id.0 as u32,
                sorting_sub_key: 0,
            })
            .image(card_rendering::card_image(battle, character_card_id))
            .name(ability_name)
            .maybe_cost(cost)
            .maybe_card_type(Some("Activated Ability".to_string()))
            .rules_text(card_rendering::rules_text(battle, character_card_id))
            .actions(CardActions {
                can_play: if is_legal_action { activate_action } else { None },
                play_effect_preview: if is_legal_action {
                    Some(outcome_simulation::action_effect_preview(
                        battle,
                        builder.act_for_player(),
                        action,
                    ))
                } else {
                    None
                },
                ..Default::default()
            })
            .maybe_outline_color(if is_legal_action { Some(display_color::GREEN) } else { None })
            .is_fast(
                ability_data.ability.options.as_ref().map(|opts| opts.is_fast).unwrap_or(false),
            )
            .build(),
    )
}

#[derive(Builder)]
struct TokenCardView {
    id: ClientCardId,
    position: ObjectPosition,
    image: SpriteAddress,
    name: String,
    cost: Option<Energy>,
    spark: Option<Spark>,
    card_type: Option<String>,
    rules_text: String,
    create_position: Option<ObjectPosition>,
    destroy_position: Option<ObjectPosition>,
    #[builder(default)]
    is_fast: bool,
    create_sound: Option<AudioClipAddress>,
    #[builder(default)]
    actions: CardActions,
    outline_color: Option<core_data::display_color::DisplayColor>,
}

fn token_card_view(view: TokenCardView) -> CardView {
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
            info_zoom_data: None,
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
