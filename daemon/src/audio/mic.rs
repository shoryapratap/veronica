use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};

#[derive(Debug, Clone)]
pub enum AudioStreamEvent {
    Chunk(Vec<u8>),
    StreamEnd,
}

struct VadProcessor {
    sample_rate: u32,
    chunk_samples_native: usize,
    accumulator: Vec<f32>,
    pre_speech_buffer: std::collections::VecDeque<Vec<u8>>,
    is_speaking: bool,
    silence_counter: usize,
    noise_floor: f32,
    tx: tokio::sync::mpsc::Sender<AudioStreamEvent>,
}

impl VadProcessor {
    fn new(
        sample_rate: u32,
        chunk_duration_sec: f32,
        tx: tokio::sync::mpsc::Sender<AudioStreamEvent>,
    ) -> Self {
        let chunk_samples_native = (sample_rate as f32 * chunk_duration_sec).round() as usize;
        Self {
            sample_rate,
            chunk_samples_native,
            accumulator: Vec::with_capacity(chunk_samples_native * 2),
            pre_speech_buffer: std::collections::VecDeque::with_capacity(6),
            is_speaking: false,
            silence_counter: 0,
            noise_floor: 0.002,
            tx,
        }
    }

    fn push_mono_samples(&mut self, samples: &[f32]) {
        self.accumulator.extend_from_slice(samples);

        while self.accumulator.len() >= self.chunk_samples_native {
            let native_chunk: Vec<f32> = self.accumulator.drain(..self.chunk_samples_native).collect();

            // Calculate RMS energy of this 50ms audio chunk
            let sum_sq: f32 = native_chunk.iter().map(|&s| s * s).sum();
            let rms = (sum_sq / native_chunk.len() as f32).sqrt();

            if !self.is_speaking {
                // Dynamically track ambient background noise floor
                self.noise_floor = self.noise_floor * 0.90 + rms * 0.10;
            }

            // Speech threshold is dynamically set at 3.5x background noise floor (clamped between 0.010 and 0.035)
            // Human speech typically measures 0.03..0.20 RMS, safely above this threshold
            let speech_threshold = (self.noise_floor * 3.5).clamp(0.010, 0.035);

            let pcm16 = resample_and_encode_pcm16(&native_chunk, self.sample_rate, 16000);

            const MAX_PRE_SPEECH_CHUNKS: usize = 4;   // ~200ms pre-speech buffer (preserves first syllable)
            const TRAILING_SILENCE_CHUNKS: usize = 8; // ~400ms trailing padding (crisp, low-latency turn completion)

            if self.is_speaking {
                if rms >= speech_threshold {
                    self.silence_counter = 0;
                    let _ = self.tx.try_send(AudioStreamEvent::Chunk(pcm16));
                } else {
                    self.silence_counter += 1;
                    // Send trailing silence padding so word endings aren't clipped
                    let _ = self.tx.try_send(AudioStreamEvent::Chunk(pcm16));

                    if self.silence_counter >= TRAILING_SILENCE_CHUNKS {
                        // User stopped speaking: finalize turn immediately
                        self.is_speaking = false;
                        self.silence_counter = 0;
                        let _ = self.tx.try_send(AudioStreamEvent::StreamEnd);
                    }
                }
            } else {
                if rms >= speech_threshold {
                    // User started speaking!
                    self.is_speaking = true;
                    self.silence_counter = 0;

                    // Flush pre-speech buffer to preserve initial consonant/syllable
                    for buffered in self.pre_speech_buffer.drain(..) {
                        let _ = self.tx.try_send(AudioStreamEvent::Chunk(buffered));
                    }
                    let _ = self.tx.try_send(AudioStreamEvent::Chunk(pcm16));
                } else {
                    // Ambient silence: preserve rolling pre-speech buffer and send NO network packets
                    self.pre_speech_buffer.push_back(pcm16);
                    if self.pre_speech_buffer.len() > MAX_PRE_SPEECH_CHUNKS {
                        self.pre_speech_buffer.pop_front();
                    }
                }
            }
        }
    }
}

/// Continuous background microphone streamer that downmixes, resamples,
/// and applies hybrid Voice Activity Detection (VAD) to trigger instant Gemini turns.
pub struct AudioStreamer {
    _stream: Stream,
}

impl AudioStreamer {
    pub fn start(tx: tokio::sync::mpsc::Sender<AudioStreamEvent>) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| "No default audio input device (microphone) found.".to_string())?;

        let device_name = device.name().unwrap_or_else(|_| "Default Microphone".to_string());
        println!("🎤 [Microphone] Streaming with Low-Latency VAD from: {}", device_name);

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to read microphone configuration: {}", e))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let sample_format = config.sample_format();

        // 50ms chunk duration for optimal responsiveness and bandwidth
        let vad_processor = Arc::new(Mutex::new(VadProcessor::new(sample_rate, 0.050, tx)));
        let proc_clone = Arc::clone(&vad_processor);
        let err_fn = |err| eprintln!("❌ [Microphone Stream Error]: {}", err);

        let stream = match sample_format {
            SampleFormat::F32 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &_| {
                        let mut mono_buf = Vec::with_capacity(data.len() / channels);
                        for frame in data.chunks(channels) {
                            let mono: f32 = frame.iter().sum::<f32>() / channels as f32;
                            mono_buf.push(mono);
                        }
                        if let Ok(mut proc) = proc_clone.lock() {
                            proc.push_mono_samples(&mono_buf);
                        }
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::I16 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &_| {
                        let mut mono_buf = Vec::with_capacity(data.len() / channels);
                        for frame in data.chunks(channels) {
                            let mono: f32 = frame.iter().map(|&s| s as f32 / 32768.0).sum::<f32>()
                                / channels as f32;
                            mono_buf.push(mono);
                        }
                        if let Ok(mut proc) = proc_clone.lock() {
                            proc.push_mono_samples(&mono_buf);
                        }
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::U16 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[u16], _: &_| {
                        let mut mono_buf = Vec::with_capacity(data.len() / channels);
                        for frame in data.chunks(channels) {
                            let mono: f32 = frame
                                .iter()
                                .map(|&s| (s as f32 - 32768.0) / 32768.0)
                                .sum::<f32>()
                                / channels as f32;
                            mono_buf.push(mono);
                        }
                        if let Ok(mut proc) = proc_clone.lock() {
                            proc.push_mono_samples(&mono_buf);
                        }
                    },
                    err_fn,
                    None,
                )
            }
            _ => return Err(format!("Unsupported audio sample format: {:?}", sample_format)),
        }
        .map_err(|e| format!("Failed to build audio input stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start streaming microphone: {}", e))?;

        Ok(Self { _stream: stream })
    }
}

/// High-clarity resampler converting raw hardware audio into 16kHz 16-bit mono PCM bytes
/// Uses area-averaging (boxcar) anti-aliasing filter for downsampling from 44.1k/48k to 16k.
fn resample_and_encode_pcm16(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<u8> {
    if samples.is_empty() {
        return Vec::new();
    }

    if source_rate == target_rate {
        let mut pcm = Vec::with_capacity(samples.len() * 2);
        for &s in samples {
            let sample_i16 = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
            pcm.extend_from_slice(&sample_i16.to_le_bytes());
        }
        return pcm;
    }

    let ratio = source_rate as f64 / target_rate as f64;
    let target_len = (samples.len() as f64 / ratio).round() as usize;
    let mut pcm_bytes = Vec::with_capacity(target_len * 2);

    if ratio > 1.0 {
        // Downsampling (e.g. 48kHz / 44.1kHz -> 16kHz):
        // Area-averaging prevents high-frequency aliasing and harshness
        for i in 0..target_len {
            let start_src = (i as f64 * ratio).floor() as usize;
            let end_src = (((i + 1) as f64 * ratio).ceil() as usize).min(samples.len());
            let count = (end_src - start_src).max(1);
            let avg: f32 = samples[start_src..end_src].iter().sum::<f32>() / count as f32;
            let sample_i16 = (avg.clamp(-1.0, 1.0) * 32767.0) as i16;
            pcm_bytes.extend_from_slice(&sample_i16.to_le_bytes());
        }
    } else {
        // Upsampling: linear interpolation
        for i in 0..target_len {
            let src_idx = i as f64 * ratio;
            let idx0 = src_idx.floor() as usize;
            let idx1 = (idx0 + 1).min(samples.len() - 1);
            let frac = (src_idx - idx0 as f64) as f32;
            let s0 = samples.get(idx0).copied().unwrap_or(0.0);
            let s1 = samples.get(idx1).copied().unwrap_or(0.0);
            let val = s0 + (s1 - s0) * frac;
            let sample_i16 = (val.clamp(-1.0, 1.0) * 32767.0) as i16;
            pcm_bytes.extend_from_slice(&sample_i16.to_le_bytes());
        }
    }

    pcm_bytes
}

/// Fallback single-turn audio recorder
pub struct AudioRecorder {
    buffer: Arc<Mutex<Vec<f32>>>,
    stream: Option<Stream>,
    sample_rate: u32,
}

impl AudioRecorder {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| "No default audio input device (microphone) found.".to_string())?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to read microphone configuration: {}", e))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let sample_format = config.sample_format();

        let buffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = Arc::clone(&buffer);

        let err_fn = |err| eprintln!("❌ [Microphone Stream Error]: {}", err);

        let stream = match sample_format {
            SampleFormat::F32 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &_| {
                        let mut buf = buffer_clone.lock().unwrap();
                        for frame in data.chunks(channels) {
                            let mono: f32 = frame.iter().sum::<f32>() / channels as f32;
                            buf.push(mono);
                        }
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::I16 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &_| {
                        let mut buf = buffer_clone.lock().unwrap();
                        for frame in data.chunks(channels) {
                            let mono: f32 = frame.iter().map(|&s| s as f32 / 32768.0).sum::<f32>()
                                / channels as f32;
                            buf.push(mono);
                        }
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::U16 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[u16], _: &_| {
                        let mut buf = buffer_clone.lock().unwrap();
                        for frame in data.chunks(channels) {
                            let mono: f32 = frame
                                .iter()
                                .map(|&s| (s as f32 - 32768.0) / 32768.0)
                                .sum::<f32>()
                                / channels as f32;
                            buf.push(mono);
                        }
                    },
                    err_fn,
                    None,
                )
            }
            _ => return Err(format!("Unsupported audio sample format: {:?}", sample_format)),
        }
        .map_err(|e| format!("Failed to build audio input stream: {}", e))?;

        Ok(Self {
            buffer,
            stream: Some(stream),
            sample_rate,
        })
    }

    pub fn start(&self) -> Result<(), String> {
        if let Some(stream) = &self.stream {
            self.buffer.lock().unwrap().clear();
            stream.play().map_err(|e| format!("Failed to start recording: {}", e))?;
            Ok(())
        } else {
            Err("Audio input stream is not initialized".to_string())
        }
    }

    pub fn stop(&self) -> Result<Vec<u8>, String> {
        if let Some(stream) = &self.stream {
            stream.pause().map_err(|e| format!("Failed to pause recording: {}", e))?;
            let raw_samples = {
                let mut buf = self.buffer.lock().unwrap();
                let samples = buf.clone();
                buf.clear();
                samples
            };

            let pcm_16k = resample_and_encode_pcm16(&raw_samples, self.sample_rate, 16000);
            Ok(pcm_16k)
        } else {
            Err("Audio input stream is not initialized".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mic_readings() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(128);
        if let Ok(_streamer) = AudioStreamer::start(tx) {
            println!("🎤 Started streamer for test...");
            let start = std::time::Instant::now();
            let mut count = 0;
            while start.elapsed() < std::time::Duration::from_millis(600) {
                if let Ok(event) = rx.try_recv() {
                    count += 1;
                    match event {
                        AudioStreamEvent::Chunk(c) => println!("Got chunk {} (len={})", count, c.len()),
                        AudioStreamEvent::StreamEnd => println!("Got StreamEnd"),
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
            println!("Total events received in 600ms: {}", count);
        }
    }
}
