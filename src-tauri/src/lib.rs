pub mod autostart;
pub mod cli;
pub mod db;
pub mod downloader;
pub mod github_releases;
pub mod hardware;
pub mod ollama_compat;
pub mod server;
pub mod sidecar;
pub mod window_manager;

use db::{init_db, DbState};
use sidecar::{stop_sidecar_internal, SidecarState};
use std::sync::Mutex;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let conn = init_db().expect("Failed initializing SQLite database");
    let sidecar_state = SidecarState::new();
    let sidecar_state_for_server = sidecar_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::AppleScript,
            Some(vec!["--autostart"]),
        ))
        .setup(move |app| {
            // Start embedded Ollama REST API compatibility proxy on port 11434 within active Tokio runtime
            server::start_server_background(sidecar_state_for_server, 11434);

            let show_item = MenuItemBuilder::with_id("show", "Open Ollama Lite").build(app)?;
            let hide_item = MenuItemBuilder::with_id("hide", "Minimize to Tray").build(app)?;
            let stop_item = MenuItemBuilder::with_id("stop", "Unload Model Sidecar").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit Ollama Lite").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .item(&hide_item)
                .separator()
                .item(&stop_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("Ollama Lite")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    "stop" => {
                        let state = app.state::<SidecarState>();
                        stop_sidecar_internal(&state);
                    }
                    "quit" => {
                        let state = app.state::<SidecarState>();
                        stop_sidecar_internal(&state);
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
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    let _ = window.hide();
                    api.prevent_close();
                }
                tauri::WindowEvent::Destroyed => {
                    let state = window.state::<SidecarState>();
                    stop_sidecar_internal(&state);
                }
                _ => {}
            }
        })
        .manage(sidecar_state)
        .manage(DbState { db: Mutex::new(conn) })
        .invoke_handler(tauri::generate_handler![
            window_manager::hide_window,
            window_manager::show_window,
            // Hardware
            hardware::get_hardware_info,
            hardware::get_hardware_recommendation,
            // Engine releases & installation
            github_releases::fetch_llama_releases,
            github_releases::check_latest_build_number,
            downloader::check_binary_installed,
            downloader::download_llama_engine,
            downloader::get_installed_engine_info,
            // Model downloader & manager
            downloader::download_model_file,
            downloader::list_installed_models,
            downloader::inspect_gguf_file,
            downloader::delete_installed_model,
            // Sidecar process management
            sidecar::start_sidecar,
            sidecar::stop_sidecar,
            sidecar::get_sidecar_status,
            sidecar::get_local_ip_addresses,
            sidecar::get_local_ip,
            // Ollama compat & tags
            ollama_compat::get_ollama_tags,
            // CLI Installer
            cli::install_cli_to_path,
            cli::check_cli_installed,
            // Autostart
            autostart::set_autostart_enabled,
            autostart::is_autostart_enabled,
            // SQLite Database & Settings
            db::create_chat_session,
            db::get_chat_sessions,
            db::delete_chat_session,
            db::save_chat_message,
            db::get_session_messages,
            db::get_db_setting,
            db::set_db_setting,
            db::update_chat_session_title,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Ollama Lite application");
}
