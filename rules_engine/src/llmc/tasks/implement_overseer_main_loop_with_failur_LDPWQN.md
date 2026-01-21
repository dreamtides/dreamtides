---
lattice-id: LDPWQN
name: implement-overseer-main-loop-with-failur
description: Implement overseer main loop with failure spiral detection
task-type: feature
priority: 1
labels:
- auto-overseer
- core
- overseer
blocking:
- LDQWQN
- LDSWQN
- LDXWQN
blocked-by:
- LDLWQN
- LDMWQN
- LDOWQN
created-at: 2026-01-21T04:02:14.885465Z
updated-at: 2026-01-21T04:06:50.858726Z
---

## Overview

Implement the main overseer loop that monitors the daemon, triggers remediation on failures, and prevents infinite failure spirals.

## Implementation Steps

1. **Create overseer_loop.rs** under `src/overseer_mode/`:
   - Main entry point: `run_overseer(config: &Config) -> Result<()>`
   - Orchestrates all overseer components

2. **Startup sequence**:
   - Validate configuration (remediation_prompt required)
   - Create overseer TMUX session if missing
   - Start overseer Claude Code process
   - Start daemon via shell command: `llmc up --auto`
     - IMPORTANT: Use shell execution to resolve `llmc` via caller's $PATH
     - This allows remediation to modify llmc code and have changes take effect
   - Watch for daemon registration file (`.llmc/daemon.json`)
   - Record daemon start time for cooldown tracking

3. **Monitor loop**:
   - Run health checks periodically (every few seconds)
   - On healthy status: continue monitoring
   - On any failure detected:
     - Execute daemon termination protocol
     - Enter remediation mode

4. **Remediation mode**:
   - Build remediation prompt with failure context
   - Execute remediation via overseer Claude session
   - Wait for remediation completion
   - Check for manual_intervention_needed files
     - If found: log contents and terminate overseer
   - Attempt daemon restart

5. **Failure spiral detection**:
   - Track daemon start time after each restart
   - If daemon fails within `restart_cooldown_secs` of start:
     - This is a failure spiral
     - Do NOT repeat remediation
     - Terminate overseer with detailed error message
   - Rationale: prevents infinite loops on unfixable issues

6. **Graceful shutdown**:
   - Handle Ctrl-C to terminate overseer
   - Terminate daemon if running
   - Clean up overseer session (optional - may want to preserve)

## Acceptance Criteria

- Daemon started via shell (not direct Rust call)
- Health monitoring runs continuously
- Failures trigger remediation automatically
- Failure spirals detected and terminate overseer
- Manual intervention requests honored
- Ctrl-C cleanly shuts down overseer and daemon
