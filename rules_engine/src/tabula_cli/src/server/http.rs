use std::collections::HashMap;
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
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::sync::{Mutex, oneshot};

use crate::server::listener_runner::{self, ListenerContext};
use crate::server::listeners::conditional_formatting::ConditionalFormattingListener;
use crate::server::listeners::ensure_uuid::EnsureUuidListener;
use crate::server::listeners::fluent_rules_text::FluentRulesTextListener;
use crate::server::listeners::partial_formatting::PartialFormattingListener;
use crate::server::model::{Response, ResponseStatus};
use crate::server::server_config::ServerConfig;
use crate::server::{serialization, server_workbook_snapshot};

pub async fn serve(config: ServerConfig) -> Result<()> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let state = Arc::new(ServerState {
        once: config.once,
        shutdown: Mutex::new(Some(shutdown_tx)),
        cache: Mutex::new(HashMap::new()),
    });
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

async fn handle_notify(State(state): State<Arc<ServerState>>, body: Bytes) -> impl IntoResponse {
    let response = match serialization::parse_request(&body) {
        Ok(request) => {
            let cache_key =
                (request.workbook_path.clone(), request.workbook_mtime, request.workbook_size);
            let mut cache = state.cache.lock().await;
            if let Some(cached_response) = cache.get(&cache_key) {
                return (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                    cached_response.clone(),
                );
            }
            let workbook_path = request.workbook_path.clone();
            let expected_metadata = server_workbook_snapshot::FileMetadata {
                size: request.workbook_size,
                mtime: request.workbook_mtime,
            };
            let snapshot_result = tokio::task::spawn_blocking(move || {
                let path = std::path::Path::new(&workbook_path);
                server_workbook_snapshot::read_snapshot(path, Some(expected_metadata))
            })
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!("Task join error: {}", e)));
            match snapshot_result {
                Ok(snapshot) => {
                    let context = ListenerContext {
                        request_id: request.request_id.clone(),
                        workbook_path: request.workbook_path.clone(),
                        changed_range: request.changed_range.clone(),
                    };
                    let listeners = build_listeners();
                    let listener_result =
                        listener_runner::run_listeners(&listeners, &snapshot, &context);
                    let changeset_id = serialization::compute_changeset_id(
                        &request.workbook_path,
                        request.workbook_mtime,
                        request.workbook_size,
                        &listener_result.changes,
                    );
                    let response = Response {
                        request_id: Some(request.request_id),
                        status: ResponseStatus::Ok,
                        retry_after_ms: None,
                        warnings: listener_result.warnings,
                        changes: listener_result.changes,
                        changeset_id: Some(changeset_id.clone()),
                    };
                    let serialized = serialization::serialize_response(&response);
                    cache.insert(cache_key, serialized.clone());
                    response
                }
                Err(e) => Response {
                    request_id: Some(request.request_id),
                    status: ResponseStatus::Error,
                    retry_after_ms: Some(1000),
                    warnings: vec![format!("Failed to read workbook: {}", e)],
                    changes: vec![],
                    changeset_id: None,
                },
            }
        }
        Err(e) => Response {
            request_id: None,
            status: ResponseStatus::Error,
            retry_after_ms: Some(1000),
            warnings: vec![format!("Failed to parse request: {}", e)],
            changes: vec![],
            changeset_id: None,
        },
    };
    if state.once
        && let Some(sender) = state.shutdown.lock().await.take()
    {
        let _ = sender.send(());
    }
    let serialized = serialization::serialize_response(&response);
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/plain; charset=utf-8")], serialized)
}

async fn shutdown_signal(once: bool, shutdown_rx: Receiver<()>) {
    if once {
        let _ = shutdown_rx.await;
    } else {
        future::pending::<()>().await;
    }
}

fn build_listeners() -> Vec<Box<dyn super::listener_runner::Listener>> {
    let mut listeners: Vec<Box<dyn super::listener_runner::Listener>> = vec![
        Box::new(ConditionalFormattingListener),
        Box::new(PartialFormattingListener),
        Box::new(EnsureUuidListener),
    ];
    match FluentRulesTextListener::new() {
        Ok(listener) => {
            listeners.push(Box::new(listener));
        }
        Err(e) => {
            eprintln!("Failed to initialize FluentRulesTextListener: {e}");
        }
    }
    listeners
}

struct ServerState {
    once: bool,
    shutdown: Mutex<Option<Sender<()>>>,
    cache: Mutex<HashMap<(String, i64, u64), String>>,
}
