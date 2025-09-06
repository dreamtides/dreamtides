#![allow(clippy::missing_safety_doc)] // You only live once, that's the motto - Drake

use std::backtrace::Backtrace;
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
use logging::android_logging;
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreamtides_connect(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> i32 {
    unsafe { error_boundary(|| connect_impl(request, request_length, response, response_length)) }
}

unsafe fn connect_impl(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> Result<i32> {
    println!("Connecting to plugin...");
    android_logging::write_to_logcat("Connecting to plugin...");

    let request_data = unsafe { std::slice::from_raw_parts(request, request_length as usize) };
    let deserialized_request = serde_json::from_slice::<ConnectRequest>(request_data)?;
    let context = RequestContext {
        logging_options: LoggingOptions {
            log_directory: Some(PathBuf::from(&deserialized_request.persistent_data_path)),
            log_ai_search_diagram: false,
            enable_action_legality_check: true,
        },
    };

    logging::maybe_initialize(&context);

    let scene = engine::connect(&deserialized_request, context);
    let json = serde_json::to_string(&scene)?;
    let json_bytes = json.as_bytes();

    if json_bytes.len() > response_length as usize {
        return Err(anyhow::anyhow!("Response buffer too small"));
    }

    let out = unsafe { std::slice::from_raw_parts_mut(response, response_length as usize) };
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreamtides_perform_action(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> i32 {
    unsafe { error_boundary(|| perform_impl(request, request_length, response, response_length)) }
}

unsafe fn perform_impl(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> Result<i32> {
    let request_data = unsafe { std::slice::from_raw_parts(request, request_length as usize) };
    let deserialized_request = serde_json::from_slice::<PerformActionRequest>(request_data)?;
    let metadata = deserialized_request.metadata;

    let response_data = {
        TOKIO_RUNTIME.spawn(async move {
            engine::perform_action(deserialized_request);
        });

        // Currently we do not return any commands from the perform action call, but
        // maybe one day we will as an optimization.
        PerformActionResponse { metadata, commands: CommandSequence::default() }
    };

    let json = serde_json::to_string(&response_data)?;
    let json_bytes = json.as_bytes();

    if json_bytes.len() > response_length as usize {
        return Err(anyhow::anyhow!("Response buffer too small"));
    }

    let out = unsafe { std::slice::from_raw_parts_mut(response, response_length as usize) };
    out[..json_bytes.len()].copy_from_slice(json_bytes);
    Ok(json_bytes.len() as i32)
}

/// Checks whether any updates are available to retreive from the poll()
/// function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreamtides_has_updates() -> bool {
    engine::any_user_has_updates()
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreamtides_poll(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> i32 {
    unsafe { error_boundary(|| poll_impl(request, request_length, response, response_length)) }
}

unsafe fn poll_impl(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> Result<i32> {
    let request_data = unsafe { std::slice::from_raw_parts(request, request_length as usize) };
    let deserialized_request = serde_json::from_slice::<PollRequest>(request_data)?;
    let user_id = deserialized_request.metadata.user_id;

    let response_data = match engine::poll(user_id, deserialized_request.metadata) {
        Some(response) => response,
        None => PollResponse {
            metadata: deserialized_request.metadata,
            commands: None,
            response_type: PollResponseType::None,
            response_version: None,
        },
    };

    let json = serde_json::to_string(&response_data)?;
    let json_bytes = json.as_bytes();

    if json_bytes.len() > response_length as usize {
        return Err(anyhow::anyhow!("Response buffer too small"));
    }

    let out = unsafe { std::slice::from_raw_parts_mut(response, response_length as usize) };
    out[..json_bytes.len()].copy_from_slice(json_bytes);
    Ok(json_bytes.len() as i32)
}

/// Logs events from the client.
///
/// `request` should be a buffer including the json serialization of a
/// `ClientLogRequest` message of `request_length` bytes.
///
/// Returns 0 on success, or -1 on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dreamtides_log(request: *const u8, request_length: i32) -> i32 {
    unsafe { error_boundary(|| log_impl(request, request_length)) }
}

unsafe fn log_impl(request: *const u8, request_length: i32) -> Result<i32> {
    let request_data = unsafe { std::slice::from_raw_parts(request, request_length as usize) };
    let deserialized_request = serde_json::from_slice::<ClientLogRequest>(request_data)?;
    client_logging::log_client_events(deserialized_request);
    Ok(0)
}

#[expect(clippy::print_stderr)]
unsafe fn error_boundary(function: impl FnOnce() -> Result<i32> + UnwindSafe) -> i32 {
    let result = panic::catch_unwind(|| match function() {
        Ok(i) => i,
        Err(e) => {
            let mut chain = String::new();
            chain.push_str(&format!("{e}"));
            let mut source_opt = e.source();
            while let Some(src) = source_opt {
                chain.push_str(&format!(" | caused by: {src}"));
                source_opt = src.source();
            }
            android_logging::write_to_logcat(format!("ERROR: {chain}"));
            eprintln!("ERROR: {chain}");
            -1
        }
    });

    match result {
        Ok(value) => value,
        Err(panic_payload) => {
            let panic_msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                *s
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.as_str()
            } else {
                "Unknown panic payload"
            };
            let bt = Backtrace::capture();
            let log_msg = format!("PANIC: {panic_msg}\nBacktrace:\n{bt}");
            android_logging::write_to_logcat(log_msg.clone());
            eprintln!("{log_msg}");
            -1
        }
    }
}
