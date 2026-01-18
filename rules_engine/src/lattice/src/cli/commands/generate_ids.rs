use tracing::info;

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::task_args::GenerateIdsArgs;
use crate::error::error_types::LatticeError;
use crate::git::client_config;
use crate::id::id_generator::INITIAL_COUNTER;
use crate::id::lattice_id::LatticeId;
use crate::index::client_counters;

/// Executes the `lat generate-ids` command.
///
/// Pre-allocates IDs for offline authoring by generating the requested number
/// of IDs and outputting them to stdout. Each ID is output on its own line.
pub fn execute(context: CommandContext, args: GenerateIdsArgs) -> LatticeResult<()> {
    info!(count = args.count, "Executing generate-ids command");

    let client_id = get_or_create_client_id(&context)?;
    let ids = generate_ids(&context, &client_id, args.count)?;

    if context.global.json {
        let json = serde_json::json!({ "ids": ids });
        println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
    } else {
        for id in &ids {
            println!("{id}");
        }
    }

    info!(count = ids.len(), "Generated IDs");
    Ok(())
}

/// Gets the client ID for this repository, creating one if needed.
fn get_or_create_client_id(context: &CommandContext) -> LatticeResult<String> {
    if let Some(client_id) = client_config::get_client_id(&context.repo_root)? {
        return Ok(client_id);
    }

    let client_id = client_config::generate_client_id();
    client_config::set_client_id(&context.repo_root, &client_id)?;
    info!(client_id, "Created new client ID");
    Ok(client_id)
}

/// Generates the requested number of IDs, persisting counter state.
fn generate_ids(
    context: &CommandContext,
    client_id: &str,
    count: usize,
) -> LatticeResult<Vec<String>> {
    if count == 0 {
        return Err(LatticeError::InvalidArgument {
            message: "count must be at least 1".to_string(),
        });
    }

    let mut ids = Vec::with_capacity(count);

    for _ in 0..count {
        let counter = client_counters::get_and_increment(&context.conn, client_id)?;
        let effective_counter = counter + INITIAL_COUNTER;
        let id = LatticeId::from_parts(effective_counter, client_id);
        ids.push(id.to_string());
    }

    Ok(ids)
}
