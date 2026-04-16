mod ai;
mod audio;
mod commands;
mod screen;
mod stealth;

use commands::AppState;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::take_screenshot,
            commands::send_message,
            commands::set_config,
            commands::get_config,
            commands::get_transcript,
            commands::add_transcript_entry,
            commands::move_window,
            commands::toggle_visibility,
            commands::start_recording,
            commands::stop_recording,
            commands::get_recording_status,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main")
                .expect("Failed to get main window");

            // Apply all stealth layers
            if let Err(e) = stealth::apply_all_stealth(&window) {
                log::error!("Failed to apply stealth: {}", e);
            }

            register_global_shortcuts(app)?;

            log::info!("Invisible Agent started successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn register_global_shortcuts(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut_toggle = Shortcut::new(Some(Modifiers::SUPER), Code::KeyB);
    let shortcut_screenshot = Shortcut::new(Some(Modifiers::SUPER), Code::Backquote);
    let shortcut_up = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::ArrowUp);
    let shortcut_down = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::ArrowDown);
    let shortcut_left = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::ArrowLeft);
    let shortcut_right = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::ArrowRight);

    app.global_shortcut().on_shortcuts(
        [shortcut_toggle, shortcut_screenshot, shortcut_up, shortcut_down, shortcut_left, shortcut_right],
        move |app_handle, shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }

            let Some(window) = app_handle.get_webview_window("main") else {
                return;
            };

            if shortcut == &shortcut_toggle {
                let visible = window.is_visible().unwrap_or(true);
                if visible {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    // Re-apply stealth after showing -- macOS may reset window
                    // properties when a window is shown again
                    if let Err(e) = stealth::apply_all_stealth(&window) {
                        log::error!("Failed to re-apply stealth after show: {}", e);
                    }
                }
            } else if shortcut == &shortcut_screenshot {
                let _ = window.emit_to("main", "trigger-screenshot", ());
            } else {
                let (dx, dy) = if shortcut == &shortcut_up {
                    (0.0, -50.0)
                } else if shortcut == &shortcut_down {
                    (0.0, 50.0)
                } else if shortcut == &shortcut_left {
                    (-50.0, 0.0)
                } else {
                    (50.0, 0.0)
                };

                if let Ok(pos) = window.outer_position() {
                    let _ = window.set_position(tauri::Position::Physical(
                        tauri::PhysicalPosition {
                            x: pos.x + dx as i32,
                            y: pos.y + dy as i32,
                        },
                    ));
                }
            }
        },
    )?;

    log::info!("Global shortcuts registered: Cmd+B (toggle), Cmd+` (screenshot), Cmd+Shift+Arrows (move)");
    Ok(())
}
