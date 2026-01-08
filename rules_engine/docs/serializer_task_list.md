# Serializer Task List

This document outlines tasks for improving and expanding the ability serializer in
`src/parser_v2/src/serializer/`. The serializer converts `Ability` data structures into
human-readable rules text strings.

## Current Architecture

The serializer is organized into several modules:
- `ability_serializer.rs` - Top-level entry point for serializing abilities
- `effect_serializer.rs` - Serializes `StandardEffect` and `Effect` types
- `trigger_serializer.rs` - Serializes `TriggerEvent` types
- `cost_serializer.rs` - Serializes `Cost` types
- `static_ability_serializer.rs` - Serializes `StaticAbility` types
- `predicate_serializer.rs` - Serializes `Predicate` and `CardPredicate` types
- `serializer_utils.rs` - Shared utilities (`capitalize_first_letter`, `serialize_operator`)

---

## Part 3: Effect Serialization Features

---

## Part 4: Other Serializer Features

(No pending tasks)

---
