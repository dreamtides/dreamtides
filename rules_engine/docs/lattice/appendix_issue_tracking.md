# Appendix: Issue Tracking

## Issue vs Knowledge Base Documents

Any Lattice document can be either:
- **Knowledge base**: Has `lattice-id`, `name`, `description`
- **Issue**: Additionally has `issue-type` and `status` fields

The presence of `issue-type` is the discriminator. Documents without it
are treated as knowledge base entries.

## Issue Types

| Type | Description | Use Case |
|------|-------------|----------|
| `bug` | Something broken | Defects, regressions |
| `feature` | New functionality | User-facing capabilities |
| `task` | Work item | Tests, docs, refactoring |
| `epic` | Directory root | Grouping/parent issues |
| `chore` | Maintenance | Dependencies, tooling |

The `epic` type is special: it typically corresponds to a directory root
document (`!*.md`) and represents a collection of related issues.

## Status State Machine

### States

| Status | Description | Appears in `lat ready` |
|--------|-------------|------------------------|
| `open` | Ready for work | Yes (unless claimed) |
| `blocked` | Waiting on dependencies | No |
| `deferred` | Intentionally delayed | No |
| `closed` | Work completed | No |
| `tombstone` | Permanently removed | No |
| `pinned` | Always open | Yes (unless claimed) |

**Note**: There is no `in_progress` status. Instead, use `lat claim` to
track work locally without modifying issue files. See
[Appendix: Commands](appendix_commands.md#lat-claim) for details.

### Transitions

```
              ┌──────────────┐
              │              │
              ▼              │
  ┌────────► open ◄─────────┤
  │           │              │
  │           ▼              │
  │        blocked ─────────►│
  │           │              │
  │           ▼              │
  │       deferred ──────────┤
  │                          │
  │                          │
  └─────────── closed ◄──────┘
```

**Valid transitions:**
- `open` → `blocked`, `deferred`, `closed`
- `blocked` → `open`, `deferred` (auto-transitions to open when unblocked)
- `deferred` → `open`, `closed`
- `closed` → `open` (via `lat reopen`)
- `pinned` → `closed` (manual only)
- `tombstone` → (no transitions, terminal state)

## Work Tracking with lat claim

Instead of an `in_progress` status, Lattice uses local claiming:

```bash
lat claim L1234    # Mark issue as being worked on
lat ready          # Issue no longer appears (claimed)
lat show L1234     # Shows "Claimed: yes"
lat close L1234    # Auto-releases the claim
```

Claims are stored in `~/.lattice/claims.json`, not in the issue file:
- Not tracked in git
- Shared across worktrees on same machine
- Atomically updated with file locking

This design supports multi-agent workflows where:
- Multiple agents work in different worktrees
- A coordinator assigns issues without modifying files
- Work state doesn't create git conflicts

See [Appendix: Commands](appendix_commands.md#lat-claim) for full claim
command documentation.

## Priority Levels

| Priority | Label | Description |
|----------|-------|-------------|
| 0 | P0 | Critical: security, data loss, broken builds |
| 1 | P1 | High: major features, important bugs |
| 2 | P2 | Medium: nice-to-have, minor bugs |
| 3 | P3 | Low: polish, optimization |
| 4 | P4 | Backlog: future ideas |

Priority affects `lat ready` sorting: P0 issues appear first.

## Filesystem Hierarchy

### Directory Organization

Issues are organized by filesystem location:

```
project/
├── !overview.md              # Epic for project
├── auth/
│   ├── !authentication.md    # Epic for auth module
│   ├── login_bug.md          # Issue: bug in login
│   └── oauth_feature.md      # Issue: add OAuth
└── api/
    ├── !api_design.md        # Epic for API
    └── rate_limiting.md      # Issue: implement limits
```

### Implicit Hierarchy

- All issues in a directory are siblings
- The directory's root document (`!*.md`) is their parent/epic
- Nesting creates multi-level hierarchy automatically

This replaces beads' explicit `parent` field.

## Dependency System

### Blocking Relationships

Issues track hard dependencies:

```yaml
blocking: [LYYYY, LZZZZ]   # This issue blocks these
blocked-by: [LWWWW]        # These block this issue
```

These create edges in the dependency graph.

### Ready Calculation

An issue is "ready" if:
1. Status is `open` or `pinned`
2. No `blocked-by` issues are non-closed
3. Priority is not P4 (backlog)
4. Not currently claimed (unless `--include-claimed`)

The `lat ready` command returns issues meeting these criteria.

### Discovered-From

The `discovered-from` field tracks issue provenance:

```yaml
discovered-from: [LXXXX]  # Discovered while working on LXXXX
```

This is a soft relationship (not blocking) for traceability.

## Labels

### Usage

Labels provide cross-cutting categorization:

```yaml
labels: [security, frontend, needs-review]
```

### Queries

Labels support AND and OR queries:

```bash
lat list --label security,urgent      # Has ALL (AND)
lat list --label-any frontend,backend # Has ANY (OR)
```

### Built-In Labels

No labels are reserved. Common conventions:
- `bug`, `feature`, `enhancement`
- `security`, `performance`
- `needs-review`, `needs-triage`
- Component names: `auth`, `api`, `ui`

## Timestamps

### Automatic Fields

```yaml
created-at: 2024-01-15T10:30:00Z
updated-at: 2024-01-16T14:22:00Z
closed-at: 2024-01-17T09:00:00Z
```

- `created-at`: Set on creation, never modified
- `updated-at`: Set on any modification
- `closed-at`: Set when status becomes `closed`

### Staleness

The `lat stale` command uses `updated-at`:

```bash
lat stale --days 30  # Issues not updated in 30 days
```

## Issue Document Structure

### Minimal Issue

```yaml
---
lattice-id: LXXXX
issue-type: task
status: open
priority: 2
---

Brief description of the task.
```

### Full Issue

```yaml
---
lattice-id: LXXXX
name: implement-oauth
issue-type: feature
status: open
priority: 1
labels: [auth, security]
blocking: [LYYYY]
blocked-by: [LZZZZ]
discovered-from: [LWWWW]
created-at: 2024-01-15T10:30:00Z
updated-at: 2024-01-16T14:22:00Z
---

## Description

Add OAuth 2.0 authentication support for the application.

## Acceptance Criteria

- Support Google OAuth
- Support GitHub OAuth
- Token refresh handling

## Notes

Initial implementation complete, pending review.
```

Note that for issues, the body text serves as the description (matching
beads behavior). There is no separate `description` frontmatter field
for issues.

## CLI Differences from Beads

| Beads | Lattice | Notes |
|-------|---------|-------|
| `bd create` | `lat create --path` | Path is required |
| `--parent` | Directory location | Implicit hierarchy |
| `--title` | `--name` / document name | Mapped to name field |
| `bd show` | `lat show` | Follows bd format |
| `bd sync` | Not applicable | No push operations |
| `--status in_progress` | `lat claim` | Local-only tracking |
| `--assignee` | Not applicable | No assignee concept |
| epic type | root documents | `!*.md` convention |
