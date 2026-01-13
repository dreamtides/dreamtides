# Appendix: Git Edge Cases

This appendix documents Lattice's behavior in non-standard git repository
configurations. The design philosophy is **graceful degradation**: Lattice
should continue operating wherever possible, falling back to full index
rebuilds when git metadata is unreliable.

## Detection Strategy

On startup, Lattice detects the repository configuration by examining:

1. `.git/shallow` file presence (shallow clone indicator)
2. `.git` file vs directory (worktree indicator)
3. `.gitmodules` file presence (submodule usage)
4. `git rev-parse --is-shallow-repository` output
5. `git config --get remote.origin.promisor` (partial clone indicator)
6. `git rev-parse --git-dir` output (worktree detection)

The detection result is cached in `.lattice/repo_config.json` and refreshed
when the repository root's `.git` mtime changes.

## Shallow Clones

### Description

Shallow clones truncate commit history at a specified depth, creating
"graft points" where git pretends commits are root commits. Common in CI
environments (GitHub Actions defaults to `depth: 1`).

### Detection

```bash
test -f .git/shallow  # File exists in shallow clones
git rev-parse --is-shallow-repository  # Returns "true"
```

### Implications

| Operation | Impact |
|-----------|--------|
| `git ls-files` | Works normally |
| `git diff <commit>..HEAD` | Fails if commit is beyond shallow boundary |
| `git log` | Truncated at graft point |
| `git blame` | May fail for old lines |

### Lattice Behavior

**Indexing:** Document discovery via `git ls-files` works normally. Full
rebuild is always safe.

**Incremental reconciliation:** If `index_metadata.last_commit` is beyond
the shallow boundary, `git diff` fails. Lattice detects this and falls
back to full rebuild.

**Client ID discovery:** Scanning existing IDs may be incomplete if documents
were added in commits beyond the shallow boundary. Lattice logs a warning
and proceeds with visible IDs only, accepting slightly higher collision risk.

**Recommendation:** For repositories using Lattice heavily, deepen clones
to include full history, or accept that incremental reconciliation will
frequently fall back to full rebuilds.

```bash
# Deepen a shallow clone
git fetch --unshallow
```

## Partial Clones

### Description

Partial clones omit certain objects (blobs or trees) during initial clone,
fetching them on-demand. Two primary variants:

- **Blobless** (`--filter=blob:none`): Full commit/tree history, blobs
  fetched on checkout
- **Treeless** (`--filter=tree:0`): Only current tree, minimal history

### Detection

```bash
git config --get remote.origin.promisor  # Non-empty if partial clone
git config --get remote.origin.partialclonefilter  # Filter specification
```

### Implications

| Clone Type | ls-files | diff | checkout | blame |
|------------|----------|------|----------|-------|
| Blobless | Works | Works | May fetch | May fetch |
| Treeless | Works | Slow | May fetch | Slow |

### Lattice Behavior

**Blobless clones:** Fully supported. Document discovery and change
detection work normally. Reading document content may trigger blob fetches,
which is acceptable—Lattice needs the content anyway.

**Treeless clones:** Supported with caveats. Operations that traverse
history (finding highest client ID) may be slow or trigger network
fetches. Lattice logs a warning recommending blobless clones for
development use.

**Network awareness:** Lattice assumes network availability in partial
clones. If offline, blob/tree fetches fail, and Lattice reports a clear
error suggesting the user go online or convert to a full clone.

## Sparse Checkout

### Description

Sparse checkout limits which files are materialized in the working
directory. Combined with partial clones, this enables efficient work on
large monorepos.

### Detection

```bash
git config --get core.sparseCheckout  # "true" if enabled
git sparse-checkout list  # Returns checked-out patterns
```

### Implications

- `git ls-files` returns ALL tracked files, not just checked-out ones
- Working directory contains only sparse-checkout patterns
- Files outside sparse patterns exist in git but not on disk

### Lattice Behavior

**Document discovery:** Lattice filters `git ls-files` output against
files actually present on disk. A file tracked by git but not
materialized is skipped with a debug log entry.

**Cross-references:** Links to documents outside the sparse checkout
are valid but unresolvable locally. `lat check` reports these as
warnings (not errors) with the message "Document {id} is in sparse
exclusion zone."

**Expanding sparse checkout:** When a user requests `lat show <id>` for
a document outside the sparse checkout, Lattice offers to expand the
sparse pattern:

```
Document LXYZ is outside your sparse checkout.
Run: git sparse-checkout add path/to/document.md
Then retry: lat show LXYZ
```

**Configuration:** The `[sparse]` section in `.lattice/config.toml`
controls behavior:

```toml
[sparse]
# Warn about links to non-materialized documents (default: true)
warn_sparse_links = true
# Automatically expand sparse checkout for lat show (default: false)
auto_expand = false
```

## Git Worktrees

### Description

Worktrees enable multiple working directories sharing a single repository.
Each worktree has its own HEAD, index, and working tree, but shares the
object database and refs.

### Detection

```bash
git rev-parse --git-dir  # Returns path like "../.git/worktrees/name"
git worktree list  # Lists all worktrees
```

The `.git` entry in a worktree is a file (not directory) containing:
```
gitdir: /path/to/main/.git/worktrees/name
```

### Implications

- Multiple `.lattice/` directories (one per worktree)
- Each worktree has independent index state
- Client ID applies to all worktrees (shared config)
- Document IDs must be unique across all worktrees

### Lattice Behavior

**Index isolation:** Each worktree maintains its own `.lattice/index.sqlite`.
This ensures worktrees on different branches have appropriate indices.

**Client ID sharing:** The `~/.lattice.toml` client ID mapping uses the
main repository path (resolved via `git rev-parse --git-common-dir`), not
the worktree path. All worktrees share one client ID.

**ID uniqueness:** When generating new IDs, Lattice queries documents
across all worktrees by examining the shared object database. This prevents
ID collisions between worktrees on different branches.

**Reconciliation:** Each worktree reconciles independently based on its
own HEAD. The main repository's index doesn't affect worktree indices.

**Recommended workflow:**

```bash
# Main worktree
cd /project
lat check  # Validates main branch

# Feature worktree
cd /project-feature
lat check  # Validates feature branch independently
```

## Git Submodules

### Description

Submodules embed external repositories within a parent repository. The
parent tracks a specific commit of each submodule.

### Detection

```bash
test -f .gitmodules  # File exists if submodules configured
git submodule status  # Lists submodules and their state
```

### Implications

- Submodule directories contain separate `.git` references
- `git ls-files` in parent does NOT include submodule contents
- Each submodule is an independent repository

### Lattice Behavior

**Scope isolation:** Lattice operates within a single repository boundary.
Documents in submodules are NOT indexed by the parent repository's Lattice
instance. Each submodule can have its own `.lattice/` directory and
independent document graph.

**Cross-repository links:** Links to documents in submodules are not
supported. The Lattice ID namespace is repository-scoped. Attempting to
reference a submodule document's ID from the parent produces a "missing
ID" error.

**Recommended patterns:**

1. **Separate knowledge bases:** Treat each submodule as independent.
   Run `lat` commands within submodule directories.

2. **Unified view:** If cross-submodule linking is required, consider:
   - Using subtree instead of submodule
   - Maintaining a separate "meta" repository with documents that
     reference all projects

**Nested submodules:** The same isolation applies recursively. Lattice
does not traverse into nested submodules.

## Detached HEAD State

### Description

HEAD points directly to a commit rather than a branch ref. Common during:
- `git checkout <commit>`
- `git bisect`
- CI pipelines checking out specific commits

### Detection

```bash
git symbolic-ref HEAD  # Fails with "not a symbolic ref" if detached
git rev-parse --abbrev-ref HEAD  # Returns "HEAD" if detached
```

### Lattice Behavior

**Normal operations:** All Lattice operations work normally in detached
HEAD. Document discovery, indexing, and queries proceed unchanged.

**Branch-based features:** Features that reference branch names (e.g.,
"documents modified on this branch") report "detached HEAD" instead of
a branch name.

**Index watermark:** The `last_commit` field stores the commit hash,
which works regardless of whether HEAD is detached.

**Warning suppression:** Lattice does not warn about detached HEAD
since it's a legitimate workflow state.

## In-Progress Git Operations

### Description

During rebase, merge, cherry-pick, or revert, the repository is in a
transitional state with special files indicating progress.

### Detection

```bash
test -d .git/rebase-merge  # Interactive rebase in progress
test -d .git/rebase-apply  # Rebase via git am
test -f .git/MERGE_HEAD    # Merge in progress
test -f .git/CHERRY_PICK_HEAD  # Cherry-pick in progress
test -f .git/REVERT_HEAD   # Revert in progress
```

### Implications

- Files may contain conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`)
- HEAD may point to intermediate states
- Working directory may be inconsistent with any single commit

### Lattice Behavior

**Conflict detection:** During YAML frontmatter parsing, Lattice detects
git conflict markers and skips the file with a warning:

```
Warning: Skipping path/to/doc.md (contains unresolved conflicts)
```

**Partial indexing:** Non-conflicted documents are indexed normally.
The index represents the repository's current filesystem state, not
any specific commit.

**lat check behavior:** Reports conflicts as errors requiring resolution:

```
Error: 3 documents have unresolved merge conflicts:
  - path/to/doc1.md
  - path/to/doc2.md
  - path/to/doc3.md
Run 'git status' to see conflict state.
```

**Post-resolution:** After resolving conflicts and completing the git
operation, run `lat check` to verify document validity and rebuild
index as needed.

## Bare Repositories

### Description

Bare repositories have no working directory, only the git object database.
Used as central/server repositories.

### Detection

```bash
git rev-parse --is-bare-repository  # Returns "true"
```

### Lattice Behavior

**Not supported:** Lattice requires a working directory to read document
content. Running `lat` in a bare repository produces:

```
Error: Lattice requires a working directory.
This appears to be a bare repository.
```

**Server-side hooks:** For pre-receive hooks that need document validation,
clone to a temporary directory first:

```bash
# In pre-receive hook
tmp=$(mktemp -d)
git clone --local . "$tmp"
cd "$tmp" && lat check
rm -rf "$tmp"
```

## Summary Table

| Configuration | Support Level | Notes |
|--------------|---------------|-------|
| Shallow clone | Full | Falls back to rebuild when history unavailable |
| Blobless clone | Full | May trigger network fetches |
| Treeless clone | Degraded | Slow operations, warning logged |
| Sparse checkout | Full | Warns about non-materialized links |
| Worktrees | Full | Each worktree has independent index |
| Submodules | Isolated | Each repo has independent namespace |
| Detached HEAD | Full | No special handling needed |
| In-progress ops | Degraded | Skips conflicted files |
| Bare repository | None | Requires working directory |

## Implementation Notes

### Detection Caching

Repository configuration detection runs once per `lat` invocation and
caches results in memory. The `.lattice/repo_config.json` file persists
detection results across invocations for faster startup:

```json
{
  "detected_at": "2025-01-13T10:30:00Z",
  "git_mtime": 1736765400,
  "is_shallow": false,
  "is_partial": false,
  "partial_filter": null,
  "is_sparse": false,
  "is_worktree": false,
  "main_git_dir": "/project/.git",
  "has_submodules": true,
  "in_progress_op": null
}
```

Cache invalidation occurs when `.git` directory mtime changes.

### Error Messages

All edge-case handling produces clear, actionable error messages:

- State what was detected
- Explain why it matters
- Suggest remediation steps

Example:
```
Warning: This repository uses sparse checkout.
Documents outside your sparse patterns won't be indexed.
Currently checked out: src/**, docs/api/**
To include more: git sparse-checkout add <pattern>
```

### Logging

All edge-case detections are logged to `.lattice/logs.jsonl` with type
`observation`:

```json
{
  "ts": "2025-01-13T10:30:00.123456Z",
  "type": "observation",
  "msg": "Detected shallow clone (depth unknown, graft points present)"
}
```
