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

## CRITICAL: Isolated Test Environment

**All scenario tests MUST run in an isolated LLMC instance.**

See `isolated_test_environment.md` in this directory for complete setup
instructions, including:

- Why isolation is required
- How to set up an isolated test environment
- Standard test script template
- Key paths and environment variables
- Debug commands and cleanup procedures

## Quick Start

```bash
# Create isolated test environment
export TEST_DIR="/tmp/llmc-test-$(date +%s)"
export LLMC_ROOT="$TEST_DIR"

llmc init --source ~/Documents/GoogleDrive/dreamtides --target "$TEST_DIR"

# Run your test...
# All commands use $LLMC_ROOT implicitly

# Cleanup when done
llmc down --force
rm -rf "$TEST_DIR"
unset LLMC_ROOT
```

## Testing Workflow

1. **Create isolated environment** (see `isolated_test_environment.md`)
2. Set `LLMC_ROOT` environment variable
3. Ensure source repo is clean (`git status`)
4. Create simple test task via lat create (or use test_scripts)
5. Start overseer: `llmc overseer --task-pool-command "lat pop --max-claims 2"`
6. Monitor with `llmc status` and `llmc peek <worker>`
7. Verify via `git log` and `lat show <TASK-ID>`

# [Lattice] Acceptance Criteria

When complete, please:

1) Run `lat close <ID>` to mark the issue as complete
2) Create a git commit with a description of your work
