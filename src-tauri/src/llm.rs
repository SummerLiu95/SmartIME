use crate::credentials::{self, Keychain};
use crate::error::{AppError, Result};
use crate::input_source::InputSource;
use genai::adapter::AdapterKind;
use genai::chat::{ChatOptions, ChatRequest, ChatResponseFormat, ReasoningEffort};
use genai::resolver::{AuthData, AuthResolver};
use genai::{Client as GenAiClient, ModelIden};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
#[cfg(debug_assertions)]
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use zeroize::{Zeroize, Zeroizing};

// Serialize migration and updates even when a prediction owns a cloned client.
static CREDENTIAL_ACCESS: std::sync::Mutex<()> = std::sync::Mutex::new(());
const HTTP_REQUEST_TIMEOUT_SECONDS: u64 = 60;
const CONNECTION_TEST_MAX_TOKENS: u32 = 16;
const BATCH_MAX_TOKENS: u32 = 2048;

fn credential_access() -> Result<std::sync::MutexGuard<'static, ()>> {
    CREDENTIAL_ACCESS
        .lock()
        .map_err(|_| AppError::Config("密钥操作暂不可用，请重新启动应用".into()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LLMProvider {
    #[default]
    #[serde(rename = "deepseek")]
    DeepSeek,
    #[serde(rename = "openai")]
    OpenAi,
    Anthropic,
    Gemini,
}

impl LLMProvider {
    pub(crate) fn adapter_kind(self) -> AdapterKind {
        match self {
            Self::DeepSeek => AdapterKind::DeepSeek,
            Self::OpenAi => AdapterKind::OpenAI,
            Self::Anthropic => AdapterKind::Anthropic,
            Self::Gemini => AdapterKind::Gemini,
        }
    }

    pub(crate) fn infer_legacy(model: &str, base_url: Option<&str>) -> Self {
        let model = model.trim().to_ascii_lowercase();
        let base_url = base_url.unwrap_or_default().to_ascii_lowercase();
        if model.starts_with("deepseek-") || base_url.contains("deepseek") {
            Self::DeepSeek
        } else if model.starts_with("claude-") || base_url.contains("anthropic") {
            Self::Anthropic
        } else if model.starts_with("gemini-")
            || base_url.contains("generativelanguage.googleapis.com")
        {
            Self::Gemini
        } else {
            Self::OpenAi
        }
    }
}

#[derive(Deserialize)]
pub struct LLMConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub provider: LLMProvider,
    pub model: String,
}

impl Drop for LLMConfig {
    fn drop(&mut self) {
        self.api_key.zeroize();
    }
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            api_key: "".to_string(),
            provider: LLMProvider::DeepSeek,
            model: "deepseek-v4-pro".to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct LLMConfigStatus {
    pub provider: LLMProvider,
    pub model: String,
    pub has_api_key: bool,
}

fn http_client() -> Result<HttpClient> {
    HttpClient::builder()
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(HTTP_REQUEST_TIMEOUT_SECONDS))
        .build()
        .map_err(|_| AppError::Llm("无法初始化安全连接".into()))
}

fn safe_genai_error(error: genai::Error, stage: &str) -> AppError {
    if let genai::Error::HttpError { status, .. } = &error {
        return AppError::Llm(format!("请求失败（HTTP {}）", status.as_u16()));
    }
    if error.to_string().to_ascii_lowercase().contains("timeout") {
        return AppError::Llm(format!(
            "{stage}超时（{} 秒）",
            HTTP_REQUEST_TIMEOUT_SECONDS
        ));
    }
    AppError::Llm(format!("{stage}失败，请检查服务商、模型名称和网络连接"))
}

fn genai_client(config: &LLMConfig) -> Result<GenAiClient> {
    let secret = Arc::new(Zeroizing::new(config.api_key.clone()));
    let auth_resolver = AuthResolver::from_resolver_fn(move |_model_iden: ModelIden| {
        Ok(Some(AuthData::from_single(secret.as_str().to_owned())))
    });
    Ok(GenAiClient::builder()
        .with_auth_resolver(auth_resolver)
        .with_reqwest(http_client()?)
        .build())
}

fn model_iden(config: &LLMConfig) -> ModelIden {
    ModelIden::new(
        config.provider.adapter_kind(),
        config.model.trim().to_owned(),
    )
}

fn chat_options(provider: LLMProvider, max_tokens: u32, json_mode: bool) -> ChatOptions {
    let mut options = ChatOptions::default()
        .with_temperature(0.1)
        .with_max_tokens(max_tokens);
    if json_mode {
        options = options.with_response_format(ChatResponseFormat::JsonMode);
    }
    if provider == LLMProvider::DeepSeek {
        options = options.with_reasoning_effort(ReasoningEffort::None);
    }
    options
}

#[derive(Clone)]
pub struct LLMClient {
    file_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct BatchPredictionItem {
    bundle_id: String,
    #[serde(alias = "input_source_id")]
    preferred_input: String,
}

impl LLMClient {
    pub fn new() -> Self {
        Self {
            file_path: dirs::config_dir().map(|root| root.join("smartime/llm_config.json")),
        }
    }

    fn path(&self) -> Result<&std::path::Path> {
        self.file_path
            .as_deref()
            .ok_or_else(|| AppError::Config("无法定位用户配置目录".into()))
    }

    fn prepare(&self) -> Result<&std::path::Path> {
        let path = self.path()?;
        // Developer-only one-time import. Never fall back on corrupt files or Keychain errors.
        #[cfg(debug_assertions)]
        if !path.exists() {
            if let Some(config) = Self::load_from_env() {
                credentials::save(path, &Keychain, config)?;
            }
        }
        Ok(path)
    }

    pub fn update_config(&mut self, config: LLMConfig) -> Result<()> {
        let _access = credential_access()?;
        credentials::save(self.path()?, &Keychain, config)
    }

    pub fn get_config(&self) -> Result<LLMConfigStatus> {
        let _access = credential_access()?;
        credentials::status(self.prepare()?, &Keychain)
    }

    pub fn request_config(&self, input: Option<LLMConfig>) -> Result<LLMConfig> {
        let _access = credential_access()?;
        credentials::resolve(self.prepare()?, &Keychain, input)
    }

    pub fn delete_key(&mut self) -> Result<()> {
        let _access = credential_access()?;
        credentials::delete(self.path()?, &Keychain)
    }

    /// 从 .env.llm 文件加载配置
    #[cfg(debug_assertions)]
    fn load_from_env() -> Option<LLMConfig> {
        // 尝试加载 .env.llm
        let env_path = PathBuf::from(".env.llm");
        if env_path.exists() {
            dotenvy::from_filename(env_path).ok();
        }

        let api_key = env::var("LLM_API_KEY").ok()?;
        let defaults = LLMConfig::default();
        let model = env::var("LLM_MODEL").unwrap_or_else(|_| defaults.model.clone());
        let provider = env::var("LLM_PROVIDER")
            .ok()
            .and_then(|value| {
                serde_json::from_str(&format!("\"{}\"", value.to_ascii_lowercase())).ok()
            })
            .unwrap_or_else(|| {
                LLMProvider::infer_legacy(&model, env::var("LLM_BASE_URL").ok().as_deref())
            });

        Some(LLMConfig {
            api_key,
            provider,
            model,
        })
    }

    /// 检查 LLM 连接配置是否有效
    pub async fn check_connection(config: &LLMConfig) -> Result<()> {
        if config.api_key.is_empty() {
            return Err(AppError::Llm("API Key cannot be empty".to_string()));
        }

        if config.model.trim().is_empty() {
            return Err(AppError::Config("请输入模型名称".into()));
        }

        let client = genai_client(config)?;
        let request = ChatRequest::from_user("Hi");
        let options = chat_options(config.provider, CONNECTION_TEST_MAX_TOKENS, false);
        client
            .exec_chat(model_iden(config), request, Some(&options))
            .await
            .map_err(|error| safe_genai_error(error, "连接测试"))?;

        Ok(())
    }

    pub async fn predict_batch(
        config: &LLMConfig,
        apps: &[(String, String)],
        input_sources: &[InputSource],
        preferred_language: Option<&str>,
    ) -> Result<HashMap<String, String>> {
        if apps.is_empty() {
            return Ok(HashMap::new());
        }

        let prompt = build_prediction_prompt(apps, input_sources, preferred_language);

        let client = genai_client(config)?;
        let request = ChatRequest::from_user(prompt);
        let options = chat_options(config.provider, BATCH_MAX_TOKENS, true);
        let completion = client
            .exec_chat(model_iden(config), request, Some(&options))
            .await
            .map_err(|error| safe_genai_error(error, "等待模型响应"))?;
        let Some(content) = completion.first_text() else {
            return Err(AppError::Llm("No response from AI".to_string()));
        };

        parse_batch_prediction_response(content, apps, input_sources)
    }
}

fn build_prediction_prompt(
    apps: &[(String, String)],
    input_sources: &[InputSource],
    preferred_language: Option<&str>,
) -> String {
    let sources_desc = input_sources
        .iter()
        .map(|s| format!("- ID: {}, Name: {}", s.id, s.name))
        .collect::<Vec<_>>()
        .join("\n");
    let apps_desc = apps
        .iter()
        .map(|(name, bundle_id)| format!("- Bundle ID: {bundle_id}, Name: {name}"))
        .collect::<Vec<_>>()
        .join("\n");
    let preferred_language = preferred_language.unwrap_or("unknown");

    format!(
        r#"You are an intelligent assistant for macOS input method switching.

Available Input Sources:
{sources_desc}

User Context:
- macOS preferred language: {preferred_language}

Target Applications:
{apps_desc}

Task:
Select the most appropriate input source ID for every target application.
- For code editors (VS Code, IntelliJ, Terminal), English is usually preferred.
- If a Chinese input source is available, prefer it for Chinese-first communication, content, productivity, and consumer apps such as WeChat, QQ, Kimi, Douyin, Notes, and Reminders.
- Use the localized app name, Bundle ID, and macOS preferred language as evidence. Do not default every application to the first available input source.
- For browsers, English is a safe default unless it is a specific Chinese site wrapper.

Response Format:
Return only a JSON object mapping each target Bundle ID to exactly one available input source ID.
Example:
{{"com.tencent.xinWeChat":"com.apple.inputmethod.SCIM.ITABC","com.microsoft.VSCode":"com.apple.keylayout.ABC"}}
"#,
        sources_desc = sources_desc,
        apps_desc = apps_desc,
        preferred_language = preferred_language
    )
}

fn parse_batch_prediction_response(
    content: &str,
    apps: &[(String, String)],
    input_sources: &[InputSource],
) -> Result<HashMap<String, String>> {
    let target_bundle_ids: HashSet<&str> = apps
        .iter()
        .map(|(_, bundle_id)| bundle_id.as_str())
        .collect();
    let available_input_ids: HashSet<&str> = input_sources
        .iter()
        .map(|source| source.id.as_str())
        .collect();
    let json = extract_json_payload(content)?;
    let value: serde_json::Value = serde_json::from_str(json)?;

    let mut predictions = HashMap::new();
    collect_batch_predictions(
        &value,
        &target_bundle_ids,
        &available_input_ids,
        &mut predictions,
    );
    Ok(predictions)
}

fn extract_json_payload(content: &str) -> Result<&str> {
    let trimmed = content.trim();
    let start = trimmed.find(|ch| ch == '{' || ch == '[').ok_or_else(|| {
        AppError::Llm("Batch prediction response did not contain JSON".to_string())
    })?;
    let end = trimmed.rfind(|ch| ch == '}' || ch == ']').ok_or_else(|| {
        AppError::Llm("Batch prediction response did not contain JSON".to_string())
    })?;

    if end < start {
        return Err(AppError::Llm(
            "Batch prediction response contained malformed JSON boundaries".to_string(),
        ));
    }

    Ok(&trimmed[start..=end])
}

fn collect_batch_predictions(
    value: &serde_json::Value,
    target_bundle_ids: &HashSet<&str>,
    available_input_ids: &HashSet<&str>,
    predictions: &mut HashMap<String, String>,
) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(nested) = object.get("predictions").or_else(|| object.get("rules")) {
                collect_batch_predictions(
                    nested,
                    target_bundle_ids,
                    available_input_ids,
                    predictions,
                );
                return;
            }

            if let Ok(item) = serde_json::from_value::<BatchPredictionItem>(value.clone()) {
                add_prediction(
                    &item.bundle_id,
                    &item.preferred_input,
                    target_bundle_ids,
                    available_input_ids,
                    predictions,
                );
                return;
            }

            let direct_map = object.values().all(|value| value.as_str().is_some());
            if direct_map {
                for (bundle_id, input_id) in object {
                    add_prediction(
                        bundle_id,
                        input_id.as_str().unwrap_or_default(),
                        target_bundle_ids,
                        available_input_ids,
                        predictions,
                    );
                }
                return;
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_batch_predictions(
                    item,
                    target_bundle_ids,
                    available_input_ids,
                    predictions,
                );
            }
        }
        _ => {}
    }
}

fn add_prediction(
    bundle_id: &str,
    input_id: &str,
    target_bundle_ids: &HashSet<&str>,
    available_input_ids: &HashSet<&str>,
    predictions: &mut HashMap<String, String>,
) {
    if target_bundle_ids.contains(bundle_id) && available_input_ids.contains(input_id) {
        predictions.insert(bundle_id.to_string(), input_id.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_provider_selects_native_genai_adapter() {
        assert_eq!(
            LLMProvider::DeepSeek.adapter_kind(),
            genai::adapter::AdapterKind::DeepSeek
        );
        assert_eq!(
            LLMProvider::OpenAi.adapter_kind(),
            genai::adapter::AdapterKind::OpenAI
        );
        assert_eq!(
            LLMProvider::Anthropic.adapter_kind(),
            genai::adapter::AdapterKind::Anthropic
        );
        assert_eq!(
            LLMProvider::Gemini.adapter_kind(),
            genai::adapter::AdapterKind::Gemini
        );
    }

    #[test]
    fn legacy_provider_is_inferred_from_model_or_service_address() {
        assert_eq!(
            LLMProvider::infer_legacy("deepseek-v4-pro", Some("https://api.deepseek.com")),
            LLMProvider::DeepSeek
        );
        assert_eq!(
            LLMProvider::infer_legacy("claude-3-5-haiku-latest", None),
            LLMProvider::Anthropic
        );
        assert_eq!(
            LLMProvider::infer_legacy("gemini-2.5-flash", None),
            LLMProvider::Gemini
        );
    }

    fn test_apps() -> Vec<(String, String)> {
        vec![
            ("Safari".to_string(), "com.apple.Safari".to_string()),
            ("Code".to_string(), "com.microsoft.VSCode".to_string()),
        ]
    }

    fn test_sources() -> Vec<InputSource> {
        vec![
            InputSource {
                id: "com.apple.keylayout.ABC".to_string(),
                name: "ABC".to_string(),
                category: "keyboard".to_string(),
            },
            InputSource {
                id: "com.apple.inputmethod.SCIM.ITABC".to_string(),
                name: "简体拼音".to_string(),
                category: "inputmethod".to_string(),
            },
        ]
    }

    #[test]
    fn parse_batch_prediction_response_accepts_direct_json_map() {
        let parsed = parse_batch_prediction_response(
            r#"{
              "com.apple.Safari": "com.apple.inputmethod.SCIM.ITABC",
              "com.microsoft.VSCode": "com.apple.keylayout.ABC"
            }"#,
            &test_apps(),
            &test_sources(),
        )
        .expect("parse batch response");

        assert_eq!(
            parsed.get("com.apple.Safari"),
            Some(&"com.apple.inputmethod.SCIM.ITABC".to_string())
        );
        assert_eq!(
            parsed.get("com.microsoft.VSCode"),
            Some(&"com.apple.keylayout.ABC".to_string())
        );
    }

    #[test]
    fn parse_batch_prediction_response_ignores_unknown_and_invalid_entries() {
        let parsed = parse_batch_prediction_response(
            r#"```json
            {
              "com.apple.Safari": "com.apple.inputmethod.SCIM.ITABC",
              "com.example.Unknown": "com.apple.keylayout.ABC",
              "com.microsoft.VSCode": "com.example.missing"
            }
            ```"#,
            &test_apps(),
            &test_sources(),
        )
        .expect("parse batch response");

        assert_eq!(parsed.len(), 1);
        assert!(parsed.contains_key("com.apple.Safari"));
    }

    #[test]
    fn parse_batch_prediction_response_accepts_rules_array() {
        let parsed = parse_batch_prediction_response(
            r#"{
              "rules": [
                {
                  "bundle_id": "com.apple.Safari",
                  "preferred_input": "com.apple.inputmethod.SCIM.ITABC"
                },
                {
                  "bundle_id": "com.microsoft.VSCode",
                  "input_source_id": "com.apple.keylayout.ABC"
                }
              ]
            }"#,
            &test_apps(),
            &test_sources(),
        )
        .expect("parse batch response");

        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn parse_batch_prediction_response_rejects_malformed_json() {
        let parsed = parse_batch_prediction_response("not json", &test_apps(), &test_sources());

        assert!(parsed.is_err());
    }

    #[test]
    fn prediction_prompt_includes_locale_and_balanced_language_examples() {
        let prompt = build_prediction_prompt(&test_apps(), &test_sources(), Some("zh-Hans-CN"));

        assert!(prompt.contains("macOS preferred language: zh-Hans-CN"));
        assert!(prompt.contains("com.tencent.xinWeChat"));
        assert!(prompt.contains("com.apple.inputmethod.SCIM.ITABC"));
        assert!(prompt.contains("com.microsoft.VSCode"));
        assert!(prompt.contains("com.apple.keylayout.ABC"));
    }

    #[test]
    fn batch_options_are_bounded_and_request_json_for_every_provider() {
        for provider in [
            LLMProvider::DeepSeek,
            LLMProvider::OpenAi,
            LLMProvider::Anthropic,
            LLMProvider::Gemini,
        ] {
            let options = chat_options(provider, BATCH_MAX_TOKENS, true);
            assert_eq!(options.max_tokens, Some(BATCH_MAX_TOKENS));
            assert!(matches!(
                options.response_format,
                Some(ChatResponseFormat::JsonMode)
            ));
        }
    }
}
