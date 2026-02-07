# RLF v2 Migration Plan

Migrate Dreamtides from RLF v1 syntax to RLF v2 syntax. The local RLF repo
(patched via `.cargo/config.toml`) has been updated to v2, and Dreamtides no
longer compiles.

## Background

RLF v2 introduces several changes from v1:

1. **$ prefix**: All parameter references use `$`. Bare names always refer to
   terms or phrases. `draw(n)` becomes `draw($n)`, `{n}` becomes `{$n}`,
   `{energy(e)}` becomes `{energy($e)}`, `{card:n}` becomes `{card:$n}`.

2. **Terms vs phrases**: Definitions split into **terms** (no parameters,
   selected with `:`) and **phrases** (with parameters, called with `()`).
   Using `:` on a bare phrase name or `()` on a term name is a compile-time
   error.

3. **`:match` keyword**: Phrases with variant blocks must use `:match($param)`
   to specify which parameter drives branch selection. Supports exact numeric
   keys (`1`, `2`, `3`), CLDR plural categories (`one`, `other`), and tag names
   (`masc`, `fem`). Exactly one branch is marked with `*` as the default.

4. **Escape sequence changes**: `:`, `@`, and `$` are only special inside
   `{}` expressions. In regular text they are literal and need no escaping.
   Only `{` and `}` need escaping in text (as `{{` and `}}`). The `::` escape
   for literal `:` is only needed inside `{}` expressions, not in text.

## RLF v2 Availability

All v2 features are implemented in the local RLF repo at `~/rlf/`:
- `$` prefix enforcement (M1a-c)
- Term/phrase distinction (M2a-b)
- `*` default variant marker (M3)
- `:match` keyword in AST and parsers (M4a)
- Dynamic transform context `()` syntax (M6)
- Literal number and string arguments (M7)
- Escape sequence alignment (M8)

**Dependency**: `:match` evaluation in the evaluator (M4b) must be completed
in the RLF repo before Task 3 below can be verified at runtime. Tasks 1-2 and
4-8 can proceed without M4b.

## Patch Configuration

The RLF dependency is patched in `.cargo/config.toml` (project root):
```toml
[patch."https://github.com/thurn/rlf"]
rlf = { path = "/Users/dthurn/rlf/crates/rlf" }
rlf-macros = { path = "/Users/dthurn/rlf/crates/rlf-macros" }
```

The canonical dependency is in `rules_engine/Cargo.toml`:
```toml
rlf = { git = "https://github.com/thurn/rlf", features = ["global-locale"] }
rlf-macros = { git = "https://github.com/thurn/rlf" }
```

## Scope of Changes

The migration touches six layers:

1. **`rlf!` macro definitions** (`strings.rs`) — 120 definitions across 454
   lines: add `$` to ~20 phrases, fix `::` escapes, convert 13 variant blocks
   to `:match`
2. **Serializer template strings** (7 serializer files) — ~90 template strings
   needing `$` prefix on arguments
3. **TOML `rules-text` fields** (4 TOML files + 4 client copies) — all card
   display templates needing `$` prefix
4. **Parser substitution system** (`parser_substitutions.rs`) — logic changes
   to strip `$` from variable names during parsing
5. **Tests** (parser tests, round-trip tests, RLF eval tests) — expected
   strings needing `$` prefix
6. **Regeneration** — `just tabula-generate` to update `test_card.rs` and
   `parsed_abilities.json`

**Not affected**: Display/rendering call sites (`strings::` function calls,
PhraseId usage, `rlf_helper.rs`, `card_rendering.rs`, `labels.rs`). The
`global-locale` feature means generated functions take no `&locale` argument.
`PhraseId::resolve_global()` and `call_global()` are unchanged.
`build_params()` keys in `rlf_helper.rs` remain bare names (the `$` is
template syntax only, not a HashMap key prefix).

---

## Task 1: Add `$` Prefix to `rlf!` Definitions in `strings.rs`

**File**: `rules_engine/src/strings/src/strings.rs`

This task is the minimum change needed to get the `rlf!` macro to compile.
Add `$` to all parameter declarations and all references to parameters in
template bodies.

**A. Simple phrases (~20 definitions)**

Every phrase that declares parameters needs `$` on the parameter names AND on
all references within template bodies.

Affected phrases and their parameter names:
- `energy(e)`, `points(p)`, `maximum_energy(max)`, `trigger(t)`,
  `keyword(k)`, `kindle(k)`, `foresee(n)`, `reclaim_for_cost(r)`,
  `spark_value(s)`, `count(n)`, `figment(f)`, `figments_plural(f)`,
  `a_figment(f)`, `cards_numeral(n)`,
  `modal_effect_choice_card_name(number)`,
  `character_ability_card_name(character_name)`,
  `pay_energy_prompt_button(e)`, `pay_energy_additional_cost_button(e)`,
  `card_rules_text_energy_paid(e)`,
  `help_text_foresee_n(n)`, `help_text_reclaim_with_cost(e)`.

Example transformation:
```
energy(e) = "<color=#00838F>{e}●</color>";
  →
energy($e) = "<color=#00838F>{$e}●</color>";
```

For phrases with nested phrase calls, the arguments also need `$`:
```
a_figment(f) = "a {figment(f)}";
  →
a_figment($f) = "a {figment($f)}";
```

For `cards_numeral`, the selection parameter also needs `$`:
```
cards_numeral(n) = "{n} {card:n}";
  →
cards_numeral($n) = "{$n} {card:$n}";
```

**B. Variant block phrases (~13 definitions)**

Phrases with variant blocks need `$` on parameter declarations, `$` in body
references, and `*` on the default variant. Keep `one:`/`other:` keys for now
(`:match` conversion happens in Task 3).

Affected phrases: `cards`, `top_n_cards`, `count_allies`,
`count_allied_subtype`, `n_figments`, `text_number`, `this_turn_times`,
`multiply_by`, `copies`, `n_random_characters`, `up_to_n_events`,
`up_to_n_allies`, `it_or_them`.

Example:
```
cards(n) = {
    one: "a card",
    other: "{n} cards"
};
  →
cards($n) = {
    *one: "a card",
    other: "{$n} cards",
};
```

Multi-parameter:
```
count_allied_subtype(n, s) = {
    one: "an allied {subtype(s)}",
    other: "{n} allied {subtype(s):other}"
};
  →
count_allied_subtype($n, $s) = {
    *one: "an allied {subtype($s)}",
    other: "{$n} allied {subtype($s):other}",
};
```

Single-variant (`multiply_by` has only `other:`):
```
multiply_by(n) = {
    other: "Multiply by {n}"
};
  →
multiply_by($n) = {
    *other: "Multiply by {$n}",
};
```

**C. `:from` phrase (1 definition)**

```
subtype(s) = :from(s) "<color=#2E7D32><b>{s}</b></color>";
  →
subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
```

**D. Terms (~60 definitions) — NO changes needed**

Terms have no parameters and are unchanged. Subtype terms (`ancient`,
`warrior`, etc.), keyword terms (`dissolve`, `banish`, etc.), symbol terms,
UI strings — all stay as-is. Term variant blocks (`card`, `ally`, subtypes)
do not need `*` markers (first variant is the default).

**E. Verify term/phrase distinction**

Audit that no template body uses `:` on a phrase name without `()` or `()`
on a term name. Current patterns are already correct:
- `{Dissolve}` — term reference (`dissolve` is a term), OK
- `{cards(n)}` → `{cards($n)}` — phrase call, OK
- `{subtype(s):other}` → `{subtype($s):other}` — selection on phrase result,
  OK

---

## Task 2: Fix Escape Sequences in `strings.rs`

**File**: `rules_engine/src/strings/src/strings.rs`

In v2, `:` is only special inside `{}` expressions. All `::` that appear in
regular text (outside `{}`) must be changed to single `:`. There are 17
instances across the file.

**Trigger definitions (6 instances):**
```
trigger(t) = "▸ <b>{t}::</b>";
  → trigger($t) = "▸ <b>{$t}:</b>";

materialized = "▸ <b>Materialized::</b>";
  → materialized = "▸ <b>Materialized:</b>";

judgment = "▸ <b>Judgment::</b>";
  → judgment = "▸ <b>Judgment:</b>";

dissolved = "▸ <b>Dissolved::</b>";
  → dissolved = "▸ <b>Dissolved:</b>";

materialized_judgment = "▸ <b>Materialized, Judgment::</b>";
  → materialized_judgment = "▸ <b>Materialized, Judgment:</b>";

materialized_dissolved = "▸ <b>Materialized, Dissolved::</b>";
  → materialized_dissolved = "▸ <b>Materialized, Dissolved:</b>";
```

**Choose one header (1 instance):**
```
choose_one = "<b>Choose One::</b>";
  → choose_one = "<b>Choose One:</b>";
```

**Limit warning messages (3 instances):**
```
"Note:: Cards drawn..."  →  "Note: Cards drawn..."
"Character limit exceeded:: A character..."  →  "Character limit exceeded: A character..."
"Character limit exceeded:: A character..."  →  "Character limit exceeded: A character..."
```

**Help text definitions (7 instances):**
```
"{@cap dissolve}:: Send a character..."
  → "{@cap dissolve}: Send a character..."

"{@cap prevent}:: Send a card..."
  → "{@cap prevent}: Send a card..."

"<color=#AA00FF>Foresee</color> 1:: Look at..."
  → "<color=#AA00FF>Foresee</color> 1: Look at..."

"<color=#AA00FF>Foresee</color> {n}:: Look at..."
  → "<color=#AA00FF>Foresee</color> {$n}: Look at..."

"<color=#AA00FF><b>Anchored</b></color>:: Cannot..."
  → "<color=#AA00FF><b>Anchored</b></color>: Cannot..."

"{@cap reclaim}:: You may play..."
  → "{@cap reclaim}: You may play..."

"{@cap reclaim} {energy(e)}:: You may play..."
  → "{@cap reclaim} {energy($e)}: You may play..."
```

Note: The last two `help_text` entries also have `$` prefix changes from
Task 1. The escape fix and `$` fix happen together on these lines.

---

## Task 3: Convert Variant Block Phrases to `:match` and Restore Number Features

**File**: `rules_engine/src/strings/src/strings.rs`

**Dependency**: Requires RLF M4b (`:match` evaluation) to be completed in
the RLF repo.

All 13 phrases with variant blocks convert from implicit selection to explicit
`:match`. This also restores number-to-word conversion features that were
broken during the original Fluent-to-RLF migration.

**A. `text_number` — restore word conversion**

The original Fluent version mapped numbers to English words ("one", "two",
"three", etc.). The RLF v1 adoption lost this because v1 had no exact numeric
matching — only CLDR categories (`one`/`other`). With `:match` and exact
numeric keys, the full word table is restored:

```
text_number($n) = :match($n) {
    1: "one",
    2: "two",
    3: "three",
    4: "four",
    5: "five",
    *other: "{$n}",
};
```

**B. `multiply_by` — restore "double"/"triple"**

The original Fluent version produced "Double", "Triple" for n=2, n=3. The
v1 adoption broke this to always show "Multiply by N". Restore with `:match`:

```
multiply_by($n) = :match($n) {
    2: "Double",
    3: "Triple",
    *other: "Multiply by {$n}",
};
```

**C. `copies` — already uses `text_number`, just add `:match`**

```
copies($n) = :match($n) {
    1: "a copy",
    *other: "{text_number($n)} copies",
};
```

**D. `cards` — standard count pattern**

```
cards($n) = :match($n) {
    1: "a card",
    *other: "{$n} cards",
};
```

**E. All other variant block phrases**

Apply the same pattern — `one:` key becomes `1:` (exact numeric match), the
default branch gets `*other:`:

```
top_n_cards($n) = :match($n) {
    1: "top card",
    *other: "top {$n} cards",
};

count_allies($n) = :match($n) {
    1: "an ally",
    *other: "{$n} allies",
};

count_allied_subtype($n, $s) = :match($n) {
    1: "an allied {subtype($s)}",
    *other: "{$n} allied {subtype($s):other}",
};

n_figments($n, $f) = :match($n) {
    1: "a {figment($f)}",
    *other: "{text_number($n)} {figments_plural($f)}",
};

this_turn_times($n) = :match($n) {
    1: "this turn",
    *other: "this turn {text_number($n)} times",
};

n_random_characters($n) = :match($n) {
    1: "a random character",
    *other: "{text_number($n)} random characters",
};

up_to_n_events($n) = :match($n) {
    1: "an event",
    *other: "up to {$n} events",
};

up_to_n_allies($n) = :match($n) {
    1: "an ally",
    *other: "up to {$n} allies",
};

it_or_them($n) = :match($n) {
    1: "it",
    *other: "them",
};
```

---

## Task 4: Update Serializer Template Strings

**Directory**: `rules_engine/src/parser_v2/src/serializer/`

Seven serializer files produce RLF template strings evaluated at runtime via
`eval_str()`. Every parameter reference in these templates needs the `$`
prefix. Variable binding code (`bindings.insert(...)`) does NOT change —
binding keys remain bare names like `"c"`, `"e"`, `"t"`. The `$` is purely
template syntax — `eval_str` maps `$c` in the template to the `"c"` key in
the params HashMap.

**Files and change counts:**

- `effect_serializer.rs` (~40 templates): draw, discard, gain energy/points,
  foresee, kindle, spark changes, materialize, banish, figments, shuffle,
  random characters, multiply, copy, event return, modal choices.
- `predicate_serializer.rs` (~20 templates): subtype predicates, cost/spark
  constraints, negation patterns, allied/enemy/other modifiers, void
  predicates.
- `static_ability_serializer.rs` (~12 templates): cost modifications, spark
  bonuses, play-from-void, reclaim-for-cost, multiply effects.
- `cost_serializer.rs` (~7 templates): energy costs, discard costs, abandon
  costs, banish costs, maximum energy loss.
- `condition_serializer.rs` (~6 templates): ally count conditions, draw count
  conditions, void count conditions, subtype conditions.
- `trigger_serializer.rs` (~4 templates): play-count triggers, abandon-count
  triggers, draw-count triggers, materialize-count triggers.
- `ability_serializer.rs` (~2 templates): reclaim-for-cost top-level, fast
  prefix.

**Transformation patterns (uniform across all files):**

All parameter references get `$`:
- `{n}` → `{$n}` (bare variable interpolation)
- `{s}` → `{$s}` (bare variable interpolation)
- `{cards(c)}` → `{cards($c)}` (phrase call argument)
- `{energy(e1)}` → `{energy($e1)}` (numbered variant argument)
- `{@a subtype(t)}` → `{@a subtype($t)}` (transform + phrase call argument)
- `{@plural subtype(t)}` → `{@plural subtype($t)}` (transform + phrase call)
- `{count_allied_subtype(a, t)}` → `{count_allied_subtype($a, $t)}`
  (multi-arg)
- `{subtype(t):other}` → `{subtype($t):other}` (phrase call + selector —
  the selector `other` is a static key, no `$`)
- `{@cap @a subtype(t)}` → `{@cap @a subtype($t)}` (chained transforms)

**What does NOT change:**
- Bare term references: `{banish}`, `{materialize}`, `{dissolve}`,
  `{energy_symbol}`, `{Banish}`, `{Judgment}` — no parameters, no `$`
- `format!()` double-braces `{{cards(c)}}` — the `{{`/`}}` is Rust's brace
  escaping for `format!()`, unrelated to RLF. Inside these, the argument
  still gets `$`: `{{cards(c)}}` → `{{cards($c)}}`
- Binding keys: `bindings.insert("c".to_string(), ...)` — unchanged

**Important distinction**: Many serializer templates use `format!()` with `{}`
for Rust string interpolation (inserting predicate text, etc.) and `{{...}}`
for literal braces that become RLF expressions. Only the RLF argument names
inside these get `$`, not the Rust `{}` placeholders. Example:

```rust
// Before:
format!("draw {{cards(c)}} for each {}.", serialize_for_count(...))
// After:
format!("draw {{cards($c)}} for each {}.", serialize_for_count(...))
```

The `{}` (Rust format placeholder) stays. The `{{cards($c)}}` (escaped braces
producing literal `{cards($c)}` in the output) gets `$` on the argument.

---

## Task 5: Update TOML `rules-text` Fields

**Files** (4 source + 4 client copies):
- `rules_engine/tabula/cards.toml` (very large — use search-and-replace)
- `rules_engine/tabula/test-cards.toml`
- `rules_engine/tabula/dreamwell.toml`
- `rules_engine/tabula/test-dreamwell.toml`
- `client/Assets/StreamingAssets/Tabula/cards.toml`
- `client/Assets/StreamingAssets/Tabula/test-cards.toml`
- `client/Assets/StreamingAssets/Tabula/dreamwell.toml`
- `client/Assets/StreamingAssets/Tabula/test-dreamwell.toml`

Every parameter reference in `rules-text` fields needs the `$` prefix.

**All phrase call patterns found in cards.toml** (from grep analysis):

Lowercase single-arg:
`{cards(c)}`, `{cards(d)}`, `{energy(e)}`, `{energy(e1)}`, `{energy(e2)}`,
`{points(p)}`, `{foresee(f)}`, `{kindle(k)}`, `{subtype(t)}`,
`{top_n_cards(v)}`, `{cards_numeral(c)}`, `{count(n)}`, `{count_allies(a)}`,
`{multiply_by(n)}`, `{maximum_energy(m)}`, `{reclaim_for_cost(r)}`,
`{a_figment(g)}`, `{this_turn_times(n)}`, `{it_or_them(n)}`,
`{up_to_n_allies(n)}`, `{up_to_n_events(n)}`, `{n_random_characters(n)}`

Lowercase multi-arg:
`{count_allied_subtype(a, t)}`, `{n_figments(n, g)}`

Capitalized (auto-`@cap`):
`{Foresee(f)}`, `{Kindle(k)}`, `{Reclaim_For_Cost(r)}`

Transform + phrase:
`{@a subtype(t)}`, `{@plural subtype(t)}`, `{@cap @a subtype(t)}`

Bare variable:
`{s}` (spark modifier — used in ~20 cards)

**What does NOT change:**
- Bare term references: `{Dissolve}`, `{Prevent}`, `{Materialized}`,
  `{Banish}`, `{Fast}`, `{Discover}`, `{Reclaim}`, `{Judgment}`,
  `{Dissolved}`, `{Materialize}`, `{Materialized_Judgment}`,
  `{Materialized_Dissolved}`, `{energy_symbol}`, `{choose_one}`, `{bullet}`,
  `{fast}`, `{banish}`, `{materialize}`, `{dissolve}`, `{reclaim}`,
  `{discover}`, `{prevent}`, `{materialized}`
- The `variables` field values: `"c: 2"` stays `"c: 2"` (bare names)

**Approach**: Use regex search-and-replace. Inside `{...}`, any bare lowercase
identifier that appears as a function argument (inside `()`) or as a
standalone single-letter variable reference (`{s}`) gets `$` prefixed. Static
selectors like `:other`, `:one` do not.

Recommended sed patterns (run on all 4 source TOML files):
1. Arguments inside `()`: Match `([a-z][a-z0-9_]*)` inside phrase calls and
   prefix with `$`. Handle multi-arg patterns like `(a, t)` → `($a, $t)`.
2. Bare variable `{s}` → `{$s}`: Only `{s}` appears as a bare variable in
   TOML data.

After updating source files, copy to client:
```sh
cp rules_engine/tabula/cards.toml client/Assets/StreamingAssets/Tabula/
cp rules_engine/tabula/test-cards.toml client/Assets/StreamingAssets/Tabula/
cp rules_engine/tabula/dreamwell.toml client/Assets/StreamingAssets/Tabula/
cp rules_engine/tabula/test-dreamwell.toml client/Assets/StreamingAssets/Tabula/
```

---

## Task 6: Update Parser Substitution System

**File**: `rules_engine/src/parser_v2/src/variables/parser_substitutions.rs`

The Dreamtides ability parser has its own tokenizer and resolution system
separate from RLF. It parses `rules-text` to extract semantic game information
(what ability a card has). After Task 5 updates TOML templates to use `$`,
this parser must understand the `$` prefix.

**How the parser currently works:**

1. The lexer tokenizes `{...}` blocks as `Directive(String)` tokens. For
   `{cards(c)}`, it produces `Directive("cards(c)")`. For `{s}`, it produces
   `Directive("s")`. The lexer captures everything inside `{}` as a raw
   string.

2. `resolve_directive()` is the entry point. It checks if a directive matches
   the PHRASES, BARE_PHRASES, SUBTYPE_PHRASES, or FIGMENT_PHRASES tables,
   then falls through to `resolve_rlf_syntax()` for function call patterns.

3. `resolve_rlf_syntax()` uses regex-like parsing to extract phrase name and
   arguments from patterns like `energy(e)`, `@a subtype(t)`,
   `subtype(t):other`. It looks up argument names in variable bindings.

**Changes needed:**

**A. Strip `$` from arguments in `resolve_rlf_syntax()`**

When parsing `energy($e)`, the function extracts argument `$e`. Before
looking up in bindings, strip the `$` prefix:

```rust
// When extracting arg from "energy($e)":
let arg = "$e";
let binding_key = arg.strip_prefix('$').unwrap_or(arg);
// Look up binding_key "e" in bindings
```

Apply this to all argument extraction paths in the function, including:
- Single-arg patterns: `energy($e)` → extract `$e` → strip to `e`
- Multi-arg patterns: `count_allied_subtype($a, $t)` → extract `$a`, `$t` →
  strip to `a`, `t`
- Numbered variants: `energy($e1)` → extract `$e1` → strip to `e1`

**B. Handle bare `$`-prefixed variable references**

With v2, bare `{$s}` arrives as `Directive("$s")`. The resolution path must
recognize `$`-prefixed names as variable references:

```rust
// In resolve_directive():
if let Some(bare_name) = directive.strip_prefix('$') {
    // Look up bare_name in bindings
    // Return appropriate ResolvedToken based on binding type
}
```

**C. Handle `$` in transform patterns**

Transform patterns like `{@a subtype($t)}` and `{@cap @a subtype($t)}`
arrive as directives containing `$`. The `@`-prefix stripping and argument
extraction must handle `$` in the argument position.

**No changes needed to:**
- The PHRASES/BARE_PHRASES/SUBTYPE_PHRASES/FIGMENT_PHRASES tables (these map
  phrase names, not variable names — phrase names don't have `$`)
- The `ResolvedToken` enum (semantic types are unchanged)
- Variable binding parsing (`parser_bindings.rs`) — keys remain bare names
- The lexer — it already captures everything inside `{}` as a directive
  string, `$` is just another character in the raw text

---

## Task 7: Update Test Expected Strings

Several test files contain expected RLF template strings that must match the
updated serializer output and TOML templates.

**Round-trip test files** (in `rules_engine/tests/parser_v2_tests/tests/
round_trip_tests/`):
- `activated_ability_round_trip_tests.rs`
- `cards_toml_round_trip_tests.rs`
- `dreamwell_toml_round_trip_tests.rs`
- `event_effect_round_trip_tests.rs`
- `judgment_ability_round_trip_tests.rs`
- `materialized_ability_round_trip_tests.rs`
- `static_ability_round_trip_tests.rs`
- `triggered_ability_round_trip_tests.rs`

These tests call `assert_round_trip(rules_text, variables)` with expected
template strings. Every parameter reference in these strings needs `$`.

Example:
```rust
// Before:
assert_round_trip("{energy(e)}, Discard {cards(d)}: {Kindle(k)}.", "e: 1\nd: 1\nk: 2");
// After:
assert_round_trip("{energy($e)}, Discard {cards($d)}: {Kindle($k)}.", "e: 1\nd: 1\nk: 2");
```

Note: The `variables` string values (`"e: 1\nd: 1\nk: 2"`) do NOT change —
binding keys remain bare names.

**RLF evaluation test files:**
- `rules_engine/tests/tv_tests/src/derived_tests/rules_preview_tests/
  rlf_tests.rs` — tests that pass template strings through `make_inputs()`
  for RLF evaluation. Templates need `$` on parameters.
- `rules_engine/tests/tv_tests/src/derived_tests/cards_rlf_eval_tests.rs` —
  tests that verify card text rendering through the full RLF pipeline

**Parser variable test files:**
- `rules_engine/tests/parser_v2_tests/src/parser_variable_tests.rs` — tests
  that verify parsed variable resolution against expected template strings.
  Expected strings need `$`.

---

## Task 8: Regenerate and Verify

After all code changes:

1. Run `just fmt` to apply formatting rules.

2. Run `just tabula-generate` to regenerate `test_card.rs` and
   `parsed_abilities.json` from updated TOML data. This is required because
   the TOML `rules-text` fields changed in Task 5.

   The command: `just tabula-generate`
   (Defined in justfile as: `cargo run --manifest-path rules_engine/Cargo.toml
   -p tabula_cli -- generate`)

   **Important**: `just tabula-generate` itself compiles the project, so it
   will fail until Tasks 1-6 are complete. It must be run after all code
   changes.

3. Run `just check` to verify compilation succeeds.

4. Run `just clippy` to check for lint warnings.

5. Run `just review` to run the full validation suite (clippy + style +
   tests).

6. If round-trip tests fail, diff actual vs expected output to identify any
   missed `$` prefix updates in TOML, serializer, or test files. The most
   common failure mode is a template string that still has a bare parameter
   reference without `$`.

7. If RLF eval tests fail, check that:
   - The `:match` evaluator (M4b) is implemented in the RLF repo
   - The variant block phrases in strings.rs use correct `:match` syntax
   - The `text_number` phrase has the expected word mappings

---

## Implementation Order

```
Task 1 (strings.rs: $ prefix)     ─┐
Task 2 (strings.rs: :: cleanup)    ─┤
Task 3 (strings.rs: :match)        ─┤ Can be done together as one commit
                                    │
Task 4 (serializers: $ prefix)     ─┤ Independent of Tasks 1-3
Task 5 (TOML: $ prefix + copy)    ─┤ Independent of Tasks 1-4
Task 6 (parser: $ handling)        ─┤ Independent of Tasks 1-5
Task 7 (tests: $ prefix)          ─┤ Depends on Tasks 4+5 (must match)
                                    │
Task 8 (regenerate + verify) ◄─────┘ Depends on all above
```

Tasks 1-3 touch only `strings.rs` and can be done as a single change.
Tasks 4, 5, 6 touch disjoint files and can be parallelized.
Task 7 must match the output of Tasks 4+5.
Task 8 runs last to verify everything.

It is fine for the project to not compile between individual tasks.
The critical ordering is:
- Tasks 4 and 7 must agree (serializer output matches test expectations)
- Tasks 5 and 7 must agree (TOML data matches round-trip test expectations)
- Task 8 runs after everything else

---

## Risk Assessment

**Low risk**: The `$` prefix is a mechanical, uniform transformation. Every
parameter reference gets `$`, every binding key stays bare. No semantic
changes to game logic, no API changes to call sites.

**Low risk**: The `::` → `:` escape fix is a simple text replacement in
17 known locations in `strings.rs`. No TOML files have `::` issues.

**Medium risk**: The parser substitution system (Task 6) requires logic
changes rather than mechanical text replacement. The `resolve_rlf_syntax()`
and `resolve_directive()` functions must correctly strip `$` in all code
paths. Testing coverage through round-trip tests mitigates this.

**Medium risk**: The `:match` conversion (Task 3) restores features that
have been broken since the Fluent migration. The exact numeric key values
for `text_number` and `multiply_by` must match what the serializers and
TOML data actually produce. Verify with round-trip tests.

**Low risk**: The `global-locale` feature means all ~100 `strings::` call
sites across 9 rendering files need NO changes. PhraseId usage in `labels.rs`
also remains unchanged.

**Dependency risk**: Task 3 (`:match` conversion) requires RLF M4b to be
completed. If M4b is not ready, Tasks 1-2 and 4-8 can still proceed — the
variant block phrases will keep the v1 implicit-selector syntax with `$` and
`*` markers, and convert to `:match` later when M4b lands.
