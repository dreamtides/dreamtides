# Start-of-Turn Judgment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore a single start-of-turn `Judgment` phase, remove `Dawn`, and
make the active player's front-rank characters attack during their own
`Judgment`.

**Architecture:** The change has three coupled layers: the battle turn state
machine, the trigger/parser/string surface, and the card-data/docs/test surface.
Implement the engine timing and trigger rename first so the parser and generated
data can target one stable model, then regenerate and verify the repo with the
normal `just` flow.

**Tech Stack:** Rust rules engine, parser/serializer pipeline, RLF localization
strings, TOML card data, Rust integration/parser tests

______________________________________________________________________

## File Structure

- Modify: `rules_engine/src/battle_state/src/battle/battle_turn_phase.rs`
- Modify: `rules_engine/src/battle_state/src/triggers/trigger.rs`
- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/turn.rs`
- Modify:
  `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`
- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/mod.rs`
- Delete: `rules_engine/src/battle_mutations/src/phase_mutations/dawn_phase.rs`
- Modify:
  `rules_engine/src/battle_queries/src/card_ability_queries/trigger_queries.rs`
- Modify:
  `rules_engine/src/battle_queries/src/battle_card_queries/card_abilities.rs`
- Modify: `rules_engine/src/ability_data/src/trigger_event.rs`
- Modify: `rules_engine/src/parser/src/parser/trigger_parser.rs`
- Modify: `rules_engine/src/parser/src/serializer/trigger_serializer.rs`
- Modify: `rules_engine/src/parser/src/parser/effect/game_effects_parsers.rs`
- Modify: `rules_engine/src/parser/src/variables/parser_substitutions.rs`
- Modify: `rules_engine/src/strings/src/strings.rlf.rs`
- Modify: `rules_engine/src/strings/locales/bracket.rlf`
- Modify: `rules_engine/src/strings/locales/ru.rlf`
- Modify: `client/Assets/StreamingAssets/Tabula/cards.toml`
- Modify: `docs/battle_rules/battle_rules.md`
- Test:
  `rules_engine/tests/battle_tests/tests/battle_tests/basic_tests/turn_sequence_tests.rs`
- Test: `rules_engine/tests/parser_tests/tests/game_effects_parser_tests.rs`
- Test: `rules_engine/tests/parser_tests/tests/lexer_tests.rs`
- Test:
  `rules_engine/tests/parser_tests/tests/round_trip_tests/judgment_ability_round_trip_tests.rs`

### Task 1: Rebuild The Turn State Machine Around Start-of-Turn Judgment

**Files:**

- Modify: `rules_engine/src/battle_state/src/battle/battle_turn_phase.rs`

- Modify: `rules_engine/src/battle_state/src/triggers/trigger.rs`

- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/turn.rs`

- Modify:
  `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`

- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/mod.rs`

- Delete: `rules_engine/src/battle_mutations/src/phase_mutations/dawn_phase.rs`

- Test:
  `rules_engine/tests/battle_tests/tests/battle_tests/basic_tests/turn_sequence_tests.rs`

- [ ] **Step 1: Write a failing turn-sequence test for start-of-turn Judgment**

```rust
#[test]
fn next_turn_starts_with_judgment_before_draw() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, test_card::TEST_DRAW_ONE);
    s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_DRAW_ONE);

    s.click_primary_button(DisplayPlayer::User, "End Turn");
    s.click_primary_button(DisplayPlayer::Enemy, "Next Turn");

    assert_eq!(
        s.user_client.opponent.view.as_ref().unwrap().phase_name,
        "Judgment",
        "enemy turn starts in Judgment before Dreamwell/Draw/Main"
    );
}
```

- [ ] **Step 2: Run the battle test to verify the current model still reports
  Dawn/Main timing**

Run: `just battle-test turn_sequence_tests`

Expected: FAIL because the current turn sequence still includes `Dawn` and does
not begin the turn in `Judgment`.

- [ ] **Step 3: Remove `Dawn` from the phase enum and transition order**

```rust
pub enum BattleTurnPhase {
    Starting,
    Judgment,
    Dreamwell,
    Draw,
    Main,
    Ending,
    EndingPhaseFinished,
    FiringEndOfTurnTriggers,
}
```

```rust
BattleTurnPhase::Starting => {
    battle.phase = BattleTurnPhase::Judgment;
    let player = battle.turn.active_player;
    let source = EffectSource::Game { controller: player };
    battle.turn.judgment_position = 0;
    battle.turn.judgment_participants.clear();
    battle.triggers.push(source, Trigger::Judgment(player));
    apply_effect::execute_pending_effects_if_no_active_prompt(battle);
    fire_triggers::execute_if_no_active_prompt(battle);
}
BattleTurnPhase::Judgment => {
    let player = battle.turn.active_player;
    let source = EffectSource::Game { controller: player };
    let finished = judgment_phase::run(battle, player, source);
    apply_effect::execute_pending_effects_if_no_active_prompt(battle);
    fire_triggers::execute_if_no_active_prompt(battle);
    if finished && battle.prompts.is_empty() {
        battle.phase = BattleTurnPhase::Dreamwell;
    }
}
```

- [ ] **Step 4: Flip combat so the active player attacks and remove the
  post-combat trigger push**

```rust
let attacker_id = battle.cards.battlefield(player).front[position as usize];
let blocker_id = battle.cards.battlefield(opponent).front[position as usize];
```

```rust
match keyword {
    Trigger::Judgment(player) => owning_card_controller == player,
    _ => false,
}
```

Delete `rules_engine/src/battle_mutations/src/phase_mutations/dawn_phase.rs` and
remove its module export.

- [ ] **Step 5: Run the focused battle test again**

Run: `just battle-test turn_sequence_tests`

Expected: PASS with the new `Judgment -> Dreamwell -> Draw -> Main -> Ending`
order.

- [ ] **Step 6: Commit the engine timing change**

```bash
git add rules_engine/src/battle_state/src/battle/battle_turn_phase.rs \
  rules_engine/src/battle_state/src/triggers/trigger.rs \
  rules_engine/src/battle_mutations/src/phase_mutations/turn.rs \
  rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs \
  rules_engine/src/battle_mutations/src/phase_mutations/mod.rs \
  rules_engine/src/battle_queries/src/card_ability_queries/trigger_queries.rs \
  rules_engine/src/battle_queries/src/battle_card_queries/card_abilities.rs \
  rules_engine/tests/battle_tests/tests/battle_tests/basic_tests/turn_sequence_tests.rs
git commit -m "feat: restore start-of-turn judgment timing"
```

### Task 2: Rename Trigger And Parser Surface From Dawn Back To Judgment

**Files:**

- Modify: `rules_engine/src/ability_data/src/trigger_event.rs`

- Modify: `rules_engine/src/parser/src/parser/trigger_parser.rs`

- Modify: `rules_engine/src/parser/src/serializer/trigger_serializer.rs`

- Modify: `rules_engine/src/parser/src/parser/effect/game_effects_parsers.rs`

- Modify: `rules_engine/src/parser/src/variables/parser_substitutions.rs`

- Modify: `rules_engine/src/strings/src/strings.rlf.rs`

- Modify: `rules_engine/src/strings/locales/bracket.rlf`

- Modify: `rules_engine/src/strings/locales/ru.rlf`

- Test: `rules_engine/tests/parser_tests/tests/game_effects_parser_tests.rs`

- Test: `rules_engine/tests/parser_tests/tests/lexer_tests.rs`

- Test:
  `rules_engine/tests/parser_tests/tests/round_trip_tests/judgment_ability_round_trip_tests.rs`

- [ ] **Step 1: Write a failing parser expectation using `{Judgment}`**

```rust
#[test]
fn test_judgment_foresee() {
    let result = parse_ability("{Judgment} {Foresee}.", "f: 3");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: Effect(Foresee(
        count: 3,
      )),
    ))
    "###);
}
```

- [ ] **Step 2: Run the parser test to verify `{Judgment}` is not yet accepted
  everywhere**

Run: `just parser-test game_effects_parser_tests`

Expected: FAIL because the parser, serializer, and localized phrases still use
`Dawn`.

- [ ] **Step 3: Rename the keyword and related phrase helpers**

```rust
pub enum TriggerKeyword {
    Materialized,
    Judgment,
    Dissolved,
}
```

```rust
choice((
    directive("judgment").to(TriggerKeyword::Judgment),
    directive("materialized").to(TriggerKeyword::Materialized),
    directive("dissolved").to(TriggerKeyword::Dissolved),
))
```

```rust
[TriggerKeyword::Judgment] => strings::judgment(),
[TriggerKeyword::Materialized, TriggerKeyword::Judgment] => strings::materialized_judgment(),
```

Also rename `dawn_phase_name`, `dawn_keyword_name`, `materialized_dawn`, and
related helper strings to their `judgment` equivalents.

- [ ] **Step 4: Update parser tests and round-trip fixtures to use
  `{Judgment}`**

```rust
assert_rendered_match("{Judgment} Return this character from your void to your hand.", "");
assert_rendered_match("{Materialized_Judgment} With {count_allied_subtype($a, $t)}, gain {energy($e)}.", "t: Warrior\na: 2\ne: 1");
```

- [ ] **Step 5: Run the focused parser suites again**

Run: `just parser-test lexer_tests` Run:
`just parser-test game_effects_parser_tests` Run:
`just parser-test judgment_ability_round_trip_tests`

Expected: PASS with `{Judgment}` replacing `{Dawn}` throughout parser-facing
tests.

- [ ] **Step 6: Commit the parser/string rename**

```bash
git add rules_engine/src/ability_data/src/trigger_event.rs \
  rules_engine/src/parser/src/parser/trigger_parser.rs \
  rules_engine/src/parser/src/serializer/trigger_serializer.rs \
  rules_engine/src/parser/src/parser/effect/game_effects_parsers.rs \
  rules_engine/src/parser/src/variables/parser_substitutions.rs \
  rules_engine/src/strings/src/strings.rlf.rs \
  rules_engine/src/strings/locales/bracket.rlf \
  rules_engine/src/strings/locales/ru.rlf \
  rules_engine/tests/parser_tests/tests/game_effects_parser_tests.rs \
  rules_engine/tests/parser_tests/tests/lexer_tests.rs \
  rules_engine/tests/parser_tests/tests/round_trip_tests/judgment_ability_round_trip_tests.rs
git commit -m "refactor: rename dawn triggers back to judgment"
```

### Task 3: Update Card Data, Regenerate Artifacts, And Refresh Rules Docs

**Files:**

- Modify: `client/Assets/StreamingAssets/Tabula/cards.toml`

- Modify: `docs/battle_rules/battle_rules.md`

- Regenerate: `client/Assets/StreamingAssets/Tabula/parsed_abilities.json`

- Regenerate: generated Rust sources affected by `just tabula-generate`

- [ ] **Step 1: Rewrite player-facing card text and rules text to use Judgment**

```toml
rules-text = "{Judgment} Gain {energy($e)}."
rules-text = "At the end of this turn, trigger an additional {judgment_phase_name} phase."
rules-text = "When you {materialize} a character, trigger the {Judgment} ability of each ally."
```

- [ ] **Step 2: Update the battle rules doc to describe the restored phase
  order**

```md
1. **Judgment** — Start-of-turn trigger window and front-rank combat. The active
   player's front-rank characters attack; the opponent's front-rank characters
   block.
2. **Dreamwell** — The active player draws the next Dreamwell card...
3. **Draw** — The active player draws one card from their deck...
4. **Main** — ...
5. **Ending** — ...
```

- [ ] **Step 3: Regenerate parser/card artifacts after the TOML change**

Run: `just tabula-generate`

Expected: generated files update to the restored `{Judgment}` naming with no
manual edits required.

- [ ] **Step 4: Run formatting and the full review gate**

Run: `just fmt` Run: `just review`

Expected: PASS after the phase rename, generated artifacts, and docs updates
settle.

- [ ] **Step 5: Commit the regenerated content and doc refresh**

```bash
git add client/Assets/StreamingAssets/Tabula/cards.toml \
  client/Assets/StreamingAssets/Tabula/parsed_abilities.json \
  docs/battle_rules/battle_rules.md
git commit -m "docs: restore judgment terminology in battle rules and card data"
```

## Self-Review

- Spec coverage: Task 1 covers turn order and combat timing, Task 2 covers
  trigger/parser/string semantics, and Task 3 covers card text, generated
  artifacts, docs, and repo-wide verification.
- Placeholder scan: no `TODO`, `TBD`, or deferred “handle later” steps remain.
- Type consistency: the plan consistently uses `Judgment` as the start-of-turn
  trigger keyword and phase name, and removes `Dawn` rather than introducing a
  second synonym.
