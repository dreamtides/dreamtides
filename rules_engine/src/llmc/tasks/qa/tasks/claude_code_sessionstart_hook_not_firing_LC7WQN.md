---
lattice-id: LC7WQN
name: claude-code-sessionstart-hook-not-firing
description: Claude Code SessionStart hook not firing automatically for auto workers
parent-id: LB6WQN
task-type: bug
priority: 2
labels:
- llmc-auto
- auto-mode
- hooks
- bug
created-at: 2026-01-22T03:02:27.377971Z
updated-at: 2026-01-22T03:02:27.377971Z
---

# Bug: Claude Code SessionStart Hook Not Firing Automatically

## Summary

When an auto worker starts Claude Code with properly configured hooks in `.claude/settings.json`, the SessionStart hook is not being fired automatically when Claude Code finishes initialization. The hook only works when manually triggered.

## Observation

- Settings.json has correct SessionStart hook configuration
- Claude Code starts and shows "Welcome back Derek!" message
- Claude Code is ready for input (shows prompt line)
- SessionStart hook never fires automatically
- Worker stays in "offline" state

When manually triggering the hook, it works correctly:
```bash
LLMC_ROOT=/path/to/root llmc hook session-start --worker auto-1 << EOF
{"session_id": "test-session-123"}
EOF
```

After manual trigger, worker correctly transitions offline -> idle -> working.

## Possible Causes

1. Claude Code's SessionStart hook feature may not work in non-interactive mode
2. Hook might need terminal attached to fire
3. Timing issue - hook might fire before socket is ready
4. Hook might be suppressed when Claude starts via tmux send-keys

## Configuration

Settings at `.worktrees/auto-1/.claude/settings.json`:
```json
{
  "hooks": {
    "SessionStart": [{
      "hooks": [{
        "command": "LLMC_ROOT=/path/to/root llmc hook session-start --worker auto-1",
        "timeout": 5,
        "type": "command"
      }]
    }]
  }
}
```

## Environment

- Claude Code version: 2.1.15
- Worker started via `llmc up --auto`
- Session created via tmux with send-keys

## Workaround

Manual hook trigger works, but this defeats the purpose of automatic auto-mode operation.

## Impact

This bug prevents fully automatic operation of auto mode. Workers cannot receive tasks without manual intervention to trigger the SessionStart hook.