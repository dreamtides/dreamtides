---
lattice-id: LEEWQN
name: daemon-in-memory-state-not-synced-with-s
description: Daemon in-memory state not synced with state.json changes from other commands
parent-id: LB6WQN
task-type: bug
priority: 1
labels:
- llmc-auto
- state-sync
- daemon
blocking:
- LH7WQN
created-at: 2026-01-23T04:33:29.695242Z
updated-at: 2026-01-23T05:56:11.005120Z
closed-at: 2026-01-23T05:56:11.005120Z
---

# Daemon In-Memory State Desync with state.json

## Summary

When `llmc reset` (or potentially other state-modifying commands) runs while the daemon is active, the daemon's in-memory state doesn't get updated to reflect the changes in state.json. This causes the daemon to continue operating based on stale state.

## Observed Behavior

1. Worker is in "working" state (stuck due to race condition bug)
2. User runs `llmc reset auto-1 --yes`
3. Reset command updates state.json to `"status": "offline"`
4. `llmc status` correctly shows worker as "offline"
5. Daemon log continues showing `auto_worker_states="[(\"auto-1\", Working)]"`
6. Daemon doesn't restart the worker or assign new tasks

## Evidence

**state.json after reset:**
```json
{
  "workers": {
    "auto-1": {
      "status": "offline",
      ...
    }
  }
}
```

**Daemon log after reset (same iteration pattern):**
```json
{"message":"Auto mode loop iteration","iteration":35,"auto_worker_states":"[(\"auto-1\", Working)]"}
```

## Expected Behavior

The daemon should detect state changes made by other LLMC commands and update its in-memory state accordingly. Options:

1. Watch state.json for changes and reload on modification
2. Use IPC to notify daemon of state changes
3. Re-read state.json on each patrol iteration

## Impact

- Workers cannot be reset while daemon is running
- Stuck workers cannot be recovered without stopping the daemon
- Operations like `llmc reset`, `llmc stop`, etc. don't work as expected when daemon is running

## Workaround

Stop the daemon (`llmc down`) before running reset commands, then restart.