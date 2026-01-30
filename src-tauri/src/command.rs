use crate::config::{AppConfig, AppRule, AppState};
use crate::error::Result;
use crate::input_source::{get_system_input_sources, select_input_source, InputSource};
use crate::llm::LLMConfig;
use tauri::State;

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
pub fn cmd_save_config(config: AppConfig, state: State<'_, AppState>) -> Result<()> {
    let mut manager = state.config.lock().map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    manager.set_config(config)
}

#[tauri::command]
pub fn cmd_get_config(state: State<'_, AppState>) -> Result<AppConfig> {
    let manager = state.config.lock().map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    Ok(manager.get_config())
}

// LLM Commands

#[tauri::command]
pub fn cmd_save_llm_config(config: LLMConfig, state: State<'_, AppState>) -> Result<()> {
    let mut llm = state.llm.lock().map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
    llm.update_config(config);
    Ok(())
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
pub async fn cmd_scan_and_predict(state: State<'_, AppState>) -> Result<Vec<AppRule>> {
    // 1. 获取系统已安装的应用
    let installed_apps = crate::system_apps::get_installed_apps()?;
    
    // 限制扫描数量用于测试或避免过多请求 (可选，这里先不限制，或者限制前N个)
    // 实际生产可能需要分批处理或者用户选择
    // 为了演示，我们只取前 5 个非系统应用（根据 bundle id 简单过滤）
    // 或者全部预测。考虑到 LLM 成本和时间，这里暂时全部预测，但请注意这可能会很慢。
    // 更好的做法是只预测常用应用，或者让用户勾选。
    // 这里我们简单过滤掉 com.apple. 开头的应用，减少数量
    let target_apps: Vec<_> = installed_apps.into_iter()
        .filter(|app| !app.bundle_id.starts_with("com.apple."))
        .take(10) // 暂时限制 10 个，避免等待太久
        .collect();

    let input_sources = get_system_input_sources()?;
    
    // 获取 LLM Client 的克隆，避免持有锁跨越 await
    let llm_client = {
        let guard = state.llm.lock().map_err(|e| crate::error::AppError::Lock(e.to_string()))?;
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

#[tauri::command]
pub fn cmd_check_permissions() -> bool {
    // 简单检查是否能获取到输入法列表（通常需要权限）
    // 更严格的检查可能需要 AXIsProcessTrusted
    get_system_input_sources().is_ok()
}

#[tauri::command]
pub fn cmd_open_system_settings() {
    // 打开 macOS 隐私设置
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn();
}
