use super::{TranscriptEntry, Speaker};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use reqwest::multipart;

pub struct TranscriptionEngine {
    transcript: Arc<Mutex<Vec<TranscriptEntry>>>,
    api_key: Option<String>,
    base_url: String,
}

impl TranscriptionEngine {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            transcript: Arc::new(Mutex::new(Vec::new())),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    pub fn set_api_key(&mut self, key: String) {
        self.api_key = Some(key);
    }

    pub fn get_transcript(&self) -> Vec<TranscriptEntry> {
        self.transcript.lock().unwrap().clone()
    }

    pub fn get_recent_transcript(&self, max_entries: usize) -> Vec<TranscriptEntry> {
        let transcript = self.transcript.lock().unwrap();
        let start = transcript.len().saturating_sub(max_entries);
        transcript[start..].to_vec()
    }

    pub fn get_transcript_text(&self, max_entries: usize) -> String {
        self.get_recent_transcript(max_entries)
            .iter()
            .map(|e| format!("[{}]: {}", e.speaker, e.text))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Process audio chunks from the capture pipeline and transcribe them.
    pub async fn process_audio_stream(
        &self,
        mut rx: mpsc::UnboundedReceiver<super::capture::AudioChunk>,
        event_tx: mpsc::UnboundedSender<TranscriptEntry>,
    ) {
        log::info!("Transcription engine started");

        while let Some(chunk) = rx.recv().await {
            let speaker = if chunk.is_microphone {
                Speaker::Me
            } else {
                Speaker::Interviewer
            };

            match self.transcribe_chunk(&chunk.samples, chunk.sample_rate).await {
                Ok(text) if !text.trim().is_empty() => {
                    let entry = TranscriptEntry {
                        speaker,
                        text: text.trim().to_string(),
                        timestamp_ms: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64,
                    };

                    self.transcript.lock().unwrap().push(entry.clone());
                    let _ = event_tx.send(entry);
                }
                Ok(_) => {}
                Err(e) => {
                    log::error!("Transcription error: {}", e);
                }
            }
        }
    }

    /// Transcribe audio samples using OpenAI Whisper API.
    async fn transcribe_chunk(&self, samples: &[f32], sample_rate: u32) -> anyhow::Result<String> {
        if samples.is_empty() {
            return Ok(String::new());
        }

        // Check if samples contain any non-silence
        let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
        if rms < 0.005 {
            return Ok(String::new());
        }

        let Some(ref api_key) = self.api_key else {
            return Ok(String::new());
        };

        let wav_bytes = encode_wav(samples, sample_rate)?;

        let part = multipart::Part::bytes(wav_bytes)
            .file_name("audio.wav")
            .mime_str("audio/wav")?;

        let form = multipart::Form::new()
            .text("model", "whisper-1")
            .text("language", "en")
            .text("response_format", "text")
            .part("file", part);

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/audio/transcriptions", self.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Whisper API error {}: {}", status, body);
        }

        Ok(resp.text().await?)
    }

    pub fn add_manual_entry(&self, speaker: Speaker, text: String) {
        let entry = TranscriptEntry {
            speaker,
            text,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };
        self.transcript.lock().unwrap().push(entry);
    }
}

/// Encode f32 PCM samples to WAV format in memory.
fn encode_wav(samples: &[f32], sample_rate: u32) -> anyhow::Result<Vec<u8>> {
    let mut cursor = std::io::Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
    for &sample in samples {
        let s16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(s16)?;
    }
    writer.finalize()?;
    Ok(cursor.into_inner())
}
