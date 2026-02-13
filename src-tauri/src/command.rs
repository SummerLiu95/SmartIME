use crate::config::{AppConfig, AppRule, AppState};
use crate::error::{AppError, Result};
use crate::general_settings;
use crate::input_source::{get_system_input_sources, select_input_source, InputSource};
use crate::llm::LLMConfig;
use crate::system_apps::SystemApp;
use tauri::{AppHandle, State};

// Input Source Commands

#[tauri::command]
pub fn cmd_get_system_input_sources() -> Result<Vec<InputSource>> {
    get_system_input_sources()
}

#[tauri::command]
pub fn cmd_select_input_source(id: String) -> Result<()> {
    select_input_source(&id)
}

// Config Commands

#[tauri::command]
pub fn cmd_get_installed_apps() -> Result<Vec<SystemApp>> {
    crate::system_apps::get_installed_apps()
}

#[tauri::command]
pub fn cmd_save_config(
    config: AppConfig,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<()> {
    let mut manager = state
        .config
        .lock()
        .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    let previous = manager.get_config();

    if previous.general != config.general {
        general_settings::apply_general_settings(&app, &config.general)?;
    }

    manager.set_config(config)
}

#[tauri::command]
pub fn cmd_save_rules(rules: Vec<AppRule>, state: State<'_, AppState>) -> Result<()> {
    let mut manager = state
        .config
        .lock()
        .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;

    let mut config = manager.get_config();
    config.rules = rules;
    manager.set_config(config)
}

#[tauri::command]
pub fn cmd_get_config(state: State<'_, AppState>) -> Result<AppConfig> {
    let manager = state.config.lock().map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    Ok(manager.get_config())
}

#[tauri::command]
pub fn cmd_has_config(state: State<'_, AppState>) -> Result<bool> {
    let manager = state.config.lock().map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    Ok(manager.has_config_file())
}

// LLM Commands

#[tauri::command]
pub async fn cmd_check_llm_connection(config: LLMConfig) -> Result<bool> {
    crate::llm::LLMClient::check_connection(&config).await?;
    Ok(true)
}

#[tauri::command]
pub fn cmd_save_llm_config(config: LLMConfig, state: State<'_, AppState>) -> Result<()> {
    let mut llm = state.llm.lock().map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    llm.update_config(config)
}

#[tauri::command]
pub fn cmd_get_llm_config(state: State<'_, AppState>) -> Result<LLMConfig> {
    let llm = state.llm.lock().map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    let mut config = llm.get_config();
    // 脱敏处理
    if !config.api_key.is_empty() {
        config.api_key = "******".to_string();
    }
    Ok(config)
}

#[tauri::command]
pub async fn cmd_scan_and_predict(
    input_sources: Vec<InputSource>,
    state: State<'_, AppState>,
) -> Result<Vec<AppRule>> {
    let installed_apps = crate::system_apps::get_installed_apps()?;
    let target_apps = filter_target_apps(installed_apps);

    if input_sources.is_empty() {
        return Err(AppError::InputSource("No available input sources".to_string()));
    }
    
    let llm_client = {
        let guard = state.llm.lock().map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
        let config = guard.get_config();
        if config.api_key.trim().is_empty()
            || config.model.trim().is_empty()
            || config.base_url.trim().is_empty()
        {
            return Err(AppError::Llm("LLM configuration is incomplete".to_string()));
        }
        guard.clone()
    };
    
    let mut rules = Vec::new();

    for app in target_apps {
        match llm_client.predict(&app.name, &app.bundle_id, &input_sources).await {
            Ok(preferred_input) => {
                rules.push(AppRule {
                    bundle_id: app.bundle_id,
                    app_name: app.name,
                    preferred_input,
                    is_ai_generated: true,
                });
            }
            Err(e) => {
                eprintln!("Failed to predict for {}: {}", app.name, e);
            }
        }
    }

    Ok(rules)
}

fn filter_target_apps(apps: Vec<SystemApp>) -> Vec<SystemApp> {
    apps.into_iter()
        .filter(|app| !app.bundle_id.starts_with("com.apple."))
        .collect()
}

#[tauri::command]
pub fn cmd_check_permissions() -> bool {
    #[cfg(target_os = "macos")]
    {
        return request_accessibility_permission(false);
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

#[tauri::command]
pub fn cmd_request_permissions() -> bool {
    #[cfg(target_os = "macos")]
    {
        return request_accessibility_permission(true);
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

#[tauri::command]
pub fn cmd_open_system_settings() {
    // 打开 macOS 隐私设置
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn();
}

#[cfg(target_os = "macos")]
fn request_accessibility_permission(prompt: bool) -> bool {
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
    use core_foundation::string::{CFString, CFStringRef};

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
        static kAXTrustedCheckOptionPrompt: CFStringRef;
    }

    unsafe {
        if !prompt {
            return AXIsProcessTrustedWithOptions(std::ptr::null());
        }

        let prompt_key = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt);
        let options: CFDictionary<CFString, CFBoolean> =
            CFDictionary::from_CFType_pairs(&[(prompt_key, CFBoolean::true_value())]);
        AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_filter_target_apps() {
        let apps = vec![
            SystemApp {
                name: "Safari".to_string(),
                bundle_id: "com.apple.Safari".to_string(),
                path: PathBuf::from("/Applications/Safari.app"),
            },
            SystemApp {
                name: "Chrome".to_string(),
                bundle_id: "com.google.Chrome".to_string(),
                path: PathBuf::from("/Applications/Google Chrome.app"),
            },
        ];

        let filtered = filter_target_apps(apps);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].bundle_id, "com.google.Chrome");
    }
}
