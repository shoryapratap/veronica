use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};

pub mod ai;
pub mod audio;
pub mod schedule;
mod server;
pub mod system;
pub mod vision;

#[tokio::main]
async fn main() {
    // 1. Load environment variables from .env
    dotenvy::dotenv().ok();

    // 2. Initialize PostgreSQL Schedule Engine (with description context)
    let schedule_engine = Arc::new(schedule::ScheduleEngine::new(None).await);

    // 2.1 Alert Queue: Background worker notifies AI live session & GUI when reminder is due
    let (alert_queue_tx, mut alert_queue_rx) = tokio::sync::mpsc::channel::<schedule::ScheduleItem>(32);

    // Spawn 24/7 background scheduler alert worker with auto-cleanup queue
    let worker_engine = Arc::clone(&schedule_engine);
    tokio::spawn(async move {
        worker_engine.start_background_worker(Some(alert_queue_tx)).await;
    });

    // 3. Initialize AI Tool Manager (handles all tool declarations and routing)
    let tool_manager = ai::ToolManager::new(Arc::clone(&schedule_engine));

    let api_key = std::env::var("GEMINI_API_KEY")
        .expect("❌ Error: GEMINI_API_KEY not found in daemon/.env file!");
    let live_model = std::env::var("GEMINI_LIVE_MODEL")
        .unwrap_or_else(|_| "gemini-2.5-flash-native-audio-latest".to_string());
    let voice_name = std::env::var("GEMINI_VOICE")
        .unwrap_or_else(|_| "Aoede".to_string());

    // 4. Initialize Streaming Audio Player (Low-latency in-memory playback)
    let audio_player = audio::StreamingAudioPlayer::new()
        .expect("❌ Failed to initialize streaming audio player");

    // 5. Detect system environment
    let env_info = system::detector::detect_environment();

    // 6. Connect to Gemini Live WebSocket
    let live_client = match ai::GeminiLiveClient::connect(&api_key, &live_model, &voice_name).await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("❌ [Gemini Live] Connection failed: {}", err);
            return;
        }
    };

    // 7. Initialize GUI Event Broadcast Channel
    let (broadcast_tx, _) = tokio::sync::broadcast::channel::<server::protocol::DaemonMessage>(128);

    // 8. Start full-duplex Live Session with GUI broadcasting & Tool Manager
    let session = live_client.start_duplex_session(
        audio_player.clone(),
        Some(broadcast_tx.clone()),
        tool_manager,
    );

    // 8.1 Hook Alert Queue to AI: when reminder is due, instruct Veronica to speak out to user
    let session_alert = session.clone();
    let broadcast_alert = broadcast_tx.clone();
    tokio::spawn(async move {
        while let Some(item) = alert_queue_rx.recv().await {
            println!("🔔 [AI Alert Queue]: Instructing Veronica to announce reminder: '{}'", item.title);

            let prompt = format!(
                "System Notice: It is now time for the user's scheduled task: '{}'. Detailed user context: '{}'. Please immediately speak to the user, reminding them about this warmly and concisely.",
                item.title, item.description
            );
            let _ = session_alert.send_text(&prompt).await;

            let _ = broadcast_alert.send(server::protocol::DaemonMessage::Transcript {
                speaker: "VERONICA".to_string(),
                text: format!("🔔 [Reminder Alert] {}: {}", item.title, item.description),
                is_final: true,
            });
        }
    });

    // 9. Start Localhost WebSocket Server for GUI (127.0.0.1:8765)
    if let Err(e) = server::start_server(8765, session.clone(), broadcast_tx.clone()).await {
        eprintln!("⚠️ Failed to bind GUI server on port 8765: {}", e);
    }

    // 10. Start Continuous Microphone Streaming with Low-Latency VAD
    let (mic_tx, mut mic_rx) = tokio::sync::mpsc::channel::<audio::mic::AudioStreamEvent>(128);
    let mic_streamer = audio::AudioStreamer::start(mic_tx);
    let has_mic = mic_streamer.is_ok();

    if has_mic {
        let session_mic = session.clone();
        tokio::spawn(async move {
            while let Some(event) = mic_rx.recv().await {
                match event {
                    audio::mic::AudioStreamEvent::Chunk(chunk) => {
                        if let Err(_) = session_mic.send_audio_chunk(&chunk).await {
                            break;
                        }
                    }
                    audio::mic::AudioStreamEvent::StreamEnd => {
                        if let Err(_) = session_mic.send_audio_stream_end().await {
                            break;
                        }
                    }
                }
            }
        });
    }

    println!("==================================================");
    println!("  ✨ Project Veronica: Autonomous AI Assistant");
    println!("  🖥️  Environment: {}", env_info);
    println!("  ⚡ Mode: Continuous Voice + Concurrent Text Input");
    println!("  🐘 Database: PostgreSQL (Auto-Cleanup Queue Active)");
    if has_mic {
        println!("  🎙️  Microphone: Always Listening (Speak naturally)");
    } else {
        println!("  ⚠️  Microphone: Unavailable (Text Only Mode)");
    }
    println!("  ⌨️  Text Input: Active (Type anytime & press Enter)");
    println!("  🗣️  Voice: {} (Google Neural Audio)", voice_name);
    println!("==================================================");
    println!("
Veronica is listening! Speak naturally, or type your message below.");
    println!("(Type 'exit' to quit)
");

    // 11. Concurrent Terminal Text Input
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    while let Ok(Some(line)) = reader.next_line().await {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
            println!("Veronica > Goodbye! Shutting down.");
            break;
        }

        if trimmed.eq_ignore_ascii_case("/schedule") || trimmed.eq_ignore_ascii_case("/agenda") {
            println!("
📅 [Veronica Agenda - Upcoming Schedules (PostgreSQL)]:");
            match schedule_engine.list_upcoming(10).await {
                Ok(items) if items.is_empty() => println!("   ✨ No upcoming reminders or meetings. Schedule is clear!"),
                Ok(items) => {
                    for (i, item) in items.iter().enumerate() {
                        println!("   {}. [{}] {} @ {}", i + 1, item.schedule_type.to_uppercase(), item.title, item.target_time);
                        println!("      📝 Context: {}", item.description);
                    }
                }
                Err(e) => eprintln!("   ❌ Error reading PostgreSQL: {}", e),
            }
            println!();
            continue;
        }

        if trimmed.eq_ignore_ascii_case("/screen") || trimmed.eq_ignore_ascii_case("/look") {
            println!("📸 [Terminal Vision]: Capturing screen and sending to Veronica...");
            match session.capture_and_send_screen().await {
                Ok(_) => {
                    println!("✅ Screen attached! Asking Veronica to inspect...");
                    let _ = session.send_text("I have shared my screen with you. Please describe what is visible on my screen.").await;
                }
                Err(e) => eprintln!("❌ Screen capture error: {}", e),
            }
            continue;
        }

        // Send user typed text concurrently to the live session
        if let Err(err) = session.send_text(trimmed).await {
            eprintln!("❌ Failed to send message: {}", err);
        }
    }
}
