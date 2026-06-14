use crate::error::{AppError, Result};
use crate::input_source::InputSource;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
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

#[derive(Clone)]
pub struct LLMClient {
    client: Client,
    config: LLMConfig,
    file_path: PathBuf,
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
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("smartime");
        let _ = fs::create_dir_all(&config_dir);
        let file_path = config_dir.join("llm_config.json");

        // 优先读取持久化配置，随后回退到 .env.llm
        let config = Self::load_from_file(&file_path)
            .or_else(Self::load_from_env)
            .unwrap_or_default();

        Self {
            client: Client::new(),
            config,
            file_path,
        }
    }

    pub fn update_config(&mut self, config: LLMConfig) -> Result<()> {
        self.config = config;
        self.save_to_file()
    }

    pub fn get_config(&self) -> LLMConfig {
        self.config.clone()
    }

    /// 从 .env.llm 文件加载配置
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

    fn load_from_file(path: &Path) -> Option<LLMConfig> {
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn save_to_file(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    /// 检查 LLM 连接配置是否有效
    pub async fn check_connection(config: &LLMConfig) -> Result<()> {
        if config.api_key.is_empty() {
            return Err(AppError::Llm("API Key cannot be empty".to_string()));
        }

        let client = Client::new();
        let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));

        let request = ChatCompletionRequest {
            model: config.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            temperature: 0.1,
        };

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(AppError::Llm(format!("Connection failed: {}", error_text)));
        }

        Ok(())
    }

    pub async fn predict_batch(
        &self,
        apps: &[(String, String)],
        input_sources: &[InputSource],
    ) -> Result<HashMap<String, String>> {
        if self.config.api_key.is_empty() {
            return Err(AppError::Llm("API Key not configured".to_string()));
        }
        if apps.is_empty() {
            return Ok(HashMap::new());
        }

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

        let prompt = format!(
            r#"You are an intelligent assistant for macOS input method switching.

Available Input Sources:
{sources_desc}

Target Applications:
{apps_desc}

Task:
Select the most appropriate input source ID for every target application.
- For code editors (VS Code, IntelliJ, Terminal), English is usually preferred.
- For chat apps (WeChat, WhatsApp), local language (Chinese) is often preferred, but depends on context.
- For browsers, English is a safe default unless it is a specific Chinese site wrapper.

Response Format:
Return only a JSON object mapping each target Bundle ID to exactly one available input source ID.
Example:
{{"com.example.App":"com.apple.keylayout.ABC"}}
"#,
            sources_desc = sources_desc,
            apps_desc = apps_desc
        );

        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: 0.1,
        };

        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(AppError::Llm(format!("API request failed: {}", error_text)));
        }

        let completion: ChatCompletionResponse = resp.json().await?;
        let Some(choice) = completion.choices.first() else {
            return Err(AppError::Llm("No response from AI".to_string()));
        };

        parse_batch_prediction_response(&choice.message.content, apps, input_sources)
    }
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
    fn test_load_env() {
        // 设置环境变量进行测试
        std::env::set_var("LLM_API_KEY", "test-key");
        std::env::set_var("LLM_MODEL", "test-model");

        let config = LLMClient::load_from_env();
        assert!(config.is_some());
        let c = config.unwrap();
        assert_eq!(c.api_key, "test-key");
        assert_eq!(c.model, "test-model");
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
}
