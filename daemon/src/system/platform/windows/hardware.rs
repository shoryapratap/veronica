use std::env;
use std::fs;
use std::path::PathBuf;
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sysinfo::{Disks, System};

// ============================================================================
// 1. DATA MODELS & TELEMETRY STRUCTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiInterfaceStatus {
    pub connected: bool,
    pub ssid: String,
    pub signal_percent: u32,
    pub radio_type: String,
    pub state: String,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetworkInfo {
    pub ssid: String,
    pub signal_percent: u32,
    pub authentication: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDevice {
    pub name: String,
    pub present: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub charge_percent: u32,
    pub is_charging: bool,
    pub status_text: String,
    pub estimated_runtime_minutes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskTelemetry {
    pub mount_point: String,
    pub total_gb: f64,
    pub free_gb: f64,
    pub used_gb: f64,
    pub used_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTelemetry {
    pub cpu_usage_percent: f32,
    pub cpu_core_count: usize,
    pub ram_total_mb: u64,
    pub ram_used_mb: u64,
    pub ram_free_mb: u64,
    pub ram_used_percent: f64,
    pub disks: Vec<DiskTelemetry>,
    pub uptime_formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySensorStatus {
    pub microphone_in_use: bool,
    pub microphone_active_apps: Vec<String>,
    pub camera_in_use: bool,
    pub camera_active_apps: Vec<String>,
}

// ============================================================================
// HELPER: SILENT POWERSHELL EXECUTION
// ============================================================================

async fn run_powershell_cmd(cmd: &str) -> Result<String, String> {
    let output = tokio::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", cmd])
        .output()
        .await
        .map_err(|e| format!("PowerShell execution failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() && stdout.is_empty() {
        return Err(stderr);
    }
    Ok(stdout)
}

// ============================================================================
// 1. NETWORK (Wi-Fi, Hotspot, Ping)
// ============================================================================

/// Returns current Wi-Fi connection status, active SSID, signal strength, and IP
pub async fn get_network_status() -> Result<WifiInterfaceStatus, String> {
    let output = run_powershell_cmd("netsh wlan show interfaces").await?;
    
    let mut connected = false;
    let mut ssid = String::new();
    let mut signal_percent = 0u32;
    let mut radio_type = String::new();
    let mut state = String::from("disconnected");

    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().to_lowercase();
            let val = parts[1].trim();
            if key == "state" {
                state = val.to_string();
                if val.eq_ignore_ascii_case("connected") {
                    connected = true;
                }
            } else if key == "ssid" && ssid.is_empty() {
                ssid = val.to_string();
            } else if key == "signal" {
                let clean_pct = val.replace('%', "").trim().to_string();
                signal_percent = clean_pct.parse::<u32>().unwrap_or(0);
            } else if key == "radio type" {
                radio_type = val.to_string();
            }
        }
    }

    // Fetch IP address
    let ip_cmd = "(Get-NetIPAddress -InterfaceAlias '*Wi-Fi*' -AddressFamily IPv4 -ErrorAction SilentlyContinue).IPAddress";
    let ip_output = run_powershell_cmd(ip_cmd).await.ok();
    let ip_address = ip_output.and_then(|ip| {
        let first = ip.lines().next().unwrap_or("").trim().to_string();
        if first.is_empty() { None } else { Some(first) }
    });

    Ok(WifiInterfaceStatus {
        connected,
        ssid,
        signal_percent,
        radio_type,
        state,
        ip_address,
    })
}

/// Scans and lists nearby visible Wi-Fi networks
pub async fn scan_wifi_networks() -> Result<Vec<WifiNetworkInfo>, String> {
    let output = run_powershell_cmd("netsh wlan show networks mode=Bssid").await?;
    let mut networks = Vec::new();
    let mut current_ssid = String::new();
    let mut current_auth = String::new();
    let mut current_signal = 0u32;

    for line in output.lines() {
        let line_trim = line.trim();
        if line_trim.starts_with("SSID") && line_trim.contains(':') {
            if !current_ssid.is_empty() {
                networks.push(WifiNetworkInfo {
                    ssid: current_ssid.clone(),
                    signal_percent: current_signal,
                    authentication: current_auth.clone(),
                });
            }
            if let Some(pos) = line_trim.find(':') {
                current_ssid = line_trim[pos + 1..].trim().to_string();
            }
            current_auth.clear();
            current_signal = 0;
        } else if line_trim.starts_with("Authentication") {
            if let Some(pos) = line_trim.find(':') {
                current_auth = line_trim[pos + 1..].trim().to_string();
            }
        } else if line_trim.starts_with("Signal") {
            if let Some(pos) = line_trim.find(':') {
                let val = line_trim[pos + 1..].replace('%', "").trim().to_string();
                current_signal = val.parse::<u32>().unwrap_or(0);
            }
        }
    }

    if !current_ssid.is_empty() {
        networks.push(WifiNetworkInfo {
            ssid: current_ssid,
            signal_percent: current_signal,
            authentication: current_auth,
        });
    }

    Ok(networks)
}

/// Connects to a known/configured Wi-Fi profile
pub async fn connect_wifi(ssid: &str) -> Result<String, String> {
    let cmd = format!("netsh wlan connect name=\"{}\"", ssid);
    run_powershell_cmd(&cmd).await
}

/// Disconnects from the current Wi-Fi network
pub async fn disconnect_wifi() -> Result<String, String> {
    run_powershell_cmd("netsh wlan disconnect").await
}

/// Toggles Wi-Fi adapter On or Off
pub async fn toggle_wifi(enable: bool) -> Result<String, String> {
    let action = if enable { "admin=enable" } else { "admin=disable" };
    let cmd = format!("netsh interface set interface \"Wi-Fi\" {}", action);
    run_powershell_cmd(&cmd).await
}

/// Toggles Windows Mobile Hotspot service
pub async fn toggle_hotspot(enable: bool) -> Result<String, String> {
    let cmd = if enable {
        "Set-Service -Name icssvc -StartupType Manual; Start-Service icssvc"
    } else {
        "Stop-Service icssvc"
    };
    run_powershell_cmd(cmd).await
}

/// Pings Google DNS (8.8.8.8) to check actual internet connectivity and latency
pub async fn check_internet_ping() -> Result<serde_json::Value, String> {
    let cmd = "Test-Connection -ComputerName 8.8.8.8 -Count 1 -Quiet";
    let reachable = run_powershell_cmd(cmd).await.map(|s| s.eq_ignore_ascii_case("True")).unwrap_or(false);

    let ping_cmd = "(Test-Connection -ComputerName 8.8.8.8 -Count 1).ResponseTime";
    let latency_ms = run_powershell_cmd(ping_cmd).await.ok().and_then(|s| s.trim().parse::<u32>().ok()).unwrap_or(0);

    Ok(json!({
        "internet_reachable": reachable,
        "latency_ms": latency_ms,
        "dns_target": "8.8.8.8"
    }))
}

// ============================================================================
// 2. BLUETOOTH
// ============================================================================

/// Returns status of Bluetooth adapter
pub async fn get_bluetooth_status() -> Result<serde_json::Value, String> {
    let cmd = "Get-PnpDevice -Class Bluetooth | Where-Object { $_.FriendlyName -like '*Adapter*' -or $_.FriendlyName -like '*Bluetooth Radio*' } | Select-Object FriendlyName, Status, Present | ConvertTo-Json";
    let output = run_powershell_cmd(cmd).await?;
    let json_val: serde_json::Value = serde_json::from_str(&output).unwrap_or(json!({}));
    Ok(json_val)
}

/// Lists paired and discovered Bluetooth devices
pub async fn list_bluetooth_devices() -> Result<Vec<BluetoothDevice>, String> {
    let cmd = "Get-PnpDevice -Class Bluetooth | Where-Object { $_.FriendlyName -notlike '*Enumerator*' -and $_.FriendlyName -notlike '*Protocol*' -and $_.FriendlyName -notlike '*Generic*' } | Select-Object FriendlyName, Status, Present | ConvertTo-Json";
    let output = run_powershell_cmd(cmd).await?;
    
    if output.is_empty() {
        return Ok(Vec::new());
    }

    if let Ok(devices) = serde_json::from_str::<Vec<serde_json::Value>>(&output) {
        let list = devices.into_iter().map(|d| BluetoothDevice {
            name: d.get("FriendlyName").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            present: d.get("Present").and_then(|v| v.as_bool()).unwrap_or(false),
            status: d.get("Status").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
        }).collect();
        return Ok(list);
    }

    Ok(Vec::new())
}

/// Enables or disables Bluetooth adapter
pub async fn toggle_bluetooth(enable: bool) -> Result<String, String> {
    let cmd = if enable {
        "Get-PnpDevice -Class Bluetooth | Where-Object { $_.FriendlyName -like '*Adapter*' } | Enable-PnpDevice -Confirm:$false"
    } else {
        "Get-PnpDevice -Class Bluetooth | Where-Object { $_.FriendlyName -like '*Adapter*' } | Disable-PnpDevice -Confirm:$false"
    };
    run_powershell_cmd(cmd).await
}

// ============================================================================
// 3. BATTERY & POWER
// ============================================================================

/// Returns battery charge %, charging state, and estimated runtime
pub async fn get_battery_status() -> Result<BatteryInfo, String> {
    let cmd = "Get-CimInstance -ClassName Win32_Battery | Select-Object EstimatedChargeRemaining, BatteryStatus, EstimatedRunTime | ConvertTo-Json";
    let output = run_powershell_cmd(cmd).await?;
    
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&output) {
        let charge_percent = val.get("EstimatedChargeRemaining").and_then(|v| v.as_u64()).unwrap_or(100) as u32;
        let b_status = val.get("BatteryStatus").and_then(|v| v.as_u64()).unwrap_or(2);
        
        let (is_charging, status_text) = match b_status {
            1 => (false, "Discharging".to_string()),
            2 => (true, "Plugged In / AC Power".to_string()),
            3 => (true, "Fully Charged".to_string()),
            4 => (false, "Low Battery".to_string()),
            5 => (false, "Critical Battery".to_string()),
            6 => (true, "Charging".to_string()),
            _ => (true, "AC Connected".to_string()),
        };

        let runtime = val.get("EstimatedRunTime").and_then(|v| v.as_u64()).filter(|&m| m < 70000000).map(|m| m as u32);

        return Ok(BatteryInfo {
            charge_percent,
            is_charging,
            status_text,
            estimated_runtime_minutes: runtime,
        });
    }

    Err("No battery detected (Desktop PC or hardware AC power)".to_string())
}

/// Sets active power plan (e.g. "balanced", "high_performance", "power_saver")
pub async fn set_power_plan(plan: &str) -> Result<String, String> {
    let guid = match plan.to_lowercase().as_str() {
        "high" | "high_performance" | "performance" => "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c",
        "power_saver" | "saver" => "a1841308-3541-4fab-bc81-f71556f20b4a",
        _ => "381b4222-f694-41f0-9685-ff5bb260df2e", // Balanced
    };
    let cmd = format!("powercfg /setactive {}", guid);
    run_powershell_cmd(&cmd).await
}

/// Executes power actions: lock, sleep, hibernate, restart, shutdown
pub async fn power_action(action: &str) -> Result<String, String> {
    match action.to_lowercase().as_str() {
        "lock" => {
            run_powershell_cmd("rundll32.exe user32.dll,LockWorkStation").await?;
            Ok("Workstation locked successfully".to_string())
        }
        "sleep" => {
            run_powershell_cmd("rundll32.exe powrprof.dll,SetSuspendState 0,1,0").await?;
            Ok("System entering sleep state".to_string())
        }
        "hibernate" => {
            run_powershell_cmd("shutdown /h").await?;
            Ok("System entering hibernation".to_string())
        }
        "restart" => {
            run_powershell_cmd("shutdown /r /t 0").await?;
            Ok("System reboot initiated".to_string())
        }
        "shutdown" => {
            run_powershell_cmd("shutdown /s /t 0").await?;
            Ok("System shutdown initiated".to_string())
        }
        other => Err(format!("Unknown power action '{}'. Supported: lock, sleep, hibernate, restart, shutdown", other))
    }
}

// ============================================================================
// 4. SOUND & AUDIO
// ============================================================================

/// Toggles master mute / unmute
pub async fn toggle_mute() -> Result<String, String> {
    let cmd = "& { (New-Object -ComObject Wscript.Shell).SendKeys([char]173) }";
    run_powershell_cmd(cmd).await?;
    Ok("Toggled audio mute state".to_string())
}

/// Adjusts master volume up or down by step (+10%, -10%, etc.)
pub async fn adjust_volume(delta: i32) -> Result<String, String> {
    let steps = (delta.abs() / 2).max(1);
    let key_char = if delta > 0 { 175 } else { 174 }; // 175 = Vol Up, 174 = Vol Down
    let mut cmd = String::from("& { $w = New-Object -ComObject Wscript.Shell; ");
    for _ in 0..steps {
        cmd.push_str(&format!("$w.SendKeys([char]{}); ", key_char));
    }
    cmd.push('}');
    run_powershell_cmd(&cmd).await?;
    Ok(format!("Adjusted volume by {} steps", steps))
}

/// Sets volume to an exact percentage (0-100%) by resetting to 0 and stepping up
pub async fn set_volume(level: u32) -> Result<String, String> {
    let target = level.min(100);
    let steps_up = target / 2;
    let mut cmd = String::from("& { $w = New-Object -ComObject Wscript.Shell; ");
    // Reset to 0 with 50 down steps
    for _ in 0..50 {
        cmd.push_str("$w.SendKeys([char]174); ");
    }
    // Step up to target
    for _ in 0..steps_up {
        cmd.push_str("$w.SendKeys([char]175); ");
    }
    cmd.push('}');
    run_powershell_cmd(&cmd).await?;
    Ok(format!("Master volume set to approx {}%", target))
}

// ============================================================================
// 5. BRIGHTNESS & DISPLAY
// ============================================================================

/// Returns current screen brightness percentage (0-100)
pub async fn get_brightness() -> Result<u32, String> {
    let cmd = "(Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness -ErrorAction SilentlyContinue).CurrentBrightness";
    let output = run_powershell_cmd(cmd).await?;
    output.trim().parse::<u32>().map_err(|_| "Brightness sensor unavailable (external monitor or desktop GPU)".to_string())
}

/// Sets display brightness (0-100)
pub async fn set_brightness(level: u32) -> Result<String, String> {
    let target = level.min(100);
    let cmd = format!(
        "Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods | Invoke-CimMethod -MethodName WmiSetBrightness -Arguments @{{Timeout=0; Brightness={}}}",
        target
    );
    run_powershell_cmd(&cmd).await?;
    Ok(format!("Display brightness set to {}%", target))
}

/// Adjusts brightness up or down by delta (+10, -10)
pub async fn adjust_brightness(delta: i32) -> Result<String, String> {
    let current = get_brightness().await.unwrap_or(50);
    let new_level = (current as i32 + delta).clamp(0, 100) as u32;
    set_brightness(new_level).await
}

/// Changes multi-display topology (internal, clone, extend, external)
pub async fn set_display_mode(mode: &str) -> Result<String, String> {
    let flag = match mode.to_lowercase().as_str() {
        "clone" | "duplicate" => "/clone",
        "extend" => "/extend",
        "external" | "second" => "/external",
        _ => "/internal", // PC screen only
    };
    run_powershell_cmd(&format!("displayswitch.exe {}", flag)).await?;
    Ok(format!("Display switched to mode: {}", mode))
}

/// Turns off monitor display immediately without sleeping the computer
pub async fn turn_off_display() -> Result<String, String> {
    let cmd = "powershell -NoProfile -Command \"(Add-Type -MemberDefinition '[DllImport(\\\"user32.dll\\\")] public static extern int SendMessage(int hWnd, int hMsg, int wParam, int lParam);' -Name 'W' -Namespace 'M' -PassThru)::SendMessage(0xffff, 0x0112, 0xF170, 2)\"";
    run_powershell_cmd(cmd).await?;
    Ok("Monitors entered low-power standby".to_string())
}

// ============================================================================
// 6. SCREEN (Screenshots & Recording)
// ============================================================================

/// Captures primary screen and saves JPEG to Pictures/Screenshots with timestamp
pub async fn capture_screenshot(custom_path: Option<String>) -> Result<serde_json::Value, String> {
    let bytes = crate::vision::capture_primary_screen()?;

    let save_path = match custom_path {
        Some(p) => PathBuf::from(p),
        None => {
            let user_profile = env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
            let dir = PathBuf::from(user_profile).join("Pictures").join("Screenshots");
            let _ = fs::create_dir_all(&dir);
            let filename = format!("Screenshot_{}.jpg", Local::now().format("%Y%m%d_%H%M%S"));
            dir.join(filename)
        }
    };

    if let Some(parent) = save_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    fs::write(&save_path, &bytes).map_err(|e| format!("Failed to save screenshot: {}", e))?;

    Ok(json!({
        "status": "success",
        "saved_path": save_path.to_string_lossy(),
        "size_kb": bytes.len() / 1024
    }))
}

/// Toggles Windows built-in screen recording (Xbox Game Bar Win+Alt+R)
pub async fn toggle_screen_recording() -> Result<String, String> {
    let cmd = "& { $wshell = New-Object -ComObject WScript.Shell; $wshell.SendKeys('%^{r}') }";
    run_powershell_cmd(cmd).await?;
    Ok("Sent Win+Alt+R shortcut to toggle screen recording".to_string())
}

// ============================================================================
// 7. SYSTEM TELEMETRY (CPU, RAM, Disks, Uptime)
// ============================================================================

/// Returns real-time CPU, RAM, Disk storage, and uptime telemetry
pub fn get_system_telemetry() -> SystemTelemetry {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_usage_percent = sys.global_cpu_usage();
    let cpu_core_count = sys.cpus().len();

    let ram_total_mb = sys.total_memory() / 1024 / 1024;
    let ram_used_mb = sys.used_memory() / 1024 / 1024;
    let ram_free_mb = sys.free_memory() / 1024 / 1024;
    let ram_used_percent = if ram_total_mb > 0 {
        ((ram_used_mb as f64 / ram_total_mb as f64) * 100.0 * 10.0).round() / 10.0
    } else {
        0.0
    };

    let disks_info = Disks::new_with_refreshed_list();
    let mut disks = Vec::new();
    for d in &disks_info {
        let total = d.total_space() as f64 / 1024.0 / 1024.0 / 1024.0;
        let free = d.available_space() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used = total - free;
        let used_pct = if total > 0.0 { ((used / total) * 100.0 * 10.0).round() / 10.0 } else { 0.0 };
        disks.push(DiskTelemetry {
            mount_point: d.mount_point().to_string_lossy().to_string(),
            total_gb: (total * 10.0).round() / 10.0,
            free_gb: (free * 10.0).round() / 10.0,
            used_gb: (used * 10.0).round() / 10.0,
            used_percent: used_pct,
        });
    }

    let uptime_sec = System::uptime();
    let hours = uptime_sec / 3600;
    let minutes = (uptime_sec % 3600) / 60;
    let uptime_formatted = format!("{}h {}m", hours, minutes);

    SystemTelemetry {
        cpu_usage_percent,
        cpu_core_count,
        ram_total_mb,
        ram_used_mb,
        ram_free_mb,
        ram_used_percent,
        disks,
        uptime_formatted,
    }
}

// ============================================================================
// 8. FOCUS ASSIST / DO NOT DISTURB
// ============================================================================

/// Toggles Windows notification focus assist / do not disturb mode
pub async fn set_focus_assist(enable: bool) -> Result<String, String> {
    // 0 = Muted / Do Not Disturb, 1 = Notifications enabled
    let val = if enable { 0 } else { 1 };
    let cmd = format!(
        "Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\PushNotifications' -Name 'ToastEnabled' -Value {} -Type DWord -Force",
        val
    );
    run_powershell_cmd(&cmd).await?;
    let status = if enable { "enabled (notifications silenced)" } else { "disabled (notifications active)" };
    Ok(format!("Do Not Disturb is now {}", status))
}

// ============================================================================
// 9. PERIPHERALS & USB
// ============================================================================

/// Lists connected USB drives and removable storage
pub async fn list_usb_devices() -> Result<serde_json::Value, String> {
    let cmd = "Get-CimInstance Win32_DiskDrive | Where-Object { $_.InterfaceType -eq 'USB' } | Select-Object Model, InterfaceType, Size | ConvertTo-Json";
    let output = run_powershell_cmd(cmd).await?;
    let res: serde_json::Value = serde_json::from_str(&output).unwrap_or(json!([]));
    Ok(res)
}

/// Safely ejects a removable USB drive (e.g. "E:")
pub async fn eject_drive(drive_letter: &str) -> Result<String, String> {
    let letter = drive_letter.replace(':', "").trim().to_uppercase();
    let cmd = format!("(New-Object -ComObject Shell.Application).Namespace(17).ParseName('{}:').InvokeVerb('Eject')", letter);
    run_powershell_cmd(&cmd).await?;
    Ok(format!("Eject signal sent for drive {}:", letter))
}

// ============================================================================
// 10. PRIVACY SENSORS (Active Camera & Mic In-Use Detection)
// ============================================================================

/// Checks Windows CapabilityAccessManager to detect active camera and microphone use
pub async fn check_privacy_sensors() -> Result<PrivacySensorStatus, String> {
    let mic_cmd = "Get-ChildItem -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\microphone\\NonPackaged' -ErrorAction SilentlyContinue | ForEach-Object { $stop = (Get-ItemProperty -Path $_.PSPath).LastUsedTimeStop; if ($stop -eq 0) { $_.PSChildName } }";
    let cam_cmd = "Get-ChildItem -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\webcam\\NonPackaged' -ErrorAction SilentlyContinue | ForEach-Object { $stop = (Get-ItemProperty -Path $_.PSPath).LastUsedTimeStop; if ($stop -eq 0) { $_.PSChildName } }";

    let mic_out = run_powershell_cmd(mic_cmd).await.unwrap_or_default();
    let cam_out = run_powershell_cmd(cam_cmd).await.unwrap_or_default();

    let mic_apps: Vec<String> = mic_out.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let cam_apps: Vec<String> = cam_out.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

    Ok(PrivacySensorStatus {
        microphone_in_use: !mic_apps.is_empty(),
        microphone_active_apps: mic_apps,
        camera_in_use: !cam_apps.is_empty(),
        camera_active_apps: cam_apps,
    })
}

// ============================================================================
// 11. MASTER FUNCTIONS (2-Step JIT Architecture for Gemini Live)
// ============================================================================

/// Master Schema for hardware features
pub fn master_schema(function_name: &str) -> serde_json::Value {
    match function_name {
        "network" => json!({
            "type": "OBJECT",
            "properties": {
                "action": {
                    "type": "STRING",
                    "description": "Action: 'status', 'scan', 'connect', 'disconnect', 'toggle_wifi', 'toggle_hotspot', 'ping'"
                },
                "ssid": { "type": "STRING", "description": "Target Wi-Fi SSID for connection" },
                "enable": { "type": "BOOLEAN", "description": "True to enable, False to disable" }
            },
            "required": ["action"]
        }),
        "bluetooth" => json!({
            "type": "OBJECT",
            "properties": {
                "action": { "type": "STRING", "description": "Action: 'status', 'list_devices', 'toggle'" },
                "enable": { "type": "BOOLEAN", "description": "True to enable, False to disable" }
            },
            "required": ["action"]
        }),
        "power" => json!({
            "type": "OBJECT",
            "properties": {
                "action": {
                    "type": "STRING",
                    "description": "Action: 'battery_status', 'set_plan', 'lock', 'sleep', 'hibernate', 'restart', 'shutdown'"
                },
                "plan": { "type": "STRING", "description": "Power plan: 'balanced', 'performance', 'power_saver'" }
            },
            "required": ["action"]
        }),
        "audio" => json!({
            "type": "OBJECT",
            "properties": {
                "action": { "type": "STRING", "description": "Action: 'toggle_mute', 'set_volume', 'adjust_volume'" },
                "level": { "type": "INTEGER", "description": "Target volume percentage 0-100" },
                "delta": { "type": "INTEGER", "description": "Volume change (+10, -10, etc.)" }
            },
            "required": ["action"]
        }),
        "display" => json!({
            "type": "OBJECT",
            "properties": {
                "action": {
                    "type": "STRING",
                    "description": "Action: 'get_brightness', 'set_brightness', 'adjust_brightness', 'set_mode', 'turn_off_screen'"
                },
                "level": { "type": "INTEGER", "description": "Target brightness 0-100" },
                "delta": { "type": "INTEGER", "description": "Brightness change (+10, -10)" },
                "mode": { "type": "STRING", "description": "Display mode: 'internal', 'clone', 'extend', 'external'" }
            },
            "required": ["action"]
        }),
        "screen" => json!({
            "type": "OBJECT",
            "properties": {
                "action": { "type": "STRING", "description": "Action: 'screenshot', 'toggle_recording'" },
                "custom_path": { "type": "STRING", "description": "Optional custom file path to save screenshot" }
            },
            "required": ["action"]
        }),
        "telemetry" => json!({
            "type": "OBJECT",
            "properties": {
                "action": { "type": "STRING", "description": "Action: 'get_stats'" }
            },
            "required": ["action"]
        }),
        "focus_assist" => json!({
            "type": "OBJECT",
            "properties": {
                "action": { "type": "STRING", "description": "Action: 'enable', 'disable'" }
            },
            "required": ["action"]
        }),
        "peripherals" => json!({
            "type": "OBJECT",
            "properties": {
                "action": { "type": "STRING", "description": "Action: 'list_usb', 'eject'" },
                "drive_letter": { "type": "STRING", "description": "Drive letter to eject, e.g. 'E:'" }
            },
            "required": ["action"]
        }),
        "privacy" => json!({
            "type": "OBJECT",
            "properties": {
                "action": { "type": "STRING", "description": "Action: 'check_sensors'" }
            },
            "required": ["action"]
        }),
        other => json!({
            "error": format!("Unknown hardware feature '{}'", other)
        })
    }
}

/// Master Execute Dispatcher for hardware features
pub async fn master_execute(function_name: &str, data: &serde_json::Value) -> serde_json::Value {
    let action = data.get("action").and_then(|v| v.as_str()).unwrap_or("");

    match function_name {
        "network" => match action {
            "status" => match get_network_status().await {
                Ok(s) => json!({ "status": "success", "network": s }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            "scan" => match scan_wifi_networks().await {
                Ok(nets) => json!({ "status": "success", "networks": nets }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            "connect" => {
                let ssid = data.get("ssid").and_then(|v| v.as_str()).unwrap_or("");
                match connect_wifi(ssid).await {
                    Ok(msg) => json!({ "status": "success", "message": msg }),
                    Err(e) => json!({ "status": "error", "message": e }),
                }
            }
            "disconnect" => match disconnect_wifi().await {
                Ok(msg) => json!({ "status": "success", "message": msg }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            "toggle_wifi" => {
                let enable = data.get("enable").and_then(|v| v.as_bool()).unwrap_or(true);
                match toggle_wifi(enable).await {
                    Ok(msg) => json!({ "status": "success", "message": msg }),
                    Err(e) => json!({ "status": "error", "message": e }),
                }
            }
            "toggle_hotspot" => {
                let enable = data.get("enable").and_then(|v| v.as_bool()).unwrap_or(true);
                match toggle_hotspot(enable).await {
                    Ok(msg) => json!({ "status": "success", "message": msg }),
                    Err(e) => json!({ "status": "error", "message": e }),
                }
            }
            "ping" => match check_internet_ping().await {
                Ok(res) => json!({ "status": "success", "result": res }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            _ => json!({ "status": "error", "message": format!("Unknown network action: '{}'", action) }),
        },

        "bluetooth" => match action {
            "status" => match get_bluetooth_status().await {
                Ok(res) => json!({ "status": "success", "result": res }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            "list_devices" => match list_bluetooth_devices().await {
                Ok(devices) => json!({ "status": "success", "devices": devices }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            "toggle" => {
                let enable = data.get("enable").and_then(|v| v.as_bool()).unwrap_or(true);
                match toggle_bluetooth(enable).await {
                    Ok(msg) => json!({ "status": "success", "message": msg }),
                    Err(e) => json!({ "status": "error", "message": e }),
                }
            }
            _ => json!({ "status": "error", "message": format!("Unknown bluetooth action: '{}'", action) }),
        },

        "power" => match action {
            "battery_status" => match get_battery_status().await {
                Ok(b) => json!({ "status": "success", "battery": b }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            "set_plan" => {
                let plan = data.get("plan").and_then(|v| v.as_str()).unwrap_or("balanced");
                match set_power_plan(plan).await {
                    Ok(msg) => json!({ "status": "success", "message": msg }),
                    Err(e) => json!({ "status": "error", "message": e }),
                }
            }
            "lock" | "sleep" | "hibernate" | "restart" | "shutdown" => {
                match power_action(action).await {
                    Ok(msg) => json!({ "status": "success", "message": msg }),
                    Err(e) => json!({ "status": "error", "message": e }),
                }
            }
            _ => json!({ "status": "error", "message": format!("Unknown power action: '{}'", action) }),
        },

        "audio" => match action {
            "toggle_mute" => match toggle_mute().await {
                Ok(msg) => json!({ "status": "success", "message": msg }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            "adjust_volume" => {
                let delta = data.get("delta").and_then(|v| v.as_i64()).unwrap_or(10) as i32;
                match adjust_volume(delta).await {
                    Ok(msg) => json!({ "status": "success", "message": msg }),
                    Err(e) => json!({ "status": "error", "message": e }),
                }
            }
            "set_volume" => {
                let level = data.get("level").and_then(|v| v.as_u64()).unwrap_or(50) as u32;
                match set_volume(level).await {
                    Ok(msg) => json!({ "status": "success", "message": msg }),
                    Err(e) => json!({ "status": "error", "message": e }),
                }
            }
            _ => json!({ "status": "error", "message": format!("Unknown audio action: '{}'", action) }),
        },

        "display" => match action {
            "get_brightness" => match get_brightness().await {
                Ok(b) => json!({ "status": "success", "brightness_percent": b }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            "set_brightness" => {
                let level = data.get("level").and_then(|v| v.as_u64()).unwrap_or(70) as u32;
                match set_brightness(level).await {
                    Ok(msg) => json!({ "status": "success", "message": msg }),
                    Err(e) => json!({ "status": "error", "message": e }),
                }
            }
            "adjust_brightness" => {
                let delta = data.get("delta").and_then(|v| v.as_i64()).unwrap_or(10) as i32;
                match adjust_brightness(delta).await {
                    Ok(msg) => json!({ "status": "success", "message": msg }),
                    Err(e) => json!({ "status": "error", "message": e }),
                }
            }
            "set_mode" => {
                let mode = data.get("mode").and_then(|v| v.as_str()).unwrap_or("internal");
                match set_display_mode(mode).await {
                    Ok(msg) => json!({ "status": "success", "message": msg }),
                    Err(e) => json!({ "status": "error", "message": e }),
                }
            }
            "turn_off_screen" => match turn_off_display().await {
                Ok(msg) => json!({ "status": "success", "message": msg }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            _ => json!({ "status": "error", "message": format!("Unknown display action: '{}'", action) }),
        },

        "screen" => match action {
            "screenshot" => {
                let custom = data.get("custom_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                match capture_screenshot(custom).await {
                    Ok(res) => res,
                    Err(e) => json!({ "status": "error", "message": e }),
                }
            }
            "toggle_recording" => match toggle_screen_recording().await {
                Ok(msg) => json!({ "status": "success", "message": msg }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            _ => json!({ "status": "error", "message": format!("Unknown screen action: '{}'", action) }),
        },

        "telemetry" => {
            let telem = get_system_telemetry();
            json!({
                "status": "success",
                "telemetry": telem
            })
        }

        "focus_assist" => match action {
            "enable" => match set_focus_assist(true).await {
                Ok(msg) => json!({ "status": "success", "message": msg }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            "disable" => match set_focus_assist(false).await {
                Ok(msg) => json!({ "status": "success", "message": msg }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            _ => json!({ "status": "error", "message": format!("Unknown focus assist action: '{}'", action) }),
        },

        "peripherals" => match action {
            "list_usb" => match list_usb_devices().await {
                Ok(devs) => json!({ "status": "success", "usb_devices": devs }),
                Err(e) => json!({ "status": "error", "message": e }),
            },
            "eject" => {
                let letter = data.get("drive_letter").and_then(|v| v.as_str()).unwrap_or("");
                match eject_drive(letter).await {
                    Ok(msg) => json!({ "status": "success", "message": msg }),
                    Err(e) => json!({ "status": "error", "message": e }),
                }
            }
            _ => json!({ "status": "error", "message": format!("Unknown peripherals action: '{}'", action) }),
        },

        "privacy" => match check_privacy_sensors().await {
            Ok(sensors) => json!({ "status": "success", "privacy_sensors": sensors }),
            Err(e) => json!({ "status": "error", "message": e }),
        },

        other => json!({
            "status": "error",
            "message": format!("Unknown hardware feature: '{}'", other)
        }),
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_telemetry() {
        let telem = get_system_telemetry();
        println!("CPU Cores: {}, Total RAM: {} MB, Uptime: {}", telem.cpu_core_count, telem.ram_total_mb, telem.uptime_formatted);
        assert!(telem.ram_total_mb > 0);
        assert!(!telem.disks.is_empty());
    }

    #[tokio::test]
    async fn test_network_and_ping() {
        let net = get_network_status().await;
        println!("Network status: {:?}", net);
        assert!(net.is_ok());

        let ping = check_internet_ping().await;
        println!("Ping test: {:?}", ping);
        assert!(ping.is_ok());
    }

    #[tokio::test]
    async fn test_battery_query() {
        let battery = get_battery_status().await;
        println!("Battery status: {:?}", battery);
        // Note: Battery can be Ok or Err depending on if host is a laptop or desktop
    }

    #[tokio::test]
    async fn test_privacy_sensors() {
        let privacy = check_privacy_sensors().await;
        println!("Privacy sensors: {:?}", privacy);
        assert!(privacy.is_ok());
    }

    #[test]
    fn test_master_schema_definitions() {
        let network_schema = master_schema("network");
        assert_eq!(network_schema["type"], "OBJECT");

        let power_schema = master_schema("power");
        assert_eq!(power_schema["type"], "OBJECT");

        let audio_schema = master_schema("audio");
        assert_eq!(audio_schema["type"], "OBJECT");
    }
}
