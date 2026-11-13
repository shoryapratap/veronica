pub mod protocol;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;
use sysinfo::System;

use crate::ai::live::LiveSessionHandle;
use protocol::{ClientMessage, DaemonMessage};

/// Starts the loopback WebSocket server on 127.0.0.1:port for GUI connections
pub async fn start_server(
    port: u16,
    session: LiveSessionHandle,
    broadcast_tx: broadcast::Sender<DaemonMessage>,
) -> Result<(), String> {
    let addr: SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .map_err(|e| format!("Invalid socket address: {}", e))?;

    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind WebSocket server on {}: {}", addr, e))?;

    println!("🌐 [GUI Server] Listening on ws://{}", addr);

    // 1. Background Hardware Telemetry Broadcast Task (1 Hz)
    let tele_tx = broadcast_tx.clone();
    tokio::spawn(async move {
        let mut sys = System::new();
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            sys.refresh_cpu_all();
            sys.refresh_memory();

            let cpu_usage = sys.global_cpu_usage();
            let ram_used_mb = sys.used_memory() / (1024 * 1024);
            let ram_total_mb = sys.total_memory() / (1024 * 1024);

            let msg = DaemonMessage::Telemetry {
                cpu_usage,
                ram_used_mb,
                ram_total_mb,
            };

            let _ = tele_tx.send(msg);
        }
    });

    // 2. Accept incoming GUI client connections
    let session_arc = Arc::new(session);
    let broadcast_tx_arc = Arc::new(broadcast_tx);

    tokio::spawn(async move {
        while let Ok((stream, client_addr)) = listener.accept().await {
            let session_clone = Arc::clone(&session_arc);
            let broadcast_tx_clone = Arc::clone(&broadcast_tx_arc);

            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, client_addr, session_clone, broadcast_tx_clone).await {
                    eprintln!("⚠️ [GUI Client] Disconnected ({}): {}", client_addr, e);
                }
            });
        }
    });

    Ok(())
}

async fn handle_connection(
    raw_stream: TcpStream,
    addr: SocketAddr,
    session: Arc<LiveSessionHandle>,
    broadcast_tx: Arc<broadcast::Sender<DaemonMessage>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔌 [GUI Client] Connected from {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream).await?;
    let (mut ws_sink, mut ws_stream) = ws_stream.split();

    // Send initial handshake
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let welcome = DaemonMessage::Heartbeat {
        timestamp: now,
        status: "ONLINE".to_string(),
    };
    if let Ok(txt) = serde_json::to_string(&welcome) {
        let _ = ws_sink.send(Message::Text(txt.into())).await;
    }

    // Subscribe to server broadcast channel
    let mut rx = broadcast_tx.subscribe();

    // Outbound loop: forwards Daemon broadcasts to the GUI client
    let mut outbound_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json_str) = serde_json::to_string(&msg) {
                if ws_sink.send(Message::Text(json_str.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Inbound loop: receives commands and typed text from GUI client
    let mut inbound_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_stream.next().await {
            if let Message::Text(text) = msg {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    match client_msg {
                        ClientMessage::Ping => {
                            // Handled automatically
                        }
                        ClientMessage::TextMessage { text } => {
                            let trimmed = text.trim();
                            if !trimmed.is_empty() {
                                if trimmed.eq_ignore_ascii_case("/screen") || trimmed.eq_ignore_ascii_case("/look") {
                                    println!("   📸 [GUI Vision]: Manual screen capture triggered by user");
                                    if let Err(e) = session.capture_and_send_screen().await {
                                        eprintln!("   ❌ Failed to send screen: {}", e);
                                    } else {
                                        let _ = session.send_text("I have shared my screen with you. Please describe what is visible on my screen.").await;
                                    }
                                } else if let Some(target) = trimmed.strip_prefix("launch ") {
                                    let target_app = target.trim();
                                    println!("\rYou (GUI) > {}", trimmed);
                                    println!("   🚀 [Direct App Launch]: Opening '{}' immediately", target_app);
                                    let launch_res = crate::system::platform::windows::app::open_app(target_app).await;
                                    match launch_res {
                                        Ok(msg) => {
                                            println!("   ✅ {}", msg);
                                            let _ = broadcast_tx.send(DaemonMessage::Transcript {
                                                speaker: "VERONICA".to_string(),
                                                text: format!("Launching {}.", target_app),
                                                is_final: true,
                                            });
                                            let _ = session.send_text(&format!("I just launched '{}'. Acknowledge briefly as Veronica.", target_app)).await;
                                        }
                                        Err(e) => {
                                            eprintln!("   ❌ Failed to launch '{}': {}", target_app, e);
                                            let _ = broadcast_tx.send(DaemonMessage::Transcript {
                                                speaker: "VERONICA".to_string(),
                                                text: format!("Failed to launch {}: {}", target_app, e),
                                                is_final: true,
                                            });
                                        }
                                    }
                                } else if let Some(target) = trimmed.strip_prefix("close ").or_else(|| trimmed.strip_prefix("kill ")) {
                                    let target_app = target.trim();
                                    println!("\rYou (GUI) > {}", trimmed);
                                    println!("   🛑 [Direct App Terminate]: Closing '{}' immediately", target_app);
                                    let close_res = crate::system::platform::windows::app::close_app(target_app).await;
                                    match close_res {
                                        Ok(msg) => {
                                            println!("   ✅ {}", msg);
                                            let _ = broadcast_tx.send(DaemonMessage::Transcript {
                                                speaker: "VERONICA".to_string(),
                                                text: format!("Closed {}.", target_app),
                                                is_final: true,
                                            });
                                            let _ = session.send_text(&format!("I just closed '{}'. Acknowledge briefly as Veronica.", target_app)).await;
                                        }
                                        Err(e) => {
                                            eprintln!("   ❌ Failed to close '{}': {}", target_app, e);
                                            let _ = broadcast_tx.send(DaemonMessage::Transcript {
                                                speaker: "VERONICA".to_string(),
                                                text: format!("Failed to close {}: {}", target_app, e),
                                                is_final: true,
                                            });
                                        }
                                    }
                                } else {
                                    println!("\rYou (GUI) > {}", trimmed);
                                    let _ = session.send_text(trimmed).await;
                                }
                            }
                        }
                        ClientMessage::Command { action } => {
                            println!("   ⚡ [GUI Command]: {}", action);
                            if action == "capture_screen" || action == "vision" {
                                println!("   📸 [GUI Vision]: Screen snapshot triggered");
                                if let Err(e) = session.capture_and_send_screen().await {
                                    eprintln!("   ❌ Failed to send screen: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    // Wait until either direction closes
    tokio::select! {
        _ = (&mut outbound_task) => inbound_task.abort(),
        _ = (&mut inbound_task) => outbound_task.abort(),
    }

    println!("🔌 [GUI Client] Closed connection from {}", addr);
    Ok(())
}
