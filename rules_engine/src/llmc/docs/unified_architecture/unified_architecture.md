---
lattice-id: LBUWQN
name: unified-architecture
description: |-
  Migration design from dual-repository LLMC architecture to unified worktree
  architecture for simplified git operations and improved reliability.
created-at: 2026-01-19T05:00:00Z
updated-at: 2026-01-19T05:08:18.140125Z
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

- [Architecture Comparison](docs/architecture_comparison.md#LBVWQN) - Current vs
  new architecture diagrams and comparison
- [Detailed Design](docs/detailed_design.md#LBWWQN) - Repository concepts, key
  decisions, and state file changes
- [Configuration Changes](docs/configuration_changes.md#LBXWQN) - New config
  schema and resolution logic
- [Code Changes](docs/code_changes.md#LBYWQN) - File-by-file implementation
  guide
- [Git Operation Safety](docs/git_operation_safety.md#LBZWQN) - Pre-operation
  checks and atomic operations
- [Edge Cases](docs/edge_cases.md#LB2WQN) - Git state edge cases and failure
  recovery
- [Recovery Tools](docs/recovery_tools.md#LB3WQN) - Doctor, salvage, and rescue
  commands
- [Migration Plan](docs/migration_plan.md#LB4WQN) - Steps and verification
  checklist
- [Session Specifications](docs/session_specifications.md#LB5WQN) - Opus session
  prompts
- [Testing Procedures](docs/testing_procedures.md#LB6WQN) - Unit tests and
  integration testing checklist
- [Logging Specification](docs/logging_specification.md#LB7WQN) - Log levels and
  structured fields
- [Rollback Plan](docs/rollback_plan.md#LCAWQN) - Recovery if migration fails
