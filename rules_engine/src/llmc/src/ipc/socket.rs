use std::fs;
use std::os::unix::net::UnixListener as StdUnixListener;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc::{self, Receiver};
use tokio::time::timeout;

use crate::config;
use crate::ipc::messages::{HookEvent, HookMessage, HookResponse};

const READ_TIMEOUT: Duration = Duration::from_secs(5);

pub struct IpcListener {
    listener: UnixListener,
    socket_path: PathBuf,
}

pub fn get_socket_path() -> PathBuf {
    config::get_llmc_root().join("llmc.sock")
}

/// Returns the socket path for remediation IPC communication.
///
/// This is separate from the main daemon socket to prevent conflicts when
/// remediation runs while daemon state might still exist on disk.
pub fn get_remediation_socket_path() -> PathBuf {
    config::get_llmc_root().join("llmc_remediation.sock")
}

pub async fn handle_connection(stream: UnixStream) -> Result<Option<HookMessage>> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    let read_result = timeout(READ_TIMEOUT, reader.read_line(&mut line)).await;

    let message: HookMessage = match read_result {
        Ok(Ok(0)) => {
            return Ok(None);
        }
        Ok(Ok(_)) => serde_json::from_str(line.trim())
            .with_context(|| format!("Failed to parse hook message: {}", line.trim()))?,
        Ok(Err(e)) => {
            let response = HookResponse::error(format!("Read error: {}", e));
            let _ = writer.write_all(serde_json::to_string(&response)?.as_bytes()).await;
            let _ = writer.write_all(b"\n").await;
            return Err(e.into());
        }
        Err(_) => {
            let response = HookResponse::error("Read timeout");
            let _ = writer.write_all(serde_json::to_string(&response)?.as_bytes()).await;
            let _ = writer.write_all(b"\n").await;
            return Err(anyhow::anyhow!("Read timeout"));
        }
    };

    let response = HookResponse::success();
    writer.write_all(serde_json::to_string(&response)?.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    Ok(Some(message))
}

pub async fn send_event(socket_path: &Path, event: HookEvent) -> Result<HookResponse> {
    let stream = UnixStream::connect(socket_path)
        .await
        .with_context(|| format!("Failed to connect to socket: {}", socket_path.display()))?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let message = HookMessage::new(event);
    let json = serde_json::to_string(&message)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    let mut response_line = String::new();
    let read_result = timeout(READ_TIMEOUT, reader.read_line(&mut response_line)).await;

    match read_result {
        Ok(Ok(0)) => Err(anyhow::anyhow!("Connection closed before response")),
        Ok(Ok(_)) => {
            let response: HookResponse = serde_json::from_str(response_line.trim())?;
            Ok(response)
        }
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Err(anyhow::anyhow!("Response timeout")),
    }
}

pub fn spawn_ipc_listener(socket_path: PathBuf) -> Result<Receiver<HookMessage>> {
    let (tx, rx) = mpsc::channel::<HookMessage>(100);

    let listener = IpcListener::bind(&socket_path)?;

    tokio::spawn(async move {
        tracing::info!("IPC listener task started");
        loop {
            match listener.accept().await {
                Ok(stream) => {
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        match handle_connection(stream).await {
                            Ok(Some(msg)) => {
                                tracing::debug!("Received hook event: {:?}", msg.event);
                                if tx.send(msg).await.is_err() {
                                    tracing::error!(
                                        "Failed to send hook message to channel (receiver dropped) \
                                         - daemon may be shutting down. Hook event will be lost."
                                    );
                                }
                            }
                            Ok(None) => {
                                tracing::debug!("Connection closed without message");
                            }
                            Err(e) => {
                                tracing::info!(
                                    error = %e,
                                    "Error handling IPC connection (this is usually transient \
                                     and can be ignored unless it happens repeatedly)"
                                );
                            }
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to accept connection: {}", e);
                }
            }
        }
    });

    Ok(rx)
}

impl IpcListener {
    pub fn bind(socket_path: &Path) -> Result<Self> {
        if socket_path.exists() {
            fs::remove_file(socket_path).with_context(|| {
                format!("Failed to remove existing socket: {}", socket_path.display())
            })?;
        }

        if let Some(parent) = socket_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create socket directory: {}", parent.display())
            })?;
        }

        let listener = StdUnixListener::bind(socket_path)
            .with_context(|| format!("Failed to bind socket: {}", socket_path.display()))?;
        listener.set_nonblocking(true)?;
        let listener = UnixListener::from_std(listener)?;

        tracing::info!("IPC listener bound to {}", socket_path.display());

        Ok(Self { listener, socket_path: socket_path.to_path_buf() })
    }

    pub async fn accept(&self) -> Result<UnixStream> {
        let (stream, _addr) = self.listener.accept().await?;
        Ok(stream)
    }
}

impl Drop for IpcListener {
    fn drop(&mut self) {
        if self.socket_path.exists() {
            let _ = fs::remove_file(&self.socket_path);
        }
    }
}
