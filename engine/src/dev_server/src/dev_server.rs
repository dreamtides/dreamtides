use axum::extract::Json;
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};

// Represents the request for "/connect" (no fields)
#[derive(Deserialize)]
struct ConnectionRequest {}

// The response for "/connect" with a fixed number
#[derive(Serialize)]
struct ConnectionResponse {
    number: i32,
}

// The request for "/handle_action", containing a single String
#[derive(Deserialize)]
struct ActionRequest {
    payload: String,
}

// The response for "/handle_action", containing the length of the string
#[derive(Serialize)]
struct ActionResponse {
    length: usize,
}

// Handler for GET /connect
// Although we defined ConnectionRequest, for a GET endpoint we won't parse an
// empty body.
async fn connect() -> Json<ConnectionResponse> {
    Json(ConnectionResponse { number: 4 })
}

// Handler for POST /handle_action
// Axum will deserialize the JSON body into ActionRequest automatically.
async fn handle_action(Json(req): Json<ActionRequest>) -> Json<ActionResponse> {
    Json(ActionResponse { length: req.payload.len() })
}

#[tokio::main]
async fn main() {
    println!("Starting server on port 26598");
    let app =
        Router::new().route("/connect", get(connect)).route("/handle_action", post(handle_action));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:26598").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
