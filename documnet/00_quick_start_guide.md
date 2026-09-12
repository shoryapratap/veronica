# Project Veronica: Developer Quick Start & Daily Runbook

```text
Document ID:     VER-GUIDE-000
Title:           Daily Operations & Command Cheat Sheet
Target Audience: Developers & Operators
Status:          ACTIVE REFERENCE
```

---

## 1. Quick Start Matrix (At a Glance)

Whenever you sit down to develop or run Project Veronica, open three terminal tabs and run these commands:

| Tier | Working Directory (CWD) | Command to Start | Where It Lives / Port |
| :--- | :--- | :--- | :--- |
| **Tier 1: Backend** | `Veronica/backend/` | `.\.venv\Scripts\Activate.ps1`<br>`uvicorn app.main:app --reload` | `http://localhost:8000`<br>Docs: `http://localhost:8000/docs` |
| **Tier 2: Daemon** | `Veronica/daemon/` | `cargo run` | `127.0.0.1:8765` (Localhost WS) |
| **Tier 3: GUI Face**| `Veronica/gui/veronica/` | `npm run tauri dev` | Standalone Native Windows App |

---

## 2. Tier 1: Central Backend (`backend/`)

### How to Start the Backend Server:
```powershell
# 1. Navigate to backend
cd "c:\coding\personal project\Veronica\backend"

# 2. Activate the virtual environment
.\.venv\Scripts\Activate.ps1

# 3. Start the server with hot-reload
uvicorn app.main:app --reload
```
* **Interactive API Documentation**: Open **`http://localhost:8000/docs`** in your browser.
* **Stop the Server**: Press `Ctrl + C` in the terminal.

### Regular Commands for Backend:
* **Install a new Python package**:
  ```powershell
  pip install <package-name>
  ```
* **Update `requirements.txt` after installing new tools**:
  ```powershell
  pip freeze > requirements.txt
  ```
* **Run database migrations (when we set up Alembic tables)**:
  ```powershell
  alembic revision --autogenerate -m "description of changes"
  alembic upgrade head
  ```

---

## 3. Tier 2: Local Headless Daemon (`daemon/`)

### How to Start the Daemon:
```powershell
# 1. Navigate to daemon
cd "c:\coding\personal project\Veronica\daemon"

# 2. Compile and run the daemon
cargo run
```
* **What it does**: Boots the Tokio runtime, starts the background loop, and listens on `127.0.0.1:8765`.
* **Stop the Daemon**: Press `Ctrl + C`.

### Regular Commands for Daemon:
* **Fast syntax & type check (without compiling a binary)**:
  ```powershell
  cargo check
  ```
  *(Always run this while writing code—it validates your Rust code in 1–2 seconds).*
* **Add a new Rust crate (dependency)**:
  ```powershell
  cargo add <crate-name>
  ```
* **Build a standalone production `.exe`**:
  ```powershell
  cargo build --release
  ```
  *(Your final optimized `veronica-daemon.exe` will be generated in `daemon/target/release/`)*.

---

## 4. Tier 3: GUI Desktop Client (`gui/veronica/`)

### How to Launch the Native Desktop Window:
```powershell
# 1. Navigate to GUI directory
cd "c:\coding\personal project\Veronica\gui\veronica"

# 2. Launch the Tauri desktop app
npm run tauri dev
```
* **What it does**: Boots Vite's frontend server and pops open the native Windows desktop application with hot-reloading enabled.
* **Close the App**: Close the desktop window or press `Ctrl + C` in the terminal.

### Fast Web-Only Mode (No Rust needed):
If you only want to tweak CSS or React components in your browser without opening the desktop window:
```powershell
npm run dev
```
* Then open `http://localhost:1420` in your browser.

### Regular Commands for GUI:
* **Install a new npm package**:
  ```powershell
  npm install <package-name>
  ```
* **Build the final Windows installer (`.exe` / `.msi`)**:
  ```powershell
  npm run tauri build
  ```
  *(Generates the standalone production installer for end-users)*.

---

## 5. Daily Troubleshooting & Quick Fixes

### Fix 1: PowerShell blocks `.venv` activation script
If you see `File Activate.ps1 cannot be loaded because running scripts is disabled`:
```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
.\.venv\Scripts\Activate.ps1
```

### Fix 2: Port already in use error (e.g. port 8000 or 8765)
If a previous process didn't close cleanly and port 8000 is blocked:
```powershell
# Find the process ID (PID) using port 8000
netstat -ano | findstr :8000

# Kill that process (replace 12345 with the PID from the last column)
taskkill /PID 12345 /F
```

### Fix 3: Clean corrupted build caches
* **Rust**: Run `cargo clean` inside `daemon/` or `gui/veronica/src-tauri/`.
* **Node**: Delete `node_modules` and run `npm install`.
* **Python**: Delete `.venv` and re-run `python -m venv .venv && pip install -r requirements.txt`.
