use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use toml::Value;
use zip::ZipArchive;

use crate::commands::{build_toml, rebuild_images, strip_images};
use crate::core::{column_names, excel_reader, paths, toml_data};

#[derive(Clone, Copy, ValueEnum)]
pub enum Hook {
    PreCommit,
    PostCheckout,
    PostMerge,
    PostCommit,
}

pub fn git_setup() -> Result<()> {
    let root = paths::git_root()?;
    git_setup_for_root(&root)
}

pub fn git_setup_for_root(root: &Path) -> Result<()> {
    ensure_gitattributes(root)?;
    let cache_dir = paths::image_cache_dir_for(root);
    fs::create_dir_all(&cache_dir)
        .with_context(|| format!("Cannot write to output directory {}", cache_dir.display()))?;
    install_hook(root, Hook::PreCommit)?;
    install_hook(root, Hook::PostCheckout)?;
    install_hook(root, Hook::PostMerge)?;
    install_hook(root, Hook::PostCommit)?;
    Ok(())
}

pub fn run_hook(hook: Hook) -> Result<()> {
    let root = paths::git_root()?;
    run_hook_for_root(hook, &root)
}

pub fn run_hook_for_root(hook: Hook, root: &Path) -> Result<()> {
    match hook {
        Hook::PreCommit => run_pre_commit(root),
        Hook::PostCheckout | Hook::PostMerge | Hook::PostCommit => rebuild_after_checkout(root),
    }
}

fn run_pre_commit(root: &Path) -> Result<()> {
    let xlsm_path = tabula_path(root);
    if !xlsm_path.exists() {
        bail!("Tabula spreadsheet not found at {}", display_path(root, &xlsm_path));
    }
    let toml_dir = tabula_toml_dir(root);
    fs::create_dir_all(&toml_dir)
        .with_context(|| format!("Cannot write to output directory {}", toml_dir.display()))?;
    ensure_toml_not_newer(root, &xlsm_path, &toml_dir)?;
    build_toml::build_toml(Some(xlsm_path.clone()), Some(toml_dir.clone()))?;
    strip_images::strip_images(Some(xlsm_path.clone()), Some(xlsm_path.clone()))?;
    ensure_stripped(&xlsm_path)?;
    stage_paths(root, &[xlsm_path.as_path(), toml_dir.as_path()])?;
    ensure_staged_stripped(root, &xlsm_path)
}

fn rebuild_after_checkout(root: &Path) -> Result<()> {
    let xlsm_path = tabula_path(root);
    if !xlsm_path.exists() {
        return Ok(());
    }
    rebuild_images::rebuild_images(Some(xlsm_path), false, true)
}

fn ensure_toml_not_newer(root: &Path, xlsm_path: &Path, toml_dir: &Path) -> Result<()> {
    let expected = expected_toml(root, xlsm_path)?;
    let metadata = fs::metadata(xlsm_path)
        .with_context(|| format!("Original XLSM not found at {}", display_path(root, xlsm_path)))?;
    let xlsm_modified = metadata
        .modified()
        .with_context(|| format!("Cannot read timestamp for {}", display_path(root, xlsm_path)))?;
    let entries = fs::read_dir(toml_dir)
        .with_context(|| format!("Cannot open TOML directory {}", display_path(root, toml_dir)))?;
    for entry in entries {
        let entry = entry.with_context(|| {
            format!("Cannot open TOML directory {}", display_path(root, toml_dir))
        })?;
        let path = entry.path();
        if path.extension() != Some(OsStr::new("toml")) {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|m| m.modified())
            .with_context(|| format!("Cannot read timestamp for {}", display_path(root, &path)))?;
        if modified > xlsm_modified {
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                bail!(
                    "TOML file {} is newer than {}; run tabula build-xls and retry",
                    display_path(root, &path),
                    display_path(root, xlsm_path)
                );
            };
            let Some(expected_value) = expected.get(file_name) else {
                bail!(
                    "TOML file {} is newer than {}; run tabula build-xls and retry",
                    display_path(root, &path),
                    display_path(root, xlsm_path)
                );
            };
            let contents = fs::read_to_string(&path)
                .with_context(|| format!("Cannot open TOML file {}", display_path(root, &path)))?;
            let actual: Value = toml::from_str(&contents)
                .with_context(|| format!("Cannot parse TOML file {}", display_path(root, &path)))?;
            let actual = toml_data::canonicalize_numbers(actual);
            if &actual != expected_value {
                bail!(
                    "TOML file {} differs from {} and is newer; run tabula build-xls and retry",
                    display_path(root, &path),
                    display_path(root, xlsm_path)
                );
            }
        }
    }
    Ok(())
}

fn expected_toml(root: &Path, xlsm_path: &Path) -> Result<BTreeMap<String, Value>> {
    let tables = excel_reader::extract_tables(xlsm_path)?;
    let mut expected = BTreeMap::new();
    for table in tables {
        let file_name = format!("{}.toml", column_names::normalize_table_name(table.name.as_str()));
        let serialized = toml_data::table_to_toml(&table)?;
        let parsed: Value = toml::from_str(&serialized).with_context(|| {
            format!(
                "Cannot parse TOML for table {} in {}",
                table.name,
                display_path(root, xlsm_path)
            )
        })?;
        expected.insert(file_name, toml_data::canonicalize_numbers(parsed));
    }
    Ok(expected)
}

fn ensure_stripped(xlsm_path: &Path) -> Result<()> {
    let file = fs::File::open(xlsm_path)
        .with_context(|| format!("Cannot open spreadsheet at {}", xlsm_path.display()))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("Cannot open spreadsheet at {}", xlsm_path.display()))?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        if !name.starts_with("xl/media/") {
            continue;
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        if data != strip_images::PLACEHOLDER_JPEG {
            bail!("Embedded image {name} still present after strip-images");
        }
    }
    Ok(())
}

fn ensure_staged_stripped(root: &Path, xlsm_path: &Path) -> Result<()> {
    let bytes = staged_bytes(root, xlsm_path)?;
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)
        .with_context(|| format!("Cannot open spreadsheet at {}", xlsm_path.display()))?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        if !name.starts_with("xl/media/") {
            continue;
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        if data != strip_images::PLACEHOLDER_JPEG {
            bail!("Staged spreadsheet still contains embedded image {name}");
        }
    }
    Ok(())
}

fn staged_bytes(root: &Path, path: &Path) -> Result<Vec<u8>> {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let output = Command::new("git")
        .current_dir(root)
        .arg("show")
        .arg(format!(":{}", relative.display()))
        .output()
        .context("Failed to read staged Tabula.xlsm from index")?;
    if !output.status.success() {
        bail!("Staged Tabula.xlsm not found in index; run git add and retry");
    }
    Ok(output.stdout)
}

fn ensure_gitattributes(root: &Path) -> Result<()> {
    let path = root.join(".gitattributes");
    let desired = "client/Assets/StreamingAssets/Tabula.xlsm filter=lfs diff=lfs merge=lfs -text\n";
    let mut content = if path.exists() {
        fs::read_to_string(&path).with_context(|| format!("Cannot open {}", path.display()))?
    } else {
        String::new()
    };
    if content.contains(desired.trim_end()) {
        return Ok(());
    }
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(desired);
    fs::write(&path, content).with_context(|| format!("Cannot write to {}", path.display()))
}

fn install_hook(root: &Path, hook: Hook) -> Result<()> {
    let hooks_dir = root.join(".git/hooks");
    fs::create_dir_all(&hooks_dir)
        .with_context(|| format!("Cannot write to output directory {}", hooks_dir.display()))?;
    let path = hooks_dir.join(hook.script_name());
    let script = hook_script(hook);
    if path.exists() {
        let existing =
            fs::read(&path).with_context(|| format!("Cannot open hook {}", path.display()))?;
        if existing == script.as_bytes() {
            set_executable(&path)?;
            return Ok(());
        }
        bail!("Hook {} already exists; remove it or edit manually", hook.script_name());
    }
    fs::write(&path, script).with_context(|| format!("Cannot write hook {}", path.display()))?;
    set_executable(&path)
}

fn stage_paths(root: &Path, paths: &[&Path]) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let status = Command::new("git")
        .current_dir(root)
        .arg("add")
        .arg("--")
        .args(paths)
        .status()
        .context("Failed to stage files with git add")?;
    if status.success() { Ok(()) } else { bail!("git add failed") }
}

fn hook_script(hook: Hook) -> String {
    let hook_arg = hook.arg();
    format!(
        "#!/bin/sh\nset -e\nROOT=\"$(git rev-parse --show-toplevel)\"\ncd \"$ROOT\"\nif command -v tabula >/dev/null 2>&1; then\n  tabula git-hook {hook_arg}\nelif command -v tabula_cli >/dev/null 2>&1; then\n  tabula_cli git-hook {hook_arg}\nelif [ -x \"$ROOT/rules_engine/target/debug/tabula_cli\" ]; then\n  \"$ROOT/rules_engine/target/debug/tabula_cli\" git-hook {hook_arg}\nelse\n  cargo run --quiet --manifest-path \"$ROOT/rules_engine/Cargo.toml\" -p tabula_cli -- git-hook {hook_arg}\nfi\n"
    )
}

fn tabula_path(root: &Path) -> PathBuf {
    root.join("client/Assets/StreamingAssets/Tabula.xlsm")
}

fn tabula_toml_dir(root: &Path) -> PathBuf {
    root.join("client/Assets/StreamingAssets/Tabula")
}

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

impl Hook {
    fn script_name(self) -> &'static str {
        match self {
            Hook::PreCommit => "pre-commit",
            Hook::PostCheckout => "post-checkout",
            Hook::PostMerge => "post-merge",
            Hook::PostCommit => "post-commit",
        }
    }

    fn arg(self) -> &'static str {
        match self {
            Hook::PreCommit => "pre-commit",
            Hook::PostCheckout => "post-checkout",
            Hook::PostMerge => "post-merge",
            Hook::PostCommit => "post-commit",
        }
    }
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .with_context(|| format!("Cannot open hook {}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("Cannot write hook {}", path.display()))
}

#[cfg(not(unix))]
fn set_executable(path: &Path) -> Result<()> {
    let permissions = fs::metadata(path)
        .with_context(|| format!("Cannot open hook {}", path.display()))?
        .permissions();
    fs::set_permissions(path, permissions)
        .with_context(|| format!("Cannot write hook {}", path.display()))
}
