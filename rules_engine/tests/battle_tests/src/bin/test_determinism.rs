use battle_tests::determinism::run_determinism_test;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "test_determinism")]
#[command(about = "Test determinism of battle actions", long_about = None)]
struct Args {
    /// Random seed for the test
    #[arg(short, long, default_value_t = 42)]
    seed: u64,

    /// Number of runs to validate
    #[arg(short = 'r', long, default_value_t = 3)]
    num_runs: usize,

    /// Maximum actions per run
    #[arg(short = 'a', long, default_value_t = 100)]
    actions_per_run: usize,
}

fn main() {
    let args = Args::parse();

    println!("Testing battle determinism:");
    println!("  Seed: {}", args.seed);
    println!("  Runs: {}", args.num_runs);
    println!("  Max actions per run: {}", args.actions_per_run);
    println!();

    let result = run_determinism_test(args.seed, args.num_runs, args.actions_per_run);

    if result.success {
        println!("✓ Success!");
        println!(
            "  Validated {} runs with {} actions each were deterministic",
            result.num_runs, result.actions_executed
        );
    } else {
        println!("✗ Failed!");
        if let Some(error) = result.error_message {
            println!("  Error: {}", error);
        }
        println!("  Executed {} actions before failure", result.actions_executed);
        std::process::exit(1);
    }
}
