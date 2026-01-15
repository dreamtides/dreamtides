# Appendix: Doctor Command

This appendix documents the `lat doctor` command for system health diagnostics.
See [Lattice Design](lattice_design.md#linter-and-formatter) for how this
relates to document validation.

## Purpose

The `lat doctor` command diagnoses infrastructure and system health issues,
complementing `lat check` which validates document content:

| Command | Scope | Examples |
|---------|-------|----------|
| `lat check` | Document content | Broken links, invalid frontmatter, circular deps |
| `lat doctor` | System health | Index corruption, stale claims, config errors |

Run `lat doctor` when experiencing unexpected behavior, after crashes, or
periodically as a health check.

## Usage

```
lat doctor [options]

OPTIONS:
  --fix        Automatically repair fixable issues
  --dry-run    Preview fixes without applying (requires --fix)
  --deep       Run additional integrity checks (slower)
  --json       Machine-readable output
  --quiet      Only show warnings and errors
```

## Check Categories

### Core System

| Check | Description | Severity |
|-------|-------------|----------|
| Installation | `.lattice/` directory exists | Error |
| Index exists | `index.sqlite` present | Error |
| Schema version | Index schema matches CLI version | Warning |
| WAL health | No corrupted `-wal` or `-shm` files | Error |

### Index Integrity

| Check | Description | Severity |
|-------|-------------|----------|
| Filesystem sync | Every indexed ID has corresponding file | Error |
| Coverage | Every `.md` with `lattice-id` is indexed | Warning |
| Duplicate IDs | No ID appears twice in index | Error |
| Closed state | `is_closed` matches `.closed/` in path | Warning |
| Root state | `is_root` matches filename = directory | Warning |
| Parent consistency | All `parent_id` values reference existing docs | Warning |

### Git Integration

| Check | Description | Severity |
|-------|-------------|----------|
| Repository | Valid git repository detected | Error |
| Edge cases | Detect shallow/sparse/worktree/submodule | Info |
| Working tree | No in-progress merge/rebase with conflicts | Warning |
| Detached HEAD | HEAD not pointing to branch | Info |

### Configuration

| Check | Description | Severity |
|-------|-------------|----------|
| User config | `~/.lattice.toml` parseable if present | Warning |
| Repo config | `.lattice/config.toml` valid if present | Warning |
| Client ID | Client ID assigned for this repository | Warning |
| Config values | All values within valid ranges | Warning |

### Claims

| Check | Description | Severity |
|-------|-------------|----------|
| Stale claims | No claims for closed tasks | Warning |
| Missing tasks | No claims for deleted tasks | Warning |
| Orphaned worktrees | No claims for non-existent worktree paths | Warning |

### Skills

| Check | Description | Severity |
|-------|-------------|----------|
| Symlink validity | All `.claude/skills/` symlinks resolve | Warning |
| Symlink coverage | All `skill: true` documents have symlinks | Warning |
| Symlink staleness | Symlinks point to current document paths | Warning |

## Output Format

### Default Output

```
$ lat doctor
lat doctor v0.1.0

CORE SYSTEM
  ✓  Installation .lattice/ directory found
  ✓  Index Database index.sqlite exists (245 documents)
  ✓  Schema Version 1 (current)
  ✓  WAL Health No corruption detected

INDEX INTEGRITY
  ✓  Filesystem Sync All 245 indexed documents exist
  ✓  Coverage All documents indexed
  ✓  No Duplicates No duplicate IDs
  ✓  Closed State All is_closed flags correct

GIT INTEGRATION
  ✓  Repository Valid git repository
  ℹ  Configuration Sparse checkout detected
     └─ 3 documents outside sparse patterns
  ✓  Working Tree Clean (no conflicts)

CONFIGURATION
  ✓  User Config ~/.lattice.toml valid
  ✓  Repo Config .lattice/config.toml valid
  ✓  Client ID Assigned: DTX

CLAIMS
  ✓  Active Claims 2 active claims
  ⚠  Stale Claims 1 claim for closed task
     └─ LXXXXX: task closed, claim not released

SKILLS
  ✓  Symlinks 3 skill symlinks valid
  ✓  Coverage All skill documents linked

──────────────────────────────────────────
✓ 15 passed  ⚠ 1 warning  ℹ 1 info  ✖ 0 failed

⚠  WARNINGS
  1. Stale Claims: 1 claim for closed task
     └─ Fix: lat doctor --fix (or: lat claim --gc)
```

### JSON Output

```json
{
  "version": "0.1.0",
  "checks": [
    {
      "category": "core",
      "name": "installation",
      "status": "passed",
      "message": ".lattice/ directory found"
    },
    {
      "category": "claims",
      "name": "stale_claims",
      "status": "warning",
      "message": "1 claim for closed task",
      "details": ["LXXXXX"],
      "fixable": true,
      "fix_command": "lat claim --gc"
    }
  ],
  "summary": {
    "passed": 15,
    "warnings": 1,
    "info": 1,
    "failed": 0
  }
}
```

## Fixable Issues

The `--fix` flag automatically repairs these issues:

| Issue | Fix Action |
|-------|------------|
| Missing index | Full rebuild from filesystem |
| Schema mismatch | Rebuild index with current schema |
| WAL corruption | Delete `-wal` and `-shm` files, rebuild |
| Index-filesystem desync | Rebuild index |
| Incorrect `is_closed` flags | Update flags from paths |
| Incorrect `is_root` flags | Update flags from paths |
| Stale claims | Delete claim files |
| Orphaned claims | Delete claim files |
| Missing skill symlinks | Create symlinks |
| Broken skill symlinks | Recreate symlinks |
| Stale skill symlinks | Update symlink targets |

**Non-fixable issues** require manual intervention:

| Issue | Manual Action |
|-------|---------------|
| Missing `.lattice/` directory | Run `lat init` (if implemented) or create manually |
| Invalid config syntax | Edit config file |
| Git conflicts | Resolve conflicts with git |
| Duplicate IDs in files | Use `lat track --force` to regenerate |

### Fix Modes

```bash
lat doctor --fix              # Fix all, prompt for confirmation
lat doctor --fix --dry-run    # Preview fixes without applying
lat doctor --fix --yes        # Fix all without confirmation
```

## Deep Mode

The `--deep` flag enables additional checks that may be slow on large
repositories:

| Check | Description |
|-------|-------------|
| Full link validation | Verify all link targets exist (like `lat check`) |
| Content hash verification | Recompute body hashes, detect index staleness |
| FTS index integrity | Verify full-text search index matches documents |
| View count consistency | Verify `documents.view_count` matches `views` table |

Deep mode is recommended after crashes or suspected corruption.

## Comparison with lat check

| Aspect | `lat check` | `lat doctor` |
|--------|-------------|--------------|
| Scope | Document content | System infrastructure |
| Speed | Fast (uses index) | Fast (index + filesystem) |
| Deep mode | N/A | Available for thorough checks |
| Auto-fix | Some formatting issues | Index, claims, symlinks |
| When to use | Before commits | After crashes, periodic health |

**Typical workflow:**

```bash
# Regular development
lat check                    # Validate documents before commit

# Troubleshooting / periodic
lat doctor                   # Check system health
lat doctor --fix             # Repair any issues
lat check --rebuild-index    # Force index rebuild if needed
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All checks passed (or only info) |
| 1 | System error during checks |
| 2 | One or more checks failed (errors) |
| 3 | One or more warnings (no errors) |
