use tauri::WebviewWindow;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSWindow,
    NSWindowCollectionBehavior, NSWindowSharingType,
};
use objc2_foundation::MainThreadMarker;
use std::ffi::CStr;

// Objective-C runtime fn for swapping a live object's class.
// https://developer.apple.com/documentation/objectivec/1418905-object_setclass
extern "C" {
    fn object_setClass(obj: *mut AnyObject, cls: *const AnyClass) -> *const AnyClass;
}

/// NSWindowStyleMaskNonactivatingPanel = 1 << 7 = 128.
/// Only valid on an NSPanel. Tells AppKit: this window never becomes key
/// when clicked, never activates the app, behaves like a HUD.
const NS_WINDOW_STYLE_MASK_NON_ACTIVATING_PANEL: u64 = 1 << 7;

/// Apply screen-share exclusion to ALL windows belonging to this application.
/// Instead of trying to find the "right" NSWindow through Tauri's webview handle
/// (which is fragile), we iterate every window the app owns and mark them all
/// as excluded from capture via NSWindowSharingType.none.
pub fn set_sharing_type_none(_window: &WebviewWindow) -> anyhow::Result<()> {
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| anyhow::anyhow!("Must be called from main thread"))?;

    apply_sharing_none_to_all_windows(mtm);

    // Tauri may create/reconfigure windows after our initial setup
    // (e.g., applying transparency via macOSPrivateApi). Re-apply after delays.
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(500));
        unsafe {
            dispatch_async_main(|| {
                if let Some(mtm) = MainThreadMarker::new() {
                    apply_sharing_none_to_all_windows(mtm);
                    log::info!("Re-applied NSWindowSharingType.none after 500ms");
                }
            });
        }
    });

    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(2000));
        unsafe {
            dispatch_async_main(|| {
                if let Some(mtm) = MainThreadMarker::new() {
                    apply_sharing_none_to_all_windows(mtm);
                    log::info!("Re-applied NSWindowSharingType.none after 2000ms");
                }
            });
        }
    });

    Ok(())
}

fn apply_sharing_none_to_all_windows(mtm: MainThreadMarker) {
    let app = NSApplication::sharedApplication(mtm);
    let windows = app.windows();
    let count = windows.count();

    for i in 0..count {
        let ns_window: Retained<NSWindow> =
            unsafe { objc2::msg_send![&*windows, objectAtIndex: i] };

        // After `object_setClass` swap to NSPanel, AppKit and the WindowServer
        // can end up out of sync re: sharingType -- setSharingType(None) is a
        // no-op from AppKit's perspective because the client-side ivar already
        // says "None". Force a real state change every cycle: ReadOnly -> None.
        // This makes AppKit emit a fresh NSWindowServerCommunicationSkein
        // update and Zoom / ScreenCaptureKit see the current value.
        ns_window.setSharingType(NSWindowSharingType::ReadOnly);
        ns_window.setSharingType(NSWindowSharingType::None);

        // OR-in cross-space flags without touching Tauri/system defaults.
        // CanJoinAllSpaces     -> appears on every Space
        // FullScreenAuxiliary  -> allowed to render over full-screen apps
        let mut behavior = ns_window.collectionBehavior();
        behavior |= NSWindowCollectionBehavior::CanJoinAllSpaces;
        behavior |= NSWindowCollectionBehavior::FullScreenAuxiliary;
        ns_window.setCollectionBehavior(behavior);
    }
}

/// Configure every app window to hover above all other app windows (system HUD
/// level), never hide on deactivation, and immediately reorder to the front.
pub fn set_non_activating_panel(_window: &WebviewWindow) -> anyhow::Result<()> {
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| anyhow::anyhow!("Must be called from main thread"))?;

    apply_hover_level_to_all_windows(mtm);

    // Re-apply at 500ms and 2000ms to catch any windows Tauri configures later
    // (e.g. the webview container Tauri recreates after applying transparency).
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(500));
        unsafe {
            dispatch_async_main(|| {
                if let Some(mtm) = MainThreadMarker::new() {
                    apply_hover_level_to_all_windows(mtm);
                    log::info!("Re-applied hover level after 500ms");
                }
            });
        }
    });

    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(2000));
        unsafe {
            dispatch_async_main(|| {
                if let Some(mtm) = MainThreadMarker::new() {
                    apply_hover_level_to_all_windows(mtm);
                    log::info!("Re-applied hover level after 2000ms");
                }
            });
        }
    });

    Ok(())
}

fn apply_hover_level_to_all_windows(mtm: MainThreadMarker) {
    let app = NSApplication::sharedApplication(mtm);
    let windows = app.windows();
    let count = windows.count();

    let desired_level =
        unsafe { CGWindowLevelForKey(K_CG_SCREEN_SAVER_WINDOW_LEVEL_KEY) } as isize;

    for i in 0..count {
        let ns_window: Retained<NSWindow> =
            unsafe { objc2::msg_send![&*windows, objectAtIndex: i] };

        ns_window.setLevel(desired_level);

        unsafe {
            let _: () = objc2::msg_send![&*ns_window, setHidesOnDeactivate: false];
            let _: () = objc2::msg_send![&*ns_window, orderFrontRegardless];
        }
    }
}

/// Swap every app window from NSWindow to NSPanel and give it the
/// `nonactivatingPanel` style mask. This is the ingredient that lets the
/// window actually *float over* full-screen apps -- NSWindow cannot, NSPanel
/// can, regardless of level/collection-behavior settings. This is what
/// Electron's `new BrowserWindow({ type: 'panel' })` does under the hood and
/// what Cluely / Interview Coder rely on.
///
/// Idempotent: if the window is already an NSPanel we skip the class swap.
pub fn convert_windows_to_panels() {
    let Some(mtm) = MainThreadMarker::new() else {
        log::error!("convert_windows_to_panels called off main thread");
        return;
    };

    let panel_class = unsafe { CStr::from_bytes_with_nul_unchecked(b"NSPanel\0") };
    let Some(panel_cls) = AnyClass::get(panel_class) else {
        log::error!("Failed to look up NSPanel class");
        return;
    };

    let app = NSApplication::sharedApplication(mtm);
    let windows = app.windows();
    let count = windows.count();
    let mut converted = 0usize;

    for i in 0..count {
        let ns_window: Retained<NSWindow> =
            unsafe { objc2::msg_send![&*windows, objectAtIndex: i] };

        let current_cls: *const AnyClass = unsafe {
            objc2::msg_send![&*ns_window, class]
        };

        if current_cls != panel_cls as *const _ {
            let raw = (&*ns_window as *const NSWindow) as *mut AnyObject;
            unsafe {
                object_setClass(raw, panel_cls as *const _);
            }
            converted += 1;
        }

        // OR-in the nonactivating-panel style mask so clicking our window
        // never activates our app or steals focus.
        unsafe {
            let current_mask: u64 = objc2::msg_send![&*ns_window, styleMask];
            let new_mask = current_mask | NS_WINDOW_STYLE_MASK_NON_ACTIVATING_PANEL;
            if new_mask != current_mask {
                let _: () = objc2::msg_send![&*ns_window, setStyleMask: new_mask];
            }

            // Mark as a floating panel (stays above regular windows within its level).
            let _: () = objc2::msg_send![&*ns_window, setFloatingPanel: true];
            // Becomes key only when it genuinely needs keyboard input,
            // not on every click.
            let _: () = objc2::msg_send![&*ns_window, setBecomesKeyOnlyIfNeeded: true];
        }
    }

    if converted > 0 {
        log::info!(
            "Converted {} NSWindow(s) -> NSPanel with nonactivating style",
            converted
        );
    }
}

/// One-shot read-back of the state of every app window. Used at startup to
/// confirm our level + collection behavior actually took effect.
pub fn log_window_state_for_debugging() {
    let Some(mtm) = MainThreadMarker::new() else {
        log::error!("log_window_state_for_debugging called off main thread");
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let windows = app.windows();
    let count = windows.count();
    log::info!("=== Hover diagnostic: {} app window(s) ===", count);
    for i in 0..count {
        let ns_window: Retained<NSWindow> =
            unsafe { objc2::msg_send![&*windows, objectAtIndex: i] };
        let level: isize = unsafe { objc2::msg_send![&*ns_window, level] };
        let behavior_raw: u64 = ns_window.collectionBehavior().0 as u64;
        let has_can_join_all = behavior_raw & 1 != 0;
        let has_full_screen_aux = behavior_raw & (1 << 8) != 0;
        let style_mask: u64 = unsafe { objc2::msg_send![&*ns_window, styleMask] };
        let is_nonactivating_panel =
            style_mask & NS_WINDOW_STYLE_MASK_NON_ACTIVATING_PANEL != 0;
        let cls: *const AnyClass = unsafe { objc2::msg_send![&*ns_window, class] };
        let cls_name = unsafe { (*cls).name().to_string_lossy().into_owned() };
        let sharing_type: u64 = unsafe { objc2::msg_send![&*ns_window, sharingType] };
        let sharing_name = match sharing_type {
            0 => "None",
            1 => "ReadOnly",
            2 => "ReadWrite",
            _ => "Unknown",
        };
        log::info!(
            "  window[{i}] class={} level={level} sharingType={}({}) collectionBehavior=0x{:x} styleMask=0x{:x} (canJoinAllSpaces={}, fullScreenAux={}, nonActivatingPanel={})",
            cls_name, sharing_name, sharing_type, behavior_raw, style_mask, has_can_join_all, has_full_screen_aux, is_nonactivating_panel
        );
    }
}

/// Start a background thread that continuously re-asserts the hover level +
/// collection behavior + sharing=none on every window, every 400ms, forever.
///
/// Rationale: macOS and/or Tauri reset window properties at events we don't
/// directly observe -- Space changes, app activation, full-screen transitions,
/// etc. Rather than hook every one of those notifications (which requires
/// Objective-C block callbacks from Rust), we just re-assert the properties
/// on a fast cadence. The operations are idempotent and cheap (a few
/// setter calls on a handful of NSWindow objects), so the overhead is
/// negligible.
///
/// This is started once at app launch via `apply_all_stealth`.
pub fn start_hover_reassertion_loop() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::spawn(|| {
        let mut tick: u64 = 0;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(250));
            tick += 1;
            let should_log = tick % 20 == 0; // ~every 5 seconds
            unsafe {
                dispatch_async_main(move || {
                    if let Some(mtm) = MainThreadMarker::new() {
                        // Order matters: class swap / style mask / level first,
                        // *then* sharingType last, so panel-related setters
                        // cannot stomp on the sharing state afterwards.
                        convert_windows_to_panels();
                        apply_hover_level_to_all_windows(mtm);
                        apply_sharing_none_to_all_windows(mtm);
                        if should_log {
                            log_window_state_for_debugging();
                        }
                    }
                });
            }
        }
    });

    log::info!("Hover reassertion loop started (250ms cadence)");
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

#[allow(dead_code)]
const K_CG_FLOATING_WINDOW_LEVEL_KEY: i32 = 5;
#[allow(dead_code)]
const K_CG_STATUS_WINDOW_LEVEL_KEY: i32 = 9;
/// Screen-saver level. Higher than *every* normal window including full-screen
/// apps. This is what Electron calls "screen-saver" level and is the level
/// Cluely / Interview Coder / Bartender / Rectangle use to hover over
/// full-screen Chrome, Zoom, etc.
const K_CG_SCREEN_SAVER_WINDOW_LEVEL_KEY: i32 = 13;

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
