use tauri::WebviewWindow;

/// Exclude the window from screen capture / screen sharing.
/// Uses NSWindowSharingType.none on macOS,
/// SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE) on Windows.
pub fn exclude_from_capture(window: &WebviewWindow) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        super::macos::set_sharing_type_none(window)?;
    }

    #[cfg(target_os = "windows")]
    {
        super::windows::set_display_affinity_exclude(window)?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = window;
        log::warn!("Screen share exclusion not implemented for this OS");
    }

    Ok(())
}
