---
lattice-id: LBWWQN
name: remediation-ipc-socket-conflicts-with-da
description: Remediation IPC socket conflicts with daemon socket
parent-id: LB6WQN
task-type: bug
priority: 1
labels:
- llmc-auto
blocking:
- LICWQN
created-at: 2026-01-23T14:09:04.133683Z
updated-at: 2026-01-23T14:41:23.649646Z
closed-at: 2026-01-23T14:41:23.649645Z
---

## Bug Description

When the remediation executor starts its IPC listener, it removes the existing socket (which belongs to the daemon) and creates its own. This causes a conflict because:

1. The daemon's auto mode loop loses its IPC socket
2. The daemon can't receive hook events from workers
3. The remediation executor's socket won't receive Stop events from Claude Code sessions that were configured to send hooks to the daemon's socket

## Steps to Reproduce

1. Start overseer with daemon
2. Kill daemon to trigger remediation
3. Observe that remediation executor creates its own IPC listener at the same socket path
4. The daemon's socket is removed

## Log Evidence

From llmc.jsonl:
- 14:02:11: Daemon's IPC listener starts at `llmc.sock`
- 14:02:16: Remediation executor removes existing socket and creates new one

## Expected Behavior

The remediation executor should either:
1. Use a separate socket path (e.g., `llmc_remediation.sock`)
2. Or use the existing socket path but ensure the Claude Code hooks are configured to send to it

## Actual Behavior

The remediation executor removes the daemon's socket and creates its own, but the daemon keeps running (it was already started before the identity mismatch was detected).

## Test Scenario

Found during manual test LICWQN: Overseer Failure Detection and Remediation
