use lattice::id::id_generator::{DocumentCounter, INITIAL_COUNTER};
use lattice::id::lattice_id::LatticeId;

#[test]
fn counter_starts_at_initial_value() {
    let counter = DocumentCounter::new();
    assert_eq!(
        counter.current(),
        INITIAL_COUNTER,
        "New counter should start at INITIAL_COUNTER ({INITIAL_COUNTER})"
    );
}

#[test]
fn starting_at_below_initial_uses_initial() {
    let counter = DocumentCounter::starting_at(10);
    assert_eq!(
        counter.current(),
        INITIAL_COUNTER,
        "Counter started below initial should use initial value"
    );
}

#[test]
fn starting_at_above_initial_uses_provided_value() {
    let counter = DocumentCounter::starting_at(200);
    assert_eq!(counter.current(), 200, "Counter should start at provided value when above initial");
}

#[test]
fn ensure_at_least_advances_counter_to_avoid_reuse() {
    let mut counter = DocumentCounter::new();
    counter.ensure_at_least(100);
    assert_eq!(
        counter.current(),
        101,
        "ensure_at_least(100) should set counter to 101 to avoid reusing 100"
    );
}

#[test]
fn ensure_at_least_is_idempotent_for_lower_values() {
    let mut counter = DocumentCounter::starting_at(200);
    counter.ensure_at_least(100);
    assert_eq!(
        counter.current(),
        200,
        "ensure_at_least with lower value should not change counter"
    );
}

#[test]
fn ensure_at_least_handles_boundary_at_current_minus_one() {
    let mut counter = DocumentCounter::starting_at(100);
    counter.ensure_at_least(99);
    assert_eq!(counter.current(), 100, "ensure_at_least(99) should not affect counter at 100");
}

#[test]
fn ensure_at_least_handles_exact_current_value() {
    let mut counter = DocumentCounter::starting_at(100);
    counter.ensure_at_least(100);
    assert_eq!(counter.current(), 101, "ensure_at_least(100) when at 100 should advance to 101");
}

#[test]
fn extract_counter_from_ids_with_3_char_client() {
    let id = LatticeId::from_parts(INITIAL_COUNTER, "ABC");
    let extracted = id.counter_assuming_client_len(3).expect("Should extract counter");
    assert_eq!(extracted, INITIAL_COUNTER, "Extracted counter should match original");
}

#[test]
fn extract_counter_from_ids_with_4_char_client() {
    let id = LatticeId::from_parts(75, "WXYZ");
    let extracted = id.counter_assuming_client_len(4).expect("Should extract counter");
    assert_eq!(extracted, 75, "Extracted counter should be 75 for 4-char client");
}

#[test]
fn extract_counter_from_ids_with_5_char_client() {
    let id = LatticeId::from_parts(100, "ABCDE");
    let extracted = id.counter_assuming_client_len(5).expect("Should extract counter");
    assert_eq!(extracted, 100, "Extracted counter should be 100 for 5-char client");
}

#[test]
fn extract_counter_from_ids_with_6_char_client() {
    let id = LatticeId::from_parts(999, "UVWXYZ");
    let extracted = id.counter_assuming_client_len(6).expect("Should extract counter");
    assert_eq!(extracted, 999, "Extracted counter should be 999 for 6-char client");
}

#[test]
fn recovery_simulation_finds_highest_counter() {
    let ids = vec![
        LatticeId::from_parts(50, "AAA"),
        LatticeId::from_parts(75, "AAA"),
        LatticeId::from_parts(60, "AAA"),
        LatticeId::from_parts(100, "AAA"),
        LatticeId::from_parts(85, "AAA"),
    ];

    let mut highest = 0u64;
    for id in &ids {
        if let Ok(counter) = id.counter_assuming_client_len(3) {
            highest = highest.max(counter);
        }
    }

    let mut recovered = DocumentCounter::new();
    recovered.ensure_at_least(highest);

    assert_eq!(recovered.current(), 101, "Recovered counter should be 101 (one past highest 100)");
}

#[test]
fn recovery_handles_gaps_in_sequence() {
    let ids = vec![
        LatticeId::from_parts(50, "XYZ"),
        LatticeId::from_parts(100, "XYZ"),
        LatticeId::from_parts(500, "XYZ"),
    ];

    let mut highest = 0u64;
    for id in &ids {
        if let Ok(counter) = id.counter_assuming_client_len(3) {
            highest = highest.max(counter);
        }
    }

    let mut recovered = DocumentCounter::new();
    recovered.ensure_at_least(highest);

    assert_eq!(
        recovered.current(),
        501,
        "Recovery should use highest counter (500) + 1 regardless of gaps"
    );
}

#[test]
fn recovery_with_mixed_client_ids() {
    let mut highest_per_client: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();

    let ids = vec![
        ("ABC", LatticeId::from_parts(50, "ABC")),
        ("ABC", LatticeId::from_parts(75, "ABC")),
        ("DEF", LatticeId::from_parts(60, "DEF")),
        ("DEF", LatticeId::from_parts(100, "DEF")),
    ];

    for (client, id) in &ids {
        if let Ok(counter) = id.counter_assuming_client_len(3) {
            let entry = highest_per_client.entry(client.to_string()).or_insert(0);
            *entry = (*entry).max(counter);
        }
    }

    assert_eq!(highest_per_client.get("ABC"), Some(&75), "ABC's highest counter should be 75");
    assert_eq!(highest_per_client.get("DEF"), Some(&100), "DEF's highest counter should be 100");
}

#[test]
fn counter_next_value_returns_current_then_increments() {
    let mut counter = DocumentCounter::starting_at(100);

    assert_eq!(counter.next_value(), 100, "First call should return 100");
    assert_eq!(counter.next_value(), 101, "Second call should return 101");
    assert_eq!(counter.next_value(), 102, "Third call should return 102");
    assert_eq!(counter.current(), 103, "After 3 calls, current should be 103");
}

#[test]
fn counter_recovery_from_empty_repository() {
    let ids: Vec<LatticeId> = vec![];
    let mut highest = 0u64;

    for id in &ids {
        if let Ok(counter) = id.counter_assuming_client_len(3) {
            highest = highest.max(counter);
        }
    }

    let mut recovered = DocumentCounter::new();
    if highest > 0 {
        recovered.ensure_at_least(highest);
    }

    assert_eq!(recovered.current(), INITIAL_COUNTER, "Empty repository should use INITIAL_COUNTER");
}

#[test]
fn large_counter_values_handled_correctly() {
    let large_counter = 1_000_000u64;
    let id = LatticeId::from_parts(large_counter, "ZZZ");
    let extracted = id.counter_assuming_client_len(3).expect("Should extract large counter");

    assert_eq!(extracted, large_counter, "Large counter values should round-trip correctly");

    let mut counter = DocumentCounter::new();
    counter.ensure_at_least(large_counter);
    assert_eq!(counter.current(), large_counter + 1, "Should advance past large counter");
}
