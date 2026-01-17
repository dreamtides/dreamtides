use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use crate::state::{State, WorkerStatus};
use crate::tmux::sender::TmuxSender;
use crate::{config, editor};

/// Runs the message command, sending a follow-up message to a worker
pub fn run_message(worker: &str, message: Option<String>, json: bool) -> Result<()> {
    let llmc_root = config::get_llmc_root();
    if !llmc_root.exists() {
        bail!(
            "LLMC workspace not initialized. Run 'llmc init' first.\n\
             Expected workspace at: {}",
            llmc_root.display()
        );
    }

    let (mut state, _config) = crate::state::load_state_with_patrol()?;

    let worker_record = state.get_worker(worker).ok_or_else(|| {
        anyhow::anyhow!(
            "Worker '{}' not found\n\
             Available workers: {}",
            worker,
            format_all_workers(&state)
        )
    })?;

    verify_valid_state_for_message(worker, worker_record.status)?;

    let message_text = match message {
        Some(m) if !m.trim().is_empty() => m,
        Some(_) => bail!("Message cannot be empty"),
        None => {
            let template = "# Enter your message to the worker below.\n\
                            # Lines starting with '#' will be ignored.\n\
                            # Save and close the editor to send, or leave empty to abort.\n\n";
            editor::open_editor(Some(template), "message")?
        }
    };

    println!("Sending message to worker '{}'...", worker);

    let tmux_sender = TmuxSender::new();
    tmux_sender
        .send(&worker_record.session_id, &message_text)
        .with_context(|| format!("Failed to send message to worker '{}'", worker))?;

    let worker_mut = state.get_worker_mut(worker).unwrap();
    worker_mut.last_activity_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    state.save(&crate::state::get_state_path())?;

    if json {
        let output =
            crate::json_output::MessageOutput { worker: worker.to_string(), message_sent: true };
        crate::json_output::print_json(&output);
    } else {
        println!("✓ Message sent to worker '{}'", worker);
    }

    Ok(())
}

fn verify_valid_state_for_message(worker: &str, status: WorkerStatus) -> Result<()> {
    match status {
        WorkerStatus::Working
        | WorkerStatus::NeedsReview
        | WorkerStatus::Rejected
        | WorkerStatus::Rebasing
        | WorkerStatus::Error => Ok(()),
        _ => {
            bail!(
                "Worker '{}' is in state {:?}, which cannot receive messages\n\
                 Valid states for messaging: Working, NeedsReview, Rejected, Rebasing, Error",
                worker,
                status
            )
        }
    }
}

fn format_all_workers(state: &State) -> String {
    if state.workers.is_empty() {
        return "none".to_string();
    }
    state.workers.keys().map(String::as_str).collect::<Vec<_>>().join(", ")
}
