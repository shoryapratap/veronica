use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum DaemonMessage {
    #[serde(rename = "HEARTBEAT")]
    Heartbeat { timestamp: u64, status: String },

    #[serde(rename = "TELEMETRY")]
    Telemetry { cpu_usage: f32, ram_used_mb: u64, ram_total_mb: u64 },

    #[serde(rename = "LOG")]
    Log { level: String, message: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    #[serde(rename = "PING")]
    Ping,

    #[serde(rename = "COMMAND")]
    Command { action: String },
}
