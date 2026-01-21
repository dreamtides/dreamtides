---
lattice-id: LBTWQN
name: lattice-design
description: |-
  Master technical design document for the `lat` command and lattice knowledge
  base and task tracking system
created-at: 2026-01-18T05:53:44.399328Z
updated-at: 2026-01-21T22:31:38.481497Z
---

# Lattice Document System Technical Design

## Executive Summary

Lattice is a unified knowledge base and task tracking system built on markdown
files stored in git repositories, with SQLite providing an ephemeral index for
query performance. The system prioritizes document atomicity through strict
size limits, rich cross-referencing capabilities, and compatibility with
AI agent workflows.

The core innovation of Lattice is treating markdown documents as first-class
database entities while maintaining full git compatibility and human
readability. Documents can exist anywhere in a project hierarchy, colocated
with relevant code, and are identified by their `lattice-id` YAML annotation
rather than filesystem location.

## Design Philosophy

### Document Atomicity

The 500-line soft limit per document encourages atomic, focused documentation.
Users are encouraged to split large documents through the `lat split` command,
and the linter warns when documents exceed this threshold. This promotes better
organization and makes documents easier to navigate and reference.

The system draws inspiration from Claude's SKILL.md format, using YAML
frontmatter for metadata and markdown body for content. This compatibility
allows Lattice documents to function as Claude Skills when marked with
`skill: true` in their frontmatter, with symlinks automatically generated
in the .claude directory.

### Git as Source of Truth

SQLite serves exclusively as a performance cache, never as the canonical data
store. Any SQLite state can be safely deleted and rebuilt from the git
repository. This design principle means Lattice never runs background daemon
processes; instead, common maintenance operations execute opportunistically
before user commands, treating each `lat` invocation as an ad-hoc daemon
opportunity. See [Appendix: Startup Operations](appendix_startup_operations.md)
for the complete list of operations that run on every command.

The index reconciliation strategy uses git's change detection mechanisms to
identify stale index entries. When the index cannot be confidently validated,
Lattice falls back to a full rebuild rather than attempting complex incremental
repairs. See [Appendix: Indexing](appendix_indexing.md) for the schema and
reconciliation algorithm.

### Bulletproof Self-Healing

Lattice aims to be a self-healing abstraction that handles errors gracefully
and continues operation whenever possible. When recovery is impossible, errors
are reported with clear, human-readable messages rather than stack traces. The
system distinguishes between user errors (invalid input) and system errors
(internal failures), following the HTTP 400/500 distinction in exit codes.

Silent failure is treated as a critical design flaw. All operations log
extensively to `.lattice/logs.jsonl`, capturing user operations, git
operations, SQLite operations, and general observations about repository
state. This logging enables post-hoc debugging when problems arise.

## Document Structure

### YAML Frontmatter

Every Lattice document begins with YAML frontmatter containing three required
fields: `lattice-id`, `name`, and `description`. Missing any of these is a
linter error.

The frontmatter keys are deliberately designed to avoid conflicts with
Claude's SKILL.md format. The reserved keys include:

**Identity Keys:**

- `lattice-id`: Unique document identifier (required)
- `parent-id`: ID of parent document, auto-populated by `lat fmt` from directory
  root
- `name`: Lowercase-hyphenated identifier derived from filename (required, max
  64 chars)
- `description`: Human-readable summary (required, max 1024 chars)

The `name` field is always derived from the document's filename: underscores
become hyphens and the `.md` extension is stripped. Filenames may optionally
include a trailing lattice ID suffix (e.g., `fix_login_bug_LABCDEF.md`), which
is stripped when deriving the name. Examples:

- `fix_login_bug.md` → `fix-login-bug`
- `fix_login_bug_LABCDEF.md` → `fix-login-bug` (ID suffix stripped)

This is a core Lattice invariant enforced by the linter.

For tasks, `description` serves as the human-readable task title shown in
`lat show` and other outputs (e.g., "Fix login bug after password reset"). For
knowledge base documents, `description` provides a purpose summary for AI
context.

**Task Tracking Keys:**

- `task-type`: bug/feature/task/chore
- `priority`: 0-4 (0 highest)
- `labels`: List of arbitrary string labels
- `blocking`: List of task IDs with hard dependencies on this task
- `blocked-by`: List of task IDs this task depends on
- `discovered-from`: List of parent tasks from which this was discovered
- `created-at`, `updated-at`, `closed-at`: ISO 8601 timestamps

**Skill Integration Keys:**

- `skill`: Boolean enabling Claude Skill generation

See [Appendix: Task Tracking](appendix_task_tracking.md) for the complete
task lifecycle state machine.

### Document Body

The markdown body follows standard CommonMark syntax with Lattice extensions
for ID-based linking. Links use standard markdown syntax with relative file
paths and Lattice IDs as URL fragments:

```
See the [error handling](error_handling.md#LJCQ2X) document for more information
```

Users can write partial links like `[text](../path/to/doc.md)` or
`[text](LJCQ2X)`, and the `lat fmt` command will normalize them to include both
the file path and Lattice ID. The formatter also handles document renames and
moves, rewriting links based on their Lattice ID to point to updated file
paths.

The body text has no hard length limit, but the linter warns at 500 lines.
Documents exceeding this should be split into multiple files using the
`lat split` command.

### Root Documents and Hierarchy

A **root document** is either:

- A document whose filename (without `.md` extension) matches its containing
  directory name (e.g., `api/api.md`)
- A document whose filename starts with `00_` (e.g., `lattice/00_design.md`)

The `00_` prefix indicates a document that is part of a numbered series (00_,
01_, 02_, etc.) where the 00_ document serves as the root. This convention is
useful when you want to organize related documents in a sequence. The numbered
series convention is not enforced by Lattice; only the `00_` prefix is
recognized as a root indicator.

It is an error to have both forms in the same directory (see E013). Root
documents serve as the default parent for all other documents in that directory.

Example directory structure:

```
project/
├── api/
│   ├── api.md                    # Root document (matches directory name)
│   ├── api_design.md             # Knowledge base document
│   ├── security_model.md
│   ├── implement_auth.md         # Task document
│   ├── fix_rate_limit.md
│   └── .closed/
│       └── add_logging.md        # Closed task
└── database/
    ├── database.md               # Root document
    ├── schema_design.md
    └── migrate_tables.md
```

Directory organization is a user convention; Lattice does not enforce any
particular structure beyond root documents and `.closed/` directories. Users
may organize documents however they prefer (e.g., separate `tasks/` and `docs/`
subdirectories, flat structures, or any other arrangement).

The `lat fmt` and `lat create` commands automatically populate the `parent-id`
field in each document's frontmatter based on the directory's root document.
This makes hierarchy explicit without requiring manual parent specification.
Documents without a root document in their directory have no `parent-id`.

See [Appendix: Linter](appendix_linter.md) for the complete rule set.

## The ID System

A Lattice ID is a compact, human-typeable identifier with minimum 6 characters:
`L` prefix + Base32 document counter (2+ digits) + Base32 client ID (3-6
digits).
Example: `LJCQ2X`. Uses RFC 4648 Base32 (A-Z, 2-7) avoiding ambiguous
characters.

The counter and client ID are concatenated and scrambled using an 8-round
Feistel
permutation with hardcoded keys before encoding. This ensures consecutive IDs
appear visually distinct (e.g., `LJCQ2X`, `LWN5RP`, `L4DKAT`) while remaining
deterministic and reversible across all clients.

See [Appendix: ID System](appendix_id_system.md) for the scrambling algorithm,
client ID selection, and collision handling.

## Command Overview

Lattice provides commands for document creation, viewing, and work management,
designed for compatibility with beads (`bd`) while supporting Lattice's
filesystem-centric model.

### Workflow Commands

Commands for viewing documents and managing work progress.

**lat show** - Displays document details following `bd show` format. Supports
single or multiple documents, with `--json`, `--short`, and `--refs` options.
Default output includes parent, dependencies, blocking tasks, and related
documents—providing full context for AI agents in a single call.

**lat ready** - Shows work available to start: tasks that are not closed, have
no open blockers, and are not claimed. Supports `--parent` for directory
filtering, `--pretty` for visual tree display, and `--json` for full task
details.

**lat overview** - Provides repository-level context for AI agents. Shows the
most critical documents based on view frequency, recency, and root document
priority. Supports `--limit`, `--json`, and various filtering options. Tracks
local view counts in SQLite (concurrent-safe) to surface frequently-referenced
documents.

**lat prime** - Outputs AI-optimized workflow context.

**lat claim** - Marks tasks as locally in progress on the current machine.
Claims are stored in `.lattice/claims/` (gitignored) as one file per claim, not
in markdown files. Supports automatic release when tasks are closed.

**lat pop** - The primary interface for AI agents to start work. Combines
`lat ready`, `lat claim`, and `lat show` into a single operation: finds the
highest-priority ready task (sorted by priority then age), claims it, and
outputs full context. Returns silently with exit code 0 if no ready tasks
are available (no output to stdout or stderr). Supports `--json` output
(recommended for programmatic use),
`--dry-run` to preview without claiming, `--no-claim` to skip claiming,
`--max-claims` to fail if too many active claims exist, and all filter options
from `lat ready` (`--type`, `--priority`, `--label`, etc.).
JSON output includes the complete ShowOutput structure with task metadata, body,
composed context and acceptance criteria from ancestor templates, dependencies,
dependents, and related documents.

See [Appendix: Workflow](appendix_workflow.md) for complete command
specifications,
output formats, and claiming behavior, and
[Appendix: Overview Command](appendix_overview_command.md) for the ranking
algorithm.

### Task and Document Management

Commands for creating and modifying tasks and documents.

**lat create** - Creates new documents with `lat create <parent> "<description>"
[options]`. The `<parent>` argument specifies the parent directory; the filename
is auto-generated from the description (lowercase, underscores, significant
words). Documents are created directly in the specified directory. For root
documents, specify the full path explicitly (e.g., `api/api.md`). Supports
priority, labels, and dependencies for tasks. Use `--interactive` (`-i`) to
prompt for the parent directory (with tab completion) and open `$EDITOR` for
writing body content; the description and filename are auto-generated from the
body text. The last used parent directory is remembered for subsequent
interactive calls. Use `--commit` (`-c`) to create a git commit after document
creation; the commit message includes the full document text with the first line
formatted as "Create {type} {id}" (e.g., "Create feature request L34567"). This
works with both regular and interactive mode.

**lat update** - Modifies existing documents with `lat update <id> [id...]
[options]`. Supports changing priority, type, and managing labels. To change
task state, use `lat close` or `lat reopen`. Can update multiple tasks
atomically, useful for bulk operations like changing priority across related
tasks.

**lat close** - Closes tasks by moving them to a `.closed/` subdirectory under
their current location (e.g., `api/tasks/foo.md` → `api/tasks/.closed/foo.md`).
All markdown links to closed tasks are automatically rewritten to the new path,
similar to `lat mv`. Sets the `closed-at` timestamp and releases any local
claims.

**lat prune** - Permanently removes closed tasks from the repository. Requires
either a path argument or `--all` to prune all closed tasks. YAML frontmatter
references to pruned tasks are removed automatically. Inline markdown links to
pruned tasks produce an error unless `--force` is passed, which converts them
to plain text.

See [Appendix: Task Tracking](appendix_task_tracking.md) for the complete
task lifecycle and [Appendix: CLI Structure](appendix_cli_structure.md) for
full command reference.

### Document Management

Commands for searching, validating, and formatting documents.

**lat list** - Searches and filters documents with flexible query options.
Supports filtering by state (open/blocked/closed), priority, type, labels,
timestamps, and path prefix. By default excludes closed tasks; use
`--include-closed` to see them. Returns formatted lists with `--pretty` for
visual display or `--json` for programmatic access.

**lat check** - Validates documents and repository state before committing.
Detects duplicate IDs, broken links, invalid frontmatter, and circular
dependencies. Warns about documents exceeding the 500-line size limit. Essential
for maintaining repository integrity.

**lat fmt** - Formats documents and normalizes links. Wraps text at 80
characters, adds missing Lattice ID fragments to links, and updates file paths
when documents are renamed or moved.

See [Appendix: Linter](appendix_linter.md) for validation rules and
[Appendix: CLI Structure](appendix_cli_structure.md) for full command reference.

### Query Commands

Commands for searching and filtering documents.

**lat stale** - Find tasks not updated recently, with configurable staleness
threshold.

**lat blocked** - Show tasks with unresolved blockers. Use `--show-blockers`
to display what's blocking each task.

**lat changes** - Show documents changed since a date or git commit.

**lat stats** - Display project statistics: document counts by state, priority
and type breakdowns, recent activity, and health metrics.

### Hierarchy Commands

Commands for navigating the document tree structure.

**lat tree** - Display directory structure with documents. Supports `--depth`
option.

**lat roots** - List all root documents (those whose filename matches their
directory name) with child counts.

**lat children** - List documents under a root's directory, optionally
recursive.

### Relationship Commands

Commands for exploring links between documents.

**lat links-from** - Show documents this document links to.

**lat links-to** - Show documents that link to this document (backlinks).

**lat path** - Find shortest path between two documents.

**lat orphans** - Find documents with no incoming links. Use `--exclude-roots`
to omit root documents from results.

**lat impact** - Analyze what would be affected by changes to a document.

### Utility Commands

Additional commands for document maintenance and exploration.

**lat track** - Add Lattice tracking to existing markdown files. Use `--force`
to regenerate IDs for documents with duplicates.

**lat edit** - Open document in editor (human-only, not for AI agents).

**lat reopen** - Reopens closed tasks by moving them from `.closed/` back to
their original parent directory. All links are rewritten to the restored path.

**lat generate-ids** - Pre-allocate IDs for offline authoring.

**lat mv** - Move document to new location, updating `parent-id` and all links.

**lat search** - Full-text search across document content using FTS5 queries.

**lat dep** - Manage dependencies: `add`, `remove`, and `tree` subcommands.

**lat label** - Manage labels: `add`, `remove`, `list`, and `list-all`
subcommands.

## Linking System

Lattice links use standard markdown with relative paths and Lattice ID
fragments: `[text](path/doc.md#LJCQ2X)`. Write shorthand `[text](LJCQ2X)` and
`lat fmt` expands to full path. The index maintains bidirectional references
for impact analysis.

See [Appendix: Linking System](appendix_linking_system.md) for link format
specification, normalization algorithm, and edge cases.

## Task Tracking

Tasks and knowledge base documents share a unified ID space. Hierarchy comes
from the filesystem: documents in the same directory are siblings, with the
directory's root document (filename matching directory name) as their parent.
The parent is typically a root document (not itself a task), though tasks can
have other tasks as parents if desired—in that case, use the standard task types
(bug, feature, task, chore) for the parent task.

Task state is determined by filesystem location, not by a status field:

- **Open**: Task exists outside of any `.closed/` directory
- **Blocked**: Task has open (non-closed) entries in its `blocked-by` field
- **Closed**: Task resides in a `.closed/` subdirectory

The `lat close` command moves tasks to a `.closed/` subdirectory under their
current location, and `lat reopen` moves them back. The `lat prune` command
permanently deletes closed tasks. There is no `in_progress` status; use
`lat claim` for local work tracking.

See [Appendix: Task Tracking](appendix_task_tracking.md) for lifecycle, types,
priorities, and dependencies.

## Task Templates

Root documents (those with filenames matching their directory) can include
`[Lattice] Context` and `[Lattice] Acceptance Criteria` sections that compose
into descendant tasks at display time. Context composes general→specific;
acceptance composes specific→general.

See [Appendix: Task Templates](appendix_task_templates.md) for section format
and composition rules.

## Linter and Formatter

**`lat check`** validates documents: duplicate/invalid IDs, broken references,
invalid frontmatter, circular dependencies, missing required fields, and
document naming conventions. Warnings for documents exceeding 500 lines.

**`lat doctor`** validates system health: index integrity, git repository state,
configuration, claims, and skill symlinks. Use `lat check` for document content
issues, `lat doctor` for infrastructure issues. See
[Appendix: Doctor Command](appendix_doctor.md) for the complete check list and
fix capabilities.

**`lat fmt`** normalizes formatting: 80-char wrapping, ATX headers, dash list
markers. Expands shorthand links, updates paths on rename/move.

**`lat split`** divides large documents by top-level sections into a root
document with linked children.

See [Appendix: Linter](appendix_linter.md) for complete rule set (E001-E012,
W001-W020, S001-S003).

## Index Architecture

SQLite index (`.lattice/index.sqlite`, gitignored) stores document metadata,
links, labels, and FTS5 full-text search. Reconciliation uses git to detect
changes; falls back to full rebuild when uncertain.

See [Appendix: Indexing](appendix_indexing.md) for schema, reconciliation
algorithm, and performance tuning.

## Git Integration

Git is the authoritative store. Documents discovered via `git ls-files`;
changes via `git diff`/`git status`. Lattice never performs git push/sync.

See [Appendix: Git Integration](appendix_git_integration.md) for operations
and [Appendix: Git Edge Cases](appendix_git_edge_cases.md) for shallow clones,
worktrees, submodules, etc.

## AI Integration

Lattice provides two integration points for AI coding agents:

**MCP Tools:** The `lat setup claude` command registers Lattice MCP tools
(`lattice_create_task`, `lattice_create_document`) with Claude Code. Each tool
invocation runs a stateless `lat mcp` command, enabling agents to create
documents with structured input and avoiding shell escaping issues. The
`lattice_create_task` tool appends the lattice ID to the filename (e.g.,
`fix_login_bug_LABCDEF.md`) for uniqueness; the `name` field is derived without
this suffix.

**Skill Documents:** Documents with `skill: true` become Claude Skills via
symlinks in `.claude/skills/`. The `name` (max 64 chars) and `description`
(max 1024 chars) fields follow Claude's SKILL.md validation rules.

See [Appendix: AI Integration](appendix_ai_integration.md) for MCP tool
specifications and
[Appendix: Startup Operations](appendix_startup_operations.md)
for symlink sync.

## Configuration

Layered: defaults → `~/.lattice.toml` → `.lattice/config.toml` → env vars →
CLI flags. See [Appendix: Configuration](appendix_configuration.md).

## Logging

Operations log to `.lattice/logs.jsonl` (JSONL). Use `--verbose` for detail,
`--json` for structured output. Logging uses the `tracing` crate.

## Testing

Black-box CLI tests with `GitOps` trait for injection (`FakeGit` in tests).
See [Appendix: Testing Strategy](appendix_testing_strategy.md).

## Benchmarking

Criterion benchmarks measure index rebuild, document parsing, and query latency.
See [Appendix: Benchmarking](appendix_benchmarking.md).

## Code Review

All Lattice changes should be reviewed against the checklist in
[Appendix: Code Review](appendix_code_review.md), covering error handling,
logging, concurrency, data integrity, performance, and code organization.

## Chaos Monkey

`lat chaosmonkey` runs random operations until system error, surfacing edge
cases. See [Appendix: Chaos Monkey](appendix_chaos_monkey.md).

## UI Design

Ayu color theme: green (success), yellow (warning), red (error), blue (accent),
gray (muted). Following Tufte's principles—maximize data-ink ratio, reserve
color for semantic states.

## Error Handling

Error handling in Lattice is divided into *expected* and *unexpected* failure
states. Expected errors are problems with user input or external systems the
user manages like the file system, while unexpected errors are internal
invariants and "impossible" code paths (we call these "system errors"). This is
a similar distinction to HTTP 400-series vs 500-series error codes.

Expected errors like invalid syntax, missing fields, invalid arguments, missing
files, permission problems, etc are handled via the `thiserror` crate and the
`LatticeError` enum.

Unexpected errors like invariant violations, index corruption, git operation
failures (lattice should ensure valid git state), out of memory errors, etc are
handled via `panic!` in Rust. Lattice uses the `human-panic` crate to format
error messages in a clear manner. The [Chaos Monkey](appendix_chaos_monkey.md)
searches for panics and runs using `RUST_BACKTRACE=1`.

Essentially this distinction is about *ownership*. If it is Lattice's "fault"
that a problem happened, we should panic. If it was the user's "fault" because
they did something wrong, we should not. Obviously this is a judgment call; in
gray areas we can default to the panic option.

For user errors, provide clear guidance on how to fix the problem. For system
errors, log extensively and suggest running `lat check` or rebuilding the index.
Never silently ignore errors. The `--json` flag provides structured error output
for programmatic handling.

See [Appendix: Error Handling](appendix_error_handling.md) for the complete
error taxonomy, structured output format, recovery strategies, and
implementation patterns.

## Project File Layout

Implementation: `rules_engine/src/lattice/` with modules for cli, index,
document, git, format, link, claim, lint, id, task, skill, log, error, test.

See [Appendix: File Layout](appendix_file_layout.md) for detailed structure.
