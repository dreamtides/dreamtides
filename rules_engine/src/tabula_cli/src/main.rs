use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
    },

    #[command(about = "Restore images from URLs")]
    RebuildImages {
        #[arg(help = "Path to the XLSM file")]
        xlsm_path: Option<PathBuf>,
    },

    #[command(about = "Configure Git for the tabula workflow")]
    GitSetup,
}

#[expect(clippy::print_stdout)]
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::BuildToml { xlsm_path, output_dir } => {
            println!(
                "build-toml: xlsm_path={xlsm_path:?}, output_dir={output_dir:?} (not yet implemented)"
            );
        }
        Commands::BuildXls { dry_run, toml_dir, xlsm_path } => {
            println!(
                "build-xls: dry_run={dry_run}, toml_dir={toml_dir:?}, xlsm_path={xlsm_path:?} (not yet implemented)"
            );
        }
        Commands::Validate { applescript, strip_images, toml_dir } => {
            println!(
                "validate: applescript={applescript}, strip_images={strip_images}, toml_dir={toml_dir:?} (not yet implemented)"
            );
        }
        Commands::StripImages { xlsm_path } => {
            println!("strip-images: xlsm_path={xlsm_path:?} (not yet implemented)");
        }
        Commands::RebuildImages { xlsm_path } => {
            println!("rebuild-images: xlsm_path={xlsm_path:?} (not yet implemented)");
        }
        Commands::GitSetup => {
            println!("git-setup (not yet implemented)");
        }
    }
}
