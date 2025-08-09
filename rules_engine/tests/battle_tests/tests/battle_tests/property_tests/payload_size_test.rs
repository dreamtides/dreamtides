use battle_tests::payload_size;

#[test]
fn payload_size_under_limit() {
    let json_string = payload_size::generate_payload_json(false);

    let size_bytes = json_string.len();
    const MAX_SIZE_BYTES: usize = 2_500_000;

    assert!(
        size_bytes < MAX_SIZE_BYTES,
        "Payload size {size_bytes} bytes exceeds limit of {MAX_SIZE_BYTES} bytes"
    );

    println!("Payload size: {size_bytes} bytes (under {MAX_SIZE_BYTES} byte limit)");
}
