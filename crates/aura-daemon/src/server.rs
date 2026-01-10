//! IPC server - listens for events from hook handlers

use crate::registry::SessionRegistry;
use aura_common::{socket_path, IpcMessage, IpcResponse};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

/// Start the IPC server
pub async fn run(registry: Arc<Mutex<SessionRegistry>>) -> std::io::Result<()> {
    let path = socket_path();

    // Remove existing socket if present
    if path.exists() {
        std::fs::remove_file(&path)?;
    }

    let listener = UnixListener::bind(&path)?;
    eprintln!("aura: listening on {}", path.display());

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let registry = Arc::clone(&registry);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, registry).await {
                        eprintln!("aura: connection error: {e}");
                    }
                });
            }
            Err(e) => {
                eprintln!("aura: accept error: {e}");
            }
        }
    }
}

async fn handle_connection(
    stream: UnixStream,
    registry: Arc<Mutex<SessionRegistry>>,
) -> std::io::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Read one line (JSON message)
    let n = reader.read_line(&mut line).await?;
    if n == 0 {
        return Ok(()); // EOF
    }

    // Parse message
    let response = match serde_json::from_str::<IpcMessage>(&line) {
        Ok(IpcMessage::Event(event)) => {
            let mut reg = registry.lock().await;
            reg.process_event(event);
            IpcResponse::Ok
        }
        Ok(IpcMessage::Ping) => IpcResponse::Pong,
        Err(e) => IpcResponse::Error {
            message: format!("invalid message: {e}"),
        },
    };

    // Send response
    let response_json = serde_json::to_string(&response).unwrap();
    writer.write_all(response_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    Ok(())
}
