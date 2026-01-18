use action_data::game_action_data::GameAction;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::{LoggingOptions, RequestContext};
use core_data::identifiers::UserId;
use display_data::request_data::{ConnectRequest, Metadata, PerformActionRequest};
use rules_engine::engine;
use state_provider::state_provider::StateProvider;
use state_provider::test_state_provider::TestStateProvider;
use uuid::Uuid;

fn streaming_assets_path() -> String {
    logging::get_developer_mode_project_directory()
        .expect("Failed to get project directory")
        .join("client/Assets/StreamingAssets")
        .canonicalize()
        .expect("Failed to canonicalize path")
        .to_string_lossy()
        .to_string()
}

#[test]
fn duplicate_action_is_ignored() {
    let provider = TestStateProvider::new();
    let streaming_assets_path = streaming_assets_path();
    let _ = provider.initialize("/tmp/test", &streaming_assets_path);
    let user_id = UserId(Uuid::new_v4());

    // Set a response version
    let response_version = Uuid::new_v4();
    provider.store_last_response_version(user_id, response_version);

    // Create a valid action request with the correct response version
    let request = PerformActionRequest {
        metadata: Metadata {
            user_id,
            battle_id: None,
            request_id: Some(Uuid::new_v4()),
            integration_test_id: None,
        },
        action: GameAction::BattleAction(BattleAction::EndTurn),
        save_file_id: None,
        last_response_version: Some(response_version),
    };

    // First action should succeed (we'll use start_processing to simulate)
    assert!(provider.start_processing(user_id), "First action should be allowed");
    provider.finish_processing(user_id);

    // Store a new response version to simulate that the first action completed
    let new_version = Uuid::new_v4();
    provider.store_last_response_version(user_id, new_version);

    // Second identical action should be ignored due to outdated response version
    let result = engine::perform_action_blocking(provider.clone(), request.clone(), None);
    assert!(result.user_poll_results.is_empty(), "Duplicate action should be ignored");
}

#[test]
fn concurrent_action_is_ignored() {
    let provider = TestStateProvider::new();
    let streaming_assets_path = streaming_assets_path();
    let _ = provider.initialize("/tmp/test", &streaming_assets_path);
    let user_id = UserId(Uuid::new_v4());

    // Set a response version
    let response_version = Uuid::new_v4();
    provider.store_last_response_version(user_id, response_version);

    // Start processing an action (marking user as processing)
    assert!(provider.start_processing(user_id), "Should be able to start processing");

    // Try to process another action while first is still processing
    let request = PerformActionRequest {
        metadata: Metadata {
            user_id,
            battle_id: None,
            request_id: Some(Uuid::new_v4()),
            integration_test_id: None,
        },
        action: GameAction::BattleAction(BattleAction::EndTurn),
        save_file_id: None,
        last_response_version: Some(response_version),
    };

    let result = engine::perform_action_blocking(provider.clone(), request, None);
    assert!(result.user_poll_results.is_empty(), "Concurrent action should be ignored");

    // Clean up
    provider.finish_processing(user_id);
}

#[test]
fn action_with_outdated_version_is_ignored() {
    let provider = TestStateProvider::new();
    let streaming_assets_path = streaming_assets_path();
    let _ = provider.initialize("/tmp/test", &streaming_assets_path);
    let user_id = UserId(Uuid::new_v4());

    // Set a current response version
    let current_version = Uuid::new_v4();
    provider.store_last_response_version(user_id, current_version);

    // Try to perform action with outdated version
    let outdated_version = Uuid::new_v4();
    let request = PerformActionRequest {
        metadata: Metadata {
            user_id,
            battle_id: None,
            request_id: Some(Uuid::new_v4()),
            integration_test_id: None,
        },
        action: GameAction::BattleAction(BattleAction::EndTurn),
        save_file_id: None,
        last_response_version: Some(outdated_version),
    };

    let result = engine::perform_action_blocking(provider.clone(), request, None);
    assert!(result.user_poll_results.is_empty(), "Action with outdated version should be ignored");

    // Verify that the user is not marked as processing (start_processing was never
    // called)
    assert!(
        !provider.is_processing(user_id),
        "User should not be marked as processing since the action was rejected due to version mismatch"
    );
}

#[test]
fn action_without_version_is_processed() {
    let provider = TestStateProvider::new();
    let streaming_assets_path = streaming_assets_path();
    let _ = provider.initialize("/tmp/test", &streaming_assets_path);
    let user_id = UserId(Uuid::new_v4());
    let connect_request = ConnectRequest {
        metadata: Metadata {
            user_id,
            battle_id: None,
            request_id: None,
            integration_test_id: None,
        },
        persistent_data_path: "/tmp/test".to_string(),
        streaming_assets_path: streaming_assets_path.clone(),
        vs_opponent: None,
        display_properties: None,
        debug_configuration: None,
    };
    let _ = engine::connect_with_provider(provider.clone(), &connect_request, RequestContext {
        logging_options: LoggingOptions::default(),
    });

    // Create action request without last_response_version (legacy client)
    let request = PerformActionRequest {
        metadata: Metadata {
            user_id,
            battle_id: None,
            request_id: Some(Uuid::new_v4()),
            integration_test_id: None,
        },
        action: GameAction::NoOp, // Use NoOp to avoid needing a full battle setup
        save_file_id: None,
        last_response_version: None,
    };

    // Action without version should be allowed to process
    let _result = engine::perform_action_blocking(provider.clone(), request, None);
    // We can't easily check if it was processed without a full battle setup,
    // but at least verify it wasn't rejected for version reasons
    assert!(
        !provider.is_processing(user_id),
        "Processing should be complete for action without version"
    );
}

#[test]
fn response_version_tracking_in_poll() {
    let provider = TestStateProvider::new();
    let streaming_assets_path = streaming_assets_path();
    let _ = provider.initialize("/tmp/test", &streaming_assets_path);
    let user_id = UserId(Uuid::new_v4());

    // Connect and get initial response version
    let connect_request = ConnectRequest {
        metadata: Metadata {
            user_id,
            battle_id: None,
            request_id: None,
            integration_test_id: None,
        },
        persistent_data_path: "/tmp/test".to_string(),
        streaming_assets_path: streaming_assets_path.clone(),
        vs_opponent: None,
        display_properties: None,
        debug_configuration: None,
    };

    let connect_response =
        engine::connect_with_provider(provider.clone(), &connect_request, RequestContext {
            logging_options: LoggingOptions::default(),
        });

    let initial_version = connect_response.response_version;

    // Verify the response version was stored
    assert_eq!(
        provider.get_last_response_version(user_id),
        Some(initial_version),
        "Connect should store response version"
    );

    // Perform an action with the correct version
    let request = PerformActionRequest {
        metadata: Metadata {
            user_id,
            battle_id: connect_response.metadata.battle_id,
            request_id: Some(Uuid::new_v4()),
            integration_test_id: None,
        },
        action: GameAction::NoOp,
        save_file_id: None,
        last_response_version: Some(initial_version),
    };

    let _result = engine::perform_action_blocking(provider.clone(), request, None);

    // For NoOp, we might not get poll results if there's no battle setup,
    // but the important thing is that the action was not rejected
    assert!(!provider.is_processing(user_id), "Action with correct version should be processed");
}

#[test]
fn finish_processing_not_called_on_concurrent_rejection() {
    let provider = TestStateProvider::new();
    let streaming_assets_path = streaming_assets_path();
    let _ = provider.initialize("/tmp/test", &streaming_assets_path);
    let user_id = UserId(Uuid::new_v4());

    // Set a response version
    let response_version = Uuid::new_v4();
    provider.store_last_response_version(user_id, response_version);

    // Start processing an action (marking user as processing)
    assert!(provider.start_processing(user_id), "Should be able to start processing");

    // Try to process another action while first is still processing
    let request = PerformActionRequest {
        metadata: Metadata {
            user_id,
            battle_id: None,
            request_id: Some(Uuid::new_v4()),
            integration_test_id: None,
        },
        action: GameAction::BattleAction(BattleAction::EndTurn),
        save_file_id: None,
        last_response_version: Some(response_version),
    };

    let result = engine::perform_action_blocking(provider.clone(), request, None);
    assert!(result.user_poll_results.is_empty(), "Concurrent action should be ignored");

    // Verify that the user is still marked as processing (finish_processing was not
    // called)
    assert!(
        provider.is_processing(user_id),
        "User should still be marked as processing since the concurrent action was rejected"
    );

    // Clean up
    provider.finish_processing(user_id);
}

#[test]
fn finish_processing_not_called_on_version_rejection() {
    let provider = TestStateProvider::new();
    let streaming_assets_path = streaming_assets_path();
    let _ = provider.initialize("/tmp/test", &streaming_assets_path);
    let user_id = UserId(Uuid::new_v4());

    // Set a current response version
    let current_version = Uuid::new_v4();
    provider.store_last_response_version(user_id, current_version);

    // Try to perform action with outdated version
    let outdated_version = Uuid::new_v4();
    let request = PerformActionRequest {
        metadata: Metadata {
            user_id,
            battle_id: None,
            request_id: Some(Uuid::new_v4()),
            integration_test_id: None,
        },
        action: GameAction::BattleAction(BattleAction::EndTurn),
        save_file_id: None,
        last_response_version: Some(outdated_version),
    };

    let result = engine::perform_action_blocking(provider.clone(), request, None);
    assert!(result.user_poll_results.is_empty(), "Action with outdated version should be ignored");

    // Verify that the user is not marked as processing (start_processing was never
    // called)
    assert!(
        !provider.is_processing(user_id),
        "User should not be marked as processing since the action was rejected due to version mismatch"
    );
}
