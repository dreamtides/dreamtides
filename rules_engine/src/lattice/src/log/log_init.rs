use std::path::Path;
use std::str::FromStr;

use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::log::tracing_layer::JsonlLayer;

/// Verbosity level for stderr output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// No stderr output except errors.
    #[default]
    Quiet,
    /// Normal output (warnings and above).
    Normal,
    /// Verbose output (info and above).
    Verbose,
    /// Debug output (debug and above).
    Debug,
    /// Trace output (all messages).
    Trace,
}

/// Configuration for initializing the logging system.
pub struct LogConfig {
    /// Path to the `.lattice` directory for JSONL file output.
    /// If None, file logging is disabled.
    pub lattice_dir: Option<Box<Path>>,

    /// Verbosity level for stderr output.
    pub verbosity: Verbosity,

    /// Override log level via `LATTICE_LOG_LEVEL` environment variable.
    pub respect_env_filter: bool,
}

/// Initializes the logging system with the given configuration.
///
/// Sets up:
/// - JSONL file logging to `.lattice/logs.jsonl` (if lattice_dir is provided)
/// - Optional stderr output based on verbosity level
/// - Environment variable override via `LATTICE_LOG_LEVEL`
///
/// This function should be called once at program startup. Subsequent calls
/// will have no effect (tracing subscriber can only be set once).
pub fn init_logging(config: LogConfig) {
    let stderr_level = config.verbosity.to_level_filter();

    // Build the base filter
    let env_filter = if config.respect_env_filter {
        EnvFilter::try_from_env("LATTICE_LOG_LEVEL")
            .unwrap_or_else(|_| EnvFilter::new(stderr_level.to_string()))
    } else {
        EnvFilter::new(stderr_level.to_string())
    };

    // Create stderr layer for human-readable output
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_span_events(FmtSpan::NONE)
        .with_writer(std::io::stderr);

    // Build and install the subscriber
    if let Some(ref lattice_dir) = config.lattice_dir {
        let jsonl_layer = JsonlLayer::new(lattice_dir);
        tracing_subscriber::registry()
            .with(env_filter)
            .with(stderr_layer)
            .with(jsonl_layer)
            .try_init()
            .ok();
    } else {
        tracing_subscriber::registry().with(env_filter).with(stderr_layer).try_init().ok();
    }
}

/// Initializes logging for tests with no file output.
///
/// Only enables stderr output at the specified level.
pub fn init_test_logging(verbosity: Verbosity) {
    let config = LogConfig::default().with_verbosity(verbosity).without_env_filter();
    init_logging(config);
}

impl Verbosity {
    /// Converts verbosity to a tracing level filter.
    fn to_level_filter(self) -> Level {
        match self {
            Verbosity::Quiet => Level::ERROR,
            Verbosity::Normal => Level::WARN,
            Verbosity::Verbose => Level::INFO,
            Verbosity::Debug => Level::DEBUG,
            Verbosity::Trace => Level::TRACE,
        }
    }
}

impl FromStr for Verbosity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "quiet" | "q" => Ok(Verbosity::Quiet),
            "normal" | "n" => Ok(Verbosity::Normal),
            "verbose" | "v" => Ok(Verbosity::Verbose),
            "debug" | "d" => Ok(Verbosity::Debug),
            "trace" | "t" => Ok(Verbosity::Trace),
            _ => Err(format!("Unknown verbosity level: {s}")),
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self { lattice_dir: None, verbosity: Verbosity::Quiet, respect_env_filter: true }
    }
}

impl LogConfig {
    /// Creates a configuration with JSONL file logging to the given directory.
    pub fn with_lattice_dir(lattice_dir: &Path) -> Self {
        Self { lattice_dir: Some(lattice_dir.into()), ..Default::default() }
    }

    /// Sets the verbosity level for stderr output.
    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Disables environment variable override.
    pub fn without_env_filter(mut self) -> Self {
        self.respect_env_filter = false;
        self
    }
}
