---
lattice-id: LDXWQN
name: write-integration-tests-overseer
description: Write integration tests for overseer
task-type: task
priority: 2
labels:
- auto-overseer
- overseer
- testing
blocked-by:
- LDPWQN
- LDRWQN
created-at: 2026-01-21T04:03:13.357144Z
updated-at: 2026-01-21T19:14:12.283967Z
closed-at: 2026-01-21T19:14:12.283967Z
---

## Overview

Write integration tests that verify the overseer functionality works correctly.

## Implementation Steps

1. **Create test infrastructure**:
   - Mock daemon process that can simulate failures
   - Mock Claude session for remediation
   - Test fixtures for configuration and state

2. **Health monitoring tests**:
   - Test: healthy daemon detected correctly
   - Test: stale heartbeat detected
   - Test: missing process detected
   - Test: log errors detected
   - Test: stall detected after timeout

3. **Daemon termination tests**:
   - Test: SIGTERM sent on failure detection
   - Test: SIGKILL sent if SIGTERM doesn't work
   - Test: process identity verified before signals
   - Test: race conditions handled

4. **Remediation tests**:
   - Test: prompt constructed with all context
   - Test: prompt sent to Claude session
   - Test: completion detected via hooks
   - Test: manual_intervention_needed file detected
   - Test: remediation logged to dedicated file

5. **Failure spiral tests**:
   - Test: rapid failure triggers spiral detection
   - Test: overseer terminates on spiral
   - Test: successful restart resets cooldown

6. **Session protection tests**:
   - Test: llmc down doesn't kill overseer session
   - Test: llmc nuke --all doesn't affect overseer

## Acceptance Criteria

- All monitoring scenarios tested
- Remediation flow tested end-to-end
- Failure spiral detection tested
- Tests use mocks appropriately
- Tests are reliable and fast
