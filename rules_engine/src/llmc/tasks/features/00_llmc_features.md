---
lattice-id: LBYWQN
name: 00-llmc-features
description: LLMC features template
parent-id: LBSWQN
created-at: 2026-01-21T19:06:10.982040Z
updated-at: 2026-01-21T22:38:24.942563Z
---

# [Lattice] Context

You are working on implementing the below for the `llmc` tool.

See `@rules_engine/docs/llmc.md` for broader context on the llmc system.

See llmc code at: `@rules_engine/src/llmc/`

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

- Function calls and enum values: exactly one qualifier (e.g.,
  `move_card::run()`)
- Struct/enum type names: zero qualifiers (e.g., `BattleState`, not
  `battle::BattleState`)
- No `pub use` declarations
- No code in `mod.rs`/`lib.rs` except module declarations
- No `use` declarations inside function bodies

## Code Review

Review your work against @rules_engine/docs/lattice/appendix_code_review.md.

Critical points:

- Write high-quality, non duplicative tests
- Create high-quality, useful logs via `tracing`
- Functions < 50 lines, files < 500 lines
- Aggressively refactor code to remove duplication
