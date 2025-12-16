use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use tabula_cli::commands::{build_toml, strip_images};

#[derive(Parser)]
#[command(name = "tabula")]
#[command(version, about = "Manage Excel spreadsheets in a Git-friendly way")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Convert Excel tables to TOML files")]
    BuildToml {
        #[arg(help = "Path to the XLSM file")]
        xlsm_path: Option<PathBuf>,

        #[arg(help = "Output directory for TOML files")]
        output_dir: Option<PathBuf>,
    },

    #[command(about = "Update Excel from TOML files")]
    BuildXls {
        #[arg(long, help = "Perform a dry run without writing changes")]
        dry_run: bool,

        #[arg(help = "Directory containing TOML files")]
        toml_dir: Option<PathBuf>,

        #[arg(help = "Path to the XLSM file")]
        xlsm_path: Option<PathBuf>,
    },

    #[command(about = "Validate round-trip conversion")]
    Validate {
        #[arg(long, help = "Use AppleScript to verify Excel can open the file")]
        applescript: bool,

        #[arg(long, help = "Include image stripping in validation")]
        strip_images: bool,

        #[arg(help = "Directory containing TOML files")]
        toml_dir: Option<PathBuf>,
    },

    #[command(about = "Replace images with placeholders")]
    StripImages {
        #[arg(help = "Path to the XLSM file")]
        xlsm_path: Option<PathBuf>,

        #[arg(long, help = "Path for the stripped XLSM output")]
        output_path: Option<PathBuf>,
    },

    #[command(about = "Restore images from URLs")]
    RebuildImages {
        #[arg(help = "Path to the XLSM file")]
        xlsm_path: Option<PathBuf>,
    },

    #[command(about = "Configure Git for the tabula workflow")]
    GitSetup,
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
        Commands::BuildToml { xlsm_path, output_dir } => {
            build_toml::build_toml(xlsm_path, output_dir)?;
        }
        Commands::BuildXls { dry_run, toml_dir, xlsm_path } => bail!(
            "build-xls not yet implemented: dry_run={dry_run}, toml_dir={toml_dir:?}, xlsm_path={xlsm_path:?}"
        ),
        Commands::Validate { applescript, strip_images, toml_dir } => bail!(
            "validate not yet implemented: applescript={applescript}, strip_images={strip_images}, toml_dir={toml_dir:?}"
        ),
        Commands::StripImages { xlsm_path, output_path } => {
            strip_images::strip_images(xlsm_path, output_path)?;
        }
        Commands::RebuildImages { xlsm_path } => {
            bail!("rebuild-images not yet implemented: xlsm_path={xlsm_path:?}")
        }
        Commands::GitSetup => bail!("git-setup not yet implemented"),
    }

    Ok(())
}
