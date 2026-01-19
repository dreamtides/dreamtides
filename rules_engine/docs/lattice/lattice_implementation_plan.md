# Lattice Implementation Plan

## Workflow and Coordination

### Parallel Development Strategy

- **API-First Development:** Define all public traits, types, and function
  signatures before implementation. Each module's `mod.rs` declares submodules;
  implementation files contain only their logic.
- **Interface Contracts:** Each feature section below specifies the traits and
  types it exports. Downstream features depend only on these interfaces, not
  implementation details.
- **File Ownership:** Each source file should have a single owner during active
  development. The file layout in [Appendix: File
  Layout](appendix_file_layout.md) maps directly to ownership boundaries.
- **Test Isolation:** Tests live in `tests/lattice/` mirroring source structure.
  Test authors own their test files independently.

### Merge Conflict Avoidance

- **Vertical Slicing:** Features are structured so one developer implements a
  complete vertical slice (e.g., one command from CLI to index queries) rather
  than horizontal layers.
- **Shared Code Protocol:** Changes to shared modules (`error/`, `log/`,
  `cli/output_format.rs`) require team notification. Prefer extending over
  modifying.
- **mod.rs Convention:** `mod.rs` files contain only module declarations. No
  logic in `mod.rs` = no conflicts on module additions.
- **Explicit Dependencies:** Each work item lists exact files touched. Two work
  items touching the same file cannot run in parallel.

### Quality From Day One

Every code change must satisfy [Appendix: Code Review](appendix_code_review.md)
checklist:
- Error classification: `LatticeError` for user errors, `panic!` for system
  errors
- Logging: All git/SQLite/file operations logged via `tracing`
- Atomicity: Multi-step operations use temp file + rename
- Concurrency: Safe under concurrent `lat` invocations
- Performance: No O(n²) on document count
- Tests: Happy path + each error case

### Development Order

Work proceeds in dependency order. Features with no upstream dependencies can
start immediately. The critical path is:

```
Foundation → Git Layer → Document Model → Index Core → CLI Framework
     ↓            ↓            ↓               ↓            ↓
     └────────────┴────────────┴───────────────┴────────────┘
                               ↓
                    First Working Command (lat show)
                               ↓
              Remaining features (parallel where possible)
```

---

## Feature: Foundation [DONE]

**Goal:** Core types, error handling, logging, configuration—everything needed
before any module can function.

**Depends on:** Nothing (start immediately)

**Exports:**
- `LatticeId` type with parsing/validation
- `LatticeError` enum with all variants
- `LatticeResult<T>` type alias
- Logging macros and JSONL writer
- `Config` struct with layered loading

**Reference:** [Appendix: Error Handling](appendix_error_handling.md),
[Appendix: Configuration](appendix_configuration.md), [Appendix: ID
System](appendix_id_system.md)

### Work Items

#### F1: Error Types (`error/`)
**Files:** `error/error_types.rs`, `error/error_formatting.rs`,
`error/exit_codes.rs`, `error/structured_output.rs`

- Define `LatticeError` enum with all variants from error taxonomy
- Implement `Display` and `Error` traits via `thiserror`
- Define exit codes: 0 (success), 1 (system), 2 (validation), 3 (user), 4 (not
  found)
- Implement JSON error serialization for `--json` mode
- Integrate `human-panic` for panic formatting

#### F2: ID System (`id/`)
**Files:** `id/lattice_id.rs`, `id/base32_encoding.rs`, `id/id_generator.rs`,
`id/client_selection.rs`

- Define `LatticeId` newtype with `L` prefix validation
- Implement RFC 4648 Base32 encoding/decoding (alphabet: A-Z, 2-7)
- Document counter starting at 50 (ensures 6+ char IDs)
- Client ID generation with length scaling (3-6 chars based on known client
  count)
- Validation: minimum 6 chars, valid Base32, `L` prefix
- Counter recovery: scan existing documents to find highest counter per client
  when repo re-cloned

#### F3: Logging (`log/`)
**Files:** `log/jsonl_writer.rs`, `log/log_entry.rs`, `log/log_reader.rs`,
`log/log_init.rs`, `log/tracing_layer.rs`

- Configure `tracing` crate with JSONL subscriber via custom layer
- Write to `.lattice/logs.jsonl`
- Log entry struct: timestamp, level, operation, duration, details
- Log rotation: rename to `.1` when >10MB
- Support `--verbose` flag for debug output

#### F4: Configuration (`git/client_config.rs`, new `config/` module)
**Files:** `config/config_loader.rs`, `config/config_schema.rs`,
`git/client_config.rs`

- Parse `~/.lattice.toml` for user config (client IDs, defaults)
- Parse `.lattice/config.toml` for repo config (overview weights, format
  settings, etc.)
- Implement precedence: defaults → user → repo → env vars → CLI flags
- All fields from [Appendix: Configuration](appendix_configuration.md)

---

## Feature: Git Layer [DONE]

**Goal:** Abstract git operations behind a trait for testing. Implement document
discovery and change detection.

**Depends on:** Foundation (for error types, logging)

**Exports:**
- `GitOps` trait
- `RealGit` implementation
- `ModifiedFiles` detection
- Repository edge case detection

**Reference:** [Appendix: Git Integration](appendix_git_integration.md),
[Appendix: Git Edge Cases](appendix_git_edge_cases.md)

### Work Items

#### G1: GitOps Trait (`git/git_ops.rs`)
**Files:** `git/git_ops.rs`

- Define trait with methods: `ls_files`, `diff`, `status`, `rev_parse`, `log`,
  `config_get`
- All methods return `Result<T, LatticeError>`
- Trait object safe for dependency injection

#### G2: RealGit Implementation (`git/real_git.rs`)
**Files:** `git/real_git.rs`

- Shell out to git CLI
- Parse output for each operation
- Handle git errors as `LatticeError::GitError`
- Support pathspec patterns (`*.md`)

#### G3: Document Discovery (`git/modified_files.rs`)
**Files:** `git/modified_files.rs`

- `git ls-files '*.md'` for full enumeration
- `git diff --name-only <commit>..HEAD -- '*.md'` for changes
- `git status --porcelain -- '*.md'` for uncommitted changes
- Return `Vec<PathBuf>` for all operations

#### G4: Edge Case Detection (`git/repo_detection.rs`, `git/edge_cases.rs`)
**Files:** `git/repo_detection.rs`, `git/edge_cases.rs`,
`git/conflict_detection.rs`

- Detect: shallow clone, partial clone, sparse checkout, worktree, submodule,
  detached HEAD, in-progress ops
- Cache detection in `.lattice/repo_config.json`
- Invalidate when `.git` mtime changes
- Handle each case per [Appendix: Git Edge Cases](appendix_git_edge_cases.md)

---

## Feature: Document Model [DONE]

**Goal:** Parse and serialize Lattice documents. Validate frontmatter fields.

**Depends on:** Foundation (for ID types, error types)

**Exports:**
- `Document` struct with frontmatter and body
- `Frontmatter` struct with all fields
- Parse/serialize functions
- Field validation

**Reference:** [Lattice Design: Document
Structure](lattice_design.md#document-structure), [Appendix: Task
Tracking](appendix_task_tracking.md)

### Work Items

#### D1: Frontmatter Schema (`document/frontmatter_schema.rs`)
**Files:** `document/frontmatter_schema.rs`

- Define all frontmatter fields as typed struct
- Required: `lattice-id`, `name`, `description`
- Task fields: `task-type`, `priority`, `labels`, `blocking`, `blocked-by`,
  `discovered-from`
- Timestamps: `created-at`, `updated-at`, `closed-at`
- Skill: `skill` boolean
- `parent-id` (auto-populated)

#### D2: Frontmatter Parser (`document/frontmatter_parser.rs`)
**Files:** `document/frontmatter_parser.rs`

- Extract YAML between `---` markers
- Use `serde_yaml` for deserialization
- Return `LatticeError::InvalidFrontmatter` on parse failure
- Handle unknown keys (error E003)

#### D3: Field Validation (`document/field_validation.rs`)
**Files:** `document/field_validation.rs`

- Validate `name`: lowercase-hyphen format, max 64 chars, must match filename
- Validate `description`: non-empty, max 1024 chars
- Validate `priority`: 0-4 integer
- Validate `task-type`: bug/feature/task/chore
- Validate ID references in `blocking`/`blocked-by`/`discovered-from`

#### D4: Document Reader/Writer (`document/document_reader.rs`, `document/document_writer.rs`)
**Files:** `document/document_reader.rs`, `document/document_writer.rs`,
`document/markdown_body.rs`

- Read document from path, split frontmatter and body
- Write document: serialize frontmatter + `---` + body
- Atomic write: temp file + rename
- Body manipulation utilities for appending content

---

## Feature: Index Core [DONE]

**Goal:** SQLite schema, connection management, and core queries. No
reconciliation yet.

**Depends on:** Foundation, Document Model (for types)

**Exports:**
- `Index` struct with connection pool
- Schema creation/migration
- Document CRUD operations
- Link/label queries

**Reference:** [Appendix: Indexing](appendix_indexing.md)

### Work Items

#### I1: Schema Definition (`index/schema_definition.rs`)
**Files:** `index/schema_definition.rs`

- Tables: `documents`, `links`, `labels`, `index_metadata`, `client_counters`,
  `directory_roots`, `content_cache`, `views`
- All columns per [Appendix: Indexing](appendix_indexing.md#schema)
- Triggers for `link_count`/`backlink_count`/`view_count` denormalization
- FTS5 virtual table with external content mode

#### I2: Connection Pool (`index/connection_pool.rs`)
**Files:** `index/connection_pool.rs`

- SQLite connection with WAL mode
- PRAGMA configuration per [Appendix:
  Indexing](appendix_indexing.md#sqlite-configuration)
- `busy_timeout = 5000`
- Connection creation and closing with `PRAGMA optimize`

#### I3: Document Queries (`index/document_queries.rs`)
**Files:** `index/document_queries.rs`

- Insert/update/delete document
- Lookup by ID, path, or name
- Filter queries: by state, priority, type, labels, path prefix, timestamps
- Batch operations for efficiency

#### I4: Link Queries (`index/link_queries.rs`)
**Files:** `index/link_queries.rs`

- Insert/delete links for a document
- Query outgoing links (links-from)
- Query incoming links (links-to, backlinks)
- Link type filtering (body vs frontmatter)

#### I5: Label Queries (`index/label_queries.rs`)
**Files:** `index/label_queries.rs`

- Add/remove labels from document
- Query documents by label (AND/OR)
- List all labels with counts

#### I6: Full-Text Search (`index/fulltext_search.rs`)
**Files:** `index/fulltext_search.rs`

- FTS5 index sync via triggers
- Search with FTS5 query syntax
- Result ranking and snippet extraction
- Index optimization after bulk operations

#### I7: View Tracking (`index/view_tracking.rs`)
**Files:** `index/view_tracking.rs`

- Record view on `lat show`
- Query view counts and last-viewed timestamps
- Reset views (`--reset-views`)
- Concurrent-safe via SQLite transactions

#### I8: Content Cache (`index/content_cache.rs`)
**Files:** `index/content_cache.rs`

- L2 cache for document body content (L1 is OS filesystem cache)
- Validate via `file_mtime` comparison on access
- Evict least-recently-accessed when >100 documents cached
- Used by `lat show`, `lat search` snippets, template composition

---

## Feature: CLI Framework [DONE]

**Goal:** Argument parsing, command dispatch, output formatting. Skeleton for
all commands.

**Depends on:** Foundation (for error types)

**Exports:**
- Clap-based argument parser
- Command dispatch mechanism
- Text/JSON/pretty output formatters
- Color theme

**Reference:** [Appendix: CLI Structure](appendix_cli_structure.md), [Lattice
Design: UI Design](lattice_design.md#ui-design)

### Work Items

#### C1: Argument Parser (`cli/argument_parser.rs`)
**Files:** `cli/argument_parser.rs`

- Define all commands and subcommands using Clap derive macros
- Global options: `--json`, `--verbose`, `--quiet`, `--help`, `--version`
- Per-command options per [Appendix: CLI Structure](appendix_cli_structure.md)
- Environment variable bindings (`LATTICE_LOG_LEVEL`, etc.)

#### C2: Command Dispatch (`cli/command_dispatch.rs`)
**Files:** `cli/command_dispatch.rs`

- Match parsed command to handler function
- Thread context (GitOps, Index) through handlers
- Startup operations hook (runs before command)
- Exit code mapping from results

#### C3: Output Formatting (`cli/output_format.rs`)
**Files:** `cli/output_format.rs`

- Text formatter for human-readable output
- JSON formatter with serde serialization
- Pretty formatter with trees and colors
- Consistent document reference format: `<id>: <name> - <description> [<type>]`

#### C4: Color Theme (`cli/color_theme.rs`)
**Files:** `cli/color_theme.rs`

- Ayu color palette: green (success), yellow (warning), red (error), blue
  (accent), gray (muted)
- Terminal detection for color support
- `LATTICE_NO_COLOR` environment variable support
- Semantic coloring functions

#### C5: Global Options (`cli/global_options.rs`)
**Files:** `cli/global_options.rs`

- Process `--json`, `--verbose`, `--quiet` flags
- Configure logging level based on flags
- Output mode selection (text/json/quiet)

---

## Feature: Index Core (Continued) [DONE]

**Goal:** Additional Index Core functionality discovered during post-implementation
review that supports Reconciliation and other features.

**Depends on:** Index Core

**Exports:**
- Index metadata query functions for reconciliation fast-path
- Client counter management for ID generation
- Directory roots hierarchy queries for template composition
- Document filter name search capability

### Work Items

#### I9: Index Metadata Query Functions (`index/index_metadata.rs`)
**Files:** `index/index_metadata.rs`

- `get_metadata()` - retrieves full IndexMetadata struct
- `get_last_commit()` - gets last indexed git commit hash
- `get_last_indexed()` - gets last indexed timestamp
- `set_last_commit()` - updates commit hash and touches timestamp
- `touch_last_indexed()` - updates timestamp only

#### I10: Client Counters Query Functions (`index/client_counters.rs`)
**Files:** `index/client_counters.rs`

- `get_and_increment()` - atomic get-and-increment for ID generation
- `get_counter()` - read current counter without incrementing
- `set_counter()` - set counter value (for recovery)
- `set_counter_if_higher()` - conditional set for recovery
- `list_all()` - list all client counters
- `delete()` - remove a client's counter

#### I11: Directory Roots Hierarchy Queries (`index/directory_roots.rs`)
**Files:** `index/directory_roots.rs`

- `upsert()` - insert or update directory root entry
- `get()` - get full DirectoryRoot entry
- `get_root_id()` - get just the root document ID
- `get_ancestors()` - get all ancestors ordered root-to-parent
- `get_children()` - get immediate child directories
- `list_at_depth()` - list all roots at a specific depth
- `list_all()` - list all roots ordered by depth then path
- `delete()` - remove a directory root entry
- `clear_all()` - clear all entries (for full rebuild)

#### I12: Document Types Documentation
**Files:** `index/document_types.rs`

- Add module documentation explaining `DocumentRow` vs parsed `Document`
- Document the separation between index cache and full document parsing

#### I13: Document Filter Documentation
**Files:** `index/document_filter.rs`

- Add module documentation for `DocumentFilter` builder pattern
- Document `name_contains` filter for substring matching on document names
- Document available filter criteria and sort options

---

## Feature: First Working Command (lat show) [DONE]

**Goal:** End-to-end working `lat show` command. Validates entire pipeline.

**Depends on:** All previous features

**Milestone:** After this feature, `lat show <id>` works against real
repositories.

**Reference:** [Appendix: Workflow](appendix_workflow.md#lat-show)

### Work Items

**Note:** Work items below specify key functionality. For complete command option
lists (flags, arguments, output modes), implementers should reference
[Appendix: CLI Structure](appendix_cli_structure.md).

#### S1: Show Command Implementation (`cli/commands/show_command/`)
**Files:** `cli/commands/show_command/show_executor.rs`,
`cli/commands/show_command/document_formatter.rs`,
`cli/commands/show_command/output_formats.rs`

- Parse arguments: ID(s), `--short`, `--refs`, `--peek`, `--raw`, `--json`
- Lookup document(s) by ID in index
- Format task documents vs knowledge base documents
- Display dependencies, blocking, related documents
- Record view in view tracking table

#### S2: Startup Operations (`cli/startup.rs` or integrate into `command_dispatch.rs`)
**Files:** `cli/startup.rs` (new file or logic in dispatch)

**Reference:** [Appendix: Startup Operations](appendix_startup_operations.md)

- Index reconciliation (calls into index module)
- Skill symlink sync *(placeholder until Skill Integration feature)*
- Claim cleanup ✅ (calls `stale_cleanup::cleanup_stale_claims`)
- Log rotation
- Performance budget: <100ms total for all startup operations combined
- Hidden `--no-startup` flag for debugging (skips all startup ops, may cause stale data)
- Emit debug log entry if startup exceeds 100ms for investigation

**Note:** Skill symlink sync is implemented as a no-op placeholder in the First
Working Command milestone. It will be completed when the Skill Integration
feature is implemented.

---

## Feature: Index Reconciliation [DONE]

**Goal:** Keep SQLite index in sync with git repository state.

**Depends on:** Index Core, Git Layer, Document Model

**Reference:** [Appendix: Indexing](appendix_indexing.md#reconciliation),
[Appendix: Startup Operations](appendix_startup_operations.md)

### Work Items

#### R1: Reconciliation Coordinator (`index/reconciliation/reconciliation_coordinator.rs`)
**Files:** `index/reconciliation/reconciliation_coordinator.rs`

- Entry point for reconciliation
- Strategy selection: fast path, incremental, full rebuild
- Call appropriate strategy based on state

#### R2: Change Detection (`index/reconciliation/change_detection.rs`)
**Files:** `index/reconciliation/change_detection.rs`

- Check HEAD commit vs `index_metadata.last_commit`
- Check for uncommitted `.md` changes via git status
- Determine if fast path applies

#### R3: Sync Strategies (`index/reconciliation/sync_strategies.rs`)
**Files:** `index/reconciliation/sync_strategies.rs`

- **Fast path:** HEAD unchanged, no uncommitted changes → skip
- **Incremental:** Parse changed files from git diff, update index
- **Full rebuild:** Delete index, create schema, parse all `.md` files
- Schema version check triggers rebuild on mismatch
- Skip files with conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`) during
  parsing
- Log warning for skipped conflicted files

#### R4: Index Metadata Integration
**Files:** Uses `index/index_metadata.rs` (I9)

- Use I9 query functions for fast-path checks
- Update last_commit after successful reconciliation
- Update last_indexed timestamp

**Note:** Query functions are implemented in Index Core (I9). This work item
integrates those functions into the reconciliation workflow.

---

## Feature: Link System [DONE]

**Goal:** Extract, normalize, and track links between documents.

**Depends on:** Document Model, Index Core

**Exports:**
- Link extraction from markdown
- Link normalization (path + fragment)
- Bidirectional reference tracking

**Reference:** [Appendix: Linking System](appendix_linking_system.md)

### Work Items

#### L1: Link Extractor (`link/link_extractor.rs`)
**Files:** `link/link_extractor.rs`

- Parse markdown to find all links
- Extract link text, path, and fragment
- Handle shorthand ID-only links
- Classify links: Canonical, ShorthandId, PathOnly, External, Other

#### L2: Link Resolver (`link/link_resolver.rs`)
**Files:** `link/link_resolver.rs`

- Resolve Lattice ID to current file path via index
- Compute relative path from source document
- Handle missing targets (return error info)

#### L3: Link Normalization (`link/link_normalization/`)
**Files:** `link/link_normalization/normalization_executor.rs`,
`link/link_normalization/link_analysis.rs`,
`link/link_normalization/link_transforms.rs`

- Add missing Lattice ID fragments to path-only links
- Expand ID-only links to full path+fragment
- Update stale paths when documents moved
- Validation: flag missing fragments, stale paths

#### L4: Reference Tracker (`link/reference_tracker.rs`)
**Files:** `link/reference_tracker.rs`

- Maintain bidirectional link index
- Query forward links (what does X link to?)
- Query reverse links (what links to X?)
- Used by `lat links-from`, `lat links-to`, `lat impact`

#### L5: Frontmatter Links (`link/frontmatter_links.rs`)
**Files:** `link/frontmatter_links.rs`

- Extract IDs from `blocking`, `blocked-by`, `discovered-from`
- Handle custom `*-id` and `*-ids` fields
- Mark link type as 'frontmatter' in index

### Post-Implementation Notes

**Integration with Index Reconciliation:** The index reconciliation module
(`index/reconciliation/sync_strategies.rs`) has its own link ID extraction in
`extract_body_link_ids()` rather than using `link_extractor`. This is a minor
code duplication that can be refactored later. The reconciliation only needs
target IDs, so the current simple implementation is sufficient. A future
refactoring could call `link_extractor::extract()` and map results to IDs.

**Dependent Features Not Yet Implemented:**
- Commands that use link rewriting: `lat close`, `lat reopen`, `lat prune`,
  `lat mv`, `lat fmt`
- Relationship commands: `lat links-from`, `lat links-to`, `lat orphans`,
  `lat impact`, `lat path`
- Lint rules for link validation: W008 (self-reference), W010 (path mismatch),
  W010b (missing fragment), E002 (missing reference target)

These features will use the Link System exports when implemented.

---

## Feature: Task System [DONE]

**Goal:** Task types, states, dependencies, and ready calculation.

**Depends on:** Document Model, Index Core

**Exports:**
- Task state computation
- Ready work calculation
- Dependency graph
- Close/reopen operations

**Reference:** [Appendix: Task Tracking](appendix_task_tracking.md)

### Work Items

#### T1: Task Types and Priority (`task/task_types.rs`, `task/task_priority.rs`)
**Files:** `task/task_types.rs`, `task/task_priority.rs`

- `TaskType` enum: bug, feature, task, chore
- `Priority` enum: P0-P4 with descriptions
- Parse from string, serialize to string

#### T2: Task State (`task/task_state.rs`)
**Files:** `task/task_state.rs`

- State enum: Open, Blocked, Closed
- Compute from path: `.closed/` in path → Closed
- Compute blocked: any `blocked-by` entry not closed → Blocked
- Otherwise → Open

#### T3: Ready Calculator (`task/ready_calculator.rs`)
**Files:** `task/ready_calculator.rs`

- Task is ready if: not closed, not blocked, not P4, not claimed
- Query index with filters
- Support `--include-backlog`, `--include-claimed`
- Sort policies: hybrid, priority, oldest

#### T4: Dependency Graph (`task/dependency_graph.rs`)
**Files:** `task/dependency_graph.rs`

- Build graph from `blocking`/`blocked-by` fields
- Cycle detection (DFS for back edges)
- Dependency tree traversal for `lat dep tree`
- Topological ordering for impact analysis

#### T5: Closed Directory Utilities (`task/closed_directory.rs`)
**Files:** `task/closed_directory.rs`

- Compute `.closed/` path for a task
- Detect if path is in `.closed/`
- Move task to/from `.closed/` (for close/reopen)

#### T6: Root Detection (`task/root_detection.rs`)
**Files:** `task/root_detection.rs`

- Detect if document is root (filename matches directory)
- Compute parent-id from directory's root document
- Support hierarchy queries

#### T7: Directory Structure (`task/directory_structure.rs`)
**Files:** `task/directory_structure.rs`

- Validate `tasks/` and `docs/` directory structure
- Check document placement rules
- Support for structure lint rules (W017-W019)

---

## Feature: Linting and Formatting [ONGOING]

**Goal:** Validate documents (`lat check`) and normalize formatting (`lat fmt`).

**Depends on:** Document Model, Link System, Task System, Index Core

**Reference:** [Appendix: Linter](appendix_linter.md), [Lattice Design: Linter
and Formatter](lattice_design.md#linter-and-formatter)

### Work Items

#### LF1: Rule Engine (`lint/rule_engine.rs`)
**Files:** `lint/rule_engine.rs`

- Execute all rules against documents
- Collect results: errors vs warnings
- Support `--errors-only`, `--path` filtering
- Result aggregation and reporting

#### LF2: Error Rules (`lint/error_rules.rs`)
**Files:** `lint/error_rules.rs`

- E001: Duplicate Lattice ID
- E002: Missing reference target
- E003: Invalid frontmatter key
- E004: Missing required field (priority for tasks)
- E005: Invalid field value
- E006: Circular blocking
- E007: Invalid ID format
- E008: Name-filename mismatch
- E009: Missing name field
- E010: Missing description field
- E011: Invalid closed directory structure
- E012: Non-task in closed directory

#### LF3: Warning Rules (`lint/warning_rules.rs`)
**Files:** `lint/warning_rules.rs`

- W001: Document too large (>500 lines)
- W002: Name too long (>64 chars)
- W003: Description too long (>1024 chars)
- W004: Invalid name characters
- W005: Inconsistent header style
- W006: Inconsistent list markers
- W007: Bare URL
- W008: Self-reference
- W009: Backslash in path
- W010: Link path mismatch
- W010b: Missing link fragment
- W011: Trailing whitespace
- W012: Multiple blank lines
- W013: Missing final newline
- W014: Heading without blank lines
- W015: List without blank lines
- W016: Template section in non-root

#### LF4: Structure Rules (`lint/structure_rules.rs`)
**Files:** `lint/structure_rules.rs`

- W017: Document not in standard location
- W018: Task in docs/ directory
- W019: KB document in tasks/ directory
- W020: Invalid document name format

#### LF5: Skill Rules (`lint/skill_rules.rs`)
**Files:** `lint/skill_rules.rs`

- S001: Name contains reserved word (anthropic, claude)
- S002: Description empty
- S003: Name contains XML tags

#### LF6: Result Reporter (`lint/result_reporter.rs`)
**Files:** `lint/result_reporter.rs`

- Format errors/warnings for text output
- JSON output format per [Appendix: Linter](appendix_linter.md#json-output)
- Summary line with counts

#### LF7: Autofix Engine (`lint/autofix_engine.rs`)
**Files:** `lint/autofix_engine.rs`

- Apply automatic fixes for `--fix` flag
- Fixable: W004-W006, W011-W015, E008 (name mismatch)
- Track which fixes were applied

#### LF8: Check Command (`cli/commands/check_command/`)
**Files:** `cli/commands/check_command/check_executor.rs`,
`cli/commands/check_command/check_fixes.rs`,
`cli/commands/check_command/check_output.rs`

- Orchestrate rule execution
- Handle `--staged-only` filtering
- Handle `--rebuild-index` flag
- Output formatting

#### LF9: Markdown Formatter (`format/markdown_formatter.rs`)
**Files:** `format/markdown_formatter.rs`

- Orchestrate all formatting operations
- Apply to single file or directory

#### LF10: Text Wrapper (`format/text_wrapper.rs`)
**Files:** `format/text_wrapper.rs`

- Wrap lines at configurable width (default 80)
- Preserve code blocks, lists, tables
- Handle CJK character width

#### LF11: Header/List/Whitespace Normalizers (`format/header_normalizer.rs`, `format/list_normalizer.rs`, `format/whitespace_cleaner.rs`)
**Files:** `format/header_normalizer.rs`, `format/list_normalizer.rs`,
`format/whitespace_cleaner.rs`

- ATX header enforcement
- List marker consistency (dashes)
- Trailing whitespace removal
- Multiple blank line collapse
- Final newline enforcement
- Blank lines around headings/lists

#### LF12: Fmt Command (`cli/commands/fmt_command.rs`)
**Files:** `cli/commands/fmt_command.rs`

- Apply formatting and link normalization
- Handle `--check` flag (exit 1 if changes needed)
- Handle `--path` filtering
- Handle `--line-width` override

---

## Feature: Core Task Commands [ONGOING]

**Goal:** Create, update, close, reopen, prune commands.

**Depends on:** Document Model, Task System, Link System, Index Core

**Reference:** [Appendix: CLI Structure](appendix_cli_structure.md), [Appendix:
Task Tracking](appendix_task_tracking.md)

### Work Items

#### TC1: Create Command (`cli/commands/create_command.rs`)
**Files:** `cli/commands/create_command.rs`

- Parse: `lat create <parent> "<description>" [options]`
- Auto-placement: tasks to `tasks/`, KB to `docs/`
- Auto-naming: description → lowercase underscore filename
- Generate new Lattice ID
- Write document with frontmatter
- Update index

#### TC2: Update Command (`cli/commands/update_command.rs`)
**Files:** `cli/commands/update_command.rs`

- Parse: `lat update <id> [id...] [options]`
- Modify priority, type, labels
- Batch support for multiple IDs
- Update `updated-at` timestamp
- Update index

#### TC3: Close Command (`cli/commands/close_command.rs`)
**Files:** `cli/commands/close_command.rs`

- Parse: `lat close <id> [id...] [options]`
- Move document to `.closed/` subdirectory
- Rewrite all links pointing to moved document
- Set `closed-at` timestamp
- Release any claims
- Update index

#### TC4: Reopen Command (`cli/commands/reopen_command.rs`)
**Files:** `cli/commands/reopen_command.rs`

- Parse: `lat reopen <id> [id...]`
- Move document from `.closed/` back to parent
- Rewrite all links
- Clear `closed-at` timestamp
- Update index

#### TC5: Prune Command (`cli/commands/prune_command.rs`)
**Files:** `cli/commands/prune_command.rs`

- Parse: `lat prune <path>` or `lat prune --all`
- Permanently delete closed tasks
- Remove frontmatter references (`blocking`, `blocked-by`, `discovered-from`)
- Handle inline links: error by default, `--force` converts to plain text
- Update index

---

## Feature: Query Commands [ONGOING]

**Goal:** List, search, and filter documents.

**Depends on:** Index Core, Task System

**Reference:** [Appendix: CLI Structure](appendix_cli_structure.md)

### Work Items

#### Q1: List Command (`cli/commands/list_command/`)
**Files:** `cli/commands/list_command/list_executor.rs`,
`cli/commands/list_command/filter_builder.rs`,
`cli/commands/list_command/list_output.rs`

- All filter options from CLI spec
- Build SQL query from filters
- Output formats: rich, compact, oneline
- Sort and limit

#### Q2: Ready Command (`cli/commands/ready_command/`)
**Files:** `cli/commands/ready_command/ready_executor.rs`,
`cli/commands/ready_command/ready_filter.rs`,
`cli/commands/ready_command/ready_output.rs`

- Ready criteria: not closed, not blocked, not P4, not claimed
- Filter options: parent, priority, type, labels
- Sort policies: hybrid, priority, oldest
- Pretty tree output

#### Q3: Search Command (`cli/commands/search_command.rs`)
**Files:** `cli/commands/search_command.rs`

- FTS5 query execution
- Snippet extraction for results
- Filter by path, type
- Limit results

#### Q4: Stale Command (`cli/commands/stale_command.rs`)
**Files:** `cli/commands/stale_command.rs`

- Find tasks not updated recently
- Configurable staleness threshold (default 30 days)
- Reuse list filters

#### Q5: Blocked Command (`cli/commands/blocked_command.rs`)
**Files:** `cli/commands/blocked_command.rs`

- Find tasks with unresolved blockers
- `--show-blockers` to display blocking tasks
- Filter by path

#### Q6: Changes Command (`cli/commands/changes_command.rs`)
**Files:** `cli/commands/changes_command.rs`

- Documents changed since date or commit
- Use git log for timestamp correlation
- Output as list

#### Q7: Stats Command (`cli/commands/stats_command.rs`)
**Files:** `cli/commands/stats_command.rs`

- Document counts by state (open/blocked/closed)
- Priority and type breakdowns
- Recent activity period
- Health metrics

---

## Feature: Hierarchy Commands [DONE]

**Goal:** Navigate document tree structure.

**Depends on:** Index Core, Task System

**Reference:** [Appendix: CLI Structure](appendix_cli_structure.md)

### Work Items

#### H1: Tree Command (`cli/commands/tree_command.rs`)
**Files:** `cli/commands/tree_command.rs`

- Display directory structure with documents
- `--depth`, `--counts`, `--tasks-only`, `--docs-only`
- Visual tree formatting

#### H2: Roots Command (`cli/commands/roots_command.rs`)
**Files:** `cli/commands/roots_command.rs`

- List all root documents (filename matches directory)
- Include child counts

#### H3: Children Command (`cli/commands/children_command.rs`)
**Files:** `cli/commands/children_command.rs`

- List documents under a root's directory
- `--recursive`, `--tasks`, `--docs` filters

---

## Feature: Relationship Commands [ONGOING]

**Goal:** Explore links between documents.

**Depends on:** Link System, Index Core

**Reference:** [Appendix: CLI Structure](appendix_cli_structure.md), [Appendix:
Linking System](appendix_linking_system.md)

### Work Items

#### RL1: Links-From Command (`cli/commands/links_from.rs`)
**Files:** `cli/commands/links_from.rs`

- Show documents this document links to
- Query outgoing links from index

#### RL2: Links-To Command (`cli/commands/links_to.rs`)
**Files:** `cli/commands/links_to.rs`

- Show documents that link to this document (backlinks)
- Query incoming links from index

#### RL3: Path Command (`cli/commands/path_command.rs`)
**Files:** `cli/commands/path_command.rs`

- Find shortest path between two documents
- BFS on link graph
- Report if no path exists

#### RL4: Orphans Command (`cli/commands/orphans_command.rs`)
**Files:** `cli/commands/orphans_command.rs`

- Find documents with no incoming links
- `--exclude-roots` to omit root documents
- `--path` filter

#### RL5: Impact Command (`cli/commands/impact_command.rs`)
**Files:** `cli/commands/impact_command.rs`

- Analyze what would be affected by changes
- Recursive backlink traversal
- Show dependency chain

---

## Feature: Dependency Commands

**Goal:** Manage task dependencies.

**Depends on:** Task System, Document Model

**Reference:** [Appendix: CLI Structure](appendix_cli_structure.md)

### Work Items

#### DP1: Dep Add (`cli/commands/dep_command.rs` - add subcommand)
**Files:** `cli/commands/dep_command.rs`

- `lat dep add <id> <depends-on-id>`
- Add to `blocked-by` field of first task
- Add to `blocking` field of second task
- Validate no cycle created

#### DP2: Dep Remove (`cli/commands/dep_command.rs` - remove subcommand)
**Files:** `cli/commands/dep_command.rs`

- `lat dep remove <id> <depends-on-id>`
- Remove from both sides
- Update index

#### DP3: Dep Tree (`cli/commands/dep_command.rs` - tree subcommand)
**Files:** `cli/commands/dep_command.rs`

- Display dependency tree with state indicators
- Show upstream (depends on) and downstream (blocks)
- Visual tree format

---

## Feature: Label Commands

**Goal:** Manage document labels.

**Depends on:** Index Core, Document Model

**Reference:** [Appendix: CLI Structure](appendix_cli_structure.md)

### Work Items

#### LB1: Label Add/Remove (`cli/commands/label_command.rs`)
**Files:** `cli/commands/label_command.rs`

- `lat label add <id> [id...] <label>`
- `lat label remove <id> [id...] <label>`
- Update document frontmatter
- Update index

#### LB2: Label List (`cli/commands/label_command.rs` - list subcommand)
**Files:** `cli/commands/label_command.rs`

- `lat label list <id>` - labels on document
- `lat label list-all` - all labels with counts

---

## Feature: Utility Commands

**Goal:** Document maintenance and exploration.

**Depends on:** Various

**Reference:** [Appendix: CLI Structure](appendix_cli_structure.md)

### Work Items

#### U1: Track Command (`cli/commands/track_command.rs`)
**Files:** `cli/commands/track_command.rs`

- Add Lattice tracking to existing markdown files
- Generate new ID, add frontmatter
- `--force` to regenerate ID for duplicates

#### U2: Generate-IDs Command (`cli/commands/generate_ids.rs`)
**Files:** `cli/commands/generate_ids.rs`

- Pre-allocate IDs for offline authoring
- `-n` count (default 10)
- Output IDs to stdout

#### U3: Split Command (`cli/commands/split_command.rs`)
**Files:** `cli/commands/split_command.rs`

- Split document by top-level sections
- Create root document linking to children
- `--output-dir`, `--dry-run`

#### U4: Mv Command (`cli/commands/mv_command.rs`)
**Files:** `cli/commands/mv_command.rs`

- Move document to new location
- Update `parent-id` based on new directory
- Derive `name` from new filename
- Rewrite all links pointing to document
- Update index

#### U5: Edit Command (`cli/commands/edit_command.rs`)
**Files:** `cli/commands/edit_command.rs`

- Open document in `$EDITOR`
- Human-only (not for AI agents)
- `--name`, `--description`, `--body` modes

---

## Feature: Overview and Prime Commands

**Goal:** AI-optimized context commands.

**Depends on:** Index Core, View Tracking, Task System

**Reference:** [Appendix: Overview Command](appendix_overview_command.md),
[Appendix: Workflow](appendix_workflow.md)

### Work Items

#### OP1: Overview Command (`cli/commands/overview_command.rs`)
**Files:** `cli/commands/overview_command.rs`

- Repository-level: rank by views, recency, root priority
- Contextual (`lat overview <id>`): graph distance model
- Configurable weights from config
- `--limit`, `--type`, `--path`, `--include-closed`, `--reset-views`
- JSON output

#### OP2: Prime Command (`cli/commands/prime_command.rs`)
**Files:** `cli/commands/prime_command.rs`

- Output workflow context for AI agents
- Session protocol checklist
- Core commands reference
- Link authoring guidance
- `--full`, `--export` modes
- Custom checklist from config

---

## Feature: Claim System [DONE]

**Goal:** Local work tracking without git-tracked state.

**Depends on:** Foundation, Task System

**Reference:** [Appendix: Workflow](appendix_workflow.md#lat-claim)

### Work Items

#### CL1: Claim Storage (`claim/claim_storage.rs`)
**Files:** `claim/claim_storage.rs`

- File-based storage in `~/.lattice/claims/<repo-hash>/`
- One JSON file per claim
- Atomic create/delete for concurrent safety
- Repo hash: first 8 chars of SHA-256 of canonical root path

#### CL2: Claim Operations (`claim/claim_operations.rs`)
**Files:** `claim/claim_operations.rs`

- Claim task: create file
- Release task: delete file
- Check if task claimed
- List claims for repo

#### CL3: Stale Cleanup (`claim/stale_cleanup.rs`)
**Files:** `claim/stale_cleanup.rs`

- Detect stale claims: task closed, task deleted, worktree gone, age > threshold
- `--gc` to clean up
- Run on startup

#### CL4: Claim Command (`cli/commands/claim_command.rs`)
**Files:** `cli/commands/claim_command.rs`

- `lat claim <id>` - claim task
- `lat claim --list` - show claims
- `lat claim --release <id>` - release specific
- `lat claim --release-all` - release all
- `lat claim --release-worktree <path>` - release by worktree
- `lat claim --gc` - garbage collection

### Integration Points

The following integration points connect claims to other features:

- **Startup cleanup (S2):** `run_claim_cleanup()` in `command_dispatch.rs` runs
  stale claim cleanup on every command. ✅ DONE
- **lat show (S1):** `show_executor.rs` displays claim status in output. ✅ DONE
- **lat close (TC3):** Close Command will auto-release claims when tasks are
  closed. *Pending - part of Core Task Commands feature.*
- **lat ready (Q2):** Ready Command will filter out claimed tasks by default.
  *Pending - part of Query Commands feature.*

---

## Feature: Template System

**Goal:** Compose context and acceptance criteria from ancestor root documents.

**Depends on:** Document Model, Index Core, Task System

**Reference:** [Appendix: Task Templates](appendix_task_templates.md)

### Work Items

#### TM1: Template Composer (`task/template_composer.rs`)
**Files:** `task/template_composer.rs`

- Walk ancestor chain via `directory_roots` table
- Extract `[Lattice] Context` and `[Lattice] Acceptance Criteria` sections
- Compose context: general → specific order
- Compose acceptance: specific → general order

#### TM2: Section Extractor (`task/section_extractor.rs` or within `template_composer.rs`)
**Files:** `task/template_composer.rs` (or new file)

- Parse markdown headings for `[Lattice]` prefix
- Extract section content up to next heading of same/higher level
- Handle any heading level (# through ######)

#### TM3: Directory Roots Population
**Files:** Uses `index/directory_roots.rs` (I11)

- Populate directory_roots table during reconciliation
- Compute depth and parent_path for each root document
- Update entries when documents are moved

**Note:** Query functions are implemented in Index Core (I11). This work item
populates the table during index reconciliation.

---

## Feature: Skill Integration

**Goal:** Claude Code skill symlinks and hooks.

**Depends on:** Document Model, Index Core

**Reference:** [Appendix: AI Integration](appendix_ai_integration.md), [Lattice
Design: Skill Integration](lattice_design.md#skill-integration)

### Work Items

#### SK1: Symlink Manager (`skill/symlink_manager.rs`)
**Files:** `skill/symlink_manager.rs`

- Create `.claude/skills/` directory
- Create symlinks: `<name>.md` → actual document path
- Remove stale symlinks
- Update symlinks when documents move

#### SK2: Skill Validation (`skill/skill_validation.rs`)
**Files:** `skill/skill_validation.rs`

- Validate skill requirements: name max 64 chars, no reserved words, no XML
- Description non-empty, max 1024 chars
- Used by lint rules S001-S003

#### SK3: Setup Command (`cli/commands/setup_command.rs`)
**Files:** `cli/commands/setup_command.rs`, embedded Python hook script

- `lat setup claude` - install hooks and config
- `--check` - verify installation
- `--remove` - uninstall
- `--project` - project-only vs global
- Create `.claude/hooks/lattice-read-guard.py`:
  - Intercept Read tool calls
  - Check if target file contains `lattice-id:` in frontmatter
  - If yes: block read, return guidance with document ID and `lat show` command
  - If no: allow read to proceed
- Update `.claude/settings.json` with PreToolUse hook configuration

---

## Feature: Doctor Command

**Goal:** System health diagnostics and auto-fix.

**Depends on:** Index Core, Git Layer, Claim System, Skill Integration

**Reference:** [Appendix: Doctor Command](appendix_doctor.md)

### Work Items

#### DR1: Doctor Command (`cli/commands/doctor_command.rs`)
**Files:** `cli/commands/doctor_command.rs`

**Reference:** [Appendix: Doctor Command](appendix_doctor.md) for complete check
specifications, severity levels, output formats, and fix capabilities.

- Execute all check categories
- `--fix` for auto-repair
- `--dry-run` for preview
- `--deep` for thorough checks
- `--json` output
- Exit codes: 0 (passed), 1 (system error), 2 (errors), 3 (warnings)

#### DR2: Core System Checks

Implement checks from [Appendix: Doctor Command - Core System](appendix_doctor.md#core-system):

- Installation: `.lattice/` exists
- Index exists: `index.sqlite` present
- Schema version matches
- WAL health: no corruption

#### DR3: Index Integrity Checks

Implement checks from [Appendix: Doctor Command - Index Integrity](appendix_doctor.md#index-integrity):

- Filesystem sync: every indexed ID has file
- Coverage: every `.md` with ID is indexed
- No duplicate IDs
- Closed state matches path
- Root state matches filename
- Parent consistency

#### DR4: Git Integration Checks

Implement checks from [Appendix: Doctor Command - Git Integration](appendix_doctor.md#git-integration):

- Valid repository
- Edge case detection
- Working tree state
- Detached HEAD info

#### DR5: Configuration Checks

Implement checks from [Appendix: Doctor Command - Configuration](appendix_doctor.md#configuration):

- User config parseable
- Repo config valid
- Client ID assigned
- Values in range

#### DR6: Claims Checks

Implement checks from [Appendix: Doctor Command - Claims](appendix_doctor.md#claims):

- No claims for closed tasks
- No claims for deleted tasks
- No orphaned worktree claims

#### DR7: Skills Checks

Implement checks from [Appendix: Doctor Command - Skills](appendix_doctor.md#skills):

- Symlinks resolve
- All skill documents linked
- Symlinks point to current paths

#### DR8: Fix Actions

Implement fixes from [Appendix: Doctor Command - Fixable Issues](appendix_doctor.md#fixable-issues):

- Rebuild index for sync issues
- Delete stale claims
- Create/update/remove symlinks

---

## Feature: Shell Completions

**Goal:** Generate shell completion scripts.

**Depends on:** CLI Framework

**Reference:** [Appendix: CLI
Structure](appendix_cli_structure.md#shell-completions)

### Work Items

#### SH1: Completion Command (`cli/commands/completion_command.rs`)
**Files:** `cli/commands/completion_command.rs`

- Generate for bash, zsh, fish
- Include command names, flags
- Dynamic completion for Lattice IDs (query index)
- Output to stdout for piping to completion files

---

## Feature: Testing Infrastructure

**Goal:** Test utilities, FakeGit, assertions.

**Depends on:** Git Layer (for GitOps trait)

**Reference:** [Appendix: Testing Strategy](appendix_testing_strategy.md)

### Work Items

#### TS1: Test Environment (`test/test_environment.rs`)
**Files:** `test/test_environment.rs`

- `TestEnv` struct with temp directory
- FakeGit injection
- In-memory SQLite (`:memory:`)
- Setup and teardown helpers

#### TS2: FakeGit (`test/fake_git.rs`)
**Files:** `test/fake_git.rs`

- Implement `GitOps` trait
- In-memory file state and commit history
- Support all required git operations
- Configurable failure injection

#### TS3: Test Assertions (`test/test_assertions.rs`)
**Files:** `test/test_assertions.rs`

- Custom assertion helpers
- Clear failure messages
- Document state assertions
- Index state assertions

#### TS4: Test Fixtures (`test/test_fixtures.rs`)
**Files:** `test/test_fixtures.rs`

- Common test document templates
- Pre-built repository states
- Fixture generators

#### TS5: Command Tests (`tests/lattice/commands/`)
**Files:** `tests/lattice/commands/*.rs`

- Per-command test files
- Happy path and error cases
- One file per command or command group

#### TS6: Integration Tests (`tests/lattice/integration/`)
**Files:** `tests/lattice/integration/*.rs`

- Multi-command workflow tests
- Git edge case tests
- Template composition tests

#### TS7: Foundation Edge Case Tests (`tests/lattice/foundation/`)
**Files:** `tests/lattice/foundation/*.rs`

Tests for edge cases in Foundation, Git Layer, and Document Model:

- **Log rotation:** Test behavior when `.1` file already exists, verify atomic
  rotation
- **Client ID collision:** Test ID generation when random ID collides with
  existing client
- **Counter recovery:** Test scanning documents with mixed client ID lengths
  (3, 4, 5, 6 chars)
- **Unknown frontmatter key suggestions:** Test edit distance calculation and
  suggestion threshold
- **Partial/treeless clone detection:** Test behavior differences between
  blobless and treeless clones

---

## Feature: Benchmarking

**Goal:** Performance measurement with Criterion.

**Depends on:** Index Core, Document Model

**Reference:** [Appendix: Benchmarking](appendix_benchmarking.md)

### Work Items

#### BM1: Benchmark Setup (`benches/lattice/`)
**Files:** `benches/lattice/mod.rs`, `benches/lattice/index_bench.rs`,
`benches/lattice/document_bench.rs`, `benches/lattice/query_bench.rs`

- Criterion benchmark groups
- Test repository generators (10, 100, 500, 1000 docs)
- HTML report generation

#### BM2: Index Benchmarks
- Full rebuild at various sizes
- Incremental reconciliation
- Document lookup by ID
- FTS search queries

#### BM3: Document Benchmarks
- Frontmatter parsing
- Link extraction
- Formatting operations

#### BM4: Query Benchmarks
- `lat list` with filters
- `lat ready` filtering
- `lat overview` ranking

---

## Feature: Chaos Monkey

**Goal:** Fuzz testing for robustness.

**Depends on:** All other features (comprehensive testing)

**Reference:** [Appendix: Chaos Monkey](appendix_chaos_monkey.md)

### Work Items

#### CM1: Chaos Monkey Command (`cli/commands/chaos_monkey.rs`)
**Files:** `cli/commands/chaos_monkey.rs`

- `lat chaosmonkey [options]`
- `--seed` for reproducibility
- `--max-ops` limit
- `--operations` include list
- `--exclude` exclude list
- `--stop-before-last` for debugging

#### CM2: Operation Generators
- High-level: random lat commands with valid/invalid args
- Low-level: direct filesystem manipulation
- Git operations: add, commit, checkout, merge, stash

#### CM3: Invariant Checkers
- Index-filesystem consistency
- ID uniqueness
- ID format validity
- Git state validity
- No panics
- Closed state consistency
- Root document consistency
- Directory structure consistency
- Link path validity after close/reopen

#### CM4: Failure Reporting
- Output seed, operation count, failing operation
- Specific invariant that failed
- Repository state for reproduction

---

## Dependency Summary

```
Foundation ──────────────────────────────────────────────────────────┐
     │                                                                │
     ├── Git Layer ──────────────────────────────────────────────────┤
     │        │                                                       │
     │        └── Document Model ────────────────────────────────────┤
     │                   │                                            │
     │                   ├── Index Core ─────────────────────────────┤
     │                   │        │                                   │
     │                   │        ├── Index Reconciliation           │
     │                   │        │                                   │
     │                   │        ├── Link System ───────────────────┤
     │                   │        │        │                          │
     │                   │        │        └── Relationship Commands │
     │                   │        │                                   │
     │                   │        └── View Tracking                  │
     │                   │                                            │
     │                   └── Task System ────────────────────────────┤
     │                            │                                   │
     │                            ├── Template System                │
     │                            │                                   │
     │                            └── Core Task Commands             │
     │                                                                │
     ├── CLI Framework ──────────────────────────────────────────────┤
     │        │                                                       │
     │        └── First Working Command (lat show) ←─────────────────┤
     │                   │                                            │
     │                   └── All Other Commands                      │
     │                                                                │
     ├── Claim System (standalone, uses Foundation only)             │
     │                                                                │
     ├── Skill Integration (uses Document Model, Index)              │
     │                                                                │
     ├── Linting & Formatting (uses Link, Task, Document, Index)     │
     │                                                                │
     ├── Doctor Command (uses all systems for diagnostics)           │
     │                                                                │
     ├── Testing Infrastructure (uses Git Layer for FakeGit)         │
     │                                                                │
     ├── Benchmarking (uses Index, Document)                         │
     │                                                                │
     └── Chaos Monkey (uses all features)                            │
```

## Hard Dependencies (Must Complete Before)

| Feature | Hard Dependencies |
|---------|-------------------|
| Git Layer | Foundation |
| Document Model | Foundation |
| Index Core | Foundation, Document Model |
| CLI Framework | Foundation |
| First Working Command | Git Layer, Document Model, Index Core, CLI Framework |
| Index Reconciliation | Index Core, Git Layer |
| Link System | Document Model, Index Core |
| Task System | Document Model, Index Core |
| Linting & Formatting | Document Model, Link System, Task System, Index Core |
| Core Task Commands | Document Model, Task System, Link System, Index Core |
| Query Commands | Index Core, Task System |
| Hierarchy Commands | Index Core, Task System |
| Relationship Commands | Link System, Index Core |
| Overview/Prime | Index Core, View Tracking, Task System |
| Claim System | Foundation, Task System |
| Template System | Document Model, Index Core, Task System |
| Skill Integration | Document Model, Index Core |
| Doctor Command | All systems |
| Testing Infrastructure | Git Layer |
| Benchmarking | Index Core, Document Model |
| Chaos Monkey | All features |

## Parallel Development Opportunities

These features can be developed in parallel after their dependencies are met:

**After Foundation:**
- Git Layer [DONE]
- CLI Framework [DONE]

**After Foundation + Git Layer + Document Model + Index Core:**
- First Working Command [DONE]
- Index Reconciliation [DONE]
- Link System [DONE]
- Task System [DONE]
- Claim System (only needs Foundation + Task types) [DONE]

**After Link/Task Systems:**
- Linting & Formatting [DONE]
- Core Task Commands [DONE]
- Query Commands [DONE]
- Hierarchy Commands [DONE]
- Relationship Commands
- Template System

**Standalone (after respective dependencies):**
- Testing Infrastructure (after Git Layer)
- Benchmarking (after Index + Document)
- Shell Completions (after CLI Framework)
- Skill Integration (after Document + Index)

**Final Integration:**
- Doctor Command (after all systems)
- Chaos Monkey (after all features)
