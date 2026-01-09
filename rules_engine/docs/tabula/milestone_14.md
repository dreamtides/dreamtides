# Milestone 13: State Provider Integration

## Objective

Update `state_provider` to use `tabula_data_v2` for loading card data.

## Tasks

1. Update state_provider Cargo.toml to depend on tabula_data_v2
2. Replace Tabula loading code with V2 API
3. Handle Android-specific file loading
4. Update streaming assets path handling
5. Test loading on all platforms (desktop, Android)

## State Provider Updates

Replace V1 loading:

```rust
// Before (V1)
fn load_tabula(streaming_assets_path: &str) -> Result<Tabula> {
    let json = load_tabula_raw(streaming_assets_path)?;
    let raw: TabulaRaw = serde_json::from_str(&json)?;
    Tabula::build(raw)
}

// After (V2)
fn load_tabula(streaming_assets_path: &str) -> Result<Tabula> {
    let base_path = Path::new(streaming_assets_path).join("Tabula");
    let source = TabulaSource::Production;
    Tabula::load(source, &base_path)
}
```

## Android File Loading

Create Android-aware loading in tabula_data_v2:

```rust
#[cfg(target_os = "android")]
pub fn load_toml_android(asset_path: &str) -> Result<String, TabulaError> {
    core_data::android::android_asset_read(asset_path)
        .map_err(|e| TabulaError::IoError {
            path: PathBuf::from(asset_path),
            source: e,
        })
}

#[cfg(not(target_os = "android"))]
pub fn load_toml_file(path: &Path) -> Result<String, TabulaError> {
    std::fs::read_to_string(path)
        .map_err(|e| TabulaError::IoError {
            path: path.to_path_buf(),
            source: e,
        })
}
```

## Platform-Aware Tabula Loading

```rust
impl Tabula {
    #[cfg(target_os = "android")]
    pub fn load_from_assets(base_path: &str, source: TabulaSource) -> Result<Self> {
        let cards_toml = load_toml_android(&format!("{}/{}", base_path, source.cards_filename()))?;
        // Parse from strings instead of files
        Self::load_from_strings(cards_toml, /* ... */)
    }

    #[cfg(not(target_os = "android"))]
    pub fn load_from_path(base_path: &Path, source: TabulaSource) -> Result<Self> {
        // Normal file-based loading
        Self::load(source, base_path)
    }
}
```

## Integration Points

Update these files in state_provider:
- `src/state_provider/src/state_provider.rs` - Main loading code
- Remove references to `tabula.json`
- Update `load_tabula_raw()` and `load_tabula_raw_android()`

## Testing

Test on desktop:
```rust
#[test]
fn test_state_provider_loads_tabula() {
    let state = StateProvider::new(assets_path()).unwrap();
    assert!(state.tabula().cards.len() > 0);
}
```

For Android testing, rely on integration tests on device.

## Verification

- Desktop builds and tests pass
- Android build succeeds
- `just check` passes
- No references to tabula.json in state_provider

## Context Files

1. `src/state_provider/src/state_provider.rs` - Current loading code
2. `src/core_data/src/android.rs` - Android asset loading
3. `docs/tabula/tabula_v2_design_document.md` - Android loading notes
