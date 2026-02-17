# Documentation Roadmap

Ordered list of proposed documentation appendices for Dreamtides, prioritized by
frequency of developer confusion and cost of mistakes.

## 1. Card Ability Parser Pipeline

The most complex subsystem. Adding a new keyword or effect requires coordinated
changes across 5+ files with no existing guide.

Document should cover:
- End-to-end walkthrough: "how to add a new keyword" from TOML rules-text
  through to rendered UI text.
- The four PHRASES tables in parser_substitutions.rs (PHRASES, BARE_PHRASES,
  SUBTYPE_PHRASES, FIGMENT_PHRASES) — when to use which, what the default_var
  convention means.
- ResolvedToken enum design — each semantic concept gets its own variant, how to
  add a new one.
- Chumsky parser structure in ability_parser.rs — the five top-level
  alternatives (Triggered, Activated, Named, Static, Event), how sub-parsers
  compose.
- StandardEffect — how to add a new variant, what fields it needs, how
  Predicate/CardPredicate targeting works.
- The serializer inverse path — adding the corresponding serialization in
  effect_serializer.rs and connecting to RLF phrases in strings.rlf.rs.
- The tabula generate → parsed_abilities.json → runtime load lifecycle. When to
  regenerate, how tabula-check validates staleness.
- Common pitfalls: lexer lowercases everything, double newlines split abilities,
  display-only directives like {card:$c} are filtered out during parsing.

Key files: parser_v2/src/variables/parser_substitutions.rs,
parser_v2/src/parser/ability_parser.rs, ability_data/src/standard_effect.rs,
parser_v2/src/serializer/effect_serializer.rs, strings/src/strings.rlf.rs,
tabula_cli/src/commands/generate.rs.

## 2. TOML Card Definition Format

Anyone adding or modifying cards needs this reference. The format has many
implicit conventions not documented anywhere.

Document should cover:
- Complete field reference for [[cards]] entries: id, name, energy-cost (integer
  or "*" for variable), card-type, subtype, spark, is-fast, rules-text,
  variables, prompts, image-number, rarity.
- The rules-text directive syntax: {keyword}, {keyword($var)},
  {@transform keyword($var)}, {keyword($var):selector}. Concrete examples of
  each pattern.
- The variables field format: comma/newline-separated "key: value" pairs where
  values are integers, subtypes (CardSubtype enum values), or figments
  (FigmentType enum values).
- Modal card conventions: energy-cost "*", variables with e1/e2/c1/c2 numbered
  variants, {choose_one} directive.
- Dreamwell card format (dreamwell.toml): produced-energy, effects, phase.
- File locations and sync requirements: rules_engine/tabula/ vs
  client/Assets/StreamingAssets/Tabula/. Both must be kept in sync manually.
- Test card files: test-cards.toml, test-dreamwell.toml — conventions for test
  card naming and ID assignment.
- The tv app for visual TOML editing.

Key files: rules_engine/tabula/test-cards.toml (readable examples),
tabula_data/src/card_definition_raw.rs (field definitions),
tabula_data/src/toml_loader.rs (deserialization).

## 3. RLF Localization System

An in-house DSL with no external documentation. Every UI string change requires
understanding this system.

Document should cover:
- The rlf! macro syntax: constant phrases, parameterized phrases ($var),
  plural matching (:match), variant metadata (:a, :an, :from), variant
  selection (:one, :other).
- How to add a new phrase: define in strings.rlf.rs, use via
  strings::phrase_name() which returns rlf::Phrase.
- Phrase composition: Phrase::empty(), map_text(), to_string(),
  capitalized_sentence() wrapper.
- Rich text conventions: Unity-compatible tags (<color=#HEX>, <b>, <u>),
  color coding by keyword category (purple for keywords like dissolve/banish,
  teal for energy, etc.).
- How serializers connect to RLF: effect_serializer calls strings::* functions,
  bindings.insert() for variable values, VariableValue types.
- The rlf_fmt and rlf_lint tools: what they validate, how to run them.
- Relationship to the parser: RLF function call syntax in rules-text directives
  (energy($e), @a subtype($t)) is resolved by resolve_rlf_syntax() in
  parser_substitutions.rs, separate from the display serialization path.

Key files: strings/src/strings.rlf.rs, parser_v2/src/serializer/ (all files),
parser_v2/src/variables/parser_substitutions.rs (resolve_rlf_syntax).

## 4. Effect Resolution & Trigger System

The most subtle runtime system. Incorrect assumptions about ordering cause bugs
that only manifest in specific card interactions.

Document should cover:
- The three cleanup passes after every action: drain pending_effects, fire
  triggers, advance turn state machine. Each auto-executed action runs all three.
- Pending effects queue (VecDeque<PendingEffect>): FIFO processing, how List
  effects decompose (first element executed, remainder re-queued at front).
- Prompt interleaving: how PromptData halts all three loops, how OnSelected
  links prompt responses back to pending effects or stack targets.
- The nine PromptType variants and when each is used.
- Trigger lifecycle: listener registration on zone entry (on_enter_battlefield),
  deregistration on zone exit, TriggerState push → TriggerForListener creation,
  battlefield check before firing, trigger_queries::matches() conditions.
- Trigger chaining: no recursion, flat queue drain. New triggers from effect
  resolution append to back of events queue. Bounded by finite cards in play.
- How to add a new triggered effect: push Trigger variant in mutation code,
  handle in fire_triggers, connect to StandardEffect.
- EffectSource tracking: Game/Player/Dreamwell/Event/Activated/Triggered/IfYouDo
  variants, how controller is propagated.

Key files: battle_mutations/src/effects/apply_effect.rs,
battle_mutations/src/effects/apply_standard_effect.rs,
battle_mutations/src/phase_mutations/fire_triggers.rs,
battle_state/src/triggers/, battle_state/src/prompt_types/prompt_data.rs.

## 5. Display & Animation Patterns

How game state changes become visual sequences on the client.

Document should cover:
- Animation recording: BattleState::push_animation() captures a full state
  snapshot with each BattleAnimation event. Closure-based to skip when animations
  disabled (AI simulation). ~21 call sites across battle_mutations.
- How to add a new BattleAnimation variant: add to enum, push in mutation code,
  handle in animations::render(), add card-specific VFX via apply_card_fx.
- The rendering pipeline: renderer::connect() (single snapshot) vs
  renderer::render_updates() (animation replay + final snapshot).
- ResponseBuilder API: push() for sequential commands, push_battle_view() for
  snapshots with parallel commands, run_with_next_battle_view() for commands
  that fire alongside the next snapshot. is_for_animation() flag.
- CommandSequence structure: sequential ParallelCommandGroups, each containing
  parallel Commands. The 18 Command variants and when to use each.
- BattleView assembly: card_rendering for all cards across zones,
  interface_rendering for UI overlays, player_view for status, arrows for
  targeting lines.
- The "react-style" philosophy: send complete snapshots, not deltas. Client
  reconstructs UI from each UpdateBattle. When this works well vs when
  imperative commands (FireProjectile, DissolveCard) are needed.

Key files: display/src/rendering/renderer.rs, display/src/rendering/animations.rs,
display/src/rendering/battle_rendering.rs, display/src/core/response_builder.rs,
display_data/src/command.rs, display_data/src/battle_view.rs.

## 6. Testing Cookbook

Tests are the primary way new behavior is validated. The test API has conventions
that aren't obvious from reading individual tests.

Document should cover:
- TestBattle builder: default state (99 energy, both players, user's turn),
  how to configure players (TestPlayer builder), setting seed, configuring
  dreamwell, enabling AI opponent.
- TestSession: connect() initialization, perform_user_action() /
  perform_enemy_action(), how both user and enemy clients are updated.
- TestClient query API: cards.user_hand(), cards.enemy_battlefield(), etc.
  How to find specific cards, assert on card state.
- Common test patterns: play a card and verify effect, test triggered abilities,
  test prompt interactions, verify both client views match.
- TestStateProvider: global Tabula cache via OnceLock, should_panic_on_error
  behavior, default deck (Vanilla) and dreamwell (TestDreamwellNoAbilities).
- Test card conventions in test-cards.toml: naming, how test card IDs are
  generated constants in tabula_generated.
- Running tests: just battle-test <NAME>, just parser-test, just test.
  RUST_MIN_STACK for parser tests.
- DebugBattleAction for test setup: moving cards between zones, setting
  energy/points/spark.

Key files: test_utils/src/battle/test_battle.rs,
test_utils/src/session/test_session.rs, test_utils/src/client/test_client.rs,
tests/battle_tests/ (examples).

## 7. Client Masonry & UI Panels

Building new UI from Rust that renders in Unity. Both sides of the bridge.

Document should cover:
- Rust side: Component trait (render() for composition, flex_node() for
  resolution), WrapperComponent for type erasure.
- Available components: BoxComponent, TextComponent, ButtonComponent,
  PanelComponent, ScrollViewComponent, CloseButtonComponent. Builder patterns.
- FlexNode structure: children, style, hover_style, pressed_style,
  on_attach_style, event_handlers mapping to GameAction.
- FlexStyle: full CSS flexbox analog (~45 properties). Dimension units (Px,
  Percentage, ViewportWidth/Height, SafeAreaInsets). Typography enum.
- How interface_rendering.rs builds the screen overlay from components.
- Client side: Reconciler.cs virtual-DOM diffing by node name. MasonRenderer.cs
  style application. DocumentService.cs bridge with four overlay containers.
- Event handling: EventHandlers on FlexNode map to GameAction, dispatched via
  ActionService.PerformAction() as BattleDisplayAction.
- DisplayProperties: screen dimensions, is_mobile_device flag, mobile font
  scaling (0.65x).

Key files: masonry/src/flex_node.rs, masonry/src/flex_style.rs,
ui_components/src/ (all), display/src/rendering/interface_rendering.rs,
client/Assets/Dreamtides/Masonry/ (all).

## 8. Style & Code Ordering Rules

The style_validator causes frequent just review failures for contributors
unfamiliar with the conventions.

Document should cover:
- Item ordering in files: PrivateConst → PrivateStatic → ThreadLocal →
  PublicTypeAlias → PublicConst → PublicTrait → PublicStructOrEnum →
  PublicFunction → PrivateItems → TestModule. Private consts/statics come
  BEFORE public items.
- Import conventions: use crate:: not super::, no use declarations inside
  function bodies, no pub use.
- Naming conventions: function calls use exactly one qualifier, struct names
  use zero qualifiers, enum values use one qualifier.
- No inline mod tests — tests go in rules_engine/tests/.
- No code in mod.rs or lib.rs except module declarations.
- Cargo.toml: dependencies alphabetized in two groups (internal then external).
- The --fix flag: just fmt runs style_validator --fix automatically.
- Clippy configuration: workspace-level lints in Cargo.toml, the
  unnested_or_patterns exception for chumsky select! macro.

Key files: style_validator/src/ (all), rules_engine/Cargo.toml (workspace
lints section), justfile (fmt and review recipes).
