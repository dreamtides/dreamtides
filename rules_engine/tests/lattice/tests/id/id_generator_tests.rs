use lattice::id::id_generator;
use lattice::id::id_generator::{DocumentCounter, INITIAL_COUNTER};

#[test]
fn new_counter_starts_at_initial() {
    let counter = DocumentCounter::new();
    assert_eq!(counter.current(), INITIAL_COUNTER);
}

#[test]
fn counter_increments() {
    let mut counter = DocumentCounter::new();
    assert_eq!(counter.next_value(), INITIAL_COUNTER);
    assert_eq!(counter.next_value(), INITIAL_COUNTER + 1);
    assert_eq!(counter.next_value(), INITIAL_COUNTER + 2);
    assert_eq!(counter.current(), INITIAL_COUNTER + 3);
}

#[test]
fn starting_at_respects_minimum() {
    let counter = DocumentCounter::starting_at(10);
    assert_eq!(counter.current(), INITIAL_COUNTER);

    let counter = DocumentCounter::starting_at(100);
    assert_eq!(counter.current(), 100);
}

#[test]
fn ensure_at_least_updates_when_needed() {
    let mut counter = DocumentCounter::new();
    counter.ensure_at_least(30); // Below INITIAL_COUNTER
    assert_eq!(counter.current(), INITIAL_COUNTER);

    counter.ensure_at_least(100);
    assert_eq!(counter.current(), 101);

    counter.ensure_at_least(50); // Already past this
    assert_eq!(counter.current(), 101);
}

#[test]
fn generate_id_creates_valid_ids() {
    let mut counter = DocumentCounter::new();
    let id = id_generator::generate_id(&mut counter, "DTX");

    assert!(id.as_str().starts_with('L'));
    assert!(id.as_str().len() >= 6);
    assert_eq!(id.counter_assuming_client_len(3).unwrap(), INITIAL_COUNTER);
    assert_eq!(id.client_id_assuming_len(3).unwrap(), "DTX");
}

#[test]
fn generated_ids_are_unique() {
    let mut counter = DocumentCounter::new();
    let id1 = id_generator::generate_id(&mut counter, "AAA");
    let id2 = id_generator::generate_id(&mut counter, "AAA");
    let id3 = id_generator::generate_id(&mut counter, "AAA");

    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}

#[test]
fn different_clients_different_ids() {
    let mut counter1 = DocumentCounter::new();
    let mut counter2 = DocumentCounter::new();

    let id1 = id_generator::generate_id(&mut counter1, "AAA");
    let id2 = id_generator::generate_id(&mut counter2, "BBB");

    // Same counter value but different client IDs
    assert_ne!(id1, id2);
}
