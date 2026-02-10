use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRule {
    pub bundle_id: String,
    pub app_name: String,
    pub preferred_input: String,
    pub is_ai_generated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneralSettings {
    pub auto_start: bool,
    pub hide_dock_icon: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            auto_start: true,
            hide_dock_icon: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    pub global_switch: bool,
    pub default_input: String, // "en", "zh", "keep"
    #[serde(default)]
    pub general: GeneralSettings,
    pub rules: Vec<AppRule>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            global_switch: true,
            default_input: "keep".to_string(),
            general: GeneralSettings::default(),
            rules: Vec::new(),
        }
    }
}

pub struct ConfigManager {
    config: AppConfig,
    file_path: PathBuf,
    // 内存缓存优化查询
    rule_map: HashMap<String, String>, 
}

impl ConfigManager {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("smartime");
        
        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }

        let file_path = config_dir.join("config.json");
        let config = Self::load_from_file(&file_path).unwrap_or_default();
        
        let mut manager = Self {
            config: config.clone(),
            file_path,
            rule_map: HashMap::new(),
        };
        manager.rebuild_cache();
        manager
    }

    fn load_from_file(path: &PathBuf) -> Result<AppConfig> {
        if !path.exists() {
            return Ok(AppConfig::default());
        }
        let content = fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    fn rebuild_cache(&mut self) {
        self.rule_map.clear();
        for rule in &self.config.rules {
            self.rule_map.insert(rule.bundle_id.clone(), rule.preferred_input.clone());
        }
    }

    pub fn get_config(&self) -> AppConfig {
        self.config.clone()
    }

    pub fn has_config_file(&self) -> bool {
        self.file_path.exists()
    }

    pub fn set_config(&mut self, config: AppConfig) -> Result<()> {
        self.config = config;
        self.rebuild_cache();
        self.save()
    }

    pub fn get_rule(&self, bundle_id: &str) -> Option<String> {
        if !self.config.global_switch {
            return None;
        }
        self.rule_map.get(bundle_id).cloned()
    }
    
    pub fn add_or_update_rule(&mut self, rule: AppRule) -> Result<()> {
        // 移除旧规则
        self.config.rules.retain(|r| r.bundle_id != rule.bundle_id);
        self.config.rules.push(rule);
        self.rebuild_cache();
        self.save()
    }
}

// 供 Tauri 状态管理的线程安全容器
pub struct AppState {
    pub config: Mutex<ConfigManager>,
    pub llm: Mutex<crate::llm::LLMClient>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(ConfigManager::new()),
            llm: Mutex::new(crate::llm::LLMClient::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_settings_default() {
        let defaults = GeneralSettings::default();
        assert!(defaults.auto_start);
        assert!(!defaults.hide_dock_icon);
    }

    #[test]
    fn test_app_config_deserialize_missing_general() {
        let raw = r#"{
            "version": 1,
            "global_switch": true,
            "default_input": "keep",
            "rules": []
        }"#;
        let parsed: AppConfig = serde_json::from_str(raw).expect("deserialize AppConfig");
        assert_eq!(parsed.general, GeneralSettings::default());
    }
}
