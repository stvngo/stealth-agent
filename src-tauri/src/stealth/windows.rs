use tauri::WebviewWindow;

/// Set WDA_EXCLUDEFROMCAPTURE to hide from screen capture on Windows.
#[cfg(target_os = "windows")]
pub fn set_display_affinity_exclude(window: &WebviewWindow) -> anyhow::Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE};

    let hwnd = window.hwnd().map_err(|e| anyhow::anyhow!("Failed to get HWND: {}", e))?;
    unsafe {
        SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)
            .map_err(|e| anyhow::anyhow!("SetWindowDisplayAffinity failed: {}", e))?;
    }
    log::info!("Set WDA_EXCLUDEFROMCAPTURE -- invisible to screen sharing on Windows");
    Ok(())
}

/// Set WS_EX_NOACTIVATE so the window never steals focus on Windows.
#[cfg(target_os = "windows")]
pub fn set_no_activate(window: &WebviewWindow) -> anyhow::Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TOPMOST,
    };

    let hwnd = window.hwnd().map_err(|e| anyhow::anyhow!("Failed to get HWND: {}", e))?;
    unsafe {
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        SetWindowLongW(
            hwnd,
            GWL_EXSTYLE,
            ex_style | WS_EX_NOACTIVATE.0 as i32 | WS_EX_TOPMOST.0 as i32,
        );
    }
    log::info!("Set WS_EX_NOACTIVATE -- window won't steal focus on Windows");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn set_display_affinity_exclude(_window: &WebviewWindow) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn set_no_activate(_window: &WebviewWindow) -> anyhow::Result<()> {
    Ok(())
}
