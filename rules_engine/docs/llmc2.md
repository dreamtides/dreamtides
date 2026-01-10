# LLMC v2 Design Document

## Overview

LLMC v2 is a complete rewrite of the LLMC agent coordination system. It manages
multiple Claude Code CLI sessions running in parallel git worktrees, using TMUX
for persistent session management. The system enables coordinated development
work across multiple workers while maintaining a clean single-commit workflow on
the master branch.

### Key Differences from V1

- **TMUX-based sessions**: Workers are persistent Claude Code sessions in TMUX,
  not transient subprocesses
- **Interactive control**: Full interactive access to worker sessions via
  `llmc attach`
- **Persistent daemon**: `llmc up` runs continuously, monitoring workers and
  orchestrating state transitions
- **Patrol system**: Background process that maintains system health and
  facilitates rebasing

## Repository Layout

```
~/llmc/                           # LLMC root directory
├── config.toml                   # Global configuration
├── state.json                    # Worker registry and state
├── logs/                         # Per-worker logs
│   └── <worker>.log
├── .worktrees/                   # Git worktrees (one per worker)
│   ├── adam/                     # Worker "adam"'s worktree
│   ├── baker/                    # Worker "baker"'s worktree
│   └── ...
└── .git/                         # Git repository (local clone of master repo)

~/Documents/GoogleDrive/dreamtides/   # Master repository (source of truth)
```

## Worker State Machine

Each worker progresses through a well-defined state machine:

```
                      ┌─────────────────────────────────────────┐
                      │                                         │
                      ▼                                         │
┌─────────┐      ┌──────────┐      ┌───────────────┐           │
│  IDLE   │─────▶│ WORKING  │─────▶│ NEEDS_REVIEW  │───────────┤
└─────────┘ start└──────────┘commit└───────────────┘   accept  │
     ▲               │                    │                     │
     │               │ no commit          │ reject              │
     │               ▼                    ▼                     │
     │        ┌─────────────┐      ┌──────────┐                │
     │        │ NEEDS_INPUT │      │ REJECTED │────────────────┘
     │        └─────────────┘      └──────────┘    completes
     │               │                    │
     │               │ message            │ completes
     │               ▼                    │
     └───────────────────────────────────-┘

Special States:
- REBASING: Transitional state during rebase operations
- ERROR: Worker encountered an unrecoverable error
- OFFLINE: TMUX session not running (needs `llmc up`)
```

### State Definitions

| State | Description |
|-------|-------------|
| `idle` | Worker has no active task, ready to receive work |
| `working` | Worker is actively implementing a task |
| `needs_input` | Worker stopped without committing, likely waiting for clarification |
| `needs_review` | Worker completed work and committed, awaiting human review |
| `rejected` | Work was rejected with feedback, worker is implementing changes |
| `rebasing` | Worker is resolving merge conflicts after a rebase |
| `error` | Worker is in an error state requiring manual intervention |
| `offline` | TMUX session is not running |

## Configuration

Configuration is stored in `~/llmc/config.toml`:

```toml
# Global defaults
[defaults]
model = "opus"
skip_permissions = true
allowed_tools = ["Bash", "Edit", "Read", "Write", "Glob", "Grep"]
patrol_interval_secs = 60
sound_on_review = true

# Master repository location
[repo]
source = "~/Documents/GoogleDrive/dreamtides"

# Worker-specific overrides
[workers.adam]
model = "opus"
role_prompt = """
You are Adam, a senior engineer focused on backend systems.
You prefer simple, direct solutions over complex abstractions.
"""
excluded_from_pool = false

[workers.baker]
model = "sonnet"
role_prompt = """
You are Baker, focused on UI and user experience.
You pay close attention to visual details and accessibility.
"""

[workers.charlie]
model = "sonnet"
excluded_from_pool = true  # Only receives manually-assigned work
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `model` | string | `"opus"` | Claude model to use (`opus`, `sonnet`, `haiku`) |
| `skip_permissions` | bool | `true` | Use `--dangerously-skip-permissions` |
| `allowed_tools` | string[] | see above | Tools to allow via `--allowedTools` |
| `patrol_interval_secs` | u32 | `60` | Seconds between patrol runs |
| `sound_on_review` | bool | `true` | Play terminal bell when work needs review |
| `role_prompt` | string | `""` | Additional context for the worker |
| `excluded_from_pool` | bool | `false` | Exclude from automatic task assignment |

## Data Model

### State File (`~/llmc/state.json`)

```json
{
  "workers": {
    "adam": {
      "name": "adam",
      "worktree_path": "/Users/user/llmc/.worktrees/adam",
      "branch": "llmc/adam",
      "status": "working",
      "current_prompt": "Implement the new authentication flow...",
      "created_at_unix": 1704567890,
      "last_activity_unix": 1704568890,
      "commit_sha": null,
      "session_id": "llmc-adam"
    }
  },
  "last_reviewed_worker": "baker",
  "patrol_last_run_unix": 1704568800
}
```

### Worker Record Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Unique worker identifier |
| `worktree_path` | string | Absolute path to git worktree |
| `branch` | string | Git branch name (`llmc/<name>`) |
| `status` | Status | Current worker state |
| `current_prompt` | string | Full prompt text for current task |
| `created_at_unix` | u64 | Unix timestamp of worker creation |
| `last_activity_unix` | u64 | Unix timestamp of last state change |
| `commit_sha` | Option<string> | SHA of commit awaiting review |
| `session_id` | string | TMUX session identifier |

## TMUX Integration

### Session Management

LLMC uses TMUX for persistent Claude Code sessions. The `tmux_interface` crate
provides Rust bindings, but we implement our own higher-level abstractions for
reliability.

#### Session Naming Convention

```
llmc-<worker>
```

Examples: `llmc-adam`, `llmc-baker`, `llmc-charlie`

#### Session Startup Sequence

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

### Reliable Communication Protocol

**This is the most critical component of the system.** Claude Code sessions can
be in various states, and sending input at the wrong time leads to lost or
corrupted messages.

#### Input Debouncing

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

#### State Detection

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

#### Output Monitoring

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

### Failure Recovery

The system must be resilient to various failure modes:

#### 1. Lost Input

**Symptom**: Message sent but Claude doesn't respond.

**Detection**: After sending input, if Claude remains in "Ready" state for
longer than expected.

**Recovery**:
- Capture pane output to verify message wasn't received
- Re-send with increased debounce delay
- If still failing, notify user and mark worker as `error`

#### 2. Session Crash

**Symptom**: TMUX session exists but Claude process has exited.

**Detection**: Pane command is a shell (bash/zsh) instead of Claude.

**Recovery**:
- Log the crash with pane output for debugging
- Respawn Claude process in the same pane
- Re-send the last prompt if worker was `working`
- Transition to `error` if respawn fails

#### 3. Orphaned Sessions

**Symptom**: State file references sessions that don't exist.

**Detection**: `llmc up` startup reconciliation.

**Recovery**:
- Mark workers with missing sessions as `offline`
- Create sessions for offline workers
- Re-initialize each worker to `idle` state

#### 4. Stuck Processing

**Symptom**: Worker is `working` but Claude is ready (prompt visible).

**Detection**: Patrol detects ready state but status is `working`.

**Recovery**:
- Capture output to determine what happened
- If commit detected, transition to `needs_review`
- If no commit, transition to `needs_input`
- Play alert sound

#### 5. Partial Sends

**Symptom**: Message partially pasted before Enter was sent.

**Detection**: Pane shows partial message in input line.

**Recovery**:
- Send Ctrl-U to clear the line
- Wait for clear to take effect
- Re-send the full message

## The Patrol System

The Patrol is a background process that runs periodically during `llmc up` to
maintain system health.

### Patrol Operations

```rust
pub struct Patrol {
    state: Arc<Mutex<StateFile>>,
    tmux: TmuxManager,
    git: GitOperations,
}

impl Patrol {
    pub fn run(&self) -> Result<PatrolReport> {
        let mut report = PatrolReport::default();

        // 1. Check session health
        self.check_sessions(&mut report)?;

        // 2. Detect state transitions
        self.detect_transitions(&mut report)?;

        // 3. Rebase workers in needs_review
        self.rebase_pending_reviews(&mut report)?;

        // 4. Check for stuck workers
        self.detect_stuck_workers(&mut report)?;

        Ok(report)
    }

    fn check_sessions(&self, report: &mut PatrolReport) -> Result<()> {
        for (name, worker) in &self.state.lock().workers {
            let session_exists = self.tmux.has_session(&worker.session_id)?;

            if !session_exists && worker.status != Status::Offline {
                report.session_missing.push(name.clone());
                self.mark_offline(name)?;
            } else if session_exists && worker.status == Status::Offline {
                report.session_restored.push(name.clone());
                self.initialize_worker(name)?;
            }
        }
        Ok(())
    }

    fn detect_transitions(&self, report: &mut PatrolReport) -> Result<()> {
        for (name, worker) in &self.state.lock().workers {
            if worker.status != Status::Working {
                continue;
            }

            let claude_state = self.tmux.detect_state(&worker.session_id)?;
            if claude_state == ClaudeState::Ready {
                // Worker finished but we haven't processed it yet
                let event = self.classify_completion(name)?;
                self.handle_completion(name, event, report)?;
            }
        }
        Ok(())
    }

    fn rebase_pending_reviews(&self, report: &mut PatrolReport) -> Result<()> {
        for (name, worker) in &self.state.lock().workers {
            if worker.status != Status::NeedsReview {
                continue;
            }

            // Check if master has advanced
            if self.git.needs_rebase(&worker.worktree_path)? {
                report.rebase_triggered.push(name.clone());
                self.trigger_rebase(name)?;
            }
        }
        Ok(())
    }
}
```

### Patrol Schedule

The patrol runs on a configurable interval (default: 60 seconds) unless:

- A patrol is already running
- The user has executed `--no-patrol` on a command
- The system is shutting down

## Command Reference

### `llmc init`

Initializes a new LLMC project directory.

```bash
llmc init [--source <path>] [--target <path>]
```

**Steps**:
1. Validate required binaries: `git`, `git-lfs`, `claude`, `tmux`, `difft`
2. Create target directory if needed
3. Clone source repository with `--local` for efficiency
4. Configure git: `rerere.enabled=true`, `rerere.autoupdate=true`
5. Install git LFS and pull LFS files
6. Create `.worktrees/`, `logs/` directories
7. Initialize `state.json` and default `config.toml`
8. Copy `Tabula.xlsm` from source repo if present

**Tabula.xlsm Handling**:
```rust
fn copy_tabula_xlsm(source_root: &Path, target_root: &Path) -> Result<()> {
    let source = source_root.join("client/Assets/StreamingAssets/Tabula.xlsm");
    let dest = target_root.join("client/Assets/StreamingAssets/Tabula.xlsm");

    if source.exists() {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&source, &dest)?;
    }

    Ok(())
}
```

### `llmc up`

Starts the LLMC daemon, bringing up all worker sessions.

```bash
llmc up [--no-patrol]
```

**Operation**:
1. Load configuration and state
2. Start TMUX server if not running
3. For each configured worker:
   - Create worktree if missing
   - Create TMUX session if missing
   - Start Claude in session
   - Wait for Claude to be ready
   - Send `/clear` command
4. Enter main loop:
   - Monitor all sessions for state changes
   - Run patrol on schedule
   - Handle user interrupt (Ctrl-C) for graceful shutdown
5. Print status updates to console

**Graceful Shutdown**:
```rust
fn handle_shutdown(&self) -> Result<()> {
    println!("Shutting down LLMC...");

    for (name, worker) in &self.state.lock().workers {
        // Send Ctrl-C to gracefully stop Claude
        self.tmux.send_raw(&worker.session_id, "C-c")?;
        sleep(Duration::from_millis(500));

        // Kill the session
        self.tmux.kill_session(&worker.session_id)?;
    }

    // Save final state
    self.save_state()?;

    Ok(())
}
```

### `llmc down`

Stops all worker sessions.

```bash
llmc down [--force]
```

### `llmc add <name>`

Adds a new worker to the system.

```bash
llmc add <name> [--model <model>] [--role-prompt <prompt>]
```

**Steps**:
1. Validate name is unique
2. Create git worktree: `git worktree add -b llmc/<name> .worktrees/<name> master`
3. Copy `Tabula.xlsm` to worktree
4. Add worker entry to state
5. If `llmc up` is running, signal it to create the session

### `llmc nuke <name>`

Permanently removes a worker.

```bash
llmc nuke <name>
llmc nuke --all
```

**Steps**:
1. Kill TMUX session if running
2. Remove git worktree: `git worktree remove .worktrees/<name>`
3. Delete branch: `git branch -D llmc/<name>`
4. Remove worker from state

### `llmc status`

Displays status of all workers.

```bash
llmc status [--json]
```

**Output**:
```
LLMC Status
===========
adam     [working]      Implementing auth flow... (5m ago)
baker    [needs_review] Add user profile page (12m ago)
charlie  [idle]         Ready for tasks
```

### `llmc start`

Assigns a task to an idle worker.

```bash
llmc start --prompt "Implement feature X"
llmc start --prompt-file task.md
llmc start --worker adam --prompt "..."
```

**Steps**:
1. Select worker (specified or first idle from pool)
2. Verify worker is idle
3. Update worktree to latest master: `git pull --rebase`
4. Copy `Tabula.xlsm` from master repo
5. Build full prompt (preamble + user prompt)
6. Send `/clear` to worker
7. Send prompt to worker
8. Update state to `working`

**Prompt Preamble**:
```
You are working in: /Users/user/llmc/.worktrees/<worker>
Repository root: /Users/user/llmc

Follow AGENTS.md conventions. When finished:
- Run: just fmt, just check, just clippy
- Create a single commit with a detailed message
- DO NOT push to any remote
```

### `llmc message <worker> <message>`

Sends a message to a worker.

```bash
llmc message adam "Use the existing auth helper instead"
```

### `llmc attach <worker>`

Attaches to a worker's TMUX session for interactive use.

```bash
llmc attach adam
```

### `llmc review [worker]`

Shows diff for a worker awaiting review.

```bash
llmc review          # Reviews oldest pending worker
llmc review baker    # Reviews specific worker
llmc review --interface diff       # Plain git diff
llmc review --interface difftastic # Difftastic (default)
llmc review --interface vscode     # Open in VS Code
```

**Before Review**:
- Trigger rebase via patrol if master has advanced

### `llmc reject [message]`

Sends feedback to the most recently reviewed worker.

```bash
llmc reject "Please add error handling for the API call"
llmc reject --file feedback.md
```

**Steps**:
1. Identify target worker (last reviewed or specified)
2. Verify worker is in `needs_review` state
3. Do NOT send `/clear` (preserve context)
4. Send reject message with original diff context
5. Update state to `rejected`

### `llmc accept [worker]`

Accepts a worker's changes and merges to master.

```bash
llmc accept        # Accepts most recently reviewed worker
llmc accept baker
```

**Steps**:
1. Verify worker is in `needs_review` state
2. Ensure worktree is clean
3. Rebase onto latest master (via patrol)
4. Squash to single commit if multiple commits exist
5. Strip agent attribution from commit message
6. Fast-forward merge to master
7. Remove worktree and branch
8. Reset worker to `idle` with new worktree
9. Trigger background rebase for all other `needs_review` workers

**Commit Message Stripping**:
```rust
fn strip_agent_attribution(message: &str) -> String {
    message
        .lines()
        .filter(|line| {
            !line.contains("Generated with") &&
            !line.contains("Co-Authored-By")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
```

### `llmc rebase <worker>`

Manually triggers a rebase for a worker.

```bash
llmc rebase adam
```

**Steps**:
1. Fetch latest master
2. Attempt rebase: `git rebase master`
3. If conflicts:
   - Mark worker as `rebasing`
   - Send conflict resolution prompt to worker
   - Attach git status output
4. If clean:
   - Verify with `just check` and `just clippy`

## Merge Conflict Resolution UX

This section describes how LLMC2 handles merge conflicts during rebase operations.

### Overview

Merge conflicts can occur when:
1. Patrol rebases a `needs_review` worker onto a newly-advanced master
2. User manually triggers `llmc rebase <worker>`
3. `llmc accept` performs final rebase before merge

The system must present conflicts clearly to workers, allow them to resolve autonomously,
detect when resolution is complete, and handle failures gracefully.

### Conflict Detection

After running `git rebase master`, check the exit code and repository state:

```rust
pub struct RebaseResult {
    /// Rebase completed successfully (no conflicts)
    success: bool,
    /// List of files with conflicts
    conflicted_files: Vec<ConflictedFile>,
    /// Current rebase state
    rebase_state: RebaseState,
}

pub struct ConflictedFile {
    /// Path relative to repo root
    path: String,
    /// Type of conflict
    conflict_type: ConflictType,
    /// Number of conflict markers in file
    marker_count: u32,
}

pub enum ConflictType {
    /// Both sides modified the same lines
    Content,
    /// File modified on one side, deleted on other
    ModifyDelete,
    /// Both sides added a file with same name
    AddAdd,
    /// File renamed differently on each side
    RenameRename,
}

pub enum RebaseState {
    /// No rebase in progress
    None,
    /// Rebase paused waiting for conflict resolution
    InProgress { current_commit: String, remaining: u32 },
    /// Rebase completed
    Completed,
}

fn detect_conflicts(worktree: &Path) -> Result<RebaseResult> {
    // Check if rebase is in progress
    let rebase_dir = worktree.join(".git/rebase-merge");
    let rebase_apply = worktree.join(".git/rebase-apply");
    let in_rebase = rebase_dir.exists() || rebase_apply.exists();

    if !in_rebase {
        return Ok(RebaseResult {
            success: true,
            conflicted_files: vec![],
            rebase_state: RebaseState::None,
        });
    }

    // Get list of conflicted files
    let output = Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .current_dir(worktree)
        .output()?;

    let conflicted_paths: Vec<&str> = std::str::from_utf8(&output.stdout)?
        .lines()
        .filter(|s| !s.is_empty())
        .collect();

    let mut conflicted_files = vec![];
    for path in conflicted_paths {
        let conflict_type = detect_conflict_type(worktree, path)?;
        let marker_count = count_conflict_markers(worktree, path)?;
        conflicted_files.push(ConflictedFile {
            path: path.to_string(),
            conflict_type,
            marker_count,
        });
    }

    // Get rebase progress
    let rebase_state = get_rebase_state(worktree)?;

    Ok(RebaseResult {
        success: false,
        conflicted_files,
        rebase_state,
    })
}

fn count_conflict_markers(worktree: &Path, file: &str) -> Result<u32> {
    let content = fs::read_to_string(worktree.join(file))?;
    Ok(content.matches("<<<<<<<").count() as u32)
}
```

### Conflict Presentation Strategy

When presenting conflicts to a worker, provide:

1. **High-level summary**: What happened and what needs to be done
2. **Conflicted file list**: With conflict types and marker counts
3. **Context for each conflict**: The actual conflicted regions
4. **Clear instructions**: Step-by-step resolution process

#### Information Hierarchy

```
Level 1: Summary
├── Why: "Master has advanced, your changes need rebasing"
├── What: "3 files have conflicts"
└── Goal: "Resolve conflicts, then continue rebase"

Level 2: File List
├── src/foo.rs (Content conflict, 2 regions)
├── src/bar.rs (ModifyDelete - you modified, master deleted)
└── Cargo.toml (Content conflict, 1 region)

Level 3: Conflict Details (per file)
├── Surrounding context (20 lines before/after)
├── YOUR changes (what you wrote)
├── MASTER changes (what master has)
└── Semantic meaning (if detectable)
```

### Conflict Resolution Prompt Template

```rust
const CONFLICT_RESOLUTION_PREAMBLE: &str = r#"
## Merge Conflict Resolution Required

Your branch `{branch}` needs to be rebased onto the latest master. Git has
identified conflicts that require manual resolution.

### Conflict Summary

{conflict_summary}

### Resolution Process

1. For each conflicted file, examine the conflict markers
2. Decide how to resolve each conflict (keep yours, keep theirs, or merge both)
3. Remove ALL conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`)
4. After resolving ALL conflicts in a file, stage it: `git add <file>`
5. Once ALL files are resolved and staged, continue: `git rebase --continue`
6. Run validation: `just fmt && just check && just clippy`

### Conflict Details

{conflict_details}

### Important Notes

- DO NOT use `git rebase --abort` unless you cannot resolve the conflicts
- Ensure no conflict markers remain in any file after resolution
- The goal is to preserve your intended changes while incorporating master's updates
- If master's changes make your changes obsolete, you may discard yours
- If you need to see the original versions:
  - Your version: `git show :2:<file>` (or `REBASE_HEAD:<file>`)
  - Master version: `git show :3:<file>` (or `HEAD:<file>`)

Please resolve all conflicts now.
"#;
```

#### Generating Conflict Summary

```rust
fn generate_conflict_summary(result: &RebaseResult) -> String {
    let mut summary = String::new();

    let file_count = result.conflicted_files.len();
    let total_regions: u32 = result.conflicted_files.iter()
        .map(|f| f.marker_count)
        .sum();

    writeln!(summary, "**{} file(s)** with **{} conflict region(s)**:\n",
        file_count, total_regions).unwrap();

    for file in &result.conflicted_files {
        let conflict_desc = match file.conflict_type {
            ConflictType::Content =>
                format!("{} conflict region(s)", file.marker_count),
            ConflictType::ModifyDelete =>
                "modified by you, deleted in master".to_string(),
            ConflictType::AddAdd =>
                "added by both you and master".to_string(),
            ConflictType::RenameRename =>
                "renamed differently by you and master".to_string(),
        };
        writeln!(summary, "- `{}`: {}", file.path, conflict_desc).unwrap();
    }

    summary
}
```

#### Generating Conflict Details

For each conflicted file, show the actual conflict with context:

```rust
fn generate_conflict_details(worktree: &Path, files: &[ConflictedFile]) -> String {
    let mut details = String::new();

    for file in files {
        writeln!(details, "---\n### `{}`\n", file.path).unwrap();

        match file.conflict_type {
            ConflictType::Content => {
                // Show the file with conflict markers and surrounding context
                let content = fs::read_to_string(worktree.join(&file.path))
                    .unwrap_or_default();
                let regions = extract_conflict_regions(&content);

                for (i, region) in regions.iter().enumerate() {
                    writeln!(details, "**Conflict {}:**\n", i + 1).unwrap();
                    writeln!(details, "```\n{}\n```\n", region).unwrap();
                }
            }
            ConflictType::ModifyDelete => {
                writeln!(details,
                    "This file was **modified** in your branch but **deleted** in master.\n\n\
                    Options:\n\
                    - Keep your version: `git add {}`\n\
                    - Accept deletion: `git rm {}`\n",
                    file.path, file.path
                ).unwrap();
            }
            ConflictType::AddAdd => {
                writeln!(details,
                    "This file was **added** by both your branch and master with different content.\n\n\
                    You'll need to merge the contents manually, then `git add {}`\n",
                    file.path
                ).unwrap();
            }
            ConflictType::RenameRename => {
                writeln!(details,
                    "This file was **renamed** differently in your branch vs master.\n\n\
                    Decide which name to use, resolve any content conflicts, then stage.\n"
                ).unwrap();
            }
        }
    }

    details
}

fn extract_conflict_regions(content: &str) -> Vec<String> {
    let mut regions = vec![];
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if lines[i].starts_with("<<<<<<<") {
            // Found conflict start, capture with context
            let start = i.saturating_sub(5); // 5 lines before
            let mut end = i;

            // Find conflict end
            while end < lines.len() && !lines[end].starts_with(">>>>>>>") {
                end += 1;
            }
            end = (end + 6).min(lines.len()); // 5 lines after

            let region: String = lines[start..end].join("\n");
            regions.push(region);

            i = end;
        } else {
            i += 1;
        }
    }

    regions
}
```

### Detecting Conflict Resolution Completion

The worker has successfully resolved conflicts when:

1. No conflict markers remain in any file
2. All previously-conflicted files are staged
3. `git rebase --continue` completes successfully
4. Validation passes (`just check`, `just clippy`)

#### Detection State Machine

```rust
pub enum RebaseResolutionState {
    /// Conflicts exist, waiting for worker to resolve
    Resolving,
    /// Worker ran `git rebase --continue`
    ContinueAttempted,
    /// Rebase completed successfully
    Completed,
    /// Rebase was aborted
    Aborted,
    /// Worker is stuck or confused
    Stuck,
}

impl RebaseStateDetector {
    pub fn detect(&self, worktree: &Path, pane_output: &str) -> RebaseResolutionState {
        // Check if rebase is still in progress
        let in_rebase = worktree.join(".git/rebase-merge").exists()
            || worktree.join(".git/rebase-apply").exists();

        if !in_rebase {
            // Rebase is no longer in progress
            if pane_output.contains("rebase --abort")
                || pane_output.contains("Rebase aborted")
            {
                return RebaseResolutionState::Aborted;
            }
            return RebaseResolutionState::Completed;
        }

        // Check for continue attempt indicators
        if pane_output.contains("rebase --continue")
            || pane_output.contains("Applying:")
        {
            // Attempted continue, but still in rebase = more conflicts or error
            if self.has_remaining_conflicts(worktree) {
                return RebaseResolutionState::Resolving;
            }
            return RebaseResolutionState::ContinueAttempted;
        }

        // Check for stuck indicators
        if self.is_stuck(pane_output) {
            return RebaseResolutionState::Stuck;
        }

        RebaseResolutionState::Resolving
    }

    fn has_remaining_conflicts(&self, worktree: &Path) -> bool {
        let output = Command::new("git")
            .args(["diff", "--check"])
            .current_dir(worktree)
            .output();

        match output {
            Ok(out) => !out.stdout.is_empty(),
            Err(_) => false,
        }
    }

    fn is_stuck(&self, output: &str) -> bool {
        // Worker asking for help or expressing confusion
        let stuck_indicators = [
            "don't know how to",
            "not sure how to proceed",
            "need help with",
            "I'm stuck",
            "I cannot resolve",
            "unclear how to",
        ];
        stuck_indicators.iter().any(|ind| output.to_lowercase().contains(ind))
    }
}
```

#### Output Pattern Detection

Monitor terminal output for resolution progress:

```rust
pub struct RebaseOutputMonitor {
    /// Patterns indicating progress
    progress_patterns: Vec<Regex>,
    /// Patterns indicating completion
    completion_patterns: Vec<Regex>,
    /// Patterns indicating failure/abort
    failure_patterns: Vec<Regex>,
}

impl RebaseOutputMonitor {
    pub fn new() -> Self {
        Self {
            progress_patterns: vec![
                Regex::new(r"Resolved conflicts? in").unwrap(),
                Regex::new(r"git add").unwrap(),
                Regex::new(r"Staged:").unwrap(),
            ],
            completion_patterns: vec![
                Regex::new(r"Successfully rebased").unwrap(),
                Regex::new(r"rebase.*complete").unwrap(),
                Regex::new(r"Applied:.*commits?").unwrap(),
                // Worker confirms completion
                Regex::new(r"conflicts? (have been|are) resolved").unwrap(),
                Regex::new(r"rebase (is )?complete").unwrap(),
            ],
            failure_patterns: vec![
                Regex::new(r"rebase --abort").unwrap(),
                Regex::new(r"Rebase aborted").unwrap(),
                Regex::new(r"Cannot continue").unwrap(),
                Regex::new(r"CONFLICT.*not resolved").unwrap(),
            ],
        }
    }

    pub fn classify(&self, output: &str) -> RebaseOutputEvent {
        for pattern in &self.failure_patterns {
            if pattern.is_match(output) {
                return RebaseOutputEvent::Failed;
            }
        }

        for pattern in &self.completion_patterns {
            if pattern.is_match(output) {
                return RebaseOutputEvent::Completed;
            }
        }

        for pattern in &self.progress_patterns {
            if pattern.is_match(output) {
                return RebaseOutputEvent::Progress;
            }
        }

        RebaseOutputEvent::None
    }
}

pub enum RebaseOutputEvent {
    None,
    Progress,
    Completed,
    Failed,
}
```

### Handling Resolution Failures

#### Failure Modes and Recovery

| Failure Mode | Detection | Recovery Action |
|--------------|-----------|-----------------|
| Worker aborts rebase | `git rebase --abort` in output | Reset to `needs_review`, notify user |
| Worker stuck | Stuck indicators + timeout | Send help prompt with specific guidance |
| Persistent conflicts | Multiple continue failures | Escalate to user via sound + status |
| Validation failure | `just check` fails post-rebase | Send error output, worker fixes |

#### Stuck Worker Recovery Prompt

```rust
const STUCK_RECOVERY_PROMPT: &str = r#"
## Conflict Resolution Help

It looks like you may be having trouble with the merge conflicts. Let me help:

### Current Status

{git_status_output}

### Common Issues

1. **Conflict markers still present**
   - Search for `<<<<<<<` in conflicted files
   - ALL markers must be removed, not just some

2. **Forgot to stage resolved files**
   - After fixing a file, run: `git add <filename>`
   - Check with: `git status`

3. **Unclear which version to keep**
   - `HEAD` or "ours" = master's version (what you're rebasing onto)
   - `REBASE_HEAD` or "theirs" = your version (what you're rebasing)

4. **Content conflict in generated files**
   - For `Cargo.lock`: delete and run `cargo build` to regenerate
   - For other generated files: regenerate after resolving source conflicts

### Next Steps

1. Run `git status` to see current state
2. For each file marked "both modified", open and resolve conflicts
3. Stage each resolved file with `git add`
4. When all resolved, run `git rebase --continue`

If you truly cannot resolve these conflicts, you may abort with:
`git rebase --abort`
This will return your branch to its pre-rebase state.
"#;
```

#### Failure State Transitions

```rust
pub fn handle_rebase_failure(
    worker: &mut Worker,
    failure: RebaseFailure,
) -> Result<WorkerTransition> {
    match failure {
        RebaseFailure::Aborted => {
            // Worker gave up, return to needs_review
            // User will need to decide next steps
            log::warn!("Worker {} aborted rebase", worker.name);
            play_alert_sound();
            Ok(WorkerTransition::ToNeedsReview {
                note: "Rebase aborted by worker".to_string(),
            })
        }

        RebaseFailure::Stuck { duration } => {
            if duration < Duration::from_secs(300) {
                // Under 5 minutes, send help prompt
                send_stuck_recovery_prompt(worker)?;
                Ok(WorkerTransition::None)
            } else {
                // Over 5 minutes stuck, escalate
                log::error!("Worker {} stuck on rebase for {:?}", worker.name, duration);
                play_alert_sound();
                Ok(WorkerTransition::ToError)
            }
        }

        RebaseFailure::ValidationFailed { output } => {
            // Post-rebase validation failed, worker needs to fix
            let prompt = format!(
                "The rebase completed but validation failed:\n\n```\n{}\n```\n\n\
                Please fix these issues and commit the fixes.",
                output
            );
            send_message_to_worker(worker, &prompt)?;
            // Stay in rebasing state until fixed
            Ok(WorkerTransition::None)
        }

        RebaseFailure::ConflictLoop { attempts } => {
            // Same conflicts keep appearing
            if attempts >= 3 {
                log::error!(
                    "Worker {} in conflict loop after {} attempts",
                    worker.name, attempts
                );
                play_alert_sound();
                Ok(WorkerTransition::ToError)
            } else {
                // Try more specific guidance
                send_detailed_conflict_help(worker)?;
                Ok(WorkerTransition::None)
            }
        }
    }
}

pub enum RebaseFailure {
    Aborted,
    Stuck { duration: Duration },
    ValidationFailed { output: String },
    ConflictLoop { attempts: u32 },
}
```

### Complete Rebase Flow

```rust
pub async fn execute_rebase(worker: &mut Worker) -> Result<RebaseOutcome> {
    // 1. Mark worker as rebasing
    worker.status = Status::Rebasing;
    save_state()?;

    // 2. Fetch latest master
    git::fetch(&worker.worktree_path, "origin")?;

    // 3. Attempt rebase
    let rebase_result = git::rebase(&worker.worktree_path, "master")?;

    if rebase_result.success {
        // 4a. Clean rebase - run validation
        let validation = run_validation(&worker.worktree_path)?;
        if validation.success {
            worker.status = Status::NeedsReview;
            save_state()?;
            return Ok(RebaseOutcome::Success);
        } else {
            // Validation failed, send to worker
            send_validation_failure(worker, &validation)?;
            return Ok(RebaseOutcome::ValidationFailed);
        }
    }

    // 4b. Conflicts detected - send resolution prompt to worker
    let prompt = build_conflict_prompt(&rebase_result)?;

    // Send /clear first to reset context
    send_clear_command(worker)?;
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Send conflict resolution prompt
    send_message_to_worker(worker, &prompt)?;

    // 5. Monitor for resolution
    let monitor = RebaseOutputMonitor::new();
    let mut stuck_start: Option<Instant> = None;

    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;

        let state = detect_rebase_state(&worker.worktree_path)?;
        let output = capture_recent_output(worker)?;
        let event = monitor.classify(&output);

        match (state, event) {
            (RebaseResolutionState::Completed, _) |
            (_, RebaseOutputEvent::Completed) => {
                // Rebase completed, run validation
                let validation = run_validation(&worker.worktree_path)?;
                if validation.success {
                    worker.status = Status::NeedsReview;
                    save_state()?;
                    play_success_sound();
                    return Ok(RebaseOutcome::Success);
                } else {
                    handle_rebase_failure(worker, RebaseFailure::ValidationFailed {
                        output: validation.output,
                    })?;
                }
            }

            (RebaseResolutionState::Aborted, _) |
            (_, RebaseOutputEvent::Failed) => {
                handle_rebase_failure(worker, RebaseFailure::Aborted)?;
                return Ok(RebaseOutcome::Aborted);
            }

            (RebaseResolutionState::Stuck, _) => {
                let duration = stuck_start
                    .get_or_insert(Instant::now())
                    .elapsed();
                handle_rebase_failure(worker, RebaseFailure::Stuck { duration })?;
            }

            (RebaseResolutionState::Resolving, _) => {
                // Still working, reset stuck timer on progress
                if matches!(event, RebaseOutputEvent::Progress) {
                    stuck_start = None;
                }
            }

            _ => {
                // Continue monitoring
            }
        }
    }
}

pub enum RebaseOutcome {
    Success,
    Aborted,
    ValidationFailed,
    Error(String),
}
```

### Worker State During Rebasing

The `rebasing` status is special:
- Worker cannot receive new tasks
- Worker cannot be reviewed (changes are incomplete)
- Patrol monitors for resolution completion
- Timeouts trigger escalation

```rust
// State file entry during rebasing
{
    "name": "adam",
    "status": "rebasing",
    "current_prompt": "Merge conflict resolution...",
    "rebase_started_at_unix": 1704567890,
    "conflicted_files": ["src/foo.rs", "Cargo.toml"],
    "resolution_attempts": 1
}
```

### `llmc doctor`

Runs health checks on the system.

```bash
llmc doctor
```

**Checks**:
- All required binaries present
- State file valid
- All workers have matching TMUX sessions
- All worktrees exist and are clean
- Git configuration correct
- No orphaned branches

## Architecture

### Module Layout

```
src/llmc/
├── Cargo.toml
└── src/
    ├── main.rs           # CLI entrypoint
    ├── cli.rs            # clap command definitions
    ├── config.rs         # Configuration loading
    ├── state.rs          # State file operations
    ├── tmux/
    │   ├── mod.rs
    │   ├── session.rs    # Session management
    │   ├── sender.rs     # Reliable input sending
    │   └── monitor.rs    # Output monitoring
    ├── patrol.rs         # Patrol system
    ├── git.rs            # Git operations
    ├── worker.rs         # Worker lifecycle
    ├── commands/
    │   ├── mod.rs
    │   ├── init.rs
    │   ├── up.rs
    │   ├── down.rs
    │   ├── add.rs
    │   ├── nuke.rs
    │   ├── status.rs
    │   ├── start.rs
    │   ├── message.rs
    │   ├── attach.rs
    │   ├── review.rs
    │   ├── reject.rs
    │   ├── accept.rs
    │   ├── rebase.rs
    │   └── doctor.rs
    └── sound.rs          # Terminal bell/sounds
```

### Dependencies

```toml
[dependencies]
# CLI
clap = { version = "4", features = ["derive"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Error handling
anyhow = "1"
thiserror = "1"

# TMUX interface
tmux_interface = "0.3"

# Regex for output parsing
regex = "1"

# Signal handling
ctrlc = "3"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Error Handling

All fallible operations use `anyhow::Result` with context:

```rust
self.tmux.send(&session, &prompt)
    .with_context(|| format!("Failed to send prompt to worker {}", name))?;
```

User-facing errors should include remediation hints:

```rust
if !self.tmux.has_session(&session_id)? {
    return Err(anyhow!(
        "Session {} not found. Run 'llmc up' to start workers.",
        session_id
    ));
}
```

## Sound Notifications

When a worker enters `needs_review` state, LLMC plays a terminal bell:

```rust
fn notify_needs_review(&self, worker: &str) {
    if self.config.sound_on_review {
        // Terminal bell
        print!("\x07");
        io::stdout().flush().ok();
    }
}
```

## Learnings from V1 and Gastown

### From LLMC V1

1. **Atomic state updates**: Use temp file + rename for state.json
2. **Commit message cleaning**: Strip "Generated with" and "Co-Authored-By"
3. **Tabula.xlsm handling**: Copy gitignored asset to worktrees
4. **Single-commit workflow**: Squash before merge

### From Gastown

1. **Debounce patterns**: 500ms base + 100ms per KB, capped at 2000ms
2. **Enter key retry**: 3 attempts with 200ms delays
3. **Claude state detection**: Check pane command for "node", "claude", or version
4. **Prompt readiness**: Poll for ">" at line start
5. **Bypass permissions dialog**: Send Down + Enter sequence
6. **Session identity**: Environment variables for worker identification
7. **Crash hooks**: Use tmux pane-died hook for detection
8. **Non-fatal startup**: Many configuration steps are best-effort

### Key Files in Gastown for Reference

- Code lives in `~/gastown`
- `internal/tmux/tmux.go`: Core TMUX wrapper (SendKeysDebounced, NudgeSession, WaitForClaudeReady)
- `internal/polecat/session_manager.go`: Session lifecycle (Start, Stop, Inject)
- `internal/agent/state.go`: Generic state management patterns
- `internal/witness/protocol.go`: Protocol-based message classification

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

## Security Considerations

1. **Skip permissions mode**: Workers use `--dangerously-skip-permissions` by
   default. This is acceptable because:
   - Workers operate in isolated worktrees
   - No network access to remotes (local clone only)
   - Human review required before merge to master

2. **Prompt injection**: Prompts are sent via tmux literal mode (`-l` flag) to
   prevent interpretation of special characters.

3. **State file**: Contains prompts and paths. Should not contain secrets.

## State Detection Heuristics (Refined Design)

This section provides detailed design for Claude state detection, building on the basic
patterns in the State Detection section above. The goal is reliable detection of all
Claude states needed for autonomous worker management.

### Design Principles

1. **Multi-Signal Detection**: Use multiple signals (pane command, output patterns,
   timing) to avoid false positives/negatives
2. **Hierarchical Classification**: Check states in priority order (crash > confirmation
   > question > ready > processing)
3. **Confidence Scoring**: Some detections are high-confidence (process exit), others
   need corroboration (output patterns)
4. **Stateful Context**: Track previous states to improve classification accuracy

### ClaudeState Enum (Extended)

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

### Detection Heuristics

#### 1. Process Health Check (Highest Priority)

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

#### 2. Ready State Detection

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

#### 3. AskUserQuestion Detection (Multiple Choice Questions)

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

#### 4. Tool Permission Prompt Detection

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

#### 5. Distinguishing Completed vs. Waiting for Input

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

#### 6. Crash vs. Normal Exit Detection

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

#### 7. Error State Detection

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

### Complete State Detection Flow

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

### Handling Detected States

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

### Bypassing Permissions Dialog at Startup

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

### Considerations for --output-format

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

## Future Considerations

1. **Multi-project support**: Currently hardcoded to dreamtides layout
2. **Remote workers**: Could extend to SSH-based TMUX sessions
3. **Web UI**: Dashboard for monitoring workers
4. **Prompt templates**: Reusable prompt fragments
5. **Automatic retry**: Re-run failed tasks with modified prompts
