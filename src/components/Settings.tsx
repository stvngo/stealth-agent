import { useAppStore } from "../stores/appStore";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

export function Settings() {
  const { config, setConfig } = useAppStore();
  const [saved, setSaved] = useState(false);

  const handleSave = async () => {
    try {
      await invoke("set_config", {
        apiKey: config.apiKey || null,
        model: config.model || null,
        baseUrl: config.baseUrl || null,
        resume: config.resumeText || null,
      });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("Failed to save config:", e);
    }
  };

  return (
    <div className="flex flex-col h-full overflow-y-auto px-3 py-3 space-y-4">
      <div>
        <label className="block text-[11px] font-medium mb-1" style={{ color: "var(--text-secondary)" }}>
          API Key
        </label>
        <input
          type="password"
          value={config.apiKey}
          onChange={(e) => setConfig({ apiKey: e.target.value })}
          placeholder="sk-..."
          className="w-full rounded px-2 py-1.5 text-[12px] outline-none"
          style={{
            background: "var(--bg-input)",
            color: "var(--text-primary)",
            border: "1px solid var(--border)",
          }}
        />
      </div>

      <div>
        <label className="block text-[11px] font-medium mb-1" style={{ color: "var(--text-secondary)" }}>
          Model
        </label>
        <select
          value={config.model}
          onChange={(e) => setConfig({ model: e.target.value })}
          className="w-full rounded px-2 py-1.5 text-[12px] outline-none"
          style={{
            background: "var(--bg-input)",
            color: "var(--text-primary)",
            border: "1px solid var(--border)",
          }}
        >
          <option value="gpt-4o">GPT-4o</option>
          <option value="gpt-4o-mini">GPT-4o Mini</option>
          <option value="gpt-4.1">GPT-4.1</option>
          <option value="gpt-4.1-mini">GPT-4.1 Mini</option>
          <option value="claude-sonnet-4-20250514">Claude Sonnet 4</option>
          <option value="claude-3-5-sonnet-20241022">Claude 3.5 Sonnet</option>
        </select>
      </div>

      <div>
        <label className="block text-[11px] font-medium mb-1" style={{ color: "var(--text-secondary)" }}>
          API Base URL
        </label>
        <input
          value={config.baseUrl}
          onChange={(e) => setConfig({ baseUrl: e.target.value })}
          placeholder="https://api.openai.com/v1"
          className="w-full rounded px-2 py-1.5 text-[12px] outline-none"
          style={{
            background: "var(--bg-input)",
            color: "var(--text-primary)",
            border: "1px solid var(--border)",
          }}
        />
        <p className="text-[10px] mt-0.5 opacity-50">
          Change for Anthropic, Azure, or self-hosted models
        </p>
      </div>

      <div>
        <label className="block text-[11px] font-medium mb-1" style={{ color: "var(--text-secondary)" }}>
          Resume / Background
        </label>
        <textarea
          value={config.resumeText}
          onChange={(e) => setConfig({ resumeText: e.target.value })}
          placeholder="Paste your resume or background info here..."
          rows={6}
          className="w-full rounded px-2 py-1.5 text-[12px] outline-none resize-none"
          style={{
            background: "var(--bg-input)",
            color: "var(--text-primary)",
            border: "1px solid var(--border)",
          }}
        />
      </div>

      <button
        onClick={handleSave}
        className="w-full py-2 rounded text-[12px] font-medium transition-colors"
        style={{
          background: saved ? "var(--success)" : "var(--accent)",
          color: "white",
        }}
      >
        {saved ? "Saved ✓" : "Save Settings"}
      </button>

      <div className="pt-2 border-t" style={{ borderColor: "var(--border)" }}>
        <p className="text-[10px] font-medium mb-1" style={{ color: "var(--text-secondary)" }}>
          Keyboard Shortcuts
        </p>
        <div className="space-y-1 text-[11px]" style={{ color: "var(--text-secondary)" }}>
          <div className="flex justify-between">
            <span>Toggle visibility</span>
            <kbd className="px-1 rounded" style={{ background: "var(--bg-input)" }}>⌘ B</kbd>
          </div>
          <div className="flex justify-between">
            <span>Screenshot + AI</span>
            <kbd className="px-1 rounded" style={{ background: "var(--bg-input)" }}>⌘ `</kbd>
          </div>
          <div className="flex justify-between">
            <span>Move window</span>
            <kbd className="px-1 rounded" style={{ background: "var(--bg-input)" }}>⌘ ⇧ ↑↓←→</kbd>
          </div>
        </div>
      </div>
    </div>
  );
}
