# Project Veronica - Architecture & Development Charter

## System Identity
- **Name**: Project Veronica
- **Purpose**: Autonomous, 3-tier decoupled personal AI assistant.

## Non-Negotiable Architecture Rules
1. **Tier 1: Central Backend (`backend/`)**
   - Cloud/remote server for heavy AI orchestration, long-term memory, and LLM APIs.
   - Must never assume a local GUI or local audio hardware exists on its machine.

2. **Tier 2: Local Headless Engine (`daemon/`)**
   - Silent, headless background service running directly on the user's OS.
   - Zero UI. Manages local hardware (mic/speaker), OS commands, and file operations.
   - Exposes a secure localhost WebSocket server on `127.0.0.1`.
   - Must run 24/7 independently without requiring the GUI to be active.

3. **Tier 3: The Face (`gui/`)**
   - Presentation-only desktop client built with Tauri + React.
   - Zero direct OS/hardware access. Communicates exclusively with the Daemon via localhost WebSocket.
   - Can be opened, closed, or restarted at any time without interrupting the Daemon.

## Code Standards
- Every tier must maintain isolated dependencies and its own configuration.
- No direct coupling between `gui/` and `backend/`. All traffic routes through the `daemon/`.
