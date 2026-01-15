# Appendix: Git Integration

This appendix documents how Lattice interacts with git for document discovery
and change detection. See [Lattice Design](lattice_design.md#git-integration)
for an overview and [Appendix: Git Edge Cases](appendix_git_edge_cases.md) for
behavior in non-standard repository configurations.

## Design Philosophy

Lattice uses git as the authoritative store for all document content. The
SQLite index is derived entirely from git-tracked files. This design ensures
documents are versioned, mergeable, and recoverable.

All git operations go through the `GitOps` trait, allowing tests to inject
a fake implementation. See [Appendix: Testing Strategy](appendix_testing_strategy.md).

## Required Git Operations

### Document Discovery

**Command:** `git ls-files '*.md'`

Returns all tracked markdown files. This excludes gitignored files and
ensures consistent behavior between clean and dirty working directories.

**Usage:** Full index rebuild, `lat list` without filters.

### Change Detection

**Command:** `git diff --name-only <commit>..HEAD -- '*.md'`

Returns files modified between commits. Used for incremental reconciliation.

**Command:** `git status --porcelain -- '*.md'`

Returns uncommitted changes (staged and unstaged). Used to detect
modifications not yet in any commit.

### Commit Information

**Command:** `git rev-parse HEAD`

Returns current HEAD commit hash. Stored in index for reconciliation.

**Command:** `git log -1 --format='%H' -- <path>`

Returns last commit touching a specific file. Used for staleness detection.

## Repository State Handling

### Clean State

Repository is clean when:
- `git status --porcelain` returns empty
- HEAD points to a branch (not detached)
- No rebase/merge in progress

In clean state, all git-based operations work normally.

### Dirty Working Directory

When uncommitted changes exist:
- Document discovery still uses `git ls-files`
- Untracked .md files are NOT indexed
- Modified files are detected via `git status`
- Index reconciliation processes uncommitted modifications

### Non-Standard Configurations

Lattice supports various git repository configurations including shallow
clones, partial clones, sparse checkout, worktrees, and submodules. Each
configuration has specific behavior implications.

See [Appendix: Git Edge Cases](appendix_git_edge_cases.md) for comprehensive
documentation of detection strategies, behavior modifications, and
recommendations for each configuration.

## Client ID Storage

### Configuration File

Client IDs are stored in `~/.lattice.toml`:

```toml
[clients]
"/path/to/repo" = "DT"
"/other/repo" = "K2"
```

This file persists across repository deletion and re-clone, preserving
the client's ID assignment.

### ID Selection

When a client first uses Lattice in a repository:
1. Check if an ID exists in `~/.lattice.toml` for this path
2. If not, query the repository for all existing client IDs
3. Generate a random ID avoiding existing IDs
4. Store the mapping in `~/.lattice.toml`

### ID Length Rules

- Default: 3-character IDs (32768 possible values) for 0-16 known clients
- 4 characters if >16 known clients (1048576 possible)
- 5 characters if >64 known clients (33554432 possible)
- 6 characters if >256 known clients (1073741824 possible)

## Conflict Detection

`lat check` verifies no two documents share the same Lattice ID. This catches:
- Copy-paste errors
- Failed merge resolutions
- Manual ID assignment mistakes
- Client ID collisions from parallel contributors

## Branch Operations

### Merge Handling

During merge:
- Conflicting documents may have invalid frontmatter
- Lattice operations log warnings but don't fail
- After conflict resolution, run `lat check` to verify

### Rebase Handling

During interactive rebase:
- Documents may be in transitional states
- Index reconciliation handles missing commits gracefully
- Full rebuild after rebase completes

## File Operations

### Document Creation

New documents are created as regular files. They become tracked when
staged and committed. Until committed, they appear in `git status` but
not in `git ls-files`.

### Document Deletion

Deleting a document removes it from git and triggers index cleanup during
reconciliation. References to deleted documents become "missing ID" errors
detected by `lat check`.

### Document Rename

Git detects renames automatically. The index handles path changes during
reconciliation. Lattice IDs remain stable across renames.

## Performance Notes

### Git Command Overhead

Each git command spawns a subprocess. Lattice minimizes git calls:
- Single `git ls-files` for full enumeration
- Single `git diff` for change detection
- Single `git status` for uncommitted changes

### Large Repositories

For repositories with many files:
- Git operations are fast (git is optimized for this)
- Path filtering (`-- '*.md'`) limits scope
- Only markdown files are processed

### Network Operations

Lattice never performs network git operations (fetch, pull, push). These
are left to the user's normal git workflow. This prevents unexpected
network access and authentication problems.
