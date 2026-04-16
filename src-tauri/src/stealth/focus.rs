use tauri::WebviewWindow;

/// Make the window non-activating so it never steals focus from the browser.
/// Interview platforms detect window.blur events -- this prevents triggering them.
pub fn make_non_activating(window: &WebviewWindow) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        super::macos::set_non_activating_panel(window)?;
    }

    #[cfg(target_os = "windows")]
    {
        super::windows::set_no_activate(window)?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = window;
        log::warn!("Non-activating window not implemented for this OS");
    }

    Ok(())
}
