---
lattice-id: LDKWQN
name: implement-overseer-claude-code-session-m
description: Implement overseer Claude Code session management
task-type: feature
priority: 1
labels:
- auto-overseer
- foundation
- overseer
blocking:
- LDOWQN
- LDRWQN
- LDLWQN
blocked-by:
- LDDWQN
- LDIWQN
created-at: 2026-01-21T04:01:28.826051Z
updated-at: 2026-01-21T13:48:48.596304Z
closed-at: 2026-01-21T13:48:48.596304Z
---

## Overview

Implement management of the dedicated Claude Code session used by the overseer for remediation tasks.

## Implementation Steps

1. **Create overseer_session.rs** under `src/overseer_mode/`:
   - TMUX session name: `llmc-overseer`
   - Session runs in main project directory (repo.source), NOT a worktree
   - Implement session creation, health checking, and restart logic

2. **Session creation**:
   - Create TMUX session if it doesn't exist
   - Start Claude Code with standard flags (model from config, skip_permissions, etc.)
   - Set environment variables for identification (LLMC_OVERSEER=true)
   - Use wide terminal width to prevent truncation

3. **Session health monitoring**:
   - Implement `is_overseer_session_healthy() -> bool`
   - Check if TMUX session exists
   - Check if Claude Code process is running within session
   - Called periodically by overseer main loop

4. **Automatic restart**:
   - If session is unhealthy, recreate it automatically
   - This is an AUTO recovery - overseer handles it without external help
   - Log restart events

5. **Session protection**:
   - Add `is_overseer_session(session_name: &str) -> bool` helper
   - This will be used by other tasks to exclude overseer from kill operations

6. **Hook configuration**:
   - Ensure overseer session has hook config for completion detection
   - Reuse existing hook setup logic from worker management

## Acceptance Criteria

- Overseer TMUX session created in main project directory
- Session automatically restarted if it crashes
- Claude Code hooks configured for completion detection
- Helper functions available for session identification
