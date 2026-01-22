---
lattice-id: LB5WQN
name: 00-llmc-scenarios
description: llmc secnarios
parent-id: LBSWQN
created-at: 2026-01-21T21:52:11.367551Z
updated-at: 2026-01-21T22:38:24.953814Z
---

# [Lattice] Context

You are working on manually testing the `llmc` tool and auto/overseer features

Please run `lat show LDBWQN` to view the primary design document.

llmc code: `@rules_engine/src/llmc/`

See `@rules_engine/docs/llmc.md` for broader context on the llmc system.

Please report all issues encountered during this test via the
`lattice_create_task()` MCP tool, in the `rules_engine/src/llmc/tasks/qa/`
parent directory.

You MUST put the label `llmc-auto` on every task you file.

  Key Gotchas

  - Source repo dirty detection: Any untracked files in the source repo block
    auto-accept with exponential backoff (60s→120s→240s). Backoff state persists
    in state.json across restarts.
  - Claim limits: lat pop --max-claims 2 fails and crashes the daemon if 2 tasks
    are already claimed. Close stale tasks first.
  - State persistence: Worker commits survive daemon restarts. Patrol will
    auto-detect and transition idle workers with commits to needs_review.

  Testing Workflow

  1. Ensure source repo is clean (git status)
  2. Create simple test task via lat create
  3. Start overseer: just llmc overseer --task-pool-command "lat pop
     --max-claims 2"
  4. Monitor with just llmc status and just llmc peek <worker>
  5. Verify via git log and lat show <TASK-ID>

  Key Debug Commands

  # Check for errors

  grep -E "ERROR|WARN" ~/llmc/logs/llmc.jsonl | tail -20

  # Check backoff state

  grep -E "dirty|backoff" ~/llmc/state.json

  # Clear backoff (if stuck)

  # Edit state.json: set source_repo_dirty_* fields to null

  Cleanup

  pkill -f "llmc overseer" || true
  pkill -f "llmc up" || true
  lat close <TASK-ID>
  just llmc reset --all --yes  # if workers in bad state

# [Lattice] Acceptance Criteria

When complete, please:

1) Run `lat close <ID>` to mark the issue as complete
2) Create a git commit with a description of your work
