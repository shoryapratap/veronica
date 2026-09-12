# Project Veronica: Technology Stack & Architectural Specification

```text
Document ID:     VER-ADR-0001
Revision:        1.0.0
Status:          APPROVED
Classification:  System Architecture Specification / ADR (Architecture Decision Record)
System Scope:    Decoupled 3-Tier Autonomous AI Assistant
Target Platform: Windows 10/11 (Local Host) & Cloud / Container (Remote Core)
```

---

## 1. Executive Summary

**Project Veronica** is an autonomous, multi-tier personal artificial intelligence assistant built on a **strictly decoupled 3-tier topology**. Unlike conventional monolithic or 2-tier client-server assistants, Project Veronica physically and logically isolates:

1. **Cognitive Cloud Infrastructure (The Brain)** from local system access.
2. **Local Operating System Automation & Hardware I/O (The Engine)** from user interface lifecycles.
3. **Desktop Presentation & Interaction (The Face)** from system-level privileges.

This document establishes the official technology stack, architectural standards, process lifecycle models, and communication protocols across all three tiers.

---

## 2. Architectural Principles & System Topology

### 2.1 Core Architectural Tenets
* **Strict Decoupling**: No tier shares memory, internal state, or direct process bindings with another tier.
* **Persistent Daemon Autonomy**: Tier 2 (Local Daemon) operates 24/7 as an unprivileged/user-elevated background service. It continues executing scheduled tasks, file monitoring, and wake-word detection regardless of whether the Desktop GUI is open or terminated.
* **Zero-Privilege UI Sandboxing**: Tier 3 (Desktop GUI) is strictly a presentation terminal. It contains zero filesystem, shell, or raw socket permissions. It communicates exclusively with Tier 2 over an authenticated `127.0.0.1` loopback WebSocket.
* **Modular Replaceability**: Any individual tier may be refactored, migrated, or completely re-written without altering the codebase of the remaining two tiers, provided the wire protocol contract is maintained.

### 2.2 System Topology Diagram

```mermaid
flowchart TD
    subgraph Tier1 ["Tier 1: Central Backend (Remote / Cloud)"]
        A1["FastAPI Application Server"]
        A2["PostgreSQL + pgvector"]
        A3["SQLAlchemy 2.0 (Async) + Alembic"]
        A4["AI Model Gateway (Gemini / OpenAI / Custom)"]
        A1 <--> A2
        A1 <--> A4
    end

    subgraph Tier2 ["Tier 2: Local Headless Daemon (Localhost OS Service)"]
        B1["Rust Tokio Async Core"]
        B2["Localhost WS Server (127.0.0.1:8765)"]
        B3["CPAL Audio Pipeline (Mic / Speaker)"]
        B4["OS Automation & Sysinfo Engine"]
        B1 <--> B2
        B1 <--> B3
        B1 <--> B4
    end

    subgraph Tier3 ["Tier 3: Desktop GUI Client (The Face)"]
        C1["Tauri v2 Native Shell (WebView2)"]
        C2["React 18/19 + TypeScript + Vite"]
        C3["Tailwind CSS + Framer Motion HUD"]
        C4["Native WebSocket Client"]
        C1 --- C2
        C2 <--> C4
    end

    Tier1 <===="TLS / WSS Duplex Uplink (Internet)"====> Tier2
    Tier2 <===="Localhost WS (127.0.0.1 - Token Handshake)"====> Tier3
```

---

## 3. Tier 1: Central Backend Specification (The Brain)

### 3.1 Overview
The Central Backend manages compute-heavy cognitive tasks, long-term state, LLM routing, user authentication, and persistent storage.

### 3.2 Technology Matrix

| Component | Technology | Specification / Version | Architectural Justification |
| :--- | :--- | :--- | :--- |
| **Language** | Python | `>= 3.11` | Industry-standard language for modern AI, Hugging Face, PyTorch, and LLM orchestration. |
| **API Framework** | FastAPI | `Latest (0.115+)` | High-performance asynchronous routing, native Pydantic integration, auto-generated OpenAPI (`/docs`). |
| **ASGI Web Server** | Uvicorn | `[standard] worker` | High-throughput asynchronous event loop (`uvloop`) for sub-millisecond route handling. |
| **Data Validation** | Pydantic v2 | `Latest` | High-speed Rust-backed data validation and serialization; enforces strict request/response contracts. |
| **Config Management** | Pydantic Settings | `pydantic-settings` | Strongly typed `.env` file parsing with fail-fast boot validation. |
| **Primary Database** | PostgreSQL | `>= 15.0` | Enterprise ACID-compliant relational storage; supports `pgvector` for semantic AI embeddings. |
| **ORM / Query Engine** | SQLAlchemy | `2.0+ (Async)` | Async database mapping; prevents SQL injection and provides type-safe query generation. |
| **Schema Migrations** | Alembic | `Latest` | Version-controlled database schema migrations (equivalent to Prisma Migrate). |
| **Authentication** | PyJWT + Passlib | `Argon2 / Bcrypt` | Stateless JWT access/refresh token lifecycle and industry-grade password hashing. |
| **HTTP Client** | HTTPX | `Latest (Async)` | Non-blocking async client for upstream AI model APIs (Google Gemini, OpenAI, Grok). |

### 3.3 Design Patterns & Rejected Alternatives
* **Rejected Alternative (Node.js / Express)**: Lacks native, unified tooling for self-hosted AI model inference and deep integration with Python-first AI frameworks (PyTorch, vLLM, LangGraph).
* **Rejected Alternative (Django)**: Overly monolithic and synchronous by design; unneeded overhead for a microservices/API-first architecture.
* **Provider Abstraction Pattern**: All model calls must pass through a unified `BaseLLMProvider` interface to ensure zero-cost switching between third-party APIs and future self-hosted models.

---

## 4. Tier 2: Local Headless Daemon Specification (The Engine)

### 4.1 Overview
The Daemon is a zero-UI, low-resource background process executing on the user's local operating system. It holds system-level permissions, monitors hardware, captures audio, and exposes a local WebSocket server.

### 4.2 Technology Matrix

| Component | Technology | Specification / Version | Architectural Justification |
| :--- | :--- | :--- | :--- |
| **Language & Toolchain** | Rust | `2021 Edition (rustc / cargo)` | Bare-metal execution speed, zero garbage collection pauses, and compiles to a single standalone `.exe`. |
| **Async Runtime** | Tokio | `1.0+ (features: full)` | Multi-threaded asynchronous runtime optimized for high-throughput non-blocking I/O. |
| **Localhost WS Server** | Tokio-Tungstenite | `Latest` | Ultra-low latency WebSocket server listening on `127.0.0.1:8765` for GUI connections. |
| **Upstream Uplink** | Reqwest + Tungstenite | `TLS / WSS` | Persistent duplex uplink to Tier 1 Central Backend. |
| **Serialization** | Serde / Serde JSON | `1.0+` | Compile-time zero-copy JSON serialization and deserialization. |
| **Process Execution** | Tokio Process | `tokio::process::Command` | Non-blocking execution of local OS commands and subprocess management. |
| **Hardware Telemetry** | Sysinfo | `Latest` | Real-time monitoring of host CPU, RAM, battery, thermal, and network metrics. |
| **Audio I/O** | CPAL | `Latest` | Cross-Platform Audio Library for raw low-latency microphone capture and speaker streaming. |
| **Voice Detection** | Tract-ONNX | `Latest` | Runs lightweight local neural models (Silero VAD) to detect voice activity before transmission. |
| **Silent Process Mode** | Windows Subsystem | `#![windows_subsystem = "windows"]` | Strips console window allocation on Windows, enabling silent background execution. |

### 4.3 Resource & Execution Budgets
* **Idle Memory (RAM)**: `< 15 MB`
* **Active I/O Latency**: `< 2 ms` on local loopback.
* **Crash Recovery**: Auto-restarts via Windows Service Manager or scheduled task watchdog.

---

## 5. Tier 3: Desktop GUI Client Specification (The Face)

### 5.1 Overview
The Desktop GUI is a presentation layer designed to deliver visual status, audio visualizers, and interactive chat. It has no direct access to local system resources.

### 5.2 Technology Matrix

| Component | Technology | Specification / Version | Architectural Justification |
| :--- | :--- | :--- | :--- |
| **Desktop Native Shell** | Tauri v2 | `2.0+ (Rust Core)` | Uses native OS WebView2 on Windows; produces tiny `< 10MB` installers with `< 40MB` idle RAM. |
| **UI Framework** | React | `18 / 19` | Declarative, component-driven UI with robust ecosystem and rapid state reconciliation. |
| **Type System** | TypeScript | `5.0+` | Static typing shared with daemon packet definitions to eliminate runtime type mismatches. |
| **Build Tool & HMR** | Vite | `Latest` | Sub-50ms Hot Module Replacement (HMR) and optimized static asset compilation. |
| **Styling Engine** | Tailwind CSS | `Latest` | Utility-first CSS framework enabling dark-mode glassmorphic HUD interfaces. |
| **Micro-Animations** | Framer Motion | `Latest` | Fluid visual transitions, pulsing auras, and state-driven animations. |
| **Audio Visualization** | HTML5 Canvas / Web Audio | Native Browser API | Renders real-time audio waveforms without third-party library overhead. |
| **IPC Transport** | Native WebSocket | Browser Standard | Connects to `ws://127.0.0.1:8765` using standard browser networking. |

### 5.3 Design Decisions & Rejected Alternatives
* **Rejected Alternative (Electron)**: Bundles an entire Chromium browser and Node.js runtime, consuming `200MB–400MB` RAM and inflating installer size to `> 100MB`.
* **Rejected Alternative (Next.js)**: Server-Side Rendering (SSR) and Server Components introduce unnecessary complexity for a local single-page desktop client.

---

## 6. Inter-Tier Protocol & Communication Matrix

```text
+-----------------------+--------------------------+-----------------------------+
| Connection Path       | Protocol / Transport     | Security / Authentication   |
+-----------------------+--------------------------+-----------------------------+
| Tier 3 <--> Tier 2    | WebSocket (127.0.0.1)    | Ephemeral Session Token     |
| (GUI to Daemon)       | JSON / Binary Audio      | (~/.veronica/session.token) |
+-----------------------+--------------------------+-----------------------------+
| Tier 2 <--> Tier 1    | WSS / HTTPS (Internet)   | Machine API Key             |
| (Daemon to Backend)   | JSON Envelope + TLS 1.3  | (X-Veronica-Daemon-Key)     |
+-----------------------+--------------------------+-----------------------------+
| User <--> Tier 1      | HTTPS (REST)             | OAuth2 Bearer Token         |
| (User to Backend)     | JSON Payloads            | (JWT Access / Refresh)      |
+-----------------------+--------------------------+-----------------------------+
```

---

## 7. Document Approval & Maintenance

* **Author**: Engineering Architecture Team (Veronica Co-Pilot & Lead Architect)
* **Change Management**: Any modification to tier protocols, frameworks, or security boundaries requires an official update to this specification and an associated Architecture Decision Record (ADR).
