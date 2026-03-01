---
name: tv
description: Use when working with the TV (TOML Viewer) desktop application for editing TOML files in a spreadsheet format. Use when implementing features for TV, understanding the TV architecture, working with Tauri commands, or debugging TV issues.
---

# TV (TOML Viewer) Development Guide

Desktop app for viewing/editing TOML files as spreadsheets. Built with
Tauri V2 (Rust backend) + React/Univer (TypeScript frontend).

## Build Commands

Always use `just`, never raw `cargo` or `pnpm` directly.

| Command | Purpose |
|---------|---------|
| `just tv-dev` | Start dev server (kills stale processes first) |
| `just tv-cards` | Dev with cards.toml loaded |
| `just tv-check` | Rust check + TypeScript tsc + ESLint |
| `just tv-clippy` | Rust clippy with `-D warnings` |
| `just tv-test NAME` | Run specific integration test |
| `just tv-build` | Production build |
| `just fmt` | Format all code (run first to auto-fix) |
| `just review` | Full gate (includes tv-check, tv-clippy, tv-test) |

## Architecture (3 Layers)

**Layer 1 - TOML (Rust):** File I/O via `toml_edit` for structure
preservation. Atomic writes via temp files + rename.

**Layer 2 - Application (Rust):** Independent state managers
(sort, filter, derived columns, images, sync, permissions).
Managers never communicate directly; frontend coordinates.

**Layer 3 - UI (TypeScript):** React + Univer spreadsheet.
Debounced cell edits (500ms), IPC via Tauri commands/events.

## Key Files to Read First

**Rust backend** (`rules_engine/src/tv/src-tauri/src/`):
- `lib.rs` — Tauri builder, all 47 command registrations, state setup
- `commands/` — One module per command group (load, save, sort, etc.)
- `toml/document_loader.rs` — TOML parsing into spreadsheet data
- `toml/document_writer.rs` — Atomic writes, cell-level updates
- `toml/metadata/` — 7 submodules parsing metadata categories
- `error/error_types.rs` — `TvError` enum (all failure modes)
- `CLAUDE.md` — Backend-specific AI guidance

**TypeScript frontend** (`rules_engine/src/tv/src/`):
- `ipc_bridge.ts` — All Tauri command/event type definitions
- `UniverSpreadsheet.tsx` — Main Univer wrapper (largest file)
- `app_root.tsx` — Root component, data loading, sync handling
- `univer_config.ts` — Univer plugin initialization order
- `image_cell_renderer.ts` — Image rendering workaround

**Configuration:**
- `rules_engine/src/tv/src-tauri/tauri.conf.json` — Tauri app config
- `rules_engine/src/tv/vite.config.ts` — Vite + RxJS recursion fix
- `rules_engine/src/tv/eslint.config.js` — Strict TypeScript lint rules

**Design docs** (`rules_engine/src/tv/docs/`):
- `tv_design_document.md` — Full architecture reference
- `appendix_d_univer_integration.md` — Univer pitfalls and workarounds

## Adding a New Tauri Command

1. Create handler in `commands/<group>_command.rs`:
```rust
#[tauri::command]
pub fn my_command(
    app_handle: AppHandle,
    param: String,
) -> Result<ResponseType, TvError> {
    // Implementation — never panic, always return Result
}
```
2. Register in `lib.rs` `invoke_handler` macro
3. Add IPC wrapper in `src/ipc_bridge.ts`
4. Use tracing: `tracing::debug!(component = "tv.commands.my_cmd", ...)`

## Testing

Tests live in `rules_engine/tests/tv_tests/` as integration tests.
**Never** write inline `mod tests {}`. Test against the public API.

**`_with_fs` pattern:** Functions doing file I/O have two variants:
```rust
pub fn load(path: &str) -> Result<Data, TvError> {
    load_with_fs(&RealFileSystem, path)
}
pub fn load_with_fs(fs: &dyn FileSystem, path: &str) -> Result<Data, TvError> {
    // tests inject FakeFileSystem here
}
```

Test utilities: `rules_engine/tests/tv_tests/src/test_utils/` has
`mock_filesystem.rs`, `mock_clock.rs`, `harness.rs`, `fixture_loader.rs`.

## TypeScript Code Quality

ESLint enforces strict TypeScript safety — **no `any` allowed**:
- `@typescript-eslint/no-explicit-any`: error
- `@typescript-eslint/no-unsafe-assignment`: error
- `@typescript-eslint/no-unsafe-member-access`: error
- `@typescript-eslint/no-unsafe-call`: error
- `@typescript-eslint/no-unsafe-return`: error
- Unused vars allowed only with `_` prefix

## Univer Pitfalls

Read [appendix_d_univer_integration.md](../../rules_engine/src/tv/docs/appendix_d_univer_integration.md) before working on Univer code.

**Version pinning:** All `@univerjs/*` packages must be the same exact
version (currently 0.15.3). Mismatched versions break everything.

**Facade API is broken under Vite:** `insertImage()` and similar facade
methods fail due to class prototype duplication from Vite pre-bundling.
Use direct command execution instead:
```typescript
univerAPI.executeCommand("sheet.command.insert-sheet-image", { ... });
```

**RxJS recursion bug:** `bufferWhen` causes infinite recursion with
Univer's lifecycle system. Fixed by a Vite plugin in `vite.config.ts`
that patches RxJS at build time. Do not remove this.

**Plugin load order matters.** See `univer_config.ts` for the required
sequence (render engine first, then formula, then UI, then sheets...).

## Rust Error Handling

Always return `Result<T, TvError>`. Never `unwrap()` or `expect()` in
command handlers. Use helpers:
- `map_io_error_for_read(error, path)` for reads
- `map_io_error_for_write(error, path)` for writes

## Acceptance Checklist

After every TV task:
1. `just fmt` (auto-fixes style)
2. `just review` (runs tv-check + tv-clippy + tv-test)
3. Commit with detailed description
