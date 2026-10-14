mod ipc_client;

use ipc_client::send_daemon_request;
use serde_json::json;

#[tauri::command]
async fn get_networks() -> Result<serde_json::Value, String> {
    send_daemon_request(json!({
        "action": "list_networks"
    }))
    .await
}

#[tauri::command]
async fn get_status(interface: String) -> Result<serde_json::Value, String> {
    send_daemon_request(json!({
        "action": "get_status",
        "payload": {
            "interface": interface
        }
    }))
    .await
}

#[tauri::command]
async fn toggle_connect(interface: String, enable: bool) -> Result<serde_json::Value, String> {
    let action = if enable { "up" } else { "down" };
    send_daemon_request(json!({
        "action": action,
        "payload": {
            "interface": interface
        }
    }))
    .await
}

#[tauri::command]
async fn get_peers(interface: String) -> Result<serde_json::Value, String> {
    send_daemon_request(json!({
        "action": "get_peers",
        "payload": {
            "interface": interface
        }
    }))
    .await
}

#[tauri::command]
async fn fetch_sync(interface: String) -> Result<serde_json::Value, String> {
    send_daemon_request(json!({
        "action": "fetch",
        "payload": {
            "interface": interface
        }
    }))
    .await
}

#[tauri::command]
async fn install_invite(invite_toml: String) -> Result<serde_json::Value, String> {
    send_daemon_request(json!({
        "action": "install_invite",
        "payload": {
            "invite_toml": invite_toml,
            "listen_port": serde_json::Value::Null
        }
    }))
    .await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_networks,
            get_status,
            toggle_connect,
            get_peers,
            fetch_sync,
            install_invite
        ])
        .run(tauri::generate_context!())
        .expect("error while running innernet desktop application");
}
