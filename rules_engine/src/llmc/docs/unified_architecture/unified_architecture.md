---
lattice-id: LRMWQN
name: unified-architecture
description: |-
  Migration design from dual-repository LLMC architecture to unified worktree
  architecture for simplified git operations and improved reliability.
created-at: 2026-01-19T05:00:00Z
updated-at: 2026-01-21T22:38:47.395069Z
---

# LLMC Unified Architecture: Technical Design

This document describes the migration of LLMC from a dual-repository
architecture
(separate clone + source repo) to a unified architecture where git worktrees are
created directly from the main repository. This simplification eliminates sync
issues, reduces complexity, and provides a cleaner mental model while
maintaining
all existing functionality.

## Key Changes

- Remove the intermediate git clone at `~/llmc/`
- Create worktrees directly from the source repository
- Simplify the accept flow to merge directly to master
- Add robust recovery tools for edge cases

## Non-Goals

- Changing the worker state machine
- Modifying the TMUX integration
- Altering the patrol system's responsibilities (only its implementation)

## Document Index

This design is split into the following sections:

- [Architecture Comparison](../../../lattice/.closed/please_investigate_following_lattice_iss.md#LBVWQN)
  - Current vs
  new architecture diagrams and comparison
- [Detailed Design](../../../lattice/tasks/features/00_lattice_feature_template.md#LBWWQN)
  - Repository concepts, key
  decisions, and state file changes
- [Configuration Changes](../../../lattice/tasks/features/.closed/i_would_like_lat_create_--interactive_be.md#LBXWQN)
  - New config
  schema and resolution logic
- [Code Changes](../../tasks/features/00_llmc_features.md#LBYWQN) - File-by-file
  implementation
  guide
- [Git Operation Safety](../../tasks/.closed/please_restructure_llmc_daemon_output_be.md#LBZWQN)
  - Pre-operation
  checks and atomic operations
- [Edge Cases](../../tasks/features/.closed/when_there_are_no_tasks_available_from_t.md#LB2WQN)
  - Git state edge cases and failure
  recovery
- [Recovery Tools](../../../../.closed/please_create_file_named_bananatesttxt_w.md#LB3WQN)
  - Doctor, salvage, and rescue
  commands
- [Migration Plan](../../../../.closed/delete_banana_testtxt_file.md#LB4WQN) -
  Steps and verification
  checklist
- [Session Specifications](../../tasks/scenarios/00_llmc_scenarios.md#LB5WQN) -
  Opus session
  prompts
- [Testing Procedures](../../tasks/qa/00_llmc_qa_task.md#LB6WQN) - Unit tests
  and
  integration testing checklist
- [Logging Specification](docs/logging_specification.md#LB7WQN) - Log levels and
  structured fields
- [Rollback Plan](docs/rollback_plan.md#LCAWQN) - Recovery if migration fails
