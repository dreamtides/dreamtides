use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::state::Runtime;

#[derive(Parser)]
#[command(name = "llmc")]
#[command(about = "Coordinate multiple CLI agents over git worktrees")]
pub struct Cli {
    #[arg(long, help = "Override repo root detection")]
    pub repo: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Setup(SetupArgs),
    Start(StartArgs),
    Status(StatusArgs),
    Rebase(AgentArgs),
    Review(ReviewArgs),
    Delete(AgentArgs),
    Clean(CleanArgs),
    Reject(RejectArgs),
    Accept(AcceptArgs),
}

#[derive(Args)]
pub struct SetupArgs {
    #[arg(long, value_name = "PATH", help = "Source checkout for cloning")]
    pub source: Option<PathBuf>,

    #[arg(long, value_name = "PATH", help = "Target checkout path")]
    pub target: Option<PathBuf>,
}

#[derive(Args)]
pub struct StartArgs {
    #[arg(long, help = "Agent identifier")]
    pub agent: Option<String>,

    #[arg(long, value_enum, default_value = "codex", help = "Runtime to use")]
    pub runtime: Option<Runtime>,

    #[arg(long, help = "Prompt text for the agent")]
    pub prompt: Option<String>,

    #[arg(long, value_name = "PATH", help = "Prompt file to append")]
    pub prompt_file: Vec<PathBuf>,

    #[arg(value_name = "PATH", help = "Prompt file to append")]
    pub prompt_file_pos: Option<PathBuf>,

    #[arg(long, help = "Run without streaming output")]
    pub background: bool,

    #[arg(long, help = "Disable notification when complete")]
    pub no_notify: bool,

    #[arg(long, help = "Write logs to .llmc/logs")]
    pub log: bool,
}

#[derive(Args)]
pub struct AgentArgs {
    #[arg(long, help = "Agent identifier")]
    pub agent: Option<String>,
}

#[derive(Args)]
pub struct AcceptArgs {
    #[arg(long, help = "Agent identifier")]
    pub agent: Option<String>,

    #[arg(long, help = "Do not rebase the commit onto the dreamtides master branch")]
    pub nopull: bool,
}

#[derive(Args)]
pub struct StatusArgs {
    #[arg(long, help = "Agent identifier")]
    pub agent: Option<String>,
}

#[derive(Args)]
pub struct CleanArgs {}

#[derive(ValueEnum, Clone, Debug)]
pub enum ReviewInterface {
    Diff,
    Difftastic,
    Vscode,
    Forgejo,
}

#[derive(Args)]
pub struct ReviewArgs {
    #[arg(long, help = "Agent identifier")]
    pub agent: Option<String>,

    #[arg(long, value_enum, default_value = "difftastic", help = "Review interface")]
    pub interface: ReviewInterface,
}

#[derive(Args)]
pub struct RejectArgs {
    #[arg(long, help = "Agent identifier")]
    pub agent: Option<String>,

    #[arg(long, help = "Reviewer notes to append")]
    pub notes: Option<String>,

    #[arg(long, value_name = "PATH", help = "Notes file to append")]
    pub notes_file: Option<PathBuf>,

    #[arg(value_name = "PATH", help = "Notes file to append")]
    pub notes_file_pos: Option<PathBuf>,

    #[arg(long, help = "Include the original user prompt context")]
    pub include_prompt: bool,
}
