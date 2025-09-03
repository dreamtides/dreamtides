use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;

use anyhow::{Context, Result};
use clap::Parser;
use google_sheets4::Sheets;
use google_sheets4::yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use tabula_cli::google_sheet::GoogleSheet;
use tabula_cli::missing_cards_table::write_missing_cards_html;
use tabula_cli::spreadsheet::Spreadsheet;
use tabula_cli::{ability_parsing, tabula_codegen, tabula_sync};
use tabula_data::localized_strings::LanguageId;
use tabula_data::tabula::{self, TabulaBuildContext, TabulaRaw};
use yup_oauth2::hyper_rustls::HttpsConnectorBuilder;

#[derive(Parser, Debug)]
#[command(name = "tabula", version, about = "Google Sheets reader via Service Account")]
pub struct Args {
    #[arg(long, value_name = "PATH", help = "Path to service account private_key.json")]
    key_file: String,
    #[arg(long, value_name = "SPREADSHEET_ID", help = "Google Sheets spreadsheet ID")]
    spreadsheet_id: String,
    #[arg(long, value_name = "PATH", help = "Generate string_id.rs at the given path")]
    string_ids: Option<String>,
    #[arg(long, value_name = "PATH", help = "Generate test_card_id.rs at the given path")]
    test_card_ids: Option<String>,
    #[arg(long, value_name = "PATH", help = "Generate card_lists.rs at the given path")]
    card_lists: Option<String>,
    #[arg(
        long,
        value_name = "PATH",
        help = "Write Tabula as pretty-printed JSON to the given path"
    )]
    write_json: Option<String>,
}

#[expect(clippy::print_stderr)]
#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {err}");
        for cause in err.chain().skip(1) {
            eprintln!("Caused by: {cause}");
        }
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
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

    println!("Sending Google Sheets request");
    let tables = spreadsheet.read_all_tables().await?;
    println!("Got Google Sheets response");

    let mut tabula_raw = tabula_sync::sync(tables)?;
    ability_parsing::parse_all_abilities_for_raw_tabula(&mut tabula_raw)?;

    let _ = tabula::build(
        &TabulaBuildContext { current_language: LanguageId::EnglishUnitedStates },
        &tabula_raw,
    )
    .map_err(|errs| {
        anyhow::anyhow!(errs.into_iter().map(|e| e.format()).collect::<Vec<_>>().join("\n"))
    })?;

    if let Some(path) = args.write_json.as_deref() {
        if let Ok(local_file) = File::open(path) {
            let local_raw: TabulaRaw = serde_json::from_reader(BufReader::new(local_file))
                .with_context(|| format!("failed to parse local JSON at {path}"))?;
            let local_ids: HashSet<_> = local_raw.cards.0.iter().map(|r| r.id).collect();
            let remote_ids: HashSet<_> = tabula_raw.cards.0.iter().map(|r| r.id).collect();
            let missing_ids: Vec<_> = local_ids.difference(&remote_ids).cloned().collect();
            if !missing_ids.is_empty() {
                let context =
                    TabulaBuildContext { current_language: LanguageId::EnglishUnitedStates };
                let tabula = tabula::build(&context, &local_raw).map_err(|errs| {
                    anyhow::anyhow!(
                        errs.into_iter().map(|e| e.format()).collect::<Vec<_>>().join("\n")
                    )
                })?;
                let mut rows = Vec::new();
                for id in missing_ids.iter() {
                    if let Some(def) = tabula.cards.get(id) {
                        rows.push((*id, def.clone()));
                    }
                }
                let out_path = std::path::Path::new("missing_cards.html");
                write_missing_cards_html(out_path, &rows)
                    .context("failed to write missing_cards.html in project root")?;
                anyhow::bail!(
                    "local test card IDs missing from Google Sheets: {}. Wrote missing_cards.html with rows to paste into Sheets.",
                    missing_ids.iter().map(|id| id.0.to_string()).collect::<Vec<_>>().join(", ")
                );
            }
        }
    }

    if let Some(path) = args.string_ids.as_deref() {
        tabula_codegen::generate_string_ids(&tabula_raw, path)?;
    }

    if let Some(path) = args.test_card_ids.as_deref() {
        tabula_codegen::generate_test_card_ids(&tabula_raw, path)?;
    }

    if let Some(path) = args.write_json.as_deref() {
        serde_json::to_writer_pretty(
            File::create(path)
                .with_context(|| format!("failed to create JSON output file at {path}"))?,
            &tabula_raw,
        )
        .context("failed to serialize Tabula to JSON")?;
    }

    if let Some(path) = args.card_lists.as_deref() {
        tabula_codegen::generate_card_lists(&tabula_raw, path)?;
    }

    Ok(())
}
