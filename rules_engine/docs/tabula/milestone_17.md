# Milestone 17: Cleanup & Deletion

## Objective

Remove V1 tabula system and finalize V2 as the only implementation.

## Tasks

1. Delete `tabula_data` crate
2. Delete `old_tabula_cli` crate
3. Delete `tabula.json` file
4. Rename `tabula_data_v2` to `tabula_data`
5. Update all workspace references
6. Final verification with `just review`

## Deletion Order

Delete in this order to avoid broken dependencies:

### Step 1: Verify No V1 References

```bash
# Ensure no code references old crate
grep -r "tabula_data::" src/ tests/ --include="*.rs"
# Should only show tabula_data_v2 references

# Ensure no Cargo.toml references old crate
grep -r 'tabula_data = ' src/ --include="Cargo.toml"
# Should show tabula_data_v2 only
```

### Step 2: Delete old_tabula_cli

```bash
rm -rf src/old_tabula_cli
```

Update workspace Cargo.toml to remove the member.

### Step 3: Delete tabula_data

```bash
rm -rf src/tabula_data
```

Update workspace Cargo.toml to remove the member.

### Step 4: Delete tabula.json

```bash
rm rules_engine/tabula.json
```

Update any scripts or .gitignore entries referencing it.

### Step 5: Rename tabula_data_v2

```bash
mv src/tabula_data_v2 src/tabula_data
```

Update `src/tabula_data/Cargo.toml`:
```toml
[package]
name = "tabula_data"
```

### Step 6: Update All References

Find and replace in all Cargo.toml files:
- `tabula_data_v2` -> `tabula_data`

Find and replace in all .rs files:
- `use tabula_data_v2` -> `use tabula_data`
- `tabula_data_v2::` -> `tabula_data::`

### Step 7: Rename tabula_ids to tabula_generated

If not already done in milestone 9:
```bash
mv src/tabula_ids src/tabula_generated
```

Update package name and all references.

## Final Verification

Run full validation:

```bash
just fmt
just check
just clippy
just review
cargo test --workspace
```

## Cleanup Checklist

- [ ] No `tabula_data` (V1) in workspace
- [ ] No `old_tabula_cli` in workspace
- [ ] No `tabula.json` file exists
- [ ] `tabula_data_v2` renamed to `tabula_data`
- [ ] `tabula_ids` renamed to `tabula_generated`
- [ ] All tests pass
- [ ] `just review` passes
- [ ] No warnings about unused dependencies

## Documentation Updates

Update any documentation referencing old structure:
- CLAUDE.md if it mentions tabula_data
- Any design docs referencing V1
- README or setup instructions

## Verification

- `just review` passes
- `cargo test --workspace` passes
- Clean git status (no unexpected changes)
- Build succeeds on all targets

## Context Files

1. `Cargo.toml` (workspace) - Member list to update
2. `CLAUDE.md` - Documentation to check
3. `src/tabula_data_v2/Cargo.toml` - To rename
4. All dependent crate Cargo.toml files
