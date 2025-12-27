use std::net::IpAddr;
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tabula_cli::commands::git_setup::Hook;
use tabula_cli::commands::rebuild_images::rebuild;
use tabula_cli::commands::validate::runner;
use tabula_cli::commands::{
    build_toml, build_xls, git_setup, repair, server, server_install, strip_images,
};

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

        #[arg(
            long,
            required = true,
            help = "Path for the XLSM output (pass the input path to overwrite in place)"
        )]
        output_path: Option<PathBuf>,

        #[arg(help = "Directory containing TOML files")]
        toml_dir: Option<PathBuf>,

        #[arg(help = "Path to the XLSM file")]
        xlsm_path: Option<PathBuf>,
    },

    #[command(about = "Validate round-trip conversion")]
    Validate {
        #[arg(long, help = "Include image stripping in validation")]
        strip_images: bool,

        #[arg(long, help = "Report all validation problems instead of stopping at the first")]
        all: bool,

        #[arg(long, help = "Show surrounding XML lines for file differences")]
        verbose: bool,

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

        #[arg(
            long,
            help = "Download images from IMAGE() formulas instead of using the .git cache"
        )]
        from_urls: bool,

        #[arg(
            long,
            help = "Restore from cache and fall back to downloading IMAGE() URLs on failure"
        )]
        auto: bool,
    },

    #[command(about = "Configure Git for the tabula workflow")]
    GitSetup,

    #[command(about = "Install the AppleScriptTask helper for tabula server")]
    ServerInstall,

    #[command(about = "Run the tabula server")]
    Server {
        #[arg(long, default_value = "127.0.0.1", help = "Host address to bind")]
        host: IpAddr,

        #[arg(long, default_value_t = 3030, help = "Port to listen on")]
        port: u16,

        #[arg(long, default_value_t = 1_048_576, help = "Maximum request payload size in bytes")]
        max_payload_bytes: usize,

        #[arg(long, help = "Handle a single request then exit")]
        once: bool,
    },

    #[command(hide = true)]
    GitHook {
        #[arg(value_enum)]
        hook: Hook,
    },

    #[command(about = "Repair XLSM CRC errors")]
    Repair {
        #[arg(help = "Path to the XLSM file")]
        xlsm_path: Option<PathBuf>,

        #[arg(long, help = "Rebuild IMAGE() entries and cache after fixing CRCs")]
        rebuild_images: bool,
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
        Commands::BuildToml { xlsm_path, output_dir } => {
            build_toml::build_toml(xlsm_path, output_dir)?;
        }
        Commands::BuildXls { dry_run, toml_dir, xlsm_path, output_path } => {
            build_xls::build_xls(dry_run, toml_dir, xlsm_path, output_path)?;
        }
        Commands::Validate { strip_images, all, verbose, toml_dir } => {
            runner::validate(
                runner::ValidateConfig { strip_images, report_all: all, verbose },
                toml_dir,
                None,
            )?;
        }
        Commands::StripImages { xlsm_path, output_path } => {
            strip_images::strip_images(xlsm_path, output_path)?;
        }
        Commands::RebuildImages { xlsm_path, from_urls, auto } => {
            rebuild::rebuild_images(xlsm_path, from_urls, auto)?;
        }
        Commands::GitSetup => git_setup::git_setup()?,
        Commands::ServerInstall => server_install::server_install()?,
        Commands::Server { host, port, max_payload_bytes, once } => {
            server::server(host, port, max_payload_bytes, once)?;
        }
        Commands::GitHook { hook } => git_setup::run_hook(hook)?,
        Commands::Repair { xlsm_path, rebuild_images } => {
            repair::repair(xlsm_path, rebuild_images)?;
        }
    }

    Ok(())
}
