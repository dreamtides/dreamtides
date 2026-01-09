# Milestone 3: Crate Setup & Scaffolding

## Objective

Create the `tabula_data_v2` crate structure with Cargo.toml and empty module files.

## Tasks

1. Create `src/tabula_data_v2/Cargo.toml` with all required dependencies
2. Create `src/tabula_data_v2/src/lib.rs` with module declarations (no code)
3. Create empty stub files for each module listed in the design document
4. Run `just check` to verify the crate compiles
5. Add the crate to the workspace `Cargo.toml`

## Dependencies to Add

```toml
[dependencies]
ability_data = { path = "../ability_data" }
core_data = { path = "../core_data" }
parser_v2 = { path = "../parser_v2" }

anyhow = "1"
fluent = "0.16"
fluent-bundle = "0.15"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
uuid = { version = "1", features = ["v4", "serde"] }
```

## Module Stubs to Create

- `card_definition_raw.rs`
- `card_definition_builder.rs`
- `card_definition.rs`
- `dreamwell_definition.rs`
- `card_effect_row.rs`
- `card_list_row.rs`
- `fluent_loader.rs`
- `ability_parser.rs`
- `toml_loader.rs`
- `tabula_struct.rs`
- `tabula_error.rs`

## Verification

- `just check` passes
- `just clippy` passes (with empty stubs)

## Context Files

1. `src/tabula_data/Cargo.toml` - Reference for dependency patterns
2. `docs/tabula/tabula_v2_design_document.md` - Overall architecture
3. `Cargo.toml` (workspace root) - How to add workspace members
