use std::fs;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tempfile::NamedTempFile;
use tmux_interface::{LoadBuffer, PasteBuffer, SendKeys, Tmux};
const LARGE_MESSAGE_THRESHOLD: usize = 1024;
/// Handles reliable message sending to TMUX sessions with debouncing
#[derive(Clone)]
pub struct TmuxSender {
    /// Base delay in milliseconds (default: 500ms)
    debounce_base_ms: u32,
    /// Additional delay per KB (default: 100ms)
    debounce_per_kb_ms: u32,
    /// Maximum debounce delay cap (default: 2000ms)
    max_debounce_ms: u32,
    /// Number of Enter retry attempts (default: 3)
    enter_retry_count: u32,
    /// Delay between Enter retries (default: 200ms)
    enter_retry_delay_ms: u32,
}
impl Default for TmuxSender {
    fn default() -> Self {
        Self::new()
    }
}
impl TmuxSender {
    /// Creates a new TmuxSender with default timing parameters
    pub fn new() -> Self {
        Self {
            debounce_base_ms: 500,
            debounce_per_kb_ms: 100,
            max_debounce_ms: 2000,
            enter_retry_count: 3,
            enter_retry_delay_ms: 200,
        }
    }

    /// Sends a message to a TMUX session with appropriate debouncing
    pub fn send(&self, session: &str, message: &str) -> Result<()> {
        let start = std::time::Instant::now();
        let send_type = if message.len() >= LARGE_MESSAGE_THRESHOLD { "large" } else { "small" };
        let truncated = if message.len() > 200 {
            format!("{}...", &message[..200])
        } else {
            message.to_string()
        };
        let result = if message.len() >= LARGE_MESSAGE_THRESHOLD {
            self.send_large_message(session, message)
        } else {
            self.send_small_message(session, message)
        };
        let duration_ms = start.elapsed().as_millis() as u64;
        let debounce_ms = self.calculate_delay(message.len()).as_millis() as u64;
        match &result {
            Ok(_) => {
                tracing::debug!(
                    operation = "tmux_send",
                    session_id = session,
                    send_type,
                    message_size_bytes = message.len(),
                    message_truncated = truncated,
                    debounce_ms,
                    duration_ms,
                    result = "success",
                    "TMUX send succeeded"
                );
            }
            Err(e) => {
                tracing::error!(
                    operation = "tmux_send", session_id = session, send_type,
                    message_size_bytes = message.len(), message_truncated = truncated,
                    debounce_ms, duration_ms, result = "error", error = % e,
                    "TMUX send failed"
                );
            }
        }
        result
    }

    /// Sends raw keys to a TMUX session without debouncing
    pub fn send_keys_raw(&self, session: &str, keys: &str) -> Result<()> {
        Tmux::with_command(SendKeys::new().target_pane(session).key(keys))
            .output()
            .with_context(|| format!("Failed to send keys to session '{}'", session))?;
        Ok(())
    }

    /// Sends a large message (>=1KB) using TMUX load-buffer method
    pub fn send_large_message(&self, session: &str, message: &str) -> Result<()> {
        let tmp = NamedTempFile::new().context("Failed to create temporary file")?;
        fs::write(tmp.path(), message).context("Failed to write message to temp file")?;
        Tmux::with_command(
            LoadBuffer::new().buffer_name("prompt").path(tmp.path().to_string_lossy().as_ref()),
        )
        .output()
        .with_context(|| format!("Failed to load buffer in session '{}'", session))?;
        Tmux::with_command(PasteBuffer::new().target_pane(session).buffer_name("prompt"))
            .output()
            .with_context(|| format!("Failed to paste buffer in session '{}'", session))?;
        let delay = self.calculate_delay(message.len());
        sleep(delay);
        self.send_enter_with_retry(session)
    }

    fn send_small_message(&self, session: &str, message: &str) -> Result<()> {
        Tmux::with_command(SendKeys::new().target_pane(session).disable_lookup().key(message))
            .output()
            .with_context(|| format!("Failed to send message to session '{}'", session))?;
        let delay = self.calculate_delay(message.len());
        sleep(delay);
        self.send_enter_with_retry(session)
    }

    fn send_enter_with_retry(&self, session: &str) -> Result<()> {
        for attempt in 0..self.enter_retry_count {
            if attempt > 0 {
                sleep(Duration::from_millis(self.enter_retry_delay_ms as u64));
            }
            if Tmux::with_command(SendKeys::new().target_pane(session).key("Enter"))
                .output()
                .is_ok()
            {
                return Ok(());
            }
        }
        bail!(
            "Failed to send Enter after {} attempts to session '{}'",
            self.enter_retry_count,
            session
        )
    }

    fn calculate_delay(&self, message_len: usize) -> Duration {
        let kb_count = message_len.saturating_div(1024) as u32;
        let delay = self.debounce_base_ms + kb_count * self.debounce_per_kb_ms;
        Duration::from_millis(delay.min(self.max_debounce_ms) as u64)
    }
}
#[cfg(test)]
mod tests {
    use crate::llmc::tmux::sender::*;
    #[test]
    fn test_default_sender() {
        let sender = TmuxSender::new();
        assert_eq!(sender.debounce_base_ms, 500);
        assert_eq!(sender.debounce_per_kb_ms, 100);
        assert_eq!(sender.max_debounce_ms, 2000);
        assert_eq!(sender.enter_retry_count, 3);
        assert_eq!(sender.enter_retry_delay_ms, 200);
    }
    #[test]
    fn test_calculate_delay() {
        let sender = TmuxSender::new();
        assert_eq!(sender.calculate_delay(0), Duration::from_millis(500));
        assert_eq!(sender.calculate_delay(512), Duration::from_millis(500));
        assert_eq!(sender.calculate_delay(1024), Duration::from_millis(600));
        assert_eq!(sender.calculate_delay(2048), Duration::from_millis(700));
        assert_eq!(sender.calculate_delay(10240), Duration::from_millis(1500));
        assert_eq!(sender.calculate_delay(102400), Duration::from_millis(2000));
    }
    #[test]
    fn test_large_message_threshold() {
        let _sender = TmuxSender::new();
        assert!(999 < LARGE_MESSAGE_THRESHOLD);
        assert!(1024 >= LARGE_MESSAGE_THRESHOLD);
    }
}
