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
- `parent-id`: ID of parent document, auto-populated by `lat fmt` from directory root
- `name`: Lowercase-hyphenated identifier derived from filename (required, max 64 chars)
- `description`: Human-readable summary (required, max 1024 chars)

The `name` field is always derived from the document's filename: underscores
become hyphens and the `.md` extension is stripped (e.g., `fix_login_bug.md`
→ `fix-login-bug`). This is a core Lattice invariant enforced by the linter.

For tasks, `description` serves as the display title shown in `lat show` and
other outputs (e.g., "Fix login bug after password reset"). For knowledge base
documents, `description` provides a purpose summary for AI context.

**Task Tracking Keys:**
- `task-type`: bug/feature/task/epic/chore
- `status`: open/blocked/closed/tombstone
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

Files with names starting with numeric prefixes like `00_`, `01_`, `02_`, etc.
(e.g., `00_master_plan.md`) indicate document priority within a directory.
Alternatively, a file named `README.md` serves the same purpose as a `00_`
prefixed file. Both naming conventions are equally acceptable for marking the
highest priority document that serves as the directory root, providing parent
context for all other documents in that directory.

The `lat fmt` and `lat create` commands automatically populate the `parent-id`
field in each document's frontmatter based on the directory's root document.
This makes hierarchy explicit without requiring manual parent specification.
Documents without a root document in their directory have no `parent-id`.

Higher-numbered prefixes (`01_`, `02_`, etc.) indicate progressively lower
priority. Commands like `lat show` and `lat overview` use these prefixes when
selecting which related documents to highlight.

## The ID System

A Lattice ID is a compact, human-typeable identifier: `L` prefix + Base32
document counter + Base32 client ID. Example: `LK3DTX` (document 675, client
DTX). Uses RFC 4648 Base32 (A-Z, 2-7) avoiding ambiguous characters.

See [Appendix: ID System](appendix_id_system.md) for generation algorithm,
collision handling, and client identification.

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

**lat ready** - Shows work available to start: open tasks with no blockers
that are not claimed. Supports `--parent` for directory filtering, `--pretty`
for visual tree display, and `--json` for full task details.

**lat overview** - Provides repository-level context for AI agents. Shows the
most critical documents based on view frequency, recency, and priority. Supports
`--limit`, `--json`, and various filtering options. Tracks local view counts
in `.lattice/views.json` to surface frequently-referenced documents.

**lat prime** - Outputs AI-optimized workflow context including recommended
link authoring format (shorthand `[text](ID)` links that `lat fmt` expands).
Supports custom checklist via `.lattice/config.toml`.

**lat claim** - Marks tasks as locally in progress on the current machine.
Claims are stored in `~/.lattice/claims/` as one file per claim, not in markdown
files. Supports atomic updates across multiple worktrees and automatic release
on status change.

See [Appendix: Workflow](appendix_workflow.md) for complete command specifications,
output formats, and claiming behavior, and
[Appendix: Overview Command](appendix_overview_command.md) for the ranking algorithm.

### Task and Document Management

Commands for creating and modifying tasks and documents.

**lat create** - Creates new documents with `lat create <path> "<description>"
[options]`. Works for both tasks and knowledge base documents. The `name` field
is derived from the filename automatically. The description is a required
positional argument. For tasks, add `-t <type>` to specify task type; omitting
`-t` creates a knowledge base document. Supports priority, labels, and
dependencies for tasks.

**lat update** - Modifies existing documents with `lat update <id> [id...]
[options]`. Supports changing status, priority, type, and managing labels.
Can update multiple tasks atomically, useful for bulk operations like marking
dependencies as blocked or changing priority across related tasks.

**lat close** - Marks tasks as closed, accepting single or multiple lattice IDs.
Automatically releases any local claims and sets the `closed-at` timestamp.
Supports `--reason` for documenting why the task was closed.

See [Appendix: Task Tracking](appendix_task_tracking.md) for the complete
task lifecycle and [Appendix: CLI Structure](appendix_cli_structure.md) for
full command reference.

### Document Management

Commands for searching, validating, and formatting documents.

**lat list** - Searches and filters documents with flexible query options.
Supports filtering by status, priority, type, labels, timestamps, and path
prefix. Returns formatted lists with `--pretty` for visual display or `--json`
for programmatic access.

**lat check** - Validates documents and repository state before committing.
Detects duplicate IDs, broken links, invalid frontmatter, circular
dependencies, and documents exceeding size limits. Essential for maintaining
repository integrity.

**lat fmt** - Formats documents and normalizes links. Wraps text at 80
characters, adds missing Lattice ID fragments to links, and updates file paths
when documents are renamed or moved.

See [Appendix: Linter](appendix_linter.md) for validation rules and
[Appendix: CLI Structure](appendix_cli_structure.md) for full command reference.

## Linking System

Lattice links use standard markdown with relative paths and Lattice ID
fragments: `[text](path/doc.md#LJCQ2X)`. Write shorthand `[text](LJCQ2X)` and
`lat fmt` expands to full path. The index maintains bidirectional references
for impact analysis.

See [Appendix: Linking System](appendix_linking_system.md) for link format
specification, normalization algorithm, and edge cases.

## Task Tracking

Tasks and knowledge base documents share a unified ID space. Hierarchy comes
from the filesystem: all tasks in a directory are siblings, with the root
document (`README.md` or `00_*`) as their parent epic.

Status transitions: `open ↔ blocked → closed` (plus `tombstone` for permanent
deletion). No `in_progress` status; use `lat claim` for local work tracking.

See [Appendix: Task Tracking](appendix_task_tracking.md) for lifecycle, types,
priorities, and dependencies.

## Task Templates

Directory root documents can include `[Lattice] Context` and `[Lattice]
Acceptance Criteria` sections that compose into descendant tasks at display
time. Context composes general→specific; acceptance composes specific→general.

See [Appendix: Task Templates](appendix_task_templates.md) for section format
and composition rules.

## Linter and Formatter

**`lat check`** validates documents: duplicate/invalid IDs, broken references,
invalid frontmatter, circular dependencies, missing required fields. Warnings
for documents exceeding 500 lines.

**`lat fmt`** normalizes formatting: 80-char wrapping, ATX headers, dash list
markers. Expands shorthand links, updates paths on rename/move.

**`lat split`** divides large documents by top-level sections into a root
document with linked children.

See [Appendix: Linter](appendix_linter.md) for complete rule set (E001-E010,
W001-W017, S001-S003).

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

## Skill Integration

Documents with `skill: true` become Claude Skills via symlinks in
`.claude/skills/`. The `name` (max 64 chars) and `description` (max 1024 chars)
fields follow Claude's SKILL.md validation rules.

See [Appendix: AI Integration](appendix_ai_integration.md) for hooks and
[Appendix: Startup Operations](appendix_startup_operations.md) for symlink sync.

## Configuration

Layered: defaults → `~/.lattice.toml` → `.lattice/config.toml` → env vars →
CLI flags. See [Appendix: Configuration](appendix_configuration.md).

## Logging

Operations log to `.lattice/logs.jsonl` (JSONL). Use `--verbose` for detail,
`--json` for structured output.

## Testing

Black-box CLI tests with `GitOps` trait for injection (`FakeGit` in tests).
See [Appendix: Testing Strategy](appendix_testing_strategy.md).

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
