use std::fmt;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use display_data::command::CommandSequence;
use display_data::request_data::{
    ConnectRequest, ConnectResponse, PerformActionRequest, PerformActionResponse, PollRequest,
    PollResponse,
};
use rules_engine::engine;
use serde::de::DeserializeOwned;
use tracing::{error, info, info_span};

// Custom error type for better error handling
pub enum AppError {
    BadRequest(String),
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        error!(%status, error.message = %message, "Request error");
        (status, message).into_response()
    }
}

type AppResult<T> = Result<T, AppError>;

fn parse_json<T: DeserializeOwned>(json_str: &str) -> AppResult<T> {
    serde_json::from_str(json_str).map_err(|e| {
        error!(input = %json_str, error = %e, "JSON parsing error");
        AppError::BadRequest(format!("Invalid JSON: {}\nInput: {}", e, json_str))
    })
}

async fn connect(body: String) -> AppResult<Json<ConnectResponse>> {
    println!();

    let req: ConnectRequest = parse_json(&body)?;
    let user_id = req.metadata.user_id;

    if let Some(scenario) = req.test_scenario.as_ref() {
        info!(?user_id, ?scenario, "Got connect request");
        Ok(Json(client_test_scenarios::connect(&req, scenario)))
    } else {
        info!(?user_id, "Got connect request");
        Ok(Json(engine::connect(&req)))
    }
}

async fn perform_action(body: String) -> AppResult<Json<PerformActionResponse>> {
    println!();

    let req: PerformActionRequest = parse_json(&body)?;
    let action = req.action;
    let user_id = req.metadata.user_id;

    let _span = info_span!("perform_action", ?action);
    if let Some(scenario) = req.test_scenario.as_ref() {
        info!(?action, ?scenario, ?user_id, "Got perform action request");
        Ok(Json(client_test_scenarios::perform_action(&req, scenario)))
    } else {
        info!(?action, ?user_id, "Got perform action request");
        let metadata = req.metadata;
        engine::perform_action(req);
        Ok(Json(PerformActionResponse { metadata, commands: CommandSequence::default() }))
    }
}

async fn poll(body: String) -> AppResult<Json<PollResponse>> {
    let req: PollRequest = parse_json(&body)?;
    let user_id = req.metadata.user_id;
    let commands = engine::poll(user_id);
    Ok(Json(PollResponse { metadata: req.metadata, commands }))
}

#[tokio::main]
async fn main() {
    logging::initialize();
    info!("Starting server on port 26598");

    let app = Router::new()
        .route("/connect", get(connect))
        .route("/perform_action", post(perform_action))
        .route("/poll", get(poll));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:26598").await.unwrap_or_else(|e| {
        error!(error.message = %e, "Failed to bind to port 26598");
        panic!("Server initialization failed: {}", e);
    });

    info!("Server running on http://0.0.0.0:26598");
    axum::serve(listener, app).await.unwrap_or_else(|e| {
        error!(error.message = %e, "Server error");
        panic!("Server error: {}", e);
    });
}
