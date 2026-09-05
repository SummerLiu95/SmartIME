use crate::error::Result;
use crate::system_apps::SystemApp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
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
            auto_start: false,
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
            self.rule_map
                .insert(rule.bundle_id.clone(), rule.preferred_input.clone());
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
}

#[derive(Debug, Clone)]
struct CachedAppIcon {
    app_path: PathBuf,
    data_url: Option<String>,
}

#[derive(Default)]
pub struct RuntimeAppCache {
    app_metadata: HashMap<String, SystemApp>,
    app_icons: HashMap<String, CachedAppIcon>,
}

impl RuntimeAppCache {
    pub fn refresh_installed_apps(&mut self, apps: &[SystemApp]) {
        self.app_metadata = apps
            .iter()
            .cloned()
            .map(|app| (app.bundle_id.clone(), app))
            .collect();
        self.app_icons.retain(|bundle_id, icon| {
            self.app_metadata
                .get(bundle_id)
                .is_some_and(|app| app.path == icon.app_path)
        });
    }

    pub fn clear_icons(&mut self) {
        self.app_icons.clear();
    }

    pub fn has_metadata_for_all(&self, bundle_ids: &[String]) -> bool {
        bundle_ids
            .iter()
            .all(|bundle_id| self.app_metadata.contains_key(bundle_id))
    }

    pub fn cached_icons(&self, bundle_ids: &[String]) -> HashMap<String, String> {
        bundle_ids
            .iter()
            .filter_map(|bundle_id| {
                let app = self.app_metadata.get(bundle_id)?;
                let icon = self.app_icons.get(bundle_id)?;
                (icon.app_path == app.path)
                    .then(|| icon.data_url.clone())
                    .flatten()
                    .map(|data_url| (bundle_id.clone(), data_url))
            })
            .collect()
    }

    pub fn pending_icon_targets(&self, bundle_ids: &[String]) -> Vec<(String, PathBuf)> {
        bundle_ids
            .iter()
            .filter_map(|bundle_id| {
                let app = self.app_metadata.get(bundle_id)?;
                if self
                    .app_icons
                    .get(bundle_id)
                    .is_some_and(|icon| icon.app_path == app.path)
                {
                    return None;
                }

                Some((bundle_id.clone(), app.path.clone()))
            })
            .collect()
    }

    pub fn store_icon_results(
        &mut self,
        targets: &[(String, PathBuf)],
        resolved_icons: &HashMap<String, String>,
    ) {
        for (bundle_id, app_path) in targets {
            self.app_icons.insert(
                bundle_id.clone(),
                CachedAppIcon {
                    app_path: app_path.clone(),
                    data_url: resolved_icons.get(bundle_id).cloned(),
                },
            );
        }
    }
}

// 供 Tauri 状态管理的线程安全容器
pub struct AppState {
    pub config: Mutex<ConfigManager>,
    pub llm: Mutex<crate::llm::LLMClient>,
    pub runtime_apps: Mutex<RuntimeAppCache>,
    pub is_rescanning: AtomicBool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(ConfigManager::new()),
            llm: Mutex::new(crate::llm::LLMClient::new()),
            runtime_apps: Mutex::new(RuntimeAppCache::default()),
            is_rescanning: AtomicBool::new(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn test_system_app(path: &str, name: &str, bundle_id: &str) -> SystemApp {
        SystemApp {
            name: name.to_string(),
            bundle_id: bundle_id.to_string(),
            path: Path::new(path).to_path_buf(),
        }
    }

    #[test]
    fn test_general_settings_default() {
        let defaults = GeneralSettings::default();
        assert!(!defaults.auto_start);
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

    #[test]
    fn test_runtime_app_cache_reuses_icons_and_prunes_stale_entries() {
        let mut cache = RuntimeAppCache::default();
        let alpha = test_system_app("/Applications/Alpha.app", "Alpha", "com.example.alpha");
        let beta = test_system_app("/Applications/Beta.app", "Beta", "com.example.beta");

        cache.refresh_installed_apps(&[alpha.clone(), beta.clone()]);
        cache.store_icon_results(
            &[
                (alpha.bundle_id.clone(), alpha.path.clone()),
                (beta.bundle_id.clone(), beta.path.clone()),
            ],
            &HashMap::from([(
                alpha.bundle_id.clone(),
                "data:image/png;base64,alpha".to_string(),
            )]),
        );

        let bundle_ids = vec![alpha.bundle_id.clone(), beta.bundle_id.clone()];
        let cached = cache.cached_icons(&bundle_ids);
        assert_eq!(cached.len(), 1);
        assert_eq!(
            cached.get(&alpha.bundle_id),
            Some(&"data:image/png;base64,alpha".to_string())
        );
        assert!(cache.pending_icon_targets(&bundle_ids).is_empty());

        let moved_beta = test_system_app(
            "/Applications/Utilities/Beta.app",
            "Beta",
            "com.example.beta",
        );
        cache.refresh_installed_apps(&[alpha.clone(), moved_beta.clone()]);

        let pending = cache.pending_icon_targets(&bundle_ids);
        assert_eq!(
            pending,
            vec![(moved_beta.bundle_id.clone(), moved_beta.path)]
        );
        assert_eq!(cache.cached_icons(&bundle_ids).len(), 1);
    }

    #[test]
    fn test_runtime_app_cache_can_clear_icons_without_losing_metadata() {
        let mut cache = RuntimeAppCache::default();
        let alpha = test_system_app("/Applications/Alpha.app", "Alpha", "com.example.alpha");
        let bundle_ids = vec![alpha.bundle_id.clone()];

        cache.refresh_installed_apps(std::slice::from_ref(&alpha));
        cache.store_icon_results(
            &[(alpha.bundle_id.clone(), alpha.path.clone())],
            &HashMap::from([(
                alpha.bundle_id.clone(),
                "data:image/png;base64,alpha".to_string(),
            )]),
        );
        assert!(cache
            .cached_icons(&bundle_ids)
            .contains_key(&alpha.bundle_id));

        cache.clear_icons();

        assert!(cache.has_metadata_for_all(&bundle_ids));
        assert!(cache.cached_icons(&bundle_ids).is_empty());
        assert_eq!(
            cache.pending_icon_targets(&bundle_ids),
            vec![(alpha.bundle_id, alpha.path)]
        );
    }
}
