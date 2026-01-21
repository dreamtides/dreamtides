---
lattice-id: LDSWQN
name: add-llmc-overseer-cli-command
description: Add llmc overseer CLI command
parent-id: LBSWQN
task-type: feature
priority: 2
labels:
- auto-overseer
- cli
- integration
blocking:
- LDQWQN
blocked-by:
- LDPWQN
- LDTWQN
created-at: 2026-01-21T04:02:52.268229Z
updated-at: 2026-01-21T22:31:38.672874Z
closed-at: 2026-01-21T17:00:05.263957Z
---

## Overview

Add the new `llmc overseer` command that starts the overseer supervisor process.

## Implementation Steps

1. **Update cli.rs**:
   - Add `Overseer` variant to Commands enum
   - No subcommands needed - just `llmc overseer` starts it
   - Add help text explaining the command's purpose

2. **Create overseer_command.rs** under `src/commands/`:
   - Entry point: `run_overseer(config: &Config) -> Result<()>`
   - Delegates to `overseer_mode::overseer_loop::run_overseer()`

3. **Pre-flight checks**:
   - Validate overseer configuration exists and is valid
   - Check that remediation_prompt is configured
   - Ensure no other overseer is already running (check overseer.json)
   - Provide clear error messages for missing configuration

4. **Runtime behavior**:
   - Runs in foreground (not daemonized)
   - Ctrl-C terminates the overseer
   - Prints high-level status updates to stdout as tasks complete
   - No separate `overseer start` or `overseer stop` commands

5. **Overseer registration**:
   - Write `.llmc/overseer.json` on startup:
     - `pid`: Overseer process ID
     - `start_time_unix`: Start timestamp
     - `instance_id`: Random UUID
   - Clean up file on graceful shutdown
   - Used by `llmc status` to detect active overseer

6. **Signal handling**:
   - Handle SIGINT (Ctrl-C) for graceful shutdown
   - Handle SIGTERM for graceful shutdown
   - Ensure daemon is terminated when overseer exits

## Acceptance Criteria

- `llmc overseer` command available and documented
- Validates configuration before starting
- Prevents multiple overseers from running
- Ctrl-C cleanly shuts down overseer and daemon
- Registration file created and cleaned up
