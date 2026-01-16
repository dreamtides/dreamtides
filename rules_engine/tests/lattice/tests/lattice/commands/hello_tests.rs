use lattice::cli::hello_command;

#[test]
fn test_hello_world() {
    assert_eq!(hello_command::hello_world(), "Hello from Lattice!");
}
