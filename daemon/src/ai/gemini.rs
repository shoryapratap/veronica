use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub args: Value,
}

#[derive(Debug, Clone, Default)]
pub struct AiResponse {
    pub text: Option<String>,
    pub audio_data: Option<String>,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Clone)]
pub struct GeminiClient {
    api_key: String,
    model: Arc<RwLock<String>>,
    http_client: reqwest::Client,
}

impl GeminiClient {
    pub fn new(api_key: String, default_model: String) -> Self {
        Self {
            api_key,
            model: Arc::new(RwLock::new(default_model)),
            http_client: reqwest::Client::new(),
        }
    }

    /// Gets the name of the currently active model (e.g. "gemini-2.0-flash")
    pub async fn get_model(&self) -> String {
        self.model.read().await.clone()
    }

    /// Dynamically switches the active model at runtime without restarting the daemon!
    pub async fn set_model(&self, new_model: &str) {
        let mut model_lock = self.model.write().await;
        *model_lock = new_model.to_string();
        println!("🔄 [Veronica AI] Switched active model to: {}", new_model);
    }

    /// Sends a prompt to Gemini with system instructions and OS tool definitions
    pub async fn prompt(&self, user_input: &str) -> Result<AiResponse, String> {
        let current_model = self.get_model().await;
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            current_model, self.api_key
        );

        let body = serde_json::json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [{ "text": user_input }]
                }
            ],
            "system_instruction": {
                "parts": [{
                    "text": "You are Veronica, an autonomous personal AI assistant running directly inside the user's operating system. You have direct control over the OS tools. When a user asks you to open an application or change your AI model, call the appropriate tool. Always keep your verbal responses natural, concise, and helpful."
                }]
            },
            "generation_config": {
                "response_modalities": ["AUDIO"],
                "speech_config": {
                    "voice_config": {
                        "prebuilt_voice_config": {
                            "voice_name": "Aoede"
                        }
                    }
                }
            },
            "tools": [
                {
                    "function_declarations": [
                        {
                            "name": "launch_app",
                            "description": "Launches an application by name on the user's operating system (e.g. notepad, calc, spotify, chrome)",
                            "parameters": {
                                "type": "OBJECT",
                                "properties": {
                                    "target": {
                                        "type": "STRING",
                                        "description": "The exact application name or executable to launch"
                                    }
                                },
                                "required": ["target"]
                            }
                        },
                        {
                            "name": "change_model",
                            "description": "Switches Veronica's active AI model dynamically (e.g. gemini-1.5-pro, gemini-2.0-flash)",
                            "parameters": {
                                "type": "OBJECT",
                                "properties": {
                                    "new_model": {
                                        "type": "STRING",
                                        "description": "The target model name"
                                    }
                                },
                                "required": ["new_model"]
                            }
                        }
                    ]
                }
            ]
        });

        let response = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error calling Gemini: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let err_body = response.text().await.unwrap_or_default();
            return Err(format!("Gemini API Error ({}): {}", status, err_body));
        }

        let res_json: Value = response.json().await
            .map_err(|e| format!("Failed to parse Gemini response JSON: {}", e))?;

        let mut ai_res = AiResponse::default();

        if let Some(candidates) = res_json.get("candidates").and_then(|c| c.as_array()) {
            if let Some(first_candidate) = candidates.first() {
                if let Some(parts) = first_candidate.pointer("/content/parts").and_then(|p| p.as_array()) {
                    for part in parts {
                        // Extract spoken text response
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            ai_res.text = Some(text.to_string());
                        }

                        // Extract native audio bytes from Gemini
                        if let Some(inline_data) = part.get("inlineData") {
                            if let Some(data) = inline_data.get("data").and_then(|d| d.as_str()) {
                                ai_res.audio_data = Some(data.to_string());
                            }
                        }

                        // Extract tool / function calls
                        if let Some(func_call) = part.get("functionCall") {
                            if let Some(name) = func_call.get("name").and_then(|n| n.as_str()) {
                                let args = func_call.get("args").cloned().unwrap_or(Value::Null);
                                ai_res.tool_calls.push(ToolCall {
                                    name: name.to_string(),
                                    args,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(ai_res)
    }
}
