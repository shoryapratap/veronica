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

## Cross-Platform Engineering Standards (Universal OS Compatibility)
- The Daemon (`daemon/`), Backend (`backend/`), and GUI (`gui/`) must be engineered to run across **Windows, macOS, Linux (Ubuntu, Kali, Parrot, etc.), Android, iOS, and Robot OS (ROS/ROS 2)**.
- **Dynamic Device Detection & Modular Platform Adapters**:
  - The base Daemon installs as a universal, lightweight core engine.
  - Upon initial boot or deployment, the engine analyzes the host environment (detecting kernel, window manager, display server, package manager, and whether ROS / Android environment is present).
  - The daemon binds or loads the corresponding OS Platform Adapter module dynamically (or compiles the matching platform driver) to control application launching, input simulation, and process management.
- Any OS-specific execution (process launching, file systems, audio capture, notifications, input events) must strictly reside within modular platform adapters (`system/platform/`) behind a unified, universal public interface. No OS-specific code is allowed in the core server, bridge, or audio services.

## Visual Aesthetics & Design Language (GUI Tier)
- **Core Theme**: High-end futuristic luxury — Black, White, Silver Metallic, and Frosted Glass UI.
- **Palette**:
  - **Obsidian / Pure Dark**: Deep blacks (`#000000`, `#09090b`, `#121217`) for infinite depth and contrast.
  - **Silver & Chrome**: Metallic gradients, brushed steel highlights, and silver/zinc typography (`#FFFFFF`, `#E4E4E7`, `#A1A1AA`).
  - **Frosted Glassmorphism**: Translucent panels (`backdrop-blur-xl`, `bg-zinc-950/70`, delicate `border-white/10` or `border-zinc-800/60`, specular edge highlights).
- **Constraint**: Strictly avoid toy-like, saturated rainbow colors (bright reds, neon greens, blues, yellows). All badges, status pills, orbs, and controls must strictly adhere to monochrome, silver metallic, and glass aesthetic.

