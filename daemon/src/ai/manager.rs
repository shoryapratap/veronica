use std::sync::Arc;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use crate::schedule::ScheduleEngine;

/// ToolManager coordinates the 2-step tool discovery & execution flow
/// by delegating schema resolution and execution to each feature's Master Function.
#[derive(Clone)]
pub struct ToolManager {
    schedule: Arc<ScheduleEngine>,
}

impl ToolManager {
    /// Create a new ToolManager instance
    pub fn new(schedule: Arc<ScheduleEngine>) -> Self {
        Self { schedule }
    }

    /// Exposes the 2-step tools to Google Gemini Live:
    /// 1. select_feature_function: Ask the feature's Master Function for its exact JSON schema
    /// 2. execute_feature_function: Submit filled JSON parameters to the feature's Master Function
    pub fn get_declarations() -> serde_json::Value {
        json!([
            {
                "name": "select_feature_function",
                "description": "Step 1: Select a feature and function to retrieve its exact JSON structure from its Master Function.",
                "parameters": {
                    "type": "OBJECT",
                    "properties": {
                        "feature": {
                            "type": "STRING",
                            "description": "Target feature domain",
                            "enum": ["schedule", "system"]
                        },
                        "function": {
                            "type": "STRING",
                            "description": "Specific function to inspect (e.g. 'create', 'list', 'cancel', 'capture_screen', 'launch_app')"
                        }
                    },
                    "required": ["feature", "function"]
                }
            },
            {
                "name": "execute_feature_function",
                "description": "Step 2: Submit filled parameter values to the feature's Master Function for execution.",
                "parameters": {
                    "type": "OBJECT",
                    "properties": {
                        "feature": {
                            "type": "STRING",
                            "description": "Target feature domain",
                            "enum": ["schedule", "system"]
                        },
                        "function": {
                            "type": "STRING",
                            "description": "Function name to execute"
                        },
                        "data": {
                            "type": "OBJECT",
                            "description": "Filled JSON arguments matching the function schema"
                        }
                    },
                    "required": ["feature", "function", "data"]
                }
            }
        ])
    }

    /// Dispatches tool calls received from Gemini, executes them, and formats the toolResponse
    pub async fn dispatch(
        &self,
        name: &str,
        args: &serde_json::Value,
        call_id: Option<&str>,
    ) -> Message {
        println!("🛠️  [Tool Manager] AI invoked: '{}'", name);
        println!("   📦 Arguments: {}", args);

        let output_value = match name {
            // STEP 1: AI selects a feature & function -> Manager queries the feature's Master Function for Schema
            "select_feature_function" => {
                let feature = args.get("feature").and_then(|v| v.as_str()).unwrap_or("");
                let function = args.get("function").and_then(|v| v.as_str()).unwrap_or("");

                let schema = match feature {
                    "schedule" => ScheduleEngine::master_schema(function),
                    "system" => crate::system::master_schema(function),
                    other => json!({
                        "error": format!("Feature '{}' not found in system", other)
                    }),
                };

                println!("   📋 [Master Function Schema]: Provided exact schema for {}.{}", feature, function);
                json!({
                    "status": "schema_ready",
                    "feature": feature,
                    "function": function,
                    "expected_schema": schema,
                    "instruction": "Now call 'execute_feature_function' with 'data' filled according to this exact schema."
                })
            }

            // STEP 2: AI provides filled JSON -> Manager sends to the feature's Master Function for Execution
            "execute_feature_function" => {
                let feature = args.get("feature").and_then(|v| v.as_str()).unwrap_or("");
                let function = args.get("function").and_then(|v| v.as_str()).unwrap_or("");
                let empty_obj = json!({});
                let data = args.get("data").unwrap_or(&empty_obj);

                let result = match feature {
                    "schedule" => self.schedule.master_execute(function, data).await,
                    "system" => crate::system::master_execute(function, data).await,
                    other => json!({
                        "status": "error",
                        "message": format!("Feature '{}' not recognized", other)
                    }),
                };

                println!("   ✅ [Master Function Executed]: Result for {}.{}", feature, function);
                result
            }

            other => {
                eprintln!("   ⚠️ [Tool Manager] Unknown meta-tool: '{}'", other);
                json!({
                    "status": "unknown_tool",
                    "message": format!("Tool '{}' is not recognized", other)
                })
            }
        };

        // Construct Gemini Live toolResponse format
        let mut resp_obj = json!({
            "response": {
                "output": output_value
            }
        });

        if let Some(id) = call_id {
            resp_obj["id"] = json!(id);
        }

        let tool_response = json!({
            "toolResponse": {
                "functionResponses": [resp_obj]
            }
        });

        Message::Text(tool_response.to_string().into())
    }
}
