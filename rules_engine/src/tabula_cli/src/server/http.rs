use std::future;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::{Router, routing};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, oneshot};

use super::ServerConfig;

pub async fn serve(config: ServerConfig) -> Result<()> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let state =
        Arc::new(ServerState { once: config.once, shutdown: Mutex::new(Some(shutdown_tx)) });
    let app = Router::new()
        .route("/notify", routing::post(handle_notify))
        .with_state(state)
        .layer(DefaultBodyLimit::max(config.max_payload_bytes));
    let addr = SocketAddr::new(config.host, config.port);
    let listener = TcpListener::bind(addr).await.with_context(|| format!("Cannot bind {addr}"))?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(config.once, shutdown_rx))
        .await
        .with_context(|| format!("Server failed on {addr}"))?;
    Ok(())
}

async fn handle_notify(State(state): State<Arc<ServerState>>, _body: Bytes) -> impl IntoResponse {
    if state.once {
        if let Some(sender) = state.shutdown.lock().await.take() {
            let _ = sender.send(());
        }
    }
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/plain; charset=utf-8")], FIXED_RESPONSE)
}

async fn shutdown_signal(once: bool, shutdown_rx: oneshot::Receiver<()>) {
    if once {
        let _ = shutdown_rx.await;
    } else {
        future::pending::<()>().await;
    }
}

struct ServerState {
    once: bool,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
}

const FIXED_RESPONSE: &str = "TABULA/1\nSTATUS ok\n";
