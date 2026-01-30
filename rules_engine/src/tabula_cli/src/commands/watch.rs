use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::Duration;

use anyhow::{Context, Result};
use notify_debouncer_mini::new_debouncer;
use notify_debouncer_mini::notify::RecursiveMode;

use crate::commands::generate;

/// Watches tabula source files and regenerates on changes.
pub fn watch(output_dir: Option<PathBuf>) -> Result<()> {
    let tabula_dir = generate::tabula_source_dir();
    println!("Watching {} for changes...", tabula_dir.display());

    // Initial generation
    generate::generate(output_dir.clone())?;

    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, stopping...");
        running_clone.store(false, Ordering::SeqCst);
    })
    .context("Failed to set Ctrl+C handler")?;

    // Set up file watcher
    let (tx, rx) = channel();
    let mut debouncer =
        new_debouncer(Duration::from_millis(200), tx).context("Failed to create file watcher")?;
    debouncer
        .watcher()
        .watch(&tabula_dir, RecursiveMode::NonRecursive)
        .context("Failed to watch tabula directory")?;

    // Main loop
    let dirty = Arc::new(AtomicBool::new(false));

    while running.load(Ordering::SeqCst) {
        // Check for file events (non-blocking with timeout)
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(events)) => {
                let relevant = events
                    .iter()
                    .any(|e| e.path.extension().is_some_and(|ext| ext == "toml" || ext == "ftl"));
                if relevant {
                    dirty.store(true, Ordering::SeqCst);
                }
            }
            Ok(Err(e)) => {
                eprintln!("Watch error: {e:?}");
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                break;
            }
        }

        // If dirty, regenerate
        if dirty.load(Ordering::SeqCst) {
            dirty.store(false, Ordering::SeqCst);
            println!("\nChanges detected, regenerating...");
            match generate::generate(output_dir.clone()) {
                Ok(()) => println!("Regeneration complete."),
                Err(e) => eprintln!("Regeneration failed: {e:#}"),
            }
            // After generation, check if more changes came in during generation
            // and loop will handle them on next iteration
        }
    }

    println!("Watch stopped.");
    Ok(())
}
