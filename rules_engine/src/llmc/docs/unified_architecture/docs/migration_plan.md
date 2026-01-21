---
lattice-id: LRIWQN
name: migration-plan
description: |-
  Prerequisites, step-by-step migration instructions, and verification checklist
  for unified architecture migration.
parent-id: LRMWQN
created-at: 2026-01-19T05:00:00Z
updated-at: 2026-01-21T22:38:24.881147Z
---

# Migration Plan

## Prerequisites

Before beginning migration:

1. **All workers must be idle** with no pending work
2. **Backup existing state**: `cp -r ~/llmc ~/llmc.backup`
3. **Verify no TMUX sessions running**: `tmux list-sessions | grep llmc`
4. **Commit any work in source repo**

## Migration Steps

```bash

# 1. Stop LLMC daemon

llmc down --kill-consoles

# 2. Verify all workers idle

llmc status  # All should show "idle"

# 3. Salvage any work (paranoid mode)

for worker in $(llmc status --json | jq -r '.workers[].name'); do
    llmc salvage $worker --output ~/llmc-backup/$worker.patch 2>/dev/null || true
done

# 4. Backup current installation

cp -r ~/llmc ~/llmc.backup.$(date +%Y%m%d)

# 5. Nuke all workers

llmc nuke --all --yes

# 6. (After code changes) Reinitialize

llmc init --source ~/Documents/GoogleDrive/dreamtides --force

# 7. Re-add workers

llmc add adam
llmc add baker

# ... etc

# 8. Start daemon

llmc up
```

## Verification Checklist

Post-migration verification:

- [ ] `llmc status` shows all workers offline
- [ ] `llmc up` starts daemon successfully
- [ ] `llmc status` shows all workers idle
- [ ] Source repo `.gitignore` contains `.llmc-worktrees/`
- [ ] Worktrees exist under `<source>/.llmc-worktrees/`
- [ ] No `.git/` directory in `~/llmc/`
- [ ] `llmc doctor` reports no issues
- [ ] Test full cycle: start → work → accept
