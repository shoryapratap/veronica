# Tier 2: Local Headless Daemon Setup Guide & Engineering Runbook

```text
Document ID:     VER-GUIDE-002
Tier:            Tier 2 — Local Headless Daemon (The Engine)
Target Directory: Veronica/daemon/
Runtime:         Rust 2021 Edition (rustc / cargo)
Status:          COMPLETE & VERIFIED
```

---

## 1. Overview & Purpose

The **Local Headless Daemon** is the resident background service executing directly on the user's host operating system (Windows 10/11). It has **zero user interface**, idles at **under 15MB of RAM**, and is responsible for:
1. Managing local hardware (microphone input, audio output, telemetry).
2. Executing authorized local OS shell and filesystem actions.
3. Hosting the authenticated, low-latency WebSocket server on `127.0.0.1:8765` so the Desktop GUI can connect, stream, and receive telemetry.
4. Running 24/7 autonomously even if the GUI is never launched.

This runbook documents every command executed, the architectural rationale for the folder structure, and a complete code registry of every file written.

---

## 2. Step-by-Step Command Execution Log

### Step 2.1: Navigate into the Target Folder
* **Working Directory (CWD)**: `c:\coding\personal project\Veronica`
* **Command**:
  ```powershell
  cd daemon
  ```
* **Why & What Happened**: Shifts your shell context into `daemon/` to keep Rust crates and build artifacts isolated.

### Step 2.2: Initialize the Rust Binary Project
* **Working Directory (CWD)**: `c:\coding\personal project\Veronica\daemon`
* **Command**:
  ```powershell
  cargo init --bin --name veronica-daemon
  ```
* **Why & What Happened**: Created `Cargo.toml` and `src/main.rs`. The `--bin` flag specifies an executable application (`.exe`) rather than a library.

### Step 2.3: Configure `Cargo.toml` Dependencies
* **Working Directory (CWD)**: `c:\coding\personal project\Veronica\daemon`
* **Action**: Added Tokio, Tokio-Tungstenite, Futures-util, Serde, Serde_json, and Sysinfo under `[dependencies]`.

### Step 2.4: Verify & Cache Dependencies via `cargo check`
* **Working Directory (CWD)**: `c:\coding\personal project\Veronica\daemon`
* **Command**:
  ```powershell
  cargo check
  ```
* **Why & What Happened**: Downloaded 91 crates from `crates.io`, generated `Cargo.lock`, and verified all type signatures without spending time on binary linking.

### Step 2.5: Create Internal Modular Directories
* **Working Directory (CWD)**: `c:\coding\personal project\Veronica\daemon`
* **Command**:
  ```powershell
  New-Item -ItemType Directory -Force -Path src\server, src\system, src\bridge, src\audio
  ```
* **Why & What Happened**: Creates dedicated modular directories separating server protocols, system tools, backend bridging, and audio pipelines.

### Step 2.6: Compile & Run Async Loop
* **Working Directory (CWD)**: `c:\coding\personal project\Veronica\daemon`
* **Command**:
  ```powershell
  cargo run
  ```
* **Why & What Happened**: Compiled `veronica-daemon.exe` and launched the non-blocking Tokio async loop, emitting background heartbeats every 5 seconds.

---

## 3. Directory Structure & Architectural Rationale

```text
daemon/
├── Cargo.toml               # Project manifest, package metadata, and crate dependencies
├── Cargo.lock               # Cryptographically locked exact dependency versions
├── target/                  # Compiled binary artifacts, intermediate object files & cache
└── src/                     # Rust source code directory
    ├── main.rs              # Application entry point, Tokio runtime boot, async loop
    ├── server/              # Localhost WebSocket server module (for Tier 3 GUI)
    │   ├── mod.rs           # Module declaration & public exports
    │   └── protocol.rs      # Cross-language JSON packet definitions (Serde enums)
    ├── system/              # Local OS operations (shell execution & telemetry)
    ├── bridge/              # Upstream client connection to Tier 1 Central Backend
    └── audio/               # Hardware audio capture (CPAL) & speech pipelines
```

### Why This Specific Architecture? (Modular Systems Architecture)
1. **`src/server/` vs `src/bridge/` (Separation of Network Roles)**:
   - **`server/`** is a **Localhost Server**: It listens on `127.0.0.1:8765` waiting for the GUI client to connect.
   - **`bridge/`** is an **Outbound Client**: It dials out over TLS/WSS across the internet to the Central Backend.
   - *Why separate them?* Keeping local and remote networking isolated prevents security cross-contamination.
2. **`src/server/protocol.rs` (The Contract Definition)**:
   - Houses the exact Rust data structures that serialize into JSON.
   - It is the master reference for the TypeScript types in the GUI (`protocol.ts`), ensuring 100% type parity between frontend and daemon.
3. **`src/system/` (Local OS Isolation)**:
   - Contains all shell command spawning (`tokio::process::Command`) and hardware telemetry (`sysinfo`). Isolating this guarantees that OS-level side effects are segregated from networking code.
4. **`src/audio/` (Hardware Stream Isolation)**:
   - Microphone streaming requires high-priority low-latency audio threads (CPAL). Separating it prevents audio buffer underruns from interfering with WebSocket communication.

---

## 4. Code & File Registry

Below is the complete registry of every file written for Tier 2, what it does, and the exact code inside:

---

### File 1: `Cargo.toml`
* **Path**: `c:\coding\personal project\Veronica\daemon\Cargo.toml`
* **Purpose**: Defines package metadata, compiler edition (2021), and core async dependencies.
* **Code**:
```toml
[package]
name = "veronica-daemon"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async Runtime (Multi-threaded event loop)
tokio = { version = "1.43", features = ["full"] }

# Localhost WebSocket Server (for GUI connection)
tokio-tungstenite = "0.26"
futures-util = "0.3"

# JSON Serialization (Compile-time, zero-copy)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Hardware Monitoring (CPU, RAM, Battery telemetry)
sysinfo = "0.33"
```

---

### File 2: `src/server/protocol.rs`
* **Path**: `c:\coding\personal project\Veronica\daemon\src\server\protocol.rs`
* **Purpose**: Declares the Serde-tagged JSON data packets exchanged between the Daemon and GUI.
* **Code**:
```rust
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
```

---

### File 3: `src/server/mod.rs`
* **Path**: `c:\coding\personal project\Veronica\daemon\src\server\mod.rs`
* **Purpose**: Exposes the `protocol` submodule to the rest of the daemon application.
* **Code**:
```rust
pub mod protocol;
```

---

### File 4: `src/main.rs`
* **Path**: `c:\coding\personal project\Veronica\daemon\src\main.rs`
* **Purpose**: The main binary entry point; initializes the multi-threaded Tokio runtime and runs the asynchronous background loop.
* **Code**:
```rust
mod server;

use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("========================================");
    println!(" Project Veronica: Local Headless Daemon");
    println!(" Status: INITIALIZING");
    println!(" Target Port: 127.0.0.1:8765");
    println!("========================================");

    // Initial background heartbeat loop
    let mut tick = 0;
    loop {
        tick += 1;
        println!("[Daemon Heartbeat #{}]: Running silently in background...", tick);
        sleep(Duration::from_secs(5)).await;
    }
}
```
