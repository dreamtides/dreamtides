# Lattice Document System Technical Design

## Executive Summary

Lattice is a unified knowledge base and issue tracking system built on markdown
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
state. This logging enables post-hoc debugging when issues arise.

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
- `name`: Human-readable document name, max 64 lowercase hyphen-separated chars
- `description`: Purpose summary for AI context, max 1024 characters (optional for
  issues, recommended for knowledge base documents)

**Issue Tracking Keys:**
- `issue-type`: bug/feature/task/epic/chore
- `status`: open/blocked/deferred/closed/tombstone/pinned
- `priority`: 0-4 (0 highest)
- `labels`: List of arbitrary string labels
- `blocking`: List of issue IDs with hard dependencies on this issue
- `blocked-by`: List of issue IDs this issue depends on
- `discovered-from`: List of parent issues from which this was discovered
- `created-at`, `updated-at`, `closed-at`: ISO 8601 timestamps

**Skill Integration Keys:**
- `skill`: Boolean enabling Claude Skill generation

See [Appendix: Issue Tracking](appendix_issue_tracking.md) for the complete
issue lifecycle state machine.

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
The `00_` prefix marks the highest priority document, serving as the directory
root that provides parent context for all other documents in that directory.

The `lat fmt` command automatically populates the `parent-id` field in each
document's frontmatter based on the directory's root document. This makes
hierarchy explicit without requiring manual parent specification. Documents
without a root document in their directory have no `parent-id`.

Higher-numbered prefixes (`01_`, `02_`, etc.) indicate progressively lower
priority. Commands like `lat show` and `lat overview` use these prefixes when
selecting which related documents to highlight.

## The ID System

### Lattice ID Format

A Lattice ID is a compact, human-typeable identifier consisting of:

1. A literal `L` prefix
2. A document counter (minimum 2 digits, RFC 4648 Base32 encoded)
3. A client identifier (2-5 digits, RFC 4648 Base32 encoded)

Example: `LK3DT` represents document `K3` (decimal 675) from client `DT`.

The Base32 encoding uses the RFC 4648 alphabet: A-Z followed by 2-7. This
avoids ambiguous characters like 0/O and 1/I.

See [Appendix: ID System](appendix_id_system.md) for the complete ID
generation algorithm and collision handling.

### Client Identification

New clients select a random identifier and store it in `~/.lattice.toml`, which
maps git checkout paths to client IDs. The ID length scales with the number of
known clients in the repository:

- 2 digits: Up to 16 clients (1024 possible IDs)
- 3 digits: 17-64 clients (32768 possible IDs)
- 4 digits: 65-256 clients (1048576 possible IDs)
- 5 digits: 257+ clients (33554432 possible IDs)

Collision detection occurs during `lat check` and is specifically enforced
for new contributors' first commits. The `lat check` command identifies any
duplicate Lattice IDs across the repository.

### Document Counter

The document counter increments atomically from SQLite when new IDs are
requested. To prevent counter reset on repository re-clone, the system
queries existing documents to find the highest ID for the current client
and resumes from there. Counters start at 50 (Base32: `BS`) to ensure all
IDs have at least 2 digits.

The `lat generate-ids` command pre-allocates IDs for document authors. These
IDs are not marked as used immediately; only committed documents consume ID
space. This enables speculative ID generation without waste.

## Command Overview

Lattice provides commands for document creation, viewing, and work management,
designed for compatibility with beads (`bd`) while supporting Lattice's
filesystem-centric model.

### Workflow Commands

Commands for viewing documents and managing work progress.

**lat show** - Displays document details following `bd show` format. Supports
single or multiple documents, with `--json`, `--short`, and `--refs` options.
Default output includes parent, dependencies, blocking issues, and related
documents—providing full context for AI agents in a single call.

**lat ready** - Shows work available to start: open issues with no blockers
that are not claimed. Supports `--parent` for directory filtering, `--pretty`
for visual tree display, and `--json` for full issue details.

**lat overview** - Provides repository-level context for AI agents. Shows the
most critical documents based on view frequency, recency, and priority. Supports
`--limit`, `--json`, and various filtering options. Tracks local view counts
in `.lattice/views.json` to surface frequently-referenced documents.

**lat prime** - Outputs AI-optimized workflow context including recommended
link authoring format (shorthand `[text](ID)` links that `lat fmt` expands).
Supports custom checklist via `.lattice/config.toml`.

**lat claim** - Marks issues as locally in progress on the current machine.
Claims are stored in `~/.lattice/claims.json`, not in markdown files. Supports
atomic updates across multiple worktrees and automatic release on status change.

See [Appendix: Workflow](appendix_workflow.md) for complete command specifications,
output formats, and claiming behavior, and
[Appendix: Overview Command](appendix_overview.md) for the ranking algorithm.

### Issue and Document Management

Commands for creating and modifying issues and documents.

**lat create** - Creates new documents with `lat create <path/to/doc.md>
[options]`. The path specifies both directory location and filename,
establishing the document's position in the hierarchy. Supports issue type,
priority, labels, and dependencies at creation time.

**lat update** - Modifies existing documents with `lat update <id> [id...]
[options]`. Supports changing status, priority, type, and managing labels.
Can update multiple issues atomically, useful for bulk operations like marking
dependencies as blocked or changing priority across related issues.

**lat close** - Marks issues as closed, accepting single or multiple lattice IDs.
Automatically releases any local claims and sets the `closed-at` timestamp.
Supports `--reason` for documenting why the issue was closed.

See [Appendix: Issue Tracking](appendix_issue_tracking.md) for the complete
issue lifecycle and [Appendix: CLI Structure](appendix_cli_structure.md) for
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

## Issue Tracking

### Integration with Knowledge Base

Issues and knowledge base documents share a unified ID space, enabling
seamless cross-referencing. An issue can link to design documents, and
design documents can reference issues that track their implementation.

The primary organizational mechanism is the filesystem hierarchy rather
than explicit parent-child relationships. All issues in a directory are
implicitly siblings, with the directory's root document acting as their
parent or "epic." This replaces beads' explicit epic/child model.

### Issue Lifecycle

Issue status transitions follow a state machine:

```
open -> closed
  |       ^
  |       |
  +---> blocked -------+
  |
  v
deferred  (back to open when unblocked)
```

The `tombstone` status represents deleted issues that should not be
resurrected. The `pinned` status indicates permanent open items.

There is no "in_progress" status in Lattice. Instead, the `lat claim`
command tracks which machine is working on an issue locally, without
modifying the issue file.

See [Appendix: Issue Tracking](appendix_issue_tracking.md) for the complete
state machine and transition rules.

### CLI Commands

The issue CLI follows beads' design philosophy, adapted for Lattice's
filesystem-centric model. Key differences from beads:

- Issues require a path on creation to specify filesystem location
- No explicit parent/child relationships; hierarchy comes from directories
- The `name` field replaces beads' `title` concept
- No sync command; Lattice never performs git push operations

See [Appendix: CLI Structure](appendix_cli_structure.md) for the complete
command reference and [Appendix: Beads Analysis](appendix_beads_analysis.md)
for detailed analysis of beads behaviors to preserve.

## Linter and Formatter

### The Check Command

The `lat check` command validates documents and repository state:

**Error-level issues (prevent operations):**
- Duplicate Lattice IDs
- References to nonexistent IDs
- Invalid YAML frontmatter keys
- Missing required fields for issue documents
- Invalid status/type/priority values
- Circular blocking dependencies

**Warning-level issues:**
- Document exceeds 500 lines
- Missing `name` or `description` for knowledge base documents
- Markdown lint issues (inconsistent headers, bare URLs, etc.)
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

The formatter attempts to auto-correct issues identified by `lat check`
when a deterministic fix exists.

### The Split Command

The `lat split` command takes a large document and divides it by top-level
sections:

1. The first text block and first section become a root document (`00_name.md`)
2. Each subsequent section becomes a standalone document with a numeric prefix
3. The root document receives links to all child documents
4. All new documents receive generated Lattice IDs

This enables progressive breakdown of monolithic documents into
AI-friendly sizes.

## Index Architecture

### SQLite Schema

The index stores document metadata, link relationships, and search indices
in SQLite. Key tables include:

- `documents`: Core document metadata (id, path, frontmatter fields)
- `links`: Source->target relationships with link types
- `labels`: Many-to-many document-label relationships
- `fts_content`: Full-text search index on document body
- `client_counters`: Per-client document counter state
- `directory_roots`: Precomputed root document chain for hierarchy queries
- `content_cache`: Optional L2 cache for document body content

The schema version is tracked for migration support. All tables include
`indexed_at` timestamps for staleness detection.

### Reconciliation Strategy

Index reconciliation uses git metadata to determine staleness:

1. Query git for files modified since last index update
2. Re-parse and re-index modified documents
3. Remove index entries for deleted documents
4. Update the index watermark commit hash

If git state is unclear (detached HEAD, uncommitted changes), the system
falls back to full rebuild. The index stores the commit hash of its last
known-good state for comparison.

See [Appendix: Indexing](appendix_indexing.md) for the complete schema,
reconciliation algorithm, and performance tuning.

### Index Location

The index lives in `.lattice/index.sqlite` within the repository root.
This directory also contains `logs.jsonl` for operation logging. The
entire `.lattice` directory should be gitignored as it contains ephemeral
cache data.

## Git Integration

### Change Detection

Lattice uses `git diff --name-only` and `git status` to identify changed
files since the last index update. The index stores the HEAD commit hash
at the time of its last update; comparing against current HEAD reveals
what needs re-indexing.

### Document Discovery

Documents are discovered through git's tracked file list rather than
filesystem traversal. This ensures gitignored files are excluded and
provides consistent behavior across operations.

### No Push Operations

Lattice never performs git push or sync operations. This design supports
multi-agent workflows where agents work in isolated git worktrees and a
coordinator manages merging and synchronization externally. Push operations
require explicit user or coordinator control.

### Conflict Handling

`lat check` detects potential ID conflicts from parallel contributors.

See [Appendix: Git Integration](appendix_git_integration.md) for the
complete git interaction specification and
[Appendix: Git Edge Cases](appendix_git_edge_cases.md) for behavior in
non-standard repository configurations (shallow clones, worktrees,
submodules, etc.).

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

## Logging and Observability

### Log Format

All operations log to `.lattice/logs.jsonl` in newline-delimited JSON.
Each entry includes:

- Timestamp (ISO 8601 with microseconds)
- Operation type (user_command, git_operation, sqlite_query, observation)
- Operation details (command args, query text, observation text)
- Duration (milliseconds)
- Success/failure status
- Error details (if applicable)

### Verbosity Control

The `--verbose` flag increases output detail, showing operations that
would normally be silent. The `--json` flag switches output format for
programmatic consumption.

## Testing Architecture

### Black Box Testing

All tests exercise the public CLI interface rather than internal APIs.
This ensures tests validate user-facing behavior and enables safe
refactoring of internals without test rewrites.

### GitOps Trait

All git operations go through the `GitOps` trait, enabling test injection.
Production uses `RealGit` (shells out to git); tests inject `FakeGit`
(in-memory state). The filesystem and SQLite use real implementations—they're
fast enough and catch more edge cases than fakes would.

See [Appendix: Testing Strategy](appendix_testing_strategy.md) for the
complete testing architecture.

## Chaos Monkey

### Purpose

The `lat chaosmonkey` command performs automated fuzz testing by executing
random sequences of Lattice operations until a system error occurs. This
surfaces edge cases and interaction bugs that deterministic tests miss.

### Operation Types

The chaos monkey executes high-level concepts rather than raw CLI commands:

- Create document with random content
- Modify document content
- Add/remove links
- Change document metadata
- Move documents between directories
- Delete documents
- Perform git operations (commit, branch, merge)

### Error Reporting

When a system error occurs, the chaos monkey outputs:

- The operation sequence that triggered the error
- Current repository state
- Relevant log entries
- A minimal reproduction case when determinable

See [Appendix: Chaos Monkey](appendix_chaos_monkey.md) for the complete
operation specification.

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

### Error Categories

**User Errors (exit code 2+):**
- Invalid document syntax
- Missing required fields
- References to nonexistent IDs
- Invalid command arguments

**System Errors (exit code 1, "System Error" output):**
- Index corruption
- Git operation failures
- File permission issues
- Unexpected internal states

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

For user errors, provide clear guidance on how to fix the issue. For
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
