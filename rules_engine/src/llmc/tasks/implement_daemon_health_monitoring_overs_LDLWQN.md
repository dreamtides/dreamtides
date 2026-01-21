---
lattice-id: LDLWQN
name: implement-daemon-health-monitoring-overs
description: Implement daemon health monitoring for overseer
task-type: feature
priority: 1
labels:
- auto-overseer
- core
- overseer
blocking:
- LDMWQN
- LDNWQN
- LDPWQN
blocked-by:
- LDFWQN
- LDKWQN
created-at: 2026-01-21T04:01:29.130792Z
updated-at: 2026-01-21T04:09:12.952372Z
---

## Overview

Implement the health monitoring system that the overseer uses to detect daemon failures.

## Implementation Steps

1. **Create health_monitor.rs** under `src/overseer_mode/`:
   - Central module for all daemon health checks
   - Returns structured health status with failure details

2. **Process identity verification**:
   - Read `.llmc/daemon.json`
   - Compare PID, start_time_unix, and instance_id against expected values
   - Detect PID reuse (same PID but different start time or instance ID)
   - Detect unexpected daemon restarts

3. **Heartbeat monitoring**:
   - Read `.llmc/auto.heartbeat`
   - Check if timestamp is within heartbeat_timeout_secs of current time
   - Missing file treated as stale heartbeat
   - Return specific failure type for stale/missing heartbeat

4. **Log monitoring**:
   - Implement log file tailing for daemon log
   - Parse log entries for ERROR and WARN levels
   - ANY error or warning triggers failure detection
   - Track log file position to avoid re-reading old entries

5. **Progress tracking**:
   - Read `last_task_completion_unix` from state
   - Compare against stall_timeout_secs
   - If no completions within timeout, report stall

6. **Health check aggregation**:
   - `check_daemon_health() -> HealthStatus`
   - HealthStatus enum: Healthy, ProcessGone, HeartbeatStale, LogError(String), Stalled, IdentityMismatch
   - Return first detected failure (priority order)

## Acceptance Criteria

- All four health check types implemented
- Single error/warning in logs triggers failure
- Stale heartbeat detected within configured timeout
- Process identity changes detected
- Clear failure reasons returned for remediation context
