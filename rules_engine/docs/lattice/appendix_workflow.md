# Appendix: Workflow Commands

This appendix documents the primary workflow commands for viewing documents,
finding ready work, providing AI context, and claiming tasks. See
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
lat show <id> --peek
lat show <id> --raw
```

### Task Display Format

For task documents, the output follows `bd show` format:

```
LB234X: fix-review-tasks - Fix LLMC v2 code review tasks
State: open
Priority: P0
Type: bug
Created: 2026-01-10 14:37
Updated: 2026-01-10 14:37

Context:
  [Context from ancestor root documents]

Body:
Master task for addressing bugs and missing features identified in
comprehensive code review of LLMC v2 implementation.

Acceptance Criteria:
  [Acceptance from ancestor root documents]

Parent:
  LAA42X: llmc-development - LLMC Development [doc]

Depends on (1):
  L2345X: rebase-detection - Fix incorrect rebase necessity detection [P0/closed]

Blocks (5):
  L3456X: crash-count - Fix crash count not being incremented [P0]
  L4567X: worker-nudging - Fix stuck worker nudging to be async [P1]
  ...

Related (1):
  L567CX: llmc-design - LLMC design document [doc]
```

**Key sections:**

- **Header line:** ID, name (from filename), and description (human-readable description)
- **Metadata block:** State (open/blocked/closed), priority, type, timestamps, creator
- **Context:** Composed from ancestor root documents (if any have Context sections)
- **Body:** Full markdown body content
- **Acceptance Criteria:** Composed from ancestor roots (if any have Acceptance sections)
- **Parent:** Directory root document (provides context)
- **Depends on:** Tasks in the `blocked-by` field (what this depends on)
- **Blocks:** Tasks in the `blocking` field (what depends on this)
- **Related:** Other documents linked in the body text

State is computed from filesystem location: tasks in `.closed/` directories are
closed, tasks with open blockers are blocked, all others are open.

The `--raw` flag skips template composition, showing only the task's own content
without ancestor Context or Acceptance Criteria sections.

### Knowledge Base Display Format

For knowledge base documents (no `task-type`), the output uses the same
`name - description` header format as tasks:

```
LDC76X: authentication-design - OAuth 2.0 implementation design for the auth subsystem

---
[Full markdown body content here]
---

Related (2):
  LBBBBB: security-policy - Security guidelines and threat model [doc]
  L2222X: api-design - REST API design principles [doc]
```

**Header components:**

- **ID:** The Lattice ID
- **Name:** The `name` frontmatter field (lowercase-hyphenated, derived from filename)
- **Description:** The human-readable purpose summary

Knowledge base documents do not display timestamps in the default view since
they typically lack explicit `created-at`/`updated-at` fields. Git history
can be queried separately if modification dates are needed.

### Document Reference Format

All document references throughout `lat show` output use a consistent format:

```
<id>: <name> - <description> [<type-indicator>]
```

Where `<type-indicator>` is:
- For open tasks: `P<N>` (e.g., `P0`, `P1`)
- For closed tasks: `P<N>/closed` (e.g., `P0/closed`)
- For knowledge base: `doc`

The `name` is the filename-derived identifier (lowercase-hyphenated). The
`description` is the human-readable summary. This format applies consistently
to both tasks and knowledge base documents.

Examples:
```
LB234X: fix-login - Fix login after password reset [P0]
L234YX: old-bug - Previously fixed issue [P1/closed]
L567CX: authentication-design - OAuth 2.0 implementation design [doc]
LDAB2X: llmc-development - LLMC Development [doc]
```

### Related Document Selection

The "Related" section displays documents linked from the body text. A document
is considered "related" if:

1. **Explicit link:** The body contains a markdown link with the document's
   Lattice ID (either as `[text](path#ID)` or `[text](ID)`)
2. **Not a dependency:** The document is not already listed in Depends on,
   Blocks, or Parent sections

When multiple related documents exist, root documents (those whose filename
matches their directory name) are highlighted preferentially as they typically
provide important context.

The list is ordered by first appearance in the body text. Maximum 10 related
documents are shown in text output; use `--json` for complete list.

### Name vs Description Distinction

For ALL documents:
- `name`: Lowercase-hyphenated identifier derived from filename (required, max 64 chars)
- `description`: Human-readable summary (required, max 1024 chars)
- Body text: Extended content

The `name` field is always derived from the document's filename (underscores →
hyphens, `.md` stripped). This is a core Lattice invariant.

For tasks, `description` serves as the human-readable task title shown in headers
and list views (e.g., "Fix login bug after password reset"). For knowledge base
documents, `description` provides a purpose summary for AI context.

Both fields are required. The `description` appears in `lat show` headers,
`lat show --short`, and `lat overview` output.

### Short Format

The `--short` flag produces single-line output:

```
$ lat show LB234X --short
LB234X [open] P0 bug: fix-review-tasks - Fix LLMC v2 code review tasks
```

Format: `<id> [<state>] <priority> <type>: <name> - <description>`

State is `open`, `blocked`, or `closed` (derived from filesystem location and
blocker resolution).

For knowledge base documents:
```
$ lat show LDC76X --short
LDC76X [doc]: authentication-design - OAuth 2.0 implementation design
```

### Peek Format

The `--peek` flag shows a condensed preview suitable for quick context:

```
$ lat show LB234X --peek
LB234X: fix-review-tasks - Fix LLMC v2 code review tasks [P0/open/bug]
Parent: LAA42X | Blocks: 5 | Depends: 1
```

This is useful for getting minimal context when browsing multiple tasks.

### References Format

The `--refs` flag shows tasks that reference this one (reverse lookup):

```
$ lat show LB234X --refs
References to LB234X:

  Blocks (5):
    L3456X: crash-count - Fix crash count not being incremented [P0 - open]
    L4567X: worker-nudging - Fix stuck worker nudging to be async [P1 - open]
    ...

  Linked from (2):
    L7CDAX: sprint-3-planning - Sprint 3 planning document [doc] (line 42)
    LCDABX: code-review-checklist - Code review checklist [doc] (line 15)
```

### JSON Output Format

The `--json` flag produces structured output compatible with `bd show --json`:

```json
[
  {
    "id": "LB234X",
    "name": "fix-review-tasks",
    "description": "Fix LLMC v2 code review tasks",
    "body": "Master task for addressing bugs and missing features...",
    "ancestors": [
      {"id": "LAA42X", "name": "llmc", "path": "tasks/llmc/llmc.md"}
    ],
    "composed_context": "Context from ancestor root documents...",
    "composed_acceptance": "- [ ] All tests pass\n- [ ] Code reviewed",
    "state": "open",
    "priority": 0,
    "task_type": "bug",
    "created_at": "2026-01-10T14:37:59.351489-08:00",
    "updated_at": "2026-01-10T14:37:59.351489-08:00",
    "path": "tasks/llmc/fix-review-tasks.md",
    "labels": ["llmc", "code-review"],
    "dependencies": [
      {
        "id": "L2345X",
        "name": "rebase-detection",
        "description": "Fix incorrect rebase necessity detection in patrol",
        "state": "closed",
        "priority": 0,
        "task_type": "bug"
      }
    ],
    "dependents": [
      {
        "id": "L3456X",
        "name": "crash-count",
        "description": "Fix crash count not being incremented in patrol",
        "state": "open",
        "priority": 0,
        "task_type": "bug"
      }
    ],
    "related": [
      {
        "id": "L567CX",
        "name": "llmc-design",
        "description": "Design document for LLMC agent coordination system"
      }
    ],
    "parent": {
      "id": "LAA42X",
      "name": "llmc-development",
      "description": "LLMC Development"
    },
    "claimed": false
  }
]
```

**JSON keys for backwards compatibility with `bd show --json`:**

| Key | Type | Description |
|-----|------|-------------|
| `id` | string | Lattice ID |
| `name` | string | Filename-derived identifier (lowercase-hyphenated) |
| `description` | string | Human-readable description (task title or KB purpose summary) |
| `body` | string | Full markdown body content |
| `ancestors` | array | Root documents in hierarchy order (tasks only, omitted if empty) |
| `composed_context` | string | Context composed from ancestors (tasks only, omitted if null) |
| `composed_acceptance` | string | Acceptance criteria composed from ancestors (tasks only, omitted if null) |
| `state` | string | Computed task state (open/blocked/closed) |
| `priority` | int | Priority level (0-4) |
| `task_type` | string | bug/feature/task/chore |
| `created_at` | string | ISO 8601 timestamp |
| `updated_at` | string | ISO 8601 timestamp |
| `closed_at` | string | ISO 8601 timestamp (if closed) |
| `path` | string | Relative file path |
| `labels` | array | List of labels |
| `dependencies` | array | Tasks this depends on (blocked-by) |
| `dependents` | array | Tasks that depend on this (blocking) |
| `related` | array | Documents linked from body text |
| `parent` | object | Directory root document |
| `claimed` | bool | Whether locally claimed |

The `state` field is computed from filesystem location: `closed` if in a
`.closed/` directory, `blocked` if any `blocked-by` entries reference open
tasks, otherwise `open`. The `description` field matches the YAML frontmatter
field name. For tasks, this is the task title (e.g., "Fix login bug"). For
knowledge base documents, this is the purpose summary.

### Multiple Documents

When showing multiple documents, output is separated by blank lines (text) or
returned as a JSON array:

```
$ lat show LB234X L567CX
LB234X: First task description
...

L567CX: Second task description
...
```

## lat ready

The `lat ready` command shows work available to start: tasks that are not
closed (not in `.closed/`), have all blockers closed, and are not claimed.

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
Ready work (4 tasks with no blockers):

1. [P0] [bug] LB234X: Fix LLMC v2 code review tasks
2. [P1] [task] L567CX: Convert strings.toml to Fluent format
3. [P1] [feature] LDAB2X: LLMC v2: Agent Coordination System
4. [P1] [feature] L3456X: Tabula V2: Complete Card Data Loading Rewrite
```

### Ready Criteria

A task is "ready" if:
1. Not in a `.closed/` directory
2. All `blocked-by` tasks are closed (in `.closed/`)
3. Priority is not P4 (backlog) unless `--include-backlog`
4. Not currently claimed (unless `--include-claimed`)

### Filter Options

| Flag | Description |
|------|-------------|
| `--parent <id>` | Filter to descendants of this directory |
| `--priority <N>` | Filter by priority level |
| `--type <type>` | Filter by task type |
| `--label <list>` | Filter by labels (AND logic) |
| `--label-any <list>` | Filter by labels (OR logic) |
| `--limit <N>` | Maximum tasks (default 10) |
| `--include-backlog` | Include P4 tasks |
| `--include-claimed` | Include claimed tasks |

### Sort Policies

| Policy | Behavior |
|--------|----------|
| `hybrid` | Default. Priority first, then creation date |
| `priority` | Strict priority ordering |
| `oldest` | Creation date ascending |

### Pretty Format

The `--pretty` flag displays a visual tree with state symbols:

```
$ lat ready --pretty
o P0 LB234X - [BUG] Fix LLMC v2 code review tasks

o P1 L3456X - [FEATURE] Tabula V2: Complete Card Data Loading Rewrite
|-- o P1 L567CX - Convert strings.toml to Fluent format

o P1 LDAB2X - [FEATURE] LLMC v2: Agent Coordination System

--------------------------------------------------------------------------------
Total: 4 tasks (4 open, 0 claimed)

Legend: o open | x claimed | (blocked) | P0 P1 P2 P3 P4
```

### JSON Output Format

The `--json` flag produces output compatible with `bd ready --json`:

```json
[
  {
    "id": "LB234X",
    "description": "Fix LLMC v2 code review tasks",
    "body": "Master task for addressing bugs...",
    "state": "open",
    "priority": 0,
    "task_type": "bug",
    "created_at": "2026-01-10T14:37:59.351489-08:00",
    "updated_at": "2026-01-10T14:37:59.351489-08:00",
    "path": "tasks/llmc/fix-review-tasks.md",
    "labels": [],
    "parent": {
      "id": "LAA42X",
      "description": "LLMC Development"
    }
  }
]
```

Field names match the YAML frontmatter: `description` for the task title, `body`
for markdown content. The `state` field is always `open` for ready tasks (tasks
with blockers or closed tasks are never ready). The JSON output includes the
full body text for each ready task, enabling AI agents to understand task
context without additional queries.

## lat prime

The `lat prime` command outputs AI-optimized workflow context, following
`bd prime` behavior. Lattice never runs `git push` or any equivalent sync
operation, so the session protocol focuses on local validation and commits.

### Basic Usage

```
lat prime
lat prime --full
lat prime --export
```

The `--export` flag outputs the context in a format suitable for copying into
external systems or documentation.

### Default Output

```
$ lat prime
# Lattice Workflow Context

> **Context Recovery**: Run `lat prime` after compaction or new session

## Session Protocol

Before completing work, run this checklist:

[ ] 1. lat check           (validate documents)
[ ] 2. lat fmt             (normalize links)
[ ] 3. git status          (review changes)
[ ] 4. git add <files>     (stage changes)
[ ] 5. git commit -m "..." (commit work)

## Core Commands

- `lat overview` - See most critical documents
- `lat ready` - Show tasks ready to work
- `lat show <id>` - View document/task details (includes parent, dependencies, related)
- `lat claim <id>` - Claim task for local work
- `lat close <id>` - Mark task completed

## Link Authoring

Always write links in shorthand format using just the Lattice ID:

    See [the design doc](LXXXXX) for details.

Running `lat fmt` at the end of work will expand to full path+fragment format:

    See [the design doc](../path/to/doc.md#LXXXXX) for details.

This avoids needing to look up file paths when authoring documents.
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

The `lat claim` command marks a task as locally in progress on the current
machine. This state is NOT stored in markdown files or tracked in git;
instead it persists to `.lattice/claims/` under the project root (gitignored).

### Basic Usage

```
lat claim <id>                     # Claim a task
lat claim --list                   # Show all claims
lat claim --release <id>           # Release a claim
lat claim --release-all            # Release all claims
lat claim --release-worktree <path> # Release claims from a specific worktree
lat claim --gc                     # Clean up stale claims
```

### Claim Storage

Each claim is a separate file in `.lattice/claims/` under the project root:

```
project/
  .lattice/
    claims/
      LB234X.json
      L567CX.json
```

The `.lattice/` directory is gitignored and used for local state including
the SQLite index, logs, and claims. Each claim file contains minimal JSON:

```json
{
  "claimed_at": "2026-01-14T10:30:00Z",
  "work_path": "/path/to/repo1/worktree-feature"
}
```

File creation and deletion are atomic on POSIX systems, so no explicit
locking is needed. Claiming creates the file; releasing deletes it.

### Claim Lifecycle

```
lat ready          # Task shows as "ready"
lat claim LB234X    # Task now claimed
lat show LB234X     # Shows "Claimed: true" in output
lat ready          # Task no longer appears (unless --include-claimed)
lat close LB234X    # Closing auto-releases the claim
```

### Auto-Release on State Change

When a task becomes closed, its claim is automatically released. This happens
during:

- `lat close <id>` (moves task to `.closed/`)

Claims are NOT released when a task becomes blocked (blockers are added), since
the task may still be actively worked on.

### Stale Claim Detection

The `lat claim --gc` command removes stale claims:

```
$ lat claim --gc
Checking 3 claims...
Released: LB234X (task closed)
Released: L567CX (work path no longer exists)
Kept: LDAB2X (active)
```

A claim is stale if:
- The task is in a `.closed/` directory
- The work path no longer exists
- The claim is older than 7 days (configurable)

### Crash Recovery

If an agent crashes while working on a task, the claim persists. Release
it via `lat claim --release LB234X` or delete the claim file directly.

### Display in lat show

The `lat show` command indicates claim status:

```
LB234X: Fix LLMC v2 code review tasks
State: open
Priority: P0
Type: bug
Claimed: true
...
```

### Display in lat ready

By default, `lat ready` excludes claimed tasks:

```
$ lat ready
Ready work (3 tasks with no blockers):
...

$ lat ready --include-claimed
Ready work (4 tasks, 1 claimed):
1. [P0] [bug] LB234X: Fix LLMC v2... [CLAIMED]
...
```

### No Assignee Concept

Lattice intentionally has no "assignee" field. The claim system tracks
which machine is working on a task, not who. This design:

1. Supports multi-agent workflows where work is coordinated externally
2. Avoids stale assignee data when agents are replaced
3. Keeps task files clean of operational state

## Command Summary

| Command | Purpose | Modifies Files |
|---------|---------|----------------|
| `lat show` | View document details | No |
| `lat ready` | Find available work | No |
| `lat prime` | AI workflow context | No |
| `lat claim` | Track local work | No (uses .lattice/claims/) |

All four commands are read-only with respect to repository markdown files.
The claim command modifies only the local `.lattice/claims/` directory,
which is gitignored.
