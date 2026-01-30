use crate::error::{AppError, Result};
use crate::input_source::InputSource;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

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

impl LLMClient {
    pub fn new() -> Self {
        // 尝试从 .env.llm 加载
        let config = Self::load_from_env().unwrap_or_default();
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn update_config(&mut self, config: LLMConfig) {
        self.config = config;
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

    /// 预测应用最合适的输入法
    pub async fn predict(
        &self,
        app_name: &str,
        bundle_id: &str,
        input_sources: &[InputSource],
    ) -> Result<String> {
        if self.config.api_key.is_empty() {
            return Err(AppError::Llm("API Key not configured".to_string()));
        }

        let sources_desc = input_sources
            .iter()
            .map(|s| format!("- ID: {}, Name: {}", s.id, s.name))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            r#"You are an intelligent assistant for macOS input method switching.
Target Application:
- Name: {app_name}
- Bundle ID: {bundle_id}

Available Input Sources:
{sources_desc}

Task:
Select the most appropriate input source ID for the target application.
- For code editors (VS Code, IntelliJ, Terminal), English is usually preferred.
- For chat apps (WeChat, WhatsApp), local language (Chinese) is often preferred, but depends on context.
- For browsers, English is a safe default unless it's a specific Chinese site wrapper.

Response Format:
Just output the ID string of the selected input source. Do not output any other text.
"#,
            app_name = app_name,
            bundle_id = bundle_id,
            sources_desc = sources_desc
        );

        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: 0.1, // 低温度以获得确定性结果
        };

        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));

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
        
        if let Some(choice) = completion.choices.first() {
            let selected_id = choice.message.content.trim().to_string();
            // 验证返回的 ID 是否在列表中
            if input_sources.iter().any(|s| s.id == selected_id) {
                Ok(selected_id)
            } else {
                // 如果返回的 ID 不存在，尝试模糊匹配或回退
                // 这里简单处理：如果找不到，报错
                Err(AppError::Llm(format!("AI returned invalid ID: {}", selected_id)))
            }
        } else {
            Err(AppError::Llm("No response from AI".to_string()))
        }
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
}
