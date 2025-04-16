use std::env;
use std::fs::File;
use std::path::PathBuf;

use tracing::{Event, Level};
use tracing_error::ErrorLayer;
use tracing_forest::{ForestLayer, PrettyPrinter, Tag};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

/// Initializes global logging behavior for the 'tracing' crate.
pub fn initialize() {
    let env_filter =
        if let Ok(v) = env::var("RUST_LOG") { EnvFilter::new(v) } else { EnvFilter::new("debug") };
    let forest_layer = ForestLayer::new(PrettyPrinter::new(), tag_parser).with_filter(env_filter);

    let log_path = PathBuf::from("..").join("dreamtides.log");
    let log_file = File::create(log_path).expect("Error creating log file");
    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(EnvFilter::new("debug"));

    tracing_subscriber::registry()
        .with(forest_layer)
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();
}

fn tag_parser(event: &Event) -> Option<Tag> {
    let target = event.metadata().target();
    let level = *event.metadata().level();
    let icon = match target {
        _ if level == Level::ERROR => '🚨',
        _ if level == Level::WARN => '🚧',
        _ if target.contains("battle_queries") => '🔎',
        _ if target.contains("battle_mutations") => '💻',
        _ if target.contains("rules_engine") => '🌐',
        _ if target.contains("ai") => '🤖',
        _ => match level {
            Level::TRACE => '📍',
            Level::DEBUG => '📝',
            _ => '💡',
        },
    };

    let mut builder = Tag::builder().level(level).icon(icon);
    builder = builder.prefix(target).suffix("rs");

    Some(builder.build())
}
