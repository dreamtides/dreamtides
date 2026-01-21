---
lattice-id: LDRWQN
name: protect-overseer-tmux-session-from-kill-
description: Protect overseer TMUX session from kill operations
parent-id: LBSWQN
task-type: feature
priority: 2
labels:
- auto-overseer
- integration
- safety
blocking:
- LDXWQN
- LDWWQN
blocked-by:
- LDKWQN
- LDQWQN
created-at: 2026-01-21T04:02:15.329191Z
updated-at: 2026-01-21T22:31:38.778Z
closed-at: 2026-01-21T18:16:31.727962Z
---

## Overview

Ensure that the overseer TMUX session is protected from operations that kill
TMUX sessions, preventing accidental termination of the overseer.

## Implementation Steps

1. **Identify all session-killing operations**:
   - `llmc down` - kills worker sessions
   - `llmc down --kill-consoles` - kills console sessions
   - `llmc nuke --all` - removes all workers
   - `llmc down --force` - forceful session termination
   - Any "cleanup orphaned sessions" logic

2. **Add overseer session detection**:
   - Use helper function `is_overseer_session(session_name: &str) -> bool`
   - Check for `llmc-overseer` session name
   - Check for LLMC_OVERSEER environment variable in session

3. **Update llmc down**:
   - Filter out overseer session when collecting sessions to terminate
   - Log that overseer session is being preserved
   - No new flag needed - overseer is always protected

4. **Update llmc nuke --all**:
   - Exclude overseer from "all workers" collection
   - Overseer is not a worker, so this should be natural

5. **Update orphaned session cleanup**:
   - `cleanup_orphaned_llmc_sessions()` should skip overseer
   - Overseer session is intentionally long-lived

6. **Update any TMUX "kill all" patterns**:
   - Search codebase for patterns that might kill all llmc-* sessions
   - Add exclusion for llmc-overseer

7. **Documentation**:
   - Add comments explaining why overseer is protected
   - Document that only Ctrl-C on overseer process terminates it

## Acceptance Criteria

- `llmc down` does not kill overseer session
- `llmc nuke --all` does not affect overseer
- Orphan cleanup skips overseer
- Only Ctrl-C on overseer terminates it
- Protection logic is clear and documented
