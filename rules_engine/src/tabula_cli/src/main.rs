use std::fs::File;
use std::io::BufReader;

use anyhow::{Context, Result};
use clap::Parser;
use google_sheets4::Sheets;
use google_sheets4::yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use tabula_cli::google_sheet::GoogleSheet;
use tabula_cli::spreadsheet::Spreadsheet;
use tabula_cli::{tabula_codegen, tabula_sync};
use tabula_data::localized_strings::LanguageId;
use tabula_data::tabula::{self, TabulaBuildContext};
use yup_oauth2::hyper_rustls::HttpsConnectorBuilder;

#[derive(Parser, Debug)]
#[command(name = "tabula", version, about = "Google Sheets reader via Service Account")]
pub struct Args {
    #[arg(long, value_name = "PATH", help = "Path to service account private_key.json")]
    key_file: String,
    #[arg(long, value_name = "SPREADSHEET_ID", help = "Google Sheets spreadsheet ID")]
    spreadsheet_id: String,
    #[arg(long, value_name = "PATH", help = "Generate string_id.rs at the given path and exit")]
    string_ids: Option<String>,
    #[arg(
        long,
        value_name = "PATH",
        help = "Write Tabula as pretty-printed JSON to the given path"
    )]
    write_json: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let file = File::open(&args.key_file)
        .with_context(|| format!("failed to open key file at {path}", path = args.key_file))?;
    let reader = BufReader::new(file);
    let sa_key: ServiceAccountKey =
        serde_json::from_reader(reader).context("failed to parse service account key JSON")?;

    let auth = ServiceAccountAuthenticator::builder(sa_key)
        .build()
        .await
        .context("failed to build service account authenticator")?;

    let builder = HttpsConnectorBuilder::new();
    let builder = builder
        .with_native_roots()
        .map_err(|e| anyhow::anyhow!("failed to load native TLS roots: {e}"))?;
    let https = builder.https_or_http().enable_http1().enable_http2().build();
    let client = Client::builder(TokioExecutor::new()).build(https);

    let hub = Sheets::new(client, auth);
    let spreadsheet = GoogleSheet::new(args.spreadsheet_id, hub);
    let tables = spreadsheet.read_all_tables().await?;
    let tabula_raw = tabula_sync::sync(tables)?;
    let _tabula = tabula::build(
        &TabulaBuildContext { current_language: LanguageId::EnglishUnitedStates },
        &tabula_raw,
    );

    if let Some(path) = args.string_ids.as_deref() {
        tabula_codegen::generate_string_ids(&tabula_raw, path)?;
    }

    if let Some(path) = args.write_json.as_deref() {
        serde_json::to_writer_pretty(
            File::create(path)
                .with_context(|| format!("failed to create JSON output file at {path}"))?,
            &tabula_raw,
        )
        .context("failed to serialize Tabula to JSON")?;
    }

    Ok(())
}
