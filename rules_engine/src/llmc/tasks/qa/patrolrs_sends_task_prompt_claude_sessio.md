---
lattice-id: LEBWQN
name: patrolrs-sends-task-prompt-claude-sessio
description: patrol.rs sends task prompt to Claude session UUID instead of TMUX session name
parent-id: LB6WQN
task-type: bug
priority: 1
labels:
- llmc-auto
blocking:
- LH7WQN
created-at: 2026-01-23T06:08:50.712096Z
updated-at: 2026-01-23T06:09:24.169902Z
---

# Bug: patrol.rs sends task prompt to wrong session ID

## Summary

In `patrol.rs`, the `handle_session_start` function passes the Claude session UUID
(from the SessionStart hook event) to `tmux_sender.send()`, but this function expects
a TMUX session name. This causes task prompts to silently fail to appear in the
worker's terminal after `/clear` is executed.

## Impact

**Critical**: Auto mode is completely broken. Tasks are assigned but never appear
in the worker's terminal, causing workers to remain idle indefinitely.

## Root Cause

In `patrol.rs` line 1118:
```rust
if let Err(e) = tmux_sender.send(session_id, &pending_prompt) {
```

The `session_id` variable contains the Claude session UUID
(e.g., `a3eade4c-f6f1-4a15-a268-e9494d235cf0`) from the SessionStart hook,
but `tmux_sender.send()` expects a TMUX session name
(e.g., `llmc-llmc-conflict-test-63851-auto-1`).

The same bug also exists at line 1061 for conflict prompts:
```rust
sender.send(session_id, &conflict_prompt)?;
```

## Evidence from Logs

At `06:00:17.216153Z` (before /clear):
```
"session_id":"llmc-llmc-conflict-test-63851-auto-1"  <-- TMUX session (correct)
```

At `06:00:48.951074Z` (after /clear):
```
"session_id":"a3eade4c-f6f1-4a15-a268-e9494d235cf0"  <-- Claude UUID (WRONG)
```

## Fix

Replace `session_id` with `config::get_worker_session_name(worker_name)`:

### Line 1118:
```rust
let tmux_session = config::get_worker_session_name(worker_name);
if let Err(e) = tmux_sender.send(&tmux_session, &pending_prompt) {
```

### Line 1061:
```rust
let tmux_session = config::get_worker_session_name(worker_name);
sender.send(&tmux_session, &conflict_prompt)?;
```

## Reproduction Steps

1. Start auto mode with a task pool
2. Worker receives task, daemon sends `/clear`
3. After `/clear`, daemon tries to send task prompt
4. Prompt goes to wrong session ID, never appears in terminal
5. Worker sits idle forever

## Discovered During

Manual test scenario LH7WQN (Merge Conflict Handling During Auto Accept)
