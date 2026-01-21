---
lattice-id: LROWQN
name: detailed-design
description: |-
  Repository concepts, key design decisions, and state file changes for the
  unified architecture migration.
parent-id: LRMWQN
created-at: 2026-01-19T05:00:00Z
updated-at: 2026-01-21T22:38:24.859914Z
---

# Detailed Design

## Repository Concepts

| Concept | Old Architecture | New Architecture |
|---------|------------------|------------------|
| Git operations repo | `~/llmc/` | `config.repo.source` |
| Worktree parent dir | `~/llmc/.worktrees/` | `<source>/.llmc-worktrees/` |
| Metadata dir | `~/llmc/` | `~/llmc/` (unchanged) |
| Remote for fetch | `origin` (source repo) | N/A (no remote needed) |
| Master reference | `origin/master` | `master` |

## Key Design Decisions

### Decision 1: Worktree Location

Worktrees will be stored in `<source>/.llmc-worktrees/` rather than a separate
directory because:

- Git requires worktrees to be accessible relative to the main repo
- Keeping them in the repo directory is conventional
- The `.llmc-worktrees/` prefix clearly identifies them as LLMC-managed
- They can be gitignored to avoid clutter

### Decision 2: Branch Naming

Worker branches remain `llmc/<worker-name>` (e.g., `llmc/adam`). These branches
will now exist directly in the main repository.

### Decision 3: No Remote Operations

The new architecture eliminates all remote operations within LLMC. The main repo
may have its own remotes (GitHub, etc.) but LLMC never interacts with them.
All operations are local.

### Decision 4: Master Branch Reference

All references to `origin/master` become simply `master`. This is a significant
change that affects many files.

## State File Changes

The `state.json` format remains unchanged. The `worktree_path` field will now
point to paths under `<source>/.llmc-worktrees/` instead of
`~/llmc/.worktrees/`.

## Gitignore Updates

The source repository's `.gitignore` should include:
```

# LLMC worktrees

.llmc-worktrees/
```

This will be added automatically by `llmc init`.
