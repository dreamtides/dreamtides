pub mod google_sheet;
pub mod spreadsheet;

use std::fs::File;
use std::io::BufReader;

use anyhow::{Context, Result, bail};
use clap::Parser;
use google_sheet::GoogleSheet;
use google_sheets4::Sheets;
use google_sheets4::yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use yup_oauth2::hyper_rustls::HttpsConnectorBuilder;

use crate::spreadsheet::Spreadsheet;

#[derive(Parser, Debug)]
#[command(name = "tabula", version, about = "Google Sheets reader via Service Account")]
pub struct Args {
    #[arg(long, value_name = "PATH", help = "Path to service account private_key.json")]
    key_file: String,
    #[arg(long, value_name = "SPREADSHEET_ID", help = "Google Sheets spreadsheet ID")]
    spreadsheet_id: String,
    #[arg(long, value_name = "SHEET", help = "Sheet name within the spreadsheet")]
    sheet: String,
    #[arg(long, value_name = "COLUMN", help = "Column name, e.g. A or AB")]
    column: Option<String>,
    #[arg(long, value_name = "ROW", help = "Row number starting at 1")]
    row: Option<u32>,
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
    let spreadsheet = GoogleSheet::new(args.spreadsheet_id.clone(), hub);

    if let Some(write_value) = args.write_value.as_ref() {
        let Some(column) = args.column.as_ref() else {
            bail!("--column is required when --write-value is provided")
        };
        let Some(row) = args.row else { bail!("--row is required when --write-value is provided") };
        spreadsheet.write_cell(&args.sheet, column, row, write_value).await.with_context(|| {
            format!(
                "failed to write value to sheet {} in spreadsheet {}",
                args.sheet, args.spreadsheet_id
            )
        })?;
        println!("{write_value}");
        return Ok(());
    }

    match (&args.column, args.row) {
        (Some(column), Some(row)) => {
            let cell =
                spreadsheet.read_cell(&args.sheet, column, row).await.with_context(|| {
                    format!(
                        "failed to read sheet {} from spreadsheet {}",
                        args.sheet, args.spreadsheet_id
                    )
                })?;
            match cell {
                Some(s) if !s.is_empty() => println!("{s}"),
                _ => println!("Cell is empty"),
            }
        }
        _ => {
            let table = spreadsheet.read_table(&args.sheet).await.with_context(|| {
                format!(
                    "failed to read sheet {} from spreadsheet {}",
                    args.sheet, args.spreadsheet_id
                )
            })?;
            let max_rows = table.columns.iter().map(|c| c.values.len()).max().unwrap_or(0);
            let mut rows: Vec<serde_json::Map<String, serde_json::Value>> =
                Vec::with_capacity(max_rows);
            for i in 0..max_rows {
                let mut obj = serde_json::Map::new();
                for col in table.columns.iter() {
                    let v = col.values.get(i).map(|sv| sv.data.clone()).unwrap_or_default();
                    obj.insert(col.name.clone(), serde_json::Value::String(v));
                }
                rows.push(obj);
            }
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
    }

    Ok(())
}
