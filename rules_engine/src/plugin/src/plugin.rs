#![allow(clippy::missing_safety_doc)] // You only live once, that's the motto - Drake

use std::panic::{self, UnwindSafe};
use std::path::PathBuf;
use std::sync::LazyLock;

use anyhow::Result;
use battle_state::battle::battle_state::{LoggingOptions, RequestContext};
use display_data::client_log_request::ClientLogRequest;
use display_data::command::CommandSequence;
use display_data::request_data::{
    ConnectRequest, PerformActionRequest, PerformActionResponse, PollRequest, PollResponse,
    PollResponseType,
};
use rules_engine::state_provider::DefaultStateProvider;
use rules_engine::{client_logging, engine};
use tokio::runtime::Runtime;

static TOKIO_RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("Failed to create tokio runtime"));

/// Synchronize the state of an ongoing game, downloading a full description of
/// the game state.
///
/// `request` should be a buffer including the json serialization of a
/// `ConnectRequest` message of `request_length` bytes. `response` should be an
/// empty buffer of `response_length` bytes, this buffer will be populated with
/// a json-serialized `ConnectResponse` describing the current state of the
/// game.
///
/// Returns the number of bytes written to the `response` buffer, or -1 on
/// error.
#[no_mangle]
pub unsafe extern "C" fn dreamtides_connect(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> i32 {
    error_boundary(|| connect_impl(request, request_length, response, response_length))
}

unsafe fn connect_impl(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> Result<i32> {
    let request_data = std::slice::from_raw_parts(request, request_length as usize);
    let deserialized_request = serde_json::from_slice::<ConnectRequest>(request_data)?;
    let context = RequestContext {
        logging_options: LoggingOptions {
            log_directory: Some(PathBuf::from(&deserialized_request.persistent_data_path)),
            log_ai_search_diagram: false,
        },
    };
    logging::maybe_initialize(&context);
    let scene = engine::connect(DefaultStateProvider, &deserialized_request, context);
    let json = serde_json::to_string(&scene)?;
    let json_bytes = json.as_bytes();

    if json_bytes.len() > response_length as usize {
        return Err(anyhow::anyhow!("Response buffer too small"));
    }

    let out = std::slice::from_raw_parts_mut(response, response_length as usize);
    out[..json_bytes.len()].copy_from_slice(json_bytes);
    Ok(json_bytes.len() as i32)
}

/// Performs a given game action.
///
/// `request` should be a buffer including the json serialization of a
/// `PerformActionRequest` message of `request_length` bytes. `response` should
/// be an empty buffer of `response_length` bytes, this buffer will be populated
/// with a json-serialized `PerformActionResponse` describing the result of
/// performing this action.
///
/// Returns the number of bytes written to the `response` buffer, or -1 on
/// error.
#[no_mangle]
pub unsafe extern "C" fn dreamtides_perform_action(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> i32 {
    error_boundary(|| perform_impl(request, request_length, response, response_length))
}

unsafe fn perform_impl(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> Result<i32> {
    let request_data = std::slice::from_raw_parts(request, request_length as usize);
    let deserialized_request = serde_json::from_slice::<PerformActionRequest>(request_data)?;
    let metadata = deserialized_request.metadata;

    TOKIO_RUNTIME.spawn(async move {
        engine::perform_action(DefaultStateProvider, deserialized_request);
    });

    // Currently we do not return any commands from the perform action call, but
    // maybe one day we will as an optimization.
    let empty_commands = PerformActionResponse { metadata, commands: CommandSequence::default() };
    let json = serde_json::to_string(&empty_commands)?;
    let json_bytes = json.as_bytes();

    if json_bytes.len() > response_length as usize {
        return Err(anyhow::anyhow!("Response buffer too small"));
    }

    let out = std::slice::from_raw_parts_mut(response, response_length as usize);
    out[..json_bytes.len()].copy_from_slice(json_bytes);
    Ok(json_bytes.len() as i32)
}

/// Polls for pending updates for a user.
///
/// `request` should be a buffer including the json serialization of a
/// `PollRequest` message of `request_length` bytes. `response` should be an
/// empty buffer of `response_length` bytes, this buffer will be populated with
/// a json-serialized `PollResponse` describing any pending updates for the
/// user.
///
/// Returns the number of bytes written to the `response` buffer, or -1 on
/// error.
#[no_mangle]
pub unsafe extern "C" fn dreamtides_poll(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> i32 {
    error_boundary(|| poll_impl(request, request_length, response, response_length))
}

unsafe fn poll_impl(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> Result<i32> {
    let request_data = std::slice::from_raw_parts(request, request_length as usize);
    let deserialized_request = serde_json::from_slice::<PollRequest>(request_data)?;
    let user_id = deserialized_request.metadata.user_id;

    let response_data = match engine::poll(DefaultStateProvider, user_id) {
        Some(response) => response,
        None => PollResponse {
            metadata: deserialized_request.metadata,
            commands: None,
            response_type: PollResponseType::Final,
            response_version: None,
        },
    };

    let json = serde_json::to_string(&response_data)?;
    let json_bytes = json.as_bytes();

    if json_bytes.len() > response_length as usize {
        return Err(anyhow::anyhow!("Response buffer too small"));
    }

    let out = std::slice::from_raw_parts_mut(response, response_length as usize);
    out[..json_bytes.len()].copy_from_slice(json_bytes);
    Ok(json_bytes.len() as i32)
}

/// Logs events from the client.
///
/// `request` should be a buffer including the json serialization of a
/// `ClientLogRequest` message of `request_length` bytes.
///
/// Returns 0 on success, or -1 on error.
#[no_mangle]
pub unsafe extern "C" fn dreamtides_log(request: *const u8, request_length: i32) -> i32 {
    error_boundary(|| log_impl(request, request_length))
}

unsafe fn log_impl(request: *const u8, request_length: i32) -> Result<i32> {
    let request_data = std::slice::from_raw_parts(request, request_length as usize);
    let deserialized_request = serde_json::from_slice::<ClientLogRequest>(request_data)?;
    client_logging::log_client_events(deserialized_request);
    Ok(0)
}

#[expect(clippy::print_stderr)]
unsafe fn error_boundary(function: impl FnOnce() -> Result<i32> + UnwindSafe) -> i32 {
    panic::catch_unwind(|| match function() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("PANIC: {e:?}");
            -1
        }
    })
    .unwrap_or(-1)
}
