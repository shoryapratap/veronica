use tokio::process::Command;

/// Launches a Windows application asynchronously by name.
/// Example targets: "notepad", "calc", "spotify", "chrome"
pub async fn launch_app(target: &str) -> Result<String, String> {
    // 🪟 Windows implementation:
    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
        .args(["/c", "start", "", target])
        .status()
        .await;

    // 🍎 macOS implementation:
    #[cfg(target_os = "macos")]
    let status = Command::new("open")
        .args(["-a", target])
        .status()
        .await;

    // 🐧 Linux implementation:
    #[cfg(target_os = "linux")]
    let status = Command::new("xdg-open")
        .arg(target)
        .status()
        .await;

    match status {
        Ok(exit_code) if exit_code.success() => {
            Ok(format!("Successfully launched: {}", target))
        }
        Ok(exit_code) => {
            Err(format!("Failed to launch {} (Exit code: {:?})", target, exit_code))
        }
        Err(e) => {
            Err(format!("Error executing command for {}: {}", target, e))
        }
    }
}
