use action_data::game_action::GameAction;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::flex_style::FlexStyle;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ScrollBarVisibility {
    Auto,
    AlwaysVisible,
    Hidden,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TouchScrollBehavior {
    Unrestricted,
    Elastic,
    Clamped,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SliderDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScrollBar {
    pub style: Option<FlexStyle>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScrollViewNode {
    pub elasticity: Option<f32>,
    pub horizontal_page_size: Option<f32>,
    pub horizontal_scroll_bar: Option<ScrollBar>,
    pub horizontal_scroll_bar_visibility: Option<ScrollBarVisibility>,
    pub scroll_deceleration_rate: Option<f32>,
    pub touch_scroll_behavior: Option<TouchScrollBehavior>,
    pub vertical_page_size: Option<f32>,
    pub vertical_scroll_bar: Option<ScrollBar>,
    pub vertical_scroll_bar_visibility: Option<ScrollBarVisibility>,
    pub mouse_wheel_scroll_size: Option<f32>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DraggableNode {
    pub drop_target_identifiers: Vec<String>,
    pub over_target_indicator: Option<Box<FlexNode>>,
    pub on_drop: Option<GameAction>,
    pub horizontal_drag_start_distance: Option<u32>,
    pub remove_original: Option<bool>,
    pub hide_indicator_children: Vec<String>,
    pub custom_drag_indicator: Option<Box<FlexNode>>,
    pub on_drag_detected: Option<GameAction>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TextFieldNode {
    pub global_identifier: Option<String>,
    pub initial_text: Option<String>,
    pub multiline: Option<bool>,
    pub is_read_only: Option<bool>,
    pub max_length: Option<u32>,
    pub is_password_field: Option<bool>,
    pub double_click_selects_word: Option<bool>,
    pub triple_click_selects_line: Option<bool>,
    pub mask_character: Option<String>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SliderNode {
    pub initial_value: Option<f32>,
    pub label: Option<String>,
    pub preference_key: Option<String>,
    pub direction: Option<SliderDirection>,
    pub high_value: Option<f32>,
    pub low_value: Option<f32>,
    pub inverted: Option<bool>,
    pub page_size: Option<f32>,
    pub show_input_field: Option<bool>,
    pub label_style: Option<FlexStyle>,
    pub drag_container_style: Option<FlexStyle>,
    pub tracker_style: Option<FlexStyle>,
    pub dragger_style: Option<FlexStyle>,
    pub dragger_border_style: Option<FlexStyle>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum NodeType {
    Text(TextNode),
    ScrollViewNode(Box<ScrollViewNode>),
    DraggableNode(DraggableNode),
    TextFieldNode(TextFieldNode),
    SliderNode(Box<SliderNode>),
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventHandlers {
    pub on_click: Option<GameAction>,
    pub on_long_press: Option<GameAction>,
    pub on_mouse_enter: Option<GameAction>,
    pub on_mouse_leave: Option<GameAction>,
    pub on_mouse_down: Option<GameAction>,
    pub on_mouse_up: Option<GameAction>,
    pub on_field_changed: Option<GameAction>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FlexNode {
    pub name: Option<String>,
    pub node_type: Option<NodeType>,
    pub children: Vec<FlexNode>,
    pub event_handlers: Option<EventHandlers>,
    pub style: Option<FlexStyle>,
    pub hover_style: Option<FlexStyle>,
    pub pressed_style: Option<FlexStyle>,
    pub on_attach_style: Option<FlexStyle>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TextNode {
    pub label: String,
}
