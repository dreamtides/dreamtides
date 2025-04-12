use axum::extract::Json;
use axum::routing::{get, post};
use axum::Router;
use display_data::request_data::{
    ConnectRequest, ConnectResponse, PerformActionRequest, PerformActionResponse,
};
use rules_engine::engine;

async fn connect(Json(req): Json<ConnectRequest>) -> Json<ConnectResponse> {
    if let Some(scenario) = req.test_scenario.as_ref() {
        println!("Got connect request: {:?} with scenario: {}", req, scenario);
        Json(client_test_scenarios::connect(&req, scenario))
    } else {
        println!("Got connect request: {:?}", req);
        Json(engine::connect(&req))
    }
}

async fn perform_action(Json(req): Json<PerformActionRequest>) -> Json<PerformActionResponse> {
    if let Some(scenario) = req.test_scenario.as_ref() {
        println!("Got perform action request: {:?} with scenario: {}", req, scenario);
        Json(client_test_scenarios::perform_action(&req, scenario))
    } else {
        println!("Got perform action request: {:?}", req);
        Json(engine::perform_action(&req))
    }
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
