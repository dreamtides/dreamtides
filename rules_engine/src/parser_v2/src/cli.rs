use clap::Parser;

#[derive(Parser)]
#[command(name = "parser", about = "Dreamtides card ability parser")]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
    println!("Parser V2 CLI - Not yet implemented");
}
