# Project Veronica - AI Tool Manager & 2-Step Master Function Architecture

> **Design Designation**: 2-Step Just-in-Time (JIT) Tool Discovery & Master Function Execution  
> **Target Subsystem**: Local Daemon (daemon/src/ai/ & daemon/src/schedule/, daemon/src/system/)  
> **Database**: PostgreSQL (with context memory & auto-clear queue)

---

## 1. Architectural Philosophy

To avoid token bloat, latency, and hallucinated function arguments as Project Veronica scales to dozens of features, we do **NOT** send 50+ heavy JSON schemas in the initial prompt or with every user voice turn.

Instead, we enforce a strict **2-Step Dynamic Discovery Pattern**:
1. **Catalog Prompting**: AI receives only a minimal plain-text list of available features and their function names from daemon/src/ai/catalog.rs.
2. **Step 1 (Schema Request)**: AI selects the target feature & function (select_feature_function). The Tool Manager delegates to that feature's **Master Function**, which returns the exact JSON schema on-demand.
3. **Step 2 (Execution)**: AI returns the filled arguments (execute_feature_function). The Tool Manager delegates back to that feature's **Master Function**, which routes and executes the local implementation.

`mermaid
sequenceDiagram
    autonumber
    actor User as User (Voice / Text)
    participant AI as Gemini Live (Voice Engine)
    participant Mgr as Tool Manager (manager.rs)
    participant Cat as Catalog (catalog.rs)
    participant Master as Feature Master Function (e.g. Schedule)
    participant DB as PostgreSQL (veronica_db)

    Note over AI,Cat: Session Start: Prompt contains lightweight catalog
    User->>AI: "Remind me in 2 mins to submit assignment"
    AI->>Mgr: select_feature_function("schedule", "create")
    Mgr->>Master: ScheduleEngine::master_schema("create")
    Master-->>Mgr: Exact JSON Schema ({ title, description, time_expression })
    Mgr-->>AI: { status: "schema_ready", expected_schema: ... }
    AI->>Mgr: execute_feature_function("schedule", "create", data={...})
    Mgr->>Master: ScheduleEngine::master_execute("create", &data)
    Master->>DB: INSERT INTO schedules (...) WITH description context
    Master-->>Mgr: { status: "success", item: ... }
    Mgr-->>AI: Tool Response
    AI-->>User: Spoken voice confirmation
`

---

## 2. Component Directory & File Responsibilities

### File 1: AI Voice & Duplex Streaming Layer (daemon/src/ai/live.rs)
- Manages bidirectional real-time audio (24kHz output, 16kHz microphone input) with Gemini Live.
- Injects catalog::get_catalog() into Veronica's initial system instruction.
- Exposes two meta-tools from ToolManager: select_feature_function and execute_feature_function.
- Strictly decoupled: Contains zero database queries or tool-specific execution logic.

### File 2: Feature & Function Catalog (daemon/src/ai/catalog.rs)
- Lightweight registry of active system capabilities.
- Minimal text footprint (~40 tokens) so Veronica knows what she can do without carrying schema bloat.
`yaml
[AVAILABLE SYSTEM CAPABILITIES]
- schedule: [create, list, cancel]
- system: [capture_screen, launch_app]
`

### File 3: Dedicated Tool Manager (daemon/src/ai/manager.rs)
- The central coordinator between Gemini Live and all feature subsystems.
- Holds no domain-specific parameter schemas.
- Acts as a pure router:
  - When asked for a schema: Calls Feature::master_schema(function_name)
  - When asked to execute: Calls Feature::master_execute(function_name, data)

### File 4: Feature Modules with Master Functions
Every feature module (e.g. schedule, system, and future media, 
otes) must implement a **Master Function** with two duties:
1. master_schema(function_name: &str) -> serde_json::Value
   - Returns the exact parameter specification for any of its own functions.
2. master_execute(&self, function_name: &str, data: &serde_json::Value) -> serde_json::Value
   - Receives the user-filled payload and invokes the internal implementation.

---

## 3. PostgreSQL Database & Auto-Cleanup Queue (daemon/src/schedule/mod.rs)

### Table: schedules
`sql
CREATE TABLE IF NOT EXISTS schedules (
    id VARCHAR(64) PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,         -- ✨ Context: Why user set the reminder & exact words
    schedule_type VARCHAR(64) NOT NULL, -- reminder, meeting, habit, timer
    target_time VARCHAR(128) NOT NULL,
    target_timestamp BIGINT NOT NULL,
    duration_minutes INT DEFAULT 0,
    recurrence VARCHAR(64) DEFAULT 'none',
    status VARCHAR(64) NOT NULL DEFAULT 'pending',
    created_at VARCHAR(128) NOT NULL
);
`

### 24/7 Auto-Check & Instant Purge Queue
1. Background worker checks PostgreSQL every 3 seconds (	arget_timestamp <= now).
2. When a reminder triggers:
   - **Voice Alert**: Pushes to lert_queue_tx so Veronica speaks directly to the user over the audio stream.
   - **Desktop Alert**: Fires a native Windows Toast notification displaying Title and full Context Description.
   - **Database Auto-Clear**:
     - Non-recurring reminders are **immediately deleted (DELETE FROM schedules WHERE id = )** so the database never accumulates stale history.
     - Recurring items (daily, weekly) automatically advance their timestamps to the next occurrence.

---

## 4. How to Add a New Feature (Developer Checklist)

When creating a new feature (e.g. media or rowser):
1. **Create the Feature Module** (daemon/src/media/mod.rs):
   - Implement pub fn master_schema(func: &str) -> serde_json::Value.
   - Implement pub async fn master_execute(&self, func: &str, data: &serde_json::Value) -> serde_json::Value.
2. **Register in Catalog** (daemon/src/ai/catalog.rs):
   - Add the feature name and its function list.
3. **Register in Tool Manager** (daemon/src/ai/manager.rs):
   - Add a 1-line match arm forwarding eature == "media" to the media Master Function.
