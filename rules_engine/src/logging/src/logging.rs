use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Once, OnceLock};

use battle_state::battle::battle_state::RequestContext;
use tracing::{Event, Level};
use tracing_error::ErrorLayer;
use tracing_forest::{ForestLayer, PrettyPrinter, Tag};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

pub const LOG_FILTER_EMOJIS: &[&str] = &["🚨", "🚧", "🌐", "🤖", "🟢"];

static INIT: Once = Once::new();
static LOG_FILE_PATH: OnceLock<PathBuf> = OnceLock::new();
static TRACE_JSON_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Initializes global logging behavior for the 'tracing' crate if it hasn't
/// already been initialized.
pub fn maybe_initialize(request_context: &RequestContext) {
    INIT.call_once(|| {
        initialize(request_context);
    });
}

/// Initializes global logging behavior for the 'tracing' crate.
fn initialize(request_context: &RequestContext) {
    let env_filter =
        env::var("RUST_LOG").map(EnvFilter::new).unwrap_or_else(|_| EnvFilter::new("debug"));

    match request_context.logging_options.log_directory.as_ref() {
        Some(log_directory) => {
            // Set up dual output to stdout and file
            let log_path = log_directory.join("dreamtides.log");
            println!("Logging to: {}", log_path.display());
            LOG_FILE_PATH.set(log_path.clone()).expect("Failed to set log file path");
            TRACE_JSON_PATH
                .set(log_directory.join("dreamtides.json"))
                .expect("Failed to set log JSON path");
            let log_file = File::create(log_path).expect("Error creating tracing log file");

            let dual_writer = DualMakeWriter::new(log_file);
            let printer = PrettyPrinter::new().writer(dual_writer);

            let forest_layer = ForestLayer::new(printer, tag_parser).with_filter(env_filter);

            tracing_subscriber::registry().with(forest_layer).with(ErrorLayer::default()).init();
        }
        None => {
            // Stdout only
            let forest_layer =
                ForestLayer::new(PrettyPrinter::new(), tag_parser).with_filter(env_filter);

            tracing_subscriber::registry().with(forest_layer).with(ErrorLayer::default()).init();
        }
    }
}

pub fn trace_json_path() -> Option<&'static PathBuf> {
    TRACE_JSON_PATH.get()
}

pub fn log_file_path() -> Option<&'static PathBuf> {
    LOG_FILE_PATH.get()
}

/// Returns a ForestLayer configured with the given EnvFilter
pub fn create_forest_layer<S>(env_filter: EnvFilter) -> impl Layer<S> + Send + Sync
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    ForestLayer::new(PrettyPrinter::new(), tag_parser).with_filter(env_filter)
}

/// Returns the path to the developer mode project directory.
pub fn get_developer_mode_project_directory() -> Result<PathBuf, String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let manifest_path = PathBuf::from(manifest_dir);

    let workspace_root = manifest_path
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "Failed to find workspace root directory".to_string())?;

    let project_root = workspace_root
        .parent()
        .ok_or_else(|| "Failed to find project root directory".to_string())?;

    Ok(project_root.to_path_buf())
}

/// Returns the path to the developer mode streaming assets directory.
pub fn get_developer_mode_streaming_assets_path() -> String {
    get_developer_mode_project_directory()
        .expect("Failed to get project directory")
        .join("client/Assets/StreamingAssets")
        .canonicalize()
        .expect("Failed to canonicalize path")
        .to_string_lossy()
        .to_string()
}

fn tag_parser(event: &Event) -> Option<Tag> {
    let target = event.metadata().target();
    let level = *event.metadata().level();

    let icon = match (level, target) {
        (Level::ERROR, _) => '🚨',
        (Level::WARN, _) => '🚧',
        (_, target) if target.contains("macros") => '🟢',
        (_, target) if target.contains("battle_queries") => '🔎',
        (_, target) if target.contains("battle_mutations") => '💻',
        (_, target) if target.contains("client_logging") => '🌐',
        (_, target) if target.contains("rules_engine") => '📏',
        (_, target) if target.contains("ai") => '🤖',
        (Level::TRACE, _) => '📍',
        (Level::DEBUG, _) => '📝',
        _ => '💡',
    };

    let (prefix, suffix) =
        if target.contains("client_logging") { ("", "") } else { (target, "rs") };
    Some(Tag::builder().level(level).icon(icon).prefix(prefix).suffix(suffix).build())
}

/// A `MakeWriter` implementation that creates writers for both stdout and a
/// file. This allows tracing-forest to write formatted logs to multiple
/// destinations simultaneously.
#[derive(Clone)]
struct DualMakeWriter {
    file: Arc<Mutex<File>>,
}

impl DualMakeWriter {
    fn new(file: File) -> Self {
        Self { file: Arc::new(Mutex::new(file)) }
    }
}

impl<'a> MakeWriter<'a> for DualMakeWriter {
    type Writer = DualWriter;

    fn make_writer(&'a self) -> Self::Writer {
        DualWriter { file: self.file.clone() }
    }
}

/// A writer that writes to both stdout and a file simultaneously.
struct DualWriter {
    file: Arc<Mutex<File>>,
}

impl Write for DualWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Write to stdout first
        let bytes_written = io::stdout().write(buf)?;
        io::stdout().flush()?;

        if let Ok(mut file) = self.file.lock() {
            // Ignore file write errors to ensure stdout logging continues
            let _ = file.write_all(buf);
            let _ = file.flush();
        }

        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()?;

        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }

        Ok(())
    }
}
