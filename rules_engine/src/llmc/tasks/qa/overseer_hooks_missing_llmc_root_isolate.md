---
lattice-id: LBVWQN
name: overseer-hooks-missing-llmc-root-isolate
description: Overseer hooks missing LLMC_ROOT for isolated test environments
parent-id: LB6WQN
task-type: bug
priority: 1
labels:
- llmc-auto
blocking:
- LICWQN
created-at: 2026-01-23T14:06:49.218788Z
updated-at: 2026-01-23T14:09:52.968213Z
---

## Bug Description

When running the overseer in an isolated test environment (with LLMC_ROOT set to a non-default path), the Claude Code hooks in the source repository's `.claude/settings.json` do not include the LLMC_ROOT environment variable. This causes hook events (Stop, SessionStart, SessionEnd) to be sent to the wrong IPC socket.

## Steps to Reproduce

1. Set up an isolated test environment: `export LLMC_ROOT="/tmp/llmc-test"`
2. Initialize and start overseer
3. Trigger remediation (e.g., kill daemon)
4. Complete remediation by exiting Claude Code session with `/exit`
5. Observe that the overseer never detects the completion

## Expected vs Actual Behavior

Expected: Hooks include `LLMC_ROOT={path}` prefix
Actual: Hooks don't include LLMC_ROOT, events go to wrong socket

## Root Cause

The `create_overseer_claude_hooks()` function skips re-creation if hooks file exists and contains "llmc hook". Previous llmc instance may have created hooks without LLMC_ROOT.

## Suggested Fix

Regenerate hooks file when LLMC_ROOT doesn't match existing configuration.

## Test Scenario

Found during manual test LICWQN: Overseer Failure Detection and Remediation
