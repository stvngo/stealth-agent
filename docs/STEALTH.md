# Stealth Architecture

This document explains how Invisible Agent stays hidden from screen-capture while remaining visible to the user, and the specific engineering problems we had to solve to get there — most notably, making a Tauri window reliably hover over full-screen / maximized apps on macOS.

The target behavior is what Cluely and Interview Coder call "undetectability":

1. **Invisible to screen capture.** Zoom, Meet, Teams, QuickTime, `screencapture`, and ScreenCaptureKit all see a black / empty rectangle where our panel is, or skip it entirely.
2. **Visible to the user, anywhere.** The panel floats on top of every other window — regular, maximized, or full-screen — and follows the user across Mission Control Spaces.
3. **Undetectable by browser heuristics.** Interviewers can't infer our presence from `window.blur`, `visibilitychange`, focus loss, mouse leave, or process lists.

## High-level design

We treat stealth as a stack of independent layers. Each layer handles one detection vector. They are composed in `src-tauri/src/stealth/mod.rs::apply_all_stealth` and re-applied on every `window.show()` so nothing silently drifts.

```
┌────────────────────────────────────────────────────────────────┐
│  Layer 5: Watchdog reassertion loop   (macos.rs)               │
│  Layer 4: NSPanel conversion + floating / non-activating style │
│  Layer 3: Window level + cross-Space collection behavior       │
│  Layer 2: No focus stealing (non-activating panel, WS_EX_NOACTIVATE)
│  Layer 1: Capture exclusion (sharingType=None / WDA_EXCLUDEFROMCAPTURE)
│  Layer 0: Process disguise (LSUIElement, accessory activation policy)
└────────────────────────────────────────────────────────────────┘
```

Tauri gives us a `WebviewWindow`. Underneath, on macOS, that's a plain `NSWindow`. Tauri's public API doesn't expose everything we need, so the macOS layer drops down to `objc2` and AppKit's Objective-C runtime for the critical pieces.

## Why Tauri?

Most stealth overlays are Electron. Tauri was chosen for three reasons:

1. **Smaller, less fingerprintable process.** Electron apps ship a Chromium runtime — obvious in Activity Monitor and process trees. Tauri uses the OS's native WebView (`WKWebView` on macOS, WebView2 on Windows), so the process is small and generic-looking.
2. **Full native access in Rust.** We can call `objc2::msg_send![]` into AppKit directly, manipulate `NSWindow` internals, and dynamically swap the window's Objective-C class — none of which is exposed cleanly through Electron's `BrowserWindow` options.
3. **`macOSPrivateApi` switch.** Setting `app.macOSPrivateApi = true` in `tauri.conf.json` lets us use `NSWindowSharingType.none` and other APIs that would otherwise be blocked, and enables window transparency.

## Layer 0 — Process disguise

Detection vector: Dock icons, Cmd+Tab, taskbar, process listings.

- `Info.plist` sets `LSUIElement = true` → no Dock icon, no Cmd+Tab entry, no menu bar.
- `NSApplication.setActivationPolicy(.accessory)` is called at startup so the app is an accessory process.
- On Windows, `skipTaskbar: true` in `tauri.conf.json` removes us from the taskbar.

These don't stop screen capture, but they remove the easy "is there a suspicious app running?" checks.

## Layer 1 — Capture exclusion

Detection vector: Zoom / Meet / Teams / QuickTime / ScreenCaptureKit see the window.

### macOS
Every `NSWindow` has a `sharingType` property. We iterate `NSApp.windows` and pin each one to `NSWindowSharingType.none`. Any capture API — ScreenCaptureKit, `CGWindowListCreateImage`, the legacy `CGDisplayStream` — excludes windows flagged this way.

```rust
ns_window.setSharingType(NSWindowSharingType::None);
```

Important subtlety: `setSharingType` is a no-op at the AppKit layer if the ivar already matches. After we swap the window's class (Layer 4), AppKit and the WindowServer can end up out of sync: the client thinks `sharingType = None` but the server thinks otherwise. To force a real state change, we occasionally round-trip through `ReadOnly → None` — but only when we detect drift (`current_sharing != None`). Doing it unconditionally every tick caused a compositor pass that made the WebView `backdrop-filter` flicker.

### Windows
`SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)` — the Win10 2004+ equivalent.

## Layer 2 — No focus stealing

Detection vector: Browser-based heuristics. If a site registers `window.onblur` / `document.visibilitychange`, any app that steals focus when clicked becomes detectable.

- macOS: We use `NSWindowStyleMask.nonactivatingPanel` (`1 << 7`) + `setBecomesKeyOnlyIfNeeded: true`. Clicks go through to the panel but don't activate our app.
- macOS: `setHidesOnDeactivate: false` so Cmd+Tab-ing away doesn't hide us.
- Windows: `WS_EX_NOACTIVATE` + `WS_EX_TOOLWINDOW` on the extended style.

Result: clicking inside the panel does not cause a blur event on the interviewer's Chrome tab. This is the single biggest signal Cluely and Interview Coder exploit, and the one most naive overlays get wrong.

## Layer 3 — Window level & cross-Space visibility

Detection vector: None — but without this layer, the panel isn't useful because it doesn't hover.

- Window level is promoted to `kCGScreenSaverWindowLevelKey` (value `1000`) via `CGWindowLevelForKey`. This is above normal windows, above the Dock, above menu bar HUDs.
- `NSWindowCollectionBehavior` has `CanJoinAllSpaces | FullScreenAuxiliary` **OR-ed in** (not replaced — Tauri sets defaults we want to preserve). `CanJoinAllSpaces` makes the panel follow the user across Spaces; `FullScreenAuxiliary` is what allows rendering over full-screen apps.

Both settings are idempotent and only re-applied if they've been cleared.

## Layer 4 — The problem that matters: hovering over full-screen / maximized apps

This was the single hardest problem in the whole project, and it's the one most "always-on-top" libraries — including Tauri's own `set_always_on_top` — fail at on macOS.

### Symptom

After the basics (screen-saver window level + CanJoinAllSpaces + FullScreenAuxiliary) were in place, the panel:

- Floated fine above regular windows.
- **Disappeared the moment the user switched to any maximized / full-screen window** (full-screen Chrome, Zoom full-screen, IDEs in full-screen).
- Reappeared only when the user returned to the desktop.

The diagnostic logger confirmed `level=1000, canJoinAllSpaces=true, fullScreenAux=true` at all times. So why was the panel not visible in full-screen Spaces?

### Root cause

macOS treats each full-screen app as its **own Space**. A regular `NSWindow` — regardless of level or collection behavior — is anchored to a user Space. When the user enters a full-screen app's Space, the WindowServer doesn't render non-panel windows from other Spaces on top, even if they technically have a higher window level. `FullScreenAuxiliary` is necessary, but **not sufficient**.

The window class itself matters. `NSPanel` windows are auxiliary UI by design — tool palettes, inspectors, HUDs. The WindowServer treats them specially:

- They can join full-screen Spaces as overlays.
- They respect `FullScreenAuxiliary` the way the docs say they should.
- They don't steal focus with the right style mask.

Cluely, Interview Coder, and every AppKit floating palette use `NSPanel`, not `NSWindow`.

Tauri / `tao` create plain `NSWindow`s. There's no option to create an `NSPanel` instead.

### Solution: dynamic class swap at runtime

Objective-C's runtime lets us change an object's class after construction. We use `object_setClass(windowPtr, NSPanelClass)` to reclassify the Tauri window as an `NSPanel` — then OR in `nonactivatingPanel` on its style mask, call `setFloatingPanel: true`, and `setBecomesKeyOnlyIfNeeded: true`.

```rust
extern "C" {
    fn object_setClass(obj: *mut AnyObject, cls: *const AnyClass) -> *const AnyClass;
}

let panel_cls = AnyClass::get(c"NSPanel").unwrap();
let raw = (&*ns_window as *const NSWindow) as *mut AnyObject;
unsafe { object_setClass(raw, panel_cls as *const _); }

let current_mask: u64 = objc2::msg_send![&*ns_window, styleMask];
let _: () = objc2::msg_send![&*ns_window, setStyleMask: current_mask | (1u64 << 7)];
let _: () = objc2::msg_send![&*ns_window, setFloatingPanel: true];
let _: () = objc2::msg_send![&*ns_window, setBecomesKeyOnlyIfNeeded: true];
```

Once the window is an `NSPanel` and has the three collection / level / style bits in place, it hovers over everything — full-screen Chrome, full-screen Zoom, the lot.

### Secondary problem: invisibility regression after class swap

The first working version of the class swap broke capture exclusion — Zoom started seeing the panel again. The class swap invalidated the WindowServer's cached sharingType for the window. Calling `setSharingType(None)` was a client-side no-op because the AppKit ivar already said "None".

Fix: explicitly round-trip `ReadOnly → None` after any class change. This forces AppKit to flush a fresh `NSWindowServerCommunication` update to the WindowServer, and the window becomes invisible to capture again.

## Layer 5 — Watchdog reassertion loop

macOS resets window properties on events we don't directly observe: Space changes, app activation, full-screen transitions, sometimes even Dock events. Rather than subscribe to every notification (painful from Rust — requires Objective-C blocks), we run a tight poll.

Design:

- Dedicated Rust thread sleeps 250 ms between ticks.
- Each tick dispatches onto the main thread via `dispatch2::DispatchQueue::main()` and walks `NSApp.windows`.
- **Every setter is idempotent**: we read the current value, and only call the setter if the value has actually drifted from the desired state.
- `orderFrontRegardless` is **not** called during steady state — only during the one-shot init and the delayed 500ms / 2000ms re-applies. Calling it every tick forces a z-order refresh and composition flash.
- Once every 20 ticks (~5 s) we log the state of every window (class, level, collectionBehavior, styleMask, sharingType) for diagnostics.

```rust
// pseudo-code
loop {
    sleep(250ms);
    dispatch_main(|| {
        for window in NSApp.windows {
            if window.class != NSPanel { swap_class(window); }
            if window.level != screen_saver { window.set_level(screen_saver); }
            if !(window.collectionBehavior & (CanJoinAllSpaces|FullScreenAux)) {
                window.set_collection_behavior(window.collectionBehavior | both);
            }
            if window.sharingType != None {
                window.set_sharing_type(ReadOnly);
                window.set_sharing_type(None);
            }
            // ... etc, each guarded ...
        }
    });
}
```

Because every write is guarded, in steady state the loop does **zero** AppKit writes per tick. This matters: an earlier version unconditionally wrote every setter each cycle, and even "no-op" writes forced enough compositor work that the WebView's CSS `backdrop-filter` visibly pulsed between transparency levels. Guarding the writes eliminated the flicker entirely.

## Why not just use Tauri's `set_always_on_top` or `visibleOnAllWorkspaces`?

We use both as a baseline (`window.set_visible_on_all_workspaces(true)` is called in `lib.rs::run` before our custom stealth runs), but neither is enough on its own:

- `set_always_on_top(true)` uses `NSFloatingWindowLevel` (3), which is below menu bars and below Dock. It does **not** survive full-screen Space transitions. In our testing it had the exact "stays on desktop Space, disappears over full-screen apps" bug.
- `visibleOnAllWorkspaces: true` sets `CanJoinAllSpaces` but not `FullScreenAuxiliary`, and it doesn't change the window class. So the window follows you across Spaces but still can't overlay full-screen apps.

Our five-layer stack goes further: class swap to `NSPanel`, screen-saver level, both collection-behavior bits, non-activating style mask, and a watchdog that keeps everything pinned through system resets.

## Windows notes

Windows is simpler. The WebView2 window is managed by `tao` the same way as macOS, but the stealth layers reduce to:

- `SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)` for capture exclusion.
- `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` extended styles for no-focus-steal and no-taskbar.
- `SetWindowPos(..., HWND_TOPMOST, ...)` for always-on-top (Windows has a simpler compositor with no per-Space behavior to fight).

No class swaps needed. No watchdog needed. Windows is a one-shot affair.

## File map

| File | Responsibility |
|------|----------------|
| `src-tauri/src/stealth/mod.rs` | Orchestrates `apply_all_stealth`: calls into each layer. |
| `src-tauri/src/stealth/macos.rs` | `convert_windows_to_panels`, `apply_hover_level_to_all_windows`, `apply_sharing_none_to_all_windows`, `start_hover_reassertion_loop`, `log_window_state_for_debugging`. |
| `src-tauri/src/stealth/windows.rs` | `SetWindowDisplayAffinity` + `WS_EX_NOACTIVATE`. |
| `src-tauri/src/stealth/screen_share.rs` | Cross-platform entry for capture exclusion. |
| `src-tauri/src/stealth/focus.rs` | Cross-platform entry for non-activating style. |
| `src-tauri/src/stealth/process.rs` | Cross-platform process disguise (Dock / taskbar). |
| `src-tauri/Info.plist` | `LSUIElement`, microphone + screen-recording usage descriptions. |
| `src-tauri/tauri.conf.json` | `macOSPrivateApi: true`, transparent window, `visibleOnAllWorkspaces`, no decorations. |

## Testing checklist

When you change anything in the stealth stack, verify all of these:

1. **Zoom capture test.** Share your screen on a Zoom call (to yourself, with video on). The panel must not appear in the preview.
2. **QuickTime screen recording.** Record the screen and play it back. Panel must not appear.
3. **Full-screen hover.** Maximize Chrome (green traffic-light → Enter Full Screen). The panel must remain visible over Chrome.
4. **Full-screen Zoom hover.** Start a Zoom call, enter full-screen. The panel must remain visible over Zoom.
5. **Space switching.** Swipe between Spaces (three-finger swipe). The panel must follow to every Space.
6. **No focus steal.** Open any site in Chrome, open the console, run `window.addEventListener('blur', () => console.log('blur'))`. Click inside the panel. No `blur` log should appear.
7. **No Dock entry.** Check the Dock — our app should not appear.
8. **No Cmd+Tab entry.** Cmd+Tab — our app should not appear.
9. **No transparency flicker.** Watch the panel for 30 seconds with no interaction. The `backdrop-filter` blur must not pulse.
10. **Diagnostic log.** Check the dev console every ~5s for the `=== Hover diagnostic ===` block. Every window should be `class=NSPanel level=1000 sharingType=None collectionBehavior=0x101 styleMask` has bit `0x80` set.
