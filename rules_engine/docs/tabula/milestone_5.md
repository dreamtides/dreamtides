# Milestone 5: Parser V2 Integration

## Objective

Implement `ability_parser.rs` to parse card abilities using the parser_v2 system.

## Tasks

1. Create `AbilityParser` struct with cached parser
2. Implement ability parsing from rules text and variables
3. Add error handling that includes card name/ID in messages
4. Write tests parsing sample ability text

**Note:** UI rendering will use serializers from `parser_v2/src/serializer` on-demand, not stored display data.

## AbilityParser Struct

```rust
use chumsky::Parser;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::ability_parser;
use parser_v2::variables::{parser_bindings::VariableBindings, parser_substitutions};
use parser_v2::builder::parser_builder;

pub struct AbilityParser {
    // Parser is created once and reused
}

impl AbilityParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(
        &self,
        rules_text: &str,
        variables: &str,
    ) -> Result<Vec<Ability>, TabulaError> {
        // 1. Parse variable bindings
        let bindings = VariableBindings::parse(variables)?;

        // 2. Lex the rules text
        let lex_result = lexer_tokenize::lex(rules_text)?;

        // 3. Resolve variables
        let resolved = parser_substitutions::resolve_variables(
            &lex_result.tokens,
            &bindings
        )?;

        // 4. Parse abilities
        let parser = ability_parser::ability_parser();
        let parsed = parser.parse(&resolved).into_result()?;

        // 5. Build Ability structs
        let abilities = parser_builder::build_abilities(&parsed)?;

        Ok(abilities)
    }
}
```

## Performance Considerations

Create parser once per Tabula load, not per card:

```rust
impl Tabula {
    fn parse_all_cards(&self, cards: Vec<CardDefinitionRaw>) -> Vec<CardDefinition> {
        let parser = AbilityParser::new();
        cards.into_iter()
            .filter_map(|raw| self.build_card(&parser, raw).ok())
            .collect()
    }
}
```

## Testing Strategy

Reference existing parser tests for expected inputs/outputs:

```rust
#[test]
fn test_parse_draw_cards() {
    let parser = AbilityParser::new();
    let result = parser.parse("Draw {cards}.", "cards: 2").unwrap();
    assert_eq!(result.abilities.len(), 1);
}

#[test]
fn test_parse_triggered_ability() {
    let parser = AbilityParser::new();
    let result = parser.parse("{Judgment} Draw {cards}.", "cards: 2").unwrap();
    assert_eq!(result.abilities.len(), 1);
    assert!(matches!(&result.abilities[0], Ability::Triggered(_)));
}
```

## UI Rendering Note

Display text is NOT stored in CardDefinition. Instead, render on-demand using serializers:

```rust
use parser_v2::serializer::ability_serializer;

// When displaying a card in UI
for ability in &card.abilities {
    let serialized = ability_serializer::serialize_ability(ability);
    display_text(&serialized.text);
    // Use serialized.variables for {placeholder} substitution
}
```

## Verification

- `cargo test -p tabula_data_v2` passes
- Sample cards from cards.toml parse successfully
- Parsed abilities match expected structure

## Context Files

1. `benchmarks/parser_v2/src/benchmark_utils.rs` - Parser usage pattern
2. `src/parser_v2/src/builder/parser_builder.rs` - Ability building
3. `src/parser_v2/src/serializer/ability_serializer.rs` - UI text rendering
4. `src/parser_v2/src/parser/ability_parser.rs` - Parser entry point
