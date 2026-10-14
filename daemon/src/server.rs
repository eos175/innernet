use crate::protocol::{DaemonRequest, DaemonResponse};
use crate::service::DaemonService;
use anyhow::{Context, Result};
use std::{
    fs::{self, Permissions},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::broadcast,
};

pub const DEFAULT_SOCKET_PATH: &str = "/var/run/innernet/innernetd.sock";
pub const FALLBACK_SOCKET_PATH: &str = "/tmp/innernetd.sock";

/// Determine the appropriate socket path based on directory permissions.
pub fn resolve_socket_path(preferred_path: Option<&Path>) -> PathBuf {
    if let Some(path) = preferred_path {
        return path.to_path_buf();
    }

    let default_path = Path::new(DEFAULT_SOCKET_PATH);
    if let Some(parent) = default_path.parent() {
        if parent.exists() || fs::create_dir_all(parent).is_ok() {
            return default_path.to_path_buf();
        }
    }

    PathBuf::from(FALLBACK_SOCKET_PATH)
}

/// Run the IPC Unix domain socket server.
pub async fn run_server(
    socket_path: PathBuf,
    service: Arc<DaemonService>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    if let Some(parent) = socket_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create socket parent dir {}", parent.display()))?;
        }
    }

    if socket_path.exists() {
        let _ = fs::remove_file(&socket_path);
    }

    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("Failed to bind Unix socket at {}", socket_path.display()))?;

    // Allow desktop clients to read and write to the socket
    let _ = fs::set_permissions(&socket_path, Permissions::from_mode(0o666));

    log::info!("innernet daemon listening on {}", socket_path.display());

    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, _)) => {
                        let service_clone = service.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_client(stream, service_clone).await {
                                log::debug!("Client connection closed: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Error accepting connection: {}", e);
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                log::info!("Shutting down IPC server...");
                break;
            }
        }
    }

    if socket_path.exists() {
        let _ = fs::remove_file(&socket_path);
    }

    Ok(())
}

async fn handle_client(stream: UnixStream, service: Arc<DaemonService>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<DaemonRequest>(&line) {
            Ok(request) => dispatch_request(request, &service).await,
            Err(e) => DaemonResponse::Err(format!("Invalid request JSON: {}", e)),
        };

        let mut payload = serde_json::to_vec(&response)?;
        payload.push(b'\n');
        writer.write_all(&payload).await?;
        writer.flush().await?;
    }

    Ok(())
}

async fn dispatch_request(request: DaemonRequest, service: &DaemonService) -> DaemonResponse {
    match request {
        DaemonRequest::ListNetworks => match service.list_networks() {
            Ok(networks) => DaemonResponse::Ok(serde_json::to_value(networks).unwrap()),
            Err(e) => DaemonResponse::Err(e.to_string()),
        },
        DaemonRequest::GetStatus { interface } => match service.get_status(&interface) {
            Ok(status) => DaemonResponse::Ok(serde_json::to_value(status).unwrap()),
            Err(e) => DaemonResponse::Err(e.to_string()),
        },
        DaemonRequest::Up { interface } => match service.up(&interface).await {
            Ok(_) => DaemonResponse::Ok(serde_json::json!({ "success": true })),
            Err(e) => DaemonResponse::Err(e.to_string()),
        },
        DaemonRequest::Down { interface } => match service.down(&interface).await {
            Ok(_) => DaemonResponse::Ok(serde_json::json!({ "success": true })),
            Err(e) => DaemonResponse::Err(e.to_string()),
        },
        DaemonRequest::Fetch { interface } => match service.fetch_sync(&interface) {
            Ok(_) => DaemonResponse::Ok(serde_json::json!({ "success": true })),
            Err(e) => DaemonResponse::Err(e.to_string()),
        },
        DaemonRequest::GetPeers { interface } => match service.get_peers(&interface) {
            Ok(peers) => DaemonResponse::Ok(serde_json::to_value(peers).unwrap()),
            Err(e) => DaemonResponse::Err(e.to_string()),
        },
        DaemonRequest::InstallInvite {
            invite_toml,
            listen_port,
        } => match service.install_invite(&invite_toml, listen_port).await {
            Ok(iface) => DaemonResponse::Ok(serde_json::json!({ "interface": iface })),
            Err(e) => DaemonResponse::Err(e.to_string()),
        },
    }
}
