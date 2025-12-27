use std::collections::HashMap;

use ability_data::ability::{Ability, DisplayedAbility, DisplayedAbilityEffect};
use ability_data::effect::{Effect, ModelEffectChoiceIndex};
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
use core_data::identifiers::AbilityNumber;
use core_data::types::{CardFacing, PlayerName};
use display_data::card_view::{
    CardActions, CardPrefab, CardView, DisplayImage, InfoZoomData, InfoZoomIcon, RevealedCardView,
};
use fluent::fluent_args;
use masonry::flex_enums::FlexDirection;
use masonry::flex_style::FlexStyle;
use tabula_data::card_definitions::card_definition::CardDefinition;
use tabula_data::localized_strings::StringContext;
use tabula_ids::string_id;
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
    ability_help_text, apply_card_fx, card_display_state, modal_effect_prompt_rendering, positions,
};

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

    // Get the cost to display
    let cost = if let Some(stack_item) = battle.cards.stack_item(StackCardId(card_id))
        && let Some(ModelEffectChoiceIndex(index)) = stack_item.modal_choice
    {
        selected_modal_energy_cost(battle, card_id, index).map(|e| e.to_string())
    } else if card_properties::base_energy_cost(battle, card_id).is_some() {
        Some(card_properties::converted_energy_cost(battle, card_id).to_string())
    } else {
        Some(builder.string(string_id::ASTERISK_ICON))
    };

    RevealedCardView {
        image: DisplayImage::Sprite(card_image(battle, card_id)),
        name: card_name(battle, card_id),
        cost,
        produced: None,
        spark: card_properties::spark(battle, controller, CharacterId(card_id))
            .or_else(|| card_properties::base_spark(battle, card_id))
            .map(|spark| spark.to_string()),
        card_type: card_type(builder, battle, card_id),
        rules_text: rules_text(builder, battle, card_id),
        outline_color: match selection_color {
            Some(color) => Some(color),
            None if can_play => Some(display_color::GREEN),
            _ => None,
        },
        info_zoom_data: build_info_zoom_data(builder, battle, card_id),
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
    current_player: PlayerName,
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

    if let Some(prompt) = battle.prompts.front()
        && let PromptType::ChooseHandCards(choose_hand_prompt) = &prompt.prompt_type
    {
        let hand_card_id = HandCardId(card_id);
        let select = BattleAction::SelectHandCardTarget(hand_card_id);
        let selection_action = legal_actions
            .contains(select, ForPlayer::Human)
            .then_some(GameAction::BattleAction(select));

        if choose_hand_prompt.selected.contains(hand_card_id) {
            return (Some(display_color::YELLOW_500), selection_action);
        } else if choose_hand_prompt.valid.contains(hand_card_id) {
            return (Some(display_color::WHITE), selection_action);
        }
    }

    (None, None)
}

fn can_select_order_action(legal_actions: &LegalActions, card_id: CardId) -> Option<CardId> {
    if let LegalActions::SelectDeckCardOrder { .. } = legal_actions { Some(card_id) } else { None }
}

pub fn card_image(battle: &BattleState, card_id: CardId) -> SpriteAddress {
    card::get_definition(battle, card_id).image.clone()
}

pub fn card_name(battle: &BattleState, card_id: CardId) -> String {
    card::get_definition(battle, card_id).displayed_name.clone()
}

fn card_type(builder: &ResponseBuilder, battle: &BattleState, card_id: CardId) -> String {
    let definition = card::get_definition(battle, card_id);
    let type_string = if let Some(subtype) = definition.card_subtype {
        match subtype {
            CardSubtype::Ancient => string_id::CARD_SUBTYPE_ANCIENT,
            CardSubtype::Child => string_id::CARD_SUBTYPE_CHILD,
            CardSubtype::Detective => string_id::CARD_SUBTYPE_DETECTIVE,
            CardSubtype::Explorer => string_id::CARD_SUBTYPE_EXPLORER,
            CardSubtype::Hacker => string_id::CARD_SUBTYPE_HACKER,
            CardSubtype::Mage => string_id::CARD_SUBTYPE_MAGE,
            CardSubtype::Monster => string_id::CARD_SUBTYPE_MONSTER,
            CardSubtype::Musician => string_id::CARD_SUBTYPE_MUSICIAN,
            CardSubtype::Outsider => string_id::CARD_SUBTYPE_OUTSIDER,
            CardSubtype::Renegade => string_id::CARD_SUBTYPE_RENEGADE,
            CardSubtype::SpiritAnimal => string_id::CARD_SUBTYPE_SPIRIT_ANIMAL,
            CardSubtype::Super => string_id::CARD_SUBTYPE_SUPER,
            CardSubtype::Survivor => string_id::CARD_SUBTYPE_SURVIVOR,
            CardSubtype::Synth => string_id::CARD_SUBTYPE_SYNTH,
            CardSubtype::Tinkerer => string_id::CARD_SUBTYPE_TINKERER,
            CardSubtype::Trooper => string_id::CARD_SUBTYPE_TROOPER,
            CardSubtype::Visionary => string_id::CARD_SUBTYPE_VISIONARY,
            CardSubtype::Visitor => string_id::CARD_SUBTYPE_VISITOR,
            CardSubtype::Warrior => string_id::CARD_SUBTYPE_WARRIOR,
            CardSubtype::Enigma => string_id::CARD_SUBTYPE_ENIGMA,
        }
    } else {
        match definition.card_type {
            CardType::Character => string_id::CARD_TYPE_CHARACTER,
            CardType::Event => string_id::CARD_TYPE_EVENT,
            CardType::Dreamsign => string_id::CARD_TYPE_DREAMSIGN,
            CardType::Dreamcaller => string_id::CARD_TYPE_DREAMCALLER,
            CardType::Dreamwell => string_id::CARD_TYPE_DREAMWELL,
        }
    };

    let result = builder.string(type_string);
    if card_properties::is_fast(battle, card_id) { format!("\u{f0e7} {result}") } else { result }
}

/// Returns the rules text for the given ability, without including any costs.
pub fn ability_token_text(
    builder: &ResponseBuilder,
    definition: &CardDefinition,
    ability_number: AbilityNumber,
) -> String {
    let ability = &definition.displayed_abilities[ability_number.0];
    let text = match ability {
        DisplayedAbility::Event { event } => displayed_effect_text(&event.effect),
        DisplayedAbility::Static { text } => text.clone(),
        DisplayedAbility::Activated { effect, .. } => displayed_effect_text(effect),
        DisplayedAbility::Triggered { text } => text.clone(),
        DisplayedAbility::Named { name } => name.clone(),
    };
    builder.tabula().strings.format_display_string(&text, StringContext::CardText, fluent_args![])
}

pub fn rules_text(builder: &ResponseBuilder, battle: &BattleState, card_id: CardId) -> String {
    let definition = card::get_definition(battle, card_id);
    let mut formatted = builder.tabula().strings.format_display_string(
        &get_displayed_text(&definition.displayed_abilities),
        StringContext::CardText,
        fluent_args![],
    );

    if let Some(stack_item) = battle.cards.stack_item(StackCardId(card_id))
        && let Some(ModelEffectChoiceIndex(index)) = stack_item.modal_choice
    {
        formatted = modal_effect_prompt_rendering::modal_effect_descriptions(
            builder,
            &definition.displayed_abilities,
        )[index]
            .clone();
    }

    if let Some(stack_item) = battle.cards.stack_item(StackCardId(card_id))
        && let StackCardAdditionalCostsPaid::Energy(energy) = &stack_item.additional_costs_paid
    {
        return format!(
            "{} <b><color=\"blue\">{}</color></b>",
            formatted,
            builder.string_with_args(
                string_id::CARD_RULES_TEXT_ENERGY_PAID,
                fluent_args!("energy" => energy.0)
            )
        );
    }

    if is_on_stack_from_void(battle, card_id) {
        return format!(
            "{formatted} <b><color=\"blue\">{}</color></b>",
            builder.string(string_id::CARD_RULES_TEXT_RECLAIMED)
        );
    }

    if apply_card_fx::is_anchored(battle, card_id) {
        return format!(
            "{formatted} <b><color=\"blue\">{}</color></b>",
            builder.string(string_id::CARD_RULES_TEXT_ANCHORED)
        );
    }

    formatted
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
                    BattleAction::PlayCardFromVoid(void_card_id)
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

pub fn build_info_zoom_data(
    builder: &ResponseBuilder,
    battle: &BattleState,
    card_id: CardId,
) -> Option<InfoZoomData> {
    let targeting_icons = get_targeting_icons(battle, card_id);
    let supplemental_texts = ability_help_text::help_texts(builder, battle, card_id);

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

pub fn get_displayed_text(abilities: &[DisplayedAbility]) -> String {
    // Tags must use Fluent "quoted string" syntax to avoid breaking parsing.
    let colon = "{\"<size=130%>:</size>\"}";
    let line_height_25 = "{\"<line-height=25%>\"}";
    let end_line_height = "{\"</line-height>\"}";

    abilities
        .iter()
        .map(|ability| match ability {
            DisplayedAbility::Event { event } => {
                let effect = displayed_effect_text(&event.effect);
                if let Some(additional_cost) = &event.additional_cost {
                    format!("{additional_cost}{colon} {effect}")
                } else {
                    effect
                }
            }
            DisplayedAbility::Static { text } => text.clone(),
            DisplayedAbility::Activated { cost, effect } => {
                let effect_text = displayed_effect_text(effect);
                format!("{cost}{colon} {effect_text}")
            }
            DisplayedAbility::Triggered { text } => text.clone(),
            DisplayedAbility::Named { name } => name.clone(),
        })
        .collect::<Vec<_>>()
        .join(&format!("\n{line_height_25}\n{end_line_height}"))
}

fn displayed_effect_text(effect: &DisplayedAbilityEffect) -> String {
    let colon = "{\"<size=130%>:</size>\"}";
    let indent = "{\"<indent=0.75em>\"}";
    let end_indent = "{\"</indent>\"}";
    let bullet = "{bullet}";

    match effect {
        DisplayedAbilityEffect::Effect(text) => text.clone(),
        DisplayedAbilityEffect::Modal(choices) => {
            let lines = choices
                .iter()
                .map(|c| {
                    if c.cost.is_empty() {
                        format!("{bullet} {indent}{}{end_indent}", c.effect,)
                    } else {
                        format!("{bullet} {indent}{}{colon} {}{end_indent}", c.cost, c.effect)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("{choose}\n{}", lines, choose = "{choose-one}")
        }
    }
}

/// Returns the appropriate targeting color based on card ownership
fn targeting_color(
    battle: &BattleState,
    current_player: PlayerName,
    target_card_id: CardId,
) -> DisplayColor {
    let target_controller = card_properties::controller(battle, target_card_id);
    if target_controller == current_player {
        display_color::GREEN_500
    } else {
        display_color::RED_500
    }
}

/// Extracts all modal effect choices from a card's displayed abilities
fn selected_modal_energy_cost(battle: &BattleState, card_id: CardId, index: usize) -> Option<u32> {
    let definition = card::get_definition(battle, card_id);

    let mut current_index = 0usize;

    for ability in &definition.abilities {
        match ability {
            Ability::Event(event) => {
                if let Effect::Modal(choices) = &event.effect {
                    if index < current_index + choices.len() {
                        let local_index = index - current_index;
                        if let Some(choice) = choices.get(local_index) {
                            return Some(choice.energy_cost.0);
                        }
                    }
                    current_index += choices.len();
                }
            }
            Ability::Activated(activated) => {
                if let Effect::Modal(choices) = &activated.effect {
                    if index < current_index + choices.len() {
                        let local_index = index - current_index;
                        if let Some(choice) = choices.get(local_index) {
                            return Some(choice.energy_cost.0);
                        }
                    }
                    current_index += choices.len();
                }
            }
            _ => {}
        }
    }

    None
}
