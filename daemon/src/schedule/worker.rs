use std::time::Duration;
use tokio::time::sleep;
use super::db::ScheduleDb;

/// Background task that continuously checks for due reminders and meetings
pub async fn start_schedule_worker(db: ScheduleDb) {
    println!("   📅 [Schedule Worker]: Background scheduler engine initialized.");

    loop {
        sleep(Duration::from_secs(3)).await;

        match db.get_due_schedules() {
            Ok(due_items) => {
                for item in due_items {
                    println!("\n=======================================================");
                    println!("   ⏰ [VERONICA ALERT]: {}", item.title.to_uppercase());
                    println!("   📝 Type:        {}", item.schedule_type);
                    println!("   🕒 Scheduled:   {}", item.target_time);
                    println!("   📋 Description: {}", item.description);
                    println!("=======================================================\n");

                    // 1. Show Native Windows Notification
                    let notif_body = format!("{}\n{}", item.title, item.description);
                    let _ = show_os_notification(&item.title, &notif_body);

                    // 2. Mark triggered in DB (or reschedule if recurring)
                    if let Err(e) = db.mark_triggered(&item.id) {
                        eprintln!("   ⚠️ Failed to update schedule status: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("   ⚠️ [Schedule Worker Error]: {}", e);
            }
        }
    }
}

/// Dispatches a lightweight native notification on the host OS
pub fn show_os_notification(title: &str, body: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; \
             $ni = New-Object System.Windows.Forms.NotifyIcon; \
             $ni.Icon = [System.Drawing.SystemIcons]::Information; \
             $ni.Visible = $true; \
             $ni.ShowBalloonTip(7000, '{}', '{}', [System.Windows.Forms.ToolTipIcon]::Info); \
             Start-Sleep -Milliseconds 600; \
             $ni.Dispose();",
            title.replace('\'', "''"),
            body.replace('\'', "''")
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
            .spawn();
    }
    Ok(())
}
