use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum AssistantState {
    #[serde(rename = "IDLE")]
    Idle,
    #[serde(rename = "LISTENING")]
    Listening,
    #[serde(rename = "SPEAKING")]
    Speaking,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum DaemonMessage {
    #[serde(rename = "HEARTBEAT")]
    Heartbeat { timestamp: u64, status: String },

    #[serde(rename = "TELEMETRY")]
    Telemetry { cpu_usage: f32, ram_used_mb: u64, ram_total_mb: u64 },

    #[serde(rename = "STATE_CHANGE")]
    StateChange { state: AssistantState },

    #[serde(rename = "TRANSCRIPT")]
    Transcript {
        speaker: String, // "USER" or "VERONICA"
        text: String,
        is_final: bool,
    },

    #[serde(rename = "LOG")]
    Log { level: String, message: String },
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    #[serde(rename = "PING")]
    Ping,

    #[serde(rename = "TEXT_MESSAGE")]
    TextMessage { text: String },

    #[serde(rename = "COMMAND")]
    Command { action: String },
}
