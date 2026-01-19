use std::collections::HashSet;

use lattice::id::client_selection;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[test]
fn collision_triggers_retry_with_new_value() {
    let mut existing: HashSet<String> = HashSet::new();
    let mut rng = StdRng::seed_from_u64(0);

    let first_id = client_selection::generate_client_id(&existing, &mut rng);
    existing.insert(first_id.clone());

    let mut collision_rng = StdRng::seed_from_u64(0);
    let second_id = client_selection::generate_client_id(&existing, &mut collision_rng);

    assert_ne!(
        first_id, second_id,
        "Second ID should differ from first even with same seed due to collision retry"
    );
    assert!(
        client_selection::is_valid_client_id(&second_id),
        "Retried ID should still be valid: {second_id}"
    );
}

#[test]
fn high_collision_environment_still_generates_unique_ids() {
    let mut rng = StdRng::seed_from_u64(42);

    let mut existing: HashSet<String> = HashSet::new();
    for _ in 0..100 {
        existing.insert(client_selection::generate_client_id(&existing, &mut rng));
    }

    let mut test_rng = StdRng::seed_from_u64(99);
    for iteration in 0..50 {
        let new_id = client_selection::generate_client_id(&existing, &mut test_rng);
        assert!(
            !existing.contains(&new_id),
            "Iteration {iteration}: Generated duplicate ID {new_id}"
        );
        existing.insert(new_id);
    }
}

#[test]
fn length_transitions_at_correct_thresholds() {
    let mut rng = StdRng::seed_from_u64(12345);
    let mut existing: HashSet<String> = HashSet::new();

    for i in 0..17 {
        let id = client_selection::generate_client_id(&existing, &mut rng);
        assert_eq!(
            id.len(),
            3,
            "ID {i} with {} existing clients should be 3 chars, got {} chars: {id}",
            existing.len(),
            id.len()
        );
        existing.insert(id);
    }

    let id_17 = client_selection::generate_client_id(&existing, &mut rng);
    assert_eq!(
        id_17.len(),
        4,
        "With 17 existing clients, new ID should be 4 chars, got {}: {id_17}",
        id_17.len()
    );
    existing.insert(id_17);

    while existing.len() < 65 {
        let id = client_selection::generate_client_id(&existing, &mut rng);
        assert_eq!(
            id.len(),
            4,
            "ID with {} existing clients should be 4 chars, got {}: {id}",
            existing.len(),
            id.len()
        );
        existing.insert(id);
    }

    let id_65 = client_selection::generate_client_id(&existing, &mut rng);
    assert_eq!(
        id_65.len(),
        5,
        "With 65 existing clients, new ID should be 5 chars, got {}: {id_65}",
        id_65.len()
    );
    existing.insert(id_65);

    while existing.len() < 257 {
        existing.insert(client_selection::generate_client_id(&existing, &mut rng));
    }

    let id_257 = client_selection::generate_client_id(&existing, &mut rng);
    assert_eq!(
        id_257.len(),
        6,
        "With 257 existing clients, new ID should be 6 chars, got {}: {id_257}",
        id_257.len()
    );
}

#[test]
fn all_generated_ids_are_valid_base32() {
    let mut rng = StdRng::seed_from_u64(7777);
    let mut existing: HashSet<String> = HashSet::new();

    for _ in 0..200 {
        let id = client_selection::generate_client_id(&existing, &mut rng);
        assert!(client_selection::is_valid_client_id(&id), "Generated ID should be valid: {id}");

        for ch in id.chars() {
            assert!(
                ('A'..='Z').contains(&ch) || ('2'..='7').contains(&ch),
                "ID {id} contains invalid Base32 character: {ch}"
            );
        }

        existing.insert(id);
    }
}

#[test]
fn extract_handles_various_client_id_lengths() {
    let ids = vec![
        "LBSABC",   // 3-char client ID "ABC"
        "LK3WXYZ",  // 4-char client ID "WXYZ"
        "LVDABCDE", // 5-char client ID "ABCDE"
    ];

    let clients_3 = client_selection::extract_client_ids(ids.iter().map(|s| *s), 3);
    assert!(clients_3.contains("ABC"), "Should extract 3-char client ID ABC");

    let clients_4 = client_selection::extract_client_ids(ids.iter().map(|s| *s), 4);
    assert!(clients_4.contains("WXYZ"), "Should extract 4-char client ID WXYZ");

    let clients_5 = client_selection::extract_client_ids(ids.iter().map(|s| *s), 5);
    assert!(clients_5.contains("ABCDE"), "Should extract 5-char client ID ABCDE");
}

#[test]
fn extract_normalizes_case() {
    let ids = vec!["LBSABC", "lbsdef", "Lbsghi"];
    let clients = client_selection::extract_client_ids(ids.iter().map(|s| *s), 3);

    assert!(clients.contains("ABC"), "Should extract ABC normalized");
    assert!(clients.contains("DEF"), "Should extract def as DEF");
    assert!(clients.contains("GHI"), "Should extract ghi as GHI");
    assert!(!clients.contains("abc"), "Should not have lowercase abc");
}

#[test]
fn required_length_returns_minimum_for_empty_set() {
    assert_eq!(
        client_selection::required_length(0),
        3,
        "Empty client set should require 3-char IDs"
    );
}

#[test]
fn required_length_caps_at_6_chars() {
    assert_eq!(
        client_selection::required_length(1000),
        6,
        "Very large client count should cap at 6-char IDs"
    );
    assert_eq!(
        client_selection::required_length(10000),
        6,
        "Even larger client count should still be 6-char IDs"
    );
}

#[test]
fn extract_ignores_ids_too_short_for_assumed_length() {
    let ids = vec![
        "LABCD",   // Only 4 chars after L, too short for 4-char client + 2-char counter
        "LABCDEF", // 6 chars after L, valid for 4-char client + 2-char counter
    ];

    let clients = client_selection::extract_client_ids(ids.iter().map(|s| *s), 4);

    assert_eq!(clients.len(), 1, "Should only extract from sufficiently long ID");
    assert!(clients.contains("CDEF"), "Should extract CDEF from LABCDEF");
}
