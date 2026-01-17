use lattice::index::client_counters::{
    delete, get_and_increment, get_counter, list_all, set_counter, set_counter_if_higher,
};
use lattice::index::schema_definition;
use rusqlite::Connection;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

#[test]
fn get_and_increment_creates_new_counter() {
    let conn = create_test_db();

    let count = get_and_increment(&conn, "CLIENT1").expect("Get and increment should succeed");

    assert_eq!(count, 0, "First call should return 0 (counter starts at 0)");
}

#[test]
fn get_and_increment_increments_existing() {
    let conn = create_test_db();

    get_and_increment(&conn, "CLIENT2").expect("First call should succeed");
    let second = get_and_increment(&conn, "CLIENT2").expect("Second call should succeed");
    let third = get_and_increment(&conn, "CLIENT2").expect("Third call should succeed");

    assert_eq!(second, 1, "Second call should return 1");
    assert_eq!(third, 2, "Third call should return 2");
}

#[test]
fn get_and_increment_tracks_multiple_clients() {
    let conn = create_test_db();

    let a1 = get_and_increment(&conn, "CLIENTA").expect("Should succeed");
    let b1 = get_and_increment(&conn, "CLIENTB").expect("Should succeed");
    let a2 = get_and_increment(&conn, "CLIENTA").expect("Should succeed");

    assert_eq!(a1, 0, "First call for CLIENTA should return 0");
    assert_eq!(b1, 0, "First call for CLIENTB should return 0");
    assert_eq!(a2, 1, "Second call for CLIENTA should return 1");
}

#[test]
fn get_counter_returns_none_for_missing() {
    let conn = create_test_db();

    let result = get_counter(&conn, "MISSING").expect("Get counter should succeed");

    assert!(result.is_none(), "Should return None for non-existent client");
}

#[test]
fn get_counter_returns_current_value() {
    let conn = create_test_db();
    get_and_increment(&conn, "CLIENT3").expect("Should succeed");
    get_and_increment(&conn, "CLIENT3").expect("Should succeed");
    get_and_increment(&conn, "CLIENT3").expect("Should succeed");

    let result = get_counter(&conn, "CLIENT3").expect("Get counter should succeed");

    assert_eq!(result, Some(3), "Should return current counter value");
}

#[test]
fn get_counter_does_not_increment() {
    let conn = create_test_db();
    set_counter(&conn, "CLIENT4", 10).expect("Should succeed");

    get_counter(&conn, "CLIENT4").expect("First get should succeed");
    get_counter(&conn, "CLIENT4").expect("Second get should succeed");
    let result = get_counter(&conn, "CLIENT4").expect("Third get should succeed");

    assert_eq!(result, Some(10), "Counter should remain at 10 after multiple reads");
}

#[test]
fn set_counter_creates_new_entry() {
    let conn = create_test_db();

    set_counter(&conn, "CLIENT5", 42).expect("Set counter should succeed");
    let result = get_counter(&conn, "CLIENT5").expect("Get should succeed");

    assert_eq!(result, Some(42), "Counter should be set to 42");
}

#[test]
fn set_counter_updates_existing_entry() {
    let conn = create_test_db();
    set_counter(&conn, "CLIENT6", 10).expect("First set should succeed");

    set_counter(&conn, "CLIENT6", 20).expect("Second set should succeed");
    let result = get_counter(&conn, "CLIENT6").expect("Get should succeed");

    assert_eq!(result, Some(20), "Counter should be updated to 20");
}

#[test]
fn set_counter_can_set_to_zero() {
    let conn = create_test_db();
    set_counter(&conn, "CLIENT7", 100).expect("First set should succeed");

    set_counter(&conn, "CLIENT7", 0).expect("Set to zero should succeed");
    let result = get_counter(&conn, "CLIENT7").expect("Get should succeed");

    assert_eq!(result, Some(0), "Counter should be set to 0");
}

#[test]
fn set_counter_if_higher_creates_new_entry() {
    let conn = create_test_db();

    set_counter_if_higher(&conn, "CLIENT8", 50).expect("Should succeed");
    let result = get_counter(&conn, "CLIENT8").expect("Get should succeed");

    assert_eq!(result, Some(50), "Counter should be created at 50");
}

#[test]
fn set_counter_if_higher_updates_when_higher() {
    let conn = create_test_db();
    set_counter(&conn, "CLIENT9", 10).expect("Initial set should succeed");

    set_counter_if_higher(&conn, "CLIENT9", 20).expect("Should succeed");
    let result = get_counter(&conn, "CLIENT9").expect("Get should succeed");

    assert_eq!(result, Some(20), "Counter should be updated to 20");
}

#[test]
fn set_counter_if_higher_does_not_update_when_lower() {
    let conn = create_test_db();
    set_counter(&conn, "CLIENTA0", 50).expect("Initial set should succeed");

    set_counter_if_higher(&conn, "CLIENTA0", 30).expect("Should succeed without error");
    let result = get_counter(&conn, "CLIENTA0").expect("Get should succeed");

    assert_eq!(result, Some(50), "Counter should remain at 50");
}

#[test]
fn set_counter_if_higher_does_not_update_when_equal() {
    let conn = create_test_db();
    set_counter(&conn, "CLIENTA1", 25).expect("Initial set should succeed");

    set_counter_if_higher(&conn, "CLIENTA1", 25).expect("Should succeed without error");
    let result = get_counter(&conn, "CLIENTA1").expect("Get should succeed");

    assert_eq!(result, Some(25), "Counter should remain at 25");
}

#[test]
fn list_all_returns_empty_initially() {
    let conn = create_test_db();

    let result = list_all(&conn).expect("List all should succeed");

    assert!(result.is_empty(), "Should return empty list when no counters exist");
}

#[test]
fn list_all_returns_all_counters() {
    let conn = create_test_db();
    set_counter(&conn, "CLIENTB", 10).expect("Should succeed");
    set_counter(&conn, "CLIENTA", 20).expect("Should succeed");
    set_counter(&conn, "CLIENTC", 30).expect("Should succeed");

    let result = list_all(&conn).expect("List all should succeed");

    assert_eq!(result.len(), 3, "Should return 3 counters");
    assert!(result.contains(&("CLIENTA".to_string(), 20)));
    assert!(result.contains(&("CLIENTB".to_string(), 10)));
    assert!(result.contains(&("CLIENTC".to_string(), 30)));
}

#[test]
fn list_all_returns_sorted_by_client_id() {
    let conn = create_test_db();
    set_counter(&conn, "ZZZ", 1).expect("Should succeed");
    set_counter(&conn, "AAA", 2).expect("Should succeed");
    set_counter(&conn, "MMM", 3).expect("Should succeed");

    let result = list_all(&conn).expect("List all should succeed");

    assert_eq!(result[0].0, "AAA", "First entry should be AAA");
    assert_eq!(result[1].0, "MMM", "Second entry should be MMM");
    assert_eq!(result[2].0, "ZZZ", "Third entry should be ZZZ");
}

#[test]
fn delete_removes_existing_counter() {
    let conn = create_test_db();
    set_counter(&conn, "CLIENTD", 100).expect("Should succeed");

    let deleted = delete(&conn, "CLIENTD").expect("Delete should succeed");

    assert!(deleted, "Delete should return true when counter existed");
    let result = get_counter(&conn, "CLIENTD").expect("Get should succeed");
    assert!(result.is_none(), "Counter should no longer exist");
}

#[test]
fn delete_returns_false_for_missing() {
    let conn = create_test_db();

    let deleted = delete(&conn, "NONEXISTENT").expect("Delete should succeed");

    assert!(!deleted, "Delete should return false when counter did not exist");
}

#[test]
fn delete_does_not_affect_other_counters() {
    let conn = create_test_db();
    set_counter(&conn, "CLIENTE", 10).expect("Should succeed");
    set_counter(&conn, "CLIENTF", 20).expect("Should succeed");

    delete(&conn, "CLIENTE").expect("Delete should succeed");

    let remaining = get_counter(&conn, "CLIENTF").expect("Get should succeed");
    assert_eq!(remaining, Some(20), "Other counter should not be affected");
}
