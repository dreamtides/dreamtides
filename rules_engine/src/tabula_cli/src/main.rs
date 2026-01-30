use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tabula_cli::commands::{check, generate, watch};

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
    #[command(about = "Watch source files and regenerate on changes")]
    Watch {
        #[arg(help = "Output directory for generated files (default: src/tabula_generated/src/)")]
        output_dir: Option<PathBuf>,
    },
    #[command(about = "Check that generated files are up to date")]
    Check,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { output_dir } => {
            generate::generate(output_dir)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Watch { output_dir } => {
            watch::watch(output_dir)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Check => check::check(),
    }
}
