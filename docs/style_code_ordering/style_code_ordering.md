# Style & Code Ordering Rules

The Dreamtides rules engine enforces a comprehensive set of style rules via a
custom `style_validator` binary, workspace-level clippy lints, and nightly
rustfmt configuration. These checks run during `just fmt` (auto-fixing where
possible) and `just review` (validation-only). Most `just review` failures for
new contributors stem from violating these conventions.

## Table of Contents

- [Item Ordering Within Files](#item-ordering-within-files)
- [Spacing Between Items](#spacing-between-items)
- [Naming Qualification Rules](#naming-qualification-rules)
- [Import Conventions](#import-conventions)
- [Module File Restrictions](#module-file-restrictions)
- [Test Location Rules](#test-location-rules)
- [Cargo.toml Dependency Ordering](#cargotoml-dependency-ordering)
- [Workspace Dependency Enforcement](#workspace-dependency-enforcement)
- [Doc Comment Link Validation](#doc-comment-link-validation)
- [Clippy Configuration](#clippy-configuration)
- [The allow_attributes Deny Rule](#the-allow_attributes-deny-rule)
- [Rustfmt Configuration](#rustfmt-configuration)
- [RLF Formatting and Linting](#rlf-formatting-and-linting)
- [Rust 2024 Edition Conventions](#rust-2024-edition-conventions)
- [The Review Pipeline](#the-review-pipeline)
- [Auto-Fix Summary](#auto-fix-summary)

## Item Ordering Within Files

The style validator enforces a strict ordering of items within each Rust source
file. Items must appear in the following sequence (enforced via derived `Ord` on
the `ItemCategory` enum discriminants):

01. **Private constants** (`const` without `pub`)
02. **Private statics** (`static` without `pub`)
03. **`thread_local!` macro invocations**
04. **Public type aliases** (`pub type`)
05. **Public constants** (`pub const`)
06. **Public traits** (`pub trait`)
07. **Public structs and enums** (`pub struct`, `pub enum`)
08. **Public functions** (`pub fn`)
09. **Private items** (everything else: private functions, private structs,
    private enums, private traits, private type aliases, impl blocks)
10. **Test modules** (`#[cfg(test)] mod ...`)

The key surprise for newcomers is that private constants and statics come
*before* all public items. The validator tracks a "current phase" that advances
monotonically through these categories — any item whose category is earlier than
the current phase triggers a violation.

Items not subject to ordering: `use` statements and non-test `mod` declarations
are skipped entirely by the ordering check. (`use` statement ordering is handled
by rustfmt, not the style validator.)

Both `just fmt` and `just review` pass the `--code-order` flag, so this check is
always active in practice. `just fmt` additionally passes `--fix` to
auto-correct spacing violations.

## Spacing Between Items

The validator enforces exactly one blank line between consecutive code items,
with one exception: consecutive constants (both private or both public) do not
require a blank line between them. A blank line is also required after the last
`use` statement before the first code item.

The `--fix` flag can auto-insert missing blank lines. It does not reorder items
— only spacing is auto-fixable for code ordering violations.

## Naming Qualification Rules

This is the most frequently violated convention and the one explicitly called
out as "the #1 agent error" in the project instructions. The style validator
programmatically enforces qualification counts on all paths using `syn` AST
walking.

**Function calls — exactly one qualifier:**

- Correct: `move_card::to_destination_zone()`, `energy::spend()`
- Wrong: `crate::zone_mutations::move_card::to_destination_zone()` (too many)
- Wrong: `to_destination_zone()` (zero qualifiers on a cross-file function)

Same-file function calls with zero qualifiers are permitted. For any function
defined in a different module, exactly one qualifier is required — import the
parent module and call `module::function()`. Bare calls to cross-file functions
and fully-qualified paths with multiple qualifiers are both violations.

**Type names — zero qualifiers:**

- Correct: `BattleState`, `PlayerName`, `EffectSource`
- Wrong: `battle_state::BattleState`, `core_data::PlayerName`

Types must be imported via `use` statements and then referenced unqualified.

**Enum variants — exactly one qualifier:**

- Correct: `Zone::Battlefield`, `Effect::List(...)`, `Trigger::PlayedCard(...)`
- Wrong: `battle_cards::Zone::Battlefield` (too many qualifiers)

The validator identifies function calls (lowercase last segment), enum variants
(PascalCase with 2+ segments), and type positions (PascalCase in type context)
and applies the appropriate rule to each.

Exemptions exist for standard library paths (`std`, `core`, `alloc`), prelude
items (`Some`, `None`, `Ok`, `Err`), trait methods (a hardcoded allowlist
including `fmt`, `from`, `into`, `default`, and others), associated types, and
generic type parameters.

**Direct function imports are banned:** You cannot write
`use crate::effects::apply_effect;` and then call `apply_effect()` bare. The
validator cross-references all `use` statements against known public functions
across the entire codebase. You must import the module and call
`apply_effect::execute()`. Two module paths are allowlisted:
`parser::parser_utils` and `parser::parser::parser_helpers`.

## Import Conventions

**`crate::` required, `super::` and `self::` banned:** All same-crate imports
must use `use crate::` paths. The validator detects `super::` and `self::` paths
and can auto-fix them by computing the equivalent `crate::` path from the file's
position relative to the crate root.

**All `use` statements at file top:** `use` declarations inside function bodies,
impl blocks, trait blocks, or nested modules trigger a violation. The auto-fixer
extracts inline `use` statements, deduplicates against existing top-level
imports, and moves them to the top of the file.

**`pub use` banned:** Re-exports via `pub use` are not permitted. All imports
must come from their original file location. One file is exempted:
`test_session_prelude.rs` (a test utility prelude). The auto-fixer downgrades
`pub use` to plain `use`.

**Rustfmt grouping:** The `.rustfmt.toml` configuration sets
`group_imports = "StdExternalCrate"` and `imports_granularity = "Module"`, which
groups imports into standard library, external crate, and local crate sections,
merging imports from the same module.

## Module File Restrictions

Files named `mod.rs` or `lib.rs` may only contain `mod` declarations (the
`mod foo;` form, not inline `mod foo { ... }`) and `use` statements. Any other
item — functions, structs, enums, impls, consts, statics, traits, type aliases —
triggers a violation. This keeps module files as pure organizational manifests.

## Test Location Rules

Three separate checks enforce test placement:

**No inline `mod tests {}` blocks:** Inline test modules with content are
banned. Tests must live as integration tests under `rules_engine/tests/`. Two
files are whitelisted: `parser/src/error/parser_error_suggestions.rs` and
`battle_state/src/battle_cards/card_set.rs`.

**No `tests/` directories under `src/`:** The validator walks
`rules_engine/src/` and flags any directory named `tests`. All test code belongs
in the top-level `rules_engine/tests/` directory, which contains five test
crates: `battle_tests`, `parser_tests`, `tabula_cli_tests`, `tabula_data_tests`,
and `tv_tests`.

**Test file naming:** Under `rules_engine/tests/`, test files must end in
`_tests.rs` (not `_test.rs`). Helper and utility files are exempted.

## Cargo.toml Dependency Ordering

Dependencies in `[dependencies]` must appear in two alphabetized groups:

1. **Internal crates first** — those with a `path` key (e.g.,
   `ability_data = { path = "../ability_data" }`)
2. **External crates second** — those with `workspace = true` (e.g.,
   `serde = { workspace = true }`)

Within each group, entries must be alphabetically sorted by crate name. The
convention is to separate the two groups with a blank line, though the validator
only enforces ordering, not the blank line itself.

The auto-fixer can sort and reorder dependencies in-place.

## Workspace Dependency Enforcement

For all Cargo.toml files under `src/` (excluding the `tv/` Tauri app directory),
external dependencies must use `workspace = true` rather than specifying
versions directly. This centralizes version management in the root
`rules_engine/Cargo.toml`. Path dependencies (internal crates) are exempt from
this rule.

## Doc Comment Link Validation

If a doc comment contains a bracketed type reference like `[TypeName]`, that
type must be either imported via a `use` statement, defined locally in the same
file, or be a built-in type from an allowlist (`String`, `Vec`, `Option`,
`Result`, `Box`, `Arc`, `Rc`, `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`,
`Cell`, `RefCell`). This prevents broken intra-doc links.

## Clippy Configuration

The workspace `[workspace.lints.clippy]` section in the root Cargo.toml denies
approximately 30 clippy lints. All are set to `deny` (compile error, not
warning). Notable lints include:

- `absolute_paths` — denied, with `absolute-paths-max-segments = 3` configured
  in `clippy.toml`, so paths with more than 3 segments are flagged
- `allow_attributes` — denied (see next section)
- `enum_glob_use` — no glob-importing enum variants
- `unnested_or_patterns` — denied workspace-wide, requiring `#[expect()]` on
  chumsky `select!` macro sites
- `implicit_clone`, `redundant_closure_for_method_calls`,
  `semicolon_if_nothing_returned` — general code quality enforcement

The `just clippy` recipe additionally passes `-D warnings -D clippy::all` to
deny all warnings and all default clippy lints beyond the workspace
configuration.

Individual crates inherit these workspace lints via `[lints] workspace = true`
in their own Cargo.toml files.

## The allow_attributes Deny Rule

The `allow_attributes` clippy lint is denied at the workspace level. This means
any use of `#[allow(...)]` is itself a compile error. The only way to suppress a
lint is via `#[expect(...)]`, which additionally warns if the expected lint
never fires (catching stale suppressions).

This is directly relevant to the chumsky `select!` macro issue: the macro
expansion triggers `unnested_or_patterns`, and the suppression must use
`#[expect(clippy::unnested_or_patterns)]`, not `#[allow()]`. Currently two
parser helper functions in `parser/src/parser/parser_helpers.rs` carry this
annotation.

The `tv/` Tauri app directory is excluded from style validation and does contain
a few `#[allow(dead_code)]` annotations.

## Rustfmt Configuration

The project uses nightly rustfmt (invoked via `cargo +nightly fmt`) with a
`.rustfmt.toml` that enables several unstable formatting options:

- `group_imports = "StdExternalCrate"` — groups imports into std/external/crate
  sections
- `imports_granularity = "Module"` — merges imports from the same module
- `reorder_impl_items = true` — sorts items within impl blocks
- `normalize_comments = true` and `wrap_comments = true` — reformats comments
- `use_small_heuristics = "Max"` — maximizes items per line
- `newline_style = "Unix"` — enforces LF line endings
- Generated code in `src/tabula_generated/` is excluded from formatting

The `just review` recipe runs `cargo +nightly fmt -- --check` to verify
formatting without modifying files.

## RLF Formatting and Linting

Two custom binaries handle the RLF localization system:

**`rlf_fmt`** formats both `.rlf` locale files and the `rlf!` macro body in
`strings.rs`, using a max width of 100 characters and 4-space indentation. It
runs as part of `just fmt`.

**`rlf_lint`** runs the `rlf` library's built-in lint checker against the
project's registered string definitions. It runs as part of `just review`.

## Rust 2024 Edition Conventions

The project uses Rust edition 2024 and expects new code to take advantage of its
features. A common error in AI-generated code is writing pre-2024 Rust idioms
when modern alternatives exist.

**Let chains:** Use `if let ... && let ...` chains instead of nested `if let`
blocks. For example, prefer
`if let Some(x) = a && let Effect::List(list) = x { ... }` over
`if let Some(x) = a { if let Effect::List(list) = x { ... } }`. This flattens
control flow and reduces indentation.

**Inline variables in format strings:** Use `format!("{variable}")` and
`println!("{value}")` instead of `format!("{}", variable)` or
`println!("{}", value)`. Inline captured identifiers directly into format string
placeholders whenever the expression is a simple variable name.

**`gen` blocks and other 2024 features:** Where applicable, prefer the idiomatic
2024 edition patterns over older workarounds. When in doubt, match the style
used in surrounding code.

## The Review Pipeline

`just review` is the pre-push validation gate. It runs the following checks in
sequence (taking approximately 5 minutes):

01. Pending snapshot check
02. Rustfmt format verification (nightly, check-only)
03. Markdown documentation format check
04. Token limit check (for LLM context sizes)
05. Full workspace build
06. Clippy (workspace lints + all warnings denied)
07. Style validator (all 16 checks including code ordering, check-only)
08. RLF lint
09. Core tests (all workspace tests except parser and TV)
10. Parser tests (separate due to stack size requirements)
11. TV app checks (TypeScript type checking, ESLint, Rust clippy, tests)

A scope-aware review runner (`review_runner.py`) can skip unchanged steps for
faster iteration.

## Auto-Fix Summary

Running `just fmt` auto-fixes the following violations:

| Check                       | What gets fixed                            |
| --------------------------- | ------------------------------------------ |
| `pub use` statements        | Downgraded to plain `use`                  |
| Inline `use` statements     | Extracted to file top, deduplicated        |
| `super::`/`self::` imports  | Converted to equivalent `crate::` paths    |
| Code spacing                | Missing blank lines inserted between items |
| Cargo.toml dependency order | Internal deps sorted first, then external  |

Checks that require manual fixes: naming qualification (qualifier counts),
module file restrictions, doc comment links, inline test modules, test directory
placement, test file naming, workspace dependency enforcement, and direct
function imports.
