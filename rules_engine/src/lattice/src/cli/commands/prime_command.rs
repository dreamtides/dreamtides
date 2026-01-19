use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use serde::Serialize;
use tracing::info;

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::workflow_args::PrimeArgs;
use crate::error::error_types::LatticeError;

/// Executes the `lat prime` command.
pub fn execute(context: CommandContext, args: PrimeArgs) -> LatticeResult<()> {
    info!(full = args.full, export = ?args.export, "Executing prime command");

    let content = generate_prime_content(&context, args.full);

    if let Some(ref export_path) = args.export {
        export_to_file(export_path, &content)?;
        if !context.global.json {
            println!("Exported prime context to {export_path}");
        }
        return Ok(());
    }

    if context.global.json {
        print_json_output(&context);
    } else {
        println!("{content}");
    }

    Ok(())
}

/// JSON output structure for prime command.
#[derive(Debug, Serialize)]
struct PrimeOutput {
    session_protocol: Vec<String>,
    core_commands: Vec<CoreCommand>,
    link_authoring: LinkAuthoringGuide,
}

/// Core command reference.
#[derive(Debug, Serialize)]
struct CoreCommand {
    command: String,
    description: String,
}

/// Link authoring guidance.
#[derive(Debug, Serialize)]
struct LinkAuthoringGuide {
    shorthand_example: String,
    expanded_example: String,
    instruction: String,
}

fn generate_prime_content(context: &CommandContext, full: bool) -> String {
    let mut output = String::new();

    output.push_str("# Lattice Workflow Context\n\n");
    output.push_str("> **Context Recovery**: Run `lat prime` after compaction or new session\n\n");

    output.push_str("## Session Protocol\n\n");
    output.push_str("Before completing work, run this checklist:\n\n");

    for (i, item) in context.config.prime.checklist.iter().enumerate() {
        output.push_str(&format!("[ ] {}. {}\n", i + 1, item));
    }

    output.push_str("\n## Core Commands\n\n");
    output.push_str("- `lat overview` - See most critical documents\n");
    output.push_str("- `lat ready` - Show tasks ready to work\n");
    output.push_str(
        "- `lat show <id>` - View document/task details (includes parent, dependencies, related)\n",
    );
    output.push_str("- `lat claim <id>` - Claim task for local work\n");
    output.push_str("- `lat close <id>` - Mark task completed\n");

    output.push_str("\n## Link Authoring\n\n");
    output.push_str("Always write links in shorthand format using just the Lattice ID:\n\n");
    output.push_str("    See [the design doc](LXXXXX) for details.\n\n");
    output.push_str(
        "Running `lat fmt` at the end of work will expand to full path+fragment format:\n\n",
    );
    output.push_str("    See [the design doc](../path/to/doc.md#LXXXXX) for details.\n\n");
    output.push_str("This avoids needing to look up file paths when authoring documents.\n");

    if full {
        output.push_str("\n## Additional Commands\n\n");
        output.push_str("### Query Commands\n");
        output.push_str("- `lat list [options]` - Search and filter documents\n");
        output.push_str("- `lat search <query>` - Full-text search across content\n");
        output.push_str("- `lat blocked` - Show tasks with unresolved blockers\n");
        output.push_str("- `lat stale` - Find tasks not updated recently\n");

        output.push_str("\n### Task Management\n");
        output.push_str(
            "- `lat create <parent> \"<description>\" [options]` - Create new document\n",
        );
        output.push_str("- `lat update <id> [options]` - Modify existing tasks\n");
        output.push_str("- `lat reopen <id>` - Reopen closed tasks\n");

        output.push_str("\n### Hierarchy Commands\n");
        output.push_str("- `lat tree [path]` - Display directory structure with documents\n");
        output.push_str("- `lat roots` - List all root documents\n");
        output.push_str("- `lat children <id>` - List documents under a root\n");

        output.push_str("\n### Relationship Commands\n");
        output.push_str("- `lat links-from <id>` - Show outgoing links\n");
        output.push_str("- `lat links-to <id>` - Show incoming links (backlinks)\n");
        output.push_str("- `lat dep tree <id>` - Display dependency tree\n");

        output.push_str("\n### Maintenance Commands\n");
        output.push_str("- `lat check` - Validate documents and repository\n");
        output.push_str("- `lat fmt` - Format documents and normalize links\n");
    }

    output
}

fn export_to_file(path: &str, content: &str) -> LatticeResult<()> {
    let file_path = Path::new(path);

    if let Some(parent) = file_path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        fs::create_dir_all(parent).map_err(|e| LatticeError::WriteError {
            path: parent.to_path_buf(),
            reason: format!("Failed to create directory: {e}"),
        })?;
    }

    let mut file = File::create(file_path).map_err(|e| LatticeError::WriteError {
        path: file_path.to_path_buf(),
        reason: format!("Failed to create file: {e}"),
    })?;

    file.write_all(content.as_bytes()).map_err(|e| LatticeError::WriteError {
        path: file_path.to_path_buf(),
        reason: format!("Failed to write file: {e}"),
    })?;

    Ok(())
}

fn print_json_output(context: &CommandContext) {
    let output = PrimeOutput {
        session_protocol: context.config.prime.checklist.clone(),
        core_commands: vec![
            CoreCommand {
                command: "lat overview".to_string(),
                description: "See most critical documents".to_string(),
            },
            CoreCommand {
                command: "lat ready".to_string(),
                description: "Show tasks ready to work".to_string(),
            },
            CoreCommand {
                command: "lat show <id>".to_string(),
                description: "View document/task details".to_string(),
            },
            CoreCommand {
                command: "lat claim <id>".to_string(),
                description: "Claim task for local work".to_string(),
            },
            CoreCommand {
                command: "lat close <id>".to_string(),
                description: "Mark task completed".to_string(),
            },
        ],
        link_authoring: LinkAuthoringGuide {
            shorthand_example: "See [the design doc](LXXXXX) for details.".to_string(),
            expanded_example: "See [the design doc](../path/to/doc.md#LXXXXX) for details."
                .to_string(),
            instruction: "Write links using shorthand format; lat fmt will expand them."
                .to_string(),
        },
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()));
}
