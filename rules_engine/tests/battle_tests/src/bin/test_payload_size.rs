use std::fs;
use std::path::PathBuf;

use battle_tests::payload_size;
use clap::Parser;

#[derive(Parser)]
#[command(name = "test_payload_size")]
#[command(about = "Generate a large battle payload for testing")]
struct Args {
    /// Path to write the JSON output
    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();
    let json_string = payload_size::generate_payload_json(true);
    fs::write(&args.output, json_string).expect("Failed to write JSON file");
    println!("JSON written to: {}", args.output.display());
}
