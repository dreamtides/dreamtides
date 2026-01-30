use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tabula_cli::commands::generate;

#[derive(Parser)]
#[command(name = "tabula")]
#[command(version, about = "Tabula CLI for code generation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Generate Rust code from TOML/FTL source files")]
    Generate {
        #[arg(help = "Output directory for generated files (default: src/tabula_generated/src/)")]
        output_dir: Option<PathBuf>,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { output_dir } => generate::generate(output_dir)?,
    }

    Ok(())
}
