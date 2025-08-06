use battle_tests::payload_size;

#[test]
fn payload_size_under_limit() {
    let json_string = payload_size::generate_payload_json(false);

    let size_bytes = json_string.len();
    const MAX_SIZE_BYTES: usize = 2_500_000;

    assert!(
        size_bytes < MAX_SIZE_BYTES,
        "Payload size {} bytes exceeds limit of {} bytes",
        size_bytes,
        MAX_SIZE_BYTES
    );

    println!("Payload size: {} bytes (under {} byte limit)", size_bytes, MAX_SIZE_BYTES);
}
