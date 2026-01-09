# Milestone 2: Update TOML Files to New Ability Syntax

## Objective

Update `dreamwell.toml` and `test-cards.toml` to use new parser_v2 ability syntax with variables.

## Tasks

1. Update `test-cards.toml` rules-text fields to new syntax
2. Update `dreamwell.toml` if it has ability text
3. Add `variables` field to cards with numbers in their text or subtypes
4. Validate all updated cards parse with parser_v2
5. Run parser tests to confirm syntax is correct

## Syntax Changes

### Variable Placeholders

Old syntax used literal numbers:
```toml
rules-text = "Draw 2."
```

New syntax uses variable placeholders:
```toml
rules-text = "Draw {cards}."
variables = "cards: 2"
```

### Keyword Formatting

Keywords use braces:
```toml
rules-text = "{Dissolve} an enemy."
rules-text = "{Materialized} Draw {cards}."
rules-text = "When you {materialize} a character, gain {e}."
```

### Common Variable Names

| Variable | Meaning | Example |
|----------|---------|---------|
| `cards` | Card count | `Draw {cards}.` |
| `e` | Energy amount | `Gain {e}.` |
| `s` | Spark amount | `+{s} spark` |
| `points` | Victory points | `Gain {points}.` |
| `discards` | Discard count | `Discard {discards}.` |
| `subtype` | Card subtype | `allied {subtype}` |

## Updating test-cards.toml

For each card in test-cards.toml:
1. Find rules-text field
2. Replace literal numbers with variables
3. Add variables field with bindings
4. Test parsing with parser_v2

Example transformation:
```toml
# Before
[[test-cards]]
name = "Test Draw Two"
rules-text = "Draw 2."

# After
[[test-cards]]
name = "Test Draw Two"
rules-text = "Draw {cards}."
variables = "cards: 2"
```

## Validation Test

Once complete, we should write a unit test which ensure all card rules text parses, matching
the existing `rules_engine/tests/parser_v2_tests/tests/card_toml_validation_tests.rs`

## Verification

1. All cards in test-cards.toml parse without error
2. All cards in dreamwell.toml parse
3. `just parser-test` passes
4. No regressions in existing tests

## Context Files

1. `client/Assets/StreamingAssets/Tabula/test-cards.toml` - Test cards to update
2. `client/Assets/StreamingAssets/Tabula/dreamwell.toml` - Dreamwell cards
3. `client/Assets/StreamingAssets/Tabula/cards.toml` - Reference for syntax
4. 'rules_engine/src/tabula_cli/src/server/listeners/card_rules.ftl' - Fluent selectors for text
5. `docs/parser_v2_design.md` - Variable system documentation
