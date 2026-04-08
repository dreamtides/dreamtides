# Staggered Support Effects Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement real staggered-grid support effects for Duskborne Sentry and
Veilward Knight in the battle prototype, then manually verify them in-browser.

**Architecture:** Duskborne Sentry is implemented as a derived spark modifier in
the battle-aware spark query path, not as stored state. Veilward Knight is
implemented as an end-of-turn battlefield mutation in a focused phase helper
module. Existing consumers that currently read `battle.cards.spark(...)` are
rerouted through the battle-aware query so combat, AI, scoring, and rendering
all see the same effective spark values.

**Tech Stack:** Rust (`battle_state`, `battle_queries`, `battle_mutations`,
`ai_uct`, `display`), React/Vite battle prototype for manual QA, `agent-browser`
CLI for adversarial verification

---

### Task 1: Add Battle-Aware Supported Spark Calculation

**Files:**

- Modify: `rules_engine/src/battle_queries/src/battle_card_queries/card_properties.rs`
- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`
- Modify: `rules_engine/src/battle_mutations/src/effects/apply_standard_effect.rs`
- Modify: `rules_engine/src/display/src/rendering/card_rendering.rs`
- Modify: `rules_engine/src/display/src/display_actions/outcome_simulation.rs`
- Modify: `rules_engine/src/battle_queries/src/battle_player_queries/player_properties.rs`
- Modify: `rules_engine/src/battle_queries/src/debug_snapshot/debug_battle_snapshot.rs`
- Modify: `rules_engine/src/ai_uct/src/decision_log.rs`
- Modify: `rules_engine/src/ai_uct/src/position_assignment.rs`
- Modify: `rules_engine/src/ai_uct/src/uct_search.rs`
- Modify: `rules_engine/src/ai_uct/src/uct_search_v3.rs`
- Modify: `rules_engine/src/ai_uct/src/uct_search_v4.rs`

- [ ] **Step 1: Add private helpers for supported-slot spark queries**

In `rules_engine/src/battle_queries/src/battle_card_queries/card_properties.rs`,
expand the module so it can find a character's battlefield slot, compare card
names, and compute the Duskborne Sentry bonus:

```rust
use battle_state::battle_cards::battlefield::Battlefield;

fn has_displayed_name(battle: &BattleState, id: CharacterId, expected: &str) -> bool {
    card::get_definition(battle, id).displayed_name == expected
}

fn front_slot(battle: &BattleState, controller: PlayerName, id: CharacterId) -> Option<usize> {
    battle.cards.battlefield(controller).front.iter().position(|slot| *slot == Some(id))
}

fn supported_sentry_bonus(
    battle: &BattleState,
    controller: PlayerName,
    front_slot: usize,
) -> Spark {
    let battlefield = battle.cards.battlefield(controller);
    let supporters = Battlefield::supporting_back_slots(front_slot, battle.rules_config.front_row_size);
    let bonus = supporters
        .into_iter()
        .filter_map(|slot| battlefield.back.get(slot).copied().flatten())
        .filter(|id| has_displayed_name(battle, *id, "Duskborne Sentry"))
        .count() as u32;
    Spark(bonus * 2)
}
```

- [ ] **Step 2: Make `card_properties::spark` return effective spark**

Still in `card_properties.rs`, replace the thin wrapper with derived spark
logic:

```rust
pub fn spark(battle: &BattleState, controller: PlayerName, id: CharacterId) -> Option<Spark> {
    let stored = battle.cards.spark(controller, id)?;
    let Some(slot) = front_slot(battle, controller, id) else {
        return Some(stored);
    };
    Some(stored + supported_sentry_bonus(battle, controller, slot))
}
```

This keeps stored spark unchanged for back-rank characters while giving
front-rank characters the real effective spark used by the prototype.

- [ ] **Step 3: Replace direct `battle.cards.spark(...)` reads with the battle-aware query**

Update the engine, display, and AI callers listed above to use
`card_properties::spark(...)` instead of `battle.cards.spark(...)`. For example,
in `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`:

```rust
use battle_queries::battle_card_queries::card_properties;

let attacker_spark = card_properties::spark(battle, opponent, attacker).unwrap_or_default();
let blocker_spark = card_properties::spark(battle, player, blocker).unwrap_or_default();
```

In AI files, replace expressions like:

```rust
battle.cards.spark(player, character).unwrap_or_default().0
```

with:

```rust
card_properties::spark(battle, player, character).unwrap_or_default().0
```

- [ ] **Step 4: Run formatter and typecheck**

Run: `just fmt`
Expected: Rust formatting and style fixes apply cleanly.

Run: `just check`
Expected: The rules engine compiles with the new battle-aware spark query.

- [ ] **Step 5: Commit**

```bash
git add rules_engine/src/battle_queries/src/battle_card_queries/card_properties.rs \
  rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs \
  rules_engine/src/battle_mutations/src/effects/apply_standard_effect.rs \
  rules_engine/src/display/src/rendering/card_rendering.rs \
  rules_engine/src/display/src/display_actions/outcome_simulation.rs \
  rules_engine/src/battle_queries/src/battle_player_queries/player_properties.rs \
  rules_engine/src/battle_queries/src/debug_snapshot/debug_battle_snapshot.rs \
  rules_engine/src/ai_uct/src/decision_log.rs \
  rules_engine/src/ai_uct/src/position_assignment.rs \
  rules_engine/src/ai_uct/src/uct_search.rs \
  rules_engine/src/ai_uct/src/uct_search_v3.rs \
  rules_engine/src/ai_uct/src/uct_search_v4.rs
git commit -m "feat: add supported spark calculation for Duskborne Sentry" -m \
  "Compute effective spark through the battle-aware query path and reroute combat, AI, scoring, and rendering consumers to use that value on the staggered grid."
```

### Task 2: Add Veilward Knight End-of-Turn Support Gain

**Files:**

- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/mod.rs`
- Create: `rules_engine/src/battle_mutations/src/phase_mutations/prototype_support_effects.rs`
- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/turn.rs`

- [ ] **Step 1: Add a focused phase helper module**

Create `rules_engine/src/battle_mutations/src/phase_mutations/prototype_support_effects.rs`
with a single public function:

```rust
use battle_queries::battle_card_queries::card;
use battle_queries::battle_trace;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CharacterId;
use battle_state::battle_cards::battlefield::Battlefield;
use battle_state::core::effect_source::EffectSource;
use core_data::numerics::Spark;
use core_data::types::PlayerName;

use crate::card_mutations::spark;

pub fn apply_end_of_turn_support_gains(
    battle: &mut BattleState,
    player: PlayerName,
    source: EffectSource,
) {
    let front_row: Vec<(usize, Option<CharacterId>)> = battle
        .cards
        .battlefield(player)
        .front
        .iter()
        .copied()
        .enumerate()
        .collect();

    for (front_slot, card_id) in front_row {
        let Some(card_id) = card_id else {
            continue;
        };
        if card::get_definition(battle, card_id).displayed_name != "Veilward Knight" {
            continue;
        }
        for back_slot in Battlefield::supporting_back_slots(front_slot, battle.rules_config.front_row_size) {
            let supported = battle.cards.battlefield(player).back[back_slot];
            let Some(supported) = supported else {
                continue;
            };
            battle_trace!(
                "Applying Veilward Knight support gain",
                battle,
                player,
                front_slot,
                back_slot,
                supported
            );
            spark::gain(battle, source, supported, Spark(1));
        }
    }
}
```

- [ ] **Step 2: Register the new module**

In `rules_engine/src/battle_mutations/src/phase_mutations/mod.rs`, add:

```rust
pub mod prototype_support_effects;
```

- [ ] **Step 3: Run the helper at the end-of-turn transition**

In `rules_engine/src/battle_mutations/src/phase_mutations/turn.rs`, import the
new module and call it in the `BattleTurnPhase::EndingPhaseFinished` arm before
the generic `Trigger::EndOfTurn(...)` is pushed:

```rust
use crate::phase_mutations::{
    dawn_phase, dreamwell_phase, fire_triggers, judgment_phase, prototype_support_effects,
};

BattleTurnPhase::EndingPhaseFinished => {
    battle.phase = BattleTurnPhase::FiringEndOfTurnTriggers;
    let source = EffectSource::Game { controller: battle.turn.active_player };
    prototype_support_effects::apply_end_of_turn_support_gains(
        battle,
        battle.turn.active_player,
        source,
    );
    battle.triggers.push(source, Trigger::EndOfTurn(battle.turn.active_player));
    apply_effect::execute_pending_effects_if_no_active_prompt(battle);
    fire_triggers::execute_if_no_active_prompt(battle);
}
```

- [ ] **Step 4: Run formatter and full review gate**

Run: `just fmt`
Expected: Formatting succeeds.

Run: `just review`
Expected: The repository review gate passes, or any failures are understood and
fixed before continuing.

- [ ] **Step 5: Commit**

```bash
git add rules_engine/src/battle_mutations/src/phase_mutations/mod.rs \
  rules_engine/src/battle_mutations/src/phase_mutations/prototype_support_effects.rs \
  rules_engine/src/battle_mutations/src/phase_mutations/turn.rs
git commit -m "feat: add Veilward Knight end-of-turn support gains" -m \
  "Apply the staggered-grid back-rank support spark gain at the end-of-turn transition for Veilward Knight, with stacking behavior preserved by resolving each knight independently."
```

### Task 3: Manual QA the Prototype Support Effects

**Files:**

- Report output: `/tmp/qa-staggered-support-effects/qa-report.md`
- Screenshot output: `/tmp/qa-staggered-support-effects/*.png`

- [ ] **Step 1: Start the backend and battle prototype**

Run backend in one terminal:

```bash
just dev
```

Expected: the rules engine dev server starts on `http://localhost:26598`.

Run UI in another terminal:

```bash
just battle-dev
```

Expected: the Vite battle prototype starts on `http://localhost:5174`.

- [ ] **Step 2: Exercise the staggered-grid prototype in-browser**

Use `agent-browser` against:

```text
http://localhost:5174/?front=4&back=5
```

Target scenarios:

- place Duskborne Sentry in an edge back slot and confirm exactly one front-row
  character gets `+2 spark`
- place Duskborne Sentry in a middle back slot and confirm exactly two
  front-row characters get `+2 spark`
- overlap two Sentries on the same front-row target and confirm `+4 spark`
- place Veilward Knight in an edge front slot and confirm the supported
  back-row character gains `+1 spark` at end of turn
- place Veilward Knight in a middle front slot and confirm two supported
  back-row characters gain `+1 spark`
- overlap two Knights and confirm a shared back-row character gains `+2 spark`
- verify at least one judgment outcome changes because of the modified spark

- [ ] **Step 3: Capture the QA report**

Write the report to:

```text
/tmp/qa-staggered-support-effects/qa-report.md
```

The report must include:

- invariants tracked for effective spark values on affected characters
- any bugs or anomalies found
- the screenshots proving the stacked and edge/middle cases

- [ ] **Step 4: Commit the implementation after QA**

```bash
git add -A
git commit -m "feat: prototype staggered support effects" -m \
  "Implement Duskborne Sentry supported spark bonuses and Veilward Knight end-of-turn support gains for the staggered-grid battle prototype, then validate the behavior through manual browser QA."
```
