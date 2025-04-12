use battle_data::battle::battle_data::BattleData;
use core_data::display_color::DisplayColor;
use display::core::response_builder::ResponseBuilder;
use display::rendering::battle_rendering;
use display_data::command::{Command, CommandSequence, UpdateBattleCommand};
use masonry::flex_enums::{FlexPosition, TextAlign, WhiteSpace};
use masonry::flex_node::{FlexNode, NodeType, Text};
use masonry::flex_style::{
    BorderRadius, Dimension, DimensionGroup, DimensionUnit, FlexInsets, FlexStyle,
};

/// Attempts to display an error message to the player describing a rules engine
/// error.
pub fn display_error_message(battle: &BattleData, message: String) -> CommandSequence {
    let mut builder = ResponseBuilder { animate: false, commands: CommandSequence::default() };
    let mut view = battle_rendering::battle_view(&builder, battle);
    view.interface.screen_overlay = Some(render_message(message));
    builder.push(Command::UpdateBattle(UpdateBattleCommand { battle: view, update_sound: None }));
    builder.commands
}

fn render_message(text: String) -> FlexNode {
    let style = FlexStyle {
        background_color: Some(DisplayColor { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.95 }),
        border_radius: Some(BorderRadius {
            top_left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            top_right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom_right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom_left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
        }),
        padding: Some(DimensionGroup {
            top: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
        }),
        color: Some(DisplayColor { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 }),
        font_size: Some(Dimension { unit: DimensionUnit::Pixels, value: 6.0 }),
        min_height: Some(Dimension { unit: DimensionUnit::Pixels, value: 22.0 }),
        white_space: Some(WhiteSpace::Normal),
        text_align: Some(TextAlign::MiddleLeft),
        ..Default::default()
    };

    let message = FlexNode {
        node_type: Some(NodeType::Text(Text { label: text.into(), ..Default::default() })),
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
