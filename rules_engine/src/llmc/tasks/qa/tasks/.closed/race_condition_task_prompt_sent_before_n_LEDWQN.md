---
lattice-id: LEDWQN
name: race-condition-task-prompt-sent-before-n
description: 'Race condition: Task prompt sent before new session ready after /clear'
parent-id: LB6WQN
task-type: bug
priority: 1
labels:
- llmc-auto
- race-condition
- task-assignment
- critical
blocking:
- LH7WQN
created-at: 2026-01-23T04:33:29.642795Z
updated-at: 2026-01-23T05:40:25.855630Z
closed-at: 2026-01-23T05:40:25.855630Z
---

# Race Condition: Task Prompt Lost After /clear

## Summary

When assigning a task to an auto worker, the LLMC daemon sends `/clear` followed by the task prompt. However, the task prompt is sent **before** the new Claude Code session is ready to receive input, causing the prompt to be displayed in TMUX but never processed by Claude Code.

## Observed Behavior

1. Worker is idle, task pool returns a task
2. LLMC daemon sends `/clear` command (04:11:25.374515)
3. LLMC daemon sends task prompt 543ms later (04:11:25.917819) 
4. SessionEnd hook fires 207ms after prompt sent (04:11:26.124981)
5. SessionStart hook fires 626ms after prompt sent (04:11:26.751605)
6. Task prompt is visible in TMUX pane but Claude Code never responds
7. Worker stays in "working" state indefinitely

## Root Cause

The daemon sends the task prompt **before** receiving the SessionStart hook confirmation that the new Claude Code session is ready. The prompt arrives during the session transition period and is displayed but not processed as actual user input.

## Evidence

LLMC log timestamps:
```
04:11:25.374515 - /clear sent (TMUX)
04:11:25.917819 - Task prompt sent (TMUX) - SUCCESS
04:11:26.124981 - SessionEnd hook received
04:11:26.751605 - SessionStart hook received
```

Transcript file for session `211d412c-c1c7-447a-8414-3f72669194f8` only shows:
- SessionStart hook
- /clear command and output
- NO task prompt entry

TMUX pane shows the task prompt displayed with `❯` prefix but Claude Code is stuck at the input prompt.

## Expected Behavior

The daemon should wait for SessionStart hook confirmation **before** sending the task prompt to ensure the new session is ready to receive input.

## Suggested Fix

In the task assignment flow:
1. Send `/clear`
2. Wait for SessionEnd hook (optional, for cleanup)
3. **Wait for SessionStart hook** before proceeding
4. Send task prompt

Alternatively, increase the delay between `/clear` and task prompt sending to ensure the session is fully ready.

## Impact

- Critical for auto mode: Tasks cannot be assigned reliably
- Worker appears "working" but is completely stuck
- No timeout or retry mechanism detects this state
- Test scenario completely blocked