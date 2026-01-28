use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::SystemTime;

pub trait FileSystem: Send + Sync {
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    fn write(&self, path: &Path, content: &str) -> io::Result<()>;
    fn write_atomic(&self, path: &Path, content: &str) -> Result<(), AtomicWriteError>;
    fn exists(&self, path: &Path) -> bool;
    fn read_dir_temp_files(&self, dir: &Path, prefix: &str) -> io::Result<Vec<std::path::PathBuf>>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
}

pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
}

#[derive(Debug)]
pub enum AtomicWriteError {
    TempFileCreate(io::Error),
    Write(io::Error),
    Sync(io::Error),
    Rename { source: io::Error, temp_path: String },
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn write(&self, path: &Path, content: &str) -> io::Result<()> {
        std::fs::write(path, content)
    }

    fn write_atomic(&self, path: &Path, content: &str) -> Result<(), AtomicWriteError> {
        let parent = path.parent().unwrap_or(Path::new("."));
        let random_suffix: u64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
            ^ (std::process::id() as u64);
        let temp_name = format!(".tv_save_{:x}.tmp", random_suffix);
        let temp_path = parent.join(&temp_name);

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(AtomicWriteError::TempFileCreate)?;

        file.write_all(content.as_bytes()).map_err(AtomicWriteError::Write)?;

        file.sync_all().map_err(AtomicWriteError::Sync)?;

        std::fs::rename(&temp_path, path).map_err(|e| AtomicWriteError::Rename {
            source: e,
            temp_path: temp_path.to_string_lossy().to_string(),
        })?;

        sync_parent_directory(parent);

        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_dir_temp_files(&self, dir: &Path, prefix: &str) -> io::Result<Vec<std::path::PathBuf>> {
        let mut temp_files = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            if name_str.starts_with(prefix) && name_str.ends_with(".tmp") {
                temp_files.push(entry.path());
            }
        }
        Ok(temp_files)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
    }
}

#[cfg(unix)]
fn sync_parent_directory(parent: &Path) {
    if let Ok(dir) = File::open(parent) {
        let _ = dir.sync_all();
    }
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path) {}

pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}
