use tauri::WebviewWindow;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSWindow,
    NSWindowCollectionBehavior, NSWindowSharingType,
};
use objc2_foundation::MainThreadMarker;

/// Apply screen-share exclusion to ALL windows belonging to this application.
/// Instead of trying to find the "right" NSWindow through Tauri's webview handle
/// (which is fragile), we iterate every window the app owns and mark them all
/// as excluded from capture via NSWindowSharingType.none.
pub fn set_sharing_type_none(_window: &WebviewWindow) -> anyhow::Result<()> {
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| anyhow::anyhow!("Must be called from main thread"))?;

    apply_stealth_to_all_windows(mtm);

    // Tauri may create/reconfigure windows after our initial setup
    // (e.g., applying transparency via macOSPrivateApi). Re-apply after delays.
    for delay_ms in [500u64, 2000, 5000] {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
            unsafe {
                dispatch_async_main(move || {
                    if let Some(mtm) = MainThreadMarker::new() {
                        apply_stealth_to_all_windows(mtm);
                        log::info!("Re-applied stealth after {}ms", delay_ms);
                    }
                });
            }
        });
    }

    Ok(())
}

fn apply_stealth_to_all_windows(mtm: MainThreadMarker) {
    let app = NSApplication::sharedApplication(mtm);
    let windows = app.windows();
    let count = windows.count();
    let level = unsafe { CGWindowLevelForKey(K_CG_STATUS_WINDOW_LEVEL_KEY) };

    for i in 0..count {
        let ns_window: Retained<NSWindow> =
            unsafe { objc2::msg_send![&*windows, objectAtIndex: i] };

        ns_window.setSharingType(NSWindowSharingType::None);

        let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::IgnoresCycle
            | NSWindowCollectionBehavior::FullScreenAuxiliary;
        ns_window.setCollectionBehavior(behavior);

        ns_window.setLevel(level as isize);
        unsafe {
            let _: () = objc2::msg_send![&*ns_window, setHidesOnDeactivate: false];
            let _: () = objc2::msg_send![&*ns_window, orderFrontRegardless];
        }
    }

    log::info!(
        "Applied full stealth to {} window(s) (level={}, sharing=none)",
        count,
        level
    );
}

/// Configure the window as a non-activating floating panel.
/// Level + hidesOnDeactivate are already handled by apply_stealth_to_all_windows,
/// but we reinforce them here for robustness.
pub fn set_non_activating_panel(_window: &WebviewWindow) -> anyhow::Result<()> {
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| anyhow::anyhow!("Must be called from main thread"))?;
    apply_stealth_to_all_windows(mtm);
    log::info!("Non-activating panel properties reinforced");
    Ok(())
}

/// Hide from Dock and Cmd+Tab by setting activation policy to .accessory
pub fn hide_from_dock() -> anyhow::Result<()> {
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| anyhow::anyhow!("Must be called from main thread"))?;
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    log::info!("Hidden from Dock and Cmd+Tab (activation policy = accessory)");
    Ok(())
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowLevelForKey(key: i32) -> i32;
}

const K_CG_FLOATING_WINDOW_LEVEL_KEY: i32 = 5;
const K_CG_STATUS_WINDOW_LEVEL_KEY: i32 = 9;

/// Dispatch a closure to the main thread via GCD.
/// NSWindow operations must happen on the main thread, but our re-application
/// timers fire from background threads.
unsafe fn dispatch_async_main<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    let queue = dispatch2::DispatchQueue::main();
    queue.exec_async(f);
}
