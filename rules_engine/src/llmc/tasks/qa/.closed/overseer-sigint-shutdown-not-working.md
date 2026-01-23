---
lattice-id: LECWQN
name: overseer-sigint-shutdown-not-working
description: overseer-sigint-shutdown-not-working
parent-id: LB6WQN
task-type: bug
priority: 1
labels:
- llmc-auto
created-at: 2026-01-23T07:27:09.610482Z
updated-at: 2026-01-23T14:14:07.808527Z
closed-at: 2026-01-23T14:14:07.808526Z
---

## Summary

During manual test LIBWQN (Overseer Basic Operations), the overseer process did not respond to SIGINT (Ctrl-C) for graceful shutdown. SIGTERM was required to terminate the overseer.

## Steps to Reproduce

1. Start overseer: `llmc overseer`
2. Get the overseer PID (from ps or daemon status)
3. Send SIGINT: `kill -SIGINT <overseer_pid>`
4. Observe that overseer continues running
5. Send SIGTERM: `kill -SIGTERM <overseer_pid>`
6. Overseer terminates

## Expected Behavior

The overseer should handle SIGINT (Ctrl-C) for graceful shutdown, as documented in the test scenario:
- Overseer shuts down gracefully
- Exit code is 0
- Daemon is also terminated

## Actual Behavior

- SIGINT is ignored by the overseer process
- SIGTERM terminates the overseer
- When daemon is killed (via SIGINT to daemon), overseer correctly restarts it

## Notes

This may be intentional behavior if the overseer is designed to only respond to SIGTERM, but the test scenario explicitly tests Ctrl-C behavior.
