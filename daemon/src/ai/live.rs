use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    MaybeTlsStream, WebSocketStream,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

use super::gemini::ToolCall;
use crate::ai::manager::ToolManager;

#[derive(Debug, Clone, Default)]
pub struct LiveResponse {
    pub text: Option<String>,
    pub audio_pcm: Option<Vec<u8>>,
    pub tool_calls: Vec<ToolCall>,
}

pub struct GeminiLiveClient;

#[derive(Clone)]
pub struct LiveSessionHandle {
    outbound_tx: tokio::sync::mpsc::Sender<Message>,
}

impl LiveSessionHandle {
    /// Streams a raw 16kHz 16-bit mono PCM chunk to Gemini Live.
    /// If reconnecting and buffer is full, transient audio drops gracefully without blocking.
    pub async fn send_audio_chunk(&self, chunk: &[u8]) -> Result<(), String> {
        let msg = json!({
            "realtime_input": {
                "media_chunks": [
                    {
                        "mime_type": "audio/pcm",
                        "data": BASE64.encode(chunk)
                    }
                ]
            }
        });
        let _ = self.outbound_tx.try_send(Message::Text(msg.to_string().into()));
        Ok(())
    }

    /// Sends a typed user text message to Gemini Live. Waits for delivery.
    pub async fn send_text(&self, text: &str) -> Result<(), String> {
        let msg = json!({
            "client_content": {
                "turns": [
                    {
                        "role": "user",
                        "parts": [
                            {
                                "text": text
                            }
                        ]
                    }
                ],
                "turn_complete": true
            }
        });
        self.outbound_tx
            .send(Message::Text(msg.to_string().into()))
            .await
            .map_err(|e| format!("Failed to send text message: {}", e))
    }

    /// Signals the end of an audio stream to Gemini Live.
    /// Note: Google's automatic activity detection handles speech boundaries natively.
    /// We keep this as a clean no-op so invalid fields like 'end_of_turn' inside
    /// 'realtime_input' never cause Google to terminate the session.
    pub async fn send_audio_stream_end(&self) -> Result<(), String> {
        Ok(())
    }

    /// Sends an image frame (JPEG format) to Gemini Live for visual inspection
    pub async fn send_image_frame(&self, jpeg_bytes: &[u8]) -> Result<(), String> {
        let msg = json!({
            "realtime_input": {
                "media_chunks": [
                    {
                        "mime_type": "image/jpeg",
                        "data": BASE64.encode(jpeg_bytes)
                    }
                ]
            }
        });
        self.outbound_tx
            .send(Message::Text(msg.to_string().into()))
            .await
            .map_err(|e| format!("Failed to send image frame: {}", e))
    }

    /// Captures the primary screen in < 15ms and immediately sends it to Gemini Live
    pub async fn capture_and_send_screen(&self) -> Result<(), String> {
        let jpeg_bytes = crate::vision::capture_primary_screen()?;
        self.send_image_frame(&jpeg_bytes).await
    }
}

impl GeminiLiveClient {
    /// Starts a resilient, full-duplex Live session that automatically reconnects
    /// whenever internet drops or the server closes the connection.
    pub fn start_resilient_session(
        api_key: String,
        model_name: String,
        voice_name: String,
        audio_player: crate::audio::StreamingAudioPlayer,
        event_tx: Option<tokio::sync::broadcast::Sender<crate::server::protocol::DaemonMessage>>,
        tool_manager: ToolManager,
    ) -> LiveSessionHandle {
        let (outbound_tx, mut outbound_rx) = tokio::sync::mpsc::channel::<Message>(256);
        let active_sink_tx = Arc::new(RwLock::new(None::<tokio::sync::mpsc::Sender<Message>>));

        // Persistent forwarder: routes messages from LiveSessionHandle into the currently active WebSocket
        let active_sink_forwarder = Arc::clone(&active_sink_tx);
        tokio::spawn(async move {
            while let Some(msg) = outbound_rx.recv().await {
                let guard = active_sink_forwarder.read().await;
                if let Some(ref tx) = *guard {
                    let _ = tx.send(msg).await;
                }
            }
        });

        // Continuous Auto-Reconnect Manager Loop
        let active_sink_manager = Arc::clone(&active_sink_tx);
        tokio::spawn(async move {
            let mut retry_delay_secs: u64 = 2;

            loop {
                println!("⚡ [Gemini Live] Connecting to Google real-time server...");
                match connect_and_handshake(&api_key, &model_name, &voice_name).await {
                    Ok(ws_stream) => {
                        retry_delay_secs = 2; // Reset backoff on successful handshake
                        println!("✨ [Gemini Live] Setup complete! Real-time audio channel established.");

                        if let Some(ref tx) = event_tx {
                            let _ = tx.send(crate::server::protocol::DaemonMessage::StateChange {
                                state: crate::server::protocol::AssistantState::Idle,
                            });
                        }

                        let (mut ws_sink, mut ws_stream) = ws_stream.split();
                        let (conn_tx, mut conn_rx) = tokio::sync::mpsc::channel::<Message>(64);
                        let conn_tx_inbound = conn_tx.clone();

                        // Register this connection as active
                        {
                            let mut guard = active_sink_manager.write().await;
                            *guard = Some(conn_tx);
                        }

                        // Task: forwards messages to current WebSocket sink
                        let sink_task = tokio::spawn(async move {
                            while let Some(msg) = conn_rx.recv().await {
                                if let Err(e) = ws_sink.send(msg).await {
                                    eprintln!("⚠️ [Gemini Live] Outbound WebSocket error: {}", e);
                                    break;
                                }
                            }
                        });

                        // Task: processes incoming voice & events from Gemini Live
                        let audio_player_clone = audio_player.clone();
                        let event_tx_clone = event_tx.clone();
                        let tool_mgr_clone = tool_manager.clone();

                        let mut speaking = false;
                        let mut user_speaking = false;
                        let mut turn_has_transcript = false;

                        while let Some(msg_res) = ws_stream.next().await {
                            let msg = match msg_res {
                                Ok(m) => m,
                                Err(e) => {
                                    eprintln!("
⚠️ [Gemini Live] Read error: {}", e);
                                    break;
                                }
                            };

                            let json_str = match msg {
                                Message::Text(t) => Some(t.to_string()),
                                Message::Binary(b) => Some(String::from_utf8_lossy(&b).to_string()),
                                Message::Close(frame) => {
                                    println!("
⚠️ [Gemini Live] Session closed by server: {:?}", frame);
                                    break;
                                }
                                _ => None,
                            };

                            if let Some(txt) = json_str {
                                use std::io::Write;
                                let v: Value = serde_json::from_str(&txt).unwrap_or(Value::Null);

                                let server_content = v.get("server_content").or_else(|| v.get("serverContent"));
                                if let Some(sc) = server_content {
                                    // Barge-in interruption
                                    if let Some(true) = sc.get("interrupted").and_then(|i| i.as_bool()) {
                                        audio_player_clone.stop();
                                        speaking = false;
                                        user_speaking = false;
                                        turn_has_transcript = false;
                                        print!("
⚡ [Interrupted - Listening to you]
");
                                        let _ = std::io::stdout().flush();
                                        if let Some(ref tx) = event_tx_clone {
                                            let _ = tx.send(crate::server::protocol::DaemonMessage::StateChange {
                                                state: crate::server::protocol::AssistantState::Listening,
                                            });
                                        }
                                    }

                                    // User speech transcript
                                    let input_transcription = sc.get("input_transcription").or_else(|| sc.get("inputTranscription"));
                                    if let Some(it) = input_transcription {
                                        if let Some(user_text) = it.get("text").and_then(|t| t.as_str()) {
                                            if !user_text.is_empty() {
                                                if !user_speaking {
                                                    print!("\rYou (Voice) > ");
                                                    user_speaking = true;
                                                    if let Some(ref tx) = event_tx_clone {
                                                        let _ = tx.send(crate::server::protocol::DaemonMessage::StateChange {
                                                            state: crate::server::protocol::AssistantState::Listening,
                                                        });
                                                    }
                                                }
                                                print!("{}", user_text);
                                                let _ = std::io::stdout().flush();
                                                if let Some(ref tx) = event_tx_clone {
                                                    let _ = tx.send(crate::server::protocol::DaemonMessage::Transcript {
                                                        speaker: "USER".to_string(),
                                                        text: user_text.to_string(),
                                                        is_final: false,
                                                    });
                                                }
                                            }
                                        }
                                    }

                                    // Veronica spoken transcript
                                    let output_transcription = sc.get("output_transcription").or_else(|| sc.get("outputTranscription"));
                                    if let Some(ot) = output_transcription {
                                        if let Some(bot_text) = ot.get("text").and_then(|t| t.as_str()) {
                                            if user_speaking {
                                                println!();
                                                user_speaking = false;
                                            }
                                            if !speaking {
                                                print!("\rVeronica > ");
                                                speaking = true;
                                                if let Some(ref tx) = event_tx_clone {
                                                    let _ = tx.send(crate::server::protocol::DaemonMessage::StateChange {
                                                        state: crate::server::protocol::AssistantState::Speaking,
                                                    });
                                                }
                                            }
                                            print!("{}", bot_text);
                                            let _ = std::io::stdout().flush();
                                            turn_has_transcript = true;
                                            if let Some(ref tx) = event_tx_clone {
                                                let _ = tx.send(crate::server::protocol::DaemonMessage::Transcript {
                                                    speaker: "VERONICA".to_string(),
                                                    text: bot_text.to_string(),
                                                    is_final: false,
                                                });
                                            }
                                        }
                                    }

                                    // Model turn: neural audio chunks & tool calls
                                    let model_turn = sc.get("model_turn").or_else(|| sc.get("modelTurn"));
                                    if let Some(mt) = model_turn {
                                        if let Some(parts) = mt.get("parts").and_then(|p| p.as_array()) {
                                            for part in parts {
                                                let is_thought = part.get("thought").and_then(|th| th.as_bool()).unwrap_or(false);
                                                if !is_thought && !turn_has_transcript {
                                                    if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
                                                        if user_speaking {
                                                            println!();
                                                            user_speaking = false;
                                                        }
                                                        if !speaking {
                                                            print!("\rVeronica > ");
                                                            speaking = true;
                                                            if let Some(ref tx) = event_tx_clone {
                                                                let _ = tx.send(crate::server::protocol::DaemonMessage::StateChange {
                                                                    state: crate::server::protocol::AssistantState::Speaking,
                                                                });
                                                            }
                                                        }
                                                        print!("{}", t);
                                                        let _ = std::io::stdout().flush();
                                                        if let Some(ref tx) = event_tx_clone {
                                                            let _ = tx.send(crate::server::protocol::DaemonMessage::Transcript {
                                                                speaker: "VERONICA".to_string(),
                                                                text: t.to_string(),
                                                                is_final: false,
                                                            });
                                                        }
                                                    }
                                                }

                                                // Real-time audio playback chunk (24kHz PCM)
                                                let inline_data = part.get("inline_data").or_else(|| part.get("inlineData"));
                                                if let Some(id) = inline_data {
                                                    if let Some(data_b64) = id.get("data").and_then(|d| d.as_str()) {
                                                        if let Ok(chunk) = BASE64.decode(data_b64) {
                                                            audio_player_clone.play_pcm_chunk(chunk, 24000);
                                                        }
                                                    }
                                                }

                                                // Function calls inside parts
                                                let func_call = part.get("function_call").or_else(|| part.get("functionCall"));
                                                if let Some(fc) = func_call {
                                                    if let Some(name) = fc.get("name").and_then(|n| n.as_str()) {
                                                        let args = fc.get("args").cloned().unwrap_or(Value::Null);
                                                        let call_id = fc.get("id").or_else(|| fc.get("call_id")).and_then(|i| i.as_str());

                                                        let response_msg = tool_mgr_clone.dispatch(name, &args, call_id).await;
                                                        let _ = conn_tx_inbound.send(response_msg).await;
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    let is_turn_complete = sc.get("turn_complete").or_else(|| sc.get("turnComplete"))
                                        .and_then(|tc| tc.as_bool())
                                        .unwrap_or(false);

                                    if is_turn_complete {
                                        if user_speaking {
                                            println!();
                                            user_speaking = false;
                                        }
                                        if speaking {
                                            println!();
                                            speaking = false;
                                        }
                                        turn_has_transcript = false;
                                        if let Some(ref tx) = event_tx_clone {
                                            let _ = tx.send(crate::server::protocol::DaemonMessage::StateChange {
                                                state: crate::server::protocol::AssistantState::Idle,
                                            });
                                        }
                                    }
                                }

                                // Top-level tool_call
                                let tool_call = v.get("tool_call").or_else(|| v.get("toolCall"));
                                if let Some(tc) = tool_call {
                                    let function_calls = tc.get("function_calls").or_else(|| tc.get("functionCalls"))
                                        .and_then(|f| f.as_array());
                                    if let Some(calls) = function_calls {
                                        for fc in calls {
                                            let name = fc.get("name").and_then(|n| n.as_str());
                                            let call_id = fc.get("id").or_else(|| fc.get("call_id")).and_then(|i| i.as_str());
                                            let args = fc.get("args").cloned().unwrap_or(Value::Null);

                                            if let Some(name) = name {
                                                let response_msg = tool_mgr_clone.dispatch(name, &args, call_id).await;
                                                let _ = conn_tx_inbound.send(response_msg).await;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Connection dropped or ended
                        sink_task.abort();
                        {
                            let mut guard = active_sink_manager.write().await;
                            *guard = None;
                        }

                        println!("⚠️ [Gemini Live] Connection dropped. Automatically reconnecting in {}s...", retry_delay_secs);
                    }
                    Err(e) => {
                        eprintln!("⚠️ [Gemini Live] Connection failed: {}. Retrying in {}s...", e, retry_delay_secs);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(retry_delay_secs)).await;
                retry_delay_secs = (retry_delay_secs * 2).min(10);
            }
        });

        LiveSessionHandle { outbound_tx }
    }
}

/// Helper: connects to Google and completes the Live setup handshake
async fn connect_and_handshake(
    api_key: &str,
    model_name: &str,
    voice_name: &str,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, String> {
    let endpoint = format!(
        "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent?key={}",
        api_key
    );

    let (mut ws_stream, _) = connect_async(&endpoint)
        .await
        .map_err(|e| format!("WebSocket network connection failed: {}", e))?;

    let setup_msg = json!({
        "setup": {
            "model": format!("models/{}", model_name),
            "generation_config": {
                "response_modalities": ["AUDIO"],
                "speech_config": {
                    "voice_config": {
                        "prebuilt_voice_config": {
                            "voice_name": voice_name
                        }
                    }
                },
                "thinking_config": {
                    "thinking_budget": 0
                }
            },
            "system_instruction": {
                "parts": [
                    {
                        "text": format!(
                            "You are Veronica, an autonomous personal AI assistant running directly on the user's OS.

{}

TOOL USAGE WORKFLOW (2-STEP DISCOVERY):
1. Check the available capabilities list above. To get the exact JSON parameters for an action, call 'select_feature_function' with the chosen feature and function. Its Master Function will supply the exact schema.
2. When the schema is returned, call 'execute_feature_function' with the filled 'data' according to that schema to execute.
(Note: If you already know the required parameters, you may directly call 'execute_feature_function').

Speak warmly, concisely, and naturally. Respond directly in 1 to 2 spoken sentences.",
                            crate::ai::catalog::get_catalog()
                        )
                    }
                ]
            },
            "tools": [
                {
                    "function_declarations": ToolManager::get_declarations()
                }
            ],
            "output_audio_transcription": {},
            "input_audio_transcription": {},
            "realtime_input_config": {
                "automatic_activity_detection": {
                    "disabled": false,
                    "silence_duration_ms": 400
                }
            }
        }
    });

    ws_stream
        .send(Message::Text(setup_msg.to_string().into()))
        .await
        .map_err(|e| format!("Failed to send setup message: {}", e))?;

    let mut setup_done = false;
    while let Some(msg_res) = ws_stream.next().await {
        let json_str = match msg_res {
            Ok(Message::Text(txt)) => Some(txt.to_string()),
            Ok(Message::Binary(bin)) => Some(String::from_utf8_lossy(&bin).to_string()),
            Ok(Message::Close(frame)) => {
                return Err(format!("Google closed Live connection during setup: {:?}", frame));
            }
            Err(e) => {
                return Err(format!("WebSocket error during setup: {}", e));
            }
            _ => None,
        };

        if let Some(txt) = json_str {
            let v: Value = serde_json::from_str(&txt).unwrap_or(Value::Null);
            if v.get("setupComplete").is_some() || v.get("setup_complete").is_some() {
                setup_done = true;
                break;
            } else if let Some(err) = v.get("error") {
                return Err(format!("Gemini Live setup rejected: {}", err));
            }
        }
    }

    if !setup_done {
        return Err("Connection closed before setup completed".to_string());
    }

    Ok(ws_stream)
}
