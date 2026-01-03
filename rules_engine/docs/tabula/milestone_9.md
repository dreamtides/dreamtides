# Milestone 9: Code Generation Updates

## Objective

Update `tabula_cli` to generate code for `tabula_generated` (renamed from `tabula_ids`).

## Tasks

1. Rename `tabula_ids` crate to `tabula_generated` in Cargo.toml and workspace
2. Add code generation command to tabula_cli
3. Generate `CardEffectRowType` enum from effect-types.toml
4. Generate `CardEffectRowTrigger` enum from trigger-types.toml
5. Generate `CardEffectRowObjectPredicate` enum from predicate-types.toml
6. Update CardEffectRow to use generated enums instead of strings

## Crate Rename

Update `src/tabula_ids/Cargo.toml`:
```toml
[package]
name = "tabula_generated"
```

Update all crates that depend on `tabula_ids` to use `tabula_generated`.

## Code Generation Command

Add to tabula_cli:

```bash
tabula generate [OUTPUT_DIR]
```

Default output: `src/tabula_generated/src/`

## Effect Types Generation

Read `effect-types.toml`:
```toml
effect_types = [
    "FireProjectile",
    "DissolveTargets",
    ...
]
```

Generate `effect_types.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardEffectRowType {
    FireProjectile,
    DissolveTargets,
    // ...
}

impl CardEffectRowType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "FireProjectile" => Some(Self::FireProjectile),
            // ...
            _ => None,
        }
    }
}
```

## Generation Implementation

```rust
// src/tabula_cli/src/commands/generate.rs
pub fn generate_enums(output_dir: &Path) -> Result<()> {
    generate_effect_types(output_dir)?;
    generate_trigger_types(output_dir)?;
    generate_predicate_types(output_dir)?;
    Ok(())
}

fn generate_effect_types(output_dir: &Path) -> Result<()> {
    let toml_path = tabula_dir().join("effect-types.toml");
    let content = fs::read_to_string(&toml_path)?;
    let data: EffectTypesFile = toml::from_str(&content)?;

    let code = generate_enum_code("CardEffectRowType", &data.effect_types);
    fs::write(output_dir.join("effect_types.rs"), code)?;
    Ok(())
}
```

## Testing

- Verify generated code compiles
- Verify enum variants match TOML entries exactly
- Run `just check` after generation

## Verification

- `tabula generate` produces valid Rust code
- `just check` passes after generation
- CardEffectRow uses generated enums

## Context Files

1. `src/old_tabula_cli/src/tabula_codegen.rs` - V1 code generation
2. `src/tabula_ids/src/lib.rs` - Current structure
3. `client/Assets/StreamingAssets/Tabula/effect-types.toml` - Effect types
4. `client/Assets/StreamingAssets/Tabula/trigger-types.toml` - Trigger types
