mod server;

use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("========================================");
    println!(" Project Veronica: Local Headless Daemon");
    println!(" Status: INITIALIZING");
    println!(" Target Port: 127.0.0.1:8765");
    println!("========================================");

    // Initial background heartbeat loop
    let mut tick = 0;
    loop {
        tick += 1;
        println!("[Daemon Heartbeat #{}]: Running silently in background...", tick);
        sleep(Duration::from_secs(5)).await;
    }
}
