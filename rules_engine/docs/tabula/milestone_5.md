# Milestone 4: Fluent String Loading

## Objective

Implement `fluent_loader.rs` to load and format strings from FTL files.

## Tasks

1. Create `FluentStrings` struct wrapping `FluentBundle`
2. Implement `load()` function to parse FTL content
3. Implement `format()` function for string lookup with arguments
4. Add support for loading `card_rules.ftl` for ability text formatting
5. Write tests for string loading and formatting

## FluentStrings Struct

```rust
use fluent::{FluentBundle, FluentResource, FluentArgs, FluentValue};
use std::sync::Arc;

pub struct FluentStrings {
    bundle: FluentBundle<Arc<FluentResource>>,
}

impl FluentStrings {
    pub fn from_ftl(ftl_content: &str) -> Result<Self, TabulaError> {
        let resource = FluentResource::try_new(ftl_content.to_string())
            .map_err(|(_, errors)| /* format errors */)?;
        let mut bundle = FluentBundle::default();
        bundle.set_use_isolating(false);
        bundle.add_resource(Arc::new(resource))?;
        Ok(Self { bundle })
    }

    pub fn format(&self, id: &str, args: &FluentArgs) -> Result<String, TabulaError> {
        let msg = self.bundle.get_message(id)
            .ok_or_else(|| /* missing message error */)?;
        let pattern = msg.value()
            .ok_or_else(|| /* no value error */)?;
        let mut errors = Vec::new();
        let result = self.bundle.format_pattern(pattern, Some(args), &mut errors);
        if !errors.is_empty() { /* handle errors */ }
        Ok(result.into_owned())
    }
}
```

## Rules Text Formatting

For card rules text, use the existing `card_rules.ftl`:

```rust
pub fn format_rules_text(
    rules_ftl: &FluentStrings,
    rules_text: &str,
    variables: &str,
) -> Result<String, TabulaError> {
    // 1. Parse variables into FluentArgs
    // 2. Format rules_text as a Fluent expression
    // Reference: src/tabula_cli/src/server/listeners/fluent_rules_text.rs
}
```

## Testing Strategy

Use inline FTL content for unit tests:

```rust
#[test]
fn test_format_keyword() {
    let ftl = r#"
Dissolve = <color=#AA00FF>Dissolve</color>
"#;
    let strings = FluentStrings::from_ftl(ftl).unwrap();
    let result = strings.format("Dissolve", &FluentArgs::new()).unwrap();
    assert!(result.contains("Dissolve"));
}
```

## Verification

- `cargo test -p tabula_data_v2` passes
- Can format sample rules text with variables
- Error messages identify the problematic message ID

## Context Files

1. `src/tabula_cli/src/server/listeners/card_rules.ftl` - Rules text FTL
2. `src/tabula_cli/src/server/listeners/fluent_rules_text.rs` - Formatting logic
3. `client/Assets/StreamingAssets/Tabula/strings.toml` - UI strings (to convert)
4. `src/tabula_data/src/localized_strings.rs` - V1 string handling (for deletion)
