use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use action_data::battle_display_action::BattleDisplayAction;
use action_data::game_action_data::GameAction;
use action_data::panel_address::PanelAddress;
use bon::Builder;
use logging::LOG_FILTER_EMOJIS;
use masonry::flex_enums::{
    FlexAlign, FlexDirection, FlexJustify, FlexVisibility, FlexWrap, TextAlign, WhiteSpace,
};
use masonry::flex_style::FlexStyle;
use ui_components::box_component::BoxComponent;
use ui_components::button_component::ButtonComponent;
use ui_components::component::Component;
use ui_components::panel_component::PanelComponent;
use ui_components::scroll_view_component::ScrollViewComponent;
use ui_components::text_component::TextComponent;
use ui_components::typography::Typography;

#[derive(Clone, Builder)]
pub struct ViewLogsPanel {
    pub log_file_path: Option<PathBuf>,
    pub filter: Option<String>,
}

#[derive(Clone, Builder)]
pub struct LogFilterButtons {}

#[derive(Clone, Builder)]
pub struct FilterButton {
    #[builder(into)]
    pub emoji: String,
    pub emoji_filter: Option<String>,
}

impl Component for ViewLogsPanel {
    fn render(self) -> Option<impl Component> {
        let log_content = self.read_log_content();

        Some(
            PanelComponent::builder()
                .title("View Logs")
                .content(
                    BoxComponent::builder()
                        .name("Logs Container")
                        .style(
                            FlexStyle::builder()
                                .align_items(FlexAlign::Stretch)
                                .flex_direction(FlexDirection::Column)
                                .flex_grow(1)
                                .build(),
                        )
                        .child(LogFilterButtons::builder().build())
                        .child(
                            ScrollViewComponent::builder()
                                .child(
                                    BoxComponent::builder()
                                        .name("Log Content")
                                        .style(
                                            FlexStyle::builder()
                                                .align_items(FlexAlign::Stretch)
                                                .flex_direction(FlexDirection::Column)
                                                .flex_grow(1)
                                                .justify_content(FlexJustify::FlexStart)
                                                .padding((8, 0, 0, 0))
                                                .build(),
                                        )
                                        .child(
                                            TextComponent::builder()
                                                .text(log_content)
                                                .typography(Typography::StackTrace)
                                                .text_align(TextAlign::UpperLeft)
                                                .white_space(WhiteSpace::Normal)
                                                .build(),
                                        )
                                        .build(),
                                )
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
    }
}

impl ViewLogsPanel {
    fn read_log_content(&self) -> String {
        let Some(log_file) = self.log_file_path.as_ref() else {
            return "Log file path not initialized".to_string();
        };

        if !log_file.exists() {
            return "Log file does not exist.".to_string();
        }

        match self.read_last_lines(log_file, 1000) {
            Ok(content) => {
                if content.is_empty() {
                    "Log file is empty.".to_string()
                } else {
                    self.apply_filter(content)
                }
            }
            Err(err) => format!("Error reading log file: {err}"),
        }
    }

    fn apply_filter(&self, content: String) -> String {
        match &self.filter {
            Some(filter_str) => {
                let filter = filter_str.as_str();
                let lines = content.lines().filter(|line| !line.contains("OpenPanel(ViewLogs("));
                if filter == "🔮" {
                    lines
                        .filter(|line| {
                            line.contains("🔮") || line.contains("👿") || line.contains("🌟")
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    lines.filter(|line| line.contains(filter)).collect::<Vec<_>>().join("\n")
                }
            }
            None => content,
        }
    }

    fn read_last_lines(&self, path: &PathBuf, max_lines: usize) -> Result<String, std::io::Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut lines = Vec::new();
        let mut line = String::new();

        while reader.read_line(&mut line)? > 0 {
            lines.push(line.trim_end().to_string());
            line.clear();
        }

        let start_index = if lines.len() > max_lines { lines.len() - max_lines } else { 0 };

        Ok(lines[start_index..].join("\n"))
    }
}

impl Component for LogFilterButtons {
    fn render(self) -> Option<impl Component> {
        let mut buttons = BoxComponent::builder()
            .name("Filter Buttons")
            .style(
                FlexStyle::builder()
                    .align_items(FlexAlign::Center)
                    .flex_direction(FlexDirection::Row)
                    .justify_content(FlexJustify::Center)
                    .wrap(FlexWrap::NoWrap)
                    .overflow(FlexVisibility::Hidden)
                    .padding((8, 8, 8, 8))
                    .build(),
            )
            .child(FilterButton::builder().emoji("All".to_string()).build());

        buttons = buttons.child(
            FilterButton::builder().emoji("🔮".to_string()).emoji_filter("🔮".to_string()).build(),
        );

        for &emoji in LOG_FILTER_EMOJIS {
            buttons = buttons.child(
                FilterButton::builder()
                    .emoji(emoji.to_string())
                    .emoji_filter(emoji.to_string())
                    .build(),
            );
        }

        Some(buttons.build())
    }
}

impl Component for FilterButton {
    fn render(self) -> Option<impl Component> {
        Some(
            BoxComponent::builder()
                .name(format!("{} Filter Button", self.emoji))
                .style(FlexStyle::builder().margin(2).width(20).height(20).build())
                .child(
                    ButtonComponent::builder()
                        .label(self.emoji)
                        .action(GameAction::BattleDisplayAction(BattleDisplayAction::OpenPanel(
                            PanelAddress::ViewLogs(self.emoji_filter),
                        )))
                        .build(),
                )
                .build(),
        )
    }
}
