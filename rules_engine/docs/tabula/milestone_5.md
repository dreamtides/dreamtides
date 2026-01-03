# Milestone 5: Parser V2 Integration

## Objective

Implement `ability_parser.rs` to parse card abilities using the cached parser instance.

## Tasks

1. Create `AbilityParser` struct with cached parser
2. Implement ability parsing from rules text and variables
3. Implement SpannedAbility generation for display
4. Add error handling that includes card name/ID in messages
5. Write tests parsing sample ability text

## AbilityParser Struct

```rust
use chumsky::Parser;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::ability_parser;
use parser_v2::variables::{parser_bindings::VariableBindings, parser_substitutions};
use parser_v2::builder::{parser_builder, parser_spans::SpannedAbility};

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
    ) -> Result<ParsedAbilities, TabulaError> {
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

        // 5. Build Ability and SpannedAbility
        let abilities = parser_builder::build_abilities(&parsed)?;
        let spanned = /* build spanned from parsed and lex_result */;

        Ok(ParsedAbilities { abilities, spanned })
    }
}

pub struct ParsedAbilities {
    pub abilities: Vec<Ability>,
    pub spanned: Vec<SpannedAbility>,
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

## Verification

- `cargo test -p tabula_data_v2` passes
- Sample cards from cards.toml parse successfully
- SpannedAbility output matches expected structure

## Context Files

1. `benchmarks/parser_v2/src/benchmark_utils.rs` - Parser usage pattern
2. `tests/parser_v2_tests/tests/spanned_ability_tests.rs` - Expected outputs
3. `src/parser_v2/src/builder/parser_spans.rs` - SpannedAbility types
4. `src/parser_v2/src/parser/ability_parser.rs` - Parser entry point
