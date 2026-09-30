use std::sync::Arc;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use crate::schedule::ScheduleEngine;

/// ToolManager coordinates tool definitions and execution cleanly
/// so the AI can use all system and scheduling tools effectively.
#[derive(Clone)]
pub struct ToolManager {
    schedule: Arc<ScheduleEngine>,
}

impl ToolManager {
    /// Create a new ToolManager instance
    pub fn new(schedule: Arc<ScheduleEngine>) -> Self {
        Self { schedule }
    }

    /// Complete function declarations exposed to Google Gemini Live
    pub fn get_declarations() -> serde_json::Value {
        json!([
            {
                "name": "create_schedule",
                "description": "Create a new calendar meeting, reminder, timer, or habit in PostgreSQL. Store detailed description of user context.",
                "parameters": {
                    "type": "OBJECT",
                    "properties": {
                        "title": {
                            "type": "STRING",
                            "description": "Short, clear title for the event or reminder"
                        },
                        "description": {
                            "type": "STRING",
                            "description": "Detailed context: what the user specifically asked for, why, and any extra notes so Veronica knows exactly what she's reminding about"
                        },
                        "time_expression": {
                            "type": "STRING",
                            "description": "Natural time expression when to trigger (e.g. 'in 2 minutes', 'tomorrow at 3 PM', 'in 1 hour')"
                        },
                        "duration_minutes": {
                            "type": "INTEGER",
                            "description": "Duration in minutes (e.g. 30, 45, 60 for meetings, 0 for instant reminders)"
                        },
                        "recurrence": {
                            "type": "STRING",
                            "description": "Recurrence pattern: 'none', 'daily', or 'weekly'",
                            "enum": ["none", "daily", "weekly"]
                        },
                        "schedule_type": {
                            "type": "STRING",
                            "description": "Type of event: 'reminder', 'meeting', 'habit', or 'timer'",
                            "enum": ["reminder", "meeting", "habit", "timer"]
                        }
                    },
                    "required": ["title", "description", "time_expression"]
                }
            },
            {
                "name": "get_schedule",
                "description": "Retrieve upcoming scheduled tasks, meetings, and reminders from PostgreSQL including their full context descriptions.",
                "parameters": {
                    "type": "OBJECT",
                    "properties": {
                        "limit": {
                            "type": "INTEGER",
                            "description": "Maximum number of upcoming items to return (default 10)"
                        }
                    }
                }
            },
            {
                "name": "cancel_schedule",
                "description": "Cancel an existing scheduled event or reminder by its title or ID.",
                "parameters": {
                    "type": "OBJECT",
                    "properties": {
                        "identifier": {
                            "type": "STRING",
                            "description": "The title or ID of the schedule to cancel"
                        }
                    },
                    "required": ["identifier"]
                }
            },
            {
                "name": "capture_screen",
                "description": "Inspect the user's primary monitor to view and analyze whatever is currently on screen.",
                "parameters": {
                    "type": "OBJECT",
                    "properties": {
                        "reason": {
                            "type": "STRING",
                            "description": "Reason for capturing the screen"
                        }
                    }
                }
            },
            {
                "name": "launch_app",
                "description": "Launch a desktop application or URL on the local operating system.",
                "parameters": {
                    "type": "OBJECT",
                    "properties": {
                        "app_name": {
                            "type": "STRING",
                            "description": "Name or executable of the application (e.g., 'notepad', 'chrome', 'calc', 'code')"
                        }
                    },
                    "required": ["app_name"]
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
        println!("🛠️  [Tool Manager] AI executing tool: '{}'", name);
        println!("   📦 Arguments: {}", args);

        let output_value = match name {
            "create_schedule" => {
                let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("Reminder");
                let description = args.get("description").and_then(|v| v.as_str())
                    .unwrap_or("User requested task reminder");
                let time_expr = args.get("time_expression").and_then(|v| v.as_str())
                    .unwrap_or("in 5 minutes");
                let duration = args.get("duration_minutes").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let recurrence = args.get("recurrence").and_then(|v| v.as_str()).unwrap_or("none");
                let sched_type = args.get("schedule_type").and_then(|v| v.as_str()).unwrap_or("reminder");

                match self.schedule.create_schedule(
                    title,
                    description,
                    time_expr,
                    duration,
                    Some(recurrence),
                    Some(sched_type),
                ).await {
                    Ok(item) => {
                        println!("   ✅ [Tool Manager] Schedule saved to PostgreSQL: '{}' at {}", item.title, item.target_time);
                        json!({
                            "status": "success",
                            "message": format!("Successfully scheduled '{}' for {} with context saved.", item.title, item.target_time),
                            "item": item
                        })
                    }
                    Err(e) => {
                        eprintln!("   ❌ [Tool Manager] Schedule error: {}", e);
                        json!({
                            "status": "error",
                            "message": format!("Failed to create schedule: {}", e)
                        })
                    }
                }
            }

            "get_schedule" => {
                let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
                match self.schedule.list_upcoming(limit).await {
                    Ok(items) => {
                        println!("   📅 [Tool Manager] Retrieved {} upcoming schedules from PostgreSQL", items.len());
                        json!({
                            "status": "success",
                            "count": items.len(),
                            "schedules": items
                        })
                    }
                    Err(e) => {
                        eprintln!("   ❌ [Tool Manager] Get schedule error: {}", e);
                        json!({
                            "status": "error",
                            "message": format!("Failed to retrieve schedules: {}", e)
                        })
                    }
                }
            }

            "cancel_schedule" => {
                let identifier = args.get("identifier").and_then(|v| v.as_str()).unwrap_or("");
                match self.schedule.cancel_schedule(identifier).await {
                    Ok(true) => {
                        println!("   🗑️ [Tool Manager] Cancelled schedule matching '{}'", identifier);
                        json!({
                            "status": "success",
                            "message": format!("Successfully cancelled schedule matching '{}'", identifier)
                        })
                    }
                    Ok(false) => {
                        json!({
                            "status": "not_found",
                            "message": format!("No pending schedule found matching '{}'", identifier)
                        })
                    }
                    Err(e) => {
                        json!({
                            "status": "error",
                            "message": format!("Error cancelling schedule: {}", e)
                        })
                    }
                }
            }

            "capture_screen" => {
                println!("   📸 [Tool Manager] Screen capture requested by AI");
                match crate::vision::capture_primary_screen() {
                    Ok(jpeg_bytes) => {
                        json!({
                            "status": "success",
                            "message": "Screen snapshot captured successfully",
                            "bytes_len": jpeg_bytes.len()
                        })
                    }
                    Err(e) => {
                        json!({
                            "status": "error",
                            "message": format!("Failed to capture screen: {}", e)
                        })
                    }
                }
            }

            "launch_app" => {
                let app = args.get("app_name").and_then(|v| v.as_str()).unwrap_or("");
                println!("   🚀 [Tool Manager] Launching application: {}", app);
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

            other => {
                eprintln!("   ⚠️ [Tool Manager] Unknown tool: '{}'", other);
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
