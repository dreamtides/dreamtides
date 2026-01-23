---
lattice-id: LEGWQN
name: overseer-detects-failure-spiral-first-fa
description: Overseer detects failure spiral on first failure without remediation attempt
parent-id: LB6WQN
task-type: bug
priority: 1
labels:
- llmc-auto
- overseer
- failure-spiral
blocking:
- LIDWQN
created-at: 2026-01-23T19:15:26.195163Z
updated-at: 2026-01-23T19:15:40.873718Z
---

# Bug: Failure Spiral Detection Triggers Prematurely

## Summary

The overseer incorrectly detects a "failure spiral" on the FIRST daemon failure if it occurs within `restart_cooldown_secs`, without ever attempting remediation. According to the design document, the failure spiral check should only trigger AFTER at least one remediation attempt.

## Expected Behavior

Per `auto_overseer_design.md`:
1. Daemon fails
2. Overseer detects failure and terminates daemon
3. Overseer runs remediation (Claude Code session to fix the issue)
4. Overseer restarts daemon
5. If daemon fails AGAIN within `restart_cooldown_secs` -> failure spiral detected

## Actual Behavior

1. Daemon fails
2. Overseer detects failure
3. Overseer checks `is_failure_spiral()` **BEFORE** remediation
4. If daemon ran for less than `restart_cooldown_secs` -> failure spiral detected
5. Overseer terminates without any remediation attempt

## Root Cause

In `overseer_loop.rs`, the `is_failure_spiral()` check happens BEFORE the remediation code:

```rust
let daemon_start_time = Instant::now();
let failure = run_monitor_loop(...);
terminate_daemon_gracefully(...);

// BUG: Check happens BEFORE remediation
if is_failure_spiral(daemon_start_time, &overseer_config) {
    bail!("Failure spiral detected");
}

// Remediation never runs for quick failures
run_remediation(...);
```

## Reproduction Steps

1. Configure auto mode with a malformed task file (causes immediate parse failure)
2. Set `restart_cooldown_secs = 30`
3. Run `llmc overseer`
4. Observe: overseer detects "failure spiral" immediately without remediation

## Proposed Fix

Track whether remediation has been attempted, and only check for failure spiral after at least one remediation:

```rust
let mut remediation_attempted = false;

loop {
    let daemon_start_time = Instant::now();
    let failure = run_monitor_loop(...);
    terminate_daemon_gracefully(...);
    
    // Only check for spiral AFTER remediation has been tried
    if remediation_attempted && is_failure_spiral(daemon_start_time, &overseer_config) {
        bail!("Failure spiral detected");
    }
    
    run_remediation(...);
    remediation_attempted = true;
}
```

## Impact

High - the overseer's remediation capability is effectively disabled for any failure that happens within the first 30 seconds of daemon startup. This defeats the core purpose of the overseer.

## Related Design Doc

`rules_engine/src/llmc/docs/auto_overseer_design.md` - see "Failure Spiral Prevention" section