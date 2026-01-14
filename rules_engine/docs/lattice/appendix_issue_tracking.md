# Appendix: Issue Tracking

This appendix documents the complete issue lifecycle state machine and
transition rules. See [Lattice Design](lattice_design.md#issue-tracking) for
an overview, and [Appendix: Beads Analysis](appendix_beads_analysis.md) for
detailed analysis of beads behaviors that Lattice preserves.

## Issue vs Knowledge Base Documents

Any Lattice document can be either:
- **Knowledge base**: Has `lattice-id`, `name`, `description`
- **Issue**: Additionally has `issue-type` and `status` fields

The presence of `issue-type` is the discriminator. Documents without it
are treated as knowledge base entries.

**Note on `description` field:** For issues, the body text serves as the
full description. However, issues may optionally include a `description`
frontmatter field as a one-line summary. This is useful for AI agents that
need brief context without reading the full body.

## Issue Types

| Type | Description | Use Case |
|------|-------------|----------|
| `bug` | Something broken | Defects, regressions |
| `feature` | New functionality | User-facing capabilities |
| `task` | Work item | Tests, docs, refactoring |
| `epic` | Directory root | Grouping/parent issues |
| `chore` | Maintenance | Dependencies, tooling |

The `epic` type is special: it typically corresponds to a directory root
document (files with `00_` prefix) and represents a collection of related issues.

## Creating Issues

### Basic Creation

The `lat create` command creates new issue documents:

```bash
lat create <path/to/issue.md> [options]
```

The path argument specifies both the location and filename for the issue. This
establishes the issue's position in the hierarchy.

### Creating Different Issue Types

**Create a task (default type):**
```bash
lat create issues/auth/fix_login_bug.md -d "Users unable to log in after password reset"
```

**Create a bug with high priority:**
```bash
lat create issues/auth/authentication_crash.md \
  -t bug -p 0 -d "Application crashes on invalid credentials"
```

**Create a feature:**
```bash
lat create issues/auth/oauth_support.md \
  -t feature -p 1 -d "Add OAuth 2.0 authentication support"
```

**Create an epic (directory root):**
```bash
lat create issues/auth/00_authentication_system.md \
  -t epic -p 1 -d "Epic tracking all authentication-related work"
```

Note: For epics, prefix the filename with `00_` to mark it as the highest priority
document in the directory, serving as the parent/root for other issues in that
directory. Underscores in filenames are automatically converted to hyphens in the
YAML `name` field.

### Setting Initial Properties

**Add labels:**
```bash
lat create issues/backend/performance_optimization.md \
  -t task -l performance,backend,database \
  -d "Optimize database query performance"
```

**Add dependencies:**
```bash
lat create issues/api/implement_feature.md \
  -d "Implement new API endpoint" \
  --deps discovered-from:LK1DT
```

**Use a file for description:**
```bash
lat create issues/project/complex_issue.md \
  --body-file issue_description.md
```

### Priority Levels

When creating issues, use these priority values:

- `-p 0` or `--priority 0` (P0): Critical issues requiring immediate attention
- `-p 1` or `--priority 1` (P1): High-priority work for current milestone
- `-p 2` or `--priority 2` (P2): Medium priority (default if not specified)
- `-p 3` or `--priority 3` (P3): Low priority, nice-to-have improvements
- `-p 4` or `--priority 4` (P4): Backlog items for future consideration

### Common Patterns

**Creating a new project area:**
```bash
# 1. Create the directory
mkdir -p issues/new_feature

# 2. Create the epic (root document)
lat create issues/new_feature/00_new_feature_epic.md \
  -t epic -p 1 -d "Epic for new feature development"

# 3. Create child issues
lat create issues/new_feature/implement_backend.md \
  -t task -p 1 -d "Implement backend API endpoints"

lat create issues/new_feature/add_tests.md \
  -t task -p 1 -d "Add comprehensive test coverage"
```

**Creating a bug with context:**
```bash
lat create issues/auth/user_cannot_login.md \
  -t bug -p 0 \
  -d "Users report login failures after password reset" \
  -l security,critical
```

### Path and File Naming

The path argument determines both the directory location and the filename:

```bash
lat create issues/performance/fix_memory_leak.md
# Creates: issues/performance/fix_memory_leak.md
# YAML name field: fix-memory-leak
```

For root documents (epics), include the `00_` prefix in the filename:

```bash
lat create issues/performance/00_performance_epic.md
# Creates: issues/performance/00_performance_epic.md
# YAML name field: 00_performance-epic
```

**Important:** Use underscores in file paths (e.g., `fix_memory_leak.md`). These
are automatically converted to hyphens in the YAML `name` field (e.g.,
`name: fix-memory-leak`), following Lattice naming conventions.

The directory path establishes the issue's position in the hierarchy, while the
`00_` prefix marks documents as highest priority within their directory.

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
[Appendix: Workflow](appendix_workflow.md#lat-claim) for details.

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

See [Appendix: Workflow](appendix_workflow.md#lat-claim) for full claim
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
├── 00_overview.md              # Epic for project (highest priority)
├── auth/
│   ├── 00_authentication.md    # Epic for auth module (highest priority)
│   ├── login_bug.md            # Issue: bug in login
│   └── oauth_feature.md        # Issue: add OAuth
└── api/
    ├── 00_api_design.md        # Epic for API (highest priority)
    └── rate_limiting.md        # Issue: implement limits
```

Note: File paths use underscores (e.g., `login_bug.md`), which are converted to
hyphens in YAML names (e.g., `name: login-bug`).

### Implicit Hierarchy

- All issues in a directory are siblings
- The directory's root document (with `00_` prefix) is their parent/epic
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
parent-id: LPARENT
name: implement-oauth
description: Add OAuth 2.0 support for Google and GitHub authentication
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

The `parent-id` field is auto-populated by `lat fmt` based on the directory's
root document. The optional `description` field provides a one-line summary;
the body text contains the full issue details.

## CLI Differences from Beads

| Beads | Lattice | Notes |
|-------|---------|-------|
| `bd create` | `lat create path/to/file.md` | Path includes filename |
| `--parent` | `--parent <id>` or `--path <prefix>` | ID or path-based filtering |
| `--title` | `-d` / `--description` | Issue description |
| `--description` | `-d` / `--description` | Combined with title |
| `bd show` | `lat show` | Follows bd format |
| `bd sync` | Not applicable | No push operations |
| `--status in_progress` | `lat claim` | Local-only tracking |
| `--assignee` | Not applicable | No assignee concept |
| epic type | root documents | `00_*.md` priority prefix convention |
