# Appendix: File Layout

This appendix documents the Rust source directory organization for the Lattice
implementation. See [Lattice Design](lattice_design.md#project-file-layout)
for how this relates to the overall architecture.

All rust file names MUST be a minimum of two words separated by underscores.

## Source Directory Structure

All Rust code lives under `rules_engine/src/lattice/`:

```
rules_engine/src/lattice/
├── cli/
│   └── commands/
├── index/
├── document/
├── git/
├── format/
├── link/
├── claim/
├── lint/
├── id/
├── issue/
├── skill/
├── log/
├── error/
└── test/
```

### Directory Descriptions

**cli/** - Command-line interface layer containing argument parsing, command
dispatch, output formatting, and color theme definitions.

**cli/commands/** - Individual command implementations for each `lat` subcommand
(show, create, update, list, check, format, split, track, generate-ids, ready,
prime, claim, stale, dep, label, close, reopen, edit, chaosmonkey).

**index/** - SQLite database management including schema definitions, migrations,
connection handling, and query implementations for documents, links, labels,
sections, and full-text search. Also contains the reconciliation engine for
syncing git state to the index and counter management for client IDs.

**document/** - Document parsing and serialization including YAML frontmatter
parsing, markdown body handling, frontmatter schema definitions, field validation,
document writing, section extraction, and body manipulation utilities.

**git/** - Git integration layer including command execution, operation traits,
modified file detection, and client configuration management (~/.lattice.toml).

**format/** - Markdown formatting and normalization including text wrapping,
ATX header enforcement, list marker consistency, and overall markdown formatting.

**link/** - Link handling including extraction from markdown, ID resolution to
file paths, and bidirectional reference tracking.

**claim/** - Work claiming system including claim storage (~/.lattice/claims.json),
claim/release logic, and stale claim cleanup.

**lint/** - Validation and linting including the rule execution engine,
error-level rules, warning-level rules, skill-specific validations, and result
reporting.

**id/** - Lattice ID system including ID type definitions, base32 encoding/decoding,
ID generation, and client ID selection logic.

**issue/** - Issue management including type/status/priority enumerations, status
transition state machine, ready work calculation, and dependency graph management.

**skill/** - Skill document integration including symlink creation for .claude/
directory.

**log/** - Structured logging including JSONL output to logs.jsonl and log entry
definitions.

**error/** - Error handling including user vs system error types, human-readable
formatting, and exit code constants.

**test/** - Test utilities and fakes including test environment setup, in-memory
git fake, in-memory filesystem fake, controllable clock fake, test document
factory, and custom assertions.

## Test Directory Structure

```
rules_engine/tests/lattice/
├── cli_tests/
├── index_tests/
├── integration_tests/
└── snapshots/
```

### Test Directory Descriptions

**cli_tests/** - Tests for individual CLI commands (show, create, update, list,
check, format, claim, chaos).

**index_tests/** - Tests for database operations including reconciliation, queries,
and schema migrations.

**integration_tests/** - End-to-end workflow tests and edge case scenarios.

**snapshots/** - Snapshot test files (*.snap) for output verification.

## Runtime Directory Structure

Created in the repository root:

```
<repo>/
├── .lattice/
└── .claude/
    └── skills/
```

### Runtime Directory Descriptions

**.lattice/** - Repository-local Lattice data including the SQLite index database
(index.sqlite), WAL and shared memory files (gitignored), operation logs
(logs.jsonl), optional local config overrides (config.toml), and optional custom
prime output (PRIME.md).

**.claude/skills/** - Symlinks to skill documents for Claude Code integration.
Each symlink points to a skill document in the repository.

## User Configuration

```
~/.lattice.toml    - Global user configuration
~/.lattice/        - User-local Lattice directory
```

### User Configuration Descriptions

**~/.lattice.toml** - Global user configuration including repository-to-client-ID
mappings under [clients] section and optional global defaults under [defaults]
section.

**~/.lattice/** - User-local Lattice directory containing claims.json for tracking
claimed work across all repositories.
