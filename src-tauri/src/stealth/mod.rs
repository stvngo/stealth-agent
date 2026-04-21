pub mod screen_share;
pub mod focus;
pub mod process;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

use tauri::WebviewWindow;

pub fn apply_all_stealth(window: &WebviewWindow) -> anyhow::Result<()> {
    screen_share::exclude_from_capture(window)?;
    focus::make_non_activating(window)?;
    process::disguise_process()?;

    // On macOS, swap the window class to NSPanel + nonactivating style so we
    // can hover over full-screen apps. Then kick off a periodic main-thread
    // loop that keeps level + collection behavior pinned (system resets them
    // on Space changes / full-screen transitions).
    #[cfg(target_os = "macos")]
    {
        macos::convert_windows_to_panels();
        macos::start_hover_reassertion_loop();
    }

    log::info!("All stealth layers applied successfully");
    Ok(())
}
