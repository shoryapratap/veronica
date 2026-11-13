use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsApp {
    pub name: String,
    pub path: String,
    pub app_type: String,       // "Win32" ya "UWP"
    pub installed_by: String,   // "System" ya "User"
    pub is_system_app: bool,    // true = System, false = User Installed
    pub icon_path: Option<String>,
}

/// System SCAN APP FUNCTION to get list of all installed applications in Windows OS
/// This will store the list of apps in AppData/Veronica/installed_apps.json
pub async fn scan_app() -> Result<Vec<WindowsApp>, String> {

    let mut apps: Vec<WindowsApp> = Vec::new();
    
//to scan win32 apps
    let mut scan_dirs: Vec<PathBuf> = Vec::new();

    // System-wide apps (C:\ProgramData)
    if let Ok(pd) = env::var("ProgramData") {
        scan_dirs.push(PathBuf::from(pd).join("Microsoft\\windows\\start Menu\\Programs"));
    }

    // User-specific apps (AppData\Roaming)
    if let Ok(ad)  = env::var("AppData") {
        scan_dirs.push(PathBuf::from(ad).join("Microsoft\\windows\\start Menu\\Programs"));
    }

    for dir in scan_dirs {
        if dir.exists() {
            scan_directory_recursive(&dir, &mut apps);
        }
    }

// to scan uwp apps
    if let Ok(output) = tokio::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", "Get-StartApps | ConvertTo-Json"])
        .output()
        .await
    {
        if output.status.success() {
            let json_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                if let Some(arr) = parsed.as_array() {
                    for item in arr {
                        let name = item.get("Name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let app_id = item.get("AppID").and_then(|v| v.as_str()).unwrap_or("").to_string();

                        if !name.is_empty() && !app_id.is_empty() {
                            // Check karein: kya ye app shortcut se pehle hi add ho chuki hai?
                            let already_exists = apps.iter().any(|a| a.name.eq_ignore_ascii_case(&name));
                            if !already_exists {
                                let is_system = app_id.starts_with("Microsoft.") 
                                    || app_id.starts_with("windows.")
                                    || app_id.contains("\\Windows\\");
                                let installed_by = if is_system { "System" } else { "User" };

                                apps.push(WindowsApp {
                                    name,
                                    path: app_id,
                                    app_type: "UWP".to_string(),
                                    installed_by: installed_by.to_string(),
                                    is_system_app: is_system,
                                    icon_path: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

// 1. to sort all apps by alphabtical order
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // 2. Automatically link and extract real icons
    if let Ok(app_data_dir) = env::var("APPDATA") {
        let icons_dir = PathBuf::from(&app_data_dir).join("Veronica").join("icons");
        let mut missing_icons = false;

        for app in &mut apps {
            let clean_name = app.name.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "_").trim().to_string();
            let icon_file = icons_dir.join(format!("{}.png", clean_name));
            if icon_file.exists() {
                app.icon_path = Some(format!("/icons/{}.png", clean_name));
            } else {
                missing_icons = true;
            }
        }

        // If newly installed apps are detected without icons, trigger extraction
        if missing_icons {
            let possible_scripts = [
                PathBuf::from("scripts").join("extract_all_icons.ps1"),
                PathBuf::from("..").join("scripts").join("extract_all_icons.ps1"),
            ];
            for script in &possible_scripts {
                if script.exists() {
                    let _ = tokio::process::Command::new("powershell")
                        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script.to_str().unwrap_or("")])
                        .status()
                        .await;

                    for app in &mut apps {
                        let clean_name = app.name.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "_").trim().to_string();
                        let icon_file = icons_dir.join(format!("{}.png", clean_name));
                        if icon_file.exists() {
                            app.icon_path = Some(format!("/icons/{}.png", clean_name));
                        }
                    }
                    break;
                }
            }
        }
    }

// 3. save the list of apps in AppData/Veronica/installed_apps.json
    if let Ok(app_data_dir) = env::var("APPDATA") {
        let veronica_dir = PathBuf::from(app_data_dir).join("Veronica");
        let _ = fs::create_dir_all(&veronica_dir);
        let json_file = veronica_dir.join("installed_apps.json");

        if let Ok(json_content) = serde_json::to_string_pretty(&apps) {
            let _ = fs::write(&json_file, &json_content);
            println!("✅ [App Scanner] Saved {} apps to: {:?}", apps.len(), json_file);

            // Also keep public/installed_apps.json synced for GUI Vite dev preview
            let public_file = PathBuf::from("gui").join("veronica").join("public").join("installed_apps.json");
            if let Some(parent) = public_file.parent() {
                if parent.exists() {
                    let _ = fs::write(&public_file, &json_content);
                }
            }
        }
    }

    Ok(apps)
}

/// Helper function to search in folders and subfolders for .lnk files
fn scan_directory_recursive(dir: &Path, apps: &mut Vec<WindowsApp>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            // Agar folder ke andar aur subfolder hai, toh uske andar jao
            if path.is_dir() {
                scan_directory_recursive(&path, apps);
            } else if let Some(ext) = path.extension() {
                // Agar shortcut (.lnk) file hai
                if ext == "lnk" {
                    if let Some(file_name) = path.file_stem() {
                        let name = file_name.to_string_lossy().to_string();
                        let name_lower = name.to_lowercase();

                        // Uninstaller aur help files ko ignore karein
                        if !name_lower.contains("uninstall") && !name_lower.contains("help") && !name_lower.contains("documentation") {
                            let path_str = path.to_string_lossy().to_string();
                            let is_system = path_str.to_lowercase().contains("windows powershell")
                                || path_str.to_lowercase().contains("administrative tools")
                                || path_str.to_lowercase().contains("windows accessories")
                                || path_str.to_lowercase().contains("system32");
                            let installed_by = if is_system { "System" } else { "User" };

                            // Duplicate check
                            if !apps.iter().any(|a| a.name.eq_ignore_ascii_case(&name)) {
                                apps.push(WindowsApp {
                                    name,
                                    path: path_str.clone(),
                                    app_type: "Win32".to_string(),
                                    installed_by: installed_by.to_string(),
                                    is_system_app: is_system,
                                    icon_path: Some(path_str),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}


/// Smart Open App: Win32, UWP, common aliases, and direct binaries
pub async fn open_app(target: &str) -> Result<String, String> {
    let target_clean = target.trim();
    let target_lower = target_clean.to_lowercase();

    // 1. Resolve common aliases (e.g. "chrome" -> "Google Chrome", "code" -> "Visual Studio Code")
    let resolved_name = match target_lower.as_str() {
        "chrome" | "google-chrome" => "Google Chrome",
        "code" | "vscode" | "vs-code" => "Visual Studio Code",
        "terminal" | "wt" | "windows-terminal" => "Terminal",
        "calc" | "calculator" => "Calculator",
        "notepad" => "Notepad",
        "spotify" => "Spotify",
        "explorer" | "files" => "File Explorer",
        "taskmgr" | "task-manager" | "task manager" => "Task Manager",
        "edge" | "microsoft-edge" => "Microsoft Edge",
        "cmd" | "command-prompt" => "Command Prompt",
        "powershell" => "Windows PowerShell",
        _ => target_clean,
    };
    let resolved_lower = resolved_name.to_lowercase();

    // 2. Search in cached installed_apps.json
    if let Ok(app_data) = env::var("APPDATA") {
        let json_file = PathBuf::from(app_data).join("Veronica\\installed_apps.json");
        if let Ok(content) = fs::read_to_string(&json_file) {
            if let Ok(apps) = serde_json::from_str::<Vec<WindowsApp>>(&content) {
                // Priority 1: Exact Name Match
                let matched_app = apps.iter().find(|a| a.name.eq_ignore_ascii_case(resolved_name))
                    // Priority 2: Starts-with or full word match (avoids "Unicode" matching "code")
                    .or_else(|| {
                        apps.iter().find(|a| {
                            let a_low = a.name.to_lowercase();
                            a_low.starts_with(&resolved_lower)
                                || a_low.split_whitespace().any(|w| w == resolved_lower)
                        })
                    })
                    // Priority 3: Substring match as fallback
                    .or_else(|| {
                        apps.iter().find(|a| a.name.to_lowercase().contains(&resolved_lower))
                    });

                if let Some(app) = matched_app {
                    if app.app_type == "UWP" {
                        // Launch UWP via cmd start shell:AppsFolder (100% reliable)
                        let shell_path = format!("shell:AppsFolder\\{}", app.path);
                        let res = tokio::process::Command::new("cmd")
                            .args(["/c", "start", "", &shell_path])
                            .status()
                            .await;

                        match res {
                            Ok(s) if s.success() => return Ok(format!("Successfully launched UWP app: {}", app.name)),
                            _ => {
                                let _ = tokio::process::Command::new("powershell")
                                    .args(["-NoProfile", "-Command", &format!("Start-Process '{}'", shell_path)])
                                    .spawn();
                                return Ok(format!("Launched UWP app: {}", app.name));
                            }
                        }
                    } else {
                        // Launch Win32 Shortcut via cmd start (resolves .lnk, target dir, icon)
                        let res = tokio::process::Command::new("cmd")
                            .args(["/c", "start", "", &app.path])
                            .status()
                            .await;

                        match res {
                            Ok(s) if s.success() => return Ok(format!("Successfully launched Win32 app: {}", app.name)),
                            _ => {
                                let _ = tokio::process::Command::new("powershell")
                                    .args(["-NoProfile", "-Command", &format!("Start-Process '{}'", app.path)])
                                    .spawn();
                                return Ok(format!("Launched Win32 app: {}", app.name));
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Fallback: PowerShell Start-Process (handles system binaries cleanly without modal error)
    let ps_res = tokio::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!("Start-Process '{}'", target_clean)])
        .spawn();

    match ps_res {
        Ok(_) => Ok(format!("Launched application: {}", target_clean)),
        Err(e) => Err(format!("Failed to open '{}': {}", target_clean, e)),
    }
}

/// Smart Close App: Resolves display names, process names, and window titles
pub async fn close_app(target: &str) -> Result<String, String> {
    let target_clean = target.trim();
    let target_lower = target_clean.to_lowercase();

    // 1. Resolve common application process aliases
    let proc_targets: Vec<String> = match target_lower.as_str() {
        "chrome" | "google chrome" | "google-chrome" => vec!["chrome".to_string()],
        "code" | "vscode" | "visual studio code" | "vs code" => vec!["Code".to_string()],
        "terminal" | "wt" | "windows terminal" | "windows-terminal" => vec!["WindowsTerminal".to_string()],
        "calc" | "calculator" => vec!["CalculatorApp".to_string(), "Calculator".to_string()],
        "notepad" => vec!["notepad".to_string(), "Notepad".to_string()],
        "spotify" => vec!["Spotify".to_string()],
        "explorer" | "file explorer" | "files" => vec!["explorer".to_string()],
        "taskmgr" | "task manager" | "task-manager" => vec!["Taskmgr".to_string()],
        "edge" | "microsoft edge" | "msedge" => vec!["msedge".to_string()],
        "word" | "microsoft word" => vec!["WINWORD".to_string()],
        "excel" | "microsoft excel" => vec!["EXCEL".to_string()],
        "powerpoint" => vec!["POWERPNT".to_string()],
        "discord" => vec!["Discord".to_string()],
        "slack" => vec!["slack".to_string()],
        "steam" => vec!["steam".to_string()],
        other => vec![other.trim_end_matches(".exe").to_string()],
    };

    let mut closed_any = false;

    // 2. Try taskkill for primary known process names (fastest & cleanest on Windows)
    for proc in &proc_targets {
        let im_arg = if proc.ends_with(".exe") {
            proc.clone()
        } else {
            format!("{}.exe", proc)
        };

        let status = tokio::process::Command::new("taskkill")
            .args(["/F", "/IM", &im_arg, "/T"])
            .output()
            .await;

        if let Ok(out) = status {
            if out.status.success() {
                closed_any = true;
            }
        }
    }

    if closed_any {
        return Ok(format!("Successfully closed '{}'", target_clean));
    }

    // 3. Fallback: PowerShell by ProcessName OR MainWindowTitle
    let ps_script = format!(
        r#"$p = Get-Process | Where-Object {{ $_.ProcessName -like '*{0}*' -or $_.MainWindowTitle -like '*{0}*' }}; if ($p) {{ $p | Stop-Process -Force; exit 0 }} else {{ exit 1 }}"#,
        target_clean
    );

    let res = tokio::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .output()
        .await;

    match res {
        Ok(out) if out.status.success() => Ok(format!("Successfully closed '{}'", target_clean)),
        Ok(_) => Ok(format!("App '{}' was not active or has been closed", target_clean)),
        Err(e) => Err(format!("Failed to close '{}': {}", target_clean, e)),
    }
}

// =========================================================================
// 📁 APP GROUPS & WORKSPACES MANAGEMENT
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppGroup {
    pub name: String,
    pub apps: Vec<String>,
    pub description: Option<String>,
}

/// Helper: AppData/Veronica/app_groups.json ka path nikalta hai
fn get_app_groups_file() -> PathBuf {
    let app_data = env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(app_data).join("Veronica");
    let _ = fs::create_dir_all(&dir);
    dir.join("app_groups.json")
}

/// Helper: Saare saved app groups ko file se load karta hai
pub fn list_app_groups() -> Vec<AppGroup> {
    let file = get_app_groups_file();
    if let Ok(content) = fs::read_to_string(&file) {
        serde_json::from_str::<Vec<AppGroup>>(&content).unwrap_or_default()
    } else {
        Vec::new()
    }
}

/// Helper: Groups ko file me save karta hai
fn save_app_groups(groups: &[AppGroup]) -> Result<(), String> {
    let file = get_app_groups_file();
    let json_content = serde_json::to_string_pretty(groups)
        .map_err(|e| format!("JSON serialize error: {}", e))?;
    fs::write(&file, json_content)
        .map_err(|e| format!("Failed to write app groups file: {}", e))
}

/// 1. Create App Group: Naya group banata hai
pub fn create_app_group(name: &str, apps: Vec<String>, description: Option<String>) -> Result<AppGroup, String> {
    let name_clean = name.trim();
    if name_clean.is_empty() {
        return Err("Group name cannot be empty".to_string());
    }

    let mut groups = list_app_groups();
    if groups.iter().any(|g| g.name.eq_ignore_ascii_case(name_clean)) {
        return Err(format!("App group '{}' already exists", name_clean));
    }

    let new_group = AppGroup {
        name: name_clean.to_string(),
        apps,
        description,
    };

    groups.push(new_group.clone());
    save_app_groups(&groups)?;
    Ok(new_group)
}

/// 2. Delete App Group: Group delete karta hai
pub fn delete_app_group(name: &str) -> Result<bool, String> {
    let name_clean = name.trim();
    let mut groups = list_app_groups();
    let initial_len = groups.len();

    groups.retain(|g| !g.name.eq_ignore_ascii_case(name_clean));

    if groups.len() < initial_len {
        save_app_groups(&groups)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 3. Rename App Group: Group ka naam badalta hai
pub fn rename_app_group(old_name: &str, new_name: &str) -> Result<AppGroup, String> {
    let old_clean = old_name.trim();
    let new_clean = new_name.trim();

    if new_clean.is_empty() {
        return Err("New group name cannot be empty".to_string());
    }

    let mut groups = list_app_groups();
    if groups.iter().any(|g| g.name.eq_ignore_ascii_case(new_clean)) {
        return Err(format!("Group with name '{}' already exists", new_clean));
    }

    if let Some(group) = groups.iter_mut().find(|g| g.name.eq_ignore_ascii_case(old_clean)) {
        group.name = new_clean.to_string();
        let updated = group.clone();
        save_app_groups(&groups)?;
        Ok(updated)
    } else {
        Err(format!("Group '{}' not found", old_clean))
    }
}

/// 3. Add App to Group: Group me nayi app jodta hai
pub fn add_app_to_group(group_name: &str, app_name: &str) -> Result<AppGroup, String> {
    let grp_clean = group_name.trim();
    let app_clean = app_name.trim();

    let mut groups = list_app_groups();
    if let Some(group) = groups.iter_mut().find(|g| g.name.eq_ignore_ascii_case(grp_clean)) {
        if !group.apps.iter().any(|a| a.eq_ignore_ascii_case(app_clean)) {
            group.apps.push(app_clean.to_string());
        }
        let updated = group.clone();
        save_app_groups(&groups)?;
        Ok(updated)
    } else {
        Err(format!("Group '{}' not found", grp_clean))
    }
}

/// 3. Remove App from Group: Group se app hatata hai
pub fn remove_app_from_group(group_name: &str, app_name: &str) -> Result<AppGroup, String> {
    let grp_clean = group_name.trim();
    let app_clean = app_name.trim();

    let mut groups = list_app_groups();
    if let Some(group) = groups.iter_mut().find(|g| g.name.eq_ignore_ascii_case(grp_clean)) {
        group.apps.retain(|a| !a.eq_ignore_ascii_case(app_clean));
        let updated = group.clone();
        save_app_groups(&groups)?;
        Ok(updated)
    } else {
        Err(format!("Group '{}' not found", grp_clean))
    }
}

/// 4. Open App Group: Group ki saari apps ek sath launch karta hai
pub async fn open_app_group(name: &str) -> Result<Vec<String>, String> {
    let name_clean = name.trim();
    let groups = list_app_groups();

    if let Some(group) = groups.iter().find(|g| g.name.eq_ignore_ascii_case(name_clean)) {
        let mut results = Vec::new();
        for app in &group.apps {
            match open_app(app).await {
                Ok(msg) => results.push(msg),
                Err(e) => results.push(format!("Failed to open {}: {}", app, e)),
            }
        }
        Ok(results)
    } else {
        Err(format!("App group '{}' not found", name_clean))
    }
}

/// 5. Close App Group: Group ki saari apps ek sath close karta hai
pub async fn close_app_group(name: &str) -> Result<Vec<String>, String> {
    let name_clean = name.trim();
    let groups = list_app_groups();

    if let Some(group) = groups.iter().find(|g| g.name.eq_ignore_ascii_case(name_clean)) {
        let mut results = Vec::new();
        for app in &group.apps {
            match close_app(app).await {
                Ok(msg) => results.push(msg),
                Err(e) => results.push(format!("Failed to close {}: {}", app, e)),
            }
        }
        Ok(results)
    } else {
        Err(format!("App group '{}' not found", name_clean))
    }
}

// ==========================================
// 👑 MASTER FUNCTION: APP FEATURE
// ==========================================

/// Returns the exact JSON schema for any requested function in the app feature.
pub fn master_schema(function_name: &str) -> serde_json::Value {
    match function_name {
        "scan" => serde_json::json!({
            "type": "OBJECT",
            "properties": {}
        }),
        "open" => serde_json::json!({
            "type": "OBJECT",
            "properties": {
                "app_name": { "type": "STRING", "description": "Name or path of the app to launch" }
            },
            "required": ["app_name"]
        }),
        "close" => serde_json::json!({
            "type": "OBJECT",
            "properties": {
                "app_name": { "type": "STRING", "description": "Name of the application or process to close" }
            },
            "required": ["app_name"]
        }),
        "create_group" => serde_json::json!({
            "type": "OBJECT",
            "properties": {
                "group_name": { "type": "STRING", "description": "Name of the workspace group (e.g. 'Dev Setup')" },
                "apps": { "type": "ARRAY", "items": { "type": "STRING" }, "description": "List of app names to include" },
                "description": { "type": "STRING", "description": "Optional group description" }
            },
            "required": ["group_name", "apps"]
        }),
        "delete_group" => serde_json::json!({
            "type": "OBJECT",
            "properties": {
                "group_name": { "type": "STRING", "description": "Name of the group to delete" }
            },
            "required": ["group_name"]
        }),
        "edit_group" => serde_json::json!({
            "type": "OBJECT",
            "properties": {
                "group_name": { "type": "STRING", "description": "Target group name" },
                "action": { "type": "STRING", "enum": ["add", "remove", "rename"], "description": "Edit action" },
                "target": { "type": "STRING", "description": "App name to add/remove, or new group name if renaming" }
            },
            "required": ["group_name", "action", "target"]
        }),
        "open_group" => serde_json::json!({
            "type": "OBJECT",
            "properties": {
                "group_name": { "type": "STRING", "description": "Name of the app group to open all apps together" }
            },
            "required": ["group_name"]
        }),
        "close_group" => serde_json::json!({
            "type": "OBJECT",
            "properties": {
                "group_name": { "type": "STRING", "description": "Name of the app group to close all apps together" }
            },
            "required": ["group_name"]
        }),
        "list_groups" => serde_json::json!({
            "type": "OBJECT",
            "properties": {}
        }),
        other => serde_json::json!({
            "error": format!("Unknown function '{}' in app feature", other)
        })
    }
}

/// Dispatches and executes the requested app function using provided arguments.
pub async fn master_execute(function_name: &str, data: &serde_json::Value) -> serde_json::Value {
    match function_name {
        "scan" => match scan_app().await {
            Ok(apps) => serde_json::json!({
                "status": "success",
                "total_apps": apps.len(),
                "sample": apps.into_iter().take(5).collect::<Vec<_>>()
            }),
            Err(e) => serde_json::json!({ "status": "error", "message": e })
        },
        "open" => {
            let app_name = data.get("app_name").and_then(|v| v.as_str()).unwrap_or("");
            match open_app(app_name).await {
                Ok(msg) => serde_json::json!({ "status": "success", "message": msg }),
                Err(e) => serde_json::json!({ "status": "error", "message": e })
            }
        },
        "close" => {
            let app_name = data.get("app_name").and_then(|v| v.as_str()).unwrap_or("");
            match close_app(app_name).await {
                Ok(msg) => serde_json::json!({ "status": "success", "message": msg }),
                Err(e) => serde_json::json!({ "status": "error", "message": e })
            }
        },
        "create_group" => {
            let group_name = data.get("group_name").and_then(|v| v.as_str()).unwrap_or("");
            let apps = data.get("apps").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let desc = data.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
            match create_app_group(group_name, apps, desc) {
                Ok(grp) => serde_json::json!({ "status": "success", "group": grp }),
                Err(e) => serde_json::json!({ "status": "error", "message": e })
            }
        },
        "delete_group" => {
            let group_name = data.get("group_name").and_then(|v| v.as_str()).unwrap_or("");
            match delete_app_group(group_name) {
                Ok(true) => serde_json::json!({ "status": "success", "message": format!("Deleted group '{}'", group_name) }),
                Ok(false) => serde_json::json!({ "status": "not_found", "message": format!("Group '{}' not found", group_name) }),
                Err(e) => serde_json::json!({ "status": "error", "message": e })
            }
        },
        "edit_group" => {
            let group_name = data.get("group_name").and_then(|v| v.as_str()).unwrap_or("");
            let action = data.get("action").and_then(|v| v.as_str()).unwrap_or("");
            let target = data.get("target").and_then(|v| v.as_str()).unwrap_or("");
            let res = match action {
                "add" => add_app_to_group(group_name, target),
                "remove" => remove_app_from_group(group_name, target),
                "rename" => rename_app_group(group_name, target),
                _ => Err(format!("Unknown edit action '{}'", action))
            };
            match res {
                Ok(grp) => serde_json::json!({ "status": "success", "group": grp }),
                Err(e) => serde_json::json!({ "status": "error", "message": e })
            }
        },
        "open_group" => {
            let group_name = data.get("group_name").and_then(|v| v.as_str()).unwrap_or("");
            match open_app_group(group_name).await {
                Ok(results) => serde_json::json!({ "status": "success", "launched": results }),
                Err(e) => serde_json::json!({ "status": "error", "message": e })
            }
        },
        "close_group" => {
            let group_name = data.get("group_name").and_then(|v| v.as_str()).unwrap_or("");
            match close_app_group(group_name).await {
                Ok(results) => serde_json::json!({ "status": "success", "closed": results }),
                Err(e) => serde_json::json!({ "status": "error", "message": e })
            }
        },
        "list_groups" => {
            let groups = list_app_groups();
            serde_json::json!({ "status": "success", "count": groups.len(), "groups": groups })
        },
        other => serde_json::json!({
            "status": "error",
            "message": format!("Unknown function '{}' in app feature", other)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scan_apps() {
        let result = scan_app().await;
        assert!(result.is_ok());
        let apps = result.unwrap();
        println!("🚀 Found total {} apps on system!", apps.len());
        for app in apps.iter().take(10) {
            println!("   - [{}] {} (By: {}, System: {}) -> {}", app.app_type, app.name, app.installed_by, app.is_system_app, app.path);
        }
    }

    #[tokio::test]
    async fn test_app_groups_crud() {
        let test_grp = "Test Workspace";
        let _ = delete_app_group(test_grp);

        // 1. Create
        let created = create_app_group(test_grp, vec!["notepad".to_string()], Some("Test work".to_string()));
        assert!(created.is_ok());

        // 2. Add app
        let with_calc = add_app_to_group(test_grp, "calc");
        assert!(with_calc.is_ok());
        assert_eq!(with_calc.unwrap().apps.len(), 2);

        // 3. Remove app
        let without_notepad = remove_app_from_group(test_grp, "notepad");
        assert!(without_notepad.is_ok());
        assert_eq!(without_notepad.unwrap().apps.len(), 1);

        // 4. Rename
        let renamed = rename_app_group(test_grp, "Renamed Workspace");
        assert!(renamed.is_ok());

        // 5. Cleanup
        let deleted = delete_app_group("Renamed Workspace");
        assert!(deleted.unwrap());
    }

    #[tokio::test]
    async fn test_smart_open_app() {
        let r = open_app("notepad").await;
        assert!(r.is_ok(), "Failed to open notepad: {:?}", r);
        let _ = close_app("notepad").await;
    }
}