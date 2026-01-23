---
lattice-id: LEAWQN
name: consider-pending-task-prompt-for-llmc-st
description: consider-pending-task-prompt-for-llmc-start
task-type: task
priority: 3
created-at: 2026-01-23T05:40:19.698647Z
updated-at: 2026-01-23T19:54:53.891455Z
closed-at: 2026-01-23T19:54:53.891455Z
---

# Consider Using Pending Task Prompt for llmc start

## Context

The race condition fix (LEDWQN) added a `pending_task_prompt` mechanism for auto mode where:
1. `/clear` is sent to restart the session
2. The prompt is stored as pending
3. When SessionStart hook fires, the prompt is sent

## Current State

The `llmc start` command still uses the old approach of sending `/clear` followed immediately by the prompt. This has the same theoretical race condition vulnerability.

## Why It's Lower Priority

For `llmc start`:
- It's a manual command where the user is watching
- The session is already running before the command is called
- If something goes wrong, the user can notice and retry

## Potential Improvement

Consider updating `llmc start` to also use the `pending_task_prompt` mechanism for consistency and robustness. This would require:
1. Storing the prompt in state as pending
2. Returning success immediately after `/clear`
3. Having the daemon send the prompt when SessionStart fires

The downside is this changes the UX from synchronous to asynchronous task assignment.
