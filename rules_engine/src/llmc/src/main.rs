mod cli;
mod config;
mod nouns;
mod setup;
mod start;
mod state;
mod status;

use std::process;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Commands};

fn main() {
    if let Err(err) = self::run() {
        eprintln!("{err:#}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup(args) => {
            setup::run(&args, cli.repo.as_deref())?;
        }
        Commands::Start(args) => {
            start::run(&args, cli.repo.as_deref())?;
        }
        Commands::Status(args) => {
            status::run(&args, cli.repo.as_deref())?;
        }
        Commands::Rebase { .. } => anyhow::bail!("llmc rebase is not implemented yet"),
        Commands::Review { .. } => anyhow::bail!("llmc review is not implemented yet"),
        Commands::Reject { .. } => anyhow::bail!("llmc reject is not implemented yet"),
        Commands::Accept { .. } => anyhow::bail!("llmc accept is not implemented yet"),
    }

    Ok(())
}
