use std::fs::File;
use std::io::BufReader;

use anyhow::{Context, Result};
use clap::Parser;
use google_sheets4::Sheets;
use google_sheets4::api::ValueRange;
use google_sheets4::yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use yup_oauth2::hyper_rustls::HttpsConnectorBuilder;

#[derive(Parser, Debug)]
#[command(name = "tabula", version, about = "Google Sheets reader via Service Account")]
pub struct Args {
    #[arg(long, value_name = "PATH", help = "Path to service account private_key.json")]
    key_file: String,
    #[arg(long, value_name = "SPREADSHEET_ID", help = "Google Sheets spreadsheet ID")]
    spreadsheet_id: String,
    #[arg(long, value_name = "A1_RANGE", help = "Cell range in A1 notation, e.g. Sheet1!A1")]
    cell_range: String,
    #[arg(
        long,
        value_name = "VALUE",
        help = "If set, writes this value into the cell range instead of reading"
    )]
    write_value: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let file = File::open(&args.key_file)
        .with_context(|| format!("failed to open key file at {}", args.key_file))?;
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
        .map_err(|e| anyhow::anyhow!("failed to load native TLS roots: {}", e))?;
    let https = builder.https_or_http().enable_http1().enable_http2().build();
    let client = Client::builder(TokioExecutor::new()).build(https);

    let hub = Sheets::new(client, auth);

    if let Some(write_value) = args.write_value.as_ref() {
        let value_range = ValueRange {
            values: Some(vec![vec![serde_json::Value::String(write_value.clone())]]),
            ..Default::default()
        };
        let _ = hub
            .spreadsheets()
            .values_update(value_range, &args.spreadsheet_id, &args.cell_range)
            .value_input_option("RAW")
            .add_scope("https://www.googleapis.com/auth/spreadsheets")
            .doit()
            .await
            .with_context(|| {
                format!(
                    "failed to write value to range {} in spreadsheet {}",
                    args.cell_range, args.spreadsheet_id
                )
            })?;
        println!("{write_value}");
        return Ok(());
    }

    let result = hub
        .spreadsheets()
        .values_get(&args.spreadsheet_id, &args.cell_range)
        .add_scope("https://www.googleapis.com/auth/spreadsheets.readonly")
        .doit()
        .await
        .with_context(|| {
            format!(
                "failed to read range {} from spreadsheet {}",
                args.cell_range, args.spreadsheet_id
            )
        })?;

    let value_range = result.1;
    let Some(values) = value_range.values else {
        println!("No data found in the specified range");
        return Ok(());
    };
    let Some(first_row) = values.first() else {
        println!("No data found in the specified range");
        return Ok(());
    };
    let Some(first_cell) = first_row.first() else {
        println!("Cell is empty");
        return Ok(());
    };
    println!("{first_cell}");

    Ok(())
}
