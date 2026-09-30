use std::sync::Arc;
use chrono::{Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_postgres::{Client, NoTls};

/// Schedule item representation stored in PostgreSQL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleItem {
    pub id: String,
    pub title: String,
    pub description: String, // Context of what user said / why Veronica should remind
    pub schedule_type: String, // reminder, meeting, habit, timer
    pub target_time: String,
    pub target_timestamp: i64,
    pub duration_minutes: i32,
    pub recurrence: String,
    pub status: String,
    pub created_at: String,
}

/// PostgreSQL-backed schedule and calendar engine with auto-cleanup queue
#[derive(Clone)]
pub struct ScheduleEngine {
    client: Arc<tokio::sync::Mutex<Option<Client>>>,
    db_url: String,
}

impl ScheduleEngine {
    /// Create a new ScheduleEngine with PostgreSQL connection string
    pub async fn new(db_url: Option<String>) -> Self {
        let url = db_url.unwrap_or_else(|| {
            std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/veronica_db".to_string())
        });

        let engine = Self {
            client: Arc::new(tokio::sync::Mutex::new(None)),
            db_url: url,
        };

        if let Err(e) = engine.ensure_connected().await {
            eprintln!("⚠️ [PostgreSQL Engine] Notice: Could not connect to PostgreSQL immediately: {}", e);
            eprintln!("💡 Veronica will retry automatically when scheduling tasks.");
        } else {
            println!("✅ [PostgreSQL Engine] Connected to PostgreSQL successfully!");
            if let Err(e) = engine.init_tables().await {
                eprintln!("⚠️ [PostgreSQL Engine] Table init warning: {}", e);
            }
        }

        engine
    }

    // ==========================================
    // 👑 MASTER FUNCTION: SCHEDULE FEATURE
    // ==========================================

    /// Schedule Feature Master Function (Schema Provider):
    /// Returns the exact JSON structure for any requested function in this feature.
    pub fn master_schema(function_name: &str) -> serde_json::Value {
        match function_name {
            "create" => json!({
                "type": "OBJECT",
                "properties": {
                    "title": { "type": "STRING", "description": "Short title of the task" },
                    "description": { "type": "STRING", "description": "Detailed context: what the user specifically asked for and why" },
                    "time_expression": { "type": "STRING", "description": "When to alert (e.g. 'in 2 minutes', 'tomorrow at 3 PM')" },
                    "duration_minutes": { "type": "INTEGER", "description": "Duration in minutes (optional)" },
                    "recurrence": { "type": "STRING", "enum": ["none", "daily", "weekly"] },
                    "schedule_type": { "type": "STRING", "enum": ["reminder", "meeting", "habit", "timer"] }
                },
                "required": ["title", "description", "time_expression"]
            }),
            "list" => json!({
                "type": "OBJECT",
                "properties": {
                    "limit": { "type": "INTEGER", "description": "Max upcoming items to retrieve (default 10)" }
                }
            }),
            "cancel" => json!({
                "type": "OBJECT",
                "properties": {
                    "identifier": { "type": "STRING", "description": "Title or ID of the schedule to cancel" }
                },
                "required": ["identifier"]
            }),
            other => json!({
                "error": format!("Unknown function '{}' in schedule feature", other)
            })
        }
    }

    /// Schedule Feature Master Function (Execution Dispatcher):
    /// Receives filled JSON values from manager, calls exact local function, and executes.
    pub async fn master_execute(&self, function_name: &str, data: &serde_json::Value) -> serde_json::Value {
        match function_name {
            "create" => {
                let title = data.get("title").and_then(|v| v.as_str()).unwrap_or("Reminder");
                let description = data.get("description").and_then(|v| v.as_str())
                    .unwrap_or("User requested task reminder");
                let time_expr = data.get("time_expression").and_then(|v| v.as_str())
                    .unwrap_or("in 5 minutes");
                let duration = data.get("duration_minutes").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let recurrence = data.get("recurrence").and_then(|v| v.as_str()).unwrap_or("none");
                let sched_type = data.get("schedule_type").and_then(|v| v.as_str()).unwrap_or("reminder");

                match self.create_schedule(
                    title,
                    description,
                    time_expr,
                    duration,
                    Some(recurrence),
                    Some(sched_type),
                ).await {
                    Ok(item) => json!({
                        "status": "success",
                        "message": format!("Successfully scheduled '{}' for {} with context saved.", item.title, item.target_time),
                        "item": item
                    }),
                    Err(e) => json!({
                        "status": "error",
                        "message": format!("Failed to create schedule: {}", e)
                    })
                }
            }

            "list" => {
                let limit = data.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
                match self.list_upcoming(limit).await {
                    Ok(items) => json!({
                        "status": "success",
                        "count": items.len(),
                        "schedules": items
                    }),
                    Err(e) => json!({
                        "status": "error",
                        "message": format!("Failed to retrieve schedules: {}", e)
                    })
                }
            }

            "cancel" => {
                let identifier = data.get("identifier").and_then(|v| v.as_str()).unwrap_or("");
                match self.cancel_schedule(identifier).await {
                    Ok(true) => json!({
                        "status": "success",
                        "message": format!("Successfully cancelled schedule matching '{}'", identifier)
                    }),
                    Ok(false) => json!({
                        "status": "not_found",
                        "message": format!("No pending schedule found matching '{}'", identifier)
                    }),
                    Err(e) => json!({
                        "status": "error",
                        "message": format!("Error cancelling schedule: {}", e)
                    })
                }
            }

            other => json!({
                "status": "error",
                "message": format!("Unknown schedule function: '{}'", other)
            })
        }
    }

    /// Ensure active connection to PostgreSQL
    async fn ensure_connected(&self) -> Result<(), String> {
        let mut guard = self.client.lock().await;
        if guard.is_some() {
            return Ok(());
        }

        match tokio_postgres::connect(&self.db_url, NoTls).await {
            Ok((client, connection)) => {
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("⚠️ [PostgreSQL] Connection stream closed: {}", e);
                    }
                });
                *guard = Some(client);
                Ok(())
            }
            Err(e) => Err(format!("PostgreSQL connection failed (URL: {}): {}", self.db_url, e)),
        }
    }

    /// Ensure the 'schedules' table exists in PostgreSQL with the description column
    pub async fn init_tables(&self) -> Result<(), String> {
        self.ensure_connected().await?;
        let guard = self.client.lock().await;
        let client = guard.as_ref().ok_or("No database client")?;

        let sql = r#"
            CREATE TABLE IF NOT EXISTS schedules (
                id VARCHAR(64) PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                description TEXT NOT NULL,
                schedule_type VARCHAR(64) NOT NULL,
                target_time VARCHAR(128) NOT NULL,
                target_timestamp BIGINT NOT NULL,
                duration_minutes INT DEFAULT 0,
                recurrence VARCHAR(64) DEFAULT 'none',
                status VARCHAR(64) NOT NULL DEFAULT 'pending',
                created_at VARCHAR(128) NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_schedules_target_ts ON schedules(target_timestamp);
            CREATE INDEX IF NOT EXISTS idx_schedules_status ON schedules(status);
        "#;

        client.batch_execute(sql).await
            .map_err(|e| format!("Failed to create schedules table: {}", e))?;

        println!("   📦 [PostgreSQL] Schedules table verified (with description column)");
        Ok(())
    }

    /// Create and persist a new schedule item into PostgreSQL
    pub async fn create_schedule(
        &self,
        title: &str,
        description: &str,
        time_expression: &str,
        duration_minutes: i32,
        recurrence: Option<&str>,
        schedule_type: Option<&str>,
    ) -> Result<ScheduleItem, String> {
        self.ensure_connected().await?;
        let guard = self.client.lock().await;
        let client = guard.as_ref().ok_or("No active PostgreSQL connection")?;

        let (target_timestamp, target_time_str) = parse_time_expression(time_expression)?;
        let item_id = format!("sch_{}", Utc::now().timestamp_millis());
        let sched_type = schedule_type.unwrap_or("reminder").to_string();
        let recur = recurrence.unwrap_or("none").to_string();
        let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let sql = r#"
            INSERT INTO schedules (
                id, title, description, schedule_type,
                target_time, target_timestamp, duration_minutes,
                recurrence, status, created_at
            ) VALUES (, , , , , , , , 'pending', )
        "#;

        client.execute(
            sql,
            &[
                &item_id,
                &title,
                &description,
                &sched_type,
                &target_time_str,
                &target_timestamp,
                &duration_minutes,
                &recur,
                &now_str,
            ],
        ).await.map_err(|e| format!("Failed to insert into PostgreSQL: {}", e))?;

        Ok(ScheduleItem {
            id: item_id,
            title: title.to_string(),
            description: description.to_string(),
            schedule_type: sched_type,
            target_time: target_time_str,
            target_timestamp,
            duration_minutes,
            recurrence: recur,
            status: "pending".to_string(),
            created_at: now_str,
        })
    }

    /// List upcoming pending schedules from PostgreSQL
    pub async fn list_upcoming(&self, limit: i64) -> Result<Vec<ScheduleItem>, String> {
        self.ensure_connected().await?;
        let guard = self.client.lock().await;
        let client = guard.as_ref().ok_or("No active PostgreSQL connection")?;

        let sql = r#"
            SELECT id, title, description, schedule_type,
                   target_time, target_timestamp, duration_minutes,
                   recurrence, status, created_at
            FROM schedules
            WHERE status = 'pending'
            ORDER BY target_timestamp ASC
            LIMIT 
        "#;

        let rows = client.query(sql, &[&limit]).await
            .map_err(|e| format!("Failed to query upcoming schedules: {}", e))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(ScheduleItem {
                id: row.get(0),
                title: row.get(1),
                description: row.get(2),
                schedule_type: row.get(3),
                target_time: row.get(4),
                target_timestamp: row.get(5),
                duration_minutes: row.get(6),
                recurrence: row.get(7),
                status: row.get(8),
                created_at: row.get(9),
            });
        }

        Ok(items)
    }

    /// Cancel a scheduled task by ID or title substring
    pub async fn cancel_schedule(&self, identifier: &str) -> Result<bool, String> {
        self.ensure_connected().await?;
        let guard = self.client.lock().await;
        let client = guard.as_ref().ok_or("No active PostgreSQL connection")?;

        let sql = r#"
            DELETE FROM schedules
            WHERE (id =  OR LOWER(title) LIKE LOWER()) AND status = 'pending'
        "#;

        let search_pattern = format!("%{}%", identifier);
        let rows_affected = client.execute(sql, &[&identifier, &search_pattern]).await
            .map_err(|e| format!("Failed to delete schedule in PostgreSQL: {}", e))?;

        Ok(rows_affected > 0)
    }

    /// Auto-check worker loop with alert queue and immediate database purge
    pub async fn start_background_worker(
        self: Arc<Self>,
        alert_queue_tx: Option<tokio::sync::mpsc::Sender<ScheduleItem>>,
    ) {
        println!("🕒 [Scheduler Worker] Started auto-check queue and cleanup loop (PostgreSQL)");

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            let now = Utc::now().timestamp();
            let mut due_items = Vec::new();

            // 1. Fetch due items from PostgreSQL
            if self.ensure_connected().await.is_ok() {
                let guard = self.client.lock().await;
                if let Some(client) = guard.as_ref() {
                    let sql = r#"
                        SELECT id, title, description, schedule_type,
                               target_time, target_timestamp, duration_minutes,
                               recurrence, status, created_at
                        FROM schedules
                        WHERE status = 'pending' AND target_timestamp <= 
                        ORDER BY target_timestamp ASC
                        LIMIT 10
                    "#;

                    if let Ok(rows) = client.query(sql, &[&now]).await {
                        for row in rows {
                            due_items.push(ScheduleItem {
                                id: row.get(0),
                                title: row.get(1),
                                description: row.get(2),
                                schedule_type: row.get(3),
                                target_time: row.get(4),
                                target_timestamp: row.get(5),
                                duration_minutes: row.get(6),
                                recurrence: row.get(7),
                                status: row.get(8),
                                created_at: row.get(9),
                            });
                        }
                    }
                }
            }

            // 2. Process each due reminder: notify AI queue, popup toast, and clear from PostgreSQL
            for item in due_items {
                if let Some(ref queue_tx) = alert_queue_tx {
                    let _ = queue_tx.send(item.clone()).await;
                }

                self.trigger_alert(&item).await;

                // Auto-clear from database to keep PostgreSQL clean
                if self.ensure_connected().await.is_ok() {
                    let guard = self.client.lock().await;
                    if let Some(client) = guard.as_ref() {
                        if item.recurrence == "daily" {
                            let next_ts = item.target_timestamp + 86400;
                            let next_time_str = Local::now().checked_add_signed(Duration::days(1))
                                .map(|d| d.format("%Y-%m-%d %I:%M %p").to_string())
                                .unwrap_or_else(|| item.target_time.clone());
                            let _ = client.execute(
                                "UPDATE schedules SET target_timestamp = , target_time =  WHERE id = ",
                                &[&next_ts, &next_time_str, &item.id],
                            ).await;
                        } else if item.recurrence == "weekly" {
                            let next_ts = item.target_timestamp + (86400 * 7);
                            let next_time_str = Local::now().checked_add_signed(Duration::days(7))
                                .map(|d| d.format("%Y-%m-%d %I:%M %p").to_string())
                                .unwrap_or_else(|| item.target_time.clone());
                            let _ = client.execute(
                                "UPDATE schedules SET target_timestamp = , target_time =  WHERE id = ",
                                &[&next_ts, &next_time_str, &item.id],
                            ).await;
                        } else {
                            let _ = client.execute("DELETE FROM schedules WHERE id = ", &[&item.id]).await;
                            println!("   🧹 [PostgreSQL Auto-Clear] Delivered & removed reminder '{}' from database.", item.title);
                        }
                    }
                }
            }
        }
    }

    /// Trigger native Windows alert popup and console announcement with full description
    async fn trigger_alert(&self, item: &ScheduleItem) {
        println!("
╔══════════════════════════════════════════════════════════════════╗");
        println!("║ 🔔 [VERONICA SCHEDULE ALERT]                                     ║");
        println!("║ Task:        {:<50} ║", item.title);
        println!("║ Type:        {:<50} ║", item.schedule_type.to_uppercase());
        println!("║ Scheduled:   {:<50} ║", item.target_time);
        println!("║ Context:     {:<50} ║", item.description);
        println!("╚══════════════════════════════════════════════════════════════════╝
");

        let title_clean = item.title.replace('"', "'");
        let desc_clean = item.description.replace('"', "'");

        let ps_cmd = format!(
            "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > ;               = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02);               = .GetElementsByTagName('text');              .Item(0).AppendChild(.CreateTextNode('Veronica: {}')) > ;              .Item(1).AppendChild(.CreateTextNode('{}')) > ;               = [Windows.UI.Notifications.ToastNotification]::new();              [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Veronica AI').Show()",
            title_clean, desc_clean
        );

        tokio::process::Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_cmd])
            .spawn()
            .ok();
    }
}

/// Parse human time strings into unix timestamp and formatted string
fn parse_time_expression(expr: &str) -> Result<(i64, String), String> {
    let lower = expr.to_lowercase().trim().to_string();
    let now = Local::now();

    if lower.starts_with("in ") {
        let rest = lower.trim_start_matches("in ").trim();
        let parts: Vec<&str> = rest.split_whitespace().collect();
        if !parts.is_empty() {
            if let Ok(num) = parts[0].parse::<i64>() {
                let unit = parts.get(1).copied().unwrap_or("minutes");
                let duration = if unit.starts_with("sec") {
                    Duration::seconds(num)
                } else if unit.starts_with("min") {
                    Duration::minutes(num)
                } else if unit.starts_with("hour") || unit.starts_with("hr") {
                    Duration::hours(num)
                } else if unit.starts_with("day") {
                    Duration::days(num)
                } else {
                    Duration::minutes(num)
                };

                let target = now + duration;
                return Ok((target.timestamp(), target.format("%Y-%m-%d %I:%M %p").to_string()));
            }
        }
    }

    let target = now + Duration::minutes(10);
    Ok((target.timestamp(), target.format("%Y-%m-%d %I:%M %p").to_string()))
}
