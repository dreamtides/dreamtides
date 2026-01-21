---
lattice-id: LDCWQN
name: add-auto-configuration-parsing-configrs
description: Add auto configuration parsing to config.rs
task-type: feature
priority: 1
labels:
- auto-mode
- auto-overseer
- foundation
blocking:
- LDEWQN
- LDGWQN
- LDFWQN
- LDDWQN
blocked-by:
- LDVWQN
created-at: 2026-01-21T04:00:03.358885Z
updated-at: 2026-01-21T04:10:09.673749Z
---

## Overview

Extend the LLMC configuration system to support the new `[auto]` TOML section for autonomous operation mode.

## Implementation Steps

1. **Define AutoConfig struct** in a new `auto_config.rs` file under `src/auto_mode/`:
   - `task_pool_command: String` (required when auto mode enabled)
   - `concurrency: u32` with default of 1
   - `post_accept_command: Option<String>`
   - Implement Default trait with sensible defaults

2. **Update Config struct** in `config.rs`:
   - Add `auto: Option<AutoConfig>` field
   - Update serde deserialization to handle the new section
   - Add validation: if `--auto` flag is used, `task_pool_command` must be present

3. **Add CLI flag handling**:
   - Extend the `up` command in `cli.rs` with:
     - `--auto` boolean flag
     - `--task-pool-command <CMD>` optional override
     - `--concurrency <N>` optional override  
     - `--post-accept-command <CMD>` optional override
   - CLI flags should override TOML config values

4. **Add config accessor methods**:
   - `Config::get_task_pool_command(&self) -> Option<&str>`
   - `Config::get_auto_concurrency(&self) -> u32`
   - `Config::get_post_accept_command(&self) -> Option<&str>`
   - `Config::is_auto_mode_configured(&self) -> bool`

5. **Write unit tests**:
   - Test parsing valid `[auto]` config
   - Test missing required fields when auto mode enabled
   - Test CLI flag overrides
   - Test defaults are applied correctly

## Acceptance Criteria

- TOML config with `[auto]` section parses correctly
- CLI flags override config values
- Validation errors are clear and actionable
- All existing config tests continue to pass
