/// Disguise the process so it doesn't appear recognizable in Activity Monitor / Task Manager.
pub fn disguise_process() -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        super::macos::hide_from_dock()?;
    }

    #[cfg(target_os = "windows")]
    {
        log::info!("Windows process disguise: executable name should be set at build time");
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        log::warn!("Process disguise not implemented for this OS");
    }

    Ok(())
}
