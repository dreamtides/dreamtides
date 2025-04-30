use battle_data::battle::battle_data::BattleData;
use core_data::display_color::{self};
use core_data::types::PlayerName;
use display::core::response_builder::ResponseBuilder;
use display::rendering::battle_rendering;
use display_data::command::{Command, CommandSequence, UpdateBattleCommand};
use masonry::dimension::{Dimension, DimensionUnit, FlexInsets};
use masonry::flex_enums::{FlexPosition, TextAlign, WhiteSpace};
use masonry::flex_node::{FlexNode, NodeType, TextNode};
use masonry::flex_style::FlexStyle;

/// Attempts to display an error message to the player describing a rules engine
/// error.
pub fn display_error_message(battle: &BattleData, message: String) -> CommandSequence {
    let mut builder = ResponseBuilder::new(PlayerName::One, false);
    let mut view = battle_rendering::battle_view(&builder, battle);
    view.interface.screen_overlay = Some(render_message(message));
    builder.push(Command::UpdateBattle(UpdateBattleCommand { battle: view, update_sound: None }));
    builder.commands()
}

fn render_message(text: String) -> FlexNode {
    let style = FlexStyle::builder()
        .background_color(display_color::BLACK_ALPHA_95)
        .border_radius(4)
        .padding(4)
        .color(display_color::WHITE)
        .font_size(6)
        .min_height(22)
        .white_space(WhiteSpace::Normal)
        .text_align(TextAlign::MiddleLeft)
        .build();

    let message = FlexNode {
        node_type: Some(NodeType::Text(TextNode { label: text })),
        style: Some(style),
        ..Default::default()
    };

    FlexNode {
        style: Some(FlexStyle {
            position: Some(FlexPosition::Absolute),
            inset: Some(FlexInsets {
                top: Some(Dimension { unit: DimensionUnit::Pixels, value: 50.0 }),
                right: Some(Dimension { unit: DimensionUnit::Pixels, value: 8.0 }),
                bottom: None,
                left: Some(Dimension { unit: DimensionUnit::Pixels, value: 8.0 }),
            }),
            ..Default::default()
        }),
        children: vec![message],
        ..Default::default()
    }
}
