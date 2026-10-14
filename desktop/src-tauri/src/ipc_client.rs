use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const PRIMARY_SOCKET_PATH: &str = "/var/run/innernet/innernetd.sock";
const FALLBACK_SOCKET_PATH: &str = "/tmp/innernetd.sock";

/// Locate the daemon socket.
pub fn get_socket_path() -> PathBuf {
    if Path::new(PRIMARY_SOCKET_PATH).exists() {
        PathBuf::from(PRIMARY_SOCKET_PATH)
    } else {
        PathBuf::from(FALLBACK_SOCKET_PATH)
    }
}

/// Send a JSON-RPC request to the daemon over the Unix domain socket and receive the response.
pub async fn send_daemon_request(request: serde_json::Value) -> Result<serde_json::Value, String> {
    let socket_path = get_socket_path();

    if !socket_path.exists() {
        return Err(format!(
            "innernet daemon is not running. Expected socket at {} or {}",
            PRIMARY_SOCKET_PATH, FALLBACK_SOCKET_PATH
        ));
    }

    let stream = UnixStream::connect(&socket_path)
        .await
        .map_err(|e| format!("Failed to connect to innernet daemon socket: {}", e))?;

    let (reader, mut writer) = stream.into_split();

    let mut payload = serde_json::to_vec(&request)
        .map_err(|e| format!("Failed to serialize request: {}", e))?;
    payload.push(b'\n');

    writer
        .write_all(&payload)
        .await
        .map_err(|e| format!("Failed to send request to daemon: {}", e))?;
    writer
        .flush()
        .await
        .map_err(|e| format!("Failed to flush stream: {}", e))?;

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines
        .next_line()
        .await
        .map_err(|e| format!("Failed to read response from daemon: {}", e))?
        .context("Daemon closed connection unexpectedly")
        .map_err(|e| e.to_string())?;

    let response_val: serde_json::Value = serde_json::from_str(&response_line)
        .map_err(|e| format!("Invalid JSON response from daemon: {}", e))?;

    if let Some(status) = response_val.get("status").and_then(|s| s.as_str()) {
        if status == "ok" {
            Ok(response_val.get("data").cloned().unwrap_or(serde_json::Value::Null))
        } else {
            let err_msg = response_val
                .get("data")
                .and_then(|d| d.as_str())
                .unwrap_or("Unknown daemon error");
            Err(err_msg.to_string())
        }
    } else {
        Err("Malformed daemon response format".to_string())
    }
}
