# Milestone 0a: Update TOML Files to New Ability Syntax

## Objective

Update `dreamwell.toml` and `test-cards.toml` to use new parser_v2 ability syntax with variables.

## Tasks

1. Update `test-cards.toml` rules-text fields to new syntax
2. Update `dreamwell.toml` if it has ability text
3. Add `variables` field to cards with parameterized abilities
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

## Validation Script

Create validation in tabula_cli:

```rust
pub fn validate_card_syntax(cards_path: &Path) -> Result<()> {
    let cards = toml_loader::load_test_cards(cards_path)?;
    let parser = AbilityParser::new();

    for card in &cards {
        if let Some(rules_text) = &card.rules_text {
            let vars = card.variables.as_deref().unwrap_or("");
            parser.parse(rules_text, vars)
                .map_err(|e| anyhow!("Card '{}': {}", card.name.as_deref().unwrap_or("?"), e))?;
        }
    }
    Ok(())
}
```

Run as: `tabula validate-syntax test-cards.toml`

## Verification

1. All cards in test-cards.toml parse without error
2. All cards in dreamwell.toml parse (if applicable)
3. `cargo test -p parser_v2_tests` passes
4. No regressions in existing tests

## Context Files

1. `client/Assets/StreamingAssets/Tabula/test-cards.toml` - Test cards to update
2. `client/Assets/StreamingAssets/Tabula/dreamwell.toml` - Dreamwell cards
3. `client/Assets/StreamingAssets/Tabula/cards.toml` - Reference for syntax
4. `docs/parser_v2_design.md` - Variable system documentation
5. `tests/parser_v2_tests/tests/spanned_ability_tests.rs` - Parsing examples
