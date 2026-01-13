# Lattice Document System Technical Design

## Executive Summary

Lattice is a unified knowledge base and issue tracking system built on markdown
files stored in git repositories, with SQLite providing an ephemeral index for
query performance. The system prioritizes AI-friendliness through strict
document size limits, rich cross-referencing capabilities, and intelligent
context windowing for document retrieval.

The core innovation of Lattice is treating markdown documents as first-class
database entities while maintaining full git compatibility and human
readability. Documents can exist anywhere in a project hierarchy, colocated
with relevant code, and are identified by their `lattice-id` YAML annotation
rather than filesystem location.

## Design Philosophy

### AI-First Document Architecture

Lattice documents are designed for consumption by AI agents operating under
context window constraints. The 500-line soft limit per document ensures that
any single document can be loaded in full without excessive token consumption.
Users are encouraged to split large documents through the `lat split` command,
and the linter warns when documents exceed this threshold.

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
repairs. See [Appendix: Indexing Strategy](appendix_indexing_strategy.md) for
the detailed reconciliation algorithm.

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
- `name`: Human-readable document name, max 64 lowercase hyphen-separated chars
- `description`: Purpose description for AI context, max 1024 characters

**Context Control Keys:**
- `doc-priority`: Integer affecting sort order in context inclusion (default 0)
- `doc-context-for`: Labels triggering global context inclusion
- `doc-position`: Integer controlling output order relative to main document

**Issue Tracking Keys:**
- `issue-type`: bug/feature/task/epic/chore
- `status`: open/in_progress/blocked/deferred/closed/tombstone/pinned
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
for ID-based linking. Section headers can be annotated with Lattice IDs in
square brackets to enable granular cross-references:

```
# [LJCQ2] Error Handling
```

Links use standard markdown syntax with Lattice IDs replacing URLs:

```
See the [error handling](LJCQ2) document for more information
```

The body text has no hard length limit, but the linter warns at 500 lines.
Documents exceeding this should be split into multiple files using the
`lat split` command.

### Root Documents

Files named with a leading `!` character (e.g., `!master_plan.md`) are
directory root documents. These serve as parent context for all documents in
their directory and provide high-level overviews. The root document's ID
effectively acts as the parent ID for sibling documents, establishing
implicit hierarchy without explicit parent-child relationships.

Root documents are automatically included in the context traversal for
child documents, enabling cascading context inheritance up the filesystem
hierarchy.

## The ID System

### Lattice ID Format

A Lattice ID is a compact, human-typeable identifier consisting of:

1. A literal `L` prefix
2. A document counter (minimum 2 digits, RFC 4648 Base32 encoded)
3. A client identifier (2-5 digits, RFC 4648 Base32 encoded)

Example: `LK1DT` represents document `K1` (decimal 641) from client `DT`.

The Base32 encoding uses the RFC 4648 alphabet: A-Z followed by 2-7. This
provides 32 characters while remaining case-insensitive and avoiding
ambiguous characters like 0/O and 1/I/l.

See [Appendix: ID System](appendix_id_system.md) for the complete ID
generation algorithm and collision handling.

### Client Identification

New clients select a random identifier stored in `~/.lattice.toml`, which
maps git checkout paths to client IDs. The ID length scales with the number
of known clients in the repository:

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
and resumes from there. Counters start at 50 (Base32: `1I`) to ensure all
IDs have at least 2 digits.

The `lat generate-ids` command pre-allocates IDs for document authors. These
IDs are not marked as used immediately; only committed documents consume ID
space. This enables speculative ID generation without waste.

## Context Algorithm

### The Show Command

The `lat show <id>` command is the primary interface for viewing documents.
It supports both "push" context (automatic inclusion) and "pull" context
(explicit requests). The default behavior is configurable.

**Context Budget:** Automatic context uses a character budget (default 5000).
Set to 0 for pure pull behavior: `lat show <id> --context 0`.

**AI Mode:** The `--ai` flag optimizes output for AI consumption with no
automatic context by default: `lat show <id> --ai`.

**Intent-Based Context:** The `--intent` flag selects task-appropriate
context: `lat show <id> --intent=bug-fix` includes related bugs, error docs,
and test cases.

**Task Briefing:** The `--brief` flag provides comprehensive task-start
context with a larger budget: `lat show <issue-id> --brief`.

**Incremental Loading:** Load only what you need:
- `--peek`: Just YAML frontmatter
- `--sections`: List sections without content
- `--section "Name"`: Load specific section

See [Appendix: Context Retrieval](appendix_context_retrieval.md) for the
complete specification of context modes, intents, and loading options.

See [Appendix: Push vs Pull Analysis](appendix_push_pull_analysis.md) for
design rationale on automatic vs explicit context.

### Context Sources

Documents are considered for inclusion in the following priority order:

1. **doc-context-for matches**: Documents declaring this document's labels
2. **Body links**: Documents or sections linked in the document body
3. **Directory roots**: Root documents from this location to repository root
4. **Frontmatter links**: Documents linked in YAML frontmatter

Within each category, documents sort by their `doc-priority` value (higher
first, default 0). The `doc-position` key controls final output ordering,
with negative values appearing before the main document and positive values
appearing after.

### Output Format

Context documents are separated by two newlines, with each document's name
rendered as a markdown level-1 header. YAML frontmatter is excluded from
context documents but included for the primary document. For issues, the
frontmatter is rendered as human-readable metadata following beads'
`bd show` format.

The References section (default 500 characters, configurable via
`--references N`) lists documents that qualified for inclusion but didn't
fit the budget, showing their names, descriptions, and IDs.

## Linking System

### Link Types

Lattice supports three link target types:

1. **Document links**: Reference a complete document by its ID
2. **Section links**: Reference a specific section within a document
3. **Placeholder links**: Use `LATTICE` as target for auto-completion

Document and section links share the same ID namespace. When the `lat show`
command encounters a section ID, it includes only that section and its
subsections, not the entire parent document.

### Link Resolution

The `lat annotate` command resolves placeholder links (`LATTICE`) by
performing case-insensitive, whitespace-normalized substring matching
against section headers. If exactly one match exists, the placeholder is
replaced with the corresponding ID. Ambiguous matches generate warnings.

This command also assigns IDs to unmarked section headers. By default, only
level-1 headers receive IDs, but flags control deeper annotation.

See [Appendix: Linking System](appendix_linking_system.md) for the complete
resolution algorithm and edge cases.

### Bidirectional References

The index maintains a reverse reference map enabling queries like "what
documents link to this one?" This powers features like impact analysis
when modifying or deleting documents.

See [Appendix: Hierarchy and Relationships](appendix_hierarchy_relationships.md)
for relationship query commands including `lat links-from`, `lat links-to`,
`lat path`, `lat orphans`, and `lat impact`.

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
open -> in_progress -> closed
  |         |            ^
  |         v            |
  +---> blocked ------>--+
  |         |
  v         v
deferred  (back to open when unblocked)
```

The `tombstone` status represents deleted issues that should not be
resurrected. The `pinned` status indicates permanent open items like
maintenance hooks.

See [Appendix: Issue Tracking](appendix_issue_tracking.md) for the complete
state machine and transition rules.

### CLI Commands

The issue CLI follows beads' design philosophy, adapted for Lattice's
filesystem-centric model. Key differences from beads:

- Issues require `--path` on creation to specify filesystem location
- No explicit parent/child relationships; hierarchy comes from directories
- The `name` field replaces beads' `title` concept
- Full document text is stored, not just description fields

See [Appendix: CLI Structure](appendix_cli_structure.md) for the complete
command reference and [Appendix: Beads Analysis](appendix_beads_analysis.md)
for detailed analysis of beads behaviors to preserve.

### Session Management

Following beads' successful pattern, Lattice provides session-start context
via the `lat prime` command. This command outputs workflow instructions,
current working issue, recent changes, and ready work in a token-efficient
format (~1-2k characters).

Editor hooks (Claude Code, Cursor, Aider) call `lat prime` automatically on
session start to ensure AI agents have consistent workflow context.

Change awareness uses date-based filters (`--updated-after`, `--created-after`)
and the `lat stale` command for finding issues not updated recently. The
`lat changes` command shows what has changed since a date or git commit.

See [Appendix: Session Management](appendix_session_management.md) for the
complete session management specification.

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

The formatter attempts to auto-correct issues identified by `lat check`
when a deterministic fix exists.

### The Split Command

The `lat split` command takes a large document and divides it by top-level
sections:

1. The first text block and first section become a root document (`!name.md`)
2. Each subsequent section becomes a standalone document
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

The schema version is tracked for migration support. All tables include
`indexed_at` timestamps for staleness detection.

Beyond keyword search, Lattice optionally supports semantic search using
vector embeddings. This enables queries like "find documents about error
handling" even when documents don't contain those exact words.

See [Appendix: Semantic Search](appendix_semantic_search.md) for the
embedding model options, vector storage strategies, and Rust crate
recommendations.

### Reconciliation Strategy

Index reconciliation uses git metadata to determine staleness:

1. Query git for files modified since last index update
2. Re-parse and re-index modified documents
3. Remove index entries for deleted documents
4. Update the index watermark commit hash

If git state is unclear (detached HEAD, uncommitted changes), the system
falls back to full rebuild. The index stores the commit hash of its last
known-good state for comparison.

See [Appendix: Indexing Strategy](appendix_indexing_strategy.md) for the
complete reconciliation algorithm and [Appendix: Indexing Performance](appendix_indexing_performance.md)
for SQLite tuning and benchmarking guidance.

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

### Conflict Handling

When `lat check` runs in pre-commit hooks, it detects potential ID
conflicts from parallel contributors. The system provides guidance for
renumbering IDs in a contributor's first commit before pushing.

See [Appendix: Git Integration](appendix_git_integration.md) for the
complete git interaction specification.

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

### Log Rotation

Log files should be periodically rotated by external tooling. Lattice
does not implement automatic rotation, as this would require daemon-like
behavior.

## Testing Architecture

### Black Box Testing

All tests exercise the public CLI interface rather than internal APIs.
This ensures tests validate user-facing behavior and enables safe
refactoring of internals without test rewrites.

### Fake Implementations

External dependencies (git, filesystem) are replaced with in-memory fakes
for test performance. Tests must run fast; a slow test suite is worse
than no tests because it discourages running them.

The fake git implementation maintains an in-memory commit graph and file
tree, supporting the git operations Lattice uses (status, diff, log) while
running entirely in memory.

See [Appendix: Testing Strategy](appendix_testing_strategy.md) for the
complete testing architecture.

### Test Categories

Tests are organized by category:

- **Happy path**: Normal operation with valid inputs
- **User errors**: Invalid inputs, malformed documents
- **System errors**: Index corruption, git failures
- **Edge cases**: Empty repositories, single documents, maximum sizes

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
- **Warn (yellow)**: Attention needed, in-progress
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

### Recovery Strategy

For user errors, provide clear guidance on how to fix the issue. For
system errors, log extensively and suggest running `lat check` or
rebuilding the index. Never silently ignore errors.

### Defensive Programming

Internal assertions verify invariants. When assertions fail, they generate
system errors with diagnostic information rather than continuing with
potentially corrupted state.

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
- `context/`: Context algorithm implementation
- `test/`: Test utilities and fakes

## Follow-Up Work

1. ~~**Beads CLI Analysis**~~: Complete. See
   [Appendix: Beads Analysis](appendix_beads_analysis.md) for detailed
   findings on `bd show`, `bd ready`, and `bd list` behaviors to preserve.

2. ~~**Indexing Performance Research**~~: Complete. See
   [Appendix: Indexing Performance](appendix_indexing_performance.md) for
   SQLite best practices including WAL mode, PRAGMA configuration,
   FTS5 optimization, and benchmarking targets for 10,000+ documents.

3. **Markdown Linter Integration**: Evaluate existing markdown linters
   (markdownlint, remark-lint) for potential integration rather than
   implementing lint rules from scratch. Determine which rules align
   with Claude's Skill best practices.

4. **Git Edge Cases**: Document and test handling for git edge cases:
   shallow clones, worktrees, submodules, detached HEAD states, in-progress
   rebases, and partial clones. Define explicit behavior for each scenario.

5. **Context Algorithm Optimization**: Profile the context algorithm with
   large document sets and complex link graphs. Research graph traversal
   optimizations and caching strategies for repeated `lat show` operations
   on related documents.
