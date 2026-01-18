# LLMC Hooks Migration: Technical Project Plan

## Executive Summary

This document outlines a migration plan for LLMC to use Claude Code Hooks for
worker state detection, replacing the current polling-based architecture that
relies on TMUX pane content analysis and git output parsing.

**Feasibility Assessment: High Complexity, Significant Benefits**

The migration is feasible but represents a fundamental architectural shift. The
current 30-second polling system would be replaced by an event-driven
architecture where Claude Code hooks notify the LLMC daemon of state changes in
real-time. This eliminates race conditions, reduces latency, and improves
reliability.

**Can this be implemented by an Opus 4.5 agent in a single session?**

No. This migration requires 4-5 separate milestones due to:
- Fundamental architectural changes spanning 15+ files
- Need for incremental testing between phases
- Risk of breaking existing functionality if done all at once
- Complex IPC mechanism design and implementation
- Backward compatibility considerations during transition

---

## Part 1: Current Architecture Analysis

### 1.1 Current State Detection Mechanisms

LLMC currently detects worker state through three mechanisms:

| Mechanism | Detection Method | Latency | Reliability |
|-----------|------------------|---------|-------------|
| **Patrol Polling** | 30s periodic checks | High (0-30s) | Medium |
| **TMUX Pane Capture** | `capture_pane` + regex | Medium | Fragile |
| **Git State Analysis** | `git status`, `git log`, filesystem | Low | High |

### 1.2 State Detection Points

```
Current Detection Flow:

patrol.rs (every 30s)
    │
    ├── check_session_health()
    │   └── TMUX: get_pane_command() → is_claude_process() / is_shell()
    │
    ├── detect_state_transitions()
    │   └── GIT: has_commits_ahead_of(), has_uncommitted_changes()
    │
    ├── detect_rebasing_transition()
    │   ├── GIT: is_rebase_in_progress()
    │   └── TMUX: capture_pane() → "Successfully rebased"
    │
    └── detect_reviewing_amendments()
        └── GIT: get_head_sha() != worker.commit_sha
```

### 1.3 Files Requiring Modification

**Core Files (Major Changes):**
- `patrol.rs` - Complete redesign from polling to event-driven
- `worker.rs` - New transition trigger mechanism
- `state.rs` - Add hook-related fields
- `tmux/session.rs` - Remove pane capture logic
- `commands/up.rs` - Add IPC listener

**Supporting Files (Minor Changes):**
- `config.rs` - Add hook configuration
- `recovery.rs` - Update recovery to use hooks
- `logging/` - Add hook event logging
- All command files using `wait_for_claude_ready()`

---

## Part 2: Claude Hooks Mapping

### 2.1 Hook-to-State Mapping

| Claude Hook | Current Detection | Replaces |
|-------------|-------------------|----------|
| **Stop** | Commit detection via git polling | Working → NeedsReview transition |
| **SubagentStop** | N/A | Task subagent completion |
| **SessionStart** | TMUX session creation | Worker startup confirmation |
| **SessionEnd** | TMUX `get_pane_command()` | Crash/shutdown detection |
| **PostToolUse(Bash)** | Git status polling | `git commit` detection |
| **Notification** | N/A | Claude-generated alerts |

### 2.2 Hook Configuration Design

Each LLMC worker will have hooks configured in their worktree:

```json
// .claude/settings.json (per-worktree)
{
  "hooks": {
    "Stop": [{
      "hooks": [{
        "type": "command",
        "command": "llmc-hook stop --worker $LLMC_WORKER_NAME"
      }]
    }],
    "SessionStart": [{
      "matcher": "startup",
      "hooks": [{
        "type": "command",
        "command": "llmc-hook session-start --worker $LLMC_WORKER_NAME"
      }]
    }],
    "SessionEnd": [{
      "hooks": [{
        "type": "command",
        "command": "llmc-hook session-end --worker $LLMC_WORKER_NAME --reason $reason"
      }]
    }],
    "PostToolUse": [{
      "matcher": "Bash",
      "hooks": [{
        "type": "command",
        "command": "llmc-hook post-bash --worker $LLMC_WORKER_NAME"
      }]
    }]
  }
}
```

### 2.3 IPC Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                          LLMC Daemon (llmc up)                        │
│                                                                      │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐   │
│  │  Event Listener │◄───│   Event Queue   │◄───│   IPC Socket    │   │
│  │   (async task)  │    │   (bounded)     │    │ /tmp/llmc.sock  │   │
│  └────────┬────────┘    └─────────────────┘    └────────┬────────┘   │
│           │                                              │           │
│           ▼                                              │           │
│  ┌─────────────────┐                                     │           │
│  │  State Machine  │                                     │           │
│  │   Transitions   │                                     │           │
│  └─────────────────┘                                     │           │
└──────────────────────────────────────────────────────────┼───────────┘
                                                           │
              ┌────────────────────────────────────────────┤
              │                                            │
    ┌─────────▼─────────┐                        ┌─────────▼─────────┐
    │   llmc-hook CLI   │                        │   llmc-hook CLI   │
    │   (worker adam)   │                        │  (worker baker)   │
    └─────────▲─────────┘                        └─────────▲─────────┘
              │                                            │
    ┌─────────┴─────────┐                        ┌─────────┴─────────┐
    │  Claude Hooks     │                        │  Claude Hooks     │
    │  (adam session)   │                        │  (baker session)  │
    └───────────────────┘                        └───────────────────┘
```

---

## Part 3: Migration Strategy

### 3.1 Phased Approach

The migration is structured as **dual-mode operation** during transition:

1. **Phase 1**: Hook infrastructure (IPC, CLI, config generation)
2. **Phase 2**: Session lifecycle hooks (SessionStart, SessionEnd)
3. **Phase 3**: Stop hook for task completion detection
4. **Phase 4**: PostToolUse for git commit detection
5. **Phase 5**: Remove polling fallback, cleanup

### 3.2 Backward Compatibility

During migration, both systems run in parallel:

```rust
pub struct PatrolConfig {
    /// Use hooks for state detection (new system)
    pub use_hooks: bool,
    /// Fallback to polling when hooks don't fire (transition mode)
    pub polling_fallback: bool,
    /// Polling interval (only used in fallback mode)
    pub fallback_interval_secs: u32,
}
```

---

## Part 4: Implementation Milestones

### Milestone 1: Hook Infrastructure Foundation

**Goal:** Build the IPC mechanism and `llmc-hook` helper CLI.

**Scope:**
- Create Unix domain socket listener in daemon
- Implement `llmc-hook` binary for hook callbacks
- Design event message protocol (JSON over socket)
- Add hook configuration generation on worker creation

**Files to Create:**
- `src/ipc/mod.rs` - IPC module root
- `src/ipc/socket.rs` - Unix socket listener/client
- `src/ipc/messages.rs` - Event message types
- `src/bin/llmc-hook.rs` - Hook callback CLI

**Files to Modify:**
- `lib.rs` - Add ipc module
- `commands/up.rs` - Start IPC listener
- `commands/add.rs` - Generate hook configuration
- `config.rs` - Add hook-related settings

**Estimated Complexity:** Medium-High
**Can be done in one session:** Yes

**Key Implementation Details:**

```rust
// src/ipc/messages.rs
#[derive(Serialize, Deserialize)]
pub enum HookEvent {
    SessionStart {
        worker: String,
        session_id: String,
        timestamp: u64,
    },
    SessionEnd {
        worker: String,
        reason: String,
        timestamp: u64,
    },
    Stop {
        worker: String,
        session_id: String,
        timestamp: u64,
    },
    PostBash {
        worker: String,
        command: String,
        exit_code: i32,
        timestamp: u64,
    },
}

// src/ipc/socket.rs
pub struct IpcListener {
    socket_path: PathBuf,
    listener: UnixListener,
}

impl IpcListener {
    pub async fn accept_events(&self) -> impl Stream<Item = HookEvent> {
        // Accept connections, parse JSON events
    }
}
```

---

### Milestone 2: Session Lifecycle Hooks

**Goal:** Replace TMUX process detection with SessionStart/SessionEnd hooks.

**Scope:**
- Implement SessionStart hook handling (worker ready confirmation)
- Implement SessionEnd hook handling (crash/shutdown detection)
- Remove `wait_for_claude_ready()` polling
- Remove `check_session_health()` TMUX process checks

**Files to Modify:**
- `patrol.rs` - Remove session health polling, add hook event handling
- `worker.rs` - Update worker startup flow
- `tmux/session.rs` - Remove `wait_for_claude_ready()`, `is_claude_process()`
- `recovery.rs` - Update crash detection

**State Machine Changes:**

```
Before (polling):
  Worker created → Poll TMUX for Claude process → Mark Idle

After (hooks):
  Worker created → Wait for SessionStart event → Mark Idle
  SessionEnd event → Mark Offline (with reason)
```

**Estimated Complexity:** Medium
**Can be done in one session:** Yes

**Key Implementation Details:**

```rust
// patrol.rs (new event handler)
async fn handle_hook_event(&self, event: HookEvent, state: &mut State) -> Result<()> {
    match event {
        HookEvent::SessionStart { worker, .. } => {
            if let Some(w) = state.get_worker_mut(&worker) {
                if w.status == WorkerStatus::Offline {
                    w.transition_to(WorkerStatus::Idle)?;
                    info!(worker = %worker, "Worker ready (SessionStart hook)");
                }
            }
        }
        HookEvent::SessionEnd { worker, reason, .. } => {
            if let Some(w) = state.get_worker_mut(&worker) {
                w.transition_to(WorkerStatus::Offline)?;
                info!(worker = %worker, reason = %reason, "Worker offline (SessionEnd hook)");
            }
        }
        _ => {}
    }
    Ok(())
}
```

---

### Milestone 3: Task Completion Detection (Stop Hook)

**Goal:** Replace git commit polling with Stop hook for detecting task completion.

**Scope:**
- Implement Stop hook handling for Working → NeedsReview transition
- Add intelligent completion detection (prompt-based hook option)
- Handle self-review phase triggering
- Remove `detect_state_transitions()` polling

**Files to Modify:**
- `patrol.rs` - Replace commit polling with Stop event handling
- `worker.rs` - Update transition logic for hook-triggered completion
- `state.rs` - Add `pending_self_review` flag for hook-based triggering

**Detection Logic:**

```rust
// Stop hook handler
HookEvent::Stop { worker, .. } => {
    if let Some(w) = state.get_worker_mut(&worker) {
        match w.status {
            WorkerStatus::Working | WorkerStatus::Rejected => {
                // Check if there are commits to review
                let has_commits = git::has_commits_ahead_of(&w.worktree_path, "origin/master")?;
                if has_commits {
                    let sha = git::get_head_sha(&w.worktree_path)?;
                    w.transition_to_needs_review(sha)?;

                    if w.self_review {
                        // Queue self-review prompt (handled in next patrol tick)
                        w.pending_self_review = true;
                    }
                }
            }
            WorkerStatus::Reviewing => {
                // Self-review complete, check for amendments
                let current_sha = git::get_head_sha(&w.worktree_path)?;
                if current_sha != w.commit_sha.as_deref().unwrap_or("") {
                    w.commit_sha = Some(current_sha);
                }
                w.transition_to(WorkerStatus::NeedsReview)?;
            }
            _ => {}
        }
    }
}
```

**Estimated Complexity:** High
**Can be done in one session:** Possibly, but risky

**Note:** This milestone has the highest risk because it changes core task
completion detection. Consider keeping git polling as a fallback initially.

---

### Milestone 4: PostToolUse Hook for Git Operations

**Goal:** Detect git commits in real-time instead of polling.

**Scope:**
- Implement PostToolUse(Bash) hook to detect `git commit`
- Detect `git rebase --continue` completion
- Detect `git rebase --abort`
- Remove git polling from patrol

**Files to Modify:**
- `patrol.rs` - Add PostBash event handling
- `git.rs` - Simplify to just state queries (no polling helpers)

**Command Detection Patterns:**

```rust
fn is_git_commit_command(command: &str) -> bool {
    command.contains("git commit") ||
    command.contains("git rebase --continue")
}

fn is_git_abort_command(command: &str) -> bool {
    command.contains("git rebase --abort")
}

// PostBash handler
HookEvent::PostBash { worker, command, exit_code, .. } => {
    if exit_code == 0 {
        if is_git_commit_command(&command) {
            // Trigger state transition check
            handle_possible_commit(&worker, state)?;
        } else if is_git_abort_command(&command) {
            // Rebase aborted, may need to handle
            handle_rebase_abort(&worker, state)?;
        }
    }
}
```

**Estimated Complexity:** Medium
**Can be done in one session:** Yes

---

### Milestone 5: Cleanup and Optimization

**Goal:** Remove all polling fallbacks and legacy detection code.

**Scope:**
- Remove polling loop from patrol (keep only event handling)
- Remove TMUX pane capture code
- Remove all `wait_for_*` polling functions
- Update documentation
- Performance optimization

**Files to Modify:**
- `patrol.rs` - Remove polling, pure event-driven
- `tmux/session.rs` - Remove `capture_pane()`, detection regexes
- `worker.rs` - Clean up legacy startup flow
- `config.rs` - Remove polling-related settings
- `docs/llmc.md` - Update architecture documentation

**Estimated Complexity:** Medium
**Can be done in one session:** Yes

---

## Part 5: Detailed Technical Specifications

### 5.1 Event Message Protocol

```rust
/// Message sent from hook CLI to daemon
#[derive(Serialize, Deserialize)]
pub struct HookMessage {
    /// Protocol version for forward compatibility
    pub version: u8,
    /// Unique message ID for deduplication
    pub id: Uuid,
    /// The hook event data
    pub event: HookEvent,
}

/// Response from daemon to hook CLI
#[derive(Serialize, Deserialize)]
pub struct HookResponse {
    /// Whether the event was processed
    pub success: bool,
    /// Optional error message
    pub error: Option<String>,
}
```

### 5.2 Socket Protocol

- Path: `/tmp/llmc-{uid}.sock` (per-user isolation)
- Protocol: Line-delimited JSON (one message per line)
- Timeout: 5 seconds for hook CLI (hooks have finite timeouts)
- Retry: 3 attempts with exponential backoff

### 5.3 Hook Configuration Template

```json
{
  "hooks": {
    "Stop": [{
      "hooks": [{
        "type": "command",
        "command": "{{LLMC_BIN}} hook stop --socket {{SOCKET_PATH}} --worker {{WORKER_NAME}}",
        "timeout": 5
      }]
    }],
    "SessionStart": [{
      "matcher": "startup",
      "hooks": [{
        "type": "command",
        "command": "{{LLMC_BIN}} hook session-start --socket {{SOCKET_PATH}} --worker {{WORKER_NAME}}",
        "timeout": 5
      }]
    }],
    "SessionEnd": [{
      "hooks": [{
        "type": "command",
        "command": "{{LLMC_BIN}} hook session-end --socket {{SOCKET_PATH}} --worker {{WORKER_NAME}} --reason \"$reason\"",
        "timeout": 5
      }]
    }],
    "PostToolUse": [{
      "matcher": "Bash",
      "hooks": [{
        "type": "command",
        "command": "{{LLMC_BIN}} hook post-bash --socket {{SOCKET_PATH}} --worker {{WORKER_NAME}}",
        "timeout": 5
      }]
    }]
  }
}
```

### 5.4 Daemon Event Loop (Final Architecture)

```rust
pub async fn run_daemon(config: Config, state: State) -> Result<()> {
    let ipc = IpcListener::bind(&config.socket_path)?;
    let mut state = state;

    loop {
        tokio::select! {
            // Handle hook events (primary state detection)
            Some(event) = ipc.next_event() => {
                handle_hook_event(event, &mut state, &config).await?;
                save_state(&state)?;
            }

            // Periodic maintenance (not state detection)
            _ = maintenance_interval.tick() => {
                // Rebase pending reviews onto master
                rebase_pending_reviews(&mut state, &config).await?;
                // Send queued self-review prompts
                send_pending_self_reviews(&mut state, &config).await?;
                save_state(&state)?;
            }

            // Graceful shutdown
            _ = shutdown_signal() => {
                info!("Shutting down daemon");
                break;
            }
        }
    }

    Ok(())
}
```

---

## Part 6: Risk Assessment

### 6.1 Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Hook timeout causes missed events | Medium | High | Retry mechanism + fallback polling |
| IPC socket unavailable | Low | High | Graceful degradation to polling |
| Hook configuration overwritten | Medium | Medium | Regenerate on `llmc up` |
| Race conditions in event handling | Medium | Medium | Event queuing + sequential processing |
| Claude Code hook API changes | Low | High | Version pinning + abstraction layer |

### 6.2 Operational Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Debugging harder with async events | Medium | Medium | Comprehensive event logging |
| Users with custom hook configs | Low | Medium | Merge LLMC hooks with existing |
| Performance regression | Low | Low | Benchmark before/after |

### 6.3 Rollback Strategy

Each milestone includes a feature flag for rollback:

```toml
[defaults]
# Feature flags for hook migration
hooks_session_lifecycle = true   # Milestone 2
hooks_task_completion = true     # Milestone 3
hooks_git_detection = true       # Milestone 4
polling_fallback = false         # Disable after Milestone 5
```

---

## Part 7: Testing Strategy

### 7.1 Unit Tests

- IPC message serialization/deserialization
- Hook event handling logic
- State machine transitions from hook events
- Socket connection handling

### 7.2 Integration Tests

- Full hook → daemon → state change flow
- Multiple workers with concurrent events
- Hook timeout handling
- Socket reconnection after daemon restart

### 7.3 End-to-End Tests

- Create worker → start task → complete task → review cycle
- Crash recovery via SessionEnd hook
- Rebase during task with PostBash detection

---

## Part 8: Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| State detection latency | 0-30s | <1s |
| CPU usage (idle daemon) | ~5% | <1% |
| Missed state transitions | ~2%/day | 0 |
| Lines of code (detection) | ~1200 | ~600 |

---

## Part 9: Timeline Estimate

| Milestone | Estimated Effort | Dependencies |
|-----------|------------------|--------------|
| 1: Hook Infrastructure | 1 session | None |
| 2: Session Lifecycle | 1 session | Milestone 1 |
| 3: Task Completion | 1-2 sessions | Milestone 2 |
| 4: Git Detection | 1 session | Milestone 3 |
| 5: Cleanup | 1 session | Milestone 4 |

**Total: 5-6 agent sessions**

---

## Appendix A: Current Files to Modify/Remove

### Files with Major Changes
- `patrol.rs` (590 lines) → Event-driven rewrite
- `worker.rs` (280 lines) → Hook-triggered transitions
- `tmux/session.rs` (350 lines) → Remove pane capture

### Files to Create
- `ipc/mod.rs`
- `ipc/socket.rs` (~200 lines)
- `ipc/messages.rs` (~100 lines)
- `bin/llmc-hook.rs` (~150 lines)

### Files to Delete (Milestone 5)
- Pane capture regex patterns
- Polling helper functions
- TMUX ready-wait loops

---

## Appendix B: Hook Input JSON Examples

### Stop Hook Input
```json
{
  "session_id": "abc123",
  "transcript_path": "/Users/user/.claude/projects/llmc/transcript.json",
  "cwd": "/Users/user/llmc/.worktrees/adam",
  "hook_event_name": "Stop"
}
```

### PostToolUse (Bash) Input
```json
{
  "session_id": "abc123",
  "tool_name": "Bash",
  "tool_input": {
    "command": "git commit -m 'Fix bug'",
    "description": "Commit changes"
  },
  "tool_response": "...",
  "hook_event_name": "PostToolUse"
}
```

### SessionEnd Input
```json
{
  "session_id": "abc123",
  "cwd": "/Users/user/llmc/.worktrees/adam",
  "hook_event_name": "SessionEnd",
  "reason": "logout"
}
```
