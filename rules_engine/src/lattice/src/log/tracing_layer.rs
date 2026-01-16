use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use tracing::field::{Field, Visit};
use tracing::span::Attributes;
use tracing::{Event, Id, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

use crate::log::jsonl_writer::JsonlWriter;
use crate::log::log_entry::{LogEntry, LogLevel, OperationCategory};

/// A tracing layer that writes events to a JSONL file.
///
/// Converts tracing events into [`LogEntry`] records and writes them via
/// [`JsonlWriter`]. Extracts operation category from event fields or defaults
/// to `Observation`.
pub struct JsonlLayer {
    writer: Arc<JsonlWriter>,
}

impl JsonlLayer {
    /// Creates a new layer that writes to the given lattice directory.
    pub fn new(lattice_dir: &Path) -> Self {
        Self { writer: Arc::new(JsonlWriter::new(lattice_dir)) }
    }

    /// Creates a new layer with a specific log file path.
    pub fn with_path(log_path: impl Into<std::path::PathBuf>) -> Self {
        Self { writer: Arc::new(JsonlWriter::with_path(log_path.into())) }
    }

    /// Returns a reference to the underlying writer.
    pub fn writer(&self) -> &JsonlWriter {
        &self.writer
    }
}

impl<S> Layer<S> for JsonlLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = EventVisitor::new();
        event.record(&mut visitor);

        let level = LogLevel::from(*event.metadata().level());
        let category = visitor.category.unwrap_or(OperationCategory::Observation);
        let message = visitor.message.unwrap_or_else(|| event.metadata().name().to_string());

        let mut entry = LogEntry::new(level, category, message);

        if let Some(duration_us) = visitor.duration_us {
            entry.duration_us = Some(duration_us);
        }

        entry.details = visitor.details;

        self.writer.write(&entry);
    }

    fn on_new_span(&self, _attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
        // Spans are not written to the log file - only events
    }
}

/// Visitor that extracts fields from tracing events.
struct EventVisitor {
    message: Option<String>,
    category: Option<OperationCategory>,
    duration_us: Option<u64>,
    details: HashMap<String, String>,
}

impl EventVisitor {
    fn new() -> Self {
        Self { message: None, category: None, duration_us: None, details: HashMap::new() }
    }
}

impl Visit for EventVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let value_str = format!("{value:?}");
        match field.name() {
            "message" => self.message = Some(value_str.trim_matches('"').to_string()),
            "category" => self.category = parse_category(&value_str),
            "duration_us" => {
                self.duration_us = value_str.trim_matches('"').parse().ok();
            }
            _ => {
                self.details.insert(field.name().to_string(), value_str);
            }
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "message" => self.message = Some(value.to_string()),
            "category" => self.category = parse_category(value),
            _ => {
                self.details.insert(field.name().to_string(), value.to_string());
            }
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        if field.name() == "duration_us" {
            self.duration_us = Some(value as u64);
        } else {
            self.details.insert(field.name().to_string(), value.to_string());
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == "duration_us" {
            self.duration_us = Some(value);
        } else {
            self.details.insert(field.name().to_string(), value.to_string());
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.details.insert(field.name().to_string(), value.to_string());
    }
}

/// Parses an operation category from a string value.
fn parse_category(value: &str) -> Option<OperationCategory> {
    let normalized = value.trim_matches('"').to_lowercase();
    match normalized.as_str() {
        "git" => Some(OperationCategory::Git),
        "sqlite" => Some(OperationCategory::Sqlite),
        "file_io" | "fileio" => Some(OperationCategory::FileIo),
        "index" => Some(OperationCategory::Index),
        "command" => Some(OperationCategory::Command),
        "observation" => Some(OperationCategory::Observation),
        _ => None,
    }
}
