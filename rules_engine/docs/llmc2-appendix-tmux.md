# LLMC v2 Appendix: TMUX Integration Details

This appendix contains detailed implementation notes for TMUX session management
and reliable communication protocols.

## Session Management

LLMC uses TMUX for persistent Claude Code sessions. The `tmux_interface` crate
provides Rust bindings, but we implement our own higher-level abstractions for
reliability.

### Session Naming Convention

```
llmc-<worker>
```

Examples: `llmc-adam`, `llmc-baker`, `llmc-charlie`

### Session Startup Sequence

```
1. Create detached TMUX session with worker's worktree as cwd
2. Set environment variables (LLMC_WORKER, LLMC_ROOT)
3. Wait for shell to be ready
4. Send Claude startup command with configured flags
5. Wait for Claude to initialize (poll for ">" prompt)
6. Accept bypass permissions warning if shown
7. Send /clear command
8. Mark worker as idle
```

## Reliable Communication Protocol

**This is the most critical component of the system.** Claude Code sessions can
be in various states, and sending input at the wrong time leads to lost or
corrupted messages.

### Input Debouncing

TMUX `send-keys` can have race conditions where the Enter key arrives before
the text is fully pasted. We use a multi-phase approach:

```rust
pub struct TmuxSender {
    debounce_base_ms: u32,      // Base delay: 500ms
    debounce_per_kb_ms: u32,    // Additional per KB: 100ms
    max_debounce_ms: u32,       // Cap: 2000ms
    enter_retry_count: u32,     // Retry attempts: 3
    enter_retry_delay_ms: u32,  // Between retries: 200ms
}

impl TmuxSender {
    pub fn send(&self, session: &str, message: &str) -> Result<()> {
        // 1. Send text in literal mode
        tmux::send_keys(session, "-l", message)?;

        // 2. Calculate debounce delay based on message size
        let delay = self.debounce_base_ms +
            (message.len() as u32 / 1024) * self.debounce_per_kb_ms;
        let delay = delay.min(self.max_debounce_ms);
        sleep(Duration::from_millis(delay as u64));

        // 3. Send Enter with retry logic
        for attempt in 0..self.enter_retry_count {
            if attempt > 0 {
                sleep(Duration::from_millis(self.enter_retry_delay_ms as u64));
            }
            if tmux::send_keys(session, "Enter").is_ok() {
                return Ok(());
            }
        }

        Err(anyhow!("Failed to send Enter after {} attempts", self.enter_retry_count))
    }
}
```

### State Detection

Detecting when a worker is ready for input or has completed a task requires
parsing terminal output. We use a polling-based approach with multiple
indicators:

```rust
pub enum ClaudeState {
    /// Claude is ready for input (shows "> " prompt)
    Ready,
    /// Claude is processing (no prompt visible)
    Processing,
    /// Claude is waiting for a yes/no confirmation
    AwaitingConfirmation,
    /// Claude has crashed or exited
    Crashed,
    /// State cannot be determined
    Unknown,
}

impl StateDetector {
    pub fn detect(&self, session: &str) -> Result<ClaudeState> {
        let output = self.tmux.capture_pane(session, 20)?;
        let lines: Vec<&str> = output.lines().collect();

        // Check for crash indicators
        if self.has_crash_indicators(&output) {
            return Ok(ClaudeState::Crashed);
        }

        // Check for confirmation prompt
        if self.is_awaiting_confirmation(&output) {
            return Ok(ClaudeState::AwaitingConfirmation);
        }

        // Check for ready prompt
        if self.is_ready_for_input(&lines) {
            return Ok(ClaudeState::Ready);
        }

        // Check if Claude process is running
        let cmd = self.tmux.get_pane_command(session)?;
        if !self.is_claude_command(&cmd) {
            return Ok(ClaudeState::Crashed);
        }

        Ok(ClaudeState::Processing)
    }

    fn is_ready_for_input(&self, lines: &[&str]) -> bool {
        // Claude shows "> " or ">" at start of line when ready
        lines.iter().any(|line| {
            let trimmed = line.trim();
            trimmed == ">" || trimmed.starts_with("> ")
        })
    }

    fn is_claude_command(&self, cmd: &str) -> bool {
        // Claude reports as "node", "claude", or version like "2.0.76"
        matches!(cmd, "node" | "claude") ||
            regex::Regex::new(r"^\d+\.\d+\.\d+").unwrap().is_match(cmd)
    }
}
```

### Output Monitoring

To detect task completion, we monitor the terminal output for specific patterns:

```rust
pub struct OutputMonitor {
    poll_interval: Duration,
    commit_pattern: Regex,      // Detects "committed" or similar
    error_pattern: Regex,       // Detects error messages
    question_pattern: Regex,    // Detects Claude asking a question
}

impl OutputMonitor {
    /// Returns when a significant event is detected
    pub fn wait_for_event(&self, session: &str) -> Result<OutputEvent> {
        loop {
            let state = self.state_detector.detect(session)?;

            match state {
                ClaudeState::Ready => {
                    let output = self.capture_recent_output(session)?;
                    return self.classify_completion(&output);
                }
                ClaudeState::Crashed => {
                    return Ok(OutputEvent::Crashed);
                }
                ClaudeState::AwaitingConfirmation => {
                    return Ok(OutputEvent::NeedsConfirmation);
                }
                _ => {
                    sleep(self.poll_interval);
                }
            }
        }
    }

    fn classify_completion(&self, output: &str) -> Result<OutputEvent> {
        // Check if there's a new commit
        if let Some(sha) = self.detect_new_commit()? {
            return Ok(OutputEvent::Committed(sha));
        }

        // Check if Claude is asking a question
        if self.question_pattern.is_match(output) {
            return Ok(OutputEvent::AskingQuestion);
        }

        // Default: task completed without commit
        Ok(OutputEvent::CompletedNoCommit)
    }
}

pub enum OutputEvent {
    Committed(String),      // New commit created
    CompletedNoCommit,      // Stopped without committing
    AskingQuestion,         // Claude asked a question
    NeedsConfirmation,      // Yes/no prompt shown
    Crashed,                // Session crashed
}
```

## Failure Recovery

The system must be resilient to various failure modes:

### 1. Lost Input

**Symptom**: Message sent but Claude doesn't respond.

**Detection**: After sending input, if Claude remains in "Ready" state for
longer than expected.

**Recovery**:
- Capture pane output to verify message wasn't received
- Re-send with increased debounce delay
- If still failing, notify user and mark worker as `error`

### 2. Session Crash

**Symptom**: TMUX session exists but Claude process has exited.

**Detection**: Pane command is a shell (bash/zsh) instead of Claude.

**Recovery**:
- Log the crash with pane output for debugging
- Respawn Claude process in the same pane
- Re-send the last prompt if worker was `working`
- Transition to `error` if respawn fails

### 3. Orphaned Sessions

**Symptom**: State file references sessions that don't exist.

**Detection**: `llmc up` startup reconciliation.

**Recovery**:
- Mark workers with missing sessions as `offline`
- Create sessions for offline workers
- Re-initialize each worker to `idle` state

### 4. Stuck Processing

**Symptom**: Worker is `working` but Claude is ready (prompt visible).

**Detection**: Patrol detects ready state but status is `working`.

**Recovery**:
- Capture output to determine what happened
- If commit detected, transition to `needs_review`
- If no commit, transition to `needs_input`
- Play alert sound

### 5. Partial Sends

**Symptom**: Message partially pasted before Enter was sent.

**Detection**: Pane shows partial message in input line.

**Recovery**:
- Send Ctrl-U to clear the line
- Wait for clear to take effect
- Re-send the full message

## Debounce Timing Research Results (January 2026)

Empirical testing was conducted to validate the proposed debounce timing parameters.
Testing environment: TMUX 3.6a, macOS Darwin 24.5.0, zsh shell.

### Key Findings

**1. Terminal Width is Critical**

The most important discovery is that TMUX session width significantly impacts
message delivery. With default terminal width (~80 columns), messages >300 chars
are truncated. With wider terminals (500+ columns), messages up to 800+ chars
work reliably.

**Recommendation**: Create sessions with `-x 500` or wider:
```bash
tmux new-session -d -s "$SESSION" -x 500 -y 100
```

**2. Small Messages Need No Debounce**

For messages under 256 bytes with adequate terminal width:
- 0ms debounce delay achieved 100% reliability over 20+ trials
- The proposed 500ms base delay is conservative but safe

| Size | Min Reliable Delay |
|------|-------------------|
| 64B  | 0ms (100%) |
| 128B | 0ms (100%) |
| 256B | 0ms (100%) |

**3. System Load Has Minimal Impact**

Testing under heavy CPU load (4x `yes > /dev/null`) and I/O load (`dd`):
- 100ms debounce remained 100% reliable
- No additional delay needed under normal system load

**4. Partial Send Recovery Works**

The proposed `Ctrl-U` recovery mechanism is effective:
- Partial messages ARE detectable by examining pane content
- `Ctrl-U` reliably clears the input line (100% success rate)
- Recovery sequence: `C-u`, wait 100ms, re-send message

**5. Large Message Limitations**

Shell command-line limits (~1-2KB) prevent direct `send-keys -l` for large prompts.

**Recommendations for large prompts (>1KB)**:
1. Use `load-buffer` from a temporary file
2. Or use bracketed paste mode
3. Or split into multiple messages with reassembly markers

### Updated Timing Recommendations

Based on testing, the following parameters are recommended:

```rust
pub struct TmuxSender {
    // Terminal configuration
    session_width: u32,         // Minimum 500 columns

    // Debounce timing (validated values)
    debounce_base_ms: u32,      // Can be reduced to 100ms for small messages
    debounce_per_kb_ms: u32,    // 100ms per KB is adequate
    max_debounce_ms: u32,       // 2000ms cap is reasonable

    // Recovery timing
    recovery_delay_ms: u32,     // 100ms after Ctrl-U
}
```

For the conservative formula from Gastown (500ms base + 100ms/KB, capped at 2000ms):
- Testing confirms this works reliably
- Could be reduced to 100ms base for faster response, but 500ms provides safety margin

### Large Message Delivery Pattern

For prompts >1KB, recommended approach:

```rust
fn send_large_message(&self, session: &str, message: &str) -> Result<()> {
    // Write to temp file
    let tmp = tempfile::NamedTempFile::new()?;
    fs::write(tmp.path(), message)?;

    // Load into tmux buffer
    tmux::load_buffer(session, "-b", "prompt", tmp.path())?;

    // Paste from buffer
    tmux::paste_buffer(session, "-b", "prompt")?;

    // Standard debounce and Enter
    sleep(self.calculate_delay(message.len()));
    tmux::send_keys(session, "Enter")?;

    Ok(())
}
```

### Testing Methodology

Tests were run with:
- Multiple message sizes (64B to 16KB)
- Multiple debounce delays (0ms to 2000ms)
- Multiple trials per configuration (5-20)
- Various load conditions (baseline, CPU, I/O)

Test scripts are preserved at `/tmp/debounce_test_*.sh` for reproducibility.
