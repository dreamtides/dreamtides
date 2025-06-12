use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use bon::Builder;
use masonry::flex_enums::{FlexAlign, FlexDirection, FlexJustify, TextAlign, WhiteSpace};
use masonry::flex_style::FlexStyle;
use ui_components::box_component::BoxComponent;
use ui_components::component::Component;
use ui_components::panel_component::PanelComponent;
use ui_components::scroll_view_component::ScrollViewComponent;
use ui_components::text_component::TextComponent;
use ui_components::typography::Typography;

#[derive(Clone, Builder)]
pub struct ViewLogsPanel {
    pub log_directory: Option<PathBuf>,
}

impl Component for ViewLogsPanel {
    fn render(self) -> Option<impl Component> {
        let log_content = self.read_log_content();

        Some(
            PanelComponent::builder()
                .title("View Logs")
                .content(
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
                                        .padding((8, 8, 8, 8))
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
    }
}

impl ViewLogsPanel {
    fn read_log_content(&self) -> String {
        let log_path = match &self.log_directory {
            Some(dir) => dir.join("dreamtides.log"),
            None => return "No log directory available.".to_string(),
        };

        if !log_path.exists() {
            return "Log file does not exist.".to_string();
        }

        match self.read_last_lines(&log_path, 1000) {
            Ok(content) => {
                if content.is_empty() {
                    "Log file is empty.".to_string()
                } else {
                    content
                }
            }
            Err(err) => format!("Error reading log file: {}", err),
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
