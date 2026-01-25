use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tv_lib::traits::{AtomicWriteError, FileSystem};

pub struct MockFileSystem {
    read_result: Mutex<Option<io::Result<String>>>,
    write_result: Mutex<Option<io::Result<()>>>,
    atomic_write_result: Mutex<Option<Result<(), AtomicWriteError>>>,
    temp_files: Mutex<Vec<PathBuf>>,
    exists_result: bool,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            read_result: Mutex::new(None),
            write_result: Mutex::new(None),
            atomic_write_result: Mutex::new(Some(Ok(()))),
            temp_files: Mutex::new(Vec::new()),
            exists_result: true,
        }
    }

    pub fn with_read_content(content: &str) -> Self {
        Self {
            read_result: Mutex::new(Some(Ok(content.to_string()))),
            write_result: Mutex::new(None),
            atomic_write_result: Mutex::new(Some(Ok(()))),
            temp_files: Mutex::new(Vec::new()),
            exists_result: true,
        }
    }

    pub fn failing_read(error: io::Error) -> Self {
        Self {
            read_result: Mutex::new(Some(Err(error))),
            write_result: Mutex::new(None),
            atomic_write_result: Mutex::new(Some(Ok(()))),
            temp_files: Mutex::new(Vec::new()),
            exists_result: true,
        }
    }

    pub fn failing_write(error: io::Error) -> Self {
        Self {
            read_result: Mutex::new(None),
            write_result: Mutex::new(Some(Err(error))),
            atomic_write_result: Mutex::new(Some(Ok(()))),
            temp_files: Mutex::new(Vec::new()),
            exists_result: true,
        }
    }

    pub fn with_exists(mut self, exists: bool) -> Self {
        self.exists_result = exists;
        self
    }

    pub fn with_temp_files(self, files: Vec<PathBuf>) -> Self {
        *self.temp_files.lock().unwrap_or_else(|e| panic!("Lock poisoned: {e}")) = files;
        self
    }
}

impl Default for MockFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for MockFileSystem {
    fn read_to_string(&self, _path: &Path) -> io::Result<String> {
        self.read_result
            .lock()
            .unwrap_or_else(|e| panic!("Lock poisoned: {e}"))
            .take()
            .unwrap_or_else(|| {
                panic!("MockFileSystem: read_to_string called but no result configured")
            })
    }

    fn write(&self, _path: &Path, _content: &str) -> io::Result<()> {
        self.write_result
            .lock()
            .unwrap_or_else(|e| panic!("Lock poisoned: {e}"))
            .take()
            .unwrap_or_else(|| panic!("MockFileSystem: write called but no result configured"))
    }

    fn write_atomic(&self, _path: &Path, _content: &str) -> Result<(), AtomicWriteError> {
        self.atomic_write_result
            .lock()
            .unwrap_or_else(|e| panic!("Lock poisoned: {e}"))
            .take()
            .unwrap_or_else(|| {
                panic!("MockFileSystem: write_atomic called but no result configured")
            })
    }

    fn exists(&self, _path: &Path) -> bool {
        self.exists_result
    }

    fn read_dir_temp_files(&self, _dir: &Path, _prefix: &str) -> io::Result<Vec<PathBuf>> {
        Ok(self.temp_files.lock().unwrap_or_else(|e| panic!("Lock poisoned: {e}")).clone())
    }

    fn remove_file(&self, _path: &Path) -> io::Result<()> {
        Ok(())
    }
}
