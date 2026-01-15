# Appendix: Task Tracking

This appendix documents task lifecycle and creation. See
[Lattice Design](lattice_design.md#task-tracking) for an overview.

## Task vs Knowledge Base

Any Lattice document can be either:
- **Knowledge base**: Has `lattice-id`, `name`, `description`
- **Task**: Additionally has `task-type` and `status`

The presence of `task-type` is the discriminator.

## Task Types

| Type | Use Case |
|------|----------|
| `bug` | Defects, regressions |
| `feature` | User-facing capabilities |
| `task` | Tests, docs, refactoring |
| `epic` | Directory root / grouping |
| `chore` | Dependencies, tooling |

## Status

| Status | In `lat ready` |
|--------|----------------|
| `open` | Yes (unless claimed) |
| `blocked` | No |
| `deferred` | No |
| `closed` | No |
| `pinned` | Yes (always) |

There is no `in_progress` status. Use `lat claim` for local work tracking.

## Priority

| Priority | Description |
|----------|-------------|
| P0 | Critical: security, data loss, broken builds |
| P1 | High: major features, important bugs |
| P2 | Medium: default |
| P3 | Low: polish, optimization |
| P4 | Backlog: future ideas |

## Creating Tasks

```bash
lat create tasks/auth/fix_login_bug.md -d "Users unable to log in after reset"
lat create tasks/auth/crash.md -t bug -p 0 -d "Crash on invalid credentials"
lat create tasks/auth/oauth.md -t feature -p 1 -d "Add OAuth 2.0 support"
lat create tasks/auth/00_auth_epic.md -t epic -d "Authentication system"
```

Options:
- `-t <type>`: Task type (default: task)
- `-p <0-4>`: Priority (default: 2)
- `-d <text>`: Description
- `-l <labels>`: Comma-separated labels
- `--deps discovered-from:<id>`: Add dependency
- `--body-file <path>`: Read body from file

File paths use underscores (e.g., `fix_login_bug.md`), converted to hyphens
in YAML names (`name: fix-login-bug`).

## Filesystem Hierarchy

```
tasks/
├── 00_overview.md           # Project epic
├── auth/
│   ├── 00_authentication.md # Auth epic
│   ├── login_bug.md
│   └── oauth_feature.md
└── api/
    ├── 00_api_design.md     # API epic
    └── rate_limiting.md
```

- All tasks in a directory are siblings
- The `00_` prefixed file is their parent/epic
- Nesting creates multi-level hierarchy

## Dependencies

```yaml
blocking: [LYYYY, LZZZZ]   # This task blocks these
blocked-by: [LWWWW]        # These block this task
discovered-from: [LXXXX]   # Soft link for provenance
```

A task is "ready" if: status is `open`/`pinned`, no open `blocked-by`
tasks, priority is not P4, and not claimed.

## Labels

```yaml
labels: [security, frontend, needs-review]
```

Query with `--label` (AND) or `--label-any` (OR).

## Work Tracking

Use `lat claim` instead of status changes:

```bash
lat claim LB234    # Mark as being worked on
lat ready          # Task no longer appears
lat close LB234    # Auto-releases claim
```

Claims stored in `~/.lattice/claims/`, not in git. See
[Appendix: Workflow](appendix_workflow.md#lat-claim).

## Templates

Tasks automatically inherit context and acceptance criteria from ancestor
directory root documents (`00_*.md` files). No explicit template references
are needed—the filesystem hierarchy IS the template structure.

Root documents can include `[Lattice] Context` and `[Lattice] Acceptance
Criteria` headings (any heading level):
- Context sections prepend to descendant task bodies (general → specific)
- Acceptance sections append to descendant tasks (specific → general)

Changes to root documents propagate instantly to all descendant tasks.

See [Appendix: Task Templates](appendix_task_templates.md) for composition
rules and common patterns.

## Document Structure

```yaml
---
lattice-id: LXXXX
task-type: task
status: open
priority: 2
labels: [auth]
blocked-by: [LZZZZ]
created-at: 2024-01-15T10:30:00Z
---

Description of the task.
```

The `parent-id` field is auto-populated by `lat fmt` from the directory's
root document. Template content (context and acceptance criteria) is inherited
automatically from ancestor root documents based on filesystem location.
