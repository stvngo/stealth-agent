use base64::Engine;
use std::process::Command;

/// Capture a screenshot of the entire screen and return it as a base64-encoded PNG.
pub fn take_screenshot() -> anyhow::Result<String> {
    #[cfg(target_os = "macos")]
    {
        return take_screenshot_macos();
    }

    #[cfg(target_os = "windows")]
    {
        return take_screenshot_windows();
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        anyhow::bail!("Screenshot not implemented for this OS");
    }
}

#[cfg(target_os = "macos")]
fn take_screenshot_macos() -> anyhow::Result<String> {
    let tmp_path = std::env::temp_dir().join(format!("ia_screenshot_{}.png", uuid::Uuid::new_v4()));
    let tmp_str = tmp_path.to_string_lossy().to_string();

    let output = Command::new("screencapture")
        .args(["-x", "-C", "-t", "png", &tmp_str])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "screencapture failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let bytes = std::fs::read(&tmp_path)?;
    let _ = std::fs::remove_file(&tmp_path);

    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    log::info!("Screenshot captured ({} bytes)", bytes.len());
    Ok(b64)
}

#[cfg(target_os = "windows")]
fn take_screenshot_windows() -> anyhow::Result<String> {
    // On Windows, use PowerShell or a native API to capture screen
    // Placeholder: in production use win32 API via windows-rs
    anyhow::bail!("Windows screenshot not yet implemented -- use win32 BitBlt or DXGI")
}
