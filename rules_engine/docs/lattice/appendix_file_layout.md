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
├── task/
├── skill/
├── log/
├── error/
└── test/
```

### Directory Descriptions

**cli/** - Command-line interface layer containing argument parsing, command
dispatch, output formatting, and color theme definitions.

**cli/commands/** - Individual command implementations for each `lat` subcommand
(show, create, update, close, reopen, list, ready, overview, prime, claim,
check, fmt, split, mv, track, generate-ids, search, stale, blocked, changes,
stats, tree, roots, children, dep, label, links-from, links-to, path, orphans,
impact, setup, completion, edit, chaosmonkey).

**index/** - SQLite database management including schema definitions, migrations,
connection handling, and query implementations for documents, links, labels,
full-text search, and view tracking (for `lat overview`). Also contains the
reconciliation engine for syncing git state to the index, counter management
for client IDs, and L2 content caching.

**document/** - Document parsing and serialization including YAML frontmatter
parsing, markdown body handling, frontmatter schema definitions, field validation,
document writing, and body manipulation utilities.

**git/** - Git integration layer including command execution, operation traits,
modified file detection, and client configuration management (~/.lattice.toml).

**format/** - Markdown formatting and normalization including text wrapping,
ATX header enforcement, list marker consistency, and overall markdown formatting.

**link/** - Link handling including extraction from markdown, ID resolution to
file paths, and bidirectional reference tracking.

**claim/** - Work claiming system using atomic file operations. Claims stored as
individual files in `~/.lattice/claims/<repo-hash>/` for concurrent safety.
Includes claim/release logic and stale claim cleanup.

**lint/** - Validation and linting including the rule execution engine,
error-level rules, warning-level rules, skill-specific validations, and result
reporting.

**id/** - Lattice ID system including ID type definitions, base32 encoding/decoding,
ID generation, and client ID selection logic.

**task/** - Task management including type/priority enumerations, state
computation from filesystem paths, ready work calculation, and dependency graph
management.

**skill/** - Skill document integration including symlink creation for .claude/
directory.

**log/** - Structured logging including JSONL output to logs.jsonl and log entry
definitions.

**error/** - Error handling including user vs system error types, human-readable
formatting, and exit code constants.

**test/** - Test utilities including test environment setup, `FakeGit`
implementation of the `GitOps` trait, and custom assertions.

## Test Directory Structure

```
tests/lattice/
├── commands/
├── index/
└── integration/
```

**commands/** - Per-command tests (create, show, list, etc.)

**index/** - Reconciliation and query tests

**integration/** - Multi-command workflow tests

## Runtime Directory Structure

Created in the repository root:

```
<repo>/
├── .lattice/
└── .claude/
    └── skills/
```

### Directory Descriptions

**.lattice/** - Repository-local Lattice data including the SQLite index database
(index.sqlite with view tracking), WAL and shared memory files (gitignored),
operation logs (logs.jsonl), optional local config overrides (config.toml),
and optional custom prime output (PRIME.md).

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

**~/.lattice/** - User-local Lattice directory containing per-repository claim
subdirectories (`claims/<repo-hash>/`). Each claim is a separate file for
atomic create/delete operations (concurrent-safe).

## Detailed Rust File Breakdown

This section provides suggested file names for each module. All file names use
at least two words separated by underscores. Code goes in named files, not in
`mod.rs` (which contains only module declarations).

**Flexibility note:** Implementers should use judgment when creating files.
These suggestions provide a starting point, but additional files may be needed
as complexity emerges. When in doubt, prefer smaller focused files over large
multi-purpose ones. Files can be split further or combined based on actual
implementation needs.

### cli/

```
cli/
├── mod.rs                  # Module declarations only
├── argument_parser.rs      # Clap argument definitions and parsing
├── command_dispatch.rs     # Routes parsed args to command implementations
├── output_format.rs        # Text/JSON/pretty output helpers
├── color_theme.rs          # Ayu color theme definitions, terminal detection
├── global_options.rs       # --json, --verbose, --quiet handling
└── commands/
    └── mod.rs              # Module declarations only
```

### cli/commands/

Each command gets its own file. Commands may be split into multiple files if
complexity warrants (e.g., `show_format.rs` for display logic).

```
commands/
├── mod.rs                  # Module declarations only
├── show_command/           # lat show - display document details
│   ├── mod.rs              # Module declarations only
│   ├── show_executor.rs    # Entry point, orchestration, template composition
│   ├── document_formatter.rs # Task and KB formatting, references, dependencies
│   └── output_formats.rs   # --short, --refs, and --json output modes
├── create_command.rs       # lat create - new document creation
├── update_command.rs       # lat update - modify existing documents
├── close_command.rs        # lat close - mark tasks closed
├── reopen_command.rs       # lat reopen - reopen closed tasks
├── prune_command.rs        # lat prune - permanently delete closed tasks
├── list_command/           # lat list - search and filter documents
│   ├── mod.rs              # Module declarations only
│   ├── list_executor.rs    # Entry point and orchestration
│   ├── filter_builder.rs   # Parse all filter options, build SQL queries
│   └── list_output.rs      # Rich/compact/oneline formatting, sort ordering
├── ready_command/          # lat ready - show available work
│   ├── mod.rs              # Module declarations only
│   ├── ready_executor.rs   # Entry point and orchestration
│   ├── ready_filter.rs     # Ready criteria, blocker resolution, claim filtering
│   └── ready_output.rs     # Sort policies, --pretty tree, --json output
├── overview_command.rs     # lat overview - repository-level context
├── prime_command.rs        # lat prime - AI workflow context
├── claim_command.rs        # lat claim - local work tracking
├── check_command/          # lat check - validation and linting
│   ├── mod.rs              # Module declarations only
│   ├── check_executor.rs   # Entry point, document validation orchestration
│   ├── check_fixes.rs      # --staged-only filtering, --fix application, result aggregation
│   └── check_output.rs     # Text and --json error output formatting
├── fmt_command.rs          # lat fmt - formatting and link normalization
├── split_command.rs        # lat split - divide large documents
├── mv_command.rs           # lat mv - move document to new location
├── track_command.rs        # lat track - add tracking to existing files
├── generate_ids.rs         # lat generate-ids - pre-allocate IDs
├── edit_command.rs         # lat edit - open in editor
├── search_command.rs       # lat search - full-text search
├── stale_command.rs        # lat stale - find stale tasks
├── blocked_command.rs      # lat blocked - show blocked tasks
├── changes_command.rs      # lat changes - show recent changes
├── stats_command.rs        # lat stats - project statistics and health
├── tree_command.rs         # lat tree - directory structure display
├── roots_command.rs        # lat roots - list root documents
├── children_command.rs     # lat children - list directory contents
├── dep_command.rs          # lat dep - dependency management (add, tree)
├── label_command.rs        # lat label - label management (add, remove, list)
├── links_from.rs           # lat links-from - outgoing links
├── links_to.rs             # lat links-to - incoming links (backlinks)
├── path_command.rs         # lat path - find path between documents
├── orphans_command.rs      # lat orphans - find unlinked documents
├── impact_command.rs       # lat impact - change impact analysis
├── setup_command.rs        # lat setup - install hooks and config
├── completion_command.rs   # lat completion - generate shell completions
└── chaos_monkey.rs         # lat chaosmonkey - fuzz testing
```

### index/

```
index/
├── mod.rs                  # Module declarations only
├── schema_definition.rs    # SQLite table definitions, CREATE statements
├── connection_pool.rs      # Connection handling, PRAGMA configuration
├── document_queries.rs     # Document CRUD operations (or document_queries/ directory)
├── link_queries.rs         # Link table queries (outgoing, incoming)
├── label_queries.rs        # Label queries and management
├── fulltext_search.rs      # FTS5 configuration and search queries
├── reconciliation/          # Sync git state to index (fast/incremental/full)
│   ├── mod.rs               # Module declarations only
│   ├── reconciliation_coordinator.rs # Entry point, strategy selection
│   ├── sync_strategies.rs   # Fast path, incremental sync, full rebuild
│   └── change_detection.rs  # Change detection, schema migration
├── client_counters.rs      # Per-client document counter management
├── directory_roots.rs      # Precomputed hierarchy queries
├── content_cache.rs        # L2 content cache management
├── view_tracking.rs        # Document view counts (views table in SQLite index)
└── index_metadata.rs       # Schema version, last commit tracking
```

### document/

```
document/
├── mod.rs                  # Module declarations only
├── frontmatter_parser.rs   # YAML frontmatter extraction and parsing
├── frontmatter_schema.rs   # Field definitions, allowed keys, validation rules
├── field_validation.rs     # Type checking, value validation for fields
├── markdown_body.rs        # Body content handling and manipulation
├── document_writer.rs      # Serialize documents back to files
├── document_reader.rs      # Load and parse complete documents
└── body_manipulation.rs    # Utilities for modifying document body content
```

### git/

```
git/
├── mod.rs                  # Module declarations only
├── git_ops.rs              # GitOps trait definition
├── real_git.rs             # Production implementation (shells out to git)
├── modified_files.rs       # Change detection via git diff/status
├── client_config.rs        # ~/.lattice.toml management
├── repo_detection.rs       # Detect repo configuration (shallow, sparse, etc.)
├── edge_cases.rs           # Handling for worktrees, submodules, bare repos (or edge_cases/ directory)
└── conflict_detection.rs   # Detect merge conflicts, in-progress operations
```

### format/

```
format/
├── mod.rs                  # Module declarations only
├── text_wrapper.rs         # Line wrapping at configurable width
├── header_normalizer.rs    # ATX header enforcement, blank line rules
├── list_normalizer.rs      # List marker consistency (dashes)
├── whitespace_cleaner.rs   # Trailing whitespace, multiple blank lines
└── markdown_formatter.rs   # Orchestrates all formatting operations
```

### link/

```
link/
├── mod.rs                  # Module declarations only
├── link_extractor.rs       # Parse markdown to find links
├── link_resolver.rs        # Resolve IDs to file paths
├── link_normalization/     # Add missing fragments, update stale paths
│   ├── mod.rs              # Module declarations only
│   ├── normalization_executor.rs # Entry point, orchestration
│   ├── link_analysis.rs    # Fragment validation, path resolution
│   └── link_transforms.rs  # Stale path updates, fragment injection, shorthand expansion
├── reference_tracker.rs    # Bidirectional reference queries
└── frontmatter_links.rs    # Extract links from frontmatter fields
```

### claim/

```
claim/
├── mod.rs                  # Module declarations only
├── claim_storage.rs        # File-based claim persistence (~/.lattice/claims/)
├── claim_operations.rs     # Claim, release, check operations
└── stale_cleanup.rs        # Garbage collection for old/invalid claims
```

### lint/

```
lint/
├── mod.rs                  # Module declarations only
├── rule_engine.rs          # Execute rules, collect results
├── error_rules.rs          # E001-E010 (blocking errors)
├── warning_rules.rs        # W001-W016 (or warning_rules/ directory with per-rule files)
├── skill_rules.rs          # S001-S003 (skill document validation)
├── result_reporter.rs      # Format and output lint results
└── autofix_engine.rs       # Apply automatic fixes where possible
```

### id/

```
id/
├── mod.rs                  # Module declarations only
├── lattice_id.rs           # LatticeId type definition and parsing
├── base32_encoding.rs      # RFC 4648 Base32 encode/decode
├── id_generator.rs         # Generate new IDs with counter management
└── client_selection.rs     # Client ID assignment and length rules
```

### task/

```
task/
├── mod.rs                  # Module declarations only
├── task_types.rs           # TaskType enum (bug, feature, task, epic, chore)
├── task_state.rs           # State computation from path (open/blocked/closed)
├── task_priority.rs        # Priority levels (P0-P4)
├── ready_calculator.rs     # Determine which tasks are ready for work
├── dependency_graph.rs     # Build and query blocking relationships
├── closed_directory.rs     # .closed/ directory path utilities
└── template_composer.rs    # Compose context/acceptance from ancestors
```

### skill/

```
skill/
├── mod.rs                  # Module declarations only
├── symlink_manager.rs      # Create/remove .claude/skills/ symlinks
└── skill_validation.rs     # Validate skill-specific requirements
```

### log/

```
log/
├── mod.rs                  # Module declarations only
├── jsonl_writer.rs         # Write structured logs to logs.jsonl
├── log_entry.rs            # Log entry type definitions
└── log_reader.rs           # Read and parse log files (for diagnostics)
```

### error/

See [Appendix: Error Handling](appendix_error_handling.md) for the error
philosophy, taxonomy, and patterns.

```
error/
├── mod.rs                  # Module declarations only
├── error_types.rs          # UserError vs SystemError distinction
├── error_formatting.rs     # Human-readable error message generation
├── exit_codes.rs           # Exit code constants (0-4)
└── structured_output.rs    # JSON error format for --json mode
```

### test/

```
test/
├── mod.rs                  # Module declarations only
├── test_environment.rs     # TestEnv setup, temp directories
├── fake_git.rs             # FakeGit implementation of GitOps trait
├── test_assertions.rs      # Custom assertion helpers
└── test_fixtures.rs        # Common test document templates
```

## Test Directory Detailed Layout

```
tests/lattice/
├── mod.rs                     # Module declarations only
├── commands/
│   ├── mod.rs                 # Module declarations only
│   ├── create_tests.rs
│   ├── show_tests.rs          # (or show_tests/ directory if many test cases)
│   ├── update_tests.rs
│   ├── list_tests.rs
│   ├── close_tests.rs
│   ├── ready_tests.rs
│   ├── check_tests.rs
│   ├── fmt_tests.rs
│   ├── claim_tests.rs
│   ├── overview_tests.rs
│   ├── search_tests.rs
│   ├── label_tests.rs
│   ├── dep_tests.rs
│   ├── stats_tests.rs
│   └── mv_tests.rs
├── index/
│   ├── mod.rs                 # Module declarations only
│   ├── reconciliation_tests.rs
│   ├── query_tests.rs
│   └── fts_tests.rs
└── integration/
    ├── mod.rs                 # Module declarations only
    ├── workflow_tests.rs      # Multi-command sequences
    ├── git_edge_tests.rs      # Shallow clones, worktrees, etc.
    └── template_tests.rs      # Template composition workflows
```

## Implementation Guidance

**When to create additional files:**
- If a file exceeds ~300 lines, consider splitting
- If a module has clearly separable concerns, split them
- If tests for a module grow large, create focused test files

**When to combine files:**
- If two small files are always imported together
- If splitting would create excessive module boilerplate
- If the split would obscure the logical grouping

**Common patterns:**
- Query modules often benefit from splitting by table (documents, links, labels)
- Command implementations may need helper files for complex formatting
- Validation rules naturally group by severity level

**File naming conventions:**
- Use noun phrases for data types: `task_types.rs`, `lattice_id.rs`
- Use verb phrases for operations: `claim_operations.rs`, `link_resolver.rs`
- Use `_tests.rs` suffix for test files
- Use descriptive names over generic ones: `ready_calculator.rs` not `utils.rs`
