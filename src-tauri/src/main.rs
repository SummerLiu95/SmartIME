// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod command;
mod config;
mod error;
mod general_settings;
mod input_source;
mod llm;
mod observer;
mod system_apps;

use config::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(AppState::new()) // 注入全局状态
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<AppState>();

            if let Ok(manager) = state.config.lock() {
                let config = manager.get_config();
                if let Err(err) =
                    general_settings::apply_general_settings(&handle, &config.general)
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
            command::cmd_save_config,
            command::cmd_get_config,
            command::cmd_save_llm_config,
            command::cmd_get_llm_config,
            command::cmd_check_llm_connection,
            command::cmd_scan_and_predict,
            command::cmd_check_permissions,
            command::cmd_open_system_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
