---
lattice-id: LB5WQN
name: 00-llmc-scenarios
description: llmc scenarios
parent-id: LBSWQN
created-at: 2026-01-21T21:52:11.367551Z
updated-at: 2026-01-21T22:38:24.953814Z
---

# [Lattice] Context

You are working on manually testing the `llmc` tool and auto/overseer features.

Please run `lat show LDBWQN` to view the primary design document.

llmc code: `@rules_engine/src/llmc/`

See `@rules_engine/docs/llmc.md` for broader context on the llmc system.

Please report all issues encountered during this test via the
`lattice_create_task()` MCP tool, in the `rules_engine/src/llmc/tasks/qa/`
parent directory.

You MUST put the label `llmc-auto` on every task you file.

## CRITICAL: Isolated Test Environment Setup

**These tests MUST run in an isolated LLMC instance, NOT the global `~/llmc`
directory.** Running tests in the global directory risks interfering with
production work and other developers.

LLMC supports multiple concurrent instances through the `LLMC_ROOT` environment
variable and dynamic TMUX session naming. Each instance gets:
- Its own state file and configuration
- Its own worktrees and branches
- Uniquely prefixed TMUX sessions (preventing conflicts)

### Setting Up an Isolated Test Environment

```bash
# Create an isolated test directory
export TEST_LLMC_ROOT=/tmp/llmc-test-$(date +%s)
mkdir -p "$TEST_LLMC_ROOT"

# Initialize a new LLMC instance in the test directory
LLMC_ROOT="$TEST_LLMC_ROOT" llmc init --source ~/Documents/GoogleDrive/dreamtides

# All subsequent commands use the isolated instance
export LLMC_ROOT="$TEST_LLMC_ROOT"

# Verify isolation
llmc status  # Should show empty worker list
```

### Session Naming for Isolation

When `LLMC_ROOT` is set to a non-default path:
- TMUX sessions are prefixed with `llmc-<dirname>-` (e.g., `llmc-llmc-test-1234567890-adam`)
- This prevents conflicts with other LLMC instances
- The default `~/llmc` directory uses just `llmc-` prefix (e.g., `llmc-adam`)

### Test Script Template

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

# Run your test...
# <test commands here>

# Cleanup
cleanup() {
    echo "Cleaning up test environment..."
    LLMC_ROOT="$TEST_DIR" llmc down --force 2>/dev/null || true
    rm -rf "$TEST_DIR"
}
trap cleanup EXIT
```

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

## Testing Workflow

1. **Create isolated environment** (see template above)
2. Set `LLMC_ROOT` environment variable
3. Ensure source repo is clean (`git status`)
4. Create simple test task via lat create
5. Start overseer: `llmc overseer --task-pool-command "lat pop --max-claims 2"`
6. Monitor with `llmc status` and `llmc peek <worker>`
7. Verify via `git log` and `lat show <TASK-ID>`

## Key Debug Commands

```bash
# Check for errors (use $LLMC_ROOT for isolated instance)
grep -E "ERROR|WARN" "$LLMC_ROOT/logs/llmc.jsonl" | tail -20

# Check backoff state
grep -E "dirty|backoff" "$LLMC_ROOT/state.json"

# Clear backoff (if stuck)
# Edit state.json: set source_repo_dirty_* fields to null
```

## Cleanup

```bash
# Kill processes for THIS instance only
pkill -f "LLMC_ROOT=$LLMC_ROOT" || true

# Or if you know the TMUX session prefix
tmux list-sessions | grep "llmc-llmc-test" | cut -d: -f1 | xargs -I{} tmux kill-session -t {}

# Remove test directory
rm -rf "$LLMC_ROOT"
```

# [Lattice] Acceptance Criteria

When complete, please:

1) Run `lat close <ID>` to mark the issue as complete
2) Create a git commit with a description of your work
