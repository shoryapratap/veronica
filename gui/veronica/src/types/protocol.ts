// Incoming message packets from Rust Daemon to React GUI
export type DaemonMessage =
  | { type: "HEARTBEAT"; payload: { timestamp: number; status: string } }
  | { type: "TELEMETRY"; payload: { cpu_usage: number; ram_used_mb: number; ram_total_mb: number } }
  | { type: "LOG"; payload: { level: string; message: string } };

// Outgoing message packets from React GUI to Rust Daemon
export type ClientMessage =
  | { type: "PING" }
  | { type: "COMMAND"; payload: { action: string } };

export type ConnectionStatus = "CONNECTING" | "ONLINE" | "DISCONNECTED";
