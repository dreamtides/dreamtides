use llmc::ipc::socket;

#[test]
fn remediation_socket_path_is_different_from_main_socket() {
    let main_socket = socket::get_socket_path();
    let remediation_socket = socket::get_remediation_socket_path();
    assert_ne!(
        main_socket,
        remediation_socket,
        "Remediation socket path should be different from main daemon socket path. \
         Main: {}, Remediation: {}",
        main_socket.display(),
        remediation_socket.display()
    );
}

#[test]
fn remediation_socket_uses_remediation_suffix() {
    let path = socket::get_remediation_socket_path();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.ends_with("llmc_remediation.sock"),
        "Remediation socket should use 'llmc_remediation.sock' filename. Got: {}",
        path_str
    );
}

#[test]
fn main_socket_uses_llmc_sock_suffix() {
    let path = socket::get_socket_path();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.ends_with("llmc.sock"),
        "Main socket should use 'llmc.sock' filename. Got: {}",
        path_str
    );
}

#[test]
fn both_sockets_in_same_directory() {
    let main_socket = socket::get_socket_path();
    let remediation_socket = socket::get_remediation_socket_path();
    assert_eq!(
        main_socket.parent(),
        remediation_socket.parent(),
        "Both sockets should be in the same directory (LLMC_ROOT). \
         Main parent: {:?}, Remediation parent: {:?}",
        main_socket.parent(),
        remediation_socket.parent()
    );
}
