# Serializer RLF Migration — Review Synthesis

**Date:** 2026-02-08
**Reviewers:** 5-agent Opus team (RLF Architect, I18n Specialist, Codebase Analyst, Testing/Risk Analyst, Architecture Reviewer)
**Input:** `2026-02-07-serializer-rlf-migration.md`
**Individual reviews:** `review-rlf-architect.md`, `review-i18n-specialist.md`, `review-codebase-analyst.md`, `review-testing-risk.md`, `review-architecture.md`

---

## Executive Summary

The original plan proposed a Phase 1 migration extracting all hardcoded serializer strings into RLF phrases using an escaped-brace `{{...}}` mechanism to preserve round-trip test compatibility, followed by a future Phase 2 where serializers return `Phrase` objects for real i18n.

After review by 5 specialized agents, the team reached consensus on a **Phase 1.5 approach** — a scoped incremental migration that preserves the escaped-brace mechanism where needed but narrows the scope, defers the riskiest work, and is honest about what's temporary.

**Key findings that drove this recommendation:**
1. The escaped-brace mechanism is technically sound (confirmed against RLF test suite)
2. But ~80% of Phase 1 phrase bodies will be rewritten in Phase 2 (i18n specialist)
3. The plan has ~40-50% coverage gaps in the larger serializers (codebase analyst)
4. Pre-rendered `$target` strings fundamentally block i18n for 4/6 target languages (i18n specialist)
5. Phase 2 directly is too risky — changing return types, tests, and eval_str simultaneously (testing/risk + 3 others)
6. Phase 1 provides durable value: semantic phrase catalog, call-site refactoring, code organization (consensus)

---

## Critical Issues Found

### Resolved: Escaped Brace Semantics (CONFIRMED WORKING)

The testing-risk analyst flagged `{{directive($var)}}` escape behavior as a critical risk. The RLF architect confirmed via specific test citations that:
- `{{Banish}}` → literal `{Banish}` ✓
- `$var` inside `{{...}}` is NOT evaluated — it becomes literal text ✓
- `{{cards($c)}}` → literal `{cards($c)}` (the RLF parameter is ignored) ✓

The two-layer approach (RLF produces template text → `eval_str` evaluates later) is technically sound. The "unused parameter" issue (Phase 1 phrases accept parameters but don't evaluate them inside escaped braces) should be documented but is not a bug.

### Coverage Gaps (~40-50% of code paths unaddressed)

The codebase analyst found the plan covers cost_serializer well but has major gaps elsewhere:

**Completely missed functions:**
- `serialize_gains_reclaim` — 117 lines, 12+ code paths, the most complex function in effect_serializer
- `serialize_for_count_expression` — 14 match arms, used by 7+ callers
- `serialize_void_gains_reclaim` — 8 collection expression arms
- `serialize_predicate_count` in condition_serializer — 12 match arms

**Partially covered:**
- effect_serializer: ~55 StandardEffect variants, plan covers ~25-30
- predicate_serializer: ~100 match arms across 10 functions, plan covers ~60%
- static_ability_serializer: ~22 match arms, plan covers ~50%
- ability_serializer: 5 ability types + complex prefix logic, plan covers ~30%

**Missing keyword:** `{Aegis}` used in serializer output but not defined in strings.rs.

### Multilingual Fundamental Blockers

The i18n specialist demonstrated with real translations across all 6 target languages that:

1. **`$target` as String prevents case/gender agreement** — Russian needs accusative case on dissolve targets ("врага" not "враг"), German needs article declension ("einen Feind" not "ein Feind"), Spanish needs personal "a" before animate objects
2. **Escaped-brace phrases are English-only by construction** — `{{cards($c)}}` references an English-only RLF directive. Translators cannot modify the inner directive.
3. **Word order is locked to English SVO** — Chinese needs prenominal modification ("费用为2●的敌军" not "enemy with cost 2●"), German needs verb-final in subordinate clauses
4. **Passive participles need gender agreement** — "is dissolved" → Russian "растворён" (m) / "растворена" (f)

**RLF already has all features needed for Phase 2** — `:from()` tag inheritance, `:match()` gender branching, multi-dimensional case×number variants, `@der`/`@ein` German articles, `@count` Chinese classifiers. The blocker is the serializer architecture (String vs Phrase), not RLF capabilities.

### Architectural Concerns

The architecture reviewer argued Phase 1's escaped-brace pattern is an anti-pattern (string-generating-strings, double evaluation, false localization claims). While the team chose to proceed incrementally rather than skipping to Phase 2, the concerns informed the scoping decisions:

- Skip the predicate serializer (most throwaway work)
- Skip complex structural composition (needs Phrase returns)
- Be honest that escaped-brace phrases are temporary scaffolding

---

## Consensus Recommendation: Phase 1.5

### Guiding Principles

1. **Honest framing:** Phase 1.5 is semantic cataloging and code organization, not localization. English-only for now.
2. **Nothing throwaway except phrase bodies:** Phrase names, parameter signatures, and call-site refactoring all survive into Phase 2.
3. **Defer what needs Phrase composition:** Don't fight the framework with complex escaped-brace patterns.
4. **Validate after every task:** `just review` (not just `just parser-test`) after every task.

### What to Migrate in Phase 1.5

**Tier 1 — Full migration (leaf serializers):**
- `cost_serializer.rs` (131 lines, 12 arms) — fully self-contained, plan coverage is complete
- `trigger_serializer.rs` (127 lines, 16 arms) — plan coverage is good, add missing keyword handling
- `condition_serializer.rs` (96 lines, ~9 arms + `serialize_predicate_count` helper) — fill in the 12-arm helper the plan missed
- `serializer_utils.rs` (86 lines, 3 functions) — `serialize_operator` only; keep `capitalize_first_letter` and `lowercase_leading_keyword` as-is

**Tier 2 — Simple effect arms only:**
- effect_serializer simple arms: ~15-20 `StandardEffect` variants that take only counts (no predicates, no FormattedText). Examples: `DrawCards`, `DiscardCards`, `GainEnergy`, `GainPoints`, `Foresee`, `Kindle`, `TakeExtraTurn`, `PreventThatCard`, `YouWinTheGame`, `NoEffect`

**Tier 3 — Structural phrases (connectors and prefixes):**
- `you_may_prefix`, `cost_to_connector`, `once_per_turn_prefix`, `until_end_of_turn_prefix`, `fast_prefix`, `cost_effect_separator`, `then_joiner`, `and_joiner`, `cost_or_connector`, `cost_and_connector`
- These are self-contained text fragments with no `$target` parameters — they're real Phase 2 quality.

### What to DEFER to Phase 2

**All of these require Phrase-based composition to work correctly:**

- `predicate_serializer.rs` (entire file) — 800 lines, 15+ public functions, FormattedText coupling, called by everything
- `text_formatting.rs` (entire file) — FormattedText struct with 5 output methods
- `static_ability_serializer.rs` (entire file) — circular dependency with effect_serializer, complex conditional string building
- `ability_serializer.rs` (entire file) — orchestrator with conditional capitalization logic
- effect_serializer complex arms: anything that calls `predicate_serializer`, `serialize_gains_reclaim`, `serialize_for_count_expression`, `serialize_void_gains_reclaim`, `serialize_allied_card_predicate`
- effect_serializer structural logic: `serialize_effect_with_context` (`Effect::List` 4 branches, `Effect::ListWithOptions`, `Effect::Modal`)
- `capitalize_first_letter` / `lowercase_leading_keyword` interaction with composed output

### Phrase Categories

When writing phrases in `strings.rs`, categorize them:

```rust
// =========================================================================
// Category A: Final phrases (no escaped braces, survive unchanged into Phase 2)
// =========================================================================
discard_your_hand_cost = "discard your hand";
take_extra_turn = "take an extra turn after this one.";
cost_or_connector = " or ";
you_may_prefix = "you may ";

// =========================================================================
// Category B: Temporary phrases (escaped braces, will be rewritten in Phase 2)
// Phase 2: these require Phrase-based composition for i18n
// =========================================================================
draw_cards_effect($c) = "draw {{cards($c)}}.";
banish_another_in_void = "{{Banish}} another card in your void";
```

### Prep Work for Phase 2 (zero-risk, do during Phase 1.5)

Add gender/animacy tags to predicate terms in `strings.rs` even though they won't be used yet:

```rust
// Current:
ally = :an { one: "ally", other: "allies" };

// Enriched (ready for Phase 2 translations):
ally = :an :anim { one: "ally", other: "allies" };
enemy = :an :anim { one: "enemy", other: "enemies" };
character = :a :anim { one: "character", other: "characters" };
event = :an :inan { one: "event", other: "events" };
card = :a :inan { one: "card", other: "cards" };
```

### Tracking: Escaped-Brace Phrases to Rewrite in Phase 2

Maintain a tracked list (in the Phase 2 plan or a tracking file) of every escaped-brace phrase, so they don't become permanent tech debt:

```
# Phase 2 Phrase Rewrites Needed
# Each of these phrases uses {{...}} escaping and must be rewritten
# when serializers return Phrase instead of String

## Cost phrases
- abandon_count_allies($a) = "abandon {{count_allies($a)}}";
- discard_cards_cost($d) = "discard {{cards($d)}}";
- energy_cost_value($e) = "{{energy($e)}}";
- banish_another_in_void = "{{Banish}} another card in your void";
... (complete list maintained as phrases are added)
```

### Validation Gate

After EVERY task (not just at the end):
```bash
just review    # clippy + style validator + ALL tests including cards_toml_round_trip_tests
```

If `just review` fails, stop and fix before proceeding to the next task. Do not accumulate failures.

---

## Phase 2 Plan Sketch

Phase 2 is a separate planning effort, but here's the recommended order based on team analysis:

1. **Migrate predicate_serializer to return `Phrase`**
   - Replace `FormattedText` with real RLF terms carrying `:a`/`:an` tags and `one`/`other` variants
   - `serialize_predicate()` returns `Phrase` instead of `String`
   - This is the hardest single step but self-contained

2. **Restructure round-trip tests**
   - Split into parse tests (text → AST) and serialize tests (AST → rendered text)
   - Or: compare `parse(text) == parse(serialize(parse(text)))` instead of `text == serialize(parse(text))`
   - This removes the constraint that forces escaped braces

3. **Migrate remaining serializers to accept `Phrase` parameters**
   - Cost, trigger, condition serializers: change `$target: String` → `$target: Phrase`
   - Rewrite escaped-brace phrase bodies to use real RLF references
   - Effect serializer complex arms: now possible with Phrase composition

4. **Migrate structural composition**
   - `Effect::List`, `Effect::ListWithOptions`, `Effect::Modal`
   - `ability_serializer` orchestration
   - `serialize_gains_reclaim` and complex helpers

5. **Remove `eval_str` from display layer**
   - Serializer now returns fully rendered text
   - Display layer uses serializer output directly

6. **Remove `VariableBindings` from serializer return path**
   - Variables become internal to serialization, not exposed to callers

7. **Remove `FormattedText`, `capitalize_first_letter`, `lowercase_leading_keyword`**
   - All replaced by RLF Phrase composition and transforms

---

## Appendix: Agent Review Summaries

| Agent | Key Finding | Review File |
|-------|------------|-------------|
| RLF Architect | Escaped braces work correctly; `$param` inside `{{...}}` is literal text, not evaluated; two-layer approach is sound; all target languages supported by RLF | `review-rlf-architect.md` |
| I18n Specialist | Phase 1 is ~80% throwaway for i18n; `$target` as String blocks 4/6 languages; worked through 5 real examples in 6 languages showing concrete failures | `review-i18n-specialist.md` |
| Codebase Analyst | ~40-50% of code paths unaddressed; `serialize_gains_reclaim`, `serialize_for_count_expression`, `serialize_void_gains_reclaim` completely missed; FormattedText elimination strategy absent | `review-codebase-analyst.md` |
| Testing/Risk Analyst | Escaped-brace behavior confirmed (resolved by RLF architect); `cards_toml_round_trip_tests` must run after every task; predicate partial migration is the highest-risk operation | `review-testing-risk.md` |
| Architecture Reviewer | Recommends skipping Phase 1 for Phase 2 directly (overruled 4-1 on risk grounds); round-trip tests should be restructured; RLF confirmed as right framework vs ICU/Fluent/gettext | `review-architecture.md` |
