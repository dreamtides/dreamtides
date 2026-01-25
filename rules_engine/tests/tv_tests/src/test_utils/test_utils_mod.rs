use std::path::{Path, PathBuf};

use tempfile::TempDir;
use tv_lib::error::error_types::TvError;
use tv_lib::toml::document_loader::{load_toml_document_with_fs, TomlTableData};
use tv_lib::toml::document_writer::{
    save_batch_with_fs, save_cell_with_fs, save_toml_document_with_fs, CellUpdate, SaveBatchResult,
    SaveCellResult,
};
use tv_lib::traits::{FileSystem, RealFileSystem};

use crate::test_utils::mock_filesystem::MockFileSystem;

pub struct TvTestHarness {
    temp_dir: TempDir,
    fs: Box<dyn FileSystem>,
}

impl TvTestHarness {
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().unwrap_or_else(|e| panic!("Failed to create temp dir: {e}")),
            fs: Box::new(RealFileSystem),
        }
    }

    pub fn with_mock_fs(mock: MockFileSystem) -> Self {
        Self {
            temp_dir: TempDir::new().unwrap_or_else(|e| panic!("Failed to create temp dir: {e}")),
            fs: Box::new(mock),
        }
    }

    pub fn temp_dir(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn create_toml_file(&self, name: &str, content: &str) -> PathBuf {
        let path = self.temp_dir.path().join(name);
        std::fs::write(&path, content).unwrap_or_else(|e| panic!("Failed to write fixture: {e}"));
        path
    }

    pub fn load_table(&self, path: &Path, table_name: &str) -> Result<TomlTableData, TvError> {
        load_toml_document_with_fs(
            &*self.fs,
            path.to_str().unwrap_or_else(|| panic!("Invalid path: {path:?}")),
            table_name,
        )
    }

    pub fn save_table(
        &self,
        path: &Path,
        table_name: &str,
        data: &TomlTableData,
    ) -> Result<(), TvError> {
        save_toml_document_with_fs(
            &*self.fs,
            path.to_str().unwrap_or_else(|| panic!("Invalid path: {path:?}")),
            table_name,
            data,
        )
    }

    pub fn read_file_content(&self, path: &Path) -> String {
        std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read file {}: {e}", path.display()))
    }

    pub fn save_cell(
        &self,
        path: &Path,
        table_name: &str,
        row_index: usize,
        column_key: &str,
        value: serde_json::Value,
    ) -> Result<SaveCellResult, TvError> {
        let update = CellUpdate { row_index, column_key: column_key.to_string(), value };
        save_cell_with_fs(
            &*self.fs,
            path.to_str().unwrap_or_else(|| panic!("Invalid path: {path:?}")),
            table_name,
            &update,
        )
    }

    pub fn save_batch(
        &self,
        path: &Path,
        table_name: &str,
        updates: &[CellUpdate],
    ) -> Result<SaveBatchResult, TvError> {
        save_batch_with_fs(
            &*self.fs,
            path.to_str().unwrap_or_else(|| panic!("Invalid path: {path:?}")),
            table_name,
            updates,
        )
    }
}

impl Default for TvTestHarness {
    fn default() -> Self {
        Self::new()
    }
}
