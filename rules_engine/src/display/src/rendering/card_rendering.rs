use std::collections::HashMap;

use ability_data::effect::ModelEffectChoiceIndex;
use action_data::game_action_data::GameAction;
use battle_queries::battle_card_queries::{card, card_properties, valid_target_queries};
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
use core_data::card_types::{CardSubtype, CardType};
use core_data::display_color::{self, DisplayColor};
use core_data::display_types::SpriteAddress;
use core_data::types::CardFacing;
use display_data::card_view::{
    CardActions, CardPrefab, CardView, DisplayImage, InfoZoomData, InfoZoomIcon, RevealedCardView,
};
use fluent::fluent_args;
use masonry::flex_enums::FlexDirection;
use masonry::flex_style::FlexStyle;
use quest_state::quest::card_descriptor;
use tabula_ids::{string_id, test_card};
use ui_components::box_component::BoxComponent;
use ui_components::component::Component;
use ui_components::icon;

use crate::core::adapter;
use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::outcome_simulation;
use crate::rendering::positions::ControllerAndZone;
use crate::rendering::supplemental_card_info::SupplementalCardInfo;
use crate::rendering::{
    apply_card_fx, card_display_state, modal_effect_prompt_rendering, positions,
};

/// Returns the appropriate targeting color based on card ownership
fn targeting_color(
    battle: &BattleState,
    current_player: core_data::types::PlayerName,
    target_card_id: CardId,
) -> DisplayColor {
    let target_controller = card_properties::controller(battle, target_card_id);
    if target_controller == current_player {
        display_color::GREEN_500
    } else {
        display_color::RED_500
    }
}

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
            CardType::Character => CardPrefab::Character,
            CardType::Event => CardPrefab::Event,
            CardType::Dreamsign => CardPrefab::Dreamsign,
            CardType::Dreamcaller => CardPrefab::Identity,
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
        outline_and_selection_action(battle, &legal_actions, card_id, builder.act_for_player());
    let ControllerAndZone { controller, .. } = positions::controller_and_zone(battle, card_id);

    RevealedCardView {
        image: DisplayImage::Sprite(card_image(battle, card_id)),
        name: card_name(battle, card_id),
        cost: if card_properties::base_energy_cost_for_id(battle, card_id).is_some() {
            Some(card_properties::converted_energy_cost(battle, card_id).to_string())
        } else {
            Some(format!("<size=50%>{}</size>", builder.string(string_id::ASTERISK_ICON)))
        },
        produced: None,
        spark: card_properties::spark(battle, controller, CharacterId(card_id))
            .or_else(|| card_properties::base_spark_for_id(battle, card_id))
            .map(|spark| spark.to_string()),
        card_type: card_type(battle, card_id),
        rules_text: rules_text(builder, battle, card_id),
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
                    builder,
                    battle,
                    builder.act_for_player(),
                    play_action,
                )
            }),
            ..Default::default()
        },
        effects: apply_card_fx::persistent_card_effects(battle, card_id),
    }
}

fn outline_and_selection_action(
    battle: &BattleState,
    legal_actions: &LegalActions,
    card_id: CardId,
    current_player: core_data::types::PlayerName,
) -> (Option<DisplayColor>, Option<GameAction>) {
    if legal_actions
        .contains(BattleAction::SelectCharacterTarget(CharacterId(card_id)), ForPlayer::Human)
    {
        return (
            Some(targeting_color(battle, current_player, card_id)),
            Some(GameAction::BattleAction(BattleAction::SelectCharacterTarget(CharacterId(
                card_id,
            )))),
        );
    }

    if legal_actions
        .contains(BattleAction::SelectStackCardTarget(StackCardId(card_id)), ForPlayer::Human)
    {
        return (
            Some(targeting_color(battle, current_player, card_id)),
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
    match card_descriptor::get_base_card_id(card::get(battle, card_id).identity) {
        test_card::TEST_VANILLA_CHARACTER => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        test_card::TEST_DISSOLVE => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1907487244.png",
        ),
        test_card::TEST_NAMED_DISSOLVE => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1907487244.png",
        ),
        test_card::TEST_COUNTERSPELL_UNLESS_PAYS => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_2123360837.png",
        ),
        test_card::TEST_COUNTERSPELL => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1282908322.png",
        ),
        test_card::TEST_VARIABLE_ENERGY_DRAW => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        test_card::TEST_DRAW_ONE => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        test_card::TEST_TRIGGER_GAIN_SPARK_WHEN_MATERIALIZE_ANOTHER_CHARACTER => {
            SpriteAddress::new(
                "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
            )
        }
        test_card::TEST_TRIGGER_GAIN_SPARK_ON_PLAY_CARD_ENEMY_TURN => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_403770319.png",
        ),
        test_card::TEST_TRIGGER_GAIN_TWO_SPARK_ON_PLAY_CARD_ENEMY_TURN => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        test_card::TEST_ACTIVATED_ABILITY_DRAW_CARD => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        test_card::TEST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        test_card::TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        test_card::TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        test_card::TEST_ACTIVATED_ABILITY_DISSOLVE_CHARACTER => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        test_card::TEST_DUAL_ACTIVATED_ABILITY_CHARACTER => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        test_card::TEST_FORESEE_ONE => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        test_card::TEST_FORESEE_TWO => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        test_card::TEST_FORESEE_ONE_DRAW_A_CARD => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        test_card::TEST_DRAW_ONE_RECLAIM => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        test_card::TEST_RETURN_VOID_CARD_TO_HAND => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        test_card::TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_644603677.png",
        ),
        test_card::TEST_MODAL_DRAW_ONE_OR_DRAW_TWO => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        test_card::TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        test_card::TEST_RETURN_TO_HAND => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        test_card::TEST_PREVENT_DISSOLVE_THIS_TURN => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1621160806.png",
        ),
        test_card::TEST_COUNTERSPELL_CHARACTER => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1239919309.png",
        ),
        test_card::TEST_FORESEE_ONE_RECLAIM => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1328168243.png",
        ),
        test_card::TEST_FORESEE_ONE_DRAW_RECLAIM => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1328168243.png",
        ),
        test_card::TEST_MODAL_RETURN_TO_HAND_OR_DRAW_TWO => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1200949264.png",
        ),
        _ => panic!("Unknown card: {card_id:?}"),
    }
}

pub fn card_name(battle: &BattleState, card_id: CardId) -> String {
    card_properties::display_name(card::get(battle, card_id).identity)
}

fn card_type(battle: &BattleState, card_id: CardId) -> String {
    let result = match card_properties::card_type(battle, card_id) {
        CardType::Character => match card_properties::card_subtype(battle, card_id) {
            Some(CardSubtype::Musician) => "Musician".to_string(),
            _ => "Character".to_string(),
        },
        CardType::Event => "Event".to_string(),
        CardType::Dreamsign => "Dreamsign".to_string(),
        CardType::Dreamcaller => "Dreamcaller".to_string(),
        CardType::Dreamwell => "Dreamwell".to_string(),
    };

    if card_properties::is_fast(battle, card_id) { format!("\u{f0e7} {result}") } else { result }
}

pub fn rules_text(builder: &ResponseBuilder, battle: &BattleState, card_id: CardId) -> String {
    let mut base_text = match card_descriptor::get_base_card_id(card::get(battle, card_id).identity) {
        test_card::TEST_VANILLA_CHARACTER => "<i>As the stars wept fire across the sky, he strummed the chords that once taught the heavens to sing.</i>".to_string(),
        test_card::TEST_DISSOLVE => "<color=#AA00FF><b>Dissolve</b></color> an enemy character.".to_string(),
        test_card::TEST_NAMED_DISSOLVE => "<color=#AA00FF><b>Dissolve</b></color> an enemy character.".to_string(),
        test_card::TEST_COUNTERSPELL_UNLESS_PAYS => {
            "<color=#AA00FF><b>Prevent</b></color> a played enemy event unless the enemy pays <color=#00838F><b>2\u{f7e4}</b></color>.".to_string()
        }
        test_card::TEST_COUNTERSPELL => "<color=#AA00FF><b>Prevent</b></color> a played enemy card.".to_string(),
        test_card::TEST_VARIABLE_ENERGY_DRAW => {
            "Pay one or more <color=#00838F>\u{f7e4}</color>: Draw a card for each <color=#00838F>\u{f7e4}</color> spent.".to_string()
        }
        test_card::TEST_DRAW_ONE => "Draw a card.".to_string(),
        test_card::TEST_TRIGGER_GAIN_SPARK_WHEN_MATERIALIZE_ANOTHER_CHARACTER => {
            "Whenever you materialize another character, this character gains +1 spark.".to_string()
        }
        test_card::TEST_TRIGGER_GAIN_SPARK_ON_PLAY_CARD_ENEMY_TURN => {
            "Whenever you play a card during the enemy's turn, this character gains +1 spark.".to_string()
        }
        test_card::TEST_TRIGGER_GAIN_TWO_SPARK_ON_PLAY_CARD_ENEMY_TURN => {
            "Whenever you play a card during the enemy's turn, this character gains +2 spark.".to_string()
        }
        test_card::TEST_ACTIVATED_ABILITY_DRAW_CARD => {
            "1\u{f7e4} -> Draw a card.".to_string()
        }
        test_card::TEST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => {
            "[multi] 1\u{f7e4} -> Draw a card.".to_string()
        }
        test_card::TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => {
            "[fast] 1\u{f7e4} -> Draw a card.".to_string()
        }
        test_card::TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER => {
            format!(
                "{}<space=\"-0.25px\">{} <color=#00838F><b>3\u{f7e4}</b></color><b><size=120%>:</size></b> Draw a card.",
                icon::FAST,
                icon::MULTI_ACTIVATED
            )
        }
        test_card::TEST_ACTIVATED_ABILITY_DISSOLVE_CHARACTER => {
            "2\u{f7e4} -> <b>Dissolve</b> an enemy character.".to_string()
        }
        test_card::TEST_DUAL_ACTIVATED_ABILITY_CHARACTER => {
            "1\u{f7e4} -> Draw a card.\n2\u{f7e4} -> Draw 2 cards.".to_string()
        }
        test_card::TEST_FORESEE_ONE => {
            "<b>Foresee</b> 1.".to_string()
        }
        test_card::TEST_FORESEE_TWO => {
            "<b>Foresee</b> 2.".to_string()
        }
        test_card::TEST_FORESEE_ONE_DRAW_A_CARD => {
            "<b>Foresee</b> 1. Draw a card.".to_string()
        }
        test_card::TEST_DRAW_ONE_RECLAIM => {
            "Draw a card. Reclaim 1\u{f7e4}.".to_string()
        }
        test_card::TEST_RETURN_VOID_CARD_TO_HAND => {
            "Return a card from your void to your hand.".to_string()
        }
        test_card::TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND => {
            "Return one or two events from your void to your hand.".to_string()
        }
        test_card::TEST_MODAL_DRAW_ONE_OR_DRAW_TWO => {
            "Choose one:\n • <indent=1em>1\u{f7e4} : Draw 1 card.</indent>\n • <indent=1em>3\u{f7e4}: Draw 2 cards.</indent>".to_string()
        }
        test_card::TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY => {
            "Choose one:\n • <indent=1em>1\u{f7e4}: Draw 1 card.</indent>\n • <indent=1em>2\u{f7e4}: <b>Dissolve</b> an enemy character.</indent>".to_string()
        }
        test_card::TEST_RETURN_TO_HAND => {
            "<b>Return</b> an enemy character to its owner's hand.".to_string()
        }
        test_card::TEST_PREVENT_DISSOLVE_THIS_TURN => {
            "Give an allied character <color=#AA00FF><b>anchored</b></color> until end of turn.".to_string()
        }
        test_card::TEST_COUNTERSPELL_CHARACTER => {
            "<color=#AA00FF><b>Prevent</b></color> a played enemy character.".to_string()
        }
        test_card::TEST_FORESEE_ONE_RECLAIM => {
            "<line-height=120%><color=#AA00FF><b>Foresee</b></color> 1.\n</line-height><color=#AA00FF><b>Reclaim</b></color> <color=#00838F><b>3\u{f7e4}</b></color>".to_string()
        }
        test_card::TEST_FORESEE_ONE_DRAW_RECLAIM => {
            "<line-height=120%><color=#AA00FF><b>Foresee</b></color> 1. Draw a card.\n</line-height><color=#AA00FF><b>Reclaim</b></color> <color=#00838F><b>4\u{f7e4}</b></color>".to_string()
        }
        test_card::TEST_MODAL_RETURN_TO_HAND_OR_DRAW_TWO => {
            "Choose one:\n • <indent=1em><color=#00838F><b>2\u{f7e4}</b></color>: Return an enemy character to hand.</indent>\n • <indent=1em><color=#00838F><b>3\u{f7e4}</b></color>: Draw 2 cards.</indent>".to_string()
        }
        _ => panic!("Unknown card: {card_id:?}"),
    };

    if let Some(stack_item) = battle.cards.stack_item(StackCardId(card_id))
        && let Some(ModelEffectChoiceIndex(index)) = stack_item.modal_choice
    {
        base_text =
            modal_effect_prompt_rendering::modal_effect_descriptions(&base_text)[index].clone();
    }

    if card_descriptor::get_base_card_id(card::get(battle, card_id).identity)
        == test_card::TEST_VARIABLE_ENERGY_DRAW
        && let Some(stack_item) = battle.cards.stack_item(StackCardId(card_id))
        && let StackCardAdditionalCostsPaid::Energy(energy) = &stack_item.additional_costs_paid
    {
        return format!(
            "{} <b><color=\"blue\">{}</color></b>",
            base_text,
            builder.string_with_args(
                string_id::CARD_RULES_TEXT_ENERGY_PAID,
                fluent_args!("energy" => energy.0)
            )
        );
    }

    if is_on_stack_from_void(battle, card_id) {
        return format!(
            "{base_text} <b><color=\"blue\">{}</color></b>",
            builder.string(string_id::CARD_RULES_TEXT_RECLAIMED)
        );
    }

    if apply_card_fx::is_anchored(battle, card_id) {
        return format!(
            "{base_text} <b><color=\"blue\">{}</color></b>",
            builder.string(string_id::CARD_RULES_TEXT_ANCHORED)
        );
    }

    base_text
}

/// Returns true if the the `card_id` is on the stack and was played from the
/// void.
fn is_on_stack_from_void(battle: &BattleState, card_id: CardId) -> bool {
    if battle.cards.stack_item(StackCardId(card_id)).is_none() {
        return false;
    }

    battle
        .action_history
        .as_ref()
        .map(|history| {
            history
                .actions
                .iter()
                .rev()
                .find_map(|history_action| match &history_action.action {
                    BattleAction::PlayCardFromVoid(void_card_id, _)
                        if void_card_id.card_id() == card_id =>
                    {
                        Some(true)
                    }
                    BattleAction::PlayCardFromHand(hand_card_id)
                        if hand_card_id.card_id() == card_id =>
                    {
                        Some(false)
                    }
                    _ => None,
                })
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

fn supplemental_card_info(battle: &BattleState, card_id: CardId) -> Vec<String> {
    match card_descriptor::get_base_card_id(card::get(battle, card_id).identity) {
        test_card::TEST_DISSOLVE => {
            vec!["<b>Dissolve:</b> Send a character to the void".to_string()]
        }
        test_card::TEST_NAMED_DISSOLVE => {
            vec!["<b>Dissolve:</b> Send a character to the void".to_string()]
        }
        test_card::TEST_COUNTERSPELL_UNLESS_PAYS => vec![
            "<b>Prevent:</b> Send a card to the void in response to it being played".to_string(),
        ],
        test_card::TEST_COUNTERSPELL => vec![
            "<b>Prevent:</b> Send a card to the void in response to it being played".to_string(),
        ],
        test_card::TEST_FORESEE_ONE => vec![
            "<b>Foresee 1:</b> Look at the top card of your deck. You may put it into your void."
                .to_string(),
        ],
        test_card::TEST_PREVENT_DISSOLVE_THIS_TURN => {
            vec!["<b>Anchored:</b> Cannot be dissolved.".to_string()]
        }
        test_card::TEST_FORESEE_ONE_DRAW_RECLAIM => vec![
            "<b>Foresee 1:</b> Look at the top card of your deck. You may put it into your void."
                .to_string(),
            "<b>Reclaim:</b> You may play this card from your void, then banish it.".to_string(),
        ],
        test_card::TEST_COUNTERSPELL_CHARACTER => vec![
            "<b>Prevent:</b> Send a character to the void in response to it being played"
                .to_string(),
        ],
        _ => vec![],
    }
}

pub fn build_info_zoom_data(battle: &BattleState, card_id: CardId) -> Option<InfoZoomData> {
    let targeting_icons = get_targeting_icons(battle, card_id);
    let supplemental_texts = supplemental_card_info(battle, card_id);

    let supplemental_info = if supplemental_texts.is_empty() {
        None
    } else {
        let supplemental_components: Vec<_> = supplemental_texts
            .into_iter()
            .filter_map(|text| {
                SupplementalCardInfo::builder().text(text).build().render()?.flex_node()
            })
            .collect();

        if supplemental_components.is_empty() {
            None
        } else {
            BoxComponent::builder()
                .name("Supplemental Card Info Column")
                .style(FlexStyle::builder().flex_direction(FlexDirection::Column).margin(4).build())
                .children(supplemental_components)
                .build()
                .flex_node()
        }
    };

    if targeting_icons.is_empty() && supplemental_info.is_none() {
        None
    } else {
        Some(InfoZoomData { supplemental_card_info: supplemental_info, icons: targeting_icons })
    }
}

fn get_targeting_icons(battle: &BattleState, card_id: CardId) -> Vec<InfoZoomIcon> {
    let mut icons = HashMap::new();
    let current_player = card_properties::controller(battle, card_id);

    if let Some(prompt) = battle.prompts.front()
        && prompt.source.card_id() == Some(card_id)
        && let PromptType::Choose { choices } = &prompt.prompt_type
    {
        for choice in choices {
            if let Some(targets) = &choice.targets {
                let target_card_ids = match targets {
                    EffectTargets::Standard(StandardEffectTarget::Character(card_object_id)) => {
                        vec![card_object_id.card_id.card_id()]
                    }
                    EffectTargets::Standard(StandardEffectTarget::StackCard(card_object_id)) => {
                        vec![card_object_id.card_id.card_id()]
                    }
                    EffectTargets::Standard(StandardEffectTarget::VoidCardSet(
                        void_card_targets,
                    )) => void_card_targets.iter().map(|target| target.card_id.card_id()).collect(),
                    EffectTargets::EffectList(target_list) => target_list
                        .iter()
                        .flat_map(|target_option| match target_option.as_ref() {
                            Some(StandardEffectTarget::Character(card_object_id)) => {
                                vec![card_object_id.card_id.card_id()]
                            }
                            Some(StandardEffectTarget::StackCard(card_object_id)) => {
                                vec![card_object_id.card_id.card_id()]
                            }
                            Some(StandardEffectTarget::VoidCardSet(void_card_targets)) => {
                                void_card_targets
                                    .iter()
                                    .map(|target| target.card_id.card_id())
                                    .collect()
                            }
                            None => vec![],
                        })
                        .collect(),
                };

                for target_card_id in target_card_ids {
                    icons.insert(target_card_id, InfoZoomIcon {
                        card_id: adapter::client_card_id(target_card_id),
                        icon: icon::CHEVRON_UP.to_string(),
                        color: targeting_color(battle, current_player, target_card_id),
                    });
                }
            }
        }
    }

    if let Some(targets) = valid_target_queries::displayed_targets(battle, StackCardId(card_id)) {
        let target_card_ids = match targets {
            EffectTargets::Standard(StandardEffectTarget::Character(card_object_id)) => {
                vec![card_object_id.card_id.card_id()]
            }
            EffectTargets::Standard(StandardEffectTarget::StackCard(card_object_id)) => {
                vec![card_object_id.card_id.card_id()]
            }
            EffectTargets::Standard(StandardEffectTarget::VoidCardSet(void_card_targets)) => {
                void_card_targets.iter().map(|target| target.card_id.card_id()).collect()
            }
            EffectTargets::EffectList(target_list) => target_list
                .iter()
                .flat_map(|target_option| match target_option.as_ref() {
                    Some(StandardEffectTarget::Character(card_object_id)) => {
                        vec![card_object_id.card_id.card_id()]
                    }
                    Some(StandardEffectTarget::StackCard(card_object_id)) => {
                        vec![card_object_id.card_id.card_id()]
                    }
                    Some(StandardEffectTarget::VoidCardSet(void_card_targets)) => {
                        void_card_targets.iter().map(|target| target.card_id.card_id()).collect()
                    }
                    None => vec![],
                })
                .collect(),
        };

        for target_card_id in target_card_ids {
            icons.insert(target_card_id, InfoZoomIcon {
                card_id: adapter::client_card_id(target_card_id),
                icon: icon::CHEVRON_UP.to_string(),
                color: targeting_color(battle, current_player, target_card_id),
            });
        }
    } else if let Some(stack_card) = battle.cards.stack_item(StackCardId(card_id))
        && stack_card.targets.is_some()
        && valid_target_queries::valid_targets(battle, stack_card.targets.as_ref()).is_none()
    {
        icons.insert(card_id, InfoZoomIcon {
            card_id: adapter::client_card_id(card_id),
            icon: icon::XMARK.to_string(),
            color: display_color::RED_500,
        });
    }

    icons.into_values().collect()
}
