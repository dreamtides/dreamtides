use std::collections::HashSet;

use lattice::id::client_selection;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[test]
fn required_length_scales_with_clients() {
    assert_eq!(client_selection::required_length(0), 3);
    assert_eq!(client_selection::required_length(16), 3);
    assert_eq!(client_selection::required_length(17), 4);
    assert_eq!(client_selection::required_length(64), 4);
    assert_eq!(client_selection::required_length(65), 5);
    assert_eq!(client_selection::required_length(256), 5);
    assert_eq!(client_selection::required_length(257), 6);
    assert_eq!(client_selection::required_length(1000), 6);
}

#[test]
fn generate_unique_client_id() {
    let mut rng = rand::rng();
    let existing = HashSet::new();

    let id = client_selection::generate_client_id(&existing, &mut rng);
    assert_eq!(id.len(), 3);
    assert!(client_selection::is_valid_client_id(&id));
}

#[test]
fn generate_avoids_collisions() {
    let mut rng = StdRng::seed_from_u64(12345);

    let mut existing = HashSet::new();
    for _ in 0..100 {
        let id = client_selection::generate_client_id(&existing, &mut rng);
        assert!(!existing.contains(&id), "Generated duplicate client ID: {}", id);
        existing.insert(id);
    }
}

#[test]
fn length_increases_with_client_count() {
    let mut rng = StdRng::seed_from_u64(54321);

    // Generate 17 client IDs to trigger length increase
    let mut existing = HashSet::new();
    for _ in 0..17 {
        let id = client_selection::generate_client_id(&existing, &mut rng);
        existing.insert(id);
    }

    // Next ID should be 4 characters
    let next_id = client_selection::generate_client_id(&existing, &mut rng);
    assert_eq!(next_id.len(), 4);
}

#[test]
fn validate_client_ids() {
    assert!(client_selection::is_valid_client_id("ABC"));
    assert!(client_selection::is_valid_client_id("DTX"));
    assert!(client_selection::is_valid_client_id("WXYZ"));
    assert!(client_selection::is_valid_client_id("ABCDE"));
    assert!(client_selection::is_valid_client_id("ABCDEF"));

    assert!(!client_selection::is_valid_client_id("AB")); // Too short
    assert!(!client_selection::is_valid_client_id("ABCDEFG")); // Too long
    assert!(!client_selection::is_valid_client_id("A0B")); // Invalid char
    assert!(!client_selection::is_valid_client_id("abc")); // Lowercase (Base32 check fails)
}

#[test]
fn extract_client_ids_from_lattice_ids() {
    let ids = vec!["LVDDTX", "LBSWXY", "LK3ZZZ", "LBSABC"];

    let clients = client_selection::extract_client_ids(ids.iter().map(|s| *s), 3);
    assert!(clients.contains("DTX"));
    assert!(clients.contains("WXY"));
    assert!(clients.contains("ZZZ"));
    assert!(clients.contains("ABC"));
    assert_eq!(clients.len(), 4);
}

#[test]
fn extract_handles_mixed_case() {
    let ids = vec!["LVDDTX", "lvddtx", "Lbswxy"];

    let clients = client_selection::extract_client_ids(ids.iter().map(|s| *s), 3);
    // Should normalize to uppercase
    assert!(clients.contains("DTX"));
    assert!(clients.contains("WXY"));
}

#[test]
fn extract_ignores_malformed_ids() {
    let ids = vec!["LVDDTX", "INVALID", "L", "LAB"];

    let clients = client_selection::extract_client_ids(ids.iter().map(|s| *s), 3);
    assert_eq!(clients.len(), 1);
    assert!(clients.contains("DTX"));
}
