import { ChatBox } from "./components/ChatBox";
import { TranscriptView } from "./components/TranscriptView";
import { Settings } from "./components/Settings";
import { useShortcuts } from "./hooks/useShortcuts";
import { useAppStore } from "./stores/appStore";

function App() {
  useShortcuts();
  const { activeTab, setActiveTab, clearMessages } = useAppStore();

  return (
    <div
      className="h-screen w-screen flex flex-col rounded-xl overflow-hidden"
      style={{ background: "var(--bg-primary)", border: "1px solid var(--border)" }}
    >
      {/* Drag region / Title bar */}
      <div
        data-tauri-drag-region
        className="flex items-center justify-between px-3 py-1.5 shrink-0 select-none"
        style={{ borderBottom: "1px solid var(--border)" }}
      >
        <div className="flex items-center gap-1.5" data-tauri-drag-region>
          <div className="w-2 h-2 rounded-full" style={{ background: "var(--success)" }} />
          <span
            className="text-[11px] font-medium"
            style={{ color: "var(--text-secondary)" }}
            data-tauri-drag-region
          >
            invisible
          </span>
        </div>

        <div className="flex gap-0.5">
          {(["chat", "transcript", "settings"] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className="px-2 py-0.5 rounded text-[11px] transition-colors"
              style={{
                background: activeTab === tab ? "var(--accent)" : "transparent",
                color: activeTab === tab ? "white" : "var(--text-secondary)",
              }}
            >
              {tab === "chat" ? "💬" : tab === "transcript" ? "🎤" : "⚙️"}
            </button>
          ))}
          <button
            onClick={clearMessages}
            className="px-2 py-0.5 rounded text-[11px] ml-1 transition-colors hover:opacity-80"
            style={{ color: "var(--text-secondary)" }}
            title="Clear chat"
          >
            🗑
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
