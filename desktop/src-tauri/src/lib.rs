mod ipc_client;

use ipc_client::send_daemon_request;
use serde_json::json;
use std::fs;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WindowEvent,
};

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

#[tauri::command]
async fn read_invite_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file {}: {}", path, e))
}

#[tauri::command]
async fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let toggle_item = MenuItem::with_id(app, "toggle", "Show/Hide innernet", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit innernet", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_item, &quit_item])?;
            let icon = tauri::include_image!("icons/32x32.png");

            let tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("innernet")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            let is_minimized = window.is_minimized().unwrap_or(false);
                            if is_visible && !is_minimized {
                                let _ = window.minimize();
                            } else {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            let is_minimized = window.is_minimized().unwrap_or(false);
                            if is_visible && !is_minimized {
                                let _ = window.minimize();
                            } else {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            app.manage(tray);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Minimize to taskbar/dock so user can easily restore window in GNOME/KDE
                api.prevent_close();
                let _ = window.minimize();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_networks,
            get_status,
            toggle_connect,
            get_peers,
            fetch_sync,
            install_invite,
            read_invite_file,
            hide_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running innernet desktop application");
}
