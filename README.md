# Invisible Agent

A native desktop interview assistant that floats over any window you're in, including full-screen Zoom, Meet, Teams, Chrome, and IDEs, while remaining **completely invisible** to anything capturing your screen. Built with Tauri 2 (Rust + WebView).

## Architecture at a Glance

```mermaid
flowchart TB
    User(["User"])

    subgraph OS["macOS / Windows"]
        direction LR
        Shortcuts["Global shortcuts<br/>⌘B · ⌘` · ⌘⇧↑↓←→"]
        Mic["Microphone"]
        Display["Display / Screenshot"]
    end

    subgraph Frontend["Frontend  (React + TypeScript + Vite)"]
        direction TB
        App["App.tsx<br/>shell · tabs"]
        subgraph Views["Views"]
            direction LR
            ChatBox["ChatBox"]
            Transcript["TranscriptView"]
            SettingsUI["Settings"]
        end
        subgraph Hooks["Hooks"]
            direction LR
            useAI["useAI · useAIEventBridge"]
            useAudio["useAudio"]
            useShortcuts["useShortcuts"]
        end
        Store["Zustand store<br/>messages · transcript · config"]
        App --> Views
        Views --> Hooks
        Hooks --> Store
    end

    subgraph Tauri["Tauri IPC  (invoke · emit · listen)"]
        direction LR
        Commands["commands.rs<br/>send_message · take_screenshot<br/>start_recording · move_window"]
        Events["Events<br/>ai-token · ai-done · ai-error<br/>trigger-screenshot · transcript-entry"]
    end

    subgraph Backend["Rust backend  (src-tauri/)"]
        direction TB
        Lib["lib.rs<br/>entry · global shortcuts · setup"]
        subgraph Stealth["stealth/"]
            direction LR
            MacOS["macos.rs<br/>NSPanel swap · watchdog loop<br/>sharingType=None"]
            Win["windows.rs<br/>WDA_EXCLUDEFROMCAPTURE<br/>WS_EX_NOACTIVATE"]
            Process["process.rs<br/>LSUIElement · accessory policy"]
        end
        subgraph Audio["audio/"]
            direction LR
            Capture["capture.rs (cpal)"]
            Transcribe["transcribe.rs<br/>Whisper chunks"]
        end
        subgraph AI["ai/"]
            direction LR
            Context["context.rs<br/>system prompt · history · screenshot"]
            Client["client.rs<br/>streaming OpenAI client"]
        end
        Screen["screen/ (ScreenCaptureKit)"]
        Lib --> Stealth
        Lib --> Commands
    end

    subgraph External["External APIs"]
        direction LR
        Whisper["OpenAI Whisper"]
        GPT["GPT-4o Vision"]
    end

    User -- "press shortcut" --> Shortcuts
    User -- "type / click" --> Frontend
    Shortcuts --> Lib
    Mic --> Capture
    Display --> Screen

    useAI -- "invoke send_message" --> Commands
    useAudio -- "invoke start/stop" --> Commands
    useShortcuts -- "listen trigger-screenshot" --> Events
    useAI -- "listen ai-token / ai-done" --> Events

    Commands --> AI
    Commands --> Screen
    Commands --> Audio
    Capture --> Transcribe
    Transcribe -- "chunked audio" --> Whisper
    Transcribe -- "transcript-entry" --> Events
    Context --> Client
    Client -- "streaming chat" --> GPT
    Client -- "ai-token / ai-done" --> Events

    Stealth -. "hides panel from" .-> Display

    classDef userNode fill:#111,stroke:#4c8dff,stroke-width:2px,color:#fff
    classDef osNode fill:#1a1a1a,stroke:#888,color:#ddd
    classDef feNode fill:#0d2540,stroke:#4c8dff,color:#cfe1ff
    classDef ipcNode fill:#2a1f3d,stroke:#a78bfa,color:#ead6ff
    classDef beNode fill:#0f2a1d,stroke:#4ade80,color:#c7f2d6
    classDef extNode fill:#3a1f1f,stroke:#f87171,color:#ffd6d6
    classDef stealthNode fill:#3a2a0a,stroke:#facc15,color:#fff4c2

    class User userNode
    class Shortcuts,Mic,Display osNode
    class App,ChatBox,Transcript,SettingsUI,useAI,useAudio,useShortcuts,Store feNode
    class Commands,Events ipcNode
    class Lib,Capture,Transcribe,Context,Client,Screen beNode
    class MacOS,Win,Process stealthNode
    class Whisper,GPT extNode

    click App href "./src/App.tsx" "src/App.tsx"
    click ChatBox href "./src/components/ChatBox.tsx" "src/components/ChatBox.tsx"
    click Transcript href "./src/components/TranscriptView.tsx" "src/components/TranscriptView.tsx"
    click SettingsUI href "./src/components/Settings.tsx" "src/components/Settings.tsx"
    click useAI href "./src/hooks/useAI.ts" "src/hooks/useAI.ts"
    click useAudio href "./src/hooks/useAudio.ts" "src/hooks/useAudio.ts"
    click useShortcuts href "./src/hooks/useShortcuts.ts" "src/hooks/useShortcuts.ts"
    click Store href "./src/stores/appStore.ts" "src/stores/appStore.ts"
    click Commands href "./src-tauri/src/commands.rs" "src-tauri/src/commands.rs"
    click Lib href "./src-tauri/src/lib.rs" "src-tauri/src/lib.rs"
    click MacOS href "./src-tauri/src/stealth/macos.rs" "src-tauri/src/stealth/macos.rs"
    click Win href "./src-tauri/src/stealth/windows.rs" "src-tauri/src/stealth/windows.rs"
    click Process href "./src-tauri/src/stealth/process.rs" "src-tauri/src/stealth/process.rs"
    click Capture href "./src-tauri/src/audio/capture.rs" "src-tauri/src/audio/capture.rs"
    click Transcribe href "./src-tauri/src/audio/transcribe.rs" "src-tauri/src/audio/transcribe.rs"
    click Context href "./src-tauri/src/ai/context.rs" "src-tauri/src/ai/context.rs"
    click Client href "./src-tauri/src/ai/client.rs" "src-tauri/src/ai/client.rs"
    click Screen href "./src-tauri/src/screen" "src-tauri/src/screen/"
```

Legend: <span title="blue">frontend</span> · <span title="purple">Tauri IPC</span> · <span title="green">Rust backend</span> · <span title="yellow">stealth layer</span> · <span title="red">external APIs</span>. The stealth layer is a cross-cutting concern — it intercepts the path from the panel to the display so Zoom / ScreenCaptureKit / `CGWindowList` never see us.

## Features

### Invisibility
- **Screen-share invisible**: `NSWindowSharingType.none` (macOS) / `WDA_EXCLUDEFROMCAPTURE` (Windows) excludes the window from every capture API — Zoom, Meet, Teams, QuickTime, `screencapture`, ScreenCaptureKit, etc.
- **Hover over full-screen apps**: The window is converted at runtime from `NSWindow` → `NSPanel` with the `nonactivatingPanel` style mask and promoted to `kCGScreenSaverWindowLevelKey` (1000), so it sits on top of maximized / full-screen windows without being dragged into their Space.
- **Cross-Space**: `CanJoinAllSpaces | FullScreenAuxiliary` collection behavior means the panel follows you across Mission Control Spaces and appears over full-screen apps instead of being locked to the desktop.
- **No focus stealing**: `nonactivatingPanel` + `becomesKeyOnlyIfNeeded` + `hidesOnDeactivate = false` — clicking the panel never activates the app or triggers `window.blur` on the site you're interviewing in (no Cluely-style "tab switched" signal).
- **Hidden from Dock & Cmd+Tab**: `LSUIElement = true` and `NSApplicationActivationPolicy.accessory`.
- **Idempotent watchdog loop**: A 250 ms main-thread poll re-asserts stealth properties after Space changes / full-screen transitions, but only writes when a property has actually drifted, so the WebView's `backdrop-filter` never flickers.

See [`docs/STEALTH.md`](./docs/STEALTH.md) for the full breakdown.

### Assistant
- **Audio transcription**: Microphone capture via `cpal`, transcribed in rolling chunks by OpenAI Whisper.
- **Screenshot + Vision**: `Cmd+\`` screenshots the screen and sends it to GPT-4o Vision along with the full transcript, chat history, resume/background, and custom instructions.
- **Structured answers**: AI responses are formatted as *Approach → Steps → Solution → Talking Points → Follow-ups* so you can speak fluently from the panel.
- **Streaming UI**: Responses stream token-by-token into a small draggable chat overlay.
- **Persistent config**: API key, model, base URL, and resume are saved to `localStorage`.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `⌘ B` | Toggle panel visibility |
| `⌘ \`` | Screenshot + ask AI |
| `⌘ ⇧ ↑ ↓ ← →` | Move panel 100 px in that direction |

## Prerequisites

- [Rust](https://rustup.rs/) 1.77+
- [Node.js](https://nodejs.org/) 18+
- macOS 12.3+ (ScreenCaptureKit) or Windows 10+ (`WDA_EXCLUDEFROMCAPTURE`)
- OpenAI API key (for Whisper + GPT-4o)

## Setup

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Run in production
npm run tauri:build
```

On first launch macOS will prompt for **Microphone** and **Screen Recording** permissions — both are required for audio capture and screenshots respectively.

## Configuration

1. Launch the app.
2. Open the Settings tab (gear icon).
3. Enter your OpenAI API key.
4. (Optional) Paste your resume / background text for context-aware answers.
5. Select a model (`gpt-4o`, `gpt-4o-mini`, etc.) and, if using a proxy, a base URL.

## Architecture

```
src-tauri/                  Rust backend (Tauri 2)
  src/
    stealth/                Platform invisibility layers
      macos.rs              NSPanel conversion, level/collection-behavior,
                            sharingType=None, watchdog reassertion loop
      windows.rs            SetWindowDisplayAffinity, WS_EX_NOACTIVATE
      screen_share.rs       Capture-exclusion entry point
      focus.rs              Non-activating panel style
      process.rs            Hide from Dock / Cmd+Tab
    audio/                  cpal mic capture + Whisper streaming
    screen/                 screencapture / ScreenCaptureKit
    ai/                     Context builder + OpenAI streaming client
    commands.rs             Tauri IPC commands
    lib.rs                  Entry point, global shortcuts

src/                        React + TypeScript frontend
  components/               ChatBox, TranscriptView, Settings
  hooks/                    useAI, useAudio, useShortcuts
  stores/                   Zustand state management
  index.css                 Glass aesthetic (backdrop-filter)

docs/
  STEALTH.md                How invisibility works
```

## How It Works (TL;DR)

Three OS-level tricks running simultaneously:

1. **Invisible to capture** — Every `NSWindow` in the process has its `sharingType` pinned to `None`, so ScreenCaptureKit / CGWindowList / the legacy capture pipeline skip it entirely.
2. **Visible to you, anywhere** — The window is dynamically reclassified as an `NSPanel` with `nonactivatingPanel` + `floatingPanel`, promoted to the screen-saver window level, and granted `CanJoinAllSpaces | FullScreenAuxiliary` so it floats above *any* window — full-screen Zoom included.
3. **Undetectable by heuristics** — No focus stealing, no Dock icon, no Cmd+Tab entry, no taskbar. The Zoom/Meet tab never fires `window.blur`, `visibilitychange`, or focus events when you click us.

Read the full writeup in [`docs/STEALTH.md`](./docs/STEALTH.md).

## License

MIT
