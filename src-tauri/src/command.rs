use crate::config::{AppConfig, AppRule, AppState};
use crate::error::{AppError, Result};
use crate::general_settings;
use crate::input_source::{get_system_input_sources, select_input_source, InputSource};
use crate::llm::{LLMConfig, LLMConfigStatus};
use crate::system_apps::SystemApp;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

const FIRST_SCREEN_ICON_BATCH_SIZE: usize = 8;

// Input Source Commands

#[tauri::command]
pub async fn cmd_get_system_input_sources(app: AppHandle) -> Result<Vec<InputSource>> {
    run_input_source_task_on_main_thread_async(
        app,
        "input source scan",
        Duration::from_secs(5),
        get_system_input_sources,
    )
    .await
}

#[tauri::command]
pub async fn cmd_select_input_source(id: String, app: AppHandle) -> Result<()> {
    run_input_source_task_on_main_thread_async(
        app,
        "input source selection",
        Duration::from_millis(500),
        move || select_input_source(&id),
    )
    .await
}

// Config Commands

#[tauri::command]
pub fn cmd_get_installed_apps(state: State<'_, AppState>) -> Result<Vec<SystemApp>> {
    get_installed_apps_for_runtime(&state, false)
}

#[tauri::command]
pub async fn cmd_get_app_icons(
    bundle_ids: Vec<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<HashMap<String, String>> {
    load_app_icons(bundle_ids, &state, &app).await
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
pub async fn cmd_check_llm_connection(config: LLMConfig, app: AppHandle) -> Result<bool> {
    let config = tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let llm = state
            .llm
            .lock()
            .map_err(|_| AppError::Config("配置暂不可用".into()))?;
        llm.request_config(Some(config))
    })
    .await
    .map_err(|_| AppError::Config("读取密钥失败".into()))??;
    crate::llm::LLMClient::check_connection(&config).await?;
    Ok(true)
}

#[tauri::command]
pub async fn cmd_save_llm_config(config: LLMConfig, app: AppHandle) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let mut llm = state
            .llm
            .lock()
            .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
        llm.update_config(config)
    })
    .await
    .map_err(|_| AppError::Config("保存配置失败".into()))?
}

#[tauri::command]
pub async fn cmd_get_llm_config(app: AppHandle) -> Result<LLMConfigStatus> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let llm = state
            .llm
            .lock()
            .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
        llm.get_config()
    })
    .await
    .map_err(|_| AppError::Config("读取配置失败".into()))?
}

#[tauri::command]
pub async fn cmd_delete_llm_key(app: AppHandle) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let mut llm = state
            .llm
            .lock()
            .map_err(|_| AppError::Config("配置暂不可用".into()))?;
        llm.delete_key()
    })
    .await
    .map_err(|_| AppError::Config("删除密钥失败".into()))?
}

#[tauri::command]
pub async fn cmd_scan_and_predict(
    input_sources: Vec<InputSource>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<Vec<AppRule>> {
    let target_apps = get_target_apps(&state, true)?;
    let generated = predict_rules_for_apps(&target_apps, &input_sources, &state).await?;
    let aligned = align_rules_with_apps(&target_apps, generated, &[], &input_sources);
    if let Err(err) = warm_rule_icon_cache(&aligned, &state, &app).await {
        eprintln!("Failed to warm app icon cache after initial scan: {err}");
    }
    Ok(aligned)
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
    let target_apps = get_target_apps(&state, true)?;

    let existing_rules = {
        let manager = state
            .config
            .lock()
            .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
        manager.get_config().rules
    };

    let apps_to_predict = apps_requiring_prediction(&target_apps, &existing_rules, &input_sources);
    let generated = predict_rules_for_apps(&apps_to_predict, &input_sources, &state).await?;
    let aligned = align_rules_with_apps(&target_apps, generated, &existing_rules, &input_sources);

    {
        let mut manager = state
            .config
            .lock()
            .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
        let mut config = manager.get_config();
        config.rules = aligned.clone();
        manager.set_config(config)?;
    }

    if let Err(err) = warm_rule_icon_cache(&aligned, &state, &app).await {
        eprintln!("Failed to warm app icon cache after rescan: {err}");
    }

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
    run_input_source_task_on_main_thread(
        app,
        "input source scan",
        Duration::from_secs(5),
        get_system_input_sources,
    )
}

async fn run_input_source_task_on_main_thread_async<T, F>(
    app: AppHandle,
    task_name: &'static str,
    timeout: Duration,
    task: F,
) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        run_input_source_task_on_main_thread(&app, task_name, timeout, task)
    })
    .await
    .map_err(|e| AppError::InputSource(format!("Failed to join {task_name}: {e}")))?
}

fn run_input_source_task_on_main_thread<T, F>(
    app: &AppHandle,
    task_name: &'static str,
    timeout: Duration,
    task: F,
) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    let (tx, rx) = mpsc::channel::<std::result::Result<T, String>>();

    app.run_on_main_thread(move || {
        let result = task().map_err(|e| e.to_string());
        let _ = tx.send(result);
    })
    .map_err(|e| {
        AppError::InputSource(format!(
            "Failed to schedule {task_name} on main thread: {e}"
        ))
    })?;

    let result = rx.recv_timeout(timeout).map_err(|e| {
        AppError::InputSource(format!(
            "Timed out waiting for main-thread {task_name}: {e}"
        ))
    })?;

    result.map_err(AppError::InputSource)
}

fn get_target_apps(app_state: &AppState, invalidate_icon_cache: bool) -> Result<Vec<SystemApp>> {
    let installed_apps = get_installed_apps_for_runtime(app_state, invalidate_icon_cache)?;
    Ok(filter_target_apps(installed_apps))
}

async fn predict_rules_for_apps(
    target_apps: &[SystemApp],
    input_sources: &[InputSource],
    app_state: &AppState,
) -> Result<Vec<AppRule>> {
    if input_sources.is_empty() {
        return Err(AppError::InputSource(
            "No available input sources".to_string(),
        ));
    }
    if target_apps.is_empty() {
        return Ok(Vec::new());
    }

    let llm_client = {
        let guard = app_state
            .llm
            .lock()
            .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
        guard.clone()
    };

    let app_targets = target_apps
        .iter()
        .map(|app| (app.name.clone(), app.bundle_id.clone()))
        .collect::<Vec<_>>();

    let config = tauri::async_runtime::spawn_blocking(move || llm_client.request_config(None))
        .await
        .map_err(|_| AppError::Config("读取密钥失败".into()))??;
    let predictions =
        match crate::llm::LLMClient::predict_batch(config, &app_targets, input_sources).await {
            Ok(predictions) => predictions,
            Err(e) => {
                eprintln!("Failed to batch predict app rules: {}", e);
                HashMap::new()
            }
        };

    Ok(target_apps
        .iter()
        .filter_map(|app| {
            predictions
                .get(&app.bundle_id)
                .map(|preferred_input| AppRule {
                    bundle_id: app.bundle_id.clone(),
                    app_name: app.name.clone(),
                    preferred_input: preferred_input.clone(),
                    is_ai_generated: true,
                })
        })
        .collect())
}

fn apps_requiring_prediction(
    target_apps: &[SystemApp],
    existing_rules: &[AppRule],
    input_sources: &[InputSource],
) -> Vec<SystemApp> {
    let valid_input_ids: HashSet<&str> = input_sources
        .iter()
        .map(|source| source.id.as_str())
        .collect();
    let existing_by_bundle: HashMap<&str, &AppRule> = existing_rules
        .iter()
        .map(|rule| (rule.bundle_id.as_str(), rule))
        .collect();

    target_apps
        .iter()
        .filter(|app| match existing_by_bundle.get(app.bundle_id.as_str()) {
            Some(rule) if !rule.is_ai_generated => false,
            Some(rule) if valid_input_ids.contains(rule.preferred_input.as_str()) => false,
            _ => true,
        })
        .cloned()
        .collect()
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
        .filter(|app| !app.bundle_id.trim().is_empty() && !app.name.trim().is_empty())
        .collect()
}

fn get_installed_apps_for_runtime(
    app_state: &AppState,
    invalidate_icon_cache: bool,
) -> Result<Vec<SystemApp>> {
    let installed_apps = crate::system_apps::get_installed_apps()?;
    refresh_runtime_app_cache(app_state, &installed_apps, invalidate_icon_cache)?;
    Ok(installed_apps)
}

fn refresh_runtime_app_cache(
    app_state: &AppState,
    installed_apps: &[SystemApp],
    invalidate_icon_cache: bool,
) -> Result<()> {
    let mut runtime_apps = app_state
        .runtime_apps
        .lock()
        .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    runtime_apps.refresh_installed_apps(installed_apps);
    if invalidate_icon_cache {
        runtime_apps.clear_icons();
    }
    Ok(())
}

fn runtime_metadata_complete(app_state: &AppState, bundle_ids: &[String]) -> Result<bool> {
    let runtime_apps = app_state
        .runtime_apps
        .lock()
        .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    Ok(runtime_apps.has_metadata_for_all(bundle_ids))
}

fn cached_icon_state(
    app_state: &AppState,
    bundle_ids: &[String],
) -> Result<(HashMap<String, String>, Vec<(String, std::path::PathBuf)>)> {
    let runtime_apps = app_state
        .runtime_apps
        .lock()
        .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    Ok((
        runtime_apps.cached_icons(bundle_ids),
        runtime_apps.pending_icon_targets(bundle_ids),
    ))
}

fn store_icon_results(
    app_state: &AppState,
    targets: &[(String, std::path::PathBuf)],
    resolved_icons: &HashMap<String, String>,
) -> Result<()> {
    let mut runtime_apps = app_state
        .runtime_apps
        .lock()
        .map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    runtime_apps.store_icon_results(targets, resolved_icons);
    Ok(())
}

fn normalized_bundle_ids(bundle_ids: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::with_capacity(bundle_ids.len());
    let mut normalized = Vec::with_capacity(bundle_ids.len());

    for bundle_id in bundle_ids {
        let bundle_id = bundle_id.trim();
        if bundle_id.is_empty() {
            continue;
        }

        let bundle_id = bundle_id.to_string();
        if seen.insert(bundle_id.clone()) {
            normalized.push(bundle_id);
        }
    }

    normalized
}

fn first_rule_bundle_ids(rules: &[AppRule], limit: usize) -> Vec<String> {
    let mut seen = HashSet::with_capacity(limit);
    let mut bundle_ids = Vec::with_capacity(limit);

    for rule in rules {
        if rule.bundle_id.trim().is_empty() {
            continue;
        }

        if seen.insert(rule.bundle_id.clone()) {
            bundle_ids.push(rule.bundle_id.clone());
        }

        if bundle_ids.len() >= limit {
            break;
        }
    }

    bundle_ids
}

async fn load_app_icons(
    bundle_ids: Vec<String>,
    app_state: &AppState,
    app: &AppHandle,
) -> Result<HashMap<String, String>> {
    let bundle_ids = normalized_bundle_ids(bundle_ids);
    if bundle_ids.is_empty() {
        return Ok(HashMap::new());
    }

    if !runtime_metadata_complete(app_state, &bundle_ids)? {
        let installed_apps =
            tauri::async_runtime::spawn_blocking(crate::system_apps::get_installed_apps)
                .await
                .map_err(|e| {
                    AppError::Config(format!("Failed to join app icon path scan: {e}"))
                })??;
        refresh_runtime_app_cache(app_state, &installed_apps, false)?;
    }

    let (mut icons, icon_targets) = cached_icon_state(app_state, &bundle_ids)?;
    if icon_targets.is_empty() {
        return Ok(icons);
    }

    let requested_targets = icon_targets.clone();
    let resolved_icons = run_input_source_task_on_main_thread_async(
        app.clone(),
        "app icon lookup",
        Duration::from_secs(5),
        move || crate::app_icon::app_icon_data_urls(&requested_targets),
    )
    .await?;

    store_icon_results(app_state, &icon_targets, &resolved_icons)?;
    icons.extend(resolved_icons);
    Ok(icons)
}

async fn warm_rule_icon_cache(
    rules: &[AppRule],
    app_state: &AppState,
    app: &AppHandle,
) -> Result<()> {
    let bundle_ids = first_rule_bundle_ids(rules, FIRST_SCREEN_ICON_BATCH_SIZE);
    if bundle_ids.is_empty() {
        return Ok(());
    }

    let _ = load_app_icons(bundle_ids, app_state, app).await?;
    Ok(())
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
        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .any(|app| app.bundle_id == "com.apple.Safari"));
        assert!(filtered
            .iter()
            .any(|app| app.bundle_id == "com.google.Chrome"));
    }

    #[test]
    fn test_filter_target_apps_skips_empty_identity_only() {
        let apps = vec![
            SystemApp {
                name: "Safari".to_string(),
                bundle_id: "com.apple.Safari".to_string(),
                path: PathBuf::from("/Applications/Safari.app"),
            },
            SystemApp {
                name: "".to_string(),
                bundle_id: "com.example.empty-name".to_string(),
                path: PathBuf::from("/Applications/EmptyName.app"),
            },
            SystemApp {
                name: "Empty Bundle".to_string(),
                bundle_id: " ".to_string(),
                path: PathBuf::from("/Applications/EmptyBundle.app"),
            },
        ];

        let filtered = filter_target_apps(apps);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].bundle_id, "com.apple.Safari");
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

    fn test_input_sources() -> Vec<InputSource> {
        vec![
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
        ]
    }

    fn test_system_app(name: &str, bundle_id: &str) -> SystemApp {
        SystemApp {
            name: name.to_string(),
            bundle_id: bundle_id.to_string(),
            path: PathBuf::from(format!("/Applications/{name}.app")),
        }
    }

    fn test_rule(
        app_name: &str,
        bundle_id: &str,
        preferred_input: &str,
        is_ai_generated: bool,
    ) -> AppRule {
        AppRule {
            app_name: app_name.to_string(),
            bundle_id: bundle_id.to_string(),
            preferred_input: preferred_input.to_string(),
            is_ai_generated,
        }
    }

    #[test]
    fn test_apps_requiring_prediction_reuses_valid_rules_and_preserves_manual_rules() {
        let target_apps = vec![
            test_system_app("Manual", "com.example.manual"),
            test_system_app("Existing AI", "com.example.existing-ai"),
            test_system_app("Invalid AI", "com.example.invalid-ai"),
            test_system_app("New App", "com.example.new"),
        ];
        let existing_rules = vec![
            test_rule(
                "Manual",
                "com.example.manual",
                "com.example.removed-input",
                false,
            ),
            test_rule(
                "Existing AI",
                "com.example.existing-ai",
                "com.apple.keylayout.ABC",
                true,
            ),
            test_rule(
                "Invalid AI",
                "com.example.invalid-ai",
                "com.example.removed-input",
                true,
            ),
            test_rule(
                "Stale",
                "com.example.stale",
                "com.apple.keylayout.ABC",
                true,
            ),
        ];

        let gaps = apps_requiring_prediction(&target_apps, &existing_rules, &test_input_sources());

        assert_eq!(gaps.len(), 2);
        assert!(gaps
            .iter()
            .any(|app| app.bundle_id == "com.example.invalid-ai"));
        assert!(gaps.iter().any(|app| app.bundle_id == "com.example.new"));
        assert!(!gaps.iter().any(|app| app.bundle_id == "com.example.manual"));
        assert!(!gaps
            .iter()
            .any(|app| app.bundle_id == "com.example.existing-ai"));
        assert!(!gaps.iter().any(|app| app.bundle_id == "com.example.stale"));
    }

    #[test]
    fn test_normalized_bundle_ids_trims_and_deduplicates_preserving_order() {
        let bundle_ids = normalized_bundle_ids(vec![
            " com.example.alpha ".to_string(),
            "".to_string(),
            "com.example.beta".to_string(),
            "com.example.alpha".to_string(),
            "   ".to_string(),
            "com.example.gamma".to_string(),
        ]);

        assert_eq!(
            bundle_ids,
            vec![
                "com.example.alpha".to_string(),
                "com.example.beta".to_string(),
                "com.example.gamma".to_string(),
            ]
        );
    }

    #[test]
    fn test_first_rule_bundle_ids_limits_to_first_screen_batch() {
        let rules = vec![
            test_rule("Alpha", "com.example.alpha", "abc", true),
            test_rule("Beta", "com.example.beta", "abc", true),
            test_rule("Alpha Duplicate", "com.example.alpha", "abc", true),
            test_rule("Gamma", "com.example.gamma", "abc", true),
        ];

        let bundle_ids = first_rule_bundle_ids(&rules, 2);

        assert_eq!(
            bundle_ids,
            vec![
                "com.example.alpha".to_string(),
                "com.example.beta".to_string(),
            ]
        );
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
            SystemApp {
                name: "Safari".to_string(),
                bundle_id: "com.apple.Safari".to_string(),
                path: PathBuf::from("/Applications/Safari.app"),
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
            AppRule {
                bundle_id: "com.apple.Safari".to_string(),
                app_name: "Safari".to_string(),
                preferred_input: "com.apple.inputmethod.SCIM.ITABC".to_string(),
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

        assert_eq!(aligned.len(), 4);
        assert_eq!(aligned[0].bundle_id, "com.example.alpha");
        assert_eq!(aligned[0].preferred_input, "com.apple.keylayout.ABC");
        assert!(!aligned[0].is_ai_generated);

        assert_eq!(aligned[1].bundle_id, "com.example.beta");
        assert_eq!(aligned[1].preferred_input, "com.apple.keylayout.ABC");

        assert_eq!(aligned[2].bundle_id, "com.example.delta");
        assert_eq!(aligned[2].preferred_input, "com.apple.keylayout.ABC");

        assert_eq!(aligned[3].bundle_id, "com.apple.Safari");
        assert_eq!(
            aligned[3].preferred_input,
            "com.apple.inputmethod.SCIM.ITABC"
        );
        assert!(!aligned[3].is_ai_generated);

        assert!(!aligned
            .iter()
            .any(|rule| rule.bundle_id == "com.example.gamma"));
    }
}
