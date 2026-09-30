use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleItem {
    pub id: String,
    pub title: String,
    pub description: String, // Context of what was asked or needs to be remembered
    pub schedule_type: String, // 'reminder', 'meeting', 'habit', 'timer'
    pub target_time: String, // Human readable ISO format
    pub target_timestamp: i64, // Unix epoch seconds
    pub duration_minutes: i64,
    pub recurrence: Option<String>, // 'none', 'daily', 'weekly'
    pub status: String, // 'pending', 'triggered', 'completed', 'cancelled'
    pub created_at: String,
}

#[derive(Clone)]
pub struct ScheduleDb {
    conn: Arc<Mutex<Connection>>,
}

impl ScheduleDb {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, String> {
        if let Some(parent) = db_path.as_ref().parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open SQLite database: {}", e))?;

        // Initialize schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schedules (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                schedule_type TEXT NOT NULL,
                target_time TEXT NOT NULL,
                target_timestamp INTEGER NOT NULL,
                duration_minutes INTEGER DEFAULT 0,
                recurrence TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL
            );",
            [],
        )
        .map_err(|e| format!("Failed to create schedules table: {}", e))?;

        let _ = conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_schedules_time_status ON schedules (target_timestamp, status);",
            [],
        );

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Add a new schedule item to SQLite database
    pub fn add(&self, item: &ScheduleItem) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO schedules (
                id, title, description, schedule_type, target_time, 
                target_timestamp, duration_minutes, recurrence, status, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);",
            params![
                item.id,
                item.title,
                item.description,
                item.schedule_type,
                item.target_time,
                item.target_timestamp,
                item.duration_minutes,
                item.recurrence,
                item.status,
                item.created_at
            ],
        )
        .map_err(|e| format!("Failed to insert schedule: {}", e))?;
        Ok(())
    }

    /// List upcoming pending items sorted by target time
    pub fn list_upcoming(&self, limit: usize) -> Result<Vec<ScheduleItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = Utc::now().timestamp();
        let mut stmt = conn
            .prepare(
                "SELECT id, title, description, schedule_type, target_time, 
                        target_timestamp, duration_minutes, recurrence, status, created_at
                 FROM schedules 
                 WHERE status = 'pending' AND target_timestamp >= ?1 
                 ORDER BY target_timestamp ASC LIMIT ?2;",
            )
            .map_err(|e| e.to_string())?;

        let items = stmt
            .query_map(params![now - 60, limit as i64], |row| {
                Ok(ScheduleItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    schedule_type: row.get(3)?,
                    target_time: row.get(4)?,
                    target_timestamp: row.get(5)?,
                    duration_minutes: row.get(6)?,
                    recurrence: row.get(7)?,
                    status: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        Ok(items)
    }

    /// Get all schedules that are due (target_timestamp <= now) and still pending
    pub fn get_due_schedules(&self) -> Result<Vec<ScheduleItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = Utc::now().timestamp();
        let mut stmt = conn
            .prepare(
                "SELECT id, title, description, schedule_type, target_time, 
                        target_timestamp, duration_minutes, recurrence, status, created_at
                 FROM schedules 
                 WHERE status = 'pending' AND target_timestamp <= ?1 
                 ORDER BY target_timestamp ASC;",
            )
            .map_err(|e| e.to_string())?;

        let items = stmt
            .query_map(params![now], |row| {
                Ok(ScheduleItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    schedule_type: row.get(3)?,
                    target_time: row.get(4)?,
                    target_timestamp: row.get(5)?,
                    duration_minutes: row.get(6)?,
                    recurrence: row.get(7)?,
                    status: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        Ok(items)
    }

    /// Mark a schedule as triggered or advance if recurring
    pub fn mark_triggered(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        
        let mut stmt = conn.prepare("SELECT recurrence, target_timestamp FROM schedules WHERE id = ?1;").map_err(|e| e.to_string())?;
        let rec: Option<(Option<String>, i64)> = stmt.query_row(params![id], |row| Ok((row.get(0)?, row.get(1)?))).ok();

        if let Some((Some(ref r), ts)) = rec {
            let next_ts = match r.to_lowercase().as_str() {
                "daily" => Some(ts + 86400),
                "weekly" => Some(ts + 604800),
                _ => None,
            };
            if let Some(nts) = next_ts {
                conn.execute(
                    "UPDATE schedules SET target_timestamp = ?1 WHERE id = ?2;",
                    params![nts, id],
                ).map_err(|e| e.to_string())?;
                return Ok(());
            }
        }

        conn.execute(
            "UPDATE schedules SET status = 'triggered' WHERE id = ?1;",
            params![id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Cancel a schedule by ID or matching title
    pub fn cancel(&self, query: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let count = conn.execute(
            "UPDATE schedules SET status = 'cancelled' 
             WHERE status = 'pending' AND (id = ?1 OR LOWER(title) LIKE LOWER(?2));",
            params![query, format!("%{}%", query)],
        )
        .map_err(|e| e.to_string())?;
        Ok(count)
    }

    /// Check if any existing meeting overlaps with the requested interval
    pub fn check_conflicts(&self, start_ts: i64, duration_mins: i64) -> Result<Vec<ScheduleItem>, String> {
        let end_ts = start_ts + (duration_mins * 60);
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, title, description, schedule_type, target_time, 
                        target_timestamp, duration_minutes, recurrence, status, created_at
                 FROM schedules 
                 WHERE status = 'pending' AND schedule_type = 'meeting'
                   AND ((target_timestamp >= ?1 AND target_timestamp < ?2)
                        OR (target_timestamp + (duration_minutes * 60) > ?1 AND target_timestamp <= ?1));",
            )
            .map_err(|e| e.to_string())?;

        let items = stmt
            .query_map(params![start_ts, end_ts], |row| {
                Ok(ScheduleItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    schedule_type: row.get(3)?,
                    target_time: row.get(4)?,
                    target_timestamp: row.get(5)?,
                    duration_minutes: row.get(6)?,
                    recurrence: row.get(7)?,
                    status: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        Ok(items)
    }
}
