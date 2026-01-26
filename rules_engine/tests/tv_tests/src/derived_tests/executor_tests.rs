use std::collections::HashMap;

use tv_lib::derived::compute_executor::{
    execute_computation, ComputationRequest, ComputeExecutor, ComputeExecutorState,
};
use tv_lib::derived::derived_types::{DerivedFunction, DerivedResult, LookupContext, RowData};
use tv_lib::derived::function_registry::{global_registry, initialize_global_registry};
use tv_lib::derived::generation_tracker::RowKey;

fn make_row_key(row: usize) -> RowKey {
    RowKey::new("/test/file.toml", "cards", row)
}

fn make_request(row: usize, visible: bool) -> ComputationRequest {
    ComputationRequest {
        row_key: make_row_key(row),
        function_name: "test_function".to_string(),
        row_data: HashMap::new(),
        generation: 1,
        is_visible: visible,
    }
}

fn make_request_with_generation(row: usize, generation: u64, visible: bool) -> ComputationRequest {
    ComputationRequest {
        row_key: make_row_key(row),
        function_name: "test_function".to_string(),
        row_data: HashMap::new(),
        generation,
        is_visible: visible,
    }
}

fn make_request_with_data(row: usize, data: RowData) -> ComputationRequest {
    ComputationRequest {
        row_key: make_row_key(row),
        function_name: "test_function".to_string(),
        row_data: data,
        generation: 1,
        is_visible: true,
    }
}

fn ensure_registry_initialized() {
    initialize_global_registry();
}

struct PanickingTestFunction;

impl DerivedFunction for PanickingTestFunction {
    fn name(&self) -> &'static str {
        "test_panic_function"
    }

    fn input_keys(&self) -> Vec<&'static str> {
        vec![]
    }

    fn compute(&self, _inputs: &RowData, _context: &LookupContext) -> DerivedResult {
        panic!("Intentional test panic");
    }
}

static REGISTER_PANICKING_FUNCTION: std::sync::Once = std::sync::Once::new();

fn ensure_panicking_function_registered() {
    ensure_registry_initialized();
    REGISTER_PANICKING_FUNCTION.call_once(|| {
        global_registry().register(Box::new(PanickingTestFunction));
    });
}

#[test]
fn test_executor_creation_with_explicit_threads() {
    let executor = ComputeExecutor::new(Some(2)).expect("Should create executor");
    assert!(!executor.is_running(), "New executor should not be running");
    assert_eq!(executor.queue_len(), 0, "New executor should have empty queue");
}

#[test]
fn test_executor_creation_with_default_threads() {
    let executor = ComputeExecutor::new(None).expect("Should create executor with default threads");
    assert!(!executor.is_running(), "New executor should not be running");
    assert_eq!(executor.queue_len(), 0, "New executor should have empty queue");
}

#[test]
fn test_executor_creation_with_single_thread() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor with 1 thread");
    assert_eq!(executor.queue_len(), 0, "New executor should have empty queue");
}

#[test]
fn test_queue_single_visible_request() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");
    executor.queue_computation(make_request(0, true));
    assert_eq!(executor.queue_len(), 1, "Queue should have 1 item");
}

#[test]
fn test_queue_single_offscreen_request() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");
    executor.queue_computation(make_request(0, false));
    assert_eq!(executor.queue_len(), 1, "Queue should have 1 item");
}

#[test]
fn test_queue_multiple_requests() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");
    executor.queue_computation(make_request(0, true));
    executor.queue_computation(make_request(1, false));
    executor.queue_computation(make_request(2, true));
    assert_eq!(executor.queue_len(), 3, "Queue should have 3 items");
}

#[test]
fn test_visible_rows_get_priority_over_offscreen() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");
    executor.queue_computation(make_request(0, false));
    executor.queue_computation(make_request(1, true));
    assert_eq!(executor.queue_len(), 2, "Queue should have 2 items");
}

#[test]
fn test_queue_batch_with_mixed_visibility() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    let requests = vec![
        make_request(0, false),
        make_request(1, true),
        make_request(2, false),
        make_request(3, true),
    ];

    executor.queue_batch(requests);
    assert_eq!(executor.queue_len(), 4, "Batch should queue all 4 items");
}

#[test]
fn test_queue_batch_empty() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");
    executor.queue_batch(vec![]);
    assert_eq!(executor.queue_len(), 0, "Empty batch should not add items");
}

#[test]
fn test_queue_batch_all_visible() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    let requests = vec![make_request(0, true), make_request(1, true), make_request(2, true)];

    executor.queue_batch(requests);
    assert_eq!(executor.queue_len(), 3, "All visible batch should queue 3 items");
}

#[test]
fn test_queue_batch_all_offscreen() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    let requests = vec![make_request(0, false), make_request(1, false), make_request(2, false)];

    executor.queue_batch(requests);
    assert_eq!(executor.queue_len(), 3, "All offscreen batch should queue 3 items");
}

#[test]
fn test_clear_queue() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    executor.queue_computation(make_request(0, true));
    executor.queue_computation(make_request(1, false));
    executor.queue_computation(make_request(2, true));
    assert_eq!(executor.queue_len(), 3, "Queue should have 3 items before clear");

    executor.clear_queue();
    assert_eq!(executor.queue_len(), 0, "Queue should be empty after clear");
}

#[test]
fn test_clear_empty_queue() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");
    executor.clear_queue();
    assert_eq!(executor.queue_len(), 0, "Clearing empty queue should be safe");
}

#[test]
fn test_queue_len_after_batch_and_clear() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    executor.queue_batch(vec![make_request(0, true), make_request(1, false)]);
    assert_eq!(executor.queue_len(), 2, "Queue should have 2 items after batch");

    executor.clear_queue();
    assert_eq!(executor.queue_len(), 0, "Queue should be empty after clear");

    executor.queue_computation(make_request(0, true));
    assert_eq!(executor.queue_len(), 1, "Queue should have 1 item after re-queue");
}

#[test]
fn test_generation_tracker_accessible_from_executor() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");
    let tracker = executor.generation_tracker();

    let key = make_row_key(0);
    assert_eq!(tracker.get_generation(&key), 0, "New tracker should have generation 0");

    let gen = tracker.increment_generation(key.clone());
    assert!(gen > 0, "Incremented generation should be positive");
    assert_eq!(tracker.get_generation(&key), gen, "Tracker should return updated generation");
}

#[test]
fn test_generation_tracker_stale_detection() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");
    let tracker = executor.generation_tracker();

    let key = make_row_key(0);
    let gen1 = tracker.increment_generation(key.clone());
    assert!(tracker.is_generation_current(&key, gen1), "Current generation should be valid");

    let gen2 = tracker.increment_generation(key.clone());
    assert!(!tracker.is_generation_current(&key, gen1), "Old generation should be stale");
    assert!(tracker.is_generation_current(&key, gen2), "New generation should be valid");
}

#[test]
fn test_set_lookup_context() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    let mut context = LookupContext::new();
    let mut cards: HashMap<String, RowData> = HashMap::new();
    let mut row: RowData = HashMap::new();
    row.insert("name".to_string(), serde_json::json!("Test Card"));
    cards.insert("card-1".to_string(), row);
    context.add_table("cards", cards);

    executor.set_lookup_context(context);
}

#[test]
fn test_set_lookup_context_multiple_times() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    executor.set_lookup_context(LookupContext::new());
    executor.set_lookup_context(LookupContext::new());

    let mut context = LookupContext::new();
    let mut cards: HashMap<String, RowData> = HashMap::new();
    let mut row: RowData = HashMap::new();
    row.insert("name".to_string(), serde_json::json!("Updated Card"));
    cards.insert("card-1".to_string(), row);
    context.add_table("cards", cards);

    executor.set_lookup_context(context);
}

#[test]
fn test_queue_request_with_row_data() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    let mut data: RowData = HashMap::new();
    data.insert("name".to_string(), serde_json::json!("Dragon Knight"));
    data.insert("cost".to_string(), serde_json::json!(5));

    executor.queue_computation(make_request_with_data(0, data));
    assert_eq!(executor.queue_len(), 1, "Request with data should be queued");
}

#[test]
fn test_queue_requests_with_different_functions() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    let request1 = ComputationRequest {
        row_key: make_row_key(0),
        function_name: "card_lookup".to_string(),
        row_data: HashMap::new(),
        generation: 1,
        is_visible: true,
    };

    let request2 = ComputationRequest {
        row_key: make_row_key(0),
        function_name: "image_url".to_string(),
        row_data: HashMap::new(),
        generation: 1,
        is_visible: true,
    };

    executor.queue_computation(request1);
    executor.queue_computation(request2);
    assert_eq!(executor.queue_len(), 2, "Same row with different functions should both be queued");
}

#[test]
fn test_queue_requests_with_different_generations() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    executor.queue_computation(make_request_with_generation(0, 1, true));
    executor.queue_computation(make_request_with_generation(0, 2, true));
    assert_eq!(
        executor.queue_len(),
        2,
        "Same row with different generations should both be queued"
    );
}

#[test]
fn test_executor_state_new_is_uninitialized() {
    let state = ComputeExecutorState::new();
    assert!(state.with_executor(|_| ()).is_none(), "New state should not have an executor");
}

#[test]
fn test_executor_state_default_is_uninitialized() {
    let state = ComputeExecutorState::default();
    assert!(state.with_executor(|_| ()).is_none(), "Default state should not have an executor");
}

#[test]
fn test_executor_state_initialize() {
    let state = ComputeExecutorState::new();
    state.initialize(Some(1)).expect("Should initialize executor");
    assert!(state.with_executor(|_| ()).is_some(), "Initialized state should have an executor");
}

#[test]
fn test_executor_state_double_initialize_is_safe() {
    let state = ComputeExecutorState::new();
    state.initialize(Some(1)).expect("First initialization should succeed");
    state.initialize(Some(2)).expect("Second initialization should also succeed");

    assert!(
        state.with_executor(|_| ()).is_some(),
        "State should still have an executor after double init"
    );
}

#[test]
fn test_executor_state_with_executor_callback() {
    let state = ComputeExecutorState::new();
    state.initialize(Some(1)).expect("Should initialize executor");

    let queue_len = state.with_executor(|exec| exec.queue_len());
    assert_eq!(queue_len, Some(0), "Executor queue should be empty");
}

#[test]
fn test_executor_state_with_executor_mut_callback() {
    let state = ComputeExecutorState::new();
    state.initialize(Some(1)).expect("Should initialize executor");

    let result = state.with_executor_mut(|exec| {
        exec.queue_computation(make_request(0, true));
        exec.queue_len()
    });
    assert_eq!(result, Some(1), "Should have queued 1 item");

    let queue_len = state.with_executor(|exec| exec.queue_len());
    assert_eq!(queue_len, Some(1), "Queue length should still be 1");
}

#[test]
fn test_executor_state_stop_before_start() {
    let state = ComputeExecutorState::new();
    state.initialize(Some(1)).expect("Should initialize executor");

    state.stop();
}

#[test]
fn test_executor_state_with_executor_on_uninitialized() {
    let state = ComputeExecutorState::new();
    assert!(
        state.with_executor(|_| 42).is_none(),
        "Uninitialized state should return None from with_executor"
    );
}

#[test]
fn test_executor_state_with_executor_mut_on_uninitialized() {
    let state = ComputeExecutorState::new();
    assert!(
        state.with_executor_mut(|_| 42).is_none(),
        "Uninitialized state should return None from with_executor_mut"
    );
}

#[test]
fn test_executor_queue_and_clear_via_state() {
    let state = ComputeExecutorState::new();
    state.initialize(Some(1)).expect("Should initialize executor");

    state.with_executor(|exec| {
        exec.queue_computation(make_request(0, true));
        exec.queue_computation(make_request(1, false));
    });

    let len = state.with_executor(|exec| exec.queue_len());
    assert_eq!(len, Some(2), "Should have 2 queued items");

    state.with_executor(|exec| exec.clear_queue());

    let len = state.with_executor(|exec| exec.queue_len());
    assert_eq!(len, Some(0), "Queue should be empty after clear");
}

#[test]
fn test_executor_generation_tracker_via_state() {
    let state = ComputeExecutorState::new();
    state.initialize(Some(1)).expect("Should initialize executor");

    let gen = state.with_executor(|exec| {
        let tracker = exec.generation_tracker();
        let key = make_row_key(0);
        tracker.increment_generation(key)
    });

    assert!(gen.is_some(), "Should get generation value");
    assert!(gen.unwrap() > 0, "Generation should be positive");
}

#[test]
fn test_computation_request_fields() {
    let key = RowKey::new("/test/file.toml", "effects", 5);
    let mut data: RowData = HashMap::new();
    data.insert("damage".to_string(), serde_json::json!(3));

    let request = ComputationRequest {
        row_key: key.clone(),
        function_name: "damage_calc".to_string(),
        row_data: data.clone(),
        generation: 42,
        is_visible: false,
    };

    assert_eq!(request.row_key, key, "Row key should match");
    assert_eq!(request.function_name, "damage_calc", "Function name should match");
    assert_eq!(request.generation, 42, "Generation should match");
    assert!(!request.is_visible, "Visibility should match");
    assert_eq!(
        request.row_data.get("damage"),
        Some(&serde_json::json!(3)),
        "Row data should match"
    );
}

#[test]
fn test_computation_request_clone() {
    let request = make_request(0, true);
    let cloned = request.clone();

    assert_eq!(request.row_key, cloned.row_key, "Cloned row key should match");
    assert_eq!(request.function_name, cloned.function_name, "Cloned function name should match");
    assert_eq!(request.generation, cloned.generation, "Cloned generation should match");
    assert_eq!(request.is_visible, cloned.is_visible, "Cloned visibility should match");
}

#[test]
fn test_executor_is_not_running_after_creation() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");
    assert!(!executor.is_running(), "Executor should not be running before start");
}

#[test]
fn test_queue_large_batch() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    let requests: Vec<ComputationRequest> = (0..100).map(|i| make_request(i, i % 2 == 0)).collect();

    executor.queue_batch(requests);
    assert_eq!(executor.queue_len(), 100, "Should queue all 100 items");
}

#[test]
fn test_queue_interleaved_single_and_batch() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    executor.queue_computation(make_request(0, true));
    executor.queue_batch(vec![make_request(1, false), make_request(2, true)]);
    executor.queue_computation(make_request(3, false));

    assert_eq!(executor.queue_len(), 4, "Should have 4 total items");
}

#[test]
fn test_executor_generation_tracker_shared_across_calls() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    let key = make_row_key(0);
    let gen1 = executor.generation_tracker().increment_generation(key.clone());
    let gen2 = executor.generation_tracker().increment_generation(key.clone());

    assert!(gen2 > gen1, "Shared tracker should maintain state across calls");
    assert_eq!(
        executor.generation_tracker().get_generation(&key),
        gen2,
        "Tracker should reflect latest generation"
    );
}

#[test]
fn test_executor_set_lookup_context_with_multiple_tables() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    let mut context = LookupContext::new();

    let mut cards: HashMap<String, RowData> = HashMap::new();
    let mut card_row: RowData = HashMap::new();
    card_row.insert("name".to_string(), serde_json::json!("Fire Dragon"));
    cards.insert("card-1".to_string(), card_row);
    context.add_table("cards", cards);

    let mut effects: HashMap<String, RowData> = HashMap::new();
    let mut effect_row: RowData = HashMap::new();
    effect_row.insert("damage".to_string(), serde_json::json!(5));
    effects.insert("effect-1".to_string(), effect_row);
    context.add_table("effects", effects);

    executor.set_lookup_context(context);
}

#[test]
fn test_queue_request_for_different_files() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    let request1 = ComputationRequest {
        row_key: RowKey::new("/test/cards.toml", "cards", 0),
        function_name: "test_function".to_string(),
        row_data: HashMap::new(),
        generation: 1,
        is_visible: true,
    };

    let request2 = ComputationRequest {
        row_key: RowKey::new("/test/effects.toml", "effects", 0),
        function_name: "test_function".to_string(),
        row_data: HashMap::new(),
        generation: 1,
        is_visible: true,
    };

    executor.queue_computation(request1);
    executor.queue_computation(request2);
    assert_eq!(executor.queue_len(), 2, "Requests for different files should both be queued");
}

#[test]
fn test_queue_request_for_different_tables() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");

    let request1 = ComputationRequest {
        row_key: RowKey::new("/test/file.toml", "cards", 0),
        function_name: "test_function".to_string(),
        row_data: HashMap::new(),
        generation: 1,
        is_visible: true,
    };

    let request2 = ComputationRequest {
        row_key: RowKey::new("/test/file.toml", "effects", 0),
        function_name: "test_function".to_string(),
        row_data: HashMap::new(),
        generation: 1,
        is_visible: true,
    };

    executor.queue_computation(request1);
    executor.queue_computation(request2);
    assert_eq!(executor.queue_len(), 2, "Requests for different tables should both be queued");
}

#[test]
fn test_execute_computation_function_not_found() {
    ensure_registry_initialized();
    let row_data = HashMap::new();
    let context = LookupContext::new();

    let outcome = execute_computation("nonexistent_function", &row_data, &context);

    match outcome.result {
        DerivedResult::Error(msg) => {
            assert!(msg.contains("not found"), "Error should mention not found: {msg}");
        }
        _ => panic!("Expected error result for nonexistent function"),
    }
}

#[test]
fn test_executor_computes_value() {
    ensure_registry_initialized();
    let mut row_data: RowData = HashMap::new();
    row_data.insert("image_number".to_string(), serde_json::json!("42"));
    let context = LookupContext::new();

    let outcome = execute_computation("image_url", &row_data, &context);

    match outcome.result {
        DerivedResult::Image(url) => {
            assert!(url.contains("42"), "Image URL should contain the image number: {url}");
        }
        other => panic!("Expected Image result, got {other:?}"),
    }
}

#[test]
fn test_executor_computes_value_with_numeric_input() {
    ensure_registry_initialized();
    let mut row_data: RowData = HashMap::new();
    row_data.insert("image_number".to_string(), serde_json::json!(99));
    let context = LookupContext::new();

    let outcome = execute_computation("image_url", &row_data, &context);

    match outcome.result {
        DerivedResult::Image(url) => {
            assert!(url.contains("99"), "Image URL should contain the number: {url}");
        }
        other => panic!("Expected Image result, got {other:?}"),
    }
}

#[test]
fn test_executor_computes_empty_value_for_missing_input() {
    ensure_registry_initialized();
    let row_data: RowData = HashMap::new();
    let context = LookupContext::new();

    let outcome = execute_computation("image_url", &row_data, &context);

    match outcome.result {
        DerivedResult::Text(text) => {
            assert!(text.is_empty(), "Missing input should produce empty text: {text}");
        }
        other => panic!("Expected empty Text result, got {other:?}"),
    }
}

#[test]
fn test_executor_catches_panic() {
    ensure_panicking_function_registered();
    let row_data: RowData = HashMap::new();
    let context = LookupContext::new();

    let outcome = execute_computation("test_panic_function", &row_data, &context);
    assert!(outcome.panicked, "Should report panic");

    match outcome.result {
        DerivedResult::Error(msg) => {
            assert!(msg.contains("panicked"), "Error should mention panic: {msg}");
            assert!(
                msg.contains("Intentional test panic"),
                "Error should contain panic message: {msg}"
            );
        }
        other => panic!("Expected Error result from panicking function, got {other:?}"),
    }
}

#[test]
fn test_executor_catches_panic_returns_error_not_crash() {
    ensure_panicking_function_registered();
    let row_data: RowData = HashMap::new();
    let context = LookupContext::new();

    let outcome = execute_computation("test_panic_function", &row_data, &context);
    assert!(
        matches!(outcome.result, DerivedResult::Error(_)),
        "Panicking function should produce Error variant, not crash"
    );

    let outcome2 = execute_computation("image_url", &row_data, &context);
    assert!(
        !matches!(outcome2.result, DerivedResult::Error(ref msg) if msg.contains("panic")),
        "Subsequent computations should work normally after a panic"
    );
}

#[test]
fn test_generation_tracker_integration_with_executor() {
    let executor = ComputeExecutor::new(Some(1)).expect("Should create executor");
    let tracker = executor.generation_tracker();

    let key = make_row_key(0);
    let gen = tracker.increment_generation(key.clone());

    let request = ComputationRequest {
        row_key: key.clone(),
        function_name: "test".to_string(),
        row_data: HashMap::new(),
        generation: gen,
        is_visible: true,
    };
    executor.queue_computation(request);

    assert_eq!(executor.queue_len(), 1, "Request should be queued");
}
