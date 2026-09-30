use tokio::io::{AsyncBufReadExt, BufReader};

pub mod ai;
pub mod audio;
pub mod system;
pub mod vision;
pub mod schedule;
mod server;

#[tokio::main]
async fn main() {
    // 1. Load environment variables from .env
        dotenvy::dotenv().ok();

    // 1.1 Initialize Local SQLite Database for persistent schedules, reminders & context
    let schedule_db = schedule::ScheduleDb::new("data/veronica.db")
        .expect("❌ Failed to initialize schedules database");

    // Spawn background scheduler worker (0% CPU, checks every 3s)
    let worker_db = schedule_db.clone();
    tokio::spawn(async move {
        schedule::start_schedule_worker(worker_db).await;
    });

    let api_key = std::env::var("GEMINI_API_KEY")
        .expect("❌ Error: GEMINI_API_KEY not found in daemon/.env file!");
    let live_model = std::env::var("GEMINI_LIVE_MODEL")
        .unwrap_or_else(|_| "gemini-2.5-flash-native-audio-latest".to_string());
    let voice_name = std::env::var("GEMINI_VOICE")
        .unwrap_or_else(|_| "Aoede".to_string());

    // 2. Initialize Streaming Audio Player (Low-latency in-memory playback)
    let audio_player = audio::StreamingAudioPlayer::new()
        .expect("❌ Failed to initialize streaming audio player");

    // 3. Detect system environment
    let env_info = system::detector::detect_environment();

    // 4. Connect to Gemini Live WebSocket
    let live_client = match ai::GeminiLiveClient::connect(&api_key, &live_model, &voice_name).await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("❌ [Gemini Live] Connection failed: {}", err);
            return;
        }
    };

    // 5. Initialize GUI Event Broadcast Channel
    let (broadcast_tx, _) = tokio::sync::broadcast::channel::<server::protocol::DaemonMessage>(128);

    // 6. Start full-duplex Live Session with GUI broadcasting
    let session = live_client.start_duplex_session(audio_player.clone(), Some(broadcast_tx.clone()), Some(schedule_db.clone()));

    // 7. Start Localhost WebSocket Server for GUI (127.0.0.1:8765)
    if let Err(e) = server::start_server(8765, session.clone(), broadcast_tx.clone()).await {
        eprintln!("⚠️ Failed to bind GUI server on port 8765: {}", e);
    }

    // 8. Start Continuous Microphone Streaming with Low-Latency VAD
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
    println!("  🤖 Project Veronica: Autonomous AI Assistant");
    println!("  🖥️  Environment: {}", env_info);
    println!("  ⚡ Mode: Continuous Voice + Concurrent Text Input");
    if has_mic {
        println!("  🎙️  Microphone: Always Listening (Speak naturally)");
    } else {
        println!("  ⚠️  Microphone: Unavailable (Text Only Mode)");
    }
    println!("  💬 Text Input: Active (Type anytime & press Enter)");
    println!("  🔒 Voice: {} (Google Neural Audio)", voice_name);
    println!("==================================================");
    println!("\nVeronica is listening! Speak naturally, or type your message below.");
    println!("(Type 'exit' to quit)\n");

    // 7. Concurrent Terminal Text Input
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
            println!("\n📋 [Veronica Agenda - Upcoming Schedules]:");
            match schedule_db.list_upcoming(10) {
                Ok(items) if items.is_empty() => println!("   ✨ No upcoming reminders or meetings. Schedule is clear!"),
                Ok(items) => {
                    for (i, item) in items.iter().enumerate() {
                        println!("   {}. [{}] {} @ {}", i + 1, item.schedule_type.to_uppercase(), item.title, item.target_time);
                        println!("      📝 Context: {}", item.description);
                    }
                }
                Err(e) => eprintln!("   ❌ Error reading database: {}", e),
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
