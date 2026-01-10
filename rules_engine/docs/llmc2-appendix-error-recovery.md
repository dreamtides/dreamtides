# LLMC v2 Appendix: Error Recovery Decision Trees

This appendix contains detailed recovery strategies for each failure mode
identified in the LLMC system.

## 1. Lost Input Recovery

**Failure Mode**: Message sent to worker but Claude doesn't respond.

### Detection

After sending input via `send_keys`, poll for state change:

```rust
fn verify_input_received(&self, session: &str, timeout: Duration) -> InputResult {
    let deadline = Instant::now() + timeout;
    let initial_state = self.detect_state(session);

    while Instant::now() < deadline {
        sleep(Duration::from_millis(500));
        let current_state = self.detect_state(session);

        // Input received if state changed from Ready to Processing
        if initial_state == ClaudeState::Ready && current_state == ClaudeState::Processing {
            return InputResult::Received;
        }

        // Input received if pane content changed significantly
        if self.pane_content_changed_significantly(session) {
            return InputResult::Received;
        }
    }

    InputResult::Lost
}
```

### Decision Tree

```
Lost Input Detected
        │
        ▼
┌─────────────────────┐
│ Attempt 1           │
│ Increase debounce   │
│ +200ms, resend      │
└──────────┬──────────┘
           │
    ┌──────┴──────┐
    │ Received?   │
    └──────┬──────┘
       No  │  Yes ──▶ Success
           ▼
┌─────────────────────┐
│ Attempt 2           │
│ Use load-buffer     │
│ method, resend      │
└──────────┬──────────┘
           │
    ┌──────┴──────┐
    │ Received?   │
    └──────┬──────┘
       No  │  Yes ──▶ Success
           ▼
┌─────────────────────┐
│ Attempt 3           │
│ Kill pane, respawn  │
│ Claude, resend      │
└──────────┬──────────┘
           │
    ┌──────┴──────┐
    │ Received?   │
    └──────┬──────┘
       No  │  Yes ──▶ Success
           ▼
┌─────────────────────┐
│ Mark worker ERROR   │
│ Log diagnostics     │
│ Notify user         │
└─────────────────────┘
```

### Implementation Details

**Attempt 1**: Increase debounce by 200ms from baseline. This handles transient
timing issues.

**Attempt 2**: Switch to `load-buffer` + `paste-buffer` method. This bypasses
shell command-line limits and handles special characters more reliably.

**Attempt 3**: Full session reset. Kill the Claude process, restart it, wait for
ready state, then resend. This handles cases where Claude is in an unresponsive
state that isn't detected as a crash.

### Retry Timing

| Attempt | Debounce | Method | Wait Before Retry |
|---------|----------|--------|------------------|
| 1 | +200ms | send-keys -l | 2s |
| 2 | base | load-buffer | 5s |
| 3 | base | respawn + send | 10s |

Total maximum time before escalation: ~17 seconds.

## 2. Session Crash Recovery

**Failure Mode**: Claude process exits unexpectedly.

### Detection

```rust
fn is_session_crashed(&self, session: &str) -> bool {
    let cmd = self.tmux.get_pane_command(session)?;
    // Shell running means Claude exited
    is_shell(&cmd)
}

fn is_shell(cmd: &str) -> bool {
    matches!(cmd, "bash" | "zsh" | "sh" | "fish" | "dash")
}
```

### Decision Tree

```
Session Crash Detected
        │
        ▼
┌─────────────────────┐
│ Classify crash type │
│ (see below)         │
└──────────┬──────────┘
           │
    ┌──────┴──────┐
    │ Crash type? │
    └──────┬──────┘
           │
    ┌──────┼──────┬──────────┐
    │      │      │          │
    ▼      ▼      ▼          ▼
 User   Rate   Other      Fatal
 Exit   Limit  Crash      Error
    │      │      │          │
    ▼      ▼      ▼          │
 Reset  Wait   Increment    │
 to     5min   crash_count  │
 Idle   retry     │         │
           │      ▼         │
           │   count < 3?   │
           │      │         │
           │   Yes│No       │
           │      │ │       │
           │      ▼ ▼       │
           │   Auto  Mark   │
           │   restart      │
           │      │  ERROR ◀┘
           │      │    │
           │      ▼    ▼
           └───▶ Restore context
                      │
                      ▼
               Was working?
                      │
               Yes    │    No
                │     │     │
                ▼     │     ▼
           Resend     │   Reset
           prompt     │   to Idle
                      │
                      ▼
                  Continue
```

### Crash Classification

| Exit Code | Classification | Recovery |
|-----------|---------------|----------|
| 0 | User exit (`/exit`, Ctrl-C) | Reset to idle |
| 130 | SIGINT | Reset to idle |
| 137 | SIGKILL | Auto-restart |
| Other | Crash | Auto-restart |

Check pane output for additional context:

```rust
fn classify_crash(&self, session: &str) -> CrashType {
    let output = self.tmux.capture_pane(session, 50)?;

    if output.contains("rate limit") || output.contains("429") {
        return CrashType::RateLimit;
    }
    if output.contains("/exit") || output.contains("Goodbye") {
        return CrashType::UserExit;
    }
    if output.contains("FATAL") || output.contains("panic") {
        return CrashType::Fatal;
    }

    CrashType::Unknown
}
```

### Context Restoration

When restarting a worker that was in `working` state:

1. Respawn Claude process in the pane
2. Wait for ready state
3. Send `/clear`
4. Send the original prompt from `current_prompt` field
5. Append context: "Note: The session crashed during processing. Previous
   partial work may be visible in the git diff. Please continue from where you
   left off."

### Crash Count Tracking

Track crashes per worker in the state file:

```rust
struct WorkerState {
    // ... existing fields
    crash_count: u32,
    last_crash_unix: Option<u64>,
}
```

Reset `crash_count` to 0 when:
- Worker successfully completes a task
- 24 hours have passed since `last_crash_unix`

## 3. Stuck Processing Recovery

**Failure Mode**: Worker remains in `working` state but Claude is ready or idle.

### Detection

The patrol system checks for stuck workers:

```rust
fn detect_stuck_worker(&self, worker: &Worker) -> Option<StuckReason> {
    if worker.status != WorkerStatus::Working {
        return None;
    }

    let working_duration = now_unix() - worker.last_activity_unix;
    let claude_state = self.detect_state(&worker.session_id);

    // Case 1: Claude is ready but we think it's working
    if claude_state == ClaudeState::Ready && working_duration > 60 {
        return Some(StuckReason::ReadyButMarkedWorking);
    }

    // Case 2: Working for too long without activity
    if working_duration > 30 * 60 {
        return Some(StuckReason::Timeout);
    }

    // Case 3: Claude is in error state
    if matches!(claude_state, ClaudeState::Error { .. }) {
        return Some(StuckReason::ErrorState);
    }

    None
}
```

### Decision Tree

```
Stuck Worker Detected
        │
        ▼
┌─────────────────────┐
│ Identify reason     │
└──────────┬──────────┘
           │
    ┌──────┴──────┬────────────┐
    │             │            │
    ▼             ▼            ▼
 Ready but     Timeout     Error State
 Working       (>30min)
    │             │            │
    ▼             ▼            ▼
 Check for    Send nudge   Log error
 commit       message      details
    │             │            │
    ▼             │            ▼
 Commit       ┌──┴──┐      Recoverable?
 found?       │Wait │          │
    │         │5min │      Yes │ No
 Yes│No       └──┬──┘          │  │
    │ │          │             ▼  ▼
    ▼ ▼          ▼          Resend  Mark
 Needs  Needs  Responds?    nudge  ERROR
 Review Input     │             │
              Yes │ No          │
                  │  │          │
                  ▼  ▼          │
              Continue Mark    │
                      ERROR ◀──┘
```

### Timeout Thresholds

| Duration | Action |
|----------|--------|
| 30 minutes | First nudge: "Are you still working? Please provide a status update." |
| 35 minutes | Check for response |
| 40 minutes | Second nudge: "The system will mark this task as requiring attention if no response in 5 minutes." |
| 45 minutes | Mark as `needs_input`, play alert sound |

### Nudge Messages

**First nudge (30 min)**:
```
Status check: You've been working on this task for 30 minutes. Are you making
progress or blocked on something? Please provide a brief update.
```

**Second nudge (40 min)**:
```
This task will be flagged for human review if there's no response in 5 minutes.
If you're blocked, please describe the issue. If you're still working, please
commit your progress so far.
```

### Commit Detection

When a stuck worker is in ready state, check for new commits:

```rust
fn check_for_new_commit(&self, worker: &Worker) -> Option<String> {
    let git_log = Command::new("git")
        .args(["log", "-1", "--format=%H", "HEAD"])
        .current_dir(&worker.worktree_path)
        .output()?;

    let current_sha = String::from_utf8_lossy(&git_log.stdout).trim().to_string();

    if worker.commit_sha.as_ref() != Some(&current_sha) {
        // New commit found
        Some(current_sha)
    } else {
        None
    }
}
```

## 4. Partial Send Recovery

**Failure Mode**: Message partially pasted before Enter was sent.

### Detection

Capture pane content and check for incomplete input:

```rust
fn detect_partial_send(&self, session: &str, expected_msg: &str) -> PartialSendStatus {
    let output = self.tmux.capture_pane(session, 10)?;
    let lines: Vec<&str> = output.lines().collect();

    // Find the input line (starts with "> ")
    let input_line = lines.iter()
        .find(|line| line.trim_start().starts_with("> "));

    if let Some(input) = input_line {
        let typed_content = input.trim_start().strip_prefix("> ").unwrap_or("");

        if typed_content.is_empty() {
            return PartialSendStatus::NoInput;
        }

        // Check if it's a prefix of our expected message
        if expected_msg.starts_with(typed_content) && typed_content != expected_msg {
            return PartialSendStatus::Partial {
                received: typed_content.len(),
                expected: expected_msg.len(),
            };
        }

        if typed_content == expected_msg {
            return PartialSendStatus::Complete;
        }
    }

    PartialSendStatus::Unknown
}
```

### Decision Tree

```
After Send Attempt
        │
        ▼
┌─────────────────────┐
│ Capture pane output │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Check input line    │
└──────────┬──────────┘
           │
    ┌──────┴──────┬──────────┐
    │             │          │
    ▼             ▼          ▼
 No input     Partial     Complete
    │         received
    │             │
    ▼             ▼
 Verify       Clear line
 timing       (Ctrl-U)
    │             │
    ▼             ▼
 Resend      Wait 100ms
    │             │
    │             ▼
    │          Increment
    │          attempt_count
    │             │
    │             ▼
    │         count < 3?
    │             │
    │         Yes │ No
    │          │  │
    │          ▼  ▼
    │       Resend Mark
    │              ERROR
    │                │
    └────────────────┘
```

### Clear and Retry Sequence

```rust
fn clear_and_retry(&self, session: &str, message: &str) -> Result<()> {
    // Send Ctrl-U to clear the input line
    self.tmux.send_keys(session, "C-u")?;

    // Wait for clear to take effect
    sleep(Duration::from_millis(100));

    // Verify line is clear
    let status = self.detect_partial_send(session, message);
    if !matches!(status, PartialSendStatus::NoInput) {
        // Try Ctrl-C as backup
        self.tmux.send_keys(session, "C-c")?;
        sleep(Duration::from_millis(200));
    }

    // Resend with increased debounce
    self.send_with_debounce(session, message, self.debounce_base_ms + 200)
}
```

### Prevention

To minimize partial sends:

1. **Wide terminal**: Create sessions with `-x 500` to prevent line wrapping
2. **Use load-buffer for large messages**: Anything >1KB should use file-based
   transfer
3. **Verify after send**: Always check that the message was received before
   sending Enter

## 5. State Corruption Recovery

**Failure Mode**: `state.json` contains invalid JSON or inconsistent data.

### Detection

State corruption can occur from:
- Interrupted writes (crash during save)
- Manual editing errors
- Disk corruption

```rust
fn validate_state(&self, path: &Path) -> ValidationResult {
    // 1. Check file exists
    if !path.exists() {
        return ValidationResult::Missing;
    }

    // 2. Parse JSON
    let content = fs::read_to_string(path)?;
    let state: Result<State, _> = serde_json::from_str(&content);

    if let Err(e) = state {
        return ValidationResult::ParseError(e.to_string());
    }

    let state = state.unwrap();

    // 3. Schema validation
    if let Err(e) = self.validate_schema(&state) {
        return ValidationResult::SchemaError(e);
    }

    // 4. Consistency checks
    if let Err(e) = self.validate_consistency(&state) {
        return ValidationResult::InconsistentState(e);
    }

    ValidationResult::Valid
}

fn validate_schema(&self, state: &State) -> Result<()> {
    for worker in &state.workers {
        // Required fields
        if worker.name.is_empty() {
            return Err(anyhow!("Worker has empty name"));
        }
        if worker.worktree_path.is_empty() {
            return Err(anyhow!("Worker {} has empty worktree_path", worker.name));
        }

        // Valid status
        if !is_valid_status(&worker.status) {
            return Err(anyhow!("Worker {} has invalid status", worker.name));
        }

        // Timestamp sanity
        if worker.created_at_unix > now_unix() + 86400 {
            return Err(anyhow!("Worker {} has future created_at", worker.name));
        }
    }
    Ok(())
}

fn validate_consistency(&self, state: &State) -> Result<()> {
    // Check for duplicate worker names
    let names: HashSet<_> = state.workers.iter().map(|w| &w.name).collect();
    if names.len() != state.workers.len() {
        return Err(anyhow!("Duplicate worker names detected"));
    }

    // Check worktree paths exist (soft check)
    for worker in &state.workers {
        if !Path::new(&worker.worktree_path).exists() {
            log::warn!("Worker {} worktree doesn't exist: {}", worker.name, worker.worktree_path);
        }
    }

    // Verify needs_review workers have commit_sha
    for worker in &state.workers {
        if worker.status == WorkerStatus::NeedsReview && worker.commit_sha.is_none() {
            return Err(anyhow!("Worker {} in needs_review but no commit_sha", worker.name));
        }
    }

    Ok(())
}
```

### Decision Tree

```
State Validation Failed
        │
        ▼
┌─────────────────────┐
│ Identify error type │
└──────────┬──────────┘
           │
    ┌──────┴──────┬──────────┬──────────┐
    │             │          │          │
    ▼             ▼          ▼          ▼
 Missing       Parse      Schema    Inconsistent
    │          Error      Error         │
    │             │          │          │
    ▼             ▼          ▼          ▼
 Check for   Try backup  Try backup  Attempt
 backup         │          │        auto-repair
    │           │          │          │
 Found?      Success?   Success?   Success?
    │           │          │          │
 Yes│No     Yes│No     Yes│No     Yes│No
    │ │        │ │        │ │        │ │
    ▼ ▼        ▼ ▼        ▼ ▼        ▼ ▼
Restore Init  Restore Init Restore Continue Mark
backup  new   backup  new  backup       ERROR
              │            │            │
              ▼            ▼            │
           Manual      Manual          │
           edit hint   intervention ◀──┘
```

### Backup and Restore

The system maintains automatic backups:

```rust
fn save_state(&self, state: &State) -> Result<()> {
    let path = self.state_path();
    let backup_path = self.backup_path();

    // 1. Validate before save
    self.validate_state_object(state)?;

    // 2. Create backup of current state
    if path.exists() {
        fs::copy(&path, &backup_path)?;
    }

    // 3. Write to temp file
    let tmp_path = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(state)?;
    fs::write(&tmp_path, &content)?;

    // 4. Atomic rename
    fs::rename(&tmp_path, &path)?;

    Ok(())
}

fn restore_from_backup(&self) -> Result<State> {
    let backup_path = self.backup_path();

    if !backup_path.exists() {
        return Err(anyhow!("No backup file found"));
    }

    let content = fs::read_to_string(&backup_path)?;
    let state: State = serde_json::from_str(&content)?;

    // Validate backup
    self.validate_state_object(&state)?;

    // Restore
    fs::copy(&backup_path, self.state_path())?;

    log::info!("Restored state from backup");
    Ok(state)
}
```

### Auto-Repair Strategies

For inconsistent (but parseable) state:

```rust
fn attempt_auto_repair(&self, state: &mut State) -> Result<Vec<String>> {
    let mut repairs = Vec::new();

    for worker in &mut state.workers {
        // Fix: needs_review without commit_sha
        if worker.status == WorkerStatus::NeedsReview && worker.commit_sha.is_none() {
            // Try to find the latest commit in the worktree
            if let Ok(sha) = get_head_commit(&worker.worktree_path) {
                worker.commit_sha = Some(sha.clone());
                repairs.push(format!("Set commit_sha for {} to {}", worker.name, sha));
            } else {
                // Can't find commit, reset to needs_input
                worker.status = WorkerStatus::NeedsInput;
                repairs.push(format!("Reset {} from needs_review to needs_input", worker.name));
            }
        }

        // Fix: working with no prompt
        if worker.status == WorkerStatus::Working && worker.current_prompt.is_empty() {
            worker.status = WorkerStatus::NeedsInput;
            repairs.push(format!("Reset {} from working to needs_input (no prompt)", worker.name));
        }

        // Fix: invalid timestamps
        let now = now_unix();
        if worker.created_at_unix > now {
            worker.created_at_unix = now;
            repairs.push(format!("Fixed future created_at for {}", worker.name));
        }
        if worker.last_activity_unix > now {
            worker.last_activity_unix = now;
            repairs.push(format!("Fixed future last_activity for {}", worker.name));
        }
    }

    Ok(repairs)
}
```

### Manual Recovery Commands

`llmc doctor --repair` provides guided recovery:

```
$ llmc doctor --repair

Checking state file...
ERROR: state.json parse error at line 45

Recovery options:
  1. Restore from backup (state.json.bak from 2 hours ago)
  2. Initialize fresh state (will lose worker history)
  3. Open in editor for manual fix

Select option [1]:
```

For complete corruption where backup also fails:

```
$ llmc doctor --rebuild

Rebuilding state from filesystem...
Found worktrees: adam, baker, charlie
Found TMUX sessions: llmc-adam, llmc-charlie

Reconstructed state:
  adam: worktree exists, session exists -> idle
  baker: worktree exists, session missing -> offline
  charlie: worktree exists, session exists -> idle

Save reconstructed state? [y/N]:
```

## Error Logging and Diagnostics

All error recovery actions are logged with context:

```rust
fn log_recovery_action(&self, worker: &str, action: &RecoveryAction, context: &str) {
    let entry = RecoveryLogEntry {
        timestamp: Utc::now(),
        worker: worker.to_string(),
        action: action.clone(),
        context: context.to_string(),
        pane_output: self.capture_pane_output(worker).ok(),
        git_status: self.get_git_status(worker).ok(),
    };

    // Append to worker's log file
    let log_path = self.logs_dir.join(format!("{}.log", worker));
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    writeln!(file, "{}", serde_json::to_string(&entry)?)?;
}
```

When escalating to user, provide diagnostic bundle:

```
Worker 'adam' requires manual intervention.

Error: Lost input after 3 retry attempts

Diagnostics:
  - Status: working
  - Last activity: 5 minutes ago
  - Prompt: "Implement the user authentication flow..."

Recent pane output:
  > Implement the user aut
  [cursor]

Suggested actions:
  1. llmc attach adam  - Connect to session manually
  2. llmc message adam "Please confirm you received the task"
  3. llmc nuke adam && llmc add adam  - Recreate worker
```
