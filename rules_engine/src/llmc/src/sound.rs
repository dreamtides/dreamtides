use std::io::{self, Write};

use anyhow::Result;

use crate::config::Config;
/// Plays a terminal bell sound to notify the user
pub fn play_bell(config: &Config) -> Result<()> {
    if !config.defaults.sound_on_review {
        return Ok(());
    }
    print!("\x07");
    io::stdout().flush()?;
    Ok(())
}
