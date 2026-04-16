use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct AudioChunk {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub is_microphone: bool,
}

/// Thread-safe recording state. The actual cpal::Stream lives on a dedicated
/// audio thread since it is !Send. We communicate via the `is_recording` flag.
pub struct AudioCapture {
    is_recording: Arc<Mutex<bool>>,
}

// Safety: AudioCapture no longer holds a cpal::Stream directly.
unsafe impl Send for AudioCapture {}
unsafe impl Sync for AudioCapture {}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&mut self) -> anyhow::Result<mpsc::UnboundedReceiver<AudioChunk>> {
        let (tx, rx) = mpsc::unbounded_channel();
        *self.is_recording.lock().unwrap() = true;

        let is_recording = self.is_recording.clone();
        let mic_tx = tx.clone();

        // Microphone capture runs on its own thread (cpal::Stream is !Send)
        std::thread::spawn(move || {
            if let Err(e) = run_microphone_capture(mic_tx, is_recording.clone()) {
                log::error!("Microphone capture error: {}", e);
            }
        });

        // System audio capture runs on its own thread
        let sys_tx = tx;
        let sys_recording = self.is_recording.clone();
        std::thread::spawn(move || {
            if let Err(e) = run_system_audio_capture(sys_tx, sys_recording) {
                log::error!("System audio capture error: {}", e);
            }
        });

        log::info!("Audio capture started");
        Ok(rx)
    }

    pub fn stop(&mut self) {
        *self.is_recording.lock().unwrap() = false;
        log::info!("Audio capture stopped");
    }

    pub fn is_recording(&self) -> bool {
        *self.is_recording.lock().unwrap()
    }
}

fn run_microphone_capture(
    tx: mpsc::UnboundedSender<AudioChunk>,
    is_recording: Arc<Mutex<bool>>,
) -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

    let config = device.default_input_config()?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;

    log::info!(
        "Microphone: {} ({}Hz, {} ch)",
        device.name().unwrap_or_default(),
        sample_rate,
        channels
    );

    let chunk_samples = sample_rate as usize * 2; // 2 seconds per chunk
    let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::with_capacity(chunk_samples)));
    let buffer_clone = buffer.clone();
    let tx_clone = tx.clone();

    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut buf = buffer_clone.lock().unwrap();
            if channels > 1 {
                for chunk in data.chunks(channels) {
                    let mono: f32 = chunk.iter().sum::<f32>() / channels as f32;
                    buf.push(mono);
                }
            } else {
                buf.extend_from_slice(data);
            }

            if buf.len() >= chunk_samples {
                let samples: Vec<f32> = buf.drain(..).collect();
                let _ = tx_clone.send(AudioChunk {
                    samples,
                    sample_rate,
                    is_microphone: true,
                });
            }
        },
        |err| {
            log::error!("Microphone stream error: {}", err);
        },
        None,
    )?;

    stream.play()?;

    // Keep stream alive until recording stops
    while *is_recording.lock().unwrap() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    drop(stream);
    Ok(())
}

fn run_system_audio_capture(
    _tx: mpsc::UnboundedSender<AudioChunk>,
    is_recording: Arc<Mutex<bool>>,
) -> anyhow::Result<()> {
    log::info!("System audio capture thread started (ScreenCaptureKit integration placeholder)");

    while *is_recording.lock().unwrap() {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    Ok(())
}
