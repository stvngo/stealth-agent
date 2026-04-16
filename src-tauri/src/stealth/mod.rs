pub mod screen_share;
pub mod focus;
pub mod process;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

use tauri::WebviewWindow;

pub fn apply_all_stealth(window: &WebviewWindow) -> anyhow::Result<()> {
    // Dock-hiding changes activation policy, which can demote window z-order.
    // Must happen BEFORE we set window levels so macOS doesn't undo them.
    process::disguise_process()?;
    screen_share::exclude_from_capture(window)?;
    focus::make_non_activating(window)?;
    log::info!("All stealth layers applied successfully");
    Ok(())
}
