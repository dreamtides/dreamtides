# Dream Journey Generation TypeScript Prototype

## Summary

Dreamtides needs a concrete Dream Journey generator, not just a collection of
good event ideas. The current design material establishes the desired feel of
the system, but it does not yet specify a practical TypeScript architecture for
authoring, generating, validating, and inspecting journeys.

This document proposes a data-driven Dream Journey generation prototype that
runs on the command line, reads journey configuration from TOML, and produces
fully precommitted text manifests. Those manifests include the one-sentence
description for every visible dream choice and, when needed, higher-level
instruction text such as `Choose 2` or `Go deeper?`.

The prototype is intentionally text-first and execution-free. It does not modify
quest state, apply effects, or implement Dream Journey gameplay. Its job is to
answer a narrower but critical question: can Dreamtides procedurally generate
Dream Journey choices that feel curated, readable, and varied while remaining
deterministic and authorable?

The core design decision is to generate a bounded interaction tree rather than
only a flat list of offers. Single-step sites still appear as one root node with
one to three visible dreams. Multi-step sites such as sequential offers,
push-your-luck scenes, and `Choose 2` menus are expanded into explicit follow-up
nodes at generation time. The CLI can therefore print the entire journey
structure without needing to execute game logic.

The second core decision is to keep the system shape-first. A Dream Journey is
not assembled from a giant undifferentiated effect soup. The generator first
chooses a journey shape, then fills that shape with compatible content entries,
renders player-facing text, validates the result, and records the committed
manifest and trace data.

The third core decision is that broad payload surfaces are in scope even though
effect execution is not. Because this prototype only produces text, it can cover
card gains, dreamsigns, custom cards, custom dreamsigns, battlefield slot
mutations, quest statuses, Dreamwell rewrites, delayed rewards, and route or
dreamscape changes so long as each payload has authorable preview text and
enough metadata to validate where it can appear.

## Related Information

- [Dream Journey Generation](dream_journey_generation.md): Existing concept
  document. It establishes the shape-first direction, pacing goals, and many of
  the candidate shape names that this implementation design makes concrete.
- [Procedural Journeys Brainstorming]
  (../../../.llms/notes/procedural_journeys.md): Raw ideation list covering
  simple effects, costs, compound effects, hidden information, statuses,
  battlefield mutations, and shape ideas. This is the broad content pool the
  prototype needs to classify and structure.
- [Quest Master Design](quests.md): High-level quest design, atlas flow,
  Dreamcaller package context, and the project-wide rule that gameplay
  configuration should live in TOML whenever reasonable.
- [Quest Prototype Notes](../../../scripts/quest_prototype/PLAN.md): Current
  reference for the existing TypeScript quest prototype. It is relevant only as
  adjacent context for run-state concepts, not as the required home of this
  generator.
- [Current Hardcoded Dream Journeys]
  (../../../scripts/quest_prototype/src/data/dream-journeys.ts): Minimal
  baseline implementation that proves the surface exists, but this document does
  not propose extending that file directly.
- [Atlas Generator]
  (../../../scripts/quest_prototype/src/atlas/atlas-generator.ts): Current
  TypeScript site-generation logic. It is a useful reference for completion
  level staging and deterministic content-pool thinking, not a required module
  boundary for this generator.
- [Quest Content Setup]
  (../../../scripts/quest_prototype/scripts/setup-assets.mjs): Existing TOML
  parsing and normalization conventions in the prototype. The standalone
  generator may borrow compatible conventions where that reduces duplicated
  authoring work.
- [Prototype Logging](../../../scripts/quest_prototype/src/logging.ts):
  Structured logging model already used elsewhere in Dreamtides. The standalone
  CLI should emit similarly machine-readable trace events.

## Problem And Context

Dream Journeys are supposed to be the most dramatic and surprising part of quest
mode. They are where Dreamtides can offer dangerous bargains, deck surgery,
delayed promises, structural run changes, and other effects that would feel too
strange or too punishing as ordinary sites.

That creative space is useful, but it is also exactly where vague design causes
the most trouble. The current material says many good things about the desired
experience:

- choices should feel curated rather than random
- symmetrical offers are often more legible than unrelated offers
- hidden information must respect a clear player contract
- effect sizes must be matched to costs
- multi-step structures are sometimes the point of the scene

What is still missing is the implementation contract. A TypeScript developer
building a standalone Dream Journey CLI needs to know:

- what the generator reads
- what it outputs
- how multi-step scenes are represented
- which pieces are authored in TOML
- how broad payload surfaces fit without execution logic
- how to validate that a generated journey is coherent
- how to inspect results from the CLI

Without that specificity, several bad outcomes are likely:

- the prototype will drift toward a tiny hardcoded journey list
- or it will drift toward random effect soup with weak topologies
- or it will defer every hard question to a future execution system
- or it will support only flat 2-3 option sites and quietly abandon the
  multi-step cases that matter most for Dream Journeys

The goal of this document is to close that gap. It does not try to solve all
future runtime integration. It specifies a text-generation prototype that is
narrow enough to implement now and rich enough to answer the real design
question.

## Goals

- Generate Dream Journey text from TOML-authored content rather than from
  hardcoded arrays.
- Produce full journey manifests that are deterministic, inspectable, and
  replayable by seed.
- Support both single-step and bounded multi-step Dream Journey structures in
  V1.
- Keep the system shape-first so generated journeys feel like authored scenes
  rather than random offer bundles.
- Allow broad payload surfaces in V1 because the prototype is only generating
  preview text, not implementing mechanics.
- Keep the implementation compatible with proven Dreamtides TypeScript patterns
  where that is useful: typed content loading, pure generation functions, and
  structured logs.
- Make the CLI useful for both single-seed inspection and large-sample content
  review.
- Preserve a path from prototype manifests to future runtime adoption if the
  output quality is good enough.

## Non-Goals

- Implement Dream Journey gameplay effects.
- Modify live quest state or add a new playable journey runtime.
- Solve final balance values for every cost and reward.
- Generate art, NPC dialog, or option names as part of V1.
- Build a general scripting language for journey behavior.
- Encode exact file layout or module boundaries for the implementation.

## Prototype Positioning

This system is a developer-facing prototype, not the authoritative quest
runtime. That matters.

The prototype should optimize for fast iteration, strong debugging, and content
review. It does not need to answer every gameplay question that a future live
runtime will answer. It does need to be rigorous enough that its output can be
trusted when evaluating design quality.

That means the prototype should behave like a serious content generator even
though it does not execute effects:

- all randomness must come from explicit seeds
- content versions must be recorded
- multi-step structures must be fully expanded
- hidden outcomes must be precommitted
- validation must reject incoherent results
- CLI output must expose enough trace data to explain why a site was generated

If the prototype later becomes the basis for the live runtime, the transition
should be additive rather than a rewrite. The manifest model, content model, and
shape model should therefore be designed as if they might survive into a real
system even though many entries will remain `preview only` for a while.

## Constraints And Requirements

- Dream Journey configuration lives in TOML.
- The generator runs as a TypeScript CLI under Node.
- The generator must support one-step sites and bounded multi-step flows.
- Root site presentation still follows the Dream Journey contract: one to three
  visible dreams at entry, with an average of roughly two.
- Every visible choice must have a mandatory one-sentence preview.
- Some nodes also require top-level instruction text, such as `Choose 2` or
  `You may continue up to 3 times`.
- The generator may react to coarse run context such as completion level, deck
  shape, dreamsign count, Banes, and active hooks, but it must not try to solve
  the run card by card.
- The output must be deterministic for a stable combination of seed, content
  version, and run context.
- Broad payload surfaces are in scope. Text-only support for a payload is enough
  so long as its preview text and placement rules are authorable.
- The system must be inspectable from the CLI without any UI layer.
- The implementation is a standalone CLI tool. It may share formats or helper
  conventions with other TypeScript tooling, but it is not part of
  `scripts/quest_prototype`.

## Proposed Design

### Overview

The proposed generator has five conceptual layers:

1. Content loading and validation.
2. Run-context snapshotting.
3. Shape selection and fill.
4. Interaction-tree expansion and text rendering.
5. CLI presentation and trace output.

The most important simplification is that the generator does not produce live
logic. It produces a fully pre-expanded journey manifest. A manifest is a small
interaction tree with committed text and committed hidden rolls. The CLI can
print it directly, save it as JSON, sample it in bulk, and validate it in tests.

### Critical Interfaces

The implementation should expose a small set of stable interfaces in prose:

- `JourneyRunContext`

  - Seed and generator version identifiers.
  - Content version hash.
  - Completion level and derived stage.
  - Dreamcaller identity and selected package tides.
  - Deck summary metrics such as deck size, starter count, Bane count, and
    transfigured-card count.
  - Resource summary metrics such as current essence and omens.
  - Current dreamsign count and any other coarse persistent run state needed by
    author rules.
  - Active future hooks and recent shape or tag history.
  - Remaining dreamscape budget or equivalent route horizon signal.

- `JourneyManifest`

  - Stable manifest id.
  - Root node id.
  - Selected shape id.
  - Selected site tags.
  - Optional top-level site instruction text.
  - Ordered node list.
  - Committed hidden outcomes and future hooks.
  - Debug metadata including seed path and generator trace summary.

- `JourneyNode`

  - Stable node id.
  - Node kind.
  - Optional instruction text.
  - Selection bounds.
  - Ordered option list.
  - Reveal contract for the node.

- `JourneyOption`

  - Stable option id.
  - Optional display label.
  - Mandatory one-sentence summary text.
  - Outcome kind: immediate result, delayed promise, future hook, or branch.
  - Optional next node id.
  - Optional detail references for cards, dreamsigns, or other named objects.

- `GenerationTrace`

  - Stage-profile selection.
  - Desired tag profile.
  - Candidate shapes and their scores.
  - Chosen fill entries.
  - Validation failures and repair actions.

These interfaces are the real product of the prototype. They are what future
runtime work will either adopt or intentionally replace.

## Interaction Kernel

The generator should not model each journey shape as a special interpreter. It
should compile every shape into a small kernel of node kinds. V1 only needs the
following node kinds:

- `choose_one`: one choice from a visible set.
- `choose_exactly_n`: choose an exact number of visible options.
- `choose_up_to_n`: choose zero to N visible options.
- `continue_or_stop`: a bounded loop node with explicit continue and stop
  branches.
- `auto_resolve`: a reveal or transition node with no new player choice.

This kernel is strong enough to represent the important multi-step structures
without turning the prototype into a scripting engine.

A few examples follow in prose:

- A normal three-dream site is one `choose_one` root node.
- `Choose 2` is one `choose_exactly_n` root node with instruction text.
- `Take up to 3` is one `choose_up_to_n` node.
- Push-your-luck is a chain of `continue_or_stop` nodes with escalating stakes.
- Sequential offers are a fixed series of `choose_one` nodes linked together.
- A delayed reveal can use an `auto_resolve` node after the commitment choice.

The generator should expand bounded loops ahead of time rather than encoding
runtime loop rules. If a shape allows three pushes, the manifest contains the
first, second, and third push nodes explicitly. This keeps the output easy to
read and easy to diff.

## Journey Shape Catalog

The architecture should support a broad shape catalog, but the initial
standalone TypeScript CLI should explicitly implement the following shapes:

- `same_cost_different_rewards`
- `same_reward_different_costs`
- `service_menu`
- `heterogeneous_pair`
- `one_target_many_operations`
- `one_operation_many_targets`
- `choose_your_loss`
- `single_offer`
- `risk_or_skip`
- `now_vs_later`
- `take_any_number`
- `take_up_to_n`
- `repeat_to_scale`
- `push_your_luck`
- `sequential_offers`
- `reward_after_trigger`

The architecture should also reserve clean support for the following follow-up
shapes, even if their initial content volume is light:

- `staged_assembly`
- `paired_return`
- `resolved_random_series`
- `alter_future_dreamscape`

This is a deliberate cut. It keeps V1 rich enough to answer the hard topology
questions without requiring the entire conceptual shape universe to ship at
once.

Within this list, `take_any_number` means a menu-style multi-pick shape such as
`Choose 2 of these 3`. `take_up_to_n` means an iterative structure where the
player may continue taking offers step by step up to a visible cap.

Each shape definition should specify:

- shape id and player-facing promise
- root node kind
- allowed node kinds in follow-up steps
- normal option-count range
- which fill axes are shared across options
- which payload roles may appear
- stage weights
- reveal-style allowances
- maximum node budget
- maximum future-hook budget
- repair preferences when filling fails

## TOML Authoring Model

The authored data model should be split into a small number of TOML concerns.
This is not a file-layout prescription. It is a schema-level contract.

### Global Generator Settings

One TOML section should hold generator-wide settings such as:

- generator version
- maximum root options
- maximum node depth
- maximum total nodes per manifest
- default stage breakpoints
- repetition penalties
- future-hook budgets
- default reveal policies

### Stage Profiles

Stage profiles should be authored in TOML so pacing is tunable without code
changes. Each profile should declare:

- which completion levels it covers
- desired site-tag weights
- shape-weight multipliers
- persistence tolerance
- novelty penalties

The default mapping should follow the current quest design:

- early stage: completion levels 0 and 1
- mid stage: completion levels 2 and 3
- late stage: completion levels 4 through 6

### Shape Definitions

Each `shape` entry should describe a topology, not specific rewards. It should
declare:

- the comparison promise
- required shared axes
- permitted node kinds
- fill slots that the generator must satisfy
- tag affinities
- forbidden tags
- selection bounds and prompt rules
- maximum branch width
- maximum branch depth
- repair order

### Content Entries

Content entries are the reusable building blocks the shapes pull from. They may
represent rewards, costs, burdens, triggers, future promises, or compound
effects. Every content entry should declare:

- content id
- role such as reward, cost, burden, target, trigger, or future hook
- surface such as cards, dreamsigns, battlefield slots, statuses, Dreamwell, or
  route changes
- one or more preview templates
- contributed site tags
- stage weights
- base selection weight
- compatibility tags
- exclusion tags
- context requirements
- persistence footprint
- reveal contract
- optional references to curated pools or named payload assets
- runtime readiness flag with values such as `preview_only`, `planned`, or
  `wired`

The runtime readiness flag is important. The prototype is explicitly text-only,
but the content library should still record which entries already have a real
gameplay implementation and which ones are still only preview text.

### Curated Pools And Named Assets

Some content entries need to point at curated data rather than inventing text
from raw nouns. The schema should therefore support auxiliary pool or asset
definitions for things such as:

- named card pools
- named dreamsign pools
- custom-card text assets
- custom-dreamsign text assets
- battlefield slot-mutation assets
- quest-status assets
- route-change assets
- delayed-package assets

These are supporting reference lists, not a third authoring layer. Their job is
to give content entries stable nouns and stable text surfaces.

## Text-First Content Authoring

The generator must not try to construct all English mechanically from tiny
tokens. That approach will sound synthetic very quickly.

Instead, content entries should carry preview text at the level of authored
sentence templates or even fully-authored sentence variants. The generator may
substitute bounded parameters such as numbers, target pools, or named assets,
but it should not depend on a large freeform grammar system to create quality
choice text.

This leads to three concrete rules:

- Every visible option summary is authored as a full sentence template.
- Instruction text such as `Choose 2` belongs to the node or shape, not to the
  effect entries.
- Complex surfaces such as slot mutations or quest statuses should be authored
  as named assets with stable preview sentences rather than as generic noun
  combinations.

This is the right tradeoff for Dream Journeys. The system still gets huge
combinatorial variety from shape selection, pool selection, and parameter
variation, but the actual sentence quality remains curated.

## Broad Payload Surfaces

Because the prototype only renders text, surface breadth is mainly a data and
validation problem, not a runtime problem. V1 should therefore support preview
generation for the following surface families:

- essence and omen changes
- card gain, loss, purge, transform, duplicate, or transfigure effects
- dreamsign gain, loss, exchange, mutation, or delayed dreamsign promises
- custom cards and custom dreamsigns
- battlefield slot mutations
- quest statuses
- Dreamwell rewrites
- future site additions or removals
- future dreamscape composition changes
- delayed rewards and conditional payouts

The constraint is not `can the real game execute this today`. The constraint is
`can the generator place this effect coherently and describe it clearly`.

Each broad surface still needs surface-specific validation rules. Examples:

- a battlefield slot mutation should not appear in a shape that promises purely
  immediate deck surgery unless the mutation is the overt point of the scene
- a route change should usually require enough remaining dreamscapes to matter
- a quest status should declare whether it counts as a major persistent hook
- a custom card or dreamsign must reference a stable preview asset rather than
  generating names on the fly

## Run Context Model

Even as a prototype, the generator needs a real run context. Otherwise it will
produce pretty but disconnected journeys.

The run context should stay coarse. The system should know enough to bias toward
build, cleanup, refinement, risk, or structural change, but not enough to hand
the player a perfect answer every time.

The required context fields for V1 are:

- run seed
- generator version
- content version hash
- completion level and derived stage
- dreamcaller id
- selected package tides
- current essence
- current omens
- current deck size
- starter-card count
- Bane count
- transfigured-card count
- dreamsign count
- active persistent hooks by category
- recent shape history
- recent site-tag history
- remaining dreamscapes estimate

The CLI should support two context-input modes:

- flag-driven quick generation for fast iteration
- a structured context file for realistic sampling

The context file may be JSON or TOML, but the journey-specific authored content
must remain TOML.

## Generation Pipeline

### 1. Load Content And Compute Content Version

The CLI loads all journey TOML content, validates it, resolves references to
shared quest data, and computes a stable content-version hash over the loaded
result. That hash is embedded in every manifest.

When shared card or dreamsign data is needed, the standalone CLI may reuse
existing normalized Dreamtides content inputs rather than creating an unrelated
second source of truth. Journey-specific configuration still remains
TOML-authored.

### 2. Build The Run Snapshot

The CLI converts flags or a context file into a `JourneyRunContext`. It also
derives coarse helper signals:

- stage
- broad run need
- hook saturation
- recent shape fatigue
- recent tag fatigue

The run-need categories should stay intentionally small in V1:

- `build`
- `cleanup`
- `refine`
- `economy`
- `risk`
- `structural`

### 3. Choose Desired Site Tags

Given the stage profile and run snapshot, the generator chooses a small desired
tag profile for the site. This is still coarse. It is not a mini optimizer.

Example early profiles:

- `build`, `reward`, `immediate`
- `cleanup`, `precise`, `immediate`
- `economy`, `reward`

Example late profiles:

- `convert`, `precise`, `sacrifice`
- `risk`, `gamble`
- `structural`, `delayed`

### 4. Enumerate Legal Shapes

The generator filters out shapes that are impossible or obviously mismatched for
the current context. Examples:

- `one_target_many_operations` is illegal with no valid targets.
- `reward_after_trigger` is illegal when hook budget is exhausted.
- `alter_future_dreamscape` is illegal when there is no route horizon left.
- `take_up_to_n` is illegal when the available content cannot support repeated
  visible decisions without duplication.

### 5. Score And Select A Shape

Shape selection should use deterministic weighted sampling rather than pure
uniform randomness or pure argmax. Shape score should combine:

- base authored weight
- stage-profile multiplier
- desired-tag fit
- context legality quality
- shape-fatigue penalty
- tag-fatigue penalty
- future-hook budget penalty

Only shapes within a top scoring band should be eligible for final weighted
selection. This protects quality while preserving variation.

### 6. Fill The Shape

Once a shape is chosen, the generator fills its slots in a fixed order:

1. shared comparison axis
2. shared target or shared motif
3. reward entries
4. cost and burden entries
5. trigger or future-hook entries
6. reveal contract
7. node instruction text
8. option summary text

This order matters. It is much easier to preserve a coherent scene when the
shape promise is satisfied first and the text is rendered last.

### 7. Expand The Interaction Tree

The filled shape is then expanded into explicit nodes.

This step is where multi-step journeys become concrete:

- `take_up_to_n` expands to a node with visible selection bounds.
- `repeat_to_scale` expands to a deterministic chain whose stakes and payoffs
  escalate by step.
- `push_your_luck` expands to repeated continue-or-stop nodes.
- `sequential_offers` expands to a fixed number of linked offer nodes.
- `reward_after_trigger` expands to a root choice plus a committed future hook.

The tree should remain bounded. V1 defaults should be conservative:

- root options: 1 to 3
- non-root options: 2 to 4
- maximum depth: 4
- maximum total nodes: 10

Those limits exist to keep the CLI output readable and the authored content
reviewable.

### 8. Render Player-Facing Text

Text rendering happens only after all content choices are committed. This avoids
text promising one thing while validation later changes the payload.

Each node renders:

- optional instruction text
- selection bounds if needed
- ordered option summaries
- optional detail references

Each option summary must be exactly one sentence. It may mention delayed or
random structure, but only within the allowed reveal contract.

### 9. Validate

Validation should happen at both content-load time and manifest-generation time.

Manifest validation checks:

- structural validity of the node graph
- shape-promise coherence
- context legality
- reveal-contract correctness
- sentence completeness
- duplicate or near-duplicate option summaries
- hook-budget limits
- branch-budget limits

### 10. Repair Or Reject

Repair should be deterministic and limited:

1. swap a conflicting content entry for another compatible one
2. simplify the fill while staying in the same shape
3. fall back to the next highest-scoring shape

If all repairs fail, the generation should error rather than silently emit low
quality output.

### 11. Emit Manifest And Trace

The CLI should emit:

- the final manifest
- a human-readable rendering when requested
- the generation trace

The manifest is the product. The pretty output is a review surface.

## Text Contract

The Dream Journey text contract must be explicit.

### Option Summary Rules

Every visible option must have one and only one summary sentence. That sentence
must communicate the class of consequence clearly enough that a player could
make a real choice from it.

The sentence:

- may mention costs, rewards, burdens, and timing
- may mention bounded randomness
- may mention named cards, dreamsigns, statuses, or slot mutations
- must not hide the existence of a meaningful downside
- must not rely on a later paragraph to explain the actual effect

### Instruction Text Rules

Instruction text is optional at the site or node level, but it is mandatory for
non-default selection rules. Examples include:

- `Choose 2`
- `Choose up to 2`
- `You may continue up to 3 times`
- `Pick a price`

Instruction text belongs to the node, not to any one option.

### Detail References

The manifest may attach detail references for later UI use, such as linked card
ids or dreamsign ids, but the prototype's main readability test should assume
that the one-sentence option summary stands on its own.

### Names

Generated option names are out of scope for V1. The prototype may optionally
emit labels if the authored content provides them, but the required quality bar
is on summary sentences and instruction text, not on flavor titles.

## Reveal Policies

The prototype should support a small reveal-policy vocabulary:

- fully revealed
- bounded hidden target with visible reward class
- visible delayed package with visible trigger
- bounded random within an explicit envelope

Every content entry and every shape should declare which reveal policies it
allows. Some combinations should be illegal. For example:

- a strong risk-or-skip scene should not also hide the entire downside
- `Choose 2` should not present invisible options
- a delayed route change should still disclose that it changes a future
  dreamscape

Because manifests are precommitted, hidden outcomes can still be stored
internally even when they are not fully shown in the pretty output.

## Multi-Step Flows

Multi-step Dream Journeys are in scope for V1 and should not be treated as
second-class citizens.

The key design rule is that every multi-step journey must remain bounded and
readable as a finite tree. There is no open-ended looping and no hidden runtime
state machine.

Different shapes imply different multi-step behaviors:

- `take_any_number` and `choose_exactly_n` are multi-select but not iterative
- `take_up_to_n` and `repeat_to_scale` are bounded iterative structures
- `push_your_luck` requires explicit continue and stop choices after each step
- `sequential_offers` reveals a series of related offers one after another
- `reward_after_trigger` creates a future hook instead of immediate follow-up

This distinction matters because it changes what the CLI needs to print and what
the validator needs to understand.

## CLI Contract

The CLI should expose three commands in V1:

- `generate`: produce one journey manifest from a context and seed.
- `sample`: produce many manifests for review, distribution checks, and manual
  content browsing.
- `validate`: parse TOML and run schema, reference, and generation tests without
  producing a final sample set.

The `generate` command should support:

- machine-readable manifest output as the default
- a `pretty` mode that prints the site and all follow-up nodes in readable order
- an `explain` mode that includes the generation trace

The `sample` command should support:

- count and seed-range controls
- optional fixed context
- aggregate summaries by shape, stage, tags, and payload surface
- optional pretty-print of selected examples

The `validate` command should support:

- schema validation
- reference validation
- content-pool coverage checks
- smoke generation across multiple seeds and stages

The CLI does not need an interactive play mode in V1. The whole point of the
manifest model is that reviewers can inspect the entire generated structure
without simulating runtime choices.

## Determinism And Traceability

Determinism is required even for a prototype. The generator should never use
ambient randomness.

Each manifest should record:

- root seed
- content version hash
- generator version
- stage profile id
- chosen shape id
- committed hidden outcomes
- manifest id

The RNG should use stable named branches for at least:

- desired tag selection
- shape selection
- content filling
- hidden-outcome commitment
- repair and fallback

The CLI should also emit structured log events for major steps. The event model
should use stable event names, explicit fields, JSON-serializable payloads, and
no reliance on console prose for debugging.

## Validation Strategy

Validation needs to cover more than schema correctness.

### Load-Time Validation

Load-time checks should include:

- TOML schema validation
- unknown field rejection
- missing reference rejection
- duplicate id rejection
- invalid stage-profile references
- invalid reveal-policy declarations
- impossible selection-bound declarations

### Manifest Validation

Manifest checks should include:

- root option count is within bounds
- every nonterminal node has legal options
- all referenced next-node ids exist
- branch graph is acyclic
- total node count stays within budget
- content entries satisfy context requirements
- reveal policies match the text and node kind
- option summaries are non-empty single sentences
- instruction text exists where selection rules require it

### Quality Validation

Quality validation should be opinionated. At minimum it should reject or
penalize manifests where:

- two options are effectively the same offer
- the shape promise is not legible from the rendered text
- the downside is hidden in a supposedly transparent bargain
- a scene mixes too many unrelated surfaces without a unifying premise
- a multi-step scene repeats with no meaningful stake change

The quality validator will never be perfect, but it must exist. Otherwise the
prototype will drift toward mechanically legal but artistically weak content.

## Testing

The implementation should include four test families.

### Unit Tests

Unit tests cover:

- TOML parsing
- context normalization
- shape compilation into node graphs
- sentence rendering helpers
- deterministic weighted selection
- validation rules

### Golden Tests

Golden tests fix seeds and contexts, then assert on emitted manifests. These
tests are especially important because the prototype's main output is content,
not state mutation.

### Sampling Tests

Sampling tests generate many journeys and assert broad distribution properties:

- every implemented shape can appear
- early stages bias toward build and cleanup
- late stages bias toward risk, sacrifice, and structural tags
- no single shape dominates unexpectedly

### Authoring Smoke Tests

Authoring smoke tests run `validate` plus a fixed sample sweep across stages and
seed ranges. Their job is to catch broken references, dead shapes, and obvious
quality regressions before content changes land.

## Rollout

The prototype should be built in three phases.

### Phase 1

Build the kernel:

- TOML schema
- content loading
- run context model
- manifest model
- deterministic RNG
- `generate` and `validate`
- a small set of single-step shapes

The goal of Phase 1 is to replace hardcoded journeys with data-driven generation
for ordinary one-step sites.

### Phase 2

Add bounded multi-step support:

- `take_any_number`
- `take_up_to_n`
- `repeat_to_scale`
- `push_your_luck`
- `sequential_offers`
- instruction-text rendering
- tree pretty-printing

The goal of Phase 2 is to prove that multi-step Dream Journey structures can be
represented cleanly without a scripting engine.

### Phase 3

Add broader surfaces and future-hook support:

- custom-card and custom-dreamsign assets
- battlefield slot mutations
- quest statuses
- delayed rewards
- route and dreamscape changes
- `reward_after_trigger`
- initial `paired_return` support if content is ready

The goal of Phase 3 is to test the real breadth of Dream Journey content while
remaining text-only.

## Alternatives Considered

### Freeform Effect Soup

Purely composing costs, rewards, predicates, and durations from large global
pools would be simpler to code initially, but it would directly recreate the
main design problem. It would generate many legal yet aesthetically weak
journeys and would give the team poor control over the topology of choices.

### Fully Hand-Authored Event Scripts

This would preserve quality but lose the main benefit of procedural generation.
It would also make iteration on stage texture and content breadth much slower.

### A Small Scene DSL

This looks tempting for multi-step scenes, but it is the wrong prototype
investment. The bounded interaction-kernel approach provides enough structure
for V1 without turning the authoring model into programming.

### Flat Only, Multi-Step Later

Deferring multi-step content would make the prototype easier, but it would also
skip exactly the content category that most needs architecture. Dream Journeys
without sequences, escalation, or explicit multi-pick structures would not be a
credible test of the design.

## Risks And Tradeoffs

### Broad Surface Scope Can Outrun Real Gameplay Support

This is an intentional trade. The mitigation is the `runtime readiness` field.
The prototype may author and sample text for many surfaces while still tracking
which ones are only previews.

### Text Quality Can Become Generic

The mitigation is to author preview text at the sentence level and to validate
for duplication and incoherence. If the system drifts toward templated sludge,
the prototype has failed its purpose.

### Too Many Shapes Could Slow Authoring

The mitigation is the phased rollout. The architecture supports more shapes than
the initial content pack uses heavily.

### Context Sensitivity Could Quietly Become A Solver

The mitigation is to keep run-need categories coarse and to limit the context to
summary signals. The generator should react to the run, not solve it.

### Debug Data Could Diverge From User-Facing Text

The mitigation is to render text only after the fill is committed and to embed
the same committed choices in the manifest and the trace.

## Operational Considerations

The CLI should be pleasant to use during design iteration. That means:

- failures should explain which content ids or shape ids were responsible
- `sample` should make it easy to inspect multiple seeds for one fixed context
- pretty output should show node order clearly
- explain mode should show the chosen tags, shape, and repairs

The prototype should also be cheap to run in bulk. Content sampling is likely to
become part of the normal design workflow, especially once broad payload
surfaces are authored.

## Open Questions

- Should V1 accept shared card and dreamsign inputs from normalized JSON, from
  direct TOML parsing, or from either source behind one loader contract? The
  prototype can support either, but the team should pick one default path.
- Should the pretty output eventually include optional flavor labels for dreams,
  or should labels stay absent until there is a separate naming system? This is
  not required for V1 quality, but it affects how sample output reads.
- When the prototype begins to overlap with live runtime work, should
  `preview only` entries remain in the main content pool with stage weights of
  zero, or should they move to a separate authoring surface? The schema can
  support either policy.

## Acceptance Criteria

The design in this document is successful when all of the following are true:

- Dream Journey content is authored in TOML and loaded by a TypeScript CLI.
- The generator emits deterministic manifests with explicit seed and content
  version data.
- The manifest can represent both single-step and bounded multi-step journeys.
- Every visible option has a one-sentence summary.
- Instruction text appears when the node requires it.
- Broad payload surfaces can be authored as preview text without needing
  execution support.
- Validation rejects structurally broken or obviously incoherent journeys.
- Designers can use `generate`, `sample`, and `validate` to inspect content
  quality without a UI implementation.

That is the right V1 target. If the team can generate and review convincing
Dream Journey text under this contract, the next step is runtime execution. If
it cannot, then the design problem is still unsolved and no amount of effect
plumbing will fix it.
