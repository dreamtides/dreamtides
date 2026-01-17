use lattice::cli::argument_parser;

fn main() {
    human_panic::setup_panic!();
    let _args = argument_parser::parse();
    println!("Lattice: command dispatch not yet implemented");
}
