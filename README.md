# Invisible Agent

An invisible interview assistant that provides real-time AI help during technical interviews. The app is completely invisible when sharing your screen on Zoom, Meet, Teams, etc.

## Features

- **Screen-share invisible**: Uses OS-level APIs (`NSWindowSharingType.none` on macOS, `WDA_EXCLUDEFROMCAPTURE` on Windows) to hide from all screen captures
- **No focus stealing**: Non-activating floating panel that never triggers `window.blur` events in the browser
- **Hidden from Dock/Task Manager**: Runs as an accessory app, invisible in Cmd+Tab and Activity Monitor
- **Audio transcription**: Captures microphone and system audio, transcribes via OpenAI Whisper API
- **Screenshot + AI**: Takes screenshots and sends them to GPT-4o Vision for multimodal analysis
- **Structured reasoning**: AI provides approach, code with comments, talking points, and follow-up guidance
- **Floating chat overlay**: Small, draggable window with keyboard shortcuts

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `⌘ B` | Toggle visibility (show/hide) |
| `⌘ \`` | Screenshot + AI help |
| `⌘ ⇧ ↑↓←→` | Move window (position over code editor) |

## Prerequisites

- [Rust](https://rustup.rs/) (1.77+)
- [Node.js](https://nodejs.org/) (18+)
- macOS 12.3+ or Windows 10+

## Setup

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Configuration

1. Launch the app
2. Go to Settings (gear icon)
3. Enter your OpenAI API key
4. Optionally paste your resume/background for context
5. Select your preferred AI model

## Architecture

```
src-tauri/          # Rust backend (Tauri 2.0)
  src/
    stealth/        # Invisibility layers (screen share, focus, process)
    audio/          # Microphone + system audio capture, Whisper transcription
    screen/         # Screenshot capture
    ai/             # LLM context builder + streaming client
    commands.rs     # Tauri IPC commands
    lib.rs          # App entry point, global shortcuts

src/                # React + TypeScript frontend
  components/       # ChatBox, TranscriptView, Settings
  hooks/            # useAI, useAudio, useShortcuts
  stores/           # Zustand state management
```
