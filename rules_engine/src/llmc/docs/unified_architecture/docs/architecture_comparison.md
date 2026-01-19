---
lattice-id: LBVWQN
name: architecture-comparison
description: |-
  Comparison of current dual-repository LLMC architecture vs new unified
  worktree architecture with diagrams.
parent-id: LBUWQN
created-at: 2026-01-19T05:00:00Z
updated-at: 2026-01-19T05:08:18.073587Z
---

# Architecture Comparison

## Current Architecture (Dual-Repository)

```
~/llmc/                               # LLMC workspace (FULL GIT CLONE)
├── .git/                             # Complete clone of source repo
│   └── config                        # origin = ~/Documents/.../dreamtides
├── .worktrees/                       # Worker worktrees (linked to ~/llmc/.git)
│   ├── adam/
│   │   ├── .git                      # File pointing to ~/llmc/.git/worktrees/adam
│   │   └── ... (working files)
│   └── baker/
├── config.toml
├── state.json
├── llmc.sock
└── logs/

~/Documents/GoogleDrive/dreamtides/   # Source repository (truth)
├── .git/
└── ... (working files)
```

### Current Accept Flow

```

1. Worker commits in worktree (~/llmc/.worktrees/adam)
2. Rebase worktree onto origin/master
3. Squash commits
4. Checkout master in ~/llmc/
5. Fast-forward merge worker branch to ~/llmc/ master
6. Fetch new commit FROM ~/llmc/ INTO source repo
7. Reset source repo to new commit
8. Recreate worker worktree

```

### Problems with Current Architecture

- Two repositories that can drift out of sync
- Complex accept flow with multiple failure points
- The fetch-from-local step is unusual and error-prone
- Disk space duplication (even with --local, git objects are hard-linked but
  new objects are not shared)
- Mental overhead of understanding two repos

## New Architecture (Unified)

```
~/Documents/GoogleDrive/dreamtides/   # Source repository (SINGLE REPO)
├── .git/
│   ├── worktrees/                    # Git's internal worktree tracking
│   │   ├── adam/
│   │   └── baker/
│   └── ...
├── .llmc-worktrees/                  # Worker worktrees (NEW LOCATION)
│   ├── adam/
│   │   ├── .git                      # File pointing to main .git/worktrees/adam
│   │   └── ... (working files)
│   └── baker/
└── ... (normal repo contents)

~/llmc/                               # Metadata only (NO .git/)
├── config.toml
├── state.json
├── llmc.sock
└── logs/
```

### New Accept Flow

```

1. Worker commits in worktree
2. Rebase worktree onto master (local ref, not origin/master)
3. Squash commits
4. Fast-forward merge worker branch directly to master
5. Recreate worker worktree

```

### Benefits

- Single source of truth
- Simpler accept flow (5 steps vs 8)
- No sync issues possible
- Worker branches visible in main repo (can be useful for debugging)
- Standard git worktree usage pattern
