use anyhow::Result;

use super::super::listener_runner::{Listener, ListenerContext, ListenerResult};
use super::super::server_workbook_snapshot::WorkbookSnapshot;

pub struct ConditionalFormattingListener;

impl Listener for ConditionalFormattingListener {
    fn name(&self) -> &str {
        "conditional_formatting"
    }

    fn run(
        &self,
        _snapshot: &WorkbookSnapshot,
        _context: &ListenerContext,
    ) -> Result<ListenerResult> {
        Ok(ListenerResult { changes: vec![], warnings: vec![] })
    }
}
