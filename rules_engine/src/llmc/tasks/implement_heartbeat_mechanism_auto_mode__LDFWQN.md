---
lattice-id: LDFWQN
name: implement-heartbeat-mechanism-auto-mode-
description: Implement heartbeat mechanism for auto mode daemon
task-type: feature
priority: 1
labels:
- auto-mode
- auto-overseer
- foundation
blocking:
- LDIWQN
- LDLWQN
- LDUWQN
blocked-by:
- LDVWQN
- LDCWQN
created-at: 2026-01-21T04:00:04.049346Z
updated-at: 2026-01-21T04:09:10.772802Z
---

## Overview

Implement the heartbeat mechanism that allows external processes (like the overseer) to detect if the auto daemon is running and healthy.

## Implementation Steps

1. **Create heartbeat_thread.rs** under `src/auto_mode/`:
   - Define heartbeat file format: `{"timestamp_unix": N, "instance_id": "UUID"}`
   - Implement background thread that updates `.llmc/auto.heartbeat` every 5 seconds
   - Use atomic writes (temp file + rename) to prevent partial reads
   - Thread should be resilient to transient I/O errors (log and retry)

2. **Create daemon registration file**:
   - On daemon startup, write `.llmc/daemon.json` containing:
     - `pid`: Process ID
     - `start_time_unix`: Unix timestamp when daemon started
     - `instance_id`: Random UUID for this daemon instance
     - `log_file`: Path to the daemon's log file
   - Use atomic write
   - Clean up file on graceful shutdown

3. **Integrate with daemon lifecycle**:
   - Start heartbeat thread after daemon registration is written
   - Ensure heartbeat thread is properly terminated on shutdown
   - Heartbeat should continue even if main loop is blocked (separate thread)

4. **Add utility functions**:
   - `read_daemon_registration() -> Option<DaemonRegistration>`
   - `read_heartbeat() -> Option<Heartbeat>`
   - `is_heartbeat_stale(heartbeat: &Heartbeat, timeout: Duration) -> bool`

## Acceptance Criteria

- Heartbeat file is updated every 5 seconds while daemon runs
- Daemon registration file contains all required fields
- Atomic writes prevent corruption
- External processes can reliably detect daemon state
