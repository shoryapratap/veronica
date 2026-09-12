# ⚡ Project Veronica

> **An autonomous, decoupled 3-tier personal AI assistant engineered for local system autonomy, bare-metal efficiency, and low-latency interaction.**

Unlike conventional monolithic assistants, Project Veronica physically and logically decouples user interface lifecycles from local operating system automation and cloud intelligence:

* 🧠 **Tier 1: Central Backend (`backend/`)**: Asynchronous Python + FastAPI server managing heavy LLM routing, long-term memory, and PostgreSQL database schema migrations.
* ⚙️ **Tier 2: Headless Daemon (`daemon/`)**: High-performance Rust background engine running on Tokio async event loops (<15MB idle RAM). Manages local hardware (audio/mic), OS automation, and exposes an authenticated loopback WebSocket server on `127.0.0.1:8765`.
* 🖥️ **Tier 3: Desktop Client (`gui/veronica/`)**: A sandboxed desktop HUD built with Tauri v2, React 19, TypeScript, and Tailwind CSS v4. Compiles native Windows `.exe` binaries in under 0.4 seconds with zero local OS permissions.

### 📐 Architecture & Standards
* **Decoupled Security**: The presentation layer has zero direct shell/filesystem access.
* **Continuous Autonomy**: The daemon operates 24/7 in the background regardless of whether the desktop GUI is open.
* **Shared Protocol**: 1:1 type parity between Rust Serde enums and TypeScript union types.
* **Specifications**: Formal Architecture Decision Record available in [`documnet/tech_stack_specification.md`](./documnet/tech_stack_specification.md).
