use std::process::Command;

use anyhow::{Context, Result};

/// Send a macOS notification via osascript.
pub fn send_notification(title: &str, message: &str) -> Result<()> {
    let script = format!("display notification \"{message}\" with title \"{title}\"");

    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .with_context(|| "Failed to execute osascript")?;

    Ok(())
}
