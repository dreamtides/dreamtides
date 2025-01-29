use axum::extract::Json;
use axum::routing::{get, post};
use axum::Router;
use display_data::request_data::{
    ConnectRequest, ConnectResponse, PerformActionRequest, PerformActionResponse,
};
use engine::test_data;

async fn connect(Json(req): Json<ConnectRequest>) -> Json<ConnectResponse> {
    println!("Got connect request: {:?}", req);
    Json(test_data::connect(&req))
}

async fn perform_action(Json(req): Json<PerformActionRequest>) -> Json<PerformActionResponse> {
    println!("Got perform action request: {:?}", req);
    Json(test_data::perform_action(&req))
}

#[tokio::main]
async fn main() {
    println!("Starting server on port 26598");
    let app = Router::new()
        .route("/connect", get(connect))
        .route("/perform_action", post(perform_action));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:26598").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
