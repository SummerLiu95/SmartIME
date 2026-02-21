use crate::config::{AppConfig, AppRule, AppState};
use crate::error::{AppError, Result};
use crate::general_settings;
use crate::input_source::{get_system_input_sources, select_input_source, InputSource};
use crate::llm::LLMConfig;
use crate::system_apps::SystemApp;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;
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
        general_settings::apply_general_settings_delta(&app, &previous.general, &config.general)?;
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
    let manager = state
        .config
        .lock()
        .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    Ok(manager.get_config())
}

#[tauri::command]
pub fn cmd_has_config(state: State<'_, AppState>) -> Result<bool> {
    let manager = state
        .config
        .lock()
        .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
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
    let mut llm = state
        .llm
        .lock()
        .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    llm.update_config(config)
}

#[tauri::command]
pub fn cmd_get_llm_config(state: State<'_, AppState>) -> Result<LLMConfig> {
    let llm = state
        .llm
        .lock()
        .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
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
    let target_apps = get_target_apps()?;
    let generated = predict_rules_for_apps(&target_apps, &input_sources, &state).await?;
    Ok(align_rules_with_apps(
        &target_apps,
        generated,
        &[],
        &input_sources,
    ))
}

#[tauri::command]
pub async fn cmd_rescan_and_save_rules(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<Vec<AppRule>> {
    if state
        .is_rescanning
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(AppError::Config(
            "Rescan is already in progress".to_string(),
        ));
    }
    let _rescan_guard = RescanGuard {
        flag: &state.is_rescanning,
    };

    let input_sources = get_system_input_sources_on_main_thread(&app)?;
    let target_apps = get_target_apps()?;

    let existing_rules = {
        let manager = state
            .config
            .lock()
            .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
        manager.get_config().rules
    };

    let generated = predict_rules_for_apps(&target_apps, &input_sources, &state).await?;
    let aligned = align_rules_with_apps(&target_apps, generated, &existing_rules, &input_sources);

    let mut manager = state
        .config
        .lock()
        .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    let mut config = manager.get_config();
    config.rules = aligned.clone();
    manager.set_config(config)?;

    Ok(aligned)
}

#[tauri::command]
pub fn cmd_is_rescanning(state: State<'_, AppState>) -> bool {
    state.is_rescanning.load(Ordering::SeqCst)
}

struct RescanGuard<'a> {
    flag: &'a std::sync::atomic::AtomicBool,
}

impl Drop for RescanGuard<'_> {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::SeqCst);
    }
}

fn get_system_input_sources_on_main_thread(app: &AppHandle) -> Result<Vec<InputSource>> {
    let (tx, rx) = mpsc::channel::<std::result::Result<Vec<InputSource>, String>>();

    app.run_on_main_thread(move || {
        let result = get_system_input_sources().map_err(|e| e.to_string());
        let _ = tx.send(result);
    })
    .map_err(|e| {
        AppError::InputSource(format!(
            "Failed to schedule input source scan on main thread: {}",
            e
        ))
    })?;

    let result = rx.recv_timeout(Duration::from_secs(5)).map_err(|e| {
        AppError::InputSource(format!(
            "Timed out waiting for main-thread input source scan: {}",
            e
        ))
    })?;

    result.map_err(AppError::InputSource)
}

fn get_target_apps() -> Result<Vec<SystemApp>> {
    let installed_apps = crate::system_apps::get_installed_apps()?;
    Ok(filter_target_apps(installed_apps))
}

async fn predict_rules_for_apps(
    target_apps: &[SystemApp],
    input_sources: &[InputSource],
    state: &State<'_, AppState>,
) -> Result<Vec<AppRule>> {
    if input_sources.is_empty() {
        return Err(AppError::InputSource(
            "No available input sources".to_string(),
        ));
    }

    let llm_client = {
        let guard = state
            .llm
            .lock()
            .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
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
        match llm_client
            .predict(&app.name, &app.bundle_id, input_sources)
            .await
        {
            Ok(preferred_input) => {
                rules.push(AppRule {
                    bundle_id: app.bundle_id.clone(),
                    app_name: app.name.clone(),
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

fn align_rules_with_apps(
    target_apps: &[SystemApp],
    generated_rules: Vec<AppRule>,
    existing_rules: &[AppRule],
    input_sources: &[InputSource],
) -> Vec<AppRule> {
    let generated_by_bundle: HashMap<String, AppRule> = generated_rules
        .into_iter()
        .map(|rule| (rule.bundle_id.clone(), rule))
        .collect();
    let manual_by_bundle: HashMap<String, AppRule> = existing_rules
        .iter()
        .filter(|rule| !rule.is_ai_generated)
        .cloned()
        .map(|rule| (rule.bundle_id.clone(), rule))
        .collect();
    let existing_by_bundle: HashMap<String, AppRule> = existing_rules
        .iter()
        .cloned()
        .map(|rule| (rule.bundle_id.clone(), rule))
        .collect();

    let fallback_input = input_sources
        .first()
        .map(|source| source.id.clone())
        .unwrap_or_default();
    let mut aligned = Vec::with_capacity(target_apps.len());

    for app in target_apps {
        let mut selected = if let Some(rule) = manual_by_bundle.get(&app.bundle_id) {
            rule.clone()
        } else if let Some(rule) = generated_by_bundle.get(&app.bundle_id) {
            rule.clone()
        } else if let Some(rule) = existing_by_bundle.get(&app.bundle_id) {
            rule.clone()
        } else {
            AppRule {
                bundle_id: app.bundle_id.clone(),
                app_name: app.name.clone(),
                preferred_input: fallback_input.clone(),
                is_ai_generated: true,
            }
        };

        selected.bundle_id = app.bundle_id.clone();
        selected.app_name = app.name.clone();
        aligned.push(selected);
    }

    normalize_rule_inputs(aligned, input_sources)
}

fn normalize_rule_inputs(mut rules: Vec<AppRule>, input_sources: &[InputSource]) -> Vec<AppRule> {
    let Some(fallback_id) = input_sources.first().map(|source| source.id.clone()) else {
        return rules;
    };

    let available_ids: HashSet<&str> = input_sources
        .iter()
        .map(|source| source.id.as_str())
        .collect();

    for rule in &mut rules {
        if !available_ids.contains(rule.preferred_input.as_str()) {
            rule.preferred_input = fallback_id.clone();
        }
    }

    rules
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

    #[test]
    fn test_normalize_rule_inputs_replaces_removed_input_method_ids() {
        let rules = vec![
            AppRule {
                bundle_id: "com.apple.TextEdit".to_string(),
                app_name: "TextEdit".to_string(),
                preferred_input: "com.apple.keylayout.ABC".to_string(),
                is_ai_generated: true,
            },
            AppRule {
                bundle_id: "com.apple.Terminal".to_string(),
                app_name: "Terminal".to_string(),
                preferred_input: "com.apple.inputmethod.Korean.2SetKorean".to_string(),
                is_ai_generated: false,
            },
        ];

        let input_sources = vec![
            InputSource {
                id: "com.apple.keylayout.ABC".to_string(),
                name: "ABC".to_string(),
                category: "TISCategoryKeyboardInputSource".to_string(),
            },
            InputSource {
                id: "com.apple.inputmethod.SCIM.ITABC".to_string(),
                name: "Pinyin - Simplified".to_string(),
                category: "TISCategoryKeyboardInputSource".to_string(),
            },
        ];

        let normalized = normalize_rule_inputs(rules, &input_sources);

        assert_eq!(normalized[0].preferred_input, "com.apple.keylayout.ABC");
        assert_eq!(normalized[1].preferred_input, "com.apple.keylayout.ABC");
    }

    #[test]
    fn test_align_rules_with_apps_keeps_only_installed_apps_and_preserves_manual_rules() {
        let target_apps = vec![
            SystemApp {
                name: "Alpha".to_string(),
                bundle_id: "com.example.alpha".to_string(),
                path: PathBuf::from("/Applications/Alpha.app"),
            },
            SystemApp {
                name: "Beta".to_string(),
                bundle_id: "com.example.beta".to_string(),
                path: PathBuf::from("/Applications/Beta.app"),
            },
            SystemApp {
                name: "Delta".to_string(),
                bundle_id: "com.example.delta".to_string(),
                path: PathBuf::from("/Applications/Delta.app"),
            },
        ];

        let generated = vec![AppRule {
            bundle_id: "com.example.alpha".to_string(),
            app_name: "Alpha".to_string(),
            preferred_input: "com.apple.inputmethod.SCIM.ITABC".to_string(),
            is_ai_generated: true,
        }];

        let existing = vec![
            AppRule {
                bundle_id: "com.example.alpha".to_string(),
                app_name: "Alpha".to_string(),
                preferred_input: "com.apple.keylayout.ABC".to_string(),
                is_ai_generated: false,
            },
            AppRule {
                bundle_id: "com.example.beta".to_string(),
                app_name: "Beta".to_string(),
                preferred_input: "com.apple.keylayout.ABC".to_string(),
                is_ai_generated: true,
            },
            AppRule {
                bundle_id: "com.example.gamma".to_string(),
                app_name: "Gamma".to_string(),
                preferred_input: "com.apple.keylayout.ABC".to_string(),
                is_ai_generated: false,
            },
        ];

        let input_sources = vec![
            InputSource {
                id: "com.apple.keylayout.ABC".to_string(),
                name: "ABC".to_string(),
                category: "TISCategoryKeyboardInputSource".to_string(),
            },
            InputSource {
                id: "com.apple.inputmethod.SCIM.ITABC".to_string(),
                name: "Pinyin - Simplified".to_string(),
                category: "TISCategoryKeyboardInputSource".to_string(),
            },
        ];

        let aligned = align_rules_with_apps(&target_apps, generated, &existing, &input_sources);

        assert_eq!(aligned.len(), 3);
        assert_eq!(aligned[0].bundle_id, "com.example.alpha");
        assert_eq!(aligned[0].preferred_input, "com.apple.keylayout.ABC");
        assert!(!aligned[0].is_ai_generated);

        assert_eq!(aligned[1].bundle_id, "com.example.beta");
        assert_eq!(aligned[1].preferred_input, "com.apple.keylayout.ABC");

        assert_eq!(aligned[2].bundle_id, "com.example.delta");
        assert_eq!(aligned[2].preferred_input, "com.apple.keylayout.ABC");

        assert!(!aligned
            .iter()
            .any(|rule| rule.bundle_id == "com.example.gamma"));
    }
}
