---
lattice-id: LCSWQN
name: hook-commands-dont-preserve-llmc-root-en
description: Hook commands don't preserve LLMC_ROOT environment variable for isolated instances
parent-id: LB6WQN
task-type: bug
priority: 1
labels:
- llmc-auto
- auto-mode
- hooks
- multi-instance
- bug
created-at: 2026-01-22T03:00:53.833494Z
updated-at: 2026-01-22T04:50:34.825519Z
closed-at: 2026-01-22T04:50:34.825519Z
---

# Bug: Hook Commands Don't Preserve LLMC_ROOT Environment Variable

## Summary

When running LLMC in an isolated test environment with `LLMC_ROOT` set to a non-default path, the Claude Code hook commands configured in `.claude/settings.json` don't have the `LLMC_ROOT` environment variable set. This causes hook events (SessionStart, SessionEnd, Stop) to be sent to the wrong daemon socket (`~/llmc/llmc.sock` instead of `$LLMC_ROOT/llmc.sock`).

## Root Cause

The hook commands in `.claude/settings.json` are configured as:
```json
{
  "hooks": {
    "SessionStart": [{
      "hooks": [{
        "command": "/path/to/llmc hook session-start --worker auto-1",
        "timeout": 5,
        "type": "command"
      }]
    }]
  }
}
```

When Claude Code executes this hook command, it doesn't inherit the `LLMC_ROOT` environment variable from the parent shell. The `llmc hook` command then uses `get_llmc_root()` which defaults to `~/llmc`.

The socket path logic in `src/ipc/socket.rs`:
```rust
pub fn get_socket_path() -> PathBuf {
    config::get_llmc_root().join("llmc.sock")
}
```

And `get_llmc_root()` in `src/config.rs`:
```rust
pub fn get_llmc_root() -> PathBuf {
    if let Ok(llmc_root) = std::env::var("LLMC_ROOT") {
        return PathBuf::from(llmc_root);
    }
    // Default to ~/llmc
    PathBuf::from(home).join("llmc")
}
```

## Impact

This bug completely breaks:
1. Isolated test environments
2. Running multiple LLMC instances concurrently
3. Any deployment where LLMC_ROOT is not `~/llmc`

Workers stay in "offline" state indefinitely because SessionStart hooks never reach the correct daemon.

## Possible Fixes

### Option 1: Include LLMC_ROOT in hook command
When creating the settings.json, include the LLMC_ROOT value explicitly:
```json
{
  "command": "LLMC_ROOT=/path/to/root /path/to/llmc hook session-start --worker auto-1"
}
```

### Option 2: Add --root flag to hook commands
Add a `--root` parameter to hook commands:
```json
{
  "command": "/path/to/llmc hook session-start --worker auto-1 --root /path/to/root"
}
```

### Option 3: Use absolute socket path in hook command
Add a `--socket` parameter to hook commands:
```json
{
  "command": "/path/to/llmc hook session-start --worker auto-1 --socket /path/to/root/llmc.sock"
}
```

## Reproduction

1. Set up isolated test environment:
   ```bash
   export LLMC_ROOT="/tmp/llmc-test-123"
   llmc init --source ~/repo --target "$LLMC_ROOT"
   ```

2. Start auto mode:
   ```bash
   llmc up --auto
   ```

3. Observe that workers stay "offline" indefinitely
4. Check the settings.json - no LLMC_ROOT in hook commands
5. The hooks are firing but sending to wrong socket

## Related Issue

This is the root cause of the "Auto worker stuck in 'offline' state" issue filed earlier in this test session.