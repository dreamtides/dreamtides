use std::io;
use std::path::Path;
use std::sync::Mutex;

use tv_lib::traits::FileSystem;

pub struct MockFileSystem {
    read_result: Mutex<Option<io::Result<String>>>,
    write_result: Mutex<Option<io::Result<()>>>,
    exists_result: bool,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self { read_result: Mutex::new(None), write_result: Mutex::new(None), exists_result: true }
    }

    pub fn with_read_content(content: &str) -> Self {
        Self {
            read_result: Mutex::new(Some(Ok(content.to_string()))),
            write_result: Mutex::new(None),
            exists_result: true,
        }
    }

    pub fn failing_read(error: io::Error) -> Self {
        Self {
            read_result: Mutex::new(Some(Err(error))),
            write_result: Mutex::new(None),
            exists_result: true,
        }
    }

    pub fn failing_write(error: io::Error) -> Self {
        Self {
            read_result: Mutex::new(None),
            write_result: Mutex::new(Some(Err(error))),
            exists_result: true,
        }
    }

    pub fn with_exists(mut self, exists: bool) -> Self {
        self.exists_result = exists;
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

    fn exists(&self, _path: &Path) -> bool {
        self.exists_result
    }
}
