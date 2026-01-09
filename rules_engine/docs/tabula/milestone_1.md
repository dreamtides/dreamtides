# Milestone 1: Strings FTL Conversion

## Objective

Convert `strings.toml` to `strings.ftl` format for Fluent-based string loading.

## Tasks

1. Read `strings.toml` to extract all string entries
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

## Output Location

Place `strings.ftl` in:
- `rules_engine/tabula/strings.ftl`

Keep `strings.toml` temporarily until migration is complete.

## Context Files

1. `client/Assets/StreamingAssets/Tabula/strings.toml` - Source file
2. `src/tabula_cli/src/commands/build_toml.rs` - Similar file processing
