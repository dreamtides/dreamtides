#![allow(dead_code)]

use std::fs;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tempfile::NamedTempFile;
use tmux_interface::{LoadBuffer, PasteBuffer, SendKeys, Tmux};

use super::session;

const LARGE_MESSAGE_THRESHOLD: usize = 1024;

/// Status of partial send detection
#[derive(Debug, Clone, PartialEq)]
pub enum PartialSendStatus {
    /// No input detected in the input line
    NoInput,
    /// Partial message received (received bytes, expected bytes)
    Partial { received: usize, expected: usize },
    /// Complete message received
    Complete,
    /// Cannot determine status
    Unknown,
}

/// Handles reliable message sending to TMUX sessions with debouncing
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

    /// Creates a TmuxSender with custom timing parameters
    pub fn with_timing(
        debounce_base_ms: u32,
        debounce_per_kb_ms: u32,
        max_debounce_ms: u32,
        enter_retry_count: u32,
        enter_retry_delay_ms: u32,
    ) -> Self {
        Self {
            debounce_base_ms,
            debounce_per_kb_ms,
            max_debounce_ms,
            enter_retry_count,
            enter_retry_delay_ms,
        }
    }

    /// Sends a message to a TMUX session with appropriate debouncing
    pub fn send(&self, session: &str, message: &str) -> Result<()> {
        if message.len() >= LARGE_MESSAGE_THRESHOLD {
            self.send_large_message(session, message)
        } else {
            self.send_small_message(session, message)
        }
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

    /// Clears the input line by sending Ctrl-U
    pub fn clear_input_line(&self, session: &str) -> Result<()> {
        self.send_keys_raw(session, "C-u")
    }

    /// Detects if a partial send occurred
    pub fn detect_partial_send(&self, session: &str, expected: &str) -> PartialSendStatus {
        let Ok(output) = session::capture_pane(session, 5) else {
            return PartialSendStatus::Unknown;
        };

        let lines: Vec<&str> = output.lines().collect();
        let last_line = lines.last().map(|s| s.trim()).unwrap_or("");

        if last_line.is_empty() {
            return PartialSendStatus::NoInput;
        }

        if last_line == expected {
            return PartialSendStatus::Complete;
        }

        if expected.starts_with(last_line) {
            return PartialSendStatus::Partial {
                received: last_line.len(),
                expected: expected.len(),
            };
        }

        PartialSendStatus::Unknown
    }

    /// Clears the input line and retries sending a message
    pub fn clear_and_retry(&self, session: &str, message: &str) -> Result<()> {
        self.clear_input_line(session)?;
        sleep(Duration::from_millis(100));
        self.send(session, message)
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
    use super::*;

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
    fn test_custom_timing() {
        let sender = TmuxSender::with_timing(100, 50, 1000, 5, 100);
        assert_eq!(sender.debounce_base_ms, 100);
        assert_eq!(sender.debounce_per_kb_ms, 50);
        assert_eq!(sender.max_debounce_ms, 1000);
        assert_eq!(sender.enter_retry_count, 5);
        assert_eq!(sender.enter_retry_delay_ms, 100);
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
    fn test_partial_send_status() {
        let sender = TmuxSender::new();

        let status = PartialSendStatus::Partial { received: 50, expected: 100 };
        assert!(matches!(status, PartialSendStatus::Partial { .. }));

        let status = PartialSendStatus::NoInput;
        assert!(matches!(status, PartialSendStatus::NoInput));

        let status = PartialSendStatus::Complete;
        assert!(matches!(status, PartialSendStatus::Complete));

        let status = PartialSendStatus::Unknown;
        assert!(matches!(status, PartialSendStatus::Unknown));
    }

    #[test]
    fn test_large_message_threshold() {
        let sender = TmuxSender::new();

        assert!(999 < LARGE_MESSAGE_THRESHOLD);
        assert!(1024 >= LARGE_MESSAGE_THRESHOLD);
    }
}
