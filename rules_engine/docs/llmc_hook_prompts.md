# LLMC Hooks Migration: Session Prompts

This document contains prompts for each Opus session needed to implement the
LLMC hooks migration. See `llmc_hooks.md` for the full technical design.

---

## Session 1: Hook Infrastructure Foundation

### Task: Implement LLMC Hook Infrastructure (Milestone 1)

See `@rules_engine/docs/llmc.md` for background.

See `@rules_engine/docs/llmc_hooks.md` for the full migration design. You are
implementing Milestone 1: Hook Infrastructure Foundation.

### Goal

Build the IPC mechanism that allows Claude Code hooks to notify the LLMC daemon
of state changes. This is foundational infrastructure - no existing detection
code changes yet.

### Deliverables

**1. Create IPC module (`src/llmc/src/ipc/`)**

- `mod.rs` - Module declarations
- `messages.rs` - Define `HookEvent` enum and `HookMessage`/`HookResponse`
  structs (see design doc Section 5.1)
- `socket.rs` - Unix domain socket listener that accepts JSON messages

Socket requirements:
- Path: `~/llmc/llmc.sock` (inside LLMC directory, not /tmp)
- Line-delimited JSON protocol
- Non-blocking async with tokio
- Handle multiple concurrent connections
- 5-second read timeout per connection

**2. Create hook subcommand (`src/llmc/src/commands/hook.rs`)**

Add `llmc hook <event> --worker <name>` subcommand that sends events to the
daemon socket. Subcommands needed:

```
llmc hook stop --worker adam
llmc hook session-start --worker adam
llmc hook session-end --worker adam --reason <reason>
llmc hook post-bash --worker adam
```

Each reads hook context from stdin (JSON from Claude), extracts relevant fields,
sends to socket, waits for ack.

**3. Modify `commands/up.rs`**

Start the IPC listener as an async task when daemon starts. For now, just log
received events - don't process them yet.

**4. Modify `commands/add.rs`**

When creating a worker, generate `.claude/settings.json` in the worktree with
hook configuration pointing to `llmc hook` commands. Use the template from
design doc Section 5.3.

**5. Update `lib.rs` and `cli.rs`**

Add module declarations and CLI definitions for the new code.

### Implementation Notes

- Follow existing code patterns in the codebase (error handling, logging style)
- Use `anyhow` for errors, `tracing` for logging
- The daemon already uses tokio - integrate with existing async runtime
- Socket listener should be a spawned task, not blocking the main loop
- Hook CLI should exit quickly (hooks have timeouts)

### Out of Scope

- Do NOT modify `patrol.rs` or state detection logic
- Do NOT remove any existing TMUX/git detection code
- Do NOT change worker state machine transitions

### Validation

After implementation:
1. `just check` and `just clippy` and `just review` pass
2. `llmc up` starts and creates socket file
3. `llmc add testworker` creates worker with `.claude/settings.json` in worktree
4. Manual test: send JSON to socket via netcat, see it logged by daemon

---

## Session 2: Session Lifecycle Hooks

### Task: Implement SessionStart/SessionEnd Hook Handling (Milestone 2)

See `@rules_engine/docs/llmc_hooks.md` for context. You are implementing
Milestone 2. Milestone 1 (IPC infrastructure) is complete.

### Goal

Replace TMUX process detection with SessionStart/SessionEnd hooks for detecting
when workers come online or crash/shutdown.

### Deliverables

**1. Add event handler in `patrol.rs`**

Create `handle_hook_event()` function that processes `HookEvent` variants:

- `SessionStart` → If worker is Offline, transition to Idle
- `SessionEnd` → Transition worker to Offline, log the reason

Wire this into the daemon's event loop (in `up.rs`) so hook events trigger state
changes.

**2. Modify worker startup in `worker.rs`**

Current flow polls TMUX for Claude readiness. Change to:
- Start TMUX session as before
- Instead of polling `wait_for_claude_ready()`, mark worker as Offline
- Let SessionStart hook trigger the Idle transition

Keep `wait_for_claude_ready()` but make it optional/fallback (feature flag).

**3. Update `state.rs`**

Add feature flag field to config or state:
```rust
pub hooks_session_lifecycle: bool  // default: true
```

When false, use legacy TMUX polling. When true, use hooks.

**4. Remove/gate TMUX process polling in `patrol.rs`**

The `check_session_health()` function currently polls TMUX to detect crashes.
Gate this behind the feature flag - when hooks are enabled, rely on SessionEnd
instead.

**5. Update crash recovery in `recovery.rs`**

Crash detection should work with SessionEnd hook's `reason` field. Map reasons:
- `"logout"` → Normal shutdown
- `"other"` → Possible crash, increment crash_count

### Implementation Notes

- Keep both code paths working during transition (feature flag)
- SessionEnd with reason="logout" is normal; reason="other" may indicate crash
- Test by killing a Claude process and verifying SessionEnd fires

### Out of Scope

- Do NOT change task completion detection (Working → NeedsReview)
- Do NOT modify git-based detection
- Do NOT remove `capture_pane()` yet (used elsewhere)

### Validation

1. `just check` and `just clippy` and `just review` pass
2. Start daemon, add worker - worker becomes Idle via SessionStart hook
3. Kill Claude process in worker - worker becomes Offline via SessionEnd hook
4. Feature flag `hooks_session_lifecycle: false` falls back to TMUX polling

---

## Session 3: Task Completion Detection (Stop Hook)

### Task: Implement Stop Hook for Task Completion (Milestone 3)

See `@rules_engine/docs/llmc.md` for background.

See `@rules_engine/docs/llmc_hooks.md` for context. Milestones 1-2 are complete.

### Goal

Replace git commit polling with the Stop hook for detecting when a worker
finishes a task and should transition to NeedsReview.

### Deliverables

**1. Handle Stop event in `patrol.rs`**

When Stop hook fires for a Working or Rejected worker:
1. Check `git::has_commits_ahead_of(worktree, "origin/master")`
2. If commits exist, get HEAD SHA and transition to NeedsReview
3. If `worker.self_review` is true, set `worker.pending_self_review = true`

When Stop fires for a Reviewing worker:
1. Compare current HEAD SHA with stored `commit_sha`
2. If different, worker amended their commit - update SHA
3. Transition to NeedsReview (self-review complete)

**2. Add `pending_self_review` field to `WorkerRecord`**

```rust
pub pending_self_review: bool,
```

This queues the self-review prompt to be sent on next maintenance tick.

**3. Modify `detect_state_transitions()` in `patrol.rs`**

Gate the existing git polling behind a feature flag:
```rust
pub hooks_task_completion: bool  // default: true
```

When hooks enabled, skip the polling - rely on Stop hook instead.

**4. Update self-review prompt sending**

Current code sends self-review after a delay from `last_activity_unix`. Change
to check `pending_self_review` flag instead (set by Stop hook handler).

**5. Handle edge cases**

- Stop fires but no commits: Worker is still thinking, do nothing
- Stop fires with uncommitted changes: Amend to existing commit first
- Multiple Stop events: Deduplicate via timestamp or state check

### Implementation Notes

- The Stop hook fires every time Claude finishes responding, not just on task
  completion. Only transition if there are actual commits.
- Keep git polling as fallback for reliability during transition
- Stop hook may fire multiple times per task - handle idempotently

### Out of Scope

- Do NOT implement PostToolUse detection yet (Milestone 4)
- Do NOT remove TMUX pane capture code
- Do NOT change rebase detection

### Validation

1. `just check` and `just clippy` and `just review` pass
2. Start worker on task, let it complete - Stop hook triggers NeedsReview
3. With self_review enabled, Stop triggers Reviewing → NeedsReview flow
4. Feature flag `hooks_task_completion: false` uses git polling fallback
5. Stop firing without commits does not cause spurious transitions

---

## ~~Session 4: Git Operation Detection (PostToolUse Hook)~~ (Deprecated)

**This session has been deprecated and should be skipped.**

See the rationale in `@rules_engine/docs/llmc_hooks.md` Milestone 4 (Deprecated).

**Summary:** The Stop hook (Milestone 3) already provides correct task completion
detection. Triggering state transitions on individual git commands would cause
spurious transitions while the agent is still working. The semantic question
"Is the agent done?" is correctly answered by the Stop hook, not by "Did the
agent run `git commit`?"

**Proceed directly to Session 4: Cleanup and Finalization.**

---

## Session 4: Cleanup and Finalization

### Task: Remove Polling Fallbacks and Legacy Code (Milestone 4)

See `@rules_engine/docs/llmc.md` for background.

See `@rules_engine/docs/llmc_hooks.md` for context. Milestones 1-3 are complete
and tested. The original Milestone 4 (PostToolUse for git) was deprecated.

### Goal

Remove polling-based detection code and TMUX pane capture logic now that hooks
provide reliable state detection.

### Deliverables

**1. Remove feature flags and legacy code paths**

Delete:
- `hooks_session_lifecycle` flag and polling fallback
- `hooks_task_completion` flag and git polling fallback

Keep only the hook-based detection paths.

**2. Clean up `tmux/session.rs`**

Remove:
- `wait_for_claude_ready()` - No longer needed
- `capture_pane()` - No longer needed for state detection
- Prompt detection regexes (`is_ready_prompt`, etc.)
- `is_claude_process()`, `is_shell()` process detection

Keep:
- `send_keys()` - Still needed for sending messages
- Session creation/management
- Any utilities still used by other code

**3. Simplify `patrol.rs`**

Remove:
- `check_session_health()` TMUX polling (when `hooks_session_lifecycle` was false)
- `detect_state_transitions()` git polling (when `hooks_task_completion` was false)
- `detect_reviewing_amendments()` polling (now handled by Stop hook)

The patrol should now only:
- Handle queued self-review prompts
- Trigger rebases when master advances
- Handle rebase state detection (can keep `detect_rebasing_transition()` for now)
- Run periodic maintenance

**4. Keep the daemon loop structure**

The current daemon loop in `up.rs` already handles hook events correctly.
No major architectural changes needed - just remove the feature flag checks.

**5. Update configuration**

- Remove `hooks_session_lifecycle` and `hooks_task_completion` from config
- Keep `patrol_interval_secs` for maintenance tasks (rebasing, self-review)
- Update `config.rs` defaults

**6. Update documentation**

Update `llmc.md`:
- Remove references to polling-based detection as primary mechanism
- Document that hooks are the primary detection mechanism
- Keep troubleshooting section about disabling hooks (no longer applicable
  after this milestone, but useful for understanding the migration)

### Implementation Notes

- This is a cleanup milestone - no new features
- Run full test suite before and after
- Keep git utility functions (still used for state queries in Stop hook handler)
- Keep `detect_rebasing_transition()` for now - rebase detection via Stop hook
  can be added later if needed

### Out of Scope

- Do NOT add new features
- Do NOT implement PostToolUse git detection (deprecated)
- Focus purely on removal and cleanup

### Validation

1. `just check` and `just clippy` and `just review` pass
2. Full workflow test: create worker → start task → complete → review → accept
3. Crash recovery works via SessionEnd hook
4. Self-review flow works correctly
5. Rebase triggered by LLMC still works
6. No TMUX pane capture occurring for session health (verify via logging)

---

## Session 5 (Optional): Testing and Hardening

### Task: Comprehensive Testing and Edge Case Handling

See `@rules_engine/docs/llmc.md` for background.

See `@rules_engine/docs/llmc_hooks.md` for context. Milestones 1-4 are complete
and tested.

### Goal

Add comprehensive tests and handle edge cases.

### Potential Deliverables

1. **Integration tests for hook flow**
   - Mock hook events, verify state transitions
   - Test concurrent events from multiple workers
   - Test socket reconnection after daemon restart

2. **Edge case handling**
   - Hook timeout (daemon doesn't respond in 5s)
   - Socket file missing (daemon not running)
   - Malformed JSON from hooks
   - Duplicate events (idempotency)

3. **Observability improvements**
   - Metrics for hook event latency
   - Alerting for missed events
   - Debug logging for troubleshooting

4. **Performance optimization**
   - Benchmark event throughput
   - Optimize socket handling if needed
   - Connection pooling if beneficial

### Validation

1. All new tests pass
2. Stress test with rapid hook events
3. Graceful degradation when socket unavailable
