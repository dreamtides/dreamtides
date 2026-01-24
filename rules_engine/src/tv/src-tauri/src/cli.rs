use clap::Parser;
use std::path::PathBuf;

pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const INVALID_ARGUMENTS: i32 = 1;
    pub const FILE_ERROR: i32 = 2;
}

#[derive(Parser, Debug)]
#[command(name = "tv")]
#[command(about = "TOML Viewer - Spreadsheet editor for TOML files")]
pub struct Args {
    #[arg(help = "Path to TOML file or directory to open. Defaults to rules_engine/tabula/ when not provided")]
    pub path: Option<PathBuf>,

    #[arg(long, help = "Output logs in JSONL format to stdout")]
    pub jsonl: bool,
}

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub files: Vec<PathBuf>,
}

pub fn resolve_paths(args: &Args) -> Result<AppPaths, String> {
    match &args.path {
        Some(path) => {
            if !path.exists() {
                if path.is_file() || path.extension().is_some() {
                    return Err(format!(
                        "File not found: {} - Please verify the path exists",
                        path.display()
                    ));
                } else {
                    return Err(format!(
                        "Directory not found: {} - Please verify the directory exists",
                        path.display()
                    ));
                }
            }

            if path.is_file() {
                Ok(AppPaths {
                    files: vec![path.clone()],
                })
            } else if path.is_dir() {
                let files = scan_directory_for_toml(path)?;
                if files.is_empty() {
                    return Err(format!(
                        "No TOML files found in directory: {}",
                        path.display()
                    ));
                }
                Ok(AppPaths { files })
            } else {
                Err(format!(
                    "Path is neither a file nor a directory: {}",
                    path.display()
                ))
            }
        }
        None => {
            let default_path = get_default_tabula_path()?;
            if !default_path.exists() {
                return Err(format!(
                    "Default directory not found: {} - Please verify the directory exists",
                    default_path.display()
                ));
            }
            let files = scan_directory_for_toml(&default_path)?;
            if files.is_empty() {
                return Err(format!(
                    "No TOML files found in directory: {}",
                    default_path.display()
                ));
            }
            Ok(AppPaths { files })
        }
    }
}

fn get_default_tabula_path() -> Result<PathBuf, String> {
    let exe_path =
        std::env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;
    let mut search_path = exe_path
        .parent()
        .ok_or("Failed to get executable directory")?
        .to_path_buf();
    loop {
        let tabula_path = search_path.join("rules_engine").join("tabula");
        if tabula_path.exists() {
            return Ok(tabula_path);
        }
        if !search_path.pop() {
            break;
        }
    }
    Err("Could not find rules_engine/tabula directory relative to executable".to_string())
}

fn scan_directory_for_toml(dir: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file() && path.extension().is_some_and(|ext| ext == "toml")
        })
        .collect();
    files.sort();
    Ok(files)
}
