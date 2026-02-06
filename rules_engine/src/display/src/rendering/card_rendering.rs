use std::collections::HashMap;

use ability_data::ability::Ability;
use ability_data::effect::{Effect, ModelEffectChoiceIndex};
use ability_data::variable_value::VariableValue;
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
use fluent::types::FluentNumber;
use fluent::{FluentArgs, FluentValue, fluent_args};
use masonry::flex_enums::FlexDirection;
use masonry::flex_style::FlexStyle;
use parser_v2::serializer::ability_serializer;
use parser_v2::variables::parser_bindings::VariableBindings;
use rlf::Value;
use strings::strings;
use tabula_data::card_definition::CardDefinition;
use tabula_data::fluent_loader::StringContext;
use tabula_data::tabula_error::TabulaError;
use tabula_generated::string_id::StringId;
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

pub fn card_image(battle: &BattleState, card_id: CardId) -> SpriteAddress {
    card::get_definition(battle, card_id).image.clone()
}

pub fn card_name(battle: &BattleState, card_id: CardId) -> String {
    card::get_definition(battle, card_id).displayed_name.clone()
}

/// Converts [VariableBindings] to Fluent [FluentArgs] for string formatting.
pub fn to_fluent_args(bindings: &VariableBindings) -> FluentArgs<'static> {
    let mut args = FluentArgs::new();
    for (name, value) in bindings.iter() {
        let fluent_value: FluentValue<'static> = match value {
            VariableValue::Integer(n) => FluentValue::Number(FluentNumber::from(*n as f64)),
            VariableValue::Subtype(subtype) => FluentValue::String(subtype.to_string().into()),
            VariableValue::Figment(figment) => FluentValue::String(figment.to_string().into()),
        };
        args.set(name.clone(), fluent_value);
    }
    args
}

/// Converts [VariableBindings] to RLF parameters for eval_str formatting.
pub fn to_rlf_params(bindings: &VariableBindings) -> HashMap<String, Value> {
    let mut params = HashMap::new();
    for (name, value) in bindings.iter() {
        let rlf_value = match value {
            VariableValue::Integer(n) => Value::Number(*n as i64),
            VariableValue::Subtype(subtype) => rlf::with_locale(|locale| {
                Value::Phrase(
                    locale.get_phrase(&subtype.to_string()).expect("subtype phrase should exist"),
                )
            }),
            VariableValue::Figment(figment) => rlf::with_locale(|locale| {
                Value::Phrase(
                    locale.get_phrase(&figment.to_string()).expect("figment phrase should exist"),
                )
            }),
        };
        params.insert(name.clone(), rlf_value);
    }
    params
}

/// Evaluates a template string with RLF variable bindings.
///
/// Rewrites `{phrase(arg)}` to `{phrase:_p_arg}` selector syntax so that
/// variant phrases correctly select based on the argument's plural category.
pub fn eval_str(template: &str, bindings: &VariableBindings) -> String {
    strings::register_source_phrases();
    let mut params = to_rlf_params(bindings);
    let rewritten = rewrite_phrase_calls_to_selectors(template, &mut params);
    rlf::with_locale(|locale| {
        locale
            .eval_str(&rewritten, params)
            .unwrap_or_else(|e| panic!("Error evaluating template {template:?}: {e}"))
            .to_string()
    })
}

/// Returns the rules text for the given ability, without including any costs.
pub fn ability_token_text(
    builder: &ResponseBuilder,
    definition: &CardDefinition,
    ability_number: AbilityNumber,
) -> String {
    let ability = &definition.abilities[ability_number.0];
    let serialized = ability_serializer::serialize_ability_effect(ability);
    let args = to_fluent_args(&serialized.variables);
    builder
        .tabula()
        .strings
        .format_display_string(&serialized.text, StringContext::CardText, args)
        .unwrap_or_default()
}

pub fn rules_text(builder: &ResponseBuilder, battle: &BattleState, card_id: CardId) -> String {
    let definition = card::get_definition(battle, card_id);
    let mut formatted =
        serialize_abilities_text(builder, &definition.abilities).unwrap_or_default();

    if let Some(stack_item) = battle.cards.stack_item(StackCardId(card_id))
        && let Some(ModelEffectChoiceIndex(index)) = stack_item.modal_choice
    {
        formatted = modal_effect_prompt_rendering::modal_effect_descriptions(&definition.abilities)
            [index]
            .clone();
    }

    if let Some(stack_item) = battle.cards.stack_item(StackCardId(card_id))
        && let StackCardAdditionalCostsPaid::Energy(energy) = &stack_item.additional_costs_paid
    {
        return format!(
            "{} <b><color=\"blue\">{}</color></b>",
            formatted,
            builder
                .string_with_args(StringId::CardRulesTextEnergyPaid, fluent_args!("e" => energy.0))
        );
    }

    if is_on_stack_from_void(battle, card_id) {
        return format!(
            "{formatted} <b><color=\"blue\">{}</color></b>",
            builder.string(StringId::CardRulesTextReclaimed)
        );
    }

    if apply_card_fx::is_anchored(battle, card_id) {
        return format!(
            "{formatted} <b><color=\"blue\">{}</color></b>",
            builder.string(StringId::CardRulesTextAnchored)
        );
    }

    formatted
}

/// Builds info zoom data for a card including targeting icons and
/// supplemental help text.
pub fn build_info_zoom_data(battle: &BattleState, card_id: CardId) -> Option<InfoZoomData> {
    let targeting_icons = get_targeting_icons(battle, card_id);
    let supplemental_texts = ability_help_text::help_texts(battle, card_id);

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

/// Rewrites RLF function call syntax `{phrase(arg)}` to selector syntax
/// `{phrase:_p_arg}` to enable proper variant selection. Also handles
/// transforms like `{@cap @a phrase(arg)}`.
///
/// Adds prefixed parameter copies (`_p_arg`) to the params map so that
/// the prefixed selector references resolve correctly without shadowing
/// phrase names.
fn rewrite_phrase_calls_to_selectors(
    template: &str,
    params: &mut HashMap<String, Value>,
) -> String {
    let original = std::mem::take(params);
    let mut sorted_keys: Vec<_> = original.keys().collect();
    sorted_keys.sort();
    for k in sorted_keys {
        params.insert(sanitize_param_name(k), original[k].clone());
    }

    let mut result = String::with_capacity(template.len());
    let mut chars = template.char_indices().peekable();

    while let Some((i, ch)) = chars.next() {
        if ch == '{'
            && let Some(close_pos) = template[i..].find('}')
        {
            let close_idx = i + close_pos;
            let content = &template[i + 1..close_idx];

            if let Some(rewritten) = rewrite_interpolation(content) {
                result.push('{');
                result.push_str(&rewritten);
                result.push('}');
                while chars.peek().is_some_and(|&(j, _)| j < close_idx) {
                    chars.next();
                }
                chars.next();
                continue;
            }
        }
        result.push(ch);
    }

    result
}

/// Rewrites a single interpolation content if it contains a function call.
///
/// Converts `phrase(arg)` to `phrase:_p_arg` and handles optional
/// leading transforms like `@cap @a`.
fn rewrite_interpolation(content: &str) -> Option<String> {
    let trimmed = content.trim();
    let (transforms_prefix, rest) = extract_transforms(trimmed);

    if let Some(paren_start) = rest.find('(')
        && let Some(paren_end) = rest.find(')')
    {
        let phrase_name = rest[..paren_start].trim();
        let args_str = &rest[paren_start + 1..paren_end];
        let suffix = &rest[paren_end + 1..];
        let args: Vec<&str> = args_str.split(',').map(str::trim).collect();

        let prefixed_args: Vec<String> = args.iter().map(|arg| sanitize_param_name(arg)).collect();
        let selector_suffix: String = prefixed_args.iter().map(|a| format!(":{a}")).collect();

        return Some(format!("{transforms_prefix}{phrase_name}{selector_suffix}{suffix}"));
    }

    None
}

/// Converts a parameter name to a prefixed, sanitized RLF identifier.
///
/// Adds `_p_` prefix and replaces hyphens with underscores since RLF
/// identifiers do not support hyphens.
fn sanitize_param_name(name: &str) -> String {
    format!("_p_{}", name.replace('-', "_"))
}

/// Extracts leading `@transform` prefixes from an interpolation, returning
/// the prefix string and the remaining content.
fn extract_transforms(content: &str) -> (String, &str) {
    let mut prefix = String::new();
    let mut rest = content;

    while let Some(stripped) = rest.strip_prefix('@') {
        if let Some(space_pos) = stripped.find(' ') {
            prefix.push('@');
            prefix.push_str(&stripped[..space_pos + 1]);
            rest = &stripped[space_pos + 1..];
        } else {
            break;
        }
    }

    (prefix, rest)
}

/// Serializes abilities using the ability serializer and formats with Fluent.
fn serialize_abilities_text(
    builder: &ResponseBuilder,
    abilities: &[Ability],
) -> Result<String, TabulaError> {
    // Tags must use Fluent "quoted string" syntax to avoid breaking parsing.
    let line_height_25 = "{\"<line-height=25%>\"}";
    let end_line_height = "{\"</line-height>\"}";

    let formatted: Result<Vec<_>, _> = abilities
        .iter()
        .map(|ability| {
            let serialized = ability_serializer::serialize_ability(ability);
            let args = to_fluent_args(&serialized.variables);
            builder.tabula().strings.format_display_string(
                &serialized.text,
                StringContext::CardText,
                args,
            )
        })
        .collect();

    Ok(formatted?.join(&format!("\n{line_height_25}\n{end_line_height}")))
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
        Some(builder.string(StringId::AsteriskIcon))
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

fn card_type(builder: &ResponseBuilder, battle: &BattleState, card_id: CardId) -> String {
    let definition = card::get_definition(battle, card_id);
    let result = if let Some(subtype) = definition.card_subtype {
        let subtype_key = match subtype {
            CardSubtype::Agent => "agent",
            CardSubtype::Ancient => "ancient",
            CardSubtype::Avatar => "avatar",
            CardSubtype::Child => "child",
            CardSubtype::Detective => "detective",
            CardSubtype::Enigma => "enigma",
            CardSubtype::Explorer => "explorer",
            CardSubtype::Guide => "guide",
            CardSubtype::Hacker => "hacker",
            CardSubtype::Mage => "mage",
            CardSubtype::Monster => "monster",
            CardSubtype::Musician => "musician",
            CardSubtype::Outsider => "outsider",
            CardSubtype::Renegade => "renegade",
            CardSubtype::Robot => "robot",
            CardSubtype::SpiritAnimal => "spirit-animal",
            CardSubtype::Super => "super",
            CardSubtype::Survivor => "survivor",
            CardSubtype::Synth => "synth",
            CardSubtype::Tinkerer => "tinkerer",
            CardSubtype::Trooper => "trooper",
            CardSubtype::Visionary => "visionary",
            CardSubtype::Visitor => "visitor",
            CardSubtype::Warrior => "warrior",
        };
        builder.string_with_args(StringId::Subtype, fluent_args!["subtype" => subtype_key])
    } else {
        let type_string = match definition.card_type {
            CardType::Character => StringId::CardTypeCharacter,
            CardType::Event => StringId::CardTypeEvent,
            CardType::Dreamsign => StringId::CardTypeDreamsign,
            CardType::Dreamcaller => StringId::CardTypeDreamcaller,
            CardType::Dreamwell => StringId::CardTypeDreamwell,
        };
        builder.string(type_string)
    };

    if card_properties::is_fast(battle, card_id) { format!("\u{f0e7} {result}") } else { result }
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
