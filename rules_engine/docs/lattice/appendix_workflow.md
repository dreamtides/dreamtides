# Appendix: Workflow Commands

This appendix documents the primary workflow commands for viewing documents,
finding ready work, providing AI context, and claiming issues. See
[Lattice Design](lattice_design.md#workflow-commands) for an overview, and
[Appendix: CLI Structure](appendix_cli_structure.md) for the complete command
reference including non-workflow commands.

## lat show

The `lat show` command displays document details, following and extending the
`bd show` text and JSON formats. It supports viewing single or multiple
documents.

### Basic Usage

```
lat show <id> [<id>...]
lat show <id> --json
lat show <id> --short
lat show <id> --refs
```

### Issue Display Format

For issue documents, the output follows `bd show` format:

```
L1234: Fix LLMC v2 code review issues
Status: open
Priority: P0
Type: epic
Created: 2026-01-10 14:37
Created by: dthurn
Updated: 2026-01-10 14:37

Description:
Master epic for addressing bugs and missing features identified in
comprehensive code review of LLMC v2 implementation.

Parent:
  L0042: LLMC Development [epic]

Depends on (1):
  L2345: Fix incorrect rebase necessity detection in patrol [P0 - closed]

Blocks (5):
  L3456: Fix crash count not being incremented in patrol [P0 - open]
  L4567: Fix stuck worker nudging to be async [P1 - open]
  ...

Related (1):
  L5678: llmc-design - LLMC design document [doc]
```

**Key sections:**

- **Header line:** ID and name (issue title)
- **Metadata block:** Status, priority, type, timestamps, creator
- **Description:** Full markdown body content
- **Parent:** Directory root document (epic context)
- **Depends on:** Issues in the `blocked-by` field (what this depends on)
- **Blocks:** Issues in the `blocking` field (what depends on this)
- **Related:** Other documents linked in the body text

### Knowledge Base Display Format

For knowledge base documents (no `issue-type`), the output emphasizes the
name and description fields which provide structured metadata:

```
L9876: authentication-design
Description: OAuth 2.0 implementation design for the auth subsystem

---
[Full markdown body content here]
---

Related (2):
  L1111: security-policy - Security guidelines and threat model [doc]
  L2222: api-design - REST API design principles [doc]
```

**Header components:**

- **ID:** The Lattice ID
- **Name:** The `name` frontmatter field (lowercase-hyphenated identifier)
- **Description:** Displayed on its own line, providing the purpose summary

Knowledge base documents do not display timestamps in the default view since
they typically lack explicit `created-at`/`updated-at` fields. Git history
can be queried separately if modification dates are needed.

### Document Reference Format

All document references throughout `lat show` output use a consistent format:

```
<id>: <name> - <description> [<type-indicator>]
```

Where `<type-indicator>` is:
- For issues: `P<N> - <status>` (e.g., `P0 - open`, `P1 - closed`)
- For knowledge base: `doc`

For issues, the name IS the title (a short description), so no separate description
is shown. For knowledge base entries, both name and description are displayed.

Examples:
```
L1234: Fix login bug [P0 - open]
L5678: authentication-design - OAuth 2.0 implementation design [doc]
L9012: LLMC Development [epic]
```

### Related Document Selection

The "Related" section displays documents linked from the body text. A document
is considered "related" if:

1. **Explicit link:** The body contains a markdown link with the document's
   Lattice ID (either as `[text](path#ID)` or `[text](ID)`)
2. **Not a dependency:** The document is not already listed in Depends on,
   Blocks, or Parent sections

The list is ordered by first appearance in the body text. Maximum 10 related
documents are shown in text output; use `--json` for complete list.

### Name vs Description Distinction

Lattice maintains separate semantics for issues and knowledge base entries:

**Issues:**
- `name`: The issue title (short, descriptive)
- Body text: The full issue description (replaces beads' description field)
- No separate `description` frontmatter field

**Knowledge Base Entries:**
- `name`: Short identifier (lowercase-hyphenated, max 64 chars)
- `description`: Purpose summary for AI context (max 1024 chars)
- Body text: Full document content

This design ensures:
1. Issues match `bd show` behavior with a single description block
2. Knowledge base entries retain structured metadata for AI context

### Short Format

The `--short` flag produces single-line output:

```
$ lat show L1234 --short
L1234 [open] P0 epic: Fix LLMC v2 code review issues
```

Format: `<id> [<status>] <priority> <type>: <name>`

For knowledge base documents:
```
$ lat show L9876 --short
L9876 [doc]: authentication-design - OAuth 2.0 implementation design
```

### References Format

The `--refs` flag shows issues that reference this one (reverse lookup):

```
$ lat show L1234 --refs
References to L1234:

  Blocks (5):
    L3456: Fix crash count not being incremented in patrol [P0 - open]
    L4567: Fix stuck worker nudging to be async [P1 - open]
    ...

  Linked from (2):
    L7890: sprint-3-planning - Sprint 3 planning document [doc] (line 42)
    L8901: code-review-checklist - Code review checklist [doc] (line 15)
```

### JSON Output Format

The `--json` flag produces structured output compatible with `bd show --json`:

```json
[
  {
    "id": "L1234",
    "title": "Fix LLMC v2 code review issues",
    "description": "Master epic for addressing bugs...",
    "status": "open",
    "priority": 0,
    "issue_type": "epic",
    "created_at": "2026-01-10T14:37:59.351489-08:00",
    "created_by": "dthurn",
    "updated_at": "2026-01-10T14:37:59.351489-08:00",
    "path": "issues/llmc/fix-review-issues.md",
    "labels": ["llmc", "code-review"],
    "dependencies": [
      {
        "id": "L2345",
        "title": "Fix incorrect rebase necessity detection in patrol",
        "status": "closed",
        "priority": 0,
        "issue_type": "bug"
      }
    ],
    "dependents": [
      {
        "id": "L3456",
        "title": "Fix crash count not being incremented in patrol",
        "status": "open",
        "priority": 0,
        "issue_type": "bug"
      }
    ],
    "related": [
      {
        "id": "L5678",
        "name": "llmc-design",
        "description": "LLMC design document"
      }
    ],
    "parent": {
      "id": "L0042",
      "title": "LLMC Development"
    },
    "claimed": false
  }
]
```

**JSON keys for backwards compatibility with `bd show --json`:**

| Key | Type | Description |
|-----|------|-------------|
| `id` | string | Lattice ID |
| `title` | string | Issue name/title (maps from `name` field) |
| `description` | string | Full body text for issues, description field for KB |
| `status` | string | Issue status |
| `priority` | int | Priority level (0-4) |
| `issue_type` | string | bug/feature/task/epic/chore |
| `created_at` | string | ISO 8601 timestamp |
| `created_by` | string | Creator identifier |
| `updated_at` | string | ISO 8601 timestamp |
| `closed_at` | string | ISO 8601 timestamp (if closed) |
| `path` | string | Relative file path |
| `labels` | array | List of labels |
| `dependencies` | array | Issues this depends on (blocked-by) |
| `dependents` | array | Issues that depend on this (blocking) |
| `related` | array | Documents linked from body text |
| `parent` | object | Directory root document (epic) |
| `claimed` | bool | Whether locally claimed |

For knowledge base documents, additional keys:

| Key | Type | Description |
|-----|------|-------------|
| `name` | string | Document name (lowercase-hyphenated) |
| `body` | string | Full markdown body content |

### Multiple Documents

When showing multiple documents, output is separated by blank lines (text) or
returned as a JSON array:

```
$ lat show L1234 L5678
L1234: First issue title
...

L5678: Second issue title
...
```

## lat ready

The `lat ready` command shows work available to start: issues that are open,
have no blockers, and are not claimed.

### Basic Usage

```
lat ready [options]
lat ready --parent <id>
lat ready --json
lat ready --pretty
```

### Default Output

```
$ lat ready
Ready work (4 issues with no blockers):

1. [P0] [epic] L1234: Fix LLMC v2 code review issues
2. [P1] [task] L5678: Convert strings.toml to Fluent format
3. [P1] [epic] L9012: LLMC v2: Agent Coordination System
4. [P1] [epic] L3456: Tabula V2: Complete Card Data Loading Rewrite
```

### Ready Criteria

An issue is "ready" if:
1. Status is `open` or `pinned`
2. All `blocked-by` issues are closed
3. Priority is not P4 (backlog) unless `--include-backlog`
4. Not currently claimed (unless `--include-claimed`)

### Filter Options

| Flag | Description |
|------|-------------|
| `--parent <id>` | Filter to descendants of this directory/epic |
| `--priority <N>` | Filter by priority level |
| `--type <type>` | Filter by issue type |
| `--label <list>` | Filter by labels (AND logic) |
| `--label-any <list>` | Filter by labels (OR logic) |
| `--limit <N>` | Maximum issues (default 10) |
| `--include-backlog` | Include P4 issues |
| `--include-claimed` | Include claimed issues |

### Sort Policies

| Policy | Behavior |
|--------|----------|
| `hybrid` | Default. Priority first, then creation date |
| `priority` | Strict priority ordering |
| `oldest` | Creation date ascending |

### Pretty Format

The `--pretty` flag displays a visual tree with status symbols:

```
$ lat ready --pretty
o P0 L1234 - [EPIC] Fix LLMC v2 code review issues

o P1 L3456 - [EPIC] Tabula V2: Complete Card Data Loading Rewrite
|-- o P1 L5678 - Convert strings.toml to Fluent format

o P1 L9012 - [EPIC] LLMC v2: Agent Coordination System

--------------------------------------------------------------------------------
Total: 4 issues (4 open, 0 claimed)

Legend: o open | x claimed | (blocked) | P0 P1 P2 P3 P4
```

### JSON Output Format

The `--json` flag produces output compatible with `bd ready --json`:

```json
[
  {
    "id": "L1234",
    "title": "Fix LLMC v2 code review issues",
    "description": "Master epic for addressing bugs...",
    "status": "open",
    "priority": 0,
    "issue_type": "epic",
    "created_at": "2026-01-10T14:37:59.351489-08:00",
    "created_by": "dthurn",
    "updated_at": "2026-01-10T14:37:59.351489-08:00",
    "path": "issues/llmc/fix-review-issues.md",
    "labels": [],
    "parent": {
      "id": "L0042",
      "title": "LLMC Development"
    }
  }
]
```

The JSON output includes the full description text for each ready issue,
enabling AI agents to understand task context without additional queries.

## lat prime

The `lat prime` command outputs AI-optimized workflow context, following
`bd prime` behavior. Lattice never runs `git push` or any equivalent sync
operation, so the session protocol focuses on local validation and commits.

### Basic Usage

```
lat prime
lat prime --full
```

### Default Output

```
$ lat prime
# Lattice Workflow Context

> **Context Recovery**: Run `lat prime` after compaction or new session

## Session Protocol

Before completing work, run this checklist:

[ ] 1. lat check           (validate documents)
[ ] 2. git status          (review changes)
[ ] 3. git add <files>     (stage changes)
[ ] 4. git commit -m "..." (commit work)

## Core Commands

- `lat ready` - Show issues ready to work
- `lat show <id>` - View issue details
- `lat claim <id>` - Claim issue for local work
- `lat close <id>` - Mark issue completed
```

### Custom Checklist

The `.lattice/config.toml` file can specify custom validation commands:

```toml
[prime]
checklist = [
    "lat check",
    "just review",
    "git status",
    "git add .",
    "git commit"
]
```

These commands appear in the session protocol output.

## lat claim

The `lat claim` command marks an issue as locally in progress on the current
machine. This state is NOT stored in markdown files or tracked in git;
instead it persists to `~/.lattice/claims.json`.

### Basic Usage

```
lat claim <id>           # Claim an issue
lat claim --list         # Show all claims
lat claim --release <id> # Release a claim
lat claim --release-all  # Release all claims
lat claim --gc           # Clean up stale claims
```

### Claim Storage

Claims are stored in `~/.lattice/claims.json`, keyed by repository root
to support multiple Lattice projects on the same machine:

```json
{
  "/path/to/repo1": {
    "L1234": {
      "claimed_at": "2026-01-14T10:30:00Z",
      "work_path": "/path/to/repo1",
      "hostname": "dev-machine"
    }
  },
  "/path/to/repo2": {
    "L5678": {
      "claimed_at": "2026-01-14T11:00:00Z",
      "work_path": "/path/to/repo2/worktree-feature",
      "hostname": "dev-machine"
    }
  }
}
```

The `work_path` field stores either:
- The main repository path when claiming from the main checkout
- The worktree path when claiming from a git worktree

This file is:
- Shared across multiple git worktrees within the same repository
- Isolated between different Lattice repositories
- NOT tracked in version control
- Updated atomically via file locking

### Atomic Updates

Multiple agents in different worktrees might claim different issues
simultaneously. Lattice uses file locking to ensure atomic updates:

1. Acquire exclusive lock on `~/.lattice/claims.lock`
2. Read current claims
3. Modify claims for the relevant repository
4. Write updated claims
5. Release lock

If the lock cannot be acquired within 5 seconds, the operation fails
with a clear error message.

### Claim Lifecycle

```
lat ready          # Issue shows as "ready"
lat claim L1234    # Issue now claimed
lat show L1234     # Shows "Claimed: true" in output
lat ready          # Issue no longer appears (unless --include-claimed)
lat close L1234    # Closing auto-releases the claim
```

### Auto-Release on State Change

When an issue's status changes (closed, blocked, etc.), its claim is
automatically released. This happens during:

- `lat close <id>`
- `lat update <id> --status blocked`
- Any operation that modifies issue status

### Stale Claim Detection

The `lat claim --gc` command removes stale claims:

```
$ lat claim --gc
Checking 3 claims...
Released: L1234 (issue closed)
Released: L5678 (work path no longer exists)
Kept: L9012 (active)
```

A claim is stale if:
- The issue is no longer open
- The work path no longer exists
- The claim is older than 7 days (configurable)

### Crash Recovery

If an agent crashes while working on an issue, the claim persists. The
coordinator can release it via:

```
lat claim --release L1234
```

Or clear all claims from a specific work path:

```
lat claim --release-path /path/to/crashed/worktree
```

### Display in lat show

The `lat show` command indicates claim status:

```
L1234: Fix LLMC v2 code review issues
Status: open
Priority: P0
Type: epic
Claimed: true
...
```

### Display in lat ready

By default, `lat ready` excludes claimed issues:

```
$ lat ready
Ready work (3 issues with no blockers):
...

$ lat ready --include-claimed
Ready work (4 issues, 1 claimed):
1. [P0] [epic] L1234: Fix LLMC v2... [CLAIMED]
...
```

### No Assignee Concept

Lattice intentionally has no "assignee" field. The claim system tracks
which machine is working on an issue, not who. This design:

1. Supports multi-agent workflows where work is coordinated externally
2. Avoids stale assignee data when agents are replaced
3. Keeps issue files clean of operational state

## Command Summary

| Command | Purpose | Modifies Files |
|---------|---------|----------------|
| `lat show` | View document details | No |
| `lat ready` | Find available work | No |
| `lat prime` | AI workflow context | No |
| `lat claim` | Track local work | No (uses ~/.lattice/) |

All four commands are read-only with respect to repository files. The
claim command modifies only the user's local `~/.lattice/` directory.
