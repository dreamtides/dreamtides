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
| `epic` | Directory root / grouping |
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

Closed tasks reside in a `.closed/` subdirectory under their original parent:

```
tasks/auth/
├── .closed/
│   ├── fix_login.md      # Closed task
│   └── oauth_bug.md      # Closed task
├── README.md             # Open epic
└── new_feature.md        # Open task
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

All documents (tasks and knowledge base) require a description as a positional
argument:

```bash
lat create <path> "<description>" [options]
```

The `name` field is derived automatically from the filename (underscores become
hyphens). The description is required for all document types.

To create a **task**, include the `-t` flag. Omitting `-t` creates a **knowledge
base document**.

Examples:

```bash
# Knowledge base documents (no -t flag)
lat create docs/auth/oauth_design.md "OAuth 2.0 implementation design"
# Creates: name: oauth-design, description: OAuth 2.0 implementation design

# Tasks (with -t flag)
lat create tasks/auth/fix_login.md "Fix login after password reset" -t bug
# Creates: name: fix-login, description: Fix login after password reset

lat create tasks/auth/oauth_support.md "Add OAuth 2.0 support" -t feature -p 1
# Creates: name: oauth-support, description: Add OAuth 2.0 support

lat create tasks/auth/README.md "Authentication system epic" -t epic
# Creates: name: readme, description: Authentication system epic
```

Options:
- `-t <type>`: Task type (bug/feature/task/epic/chore). Omit for KB documents.
- `-p <0-4>`: Priority (default: 2, tasks only)
- `-l <labels>`: Comma-separated labels
- `--deps discovered-from:<id>`: Add dependency
- `--body-file <path>`: Read extended body from file

## Filesystem Hierarchy

```
tasks/
├── README.md                # Project epic
├── auth/
│   ├── README.md            # Auth epic
│   ├── login_bug.md
│   └── oauth_feature.md
└── api/
    ├── 00_api_design.md     # API epic
    └── rate_limiting.md
```

- All tasks in a directory are siblings
- The root document (`README.md` or `00_*` prefixed file) is their parent/epic
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
# Create task with discovered-from link
lat create tasks/auth/fix_token_bug.md "Fix token validation" -t bug -p 1 \
  --deps discovered-from:LXXXXX

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

Tasks automatically inherit context and acceptance criteria from ancestor
directory root documents (`README.md` or `00_*.md` files). No explicit template
references are needed—the filesystem hierarchy IS the template structure.

Root documents can include `[Lattice] Context` and `[Lattice] Acceptance
Criteria` headings (any heading level):
- Context sections prepend to descendant task bodies (general → specific)
- Acceptance sections append to descendant tasks (specific → general)

Changes to root documents propagate instantly to all descendant tasks.

See [Appendix: Task Templates](appendix_task_templates.md) for composition
rules and common patterns.

## Document Structure

Example task document (`tasks/auth/fix_login.md`):

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
`tasks/auth/.closed/fix_login.md` and all links are updated automatically.

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
