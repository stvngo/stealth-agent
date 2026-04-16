use crate::ai::{client::AiClient, context};
use crate::audio::capture::AudioCapture;
use crate::audio::transcribe::TranscriptionEngine;
use crate::screen::capture;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Emitter, State};
use tokio::sync::mpsc;

pub struct AppState {
    pub transcription: Mutex<TranscriptionEngine>,
    pub audio_capture: Mutex<AudioCapture>,
    pub chat_history: Mutex<Vec<(String, String)>>,
    pub resume_text: Mutex<Option<String>>,
    pub api_key: Mutex<Option<String>>,
    pub model: Mutex<String>,
    pub base_url: Mutex<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            transcription: Mutex::new(TranscriptionEngine::new(None)),
            audio_capture: Mutex::new(AudioCapture::new()),
            chat_history: Mutex::new(Vec::new()),
            resume_text: Mutex::new(None),
            api_key: Mutex::new(None),
            model: Mutex::new("gpt-4o".to_string()),
            base_url: Mutex::new("https://api.openai.com/v1".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordingStatus {
    pub is_recording: bool,
}

#[tauri::command]
pub async fn take_screenshot() -> Result<String, String> {
    capture::take_screenshot().map_err(|e| e.to_string())
}

/// Start audio recording (mic + system audio) and transcription.
#[tauri::command]
pub async fn start_recording(
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<RecordingStatus, String> {
    let rx = state
        .audio_capture
        .lock()
        .unwrap()
        .start()
        .map_err(|e| e.to_string())?;

    let api_key = state.api_key.lock().unwrap().clone();
    let transcription = TranscriptionEngine::new(api_key);
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();

    let window_clone = window.clone();
    tokio::spawn(async move {
        while let Some(entry) = event_rx.recv().await {
            let _ = window_clone.emit("transcript-entry", &entry);
        }
    });

    tokio::spawn(async move {
        transcription.process_audio_stream(rx, event_tx).await;
    });

    Ok(RecordingStatus { is_recording: true })
}

/// Stop audio recording.
#[tauri::command]
pub async fn stop_recording(state: State<'_, AppState>) -> Result<RecordingStatus, String> {
    state.audio_capture.lock().unwrap().stop();
    Ok(RecordingStatus { is_recording: false })
}

/// Get whether audio is currently recording.
#[tauri::command]
pub async fn get_recording_status(state: State<'_, AppState>) -> Result<RecordingStatus, String> {
    let is_recording = state.audio_capture.lock().unwrap().is_recording();
    Ok(RecordingStatus { is_recording })
}

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    message: String,
    include_screenshot: bool,
    window: tauri::Window,
) -> Result<(), String> {
    let api_key = state.api_key.lock().unwrap().clone();
    let api_key = api_key.ok_or("API key not set. Go to Settings to configure.")?;

    let model = state.model.lock().unwrap().clone();
    let base_url = state.base_url.lock().unwrap().clone();

    let screenshot_b64 = if include_screenshot {
        capture::take_screenshot().ok()
    } else {
        None
    };

    let transcript = state
        .transcription
        .lock()
        .unwrap()
        .get_transcript_text(50);

    let resume = state.resume_text.lock().unwrap().clone();
    let history = state.chat_history.lock().unwrap().clone();

    let messages = context::build_messages(
        &transcript,
        screenshot_b64.as_deref(),
        resume.as_deref(),
        &history,
        &message,
    );

    let client = AiClient::new(api_key, Some(model), Some(base_url));
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let window_clone = window.clone();
    tokio::spawn(async move {
        let mut full_response = String::new();
        while let Some(token) = rx.recv().await {
            full_response.push_str(&token);
            let _ = window_clone.emit_to("main", "ai-token", &token);
        }
        let _ = window_clone.emit_to("main", "ai-done", &full_response);
    });

    tokio::spawn(async move {
        if let Err(e) = client.chat_stream(messages, tx).await {
            log::error!("AI stream error: {}", e);
            let _ = window.emit_to("main", "ai-error", e.to_string());
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn set_config(
    state: State<'_, AppState>,
    api_key: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
    resume: Option<String>,
) -> Result<(), String> {
    if let Some(ref key) = api_key {
        *state.api_key.lock().unwrap() = Some(key.clone());
        state.transcription.lock().unwrap().set_api_key(key.clone());
    }
    if let Some(m) = model {
        *state.model.lock().unwrap() = m;
    }
    if let Some(url) = base_url {
        *state.base_url.lock().unwrap() = url;
    }
    if let Some(r) = resume {
        *state.resume_text.lock().unwrap() = Some(r);
    }
    Ok(())
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let has_key = state.api_key.lock().unwrap().is_some();
    let model = state.model.lock().unwrap().clone();
    let base_url = state.base_url.lock().unwrap().clone();
    let has_resume = state.resume_text.lock().unwrap().is_some();

    Ok(serde_json::json!({
        "hasApiKey": has_key,
        "model": model,
        "baseUrl": base_url,
        "hasResume": has_resume,
    }))
}

#[tauri::command]
pub async fn get_transcript(
    state: State<'_, AppState>,
) -> Result<Vec<crate::audio::TranscriptEntry>, String> {
    Ok(state.transcription.lock().unwrap().get_transcript())
}

#[tauri::command]
pub async fn add_transcript_entry(
    state: State<'_, AppState>,
    speaker: String,
    text: String,
) -> Result<(), String> {
    let speaker = match speaker.to_lowercase().as_str() {
        "me" => crate::audio::Speaker::Me,
        "interviewer" => crate::audio::Speaker::Interviewer,
        _ => crate::audio::Speaker::Unknown,
    };
    state
        .transcription
        .lock()
        .unwrap()
        .add_manual_entry(speaker, text);
    Ok(())
}

#[tauri::command]
pub async fn move_window(window: tauri::Window, dx: f64, dy: f64) -> Result<(), String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    let new_x = pos.x as f64 + dx;
    let new_y = pos.y as f64 + dy;
    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: new_x as i32,
            y: new_y as i32,
        }))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn toggle_visibility(window: tauri::Window) -> Result<bool, String> {
    let visible = window.is_visible().map_err(|e| e.to_string())?;
    if visible {
        window.hide().map_err(|e| e.to_string())?;
    } else {
        window.show().map_err(|e| e.to_string())?;
    }
    Ok(!visible)
}
