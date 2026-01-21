---
lattice-id: LDTWQN
name: integrate---auto-flag-into-llmc-up-comma
description: Integrate --auto flag into llmc up command
task-type: feature
priority: 2
labels:
- auto-overseer
- cli
- integration
blocking:
- LDSWQN
- LDWWQN
blocked-by:
- LDIWQN
- LDJWQN
created-at: 2026-01-21T04:02:52.569768Z
updated-at: 2026-01-21T06:11:11.439973Z
closed-at: 2026-01-21T06:11:11.439973Z
---

## Overview

Integrate the `--auto` flag and related options into the existing `llmc up` command, branching to auto mode when specified.

## Implementation Steps

1. **Update cli.rs Up command**:
   - Add `--auto` boolean flag
   - Add `--task-pool-command <CMD>` optional string
   - Add `--concurrency <N>` optional u32
   - Add `--post-accept-command <CMD>` optional string
   - Update help text to explain auto mode

2. **Update up.rs command handler**:
   - Check if `--auto` flag is set
   - If auto mode:
     - Merge CLI flags with config (CLI wins)
     - Validate task_pool_command is available
     - Call `auto_mode::auto_orchestrator::run_auto_mode()`
   - If not auto mode:
     - Continue with existing `run_up()` behavior unchanged

3. **Configuration merging**:
   - CLI `--task-pool-command` overrides config `[auto].task_pool_command`
   - CLI `--concurrency` overrides config `[auto].concurrency`
   - CLI `--post-accept-command` overrides config `[auto].post_accept_command`
   - Create effective AutoConfig with merged values

4. **Validation**:
   - If `--auto` specified but no task_pool_command (CLI or config): error
   - Provide clear error message guiding user to configure

5. **Documentation**:
   - Update `llmc up --help` to document new flags
   - Explain relationship between CLI flags and TOML config

## Acceptance Criteria

- `llmc up --auto` enters auto mode
- CLI flags override TOML config
- Missing task_pool_command produces clear error
- Existing `llmc up` behavior unchanged without --auto
- Help text documents all new options
