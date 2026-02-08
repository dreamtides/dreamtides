# Codebase Analyst Review: Serializer RLF Migration Plan

## Executive Summary

The plan has a **fundamental architectural misunderstanding** about what the serializer outputs today and how it interacts with `eval_str`. The plan goes through multiple revisions in Task 1 (Steps 3, 3-revised, 3-final) trying to reconcile this, and arrives at a double-brace `{{...}}` escaping approach that is workable but fragile. Beyond this, the plan has significant coverage gaps — roughly **40-50% of actual code paths** in the larger serializers are not explicitly addressed.

---

## 1. Coverage Analysis

### 1.1 cost_serializer.rs (131 lines, 12 match arms)

**Plan coverage: Good.** The plan explicitly addresses all `Cost::*` variants in Task 1 including:
- `AbandonCharactersCount` (with sub-matches on `CollectionExpression`)
- `DiscardCards`, `DiscardHand`, `Energy`, `LoseMaximumEnergy`
- All `Banish*` variants
- `BanishFromHand`, `Choice`, `ReturnToHand` (8 sub-variants), `SpendOneOrMoreEnergy`
- `BanishAllCardsFromYourVoid`, `CostList`
- `serialize_trigger_cost`

**Gap: None for cost_serializer.** This is the best-covered serializer in the plan.

### 1.2 trigger_serializer.rs (127 lines, 16 match arms)

**Plan coverage: Good.** Task 2 lists phrases for all trigger variants.

**Gap: Keywords handling.** The plan says "Keep keyword serialization as-is" but doesn't address the complex `TriggerEvent::Keywords` arms:
- Single keyword: `format!("{{{}}}", serialize_keyword(&keywords[0]))` → produces `{Judgment}`, `{Materialized}`, `{Dissolved}`
- Two keywords: `format!("{{{}_{}}}", ...)` → produces `{Materialized_Judgment}`, `{Materialized_Dissolved}`, etc.
- N keywords (fallback): `format!("{{{}}}", keyword_text)` → joins with underscore

These are **RLF directive references** in the output. The plan doesn't explain how these would be migrated. They reference `strings.rs` phrases like `judgment`, `materialized`, `dissolved`, `materialized_judgment`, `materialized_dissolved`. Since these are capitalized in the output (`{Judgment}`), `eval_str` later capitalizes them via the `@cap`-like behavior. **This is a non-trivial interaction the plan doesn't address.**

### 1.3 condition_serializer.rs (96 lines, ~9 match arms + 1 helper)

**Plan coverage: Partial.** Task 3 lists phrases for 7 conditions.

**Gaps:**
1. **`serialize_predicate_count` helper function** (lines 47-94): This is a critical helper with **12 match arms** that the plan doesn't mention at all:
   - `Predicate::Another(CardPredicate::CharacterType(_))` → `{count_allied_subtype($a, $t)}`
   - `Predicate::Another(CardPredicate::Character)` → `{count_allies($a)}`
   - `Predicate::Another(card_predicate)` → delegates to `serialize_predicate_plural`
   - `Predicate::This`, `Predicate::It`, `Predicate::Them`, `Predicate::That` → all delegate to `serialize_predicate_plural`
   - `Predicate::Enemy`, `Predicate::Your`, `Predicate::Any`, `Predicate::AnyOther`, `Predicate::YourVoid`, `Predicate::EnemyVoid` → all delegate

2. **`CardsDiscardedThisTurn` has TWO match arms** (count=1 and general) — the plan mentions only one phrase `if_discarded_this_turn($target)` but the actual code has two distinct patterns, one for count=1 and one for general (though they currently produce the same text, the structural difference matters).

3. **`PredicateCount` has TWO match arms** — count=1 (special-cased for `CharacterType`) and general. The plan lists `with_allied_subtype($t)` for the count=1 case but doesn't address the general case which calls `serialize_predicate_count`.

### 1.4 predicate_serializer.rs (803 lines, ~100+ match arms across 10 functions)

**Plan coverage: Partial (~60%).** Tasks 4-6 address base terms, card predicates, and ownership qualifiers.

**Gaps (critical):**

1. **`serialize_predicate` (lines 8-49, 10 arms)**: Plan Task 4 mentions this but doesn't show concrete migration for all arms. Key issue: `Predicate::Your` has complex branching logic:
   ```rust
   Predicate::Your(card_predicate) => {
       if is_generic_card_type(card_predicate) {
           serialize_card_predicate(card_predicate, bindings)
       } else if let CardPredicate::CharacterType(subtype) = card_predicate {
           "{@a subtype($t)}"
       } else {
           serialize_card_predicate(card_predicate, bindings)
       }
   }
   ```
   This conditional logic within a single arm doesn't map cleanly to a single RLF phrase.

2. **`predicate_base_text` (lines 85-145, 10 arms)**: Not explicitly addressed in any task. This function is used by:
   - `Counterspell`/`CounterspellUnlessPaysCost` in effect_serializer
   - `BanishCharacterUntilLeavesPlay` in effect_serializer
   - `CopyNextPlayed` in effect_serializer
   - Various places in effect_serializer

   The `Predicate::Enemy` arm inside `predicate_base_text` has **4 sub-cases** (generic type, `CouldDissolve`, `CardWithCost`, other) — none are mentioned in the plan.

3. **`your_predicate_formatted` (lines 531-613, 16 arms)**: Returns `FormattedText`, not `String`. This is a core building block. The plan mentions it in Task 4 ("Refactor text_formatting.rs") but doesn't explain how `FormattedText` with its `.with_article()`, `.without_article()`, `.plural()`, `.capitalized()`, `.capitalized_with_article()` methods would be replaced. Each of these 16 arms produces a `FormattedText` with different article behavior (`.new()`, `.new_non_vowel()`, `.with_plural()`). **This is 5 distinct output methods × 16 variants = up to 80 possible outputs.**

4. **`enemy_predicate_formatted` (lines 615-696, 16 arms)**: Same complexity as `your_predicate_formatted`. Not explicitly addressed.

5. **`serialize_your_predicate_plural` (lines 698-779, 16 arms)**: Not explicitly addressed.

6. **`serialize_enemy_predicate_plural` (lines 781-795, 3 arms)**: Not explicitly addressed.

7. **`serialize_for_each_predicate` (lines 470-529, 25+ arms)**: Task 6 lists some `for_each_*` phrases but only covers ~13 of the 25 explicit match arms. Missing:
   - `Your(CharacterType(_))`, `Enemy(CharacterType(_))`, `Any(CharacterType(_))`
   - `AnyOther(Character)`, `AnyOther(CharacterType(_))`
   - `YourVoid(Character)`, `YourVoid(CharacterType(_))`, `YourVoid(Event)`
   - `EnemyVoid(Card)`, `EnemyVoid(Character)`, `EnemyVoid(Event)`
   - `Your(Event)`, `Another(CharacterWithSpark(..))`, `Your(CharacterWithSpark(..))`
   - Fallback arm `predicate => format!("each {}", predicate_base_text(predicate, bindings))`

8. **`serialize_fast_target` (lines 390-468, 16 arms)**: Task 5 mentions it but the plan's phrases don't cover this function. It mirrors `serialize_card_predicate` but outputs WITHOUT articles (`"character"` not `"a character"`). This is a separate serialization variant the plan glosses over.

9. **`serialize_card_predicate_without_article` (lines 247-275, 5 arms)**: The plan mentions it in Task 5's step list but provides no phrases for it. This function is called by `DiscardCardFromEnemyHand`, `ReturnRandomFromYourVoidToPlay`, and `predicate_base_text`.

10. **`serialize_cost_constraint_only` (lines 282-305, 2 arms)**: Mentioned briefly in Task 5, no phrases provided.

### 1.5 effect_serializer.rs (1193 lines, ~70 StandardEffect arms + 5 Effect arms + helpers)

**Plan coverage: ~50%.** Tasks 8-12 divide into phases but many arms are listed as "... more" without specifics.

**Gaps (critical):**

1. **`serialize_standard_effect` (lines 26-712)** — counting all match arms: **55 distinct `StandardEffect::*` variants.** The plan lists phrases for roughly 25-30 of them. Missing or incomplete:

   - `CreateStaticAbilityUntilEndOfTurn` — delegates to static_ability_serializer, not mentioned
   - `CreateTriggerUntilEndOfTurn` — has complex branching (keyword vs non-keyword trigger), not addressed
   - `DiscardCardFromEnemyHand` — not in any task's phrase list
   - `DiscardCardFromEnemyHandThenTheyDraw` — uses `text_formatting::card_predicate_base_text`, not addressed
   - `GainEnergyEqualToCost` — has 3 sub-cases (It/That, This, other), plan lists 2 phrases
   - `GainEnergyForEach` — uses `serialize_for_each_predicate`, listed but phrase doesn't match
   - `GainPointsForEach` — uses `serialize_for_count_expression`, listed
   - `GainsReclaim` — **massive complexity** (lines 1076-1192), 117 lines with nested helpers, 8 `CollectionExpression` sub-arms, singular vs plural reclaim directives, `this_turn` suffix, cost vs no-cost branching, special `@cap @a subtype($t)` usage. **THE PLAN DOES NOT MENTION THIS AT ALL.**
   - `EachMatchingGainsSpark` — uses private helper `serialize_allied_card_predicate`
   - `EachMatchingGainsSparkForEach` — uses both `serialize_allied_card_predicate` and `serialize_allied_card_predicate_plural`
   - `GainsSparkForQuantity` — uses `serialize_for_count_expression`
   - `SparkBecomes` — uses `serialize_allied_card_predicate`
   - `PutCardsFromYourDeckIntoVoid` — uses `{top_n_cards($v)}`
   - `PutCardsFromVoidOnTopOfDeck` — has count=1 vs count>1 branching
   - `CounterspellUnlessPaysCost` — not in any task
   - `MaterializeSilentCopy` — **massive complexity** (lines 381-416), 5 sub-cases including `QuantityExpression::ForEachEnergySpentOnThisCard`
   - `MaterializeFigments` — uses `{@a figment($g)}` and `{n_figments($n, $g)}`
   - `MaterializeFigmentsQuantity` — **very complex** (lines 427-456), nested figment_text construction + 3 quantity sub-cases
   - `ReturnToHand` — has 4 special cases (Any/Character, Another/Character, This, fallback)
   - `ReturnFromYourVoidToHand` — has YourVoid special case
   - `ReturnUpToCountFromYourVoidToHand` — uses `{up_to_n_events($n)}`
   - `BanishThenMaterialize` — has 4 sub-cases including `{up_to_n_allies($n)}` and `{it_or_them($n)}`
   - `BanishCharacterUntilLeavesPlay` — not in any task
   - `BanishUntilNextMain` — not in any task
   - `DiscoverAndThenMaterialize` — not in any task
   - `MaterializeCharacterAtEndOfTurn` — not in any task
   - `DisableActivatedAbilitiesWhileInPlay` — not in any task
   - `DrawMatchingCard` — not in any task
   - `TriggerJudgmentAbility` — 4 sub-cases, not in any task
   - `TriggerAdditionalJudgmentPhaseAtEndOfTurn` — not in any task
   - `AbandonAndGainEnergyForSpark` — not in any task
   - `AbandonAtEndOfTurn` — not in any task
   - `BanishWhenLeavesPlay` — not in any task
   - `DissolveCharactersQuantity` — not in any task
   - `PreventDissolveThisTurn` — not in any task
   - `GainsAegisThisTurn` — not in any task (and uses `{Aegis}` which is NOT in strings.rs!)
   - `GainsSparkUntilYourNextMainForEach` — not in any task
   - `GainTwiceThatMuchEnergyInstead` — listed in plan
   - `MaterializeCharacterFromVoid` — not in any task
   - `OpponentPaysCost` — not in any task
   - `PayCost` — not in any task
   - `SpendAllEnergyDissolveEnemy` — not in any task (also a multi-sentence effect)
   - `SpendAllEnergyDrawAndDiscard` — not in any task (also a multi-sentence effect)

2. **`serialize_effect_with_context` (lines 725-968)** — Task 12 mentions structural logic but the plan's coverage is very thin:
   - `Effect::WithOptions` (lines 732-758): Complex logic with `condition`, `optional`, `trigger_cost` interactions, `lowercase_leading_keyword` calls. Plan mentions `you_may_prefix` and `cost_to_connector` but doesn't address the **conditional lowercasing logic**.
   - `Effect::List` (lines 760-888): **4 major branches** — all_optional+all_have_trigger_cost, !all_optional+all_have_trigger_cost, all_optional without costs, default. Each branch has different joining behavior. Plan mentions "structural phrases" but doesn't address these 4 branches.
   - `Effect::ListWithOptions` (lines 889-946): Complex shared trigger cost logic, per-effect conditions, optional/mandatory mixing. Not addressed.
   - `Effect::Modal` (lines 947-966): Uses `{choose_one}`, `{bullet}`, `{energy($e1)}`, `{energy($e2)}` hardcoded variable names. Plan mentions these phrases exist but doesn't address how the variable naming (`e1`, `e2`) interacts with RLF.

3. **`serialize_for_count_expression` (lines 970-1040)** — **14 match arms**, not addressed in any task:
   - `QuantityExpression::Matching(predicate)` → delegates
   - `PlayedThisTurn(predicate)` → "card/character you have played this turn"
   - `AbandonedThisTurn(Character)` → "ally abandoned this turn"
   - `AbandonedThisTurn(CharacterType(_))` → "allied {subtype($t)} abandoned this turn"
   - `AbandonedThisWay(Character)` → "ally abandoned"
   - `AbandonedThisWay(CharacterType(_))` → "allied {subtype($t)} abandoned"
   - `ReturnedToHandThisWay(Character)` → "ally returned"
   - `ReturnedToHandThisWay(CharacterType(_))` → "allied {subtype($t)} returned"
   - `ReturnedToHandThisWay(predicate)` → generic fallback
   - `AbandonedThisTurn(predicate)` → generic fallback
   - `AbandonedThisWay(predicate)` → generic fallback
   - `ForEachEnergySpentOnThisCard` → "{energy_symbol} spent"
   - `CardsDrawnThisTurn(predicate)` → "card/character you have drawn this turn"
   - `DiscardedThisTurn(predicate)` → "card you have discarded this turn"
   - `DissolvedThisTurn(predicate)` → "character which dissolved this turn"

4. **Private helper functions** not addressed:
   - `serialize_allied_card_predicate` (lines 1042-1058, 2 arms)
   - `serialize_allied_card_predicate_plural` (lines 1061-1074, 2 arms)
   - `serialize_gains_reclaim` (lines 1076-1109, 4 arms)
   - `serialize_void_gains_reclaim` (lines 1111-1192, 8 arms)

### 1.6 static_ability_serializer.rs (222 lines, ~22 match arms)

**Plan coverage: ~50%.** Task 13 lists some phrases.

**Gaps:**
1. **`serialize_static_ability` (lines 11-59)**: Wrapping logic for `StaticAbility::StaticAbility` vs `StaticAbility::WithOptions`. The `WithOptions` arm has **5 sub-branches** based on condition type (ThisCardIsInYourVoid, CardsInVoidCount/PredicateCount, default). The plan doesn't address this structural logic.

2. **`StandardStaticAbility` arms not addressed:**
   - `PlayForAlternateCost` (lines 97-119): **Very complex** — has `card_type` context (Character/Event/None), optional `additional_cost`, optional `if_you_do`, and calls both `capitalize_first_letter` and `cost_serializer::serialize_cost`. This requires at least 3-4 different phrase patterns.
   - `MultiplyEnergyGainFromCardEffects` (line 128-132): uses `{multiply_by($n)}`
   - `MultiplyCardDrawFromCardEffects` (lines 133-137): uses `{multiply_by($n)}`
   - `OncePerTurnPlayFromVoid` (lines 138-143): uses `serialize_card_predicate`
   - `YouMayPlayFromTopOfDeck` (lines 150-155): uses `text_formatting::card_predicate_base_text(...).plural()`
   - `JudgmentTriggersWhenMaterialized` (lines 156-161): uses `{Judgment}` and `{materialize}`
   - `SparkEqualToPredicateCount` (lines 162-167): not in plan
   - `PlayFromHandOrVoidForCost` (lines 171-177): not in plan
   - `CostReductionForEach` (lines 181-187): calls `effect_serializer::serialize_for_count_expression`
   - `SparkBonusYourCharacters` (lines 188-198): has 3 sub-cases with special "allies" handling
   - `PlayFromVoid` (lines 199-219): **Very complex** — optional energy_cost, optional additional_cost with `capitalize_first_letter`, optional `if_you_do` effect, dynamic string building

### 1.7 ability_serializer.rs (176 lines, 5 ability types + 2 named)

**Plan coverage: ~30%.** Task 14 only lists 2 phrases.

**Gaps:**
1. **`Ability::Triggered`** (lines 27-61): Complex prefix logic — `has_once_per_turn`, `has_until_end_of_turn`, conditional capitalization, keyword vs non-keyword trigger branching. The plan lists `once_per_turn_prefix` and `until_end_of_turn_prefix` but doesn't address the **conditional capitalization logic**: `result.push_str(if has_prefix { &trigger } else { &capitalized_trigger })`.

2. **`Ability::Activated`** (lines 66-99): Multi-cost joining with `, `, optional `{Fast} --` prefix, optional "once per turn" suffix, `: ` separator. Plan mentions `fast_prefix` and `cost_effect_separator` but doesn't address how multiple costs are joined.

3. **`serialize_ability_effect`** (lines 113-125): Separate function for effect-only serialization. Not addressed.

4. **`serialize_modal_choices`** (lines 130-156): Modal choice extraction logic. Not addressed.

### 1.8 serializer_utils.rs (86 lines, 3 public functions)

**Plan coverage: Partial.** Task 7 addresses `serialize_operator`.

**Gaps:**
1. **`capitalize_first_letter`** (lines 20-31): The plan correctly identifies this needs to persist but doesn't explain how it interacts with RLF phrases. This function has special logic for keyword capitalization (`is_capitalizable_keyword`) — it only capitalizes known keywords in `{...}` and delegates to `capitalize_string` for plain text. **This function is called ~15 times across the codebase** and is fundamental to the serializer's output format.

2. **`lowercase_leading_keyword`** (lines 4-12): Called by `Effect::WithOptions` and `Effect::List` to lowercase `{Banish}` → `{banish}` when preceded by "you may" or a trigger cost. **This is a template-text operation** that won't have an equivalent in RLF phrases if the phrases use `{{Banish}}` escaping.

### 1.9 text_formatting.rs (79 lines, FormattedText struct + 1 function)

**Plan coverage: Deferred to Task 15.** But the plan says "After Tasks 4-6, most of text_formatting.rs is redundant."

**Reality: FormattedText is deeply embedded.** It's used by:
- `predicate_serializer::your_predicate_formatted` (returns FormattedText)
- `predicate_serializer::enemy_predicate_formatted` (returns FormattedText)
- `effect_serializer::serialize_for_count_expression` (8+ calls to `.without_article()`, `.plural()`)
- `effect_serializer::serialize_allied_card_predicate` (calls `.without_article()`)
- `static_ability_serializer::YouMayPlayFromTopOfDeck` (calls `.plural()`)

`FormattedText` methods:
- `.with_article()` → "a character" or "an event" (vowel-sound aware)
- `.without_article()` → "character"
- `.plural()` → "characters"
- `.capitalized()` → "Character"
- `.capitalized_with_article()` → "A character"
- `.new_non_vowel()` → forces "a" article (for "non-warrior enemy")

**The plan can't remove FormattedText until ALL predicate serialization is migrated**, because the callers depend on the article/plural/capitalization variants. RLF's `:a`/`:an` metadata could replace `.with_article()`, but the plan doesn't explain how `.without_article()`, `.plural()`, `.capitalized()`, and `.capitalized_with_article()` would be replaced.

---

## 2. Edge Cases the Plan Misses

### 2.1 `serialize_gains_reclaim` — The Most Complex Function

Lines 1076-1192 in effect_serializer.rs. This is **117 lines of interconnected logic** that the plan never mentions:

- **Two reclaim directives**: `{reclaim}` (no cost) and `{reclaim_for_cost($r)}` (with cost)
- **Two suffix variants**: "equal to its cost" (singular) and "equal to their cost" (plural)
- **`this_turn` boolean suffix**: appends " this turn"
- **4 target types**: `It`, `This`, `YourVoid(predicate)`, and generic
- **`serialize_void_gains_reclaim` sub-helper** with 8 `CollectionExpression` arms
- **Special `@cap @a subtype($t)` usage** in the Exactly(1)+CharacterType case

This would need ~15-20 RLF phrases to cover all paths.

### 2.2 `{Aegis}` keyword — Not defined in strings.rs

The effect serializer outputs `"{} gains {Aegis} this turn."` but there is NO `aegis` phrase in `strings.rs`. This means `eval_str` either handles it via a fallback mechanism or it's already handled elsewhere. **The plan doesn't acknowledge this missing keyword.**

### 2.3 Capitalization-dependent context

The serializer uses `capitalize_first_letter` and `lowercase_leading_keyword` extensively to handle context-dependent capitalization:
- `{Banish}` (capitalized) as a cost prefix → `{banish}` (lowercase) after "you may"
- Effect text is capitalized when used as sentence start vs lowercase when after trigger text

If RLF phrases use `{{Banish}}` escaping to output `{Banish}`, the lowercasing logic in `lowercase_leading_keyword` would still need to work on the phrase output. But `lowercase_leading_keyword` operates on template syntax (`{...}`), not on RLF output. **This creates a semantic conflict**: either the phrases output template syntax (and lowercasing works) or they output evaluated text (and lowercasing breaks).

### 2.4 Multi-sentence effects

Several effects produce multi-sentence text:
- `SpendAllEnergyDissolveEnemy`: "spend all your {energy_symbol}. {dissolve} an enemy..."
- `SpendAllEnergyDrawAndDiscard`: "spend all your {energy_symbol}. Draw cards..."
- `DiscardCardFromEnemyHandThenTheyDraw`: "discard a chosen ... They draw {cards($c)}."

These contain internal sentence boundaries with capitalization. A single RLF phrase would handle this naturally, but the plan's task division (simple effects vs target effects) doesn't account for these.

### 2.5 Recursive predicate composition

`CardPredicate::CardWithCost { target, ... }` and `CardPredicate::Fast { target }` are recursive — they wrap another `CardPredicate`. The serializer handles this by calling itself recursively:
```rust
CardPredicate::CardWithCost { target, cost_operator, cost } => {
    format!("{} with cost {energy($e)}{}", serialize_card_predicate(target, ...), ...)
}
```

This recursion means you can't have a fixed set of RLF phrases for card predicates — you'd need composable phrases where the `$target` parameter is itself a phrase result. The plan's approach of passing "pre-rendered predicate strings" works for the `{{...}}` escaping approach but is semantically weak for future i18n.

### 2.6 `Effect::List` branching complexity

`Effect::List` has **4 entirely different code paths** based on:
1. `all_optional && all_have_trigger_cost` → "you may [cost] to [effect1] and [effect2]."
2. `!all_optional && all_have_trigger_cost` → "[cost] to [effect1] and [effect2]."
3. `all_optional && !all_have_trigger_cost` → "you may [effect1], then [effect2]."
4. Default (mandatory, no shared cost):
   - Triggered context → "[effect1], then [effect2]."
   - Event context → "[Effect1]. [Effect2]."

Each path has different joining behavior, different capitalization rules, and different condition handling. The plan mentions "structural phrases" but doesn't address how these 4 branches would be expressed as RLF phrase compositions.

---

## 3. Migration Order Analysis

### 3.1 Proposed order: cost → trigger → condition → predicate → effect → ability

**This order is CORRECT for dependencies.** The dependency graph is:

```
ability_serializer
  ├── trigger_serializer
  │     └── predicate_serializer
  ├── cost_serializer
  │     └── predicate_serializer
  ├── effect_serializer
  │     ├── predicate_serializer
  │     ├── cost_serializer
  │     ├── condition_serializer
  │     │     └── predicate_serializer
  │     ├── trigger_serializer
  │     ├── static_ability_serializer
  │     │     ├── predicate_serializer
  │     │     ├── cost_serializer
  │     │     ├── condition_serializer
  │     │     ├── effect_serializer (recursive!)
  │     │     └── text_formatting
  │     ├── text_formatting
  │     └── serializer_utils
  └── static_ability_serializer
```

**Key concern: Circular dependency.** `static_ability_serializer` calls `effect_serializer::serialize_effect` (for `PlayFromVoid.if_you_do`), and `effect_serializer` calls `static_ability_serializer::serialize_standard_static_ability` (for `CreateStaticAbilityUntilEndOfTurn`). This means they can't be migrated fully independently — you'd need to ensure the output format is compatible during the migration.

**Key concern: predicate_serializer must come before effect_serializer.** The plan puts predicate migration (Tasks 4-6) before effect migration (Tasks 8-12). This is correct, but the plan splits predicate into 3 phases with commits between them. If effect serializer relies on predicate output format, partial predicate migration could cause issues.

### 3.2 `text_formatting.rs` timing

The plan puts `text_formatting.rs` cleanup in Task 15 (last). But `text_formatting` is used by:
- `predicate_serializer` (Tasks 4-6)
- `effect_serializer` (Tasks 8-12)
- `static_ability_serializer` (Task 13)

If we're migrating these to RLF, we need to decide HOW `FormattedText` callers will change during the migration, not after.

---

## 4. text_formatting.rs Analysis

### What FormattedText actually does

`FormattedText` is a **text variant generator**. Given a base form ("character"), it produces:
- Singular with article: "a character"
- Singular without article: "character"
- Plural: "characters"
- Capitalized: "Character"
- Capitalized with article: "A character"

It also handles:
- Vowel-sound detection for a/an ("an event" vs "a character")
- `new_non_vowel` override for cases like "non-{subtype($t)} enemy" (forces "a" even though "non" starts with 'n')
- Custom plural forms via `with_plural` ("child"/"children" — though this isn't used for these, it's used for subtypes like `{subtype($t)}/{@plural subtype($t)}`)

### Can it be replaced by RLF?

**Partially.** RLF already has `:a`/`:an` article metadata and `@a`/`@plural` transforms. So:
- `.with_article()` → `{@a term}` in RLF
- `.plural()` → `{term:other}` or `{@plural term}` in RLF

But RLF doesn't currently have:
- `.without_article()` equivalent (you'd need the bare phrase form)
- `.capitalized()` equivalent (you'd need `@cap` or a capitalization transform)
- `.capitalized_with_article()` equivalent (you'd need `@cap @a`)

The `card_predicate_base_text` function maps `CardPredicate` to `FormattedText` and is called by 8+ functions. Replacing it requires each caller to know which variant it needs and call the appropriate RLF phrase/transform.

### Recommendation

Don't try to eliminate `FormattedText` in this migration. Instead:
1. Keep `FormattedText` as-is for now
2. Have RLF phrases produce the same strings that `FormattedText` would produce
3. Plan a separate refactor to replace `FormattedText` with RLF article metadata in Phase 2

---

## 5. Patterns That Don't Fit Simple RLF Phrases

### 5.1 Dynamic string building with conditionals

`static_ability_serializer::PlayForAlternateCost`:
```rust
let card_type = match alt_cost.card_type {
    Some(CardTypeContext::Character) => "character",
    Some(CardTypeContext::Event) => "event",
    None => "card",
};
if let Some(cost) = &alt_cost.additional_cost {
    let base = format!("{}: Play this {} for {energy($e)}", capitalize(cost), card_type);
    if alt_cost.if_you_do.is_some() { format!("{}, then abandon it.", base) }
    else { format!("{}.", base) }
} else {
    format!("this {} costs {energy($e)}", card_type)
}
```

This has 6 possible outputs based on 3 conditionals. You'd need either 6 RLF phrases or a compositional approach.

### 5.2 Dynamic string building with push_str

`static_ability_serializer::PlayFromVoid`:
```rust
let mut result = String::new();
if let Some(cost) = &play_from_void.additional_cost {
    result.push_str(&format!("{}: ", capitalize(cost)));
}
result.push_str("play this card from your void for {energy($e)}");
if let Some(effect) = &play_from_void.if_you_do {
    result.push_str(&format!(", then {}", effect_text.trim_end_matches('.')));
}
result.push('.');
```

This is imperative string building with 4 possible output patterns. Each would need a separate RLF phrase.

### 5.3 The `capitalize_first_letter` / `lowercase_leading_keyword` dance

Throughout `ability_serializer` and `effect_serializer`:
```rust
// Capitalize for sentence start
serializer_utils::capitalize_first_letter(&effect_serializer::serialize_effect(...))

// Lowercase after "you may"
serializer_utils::lowercase_leading_keyword(&effect_str)
```

These operate on template text (`{banish}` → `{Banish}` and back). If RLF phrases produce `{{Banish}}` (which becomes `{Banish}` in output), then `lowercase_leading_keyword` would transform this to `{banish}`, which is correct. But if phrases produce evaluated text, these functions break.

**The `{{...}}` escaping approach preserves this interaction**, which is good for incremental migration.

### 5.4 `trim_end_matches('.')` pattern

Used 15+ times in `effect_serializer.rs` to strip trailing periods before joining effects:
```rust
serialize_standard_effect(&e.effect, bindings).trim_end_matches('.')
```

If RLF phrases include the period (e.g., `draw_cards_effect = "draw {cards($c)}."`) then the callers would still need to trim. This is a string manipulation on phrase output that works fine with the `{{...}}` approach but is semantically unclean.

---

## 6. What the Serializer Actually Outputs Today

### Current output format

The serializer produces **template text** containing:

1. **RLF function call directives**: `{energy($e)}`, `{cards($c)}`, `{foresee($f)}`, `{kindle($k)}`, `{points($p)}`, `{count($n)}`, `{subtype($t)}`, `{@a subtype($t)}`, `{@plural subtype($t)}`, `{@cap @a subtype($t)}`, `{count_allies($a)}`, `{count_allied_subtype($a, $t)}`, `{top_n_cards($v)}`, `{up_to_n_events($n)}`, `{up_to_n_allies($n)}`, `{it_or_them($n)}`, `{n_random_characters($n)}`, `{text_number($n)}`, `{cards_numeral($c)}`, `{this_turn_times($n)}`, `{multiply_by($n)}`, `{n_figments($n, $g)}`, `{@a figment($g)}`, `{figment($g)}`, `{reclaim_for_cost($r)}`, `{Reclaim_For_Cost($r)}`

2. **Keyword references** (resolved to colored text by eval_str): `{dissolve}`, `{banish}`, `{banished}`, `{materialize}`, `{reclaim}`, `{prevent}`, `{discover}`, `{fast}`, `{Banish}`, `{Judgment}`, `{Materialized}`, `{Dissolved}`, `{Materialized_Judgment}`, `{Materialized_Dissolved}`, `{Aegis}`, `{Fast}`, `{Reclaim}`, `{Reclaim_For_Cost($r)}`

3. **Display formatting references**: `{energy_symbol}`, `{choose_one}`, `{bullet}`, `{judgment_phase_name}`

4. **Plain English text** with standard punctuation

### Example complete output

For an activated ability with a cost and effect:
```
{Fast} -- {Banish} another card in your void, once per turn: {dissolve} an enemy.
```

Which `eval_str` renders to:
```
⚡fast -- <color=#AA00FF>Banish</color> another card in your void, once per turn: <color=#AA00FF>dissolve</color> an enemy.
```

### The plan's claims are correct that:
- The serializer outputs template text with `{directives}`
- `eval_str` converts this to final rendered text
- The round-trip tests compare serializer output against the original template text

---

## 7. Summary of Critical Issues

### Severity: HIGH

1. **~40-50% of code paths unaddressed**: The plan leaves most of effect_serializer and significant parts of predicate_serializer as "... more" or unmentioned. These aren't trivial — they include the most complex logic in the codebase.

2. **`serialize_gains_reclaim` completely missed**: 117 lines of interconnected logic with 12+ code paths, not mentioned anywhere in the plan.

3. **`serialize_for_count_expression` completely missed**: 14 match arms used by 7+ callers, not in any task.

4. **`serialize_void_gains_reclaim` completely missed**: 8 collection expression arms with singular/plural reclaim variant logic.

5. **FormattedText elimination strategy absent**: The plan defers this to Task 15 but doesn't explain how the 5 output methods would be replaced during the migration.

### Severity: MEDIUM

6. **`capitalize_first_letter` / `lowercase_leading_keyword` interaction**: These functions operate on template syntax. The `{{...}}` escaping approach preserves compatibility, but the plan should explicitly confirm this works.

7. **`{Aegis}` keyword missing from strings.rs**: Not defined, but used in serializer output.

8. **Circular dependency**: `static_ability_serializer` ↔ `effect_serializer`. The plan's migration order doesn't account for this.

9. **`Effect::List` 4-branch complexity**: The plan's structural phrases are far too simple to cover the actual joining logic.

10. **Keywords output format**: Trigger keywords (`{Judgment}`, `{Materialized}`, `{Materialized_Dissolved}`) are template directives that reference strings.rs phrases. The plan doesn't explain how these would be migrated.

### Severity: LOW

11. **Private helper functions**: `serialize_allied_card_predicate`, `serialize_allied_card_predicate_plural`, `is_generic_card_type` — these internal helpers aren't addressed but would be migrated as part of their parent function migration.

12. **Task granularity**: Splitting effect_serializer into 5 tasks (8-12) is reasonable but the actual split criteria aren't well-defined.

13. **`trim_end_matches('.')` pattern**: Works with `{{...}}` approach but is inelegant.
