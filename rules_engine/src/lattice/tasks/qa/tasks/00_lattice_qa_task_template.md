---
lattice-id: LC6WQN
name: 00-lattice-qa-task-template
description: Template for lattice QA tasks
parent-id: LCEWQN
created-at: 2026-01-20T14:25:07.705826Z
updated-at: 2026-01-21T22:31:38.547070Z
---

# [Lattice] Context

Please investigate the following Lattice issue.

Primary design doc: @rules_engine/docs/lattice/lattice_design.md

Code: @rules_engine/src/lattice/

## Instructions

Please do the following:

1) Start by *creating a test* which reproduces the bug, BEFORE trying to fix it
2) Report any *other* issues you notice via `lattice_create_task()` to
   rules_engine/src/lattice/tasks
3) Think one level deeper: How can we prevent this *category* of problems? How
   can the system be overall more robust / self-healing?

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

- Expected errors → `LatticeError`; system errors → `panic!` with reason
- Write high-quality, non duplicative tests
- Create high-quality, useful logs via `tracing`
- Functions < 50 lines, files < 500 lines
- Aggressively refactor code to remove duplication
