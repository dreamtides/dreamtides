---
lattice-id: LCTWQN
name: dependency-commands-feature---lat-dep-ad
description: Dependency Commands Feature - lat dep add/remove/tree
task-type: feature
priority: 1
labels:
- lattice
created-at: 2026-01-19T20:04:54.001404Z
updated-at: 2026-01-19T20:07:05.882323Z
closed-at: 2026-01-19T20:07:05.882323Z
---

## Goal

Implement the `lat dep` subcommands for managing task dependencies:
- `lat dep add` - Add a dependency relationship
- `lat dep remove` - Remove a dependency relationship
- `lat dep tree` - Display dependency tree visualization

## References

- Feature specification: @rules_engine/docs/lattice/lattice_implementation_plan.md (Feature: Dependency Commands)
- CLI specification: @rules_engine/docs/lattice/appendix_cli_structure.md (lat dep commands)
- Task tracking: @rules_engine/docs/lattice/appendix_task_tracking.md (Dependencies section)

## Dependencies

- Depends on: Task System (DONE) - provides `dependency_graph.rs`
- Depends on: Document Model (DONE) - frontmatter manipulation

## Scope

- Create `rules_engine/src/lattice/src/cli/commands/dep_command.rs`
- Create tests in `tests/lattice/commands/dep_tests.rs`
- Integrate with existing `task/dependency_graph.rs` for graph operations
- Support `--json` output for all subcommands
- Handle cycle detection on add operations

## Closure Reason

Created by mistake - using beads instead
