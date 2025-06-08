use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Once;

use battle_state::battle::battle_state::RequestContext;
use tracing::{Event, Level};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_error::ErrorLayer;
use tracing_forest::{ForestLayer, PrettyPrinter, Tag};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

static INIT: Once = Once::new();

/// Initializes global logging behavior for the 'tracing' crate if it hasn't
/// already been initialized.
pub fn maybe_initialize(request_context: &RequestContext) {
    INIT.call_once(|| {
        initialize(request_context);
    });
}

/// Initializes global logging behavior for the 'tracing' crate.
pub fn initialize(request_context: &RequestContext) {
    let env_filter =
        if let Ok(v) = env::var("RUST_LOG") { EnvFilter::new(v) } else { EnvFilter::new("debug") };
    let forest_layer = create_forest_layer(env_filter);

    let file_subscriber =
        if let Some(log_directory) = request_context.logging_options.log_directory.as_ref() {
            let log_path = log_directory.join("dreamtides.log");
            let log_file = File::create(log_path).expect("Error creating tracing log file");
            Some(
                BunyanFormattingLayer::new("dreamcaller".into(), log_file)
                    .with_filter(EnvFilter::new("debug")),
            )
        } else {
            None
        };

    tracing_subscriber::registry()
        .with(forest_layer)
        .with(JsonStorageLayer)
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();
}

/// Returns a ForestLayer configured with the given EnvFilter
pub fn create_forest_layer<S>(env_filter: EnvFilter) -> impl Layer<S> + Send + Sync
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    ForestLayer::new(PrettyPrinter::new(), tag_parser).with_filter(env_filter)
}

/// Returns the path to the developer mode log file directory.
pub fn get_developer_mode_log_directory() -> Result<PathBuf, String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // Go up two levels from tracing_macros crate to workspace root
    let manifest_path = PathBuf::from(manifest_dir);
    let parent = manifest_path
        .parent()
        .ok_or_else(|| "Failed to find parent directory of manifest".to_string())?;
    let workspace_root =
        parent.parent().ok_or_else(|| "Failed to find workspace root directory".to_string())?;
    let project_root = workspace_root
        .parent()
        .ok_or_else(|| "Failed to find project root directory".to_string())?;
    Ok(project_root.to_path_buf())
}

fn tag_parser(event: &Event) -> Option<Tag> {
    let target = event.metadata().target();
    let level = *event.metadata().level();
    let icon = match target {
        _ if level == Level::ERROR => '🚨',
        _ if level == Level::WARN => '🚧',
        _ if target.contains("battle_queries_old") => '🔎',
        _ if target.contains("battle_mutations_old") => '💻',
        _ if target.contains("rules_engine") => '🌐',
        _ if target.contains("ai") => '🤖',
        _ if target.contains("tracing_macros") => '🟢',
        _ => match level {
            Level::TRACE => '📍',
            Level::DEBUG => '📝',
            _ => '💡',
        },
    };

    Some(Tag::builder().level(level).icon(icon).prefix(target).suffix("rs").build())
}
