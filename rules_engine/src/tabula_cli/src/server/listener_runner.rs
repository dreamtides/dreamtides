use std::collections::BTreeMap;

use anyhow::Result;

use crate::tabula_cli::server::model::{Change, ChangedRange};
use crate::tabula_cli::server::server_workbook_snapshot::WorkbookSnapshot;
pub trait Listener: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, snapshot: &WorkbookSnapshot, context: &ListenerContext)
    -> Result<ListenerResult>;
}
pub struct ListenerContext {
    pub request_id: String,
    pub workbook_path: String,
    pub changed_range: Option<ChangedRange>,
}
pub struct ListenerResult {
    pub changes: Vec<Change>,
    pub warnings: Vec<String>,
}
pub fn run_listeners(
    listeners: &[Box<dyn Listener>],
    snapshot: &WorkbookSnapshot,
    context: &ListenerContext,
) -> ListenerResult {
    let mut all_changes = Vec::new();
    let mut all_warnings = Vec::new();
    for listener in listeners {
        match listener.run(snapshot, context) {
            Ok(result) => {
                all_changes.extend(result.changes);
                all_warnings.extend(result.warnings);
            }
            Err(e) => {
                all_warnings.push(format!("Listener '{}' failed: {}", listener.name(), e));
            }
        }
    }
    let (resolved_changes, conflict_warnings) = resolve_conflicts(&all_changes);
    all_warnings.extend(conflict_warnings);
    ListenerResult { changes: resolved_changes, warnings: all_warnings }
}
fn resolve_conflicts(changes: &[Change]) -> (Vec<Change>, Vec<String>) {
    let mut value_changes = BTreeMap::new();
    let mut formatting_changes: Vec<Change> = Vec::new();
    let mut warnings = Vec::new();
    for change in changes {
        match change {
            Change::SetValue { sheet, cell, .. } | Change::ClearValue { sheet, cell } => {
                let key = (sheet.clone(), cell.clone());
                if let Some(existing) = value_changes.insert(key, change.clone()) {
                    warnings.push(format!(
                        "Conflicting value changes for {sheet}!{cell}: {existing:?} vs {change:?}"
                    ));
                }
            }
            _ => formatting_changes.push(change.clone()),
        }
    }
    let mut resolved = Vec::new();
    resolved.extend(value_changes.into_values());
    resolved.extend(formatting_changes);
    (resolved, warnings)
}
