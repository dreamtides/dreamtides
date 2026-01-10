# LLMC v2 Appendix: Claude State Detection Heuristics

This appendix contains detailed implementation notes for detecting Claude's
operational state from terminal output parsing.

## Design Principles

1. **Multi-Signal Detection**: Use multiple signals (pane command, output patterns,
   timing) to avoid false positives/negatives
2. **Hierarchical Classification**: Check states in priority order (crash > confirmation
   > question > ready > processing)
3. **Confidence Scoring**: Some detections are high-confidence (process exit), others
   need corroboration (output patterns)
4. **Stateful Context**: Track previous states to improve classification accuracy

## ClaudeState Enum (Extended)

```rust
pub enum ClaudeState {
    /// Claude is ready for user input (shows "> " prompt)
    /// - Prompt visible at line start
    /// - Claude process running
    Ready,

    /// Claude is actively processing (thinking, tool calls, etc.)
    /// - No prompt visible
    /// - Claude process running
    Processing,

    /// Claude is asking a yes/no or multiple choice question
    /// - Uses AskUserQuestion tool
    /// - Shows numbered options (1, 2, 3...) or option labels
    /// - Waiting for user selection
    AwaitingQuestion {
        question_type: QuestionType,
    },

    /// Claude is showing a tool permission prompt
    /// - "Allow this command?" style dialogs
    /// - Requires yes/no/always response
    AwaitingPermission {
        tool_name: String,
    },

    /// Claude encountered an error but is still running
    /// - API errors, rate limits, etc.
    /// - May show retry countdown
    Error {
        error_type: ErrorType,
        recoverable: bool,
    },

    /// Claude process has exited
    /// - Pane command is shell (bash/zsh)
    /// - Need to determine if crash or normal exit
    Exited {
        exit_type: ExitType,
    },

    /// State cannot be reliably determined
    Unknown,
}

pub enum QuestionType {
    /// AskUserQuestion tool - shows numbered options
    MultipleChoice,
    /// Simple yes/no confirmation
    YesNo,
    /// Text input requested
    FreeText,
}

pub enum ExitType {
    /// User sent /exit or Ctrl-C
    UserInitiated,
    /// Claude crashed or was killed
    Crash,
    /// Session timeout or disconnect
    Timeout,
    /// Unknown exit reason
    Unknown,
}

pub enum ErrorType {
    /// API rate limit hit
    RateLimit,
    /// Network/API connectivity issue
    Network,
    /// Tool execution failed
    ToolError,
    /// Other API error
    ApiError,
}
```

## Detection Heuristics

### 1. Process Health Check (Highest Priority)

```rust
fn check_process_health(&self, session: &str) -> ProcessHealth {
    let cmd = self.tmux.get_pane_command(session)?;

    // Claude reports as "node", "claude", or version number like "2.0.76"
    let is_claude = matches!(cmd.as_str(), "node" | "claude")
        || SEMVER_REGEX.is_match(&cmd);

    if is_claude {
        ProcessHealth::Running
    } else if is_shell(&cmd) {
        // Shell running = Claude exited
        ProcessHealth::Exited
    } else {
        // Some other command running (unexpected)
        ProcessHealth::Unknown
    }
}

fn is_shell(cmd: &str) -> bool {
    matches!(cmd, "bash" | "zsh" | "sh" | "fish" | "dash")
}

const SEMVER_REGEX: Regex = Regex::new(r"^\d+\.\d+\.\d+").unwrap();
```

### 2. Ready State Detection

```rust
fn is_ready_for_input(&self, pane_output: &str) -> bool {
    // Claude's input prompt appears as:
    // - "> " at the start of a line (with space for text entry)
    // - ">" alone at start of line (empty prompt)
    //
    // Important: Only check the LAST few lines, as "> " may appear
    // in code blocks or quoted text earlier in the output.
    let lines: Vec<&str> = pane_output.lines().rev().take(5).collect();

    for line in lines {
        let trimmed = line.trim_start();
        // Match "> " at line start, or lone ">" at end of output
        if trimmed.starts_with("> ") || trimmed == ">" {
            return true;
        }
    }
    false
}
```

### 3. AskUserQuestion Detection (Multiple Choice Questions)

Claude's AskUserQuestion tool renders questions with numbered options. The output
pattern looks like:

```
? Which approach should we use?
  1) Option A - Description of option A
  2) Option B - Description of option B
  3) Option C - Description of option C
  4) Other
Use arrow keys to navigate, Enter to select
```

```rust
fn detect_ask_user_question(&self, output: &str) -> Option<QuestionType> {
    let lines: Vec<&str> = output.lines().collect();

    // Look for question marker and numbered options in last 20 lines
    let recent_lines: Vec<&str> = lines.iter().rev().take(20).copied().collect();
    let recent_text = recent_lines.join("\n");

    // Pattern 1: Numbered options (1), 2), 3)...)
    let has_numbered_options = NUMBERED_OPTION_REGEX.is_match(&recent_text);

    // Pattern 2: Question indicator (? at start of line, often in blue/cyan)
    let has_question_marker = recent_lines.iter()
        .any(|line| line.trim_start().starts_with("?"));

    // Pattern 3: Navigation hint text
    let has_nav_hint = recent_text.contains("arrow keys")
        || recent_text.contains("Enter to select")
        || recent_text.contains("navigate");

    // Pattern 4: Selection brackets like [x] or ( )
    let has_selection_indicators = recent_text.contains("[ ]")
        || recent_text.contains("[x]")
        || recent_text.contains("( )");

    if has_numbered_options && (has_question_marker || has_nav_hint) {
        return Some(QuestionType::MultipleChoice);
    }

    if has_selection_indicators {
        return Some(QuestionType::YesNo);
    }

    None
}

// Matches patterns like "1)" "2)" "  3)" or "1." "2." "3."
const NUMBERED_OPTION_REGEX: Regex = Regex::new(r"^\s*\d+[.)]").unwrap();
```

### 4. Tool Permission Prompt Detection

Tool permission prompts appear when Claude wants to execute a command or access
a resource. They have distinctive patterns:

```
╭────────────────────────────────────────────╮
│ Claude wants to run: bash                  │
│ Command: git status                        │
╰────────────────────────────────────────────╯
? Allow this action?
  1) Yes, allow once
  2) Yes, always allow for this tool
  3) No, deny
```

```rust
fn detect_permission_prompt(&self, output: &str) -> Option<String> {
    let recent_text = self.get_recent_lines(output, 30);

    // Pattern 1: "Claude wants to run:" or "Claude wants to:"
    let wants_to_regex = Regex::new(r"Claude wants to (?:run|use|access|execute):?\s*(\w+)").unwrap();
    if let Some(captures) = wants_to_regex.captures(&recent_text) {
        let tool_name = captures.get(1).map(|m| m.as_str().to_string());
        if tool_name.is_some() {
            return tool_name;
        }
    }

    // Pattern 2: "Allow this" followed by action type
    if recent_text.contains("Allow this action")
        || recent_text.contains("Allow this command")
        || recent_text.contains("Allow this tool")
    {
        // Try to extract tool name from surrounding context
        return self.extract_tool_from_context(&recent_text);
    }

    // Pattern 3: Box drawing characters around permission request
    // (Claude renders these in a box)
    if recent_text.contains("╭") && recent_text.contains("╯")
        && (recent_text.contains("wants to run")
            || recent_text.contains("wants to use"))
    {
        return self.extract_tool_from_context(&recent_text);
    }

    None
}

fn extract_tool_from_context(&self, text: &str) -> Option<String> {
    // Look for tool names in common patterns
    let patterns = [
        Regex::new(r"Tool:\s*(\w+)").unwrap(),
        Regex::new(r"bash.*?Command:").unwrap(), // Bash tool
        Regex::new(r"(Read|Write|Edit|Glob|Grep|Bash)").unwrap(),
    ];

    for pattern in &patterns {
        if let Some(caps) = pattern.captures(text) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }
    }

    Some("unknown".to_string())
}
```

### 5. Distinguishing Completed vs. Waiting for Input

This is the most nuanced detection. Both states show the "> " prompt, but:
- **Completed**: Claude finished the task and is ready for a new one
- **Waiting for Input**: Claude asked a question and expects an answer

```rust
fn classify_ready_state(&self, output: &str) -> ReadyStateType {
    let recent_text = self.get_recent_lines(output, 50);

    // Check for explicit question indicators BEFORE the prompt
    if self.has_pending_question(&recent_text) {
        return ReadyStateType::WaitingForInput;
    }

    // Check for completion indicators
    if self.has_completion_indicators(&recent_text) {
        return ReadyStateType::Completed;
    }

    // Check for error/issue indicators that need user attention
    if self.has_needs_attention_indicators(&recent_text) {
        return ReadyStateType::WaitingForInput;
    }

    // Default: assume completed if we can't determine otherwise
    ReadyStateType::Completed
}

fn has_pending_question(&self, text: &str) -> bool {
    // Pattern 1: Direct question to user (ends with "?")
    // Look for "?" near the end of Claude's output, before the prompt
    let question_at_end = text.lines().rev()
        .skip(1) // Skip the prompt line
        .take(5)
        .any(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && trimmed.ends_with('?')
        });

    // Pattern 2: Explicit request for input
    let explicit_request = [
        "please let me know",
        "please tell me",
        "could you clarify",
        "would you like me to",
        "should I",
        "do you want me to",
        "what would you prefer",
        "which option",
        "please confirm",
    ].iter().any(|phrase| text.to_lowercase().contains(phrase));

    // Pattern 3: Trailing ellipsis suggesting incomplete thought
    let trailing_ellipsis = text.lines().rev()
        .skip(1)
        .take(3)
        .any(|line| line.trim().ends_with("..."));

    question_at_end || explicit_request || trailing_ellipsis
}

fn has_completion_indicators(&self, text: &str) -> bool {
    let indicators = [
        // Git commit indicators
        "created commit",
        "committed",
        "[master",
        "[main",

        // Task completion phrases
        "is now complete",
        "has been completed",
        "finished implementing",
        "done with the changes",
        "changes have been made",
        "successfully",

        // File operation completions
        "wrote to",
        "saved",
        "updated",

        // Test/build completions
        "all tests pass",
        "build succeeded",
        "no errors",
    ];

    indicators.iter().any(|ind| text.to_lowercase().contains(ind))
}

fn has_needs_attention_indicators(&self, text: &str) -> bool {
    let indicators = [
        // Errors that need user decision
        "error:",
        "failed:",
        "could not",
        "unable to",

        // Blocking issues
        "blocked",
        "need clarification",
        "unclear",
        "ambiguous",

        // Explicit help needed
        "need your help",
        "need input",
        "waiting for",
    ];

    indicators.iter().any(|ind| text.to_lowercase().contains(ind))
}

pub enum ReadyStateType {
    /// Task completed successfully, ready for new work
    Completed,
    /// Waiting for user to answer a question or provide input
    WaitingForInput,
}
```

### 6. Crash vs. Normal Exit Detection

When the pane command becomes a shell, Claude has exited. Distinguishing crash
from normal exit requires examining the exit context:

```rust
fn classify_exit(&self, session: &str, output: &str) -> ExitType {
    // Method 1: Check pane dead status (if using pane-died hook)
    // tmux can report exit code via #{pane_dead_status}
    if let Some(exit_code) = self.get_pane_exit_code(session) {
        return match exit_code {
            0 => ExitType::UserInitiated,  // Clean exit
            130 => ExitType::UserInitiated, // SIGINT (Ctrl-C)
            137 => ExitType::Crash,         // SIGKILL
            _ => ExitType::Crash,           // Non-zero = error
        };
    }

    // Method 2: Analyze terminal output for exit indicators
    let recent_text = self.get_recent_lines(output, 30);

    // User-initiated exits
    if recent_text.contains("/exit")
        || recent_text.contains("Goodbye")
        || recent_text.contains("exiting")
        || recent_text.contains("session ended")
    {
        return ExitType::UserInitiated;
    }

    // Crash indicators
    if recent_text.contains("panic")
        || recent_text.contains("FATAL")
        || recent_text.contains("Error:")
        || recent_text.contains("Uncaught exception")
        || recent_text.contains("SIGABRT")
        || recent_text.contains("SIGSEGV")
    {
        return ExitType::Crash;
    }

    // Timeout indicators
    if recent_text.contains("timeout")
        || recent_text.contains("timed out")
        || recent_text.contains("connection lost")
    {
        return ExitType::Timeout;
    }

    // Default: unknown
    ExitType::Unknown
}

fn get_pane_exit_code(&self, session: &str) -> Option<i32> {
    // Query tmux for pane's last exit status
    // This only works if the pane has exited and hasn't been respawned
    let output = self.tmux.run(&[
        "display-message", "-p", "-t", session,
        "#{pane_dead_status}"
    ]).ok()?;

    output.trim().parse().ok()
}
```

### 7. Error State Detection

Claude can encounter various errors while running:

```rust
fn detect_error_state(&self, output: &str) -> Option<(ErrorType, bool)> {
    let recent_text = self.get_recent_lines(output, 20);

    // Rate limit detection
    if recent_text.contains("rate limit")
        || recent_text.contains("429")
        || recent_text.contains("too many requests")
    {
        // Usually shows retry countdown - recoverable
        return Some((ErrorType::RateLimit, true));
    }

    // Network errors
    if recent_text.contains("network error")
        || recent_text.contains("connection refused")
        || recent_text.contains("ECONNRESET")
        || recent_text.contains("fetch failed")
    {
        return Some((ErrorType::Network, true));
    }

    // API errors
    if recent_text.contains("API error")
        || recent_text.contains("500")
        || recent_text.contains("502")
        || recent_text.contains("503")
    {
        return Some((ErrorType::ApiError, true));
    }

    // Tool errors (usually recoverable)
    if recent_text.contains("tool error")
        || recent_text.contains("command failed")
    {
        return Some((ErrorType::ToolError, true));
    }

    None
}
```

## Complete State Detection Flow

```rust
impl StateDetector {
    pub fn detect(&self, session: &str) -> Result<ClaudeState> {
        // 1. Check process health first
        let health = self.check_process_health(session)?;
        if health == ProcessHealth::Exited {
            let output = self.tmux.capture_pane(session, 50)?;
            let exit_type = self.classify_exit(session, &output);
            return Ok(ClaudeState::Exited { exit_type });
        }

        // 2. Capture pane output for pattern matching
        let output = self.tmux.capture_pane(session, 50)?;

        // 3. Check for error states (while Claude is still running)
        if let Some((error_type, recoverable)) = self.detect_error_state(&output) {
            return Ok(ClaudeState::Error { error_type, recoverable });
        }

        // 4. Check for permission prompts (highest interactive priority)
        if let Some(tool_name) = self.detect_permission_prompt(&output) {
            return Ok(ClaudeState::AwaitingPermission { tool_name });
        }

        // 5. Check for AskUserQuestion prompts
        if let Some(question_type) = self.detect_ask_user_question(&output) {
            return Ok(ClaudeState::AwaitingQuestion { question_type });
        }

        // 6. Check if ready for input
        if self.is_ready_for_input(&output) {
            // Sub-classify: completed vs waiting for input
            match self.classify_ready_state(&output) {
                ReadyStateType::Completed => return Ok(ClaudeState::Ready),
                ReadyStateType::WaitingForInput => {
                    return Ok(ClaudeState::AwaitingQuestion {
                        question_type: QuestionType::FreeText,
                    });
                }
            }
        }

        // 7. Still processing
        if health == ProcessHealth::Running {
            return Ok(ClaudeState::Processing);
        }

        Ok(ClaudeState::Unknown)
    }
}
```

## Handling Detected States

```rust
pub fn handle_detected_state(
    worker: &mut Worker,
    state: ClaudeState,
) -> Result<WorkerTransition> {
    match state {
        ClaudeState::Ready => {
            // Check for commits
            if let Some(sha) = detect_new_commit(&worker.worktree_path)? {
                return Ok(WorkerTransition::ToNeedsReview { commit_sha: sha });
            }
            // Completed without commit
            Ok(WorkerTransition::ToNeedsInput)
        }

        ClaudeState::Processing => {
            // Still working, no transition
            Ok(WorkerTransition::None)
        }

        ClaudeState::AwaitingQuestion { .. } => {
            // Worker needs input from user
            Ok(WorkerTransition::ToNeedsInput)
        }

        ClaudeState::AwaitingPermission { tool_name } => {
            // Auto-accept if using --dangerously-skip-permissions
            // Otherwise, may need user intervention
            if worker.config.skip_permissions {
                // This shouldn't happen, but handle gracefully
                log::warn!("Permission prompt despite skip_permissions for {}", tool_name);
            }
            Ok(WorkerTransition::ToNeedsInput)
        }

        ClaudeState::Error { error_type, recoverable } => {
            if recoverable {
                // Wait for auto-retry
                log::info!("Recoverable error: {:?}, waiting...", error_type);
                Ok(WorkerTransition::None)
            } else {
                Ok(WorkerTransition::ToError)
            }
        }

        ClaudeState::Exited { exit_type } => {
            match exit_type {
                ExitType::UserInitiated => {
                    // Normal exit, reset to idle
                    Ok(WorkerTransition::ToIdle)
                }
                ExitType::Crash | ExitType::Timeout => {
                    // Need to respawn
                    Ok(WorkerTransition::ToError)
                }
                ExitType::Unknown => {
                    Ok(WorkerTransition::ToError)
                }
            }
        }

        ClaudeState::Unknown => {
            Ok(WorkerTransition::None)
        }
    }
}
```

## Bypassing Permissions Dialog at Startup

When Claude starts with `--dangerously-skip-permissions`, it shows a confirmation
dialog that must be dismissed:

```rust
pub fn accept_bypass_warning(&self, session: &str) -> Result<()> {
    // Wait for dialog to render
    sleep(Duration::from_millis(1000));

    let output = self.tmux.capture_pane(session, 30)?;

    // Look for the characteristic warning text
    if !output.contains("Bypass Permissions mode") {
        return Ok(()); // No warning present
    }

    // Press Down to select "Yes, I accept" (option 2)
    self.tmux.send_keys_raw(session, "Down")?;
    sleep(Duration::from_millis(200));

    // Press Enter to confirm
    self.tmux.send_keys_raw(session, "Enter")?;

    Ok(())
}
```

## Considerations for --output-format

The `--output-format` flag only works with `--print` (non-interactive mode).
For LLMC2's interactive sessions, we cannot use JSON output format.

However, there are potential future directions:

1. **Hybrid Mode**: Start Claude with `--input-format stream-json` and
   `--output-format stream-json` for machine-readable I/O, but this would
   require significant changes to how prompts are sent.

2. **Sidecar Process**: Run a separate Claude process in `--print` mode to
   analyze worker output, but this adds complexity and cost.

3. **File-Based Signaling**: Have workers write state files (e.g., `.llmc/state`)
   that the daemon can watch, but this requires modifying worker behavior.

For now, terminal output parsing remains the most practical approach for
interactive sessions.
