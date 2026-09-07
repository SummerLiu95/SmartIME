//! Credential persistence: secret-free metadata and verified Keychain writes.
use crate::error::{AppError, Result};
use crate::llm::{LLMConfig, LLMConfigStatus, LLMProvider};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use zeroize::Zeroizing;

const SERVICE: &str = "com.smartime.app.llm";

pub trait SecretStore {
    fn get(&self, id: &str) -> Result<Option<Zeroizing<String>>>;
    fn set(&self, id: &str, secret: &str) -> Result<()>;
    fn delete(&self, id: &str) -> Result<()>;
}

pub struct Keychain;

fn keychain_error() -> AppError {
    AppError::Config("无法访问系统钥匙串，请解锁钥匙串并允许 SmartIME 访问后重试".into())
}

#[cfg(target_os = "macos")]
impl SecretStore for Keychain {
    fn get(&self, id: &str) -> Result<Option<Zeroizing<String>>> {
        match security_framework::passwords::get_generic_password(SERVICE, id) {
            Ok(bytes) => {
                let bytes = Zeroizing::new(bytes);
                let value = std::str::from_utf8(&bytes).map_err(|_| keychain_error())?;
                Ok(Some(Zeroizing::new(value.to_owned())))
            }
            Err(error) if error.code() == -25300 => Ok(None), // errSecItemNotFound
            Err(_) => Err(keychain_error()),
        }
    }

    fn set(&self, id: &str, secret: &str) -> Result<()> {
        security_framework::passwords::set_generic_password(SERVICE, id, secret.as_bytes())
            .map_err(|_| keychain_error())
    }

    fn delete(&self, id: &str) -> Result<()> {
        match security_framework::passwords::delete_generic_password(SERVICE, id) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == -25300 => Ok(()),
            Err(_) => Err(keychain_error()),
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl SecretStore for Keychain {
    fn get(&self, _: &str) -> Result<Option<Zeroizing<String>>> {
        Err(keychain_error())
    }
    fn set(&self, _: &str, _: &str) -> Result<()> {
        Err(keychain_error())
    }
    fn delete(&self, _: &str) -> Result<()> {
        Err(keychain_error())
    }
}

#[derive(Serialize, Deserialize)]
struct StoredConfig {
    model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    provider: Option<LLMProvider>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    #[serde(default)]
    credential_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    retired_credentials: Vec<String>,
    // Read legacy files, but never serialize the legacy secret again.
    #[serde(default, skip_serializing)]
    api_key: String,
}

#[derive(Serialize, Deserialize)]
struct Credential {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    provider: Option<LLMProvider>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    api_key: String,
}

impl StoredConfig {
    fn resolved_provider(&self) -> LLMProvider {
        self.provider
            .unwrap_or_else(|| LLMProvider::infer_legacy(&self.model, self.base_url.as_deref()))
    }
}

impl Drop for Credential {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.api_key.zeroize();
    }
}

fn read_key(config: &StoredConfig, store: &impl SecretStore) -> Result<Option<Zeroizing<String>>> {
    let Some(id) = &config.credential_id else {
        return Ok(None);
    };
    let Some(payload) = store.get(id)? else {
        return Ok(None);
    };
    let mut credential: Credential =
        serde_json::from_str(&payload).map_err(|_| keychain_error())?;
    let provider_matches = match (credential.provider, credential.base_url.as_deref()) {
        (Some(provider), _) => provider == config.resolved_provider(),
        (None, Some(base_url)) => config
            .base_url
            .as_deref()
            .is_some_and(|saved| base_url == saved.trim_end_matches('/')),
        (None, None) => false,
    };
    if !provider_matches {
        return Err(AppError::Config(
            "服务商与已保存密钥不匹配，请重新配置密钥".into(),
        ));
    }
    Ok(Some(Zeroizing::new(std::mem::take(
        &mut credential.api_key,
    ))))
}

impl Drop for StoredConfig {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.api_key.zeroize();
    }
}

fn read(path: &Path) -> Result<StoredConfig> {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&Zeroizing::new(text))
            .map_err(|_| AppError::Config("LLM 配置无法解析，请修复配置文件后重试".into())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let defaults = LLMConfig::default();
            Ok(StoredConfig {
                model: defaults.model.clone(),
                provider: Some(defaults.provider),
                base_url: None,
                credential_id: None,
                retired_credentials: Vec::new(),
                api_key: String::new(),
            })
        }
        Err(error) => Err(error.into()),
    }
}

fn write(path: &Path, config: &StoredConfig) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::Config("配置目录不可用".into()))?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(".llm-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| -> Result<()> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temp)?;
        file.write_all(&serde_json::to_vec_pretty(config)?)?;
        file.sync_all()?;
        fs::rename(&temp, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}

fn replace_secret(
    path: &Path,
    store: &impl SecretStore,
    config: &mut StoredConfig,
    key: &str,
) -> Result<()> {
    let id = uuid::Uuid::new_v4().to_string();
    let provider = config.resolved_provider();
    // Bind the provider inside Keychain too: editing public JSON cannot reroute the key.
    let payload = Zeroizing::new(serde_json::to_string(&Credential {
        provider: Some(provider),
        base_url: None,
        api_key: key.into(),
    })?);
    store.set(&id, &payload)?;
    let verified = match store.get(&id) {
        Ok(value) => value,
        Err(error) => {
            let _ = store.delete(&id);
            return Err(error);
        }
    };
    if verified.as_deref().map(|s| s.as_str()) != Some(payload.as_str()) {
        let _ = store.delete(&id);
        return Err(AppError::Config("钥匙串写入验证失败，原配置未更改".into()));
    }
    let previous = config.credential_id.replace(id.clone());
    let retired_count = config.retired_credentials.len();
    let previous_provider = config.provider.replace(provider);
    let previous_base_url = config.base_url.take();
    if let Some(previous) = &previous {
        config.retired_credentials.push(previous.clone());
    }
    if let Err(error) = write(path, config) {
        config.credential_id = previous;
        config.provider = previous_provider;
        config.base_url = previous_base_url;
        config.retired_credentials.truncate(retired_count);
        let _ = store.delete(&id);
        return Err(error);
    }
    cleanup_retired(path, store, config)
}

fn cleanup_retired(path: &Path, store: &impl SecretStore, config: &mut StoredConfig) -> Result<()> {
    if config.retired_credentials.is_empty() {
        return Ok(());
    }
    for id in &config.retired_credentials {
        // Never let malformed public metadata delete the active credential.
        if config.credential_id.as_ref() == Some(id) {
            return Err(AppError::Config("密钥清理记录无效".into()));
        }
        store
            .delete(id)
            .map_err(|_| AppError::Config("配置已保存，旧密钥清理失败，请重试".into()))?;
    }
    config.retired_credentials.clear();
    write(path, config)
}

fn load(path: &Path, store: &impl SecretStore) -> Result<StoredConfig> {
    let mut config = read(path)?;
    cleanup_retired(path, store, &mut config)?;
    if !config.api_key.is_empty() {
        let legacy = Zeroizing::new(std::mem::take(&mut config.api_key));
        replace_secret(path, store, &mut config, &legacy)?;
    }
    Ok(config)
}

pub fn status(path: &Path, store: &impl SecretStore) -> Result<LLMConfigStatus> {
    let config = load(path, store)?;
    let has_api_key = read_key(&config, store)?.is_some();
    Ok(LLMConfigStatus {
        provider: config.resolved_provider(),
        model: config.model.clone(),
        has_api_key,
    })
}

pub fn resolve(
    path: &Path,
    store: &impl SecretStore,
    input: Option<LLMConfig>,
) -> Result<LLMConfig> {
    let config = load(path, store)?;
    let mut input = input.unwrap_or_else(|| LLMConfig {
        api_key: String::new(),
        provider: config.resolved_provider(),
        model: config.model.clone(),
    });
    if input.model.trim().is_empty() {
        return Err(AppError::Config("请输入模型名称".into()));
    }
    if input.api_key.trim().is_empty() {
        if input.provider != config.resolved_provider() {
            return Err(AppError::Config(
                "更换服务商时请重新输入对应的 API Key".into(),
            ));
        }
        input.api_key = read_key(&config, store)?
            .map(|key| key.to_string())
            .unwrap_or_default();
    }
    if input.api_key.trim().is_empty() || input.api_key == "******" {
        return Err(AppError::Config("请配置 API Key".into()));
    }
    Ok(input)
}

pub fn save(path: &Path, store: &impl SecretStore, input: LLMConfig) -> Result<()> {
    let input = resolve(path, store, Some(input))?;
    let mut config = load(path, store)?;
    config.model = input.model.clone();
    config.provider = Some(input.provider);
    config.base_url = None;
    replace_secret(path, store, &mut config, &input.api_key)
}

pub fn delete(path: &Path, store: &impl SecretStore) -> Result<()> {
    let mut config = read(path)?;
    cleanup_retired(path, store, &mut config)?;
    if let Some(id) = config.credential_id.take() {
        config.retired_credentials.push(id);
    }
    use zeroize::Zeroize;
    config.api_key.zeroize();
    // Commit the no-active-key state before deleting the Keychain item. If the
    // cleanup or its final metadata write fails, the retired ID remains retryable.
    write(path, &config)?;
    cleanup_retired(path, store, &mut config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;

    #[derive(Default)]
    struct FakeStore {
        values: RefCell<HashMap<String, String>>,
        fail: bool,
        fail_read: bool,
        fail_delete: Cell<bool>,
    }
    impl SecretStore for FakeStore {
        fn get(&self, id: &str) -> Result<Option<Zeroizing<String>>> {
            if self.fail_read {
                return Err(keychain_error());
            }
            Ok(self.values.borrow().get(id).cloned().map(Zeroizing::new))
        }
        fn set(&self, id: &str, secret: &str) -> Result<()> {
            if self.fail {
                return Err(keychain_error());
            }
            self.values.borrow_mut().insert(id.into(), secret.into());
            Ok(())
        }
        fn delete(&self, id: &str) -> Result<()> {
            if self.fail_delete.get() {
                return Err(keychain_error());
            }
            self.values.borrow_mut().remove(id);
            Ok(())
        }
    }
    fn fixture() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("smartime-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("llm_config.json");
        fs::write(
            &path,
            r#"{"model":"test","base_url":"https://example.com/v1","api_key":"fake-secret"}"#,
        )
        .unwrap();
        path
    }
    #[test]
    fn migration_removes_plaintext_and_preserves_credential() {
        let path = fixture();
        let store = FakeStore::default();
        assert!(status(&path, &store).unwrap().has_api_key);
        let disk = fs::read_to_string(&path).unwrap();
        assert!(!disk.contains("api_key") && !disk.contains("fake-secret"));
        assert_eq!(resolve(&path, &store, None).unwrap().api_key, "fake-secret");
        assert!(!serde_json::to_string(&status(&path, &store).unwrap())
            .unwrap()
            .contains("fake-secret"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn keychain_failure_keeps_legacy_file_unchanged() {
        let path = fixture();
        let before = fs::read(&path).unwrap();
        let store = FakeStore {
            fail: true,
            ..Default::default()
        };
        assert!(status(&path, &store).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn cannot_reuse_key_for_different_provider_and_delete_survives_reload() {
        let path = fixture();
        let store = FakeStore::default();
        status(&path, &store).unwrap();
        let input = LLMConfig {
            api_key: String::new(),
            provider: LLMProvider::DeepSeek,
            model: "test".into(),
        };
        assert!(resolve(&path, &store, Some(input)).is_err());
        delete(&path, &store).unwrap();
        assert!(!status(&path, &store).unwrap().has_api_key);
        assert!(store.values.borrow().is_empty());
        assert!(resolve(&path, &store, None).is_err());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn corrupt_config_fails_closed() {
        let path = fixture();
        fs::write(&path, "invalid-json").unwrap();
        assert!(status(&path, &FakeStore::default()).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "invalid-json");
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn failed_readback_does_not_remove_legacy_secret() {
        let path = fixture();
        let before = fs::read(&path).unwrap();
        let store = FakeStore {
            fail_read: true,
            ..Default::default()
        };
        assert!(status(&path, &store).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        assert!(store.values.borrow().is_empty());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn metadata_write_failure_preserves_old_reference_and_removes_new_key() {
        let path = fixture();
        let store = FakeStore::default();
        status(&path, &store).unwrap();
        let mut config = read(&path).unwrap();
        let before_id = config.credential_id.clone();
        let before_file = fs::read(&path).unwrap();
        // Renaming a temporary file over an existing directory must fail.
        let blocked = path.parent().unwrap().join("blocked");
        fs::create_dir(&blocked).unwrap();
        assert!(replace_secret(&blocked, &store, &mut config, "replacement").is_err());
        assert_eq!(config.credential_id, before_id);
        assert_eq!(store.values.borrow().len(), 1);
        assert_eq!(fs::read(&path).unwrap(), before_file);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn replacing_key_removes_old_entry_and_metadata_tampering_cannot_reroute_it() {
        let path = fixture();
        let store = FakeStore::default();
        status(&path, &store).unwrap();
        save(
            &path,
            &store,
            LLMConfig {
                api_key: "new-secret".into(),
                provider: LLMProvider::DeepSeek,
                model: "test".into(),
            },
        )
        .unwrap();
        assert_eq!(store.values.borrow().len(), 1);
        assert_eq!(resolve(&path, &store, None).unwrap().api_key, "new-secret");
        let disk = fs::read_to_string(&path).unwrap();
        assert!(!disk.contains("new-secret"));
        let tampered = disk.replace("\"deepseek\"", "\"openai\"");
        fs::write(&path, tampered).unwrap();
        assert!(resolve(&path, &store, None).is_err());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn retired_key_cleanup_is_retried_after_failure() {
        let path = fixture();
        let store = FakeStore::default();
        status(&path, &store).unwrap();
        store.fail_delete.set(true);
        assert!(save(
            &path,
            &store,
            LLMConfig {
                api_key: "replacement".into(),
                provider: LLMProvider::OpenAi,
                model: "test".into(),
            }
        )
        .is_err());
        assert_eq!(store.values.borrow().len(), 2);
        assert_eq!(read(&path).unwrap().retired_credentials.len(), 1);
        store.fail_delete.set(false);
        assert_eq!(resolve(&path, &store, None).unwrap().api_key, "replacement");
        assert_eq!(store.values.borrow().len(), 1);
        assert!(read(&path).unwrap().retired_credentials.is_empty());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn delete_cleanup_failure_commits_no_active_key_and_retries() {
        let path = fixture();
        let store = FakeStore::default();
        status(&path, &store).unwrap();
        store.fail_delete.set(true);
        assert!(delete(&path, &store).is_err());
        let pending = read(&path).unwrap();
        assert!(pending.credential_id.is_none());
        assert_eq!(pending.retired_credentials.len(), 1);
        assert_eq!(store.values.borrow().len(), 1);
        store.fail_delete.set(false);
        assert!(!status(&path, &store).unwrap().has_api_key);
        assert!(store.values.borrow().is_empty());
        assert!(read(&path).unwrap().retired_credentials.is_empty());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    #[ignore = "Uses a disposable Keychain entry and may require macOS authorization"]
    fn native_keychain_roundtrip() {
        let id = format!("smartime-test-{}", uuid::Uuid::new_v4());
        let store = Keychain;
        store.set(&id, "disposable-test-value").unwrap();
        let observed = store.get(&id);
        let cleanup = store.delete(&id);
        cleanup.unwrap();
        assert_eq!(observed.unwrap().unwrap().as_str(), "disposable-test-value");
        assert!(store.get(&id).unwrap().is_none());
    }
}
