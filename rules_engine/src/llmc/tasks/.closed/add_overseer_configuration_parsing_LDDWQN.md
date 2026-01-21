---
lattice-id: LDDWQN
name: add-overseer-configuration-parsing
description: Add overseer configuration parsing
task-type: feature
priority: 1
labels:
- auto-overseer
- foundation
- overseer
blocking:
- LDKWQN
blocked-by:
- LDVWQN
- LDCWQN
created-at: 2026-01-21T04:00:03.646532Z
updated-at: 2026-01-21T07:21:40.928090Z
closed-at: 2026-01-21T07:21:40.928089Z
---

## Overview

Extend the LLMC configuration system to support the new `[overseer]` TOML section for the overseer supervisor process.

## Implementation Steps

1. **Define OverseerConfig struct** in a new `overseer_config.rs` file under `src/overseer_mode/`:
   - `remediation_prompt: String` (required when overseer is used)
   - `heartbeat_timeout_secs: u32` with default of 30
   - `stall_timeout_secs: u32` with default of 3600
   - `restart_cooldown_secs: u32` with default of 60
   - Implement Default trait

2. **Update Config struct** in `config.rs`:
   - Add `overseer: Option<OverseerConfig>` field
   - Update serde deserialization
   - Add validation: `remediation_prompt` required if overseer command is invoked

3. **Add config accessor methods**:
   - `Config::get_remediation_prompt(&self) -> Option<&str>`
   - `Config::get_heartbeat_timeout(&self) -> Duration`
   - `Config::get_stall_timeout(&self) -> Duration`
   - `Config::get_restart_cooldown(&self) -> Duration`

4. **Write unit tests**:
   - Test parsing valid `[overseer]` config
   - Test defaults are applied
   - Test missing remediation_prompt produces clear error

## Acceptance Criteria

- TOML config with `[overseer]` section parses correctly
- Defaults are sensible and documented
- Validation produces actionable error messages
