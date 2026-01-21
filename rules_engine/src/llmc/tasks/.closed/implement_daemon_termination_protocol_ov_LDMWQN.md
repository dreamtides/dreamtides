---
lattice-id: LDMWQN
name: implement-daemon-termination-protocol-ov
description: Implement daemon termination protocol for overseer
task-type: feature
priority: 1
labels:
- auto-overseer
- core
- overseer
blocking:
- LDPWQN
- LDNWQN
blocked-by:
- LDLWQN
created-at: 2026-01-21T04:01:29.437932Z
updated-at: 2026-01-21T14:49:21.763536Z
closed-at: 2026-01-21T14:49:21.763536Z
---

## Overview

Implement the protocol the overseer uses to terminate a failed daemon process before entering remediation.

## Implementation Steps

1. **Add termination logic to health_monitor.rs or new daemon_control.rs**:
   - `terminate_daemon(registration: &DaemonRegistration) -> Result<()>`
   - Handles the full termination sequence

2. **Termination sequence**:
   - Log failure details with full context before termination
   - Send SIGTERM to daemon PID immediately (no waiting for self-recovery)
   - Wait grace period (30 seconds) for graceful shutdown
   - Check if process still exists
   - If still running, send SIGKILL
   - Verify process is fully terminated
   - Clean up stale registration files if needed

3. **Process verification**:
   - Before sending signals, verify PID still matches expected daemon
   - Check start_time and instance_id to avoid killing wrong process
   - Handle race conditions where daemon terminates between checks

4. **Platform considerations**:
   - Use appropriate signal handling for the platform (Unix signals)
   - Handle cases where process is already gone
   - Handle permission errors gracefully

5. **Logging**:
   - Log each step of termination protocol
   - Log signal sent and response
   - Log final termination confirmation

## Acceptance Criteria

- Daemon terminated gracefully when possible (SIGTERM)
- Forceful termination (SIGKILL) used as fallback
- Process identity verified before sending signals
- Race conditions handled safely
- Full termination confirmed before proceeding to remediation
