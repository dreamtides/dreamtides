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
opportunity.

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

Every Lattice document begins with YAML frontmatter containing at minimum a
`lattice-id` field. Knowledge base documents should also include `name` and
`description` fields; omitting these generates linter warnings.

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
- `status`: open/blocked/deferred/closed/tombstone/pinned
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
See the [error handling](error_handling.md#LJCQ2) document for more information
```

Users can write partial links like `[text](../path/to/doc.md)` or
`[text](LJCQ2)`, and the `lat fmt` command will normalize them to include both
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

A Lattice ID is a compact, human-typeable identifier consisting of:

1. A literal `L` prefix
2. A document counter (minimum 2 digits, RFC 4648 Base32 encoded)
3. A client identifier (2-5 digits, RFC 4648 Base32 encoded)

Example: `LK3DT` represents document `K3` (decimal 675) from client `DT`.

The Base32 encoding uses the RFC 4648 alphabet (A-Z followed by 2-7), avoiding
ambiguous characters like 0/O and 1/I. Client IDs are stored in `~/.lattice.toml`
and scale in length with the number of known clients. Document counters start at
50 (Base32: `BS`) to ensure minimum 5-character IDs overall.

Collision detection occurs during `lat check`. The `lat generate-ids` command
pre-allocates IDs for document authors.

See [Appendix: ID System](appendix_id_system.md) for the complete ID generation
algorithm, collision handling, and client identification details.

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
[Appendix: Overview Command](appendix_overview.md) for the ranking algorithm.

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

### Link Format

Lattice links use standard markdown syntax combining relative file paths with
Lattice ID fragments:

```
See the [error handling](docs/error_handling.md#LJCQ2) document for details
```

The `lat fmt` command normalizes links and handles several cases:
- `[text](../path/to/doc.md)` -> fills in Lattice ID if valid document
- `[text](LJCQ2)` -> adds file path
- Detects document renames/moves and rewrites links to new paths

All links use relative file system paths from the linking document's location.

See [Appendix: Linking System](appendix_linking_system.md) for the complete
link format specification and edge cases.

### Bidirectional References

The index maintains a reverse reference map enabling queries like "what
documents link to this one?" This powers features like impact analysis
when modifying or deleting documents.

## Task Tracking

### Integration with Knowledge Base

Tasks and knowledge base documents share a unified ID space, enabling
seamless cross-referencing. A task can link to design documents, and
design documents can reference tasks that track their implementation.

The primary organizational mechanism is the filesystem hierarchy rather
than explicit parent-child relationships. All tasks in a directory are
implicitly siblings, with the directory's root document acting as their
parent or "epic." This replaces beads' explicit epic/child model.

### Task Lifecycle

Task status transitions follow a state machine:

```
open -> closed
  |       ^
  |       |
  +---> blocked -------+
  |
  v
deferred  (back to open when unblocked)
```

The `tombstone` status represents deleted tasks that should not be
resurrected. The `pinned` status indicates permanent open items.

There is no "in_progress" status in Lattice. Instead, the `lat claim`
command tracks which machine is working on a task locally, without
modifying the task file.

See [Appendix: Task Tracking](appendix_task_tracking.md) for the complete
state machine and transition rules.

### CLI Commands

The task CLI follows beads' design philosophy, adapted for Lattice's
filesystem-centric model. Key differences from beads:

- Tasks require a path on creation to specify filesystem location
- No explicit parent/child relationships; hierarchy comes from directories
- The `name` field replaces beads' `title` concept
- No sync command; Lattice never performs git push operations

See [Appendix: CLI Structure](appendix_cli_structure.md) for the complete
command reference.

## Task Templates

Task templates provide reusable context and acceptance criteria through the
existing directory hierarchy. Directory root documents (`README.md` or `00_*.md`
files) can include `[Lattice] Context` and `[Lattice] Acceptance Criteria`
headings that automatically compose into all descendant tasks at display time.

This design requires no additional frontmatter fields—the filesystem hierarchy
IS the template structure. When displaying a task, Lattice walks up the
directory tree collecting ancestor root documents. Context sections compose
general-to-specific (project first, then domain, then subdomain). Acceptance
criteria compose specific-to-general, ensuring universal requirements like
"create git commit" anchor at the end.

The `[Lattice]` prefix ensures template sections are intentional—generic
"Context" headings in regular documents won't accidentally become templates.
Template changes propagate instantly. The `--raw` flag skips composition to
show only the task's own content.

See [Appendix: Task Templates](appendix_task_templates.md) for the complete
section format, composition rules, and common patterns.

## Linter and Formatter

### The Check Command

The `lat check` command validates documents and repository state:

**Error-level checks (prevent operations):**
- Duplicate Lattice IDs
- References to nonexistent IDs
- Invalid YAML frontmatter keys
- Missing required fields (`name`, `description`, and task-specific fields)
- Name-filename mismatch (name must derive from filename)
- Invalid status/type/priority values
- Circular blocking dependencies

**Warning-level checks:**
- Document exceeds 500 lines
- Markdown lint problems (inconsistent headers, bare URLs, etc.)
- Time-sensitive content detection (dates, "after August 2025", etc.)
- Inconsistent terminology within a document

The linter integrates mechanically verifiable rules from Claude's Skill
authoring best practices, including path format validation (no backslashes),
description length limits, and name format requirements.

See [Appendix: Linter](appendix_linter.md) for the complete rule set.

### The Format Command

The `lat fmt` command applies consistent formatting:

- Text wrapping at 80 characters (configurable)
- Consistent header styles (ATX headers with space after #)
- Consistent list markers (dashes for unordered)
- Proper indentation normalization
- Adding missing `name` fields from document filename
- Link normalization: adds Lattice ID fragments to file path links
- Link expansion: converts bare ID links `[text](LJCQ2)` to full path+fragment
- Link maintenance: updates paths when documents are renamed or moved

The formatter attempts to auto-correct problems identified by `lat check`
when a deterministic fix exists.

### The Split Command

The `lat split` command takes a large document and divides it by top-level
sections:

1. The first text block and first section become a root document (e.g., `README.md`)
2. Each subsequent section becomes a standalone document with a numeric prefix
3. The root document receives links to all child documents
4. All new documents receive generated Lattice IDs

This enables progressive breakdown of monolithic documents into
AI-friendly sizes.

## Index Architecture

The SQLite index stores document metadata, link relationships, and search
indices. Key tables include `documents`, `links`, `labels`, `fts_content`
(full-text search), `client_counters`, `directory_roots` (precomputed
hierarchy), and `content_cache`.

Index reconciliation uses git metadata to determine staleness—querying for
modified files, re-indexing changes, and falling back to full rebuild when
git state is unclear. The index lives in `.lattice/index.sqlite` (gitignored).

See [Appendix: Indexing](appendix_indexing.md) for the complete schema,
reconciliation algorithm, and performance tuning.

## Git Integration

Lattice uses git as the authoritative store. Documents are discovered via
`git ls-files` (not filesystem traversal), and changes are detected via
`git diff` and `git status`. Lattice never performs git push or sync
operations—this supports multi-agent workflows where a coordinator manages
synchronization externally.

See [Appendix: Git Integration](appendix_git_integration.md) for the complete
specification and [Appendix: Git Edge Cases](appendix_git_edge_cases.md) for
behavior in non-standard configurations (shallow clones, worktrees, submodules).

## Skill Integration

### Automatic Skill Generation

Documents with `skill: true` in their frontmatter automatically become
Claude Skills. Lattice generates symlinks in `.claude/skills/` pointing
to the actual Lattice documents, enabling Claude Code to discover them
without file duplication.

### Format Compatibility

Lattice deliberately avoids conflicts with Claude's SKILL.md format. The
`name` and `description` fields follow Claude's validation rules:

- `name`: Max 64 characters, lowercase letters/numbers/hyphens only
- `description`: Max 1024 characters, non-empty

When a Lattice document is marked as a skill, `lat check` enforces these
stricter validation rules.

See [Appendix: Claude Code Integration](appendix_ai_integration.md) for hooks
that guide agents to use `lat show` and auto-expand Lattice IDs in prompts.

## Logging and Observability

All operations log to `.lattice/logs.jsonl` in newline-delimited JSON, capturing
timestamps, operation types, details, duration, and success/failure status. The
`--verbose` flag increases output detail; `--json` switches to structured output.

## Testing Architecture

All tests exercise the public CLI interface (black-box testing). Git operations
go through the `GitOps` trait, enabling test injection—production uses `RealGit`;
tests inject `FakeGit` (in-memory state). Filesystem and SQLite use real
implementations for better edge-case coverage.

See [Appendix: Testing Strategy](appendix_testing_strategy.md) for the complete
testing architecture.

## Chaos Monkey

The `lat chaosmonkey` command performs automated fuzz testing by executing random
sequences of operations (create, modify, delete, link, git operations) until a
system error occurs. This surfaces edge cases and interaction bugs that
deterministic tests miss.

See [Appendix: Chaos Monkey](appendix_chaos_monkey.md) for the complete operation
specification and invariant definitions.

## UI Design

### Color Theme

Lattice uses the Ayu color theme following beads' UI philosophy:

- **Pass (green)**: Success, completion, ready states
- **Warn (yellow)**: Attention needed
- **Fail (red)**: Errors, blocked, critical
- **Accent (blue)**: Navigation, emphasis, links
- **Muted (gray)**: De-emphasized, closed, metadata
- **Bold**: Command names, flag names

Colors support both light and dark terminal modes through adaptive values.

### Design Principles

Following Tufte's principles:

- Maximize data-ink ratio; color only what demands attention
- Use whitespace and position for grouping
- Reserve color for semantic states, not decoration
- Keep help text copy-paste friendly (no color in examples)

## Error Handling

Error handling in Lattice is divided into *expected* and *unexpected* failure
states. Expected errors are problems with user input or external systems the
user manages like the file system, while unexpected errors are internal
invariants and "impossible" code paths. We call these "system errors". This is a
similar distinction to HTTP 400-series vs 500-series error codes.

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
they did something wrong, we should not. Obviously this a judgment call, in gray
areas we can default to the panic option.

### Structured Error Output

All errors include structured information for programmatic handling:

```json
{
  "error_code": "E002",
  "message": "Reference to nonexistent ID",
  "affected_documents": ["LXXXX"],
  "location": {"path": "docs/example.md", "line": 42},
  "suggestion": "Create the target document or correct the ID",
  "fix_command": "lat create docs/target.md"
}
```

The `--json` flag ensures all commands output errors in this structured format.

### Recovery Strategy

For user errors, provide clear guidance on how to fix the problem. For
system errors, log extensively and suggest running `lat check` or
rebuilding the index. Never silently ignore errors.

## Project File Layout

See [Appendix: File Layout](appendix_file_layout.md) for the complete
Rust source file organization.

The implementation lives under `rules_engine/src/lattice/` with the
following module structure:

- `cli/`: Command-line interface and argument parsing
- `index/`: SQLite schema, queries, and reconciliation
- `document/`: Parsing, validation, and manipulation
- `git/`: Git integration and change detection
- `format/`: Markdown formatting and wrapping
- `link/`: Link resolution and reference tracking
- `claim/`: Local claim tracking
- `test/`: Test utilities and fakes
