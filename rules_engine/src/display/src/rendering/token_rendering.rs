use battle_state::battle::battle_animation::TriggerAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use bon::Builder;
use core_data::display_types::{AudioClipAddress, SpriteAddress};
use core_data::numerics::{Energy, Spark};
use core_data::types::CardFacing;
use display_data::card_view::{
    CardActions, CardEffects, CardPrefab, CardView, ClientCardId, DisplayImage, RevealedCardView,
};
use display_data::object_position::{ObjectPosition, Position};

use crate::core::adapter;
use crate::core::response_builder::ResponseBuilder;
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
            outline_color: None,
            is_fast: view.is_fast,
            actions: CardActions::default(),
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
