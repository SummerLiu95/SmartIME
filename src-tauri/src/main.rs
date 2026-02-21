// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod command;
mod config;
mod error;
mod general_settings;
mod input_source;
mod llm;
mod observer;
mod single_instance;
mod system_apps;

use config::AppState;
use tauri::Manager;
use tauri::WindowEvent;

fn main() {
    if !single_instance::prepare_primary_instance() {
        return;
    }

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(AppState::new()) // 注入全局状态
        .on_window_event(|window, event| {
            #[cfg(target_os = "macos")]
            {
                if window.label() != "main" {
                    return;
                }

                if let WindowEvent::CloseRequested { api, .. } = event {
                    let should_hide_to_tray = window
                        .app_handle()
                        .state::<AppState>()
                        .config
                        .lock()
                        .ok()
                        .map(|manager| manager.get_config().general.hide_dock_icon)
                        .unwrap_or(false);

                    if should_hide_to_tray {
                        api.prevent_close();
                        let _ = window.app_handle().set_dock_visibility(false);
                        let _ = window.hide();
                    }
                }
            }
        })
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<AppState>();

            single_instance::start_activation_listener(handle.clone());

            if let Ok(manager) = state.config.lock() {
                let config = manager.get_config();
                if let Err(err) = general_settings::apply_general_settings(&handle, &config.general)
                {
                    eprintln!("Failed to apply general settings on startup: {}", err);
                }
            }

            // 启动应用监听
            #[cfg(target_os = "macos")]
            observer::setup_observer(handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            command::cmd_get_system_input_sources,
            command::cmd_select_input_source,
            command::cmd_get_installed_apps,
            command::cmd_save_config,
            command::cmd_save_rules,
            command::cmd_get_config,
            command::cmd_has_config,
            command::cmd_save_llm_config,
            command::cmd_get_llm_config,
            command::cmd_check_llm_connection,
            command::cmd_scan_and_predict,
            command::cmd_rescan_and_save_rules,
            command::cmd_is_rescanning,
            command::cmd_check_permissions,
            command::cmd_request_permissions,
            command::cmd_open_system_settings,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = event {
            single_instance::focus_main_window(app_handle);
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = app_handle;
            let _ = event;
        }
    });
}
