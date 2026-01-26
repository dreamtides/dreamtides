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
    stored_content: Mutex<Option<String>>,
    last_written: Mutex<Option<String>>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            read_result: Mutex::new(None),
            write_result: Mutex::new(None),
            atomic_write_result: Mutex::new(Some(Ok(()))),
            temp_files: Mutex::new(Vec::new()),
            exists_result: true,
            stored_content: Mutex::new(None),
            last_written: Mutex::new(None),
        }
    }

    pub fn with_read_content(content: &str) -> Self {
        Self {
            read_result: Mutex::new(Some(Ok(content.to_string()))),
            write_result: Mutex::new(None),
            atomic_write_result: Mutex::new(Some(Ok(()))),
            temp_files: Mutex::new(Vec::new()),
            exists_result: true,
            stored_content: Mutex::new(None),
            last_written: Mutex::new(None),
        }
    }

    /// Creates a mock that supports a read followed by an atomic write,
    /// capturing the written content for later inspection.
    pub fn with_read_and_write(content: &str) -> Self {
        Self {
            read_result: Mutex::new(None),
            write_result: Mutex::new(None),
            atomic_write_result: Mutex::new(None),
            temp_files: Mutex::new(Vec::new()),
            exists_result: true,
            stored_content: Mutex::new(Some(content.to_string())),
            last_written: Mutex::new(None),
        }
    }

    /// Returns the content from the last atomic write, if any.
    pub fn last_written_content(&self) -> Option<String> {
        self.last_written.lock().unwrap_or_else(|e| panic!("Lock poisoned: {e}")).clone()
    }

    pub fn failing_read(error: io::Error) -> Self {
        Self {
            read_result: Mutex::new(Some(Err(error))),
            write_result: Mutex::new(None),
            atomic_write_result: Mutex::new(Some(Ok(()))),
            temp_files: Mutex::new(Vec::new()),
            exists_result: true,
            stored_content: Mutex::new(None),
            last_written: Mutex::new(None),
        }
    }

    pub fn failing_write(error: io::Error) -> Self {
        Self {
            read_result: Mutex::new(None),
            write_result: Mutex::new(Some(Err(error))),
            atomic_write_result: Mutex::new(Some(Ok(()))),
            temp_files: Mutex::new(Vec::new()),
            exists_result: true,
            stored_content: Mutex::new(None),
            last_written: Mutex::new(None),
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
        if let Some(ref content) =
            *self.stored_content.lock().unwrap_or_else(|e| panic!("Lock poisoned: {e}"))
        {
            return Ok(content.clone());
        }

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

    fn write_atomic(&self, _path: &Path, content: &str) -> Result<(), AtomicWriteError> {
        *self.last_written.lock().unwrap_or_else(|e| panic!("Lock poisoned: {e}")) =
            Some(content.to_string());

        if let Some(result) =
            self.atomic_write_result.lock().unwrap_or_else(|e| panic!("Lock poisoned: {e}")).take()
        {
            return result;
        }

        Ok(())
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
