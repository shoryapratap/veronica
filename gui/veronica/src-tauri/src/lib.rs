use std::fs;
use std::path::PathBuf;
use tauri::Manager;

fn get_veronica_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("Veronica");
        let _ = fs::create_dir_all(&p);
        p
    } else {
        PathBuf::from("C:\\Users\\Default\\AppData\\Roaming\\Veronica")
    }
}

#[tauri::command]
fn get_installed_apps() -> Vec<serde_json::Value> {
    let file = get_veronica_dir().join("installed_apps.json");
    if let Ok(content) = fs::read_to_string(&file) {
        if let Ok(apps) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            if !apps.is_empty() {
                return apps;
            }
        }
    }
    // Check local public folder if appdata was empty
    let local_file = PathBuf::from("public").join("installed_apps.json");
    if let Ok(content) = fs::read_to_string(&local_file) {
        if let Ok(apps) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            return apps;
        }
    }
    vec![]
}

#[tauri::command]
fn rescan_apps() -> Vec<serde_json::Value> {
    let possible_scripts = [
        PathBuf::from("scripts").join("extract_all_icons.ps1"),
        PathBuf::from("..").join("..").join("scripts").join("extract_all_icons.ps1"),
    ];
    for script in &possible_scripts {
        if script.exists() {
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", script.to_str().unwrap_or("")])
                .status();
            break;
        }
    }
    get_installed_apps()
}

#[tauri::command]
fn get_app_groups() -> Vec<serde_json::Value> {
    let file = get_veronica_dir().join("app_groups.json");
    if let Ok(content) = fs::read_to_string(&file) {
        if let Ok(groups) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            return groups;
        }
    }
    vec![]
}

#[tauri::command]
fn save_app_group(
    name: String,
    apps: Vec<String>,
    description: Option<String>,
    old_name: Option<String>,
) -> Result<serde_json::Value, String> {
    let dir = get_veronica_dir();
    let file = dir.join("app_groups.json");

    let mut groups: Vec<serde_json::Value> = if let Ok(content) = fs::read_to_string(&file) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    };

    let group_obj = serde_json::json!({
        "name": name.trim(),
        "apps": apps,
        "description": description.unwrap_or_default()
    });

    let target_name = old_name.as_deref().unwrap_or(name.trim());

    if let Some(pos) = groups.iter().position(|g| {
        g.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case(target_name))
            .unwrap_or(false)
    }) {
        groups[pos] = group_obj.clone();
    } else {
        groups.push(group_obj.clone());
    }

    let serialized = serde_json::to_string_pretty(&groups).map_err(|e| e.to_string())?;
    fs::write(&file, serialized).map_err(|e| e.to_string())?;

    Ok(group_obj)
}

#[tauri::command]
fn rename_app_group(old_name: String, new_name: String) -> Result<serde_json::Value, String> {
    let dir = get_veronica_dir();
    let file = dir.join("app_groups.json");

    let mut groups: Vec<serde_json::Value> = if let Ok(content) = fs::read_to_string(&file) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    };

    let old_clean = old_name.trim();
    let new_clean = new_name.trim();

    if new_clean.is_empty() {
        return Err("New group name cannot be empty".to_string());
    }

    if let Some(pos) = groups.iter().position(|g| {
        g.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case(old_clean))
            .unwrap_or(false)
    }) {
        let mut group = groups[pos].clone();
        if let Some(obj) = group.as_object_mut() {
            obj.insert("name".to_string(), serde_json::Value::String(new_clean.to_string()));
        }
        groups[pos] = group.clone();

        let serialized = serde_json::to_string_pretty(&groups).map_err(|e| e.to_string())?;
        fs::write(&file, serialized).map_err(|e| e.to_string())?;
        Ok(group)
    } else {
        Err(format!("Group '{}' not found", old_clean))
    }
}

#[tauri::command]
fn remove_app_from_group(group_name: String, app_name: String) -> Result<serde_json::Value, String> {
    let dir = get_veronica_dir();
    let file = dir.join("app_groups.json");

    let mut groups: Vec<serde_json::Value> = if let Ok(content) = fs::read_to_string(&file) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    };

    let grp_clean = group_name.trim();
    let app_clean = app_name.trim();

    if let Some(pos) = groups.iter().position(|g| {
        g.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case(grp_clean))
            .unwrap_or(false)
    }) {
        let mut group = groups[pos].clone();
        if let Some(apps_arr) = group.get_mut("apps").and_then(|v| v.as_array_mut()) {
            apps_arr.retain(|v| {
                !v.as_str().map(|s| s.eq_ignore_ascii_case(app_clean)).unwrap_or(false)
            });
        }
        groups[pos] = group.clone();

        let serialized = serde_json::to_string_pretty(&groups).map_err(|e| e.to_string())?;
        fs::write(&file, serialized).map_err(|e| e.to_string())?;
        Ok(group)
    } else {
        Err(format!("Group '{}' not found", grp_clean))
    }
}

#[tauri::command]
fn add_app_to_group(group_name: String, app_name: String) -> Result<serde_json::Value, String> {
    let dir = get_veronica_dir();
    let file = dir.join("app_groups.json");

    let mut groups: Vec<serde_json::Value> = if let Ok(content) = fs::read_to_string(&file) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    };

    let grp_clean = group_name.trim();
    let app_clean = app_name.trim();

    if let Some(pos) = groups.iter().position(|g| {
        g.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case(grp_clean))
            .unwrap_or(false)
    }) {
        let mut group = groups[pos].clone();
        if let Some(apps_arr) = group.get_mut("apps").and_then(|v| v.as_array_mut()) {
            if !apps_arr.iter().any(|v| v.as_str().map(|s| s.eq_ignore_ascii_case(app_clean)).unwrap_or(false)) {
                apps_arr.push(serde_json::Value::String(app_clean.to_string()));
            }
        }
        groups[pos] = group.clone();

        let serialized = serde_json::to_string_pretty(&groups).map_err(|e| e.to_string())?;
        fs::write(&file, serialized).map_err(|e| e.to_string())?;
        Ok(group)
    } else {
        Err(format!("Group '{}' not found", grp_clean))
    }
}

#[tauri::command]
fn delete_app_group(name: String) -> Result<bool, String> {
    let dir = get_veronica_dir();
    let file = dir.join("app_groups.json");

    let mut groups: Vec<serde_json::Value> = if let Ok(content) = fs::read_to_string(&file) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        return Ok(false);
    };

    let initial_count = groups.len();
    groups.retain(|g| {
        !g.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case(name.trim()))
            .unwrap_or(false)
    });

    if groups.len() != initial_count {
        let serialized = serde_json::to_string_pretty(&groups).map_err(|e| e.to_string())?;
        fs::write(&file, serialized).map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn launch_app(
    name: String,
    path: Option<String>,
    app_type: Option<String>,
) -> Result<String, String> {
    let clean_name = name.trim();

    // 1. Direct path & type execution
    if let (Some(target_path), Some(t)) = (&path, &app_type) {
        if !target_path.is_empty() {
            if t.eq_ignore_ascii_case("uwp") {
                let shell_target = if target_path.starts_with("shell:") {
                    target_path.clone()
                } else {
                    format!("shell:AppsFolder\\{}", target_path)
                };
                let _ = std::process::Command::new("cmd")
                    .args(["/c", "start", "", &shell_target])
                    .status();
                return Ok(format!("Launched UWP app: {}", clean_name));
            } else {
                let _ = std::process::Command::new("cmd")
                    .args(["/c", "start", "", target_path])
                    .status();
                return Ok(format!("Launched Win32 app: {}", clean_name));
            }
        }
    }

    // 2. Lookup in installed_apps.json
    let file = get_veronica_dir().join("installed_apps.json");
    if let Ok(content) = fs::read_to_string(&file) {
        if let Ok(apps) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            if let Some(app) = apps.iter().find(|a| {
                a.get("name")
                    .and_then(|v| v.as_str())
                    .map(|n| n.eq_ignore_ascii_case(clean_name))
                    .unwrap_or(false)
            }) {
                let app_t = app.get("app_type").and_then(|v| v.as_str()).unwrap_or("Win32");
                let app_p = app.get("path").and_then(|v| v.as_str()).unwrap_or("");
                if app_t.eq_ignore_ascii_case("uwp") {
                    let shell_target = format!("shell:AppsFolder\\{}", app_p);
                    let _ = std::process::Command::new("cmd")
                        .args(["/c", "start", "", &shell_target])
                        .status();
                    return Ok(format!("Launched UWP app: {}", clean_name));
                } else {
                    let _ = std::process::Command::new("cmd")
                        .args(["/c", "start", "", app_p])
                        .status();
                    return Ok(format!("Launched Win32 app: {}", clean_name));
                }
            }
        }
    }

    // 3. Fallback: PowerShell Start-Process (handles system built-ins like notepad, calc, cmd, wt)
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!("Start-Process '{}'", clean_name)])
        .spawn();

    Ok(format!("Spawned {}", clean_name))
}

#[tauri::command]
fn close_app(target: String) -> Result<String, String> {
    let clean = target.trim();
    let clean_im = if clean.ends_with(".exe") {
        clean.to_string()
    } else {
        format!("{}.exe", clean)
    };

    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/IM", &clean_im, "/T"])
        .output();

    let ps_script = format!(
        r#"$p = Get-Process | Where-Object {{ $_.ProcessName -like '*{0}*' -or $_.MainWindowTitle -like '*{0}*' }}; if ($p) {{ $p | Stop-Process -Force }}"#,
        clean.trim_end_matches(".exe")
    );
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .output();

    Ok(format!("Closed {}", clean))
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct BatteryInfo {
    pub percentage: u32,
    pub is_charging: bool,
}

#[tauri::command]
fn get_battery_info() -> BatteryInfo {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-CimInstance -ClassName Win32_Battery | Select-Object -Property EstimatedChargeRemaining, BatteryStatus | ConvertTo-Json -Compress",
        ])
        .output();

    if let Ok(out) = output {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
            let pct = v
                .get("EstimatedChargeRemaining")
                .and_then(|p| p.as_u64())
                .unwrap_or(100) as u32;
            let status = v.get("BatteryStatus").and_then(|s| s.as_u64()).unwrap_or(2);
            let is_charging =
                status == 2 || status == 3 || status == 6 || status == 7 || status == 8;
            return BatteryInfo {
                percentage: pct,
                is_charging,
            };
        }
    }

    BatteryInfo {
        percentage: 100,
        is_charging: true,
    }
}

#[cfg(target_os = "windows")]
extern "system" {
    fn SetWindowDisplayAffinity(hwnd: *mut std::ffi::c_void, dw_affinity: u32) -> i32;
}

#[cfg(target_os = "windows")]
const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
#[cfg(target_os = "windows")]
const WDA_MONITOR: u32 = 0x00000001;

pub fn apply_screen_share_protection(notch: &tauri::WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        if let Ok(hwnd) = notch.hwnd() {
            unsafe {
                // WDA_EXCLUDEFROMCAPTURE (0x11): The window is excluded completely from all screen sharing,
                // recording, OBS, Discord, Teams, Zoom, Snipping Tool, and Windows Graphics Capture.
                let res = SetWindowDisplayAffinity(hwnd.0, WDA_EXCLUDEFROMCAPTURE);
                if res == 0 {
                    // Fallback to WDA_MONITOR (0x01) if on an older Windows 10 build (< 2004)
                    let _ = SetWindowDisplayAffinity(hwnd.0, WDA_MONITOR);
                }
            }
        }
    }
}

#[tauri::command]
fn restore_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.unminimize();
        let _ = main.set_focus();
    }
    if let Some(notch) = app.get_webview_window("notch") {
        let _ = notch.hide();
    }
    Ok(())
}

#[tauri::command]
fn set_notch_expanded(app: tauri::AppHandle, expanded: bool) -> Result<(), String> {
    if let Some(notch) = app.get_webview_window("notch") {
        if let Ok(Some(monitor)) = notch.primary_monitor() {
            let screen_width = monitor.size().width;
            let scale_factor = monitor.scale_factor();
            let (target_w, target_h) = if expanded {
                (480.0, 260.0)
            } else {
                (260.0, 42.0)
            };
            let phys_w = (target_w * scale_factor) as i32;
            let phys_h = (target_h * scale_factor) as i32;
            let x = (screen_width as i32 - phys_w) / 2;

            let _ = notch.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: phys_w as u32,
                height: phys_h as u32,
            }));
            let _ = notch.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x,
                y: 0,
            }));
            apply_screen_share_protection(&notch);
        }
    }
    Ok(())
}

#[tauri::command]
fn hide_to_notch(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.hide();
    }
    if let Some(notch) = app.get_webview_window("notch") {
        let _ = set_notch_expanded(app.clone(), false);
        apply_screen_share_protection(&notch);
        let _ = notch.show();
        let _ = notch.set_always_on_top(true);
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                if let Some(icon) = app.default_window_icon() {
                    let _ = window.set_icon(icon.clone());
                }
            }

            // Position notch window at top-middle of primary screen
            if let Some(notch) = app.get_webview_window("notch") {
                apply_screen_share_protection(&notch);
                if let Ok(Some(monitor)) = notch.primary_monitor() {
                    let screen_width = monitor.size().width;
                    let scale_factor = monitor.scale_factor();
                    let notch_phys_width = (260.0 * scale_factor) as i32;
                    let x = (screen_width as i32 - notch_phys_width) / 2;
                    let _ = notch.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x,
                        y: 0,
                    }));
                }
            }

            // System Tray Setup
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

            let quit_i = MenuItem::with_id(app, "quit", "Quit Veronica", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Open Veronica", true, None::<&str>)?;
            let notch_i = MenuItem::with_id(app, "notch", "Show Dynamic Notch", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &notch_i, &quit_i])?;

            if let Some(icon) = app.default_window_icon() {
                let _tray = TrayIconBuilder::new()
                    .icon(icon.clone())
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.unminimize();
                                let _ = w.set_focus();
                            }
                            if let Some(n) = app.get_webview_window("notch") {
                                let _ = n.hide();
                            }
                        }
                        "notch" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.hide();
                            }
                            if let Some(n) = app.get_webview_window("notch") {
                                let _ = set_notch_expanded(app.clone(), false);
                                apply_screen_share_protection(&n);
                                let _ = n.show();
                                let _ = n.set_always_on_top(true);
                            }
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(w) = app.get_webview_window("main") {
                                if w.is_visible().unwrap_or(false) {
                                    let _ = w.hide();
                                    if let Some(n) = app.get_webview_window("notch") {
                                        let _ = set_notch_expanded(app.clone(), false);
                                        apply_screen_share_protection(&n);
                                        let _ = n.show();
                                        let _ = n.set_always_on_top(true);
                                    }
                                } else {
                                    let _ = w.show();
                                    let _ = w.unminimize();
                                    let _ = w.set_focus();
                                    if let Some(n) = app.get_webview_window("notch") {
                                        let _ = n.hide();
                                    }
                                }
                            }
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                    if let Some(notch) = window.app_handle().get_webview_window("notch") {
                        let _ = set_notch_expanded(window.app_handle().clone(), false);
                        apply_screen_share_protection(&notch);
                        let _ = notch.show();
                        let _ = notch.set_always_on_top(true);
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_installed_apps,
            rescan_apps,
            get_app_groups,
            save_app_group,
            rename_app_group,
            delete_app_group,
            add_app_to_group,
            remove_app_from_group,
            launch_app,
            close_app,
            get_battery_info,
            restore_main_window,
            hide_to_notch,
            set_notch_expanded
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
