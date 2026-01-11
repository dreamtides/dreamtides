use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use chrono::Utc;
use tracing_subscriber::fmt::MakeWriter;

pub struct SizeRotatingWriter {
    inner: Arc<Mutex<SizeRotatingWriterInner>>,
}

struct SizeRotatingWriterInner {
    path: PathBuf,
    writer: BufWriter<File>,
    current_size: u64,
    max_size: u64,
}

impl SizeRotatingWriter {
    const MAX_SIZE: u64 = 100 * 1024 * 1024;

    pub fn new(path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("Failed to open log file: {}", path.display()))?;
        let current_size = file
            .metadata()
            .with_context(|| format!("Failed to get log file metadata: {}", path.display()))?
            .len();
        Ok(Self {
            inner: Arc::new(Mutex::new(SizeRotatingWriterInner {
                path: path.clone(),
                writer: BufWriter::new(file),
                current_size,
                max_size: Self::MAX_SIZE,
            })),
        })
    }

    fn rotate(inner: &mut SizeRotatingWriterInner) -> Result<()> {
        inner.writer.flush().ok();
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let rotated_path = inner.path.with_extension(format!("jsonl.{timestamp}"));
        std::fs::rename(&inner.path, &rotated_path)
            .with_context(|| format!("Failed to rotate log file to: {}", rotated_path.display()))?;
        let new_file =
            OpenOptions::new().create(true).append(true).open(&inner.path).with_context(|| {
                format!("Failed to open new log file: {}", inner.path.display())
            })?;
        inner.writer = BufWriter::new(new_file);
        inner.current_size = 0;
        Ok(())
    }
}

impl Write for SizeRotatingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut inner = self.inner.lock().unwrap();
        let len = inner.writer.write(buf)?;
        inner.current_size += len as u64;
        if inner.current_size >= inner.max_size {
            Self::rotate(&mut inner).ok();
        }
        Ok(len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.lock().unwrap().writer.flush()
    }
}

impl Clone for SizeRotatingWriter {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

impl<'a> MakeWriter<'a> for SizeRotatingWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}
