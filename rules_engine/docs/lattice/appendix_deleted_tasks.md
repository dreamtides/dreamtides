# Appendix: Deleted Task Reconstruction

This appendix describes a proposed alternative to the `.closed/` directory
system where closed tasks are deleted from the working tree and reconstructed
from git history on demand.

## Motivation

The current system moves closed tasks to `.closed/` subdirectories. While this
preserves closed task content and makes it visible to all collaborators, it has
drawbacks:

- **Repository clutter:** Closed tasks accumulate in `.closed/` directories
  over time, requiring periodic `lat prune` to clean up
- **Two-step closure:** Users must first close tasks, then later prune them—an
  extra cognitive burden
- **Link maintenance:** Moving files requires rewriting all links pointing to
  the closed task
- **Directory proliferation:** Every `tasks/` directory gains a `.closed/`
  sibling

The proposed system eliminates `.closed/` directories entirely. Closing a task
deletes it from the working tree. The task remains accessible via git history
and can be reconstructed on demand (e.g., for `lat show` of a deleted task).

## Design Overview

### Core Principle: IDs in Filenames

To efficiently locate deleted tasks in git history, we require that Lattice IDs
appear in task filenames. This enables fast git log queries without parsing
file content.

**Filename format:** `{name}_{lattice-id}.md`

Examples:
- `fix_login_bug_LXXXXX.md`
- `oauth_support_LYYYYY.md`
- `auth_design_LZZZZZ.md`

The ID appears at the end, before the `.md` extension, separated by underscore.
This keeps the human-readable name first while making the ID easy to extract
via pattern matching.

### Task Lifecycle Changes

```
         lat close
  open ───────────────► (deleted from working tree)
    ▲
    │ lat reopen
    │
(reconstruct from git)
```

**lat close:** Deletes the task file, removes from index, commits the deletion.
References in `blocking`/`blocked-by` fields of other tasks are cleaned up.

**lat reopen:** Reconstructs the task from git history, writes it back to the
working tree, and re-indexes it.

**lat show:** If the ID is not found in the index, searches git history for the
deleted file and displays its last-known state.

### Deleted Task Index

A new index table tracks deleted tasks for fast lookup:

```sql
CREATE TABLE deleted_tasks (
    id TEXT PRIMARY KEY,           -- Lattice ID
    last_path TEXT NOT NULL,       -- Path before deletion
    deleted_commit TEXT NOT NULL,  -- Commit that deleted the file
    last_content_commit TEXT NOT NULL, -- Last commit with file content
    deleted_at TEXT NOT NULL,      -- ISO 8601 timestamp
    parent_id TEXT,                -- Parent at time of deletion
    description TEXT,              -- Task description (cached)
    task_type TEXT,                -- Task type (cached)
    priority INTEGER               -- Priority (cached)
);

CREATE INDEX idx_deleted_tasks_parent ON deleted_tasks(parent_id);
CREATE INDEX idx_deleted_tasks_type ON deleted_tasks(task_type);
```

This table is populated during `lat close` and updated during reconciliation
by scanning git history for deleted `.md` files.

## Git History Queries

### Finding a Deleted File by ID

With IDs in filenames, finding a deleted task is a single git command:

```bash
git log --all --full-history --diff-filter=D --name-only \
    --format='%H' -- '*_LXXXXX.md'
```

This returns the commit that deleted the file and the file's path. To get the
file content:

```bash
git show <parent-commit>:<path>
```

Where `<parent-commit>` is the parent of the deletion commit.

### Bulk Discovery of Deleted Tasks

During full index rebuild, discover all deleted tasks:

```bash
git log --all --full-history --diff-filter=D --name-only \
    --format='COMMIT:%H' -- '*_L*.md' | \
    awk '/^COMMIT:/{commit=$0; next} /\.md$/{print commit, $0}'
```

This efficiently finds all deleted Lattice documents in a single pass.

### Performance Characteristics

| Operation | Command | Typical Time (10k commits) |
|-----------|---------|----------------------------|
| Find single deleted task | `git log -- '*_LXXXXX.md'` | <50ms |
| Bulk discovery | `git log --all -- '*_L*.md'` | <2s |
| Reconstruct content | `git show <commit>:<path>` | <10ms |

The filename pattern matching makes these queries fast because git can use
its internal path indexing rather than searching file content.

### Shallow Clone Handling

In shallow clones, deleted tasks beyond the shallow boundary are
inaccessible. The `deleted_tasks` index table may be incomplete. Lattice
handles this gracefully:

- `lat show <deleted-id>` returns "Task deleted, history unavailable
  (shallow clone)"
- `lat list --include-closed` shows only deleted tasks within the shallow
  boundary
- A warning is logged suggesting `git fetch --unshallow` for full history

## Command Changes

### lat close

**Current behavior:** Moves task to `.closed/` subdirectory, rewrites links.

**New behavior:**
1. Cache task metadata to `deleted_tasks` index table
2. Remove task references from `blocking`/`blocked-by` fields of other tasks
3. Remove inline markdown links (convert to plain text or delete)
4. Delete the task file
5. Remove from main index
6. Stage deletion for commit (or auto-commit with `--commit`)

**Options:**
- `--commit`: Auto-commit the deletion
- `--message <text>`: Closure note (stored in commit message)
- `--keep-refs`: Preserve inline links (they become dangling references)

### lat reopen

**Current behavior:** Moves task from `.closed/` back to parent directory.

**New behavior:**
1. Look up task in `deleted_tasks` table
2. Reconstruct file content from git history
3. Write file to original path (or prompt for new path if occupied)
4. Re-index the task
5. Optionally restore `blocked-by` relationships

**Options:**
- `--path <path>`: Override destination path
- `--restore-deps`: Restore dependency relationships

### lat show

**Current behavior:** Displays document from index/filesystem.

**New behavior (for deleted tasks):**
1. If ID not in main index, check `deleted_tasks` table
2. If found, reconstruct from git history
3. Display with `[DELETED]` indicator and deletion metadata

**Output format:**
```
LXXXXX [DELETED] fix-login-bug
Deleted: 2025-01-15T10:30:00Z
Original path: auth/tasks/fix_login_bug_LXXXXX.md

Users receive 401 errors when logging in after using the password reset flow.
...
```

### lat list

**Options:**
- `--include-closed`: Include deleted tasks (queries `deleted_tasks` table)
- `--closed-only`: Show only deleted tasks

Deleted tasks are displayed with a `[deleted]` marker:
```
LXXXXX [bug/P1/deleted] fix-login-bug - Users cannot log in after reset
```

### lat prune

**Removed.** This command becomes unnecessary since closed tasks are already
deleted. Could be repurposed for:
- Clearing the `deleted_tasks` cache
- Permanently erasing tasks from git history (requires `git filter-branch`)

## Index Schema Changes

### documents table

Remove `is_closed` column since it's no longer relevant—documents in the
index are by definition not deleted.

### New deleted_tasks table

As specified above, tracks metadata about deleted tasks for fast queries
without hitting git.

### Reconciliation Changes

**Fast path:** Unchanged.

**Incremental path:** When `git diff` shows deleted `.md` files:
1. Parse the deleted file's content from the parent commit
2. Extract Lattice ID from filename (fast) or frontmatter (fallback)
3. Add to `deleted_tasks` table
4. Remove from `documents` table

**Full rebuild:** After indexing all current documents, run bulk deleted
task discovery to populate `deleted_tasks`.

## Filename Convention

### Format

```
{name}_{lattice-id}.md
```

- `{name}`: Human-readable name, lowercase with underscores (existing convention)
- `{lattice-id}`: Full Lattice ID including L prefix
- Separated by single underscore
- `.md` extension

### Validation

The linter (E013) validates that:
1. Filename ends with `_{LATTICE-ID}.md` pattern
2. The ID in filename matches `lattice-id` in frontmatter
3. The `name` portion matches the `name` field (existing rule)

### Migration

Existing documents without IDs in filenames require migration:

```bash
lat migrate-filenames [--dry-run] [--commit]
```

This command:
1. Renames all Lattice documents to include their ID
2. Updates all links to use new paths
3. Optionally commits the renames

### Impact on Commands

**lat create:** Generates filename with ID: `{name}_{id}.md`

**lat mv:** Preserves ID suffix when renaming.

**lat track:** Adds ID to filename when tracking existing files.

## Link Handling

### When a Task is Closed

Inline markdown links to the closed task present a choice:

1. **Convert to plain text:** `[fix login bug](LXXXXX)` → `fix login bug`
2. **Preserve as dangling:** Keep the link; `lat check` warns about it
3. **Delete the reference:** Remove the entire link

Frontmatter references (`blocking`, `blocked-by`) are always cleaned up—the
closed task ID is removed from these lists.

**Default behavior:** Convert inline links to plain text (option 1).

**Rationale:** Preserving broken links clutters `lat check` output. Silently
deleting references loses information. Converting to plain text preserves
the human-readable context while removing the broken reference.

### Historical Links

When viewing a deleted task via `lat show`, links within that task may
reference:
- Still-existing documents (resolvable)
- Other deleted documents (resolvable via `deleted_tasks`)
- Documents that never existed (unresolvable)

The display indicates link status:
```
See [error handling](error_handling_LJCQ2X.md#LJCQ2X) for more information
                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                     [exists]

See [old design](old_design_LZZZZZ.md#LZZZZZ) for context
                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                 [deleted 2025-01-10]
```

## Edge Cases

### Recreating a Deleted Task

If a user creates a new task with the same ID as a deleted task (e.g., by
manually specifying an ID or via collision):

1. The new task takes precedence
2. The `deleted_tasks` entry is removed
3. The old deleted task becomes permanently inaccessible by ID

This is acceptable since:
- ID collisions are rare by design
- The old content is still in git history (searchable by path)
- Users can use git directly if needed

### Restoring Dependencies

When reopening a task that had `blocked-by` relationships:

1. If the blocker still exists and is open, restore the relationship
2. If the blocker is now closed/deleted, don't restore (task is no longer
   blocked)
3. If the blocker doesn't exist (pruned from history), skip silently

### Circular Deletions

If Task A's only reference is from Task B, and both are closed:
1. Close B: Link to A converted to plain text
2. A is now orphaned but still in `deleted_tasks`
3. `lat show A` still works
4. After git gc/repack, very old deleted tasks may become inaccessible

### Git History Rewriting

If git history is rewritten (rebase, filter-branch):
1. `deleted_tasks.last_content_commit` may become invalid
2. On next access, Lattice detects the invalid commit and re-scans
3. If the file is truly gone from history, entry is removed with warning

## Performance Optimizations

### Content Caching

When `lat show` reconstructs a deleted task:
1. Check `content_cache` table first (keyed by commit:path)
2. If cache miss, fetch from git and cache
3. Evict least-recently-used entries when cache exceeds limit

### Batch Operations

When `lat list --include-closed` needs multiple deleted tasks:
1. Collect all needed IDs
2. Single `git show` with multiple paths (faster than N separate calls)
3. Parse responses in memory

### Index Warmup

During startup, if `deleted_tasks` table is empty and repository has history:
1. Run bulk discovery in background
2. Populate table incrementally
3. Commands work immediately with partial results

## Configuration

```toml
[deleted_tasks]
# Maximum entries in deleted_tasks table (oldest evicted)
max_entries = 10000

# Cache reconstructed content in content_cache
cache_content = true

# Auto-commit on lat close
auto_commit = false

# Link handling on close: "plaintext", "preserve", "delete"
link_handling = "plaintext"
```

## Migration Path

### Phase 1: Add ID to Filenames

1. Release version with filename migration command
2. Users run `lat migrate-filenames` to update existing documents
3. New documents are created with ID in filename
4. Linter warns (not errors) about old-style filenames

### Phase 2: Deprecate .closed/

1. `lat close` moves to `.closed/` AND adds to `deleted_tasks`
2. `lat show` can display from either location
3. `lat list --include-closed` queries both
4. Linter warns about `.closed/` directories

### Phase 3: Remove .closed/ Support

1. `lat close` deletes files directly
2. `.closed/` directories trigger migration prompt
3. `lat migrate-closed` converts `.closed/` tasks to deleted

### Phase 4: Cleanup

1. Remove `.closed/` handling code
2. Remove migration commands
3. Error on `.closed/` directories

## Comparison: Current vs Proposed

| Aspect | Current (.closed/) | Proposed (Deleted) |
|--------|-------------------|-------------------|
| Storage | Files in .closed/ | Git history only |
| Discovery | Filesystem scan | Index + git log |
| Show deleted | Read file | Reconstruct from git |
| Close operation | Move file | Delete file |
| Link updates | Rewrite paths | Convert to plain text |
| Repository size | Grows with closed tasks | Constant (git handles) |
| Offline access | Full | Requires index cache |
| Shallow clone | Full | Limited to boundary |

## Open Questions

1. **Should we preserve closed-at timestamp?** Currently stored in frontmatter
   of closed tasks. Could store in `deleted_tasks` table or commit message.

2. **What about knowledge base documents?** This design focuses on tasks.
   Should KB documents follow the same pattern, or is deletion inappropriate
   for documentation?

3. **Should lat reopen recreate the original filename?** Or use a fresh
   auto-generated name based on current description?

4. **How long to retain deleted_tasks entries?** Forever? Until git gc? Based
   on count/age limits?

5. **Should the ID-in-filename requirement apply to all documents or just
   tasks?** KB documents benefit less from deletion/reconstruction.
