use std::io::Cursor;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rodio::{Decoder, OutputStream, Sink};

pub enum AudioCommand {
    PlayPcmChunk { pcm_bytes: Vec<u8>, sample_rate: u32 },
    PlayWav { wav_bytes: Vec<u8> },
    Stop,
}

/// Continuous, low-latency streaming audio player with instant interruption (barge-in) support
#[derive(Clone)]
pub struct StreamingAudioPlayer {
    sender: tokio::sync::mpsc::UnboundedSender<AudioCommand>,
}

impl StreamingAudioPlayer {
    pub fn new() -> Result<Self, String> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<AudioCommand>();

        std::thread::Builder::new()
            .name("veronica-audio-player".to_string())
            .spawn(move || {
                let (_stream, stream_handle) = match OutputStream::try_default() {
                    Ok(pair) => pair,
                    Err(err) => {
                        eprintln!("❌ [Audio Player] Failed to open default audio output device: {}", err);
                        return;
                    }
                };

                let mut current_sink = Sink::try_new(&stream_handle).ok();

                while let Some(cmd) = rx.blocking_recv() {
                    match cmd {
                        AudioCommand::PlayPcmChunk { pcm_bytes, sample_rate } => {
                            if current_sink.is_none() {
                                current_sink = Sink::try_new(&stream_handle).ok();
                            }
                            if let Some(ref sink) = current_sink {
                                let samples: Vec<i16> = pcm_bytes
                                    .chunks_exact(2)
                                    .map(|c| i16::from_le_bytes([c[0], c[1]]))
                                    .collect();
                                let source = rodio::buffer::SamplesBuffer::new(1, sample_rate, samples);
                                sink.append(source);
                                sink.play();
                            }
                        }
                        AudioCommand::PlayWav { wav_bytes } => {
                            if current_sink.is_none() {
                                current_sink = Sink::try_new(&stream_handle).ok();
                            }
                            if let Some(ref sink) = current_sink {
                                let cursor = Cursor::new(wav_bytes);
                                if let Ok(source) = Decoder::new(cursor) {
                                    sink.append(source);
                                    sink.play();
                                }
                            }
                        }
                        AudioCommand::Stop => {
                            if let Some(sink) = current_sink.take() {
                                sink.stop();
                            }
                            current_sink = Sink::try_new(&stream_handle).ok();
                        }
                    }
                }
            })
            .map_err(|e| format!("Failed to spawn audio worker thread: {}", e))?;

        Ok(Self { sender: tx })
    }

    /// Appends raw PCM bytes directly to the playback buffer in real-time
    pub fn play_pcm_chunk(&self, pcm_bytes: Vec<u8>, sample_rate: u32) {
        let _ = self.sender.send(AudioCommand::PlayPcmChunk { pcm_bytes, sample_rate });
    }

    /// Stops audio playback immediately (called on voice interruption / barge-in)
    pub fn stop(&self) {
        let _ = self.sender.send(AudioCommand::Stop);
    }
}

/// Plays base64 encoded audio in-memory
pub async fn play_gemini_audio(base64_audio: &str) {
    if let Ok(audio_bytes) = BASE64.decode(base64_audio) {
        let wav_data = if audio_bytes.starts_with(b"RIFF") {
            audio_bytes
        } else {
            pcm_to_wav(&audio_bytes, 24000, 1)
        };

        tokio::task::spawn_blocking(move || {
            if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
                if let Ok(sink) = Sink::try_new(&stream_handle) {
                    let cursor = Cursor::new(wav_data);
                    if let Ok(source) = Decoder::new(cursor) {
                        sink.append(source);
                        sink.sleep_until_end();
                    }
                }
            }
        })
        .await
        .ok();
    }
}

/// Plays raw PCM audio bytes with in-memory WAV container
pub async fn play_pcm_audio(pcm_data: &[u8], sample_rate: u32) {
    let wav_data = pcm_to_wav(pcm_data, sample_rate, 1);
    tokio::task::spawn_blocking(move || {
        if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                let cursor = Cursor::new(wav_data);
                if let Ok(source) = Decoder::new(cursor) {
                    sink.append(source);
                    sink.sleep_until_end();
                }
            }
        }
    })
    .await
    .ok();
}

/// Wraps raw 16-bit PCM audio in a valid RIFF/WAV header
pub fn pcm_to_wav(pcm_data: &[u8], sample_rate: u32, channels: u16) -> Vec<u8> {
    let mut wav = Vec::with_capacity(44 + pcm_data.len());
    let byte_rate = sample_rate * channels as u32 * 2;
    let block_align = channels * 2;
    let data_len = pcm_data.len() as u32;
    let riff_len = 36 + data_len;

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_len.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt sub-chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());

    // data sub-chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.extend_from_slice(pcm_data);

    wav
}