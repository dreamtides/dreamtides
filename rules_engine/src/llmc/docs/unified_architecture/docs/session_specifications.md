---
lattice-id: LB5WQN
name: session-specifications
description: Opus session specifications with prompts for three-phase implementation.
parent-id: LBUWQN
created-at: 2026-01-19T05:00:00Z
updated-at: 2026-01-19T05:08:18.126628Z
---

# Opus Session Specifications

## Session 1: Infrastructure Changes

**Estimated Scope:** ~60% of total work

**Objectives:**

1. Update `config.rs` with new helper methods
2. Rewrite `commands/init.rs` completely
3. Update `commands/add.rs` for new paths
4. Update `commands/nuke.rs` for new paths
5. Update `commands/reset.rs` for new paths
6. Add new git safety functions to `git.rs`
7. Basic testing of init/add/nuke cycle

**Success Criteria:**

- [ ] `just check` passes
- [ ] `just fmt` produces no changes
- [ ] `llmc init` creates metadata-only directory
- [ ] `llmc add` creates worktree in source repo
- [ ] `llmc nuke` cleanly removes worker
- [ ] No `origin/master` references in modified files
- [ ] All new git safety functions implemented

## Session 2: Accept Flow and Patrol

**Estimated Scope:** ~40% of total work

**Objectives:**

1. Simplify `commands/accept.rs` completely
2. Update `patrol.rs` to use `master` instead of `origin/master`
3. Update `commands/start.rs` path references
4. Update `commands/rebase.rs`
5. Update `commands/review.rs` if needed
6. Update `commands/doctor.rs` for new architecture
7. End-to-end testing

**Success Criteria:**

- [ ] `just check` passes
- [ ] `just clippy` has no warnings
- [ ] Full workflow test passes
- [ ] No `origin/master` references remain in codebase
- [ ] No `fetch_origin` calls remain
- [ ] Accept with dirty repo fails gracefully
- [ ] Patrol runs without errors
- [ ] `llmc doctor` reports system healthy

## Session 3: Recovery Tools and Documentation

**Estimated Scope:** Polish and hardening

**Objectives:**

1. Implement `llmc salvage` command
2. Implement `llmc rescue` command
3. Enhance `llmc doctor --repair`
4. Add self-healing to patrol
5. Update `llmc.md` documentation
6. Final testing and edge case handling

**Success Criteria:**

- [ ] `llmc salvage` works for patches and branches
- [ ] `llmc rescue` recovers from broken state
- [ ] `llmc doctor --rebuild` reconstructs state
- [ ] Patrol self-heals common issues
- [ ] `llmc.md` fully updated
- [ ] All edge cases documented and handled
