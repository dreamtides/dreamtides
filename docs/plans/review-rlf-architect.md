# RLF Architect Review: Serializer RLF Migration Plan

**Reviewer**: RLF Architect Agent
**Date**: 2026-02-07
**Document**: `2026-02-07-serializer-rlf-migration.md`
**RLF Reference**: `~/rlf/docs/DESIGN.md` and appendices

---

## 1. Escaped Braces: `{{...}}` Behavior

### 1.1 Does `{{Banish}}` produce literal `{Banish}`?

**Yes, confirmed.** This is correct and well-tested.

From DESIGN.md lines 706-710:

```
| Sequence | Output | Where |
|----------|--------|-------|
| `{{`     | `{`    | Anywhere in text |
| `}}`     | `}`    | Anywhere in text |
```

The RLF template parser (`rlf/crates/rlf/src/parser/template.rs` lines 96-103) processes escape sequences **before** interpolation parsing via `alt()` priority:

```rust
fn escape_sequence(input: &mut &str) -> ModalResult<Segment> {
    alt((
        "{{".value(Segment::Literal("{".to_string())),
        "}}".value(Segment::Literal("}".to_string())),
    ))
    .parse_next(input)
}
```

This is tried first in the parser's main `alt()` chain (line 93), so `{{` is consumed as an escape before it could be interpreted as an interpolation start. The test `test_escape_braces()` in `template_parser.rs:545-550` confirms:
- Input: `"Use {{name}} syntax"`
- Expected: `"Use {name} syntax"` (single literal segment)

**Verdict: The plan's assumption about `{{Banish}}` producing `{Banish}` is CORRECT.**

### 1.2 Does `{{count_allies($a)}}` work correctly?

**Yes, confirmed.** The `$a` inside escaped braces is treated as **literal text**, NOT as a parameter reference.

The critical test is `test_escaped_braces_with_dollar_param()` in `template_parser.rs:628-634`:
- Input: `"Use {{$name}} for params"`
- Expected: `"Use {$name} for params"` (single literal segment, `$name` is literal text)

And the end-to-end test in `rlf-macros/tests/pass/escape_sequences.rs:3-23`:
- Input: `syntax_help = "Use {{$name}} for parameters.";`
- Assertion: `assert_eq!(s.to_string(), "Use {$name} for parameters.");`

**How it works**: When the parser sees `{{`, it immediately emits a `Segment::Literal("{")` and consumes both characters. Everything until the next `}}` is then parsed as regular text (including `$`, `@`, `:` which are only special inside `{}` interpolations per DESIGN.md lines 696-698). The `}}` emits `Segment::Literal("}")`.

So `{{count_allies($a)}}` produces the literal string `{count_allies($a)}` — the `$a` inside is NOT evaluated by RLF. This is exactly what the plan needs.

**Verdict: The plan's assumption (line 390) is CORRECT.** `{{count_allies($a)}}` produces `{count_allies($a)}` as literal text. The plan correctly identified this behavior.

### 1.3 Important nuance: Adjacent escaped braces

One subtle case: `{{energy($e)}}` — does this produce `{energy($e)}` or `{energy(` followed by an evaluated `$e` followed by `)}` ?

Answer: It produces the literal `{energy($e)}`. The parser's `escape_sequence` function processes `{{` as a single unit, then all subsequent text (including `$e`) is parsed as literal characters, then `}}` is consumed as a closing escape. The entire content between `{{` and `}}` is literal. This is the key parser behavior: `{{` switches off interpolation parsing until `}}`.

**Verdict: All escaped brace patterns in the plan are sound.**

---

## 2. Phrase Patterns

### 2.1 Can RLF handle all proposed phrase patterns?

**Yes, with caveats.** Let me analyze each category:

#### Simple parameterized phrases
```rust
abandon_any_number_of($target) = "abandon any number of {$target}";
```
**Works.** `$target` receives a String and is interpolated directly. No issues.

#### Escaped brace phrases
```rust
discard_cards_cost($d) = "discard {{cards($d)}}";
```
**Works BUT with a subtle issue.** This produces `discard {cards($d)}` as literal text — the `$d` is NOT evaluated. This means the output contains a literal `$d` character, not the value of the parameter.

**This is a problem if the plan expects `$d` to be substituted.** Looking at the plan (line 386): "The `$a`, `$c`, `$d`, `$e`, `$m`, `$n` parameters receive integers."

If the intent is that `discard_cards_cost(3)` produces `discard {cards(3)}`, then escaped braces alone **do not work** — the `$d` would be literal, producing `discard {cards($d)}`.

**SHOW-STOPPER for Task 1's "Revised RLF Phrases" section (lines 347-384).** Phrases like:
```rust
abandon_count_allies($a) = "abandon {{count_allies($a)}}";
energy_cost_value($e) = "{{energy($e)}}";
banish_cards_from_void($c) = "{{Banish}} {{cards($c)}} from your void";
```

These would produce:
- `abandon_count_allies(3)` → `"abandon {count_allies($a)}"` (WRONG: literal `$a`, not `3`)
- `energy_cost_value(5)` → `"{energy($e)}"` (WRONG: literal `$e`, not `5`)
- `banish_cards_from_void(2)` → `"{Banish} {cards($c)} from your void"` (WRONG: literal `$c`)

The plan's Note 2 (line 388) asks about this exact question and then answers it incorrectly on line 390: "So `{{count_allies($a)}}` should produce `{count_allies($a)}` literally — the `$a` inside the escaped braces should NOT be evaluated because it's inside an escape sequence."

The plan says this as if it's desired behavior, but it contradicts the stated goal. If the output is `{count_allies($a)}` with a literal `$a`, then `eval_str` in the display layer would need to resolve `$a` as a variable binding — which it CAN do because `VariableBindings` contains the mapping. So this actually works in the current pipeline: the serializer produces template text with `$a` literal, and `eval_str` resolves it from bindings.

**Let me re-read the pipeline more carefully.**

Looking at `rlf_helper.rs:11-20`, `eval_str` takes a template string and `VariableBindings`, converts bindings to RLF `Value` params, and calls `locale.eval_str(template, params)`. So the flow would be:

1. Serializer calls `strings::banish_cards_from_void(2).to_string()` → `"{Banish} {cards($c)} from your void"` (literal `$c`)
2. But wait — the serializer also does `bindings.insert("c", VariableValue::Integer(2))`
3. Later, `eval_str("{Banish} {cards($c)} from your void", bindings)` would resolve `$c` from bindings

**This DOES work**, but only if `eval_str` is called on the serializer output with the correct bindings. The plan's two-layer approach depends on this.

**Revised verdict: The escaped brace pattern works for the two-layer architecture, but only because `eval_str` resolves the literal `$c` from `VariableBindings`.** The RLF phrase itself does NOT substitute `$d`/`$c`/`$a` — it produces them as literal text. The plan's description on lines 354-384 is technically correct in its output, but the reasoning about WHY it works (line 390 saying "the `$a` inside the escaped braces should NOT be evaluated") is correct about the mechanism while being misleading about the implications. The plan works because the SECOND evaluation pass (eval_str) handles the resolution.

### 2.2 Are there phrase naming or arity issues?

Looking at the proposed phrases, several use names that could conflict with existing definitions in `strings.rs`:

- `discard_your_hand` (plan line 359) — this is a term (no params). But current `strings.rs` doesn't have it. OK.
- `energy_cost_value($e)` vs existing `energy($e)` — different names, OK.
- `cost_or_connector` vs no existing — OK.

**No compile-time conflicts detected.** The plan uses unique names for all new phrases.

### 2.3 Nested phrase calls

DESIGN.md line 164: `{f(g($x))}` is an **Error** — nested phrase calls not supported.

The plan doesn't propose any nested calls. All phrase compositions pass parameters directly or use pre-rendered strings. This is correct.

### 2.4 Phrase call return type

All generated functions return `Phrase` (DESIGN.md line 535). The plan uses `.to_string()` throughout, which is fine since `Phrase` implements `Display` (DESIGN.md line 589).

---

## 3. Missing RLF Features for Target Languages

### 3.1 Chinese (Mandarin)

**Required features**: Classifiers (measure words), no pluralization, topic-comment structure.

**RLF support**:
- Classifiers: **Fully supported** via `@count` transform with measure word tags (`:zhang`, `:ge`, `:ming`, etc.). See APPENDIX_STDLIB.md lines 70-101.
- No pluralization: CLDR category is `other` only (line 90). Works fine.
- Topic-comment structure: Handled by phrase template reordering. No special RLF feature needed — the translator writes Chinese word order in the template.

**Missing**: Nothing fundamental. Chinese works well with RLF's existing features.

### 3.2 Russian

**Required features**: 6 cases, 3 genders, animacy, number agreement.

**RLF support**:
- Cases: **Fully supported** via multi-dimensional variants (`nom.one`, `acc.many`, etc.). See APPENDIX_STDLIB.md lines 319-358 and APPENDIX_RUSSIAN_TRANSLATION.md.
- Genders: Tags (`:masc`, `:fem`, `:neut`) with `:match` and tag-based selection. Fully supported.
- Animacy: Tags (`:anim`, `:inan`). Supported.
- Number agreement: CLDR categories `one`, `few`, `many`, `other` (line 332). Fully supported.
- No special transforms needed (line 1112): "variant selection handles all the complexity."

**Missing**: Nothing. The APPENDIX_RUSSIAN_TRANSLATION.md demonstrates a complete predicate serializer translation using only selection. Russian is a flagship use case for RLF.

### 3.3 Spanish

**Required features**: 2 genders, personal "a" (a + el = al), verb conjugation agreement.

**RLF support**:
- Genders: Tags (`:masc`, `:fem`). Fully supported.
- Articles: `@el` (definite) and `@un` (indefinite) with gender and number context. See APPENDIX_STDLIB.md lines 160-191 and APPENDIX_SPANISH_TRANSLATION.md.
- Personal "a": This is the "a personal" that marks animate direct objects. RLF doesn't have a built-in transform for this, but it can be handled via:
  - Phrase-level: Define separate phrases for animate vs inanimate targets
  - Or add an `@a_personal` transform that contracts with definite articles
- Verb conjugation: Not handled by RLF — verbs would need to be hardcoded in each phrase template. For a card game, verb forms are typically fixed in each phrase (imperative mood for actions), so this isn't a significant limitation.

**Missing**:
- **Personal "a" contraction** (a + el = al, a + la = a la): Could be handled by a custom `@a` transform in Spanish (note: `@a` is aliased differently in Italian per APPENDIX_STDLIB.md line 685). Would need a Spanish-specific `@a_personal` transform or manual handling in templates. **Minor gap — workaround exists.**

### 3.4 Portuguese (Brazil)

**Required features**: Contractions (de + o = do, em + a = na), gender agreement.

**RLF support**:
- Contractions: **Fully supported** via `@de` and `@em` transforms. See APPENDIX_STDLIB.md lines 285-315:
  - `@de` → do/da/dos/das
  - `@em` → no/na/nos/nas
- Gender: Tags (`:masc`, `:fem`) with `@o` (definite) and `@um` (indefinite).

**Missing**: Nothing significant. Portuguese is well-covered.

### 3.5 German

**Required features**: 4 cases, 3 genders, compound nouns, verb-second word order.

**RLF support**:
- Cases & genders: **Fully supported** via `@der` and `@ein` transforms with compound context. See APPENDIX_STDLIB.md lines 398-453.
  - `@der:acc` → den/die/das (accusative)
  - `@der:nom.other` → die (nominative plural)
  - `@ein:acc` → einen/eine/ein (accusative indefinite)
- Multi-dimensional variants: Supported for noun declension.
- Compound nouns: Not a special feature — compound nouns are just longer strings in variant definitions. No special RLF handling needed.
- Verb-second word order: Handled by phrase template reordering. The translator controls word order.

**Missing**:
- **Adjective declension**: German adjectives decline based on the determiner type (strong/weak/mixed). This would need to be handled through carefully designed variant tables or a custom transform. **Moderate gap — but workaround exists through variant tables.**

---

## 4. RLF Limitations

### 4.1 No nested phrase calls

DESIGN.md line 164: `{f(g($x))}` is a compile error. This means you can't compose phrases within interpolation expressions. The plan correctly avoids this pattern, always pre-evaluating inner phrases in Rust and passing results as parameters.

### 4.2 No expressions as arguments

DESIGN.md line 165: `{f(card:one)}` is a compile error. Arguments must be simple: parameters, term names, numbers, or strings. You can't pass a selected variant as an argument.

**Impact on plan**: The plan passes pre-rendered strings for complex cases. This is the correct workaround.

### 4.3 Parameters inside escaped braces are literal

As analyzed in Section 1.2, `{{foo($x)}}` produces `{foo($x)}` with `$x` as literal text. This is not a limitation per se — it's the desired behavior for the two-layer architecture — but it means:

1. The RLF phrase doesn't validate that `$x` is a valid parameter
2. The literal `$x` in the output depends on `eval_str` having a binding for `x`
3. Typos in variable names inside `{{...}}` are NOT caught at compile time

**Recommendation**: Consider adding a test helper that validates all serializer output contains only known variable references before `eval_str` processes it.

### 4.4 Phrase returns String-like Phrase for simple templates

When an RLF phrase has no `:from`, no variants, and no tags, calling `.to_string()` returns the rendered text. But the result is still a `Phrase` type. This is fine for the plan's usage.

### 4.5 No runtime template composition

RLF's `eval_str` evaluates a template once. You can't build a template string dynamically and evaluate it in stages (e.g., first resolve `$target`, then resolve `{Banish}`). The two-layer approach (serializer → eval_str) effectively creates two evaluation stages, which works but is architecturally unusual.

### 4.6 Global locale feature dependency

The plan uses `strings::phrase_name()` without a `locale` parameter (matching current code in `strings.rs`). This works because the project uses `global-locale` feature (DESIGN.md lines 560-577). The plan correctly follows existing patterns.

---

## 5. Two-Layer Architecture Assessment

### 5.1 Architecture description

The plan proposes:
1. **Layer 1 (Serializer)**: Call RLF phrases that use `{{...}}` escaped braces to produce template text like `{Banish} {cards($c)} from your void`
2. **Layer 2 (Display)**: `eval_str` evaluates the template text using `VariableBindings` to produce final rendered text like `<color=#AA00FF>Banish</color> 2 cards from your void`

### 5.2 Is this sound?

**Yes, with important caveats.**

**Soundness argument**: The escaped-brace mechanism is well-defined and well-tested. RLF phrases with `{{...}}` produce deterministic literal output that can be consumed by `eval_str`. The two stages are independent — Layer 1 handles sentence structure (via RLF phrases), Layer 2 handles visual rendering (keyword coloring, symbol formatting).

**Advantages**:
1. Incremental migration — no need to change the parser or display layer
2. Round-trip tests continue to work (serializer output matches parser input format)
3. `VariableBindings` continues to carry variable values for both layers

**Risks and caveats**:

1. **Double-evaluation performance**: Each string is processed by RLF twice — once in the serializer (phrase evaluation) and once in `eval_str` (template evaluation). For a card game this is negligible, but worth noting.

2. **Variable name coupling**: The literal `$c`, `$e`, etc. in escaped braces must match the keys in `VariableBindings`. These names are now encoded in TWO places: the RLF phrase definition AND the `bindings.insert()` call. A rename in one without the other causes a silent runtime bug. **This is the most significant risk.**

3. **Compile-time validation gap**: Normal RLF phrase parameters are validated at compile time (DESIGN.md lines 636-647). But escaped-brace content is just literal text — `{{cards($typo)}}` compiles fine and only fails at runtime when `eval_str` can't find `$typo` in bindings. **The plan loses compile-time safety for variable references inside escaped braces.**

4. **Translation complexity**: Translators working on the two-layer system need to understand that `{{...}}` content is "pass-through" to a second evaluation stage. This is unusual and could be confusing. The plan acknowledges this is a Phase 1 trade-off that gets eliminated in Phase 2.

5. **No language-aware rendering in escaped content**: The `{Banish}` inside escaped braces always resolves to the `banish` term in the current locale during `eval_str`. This is correct for Phase 1 (English-only), but in Phase 2 when translations exist, `eval_str` would need to resolve `{Banish}` against the correct locale. Since `eval_str` uses `rlf::with_locale()`, this should work automatically — **but only if the locale is set before `eval_str` is called.** The current code in `rlf_helper.rs` calls `rlf::with_locale()` which reads the global locale, so this should be fine.

### 5.3 Comparison to alternatives

**Alternative A: Direct rendering (no escaped braces)**
- Phrases would use `{Banish}` (no escaping), producing rendered text directly
- Breaks round-trip tests immediately
- Requires rewriting parser expectations
- Much larger blast radius

**Alternative B: Return Phrase objects from serializer**
- Serializer returns `Phrase` instead of `String`
- This is the plan's Phase 2 — correct long-term target
- Requires changing all callers of the serializer
- Too large for incremental migration

**Alternative C: String concatenation without RLF**
- Keep hardcoded strings but move them to a constants file
- No compile-time validation, no Phrase metadata
- Dead end for localization

**The two-layer approach is the right incremental strategy.** It provides the smallest blast radius while making forward progress toward full RLF adoption.

---

## 6. Specific Issues in the Plan

### 6.1 SHOW-STOPPER: Plan's Step 2 vs Step 3 inconsistency

The plan presents **three different versions** of the cost phrases:
- **Step 1 (lines 95-138)**: Phrases reference RLF terms directly (e.g., `{Banish}`, `{cards($c)}`)
- **Step 2 (lines 146-272)**: Serializer code calling those phrases
- **Step 3 revised (lines 347-384)**: Phrases with escaped braces (e.g., `{{Banish}}`, `{{cards($c)}}`)

**Step 1 and Step 3 are mutually exclusive.** Step 1 would produce rendered text (breaking round-trip tests), while Step 3 produces template text (preserving compatibility). The plan eventually settles on Step 3 (line 345), but the earlier Step 1 code should be clearly marked as superseded to avoid confusion during implementation.

### 6.2 WARNING: Phrase parameter types

In the "Revised RLF Phrases" (line 386), the plan says: "The `$a`, `$c`, `$d`, `$e`, `$m`, `$n` parameters receive integers."

But in the escaped-brace context, these parameters are **never evaluated**. The integer values passed to the generated function are ignored — the output always contains the literal `$a`, `$c`, etc. regardless of what value is passed.

For example:
```rust
abandon_count_allies($a) = "abandon {{count_allies($a)}}";
```
- `strings::abandon_count_allies(3).to_string()` → `"abandon {count_allies($a)}"` (NOT `"abandon {count_allies(3)}"`)
- `strings::abandon_count_allies(999).to_string()` → `"abandon {count_allies($a)}"` (same output!)

The integer parameter is functionally useless in Phase 1. The function signature accepts it (for API compatibility with Phase 2), but the value is discarded. This should be documented clearly.

**Recommendation**: Consider using terms (no parameters) instead of phrases for the Phase 1 escaped-brace patterns:
```rust
abandon_count_allies = "abandon {{count_allies($a)}}";
```
This is simpler and makes the "parameter is unused" issue explicit. However, this changes the generated API (term functions have no parameters), which may complicate the Phase 2 migration. Trade-off worth discussing.

### 6.3 The `$target` parameter pattern is correct

For phrases like:
```rust
abandon_any_number_of($target) = "abandon any number of {$target}";
```
The `$target` is NOT inside escaped braces, so it IS evaluated by RLF. When the serializer calls `strings::abandon_any_number_of("allies")`, the output is `"abandon any number of allies"`. This is correct — the predicate text is passed as a pre-rendered string.

### 6.4 Cost connector phrases

```rust
cost_or_connector = " or ";
cost_and_connector = " and ";
```

These are simple terms with no parameters. They work correctly. In Phase 2, translators can provide language-appropriate connectors (e.g., Spanish " o " / " y ").

---

## 7. Recommendations

### 7.1 Critical

1. **Clarify the plan's Step 1 vs Step 3 confusion.** Remove or clearly mark the superseded Step 1 phrases. The implementation should follow Step 3 (escaped braces) only.

2. **Document the "unused parameter" behavior** for escaped-brace phrases. Make it explicit that parameters like `$a` in `abandon_count_allies($a)` are not evaluated in Phase 1 and exist only for API stability.

3. **Add a round-trip validation test** that verifies serializer output contains only `{known_phrase(...)}` patterns, catching typos in escaped-brace content before they reach `eval_str`.

### 7.2 Important

4. **Consider adding a comment convention** in `strings.rs` to distinguish "Phase 1 template phrases" (with escaped braces) from "Phase 2 rendering phrases" (without). This helps future developers and reviewers understand why some phrases have `{{...}}` patterns.

5. **For the Phase 2 migration path**: The plan should consider that moving from escaped braces to direct RLF references changes the serializer's output format (from template text to rendered text). This means Phase 2 needs to:
   - Update or remove `eval_str` calls in the display layer
   - Update round-trip test expectations
   - Ensure the parser can accept rendered text (or change the round-trip test to skip re-parsing rendered output)

6. **Variable binding names should be centralized.** Currently `"c"`, `"e"`, `"a"` etc. are string literals in both the serializer and the RLF phrase definitions. A constants file or enum would prevent drift.

### 7.3 Nice-to-have

7. **Consider using `Locale::eval_str` directly** for some Phase 1 phrases instead of the `rlf!` macro. Since the escaped-brace phrases are essentially template strings, you could skip the generated function layer and use `eval_str` with inline templates. This would be closer to the existing pattern and avoid the "unused parameter" issue. However, it loses compile-time validation of the phrase definitions.

---

## 8. Summary

| Aspect | Assessment |
|--------|-----------|
| Escaped braces `{{...}}` | **Correct.** Well-tested, works as described. |
| `$param` inside `{{...}}` | **Literal text.** Not evaluated. Plan's architecture accounts for this via two-layer eval. |
| Phrase patterns | **All valid.** No nested calls, no illegal patterns. |
| Chinese support | **Complete.** `@count` with measure word tags. |
| Russian support | **Complete.** Multi-dimensional variants + tag-based selection. |
| Spanish support | **Mostly complete.** Minor gap for personal "a" contraction. |
| Portuguese support | **Complete.** `@de`/`@em` contraction transforms. |
| German support | **Mostly complete.** Moderate gap for adjective declension. |
| Two-layer architecture | **Sound.** Correct incremental strategy. Main risk is variable name coupling. |
| Plan clarity | **Needs improvement.** Step 1 vs Step 3 confusion, unused parameter issue. |

**Overall verdict: The plan is architecturally sound.** The two-layer escaped-brace approach is the right incremental strategy for Phase 1. The main risks are:
1. Variable name coupling between escaped-brace content and VariableBindings (no compile-time check)
2. Plan document has superseded code that could confuse implementers
3. Phase 2 migration will require careful coordination of format changes

No show-stoppers that would block proceeding. The plan should be updated to address the clarity issues before implementation begins.
