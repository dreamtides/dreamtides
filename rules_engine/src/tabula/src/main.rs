use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use google_sheets4::{Sheets, hyper_rustls, hyper_util, yup_oauth2};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;

#[derive(Parser)]
#[command(name = "tabula")]
#[command(
    about = "A tool for reading and writing Google Sheets via Service Account authentication"
)]
struct Args {
    #[arg(long, help = "Path to the service account private key JSON file")]
    service_account_key: PathBuf,

    #[arg(long, help = "Google Sheets spreadsheet ID")]
    spreadsheet_id: String,

    #[arg(long, help = "Cell reference to read (e.g., A1, B2, Sheet1!C3)")]
    cell: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let secret = yup_oauth2::read_service_account_key(&args.service_account_key)
        .await
        .with_context(|| {
            format!(
                "Failed to read service account key from '{}'. \
                 Please ensure the file exists and contains valid JSON with the following structure: \
                 {{ \"type\": \"service_account\", \"project_id\": \"...\", \"private_key\": \"...\", \"client_email\": \"...\" }}. \
                 You can download this file from Google Cloud Console > IAM & Admin > Service Accounts.",
                args.service_account_key.display()
            )
        })?;

    let auth = yup_oauth2::ServiceAccountAuthenticator::builder(secret)
        .build()
        .await
        .with_context(|| {
            "Failed to create Service Account authenticator. \
             This could indicate invalid credentials in the service key file, \
             network connectivity issues, or problems with Google's OAuth servers. \
             Verify that the service account has the necessary permissions and \
             that the Google Sheets API is enabled in your Google Cloud project."
        })?;

    let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build(
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .with_context(|| {
                "Failed to load native root certificates for HTTPS connections. \
                 This may indicate a problem with your system's certificate store."
            })?
            .https_or_http()
            .enable_http1()
            .build(),
    );

    let hub = Sheets::new(client, auth);

    let value = read_cell(&hub, &args.spreadsheet_id, &args.cell).await.with_context(|| {
        format!(
            "Failed to read cell '{}' from spreadsheet '{}'. \
                 This could be due to: \
                 1) The spreadsheet doesn't exist or the ID is incorrect, \
                 2) The service account doesn't have access to this spreadsheet \
                    (share the sheet with the service account email), \
                 3) The Google Sheets API is not enabled in your project, \
                 4) Network connectivity issues, \
                 5) Invalid cell reference format (use A1, B2, Sheet1!C3, etc.)",
            args.cell, args.spreadsheet_id
        )
    })?;

    println!("Cell {} contains: {}", args.cell, value);

    Ok(())
}

async fn read_cell(
    hub: &Sheets<hyper_rustls::HttpsConnector<HttpConnector>>,
    spreadsheet_id: &str,
    cell_range: &str,
) -> Result<String> {
    let result = hub.spreadsheets().values_get(spreadsheet_id, cell_range).doit().await?;

    let Some(values) = result.1.values else {
        return Ok("(empty)".to_string());
    };

    let Some(first_row) = values.first() else {
        return Ok("(empty)".to_string());
    };

    let Some(first_cell) = first_row.first() else {
        return Ok("(empty)".to_string());
    };

    Ok(first_cell.as_str().unwrap_or("").to_string())
}
