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
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::config::{Config, DefaultsConfig, RepoConfig};
    use crate::llmc::sound::*;
    #[test]
    fn test_play_bell_enabled() {
        let config = Config {
            defaults: DefaultsConfig {
                model: "opus".to_string(),
                skip_permissions: true,
                allowed_tools: vec![],
                patrol_interval_secs: 60,
                sound_on_review: true,
                self_review: None,
            },
            repo: RepoConfig { source: "/test".to_string() },
            workers: HashMap::new(),
        };
        let result = play_bell(&config);
        assert!(result.is_ok());
    }
    #[test]
    fn test_play_bell_disabled() {
        let config = Config {
            defaults: DefaultsConfig {
                model: "opus".to_string(),
                skip_permissions: true,
                allowed_tools: vec![],
                patrol_interval_secs: 60,
                sound_on_review: false,
                self_review: None,
            },
            repo: RepoConfig { source: "/test".to_string() },
            workers: HashMap::new(),
        };
        let result = play_bell(&config);
        assert!(result.is_ok());
    }
}
