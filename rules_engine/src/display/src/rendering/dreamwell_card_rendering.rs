use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle_cards::dreamwell_data::{BattleDreamwellCardId, DreamwellCard};
use core_data::types::CardFacing;
use display_data::card_view::{
    CardActions, CardEffects, CardPrefab, CardView, DisplayImage, RevealedCardView,
};
use display_data::object_position::{ObjectPosition, Position};
use fluent::fluent_args;
use tabula_data::localized_strings::StringContext;
use tabula_ids::string_id;

use crate::core::adapter;
use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::display_state;
use crate::rendering::card_rendering;

/// Returns [CardView]s for all cards present in the dreamwell.
pub fn all_cards(builder: &ResponseBuilder, battle: &BattleState) -> Vec<CardView> {
    battle
        .dreamwell
        .all_cards()
        .map(|(id, card)| dreamwell_card_view(builder, battle, id, &card))
        .collect()
}

fn dreamwell_card_view(
    builder: &ResponseBuilder,
    battle: &BattleState,
    card_id: BattleDreamwellCardId,
    card: &DreamwellCard,
) -> CardView {
    let client_id = adapter::battle_dreamwell_card_id(card_id);
    let player = builder.to_display_player(builder.display_for_player());

    let base_position = if battle.phase == BattleTurnPhase::Dreamwell
        && battle.ability_state.until_end_of_turn.active_dreamwell_card == Some(card_id)
    {
        Position::DreamwellActivation
    } else {
        Position::InDreamwell(player)
    };

    let position = if display_state::is_battlefield_shown(builder)
        && base_position == Position::DreamwellActivation
    {
        Position::OnScreenStorage
    } else {
        base_position
    };

    CardView {
        id: client_id,
        position: ObjectPosition { position, sorting_key: Into::<usize>::into(card_id) as u32 },
        revealed: Some(RevealedCardView {
            image: DisplayImage::Sprite(card.definition.image.clone()),
            name: card.definition.displayed_name.clone(),
            cost: None,
            produced: Some(card.produced_energy.to_string()),
            spark: None,
            card_type: builder.string(string_id::CARD_TYPE_DREAMWELL),
            rules_text: rules_text(builder, card),
            outline_color: None,
            info_zoom_data: None,
            is_fast: false,
            actions: CardActions::default(),
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        backless: false,
        create_position: None,
        create_sound: None,
        destroy_position: None,
        prefab: CardPrefab::Dreamwell,
    }
}

fn rules_text(builder: &ResponseBuilder, card: &DreamwellCard) -> String {
    builder.tabula().strings.format_display_string(
        &card_rendering::get_displayed_text(&card.definition.displayed_abilities),
        StringContext::CardText,
        fluent_args![],
    )
}
