# Appendix: File Layout

## Source Directory Structure

All Rust code lives under `rules_engine/src/lattice/`:

```
rules_engine/src/lattice/
├── mod.rs                      # Module root, public exports
├── cli/
│   ├── mod.rs                  # CLI module root
│   ├── arg_parser.rs           # Command argument definitions
│   ├── command_dispatch.rs     # Route args to handlers
│   ├── output_formatter.rs     # Human/JSON output rendering
│   ├── color_theme.rs          # Ayu theme color definitions
│   └── commands/
│       ├── mod.rs              # Commands module root
│       ├── show_command.rs     # lat show implementation
│       ├── create_command.rs   # lat create implementation
│       ├── update_command.rs   # lat update implementation
│       ├── list_command.rs     # lat list implementation
│       ├── check_command.rs    # lat check implementation
│       ├── format_command.rs   # lat fmt implementation
│       ├── split_command.rs    # lat split implementation
│       ├── track_command.rs    # lat track implementation
│       ├── generate_command.rs # lat generate-ids implementation
│       ├── ready_command.rs    # lat ready implementation
│       ├── stale_command.rs    # lat stale implementation
│       ├── dep_command.rs      # lat dep tree implementation
│       ├── label_command.rs    # lat label add/remove/list
│       ├── close_command.rs    # lat close implementation
│       ├── reopen_command.rs   # lat reopen implementation
│       ├── edit_command.rs     # lat edit implementation
│       └── chaos_command.rs    # lat chaosmonkey implementation
│
├── index/
│   ├── mod.rs                  # Index module root
│   ├── schema_definition.rs    # SQLite schema DDL
│   ├── schema_migration.rs     # Version upgrade logic
│   ├── index_connection.rs     # SQLite connection management
│   ├── document_queries.rs     # Document CRUD queries
│   ├── link_queries.rs         # Link relationship queries
│   ├── label_queries.rs        # Label management queries
│   ├── section_queries.rs      # Section ID queries
│   ├── search_queries.rs       # Full-text search queries
│   ├── reconcile_engine.rs     # Git-to-index sync logic
│   └── counter_manager.rs      # Client ID counter logic
│
├── document/
│   ├── mod.rs                  # Document module root
│   ├── document_parser.rs      # YAML + Markdown parsing
│   ├── frontmatter_schema.rs   # Frontmatter field definitions
│   ├── document_validator.rs   # Field validation rules
│   ├── document_writer.rs      # Document serialization
│   ├── section_extractor.rs    # Header/section parsing
│   └── body_manipulator.rs     # Content modification utilities
│
├── git/
│   ├── mod.rs                  # Git module root
│   ├── git_operations.rs       # Git command execution
│   ├── git_interface.rs        # Trait for git operations
│   ├── change_detector.rs      # Modified file detection
│   └── client_config.rs        # ~/.lattice.toml management
│
├── format/
│   ├── mod.rs                  # Format module root
│   ├── markdown_formatter.rs   # Markdown normalization
│   ├── text_wrapper.rs         # Line wrapping at N chars
│   ├── header_normalizer.rs    # ATX header enforcement
│   └── list_normalizer.rs      # List marker consistency
│
├── link/
│   ├── mod.rs                  # Link module root
│   ├── link_extractor.rs       # Extract links from markdown
│   ├── link_resolver.rs        # Resolve IDs to paths
│   └── reference_tracker.rs    # Bidirectional reference map
│
├── context/
│   ├── mod.rs                  # Context module root
│   ├── context_algorithm.rs    # Document selection logic
│   ├── candidate_gatherer.rs   # Collect related documents
│   ├── budget_allocator.rs     # Greedy inclusion logic
│   └── output_assembler.rs     # Final output construction
│
├── lint/
│   ├── mod.rs                  # Lint module root
│   ├── lint_runner.rs          # Rule execution engine
│   ├── error_rules.rs          # Error-level rule implementations
│   ├── warning_rules.rs        # Warning-level rule implementations
│   ├── skill_rules.rs          # Skill-specific validations
│   └── lint_report.rs          # Result aggregation and output
│
├── id/
│   ├── mod.rs                  # ID module root
│   ├── lattice_id.rs           # ID type definition
│   ├── base32_codec.rs         # Encoding/decoding logic
│   ├── id_generator.rs         # New ID creation
│   └── client_selector.rs      # Client ID selection logic
│
├── issue/
│   ├── mod.rs                  # Issue module root
│   ├── issue_types.rs          # Type/status/priority enums
│   ├── state_machine.rs        # Status transition rules
│   ├── ready_calculator.rs     # Ready work determination
│   └── dependency_graph.rs     # Blocking relationship graph
│
├── skill/
│   ├── mod.rs                  # Skill module root
│   └── skill_generator.rs      # Symlink creation for .claude/
│
├── log/
│   ├── mod.rs                  # Log module root
│   ├── json_logger.rs          # JSONL output to logs.jsonl
│   └── log_entry.rs            # Log entry structure
│
├── error/
│   ├── mod.rs                  # Error module root
│   ├── error_types.rs          # User vs system error types
│   ├── error_formatter.rs      # Human-readable error output
│   └── exit_codes.rs           # Exit code constants
│
└── test/
    ├── mod.rs                  # Test utilities module root
    ├── test_environment.rs     # TestEnv setup
    ├── fake_git.rs             # In-memory git fake
    ├── fake_filesystem.rs      # In-memory filesystem fake
    ├── fake_clock.rs           # Controllable time fake
    ├── document_builder.rs     # Test document factory
    └── assertion_helpers.rs    # Custom test assertions
```

## Test Directory Structure

```
rules_engine/tests/lattice/
├── cli_tests/
│   ├── mod.rs
│   ├── show_tests.rs
│   ├── create_tests.rs
│   ├── update_tests.rs
│   ├── list_tests.rs
│   ├── check_tests.rs
│   ├── format_tests.rs
│   └── chaos_tests.rs
├── index_tests/
│   ├── mod.rs
│   ├── reconcile_tests.rs
│   ├── query_tests.rs
│   └── migration_tests.rs
├── integration_tests/
│   ├── mod.rs
│   ├── workflow_tests.rs
│   └── edge_case_tests.rs
└── snapshots/
    └── *.snap
```

## Configuration Files

```
rules_engine/
├── Cargo.toml                  # Add lattice dependencies
└── src/
    └── main.rs                 # lat binary entry point (or add to existing)
```

## Runtime Directory Structure

Created in the repository root:

```
<repo>/
├── .lattice/
│   ├── index.sqlite            # SQLite index database
│   ├── index.sqlite-wal        # WAL file (gitignored)
│   ├── index.sqlite-shm        # Shared memory file (gitignored)
│   ├── logs.jsonl              # Operation log
│   └── config.toml             # Local config overrides (optional)
├── .gitignore                  # Should include .lattice/
└── .claude/
    └── skills/                 # Symlinks to skill documents
        └── skill-name.md -> ../../path/to/doc.md
```

## User Configuration

```
~/.lattice.toml
├── [clients]                   # Repository -> client ID mapping
│   └── "/path/to/repo" = "DT"
└── [defaults]                  # Optional global defaults
    ├── context_budget = 5000
    └── line_width = 80
```

## Module Dependencies

```
cli
├── commands/* → index, document, context, lint, id, issue, skill
├── arg_parser → (standalone)
├── command_dispatch → commands/*
├── output_formatter → error
└── color_theme → (standalone)

index
├── schema_* → (standalone SQL)
├── *_queries → index_connection
├── reconcile_engine → git, document
└── counter_manager → id

document
├── document_parser → frontmatter_schema
├── document_validator → frontmatter_schema, id
├── section_extractor → (standalone)
└── body_manipulator → (standalone)

git
├── git_operations → git_interface
├── change_detector → git_operations
└── client_config → (standalone TOML)

link
├── link_extractor → id
└── link_resolver → index

context
├── context_algorithm → candidate_gatherer, budget_allocator
├── candidate_gatherer → index, link
└── output_assembler → document

lint
├── lint_runner → error_rules, warning_rules, skill_rules
├── *_rules → document, index, id
└── lint_report → error

error → (standalone)
log → (standalone)
test → git, document, id (for fakes)
```

## Binary Entry Point

The `lat` binary can be:
1. A separate binary in the workspace: `rules_engine/src/bin/lat.rs`
2. A subcommand of an existing binary
3. Integrated into an existing CLI structure

Recommendation: Separate binary for clean separation of concerns:

```rust
// rules_engine/src/bin/lat.rs
fn main() {
    lattice::cli::run();
}
```
