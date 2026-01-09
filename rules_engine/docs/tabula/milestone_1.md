# Milestone 0: Strings FTL Conversion

## Objective

Convert `strings.toml` to `strings.ftl` format for Fluent-based string loading.

## Tasks

1. Parse `strings.toml` to extract all string entries
2. Convert TOML key structure to FTL message IDs (kebab-case)
3. Generate `strings.ftl` with all messages
4. Validate FTL syntax is correct
5. Place `strings.ftl` alongside other Tabula files

## Conversion Rules

### Key Naming

TOML nested keys become FTL kebab-case IDs:

```toml
# strings.toml
[keys]
main_menu_play = "Play"
main_menu_quit = "Quit"

[card_types]
character = "Character"
event = "Event"
```

Becomes:

```ftl
# strings.ftl
main-menu-play = Play
main-menu-quit = Quit
card-types-character = Character
card-types-event = Event
```

### Special Characters

FTL requires escaping for certain characters:
- Curly braces: `{` becomes `{"{"}`
- Placeholders: `{$variable}` for Fluent variables

### Comments

Preserve any comments from TOML as FTL comments:

```ftl
# UI Strings
main-menu-play = Play
```

## Script Implementation

Create a simple conversion script in `tabula_cli`:

```rust
// src/tabula_cli/src/commands/convert_strings.rs
pub fn convert_strings_toml_to_ftl(toml_path: &Path, ftl_path: &Path) -> Result<()> {
    let content = fs::read_to_string(toml_path)?;
    let parsed: toml::Value = toml::from_str(&content)?;

    let mut ftl_lines = Vec::new();
    extract_messages(&parsed, "", &mut ftl_lines);

    fs::write(ftl_path, ftl_lines.join("\n"))?;
    Ok(())
}

fn extract_messages(value: &toml::Value, prefix: &str, lines: &mut Vec<String>) {
    match value {
        toml::Value::Table(table) => {
            for (key, val) in table {
                let new_prefix = if prefix.is_empty() {
                    key.replace('_', "-")
                } else {
                    format!("{}-{}", prefix, key.replace('_', "-"))
                };
                extract_messages(val, &new_prefix, lines);
            }
        }
        toml::Value::String(s) => {
            lines.push(format!("{} = {}", prefix, s));
        }
        _ => {}
    }
}
```

## Verification

1. Run conversion script
2. Parse generated FTL with fluent crate to verify validity
3. Ensure all string IDs are present
4. Check FTL renders correctly for sample strings

## Output Location

Place `strings.ftl` in:
- `client/Assets/StreamingAssets/Tabula/strings.ftl`

Keep `strings.toml` temporarily until migration is complete.

## Testing

```rust
#[test]
fn test_ftl_parses() {
    let ftl = fs::read_to_string("strings.ftl").unwrap();
    let resource = FluentResource::try_new(ftl);
    assert!(resource.is_ok());
}
```

## Context Files

1. `client/Assets/StreamingAssets/Tabula/strings.toml` - Source file
2. `src/tabula_cli/src/commands/build_toml.rs` - Similar file processing
