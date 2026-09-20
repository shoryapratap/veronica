use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    MaybeTlsStream, WebSocketStream,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

use super::gemini::ToolCall;

#[derive(Debug, Clone, Default)]
pub struct LiveResponse {
    pub text: Option<String>,
    pub audio_pcm: Option<Vec<u8>>,
    pub tool_calls: Vec<ToolCall>,
}

pub struct GeminiLiveClient {
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    #[allow(dead_code)]
    api_key: String,
    #[allow(dead_code)]
    model: String,
    #[allow(dead_code)]
    voice: String,
}

#[derive(Clone)]
pub struct LiveSessionHandle {
    outbound_tx: tokio::sync::mpsc::Sender<Message>,
}

impl LiveSessionHandle {
    /// Streams a raw 16kHz 16-bit mono PCM chunk to Gemini Live
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
        self.outbound_tx
            .send(Message::Text(msg.to_string().into()))
            .await
            .map_err(|e| format!("Failed to send audio chunk: {}", e))
    }

    /// Sends a typed user text message to Gemini Live
    pub async fn send_text(&self, text: &str) -> Result<(), String> {
        let msg = json!({
            "client_content": {
                "turns": [
                    {
                        "role": "user",
                        "parts": [
                            { "text": text }
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

    /// Signals the end of the user's voice input stream so Gemini finalizes the turn immediately
    pub async fn send_audio_stream_end(&self) -> Result<(), String> {
        let msg = json!({
            "realtime_input": {
                "audio_stream_end": true
            }
        });
        self.outbound_tx
            .send(Message::Text(msg.to_string().into()))
            .await
            .map_err(|e| format!("Failed to send audio stream end: {}", e))
    }
}

impl GeminiLiveClient {
    /// Connects to Google's real-time Gemini Live WebSocket API with locked voice
    pub async fn connect(api_key: &str, model_name: &str, voice_name: &str) -> Result<Self, String> {
        let endpoint = format!(
            "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent?key={}",
            api_key
        );

        println!("⚡ [Gemini Live] Opening real-time WebSocket connection to Google...");
        println!("🔒 [Gemini Live] Locking Veronica's neural voice to: {}", voice_name);

        let (mut ws_stream, _) = connect_async(&endpoint)
            .await
            .map_err(|e| format!("WebSocket connection failed: {}", e))?;

        // 1. Send the initial Live Setup frame with locked voice, zero thinking delay, and system instruction
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
                            "text": "You are Veronica, an autonomous personal AI assistant running directly inside the user's operating system. Speak naturally, concisely, and warmly. Answer directly and immediately in 1 to 2 sentences. Never output meta-thoughts or internal commentary."
                        }
                    ]
                },
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

        // 2. Wait for setup_complete handshake
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
                    println!("✅ [Gemini Live] Setup complete! Real-time audio channel established.");
                    setup_done = true;
                    break;
                } else if let Some(err) = v.get("error") {
                    return Err(format!("Gemini Live setup rejected: {}", err));
                }
            }
        }

        if !setup_done {
            return Err("Gemini Live connection closed before setup completed.".to_string());
        }

        Ok(Self {
            ws_stream,
            api_key: api_key.to_string(),
            model: model_name.to_string(),
            voice: voice_name.to_string(),
        })
    }

    /// Starts a full-duplex session with continuous concurrent voice and text streaming
    pub fn start_duplex_session(
        self,
        audio_player: crate::audio::StreamingAudioPlayer,
    ) -> LiveSessionHandle {
        let (outbound_tx, mut outbound_rx) = tokio::sync::mpsc::channel::<Message>(64);
        let (mut ws_sink, mut ws_stream) = self.ws_stream.split();

        // Outbound task: forwards microphone audio chunks and text to WebSocket
        tokio::spawn(async move {
            while let Some(msg) = outbound_rx.recv().await {
                if let Err(e) = ws_sink.send(msg).await {
                    eprintln!("⚠️ [Gemini Live] Outbound WebSocket error: {}", e);
                    break;
                }
            }
        });

        // Inbound task: continuously receives voice & text from Gemini Live
        tokio::spawn(async move {
            use std::io::Write;
            let mut speaking = false;
            let mut user_speaking = false;
            let mut turn_has_transcript = false;

            while let Some(msg_res) = ws_stream.next().await {
                let msg = match msg_res {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("\n⚠️ [Gemini Live] Read error: {}", e);
                        break;
                    }
                };

                let json_str = match msg {
                    Message::Text(t) => Some(t.to_string()),
                    Message::Binary(b) => Some(String::from_utf8_lossy(&b).to_string()),
                    Message::Close(frame) => {
                        println!("\n⚠️ [Gemini Live] Session closed: {:?}", frame);
                        break;
                    }
                    _ => None,
                };

                if let Some(txt) = json_str {
                    let v: Value = serde_json::from_str(&txt).unwrap_or(Value::Null);

                    let server_content = v.get("server_content").or_else(|| v.get("serverContent"));
                    if let Some(sc) = server_content {
                        // Voice Interruption (Barge-in): User started speaking over Veronica
                        if let Some(true) = sc.get("interrupted").and_then(|i| i.as_bool()) {
                            audio_player.stop();
                            speaking = false;
                            user_speaking = false;
                            turn_has_transcript = false;
                            print!("\n⚡ [Interrupted - Listening to you]\n");
                            let _ = std::io::stdout().flush();
                        }

                        // 1. User speech transcript from input_audio_transcription
                        let input_transcription = sc.get("input_transcription").or_else(|| sc.get("inputTranscription"));
                        if let Some(it) = input_transcription {
                            if let Some(user_text) = it.get("text").and_then(|t| t.as_str()) {
                                if !user_text.is_empty() {
                                    if !user_speaking {
                                        print!("\rYou (Voice) > ");
                                        user_speaking = true;
                                    }
                                    print!("{}", user_text);
                                    let _ = std::io::stdout().flush();
                                }
                            }
                        }

                        // 2. Veronica's spoken transcript from output_audio_transcription
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
                                }
                                print!("{}", bot_text);
                                let _ = std::io::stdout().flush();
                                turn_has_transcript = true;
                            }
                        }

                        // 3. Model turn: audio PCM chunks, fallback text, tool calls
                        let model_turn = sc.get("model_turn").or_else(|| sc.get("modelTurn"));
                        if let Some(mt) = model_turn {
                            if let Some(parts) = mt.get("parts").and_then(|p| p.as_array()) {
                                for part in parts {
                                    // Spoken text transcript fallback
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
                                            }
                                            print!("{}", t);
                                            let _ = std::io::stdout().flush();
                                        }
                                    }

                                    // Real-time neural audio streaming chunk
                                    let inline_data = part.get("inline_data").or_else(|| part.get("inlineData"));
                                    if let Some(id) = inline_data {
                                        if let Some(data_b64) = id.get("data").and_then(|d| d.as_str()) {
                                            if let Ok(chunk) = BASE64.decode(data_b64) {
                                                audio_player.play_pcm_chunk(chunk, 24000);
                                            }
                                        }
                                    }

                                    // Tool calls (e.g. launch_app)
                                    let func_call = part.get("function_call").or_else(|| part.get("functionCall"));
                                    if let Some(fc) = func_call {
                                        if let Some(name) = fc.get("name").and_then(|n| n.as_str()) {
                                            let args = fc.get("args").cloned().unwrap_or(Value::Null);
                                            if name == "launch_app" {
                                                if let Some(target) = args.get("target").and_then(|t| t.as_str()) {
                                                    println!("\n   🚀 [Action]: Launching '{}'...", target);
                                                    match crate::system::command::launch_app(target).await {
                                                        Ok(msg) => println!("   ✅ [Success]: {}", msg),
                                                        Err(err) => eprintln!("   ❌ [Failed]: {}", err),
                                                    }
                                                }
                                            }
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
                        }
                    }
                }
            }
        });

        LiveSessionHandle { outbound_tx }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handshake_with_zero_thinking() {
        dotenvy::dotenv().ok();
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            let res = GeminiLiveClient::connect(&key, "gemini-2.5-flash-native-audio-latest", "Aoede").await;
            assert!(res.is_ok(), "Setup failed: {:?}", res.err());
        }
    }

    #[tokio::test]
    async fn test_audio_stream_end() {
        dotenvy::dotenv().ok();
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            let client = GeminiLiveClient::connect(&key, "gemini-2.5-flash-native-audio-latest", "Aoede").await.unwrap();
            let player = crate::audio::StreamingAudioPlayer::new().unwrap();
            let session = client.start_duplex_session(player);

            // Send silent 16kHz PCM chunk
            let silent_chunk = vec![0u8; 1600];
            let send_res = session.send_audio_chunk(&silent_chunk).await;
            assert!(send_res.is_ok(), "Failed to send chunk: {:?}", send_res.err());

            // Send audio_stream_end
            let end_res = session.send_audio_stream_end().await;
            assert!(end_res.is_ok(), "Failed to send audio stream end: {:?}", end_res.err());
        }
    }
}
