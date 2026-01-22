# LLMC Isolated Test Environment Setup

This document explains how to run LLMC scenario tests in an isolated environment
that does not interfere with production work or the global `~/llmc` directory.

## Why Isolation is Required

**All scenario tests MUST run in an isolated LLMC instance.** Running tests in
the global `~/llmc` directory risks:

- Interfering with production work and other developers
- Corrupting state files or configuration
- Creating conflicting TMUX sessions
- Leaving orphaned worktrees or branches

LLMC supports multiple concurrent instances through the `LLMC_ROOT` environment
variable and dynamic TMUX session naming. Each instance gets:

- Its own state file (`state.json`) and configuration (`config.toml`)
- Its own worktrees in `$LLMC_ROOT/.worktrees/`
- Uniquely prefixed TMUX sessions (preventing conflicts)
- Its own IPC socket (`llmc.sock`)
- Its own log files

## Setting Up an Isolated Test Environment

Before running any scenario test, create an isolated environment:

```bash
# Create an isolated test directory with a unique name
export TEST_DIR="/tmp/llmc-test-$(date +%s)"
export LLMC_ROOT="$TEST_DIR"

echo "Creating isolated test environment at: $TEST_DIR"

# Initialize a new LLMC instance
llmc init --source ~/Documents/GoogleDrive/dreamtides --target "$TEST_DIR"

# Verify isolation - should show empty worker list
llmc status
```

All subsequent commands in your test session will use `$LLMC_ROOT` implicitly.

## Session Naming for Isolation

When `LLMC_ROOT` is set to a non-default path:

- TMUX sessions are prefixed with `llmc-<dirname>-` (e.g.,
  `llmc-llmc-test-1737500000-adam`)
- This prevents conflicts with other LLMC instances
- The default `~/llmc` directory uses just `llmc-` prefix (e.g., `llmc-adam`)

## Standard Test Script Template

Use this template for scenario tests:

```bash
#!/bin/bash
set -e

# Setup isolated test environment
TEST_DIR="/tmp/llmc-test-$$"
export LLMC_ROOT="$TEST_DIR"

echo "Creating isolated test environment at: $TEST_DIR"
mkdir -p "$TEST_DIR"

# Initialize LLMC
llmc init --source ~/Documents/GoogleDrive/dreamtides --target "$TEST_DIR"

# Cleanup function
cleanup() {
    echo "Cleaning up test environment..."
    LLMC_ROOT="$TEST_DIR" llmc down --force 2>/dev/null || true
    rm -rf "$TEST_DIR"
}
trap cleanup EXIT

# --- Your test commands here ---
# Use $LLMC_ROOT for all paths, e.g.:
#   $LLMC_ROOT/config.toml
#   $LLMC_ROOT/state.json
#   $LLMC_ROOT/logs/auto.log
#   $LLMC_ROOT/.worktrees/auto-1
#   $LLMC_ROOT/test_scripts/

# --- End of test ---
```

## Key Paths in Isolated Environment

Replace hardcoded `~/llmc` paths with these `$LLMC_ROOT` equivalents:

| Hardcoded Path | Isolated Path |
|----------------|---------------|
| `~/llmc` | `$LLMC_ROOT` |
| `~/llmc/config.toml` | `$LLMC_ROOT/config.toml` |
| `~/llmc/state.json` | `$LLMC_ROOT/state.json` |
| `~/llmc/logs/` | `$LLMC_ROOT/logs/` |
| `~/llmc/.worktrees/` | `$LLMC_ROOT/.worktrees/` |
| `~/llmc/.llmc/` | `$LLMC_ROOT/.llmc/` |
| `~/llmc/test_scripts/` | `$LLMC_ROOT/test_scripts/` |

## Key Gotchas

- **Source repo dirty detection**: Any untracked files in the source repo block
  auto-accept with exponential backoff (60s→120s→240s). Backoff state persists
  in state.json across restarts.
- **Claim limits**: `lat pop --max-claims 2` fails and crashes the daemon if 2
  tasks are already claimed. Close stale tasks first.
- **State persistence**: Worker commits survive daemon restarts. Patrol will
  auto-detect and transition idle workers with commits to needs_review.
- **TMUX conflicts**: If you see "session already exists" errors, ensure
  `LLMC_ROOT` is set correctly for isolation.

## Debug Commands

```bash
# Check for errors in isolated instance
grep -E "ERROR|WARN" "$LLMC_ROOT/logs/llmc.jsonl" | tail -20

# Check backoff state
grep -E "dirty|backoff" "$LLMC_ROOT/state.json"

# Clear backoff (if stuck)
# Edit state.json: set source_repo_dirty_* fields to null
```

## Cleanup

Always clean up after tests:

```bash
# Stop the daemon for this instance
llmc down --force

# Remove test directory
rm -rf "$LLMC_ROOT"

# Unset environment variable
unset LLMC_ROOT
```

If cleanup fails, manually kill sessions:

```bash
# Kill processes for THIS instance only
pkill -f "LLMC_ROOT=$LLMC_ROOT" || true

# Or if you know the TMUX session prefix
tmux list-sessions | grep "llmc-llmc-test" | cut -d: -f1 | xargs -I{} tmux kill-session -t {}
```

## Running Multiple Instances

Each LLMC instance is fully isolated:

| Resource | Isolation |
|----------|-----------|
| State file (`state.json`) | Per-instance in `$LLMC_ROOT` |
| Config file (`config.toml`) | Per-instance in `$LLMC_ROOT` |
| Worktrees | Per-instance in `$LLMC_ROOT/.worktrees/` |
| TMUX sessions | Prefixed with unique instance identifier |
| IPC socket (`llmc.sock`) | Per-instance in `$LLMC_ROOT` |
| State lock file | Per-instance in `$LLMC_ROOT` |
| Logs | Per-instance in `$LLMC_ROOT/logs/` |

**Important considerations:**

1. **Source repository conflicts**: Multiple instances can share the same source
   repository, but only one should accept changes to master at a time to avoid
   merge conflicts.

2. **No cross-instance communication**: Instances are completely independent.
   Commands in one instance cannot affect another.

3. **Cleanup**: When done with a test instance, ensure you run `llmc down` and
   remove the directory to avoid orphaned TMUX sessions.
