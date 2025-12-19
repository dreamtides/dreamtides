use anyhow::Result;

use super::super::listener_runner::{Listener, ListenerContext, ListenerResult};
use super::super::server_workbook_snapshot::WorkbookSnapshot;

pub struct EnsureUuidListener;

impl Listener for EnsureUuidListener {
    fn name(&self) -> &str {
        "ensure_uuid"
    }

    fn run(
        &self,
        _snapshot: &WorkbookSnapshot,
        _context: &ListenerContext,
    ) -> Result<ListenerResult> {
        Ok(ListenerResult { changes: vec![], warnings: vec![] })
    }
}
