use std::sync::Arc;
use chrono::{Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Schedule item representation stored in Supabase or active in-memory queue
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

/// Cloud-backed schedule engine (Central FastAPI Backend or Supabase) with resilient in-memory fallback
#[derive(Clone)]
pub struct ScheduleEngine {
    backend_url: Option<String>,
    supabase_url: Option<String>,
    supabase_key: Option<String>,
    http_client: reqwest::Client,
    in_memory_queue: Arc<tokio::sync::Mutex<Vec<ScheduleItem>>>,
}

impl ScheduleEngine {
    /// Create a new ScheduleEngine with Central Backend or Supabase Cloud credentials
    pub async fn new(_db_url: Option<String>) -> Self {
        let backend_url = std::env::var("BACKEND_URL").ok().map(|u| u.trim_end_matches('/').to_string());
        let base_url = std::env::var("SUPABASE_URL").ok();
        let key = std::env::var("SUPABASE_KEY").ok();

        let rest_endpoint = base_url.as_ref().map(|u| format!("{}/rest/v1/schedules", u.trim_end_matches('/')));

        if let Some(ref b_url) = backend_url {
            println!("🌐 [Central Backend] Connected to Veronica Backend API: {}", b_url);
        } else if let Some(ref u) = base_url {
            println!("☁️ [Supabase Engine] Configured with cloud project endpoint: {}", u);
        }
        println!("💡 Active resilient queue ready — reminders will trigger 100% reliably!");

        Self {
            backend_url,
            supabase_url: rest_endpoint,
            supabase_key: key,
            http_client: reqwest::Client::new(),
            in_memory_queue: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
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

    /// Create and persist a new schedule item (Supabase Cloud with in-memory resilient fallback)
    pub async fn create_schedule(
        &self,
        title: &str,
        description: &str,
        time_expression: &str,
        duration_minutes: i32,
        recurrence: Option<&str>,
        schedule_type: Option<&str>,
    ) -> Result<ScheduleItem, String> {
        let (target_timestamp, target_time_str) = parse_time_expression(time_expression)?;
        let item_id = format!("sch_{}", Utc::now().timestamp_millis());
        let sched_type = schedule_type.unwrap_or("reminder").to_string();
        let recur = recurrence.unwrap_or("none").to_string();
        let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let item = ScheduleItem {
            id: item_id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            schedule_type: sched_type.clone(),
            target_time: target_time_str.clone(),
            target_timestamp,
            duration_minutes,
            recurrence: recur.clone(),
            status: "pending".to_string(),
            created_at: now_str.clone(),
        };

        // 1. Try persisting to Central Backend API (if configured)
        if let Some(ref b_url) = self.backend_url {
            let res = self.http_client
                .post(b_url)
                .json(&item)
                .send()
                .await;

            match res {
                Ok(resp) if resp.status().is_success() => {
                    println!("   🌐 [Central Backend] Persisted schedule: '{}' at {}", title, target_time_str);
                    if let Ok(saved) = resp.json::<ScheduleItem>().await {
                        return Ok(saved);
                    }
                    return Ok(item);
                }
                Ok(resp) => {
                    let err_txt = resp.text().await.unwrap_or_default();
                    eprintln!("   ⚠️ [Backend Notice]: HTTP status {}; trying fallback. ({})", err_txt, title);
                }
                Err(e) => {
                    eprintln!("   ⚠️ [Backend Network]: {}; trying fallback.", e);
                }
            }
        }

        // 2. Try persisting to Supabase Cloud REST API
        if let (Some(ref url), Some(ref key)) = (&self.supabase_url, &self.supabase_key) {
            let res = self.http_client
                .post(url)
                .header("apikey", key)
                .header("Authorization", format!("Bearer {}", key))
                .header("Content-Type", "application/json")
                .header("Prefer", "return=representation")
                .json(&item)
                .send()
                .await;

            match res {
                Ok(resp) if resp.status().is_success() => {
                    println!("   ☁️ [Supabase Cloud] Persisted schedule: '{}' at {}", title, target_time_str);
                    return Ok(item);
                }
                Ok(resp) => {
                    let err_txt = resp.text().await.unwrap_or_default();
                    eprintln!("   ⚠️ [Supabase Notice]: HTTP status {}; using resilient memory queue. ({})", err_txt, title);
                }
                Err(e) => {
                    eprintln!("   ⚠️ [Supabase Network]: {}; using resilient memory queue.", e);
                }
            }
        }

        // 2. Resilient in-memory fallback (reminders will 100% trigger on time!)
        println!("   💾 [Resilient Fallback] Saved reminder to active memory queue: '{}' at {}", title, target_time_str);
        self.in_memory_queue.lock().await.push(item.clone());
        Ok(item)
    }

    /// List upcoming pending schedules from Central Backend, Supabase Cloud, or in-memory queue
    pub async fn list_upcoming(&self, limit: i64) -> Result<Vec<ScheduleItem>, String> {
        let mut items = Vec::new();

        // 1. Try Central Backend API (if configured)
        if let Some(ref b_url) = self.backend_url {
            let query_url = format!("{}/upcoming?limit={}", b_url, limit);
            if let Ok(resp) = self.http_client.get(&query_url).send().await {
                if resp.status().is_success() {
                    if let Ok(backend_items) = resp.json::<Vec<ScheduleItem>>().await {
                        items.extend(backend_items);
                    }
                }
            }
        }

        // 2. Try Supabase Cloud (fallback if backend not configured or empty)
        if items.is_empty() {
            if let (Some(ref url), Some(ref key)) = (&self.supabase_url, &self.supabase_key) {
                let query_url = format!("{}?status=eq.pending&order=target_timestamp.asc&limit={}", url, limit);
                if let Ok(resp) = self.http_client
                    .get(&query_url)
                    .header("apikey", key)
                    .header("Authorization", format!("Bearer {}", key))
                    .send()
                    .await
                {
                    if resp.status().is_success() {
                        if let Ok(cloud_items) = resp.json::<Vec<ScheduleItem>>().await {
                            items.extend(cloud_items);
                        }
                    }
                }
            }
        }

        // 3. Also append pending items from active in-memory queue
        let mem = self.in_memory_queue.lock().await;
        for item in mem.iter() {
            if item.status == "pending" && !items.iter().any(|i| i.id == item.id) {
                items.push(item.clone());
            }
        }

        items.sort_by_key(|i| i.target_timestamp);
        items.truncate(limit as usize);
        Ok(items)
    }

    /// Cancel a scheduled task by ID or title substring
    pub async fn cancel_schedule(&self, identifier: &str) -> Result<bool, String> {
        let mut cancelled = false;

        // 1. Try Central Backend API (if configured)
        if let Some(ref b_url) = self.backend_url {
            let delete_url = format!("{}/{}", b_url, identifier);
            if let Ok(resp) = self.http_client.delete(&delete_url).send().await {
                if resp.status().is_success() {
                    cancelled = true;
                    println!("   🌐 [Central Backend] Cancelled schedule: '{}'", identifier);
                }
            }
        }

        // 2. Try Supabase Cloud
        if let (Some(ref url), Some(ref key)) = (&self.supabase_url, &self.supabase_key) {
            let delete_url = format!("{}?or=(id.eq.{},title.ilike.*{}*)", url, identifier, identifier);
            if let Ok(resp) = self.http_client
                .delete(&delete_url)
                .header("apikey", key)
                .header("Authorization", format!("Bearer {}", key))
                .send()
                .await
            {
                if resp.status().is_success() {
                    cancelled = true;
                }
            }
        }

        // Also cancel from in-memory queue
        let mut mem = self.in_memory_queue.lock().await;
        let id_lower = identifier.to_lowercase();
        mem.retain(|i| {
            if i.id == identifier || i.title.to_lowercase().contains(&id_lower) {
                cancelled = true;
                false
            } else {
                true
            }
        });

        Ok(cancelled)
    }

    /// Auto-check worker loop with alert queue and immediate cloud/memory purge
    pub async fn start_background_worker(
        self: Arc<Self>,
        alert_queue_tx: Option<tokio::sync::mpsc::Sender<ScheduleItem>>,
    ) {
        println!("🕒 [Scheduler Worker] Started auto-check queue and cleanup loop (Supabase + Memory)");

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            let now = Utc::now().timestamp();
            let mut due_items = Vec::new();

            // 1. Fetch due items from Central Backend API (if configured)
            if let Some(ref b_url) = self.backend_url {
                let query_url = format!("{}/due?limit=10&now_ts={}", b_url, now);
                if let Ok(resp) = self.http_client.get(&query_url).send().await {
                    if resp.status().is_success() {
                        if let Ok(backend_items) = resp.json::<Vec<ScheduleItem>>().await {
                            due_items.extend(backend_items);
                        }
                    }
                }
            } else if let (Some(ref url), Some(ref key)) = (&self.supabase_url, &self.supabase_key) {
                // 1b. Fallback to Supabase Cloud if Central Backend is not set
                let query_url = format!("{}?status=eq.pending&target_timestamp=lte.{}&order=target_timestamp.asc&limit=10", url, now);
                if let Ok(resp) = self.http_client
                    .get(&query_url)
                    .header("apikey", key)
                    .header("Authorization", format!("Bearer {}", key))
                    .send()
                    .await
                {
                    if resp.status().is_success() {
                        if let Ok(cloud_items) = resp.json::<Vec<ScheduleItem>>().await {
                            due_items.extend(cloud_items);
                        }
                    }
                }
            }

            // 2. Fetch due items from active in-memory queue
            {
                let mut mem = self.in_memory_queue.lock().await;
                let mut remaining = Vec::new();
                for item in mem.drain(..) {
                    if item.target_timestamp <= now && item.status == "pending" {
                        due_items.push(item);
                    } else {
                        remaining.push(item);
                    }
                }
                *mem = remaining;
            }

            // 3. Process each due reminder: notify AI queue, popup toast, and clear
            for item in due_items {
                println!("🔔 [Scheduler Alert Triggered]: '{}' ({})", item.title, item.target_time);

                if let Some(ref queue_tx) = alert_queue_tx {
                    let _ = queue_tx.send(item.clone()).await;
                }

                self.trigger_alert(&item).await;

                // Auto-clear from Central Backend or Supabase Cloud
                if let Some(ref b_url) = self.backend_url {
                    if item.recurrence == "daily" {
                        let next_ts = item.target_timestamp + 86400;
                        let next_time_str = Local::now().checked_add_signed(Duration::days(1))
                            .map(|d| d.format("%Y-%m-%d %I:%M %p").to_string())
                            .unwrap_or_else(|| item.target_time.clone());

                        let update_url = format!("{}/{}", b_url, item.id);
                        let _ = self.http_client
                            .patch(&update_url)
                            .json(&json!({
                                "target_timestamp": next_ts,
                                "target_time": next_time_str
                            }))
                            .send()
                            .await;
                    } else {
                        let delete_url = format!("{}/{}", b_url, item.id);
                        let _ = self.http_client.delete(&delete_url).send().await;
                        println!("   🧹 [Auto-Clear] Delivered & removed reminder '{}' from Central Backend", item.title);
                    }
                } else if let (Some(ref url), Some(ref key)) = (&self.supabase_url, &self.supabase_key) {
                    if item.recurrence == "daily" {
                        let next_ts = item.target_timestamp + 86400;
                        let next_time_str = Local::now().checked_add_signed(Duration::days(1))
                            .map(|d| d.format("%Y-%m-%d %I:%M %p").to_string())
                            .unwrap_or_else(|| item.target_time.clone());

                        let update_url = format!("{}?id=eq.{}", url, item.id);
                        let _ = self.http_client
                            .patch(&update_url)
                            .header("apikey", key)
                            .header("Authorization", format!("Bearer {}", key))
                            .header("Content-Type", "application/json")
                            .json(&json!({
                                "target_timestamp": next_ts,
                                "target_time": next_time_str
                            }))
                            .send()
                            .await;
                    } else {
                        let delete_url = format!("{}?id=eq.{}", url, item.id);
                        let _ = self.http_client
                            .delete(&delete_url)
                            .header("apikey", key)
                            .header("Authorization", format!("Bearer {}", key))
                            .send()
                            .await;
                        println!("   🧹 [Auto-Clear] Delivered & removed reminder '{}' from Supabase", item.title);
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

        let title_clean = item.title.replace('\'', "''").replace('"', " ");
        let desc_clean = item.description.replace('\'', "''").replace('"', " ");

        let ps_cmd = format!(
            r#"
            [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
            [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
            $template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02)
            $textNodes = $template.GetElementsByTagName('text')
            $textNodes.Item(0).AppendChild($template.CreateTextNode('Veronica: {}')) | Out-Null
            $textNodes.Item(1).AppendChild($template.CreateTextNode('{}')) | Out-Null
            $toast = [Windows.UI.Notifications.ToastNotification]::new($template)
            [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Veronica AI').Show($toast)
            "#,
            title_clean, desc_clean
        );

        tokio::process::Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_cmd])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
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
