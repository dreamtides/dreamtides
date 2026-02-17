# Dreamtides

This document describes the overall technical architecture of the Dreamtides
roguelike deckbuilding game.

## Project Architecture

Dreamtides has a split architecture. The frontend is written in Unity3D C# code.
The backend or "rules engine" is written in Rust. The core abstraction here is JSON
communication with a native Rust plugin (in production) or via HTTP calls
(during development). The Unity client performs Game Actions
(rules_engine/src/action_data/src/game_action_data.rs), which are serialized to
JSON by the ActionService
(client/Assets/Dreamtides/Services/ActionServiceImpl.cs). These ultimately get
sent to the engine entrypoint (rules_engine/src/rules_engine/src/engine.rs),
which routes actions to modules like handle_battle_action
(rules_engine/src/rules_engine/src/handle_battle_action.rs). The rules engine
then resolves the game action, mutates game state, writes the result to the save
file, and then returns a series of "Commands"
(rules_engine/src/display_data/src/command.rs), which are instructions to modify
the state of the UI. We generate C# code using JSON schema to enable the
frontend to call Rust (see the `schema_generator` cratea).

We try and follow a "react-style" philosophy with our command system, where we
describe the complete state of the target UI via commands instead of sending
back imperative mutations, but we're not always good at sticking to this rule.
We should try and do this more consistently in the future.

## Rules Engine Architecture

### Display Crate: Battle Rendering

Commands for battles are rendered by the Renderer module
(rules_engine/src/display/src/rendering/renderer.rs), which produces a
description of new state of the UI, especially the BattleView
(rules_engine/src/display_data/src/battle_view.rs) which has the overall UI
state.

### Tabula Game Data System

Game data for things like defining cards lives in a system called "tabula",
loaded from TOML files. The base card definition set lives in
client/Assets/StreamingAssets/Tabula/cards.toml and is symlinked to
rules_engine/tabula/cards.toml.

We also provide a standalone app called `tv` for viewing tabula TOML files.

### Abilities & Ability Parsing

Cards are defined by writing their rules text in English. This rules text is
then parsed by an actual language parser engine into
rules_engine/tabula/parsed_abilities.json by the `tabula generate` command. The
core card ability definition is in the
rules_engine/src/ability_data/src/ability.rs struct. Card abilities can be
serialized back to English by the "serializer" module, which is particularly
helpful for rendering UI text for individual pieces of a card's effect.

### Game AI

The battle mode has a strong AI opponent implementation in the various `ai_`
crates, based on Monte Carlo Tree Search.

### Battle State, Queries, and Mutations

Battle and card effects are resolved via mutation to the BattleState struct
(rules_engine/src/battle_state/src/battle/battle_state.rs), which must be
extremely high performance due to the constraints of AI action search. One hard
requiremenet is that battles never contain more than 128 cards, this allows
rules_engine/src/battle_state/src/battle_cards/card_set.rs to store cards in a
u128. The `battle_mutations` and `battle_queries` crates mutate and query the
battle state respectively.

### Masonry & UI Components

There are several distinct UI systems in dreamtides. We use world-space 3D
objects for some UI interactions where it makes sense, as well as Unity's UGUI
Canvas. For UI created mostly from Rust code, we use an in-house React-like
framework called Masonry (rules_engine/src/masonry/), building on top of Unity's
UIToolkit. This natively supports flexbox and many CSS-like properties.

The `ui_components` crate (rules_engine/src/ui_components/) builds some resuable
primitives on top of Masonry.

### Logging

Dreamtides contains a sophisticated logging system which captures significant
system details throughout a play session. The `battle_trace!` macro is the main
entrypoint to this (rules_engine/src/battle_queries/src/macros/battle_trace.rs)
as well as the `logging` crate.

### RLF Localization System

We have an in-house localization system called RLF which implements strings
based on a Rust macro. All UI strings must be sourced from
rules_engine/src/strings/src/strings.rlf.rs.

### State Provider & Rules Engine Testing

Dreamtides uses "black box" testing to the maximum extent possible. This
generally means that we prefer to write tests that approximate performing game
actions using the actual user interface instead of writing tests that call
internal methods to verify their behavior. We almost never write "unit" tests.

The core abstraction that enables testing is the State Provider trait
(rules_engine/src/state_provider/src/state_provider.rs). This is a fairly
standard dependency injection pattern, which helps write tests that don't
directly mutate external state.

## Client Architecture

### Services

The dreamtides frontend implements dependency resolution via a service locator
pattern, with the top level entrypoint to the system existing in the Registry
class (client/Assets/Dreamtides/Services/Registry.cs). All dreamtides gameplay
occurs within a single Unity3D scene.

### Components

Dreamtides ships standard Unity Monobehaviour components for most core game
functionality, which are instantiated from prefabs. The core Card component
(client/Assets/Dreamtides/Components/Card.cs) is the main way cards are
represented in-game.

### ObjectLayout & Displayable

Game entities in Dreamtides will often extend Displayable
(client/Assets/Dreamtides/Layout/Displayable.cs). This is a fairly simple parent
class which implements support for the custom mouse gesture detection system and
things like initialization.

Cards and other entities are often displayed in-game via the ObjectLayout class
(client/Assets/Dreamtides/Layout/ObjectLayout.cs) which handles positioning
objects on screen, coordinating with services like CardService
(client/Assets/Dreamtides/Services/CardService.cs) to update card state.

### Animations

As mentioned above, the best way to animate any game action in Dreamtides is to
rely on automatic transitions between full "snapshots" of the game state,
following a React-like philosophy. The ObjectLayout system is able to animate
changes to children it manages.

When this isn't sufficient, we use the DOTWeen library for custom animated
sequences. These are coordinated through CardAnimationService
(client/Assets/Dreamtides/Services/CardAnimationService.cs)

### Client Testing

We've recently started adding client-side unit tests to Dreamtides as well.
These run as Editor-mode tests for speed, and should be written to validate all
new UI code going foward. Client tests live in client/Assets/Dreamtides/Tests/.