import { MessageSquare, Mic, Settings as SettingsIcon, Trash2, Ghost } from "lucide-react";
import { ChatBox } from "./components/ChatBox";
import { TranscriptView } from "./components/TranscriptView";
import { Settings } from "./components/Settings";
import { useShortcuts } from "./hooks/useShortcuts";
import { useAppStore } from "./stores/appStore";

type Tab = "chat" | "transcript" | "settings";

const TABS: { id: Tab; label: string; Icon: typeof MessageSquare }[] = [
  { id: "chat", label: "Chat", Icon: MessageSquare },
  { id: "transcript", label: "Transcript", Icon: Mic },
  { id: "settings", label: "Settings", Icon: SettingsIcon },
];

function App() {
  useShortcuts();
  const { activeTab, setActiveTab, clearMessages } = useAppStore();

  return (
    <div className="app-shell h-screen w-screen flex flex-col rounded-[14px] overflow-hidden">
      {/* Drag region / Title bar */}
      <div
        data-tauri-drag-region
        className="flex items-center justify-between px-3 py-2 shrink-0 select-none"
        style={{ borderBottom: "1px solid var(--border)" }}
      >
        <div className="flex items-center gap-2" data-tauri-drag-region>
          <Ghost size={13} style={{ color: "var(--accent-hover)" }} />
          <span
            className="text-[11px] font-medium tracking-tight"
            style={{ color: "var(--text-secondary)" }}
            data-tauri-drag-region
          >
            invisible
          </span>
        </div>

        <div className="flex items-center gap-1">
          {TABS.map(({ id, label, Icon }) => (
            <button
              key={id}
              onClick={() => setActiveTab(id)}
              className="btn-ghost"
              data-active={activeTab === id}
              title={label}
              aria-label={label}
            >
              <Icon size={13} strokeWidth={2} />
            </button>
          ))}
          <div
            className="mx-1 h-4 w-px"
            style={{ background: "var(--border)" }}
            aria-hidden
          />
          <button
            onClick={clearMessages}
            className="btn-ghost"
            title="Clear chat"
            aria-label="Clear chat"
          >
            <Trash2 size={13} strokeWidth={2} />
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 min-h-0">
        {activeTab === "chat" && <ChatBox />}
        {activeTab === "transcript" && <TranscriptView />}
        {activeTab === "settings" && <Settings />}
      </div>
    </div>
  );
}

export default App;
