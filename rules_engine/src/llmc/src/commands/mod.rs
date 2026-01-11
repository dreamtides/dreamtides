pub mod accept;
pub mod add;
pub mod attach;
pub mod doctor;
pub mod down;
pub mod init;
pub mod message;
pub mod nuke;
pub mod rebase;
pub mod reject;
pub mod review;
pub mod start;
pub mod status;
pub mod up;

use anyhow::Result;

use super::config::{self, Config};
use super::patrol::Patrol;
use super::state::{self, State};

/// Runs patrol to update worker states, then returns the updated state
pub fn load_state_with_patrol() -> Result<(State, Config)> {
    let state_path = state::get_state_path();
    let mut state = State::load(&state_path)?;

    let config_path = config::get_config_path();
    let config = Config::load(&config_path)?;

    let patrol = Patrol::new(&config);
    let _report = patrol.run_patrol(&mut state, &config)?;

    state.save(&state_path)?;

    Ok((state, config))
}
