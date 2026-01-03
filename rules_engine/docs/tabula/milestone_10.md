# Milestone 10: String ID Code Generation

## Objective

Generate `StringId` enum from `strings.ftl` file.

## Tasks

1. Create `strings.ftl` from existing `strings.toml` (manual conversion)
2. Add FTL parsing to code generator to extract message IDs
3. Generate `string_id.rs` with StringId enum
4. Include method to get string key for each variant
5. Write tests for generation

## FTL File Creation

Convert `strings.toml` to `strings.ftl` format. Example:

```toml
# strings.toml
[keys]
main_menu_play = "Play"
main_menu_quit = "Quit"
```

Becomes:

```ftl
# strings.ftl
main-menu-play = Play
main-menu-quit = Quit
```

Note: FTL uses kebab-case by convention.

## StringId Enum Generation

Parse FTL to extract message IDs (lines matching `^[a-z-]+ =`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StringId {
    MainMenuPlay,
    MainMenuQuit,
    // ... all message IDs
}

impl StringId {
    pub fn key(&self) -> &'static str {
        match self {
            Self::MainMenuPlay => "main-menu-play",
            Self::MainMenuQuit => "main-menu-quit",
            // ...
        }
    }
}
```

## FTL Parsing

Simple line-based parsing (no full Fluent parser needed):

```rust
fn extract_message_ids(ftl_content: &str) -> Vec<String> {
    ftl_content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                return None;
            }
            if let Some(eq_pos) = trimmed.find('=') {
                let id = trimmed[..eq_pos].trim();
                if id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                    return Some(id.to_string());
                }
            }
            None
        })
        .collect()
}
```

## Name Conversion

Convert kebab-case FTL IDs to PascalCase enum variants:

```rust
fn to_pascal_case(kebab: &str) -> String {
    kebab
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}
```

## Testing

```rust
#[test]
fn test_extract_message_ids() {
    let ftl = r#"
# Comment
main-menu-play = Play
main-menu-quit = Quit
"#;
    let ids = extract_message_ids(ftl);
    assert_eq!(ids, vec!["main-menu-play", "main-menu-quit"]);
}
```

## Verification

- strings.ftl created and valid
- `tabula generate` produces string_id.rs
- StringId enum compiles and matches FTL content

## Context Files

1. `src/tabula_ids/src/string_id.rs` - V1 StringId enum
2. `client/Assets/StreamingAssets/Tabula/strings.toml` - Source strings
3. `src/old_tabula_cli/src/tabula_codegen.rs` - V1 generation approach
