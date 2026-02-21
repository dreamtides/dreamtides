# Dreamtides Documentation Index

- [project_overview.md](project_overview/project_overview.md): Technical
  architecture overview covering the full project structure, crate layers, card
  data pipeline, battle execution, display/animation system, client
  architecture, AI, testing, and build tooling. Read when onboarding or needing
  a broad understanding of any system.

- [parser_pipeline.md](parser_pipeline/parser_pipeline.md): End-to-end guide to
  the card ability parser pipeline — how TOML rules-text is lexed, variable-
  resolved, parsed into ability ASTs, and serialized back to rich display text.
  Includes a step-by-step checklist for adding new keywords/effects, the five
  effect sub-parser modules, predicate targeting, RLF phrase integration, tabula
  generate lifecycle, and common pitfalls. Read when adding or modifying card
  abilities, keywords, effects, or parser grammar.

- [toml_card_format.md](toml_card_format/toml_card_format.md): Complete field
  reference for TOML card definitions — regular card and dreamwell card schemas,
  rules-text directive syntax (all pattern types with descriptions), variables
  field format and value types, phrase table entries, modal card conventions,
  multi-paragraph abilities, the prompts field, file locations, generated
  artifacts, staleness checking, and the tv editor app. Read when authoring or
  modifying card data in TOML files, adding new directive patterns, or
  understanding the card data pipeline from TOML to runtime.

- [rlf_localization.md](rlf_localization/rlf_localization.md): Complete guide to
  the RLF localization system — the rlf! macro syntax (constant and
  parameterized phrases, plural matching, variant blocks, :from inheritance,
  transforms), the Phrase type API and composition patterns, how serializers
  connect to RLF to produce display text, parser-side RLF syntax resolution,
  locale overrides, and the rlf_fmt/rlf_lint tools. Read when adding or
  modifying UI strings, working with card text display, or understanding how
  rules text is formatted for rendering.

- [effect_resolution.md](effect_resolution/effect_resolution.md): How effects
  resolve, triggers fire, and prompts interleave during battle execution — the
  three-pass cleanup cascade (pending effects, triggers, turn state machine),
  pending effects queue FIFO processing and list decomposition, prompt halting
  and OnSelected resumption, trigger lifecycle (registration, matching, flat
  queue chaining), all nine PromptType variants, EffectSource controller
  propagation, the turn phase integration, and a checklist for adding new
  triggered effects. Read when debugging effect ordering, adding triggered
  abilities, working with prompt interactions, or understanding the runtime
  control flow after any player action.

- [display_animation.md](display_animation/display_animation.md): How game state
  changes become visual sequences — animation recording via push_animation()
  with snapshot-per-step capture, the BattleAnimation enum (20 event variants),
  the two-entry-point rendering pipeline (connect for single snapshots,
  render_updates for animation replay), the ResponseBuilder API and pending
  command synchronization, all 18 Command variants, BattleView assembly from
  cards/interface/arrows/preview, the react-style snapshot philosophy vs
  imperative VFX commands, client-side CardService diffing and ObjectLayout
  positioning, DOTween integration, and a checklist for adding new animations.
  Read when adding visual feedback for game events, working with the rendering
  pipeline, debugging animation sequencing, or understanding how Rust-side state
  changes translate to Unity client visuals.

- [masonry_ui_panels.md](masonry_ui_panels/masonry_ui_panels.md): How Rust-
  defined UI is built and rendered in Unity — the FlexNode tree structure and
  FlexStyle properties (~50 CSS flexbox properties with dimension units), the
  Component trait (render/flex_node composition, WrapperComponent type erasure),
  all available components (BoxComponent, TextComponent, ButtonComponent,
  PanelComponent, ScrollViewComponent, CloseButtonComponent with builder
  patterns and Typography presets), interface_rendering assembly of the screen
  overlay and action buttons, the panel system (developer, AI selection, card
  addition, log viewer), the client-side Reconciler virtual-DOM diffing by index
  and node type, MasonRenderer style mapping and interactive style layering,
  DocumentService's four overlay containers, the full event handling flow from
  FlexNode EventHandlers through GameAction dispatch to engine response, and
  DisplayProperties mobile adaptation (0.65x font scaling, scrollbar hiding).
  Read when building new UI panels or components in Rust, adding event handlers
  to masonry nodes, working with the client-side reconciler or style system,
  debugging UI rendering issues, or understanding how Rust UI reaches Unity.

- [testing_cookbook.md](testing_cookbook/testing_cookbook.md): How to write and
  run integration tests — the TestBattle builder and TestPlayer configuration,
  TestSession dual-client orchestration, TestClient zone queries and state
  assertions, the TestSessionBattleExtension high-level API (create_and_play,
  click_card, activate_ability, select_card_order, etc.), DebugBattleAction for
  test setup, test card conventions in test-cards.toml, common test patterns
  (effect verification, targeting, stack interactions, prompt/browser flows,
  dual client verification, display command inspection), TestStateProvider
  infrastructure (global Tabula cache, panic-on-error), and the distinction
  between parser tests and battle tests. Read when writing new tests, adding
  test cards, debugging test failures, or understanding the test infrastructure.

- [ai_system.md](ai_system/ai_system.md): How the AI opponent selects actions —
  the GameAI agent types, Monte Carlo Tree Search with UCT (search architecture,
  tree policy, random rollouts, backpropagation), information-set sampling for
  hidden cards, dynamic iteration budgets with phase-based multipliers, root
  parallelization via rayon, speculative search pre-computation on background
  threads, integration with the battle action loop, and the
  ai_data/ai_agents/ai_uct crate organization. Read when working on AI behavior,
  tuning search parameters, debugging AI decisions, or understanding how the
  engine drives AI turns.

- [logging.md](logging/logging.md): How the logging subsystem works — the
  tracing subscriber stack (ForestLayer tree-structured output, ErrorLayer,
  EnvFilter), emoji tag categorization by module and level, initialization paths
  for dev server vs plugin/FFI vs tests, the battle_trace! macro (conditional
  tracing with JSON state snapshots and tracing::debug events), output files
  (dreamtides.log for human-readable trees, dreamtides.json for machine-readable
  trace events with full battle state snapshots), and the logging crate public
  API. Read when debugging battle execution, working with tracing output, adding
  new log instrumentation, or understanding how logging is initialized across
  different runtime environments.

- [style_code_ordering.md](style_code_ordering/style_code_ordering.md): All
  style and code ordering rules enforced by the custom style_validator binary,
  workspace clippy lints, and rustfmt — file item ordering (the 10-category
  sequence from private consts through test modules), naming qualification rules
  (qualifier counts for function calls, type names, and enum variants), import
  conventions (crate:: required, pub use banned, no inline use), module file
  restrictions, test location enforcement, Cargo.toml dependency ordering
  (internal first then external, alphabetized), the allow_attributes deny rule
  and #[expect()] vs #[allow()] distinction, clippy configuration (~30 denied
  lints), rustfmt settings, RLF formatting/linting, the full review pipeline
  sequence, and which violations are auto-fixable via just fmt. Read when
  encountering style_validator or clippy failures during just review, adding new
  files or modules, or understanding the project's code conventions.

- [benchmarks.md](benchmarks/benchmarks.md): How to run and interpret
  performance benchmarks — Criterion statistical timing benchmarks, IAI
  instruction-level profiling via Valgrind Callgrind, the Docker-based Linux
  runner for IAI from macOS, all just commands for specific benchmark suites,
  interpreting IAI output metrics (instructions, cache hits, estimated cycles),
  the 10M cycle regression threshold for MCTS search, profiling with Samply, and
  the benchmark crate layout. Read when running benchmarks, investigating
  performance regressions, adding new benchmarks, or profiling AI search
  performance.

- [abu.md](abu/abu.md): How to run and extend ABU (Agent-Browser for Unity) —
  start the daemon, connect Unity via `DreamtidesAbuSetup`, use the
  agent-browser CLI to snapshot/click/hover/drag/screenshot, the three
  Dreamtides UI systems the walker traverses (UI Toolkit, 3D Displayables,
  CanvasButtons), occlusion rules, settled detection via
  `ActionService.IsProcessingCommands` and DOTween, the `ISceneWalker` and
  `ISettledProvider` interfaces for adapting ABU to other games, and package
  integration via `manifest.json`. Read when using the agent-browser CLI with
  Dreamtides, debugging ABU connectivity or snapshot accuracy, or porting ABU to
  a new game.
