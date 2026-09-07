use crate::credentials::{self, Keychain};
use crate::error::{AppError, Result};
use crate::input_source::InputSource;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
#[cfg(debug_assertions)]
use std::env;
use std::path::PathBuf;
use zeroize::Zeroize;

// Serialize migration and updates even when a prediction owns a cloned client.
static CREDENTIAL_ACCESS: std::sync::Mutex<()> = std::sync::Mutex::new(());
const HTTP_REQUEST_TIMEOUT_SECONDS: u64 = 60;
const DEEPSEEK_BATCH_MAX_TOKENS: u32 = 2048;

fn credential_access() -> Result<std::sync::MutexGuard<'static, ()>> {
    CREDENTIAL_ACCESS
        .lock()
        .map_err(|_| AppError::Config("密钥操作暂不可用，请重新启动应用".into()))
}

#[derive(Deserialize)]
pub struct LLMConfig {
    #[serde(default)]
    pub api_key: String,
    pub model: String,
    pub base_url: String,
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
            model: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct LLMConfigStatus {
    pub model: String,
    pub base_url: String,
    pub has_api_key: bool,
}

pub(crate) fn endpoint(base_url: &str) -> Result<reqwest::Url> {
    let url = reqwest::Url::parse(base_url)
        .map_err(|_| AppError::Config("请输入有效的 HTTPS 服务地址".into()))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(AppError::Config(
            "服务地址必须使用 HTTPS，且不能包含账号、密码、查询参数或片段".into(),
        ));
    }
    reqwest::Url::parse(&format!(
        "{}/chat/completions",
        url.as_str().trim_end_matches('/')
    ))
    .map_err(|_| AppError::Config("服务地址无效".into()))
}

fn http_client() -> Result<Client> {
    Client::builder()
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(HTTP_REQUEST_TIMEOUT_SECONDS))
        .build()
        .map_err(|_| AppError::Llm("无法初始化安全连接".into()))
}

fn safe_network_error(error: reqwest::Error, stage: &str) -> AppError {
    if error.is_timeout() {
        return AppError::Llm(format!(
            "{stage}超时（{} 秒）",
            HTTP_REQUEST_TIMEOUT_SECONDS
        ));
    }
    if error.is_connect() {
        return AppError::Llm("无法连接模型服务，请检查服务地址和网络连接".into());
    }
    if error.is_decode() {
        return AppError::Llm("读取模型响应失败：响应不是有效的 API JSON".into());
    }

    AppError::Llm(format!("{stage}失败，请检查模型服务状态"))
}

#[derive(Clone)]
pub struct LLMClient {
    file_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize)]
struct ThinkingConfig {
    #[serde(rename = "type")]
    mode: &'static str,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format: &'static str,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
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
        let model = env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
        let base_url =
            env::var("LLM_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        Some(LLMConfig {
            api_key,
            model,
            base_url,
        })
    }

    /// 检查 LLM 连接配置是否有效
    pub async fn check_connection(config: &LLMConfig) -> Result<()> {
        if config.api_key.is_empty() {
            return Err(AppError::Llm("API Key cannot be empty".to_string()));
        }

        let client = http_client()?;
        let url = endpoint(&config.base_url)?;

        let request = ChatCompletionRequest {
            model: config.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            temperature: 0.1,
            thinking: is_deepseek_model(&config.model)
                .then_some(ThinkingConfig { mode: "disabled" }),
            max_tokens: is_deepseek_model(&config.model).then_some(16),
            response_format: None,
        };

        let resp = client
            .post(url)
            .bearer_auth(&config.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|error| safe_network_error(error, "发送连接测试请求"))?;

        if !resp.status().is_success() {
            return Err(AppError::Llm(format!(
                "连接失败（HTTP {}）",
                resp.status().as_u16()
            )));
        }

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

        let request = build_batch_prediction_request(config, prompt);

        let url = endpoint(&config.base_url)?;
        let resp = http_client()?
            .post(url)
            .bearer_auth(&config.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|error| safe_network_error(error, "等待模型响应"))?;

        if !resp.status().is_success() {
            return Err(AppError::Llm(format!(
                "请求失败（HTTP {}）",
                resp.status().as_u16()
            )));
        }

        let completion: ChatCompletionResponse = resp
            .json()
            .await
            .map_err(|error| safe_network_error(error, "读取模型响应"))?;
        let Some(choice) = completion.choices.first() else {
            return Err(AppError::Llm("No response from AI".to_string()));
        };

        parse_batch_prediction_response(&choice.message.content, apps, input_sources)
    }
}

fn is_deepseek_model(model: &str) -> bool {
    model.trim().to_ascii_lowercase().starts_with("deepseek-")
}

fn build_batch_prediction_request(config: &LLMConfig, prompt: String) -> ChatCompletionRequest {
    let is_deepseek = is_deepseek_model(&config.model);

    ChatCompletionRequest {
        model: config.model.clone(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }],
        temperature: 0.1,
        thinking: is_deepseek.then_some(ThinkingConfig { mode: "disabled" }),
        max_tokens: is_deepseek.then_some(DEEPSEEK_BATCH_MAX_TOKENS),
        response_format: is_deepseek.then_some(ResponseFormat {
            format: "json_object",
        }),
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
    fn endpoints_reject_cleartext_and_embedded_secrets() {
        for url in [
            "http://example.com/v1",
            "http://127.0.0.1:8080",
            "https://user:pass@example.com",
            "https://example.com?key=secret",
            "https://example.com/#secret",
            "file:///tmp/key",
        ] {
            assert!(endpoint(url).is_err(), "accepted {url}");
        }
        assert_eq!(
            endpoint("https://example.com/v1/").unwrap().as_str(),
            "https://example.com/v1/chat/completions"
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
    fn deepseek_batch_request_disables_thinking_and_requires_json() {
        let config = LLMConfig {
            api_key: "test-key".to_string(),
            model: "deepseek-v4-pro".to_string(),
            base_url: "https://api.deepseek.com".to_string(),
        };

        let request = build_batch_prediction_request(&config, "predict these apps".to_string());
        let payload = serde_json::to_value(request).expect("serialize request");

        assert_eq!(payload["thinking"]["type"], "disabled");
        assert_eq!(payload["response_format"]["type"], "json_object");
        assert_eq!(payload["max_tokens"], 2048);
    }

    #[test]
    fn generic_provider_request_omits_deepseek_specific_controls() {
        let config = LLMConfig {
            api_key: "test-key".to_string(),
            model: "custom-chat-model".to_string(),
            base_url: "https://llm.example.com/v1".to_string(),
        };

        let request = build_batch_prediction_request(&config, "predict these apps".to_string());
        let payload = serde_json::to_value(request).expect("serialize request");

        assert!(payload.get("thinking").is_none());
        assert!(payload.get("response_format").is_none());
        assert!(payload.get("max_tokens").is_none());
    }
}
