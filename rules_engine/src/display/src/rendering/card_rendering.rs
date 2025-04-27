use action_data::battle_action_data::BattleAction;
use action_data::game_action::GameAction;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_id::{CardIdType, HandCardId};
use battle_data::prompt_types::prompt_data::Prompt;
use battle_queries::legal_action_queries::can_play_card;
use core_data::display_color;
use core_data::display_types::SpriteAddress;
use core_data::identifiers::CardId;
use core_data::types::CardFacing;
use display_data::card_view::{
    CardActions, CardEffects, CardPrefab, CardView, DisplayImage, RevealedCardView,
};

use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::rendering::positions;

pub fn card_view(builder: &ResponseBuilder, context: &CardViewContext) -> CardView {
    CardView {
        id: context.card().id.card_id(),
        position: positions::calculate(builder, context.card()),
        revealed: context
            .card()
            .is_revealed_to(builder.display_for_player())
            .then(|| revealed_card_view(builder, context)),
        revealed_to_opponents: context.card().revealed_to_opponent,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Character,
    }
}

fn revealed_card_view(builder: &ResponseBuilder, context: &CardViewContext) -> RevealedCardView {
    let battle = context.battle();
    let card_id = context.card().id.card_id();

    let can_play = context.card().controller() == builder.act_for_player()
        && can_play_card::from_hand(battle, HandCardId(card_id));
    let selection_target = selection_target(builder, context.battle(), card_id);

    RevealedCardView {
        image: DisplayImage {
            address: SpriteAddress::new(
                "Assets/ThirdParty/GameAssets/CardImages/Standard/2521694543.png",
            ),
        },
        name: format!("{:?}", context.card().id.card_id()),
        cost: context.card().properties.cost,
        produced: None,
        spark: context.card().properties.spark,
        card_type: "Character".to_string(),
        rules_text: format!("{:?}", context.card().abilities),
        outline_color: match () {
            _ if can_play => Some(display_color::GREEN),
            _ if selection_target.is_some() => Some(display_color::RED_500),
            _ => None,
        },
        supplemental_card_info: None,
        is_fast: false,
        actions: CardActions { can_play, on_click: selection_target, ..Default::default() },
        effects: CardEffects::default(),
    }
}

fn selection_target(
    builder: &ResponseBuilder,
    battle: &BattleData,
    card_id: CardId,
) -> Option<GameAction> {
    if let Some(prompt_data) = &battle.prompt {
        if prompt_data.player == builder.act_for_player() {
            match &prompt_data.prompt {
                Prompt::ChooseCharacter { valid } => {
                    return valid.iter().find(|target_id| target_id.card_id() == card_id).map(
                        |&id| GameAction::BattleAction(BattleAction::SelectCharacterTarget(id)),
                    );
                }
                Prompt::ChooseStackCard { valid } => {
                    return valid.iter().find(|target_id| target_id.card_id() == card_id).map(
                        |&id| GameAction::BattleAction(BattleAction::SelectStackCardTarget(id)),
                    );
                }
                _ => {}
            }
        }
    }

    None
}
