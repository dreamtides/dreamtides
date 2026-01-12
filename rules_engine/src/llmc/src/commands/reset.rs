use std::io::{self, Write};

use anyhow::Result;

use super::super::config::{self, Config};
use super::super::state::{self, State};
use super::super::worker;

/// Runs the reset command, force-resetting a specific worker to clean state
pub fn run_reset(worker_name: &str, yes: bool) -> Result<()> {
    let config_path = config::get_config_path();
    let config = Config::load(&config_path)?;

    let state_path = state::get_state_path();
    let mut state = State::load(&state_path)?;

    if state.get_worker(worker_name).is_none() {
        anyhow::bail!("Worker '{}' not found", worker_name);
    }

    if !yes {
        println!("This will reset worker '{}' to clean state by:", worker_name);
        println!("  - Discarding all uncommitted changes");
        println!("  - Aborting any in-progress rebases");
        println!("  - Resetting to origin/master");
        println!("  - Clearing work state and marking as Idle");
        print!("\nContinue? (y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            println!("Reset cancelled.");
            return Ok(());
        }
    }

    println!("Resetting worker '{}'...", worker_name);

    match worker::reset_worker_to_clean_state(worker_name, &mut state, &config) {
        Ok(actions) => {
            for action in &actions {
                println!("  ✓ {}", action);
            }
            state.save(&state_path)?;
            println!("\n✓ Worker '{}' reset successfully", worker_name);
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Failed to reset worker '{}': {}", worker_name, e);
        }
    }
}
