use std::sync::Arc;
use chrono::{Duration, Local, Utc};
use serde::{Deserialize, Serialize};
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

/// PostgreSQL-backed schedule and calendar engine
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

        // Attempt initial connection
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
            UPDATE schedules
            SET status = 'cancelled'
            WHERE (id =  OR LOWER(title) LIKE LOWER()) AND status = 'pending'
        "#;

        let search_pattern = format!("%{}%", identifier);
        let rows_affected = client.execute(sql, &[&identifier, &search_pattern]).await
            .map_err(|e| format!("Failed to update schedule in PostgreSQL: {}", e))?;

        Ok(rows_affected > 0)
    }

    /// Check for conflicting meetings in the given time frame
    pub async fn check_conflicts(&self, start_ts: i64, duration_mins: i32) -> Result<Option<String>, String> {
        self.ensure_connected().await?;
        let guard = self.client.lock().await;
        let client = guard.as_ref().ok_or("No active PostgreSQL connection")?;

        let end_ts = start_ts + (duration_mins as i64 * 60);

        let sql = r#"
            SELECT title, target_time
            FROM schedules
            WHERE status = 'pending'
              AND target_timestamp < 
              AND (target_timestamp + (duration_minutes * 60)) > 
            LIMIT 1
        "#;

        let rows = client.query(sql, &[&end_ts, &start_ts]).await
            .map_err(|e| format!("Failed to check conflicts: {}", e))?;

        if let Some(row) = rows.first() {
            let conflict_title: String = row.get(0);
            let conflict_time: String = row.get(1);
            Ok(Some(format!("'{}' at {}", conflict_title, conflict_time)))
        } else {
            Ok(None)
        }
    }

    /// Run 24/7 background scheduler worker to alert user on due tasks
    pub async fn start_background_worker(self: Arc<Self>) {
        println!("🕒 [Scheduler Worker] Started 24/7 background alert loop for PostgreSQL");

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            let now = Utc::now().timestamp();
            let mut due_items = Vec::new();

            // Fetch due items
            if self.ensure_connected().await.is_ok() {
                let guard = self.client.lock().await;
                if let Some(client) = guard.as_ref() {
                    let sql = r#"
                        SELECT id, title, description, schedule_type,
                               target_time, target_timestamp, duration_minutes,
                               recurrence, status, created_at
                        FROM schedules
                        WHERE status = 'pending' AND target_timestamp <= 
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

            // Trigger alerts for each due task
            for item in due_items {
                self.trigger_alert(&item).await;

                // Mark as triggered in PostgreSQL
                if let Ok(_) = self.ensure_connected().await {
                    let guard = self.client.lock().await;
                    if let Some(client) = guard.as_ref() {
                        let _ = client.execute(
                            "UPDATE schedules SET status = 'triggered' WHERE id = ",
                            &[&item.id],
                        ).await;
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

        // Native Windows PowerShell Toast Notification
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

    // Check relative minute offset: "in X min" / "in X minutes"
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

    // Default: in 10 minutes if parsing is ambiguous
    let target = now + Duration::minutes(10);
    Ok((target.timestamp(), target.format("%Y-%m-%d %I:%M %p").to_string()))
}
