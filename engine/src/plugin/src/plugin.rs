#![allow(clippy::missing_safety_doc)] // You only live once, that's the motto - Drake

use std::panic::{self, UnwindSafe};

use anyhow::Result;
use display_data::request_data::{ConnectRequest, PerformActionRequest};
use engine::test_data;

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
    println!("connect: {:?}", deserialized_request.metadata.user_id);
    let scene = test_data::connect(&deserialized_request);
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
    println!("perform_action: {:?}", deserialized_request.metadata.user_id);
    let scene = test_data::perform_action(&deserialized_request);
    let json = serde_json::to_string(&scene)?;
    let json_bytes = json.as_bytes();

    if json_bytes.len() > response_length as usize {
        return Err(anyhow::anyhow!("Response buffer too small"));
    }

    let out = std::slice::from_raw_parts_mut(response, response_length as usize);
    out[..json_bytes.len()].copy_from_slice(json_bytes);
    Ok(json_bytes.len() as i32)
}

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
