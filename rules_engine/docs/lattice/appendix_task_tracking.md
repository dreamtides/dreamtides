# Appendix: Task Tracking

This appendix documents task lifecycle and creation. See
[Lattice Design](lattice_design.md#task-tracking) for an overview.

## Task vs Knowledge Base

All Lattice documents require `lattice-id`, `name`, and `description`. The
document type determines additional fields:

- **Knowledge base**: `lattice-id`, `name`, `description` (required)
- **Task**: Above plus `task-type`, `priority` (required)

The presence of `task-type` is the discriminator. Use `lat create` with `-t` to
create tasks, or without `-t` to create knowledge base documents.

## Task Types

| Type | Use Case |
|------|----------|
| `bug` | Defects, regressions |
| `feature` | User-facing capabilities |
| `task` | Tests, docs, refactoring |
| `chore` | Dependencies, tooling |

## Task State

Task state is determined by filesystem location, not by a YAML field:

| State | Filesystem Location | In `lat ready` | Description |
|-------|---------------------|----------------|-------------|
| Open | Not in `.closed/` | Yes (unless blocked/claimed) | Available for work |
| Blocked | Not in `.closed/`, has open `blocked-by` | No | Waiting on dependencies |
| Closed | In `.closed/` subdirectory | No | Completed |

A task is **blocked** when any entry in its `blocked-by` field references a
task that is not closed. Once all blockers are closed, the task becomes open.

### The `.closed/` Directory

Closed tasks reside in a `.closed/` subdirectory under the `tasks/` directory:

```
auth/
├── auth.md               # Root document (parent for this directory)
├── docs/
│   └── auth_design.md    # Knowledge base document
└── tasks/
    ├── new_feature.md    # Open task
    └── .closed/
        ├── fix_login.md  # Closed task
        └── oauth_bug.md  # Closed task
```

The `.closed/` directory is tracked in git, making closed tasks visible to all
collaborators. Use `lat prune` to permanently remove closed tasks.

### State Transitions

```
         lat close
  open ───────────────► closed
    ▲                      │
    │ (blockers closed)    │ lat reopen
    │                      ▼
blocked ◄─────────────── open
         (add blocked-by)
```

There is no `in_progress` status. Use `lat claim` for local work tracking.

## Timestamps

| Field | Auto-set by |
|-------|-------------|
| `created-at` | `lat create` (current time) |
| `updated-at` | `lat update`, `lat fmt` when content changes |
| `closed-at` | `lat close` (current time) |

All timestamps use ISO 8601 format. If missing, `lat show --json` omits them
rather than deriving from git history.

## Priority

| Priority | Description |
|----------|-------------|
| P0 | Critical: security, data loss, broken builds |
| P1 | High: major features, important bugs |
| P2 | Medium: default |
| P3 | Low: polish, optimization |
| P4 | Backlog: future ideas |

## Creating Documents

The `lat create` command uses convention-based placement and auto-generated
filenames to minimize friction:

```bash
lat create <parent> "<description>" [options]
```

**Auto-placement:** The `-t` flag determines the subdirectory:
- With `-t`: document is created in `<parent>/tasks/`
- Without `-t`: document is created in `<parent>/docs/`

**Auto-naming:** The filename is generated from the description:
- Extract significant words (skip articles like "the", "a", "an")
- Convert to lowercase with underscores
- Cap at ~40 characters
- Append numeric suffix on collision (`fix_bug.md`, `fix_bug_2.md`)

The `name` field is derived from the generated filename (underscores become
hyphens). The filename is immutable after creation even if the description
is later edited.

Examples:

```bash
# Task - auto-placed in auth/tasks/, filename from description
lat create auth/ "Fix login after password reset" -t bug
# Creates: auth/tasks/fix_login_after_password_reset.md
# name: fix-login-after-password-reset

# Knowledge base - auto-placed in auth/docs/
lat create auth/ "OAuth 2.0 implementation design"
# Creates: auth/docs/oauth_implementation_design.md
# name: oauth-implementation-design

# Task with priority
lat create auth/ "Add OAuth 2.0 support" -t feature -p 1
# Creates: auth/tasks/add_oauth_support.md

# Root document - explicit path required (filename must match directory)
lat create auth/auth.md "Authentication system"
# Creates: auth/auth.md (root document for auth/ hierarchy)
```

**Explicit paths:** You can still specify a full path when you want control
over the filename:

```bash
lat create auth/tasks/oauth_bug.md "Fix OAuth token validation error" -t bug
# Creates: auth/tasks/oauth_bug.md (explicit short name)
```

Options:
- `-t <type>`: Task type (bug/feature/task/chore). Omit for KB documents.
- `-p <0-4>`: Priority (default: 2, tasks only)
- `-l <labels>`: Comma-separated labels
- `--deps discovered-from:<id>`: Add dependency
- `--body-file <path>`: Read extended body from file

## Filesystem Hierarchy

```
project/
├── project.md               # Project root document
├── auth/
│   ├── auth.md              # Auth root document (parent for auth/)
│   ├── docs/
│   │   └── auth_design.md   # Knowledge base documents
│   └── tasks/
│       ├── login_bug.md     # Task documents
│       ├── oauth_feature.md
│       └── .closed/
│           └── old_task.md  # Closed tasks
└── api/
    ├── api.md               # API root document (parent for api/)
    ├── docs/
    │   └── api_spec.md
    └── tasks/
        └── rate_limiting.md
```

- Root documents have filenames matching their directory name
- Tasks live in `tasks/` subdirectories, documents live in `docs/` subdirectories
- The root document is the parent for all documents in that directory tree
- Root documents are typically NOT tasks (no `task-type`), but tasks can have
  other tasks as parents if desired
- Nesting creates multi-level hierarchy

## Dependencies

```yaml
blocking: [LYYYYY, LZZZZZ]   # This task blocks these
blocked-by: [LWWWWW]        # These block this task
discovered-from: [LXXXXX]   # Soft link for provenance
```

### Dependency Types

| Type | Semantic | Affects `lat ready`? |
|------|----------|----------------------|
| `blocking` | This task must close before targets can start | Yes |
| `blocked-by` | This task cannot start until targets close | Yes |
| `discovered-from` | This task was found while working on target | No |

### discovered-from

Tracks provenance when work is discovered during another task. This is a soft link
that does not affect the ready queue.

```bash
# Create task with discovered-from link (auto-placed in auth/tasks/)
lat create auth/ "Fix token validation" -t bug -p 1 --deps discovered-from:LXXXXX

# Query tasks discovered from a parent
lat list --discovered-from LXXXXX

# View in lat show output
lat show LYYYYY --json
# Output includes: "discovered-from": ["LXXXXX"]
```

A task is "ready" if: not in `.closed/`, all `blocked-by` tasks are closed,
priority is not P4, and not claimed.

## Labels

```yaml
labels: [security, frontend, needs-review]
```

Query with `--label` (AND) or `--label-any` (OR).

## Work Tracking

Use `lat claim` for local work tracking:

```bash
lat claim LB234X    # Mark as being worked on
lat ready          # Task no longer appears
lat close LB234X    # Auto-releases claim
```

Claims stored in `~/.lattice/claims/`, not in git. See
[Appendix: Workflow](appendix_workflow.md#lat-claim).

## Templates

Tasks automatically inherit context and acceptance criteria from ancestor root
documents (documents whose filename matches their directory name). No explicit
template references are needed—the filesystem hierarchy IS the template structure.

Root documents can include `[Lattice] Context` and `[Lattice] Acceptance
Criteria` headings (any heading level):
- Context sections prepend to descendant task bodies (general → specific)
- Acceptance sections append to descendant tasks (specific → general)

Changes to root documents propagate instantly to all descendant tasks.

See [Appendix: Task Templates](appendix_task_templates.md) for composition
rules and common patterns.

## Document Structure

Example task document (`auth/tasks/fix_login.md`):

```yaml
---
lattice-id: LXXXXX
name: fix-login
description: Fix login after password reset
task-type: task
priority: 2
labels: [auth]
blocked-by: [LZZZZZ]
created-at: 2024-01-15T10:30:00Z
---

Users receive 401 errors when logging in after using the password reset flow.
This appears to be a session invalidation issue.

## Reproduction Steps

1. Request password reset
2. Complete reset flow
3. Attempt to log in with new password
4. Observe 401 error
```

When this task is closed via `lat close LXXXXX`, it moves to
`auth/tasks/.closed/fix_login.md` and all links are updated automatically.

The `name` field is always derived from the filename (underscores → hyphens,
lowercase). This is a core Lattice invariant—the linter will warn if `name`
doesn't match the filename.

The `description` field is the human-readable task title shown in `lat show`
output and list views. Both `name` and `description` are required.

The markdown body provides extended details: reproduction steps, implementation
notes, design context, etc.

The `parent-id` field is auto-populated by `lat fmt` from the directory's
root document. Template content (context and acceptance criteria) is inherited
automatically from ancestor root documents based on filesystem location.
