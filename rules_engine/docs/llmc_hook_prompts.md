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

## Session 4: Git Operation Detection (PostToolUse Hook)

### Task: Implement PostToolUse Hook for Git Detection (Milestone 4)

See `@rules_engine/docs/llmc.md` for background.

See `@rules_engine/docs/llmc_hooks.md` for context. Milestones 1-3 are complete.

### Goal

Detect git commits and rebase operations in real-time via PostToolUse(Bash)
hook instead of polling git state.

### Deliverables

**1. Handle PostBash event in `patrol.rs`**

Parse the command from hook input and detect:

```rust
fn is_commit_command(cmd: &str) -> bool {
    cmd.contains("git commit") ||
    cmd.contains("git rebase --continue")
}

fn is_abort_command(cmd: &str) -> bool {
    cmd.contains("git rebase --abort")
}
```

On commit command (exit_code == 0):
- If worker is Working/Rejected: Check for commits, transition to NeedsReview
- If worker is Rebasing: Rebase complete, transition to NeedsReview

On abort command:
- If worker is Rebasing: Rebase aborted, transition back to NeedsReview

**2. Add feature flag**

```rust
pub hooks_git_detection: bool  // default: true
```

**3. Simplify `detect_rebasing_transition()` in `patrol.rs`**

Current code polls git state and captures TMUX pane for "Successfully rebased".
When hooks enabled, rely on PostBash detecting `git rebase --continue`.

Gate the TMUX pane capture behind the feature flag.

**4. Update hook configuration template**

Ensure PostToolUse hook is configured for Bash commands in the template
generated by `add.rs` (should already be there from Milestone 1).

**5. Handle edge cases**

- `git commit --amend`: Same as regular commit for our purposes
- `git cherry-pick`: Treat like commit
- Failed commands (exit_code != 0): Ignore, worker will retry

### Implementation Notes

- PostBash fires for every Bash command, filter to git operations only
- Exit code 0 means success - only act on successful operations
- This provides faster feedback than Stop hook for git operations

### Out of Scope

- Do NOT remove polling fallbacks yet (that's Milestone 5)
- Do NOT change non-git state detection

### Validation

1. `just check` and `just clippy` and `just review` pass
2. Worker commits - PostBash hook triggers state transition immediately
3. Worker in rebase runs `git rebase --continue` - transitions out of Rebasing
4. Worker runs `git rebase --abort` - handled correctly
5. Non-git Bash commands don't cause spurious transitions

---

## Session 5: Cleanup and Finalization

### Task: Remove Polling Fallbacks and Legacy Code (Milestone 5)

See `@rules_engine/docs/llmc.md` for background.

See `@rules_engine/docs/llmc_hooks.md` for context. Milestones 1-4 are complete
and tested.

### Goal

Remove all polling-based detection code and TMUX pane capture logic. Make hooks
the sole detection mechanism.

### Deliverables

**1. Remove feature flags and legacy code paths**

Delete:
- `hooks_session_lifecycle` flag and polling fallback
- `hooks_task_completion` flag and git polling fallback
- `hooks_git_detection` flag and related fallbacks

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
- `check_session_health()` polling loop
- `detect_state_transitions()` git polling
- `detect_rebasing_transition()` pane capture

The patrol should now only:
- Handle queued self-review prompts
- Trigger rebases when master advances
- Run periodic maintenance (not state detection)

**4. Update `commands/up.rs` daemon loop**

Change from polling-based to purely event-driven:

```rust
loop {
    tokio::select! {
        Some(event) = ipc.next_event() => {
            handle_hook_event(event, &mut state)?;
        }
        _ = maintenance_interval.tick() => {
            run_maintenance(&mut state)?;
        }
        _ = shutdown.recv() => break,
    }
}
```

**5. Update configuration**

- Remove `patrol_interval_secs` from config (no longer polling)
- Add `maintenance_interval_secs` for periodic tasks (rebasing, self-review)
- Update `config.rs` and default config

**6. Update documentation**

Update `llmc.md`:
- Remove references to polling-based detection
- Document hook-based architecture
- Update the architecture diagram
- Remove appendix references to TMUX state detection heuristics

### Implementation Notes

- This is a breaking change - ensure all milestones are tested first
- Run full test suite before and after
- Keep git utility functions (they're still used for state queries, just not
  polling)

### Out of Scope

- Do NOT add new features
- Focus purely on removal and cleanup

### Validation

1. `just check` and `just clippy` and `just review` pass
2. Full workflow test: create worker → start task → complete → review → accept
3. Crash recovery works via SessionEnd hook
4. Rebase during review works via PostBash hook
5. No TMUX pane capture occurring (verify via logging)
6. CPU usage reduced compared to polling baseline

---

## Session 6 (Optional): Testing and Hardening

### Task: Comprehensive Testing and Edge Case Handling

See `@rules_engine/docs/llmc.md` for background.

See `@rules_engine/docs/llmc_hooks.md` for context. Milestones 1-5 are complete
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
