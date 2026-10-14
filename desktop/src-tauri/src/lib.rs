mod ipc_client;

use ipc_client::send_daemon_request;
use serde_json::json;
use std::fs;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_notification::NotificationExt;

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

#[tauri::command]
async fn minimize_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.minimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn drag_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.start_dragging().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn ping_peer(ip: String) -> Result<f64, String> {
    let res = send_daemon_request(json!({
        "action": "ping",
        "payload": {
            "ip": ip
        }
    }))
    .await?;

    res.as_f64()
        .ok_or_else(|| "Invalid latency response from daemon".to_string())
}

#[tauri::command]
async fn is_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_autostart(app: AppHandle, enable: bool) -> Result<bool, String> {
    let autolaunch = app.autolaunch();
    if enable {
        autolaunch.enable().map_err(|e| e.to_string())?;
    } else {
        autolaunch.disable().map_err(|e| e.to_string())?;
    }
    autolaunch.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_tray_status(
    app: AppHandle,
    is_connected: bool,
    interface: Option<String>,
    ip: Option<String>,
) -> Result<(), String> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let (icon, tooltip) = if is_connected {
            let iface_str = interface.unwrap_or_else(|| "Connected".to_string());
            let ip_str = ip.map(|i| format!(" ({})", i)).unwrap_or_default();
            (
                tauri::include_image!("icons/32x32.png"),
                format!("innernet - {}{}", iface_str, ip_str),
            )
        } else {
            (
                tauri::include_image!("icons/32x32-disconnected.png"),
                "innernet - Disconnected".to_string(),
            )
        };

        tray.set_icon(Some(icon)).map_err(|e| e.to_string())?;
        tray.set_tooltip(Some(tooltip)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn send_desktop_notification(
    app: AppHandle,
    title: String,
    body: String,
) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            // If launched with --minimized (from autostart), start hidden in tray
            if std::env::args().any(|arg| arg == "--minimized") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            let connect_item = MenuItem::with_id(app, "toggle_connect", "Toggle Connection", true, None::<&str>)?;
            let toggle_item = MenuItem::with_id(app, "toggle", "Show/Hide innernet", true, None::<&str>)?;
            let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
            let autostart_title = if autostart_enabled {
                "Start on Boot: Enabled"
            } else {
                "Start on Boot: Disabled"
            };
            let autostart_item = MenuItem::with_id(app, "toggle_autostart", autostart_title, true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit innernet", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&connect_item, &toggle_item, &autostart_item, &quit_item])?;
            let icon = tauri::include_image!("icons/32x32.png");

            let tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("innernet")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle_connect" => {
                        let _ = app.emit("tray-toggle-connect", ());
                    }
                    "toggle_autostart" => {
                        let autolaunch = app.autolaunch();
                        let currently_enabled = autolaunch.is_enabled().unwrap_or(false);
                        if currently_enabled {
                            let _ = autolaunch.disable();
                            log::info!("Autostart on boot disabled");
                        } else {
                            let _ = autolaunch.enable();
                            log::info!("Autostart on boot enabled");
                        }
                    }
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
            hide_window,
            minimize_window,
            drag_window,
            is_autostart_enabled,
            toggle_autostart,
            ping_peer,
            update_tray_status,
            send_desktop_notification
        ])
        .run(tauri::generate_context!())
        .expect("error while running innernet desktop application");
}
