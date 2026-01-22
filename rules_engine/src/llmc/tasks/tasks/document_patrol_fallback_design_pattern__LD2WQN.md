---
lattice-id: LD2WQN
name: document-patrol-fallback-design-pattern-
description: Document patrol fallback design pattern for hook failure recovery
task-type: chore
priority: 3
labels:
- documentation
- llmc-auto
- design-pattern
created-at: 2026-01-22T04:37:05.221022Z
updated-at: 2026-01-22T04:37:05.221022Z
---

# Document Patrol Fallback Design Pattern

## Background

The LLMC system uses hooks (SessionStart, SessionEnd, Stop) for event-driven
state transitions. However, hooks can fail silently for various reasons:
- Claude Code not recognizing `.claude/settings.json` in worktree
- Hook timeout (5 second default)
- IPC socket not available
- Hook process crashes

## Current Fallbacks

The patrol loop has fallback mechanisms for most hook failures:

1. **SessionStart hook failure (Offline → Idle)**: NEW - Added in LBSWQN fix.
   Patrol detects running sessions for offline workers and transitions to Idle.

2. **Stop hook failure (Working/Rejected → NeedsReview)**: Patrol waits 5 min
   after first detecting commits, then transitions to NeedsReview. See
   `check_state_consistency` in patrol.rs.

3. **SessionEnd hook failure (any → Offline)**: Patrol's `check_session_health`
   should detect missing sessions. TODO: Verify this fallback exists.

## Design Pattern

Every hook-driven transition should have a patrol fallback:

```
Hook Event → Immediate Transition (fast path)
           ↓
Patrol Loop → Observable Fact Check → Fallback Transition (slow path)
```

**Observable Facts:**
- Session exists: `session::session_exists(session_id)`
- Has commits: `git::has_commits_ahead_of(worktree, "origin/master")`
- Rebase in progress: `git::is_rebase_in_progress(worktree)`
- Worktree clean: `git::is_worktree_clean(worktree)`

## Recommendation

Document this pattern in `llmc/docs/` for future development. Each new
hook-driven feature should include:
1. The hook handler (fast path)
2. The patrol fallback (slow path)
3. Logging for both paths

## Related

LBSWQN - Original bug where Offline → Idle had no fallback