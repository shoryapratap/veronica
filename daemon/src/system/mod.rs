pub mod command;
pub mod detector;
pub mod platform;

use serde_json::json;

/// System Feature Master Function (Schema Provider):
/// Returns the exact JSON structure for system functions.
pub fn master_schema(function_name: &str) -> serde_json::Value {
    match function_name {
        "capture_screen" => json!({
            "type": "OBJECT",
            "properties": {
                "reason": { "type": "STRING", "description": "Reason for inspecting screen" }
            }
        }),
        "launch_app" => json!({
            "type": "OBJECT",
            "properties": {
                "app_name": { "type": "STRING", "description": "Application name or executable (e.g. 'notepad', 'chrome', 'calc')" }
            },
            "required": ["app_name"]
        }),
        other => json!({
            "error": format!("Unknown function '{}' in system feature", other)
        })
    }
}

/// System Feature Master Function (Execution Dispatcher):
/// Receives filled values and executes the exact system action.
pub async fn master_execute(function_name: &str, data: &serde_json::Value) -> serde_json::Value {
    match function_name {
        "capture_screen" => {
            match crate::vision::capture_primary_screen() {
                Ok(bytes) => json!({
                    "status": "success",
                    "message": "Screen snapshot captured successfully",
                    "bytes_len": bytes.len()
                }),
                Err(e) => json!({
                    "status": "error",
                    "message": format!("Failed to capture screen: {}", e)
                })
            }
        }
        "launch_app" => {
            let app = data.get("app_name").and_then(|v| v.as_str()).unwrap_or("");
            let res = tokio::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", &format!("Start-Process '{}'", app)])
                .spawn();

            match res {
                Ok(_) => json!({
                    "status": "success",
                    "message": format!("Launched application '{}'", app)
                }),
                Err(e) => json!({
                    "status": "error",
                    "message": format!("Failed to launch '{}': {}", app, e)
                })
            }
        }
        other => json!({
            "status": "error",
            "message": format!("Unknown system function: '{}'", other)
        })
    }
}
