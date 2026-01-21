---
lattice-id: LBSWQN
name: 00-auto-overseer-tasks
description: Task template for auto overseer
created-at: 2026-01-21T04:14:09.630622Z
updated-at: 2026-01-21T04:14:09.630657Z
---

# [Lattice] Context

You are working on implementing featuers for the llmc 'auto' and 'overseer' functionality.

Please run `lat show LDBWQN` to view the primary design document.

llmc code: `@rules_engine/src/llmc/`

See `@rules_engine/docs/llmc.md` for broader context on the llmc system.

Please report any *other* issues you notice via the `lattice_create_task()` MCP
tool, in the `rules_engine/src/llmc` parent directory

# [Lattice] Acceptance Criteria

When complete, please:

1) Run `just fmt` to format
2) Run `just review` to run lint & tests
3) Verify if documentation updates are required
4) Verify if logging could be improved to improve diagnostics here
5) Run `lat close <ID>` to mark the issue as complete
6) Create a git commit with a description of your work

## Code Style

Follow all rules in AGENTS.md. Key project-specific conventions:
- Function calls and enum values: exactly one qualifier (e.g., `move_card::run()`)
- Struct/enum type names: zero qualifiers (e.g., `BattleState`, not `battle::BattleState`)
- No `pub use` declarations
- No code in `mod.rs`/`lib.rs` except module declarations
- No `use` declarations inside function bodies

## Code Review

Review your work against @rules_engine/docs/lattice/appendix_code_review.md.

Critical points:
- Expected errors → `LatticeError`; system errors → `panic!` with reason
- Write high-quality, non duplicative tests
- Create high-quality, useful logs via `tracing`
- Functions < 50 lines, files < 500 lines
- Aggressively refactor code to remove duplication
