# MCTS Positioning Redesign Implementation Plan

**Status:** Implemented

**Goal:** Replace the combinatorially-explosive multi-step MCTS positioning flow
with heuristic-generated atomic position assignments, evaluated as root-level
MCTS actions.

**Architecture:** A new `MonteCarloV2` GameAI variant dispatches to a new search
module (`uct_search_v2.rs`) that generates up to 6 candidate position
assignments via heuristics, evaluates them alongside standard card-play actions
via MCTS, and uses greedy heuristic rollouts with atomic positioning for both
players. The existing `MonteCarlo` variant and `uct_search.rs` are untouched.

**Spec:** `docs/superpowers/specs/2026-04-07-mcts-positioning-redesign.md`

## Tasks (all completed)

1. Add MonteCarloV2 GameAI variant and stub search
2. Position assignment data model and heuristic generator
3. Apply assignment helper and greedy rollout turn
4. Implement V2 search core
5. Position assignment action sequencing
6. Integration test and compilation fixes
7. Head-to-head evaluation

## File Structure

| File                                                 | Action | Purpose                                 |
| ---------------------------------------------------- | ------ | --------------------------------------- |
| `rules_engine/src/ai_data/src/game_ai.rs`            | Modify | Add `MonteCarloV2(u32)` variant         |
| `rules_engine/src/ai_agents/src/agent_search.rs`     | Modify | Dispatch V2, assignment sequencing      |
| `rules_engine/src/ai_uct/src/lib.rs`                 | Modify | Register new modules                    |
| `rules_engine/src/ai_uct/src/uct_search_v2.rs`       | Create | V2 MCTS search with atomic assignments  |
| `rules_engine/src/ai_uct/src/position_assignment.rs` | Create | Assignment data model and generator     |
| `rules_engine/src/ai_uct/src/rollout_policy.rs`      | Create | Apply assignment helper, greedy rollout |
| `rules_engine/tests/.../basic_uct_search_tests.rs`   | Modify | Add V2 basic test                       |
