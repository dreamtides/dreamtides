---
lattice-id: LCAWQN
name: rollback-plan
description: |-
  Rollback procedures if migration fails mid-way or issues are discovered
  post-migration.
parent-id: LRMWQN
created-at: 2026-01-19T05:00:00Z
updated-at: 2026-01-21T22:38:24.892440Z
---

# Rollback Plan

## If Migration Fails Mid-Way

```bash

# 1. Stop daemon

llmc down --force --kill-consoles

# 2. Restore backup

rm -rf ~/llmc
cp -r ~/llmc.backup ~/llmc

# 3. Remove new worktree directory

rm -rf ~/Documents/GoogleDrive/dreamtides/.llmc-worktrees

# 4. Revert code changes

git checkout HEAD -- rules_engine/src/llmc/

# 5. Rebuild

cargo build -p llmc

# 6. Restart

llmc up
```

## If Issues Discovered Post-Migration

```bash

# 1. Salvage all work first

mkdir -p ~/llmc-emergency
for worker in $(llmc status --json | jq -r '.workers[].name'); do
    llmc salvage $worker --patch ~/llmc-emergency/$worker.patch || true
done

# 2. Follow rollback steps above

# 3. Apply patches to old system

for patch in ~/llmc-emergency/*.patch; do
    worker=$(basename $patch .patch)
    cd ~/llmc/.worktrees/$worker
    git apply $patch
done
```

## Point of No Return

The migration is reversible until:

1. New commits are accepted (merged to master)
2. Old `~/llmc/.git` is deleted

Always keep `~/llmc.backup` until confident in new system.

## Appendix: Key Paths After Migration

```
~/llmc/config.toml                          # Configuration
~/llmc/state.json                           # Worker state
~/llmc/logs/                                # Log files
~/Documents/GoogleDrive/dreamtides/         # Git repo (all operations)
~/Documents/GoogleDrive/dreamtides/.llmc-worktrees/  # Worker worktrees
```

## Appendix: Glossary

| Term | Definition |
|------|------------|
| Source repo | The main git repository (`~/Documents/GoogleDrive/dreamtides`) |
| Metadata dir | Directory containing LLMC config and state (`~/llmc`) |
| Worktree dir | Directory containing worker worktrees (`<source>/.llmc-worktrees/`) |
| Worker | A Claude Code session with its own worktree |
| Worktree | A git worktree - separate working directory sharing the same repo |
| Accept | Merging a worker's changes to master |
| Patrol | Background process maintaining system health |
